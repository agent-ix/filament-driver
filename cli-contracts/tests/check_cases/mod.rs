use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::{json, Value};

mod dependencies;
mod markdown;
mod options;
mod pending;
mod sources;

const UNIT: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function Eligible using v(n: Int[0, 12]): Boolean pure { n >= 3 }
"#;

fn case() -> Result<CliCase, ContractError> {
    let case = CliCase::from_environment()?;
    case.write("eligible.fsl", UNIT.as_bytes())?;
    Ok(case)
}

fn document(output: &Captured, exit: i32, category: &str) -> Value {
    assert_eq!(output.0.status.code(), Some(exit), "{output:?}");
    let value = output.json().expect("one machine-mode check outcome");
    assert_eq!(value["format"], "filament-outcome/1");
    assert_eq!(value["operation"], "check");
    assert_eq!(value["category"], category, "{value}");
    value
}

fn success(output: &Captured) -> Value {
    let value = document(output, 0, "success");
    assert_eq!(value["last_stage"], "S4");
    assert!(value["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .is_empty());
    assert!(output.0.stderr.is_empty(), "{output:?}");
    value
}

fn diagnostic<'a>(value: &'a Value, code: &str) -> &'a Value {
    value["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .find(|diagnostic| diagnostic["code"] == code)
        .unwrap_or_else(|| panic!("missing diagnostic {code}: {value}"))
}

fn source_locus(diagnostic: &Value, path: &str) {
    assert_eq!(diagnostic["locus"]["kind"], "region", "{diagnostic}");
    assert_eq!(
        diagnostic["locus"]["source"]["authority"], "filament",
        "{diagnostic}"
    );
    assert_eq!(
        diagnostic["locus"]["source"]["identity"], path,
        "{diagnostic}"
    );
}

fn argv_error(output: &Captured, named: &str) {
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    assert!(
        output.0.stdout.is_empty(),
        "argument usage errors have no primary result: {output:?}"
    );
    assert!(
        output
            .stderr_text()
            .expect("text diagnostic")
            .contains(named),
        "{output:?}"
    );
}

// Original caller-owned domain data in the public semantic-IR document format.
// This is an input package, never a manually assembled checked FSL package.
fn parcel_model(identity: &str) -> Value {
    let node = format!("ix://{identity}/Parcel");
    json!({
        "contractVersion":"2.0.0", "canonicalization":{"algorithm":"rfc8785-v1"},
        "source":{"identity":format!("ix://{identity}/source"),"version":"1.0.0","dialect":"spec-bundle"},
        "package":{"identity":identity,"version":"1.0.0","mappingVersions":[],"profileVersions":[]},
        "occurrences":[],"extensions":[],
        "constructs":[{"kind":{"module":identity,"name":"parcel"},"moduleVersion":"1.0.0", "construct":{"identity":"none","shape":"record","members":{},"meaning":"filament.meaning.model.object-type/v1"}}],
        "types":[{"identity":node,"displayName":"Parcel","kind":{"module":identity,"name":"parcel"},"roles":[],"origin":{"generated":{"generatorIdentity":format!("ix://{identity}/source"),"generatorVersion":"1.0.0","inputIdentities":[node]}},"constraints":[],"extensions":[],"unknownPolicy":"reject","supertypes":[],"fields":[],"operations":[]}],
        "populations":[]
    })
}

const LIBRARY: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function WithFee using v(amount: Integer): Integer pure { amount + 4 }
"#;
const IMPORTER: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
import "contracts/fees" as Fees;
function Charge using v(amount: Integer): Integer pure { Fees::WithFee(amount) }
"#;
