use super::*;

fn model_program(identity: &str) -> String {
    format!(
        r#"language "ix:native" edition "1-draft";
profile m = "filament.model.complete/v1";
model Parcels = "{identity}";
invariant ParcelPresent using m on Parcels::Parcel at current {{ true }}
"#
    )
}

/// Trace: FR-002-AC-1, FR-002-AC-25
/// Provenance: QSL-2469
#[test]
fn selected_library_uses_cwd_paths_and_checks_the_actual_library_body() -> Result<(), ContractError>
{
    let case = case()?;
    case.write("rules/charge.fsl", IMPORTER.as_bytes())?;
    case.write("library folder/fees=west.fsl", LIBRARY.as_bytes())?;
    let args = [
        "check",
        "rules/charge.fsl",
        "--library",
        "contracts/fees=library folder/fees=west.fsl",
        "--format",
        "json",
    ];
    success(&case.run(&args)?);
    case.write(
        "library folder/fees=west.fsl",
        LIBRARY.replace("amount + 4", "true").as_bytes(),
    )?;
    let value = document(&case.run(&args)?, 20, "refusal");
    diagnostic(&value, "ill_typed");
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn imported_library_without_a_selection_is_a_missing_import() -> Result<(), ContractError> {
    let case = case()?;
    case.write("charge.fsl", IMPORTER.as_bytes())?;
    let value = document(
        &case.run(&["check", "charge.fsl", "--format", "json"])?,
        20,
        "refusal",
    );
    let d = diagnostic(&value, "missing_import");
    assert!(d["message"]
        .as_str()
        .expect("message")
        .contains("contracts/fees"));
    Ok(())
}

/// Trace: FR-002-AC-1, FR-002-AC-25
/// Provenance: QSL-2469
#[test]
fn selected_model_type_must_come_from_the_supplied_domain_document() -> Result<(), ContractError> {
    let case = case()?;
    let identity = "contracts/parcels";
    case.write("parcel.fsl", model_program(identity).as_bytes())?;
    let mut model = parcel_model(identity);
    case.write("parcel-model.json", &serde_json::to_vec(&model)?)?;
    let args = [
        "check",
        "parcel.fsl",
        "--model",
        "contracts/parcels=parcel-model.json",
        "--format",
        "json",
    ];
    success(&case.run(&args)?);
    let renamed = format!("ix://{identity}/DifferentParcel");
    model["types"][0]["identity"] = json!(renamed);
    model["types"][0]["displayName"] = json!("DifferentParcel");
    model["types"][0]["origin"]["generated"]["inputIdentities"] = json!([renamed]);
    case.write("parcel-model.json", &serde_json::to_vec(&model)?)?;
    let value = document(&case.run(&args)?, 20, "refusal");
    assert!(
        value["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["message"].as_str().is_some_and(|m| m.contains("Parcel"))),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-1, FR-002-AC-25
/// Provenance: QSL-2469
#[test]
fn repeatable_model_and_library_selections_compose_without_replacing_each_other(
) -> Result<(), ContractError> {
    let case = case()?;
    let source = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
profile m = "filament.model.complete/v1";
import "contracts/fees" as Fees;
import "contracts/rebates" as Rebates;
model First = "contracts/first";
model Second = "contracts/second";
invariant FirstPresent using m on First::Parcel at current { true }
invariant SecondPresent using m on Second::Parcel at current { true }
function Net using v(amount: Integer): Integer pure {
    Fees::WithFee(amount) - Rebates::Reduction(amount)
}
"#;
    case.write("net.fsl", source.as_bytes())?;
    for (identity, path) in [
        ("contracts/first", "first.json"),
        ("contracts/second", "second.json"),
    ] {
        case.write(path, &serde_json::to_vec(&parcel_model(identity))?)?;
    }
    case.write("fees.fsl", LIBRARY.as_bytes())?;
    case.write(
        "rebates.fsl",
        LIBRARY
            .replace("WithFee", "Reduction")
            .replace("amount + 4", "amount - 1")
            .as_bytes(),
    )?;
    success(&case.run(&[
        "check",
        "net.fsl",
        "--model",
        "contracts/first=first.json",
        "--model",
        "contracts/second=second.json",
        "--library",
        "contracts/fees=fees.fsl",
        "--library",
        "contracts/rebates=rebates.fsl",
        "--format",
        "json",
    ])?);
    Ok(())
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2469
#[test]
fn malformed_and_repeated_identity_bindings_refuse_before_source_io() -> Result<(), ContractError> {
    let case = case()?;
    for option in ["--model", "--library"] {
        for bad in ["bare.fsl", "=file.fsl", "contracts/id=", ""] {
            let value = document(
                &case.run(&[
                    "check",
                    "missing-source.fsl",
                    option,
                    bad,
                    "--format",
                    "json",
                ])?,
                20,
                "refusal",
            );
            let d = diagnostic(&value, "invalid-request");
            assert!(
                d["message"].as_str().expect("message").contains(option),
                "{value}"
            );
        }
        for second in ["contracts/duplicate=one.fsl", "contracts/duplicate=two.fsl"] {
            let value = document(
                &case.run(&[
                    "check",
                    "missing-source.fsl",
                    option,
                    "contracts/duplicate=one.fsl",
                    option,
                    second,
                    "--format",
                    "json",
                ])?,
                20,
                "refusal",
            );
            let d = diagnostic(&value, "invalid-request");
            let message = d["message"].as_str().expect("message");
            assert!(
                message.contains(option) && message.contains("contracts/duplicate"),
                "{value}"
            );
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2469
#[test]
fn unused_selections_are_refused_before_their_missing_files_are_read() -> Result<(), ContractError>
{
    let case = case()?;
    for option in ["--model", "--library"] {
        let value = document(
            &case.run(&[
                "check",
                "eligible.fsl",
                option,
                "contracts/unused=missing-file",
                "--format",
                "json",
            ])?,
            20,
            "refusal",
        );
        let d = diagnostic(&value, "invalid-request");
        let message = d["message"].as_str().expect("message");
        assert!(
            message.contains(option) && message.contains("contracts/unused"),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn missing_model_selection_names_the_identity_and_both_supply_routes() -> Result<(), ContractError>
{
    let case = case()?;
    case.write("parcel.fsl", model_program("contracts/parcels").as_bytes())?;
    let value = document(
        &case.run(&["check", "parcel.fsl", "--format", "json"])?,
        20,
        "refusal",
    );
    let d = diagnostic(&value, "missing_import");
    assert_eq!(d["cause"], "missing-selection");
    let message = d["message"].as_str().expect("message");
    for required in ["contracts/parcels", "--model", "models"] {
        assert!(message.contains(required), "{value}");
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn dependent_bytes_counts_dependencies_but_not_the_primary_source() -> Result<(), ContractError> {
    let case = case()?;
    success(&case.run(&[
        "check",
        "eligible.fsl",
        "--dependent-bytes",
        "0",
        "--format",
        "json",
    ])?);
    case.write("charge.fsl", IMPORTER.as_bytes())?;
    case.write("fees.fsl", LIBRARY.as_bytes())?;
    let exact = LIBRARY.len().to_string();
    success(&case.run(&[
        "check",
        "charge.fsl",
        "--library",
        "contracts/fees=fees.fsl",
        "--dependent-bytes",
        &exact,
        "--format",
        "json",
    ])?);
    let lower = (LIBRARY.len() - 1).to_string();
    let value = document(
        &case.run(&[
            "check",
            "charge.fsl",
            "--library",
            "contracts/fees=fees.fsl",
            "--dependent-bytes",
            &lower,
            "--format",
            "json",
        ])?,
        22,
        "incomplete",
    );
    assert!(
        value["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["message"]
                .as_str()
                .is_some_and(|m| m.contains("dependent_bytes") && m.contains(&lower))),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn malformed_selected_model_and_non_utf8_library_are_input_refusals() -> Result<(), ContractError> {
    let case = case()?;
    case.write("parcel.fsl", model_program("contracts/parcels").as_bytes())?;
    case.write("model.json", b"{")?;
    let value = document(
        &case.run(&[
            "check",
            "parcel.fsl",
            "--model",
            "contracts/parcels=model.json",
            "--format",
            "json",
        ])?,
        20,
        "refusal",
    );
    assert!(!value["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .is_empty());
    case.write("charge.fsl", IMPORTER.as_bytes())?;
    case.write("bad-library.fsl", &[0xff])?;
    let value = document(
        &case.run(&[
            "check",
            "charge.fsl",
            "--library",
            "contracts/fees=bad-library.fsl",
            "--format",
            "json",
        ])?,
        20,
        "refusal",
    );
    diagnostic(&value, "invalid_utf8");
    Ok(())
}
