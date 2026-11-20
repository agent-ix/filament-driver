// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for the initial command shell.
#![allow(
    clippy::expect_used,
    reason = "process launch failures should fail this integration test"
)]

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use filament_exact::Cancel;
use filament_semantic_value::catalog::Category;
use fsl_command::command::{self, CheckInputs};
use fsl_cst::format::{FormatLimits, FormatRefusal, format_with_limits};
use fsl_foundation::{
    SettingLimits, Source, SourceIdentity,
    diagnostic::{Code, Staged},
};
use fsl_replay::{
    Operation, OutcomeDiagnostic, OutcomeDocument,
    spine::{
        DependencyInput, PackageLimits, ParseRequest, SpineLimits, check, package, parse, select,
    },
};
use ix_cli_kit::version::Agreement;
use ix_trace_rs::trace;

#[path = "common/requests.rs"]
mod requests;

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.fsl");
const INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/parse-invalid.fsl"
);
const CHECK_INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/check-invalid.fsl"
);
const CHECK_MODEL_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/check-with-model.fsl"
);
const CHECK_LIBRARY_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/check-with-library.fsl"
);
const MODEL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/population-model.semantic-ir.json"
);

struct TemporaryDirectory(std::path::PathBuf);

impl TemporaryDirectory {
    fn new(label: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("fsl-cli-{label}-{}-{unique}", std::process::id()));
        std::fs::create_dir(&path).expect("create isolated test directory");
        Self(path)
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
const COUNTER_MODEL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analyze-counter.semantic-ir.json"
);
const PARENT_ORDER_PACKAGE: &str = "test/parent-order";
const PARENT_ORDER_POPULATION: &str = "ix://test/parent-order/config_history";
const PARENT_ORDER_TYPE: &str = "ix://test/parent-order/ConfigVersion";
const PARENT_ORDER_PROGRAM: &str = concat!(
    "language \"ix:native\" edition \"1-draft\";\n",
    "profile v = \"filament.model.complete/v1\";\n",
    "model Config = \"test/parent-order\";\n",
    "invariant ParentOrder using v on Config::ConfigVersion at current { ",
    "present(self.parent) implies ",
    "deref(value(self.parent)).versionNumber < self.versionNumber }\n"
);

fn parent_order_model() -> serde_json::Value {
    serde_json::json!({
        "contractVersion": "2.0.0",
        "canonicalization": {"algorithm": "rfc8785-v1"},
        "source": {
            "identity": "ix://test/parent-order/spec",
            "version": "1.0.0",
            "dialect": "spec-bundle"
        },
        "package": {
            "identity": PARENT_ORDER_PACKAGE,
            "version": "1.0.0",
            "mappingVersions": [],
            "profileVersions": []
        },
        "occurrences": [],
        "extensions": [],
        "constructs": [
            {
                "kind": {"module": PARENT_ORDER_PACKAGE, "name": "object_type"},
                "moduleVersion": "1.0.0",
                "construct": {
                    "identity": "none",
                    "shape": "record",
                    "members": {},
                    "meaning": "filament.meaning.model.object-type/v1"
                }
            },
            {
                "kind": {"module": PARENT_ORDER_PACKAGE, "name": "population"},
                "moduleVersion": "1.0.0",
                "construct": {
                    "identity": "none",
                    "shape": "record",
                    "members": {},
                    "meaning": "filament.meaning.model.population/v1"
                }
            },
            {
                "kind": {"module": PARENT_ORDER_PACKAGE, "name": "value_type"},
                "moduleVersion": "1.0.0",
                "construct": {
                    "identity": "none",
                    "shape": "record",
                    "members": {},
                    "meaning": "filament.meaning.model.value-type/v1"
                }
            }
        ],
        "types": [
            {
                "identity": "ix://test/parent-order/VersionNumber",
                "displayName": "VersionNumber",
                "kind": {"module": PARENT_ORDER_PACKAGE, "name": "value_type"},
                "roles": [],
                "origin": {"generated": {
                    "generatorIdentity": "ix://test/parent-order/VersionNumber",
                    "generatorVersion": "1.0.0",
                    "inputIdentities": ["ix://test/parent-order/VersionNumber"]
                }},
                "extensions": [],
                "unknownPolicy": "reject",
                "scalar": "integer",
                "constraints": []
            },
            {
                "identity": PARENT_ORDER_TYPE,
                "displayName": "ConfigVersion",
                "kind": {"module": PARENT_ORDER_PACKAGE, "name": "object_type"},
                "roles": [],
                "origin": {"generated": {
                    "generatorIdentity": PARENT_ORDER_TYPE,
                    "generatorVersion": "1.0.0",
                    "inputIdentities": [PARENT_ORDER_TYPE]
                }},
                "constraints": [],
                "extensions": [],
                "unknownPolicy": "reject",
                "supertypes": [],
                "fields": [
                    {
                        "identity": "ix://test/parent-order/ConfigVersion/versionNumber",
                        "name": "versionNumber",
                        "typeRef": "ix://test/parent-order/VersionNumber",
                        "presence": "required",
                        "nullable": false,
                        "defaultKind": "none",
                        "multiplicity": {
                            "lower": 1,
                            "upper": 1,
                            "ordered": false,
                            "unique": true
                        },
                        "origin": {"generated": {
                            "generatorIdentity": "ix://test/parent-order/ConfigVersion/versionNumber",
                            "generatorVersion": "1.0.0",
                            "inputIdentities": ["ix://test/parent-order/ConfigVersion/versionNumber"]
                        }}
                    },
                    {
                        "identity": "ix://test/parent-order/ConfigVersion/parent",
                        "name": "parent",
                        "typeRef": PARENT_ORDER_TYPE,
                        "presence": "optional",
                        "nullable": false,
                        "defaultKind": "none",
                        "multiplicity": {
                            "lower": 1,
                            "upper": 1,
                            "ordered": false,
                            "unique": true
                        },
                        "origin": {"generated": {
                            "generatorIdentity": "ix://test/parent-order/ConfigVersion/parent",
                            "generatorVersion": "1.0.0",
                            "inputIdentities": ["ix://test/parent-order/ConfigVersion/parent"]
                        }}
                    }
                ],
                "operations": []
            }
        ],
        "populations": [{
            "identity": PARENT_ORDER_POPULATION,
            "displayName": "config_history",
            "kind": {"module": PARENT_ORDER_PACKAGE, "name": "population"},
            "members": [PARENT_ORDER_TYPE],
            "extent": "closed",
            "origin": {"generated": {
                "generatorIdentity": PARENT_ORDER_POPULATION,
                "generatorVersion": "1.0.0",
                "inputIdentities": [PARENT_ORDER_POPULATION]
            }}
        }]
    })
}

fn write_parent_order_model(directory: &Path, filename: &str) {
    std::fs::write(
        directory.join(filename),
        serde_json::to_vec(&parent_order_model()).expect("encode the test-local ParentOrder model"),
    )
    .expect("write the test-local ParentOrder model");
}

fn write_counter_clause_request(
    directory: &Path,
    expression: &str,
    current_value: u64,
    accounting_work_units: Option<u64>,
) -> PathBuf {
    std::fs::copy(COUNTER_MODEL, directory.join("counter-model.json"))
        .expect("write the driver-owned Counter model");
    std::fs::write(
        directory.join("counter.fsl"),
        format!(
            "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"filament.value.complete/v1\";\n\
             model M = \"test/counter\";\n\
             invariant CounterRule using v on M::Counter at current {{ {expression} }}\n"
        ),
    )
    .expect("write the focused Counter rule");
    std::fs::write(
        directory.join("current.json"),
        serde_json::json!({
            "format": "filament.state.snapshot/v1",
            "identity": {"authority": "test", "identity": "counter/current"},
            "model": {"identity": "test/counter"},
            "observation": "current",
            "anchor": {"kind": "handler", "name": "validate"},
            "populations": [{
                "population": "ix://test/counter/counters",
                "complete": true,
                "objects": [{
                    "key": "a",
                    "type": "ix://test/counter/Counter",
                    "fields": {"value": {"integer": current_value.to_string()}}
                }]
            }]
        })
        .to_string(),
    )
    .expect("write the focused Counter snapshot");

    let mut clause = serde_json::json!({
        "selection": {"clause": {
            "name": "CounterRule",
            "observation": "current",
            "self": {"population": "ix://test/counter/counters", "key": "a"},
            "anchor": {"kind": "handler", "name": "validate"}
        }},
        "observations": {"current": "current.json"}
    });
    if let Some(work_units) = accounting_work_units {
        clause["accounting"] = serde_json::json!({"work_units": work_units});
    }
    let request = serde_json::json!({
        "format": "native-run/1",
        "request": {
            "models": [{
                "format": "semantic-ir/2.0.0",
                "source": {
                    "file": "counter-model.json",
                    "authority": "test",
                    "identity": "counter-model",
                    "document": "Counter",
                    "formal_revision": 1
                }
            }],
            "program": {"source": {
                "file": "counter.fsl",
                "authority": "test",
                "identity": "counter-rules",
                "document": "CounterRules",
                "formal_revision": 1
            }},
            "clause": clause
        }
    });
    let path = directory.join("request.json");
    std::fs::write(&path, request.to_string()).expect("write Counter run request");
    path
}

fn parse_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read parse fixture");
    let source = SourceIdentity::new("issuer", "document");
    let limits = SpineLimits::default();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
        },
        limits.source,
        &Cancel::new(),
    );
    OutcomeDocument::from_parse(&result)
        .to_bytes()
        .expect("FSL serializes its parse outcome")
}

fn check_outcome_bytes(path: &'static str) -> Vec<u8> {
    command::check_outcome(Path::new(path), &CheckInputs::default())
        .document()
        .to_bytes()
        .expect("FSL serializes its check outcome")
}

fn package_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read package fixture");
    let source = SourceIdentity::new("filament", path);
    let limits = SpineLimits::default();
    let dependencies = DependencyInput::default();
    let packages = std::collections::BTreeMap::new();
    let cancel = Cancel::new();
    let document = match parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
        },
        limits.source,
        &cancel,
    ) {
        Err(failure) => OutcomeDocument::from_package(&Err(failure)),
        Ok(parsed) => match select(parsed.value(), &packages, limits.model, &cancel) {
            Err(failure) => OutcomeDocument::from_package(&Err(failure)),
            Ok(models) => match check(
                parsed.value(),
                path.to_owned(),
                models.value(),
                &dependencies,
                limits,
                &cancel,
            ) {
                Err(failure) => OutcomeDocument::from_package(&Err(failure)),
                Ok(checked) => OutcomeDocument::from_package(&package(
                    checked.value(),
                    PackageLimits::default(),
                    &cancel,
                )),
            },
        },
    };
    document
        .to_bytes()
        .expect("FSL serializes its package outcome")
}

fn format_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read format fixture");
    let source = SourceIdentity::new("issuer", "document");
    let limits = fsl_cst::Limits::default();
    let cancel = Cancel::new();
    let result = fsl_cst::parse_with_cancel(source, path, &bytes, limits, &cancel)
        .map_err(FormatRefusal::DiagnosedSource)
        .and_then(|parsed| format_with_limits(&parsed, FormatLimits::default()).map(Staged::new));
    OutcomeDocument::from_format(&result)
        .to_bytes()
        .expect("FSL serializes its format outcome")
}

fn argument_refusal_bytes(operation: Operation, message: &str) -> Vec<u8> {
    OutcomeDocument::new(operation, None, Code::InvalidRequest.category())
        .with_diagnostics(vec![OutcomeDiagnostic::new(
            None,
            Code::InvalidRequest.as_str(),
            None,
            message,
        )])
        .to_bytes()
        .expect("FSL serializes the argument refusal")
}

fn assert_fsl_document(cli_stdout: &[u8], fsl_bytes: &[u8]) {
    assert_eq!(
        cli_stdout.iter().filter(|&&byte| byte == b'\n').count(),
        1,
        "machine output has exactly one trailing line terminator"
    );
    let cli_document = cli_stdout
        .strip_suffix(b"\n")
        .expect("machine output ends with one line terminator");
    let cli_value: serde_json::Value =
        serde_json::from_slice(cli_document).expect("CLI emits one JSON document");
    let fsl_value: serde_json::Value =
        serde_json::from_slice(fsl_bytes).expect("FSL bytes encode one JSON document");
    assert_eq!(cli_value, fsl_value);
}

fn deep_parenthesis_source(depth: usize) -> String {
    let expression = format!("{}1{}", "(".repeat(depth), ")".repeat(depth));
    format!(
        concat!(
            "language \"ix:native\" edition \"1-draft\";\n",
            "profile v = \"filament.value.complete/v1\";\n",
            "function deep using v(): Integer pure {{ {expression} }}\n"
        ),
        expression = expression
    )
}

fn write_deep_source(directory: &TemporaryDirectory, depth: usize) -> PathBuf {
    let path = directory.0.join("deep.fsl");
    std::fs::write(&path, deep_parenthesis_source(depth)).expect("write deep source fixture");
    path
}

/// Trace: IR-607, one FSL exit function for unknown verbs.
#[test]
fn unknown_verb_uses_fsl_refusal_code() {
    let output = Command::new(FSL)
        .arg("not-an-fsl-verb")
        .output()
        .expect("run unknown command");
    assert_eq!(output.status.code(), Some(20));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check dispatches through FSL's typed check operation.
#[test]
fn check_emits_fsl_check_outcome_document() {
    let output = Command::new(FSL)
        .args(["check", SOURCE, "--format", "json"])
        .output()
        .expect("run check command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_fsl_document(&output.stdout, &check_outcome_bytes(SOURCE));
}

/// Trace: TC-1743, FR-662-AC-1, FSpec-TC-1243, FSpec-FR-563-AC-1, FSpec-FR-563-AC-2.
/// Provenance: QSL-2019.
#[test]
#[trace(
    "TC-1743",
    "FR-662-AC-1",
    "FSpec-TC-1243",
    "FSpec-FR-563-AC-1",
    "FSpec-FR-563-AC-2"
)]
fn check_markdown_checks_each_named_native_fence() {
    let directory = TemporaryDirectory::new("check-markdown");
    let healthy = std::fs::read_to_string(SOURCE).expect("read native source fixture");
    let broken = format!("{healthy}function Bad using v (x: Integer): Boolean pure {{ x }}\n");
    let document = format!(
        "### Healthy\n\n```ix:native\n{healthy}```\n\n### Broken\n\n```ix:native\n{broken}```\n"
    );
    std::fs::write(directory.0.join("spec.md"), &document).expect("write Markdown fixture");
    let output = Command::new(FSL)
        .current_dir(&directory.0)
        .args([
            "check",
            "--markdown",
            "agent-ix",
            "doc",
            "spec.md",
            "--format",
            "json",
        ])
        .output()
        .expect("run Markdown check");
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let outcome: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(outcome["operation"], "check", "{outcome}");
    assert_eq!(outcome["category"], "refusal", "{outcome}");
    let items = outcome["items"].as_array().expect("fence result items");
    assert_eq!(items.len(), 2, "{outcome}");
    assert_eq!(items[0]["category"], "success", "{outcome}");
    assert_eq!(items[0]["unit"]["document"]["authority"], "agent-ix");
    assert_eq!(items[0]["unit"]["document"]["identity"], "doc");
    assert_eq!(items[0]["unit"]["path"], "spec.md");
    assert_eq!(items[0]["unit"]["clause"], "Healthy");
    assert_eq!(items[0]["unit"]["body"]["identity"], "doc#Healthy");
    assert_eq!(items[0]["diagnostics"], serde_json::json!([]));

    let broken_item = &items[1];
    assert_eq!(broken_item["category"], "refusal", "{outcome}");
    let unit = &broken_item["diagnostics"][0]["unit"];
    assert_eq!(unit["document"]["authority"], "agent-ix");
    assert_eq!(unit["document"]["identity"], "doc");
    assert_eq!(unit["path"], "spec.md");
    assert_eq!(unit["clause"], "Broken");
    assert_eq!(unit["body"]["authority"], "agent-ix");
    assert_eq!(unit["body"]["identity"], "doc#Broken");

    let diagnostic = &broken_item["diagnostics"][0];
    assert!(!diagnostic["cause"].is_null(), "{outcome}");
    assert_eq!(diagnostic["code"], "ill_typed", "{outcome}");
    let body_start = broken.rfind(" x }").expect("ill-typed expression") + 1;
    assert_eq!(diagnostic["locus"]["kind"], "region", "{outcome}");
    assert_eq!(diagnostic["locus"]["source"]["identity"], "doc#Broken");
    assert_eq!(diagnostic["locus"]["start"], body_start);
    assert_eq!(diagnostic["locus"]["end"], body_start + 1);

    let original_start = document.rfind(" x }").expect("ill-typed expression") + 1;
    let prefix = &document[..original_start];
    let expected_line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let expected_column = prefix
        .rsplit('\n')
        .next()
        .expect("last source line")
        .chars()
        .count()
        + 1;
    let original = &diagnostic["original_regions"][0];
    assert_eq!(original["kind"], "located", "{outcome}");
    assert_eq!(original["file"], "spec.md");
    assert_eq!(original["span"]["start"]["line"], expected_line);
    assert_eq!(original["span"]["start"]["column"], expected_column);

    let without_markdown = Command::new(FSL)
        .current_dir(&directory.0)
        .args(["check", "agent-ix", "doc", "spec.md", "--format", "json"])
        .output()
        .expect("run native check against Markdown bytes");
    assert_eq!(
        without_markdown.status.code(),
        Some(20),
        "{without_markdown:?}"
    );
    let plain_document: serde_json::Value =
        serde_json::from_slice(&without_markdown.stdout).expect("one native check outcome");
    assert_eq!(plain_document["operation"], "check");
    assert_eq!(plain_document["category"], "refusal");
    assert!(
        plain_document.to_string().contains("invalid_syntax"),
        "{plain_document}"
    );
}

/// Trace: FR-002-AC-3.
/// Provenance: QSL-2019, FND-001.
#[test]
#[trace("FR-002-AC-3")]
fn check_markdown_source_read_refusal_keeps_requested_identity() {
    let directory = TemporaryDirectory::new("check-markdown-source-refusal");
    let path = directory.0.join("spec.md");
    std::fs::write(&path, [0xff]).expect("write invalid UTF-8 source fixture");
    let output = Command::new(FSL)
        .current_dir(&directory.0)
        .args([
            "check",
            "--markdown",
            "agent-ix",
            "doc",
            "spec.md",
            "--format",
            "json",
        ])
        .output()
        .expect("run Markdown check on invalid UTF-8");
    assert_eq!(output.status.code(), Some(20), "{output:?}");

    let limits = SpineLimits::default();
    let refusal = Source::read_typed(
        SourceIdentity::new("agent-ix", "doc"),
        "spec.md",
        &[0xff],
        limits.source.source_bytes,
    )
    .expect_err("invalid UTF-8 must be refused at source admission");
    let diagnostic = fsl_cst::diagnostic::source_refusal(*refusal, limits.source.source_bytes);
    let expected = OutcomeDocument::new(Operation::Check, None, diagnostic.code.category())
        .with_diagnostics(vec![OutcomeDiagnostic::from_source_diagnostic(&diagnostic)])
        .to_bytes()
        .expect("serialize typed source refusal");
    assert_fsl_document(&output.stdout, &expected);

    let outcome: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(outcome["category"], "refusal", "{outcome}");
    assert_eq!(outcome["diagnostics"][0]["code"], "invalid_utf8");
    assert_eq!(
        outcome["diagnostics"][0]["locus"]["source"]["authority"],
        "agent-ix"
    );
    assert_eq!(
        outcome["diagnostics"][0]["locus"]["source"]["identity"],
        "doc"
    );
}

/// Trace: FSpec-TC-1243, FSpec-FR-563-AC-2.
/// Provenance: QSL-2019.
#[test]
#[trace("FSpec-TC-1243", "FSpec-FR-563-AC-2")]
fn run_executes_a_selected_markdown_clause_in_the_default_build() {
    let directory = TemporaryDirectory::new("run-markdown-extraction");
    let markdown = format!(
        "# Doc\n\n## Invariants\n\n### Healthy\n\n```ix:native\n{PARENT_ORDER_PROGRAM}```\n"
    );
    std::fs::write(directory.0.join("spec.md"), markdown).expect("write Markdown fixture");
    write_parent_order_model(&directory.0, "spine-model.semantic-ir.json");
    std::fs::write(
        directory.0.join("spine-model.semantic-ir.json.license"),
        "SPDX-License-Identifier: AGPL-3.0-only\n",
    )
    .expect("write the domain package license metadata");
    std::fs::write(
        directory.0.join("current.json"),
        serde_json::json!({
            "format": "filament.state.snapshot/v1",
            "identity": {"authority": "agent-ix", "identity": "current"},
            "model": {"identity": PARENT_ORDER_PACKAGE},
            "observation": "current",
            "anchor": {"kind": "handler", "name": "validate"},
            "populations": [{
                "population": PARENT_ORDER_POPULATION,
                "complete": true,
                "objects": [
                    {"key": "root", "type": PARENT_ORDER_TYPE,
                     "fields": {"versionNumber": {"integer": "1"}, "parent": {"absent": {}}}},
                    {"key": "child", "type": PARENT_ORDER_TYPE,
                     "fields": {"versionNumber": {"integer": "2"},
                      "parent": {"present": {"reference": {
                          "population": PARENT_ORDER_POPULATION, "key": "root"
                      }}}}}
                ]
            }]
        })
        .to_string(),
    )
    .expect("write observation fixture");
    let request = serde_json::json!({
        "format": "native-run/1",
        "request": {
            "models": [{
                "format": "semantic-ir/2.0.0",
                "source": {
                    "file": "spine-model.semantic-ir.json",
                    "authority": "agent-ix",
                    "identity": "test:fr-664:spine-model.semantic-ir.json",
                    "document": "SpineModel",
                    "formal_revision": 1
                }
            }],
            "program": {
                "source": {
                    "file": "spec.md",
                    "authority": "agent-ix",
                    "identity": "ix://agent-ix/spec",
                    "document": "Doc",
                    "formal_revision": 1
                },
                "extraction": {
                    "body": {"authority": "agent-ix", "identity": "body", "document": "Body"}
                },
            "clauses": [{"clause": "Healthy", "package": PARENT_ORDER_PACKAGE}]
            },
            "clause": {
                "selection": {
                    "clause": {
                        "name": "ParentOrder",
                        "observation": "current",
                        "self": {"population": PARENT_ORDER_POPULATION, "key": "child"},
                        "anchor": {"kind": "handler", "name": "validate"}
                    }
                },
                "observations": {"current": "current.json"}
            }
        }
    });
    let request_path = directory.0.join("request.json");
    std::fs::write(&request_path, request.to_string()).expect("write run request");

    let output = Command::new(FSL)
        .current_dir(&directory.0)
        .arg("run")
        .arg("request.json")
        .args(["--format", "json"])
        .output()
        .expect("run extracted clause");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let outcome: serde_json::Value = serde_json::from_slice(&output.stdout).expect("outcome JSON");
    assert_eq!(outcome["format"], "filament-outcome/1", "{outcome}");
    assert_eq!(outcome["operation"], "monitor", "{outcome}");
    assert_eq!(outcome["category"], "success", "{outcome}");
    assert_eq!(
        outcome["items"].as_array().map(Vec::len),
        Some(1),
        "{outcome}"
    );
    assert!(outcome.to_string().contains("ParentOrder"), "{outcome}");
}

/// Trace: TC-1743, FR-662-AC-2.
/// Provenance: QSL-2019.
#[test]
#[trace("TC-1743", "FR-662-AC-2")]
fn check_markdown_refuses_doubled_and_headless_fences() {
    let directory = TemporaryDirectory::new("check-markdown-refusals");
    let healthy = std::fs::read_to_string(SOURCE).expect("read native source fixture");
    let document = format!(
        "```ix:native\n{healthy}```\n\n### Twice\n\n```ix:native\n{healthy}```\n\n```ix:native\n{healthy}```\n\n### Stable\n\n```ix:native\n{healthy}```\n"
    );
    std::fs::write(directory.0.join("spec.md"), &document).expect("write Markdown fixture");

    let run_check = || {
        Command::new(FSL)
            .current_dir(&directory.0)
            .args([
                "check",
                "--markdown",
                "agent-ix",
                "doc",
                "spec.md",
                "--format",
                "json",
            ])
            .output()
            .expect("run Markdown check")
    };
    let output = run_check();
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let repeated = run_check();
    assert_eq!(repeated.status.code(), Some(20), "{repeated:?}");
    assert_eq!(
        output.stdout, repeated.stdout,
        "fence report is deterministic"
    );

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(report["operation"], "check", "{report}");
    assert_eq!(report["category"], "refusal", "{report}");
    let items = report["items"].as_array().expect("fence result items");
    assert_eq!(items.len(), 4, "{report}");

    let headless = &items[0];
    assert_eq!(headless["category"], "refusal", "{headless}");
    assert_eq!(headless["unit"]["clause"], serde_json::Value::Null);
    assert_eq!(headless["diagnostics"][0]["cause"], "missing-name");
    assert_eq!(headless["diagnostics"][0]["code"], "missing_declaration");
    assert_eq!(headless["first_line"], 2);

    for doubled in [&items[1], &items[2]] {
        assert_eq!(doubled["category"], "refusal", "{doubled}");
        assert_eq!(doubled["unit"]["clause"], "Twice");
        assert_eq!(
            doubled["diagnostics"][0]["cause"],
            "extraction-clause-count"
        );
        assert_eq!(doubled["diagnostics"][0]["code"], "extraction-clause-count");
        assert_eq!(doubled["details"]["actual"], 2);
    }

    let stable = &items[3];
    assert_eq!(stable["category"], "success", "{stable}");
    assert_eq!(stable["unit"]["clause"], "Stable");
    assert_eq!(stable["unit"]["body"]["identity"], "doc#Stable");
    assert_eq!(stable["diagnostics"], serde_json::json!([]));
}

/// Trace: QSL-1972 / FR-866-AC-1; check supplies the selected domain model to FSL.
#[test]
fn check_with_model_checks_importing_unit() {
    let directory = TemporaryDirectory::new("check-model");
    let model = directory.0.join("model.json");
    std::fs::copy(MODEL, &model).expect("copy model fixture");
    let output = Command::new(FSL)
        .arg("check")
        .arg(CHECK_MODEL_SOURCE)
        .arg("--model")
        .arg(format!("driver/ir608-model={}", model.display()))
        .arg("--dependent-bytes")
        .arg("8192")
        .arg("--format")
        .arg("json")
        .output()
        .expect("check unit with model");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a JSON outcome");
    assert_eq!(document["operation"], "check");
    assert_eq!(document["category"], "success");
}

/// Trace: QSL-1972 / FR-866-AC-2; missing models preserve the selected identity and supplying flag.
#[test]
fn check_without_model_names_supplying_option() {
    let output = Command::new(FSL)
        .args(["check", CHECK_MODEL_SOURCE, "--format", "json"])
        .output()
        .expect("check importing unit without its model");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a refusal outcome");
    let serialized = document.to_string();
    assert!(serialized.contains("driver/ir608-model"), "{serialized}");
    assert!(serialized.contains("--model"), "{serialized}");
}

/// Trace: QSL-1972 / FR-866-AC-4; a model on a source with no model selection is refused.
#[test]
fn check_model_option_misuse_refused() {
    let output = Command::new(FSL)
        .arg("check")
        .arg(SOURCE)
        .arg("--model")
        .arg(format!("unused/model={MODEL}"))
        .arg("--format")
        .arg("json")
        .output()
        .expect("check source with unused model");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a refusal outcome");
    assert!(document.to_string().contains("--model"));
}

/// Trace: QSL-1972 / FR-866-AC-5; dependent-byte options bound supplied model files.
#[test]
fn check_dependent_byte_limit_is_applied() {
    let model_operand = format!("driver/ir608-model={MODEL}");
    let output = Command::new(FSL)
        .args([
            "check",
            CHECK_MODEL_SOURCE,
            "--model",
            &model_operand,
            "--dependent-bytes",
            "0",
            "--format",
            "json",
        ])
        .output()
        .expect("check unit with zero dependent-byte allowance");
    assert_eq!(output.status.code(), Some(22));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a refusal outcome");
    assert!(document.to_string().contains("dependent_bytes"));
}

/// Trace: QSL-1972 / FR-866-AC-5; each library path is offered for a source-selected identity.
#[test]
fn check_library_path_is_read_for_selected_import() {
    let directory = TemporaryDirectory::new("check-library");
    let missing_library = directory.0.join("library.native");
    let output = Command::new(FSL)
        .arg("check")
        .arg(CHECK_LIBRARY_SOURCE)
        .arg("--library")
        .arg(format!(
            "driver/check-library={}",
            missing_library.display()
        ))
        .arg("--format")
        .arg("json")
        .output()
        .expect("check unit with selected library path");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a refusal outcome");
    let serialized = document.to_string();
    assert!(
        serialized.contains(&missing_library.display().to_string()),
        "{serialized}"
    );
}

/// Trace: QSL-1972 / FR-866-AC-1; check links a supplied library selected by the source.
#[test]
fn check_with_library_checks_importing_unit() {
    let directory = TemporaryDirectory::new("check-library-success");
    let library = directory.0.join("library.native");
    std::fs::write(
        &library,
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"filament.value.complete/v1\";\n\
         predicate Ready using v (ready: Boolean): Boolean { ready }\n",
    )
    .expect("write library fixture");
    let output = Command::new(FSL)
        .arg("check")
        .arg(CHECK_LIBRARY_SOURCE)
        .arg("--library")
        .arg(format!("driver/check-library={}", library.display()))
        .arg("--format")
        .arg("json")
        .output()
        .expect("check unit with library");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
}

/// Trace: FR-312-AC-9, FR-312-AC-10, FR-491-AC-8, FR-287-AC-13.
/// Provenance: QSL-2015.
#[test]
#[trace("FR-312-AC-9", "FR-312-AC-10", "FR-491-AC-8", "FR-287-AC-13")]
fn observation_input_bytes_request_and_cli_limits_have_the_same_outcome() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "fsl-qsl-2015-observation-limit-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create isolated request directory");
    struct RemoveDirectory(std::path::PathBuf);
    impl Drop for RemoveDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = RemoveDirectory(directory.clone());

    std::fs::write(directory.join("parent-order.native"), PARENT_ORDER_PROGRAM)
        .expect("write the test-local ParentOrder rule");
    write_parent_order_model(&directory, "model.semantic-ir.json");

    let population = PARENT_ORDER_POPULATION;
    let object_type = PARENT_ORDER_TYPE;
    let observation = serde_json::json!({
        "format": "filament.state.snapshot/v1",
        "identity": {"authority": "issuer", "identity": "current"},
        "model": {"identity": PARENT_ORDER_PACKAGE},
        "observation": "current",
        "anchor": {"kind": "handler", "name": "validate"},
        "populations": [{
            "population": population,
            "complete": true,
            "objects": [
                {
                    "key": "root",
                    "type": object_type,
                    "fields": {
                        "versionNumber": {"integer": "1"},
                        "parent": {"absent": {}}
                    }
                },
                {
                    "key": "child",
                    "type": object_type,
                    "fields": {
                        "versionNumber": {"integer": "2"},
                        "parent": {"present": {"reference": {"population": population, "key": "root"}}}
                    }
                }
            ]
        }]
    });
    let observation_bytes = serde_json::to_vec(&observation).expect("encode observation");
    assert!(observation_bytes.len() > 10);
    std::fs::write(directory.join("current.json"), &observation_bytes)
        .expect("write over-limit observation");
    let limit = observation_bytes.len() - 1;

    let request = |request_limit: Option<usize>, observation_file: &str| {
        let mut clause = serde_json::json!({
            "selection": {"clause": {
                "name": "ParentOrder",
                "observation": "current",
                "self": {"population": population, "key": "child"},
                "anchor": {"kind": "handler", "name": "validate"}
            }},
            "observations": {"current": observation_file}
        });
        if let Some(limit) = request_limit {
            clause["observation_limits"] = serde_json::json!({"document_bytes": limit});
        }
        serde_json::json!({
            "format": "native-run/1",
            "request": {
                "models": [{
                    "format": "semantic-ir/2.0.0",
                    "source": {
                        "file": "model.semantic-ir.json",
                        "authority": "test",
                        "identity": "parent-order-model",
                        "document": "ParentOrderModel",
                        "formal_revision": 1
                    }
                }],
                "program": {"source": {
                    "file": "parent-order.native",
                    "authority": "test",
                    "identity": "parent-order-rule",
                    "document": "ParentOrderRule",
                    "formal_revision": 1
                }},
                "clause": clause
            }
        })
    };
    let request_path = directory.join("request-member.json");
    std::fs::write(
        &request_path,
        serde_json::to_vec(&request(Some(limit), "current.json"))
            .expect("encode request-member request"),
    )
    .expect("write request-member request");
    let flag_path = directory.join("flag.json");
    std::fs::write(
        &flag_path,
        serde_json::to_vec(&request(None, "current.json")).expect("encode CLI-limit request"),
    )
    .expect("write CLI-limit request");
    let at_limit_path = directory.join("at-limit.json");
    std::fs::write(
        &at_limit_path,
        serde_json::to_vec(&request(Some(observation_bytes.len()), "current.json"))
            .expect("encode at-limit request"),
    )
    .expect("write at-limit request");
    let duplicate_path = directory.join("duplicate-limit.json");
    std::fs::write(
        &duplicate_path,
        serde_json::to_vec(&request(Some(limit), "missing.json"))
            .expect("encode duplicate-limit request"),
    )
    .expect("write duplicate-limit request");

    let request_limited = Command::new(FSL)
        .arg("run")
        .arg(&request_path)
        .args(["--format", "json"])
        .output()
        .expect("run with the request observation limit");
    let cli_limited = Command::new(FSL)
        .arg("run")
        .arg(&flag_path)
        .args([
            "--limit",
            &format!("observation.input_bytes={limit}"),
            "--format",
            "json",
        ])
        .output()
        .expect("run with the CLI observation limit");
    assert_eq!(request_limited.status.code(), Some(22));
    assert_eq!(cli_limited.status.code(), Some(22));
    assert!(
        request_limited.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&request_limited.stderr)
    );
    assert!(
        cli_limited.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&cli_limited.stderr)
    );

    let request_document: serde_json::Value =
        serde_json::from_slice(&request_limited.stdout).expect("request limit outcome JSON");
    let cli_document: serde_json::Value =
        serde_json::from_slice(&cli_limited.stdout).expect("CLI limit outcome JSON");
    assert_eq!(request_document, cli_document);
    assert_eq!(request_document["category"], "incomplete");
    assert_eq!(request_document["items"].as_array().map(Vec::len), Some(1));
    assert!(
        request_document["diagnostics"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "the limit is carried by the result item, not a top-level diagnostic: {request_document}"
    );
    assert_eq!(
        request_document["items"][0]["cause"]["code"],
        "stage_limit_exceeded"
    );
    assert_eq!(
        request_document["items"][0]["cause"]["kind"],
        "input-bytes-exceeded"
    );

    let cause = &request_document["items"][0]["cause"];
    assert_eq!(cause["limit"], "input-bytes-exceeded", "{cause}");
    assert_eq!(cause["value"], limit, "{cause}");
    assert_eq!(cause["setting"], "observation.input_bytes", "{cause}");
    assert_eq!(cause["bound"], limit.to_string(), "{cause}");
    assert_eq!(
        cause["counter"],
        observation_bytes.len().to_string(),
        "{cause}"
    );

    let at_limit = Command::new(FSL)
        .arg("run")
        .arg(&at_limit_path)
        .args(["--format", "json"])
        .output()
        .expect("run at the observation byte limit");
    assert_eq!(at_limit.status.code(), Some(0), "{at_limit:?}");
    let at_limit_document: serde_json::Value =
        serde_json::from_slice(&at_limit.stdout).expect("at-limit outcome JSON");
    assert_eq!(
        at_limit_document["category"], "success",
        "{at_limit_document}"
    );

    let text = Command::new(FSL)
        .arg("run")
        .arg(&request_path)
        .args(["--format", "text"])
        .output()
        .expect("render the request limit hint");
    assert_eq!(text.status.code(), Some(22), "{text:?}");
    let rendered = String::from_utf8_lossy(&text.stdout);
    assert!(
        rendered.contains("--limit observation.input_bytes="),
        "{rendered}"
    );
    assert!(
        !rendered.contains("observation.document_bytes"),
        "{rendered}"
    );

    let duplicate = Command::new(FSL)
        .arg("run")
        .arg(&duplicate_path)
        .args([
            "--limit",
            &format!("observation.input_bytes={limit}"),
            "--format",
            "json",
        ])
        .output()
        .expect("run with duplicate request and CLI observation limits");
    assert_eq!(duplicate.status.code(), Some(20), "{duplicate:?}");
    let duplicate_document: serde_json::Value =
        serde_json::from_slice(&duplicate.stdout).expect("duplicate limit refusal JSON");
    assert_eq!(
        duplicate_document["category"], "refusal",
        "{duplicate_document}"
    );
    assert!(
        duplicate_document.to_string().contains("invalid-request"),
        "{duplicate_document}"
    );
    assert!(
        duplicate_document
            .to_string()
            .contains("observation_limits"),
        "{duplicate_document}"
    );
    assert!(
        !duplicate_document.to_string().contains("missing.json"),
        "the duplicate request limit refuses before reading its observation: {duplicate_document}"
    );
}

/// Trace: FSpec-FR-287-AC-13.
/// Provenance: QSL-2015.
#[test]
#[trace("FSpec-FR-287-AC-13")]
fn observation_document_bytes_is_not_a_cli_limit_setting() {
    let request = requests::request(SOURCE, Some(("yes", serde_json::json!([]))));
    let output = Command::new(FSL)
        .args([
            "run",
            request.as_str(),
            "--limit",
            "observation.document_bytes=10",
            "--format",
            "json",
        ])
        .output()
        .expect("run with the removed observation limit name");
    assert_eq!(output.status.code(), Some(20));
    let refusal = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(refusal.contains("UnknownSetting"), "{refusal}");
    assert!(refusal.contains("observation.input_bytes"), "{refusal}");
}

/// Trace: FR-002-AC-5; the CLI reads only the input path supplied on argv.
#[test]
fn check_does_not_read_an_unnamed_config_from_the_working_directory() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "fsl-cli-no-implicit-config-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create isolated working directory");
    struct RemoveDirectory(std::path::PathBuf);
    impl Drop for RemoveDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = RemoveDirectory(directory.clone());

    // A config reader would reject this malformed file. The command receives
    // its source explicitly and must not inspect ambient working-directory data.
    std::fs::write(directory.join("fsl.toml"), "[malformed\n")
        .expect("write trap configuration file");

    let output = Command::new(FSL)
        .args(["check", SOURCE, "--format", "json"])
        .current_dir(&directory)
        .output()
        .expect("run check from isolated working directory");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_fsl_document(&output.stdout, &check_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check refusals retain FSL's typed outcome.
#[test]
fn check_refusal_emits_fsl_diagnostic_cause_and_locus() {
    let output = Command::new(FSL)
        .args(["check", INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run check refusal");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());
    assert_fsl_document(&output.stdout, &check_outcome_bytes(INVALID_SOURCE));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits JSON");
    assert_eq!(document["diagnostics"][0]["message"], "expected `)`");
}

/// Trace: FR-002-AC-13.
/// Provenance: QSL-2202.
#[test]
#[trace("FR-002-AC-13")]
fn check_text_refusal_prints_its_message_once() {
    let output = Command::new(FSL)
        .args(["check", INVALID_SOURCE, "--format", "text"])
        .output()
        .expect("run check refusal in text mode");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(text.matches("expected `)`").count(), 1, "{text}");
}

/// Trace: FSpec-FR-301-AC-3.
/// Provenance: QSL-2202.
#[test]
#[trace("FSpec-FR-301-AC-3")]
fn check_bad_option_still_reports_on_stderr() {
    let output = Command::new(FSL)
        .args(["check", SOURCE, "--bogus", "--format", "json"])
        .output()
        .expect("run check with an unknown option");
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--bogus"), "{stderr}");
}

/// Trace: TC-049, FR-002-AC-25; repeated root formats refuse before opening the request.
#[test]
#[trace("TC-049", "FR-002-AC-25")]
fn repeated_format_refuses_with_run_usage_before_request_io() {
    let missing = std::env::temp_dir().join(format!(
        "fsl-missing-format-request-{}.json",
        std::process::id()
    ));
    let output = Command::new(FSL)
        .args(["run", "--format", "json", "--format", "text"])
        .arg(&missing)
        .output()
        .expect("run fsl with repeated formats and missing request");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"--format may be supplied only once; usage: fsl run [options] <request-file>\n"
    );

    let valid = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/frontend-run.json"
    );
    let duplicate = Command::new(FSL)
        .args(["run", valid, "--format", "json", "--format", "text"])
        .output()
        .expect("run fsl with repeated formats and valid request");
    assert_eq!(duplicate.status.code(), Some(20));
    assert!(duplicate.stdout.is_empty());
    assert_eq!(duplicate.stderr, output.stderr);

    let single = Command::new(FSL)
        .args(["run", valid, "--format", "json"])
        .output()
        .expect("run fsl with one output format");
    assert_eq!(single.status.code(), Some(0));
    assert!(single.stderr.is_empty());
    assert!(serde_json::from_slice::<serde_json::Value>(&single.stdout).is_ok());
}

/// Trace: TC-028, FR-006-AC-24; nested wrong-type provider members report their indexed path.
#[test]
#[trace("TC-028", "FR-006-AC-24")]
fn nested_provider_configuration_type_refusal_names_the_member_path() {
    let directory = tempfile::tempdir().expect("create provider config directory");
    let path = directory.path().join("providers.json");
    std::fs::write(
        &path,
        r#"{"format":"filament.provider-config/v1","providers":[{"path":"p"},{"path":"q","arguments":["ok",1]}]}"#,
    )
    .expect("write provider config with an indexed wrong-type member");

    let output = Command::new(FSL)
        .args([
            "run",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/frontend-run.json"
            ),
            "--provider-config",
        ])
        .arg(&path)
        .args(["--format", "text"])
        .output()
        .expect("run fsl with malformed provider configuration");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("providers[1].arguments"), "{stderr}");
    assert!(stderr.contains("invalid-request"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check source I/O failures use FSL's typed code and serializer.
#[test]
fn check_missing_source_emits_fsl_io_error_document() {
    let missing = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/missing-check-source.fsl"
    );
    let output = Command::new(FSL)
        .args(["check", missing, "--format", "json"])
        .output()
        .expect("run check with a missing source");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits JSON");
    let message = document["diagnostics"][0]["message"]
        .as_str()
        .expect("I/O refusal has a diagnostic message");
    assert!(message.starts_with(&format!("could not read source {missing}:")));

    let expected = OutcomeDocument::new(Operation::Check, None, Code::IoError.category())
        .with_diagnostics(vec![OutcomeDiagnostic::new(
            None,
            Code::IoError.as_str(),
            None,
            message,
        )])
        .to_bytes()
        .expect("FSL serializes the check I/O outcome");
    assert_fsl_document(&output.stdout, &expected);
}

/// Trace: FR-002-AC-1, FR-002-AC-3; compile dispatches through FSL's package operation.
#[test]
fn compile_emits_fsl_package_outcome_document() {
    let request = requests::request(SOURCE, None);
    let output = Command::new(FSL)
        .args(["compile", request.as_str(), "--format", "json"])
        .output()
        .expect("run compile command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_fsl_document(&output.stdout, &package_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; compile refusals retain FSL's package outcome.
#[test]
fn compile_refusal_emits_fsl_package_outcome_document() {
    let request = requests::request(INVALID_SOURCE, None);
    let output = Command::new(FSL)
        .args(["compile", request.as_str(), "--format", "json"])
        .output()
        .expect("run compile refusal");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());
    assert_fsl_document(&output.stdout, &package_outcome_bytes(INVALID_SOURCE));
}

/// Trace: QSL-2192; an empty compile program is an invalid request.
#[test]
fn compile_empty_program_is_invalid_request() {
    let directory = tempfile::tempdir().expect("create compile request directory");
    let request = directory.path().join("empty-program.json");
    std::fs::write(
        &request,
        r#"{"format":"native-compile/1","request":{"models":[],"program":[]}}"#,
    )
    .expect("write empty compile request");

    let output = Command::new(FSL)
        .arg("compile")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run compile with an empty program");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI emits a refusal document");
    assert_eq!(document["category"], "refusal");
    assert!(document["last_stage"].is_null(), "{document}");
    assert_eq!(document["diagnostics"][0]["code"], "invalid-request");
}

/// Trace: QSL-1994; `fsl run` preserves the FSL state-rule witness in JSON.
#[test]
fn run_paid_short_json_contains_the_state_rule_witness() {
    let directory = TemporaryDirectory::new("state-rule-witness");
    let request = write_counter_clause_request(&directory.0, "self.value = 3", 2, None);
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run the paid-short state-rule example");
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI emits one JSON outcome");
    assert_eq!(document["category"], "violation", "{document}");
    let item = &document["items"][0];
    assert_eq!(item["clause"], "CounterRule", "{item}");
    assert_eq!(item["category"], "violation", "{item}");
    let witness = item
        .get("witness")
        .filter(|witness| witness.is_object())
        .expect("the state-rule violation carries its witness");
    assert_eq!(
        witness["value_path"]["root"]["subject"]["object_identity"], "a",
        "{witness}"
    );
    assert_eq!(witness["value_path"]["kind"], "runtime_value", "{witness}");
    assert!(
        witness["value_path"]["steps"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "the state rule's value path is rooted at self: {witness}"
    );
    assert!(witness["deciding_comparison"].is_object(), "{witness}");
    assert_eq!(
        witness["operands"].as_array().map(Vec::len),
        Some(2),
        "{witness}"
    );
    assert_eq!(witness["operands"][0]["decimal"], "2", "{witness}");
    assert_eq!(witness["operands"][1]["decimal"], "3", "{witness}");
    assert!(
        witness["decision_path"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "{witness}"
    );
    assert_eq!(witness["deciding_element"]["kind"], "boolean", "{witness}");
    assert_eq!(witness["deciding_element"]["value"], false, "{witness}");
}

/// Trace: FSpec-FR-352-AC-9.
/// Provenance: QSL-2005.
#[test]
#[trace("FSpec-FR-352-AC-9")]
fn run_charge_changes_total_json_contains_the_frame_witness() {
    let directory = TemporaryDirectory::new("frame-witness");
    let mut model: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/analyze-counter.semantic-ir.json"))
            .expect("read the driver-owned Counter model");
    let counter = model["types"]
        .as_array_mut()
        .expect("Counter model types")
        .iter_mut()
        .find(|ty| ty["identity"] == "ix://test/counter/Counter")
        .expect("Counter object type");
    counter["operations"][0]["frame"]["modifies"] = serde_json::json!([]);
    std::fs::write(
        directory.0.join("counter-model.json"),
        serde_json::to_vec(&model).expect("encode the focused frame model"),
    )
    .expect("write the focused frame model");
    std::fs::write(
        directory.0.join("counter.fsl"),
        concat!(
            "language \"ix:native\" edition \"1-draft\";\n",
            "profile v = \"filament.value.complete/v1\";\n",
            "model M = \"test/counter\";\n",
            "post Inc using v on M::Counter::inc { self.value - pre(self.value) = 1 }\n"
        ),
    )
    .expect("write the focused Counter operation rule");
    let snapshot = |name: &str, observation: &str, value: u64| {
        serde_json::json!({
            "format": "filament.state.snapshot/v1",
            "identity": {"authority": "test", "identity": format!("counter/{name}")},
            "model": {"identity": "test/counter"},
            "observation": observation,
            "populations": [{
                "population": "ix://test/counter/counters",
                "complete": true,
                "objects": [{
                    "key": "a",
                    "type": "ix://test/counter/Counter",
                    "fields": {"value": {"integer": value.to_string()}}
                }]
            }]
        })
    };
    for (name, observation, value) in [("pre", "pre", 1), ("post", "post", 2)] {
        std::fs::write(
            directory.0.join(format!("{name}.json")),
            snapshot(name, observation, value).to_string(),
        )
        .expect("write the focused Counter snapshot");
    }
    std::fs::write(
        directory.0.join("invocation.json"),
        serde_json::json!({
            "format": "filament.state.invocation/v1",
            "identity": {"authority": "test", "identity": "counter/invocation"},
            "model": {"identity": "test/counter"},
            "context": "ix://test/counter/Counter",
            "operation": "inc",
            "self": {"population": "ix://test/counter/counters", "key": "a"},
            "pre": {"identity": {"authority": "test", "identity": "counter/pre"}},
            "post": {"identity": {"authority": "test", "identity": "counter/post"}},
            "parameters": {},
            "result": null,
            "created": [],
            "deleted": []
        })
        .to_string(),
    )
    .expect("write the focused Counter invocation");
    let request = serde_json::json!({
        "format": "native-run/1",
        "request": {
            "models": [{
                "format": "semantic-ir/2.0.0",
                "source": {
                    "file": "counter-model.json",
                    "authority": "test",
                    "identity": "counter-model",
                    "document": "Counter",
                    "formal_revision": 1
                }
            }],
            "program": {"source": {
                "file": "counter.fsl",
                "authority": "test",
                "identity": "counter-rules",
                "document": "CounterRules",
                "formal_revision": 1
            }},
            "clause": {
                "selection": {"frame": {
                    "operation": "M::Counter::inc",
                    "invocation": "invocation"
                }},
                "observations": {
                    "invocation": "invocation.json",
                    "pre": "pre.json",
                    "post": "post.json"
                }
            }
        }
    });
    let request_path = directory.0.join("request.json");
    std::fs::write(&request_path, request.to_string()).expect("write the frame run request");
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request_path)
        .args(["--format", "json"])
        .output()
        .expect("run the charge-changes-total frame example");
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI emits one JSON outcome");
    assert_eq!(document["category"], "violation", "{document}");
    let item = &document["items"][0];
    assert_eq!(item["clause"], "M::Counter::inc", "{item}");
    assert_eq!(item["category"], "violation", "{item}");
    assert_eq!(item["cause"]["code"], "frame_violation", "{item}");
    assert_eq!(item["cause"]["kind"], "unauthorized-change", "{item}");

    let witness = item
        .get("witness")
        .and_then(serde_json::Value::as_object)
        .expect("the frame violation carries its user-facing witness");
    assert_eq!(witness["change"]["object"], "a", "{witness:?}");
    assert_eq!(witness["change"]["field"], "value", "{witness:?}");
    assert_eq!(witness["before"], "1", "{witness:?}");
    assert_eq!(witness["after"], "2", "{witness:?}");
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check semantic refusals retain FSL's checked outcome.
#[test]
fn check_semantic_refusal_emits_fsl_check_outcome_document() {
    let output = Command::new(FSL)
        .args(["check", CHECK_INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run check semantic refusal");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());
    assert_fsl_document(&output.stdout, &check_outcome_bytes(CHECK_INVALID_SOURCE));
}

/// Trace: FR-986-AC-1; a `null` expression reaches S3 and exits as a refusal.
#[trace("FR-986-AC-1")]
#[test]
fn null_expression_refusal_exits_twenty_at_s3() {
    let directory = tempfile::tempdir().expect("create null expression source directory");
    let source = directory.path().join("null-expression.fsl");
    std::fs::write(
        &source,
        concat!(
            "language \"ix:native\" edition \"1-draft\";\n",
            "profile v = \"filament.value.complete/v1\";\n",
            "function bad using v(): Integer pure { null }\n"
        ),
    )
    .expect("write null expression source");

    let output = Command::new(FSL)
        .arg("check")
        .arg(&source)
        .args(["--format", "json"])
        .output()
        .expect("check null expression source");

    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI emits a check outcome");
    assert_eq!(document["last_stage"], "S3", "{document}");
    assert_eq!(document["category"], "refusal", "{document}");
    assert_eq!(
        document["diagnostics"][0]["code"], "ill_typed",
        "{document}"
    );
    // FR-986 and FSpec FR-143-AC-20: `null` is `ill_typed` with cause `type-mismatch`,
    // located at the `null` token (bytes 121..125); the message text is language-owned.
    assert_eq!(
        document["diagnostics"][0]["cause"], "type-mismatch",
        "{document}"
    );
    assert_eq!(
        document["diagnostics"][0]["locus"]["start"], 121,
        "{document}"
    );
    assert_eq!(
        document["diagnostics"][0]["locus"]["end"], 125,
        "{document}"
    );
}

/// Trace: FR-002-AC-1, FR-002-AC-3; argument refusals retain the selected FSL operation.
#[test]
fn implemented_verb_argument_refusals_use_the_selected_operation() {
    let cases = [
        (
            "parse",
            Operation::Parse,
            "unknown parse option `--unknown`",
        ),
        (
            "check",
            Operation::Check,
            "unknown check option `--unknown`",
        ),
        (
            "compile",
            Operation::Package,
            "unknown compile option `--unknown`",
        ),
        (
            "format",
            Operation::Format,
            "unknown format option `--unknown`",
        ),
    ];

    for (verb, operation, message) in cases {
        let output = Command::new(FSL)
            .args([verb, "--unknown", "--format", "json"])
            .output()
            .expect("run command with an unknown operation option");
        assert_eq!(output.status.code(), Some(20), "{verb} exit status");
        assert_eq!(
            output.stderr,
            format!("{message}\n").as_bytes(),
            "{verb} stderr"
        );
        assert_fsl_document(&output.stdout, &argument_refusal_bytes(operation, message));
    }
}

/// Trace: FR-002-AC-1, FR-002-AC-3; parse serializes FSL's typed parse result.
#[test]
fn parse_emits_fsl_parse_outcome_document() {
    let output = Command::new(FSL)
        .args(["parse", "issuer", "document", SOURCE, "--format", "json"])
        .output()
        .expect("run parse command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_fsl_document(&output.stdout, &parse_outcome_bytes(SOURCE));
}

/// Trace: TC-965, FR-480-AC-1; the CLI accepts a reference literal in a unit.
#[trace("TC-965", "FR-480-AC-1")]
#[test]
fn parse_accepts_reference_literal_in_unit() {
    let directory = TemporaryDirectory::new("reference-literal");
    let source = directory.0.join("reference-literal.fsl");
    std::fs::write(
        &source,
        concat!(
            "language \"ix:native\" edition \"1-draft\";\n",
            "profile m = \"filament.model.complete/v1\";\n",
            "pre P using m on M::T::op { M::Server#\"s1\" = M::Server#\"s2\" }\n"
        ),
    )
    .expect("write source containing reference literals");

    let output = Command::new(FSL)
        .arg("parse")
        .arg("issuer")
        .arg("reference-literal")
        .arg(&source)
        .arg("--format")
        .arg("json")
        .output()
        .expect("parse source containing reference literals through the CLI");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse emits a JSON outcome");
    assert_eq!(document["operation"], "parse");
    assert_eq!(document["category"], "success");
}

/// Trace: FR-002-AC-1, FR-002-AC-3; a parse refusal retains FSL's cause and locus.
#[test]
fn parse_refusal_emits_fsl_diagnostic_cause_and_locus() {
    let output = Command::new(FSL)
        .args([
            "parse",
            "issuer",
            "document",
            INVALID_SOURCE,
            "--format",
            "json",
        ])
        .output()
        .expect("run parse refusal");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());

    assert_fsl_document(&output.stdout, &parse_outcome_bytes(INVALID_SOURCE));
}

/// Trace: QSL-2190; a caller may raise the parser's node bound from the CLI.
#[trace("QSL-2190")]
#[test]
fn parse_accepts_raised_node_limit() {
    let directory = TemporaryDirectory::new("parse-node-limit");
    let source = write_deep_source(&directory, 6_000);
    let output = Command::new(FSL)
        .arg("parse")
        .arg("issuer")
        .arg("deep")
        .arg(&source)
        .args(["--limit", "s1.nodes=1000000", "--format", "json"])
        .output()
        .expect("parse a deep source with a raised node limit");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse emits an outcome document");
    assert_eq!(document["category"], "success");
}

/// Trace: QSL-2190; the default parser bound remains enforced.
#[trace("QSL-2190")]
#[test]
fn default_node_limit_still_stops_deep_nest() {
    let directory = TemporaryDirectory::new("default-node-limit");
    let source = write_deep_source(&directory, 6_000);
    let output = Command::new(FSL)
        .arg("parse")
        .arg("issuer")
        .arg("deep")
        .arg(&source)
        .arg("--format")
        .arg("json")
        .output()
        .expect("parse a deep source under the default node limit");
    assert_eq!(output.status.code(), Some(22));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse emits an incomplete document");
    assert_eq!(document["category"], "incomplete");
}

/// Trace: QSL-2190; check passes caller-raised source limits to FSL.
#[trace("QSL-2190")]
#[test]
fn check_accepts_raised_node_limit() {
    let directory = TemporaryDirectory::new("check-node-limit");
    let source = write_deep_source(&directory, 6_000);
    let mut command = Command::new(FSL);
    command.arg("check").arg(&source);
    for (setting, _) in SpineLimits::default().bounds() {
        command
            .arg("--limit")
            .arg(format!("{}=1000000", setting.name()));
    }
    let output = command
        .args(["--format", "json"])
        .output()
        .expect("check a deep source with caller-raised limits");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits an outcome document");
    assert_eq!(document["category"], "success");
}

/// Trace: QSL-2190, FR-003-AC-1; compile applies caller limits from FSL's bounds.
#[trace("QSL-2190", "FR-003-AC-1")]
#[test]
fn compile_accepts_limit_settings() {
    let directory = TemporaryDirectory::new("compile-node-limit");
    let source = write_deep_source(&directory, 6_000);
    let request = directory.0.join("compile.json");
    let document = serde_json::json!({
        "format": "native-compile/1",
        "request": {
            "models": [],
            "program": {"source": {
                "file": source,
                "authority": "issuer",
                "identity": "deep",
                "document": "Deep",
                "formal_revision": 1
            }}
        }
    });
    std::fs::write(
        &request,
        serde_json::to_vec(&document).expect("encode compile request"),
    )
    .expect("write compile request");

    let single_override = Command::new(FSL)
        .arg("compile")
        .arg(&request)
        .args(["--limit", "s1.nodes=1000000", "--format", "json"])
        .output()
        .expect("compile with one raised bound and other defaults");
    assert_eq!(
        single_override.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&single_override.stdout)
    );
    let single_result: serde_json::Value =
        serde_json::from_slice(&single_override.stdout).expect("compile emits an outcome document");
    assert_eq!(single_result["category"], "success");

    let mut command = Command::new(FSL);
    command.arg("compile").arg(&request);
    for (setting, _) in SpineLimits::default().bounds() {
        command
            .arg("--limit")
            .arg(format!("{}=1000000", setting.name()));
    }
    let output = command
        .args(["--format", "json"])
        .output()
        .expect("compile a deep source with caller-raised limits");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("compile emits an outcome document");
    assert_eq!(result["category"], "success");

    let invalid = Command::new(FSL)
        .arg("compile")
        .arg(&request)
        .args(["--limit", "unknown.limit=7", "--format", "json"])
        .output()
        .expect("compile with an unknown limit setting");
    assert_eq!(invalid.status.code(), Some(20));
    let refusal: serde_json::Value =
        serde_json::from_slice(&invalid.stdout).expect("invalid limit emits a refusal document");
    let message = refusal["diagnostics"][0]["message"]
        .as_str()
        .expect("limit refusal has a message");
    assert!(message.contains("unknown.limit"), "{message}");
    assert!(message.contains("s1.nodes"), "{message}");
    assert!(message.contains("s3.nodes"), "{message}");
}

/// Trace: QSL-2190, FR-003-AC-5; compile refuses recognized settings it cannot apply.
/// Provenance: FSL FR-869-AC-4 requires the accepted settings list on this refusal.
#[trace("QSL-2190", "FR-003-AC-5")]
#[test]
fn compile_refuses_recognized_unavailable_limit_setting() {
    let directory = TemporaryDirectory::new("compile-unavailable-limit");
    let missing_request = directory.0.join("missing.json");
    let output = Command::new(FSL)
        .arg("compile")
        .arg(&missing_request)
        .args(["--limit", "admission.work_units=5", "--format", "json"])
        .output()
        .expect("compile with a recognized but unavailable limit");

    assert_eq!(output.status.code(), Some(20));
    let refusal: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("unavailable limit emits a refusal");
    assert_eq!(refusal["category"], "refusal");
    assert_eq!(refusal["diagnostics"][0]["code"], "invalid-request");
    let message = refusal["diagnostics"][0]["message"]
        .as_str()
        .expect("limit refusal has a message");
    assert!(message.contains("admission.work_units"), "{message}");
    assert!(
        message.contains("unavailable for the `compile` operation"),
        "{message}"
    );
    let accepted = message
        .split_once("settings accepted by this command: ")
        .map(|(_, accepted)| accepted)
        .expect("refusal lists the command's accepted settings");
    assert!(accepted.contains("s1.nodes"), "{accepted}");
    assert!(accepted.contains("s3.nodes"), "{accepted}");
    assert!(!accepted.contains("admission.work_units"), "{accepted}");
}

/// Trace: QSL-2190; unknown settings identify the valid settings for the verb.
#[trace("QSL-2190")]
#[test]
fn frontend_limit_unknown_setting_names_accepted_settings() {
    let output = Command::new(FSL)
        .args([
            "parse",
            "issuer",
            "deep",
            SOURCE,
            "--limit",
            "unknown.limit=7",
            "--format",
            "json",
        ])
        .output()
        .expect("parse with an unknown limit setting");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid limit emits a refusal document");
    let message = document["diagnostics"][0]["message"]
        .as_str()
        .expect("limit refusal has a message");
    assert!(message.contains("unknown.limit"), "{message}");
    assert!(message.contains("s1.nodes"), "{message}");
}

/// Trace: FR-002-AC-1, FR-002-AC-3; text mode writes FSL's formatted UTF-8 value.
#[test]
fn format_writes_fsl_formatted_text_without_editing_the_source() {
    let source = std::fs::read(SOURCE).expect("read format fixture");
    let parsed = fsl_cst::parse_with_cancel(
        SourceIdentity::new("issuer", "document"),
        SOURCE,
        &source,
        fsl_cst::Limits::default(),
        &Cancel::new(),
    )
    .expect("fixture is an admissible parsed source");
    let expected = format_with_limits(&parsed, FormatLimits::default())
        .expect("fixture formats under the default limit");

    let output = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE])
        .output()
        .expect("run format command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(
        std::fs::read(SOURCE).expect("source remains readable"),
        source
    );
}

/// Trace: FR-002-AC-1, FR-002-AC-3; format machine output is FSL's typed outcome.
#[test]
fn format_machine_mode_emits_fsl_format_outcome_document() {
    let output = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE, "--format", "json"])
        .output()
        .expect("run format in machine mode");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_fsl_document(&output.stdout, &format_outcome_bytes(SOURCE));
}

/// Trace: QSL-2192; the library owns formatted text and the CLI passes it through unchanged.
#[test]
fn format_json_result_carries_text() {
    let text = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE])
        .output()
        .expect("format source as text");
    assert_eq!(text.status.code(), Some(0));
    assert!(text.stderr.is_empty());

    let json = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE, "--format", "json"])
        .output()
        .expect("format source as JSON");
    assert_eq!(json.status.code(), Some(0));
    assert!(json.stderr.is_empty());
    assert_fsl_document(&json.stdout, &format_outcome_bytes(SOURCE));
    let document: serde_json::Value =
        serde_json::from_slice(&json.stdout).expect("format emits a JSON document");
    assert_eq!(document["result"]["kind"], "formatted");
    assert_eq!(
        document["result"]["text"].as_str().map(str::as_bytes),
        Some(text.stdout.as_slice())
    );
}

/// Trace: QSL-2192; formatting an already formatted file is idempotent.
#[test]
fn format_json_is_idempotent() {
    let first = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE])
        .output()
        .expect("format original source");
    assert_eq!(first.status.code(), Some(0));
    let directory = tempfile::tempdir().expect("create formatter directory");
    let formatted = directory.path().join("formatted.fsl");
    std::fs::write(&formatted, &first.stdout).expect("write canonical text");

    let second = Command::new(FSL)
        .arg("format")
        .arg("issuer")
        .arg("document")
        .arg(&formatted)
        .output()
        .expect("format canonical source");
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(second.stdout, first.stdout);
}

/// Trace: FR-002-AC-1, FR-002-AC-3; formatter refusals retain FSL diagnostics.
#[test]
fn format_refusal_emits_fsl_diagnostic_cause_and_locus() {
    let output = Command::new(FSL)
        .args([
            "format",
            "issuer",
            "document",
            INVALID_SOURCE,
            "--format",
            "json",
        ])
        .output()
        .expect("run format refusal");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stderr.is_empty());

    assert_fsl_document(&output.stdout, &format_outcome_bytes(INVALID_SOURCE));
}

/// Trace: FR-002-AC-11, --version reports the Cargo package semantic version.
#[test]
fn version_command_reports_the_fsl_cli_package_version() {
    let output = Command::new(FSL)
        .arg("--version")
        .output()
        .expect("run fsl --version");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
    assert_eq!(
        output.stdout,
        format!("{}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    assert!(output.stderr.is_empty());
}

#[trace("TC-009", "FR-002-AC-17")]
#[test]
fn root_help_selectors_print_the_same_informative_help_without_an_operation() {
    let mut expected_help = None;
    for arguments in [&[][..], &["--help"][..], &["-h"][..], &["help"][..]] {
        let output = Command::new(FSL)
            .args(arguments)
            .output()
            .expect("run root help without a source or lifecycle operation");
        assert_eq!(output.status.code(), Some(0), "arguments: {arguments:?}");
        assert!(output.stderr.is_empty(), "arguments: {arguments:?}");
        let help = std::str::from_utf8(&output.stdout).expect("help is UTF-8");
        for required in [
            "Usage: fsl <command>",
            "--format text|json|dot",
            "dot is accepted only for inspect",
            "--version, -V",
        ] {
            assert!(help.contains(required), "missing {required:?} in {help:?}");
        }
        if let Some(expected) = &expected_help {
            assert_eq!(&output.stdout, expected, "arguments: {arguments:?}");
        } else {
            expected_help = Some(output.stdout);
        }
    }
}

#[trace("TC-009", "FR-002-AC-17")]
#[test]
fn root_help_identifies_current_command_availability_and_text_restrictions() {
    let output = Command::new(FSL)
        .arg("--help")
        .output()
        .expect("read root help");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let help = std::str::from_utf8(&output.stdout).expect("help is UTF-8");
    let command_line = |command| {
        help.lines()
            .find(|line| line.split_whitespace().next() == Some(command))
            .expect("every recognized lifecycle command has its own help row")
    };
    // FR-865: text is the default for check, compile, parse and run.
    for command in ["check", "compile", "parse", "run"] {
        let line = command_line(command);
        assert!(line.contains("text by default"), "{line}");
        assert!(line.contains("--format json selects JSON"), "{line}");
        assert!(!line.contains("unavailable"), "{line}");
    }
    let format_line = command_line("format");
    assert!(format_line.contains("Executes source formatting"));
    assert!(format_line.contains("text output is the default"));
    let analyze_line = command_line("analyze");
    assert!(analyze_line.contains("native-analyze/1"), "{analyze_line}");
    // FSL FR-624 owes each unrouted verb a named outcome document, so this
    // fixes only the exit code, never empty output.
    let monitor_line = command_line("monitor");
    assert!(monitor_line.contains("native-monitor/1"), "{monitor_line}");
    let replay_line = command_line("replay");
    assert!(replay_line.contains("native-replay/1"), "{replay_line}");
    assert!(command_line("inspect").contains("--handoff"));
    for command in ["prove", "generate", "inspect", "render"] {
        if command != "inspect" {
            assert!(command_line(command).contains("unsupported_operation"));
        }
        let unavailable = Command::new(FSL)
            .args([command, "--format", "json"])
            .output()
            .expect("observe the advertised unsupported command");
        assert_eq!(
            unavailable.status.code(),
            Some(i32::from(Category::Unsupported.exit_code())),
            "{command}"
        );
    }
}

#[trace("TC-009", "FR-002-AC-17")]
#[test]
fn informative_help_does_not_replace_invalid_command_or_format_refusals() {
    for (arguments, diagnostic) in [
        (&["not-a-command"][..], "unknown verb `not-a-command`"),
        (
            &["check", "--format", "dot"][..],
            "the `dot` format is available only for `inspect`",
        ),
    ] {
        let output = Command::new(FSL)
            .args(arguments)
            .output()
            .expect("run an invalid command selection");
        assert_eq!(output.status.code(), Some(20), "{arguments:?}");
        assert!(output.stdout.is_empty(), "{arguments:?}");
        assert!(
            std::str::from_utf8(&output.stderr)
                .expect("diagnostics are UTF-8")
                .contains(diagnostic),
            "arguments: {arguments:?}"
        );
    }
}

/// Trace: FR-002-AC-12, the built binary agrees with its Cargo package version.
#[test]
fn built_binary_version_agrees_with_cargo_package_version() {
    let agreement = Agreement::new()
        .surface("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))
        .command("fsl --version", Path::new(FSL), &["--version"]);
    let result = agreement.check();
    assert!(result.is_ok(), "version agreement failed: {result:?}");
}
/// Trace: FR-255-AC-1, `--limit` is accepted by check and parse for S1 ceilings.
#[test]
fn check_and_parse_accept_raised_s1_limits_for_deep_source() {
    let source_path =
        std::env::temp_dir().join(format!("fsl-qsl-2014-deep-{}.native", std::process::id()));
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile Complete = \"filament.value.complete/v1\";\nfunction f using Complete (x: Boolean): Boolean pure {{ {}true }}\n",
        "not ".repeat(6_000)
    );
    std::fs::write(&source_path, source).expect("write deep source fixture");
    let path = source_path.to_string_lossy().into_owned();
    for arguments in [
        vec![
            "parse",
            "issuer",
            "deep",
            path.as_str(),
            "--limit",
            "s1.nodes=1000000",
            "--format",
            "json",
        ],
        vec![
            "check",
            path.as_str(),
            "--limit",
            "s1.nodes=1000000",
            "--format",
            "json",
        ],
    ] {
        let output = Command::new(FSL)
            .args(arguments)
            .output()
            .expect("run source command with raised S1 node limit");
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(output.stderr.is_empty());
    }
    std::fs::remove_file(source_path).expect("remove deep source fixture");
}

/// Trace: FR-255-AC-2, compile accepts a named limit operand.
#[test]
fn compile_accepts_limit_operand() {
    let request = requests::request(SOURCE, None);
    let output = Command::new(FSL)
        .args([
            "compile",
            request.as_str(),
            "--limit",
            "s1.nodes=1000000",
            "--format",
            "json",
        ])
        .output()
        .expect("run compile with a named limit");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
}

/// Trace: FR-255-AC-4, the format source-byte refusal names the accepted setting.
#[test]
fn format_limit_message_names_the_source_byte_setting() {
    let source_path =
        std::env::temp_dir().join(format!("fsl-qsl-2014-format-{}.native", std::process::id()));
    std::fs::write(&source_path, b"source bytes").expect("write format source fixture");
    let path = source_path.to_string_lossy().into_owned();
    let output = Command::new(FSL)
        .args([
            "format",
            "issuer",
            "identity",
            path.as_str(),
            "--limit",
            "s1.input_bytes=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run format with a low source-byte limit");
    assert_eq!(output.status.code(), Some(22));
    let document = String::from_utf8_lossy(&output.stdout);
    assert!(document.contains("s1.input_bytes"), "{document}");
    std::fs::remove_file(source_path).expect("remove format source fixture");
}

/// Trace: FR-255-AC-4, format applies the selected output-byte limit.
#[test]
fn format_accepts_and_applies_output_byte_limit() {
    let output = Command::new(FSL)
        .args([
            "format",
            "issuer",
            "document",
            SOURCE,
            "--limit",
            "format.output_bytes=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run format with a low output-byte limit");
    assert_eq!(output.status.code(), Some(22));
    let document = String::from_utf8_lossy(&output.stdout);
    assert!(document.contains("format.output_bytes"), "{document}");
}

/// Trace: FR-255-AC-6, unknown limits keep FSL's accepted-setting list.
#[test]
fn unknown_limit_setting_lists_accepted_settings() {
    let output = Command::new(FSL)
        .args([
            "parse",
            "issuer",
            "document",
            SOURCE,
            "--limit",
            "not.a.setting=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run parse with an unknown limit setting");
    assert_eq!(output.status.code(), Some(20));
    let document = String::from_utf8_lossy(&output.stdout);
    assert!(document.contains("not.a.setting"), "{document}");
    assert!(document.contains("s1.nodes"), "{document}");
    assert!(document.contains("format.output_bytes"), "{document}");
}

/// Trace: FR-255-AC-3, run limit diagnostics omit request-stage settings.
#[test]
fn run_limit_message_only_offers_the_cli_setting() {
    let request = requests::request(SOURCE, Some(("yes", serde_json::json!([]))));
    let output = Command::new(FSL)
        .args([
            "run",
            request.as_str(),
            "--limit",
            "s1.tokens=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run with a low token limit");
    assert_eq!(output.status.code(), Some(22));
    let document = String::from_utf8_lossy(&output.stdout);
    assert!(document.contains("s1.tokens"), "{document}");
    assert!(!document.contains("stage_limits"), "{document}");
}

#[test]
fn accounting_work_units_item_setting_can_be_raised_to_finish_the_run() {
    let directory = TemporaryDirectory::new("accounting-work-units");
    let request = write_counter_clause_request(&directory.0, "self.value = 0", 0, Some(0));
    let limited = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run with the request's accounting.work_units bound");
    assert_eq!(limited.status.code(), Some(22));
    assert!(limited.stderr.is_empty());

    let limited_document: serde_json::Value =
        serde_json::from_slice(&limited.stdout).expect("limited run outcome JSON");
    assert_eq!(
        limited_document["category"], "incomplete",
        "{limited_document}"
    );
    let cause = &limited_document["items"][0]["cause"];
    assert_eq!(cause["code"], "resource_exhausted", "{cause}");
    let setting = cause["setting"]
        .as_str()
        .expect("the stopped state-rule item names its setting");
    assert_eq!(setting, "accounting.work_units");

    let raised_limit = format!("{setting}=100000");
    let completed = Command::new(FSL)
        .args([
            "run",
            request.to_str().expect("request path is valid UTF-8"),
            "--limit",
            &raised_limit,
            "--format",
            "json",
        ])
        .output()
        .expect("run with the item's named accounting limit raised");
    assert_eq!(completed.status.code(), Some(0));
    assert!(completed.stderr.is_empty());

    let completed_document: serde_json::Value =
        serde_json::from_slice(&completed.stdout).expect("completed run outcome JSON");
    assert_eq!(
        completed_document["category"], "success",
        "{completed_document}"
    );
}

fn run_fsl(arguments: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(FSL).args(arguments).output().expect("run fsl")
}

/// The canonical JSON line `fsl --format json` writes for `document`.
fn canonical_line(document: &OutcomeDocument) -> Vec<u8> {
    let mut encoded =
        ix_cli_kit::json::encode_canonical(document, false).expect("canonical outcome document");
    encoded.push('\n');
    encoded.into_bytes()
}

/// The FSL FR-865 text form of `document`, as `fsl` writes it by default.
fn text_form(document: &OutcomeDocument) -> Vec<u8> {
    fsl_inspect::render_outcome(
        document,
        fsl_inspect::RenderForm::Text,
        &fsl_inspect::RenderSources::from_sources([]),
        fsl_replay::RenderLimits::default(),
        &Cancel::new(),
    )
    .expect("text form")
    .into_value()
    .as_str()
    .as_bytes()
    .to_vec()
}

fn copy_directory(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create copy");
    for entry in std::fs::read_dir(from).expect("read directory") {
        let entry = entry.expect("directory entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

/// FR-002-AC-23 (TC-048): `fsl inspect --handoff` writes the document
/// FSL's `inspect_handoff` returns, with its exit result, and refuses
/// malformed invocations before it reads the directory.
#[test]
#[trace("TC-048", "FR-002-AC-23")]
fn inspect_handoff_verifies_the_package_and_sidecar() {
    use std::ffi::OsStr;
    let directory = tempfile::tempdir().expect("temporary directory");
    let root = directory.path();
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/spine-compile.native"
        ),
        root.join("program.native"),
    )
    .expect("copy the spine fixture");
    let request = serde_json::json!({"format": "native-compile/1", "request": {
        "models": [],
        "program": {"source": {"file": "program.native", "authority": "agent-ix",
            "identity": "test:spine", "document": "Spine", "formal_revision": 1}},
        "handoff": {"directory": "handoff", "package_name": "spine",
            "native_join": {"work_units": 1_000_000, "storage": 1_000_000}}
    }});
    std::fs::write(root.join("compile.json"), request.to_string()).expect("write request");
    let compiled = run_fsl(&[
        OsStr::new("compile"),
        root.join("compile.json").as_os_str(),
        OsStr::new("--format"),
        OsStr::new("json"),
    ]);
    assert_eq!(compiled.status.code(), Some(0), "{compiled:?}");
    let handoff = root.join("handoff");

    let inspect = |path: &Path, format: Option<&str>| {
        let mut arguments = vec![
            OsStr::new("inspect"),
            OsStr::new("--handoff"),
            path.as_os_str(),
        ];
        if let Some(format) = format {
            arguments.extend([OsStr::new("--format"), OsStr::new(format)]);
        }
        run_fsl(&arguments)
    };

    // Steps 2 and 3: the document, byte for byte, in JSON and text.
    let document = command::inspect_handoff(&handoff);
    let json = inspect(&handoff, Some("json"));
    assert_eq!(json.status.code(), Some(0), "{json:?}");
    assert_eq!(json.stdout, canonical_line(&document));
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).expect("JSON document");
    assert_eq!(value["operation"], "inspect");
    assert_eq!(value["category"], "success");
    let text = inspect(&handoff, None);
    assert_eq!(text.status.code(), Some(0), "{text:?}");
    assert_eq!(text.stdout, text_form(&document));

    // Step 4: a selection naming one node count more, and one changed byte.
    let raised = root.join("raised");
    copy_directory(&handoff, &raised);
    let mut selection: serde_json::Value =
        serde_json::from_slice(&std::fs::read(raised.join("selection.json")).expect("selection"))
            .expect("selection JSON");
    let count = &mut selection["package"]["node_counts"]["protocol/compensation"];
    *count = serde_json::json!(count.as_u64().expect("a node count") + 1);
    std::fs::write(raised.join("selection.json"), selection.to_string()).expect("write selection");
    let changed = root.join("changed");
    copy_directory(&handoff, &changed);
    let mut package = std::fs::read(changed.join("checked-package.json")).expect("package");
    let middle = package.len() / 2;
    package[middle] ^= 0x01;
    std::fs::write(changed.join("checked-package.json"), package).expect("write package");

    // Step 5: paths exactly as given, a `-`-leading directory among them.
    let empty = root.join("empty");
    std::fs::create_dir(&empty).expect("empty directory");
    let regular = root.join("compile.json");
    let odd_parent = root.join("odd");
    std::fs::create_dir(&odd_parent).expect("parent of -odd");
    copy_directory(&handoff, &odd_parent.join("-odd"));
    let odd = PathBuf::from("./-odd");
    let refused = [raised, changed, root.join("nosuch"), empty, regular];
    for path in &refused {
        let expected = command::inspect_handoff(path);
        let output = inspect(path, Some("json"));
        assert_eq!(output.status.code(), Some(20), "{path:?}: {output:?}");
        assert_eq!(output.stdout, canonical_line(&expected), "{path:?}");
    }
    let odd_run = Command::new(FSL)
        .args(["inspect", "--handoff"])
        .arg(&odd)
        .args(["--format", "json"])
        .current_dir(&odd_parent)
        .output()
        .expect("run fsl over ./-odd");
    assert_eq!(odd_run.status.code(), Some(0), "{odd_run:?}");
    assert_eq!(
        odd_run.stdout,
        canonical_line(&command::inspect_handoff(&odd_parent.join(&odd)))
    );

    // Step 6: malformed invocations refuse before any read of `handoff`.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&handoff, std::fs::Permissions::from_mode(0o000))
            .expect("remove read permission");
    }
    let h = handoff.as_os_str();
    let cases: [(&[&OsStr], &[&str]); 7] = [
        (&[OsStr::new("--handoff")], &["directory"]),
        (
            &[
                OsStr::new("--handoff"),
                OsStr::new("--format"),
                OsStr::new("json"),
            ],
            &["directory"],
        ),
        (
            &[OsStr::new("--handoff"), h, OsStr::new("extra")],
            &["extra"],
        ),
        (
            &[OsStr::new("--handoff"), h, OsStr::new("--handoff"), h],
            &["--handoff"],
        ),
        (
            &[
                OsStr::new("--handoff"),
                h,
                OsStr::new("--format"),
                OsStr::new("dot"),
            ],
            &["--format", "dot"],
        ),
        (
            &[OsStr::new("--handoff"), h, OsStr::new("--bogus")],
            &["--bogus"],
        ),
        (&[h, OsStr::new("--bogus")], &["--bogus"]),
    ];
    for (arguments, names) in cases {
        let mut full = vec![OsStr::new("inspect")];
        if !arguments.contains(&OsStr::new("--format")) {
            full.extend([OsStr::new("--format"), OsStr::new("json")]);
        }
        full.extend_from_slice(arguments);
        let output = run_fsl(&full);
        assert_eq!(output.status.code(), Some(20), "{full:?}: {output:?}");
        if full.contains(&OsStr::new("dot")) {
            // `dot` selects no JSON document: the refusal is the stderr line.
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("--format") && stderr.contains("dot"),
                "{stderr}"
            );
            assert!(output.stdout.is_empty(), "{output:?}");
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{full:?}: {error}: {output:?}"));
        assert_eq!(value["operation"], "inspect", "{full:?}");
        assert_eq!(
            value["diagnostics"][0]["code"], "invalid-request",
            "{full:?}"
        );
        let message = value["diagnostics"][0]["message"]
            .as_str()
            .expect("message");
        for name in names {
            assert!(message.contains(name), "{full:?}: {message}");
        }
        assert!(!message.contains("cannot read"), "{full:?}: {message}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&handoff, std::fs::Permissions::from_mode(0o755))
            .expect("restore read permission");
    }

    // Step 7: without `--handoff` the form stays not routed.
    let unrouted = run_fsl(&[
        OsStr::new("inspect"),
        h,
        OsStr::new("--format"),
        OsStr::new("json"),
    ]);
    assert_eq!(unrouted.status.code(), Some(21), "{unrouted:?}");
    let value: serde_json::Value = serde_json::from_slice(&unrouted.stdout).expect("document");
    assert_eq!(value["category"], "unsupported");
    assert_eq!(value["diagnostics"][0]["cause"], "operation-not-routed");
    assert_eq!(value["diagnostics"][0]["detail"]["verb"], "inspect");
}

/// QSL-2649 / FSL FR-866: the `missing_import` refusal names
/// `--model <identity>=<file>`; the operand it names, with a file, checks.
#[test]
#[trace("FR-002-AC-24", "FR-002-AC-25")]
fn check_accepts_the_model_operand_the_missing_import_message_names() {
    let directory = TemporaryDirectory::new("check-model-message");
    let model = directory.0.join("model.json");
    std::fs::copy(MODEL, &model).expect("copy model fixture");
    let refused = Command::new(FSL)
        .args(["check", CHECK_MODEL_SOURCE, "--format", "json"])
        .output()
        .expect("check without its model");
    assert_eq!(refused.status.code(), Some(20));
    let document: serde_json::Value = serde_json::from_slice(&refused.stdout).expect("document");
    let message = document["diagnostics"][0]["message"]
        .as_str()
        .expect("message");
    let named = message
        .split("--model ")
        .nth(1)
        .and_then(|rest| rest.split("<file>").next())
        .expect("the message names the --model operand");
    assert!(named.ends_with('='), "{message}");
    let operand = format!("{named}{}", model.display());
    let accepted = Command::new(FSL)
        .args([
            "check",
            CHECK_MODEL_SOURCE,
            "--model",
            &operand,
            "--format",
            "json",
        ])
        .output()
        .expect("check with the named operand");
    assert_eq!(
        accepted.status.code(),
        Some(0),
        "{operand}: {}",
        String::from_utf8_lossy(&accepted.stdout)
    );
    let absent = directory.0.join("absent.json");
    let duplicate = [
        format!("driver/ir608-model={}", absent.display()),
        format!("driver/ir608-model={}", absent.display()),
    ];
    let mut cases: Vec<Vec<String>> = ["no-equals", "=file.json", "driver/ir608-model="]
        .into_iter()
        .map(|bad| vec![bad.to_owned()])
        .collect();
    cases.push(duplicate.to_vec());
    for operands in cases {
        let mut arguments = vec!["check".to_owned(), CHECK_MODEL_SOURCE.to_owned()];
        for operand in &operands {
            arguments.extend(["--model".to_owned(), operand.clone()]);
        }
        arguments.extend(["--format".to_owned(), "json".to_owned()]);
        let output = Command::new(FSL)
            .args(&arguments)
            .output()
            .expect("check with a malformed operand");
        assert_eq!(output.status.code(), Some(20), "{operands:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("--model"), "{operands:?}: {stdout}");
        assert!(
            !stdout.contains("io-error"),
            "no file read: {operands:?}: {stdout}"
        );
    }
}

/// QSL-2649 / FSL FR-866: a unit that selects two models checks from two
/// `--model <identity>=<file>` operands, and an operand no `model` declaration
/// selects is refused after the source parses.
#[test]
#[trace("FR-002-AC-24", "FR-002-AC-25")]
fn check_takes_two_model_operands_for_a_unit_selecting_two_models() {
    let directory = TemporaryDirectory::new("check-two-models");
    let first = directory.0.join("a.json");
    let second = directory.0.join("b.json");
    std::fs::copy(MODEL, &first).expect("copy first model");
    let text = std::fs::read_to_string(MODEL).expect("read model fixture");
    std::fs::write(
        &second,
        text.replace("driver/ir608-model", "driver/ir608-model-b"),
    )
    .expect("write second model");
    let unit = directory.0.join("two.fsl");
    std::fs::write(
        &unit,
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"filament.model.complete/v1\";\n\
         model M = \"driver/ir608-model\";\n\
         model N = \"driver/ir608-model-b\";\n\
         function valid using v(): Boolean pure { true }\n",
    )
    .expect("write unit");
    let check = |operands: &[String]| {
        let mut command = Command::new(FSL);
        command.arg("check").arg(&unit);
        for operand in operands {
            command.args(["--model", operand]);
        }
        command.args(["--format", "json"]).output().expect("check")
    };
    let one = format!("driver/ir608-model={}", first.display());
    let two = format!("driver/ir608-model-b={}", second.display());
    let both = check(&[one.clone(), two.clone()]);
    assert_eq!(
        both.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&both.stdout)
    );
    let reversed = check(&[two.clone(), one.clone()]);
    assert_eq!(reversed.status.code(), Some(0));
    let only_one = check(std::slice::from_ref(&one));
    assert_eq!(only_one.status.code(), Some(20));
    assert!(String::from_utf8_lossy(&only_one.stdout).contains("driver/ir608-model-b"));
    let unselected = check(&[one, two, format!("driver/unused={}", first.display())]);
    assert_eq!(unselected.status.code(), Some(20));
    let stdout = String::from_utf8_lossy(&unselected.stdout);
    assert!(
        stdout.contains("--model") && stdout.contains("driver/unused"),
        "{stdout}"
    );
}
