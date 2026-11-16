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
/// refuses them (FR-640): `invalid-request` at stage `request`, naming both.
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
        let diagnostic = &document["diagnostics"][0];
        assert_eq!(diagnostic["code"], "invalid-request", "{document}");
        let text = diagnostic["message"].as_str().expect("a message");
        assert!(
            text.contains("clocks") && text.contains("--clocks"),
            "{text}"
        );
    }
}

/// FR-023-AC-6: the file's members are read as a request's own: an unknown
/// `kind` and a repeated member name refuse at FSL's request reader.
#[trace("TC-037", "FR-023-AC-6")]
#[test]
fn the_file_members_are_read_as_request_members() {
    let folder = Folder::new("members");
    let unknown = folder.write("unknown.json", r#"{"plant-ms": {"kind": "sundial"}}"#);
    let repeated = folder.write(
        "repeated.json",
        r#"{"plant-ms": {"kind": "timestamped_event", "timestamp_unit": "ms"},
            "plant-ms": {"kind": "timestamped_event", "timestamp_unit": "s"}}"#,
    );
    for file in [&unknown, &repeated] {
        for verb in ["run", "compile"] {
            let request = folder.request(verb, &format!("{verb}.json"), None);
            let output = fsl(verb, &request, &[file]);
            assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
            let document = document(&output);
            assert_eq!(
                document["diagnostics"][0]["code"], "invalid-request",
                "{document}"
            );
            let message = document["diagnostics"][0]["message"]
                .as_str()
                .expect("a message");
            assert!(
                message.contains("plant-ms") || message.contains("clocks"),
                "{message}"
            );
        }
    }
}
