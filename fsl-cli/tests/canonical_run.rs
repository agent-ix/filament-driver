// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller overrides through the public canonical run coordinator.

use std::path::Path;
use std::process::{Command, Output};

use fsl_command::command::{self, RequestOverrides, RunLimits};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const FSL: &str = env!("CARGO_BIN_EXE_fsl");

fn write(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec(value).expect("JSON fixture")).expect("write fixture");
}

fn cli(path: &Path, arguments: &[&str]) -> Output {
    Command::new(FSL)
        .arg("run")
        .arg(path)
        .args(["--format", "json"])
        .args(arguments)
        .output()
        .expect("run CLI")
}

fn document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("one run result")
}

fn assert_owner_parity(path: &Path, overrides: &RequestOverrides<'_>, actual: &Output) {
    let negotiation = filament_driver::AnalyzeNegotiation::fsl().expect("FSL negotiation");
    let (expected, exit) = match command::run_with_providers_and_limits_and_overrides(
        path,
        RunLimits::default(),
        overrides,
        &negotiation,
        filament_driver::run_engines(Vec::new()).expect("driver engines"),
        &fsl_analyze::provider::ProviderOptions::default(),
        &filament_exact::Cancel::new(),
    ) {
        Ok(result) => {
            let value = match result.outcome {
                Some(outcome) => serde_json::to_value(outcome).expect("outcome JSON"),
                None => result.value.as_value().clone(),
            };
            (value, result.exit_code)
        }
        Err(error) => (
            serde_json::to_value(error.outcome_document(fsl_replay::Operation::Execute))
                .expect("refusal JSON"),
            error.category().exit_code(),
        ),
    };
    assert_eq!(actual.status.code(), Some(i32::from(exit)), "{actual:?}");
    assert_eq!(document(actual), expected, "{actual:?}");
}

fn source(file: &str) -> Value {
    json!({"file":file,"authority":"test","identity":format!("canonical-run:{file}"),
        "document":"CanonicalRun","formal_revision":1})
}

/// The effective caller budget replaces request accounting for calls and ordered items.
#[trace("FR-002-AC-27", "FR-003-AC-1")]
#[test]
fn canonical_run_overrides_reach_calls_and_ordered_items() {
    let directory = tempfile::tempdir().expect("fixture directory");
    std::fs::write(directory.path().join("unit.fsl"),
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\nfunction Remaining using v(total: Integer, held: Integer): Integer pure { total - held }\n")
        .expect("source fixture");
    let call = json!({"function":"Remaining","arguments":[
        {"parameter":"total","value":{"type":"integer","value":"17"}},
        {"parameter":"held","value":{"type":"integer","value":"5"}}],
        "accounting":{"work_units":u64::MAX}});
    let mut job = json!({"format":"native-run/1","request":{
        "models":[],"program":{"source":source("unit.fsl")},"call":call}});
    let path = directory.path().join("run.json");
    let mut accounting = fsl_replay::AccountingLimits::default().0;
    accounting.work_units = 0;
    let overrides = RequestOverrides {
        accounting: Some(&accounting),
        ..Default::default()
    };
    // An explicit provider context must carry the same overrides as a plain run.
    let config = directory.path().join("providers.json");
    write(
        &config,
        &json!({"format":"filament.provider-config/v1","providers":[]}),
    );
    for items in [false, true] {
        if items {
            job["request"]
                .as_object_mut()
                .expect("request")
                .remove("call");
            job["request"]["items"] = json!({"selections":[{"call":call},{"call":call}]});
        }
        write(&path, &job);
        let control = cli(&path, &[]);
        assert_eq!(control.status.code(), Some(0), "{control:?}");
        let limited = cli(&path, &["--limit", "accounting.work_units=0"]);
        assert_eq!(limited.status.code(), Some(22), "{limited:?}");
        assert_owner_parity(&path, &overrides, &limited);
        let configured = cli(
            &path,
            &[
                "--limit",
                "accounting.work_units=0",
                "--provider-config",
                config.to_str().expect("config path"),
            ],
        );
        assert_eq!(configured.status.code(), limited.status.code());
        assert_eq!(configured.stdout, limited.stdout);
        let outcome = document(&limited);
        if items {
            let rows = outcome["items"].as_array().expect("ordered items");
            assert_eq!(rows.len(), 2, "{outcome}");
            for (index, row) in rows.iter().enumerate() {
                assert_eq!(row["selection"], index, "{row}");
                assert!(row["result"].to_string().contains("work_units"), "{row}");
            }
        } else {
            assert!(outcome.to_string().contains("work_units"), "{outcome}");
        }
    }
}

/// Plan compilation, initial-state admission, execution and duplicate clocks use owner overrides.
#[trace("FR-002-AC-27", "FR-003-AC-1", "FR-023-AC-5")]
#[test]
fn canonical_run_overrides_reach_plan_stages_observations_accounting_and_clocks() {
    let directory = tempfile::tempdir().expect("fixture directory");
    std::fs::write(
        directory.path().join("model.json"),
        include_str!("fixtures/analyze-counter.semantic-ir.json"),
    )
    .expect("model fixture");
    std::fs::write(
        directory.path().join("initial.json"),
        include_str!("fixtures/analyze-counter-start.json"),
    )
    .expect("initial fixture");
    let model_unit = include_str!("fixtures/analyze-counter.fsl")
        .replace("filament.value.complete/v1", "filament.model.complete/v1");
    let unit = format!("{model_unit}\nverify Release using v {{ check bounded claim Bounded method model_check domain M::Counter bound 2; }}\n");
    std::fs::write(directory.path().join("unit.fsl"), unit).expect("source fixture");
    let mut job = json!({"format":"native-run/1","request":{
        "models":[{"format":"semantic-ir/2.0.0","source":source("model.json")}],
        "program":{"source":source("unit.fsl")},
        "verification_plan":{"plan":"Release","initial":"initial.json","emit":true,"execute":false}}});
    let path = directory.path().join("plan.json");
    write(&path, &job);
    let control = cli(&path, &[]);
    assert_eq!(control.status.code(), Some(0), "{control:?}");
    assert_owner_parity(&path, &RequestOverrides::default(), &control);
    let mut stages = fsl_replay::spine::SpineLimits::default();
    stages.source.source_bytes = 1;
    let observation = fsl_replay::ObservationLimits::default().with_document_bytes(1);
    for (setting, overrides) in [
        (
            "s1.input_bytes=1",
            RequestOverrides {
                limits: Some(&stages),
                ..Default::default()
            },
        ),
        (
            "observation.input_bytes=1",
            RequestOverrides {
                observation: Some(&observation),
                ..Default::default()
            },
        ),
    ] {
        let actual = cli(&path, &["--limit", setting]);
        assert_eq!(actual.status.code(), Some(22), "{actual:?}");
        let outcome = document(&actual);
        assert!(
            outcome.to_string().contains("stage_limit_exceeded"),
            "{outcome}"
        );
        assert!(
            outcome
                .to_string()
                .contains(setting.split('=').next().expect("setting")),
            "{outcome}"
        );
        assert_owner_parity(&path, &overrides, &actual);
    }
    job["request"]["verification_plan"]["execute"] = json!(true);
    job["request"]["requirements"] = json!({"backend":"en-1"});
    write(&path, &job);
    let mut accounting = fsl_replay::AccountingLimits::default().0;
    accounting.work_units = 0;
    let limited = cli(&path, &["--limit", "accounting.work_units=0"]);
    assert_eq!(limited.status.code(), Some(22), "{limited:?}");
    assert!(
        document(&limited).to_string().contains("work_units"),
        "{limited:?}"
    );
    assert_owner_parity(
        &path,
        &RequestOverrides {
            accounting: Some(&accounting),
            ..Default::default()
        },
        &limited,
    );
    job["request"]["clocks"] = json!({});
    write(&path, &job);
    let clocks_path = directory.path().join("clocks.json");
    write(&clocks_path, &json!({}));
    let clocks = serde_json::value::RawValue::from_string("{}".to_owned()).expect("clocks object");
    let duplicate = cli(
        &path,
        &["--clocks", clocks_path.to_str().expect("clocks path")],
    );
    assert_eq!(duplicate.status.code(), Some(20), "{duplicate:?}");
    assert!(
        document(&duplicate).to_string().contains("invalid-request"),
        "{duplicate:?}"
    );
    let duplicate_document = document(&duplicate);
    let message = duplicate_document["diagnostics"][0]["message"]
        .as_str()
        .expect("duplicate clocks diagnostic");
    assert!(
        message.contains("clocks") && message.contains("--clocks"),
        "{message}"
    );
    assert_owner_parity(
        &path,
        &RequestOverrides {
            clocks: Some(&clocks),
            ..Default::default()
        },
        &duplicate,
    );
}

/// String, Bytes, JsonObject and String map keys retain distinct owner wire forms.
#[trace("FR-002-AC-27", "FR-650-AC-1")]
#[test]
fn canonical_run_scalar_wire_forms_match_the_owner() {
    let directory = tempfile::tempdir().expect("fixture directory");
    std::fs::write(directory.path().join("scalars.fsl"),
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\n\
         function StringValue using v(x: String): String pure { x }\n\
         function BytesValue using v(x: Bytes): Bytes pure { x }\n\
         function JsonValue using v(x: JsonObject): JsonObject pure { x }\n\
         function MapValue using v(x: Map<String, Int[0, 9]>[0, 2]): Map<String, Int[0, 9]>[0, 2] pure { x }\n")
        .expect("source fixture");
    let path = directory.path().join("scalars.json");
    let entry = |key, value| {
        json!({"key":{"type":"string","value":key},
        "value":{"type":"integer","value":value}})
    };
    for (function, input, expected) in [
        (
            "StringValue",
            json!({"type":"string","value":"e\u{0301}"}),
            json!({"type":"string","value":"e\u{0301}"}),
        ),
        (
            "BytesValue",
            json!({"type":"bytes","value":"00ff"}),
            json!({"type":"bytes","value":"00ff"}),
        ),
        (
            "JsonValue",
            json!({"type":"json_object","value":"{\"a\":1}"}),
            json!({"type":"json_object","value":"{\"a\":1}"}),
        ),
        (
            "MapValue",
            json!({"type":"map","entries":[entry("b", "2"),entry("a", "1")]}),
            json!({"type":"map","entries":[entry("a", "1"),entry("b", "2")]}),
        ),
    ] {
        write(
            &path,
            &json!({"format":"native-run/1","request":{
            "models":[],"program":{"source":source("scalars.fsl")},
            "call":{"function":function,"arguments":[{"parameter":"x","value":input}]}}}),
        );
        let actual = cli(&path, &[]);
        assert_eq!(actual.status.code(), Some(0), "{function}: {actual:?}");
        assert_eq!(
            document(&actual)["result"]["value"],
            expected,
            "{function}: {actual:?}"
        );
        assert_owner_parity(&path, &RequestOverrides::default(), &actual);
    }
}
