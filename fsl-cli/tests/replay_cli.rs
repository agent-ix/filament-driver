// SPDX-License-Identifier: AGPL-3.0-or-later
//! `fsl replay` (FSpec FR-715, FSL FR-862): the verb hands a
//! `native-replay/1` request file to FSL's one replay reader. A complete
//! request over a compiled package replays and writes the FR-323 `results`
//! document; a `clocks` member is refused at stage `request` because a replay
//! reads its clocks from the proving package (FSL FR-640, FR-023); a request
//! file of another format is refused before FSL runs it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use filament_semantic_value::catalog::Category;
use fsl_foundation::SourceIdentity;
use fsl_replay::{call_site, DependencyInput, QualifiedName, SuppliedLibrary};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const AUTHORITY: &str = "agent-ix";
const PROGRAM_ID: &str = "test:replay-program";
const LIBRARY_ID: &str = "test:replay-plant";
const LIBRARY: &str = "test/plant";
const PROGRAM: &str = "language \"ix:native\" edition \"1-draft\";\n\
     profile v = \"filament.value.complete/v1\";\n\
     import \"test/plant\" as P;\n\
     function entry using v(): Boolean pure { true }\n";
const LIBRARY_SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
     profile v = \"filament.value.complete/v1\";\n\
     function flag using v(): Boolean pure { true }\n";

fn refusal_code() -> i32 {
    i32::from(Category::Refusal.exit_code())
}

fn fsl(arguments: &[&str]) -> Output {
    Command::new(FSL).args(arguments).output().expect("run fsl")
}

fn replay(request: &Path, format: &str) -> Output {
    fsl(&[
        "replay",
        request.to_str().expect("a UTF-8 path"),
        "--format",
        format,
    ])
}

fn document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

fn write(directory: &Path, name: &str, value: &Value) -> PathBuf {
    let path = directory.join(name);
    std::fs::write(&path, value.to_string()).expect("a test file");
    path
}

/// A replay request in FSpec FR-336's mapping, well formed in shape, with
/// `extra` merged into it; the refusals below are decided before its package
/// is admitted.
fn shaped_request(extra: &Value) -> Value {
    let mut request = json!({
        "profile_selections": [],
        "package": {},
        "dependencies": [],
        "replay_target": {"members": {"function": "entry"}},
        "source": {"kind": "input", "assignments": []},
        "obligation_identity": {"node_id": {}, "model": null, "ordinal": "0"},
        "backend": "fsl",
        "state_environment": [],
        "accounting_limits": {},
        "stage_limits": {},
        "supplied_limits": null,
        "object_environment": null,
        "declared_domains": [],
        "byte_provision": []
    });
    if let (Some(members), Some(added)) = (request.as_object_mut(), extra.as_object()) {
        for (name, value) in added {
            members.insert(name.clone(), value.clone());
        }
    }
    json!({"format": "native-replay/1", "request": request})
}

/// Assert `output` is the replay's `invalid-request` refusal naming `member`,
/// with no result written and no stage reached.
fn assert_request_refusal(output: &Output, member: &str) {
    assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
    let document = document(output);
    assert_eq!(document["operation"], "replay", "{document}");
    assert_eq!(document["category"], "refusal", "{document}");
    assert_eq!(document["last_stage"], Value::Null, "{document}");
    assert!(document.get("results").is_none(), "{document}");
    let diagnostic = &document["diagnostics"][0];
    assert_eq!(diagnostic["code"], "invalid-request", "{document}");
    let message = diagnostic["message"].as_str().expect("a message");
    assert!(
        message.contains(&format!("\"{member}\"")) || message.contains(&format!("/{member}")),
        "the refusal names {member}: {message}"
    );
}

/// FSL FR-640-AC-8, FSpec FR-715-AC-1 (negative half): a replay request
/// carrying `clocks` refuses at stage `request`, `invalid-request`, exit 20,
/// naming `clocks`, with no result; an unknown member and a missing member
/// refuse the same way through the same reader.
#[trace("TC-037", "FR-640-AC-8", "FR-862-AC-1", "FSpec-FR-715-AC-1")]
#[test]
fn a_replay_request_carrying_clocks_refuses_at_stage_request() {
    let directory = tempfile::tempdir().expect("a test folder");
    let clocks = shaped_request(&json!({"clocks": {"plant": {"kind": "timed", "unit": "ms"}}}));
    assert_request_refusal(
        &replay(&write(directory.path(), "clocks.json", &clocks), "json"),
        "clocks",
    );
    let extra = shaped_request(&json!({"extra": true}));
    assert_request_refusal(
        &replay(&write(directory.path(), "extra.json", &extra), "json"),
        "extra",
    );
    let empty = json!({"format": "native-replay/1", "request": {}});
    assert_request_refusal(
        &replay(&write(directory.path(), "empty.json", &empty), "json"),
        "profile_selections",
    );
}

/// FSpec FR-301-AC-4: the clocks refusal in text mode is written, not
/// silent, and keeps exit 20.
#[trace("TC-037", "FR-640-AC-8", "FSpec-FR-301-AC-4")]
#[test]
fn a_clocks_refusal_in_text_mode_is_written() {
    let directory = tempfile::tempdir().expect("a test folder");
    let clocks = shaped_request(&json!({"clocks": {"plant": {"kind": "timed", "unit": "ms"}}}));
    let output = replay(&write(directory.path(), "clocks.json", &clocks), "text");
    assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("invalid-request"), "{text}");
    assert!(text.contains("clocks"), "{text}");
}

/// A request file of another format is refused naming `native-replay/1`
/// before FSL reads it, so `replay` never runs a run request.
#[trace("TC-037", "FSpec-FR-715-AC-1")]
#[test]
fn a_request_of_another_format_is_not_replayed() {
    let directory = tempfile::tempdir().expect("a test folder");
    let run = json!({"format": "native-run/1", "request": {
        "models": [], "program": {"source": {"file": "unit.fsl"}},
        "call": {"function": "entry", "arguments": []}}});
    let output = replay(&write(directory.path(), "run.json", &run), "json");
    assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
    let document = document(&output);
    assert_eq!(document["operation"], "replay", "{document}");
    let diagnostic = &document["diagnostics"][0];
    assert_eq!(diagnostic["code"], "invalid-request", "{document}");
    let message = diagnostic["message"].as_str().expect("a message");
    assert!(
        message.contains("native-replay/1") && message.contains("native-run/1"),
        "{message}"
    );
}

/// FSpec FR-301-AC-4: `replay` with no request operand, or with two, is a
/// usage refusal, exit 20, never the unrouted-verb result.
#[trace("TC-037", "FSpec-FR-301-AC-4")]
#[test]
fn replay_without_one_request_operand_is_a_usage_refusal() {
    for arguments in [
        &["replay", "--format", "json"][..],
        &["replay", "a.json", "b.json", "--format", "json"],
    ] {
        let output = fsl(arguments);
        assert_eq!(output.status.code(), Some(refusal_code()), "{arguments:?}");
        let document = document(&output);
        assert_eq!(document["operation"], "replay", "{document}");
        let message = document["diagnostics"][0]["message"]
            .as_str()
            .expect("a message");
        assert!(
            message.starts_with("usage: fsl replay [options] <request-file>"),
            "{message}"
        );
    }
}

fn source(file: &str, identity: &str) -> Value {
    json!({"file": file, "authority": AUTHORITY, "identity": identity,
        "document": "Replay", "formal_revision": 1})
}

fn entry() -> QualifiedName {
    QualifiedName::new(vec![
        filament_exact::Identifier::new("entry").expect("a name")
    ])
    .expect("a qualified name")
}

/// Compile the program and its library with `fsl compile --output`, and
/// return the checked package document.
fn compile_package(directory: &Path) -> Value {
    std::fs::write(directory.join("unit.native"), PROGRAM).expect("the program");
    std::fs::write(directory.join("plant.native"), LIBRARY_SOURCE).expect("the library");
    let request = write(
        directory,
        "compile.json",
        &json!({"format": "native-compile/1", "request": {
            "models": [],
            "program": {"source": source("unit.native", PROGRAM_ID)},
            "libraries": [{"identity": LIBRARY, "source": source("plant.native", LIBRARY_ID)}]}}),
    );
    let package = directory.join("package.json");
    let output = fsl(&[
        "compile",
        request.to_str().expect("a UTF-8 path"),
        "--output",
        package.to_str().expect("a UTF-8 path"),
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    serde_json::from_slice(&std::fs::read(&package).expect("the package")).expect("JSON")
}

/// The `native-replay/1` request replaying `entry` over `package`, with the
/// sources supplied as `byte_provision` files.
fn replay_request(package: &Value) -> Value {
    let dependencies = DependencyInput::new([SuppliedLibrary {
        identity: LIBRARY.to_owned(),
        source: SourceIdentity::new(AUTHORITY, LIBRARY_ID),
        path: "plant.native".to_owned(),
        bytes: LIBRARY_SOURCE.as_bytes().to_vec(),
    }])
    .expect("the library input");
    let site = call_site(
        SourceIdentity::new(AUTHORITY, PROGRAM_ID),
        package["name"].as_str().expect("a package name").to_owned(),
        "unit.native",
        PROGRAM.as_bytes(),
        std::iter::empty::<&[u8]>(),
        &dependencies,
        &fsl_replay::RequestClocks::default(),
        &entry(),
    )
    .expect("the call site resolves");
    let graph = fsl_replay::WireGraph::of_package_value(package).expect("the package graph");
    let occurrence = &site.site.declaration;
    let obligation = site.site.obligation_identity();
    json!({"format": "native-replay/1", "request": {
        "profile_selections": [],
        "package": package,
        "dependencies": [{"identity": LIBRARY,
            "sources": [{"authority": AUTHORITY, "identity": LIBRARY_ID}]}],
        "replay_target": {
            "members": {"function": "entry"},
            "arguments": false,
            "occurrence_key": {
                "node_id": graph.wire_value(occurrence.node()).expect("a node id"),
                "role": occurrence.origin().role().as_str(),
                "ordinal": occurrence.origin().ordinal().to_string()},
            "over": null},
        "source": {"kind": "input", "assignments": []},
        "obligation_identity": {
            "node_id": graph.wire_value(obligation.node()).expect("a node id"),
            "model": null,
            "ordinal": obligation.ordinal()},
        "backend": "fsl-replay-cli",
        "state_environment": [],
        "accounting_limits": {},
        "stage_limits": {},
        "supplied_limits": null,
        "object_environment": null,
        "declared_domains": [],
        "byte_provision": [
            {"authority": AUTHORITY, "identity": PROGRAM_ID, "bytes_path": "unit.native"},
            {"authority": AUTHORITY, "identity": LIBRARY_ID, "bytes_path": "plant.native"}]
    }})
}

/// FSL FR-862, FSpec FR-715: a complete replay request over the package
/// `fsl compile` wrote replays, writes the FR-323 `results` document with the
/// input arm's settlement and exits 0; the same request with `clocks` added
/// writes no result.
#[trace("TC-037", "FR-862-AC-1", "FSpec-FR-715-AC-1")]
#[test]
fn a_replay_request_over_a_compiled_package_replays() {
    let directory = tempfile::tempdir().expect("a test folder");
    let package = compile_package(directory.path());
    let mut request = replay_request(&package);
    let output = replay(&write(directory.path(), "replay.json", &request), "json");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let document = document(&output);
    let results = document["results"].as_array().expect("a results array");
    assert_eq!(results.len(), 1, "{document}");
    assert_eq!(results[0]["arm"], "input", "{document}");
    assert!(results[0]["settlement"].is_string(), "{document}");

    request["request"]["clocks"] = json!({"plant": {"kind": "timed", "unit": "ms"}});
    assert_request_refusal(
        &replay(&write(directory.path(), "clocks.json", &request), "json"),
        "clocks",
    );
}

/// FR-026-AC-1: machine mode writes the replay outcome document, carrying the
/// `replayed` result, as one canonical line.
#[trace("TC-042", "FR-026-AC-1")]
#[test]
#[ignore = "QSL-2292: the replay output contract (FR-026) is not verified yet"]
fn machine_mode_writes_the_results_document_as_one_line() {
    let directory = tempfile::tempdir().expect("a test folder");
    let package = compile_package(directory.path());
    let request = write(directory.path(), "replay.json", &replay_request(&package));
    let output = replay(&request, "json");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let text = String::from_utf8(output.stdout.clone()).expect("UTF-8");
    assert!(
        text.ends_with('\n') && text.matches('\n').count() == 1,
        "{text}"
    );
    let document = document(&output);
    assert_eq!(document["operation"], "replay", "{document}");
    assert_eq!(document["result"]["kind"], "replayed", "{document}");
    assert_eq!(
        document["result"]["results"][0]["arm"], "input",
        "{document}"
    );
    assert_eq!(
        document["result"]["results"][0]["settlement"], "reproduced-without-witness",
        "{document}"
    );
}

/// FR-026-AC-2: text mode writes the outcome header and one result line per
/// row.
#[trace("TC-042", "FR-026-AC-2")]
#[test]
#[ignore = "QSL-2292: fsl replay writes no result line in text mode yet"]
fn text_mode_writes_the_header_and_one_result_line() {
    let directory = tempfile::tempdir().expect("a test folder");
    let package = compile_package(directory.path());
    let request = write(directory.path(), "replay.json", &replay_request(&package));
    let output = fsl(&["replay", request.to_str().expect("a UTF-8 path")]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let text = String::from_utf8(output.stdout).expect("UTF-8");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines,
        [
            "replay: success (exit 0) at stage replay",
            "result 0: input reproduced-without-witness"
        ],
        "{text}"
    );
}

/// FR-026-AC-3 and AC-4 (negative): a refused replay and a request of
/// another format write the refusal header in text mode and no result line.
#[trace("TC-042", "FR-026-AC-3")]
#[trace("TC-042", "FR-026-AC-4")]
#[test]
#[ignore = "QSL-2292: the replay text refusal form (FR-026) is not verified yet"]
fn text_mode_refusals_write_no_result_line() {
    let directory = tempfile::tempdir().expect("a test folder");
    let package = compile_package(directory.path());
    let mut request = replay_request(&package);
    request["request"]["clocks"] = json!({"plant": {"kind": "timed", "unit": "ms"}});
    let clocks = write(directory.path(), "clocks.json", &request);
    let run = write(
        directory.path(),
        "run.json",
        &json!({"format": "native-run/1", "request": {"models": []}}),
    );
    for path in [clocks, run] {
        let output = fsl(&["replay", path.to_str().expect("a UTF-8 path")]);
        assert_eq!(output.status.code(), Some(refusal_code()), "{output:?}");
        let text = String::from_utf8(output.stdout).expect("UTF-8");
        assert!(text.starts_with("replay: refusal (exit 20)"), "{text}");
        assert!(
            !text.lines().any(|line| line.starts_with("result ")),
            "{text}"
        );
    }
}
