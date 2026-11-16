// SPDX-License-Identifier: AGPL-3.0-or-later
//! `--clocks` on `fsl run` and `fsl compile` (FR-023, TC-037): the flag gives
//! the outcome the request's own `clocks` member gives, and each refusal of
//! the flag records its code and cause with no stage.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use filament_semantic_value::catalog::Category;
use ix_trace_rs::trace;
use serde_json::{json, Value};

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const UNIT: &str = include_str!("fixtures/clocks-stamped.fsl");

struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("fsl-clocks-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a test folder");
        std::fs::write(path.join("unit.fsl"), UNIT).expect("the unit");
        Self(path)
    }

    fn write(&self, name: &str, bytes: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).expect("a test file");
        path
    }

    /// A request of `verb` (`run` or `compile`) with `clocks` when given.
    fn request(&self, verb: &str, name: &str, clocks: Option<&Value>) -> PathBuf {
        let program = json!({"source": {"file": "unit.fsl", "authority": "issuer",
            "identity": "document", "document": "Clocks", "formal_revision": 1}});
        let mut request = json!({"models": [], "program": program});
        if verb == "run" {
            request["call"] = json!({"function": "seven", "arguments": []});
        }
        if let Some(clocks) = clocks {
            request["clocks"] = clocks.clone();
        }
        let format = format!("native-{verb}/1");
        self.write(
            name,
            &json!({"format": format, "request": request}).to_string(),
        )
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn stamped() -> Value {
    json!({"plant-ms": {"kind": "timestamped_event", "timestamp_unit": "ms"}})
}

fn fsl(verb: &str, request: &Path, extra: &[&Path]) -> Output {
    let mut command = Command::new(FSL);
    command.arg(verb).arg(request).args(["--format", "json"]);
    for file in extra {
        command.arg("--clocks").arg(file);
    }
    command.output().expect("run fsl")
}

fn document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

fn refusal_code() -> i32 {
    i32::from(Category::Refusal.exit_code())
}

/// Assert `output` is a refusal before any stage with `code` and `cause`.
fn assert_flag_refusal(output: &Output, code: &str, cause: &str, message: &str) {
    assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
    let document = document(output);
    assert_eq!(document["last_stage"], Value::Null, "{document}");
    let diagnostic = &document["diagnostics"][0];
    assert_eq!(diagnostic["code"], code, "{document}");
    assert_eq!(diagnostic["cause"], cause, "{document}");
    let text = diagnostic["message"].as_str().expect("a message");
    assert!(text.contains(message), "{text}");
}

/// FR-023-AC-1: `fsl run --clocks` gives the outcome of the same request
/// carrying the object as its `clocks` member; without either, the run
/// refuses, so the flag is what supplied the clock.
#[trace("TC-037", "FR-023-AC-1")]
#[test]
fn run_with_clocks_flag_matches_the_request_member() {
    let folder = Folder::new("run-match");
    let clocks = stamped();
    let member = folder.request("run", "member.json", Some(&clocks));
    let bare = folder.request("run", "bare.json", None);
    let file = folder.write("clocks.json", &clocks.to_string());
    let expected = fsl("run", &member, &[]);
    assert_eq!(expected.status.code(), Some(0), "{expected:?}");
    let actual = fsl("run", &bare, &[&file]);
    assert_eq!(actual.status.code(), expected.status.code());
    assert_eq!(actual.stdout, expected.stdout);
    let without = fsl("run", &bare, &[]);
    assert_ne!(without.status.code(), Some(0), "{without:?}");
    assert_ne!(without.stdout, expected.stdout);
}

/// FR-023-AC-2: the same for `fsl compile`, whose package bytes are the
/// request member's, and which refuses without clocks.
#[trace("TC-037", "FR-023-AC-2")]
#[test]
fn compile_with_clocks_flag_matches_the_request_member() {
    let folder = Folder::new("compile-match");
    let clocks = stamped();
    let member = folder.request("compile", "member.json", Some(&clocks));
    let bare = folder.request("compile", "bare.json", None);
    let file = folder.write("clocks.json", &clocks.to_string());
    let expected = fsl("compile", &member, &[]);
    assert_eq!(expected.status.code(), Some(0), "{expected:?}");
    let actual = fsl("compile", &bare, &[&file]);
    assert_eq!(actual.status.code(), Some(0), "{actual:?}");
    assert_eq!(actual.stdout, expected.stdout);
    let without = fsl("compile", &bare, &[]);
    assert_eq!(without.status.code(), Some(refusal_code()), "{without:?}");
    assert!(
        String::from_utf8_lossy(&without.stdout).contains("missing-clock-selection"),
        "{}",
        String::from_utf8_lossy(&without.stdout)
    );
}

/// FR-023-AC-3: an unreadable file refuses `io-error` naming the path.
#[trace("TC-037", "FR-023-AC-3")]
#[test]
fn an_unreadable_clocks_file_refuses_io_error() {
    let folder = Folder::new("unreadable");
    let missing = folder.0.join("missing-clocks.json");
    for verb in ["run", "compile"] {
        let request = folder.request(verb, &format!("{verb}.json"), None);
        let output = fsl(verb, &request, &[&missing]);
        assert_flag_refusal(
            &output,
            "io-error",
            "clocks-file-unreadable",
            "missing-clocks.json",
        );
    }
}

/// FR-023-AC-4: a file that is not one JSON object refuses `invalid-request`
/// naming `--clocks`.
#[trace("TC-037", "FR-023-AC-4")]
#[test]
fn a_clocks_file_that_is_not_one_object_refuses_invalid_request() {
    let folder = Folder::new("not-object");
    for (name, content) in [
        ("array", "[]"),
        ("text", "not json"),
        ("trailing", "{} {}"),
        ("empty", ""),
    ] {
        let file = folder.write(&format!("{name}.clocks.json"), content);
        for verb in ["run", "compile"] {
            let request = folder.request(verb, &format!("{verb}-{name}.json"), None);
            let output = fsl(verb, &request, &[&file]);
            assert_flag_refusal(
                &output,
                "invalid-request",
                "clocks-file-not-an-object",
                "--clocks",
            );
        }
    }
}

/// FR-023-AC-5: both `clocks` in the request and `--clocks` refuse, as FSL
/// refuses them (FR-640): `invalid-request`, exit 20, naming both, with
/// `last_stage` null (FR-286, FR-287-AC-11).
#[trace("TC-037", "FR-023-AC-5")]
#[test]
fn clocks_in_the_request_and_on_the_command_line_refuse() {
    let folder = Folder::new("twice");
    let clocks = stamped();
    let file = folder.write("clocks.json", &clocks.to_string());
    for verb in ["run", "compile"] {
        let request = folder.request(verb, &format!("{verb}.json"), Some(&clocks));
        let output = fsl(verb, &request, &[&file]);
        assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
        let document = document(&output);
        assert_eq!(document["last_stage"], Value::Null, "{document}");
        let diagnostic = &document["diagnostics"][0];
        assert_eq!(diagnostic["code"], "invalid-request", "{document}");
        let text = diagnostic["message"].as_str().expect("a message");
        assert!(
            text.contains("clocks") && text.contains("--clocks"),
            "{text}"
        );
    }
}

/// A refusal's diagnostic code, cause, stage and message up to FSL's
/// ` at line` position suffix.
fn refusal_fields(output: &Output) -> (Value, Value, Value, String) {
    let document = document(output);
    let diagnostic = &document["diagnostics"][0];
    let message = diagnostic["message"].as_str().expect("a message");
    let message = message.split(" at line").next().unwrap_or(message);
    (
        diagnostic["code"].clone(),
        diagnostic["cause"].clone(),
        document["last_stage"].clone(),
        message.to_owned(),
    )
}

/// FR-023-AC-6: a file member FSL refuses refuses as the same object does as
/// the request's own `clocks` member (code, cause, stage and message), and a
/// repeated member name reaches FSL and is refused naming the clock.
#[trace("TC-037", "FR-023-AC-6")]
#[test]
fn the_file_members_are_read_as_request_members() {
    let folder = Folder::new("members");
    let unknown = json!({"plant-ms": {"kind": "sundial"}});
    let file = folder.write("unknown.json", &unknown.to_string());
    let repeated = folder.write(
        "repeated.json",
        r#"{"plant-ms": {"kind": "timestamped_event", "timestamp_unit": "ms"},
            "plant-ms": {"kind": "timestamped_event", "timestamp_unit": "s"}}"#,
    );
    for verb in ["run", "compile"] {
        let bare = folder.request(verb, &format!("{verb}.json"), None);
        let member = folder.request(verb, &format!("{verb}-member.json"), Some(&unknown));
        let flagged = fsl(verb, &bare, &[&file]);
        let in_request = fsl(verb, &member, &[]);
        assert_eq!(flagged.status.code(), Some(refusal_code()), "{flagged:?}");
        assert_eq!(flagged.status.code(), in_request.status.code());
        let (code, cause, stage, message) = refusal_fields(&flagged);
        assert_eq!(code, "invalid-request");
        assert_eq!((code, cause, stage, message), refusal_fields(&in_request));
        let output = fsl(verb, &bare, &[&repeated]);
        assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
        let (code, _, _, message) = refusal_fields(&output);
        assert_eq!(code, "invalid-request");
        assert!(message.contains("plant-ms"), "{message}");
    }
}

/// FR-023-AC-7: a `--clocks` file over the request byte limit refuses
/// `resource_exhausted` (incomplete, exit 22) naming `--clocks` and the limit,
/// as FSL refuses an oversize request: `run` follows `--request-bytes`.
#[trace("TC-037", "FR-023-AC-7")]
#[test]
fn an_oversize_clocks_file_refuses_resource_exhausted() {
    let folder = Folder::new("oversize");
    let file = folder.write("clocks.json", &stamped().to_string());
    assert!(std::fs::metadata(&file).expect("clocks file").len() > 30);
    let request = folder.request("run", "run.json", None);
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .args(["--request-bytes", "30", "--format", "json"])
        .arg("--clocks")
        .arg(&file)
        .output()
        .expect("run fsl");
    let incomplete = i32::from(Category::Incomplete.exit_code());
    assert_eq!(output.status.code(), Some(incomplete), "{output:?}");
    let document = document(&output);
    assert_eq!(document["last_stage"], Value::Null, "{document}");
    let diagnostic = &document["diagnostics"][0];
    assert_eq!(diagnostic["code"], "resource_exhausted", "{document}");
    assert_eq!(diagnostic["cause"], "clocks-file-too-large", "{document}");
    let message = diagnostic["message"].as_str().expect("a message");
    assert!(
        message.contains("--clocks")
            && message.contains("30")
            && message.contains("--request-bytes"),
        "{message}"
    );
    // `compile` has no limit flag; FSL's fixed 1,048,576 bytes applies.
    let large = folder.write(
        "large.json",
        &format!("{{\"a\": \"{}\"}}", "x".repeat(1_048_576)),
    );
    let request = folder.request("compile", "compile.json", None);
    let output = fsl("compile", &request, &[&large]);
    assert_eq!(output.status.code(), Some(incomplete), "{output:?}");
    let document = self::document(&output);
    let diagnostic = &document["diagnostics"][0];
    assert_eq!(diagnostic["code"], "resource_exhausted", "{document}");
    assert_eq!(diagnostic["cause"], "clocks-file-too-large", "{document}");
    let message = diagnostic["message"].as_str().expect("a message");
    assert!(message.contains("1048576"), "{message}");
}
