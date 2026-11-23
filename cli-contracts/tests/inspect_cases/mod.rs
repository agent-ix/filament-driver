// SPDX-License-Identifier: AGPL-3.0-or-later
use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::{json, Value};

mod artifacts;
mod options;
mod paths;

const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile values = "filament.value.complete/v1";
function HasReserve using values(stock: Int[0, 9]): Boolean pure { stock >= 2 }
"#;

fn handoff(directory: &str) -> Result<CliCase, ContractError> {
    let case = CliCase::from_environment()?;
    case.write("reserve.fsl", SOURCE.as_bytes())?;
    case.write("compile.json",&serde_json::to_vec(&json!({"format":"native-compile/1","request":{
        "models":[],"program":{"source":{"file":"reserve.fsl","authority":"contracts","identity":"reserve-rule","document":"ReserveRule","formal_revision":1}},
        "handoff":{"directory":directory,"package_name":"reserve-rule","native_join":{"work_units":1_000_000,"storage":1_000_000}}
    }}))?)?;
    let output = case.run(&["compile", "compile.json", "--format", "json"])?;
    assert_eq!(
        output.0.status.code(),
        Some(0),
        "fixture admission: {output:?}"
    );
    let value = output.json()?;
    assert_eq!(value["category"], "success", "{value}");
    assert!(output.0.stderr.is_empty());
    Ok(case)
}

fn document(output: &Captured, exit: i32, category: &str) -> Value {
    assert_eq!(output.0.status.code(), Some(exit), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    let value = output.json().expect("one JSON inspect outcome");
    assert_eq!(value["format"], "filament-outcome/1");
    assert_eq!(value["operation"], "inspect");
    assert_eq!(value["category"], category, "{value}");
    value
}

fn inspect(case: &CliCase, directory: &str) -> Result<Captured, ContractError> {
    case.run(&["inspect", "--handoff", directory, "--format", "json"])
}

fn verified(output: &Captured) -> Value {
    let value = document(output, 0, "success");
    assert_eq!(value["last_stage"], Value::Null);
    assert!(value["items"].as_array().unwrap().is_empty());
    assert!(value["diagnostics"].as_array().unwrap().is_empty());
    assert!(value["result"].is_null());
    value
}

fn refusal(output: &Captured, code: &str) -> Value {
    let value = document(output, 20, "refusal");
    assert!(value["items"].as_array().unwrap().is_empty());
    assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1);
    let diagnostic = &value["diagnostics"][0];
    assert_eq!(diagnostic["code"], code, "{value}");
    // FSL FR-665-AC-5: a reader refusal that names a file or directory carries an
    // `input` locus with exactly these four members, never an `artifact` locus.
    let locus = diagnostic["locus"].as_object().expect("a locus object");
    assert_eq!(locus["kind"], "input", "{value}");
    let mut keys: Vec<_> = locus.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["kind", "member", "operand", "pointer"], "{value}");
    assert!(
        locus["member"].is_null() || locus["member"].is_string(),
        "{value}"
    );
    assert!(
        locus["operand"]
            .as_str()
            .is_some_and(|operand| !operand.is_empty()),
        "{value}"
    );
    assert!(locus["pointer"].is_null(), "{value}");
    assert!(!diagnostic["message"].as_str().unwrap().is_empty());
    value
}

fn selection(case: &CliCase) -> Value {
    serde_json::from_slice(&std::fs::read(case.directory().join("handoff/selection.json")).unwrap())
        .unwrap()
}
