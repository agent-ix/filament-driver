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

/// The handoff uses its declared package name; source export uses the authored
/// identity. Their native references therefore differ in provenance comments,
/// name-map nodes and byte offsets, while their executable TLA stays identical.
fn assert_counter_export_correspondence(source: &Value, selected: &Value, package_bytes: &[u8]) {
    let model = json!({"ordinal": 0, "package": "example/counter",
        "path": ["model", "object_type", "ix://example/counter/Counter"]});
    let anchor = |package, node| {
        json!({"ordinal": 0, "package": package,
        "path": ["state", "operation_anchor", {"node": node}, "inc"]})
    };
    let clause = |package, node| {
        json!({"ordinal": 0, "package": package,
        "path": ["state", "state_clause", "invariant", {"node": node}, {"node": 12}]})
    };
    let source_anchor = anchor("example/counter-unit", 0);
    let selected_anchor = anchor("counter-unit", 5);
    let source_clause = clause("example/counter-unit", 0);
    let selected_clause = clause("counter-unit", 5);
    // The wire holds operand slots; reports name document-held graph indices.
    // Use the owning reader's resolved view to check the reported references.
    let admitted = fsl_package::admit_checked_package(
        package_bytes,
        &std::collections::BTreeMap::from([(
            "example/counter".to_owned(),
            MODEL.as_bytes().to_vec(),
        )]),
        fsl_package::V2ReadLimits::bounded(fsl_replay::spine::I2_NATIVE_JOIN_STORAGE_DEFAULT),
    )
    .expect("the original producer package with its selected model");
    let nodes: Vec<_> = admitted
        .resolved_graph()
        .nodes
        .iter()
        .map(|node| serde_json::to_value(&node.node_id).unwrap())
        .collect();
    for expected in [&model, &selected_anchor, &selected_clause] {
        assert!(nodes.contains(expected), "{expected}: {nodes:?}");
    }
    let comment =
        |node: &Value, region: &str| format!("\\* fsl: {node} test/example/counter-unit:{region}");
    let provenance = [
        (comment(&model, "0-1"), comment(&model, "0-1")),
        (
            comment(&source_anchor, "205-219"),
            comment(&selected_anchor, "205-219"),
        ),
        (
            comment(&source_clause, "384-399"),
            comment(&selected_clause, "384-399"),
        ),
    ];
    let source_text = source["target_text"].as_str().unwrap();
    let selected_text = selected["target_text"].as_str().unwrap();
    let source_lines: Vec<_> = source_text.lines().collect();
    let selected_lines: Vec<_> = selected_text.lines().collect();
    assert_eq!(source_lines.len(), selected_lines.len());
    let mut comment_counts = [0; 3];
    for (source_line, selected_line) in source_lines.iter().zip(&selected_lines) {
        if source_line.starts_with(r"\* fsl: ") {
            let index = provenance
                .iter()
                .position(|(expected, _)| expected.as_str() == *source_line)
                .expect(
                    "each source provenance comment names its admitted native identity and region",
                );
            assert_eq!(*selected_line, provenance[index].1);
            comment_counts[index] += 1;
        } else {
            assert_eq!(
                source_line, selected_line,
                "executable TLA and every other line must match exactly"
            );
        }
    }
    assert_eq!(comment_counts, [6, 1, 2]);

    let source_records = source["records"].as_array().unwrap();
    let selected_records = selected["records"].as_array().unwrap();
    assert_eq!(source_records.len(), 1);
    assert_eq!(selected_records.len(), 1);
    for member in ["obligation", "disposition", "conditions", "causes"] {
        assert_eq!(source_records[0][member], selected_records[0][member]);
    }
    // Regions are offsets into each complete, unmodified target, including its
    // own provenance. Compare the represented definitions, not numeric offsets.
    let represented = |record: &Value, text: &str| -> Vec<String> {
        record["regions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|region| {
                let start = usize::try_from(region[0].as_u64().unwrap()).unwrap();
                let end = usize::try_from(region[1].as_u64().unwrap()).unwrap();
                text.get(start..end)
                    .expect("valid target byte region")
                    .to_owned()
            })
            .collect()
    };
    let source_regions = represented(&source_records[0], source_text);
    let selected_regions = represented(&selected_records[0], selected_text);
    assert_eq!(source_regions, selected_regions);
    assert_eq!(selected_regions.len(), 2);
    assert!(selected_regions[0].starts_with("c_Bounded == "));
    assert_eq!(
        selected_regions[1],
        "THEOREM Th_Bounded == Spec => []c_Bounded"
    );

    let source_names = source["name_map"].as_array().unwrap();
    let selected_names = selected["name_map"].as_array().unwrap();
    assert_eq!(source_names.len(), 5);
    assert_eq!(selected_names.len(), 5);
    for (index, (name, source_node, selected_node, region)) in [
        (
            "Th_Bounded",
            &source_clause,
            &selected_clause,
            json!([384, 399]),
        ),
        (
            "c_Bounded",
            &source_clause,
            &selected_clause,
            json!([384, 399]),
        ),
        ("fld_value", &model, &model, json!([0, 1])),
        (
            "op_Counter_inc",
            &source_anchor,
            &selected_anchor,
            json!([205, 219]),
        ),
        ("v_Counter", &model, &model, json!([0, 1])),
    ]
    .into_iter()
    .enumerate()
    {
        for (entry, expected) in [
            (&source_names[index], source_node),
            (&selected_names[index], selected_node),
        ] {
            assert_eq!(entry["name"], name);
            assert_eq!(entry["region"], region);
            let node: Value = serde_json::from_str(entry["node"].as_str().unwrap()).unwrap();
            assert_eq!(
                node, *expected,
                "{name} must retain its own admitted native reference"
            );
        }
        assert_eq!(
            source_names[index]["binding"],
            selected_names[index]["binding"]
        );
        if name == "fld_value" {
            let binding: Value =
                serde_json::from_str(selected_names[index]["binding"].as_str().unwrap()).unwrap();
            assert_eq!(binding, model);
        }
    }
}

/// A selected checked package reaches FSL with its caller-supplied source
/// identity and version after the source is removed, preserving the source
/// export's counter semantics, obligation records and target bytes.
#[test]
#[trace("TC-3019", "FR-1039-AC-4")]
fn export_selected_package_preserves_counter_semantics_and_writes_target() {
    let directory = folder();
    let model_source = json!({"file": "counter.json", "authority": "test",
        "identity": "example/counter", "document": "ExportCounter", "formal_revision": 1});
    let program_source = json!({"file": "counter.fsl", "authority": "test",
        "identity": "example/counter-unit", "document": "ExportCounter", "formal_revision": 7});
    let models = json!([{"format": "semantic-ir/2.0.0", "source": model_source}]);
    let export = json!({"profile": "filament.output.tla2/v1", "obligations": ["Bounded"],
        "initial": "counter-0.json", "minimum_fidelity": "any"});

    let compile_request = json!({"format": "native-compile/1", "request": {
        "models": models,
        "program": {"source": program_source},
        "output": {"package": "checked-package.json"},
        "handoff": {"directory": "handoff", "package_name": "counter-unit",
            "native_join": {"work_units": 1_u64 << 40, "storage": 1_u64 << 24}}}});
    std::fs::write(
        directory.path().join("compile.json"),
        compile_request.to_string(),
    )
    .expect("the compile request");
    let compiled = fsl(
        directory.path(),
        &["compile", "compile.json", "--format", "json"],
    );
    assert_eq!(compiled.status.code(), Some(0), "{compiled:?}");
    assert!(directory.path().join("checked-package.json").is_file());
    let equality_path = directory.path().join("handoff/equality.json");
    let equality_bytes = std::fs::read(&equality_path).expect("producer-emitted equality rows");
    assert_eq!(
        std::fs::read(directory.path().join("checked-package.json")).unwrap(),
        std::fs::read(directory.path().join("handoff/checked-package.json")).unwrap(),
        "the selected package and equality rows must share the same producer emission"
    );
    let package_bytes = std::fs::read(directory.path().join("checked-package.json")).unwrap();
    let source_request = json!({"format": "native-run/1", "request": {
        "models": models, "program": {"source": program_source}, "export": export}});
    std::fs::write(
        directory.path().join("source-export.json"),
        source_request.to_string(),
    )
    .unwrap();
    let source_output = fsl(
        directory.path(),
        &["export", "source-export.json", "--format", "json"],
    );
    assert_eq!(source_output.status.code(), Some(0), "{source_output:?}");
    let source_document = document(&source_output);
    std::fs::remove_file(directory.path().join("counter.fsl")).expect("remove source file");

    let source_package = json!({"identity": "example/counter-unit", "version": "7"});
    let export_request = json!({"format": "native-run/1", "request": {
        "models": models,
        "package": {"file": "checked-package.json", "equality": "handoff/equality.json",
            "source_package": source_package},
        "export": export}});
    std::fs::write(
        directory.path().join("selected-export.json"),
        export_request.to_string(),
    )
    .expect("the selected-package export request");
    let output = fsl(
        directory.path(),
        &[
            "export",
            "selected-export.json",
            "--output",
            "selected.tla",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let value = document(&output);
    assert_eq!(value["operation"], "export", "{value}");
    assert_eq!(value["category"], "success", "{value}");
    assert_eq!(value["last_stage"], "S4", "{value}");
    assert_eq!(value["diagnostics"], json!([]), "{value}");
    assert_eq!(value["result"]["kind"], "exported", "{value}");
    let exported = &value["result"]["export"];
    assert_eq!(exported["status"], "assembled", "{value}");
    assert_eq!(exported["target_family"], "tla", "{value}");
    assert_eq!(
        exported["mapping_profile_id"], "filament.output.tla2/v1",
        "{value}"
    );
    assert_eq!(
        exported["package"]["source_package"], source_package,
        "{value}"
    );
    assert_eq!(
        exported["module"], "M_example_u2F_counter_u2D_unit",
        "{value}"
    );
    let source_export = &source_document["result"]["export"];
    assert_counter_export_correspondence(source_export, exported, &package_bytes);
    let records = exported["records"].as_array().expect("obligation records");
    assert_eq!(records.len(), 1, "{value}");
    assert_eq!(records[0]["obligation"], "Bounded", "{value}");
    assert_eq!(records[0]["disposition"], "preserved", "{value}");
    assert_eq!(records[0]["conditions"], json!([]), "{value}");
    assert_eq!(records[0]["causes"], json!([]), "{value}");
    let text = exported["target_text"]
        .as_str()
        .expect("assembled target text");
    for semantic in [
        "---- MODULE M_example_u2F_counter_u2D_unit ----",
        "VARIABLES",
        "TypeOK ==",
        "Init ==",
        "op_Counter_inc",
        "Next ==",
        "Spec ==",
        "c_Bounded",
        "fld_value",
    ] {
        assert!(text.contains(semantic), "missing {semantic}: {text}");
    }
    let names = exported["name_map"].as_array().expect("native name map");
    assert!(!names.is_empty(), "{value}");
    for entry in names {
        let name = entry["name"].as_str().expect("target identifier");
        assert!(text.contains(name), "{entry}: {text}");
        assert!(
            !entry["node"].as_str().expect("native node").is_empty(),
            "{entry}"
        );
    }
    let written = std::fs::read(directory.path().join("selected.tla")).unwrap();
    assert_eq!(written, text.as_bytes());
    assert_eq!(
        exported["output"]["bytes_written"],
        json!(written.len()),
        "{value}"
    );
    assert_eq!(exported["output"]["path"], "selected.tla", "{value}");
    assert_eq!(std::fs::read(equality_path).unwrap(), equality_bytes);
    assert_eq!(
        std::fs::read(directory.path().join("checked-package.json")).unwrap(),
        package_bytes
    );
    assert!(!directory.path().join("counter.fsl").exists());
}

/// A selected value-only package preserves the caller's source reference and
/// writes its target text through the CLI.
#[test]
#[trace("TC-045", "FR-027-AC-1")]
#[trace("TC-3019", "FR-1039-AC-4")]
fn export_selected_package_value_path_preserves_source_reference_and_writes_target() {
    let directory = folder();
    std::fs::write(
        directory.path().join("counter.fsl"),
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"filament.value.complete/v1\";\n\
         function one using v(): Integer pure { 1 }\n",
    )
    .expect("the value-only source");

    let compile_request = json!({"format": "native-compile/1", "request": {
        "models": [],
        "program": {"source": {
            "file": "counter.fsl", "authority": "test", "identity": "test:value-only-unit",
            "document": "ExportCounter", "formal_revision": 7}},
        "output": {"package": "checked-package.json"}}});
    std::fs::write(
        directory.path().join("compile.json"),
        compile_request.to_string(),
    )
    .expect("the compile request");
    let compiled = fsl(
        directory.path(),
        &["compile", "compile.json", "--format", "json"],
    );
    assert_eq!(compiled.status.code(), Some(0), "{compiled:?}");
    assert!(directory.path().join("checked-package.json").is_file());
    std::fs::remove_file(directory.path().join("counter.fsl")).expect("remove source file");

    let source_package = json!({"identity": "test:value-only-unit", "version": "7"});
    let export_request = json!({"format": "native-run/1", "request": {
        "models": [],
        "package": {"file": "checked-package.json", "source_package": source_package},
        "export": {"profile": "filament.output.tla2/v1", "obligations": ["not-a-claim"],
            "minimum_fidelity": "any"}}});
    std::fs::write(
        directory.path().join("selected-export.json"),
        export_request.to_string(),
    )
    .expect("the selected-package export request");
    let output = fsl(
        directory.path(),
        &[
            "export",
            "selected-export.json",
            "--output",
            "selected.tla",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let value = document(&output);
    assert_eq!(value["operation"], "export", "{value}");
    assert_eq!(value["category"], "success", "{value}");
    assert_eq!(
        value["result"]["export"]["package"]["source_package"], source_package,
        "FSL must retain the selected package's caller-supplied source reference"
    );
    let text = value["result"]["export"]["target_text"]
        .as_str()
        .expect("target text from selected package");
    assert_eq!(
        std::fs::read(directory.path().join("selected.tla")).expect("the target file"),
        text.as_bytes()
    );
}

/// FR-027-AC-2: an existing path, a doubled destination and a repeated
/// `--output` are each refused, exit 20, with no file written or changed.
#[test]
#[trace("TC-045", "FR-027-AC-2")]
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

/// FR-027-AC-4 / FR-865-AC-7: canonical text rendering retains the export
/// limit diagnostic when the outcome is incomplete.
#[test]
#[trace("TC-045", "FR-027-AC-4")]
#[trace("TC-2996", "FR-865-AC-7")]
fn export_text_names_output_byte_limit_for_incomplete_export() {
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
        ],
    );
    assert_eq!(output.status.code(), Some(22), "{output:?}");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("export.output_bytes"), "{text}");
    assert!(!directory.path().join("m.tla").exists());
}

/// FR-027-AC-5: `--evidence` reaches FSL's evidence intake on `export`
/// only, and refuses `--output` and a request operand beside it.
#[test]
#[trace("TC-045", "FR-027-AC-5")]
#[trace("FSpec-TC-2020", "FSpec-FR-817-AC-7")]
fn export_evidence_routes_to_the_intake_and_refuses_other_arguments() {
    let directory = folder();
    request(directory.path(), None);
    let exported = fsl(
        directory.path(),
        &["export", "request.json", "--format", "json"],
    );
    assert_eq!(exported.status.code(), Some(0), "{exported:?}");
    let package = document(&exported)["result"]["export"]["package"].clone();
    assert!(package.is_object(), "{exported:?}");
    let evidence = json!({"format": "native-run/1", "request": {"models": [],
        "export_evidence": {"package": package, "obligation": "Bounded", "tool": "tlc",
            "artifact": "counter.cfg"}}});
    std::fs::write(directory.path().join("e.json"), evidence.to_string()).unwrap();
    let output = fsl(
        directory.path(),
        &["export", "--evidence", "e.json", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(21), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let value = document(&output);
    assert_eq!(value["operation"], "export", "{value}");
    assert_eq!(value["category"], "unsupported", "{value}");
    assert!(value["last_stage"].is_null(), "{value}");
    assert!(value["result"].is_null(), "{value}");
    assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1, "{value}");
    let diagnostic = &value["diagnostics"][0];
    assert_eq!(diagnostic["code"], "unimplemented_capability", "{value}");
    assert_eq!(diagnostic["cause"], "unimplemented-meaning", "{value}");
    assert_eq!(
        diagnostic["details"]["capability"], "export_evidence",
        "{value}"
    );
    assert!(!directory.path().join("counter.cfg").exists());

    let mut malformed = evidence.clone();
    malformed["request"]["export_evidence"]["tool"] = json!("observer");
    std::fs::write(
        directory.path().join("malformed-evidence.json"),
        malformed.to_string(),
    )
    .unwrap();
    let output = fsl(
        directory.path(),
        &[
            "export",
            "--evidence",
            "malformed-evidence.json",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let value = document(&output);
    assert_eq!(
        value["diagnostics"][0]["code"], "invalid_runtime_input",
        "{value}"
    );
    assert_eq!(value["diagnostics"][0]["cause"], "invalid-value", "{value}");
    assert_eq!(
        value["diagnostics"][0]["details"]["path"], "/export_evidence/tool",
        "{value}"
    );
    assert!(value["result"].is_null(), "{value}");
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
