// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

const BIN_SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
profile m = "filament.model.complete/v1";
model Bins = "contracts/bins";
pre Room using v on Bins::Bin::add { self.used < 3 }
post Added using v on Bins::Bin::add { self.used = pre(self.used) + 1 }
invariant Capacity using m on Bins::Bin at current { self.used <= 3 }
"#;

fn origin(identity: &str) -> Value {
    json!({"generated":{"generatorIdentity":identity,"generatorVersion":"1.0.0","inputIdentities":[identity]}})
}

fn bin_model() -> Value {
    let base = "ix://contracts/bins";
    let fill = format!("{base}/Fill");
    let bin = format!("{base}/Bin");
    let used = format!("{bin}/used");
    let population = format!("{base}/bins");
    let kind = |name: &str| json!({"module":"contracts/bins","name":name});
    let constructs=[("object","filament.meaning.model.object-type/v1"),("population","filament.meaning.model.population/v1"),("integer","filament.meaning.model.value-type/v1")].iter().map(|(name,meaning)|json!({"kind":kind(name),"moduleVersion":"1.0.0","construct":{"identity":"none","shape":"record","members":{},"meaning":meaning}})).collect::<Vec<_>>();
    let constraints=[("min","0"),("max","4")].iter().map(|(name,value)|json!({"identity":format!("{fill}/{name}"),"keyword":name,"operands":{"value":value},"appliesTo":"ix://filament/native/Integer","diagnosticCode":format!("bound.{name}"),"origin":origin(&fill)})).collect::<Vec<_>>();
    json!({"contractVersion":"2.0.0","canonicalization":{"algorithm":"rfc8785-v1"},
        "source":{"identity":format!("{base}/source"),"version":"1.0.0","dialect":"spec-bundle"},
        "package":{"identity":"contracts/bins","version":"1.0.0","mappingVersions":[],"profileVersions":[]},
        "constructs":constructs,"occurrences":[],"extensions":[],
        "types":[{"identity":fill,"displayName":"Fill","kind":kind("integer"),"roles":[],"origin":origin(&fill),"constraints":constraints,"extensions":[],"unknownPolicy":"reject","scalar":"integer"},
        {"identity":bin,"displayName":"Bin","kind":kind("object"),"roles":[],"origin":origin(&bin),"constraints":[],"extensions":[],"unknownPolicy":"reject","supertypes":[],
            "fields":[{"identity":used,"name":"used","typeRef":fill,"presence":"required","nullable":false,"defaultKind":"none","multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true},"origin":origin(&used)}],
            "operations":[{"identity":format!("{bin}/add"),"name":"add","params":[],"pre":[],"post":[],"origin":{"source":{"sourceIdentity":format!("{base}/source"),"path":"bins.spec","startLine":1,"startColumn":1}},"frame":{"modifies":[used],"creates":[],"deletes":[]}}],"relationships":[]}],
        "populations":[{"identity":population,"displayName":"Bins","kind":kind("population"),"members":[bin],"extent":"closed","origin":origin(&population)}]
    })
}

fn bin_fixture() -> Result<CliCase, ContractError> {
    let case = fixture()?;
    case.write("bin.fsl", BIN_SOURCE.as_bytes())?;
    write_json(&case, "bins.json", &bin_model())?;
    let mut model_source = source("bins.json");
    model_source["identity"] = json!("contracts/bins");
    write_json(
        &case,
        "start.json",
        &json!({"format":"filament.state.snapshot/v1","identity":{"authority":"export-contracts","identity":"contracts/bins/start"},"observation":"current","model":{"identity":"contracts/bins"},"anchor":{"kind":"initialization","name":"start"},"populations":[{"population":"ix://contracts/bins/bins","complete":true,"objects":[{"key":"bin-A","type":"ix://contracts/bins/Bin","fields":{"used":{"integer":"0"}}}]}]}),
    )?;
    write_json(
        &case,
        "export.json",
        &json!({"format":"native-run/1","request":{
        "models":[{"format":"semantic-ir/2.0.0","source":model_source}],"program":{"source":source("bin.fsl")},
        "export":{"profile":"filament.output.tla2/v1","obligations":["Capacity"],"initial":"start.json"}}}),
    )?;
    Ok(case)
}

fn assembled(output: &Captured) -> Value {
    let value = outcome(output, 0, "success");
    assert_eq!(value["last_stage"], "S4", "{value}");
    assert_eq!(value["result"]["kind"], "exported", "{value}");
    assert_eq!(value["result"]["export"]["status"], "assembled", "{value}");
    value
}

/// Trace: FR-027-AC-1
/// Provenance: QSL-2474
#[test]
#[ignore = "QSL-1979: source model export loses its authoritative model binding"]
fn source_export_output_option_and_request_member_write_the_actual_target(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = bin_fixture()?;
    let value = assembled(&run(
        &case,
        "export.json",
        false,
        &["--output", "option.tla"],
    )?);
    let target = value["result"]["export"]["target_text"]
        .as_str()
        .expect("assembled target");
    let written = std::fs::read(case.directory().join("option.tla"))?;
    assert_eq!(written, target.as_bytes());
    assert!(!written.is_empty());
    let export = &value["result"]["export"];
    assert_eq!(export["output"]["path"], "option.tla");
    assert_eq!(export["output"]["bytes_written"], written.len());
    assert!(target.contains("Capacity"), "{target}");
    let other = bin_fixture()?;
    let mut input: Value =
        serde_json::from_slice(&std::fs::read(other.directory().join("export.json"))?)?;
    input["request"]["export"]["output"] = json!({"file":"member.tla"});
    write_json(&other, "member.json", &input)?;
    assembled(&run(&other, "member.json", false, &[])?);
    assert_eq!(
        std::fs::read(other.directory().join("member.tla"))?,
        written
    );
    Ok(())
}

/// Trace: FR-027-AC-4
/// Provenance: QSL-2474
#[test]
#[ignore = "QSL-1979: source model export loses its authoritative model binding"]
fn assembled_text_names_exact_destination_and_written_byte_count(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = bin_fixture()?;
    let output = case.run(&["export", "export.json", "--output", "model.tla"])?;
    assert_eq!(output.0.status.code(), Some(0), "{output:?}");
    let written = std::fs::read(case.directory().join("model.tla"))?;
    assert_eq!(output.stdout_text()?,format!("export: success (exit 0) at S4\ntarget: tla filament.output.tla2/v1\nbytes written: {}\noutput: model.tla\n",written.len()));
    Ok(())
}

/// Trace: FR-027-AC-3
/// Provenance: QSL-2474
#[test]
#[ignore = "QSL-1979: model binding blocks target classification before export limits"]
fn target_byte_limit_prevents_partial_output_and_a_sufficient_limit_assembles(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = bin_fixture()?;
    let value = outcome(
        &run(
            &case,
            "export.json",
            false,
            &["--output", "tiny.tla", "--limit", "export.output_bytes=1"],
        )?,
        22,
        "incomplete",
    );
    assert!(!case.directory().join("tiny.tla").exists());
    assert_eq!(value["result"]["kind"], "exported", "{value}");
    assert_eq!(value["result"]["export"]["status"], "incomplete", "{value}");
    assert_eq!(
        value["result"]["export"]["cause"],
        json!({"type":"limit-reached","limit":"output_bytes","value":1,"setting":"export.output_bytes"}),
        "{value}"
    );
    assert_eq!(value["result"]["export"]["records"], json!([]), "{value}");
    assembled(&run(
        &case,
        "export.json",
        false,
        &[
            "--output",
            "fitting.tla",
            "--limit",
            "export.output_bytes=16777216",
        ],
    )?);
    assert!(!std::fs::read(case.directory().join("fitting.tla"))?.is_empty());
    Ok(())
}

/// Trace: FR-027-AC-1
/// Provenance: QSL-2474
/// FSL FR-1039: a public checked package replaces source for export.
#[test]
#[ignore = "QSL-2672: package export admission; QSL-1979: subsequent model export assembly"]
fn checked_package_export_uses_its_selected_source_reference_without_source_fallback(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = bin_fixture()?;
    let input: Value =
        serde_json::from_slice(&std::fs::read(case.directory().join("export.json"))?)?;
    write_json(
        &case,
        "compile.json",
        &json!({"format":"native-compile/1","request":{
        "models":input["request"]["models"],"program":input["request"]["program"],"output":{"package":"bin.package.json"}}}),
    )?;
    let compiled = case.run(&["compile", "compile.json", "--format", "json"])?;
    assert_eq!(compiled.0.status.code(), Some(0), "{compiled:?}");
    let mut packaged = input;
    packaged["request"]
        .as_object_mut()
        .expect("request")
        .remove("program");
    packaged["request"]["package"] = json!({"file":"bin.package.json","source_package":{"identity":"contracts:bin-capacity","version":"1"}});
    write_json(&case, "package-export.json", &packaged)?;
    std::fs::remove_file(case.directory().join("bin.fsl"))?;
    let value = assembled(&run(
        &case,
        "package-export.json",
        false,
        &["--output", "package.tla"],
    )?);
    let target = value["result"]["export"]["target_text"]
        .as_str()
        .expect("target text");
    assert_eq!(
        std::fs::read(case.directory().join("package.tla"))?,
        target.as_bytes()
    );
    assert_eq!(
        value["result"]["export"]["package"]["source_package"],
        json!({"identity":"contracts:bin-capacity","version":"1"}),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-1, FSL-FR-286-AC-23
/// Provenance: QSL-2474
#[test]
#[ignore = "QSL-1979: model binding blocks assembly before destination-write failure"]
fn unavailable_output_parent_returns_internal_failure_without_partial_target(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = bin_fixture()?;
    let value = outcome(
        &run(
            &case,
            "export.json",
            false,
            &["--output", "missing-parent/model.tla"],
        )?,
        30,
        "internal-failure",
    );
    assert_eq!(value["last_stage"], "S4", "{value}");
    let d = diagnostic(&value, "output-failure");
    assert_eq!(d["cause"], "output-failure", "{value}");
    assert!(value["result"].is_null());
    assert!(!case.directory().join("missing-parent/model.tla").exists());
    Ok(())
}
