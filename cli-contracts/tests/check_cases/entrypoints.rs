use super::*;

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn header_only_is_an_empty_package() -> Result<(), ContractError> {
    let case = case()?;
    for source in ["language \"ix:native\" edition \"1-draft\";\n","language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\n"] {
        case.write("header.fsl",source.as_bytes())?;
        let value=document(&case.run(&["check","header.fsl","--format","json"])?,20,"refusal");
        let d=diagnostic(&value,"missing_declaration");
        assert_eq!(d["cause"],"empty-package");
        assert_eq!(value["last_stage"],"S3");
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
/// FSL FR-910 AC-2/3: aliases have no package node; dimensions do.
#[test]
fn alias_only_refuses_but_a_dimension_supplies_a_package_node() -> Result<(), ContractError> {
    let case = case()?;
    let header = "language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\n";
    case.write(
        "aliases.fsl",
        format!("{header}type BatchSize = Int[0, 8];\n").as_bytes(),
    )?;
    let value = document(
        &case.run(&["check", "aliases.fsl", "--format", "json"])?,
        20,
        "refusal",
    );
    let d = diagnostic(&value, "missing_declaration");
    assert_eq!(d["cause"], "empty-package");
    assert_eq!(value["last_stage"], "S3");
    source_locus(d, "aliases.fsl");
    assert_eq!(d["locus"]["start"], 0);
    assert!(d["message"]
        .as_str()
        .expect("message")
        .contains("type alias"));
    case.write(
        "dimensions.fsl",
        format!("{header}dimension Length;\n").as_bytes(),
    )?;
    success(&case.run(&["check", "dimensions.fsl", "--format", "json"])?);
    Ok(())
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2469
#[test]
fn delivered_check_help_succeeds_without_reading_a_source_operand() -> Result<(), ContractError> {
    let case = case()?;
    for args in [
        vec!["check", "--help"],
        vec!["check", "-h"],
        vec!["check", "missing.fsl", "--help"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(0));
        assert!(output.0.stderr.is_empty());
        let text = output.stdout_text()?;
        assert!(text.starts_with("Usage: fsl check"));
        for line in [
            "check options:",
            "--model <identity>=<file>",
            "--library <identity>=<file>",
            "--integer-division",
            "--extensions",
        ] {
            assert!(text.contains(line), "{text}");
        }
    }
    Ok(())
}
