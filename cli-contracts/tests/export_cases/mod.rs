use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::{json, Value};

mod intake;
mod options;
mod staged;

const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function Ready using v(): Boolean pure { true }
"#;

fn source(file: &str) -> Value {
    json!({"file":file,"authority":"export-contracts","identity":"contracts:bin-capacity","document":"BinCapacity","formal_revision":1})
}

fn request() -> Value {
    json!({"format":"native-run/1","request":{"models":[],"program":{"source":source("unit.fsl")},
        "export":{"profile":"filament.output.tla2/v1","obligations":["Capacity"]}}})
}

fn fixture() -> Result<CliCase, ContractError> {
    let case = CliCase::from_environment()?;
    case.write("unit.fsl", SOURCE.as_bytes())?;
    write_json(&case, "export.json", &request())?;
    Ok(case)
}
fn write_json(case: &CliCase, path: &str, value: &Value) -> Result<(), ContractError> {
    case.write(path, &serde_json::to_vec(value)?)?;
    Ok(())
}
fn run(
    case: &CliCase,
    path: &str,
    evidence: bool,
    extra: &[&str],
) -> Result<Captured, ContractError> {
    let mut args = vec!["export"];
    if evidence {
        args.push("--evidence");
    }
    args.push(path);
    args.extend_from_slice(&["--format", "json"]);
    args.extend_from_slice(extra);
    case.run(&args)
}
fn outcome(output: &Captured, exit: i32, category: &str) -> Value {
    assert_eq!(output.0.status.code(), Some(exit), "{output:?}");
    let value = output.json().expect("one export outcome");
    assert_eq!(value["format"], "filament-outcome/1", "{value}");
    assert_eq!(value["operation"], "export", "{value}");
    assert_eq!(value["category"], category, "{value}");
    value
}
fn diagnostic<'a>(value: &'a Value, code: &str) -> &'a Value {
    value["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .find(|d| d["code"] == code)
        .unwrap_or_else(|| panic!("missing {code}: {value}"))
}
fn invalid(output: &Captured, names: &[&str]) -> Value {
    let value = outcome(output, 20, "refusal");
    let d = diagnostic(&value, "invalid-request");
    let message = d["message"].as_str().expect("message");
    for name in names {
        assert!(message.contains(name), "{name}: {value}");
    }
    assert!(value["last_stage"].is_null(), "{value}");
    assert!(value["result"].is_null(), "{value}");
    value
}
