// SPDX-License-Identifier: AGPL-3.0-or-later
//! The filament-examples gate: builds `fsl` from this workspace, at the
//! language revision Cargo.lock resolves, and runs the examples runner of a
//! filament-examples checkout with it over every running and every staged case.
//!
//! `FSL_EXAMPLES_DIR` names the checkout; `make examples-gate` fetches it and
//! sets the variable. A missing checkout fails the gate. This target is not
//! part of a plain `cargo test` (`test = false` in Cargo.toml) because it needs
//! the fetched suite.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

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
            format!("{CHECKOUT_VAR} is not set; run `make examples-gate` to fetch the suite")
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

/// Builds the binary target `bin` with `cargo build --locked <args>` and
/// returns the executable cargo reports for it.
fn build_bin(args: &[&str], bin: &str) -> PathBuf {
    let output = Command::new(cargo())
        .args([
            "build",
            "--locked",
            "--message-format=json-render-diagnostics",
        ])
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("cannot start cargo: {e}"));
    assert!(
        output.status.success(),
        "cargo build {args:?} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|m| m["reason"] == "compiler-artifact" && m["target"]["name"] == bin)
        .find_map(|m| m["executable"].as_str().map(PathBuf::from))
        .unwrap_or_else(|| panic!("cargo build {args:?} reported no executable for {bin}"))
}

/// The `fsl` binary of this workspace's `fsl-cli` package.
fn fsl() -> &'static Path {
    static FSL: OnceLock<PathBuf> = OnceLock::new();
    FSL.get_or_init(|| {
        let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
        build_bin(
            &["--manifest-path", manifest, "-p", "fsl-cli", "--bin", "fsl"],
            "fsl",
        )
    })
}

/// The runner binary built from the checkout's `tools/fsl-examples` crate.
fn runner() -> &'static Path {
    static RUNNER: OnceLock<PathBuf> = OnceLock::new();
    RUNNER.get_or_init(|| {
        let manifest = runner_manifest(checkout());
        let manifest = manifest.to_string_lossy();
        build_bin(
            &["--manifest-path", &manifest, "--bin", "fsl-examples"],
            "fsl-examples",
        )
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
struct CaseLines {
    id: String,
    status: String,
    expect_exit: Option<String>,
    ready_exit_line: Option<usize>,
}

/// Reads the `[[case]]` tables of an `example.toml`. Only the keys the
/// negative tests edit are read.
fn case_lines(manifest: &str) -> Vec<CaseLines> {
    let mut cases: Vec<CaseLines> = Vec::new();
    for (n, line) in manifest.lines().enumerate() {
        let line = line.trim();
        if line == "[[case]]" {
            cases.push(CaseLines {
                id: String::new(),
                status: String::new(),
                expect_exit: None,
                ready_exit_line: None,
            });
            continue;
        }
        let (Some(case), Some((key, value))) = (cases.last_mut(), line.split_once(" = ")) else {
            continue;
        };
        let value = value.trim_matches('"').to_owned();
        match key {
            "id" => case.id = value,
            "status" => case.status = value,
            "expect_exit" => case.expect_exit = Some(value),
            "ready_exit" => case.ready_exit_line = Some(n),
            _ => {}
        }
    }
    cases
}

/// The first system of the checkout, sorted by folder name, with a case of
/// `status`, and that case.
fn first_system_with(status: &str) -> (String, CaseLines) {
    let mut systems: Vec<PathBuf> = fs::read_dir(checkout().join("examples"))
        .unwrap_or_else(|e| panic!("cannot list examples/: {e}"))
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()
        .unwrap_or_else(|e| panic!("cannot list examples/: {e}"));
    systems.sort();
    systems
        .iter()
        .filter_map(|dir| {
            let manifest = fs::read_to_string(dir.join("example.toml")).ok()?;
            let case = case_lines(&manifest)
                .into_iter()
                .find(|c| c.status == status)?;
            let name = dir.file_name()?.to_string_lossy().into_owned();
            Some((name, case))
        })
        .next()
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

/// Copies the system `name` into a fresh suite and returns the suite and the
/// copied system folder.
fn copied_system(name: &str) -> (tempfile::TempDir, PathBuf) {
    let suite = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let dir = suite.path().join("examples").join(name);
    copy_tree(&checkout().join("examples").join(name), &dir);
    (suite, dir)
}

#[test]
fn all_running_cases_pass_on_the_built_binary() {
    let output = run_runner(checkout(), &["check"]);
    assert_all_passed(&output, "check");
}

#[test]
fn all_staged_cases_still_fail_as_recorded() {
    let output = run_runner(checkout(), &["staged"]);
    assert_all_passed(&output, "staged");
}

#[test]
fn a_changed_verdict_in_a_running_case_fails_the_gate() {
    let (system, case) = first_system_with("running");
    let (suite, dir) = copied_system(&system);
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

#[test]
fn a_staged_case_that_starts_working_fails_the_gate() {
    let (system, case) = first_system_with("staged");
    let (suite, dir) = copied_system(&system);
    let path = dir.join("example.toml");
    let manifest = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (Some(exit), Some(line)) = (&case.expect_exit, case.ready_exit_line) else {
        panic!("staged case {} has no expect_exit or ready_exit", case.id);
    };
    // The staged case's recorded failing exit becomes its ready exit, as if
    // the driver now did what the case waits for.
    let edited: Vec<String> = manifest
        .lines()
        .enumerate()
        .map(|(n, l)| {
            if n == line {
                format!("ready_exit = {exit}")
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

#[test]
fn missing_examples_checkout_is_an_error_not_a_skip() {
    let unset = examples_checkout(None).expect_err("an unset variable is an error");
    assert!(unset.contains(CHECKOUT_VAR), "{unset}");

    let empty = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
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

#[test]
fn the_gate_runs_the_fsl_of_this_workspace() {
    let output = Command::new(cargo())
        .args(["metadata", "--format-version", "1", "--locked", "--no-deps"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap_or_else(|e| panic!("cannot start cargo metadata: {e}"));
    assert!(output.status.success(), "{}", text(&output.stderr));
    let metadata: Value =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|e| panic!("metadata: {e}"));
    let version = metadata["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|p| p["name"] == "fsl-cli")
        .and_then(|p| p["version"].as_str())
        .unwrap_or_else(|| panic!("no fsl-cli package in the workspace"))
        .to_owned();

    let reported = Command::new(fsl())
        .arg("--version")
        .output()
        .unwrap_or_else(|e| panic!("cannot start {}: {e}", fsl().display()));
    assert!(reported.status.success(), "{}", text(&reported.stderr));
    assert_eq!(text(&reported.stdout).trim_end(), version);
    let target_dir = metadata["target_directory"]
        .as_str()
        .unwrap_or_else(|| panic!("metadata has no target_directory"));
    assert!(
        fsl().starts_with(target_dir),
        "{} is not a build of this workspace ({target_dir})",
        fsl().display()
    );
}
