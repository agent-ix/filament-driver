// SPDX-License-Identifier: AGPL-3.0-or-later
//! `fsl export` (FR-027, TC-045; FSL FR-1015, FR-1022, FR-287): the verb
//! hands a `native-run/1` request carrying an `export` member to FSL's export
//! entry and `--output` to FSL as the request override, writes FSL's `export`
//! outcome document or its text, and routes `--evidence` to FSL's evidence
//! intake. The fixture is FR-756-AC-1's counter: `inc` while `value < 5`,
//! the state clause `Bounded` (`value <= 5`), starting at `value = 0`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ix_trace_rs::trace;
use serde_json::{json, Value};

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const UNIT: &str = include_str!("fixtures/export-counter.fsl");
const MODEL: &str = include_str!("fixtures/export-counter.semantic-ir.json");
const START: &str = include_str!("fixtures/export-counter-0.json");

/// A fresh directory holding the counter's unit, model and initial snapshot.
fn folder() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("a test folder");
    for (name, text) in [
        ("counter.fsl", UNIT),
        ("counter.json", MODEL),
        ("counter-0.json", START),
    ] {
        std::fs::write(directory.path().join(name), text).expect("a fixture file");
    }
    directory
}

/// A `native-run/1` export request over the counter, with `output` naming a
/// target file when given.
fn request(directory: &Path, output: Option<&str>) -> PathBuf {
    let source = |file: &str, identity: &str| {
        json!({"file": file, "authority": "test", "identity": identity,
            "document": "ExportCounter", "formal_revision": 1})
    };
    let mut export = json!({"profile": "filament.output.tla2/v1", "obligations": ["Bounded"],
        "initial": "counter-0.json"});
    if let Some(output) = output {
        export["output"] = json!({"file": output});
    }
    let request = json!({"format": "native-run/1", "request": {
        "models": [{"format": "semantic-ir/2.0.0",
            "source": source("counter.json", "example/counter")}],
        "program": {"source": source("counter.fsl", "export-counter")},
        "export": export}});
    let path = directory.join("request.json");
    std::fs::write(&path, request.to_string()).expect("the request");
    path
}

fn fsl(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(FSL)
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("run fsl")
}

fn document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

fn mentions(output: &Output, text: &str) {
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(all.contains(text), "names {text}: {all}");
}

/// FR-027-AC-1: `--output` writes FSL's target text, the document is FSL's
/// `export` outcome, and the request member writes the same bytes.
#[test]
#[trace("TC-045", "FR-027-AC-1")]
#[trace("FSL-TC-2995", "FSL-FR-287-AC-14")]
#[trace("FSpec-TC-2021", "FSpec-FR-301-AC-7")]
#[ignore = "QSL-2359: fsl export and its --output handoff are not implemented yet"]
fn export_output_writes_the_target_text_fsl_assembles() {
    let directory = folder();
    request(directory.path(), None);
    let output = fsl(
        directory.path(),
        &[
            "export",
            "request.json",
            "--output",
            "m.tla",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let value = document(&output);
    assert_eq!(value["operation"], "export", "{value}");
    assert_eq!(value["category"], "success", "{value}");
    assert_eq!(value["result"]["kind"], "exported", "{value}");
    let text = value["result"]["export"]["target_text"]
        .as_str()
        .expect("target text");
    let written = std::fs::read(directory.path().join("m.tla")).expect("the target file");
    assert_eq!(written, text.as_bytes());
    let member = folder();
    request(member.path(), Some("m.tla"));
    let output = fsl(
        member.path(),
        &["export", "request.json", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(std::fs::read(member.path().join("m.tla")).unwrap(), written);
}

/// FR-027-AC-2: an existing path, a doubled destination and a repeated
/// `--output` are each refused, exit 20, with no file written or changed.
#[test]
#[trace("TC-045", "FR-027-AC-2")]
#[ignore = "QSL-2359: fsl export refusals are not implemented yet"]
fn export_output_never_overwrites_and_has_one_writer() {
    let directory = folder();
    request(directory.path(), None);
    let target = directory.path().join("m.tla");
    std::fs::write(&target, b"keep").unwrap();
    let output = fsl(
        directory.path(),
        &[
            "export",
            "request.json",
            "--output",
            "m.tla",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let value = document(&output);
    assert_eq!(value["last_stage"], Value::Null, "{value}");
    mentions(&output, "invalid-request");
    mentions(&output, "m.tla");
    assert_eq!(std::fs::read(&target).unwrap(), b"keep");
    let both = folder();
    request(both.path(), Some("member.tla"));
    let output = fsl(
        both.path(),
        &["export", "request.json", "--output", "option.tla"],
    );
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    mentions(&output, "--output");
    assert!(!both.path().join("member.tla").exists());
    assert!(!both.path().join("option.tla").exists());
    let twice = folder();
    request(twice.path(), None);
    let output = fsl(
        twice.path(),
        &[
            "export",
            "request.json",
            "--output",
            "a.tla",
            "--output",
            "b.tla",
        ],
    );
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    mentions(&output, "--output");
}

/// FR-027-AC-3: `--limit export.output_bytes` bounds the target text.
#[test]
#[trace("TC-045", "FR-027-AC-3")]
#[ignore = "QSL-2359: fsl export --limit is not implemented yet"]
fn export_limit_bounds_the_target_text() {
    let directory = folder();
    request(directory.path(), None);
    let output = fsl(
        directory.path(),
        &[
            "export",
            "request.json",
            "--output",
            "m.tla",
            "--limit",
            "export.output_bytes=1",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(22), "{output:?}");
    mentions(&output, "export.output_bytes");
    assert!(!directory.path().join("m.tla").exists());
    let output = fsl(
        directory.path(),
        &[
            "export",
            "request.json",
            "--output",
            "m.tla",
            "--limit",
            "export.output_bytes=16777216",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
}

/// FR-027-AC-4: the text default is FSL FR-865's four export lines.
#[test]
#[trace("TC-045", "FR-027-AC-4")]
#[ignore = "QSL-2359: fsl export text mode is not implemented yet"]
fn export_text_names_the_target_size_and_path() {
    let directory = folder();
    request(directory.path(), None);
    let output = fsl(
        directory.path(),
        &["export", "request.json", "--output", "m.tla"],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let length = std::fs::read(directory.path().join("m.tla"))
        .expect("the target file")
        .len();
    let text = String::from_utf8(output.stdout).expect("UTF-8 text");
    assert_eq!(
        text.lines().collect::<Vec<_>>(),
        [
            "export: success (exit 0) at S4".to_owned(),
            "target: tla filament.output.tla2/v1".to_owned(),
            format!("bytes written: {length}"),
            "output: m.tla".to_owned(),
        ],
        "{text}"
    );
}

/// FR-027-AC-5: `--evidence` reaches FSL's evidence intake on `export`
/// only, and refuses `--output` and a request operand beside it.
#[test]
#[trace("TC-045", "FR-027-AC-5")]
#[trace("FSpec-TC-2020", "FSpec-FR-817-AC-7")]
#[ignore = "QSL-2359: fsl export --evidence is not implemented yet"]
fn export_evidence_routes_to_the_intake_and_refuses_other_arguments() {
    let directory = folder();
    request(directory.path(), None);
    let exported = fsl(
        directory.path(),
        &["export", "request.json", "--format", "json"],
    );
    let package = document(&exported)["result"]["export"]["package"].clone();
    let evidence = json!({"format": "native-run/1", "request": {
        "export_evidence": {"package": package, "obligation": "Bounded", "tool": "tlc",
            "artifact": "counter.cfg"}}});
    std::fs::write(directory.path().join("e.json"), evidence.to_string()).unwrap();
    let output = fsl(
        directory.path(),
        &["export", "--evidence", "e.json", "--format", "json"],
    );
    assert_eq!(document(&output)["operation"], "export", "{output:?}");
    for arguments in [
        &["export", "--evidence", "e.json", "--output", "m.tla"][..],
        &["export", "request.json", "--evidence", "e.json"][..],
    ] {
        let output = fsl(directory.path(), arguments);
        assert_eq!(output.status.code(), Some(20), "{arguments:?}: {output:?}");
        mentions(&output, "--evidence");
        assert!(!directory.path().join("m.tla").exists());
    }
    let output = fsl(
        directory.path(),
        &["compile", "request.json", "--evidence", "e.json"],
    );
    assert_eq!(output.status.code(), Some(20), "{output:?}");
}
