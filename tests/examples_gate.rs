// SPDX-License-Identifier: AGPL-3.0-or-later
//! The filament-examples gate: builds `fsl` from this workspace, at the
//! language revision Cargo.lock resolves, and runs the examples runner of a
//! filament-examples checkout with it over every running and every staged case.
//!
//! `FSL_EXAMPLES_DIR` names the checkout; `make examples-check` fetches it and
//! sets the variable. A missing checkout fails the gate. This target is not
//! part of a plain `cargo test` (`test = false` in Cargo.toml) because it needs
//! the fetched suite.
//!
//! With `FSL_LANGUAGE_DIR` naming a filament-spec-language checkout, `fsl` is
//! built with every language package patched to that checkout's crates; the
//! language repository's `make examples-check` runs the gate this way.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use ix_trace_rs::trace;
use serde_json::Value;

/// The variable naming the filament-examples checkout.
const CHECKOUT_VAR: &str = "FSL_EXAMPLES_DIR";

fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

/// The checkout named by `value`, or why there is none. A path without an
/// `examples/` folder or the runner crate is not a checkout.
fn examples_checkout(value: Option<OsString>) -> Result<PathBuf, String> {
    let dir = value
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            format!("{CHECKOUT_VAR} is not set; run `make examples-check` to fetch the suite")
        })?;
    if !dir.join("examples").is_dir() || !runner_manifest(&dir).is_file() {
        return Err(format!(
            "{CHECKOUT_VAR}={} is not a filament-examples checkout (no examples/ or \
             tools/fsl-examples); run `make examples-fetch`",
            dir.display()
        ));
    }
    Ok(dir)
}

fn runner_manifest(checkout: &Path) -> PathBuf {
    checkout.join("tools/fsl-examples/Cargo.toml")
}

fn checkout() -> &'static Path {
    static CHECKOUT: OnceLock<PathBuf> = OnceLock::new();
    CHECKOUT.get_or_init(|| match examples_checkout(std::env::var_os(CHECKOUT_VAR)) {
        Ok(dir) => dir,
        Err(reason) => panic!("{reason}"),
    })
}

/// What one `cargo build` produced: the executable of the requested binary and
/// the package id of every artifact the build reported.
struct Built {
    executable: PathBuf,
    package_ids: Vec<String>,
}

/// Builds the binary target `bin` with `cargo build <args>` and returns the
/// executable cargo reports for it with the package ids of every artifact.
fn build_bin(args: &[String], bin: &str) -> Built {
    let output = Command::new(cargo())
        .args(["build", "--message-format=json-render-diagnostics"])
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("cannot start cargo: {e}"));
    assert!(
        output.status.success(),
        "cargo build {args:?} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let artifacts: Vec<Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|m| m["reason"] == "compiler-artifact")
        .collect();
    let executable = artifacts
        .iter()
        .filter(|m| m["target"]["name"] == bin)
        .find_map(|m| m["executable"].as_str().map(PathBuf::from))
        .unwrap_or_else(|| panic!("cargo build {args:?} reported no executable for {bin}"));
    let package_ids = artifacts
        .iter()
        .filter_map(|m| m["package_id"].as_str().map(str::to_owned))
        .collect();
    Built {
        executable,
        package_ids,
    }
}

fn owned(args: &[&str]) -> Vec<String> {
    args.iter().map(|&a| a.to_owned()).collect()
}

/// The variable naming a filament-spec-language checkout to build `fsl` from.
const LANGUAGE_VAR: &str = "FSL_LANGUAGE_DIR";
/// The git source the driver's FSL language packages resolve from.
const LANGUAGE_SOURCE: &str = "https://github.com/agent-ix/filament-spec-language";

/// The language checkout named by `FSL_LANGUAGE_DIR`, if any, as an absolute
/// path: cargo names path packages by it.
fn language_checkout() -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os(LANGUAGE_VAR).filter(|v| !v.is_empty())?);
    Some(fs::canonicalize(&dir).unwrap_or_else(|e| panic!("{LANGUAGE_VAR}={}: {e}", dir.display())))
}

/// Runs `cargo metadata --format-version 1 <args>` and returns its document.
fn metadata(args: &[&str]) -> Value {
    let output = Command::new(cargo())
        .args(["metadata", "--format-version", "1"])
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap_or_else(|e| panic!("cannot start cargo metadata: {e}"));
    assert!(
        output.status.success(),
        "cargo metadata {args:?} failed:\n{}",
        text(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| panic!("metadata: {e}"))
}

fn packages(metadata: &Value) -> impl Iterator<Item = &Value> {
    metadata["packages"].as_array().into_iter().flatten()
}

/// Whether a cargo package id names a package of the language's git source.
fn is_language_git_package(id: &str) -> bool {
    id.starts_with(&format!("git+{LANGUAGE_SOURCE}"))
}

/// The `--config` arguments that patch every language package the driver's
/// graph resolves from git to the same-named crate of the checkout `dir`.
/// Each such package must exist in the checkout's workspace.
fn language_patches(dir: &Path) -> Vec<String> {
    let manifest = dir.join("Cargo.toml");
    let language = metadata(&["--no-deps", "--manifest-path", &manifest.to_string_lossy()]);
    let crates: BTreeMap<&str, &Path> = packages(&language)
        .filter_map(|p| {
            let name = p["name"].as_str()?;
            let manifest = Path::new(p["manifest_path"].as_str()?);
            Some((name, manifest.parent()?))
        })
        .collect();
    let driver = metadata(&["--locked"]);
    let names: BTreeSet<&str> = packages(&driver)
        .filter(|p| p["id"].as_str().is_some_and(is_language_git_package))
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert!(
        !names.is_empty(),
        "the driver graph resolves no package from {LANGUAGE_SOURCE}"
    );
    names
        .into_iter()
        .flat_map(|name| {
            let path = crates.get(name).unwrap_or_else(|| {
                panic!(
                    "{LANGUAGE_VAR}={} has no package {name}, which the driver uses",
                    dir.display()
                )
            });
            let path = serde_json::to_string(&path.to_string_lossy())
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            [
                "--config".to_owned(),
                format!("patch.\"{LANGUAGE_SOURCE}\".{name}.path={path}"),
            ]
        })
        .collect()
}

/// Makes `to` a copy of the workspace tree `from`, without `.git` and
/// `target`. A file whose bytes already match is left alone, so its
/// modification time, and cargo's fingerprint of it, survive between runs.
fn mirror(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("create {}: {e}", to.display()));
    let entries = fs::read_dir(from).unwrap_or_else(|e| panic!("list {}: {e}", from.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("list {}: {e}", from.display()));
        let name = entry.file_name();
        if name == ".git" || name == "target" {
            continue;
        }
        let (source, copy) = (entry.path(), to.join(&name));
        if source.is_dir() {
            mirror(&source, &copy);
            continue;
        }
        let bytes = fs::read(&source).unwrap_or_else(|e| panic!("{}: {e}", source.display()));
        if fs::read(&copy).ok().as_deref() != Some(bytes.as_slice()) {
            fs::write(&copy, &bytes).unwrap_or_else(|e| panic!("{}: {e}", copy.display()));
        }
    }
}

/// The `fsl` the gate runs and where its language packages came from.
struct Fsl {
    executable: PathBuf,
    /// The package ids of every artifact of the build.
    package_ids: Vec<String>,
    /// This checkout's `Cargo.lock` as it was before the build.
    lock_before: Vec<u8>,
}

fn checkout_lock() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock");
    fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The `fsl` binary of this workspace's `fsl-cli` package. With
/// `FSL_LANGUAGE_DIR` set, every language package is patched to that
/// checkout's crates. The patched build runs in a copy of this workspace under
/// `CARGO_TARGET_TMPDIR`, whose `Cargo.lock` the patch rewrites, with this
/// workspace's target directory; the checkout's own `Cargo.lock` is never
/// written.
fn fsl_build() -> &'static Fsl {
    static FSL: OnceLock<Fsl> = OnceLock::new();
    FSL.get_or_init(|| {
        let lock_before = checkout_lock();
        let fsl_cli = owned(&["-p", "fsl-cli", "--bin", "fsl"]);
        let built = match language_checkout() {
            None => {
                let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
                let args = owned(&["--locked", "--manifest-path", manifest]);
                build_bin(&[args, fsl_cli].concat(), "fsl")
            }
            Some(dir) => {
                let patches = language_patches(&dir);
                let target_dir = metadata(&["--locked", "--no-deps"])["target_directory"]
                    .as_str()
                    .unwrap_or_else(|| panic!("metadata has no target_directory"))
                    .to_owned();
                let copy = Path::new(env!("CARGO_TARGET_TMPDIR")).join("language-build");
                mirror(Path::new(env!("CARGO_MANIFEST_DIR")), &copy);
                let manifest = copy.join("Cargo.toml").to_string_lossy().into_owned();
                let args = vec![
                    "--manifest-path".to_owned(),
                    manifest,
                    "--target-dir".to_owned(),
                    target_dir,
                ];
                build_bin(&[patches, args, fsl_cli].concat(), "fsl")
            }
        };
        Fsl {
            executable: built.executable,
            package_ids: built.package_ids,
            lock_before,
        }
    })
}

fn fsl() -> &'static Path {
    &fsl_build().executable
}

/// The runner binary built from the checkout's `tools/fsl-examples` crate.
fn runner() -> &'static Path {
    static RUNNER: OnceLock<PathBuf> = OnceLock::new();
    RUNNER.get_or_init(|| {
        let manifest = runner_manifest(checkout());
        let manifest = manifest.to_string_lossy();
        build_bin(
            &owned(&[
                "--locked",
                "--manifest-path",
                &manifest,
                "--bin",
                "fsl-examples",
            ]),
            "fsl-examples",
        )
        .executable
    })
}

/// Runs the runner for the repository at `root` with the built `fsl`.
fn run_runner(root: &Path, args: &[&str]) -> Output {
    Command::new(runner())
        .arg("--root")
        .arg(root)
        .args(args)
        .env("FSL", fsl())
        .output()
        .unwrap_or_else(|e| panic!("cannot start {}: {e}", runner().display()))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Asserts the run passed and its summary line reads `<name>: N passed, 0
/// failed` with N above zero; returns N.
fn assert_all_passed(output: &Output, name: &str) -> usize {
    let stdout = text(&output.stdout);
    let stderr = text(&output.stderr);
    assert!(
        output.status.success(),
        "{name} failed with {}:\n{stderr}\n{stdout}",
        output.status
    );
    let prefix = format!("{name}: ");
    let summary = stdout
        .lines()
        .rev()
        .find_map(|l| l.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("no `{name}:` summary line in:\n{stdout}"));
    let (passed, failed) = summary
        .split_once(" passed, ")
        .unwrap_or_else(|| panic!("unexpected summary `{summary}`"));
    assert_eq!(failed, "0 failed", "{name} summary: {summary}\n{stderr}");
    let passed: usize = passed
        .parse()
        .unwrap_or_else(|e| panic!("summary count `{passed}`: {e}"));
    assert!(passed > 0, "{name} ran no cases: {summary}");
    passed
}

/// One `[[case]]` table of a manifest, as line positions in its text.
#[derive(Debug)]
struct CaseLines {
    id: String,
    status: String,
    /// The `expect_exit` line number and value.
    expect_exit: Option<(usize, String)>,
    /// The `ready_exit` line number and value.
    ready_exit: Option<(usize, String)>,
}

/// The keys the negative tests select by or edit.
const READ_KEYS: [&str; 4] = ["id", "status", "expect_exit", "ready_exit"];

/// Reads the `[[case]]` tables of an `example.toml`. The structure is the one
/// the suite uses: top-level `key = value` lines, then `[[case]]` tables of
/// `key = value` lines, with whole-line comments only. Anything else is an
/// error, so an edit never lands on a line the gate misread.
fn case_lines(manifest: &str) -> Result<Vec<CaseLines>, String> {
    let mut cases: Vec<CaseLines> = Vec::new();
    for (n, raw) in manifest.lines().enumerate() {
        let line = raw.trim();
        let at = n + 1;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            if line != "[[case]]" {
                return Err(format!(
                    "line {at}: unexpected table `{line}`; only [[case]] tables are expected"
                ));
            }
            cases.push(CaseLines {
                id: String::new(),
                status: String::new(),
                expect_exit: None,
                ready_exit: None,
            });
            continue;
        }
        let Some((key, value)) = line.split_once(" = ") else {
            return Err(format!("line {at}: `{line}` is not a `key = value` line"));
        };
        let Some(case) = cases.last_mut() else {
            continue;
        };
        if !READ_KEYS.contains(&key) {
            continue;
        }
        if value.contains('#') {
            return Err(format!("line {at}: inline comment on `{key}`"));
        }
        let value = value.trim_matches('"').to_owned();
        let duplicate = match key {
            "id" => !std::mem::replace(&mut case.id, value).is_empty(),
            "status" => !std::mem::replace(&mut case.status, value).is_empty(),
            "expect_exit" => case.expect_exit.replace((n, value)).is_some(),
            _ => case.ready_exit.replace((n, value)).is_some(),
        };
        if duplicate {
            return Err(format!("line {at}: `{key}` is repeated in one case"));
        }
    }
    match cases
        .iter()
        .position(|c| c.id.is_empty() || c.status.is_empty())
    {
        Some(i) => Err(format!("case table {} has no id or no status", i + 1)),
        None => Ok(cases),
    }
}

/// The one case of `manifest` with the id `id`.
fn case_by_id(manifest: &str, id: &str) -> Result<CaseLines, String> {
    let mut hits = case_lines(manifest)?.into_iter().filter(|c| c.id == id);
    match (hits.next(), hits.next()) {
        (Some(case), None) => Ok(case),
        (None, _) => Err(format!("no case {id}")),
        (Some(_), Some(_)) => Err(format!("more than one case {id}")),
    }
}

/// Reads the manifest at `path` and returns its case `id`.
fn manifest_case(path: &Path, id: &str) -> (String, CaseLines) {
    let manifest = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let case = case_by_id(&manifest, id).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    (manifest, case)
}

/// The first system of the checkout, sorted by folder name, with a case of
/// `status`, and that case's id.
fn first_system_with(status: &str) -> (String, String) {
    let mut systems: Vec<PathBuf> = fs::read_dir(checkout().join("examples"))
        .unwrap_or_else(|e| panic!("cannot list examples/: {e}"))
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()
        .unwrap_or_else(|e| panic!("cannot list examples/: {e}"));
    systems.sort();
    systems
        .iter()
        .filter(|dir| dir.join("example.toml").is_file())
        .find_map(|dir| {
            let path = dir.join("example.toml");
            let manifest =
                fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let cases = case_lines(&manifest).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let id = cases.into_iter().find(|c| c.status == status)?.id;
            let name = dir.file_name()?.to_string_lossy().into_owned();
            Some((name, id))
        })
        .unwrap_or_else(|| panic!("no example system has a {status} case"))
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("create {}: {e}", to.display()));
    let entries = fs::read_dir(from).unwrap_or_else(|e| panic!("list {}: {e}", from.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("list {}: {e}", from.display()));
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("copy {}: {e}", entry.path().display()));
        }
    }
}

/// The folder of one test under `CARGO_TARGET_TMPDIR`, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    /// A new empty folder for the test `test`.
    fn new(test: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("examples_gate")
            .join(test);
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap_or_else(|e| panic!("remove {}: {e}", dir.display()));
        }
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Best effort: a folder left behind is replaced by the next run of the test.
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Copies the system `name` into a fresh suite for the test `test` and
/// returns the suite and the copied system folder.
fn copied_system(test: &str, name: &str) -> (Scratch, PathBuf) {
    let suite = Scratch::new(test);
    let dir = suite.path().join("examples").join(name);
    copy_tree(&checkout().join("examples").join(name), &dir);
    (suite, dir)
}

#[trace("FR-021-AC-1")]
#[test]
fn all_running_cases_pass_on_the_built_binary() {
    let output = run_runner(checkout(), &["check"]);
    assert_all_passed(&output, "check");
}

#[trace("FR-021-AC-2")]
#[test]
fn all_staged_cases_still_fail_as_recorded() {
    let output = run_runner(checkout(), &["staged"]);
    assert_all_passed(&output, "staged");
}

#[trace("FR-021-AC-3")]
#[test]
fn a_changed_verdict_in_a_running_case_fails_the_gate() {
    let (system, id) = first_system_with("running");
    let (suite, dir) = copied_system(
        "a_changed_verdict_in_a_running_case_fails_the_gate",
        &system,
    );
    let (_, case) = manifest_case(&dir.join("example.toml"), &id);
    let path = dir.join("expected").join(format!("{}.json", case.id));
    let recorded = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut structure: Value =
        serde_json::from_str(&recorded).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let category = structure["category"]
        .as_str()
        .unwrap_or_else(|| panic!("{} has no category", path.display()))
        .to_owned();
    structure["category"] = Value::String(format!("{category}-edited"));
    fs::write(&path, structure.to_string()).unwrap_or_else(|e| panic!("{}: {e}", path.display()));

    let output = run_runner(suite.path(), &["check", &dir.to_string_lossy()]);

    let label = format!("{system}/{}", case.id);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr:\n{stderr}");
    assert!(
        stderr.contains(&format!(
            "{label}: outcome differs from expected/{}.json in category",
            case.id
        )),
        "the failure does not name {label}:\n{stderr}"
    );
}

#[trace("FR-021-AC-4")]
#[test]
fn a_staged_case_that_starts_working_fails_the_gate() {
    let (system, id) = first_system_with("staged");
    let (suite, dir) = copied_system("a_staged_case_that_starts_working_fails_the_gate", &system);
    let path = dir.join("example.toml");
    let (manifest, case) = manifest_case(&path, &id);
    let (Some((expect_line, exit)), Some((ready_line, ready))) =
        (&case.expect_exit, &case.ready_exit)
    else {
        panic!("staged case {} has no expect_exit or ready_exit", case.id);
    };
    // The two exits swap: the exit the case gives today becomes its ready
    // exit, as if the driver now did what the case waits for.
    let edited: Vec<String> = manifest
        .lines()
        .enumerate()
        .map(|(n, l)| {
            if n == *ready_line {
                format!("ready_exit = {exit}")
            } else if n == *expect_line {
                format!("expect_exit = {ready}")
            } else {
                l.to_owned()
            }
        })
        .collect();
    fs::write(&path, edited.join("\n") + "\n")
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));

    let output = run_runner(suite.path(), &["staged", &dir.to_string_lossy()]);

    let label = format!("{system}/{}", case.id);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr:\n{stderr}");
    assert!(
        stderr.contains(&format!(
            "{label}: now exits {exit}, as it should once fixed: promote {label}"
        )),
        "no promote message for {label}:\n{stderr}"
    );
}

#[trace("FR-021-AC-5")]
#[test]
fn missing_examples_checkout_is_an_error_not_a_skip() {
    let unset = examples_checkout(None).expect_err("an unset variable is an error");
    assert!(unset.contains(CHECKOUT_VAR), "{unset}");

    let empty = Scratch::new("missing_examples_checkout_is_an_error_not_a_skip");
    let absent = empty.path().join("filament-examples");
    let missing = examples_checkout(Some(absent.clone().into_os_string()))
        .expect_err("an absent checkout is an error");
    assert!(missing.contains(&absent.display().to_string()), "{missing}");

    // The runner itself refuses a suite with no examples/ instead of passing
    // zero cases.
    let output = run_runner(empty.path(), &["check"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "stdout:\n{}\nstderr:\n{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    assert!(
        !text(&output.stdout).contains("check: 0 passed"),
        "an absent suite passed with zero cases"
    );
}

#[trace("FR-021-AC-6")]
#[test]
fn the_gate_runs_the_fsl_of_this_workspace() {
    let fsl = fsl();
    let metadata = metadata(&["--locked", "--no-deps"]);
    let version = packages(&metadata)
        .find(|p| p["name"] == "fsl-cli")
        .and_then(|p| p["version"].as_str())
        .unwrap_or_else(|| panic!("no fsl-cli package in the workspace"))
        .to_owned();

    let reported = Command::new(fsl)
        .arg("--version")
        .output()
        .unwrap_or_else(|e| panic!("cannot start {}: {e}", fsl.display()));
    assert!(reported.status.success(), "{}", text(&reported.stderr));
    assert_eq!(text(&reported.stdout).trim_end(), version);
    let target_dir = metadata["target_directory"]
        .as_str()
        .unwrap_or_else(|| panic!("metadata has no target_directory"));
    assert!(
        fsl.starts_with(target_dir),
        "{} is not a build of this workspace ({target_dir})",
        fsl.display()
    );
}

#[trace("FR-021-AC-7")]
#[test]
fn the_language_packages_come_from_the_named_checkout_or_the_lock() {
    let ids = &fsl_build().package_ids;
    assert!(
        checkout_lock() == fsl_build().lock_before,
        "the fsl build changed this checkout's Cargo.lock"
    );
    let from_git: Vec<&String> = ids
        .iter()
        .filter(|id| is_language_git_package(id))
        .collect();
    match language_checkout() {
        None => {
            assert!(
                !from_git.is_empty(),
                "without {LANGUAGE_VAR} the language packages come from git: {ids:?}"
            );
            let workspace = format!("path+file://{}", env!("CARGO_MANIFEST_DIR"));
            let foreign: Vec<&String> = ids
                .iter()
                .filter(|id| id.starts_with("path+") && !id.starts_with(&workspace))
                .collect();
            assert!(
                foreign.is_empty(),
                "without {LANGUAGE_VAR} no package comes from outside this workspace: {foreign:?}"
            );
        }
        Some(dir) => {
            assert!(
                from_git.is_empty(),
                "with {LANGUAGE_VAR} no language package comes from git: {from_git:?}"
            );
            let prefix = format!("path+file://{}/", dir.display());
            assert!(
                ids.iter().any(|id| id.starts_with(&prefix)),
                "no package was built from {}: {ids:?}",
                dir.display()
            );
            // The patch rewrote the copy's lock, not this checkout's.
            let copied = Path::new(env!("CARGO_TARGET_TMPDIR")).join("language-build/Cargo.lock");
            let copied = fs::read(&copied).unwrap_or_else(|e| panic!("{}: {e}", copied.display()));
            assert!(
                copied != fsl_build().lock_before,
                "the patched build left the copy's Cargo.lock as the checkout's"
            );
            // The language repository's examples-check reads this line to
            // know the gate built its checkout.
            println!("fsl built from language checkout {}", dir.display());
        }
    }
}

#[trace("FR-021-AC-3", "FR-021-AC-4")]
#[test]
fn the_manifest_reader_refuses_a_structure_it_does_not_expect() {
    let case = "[[case]]\nid = \"a\"\nstatus = \"staged\"\nexpect_exit = 20\nready_exit = 0\n";
    let read = case_by_id(&format!("name = \"s\"\n{case}"), "a").expect("the suite's own shape");
    assert_eq!(read.status, "staged");
    assert_eq!(read.expect_exit, Some((4, "20".to_owned())));
    assert_eq!(read.ready_exit, Some((5, "0".to_owned())));

    let refusals = [
        (
            format!("{case}[extra]\nkey = 1\n"),
            "a",
            "unexpected table `[extra]`",
        ),
        (
            case.replace("= 20", "= 20 # was 21"),
            "a",
            "inline comment on `expect_exit`",
        ),
        (
            format!("{case}id = \"b\"\n"),
            "a",
            "`id` is repeated in one case",
        ),
        (
            format!("{case}\"\"\"\n"),
            "a",
            "is not a `key = value` line",
        ),
        (
            "[[case]]\nid = \"a\"\n".to_owned(),
            "a",
            "has no id or no status",
        ),
        (case.to_owned(), "b", "no case b"),
        (format!("{case}{case}"), "a", "more than one case a"),
    ];
    for (manifest, id, reason) in refusals {
        let error = case_by_id(&manifest, id).expect_err(reason);
        assert!(error.contains(reason), "`{error}` does not say `{reason}`");
    }
}
