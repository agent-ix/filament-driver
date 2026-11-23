// SPDX-License-Identifier: AGPL-3.0-or-later
//! `--output` on `fsl compile` (FR-024, TC-038): the flag makes FSL write the
//! checked package to a new file, and every refusal writes none.

use std::path::PathBuf;
use std::process::{Command, Output};

use filament_semantic_value::catalog::Category;
use ix_trace_rs::trace;
use serde_json::{json, Value};

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const UNIT: &str = include_str!("fixtures/ir608-run.fsl");
const REFUSED_UNIT: &str = include_str!("fixtures/check-invalid.fsl");

struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("fsl-output-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a test folder");
        std::fs::write(path.join("unit.fsl"), UNIT).expect("the unit");
        std::fs::write(path.join("refused.fsl"), REFUSED_UNIT).expect("the refused unit");
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// A `native-compile/1` request over `file`, with `output` when given.
    fn request(&self, name: &str, file: &str, output: Option<&str>) -> PathBuf {
        let program = json!({"source": {"file": file, "authority": "issuer",
            "identity": "document", "document": "CliFrontend", "formal_revision": 1}});
        let mut request = json!({"models": [], "program": program});
        if let Some(output) = output {
            request["output"] = json!({"package": output});
        }
        let path = self.path(name);
        let text = json!({"format": "native-compile/1", "request": request}).to_string();
        std::fs::write(&path, text).expect("a request");
        path
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn refusal_code() -> i32 {
    i32::from(Category::Refusal.exit_code())
}

/// Run `fsl` in `folder`, so a relative path resolves against the working
/// directory.
fn fsl_in(folder: &Folder, arguments: &[&str]) -> Output {
    Command::new(FSL)
        .current_dir(&folder.0)
        .args(arguments)
        .output()
        .expect("run fsl")
}

fn document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

/// Assert the outcome document of a refusal before any stage: exit 20,
/// `invalid-request`, `last_stage` null, and the message naming each of `names`.
fn assert_invalid_request(output: &Output, names: &[&str]) {
    assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
    let document = document(output);
    assert_eq!(document["last_stage"], Value::Null, "{document}");
    let diagnostic = &document["diagnostics"][0];
    assert_eq!(diagnostic["code"], "invalid-request", "{document}");
    let message = diagnostic["message"].as_str().expect("a message");
    for name in names {
        assert!(message.contains(name), "{name} missing from {message}");
    }
}

/// Assert a refusal of the command line itself: exit 20, nothing on stderr,
/// and its text refusal outcome on stdout containing `message`.
fn assert_argument_refusal(output: &Output, message: &str) {
    assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("refusal (exit 20)"), "{stdout}");
    assert!(stdout.contains(message), "{stdout}");
    assert!(!stdout.contains("panicked"), "{stdout}");
}

/// FR-024-AC-1: `fsl compile --output` writes the bytes the library writes for
/// the same request carrying the `output` member, which are the bytes
/// `command::compile` returns, and the outcome's `artifacts` stay empty.
#[trace("TC-038", "FR-024-AC-1")]
#[test]
fn compile_output_writes_the_library_bytes() {
    let folder = Folder::new("bytes");
    folder.request("member.json", "unit.fsl", Some("member.package.json"));
    let bare = folder.request("bare.json", "unit.fsl", None);
    let expected = fsl_command::command::compile(&bare).expect("compile");
    assert!(!expected.is_empty());

    let by_member = fsl_in(&folder, &["compile", "member.json", "--format", "json"]);
    assert_eq!(by_member.status.code(), Some(0), "{by_member:?}");
    assert_eq!(
        std::fs::read(folder.path("member.package.json")).expect("member file"),
        expected
    );

    // A relative path resolves against the working directory, not the request.
    let flagged = fsl_in(
        &folder,
        &[
            "compile",
            "bare.json",
            "--output",
            "cli.package.json",
            "--format",
            "json",
        ],
    );
    assert_eq!(flagged.status.code(), Some(0), "{flagged:?}");
    assert_eq!(
        std::fs::read(folder.path("cli.package.json")).expect("flag file"),
        expected
    );
    let flagged_document = document(&flagged);
    assert_eq!(
        flagged_document["artifacts"],
        json!([]),
        "{flagged_document}"
    );
    assert_eq!(flagged_document, document(&by_member));
}

/// FR-024-AC-2: an existing target is refused `invalid-request` naming the
/// path, and its bytes stay as they were.
#[trace("TC-038", "FR-024-AC-2")]
#[test]
fn an_existing_output_is_refused_and_left_untouched() {
    let folder = Folder::new("exists");
    folder.request("bare.json", "unit.fsl", None);
    let target = folder.path("taken.package.json");
    std::fs::write(&target, b"keep me").expect("a target");
    let output = fsl_in(
        &folder,
        &[
            "compile",
            "bare.json",
            "--output",
            "taken.package.json",
            "--format",
            "json",
        ],
    );
    assert_invalid_request(&output, &["taken.package.json"]);
    assert_eq!(std::fs::read(&target).expect("target"), b"keep me");
}

/// FR-024-AC-3: a request carrying `output` together with `--output` is
/// refused `invalid-request` naming both, and neither file is written.
#[trace("TC-038", "FR-024-AC-3")]
#[test]
fn output_in_the_request_and_on_the_command_line_refuse() {
    let folder = Folder::new("twice");
    folder.request("member.json", "unit.fsl", Some("member.package.json"));
    let output = fsl_in(
        &folder,
        &[
            "compile",
            "member.json",
            "--output",
            "cli.package.json",
            "--format",
            "json",
        ],
    );
    assert_invalid_request(&output, &["output", "--output"]);
    assert!(!folder.path("member.package.json").exists());
    assert!(!folder.path("cli.package.json").exists());
}

/// FR-024-AC-4: `--output` on a verb other than `compile` is an unknown
/// option.
#[trace("TC-038", "FR-024-AC-4")]
#[test]
fn output_on_another_verb_is_an_unknown_option() {
    let folder = Folder::new("verbs");
    let cases: [(&str, &[&str], &str); 3] = [
        (
            "run",
            &["run", "bare.json", "--output", "x.json"],
            "unknown run option `--output`",
        ),
        (
            "check",
            &["check", "unit.fsl", "--output", "x.json"],
            "unknown check option `--output`",
        ),
        (
            "analyze",
            &["analyze", "bare.json", "--output", "x.json"],
            "unknown analyze option `--output`",
        ),
    ];
    for (verb, arguments, message) in cases {
        let output = fsl_in(&folder, arguments);
        assert_argument_refusal(&output, message);
        assert!(!folder.path("x.json").exists(), "{verb}");
    }
}

/// FR-024-AC-5: `--output` given twice, or without a path, is refused and
/// writes no file.
#[trace("TC-038", "FR-024-AC-5")]
#[test]
fn a_repeated_or_pathless_output_flag_is_refused() {
    let folder = Folder::new("repeat");
    folder.request("bare.json", "unit.fsl", None);
    let repeated = fsl_in(
        &folder,
        &[
            "compile",
            "bare.json",
            "--output",
            "a.json",
            "--output",
            "b.json",
        ],
    );
    assert_argument_refusal(&repeated, "--output may be supplied only once");
    let pathless = fsl_in(&folder, &["compile", "bare.json", "--output"]);
    assert_argument_refusal(&pathless, "--output requires a path");
    assert!(!folder.path("a.json").exists());
    assert!(!folder.path("b.json").exists());
}

/// FR-024-AC-6: a compile that refuses writes no file.
#[trace("TC-038", "FR-024-AC-6")]
#[test]
fn a_refused_compile_writes_no_output_file() {
    let folder = Folder::new("refused");
    folder.request("refused.json", "refused.fsl", None);
    let output = fsl_in(
        &folder,
        &[
            "compile",
            "refused.json",
            "--output",
            "out.json",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
    assert_eq!(document(&output)["diagnostics"][0]["code"], "ill_typed");
    assert!(!folder.path("out.json").exists());
}
