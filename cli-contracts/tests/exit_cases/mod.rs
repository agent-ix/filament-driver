// SPDX-License-Identifier: AGPL-3.0-or-later
use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::{json, Value};

mod categories;
mod verbs;

// FSL FR-285's complete category table, including finite-window exit 11.
const CATEGORY_EXITS: [(&str, i32); 10] = [
    ("success", 0),
    ("violation", 10),
    ("undefined", 10),
    ("refusal", 20),
    ("unsupported", 21),
    ("incomplete", 22),
    ("inconclusive", 22),
    ("pending", 0),
    ("indeterminate", 11),
    ("internal-failure", 30),
];

const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function Spare using v(total: Integer, held: Integer): Integer pure { total - held }
function PositiveShift using v(n: Int[0, 3]): Boolean pure { n + 2 > n }
function BelowFour using v(n: Int[0, 3]): Boolean pure { n + 2 < 4 }
"#;

fn write_json(case: &CliCase, path: &str, value: &Value) -> Result<(), ContractError> {
    case.write(path, &serde_json::to_vec(value)?)
}

fn source_ref(file: &str) -> Value {
    json!({"source":{"file":file,"authority":"exit-contracts","identity":"exit-program",
        "document":"ExitProgram","formal_revision":1}})
}

fn fixture() -> Result<CliCase, ContractError> {
    let case = CliCase::from_environment()?;
    case.write("program.fsl", SOURCE.as_bytes())?;
    write_json(
        &case,
        "compile.json",
        &json!({"format":"native-compile/1","request":{
            "models":[],"program":source_ref("program.fsl")
        }}),
    )?;
    write_json(
        &case,
        "run.json",
        &json!({"format":"native-run/1","request":{
            "models":[],"program":source_ref("program.fsl"),"call":{"function":"Spare","arguments":[
                {"parameter":"total","value":{"type":"integer","value":"17"}},
                {"parameter":"held","value":{"type":"integer","value":"6"}}
            ]}
        }}),
    )?;
    Ok(case)
}

fn category_exit(category: &str) -> i32 {
    CATEGORY_EXITS
        .iter()
        .find_map(|(label, exit)| (*label == category).then_some(*exit))
        .unwrap_or_else(|| panic!("undocumented category {category:?}"))
}

fn assert_exit(output: &Captured, category: &str) {
    let actual = output
        .0
        .status
        .code()
        .expect("normal invocation must return a numeric exit");
    assert!(
        CATEGORY_EXITS.iter().any(|(_, exit)| *exit == actual),
        "undocumented process exit {actual}: {output:?}"
    );
    assert_eq!(actual, category_exit(category), "{output:?}");
}

fn outcome(output: &Captured, operation: &str, category: &str) -> Result<Value, ContractError> {
    assert_exit(output, category);
    assert!(output.0.stderr.is_empty(), "{output:?}");
    let value = output.json()?;
    assert_eq!(value["format"], "filament-outcome/1", "{value}");
    assert_eq!(value["operation"], operation, "{value}");
    assert_eq!(value["category"], category, "{value}");
    Ok(value)
}
