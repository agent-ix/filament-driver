use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::{json, Value};

mod functions;
mod inputs;
mod options;
mod temporal;

const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function Remaining using v(total: Integer, held: Integer): Integer pure {
    total - held
}
function IsPacked using v(n: Int[0, 10]): Boolean pure { n >= 6 }
"#;

fn request(source: &str, name: &str, args: &[(&str, Value)]) -> Value {
    json!({"format":"native-run/1","request":{
        "models":[],"program":{"source":{"file":source,"authority":"run-contracts",
            "identity":"contracts:packing","document":"Packing","formal_revision":1}},
        "call":{"function":name,"arguments":args.iter().map(|(parameter,value)|
            json!({"parameter":parameter,"value":value})).collect::<Vec<_>>()}
    }})
}

fn fixture() -> Result<CliCase, ContractError> {
    let case = CliCase::from_environment()?;
    case.write("packing.fsl", SOURCE.as_bytes())?;
    write_json(
        &case,
        "run.json",
        &request(
            "packing.fsl",
            "Remaining",
            &[
                ("total", json!({"type":"integer","value":"17"})),
                ("held", json!({"type":"integer","value":"5"})),
            ],
        ),
    )?;
    Ok(case)
}

fn write_json(case: &CliCase, path: &str, value: &Value) -> Result<(), ContractError> {
    case.write(path, &serde_json::to_vec(value)?)?;
    Ok(())
}

fn outcome(output: &Captured, exit: i32, category: &str) -> Value {
    assert_eq!(output.0.status.code(), Some(exit), "{output:?}");
    let value = output.json().expect("one run outcome");
    assert_eq!(value["format"], "filament-outcome/1", "{value}");
    assert_eq!(value["operation"], "execute", "{value}");
    assert_eq!(value["category"], category, "{value}");
    value
}

fn completed(output: &Captured, expected: Value) -> Value {
    assert!(output.0.stderr.is_empty(), "{output:?}");
    let value = outcome(output, 0, "success");
    assert_eq!(
        value["result"],
        json!({"kind":"completed","value":expected}),
        "{value}"
    );
    assert!(
        value["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .is_empty(),
        "{value}"
    );
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

fn run(case: &CliCase, path: &str, extra: &[&str]) -> Result<Captured, ContractError> {
    let mut args = vec!["run", path, "--format", "json"];
    args.extend_from_slice(extra);
    case.run(&args)
}
