use super::*;

const DIVISION: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function Quotient using v(n: Integer): Integer pure { n div 3 }
function Remainder using v(n: Integer): Integer pure { n rem 3 }
"#;

/// Trace: FR-002-AC-13
/// Provenance: QSL-2469
#[test]
fn json_text_and_default_have_equal_success_and_refusal_exits() -> Result<(), ContractError> {
    let case = case()?;
    case.write(
        "wrong-result.fsl",
        UNIT.replace("n >= 3", "n + 3").as_bytes(),
    )?;
    for (path, exit, category) in [
        ("eligible.fsl", 0, "success"),
        ("wrong-result.fsl", 20, "refusal"),
    ] {
        let machine = case.run(&["check", path, "--format", "json"])?;
        let value = document(&machine, exit, category);
        if exit == 20 {
            diagnostic(&value, "ill_typed");
        }
        let default = case.run(&["check", path])?;
        let text = case.run(&["check", path, "--format", "text"])?;
        assert_eq!(default.0.status.code(), Some(exit));
        assert_eq!(text.0.status.code(), Some(exit));
        assert_eq!(default.0.stdout, text.0.stdout);
        assert_eq!(default.0.stderr, text.0.stderr);
        assert!(text
            .stdout_text()?
            .starts_with(&format!("check: {category} (exit {exit})")));
        if exit == 20 {
            assert!(text.stdout_text()?.contains("ill_typed"));
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-1, FR-002-AC-24
/// Provenance: QSL-2469
#[test]
fn each_integer_division_law_is_accepted_and_missing_law_is_not_defaulted(
) -> Result<(), ContractError> {
    let case = case()?;
    case.write("division.fsl", DIVISION.as_bytes())?;
    for law in ["euclidean", "floor", "truncating"] {
        success(&case.run(&[
            "check",
            "division.fsl",
            "--integer-division",
            law,
            "--format",
            "json",
        ])?);
    }
    let value = document(
        &case.run(&["check", "division.fsl", "--format", "json"])?,
        20,
        "refusal",
    );
    let diagnostics = value["diagnostics"].as_array().expect("diagnostics");
    assert_eq!(diagnostics.len(), 2, "one refusal at each div/rem: {value}");
    assert!(
        diagnostics
            .iter()
            .all(|d| d["code"] == "missing_declaration" && d["cause"] == "missing-selection"),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn caller_law_can_agree_with_but_cannot_override_the_header() -> Result<(), ContractError> {
    let case = case()?;
    let source=DIVISION.replace("profile v = \"filament.value.complete/v1\";","profile v = \"filament.value.complete/v1\";\nprofile d = \"filament.value.integer-division.floor/v1\";");
    case.write("floor.fsl", source.as_bytes())?;
    success(&case.run(&[
        "check",
        "floor.fsl",
        "--integer-division",
        "floor",
        "--format",
        "json",
    ])?);
    let value = document(
        &case.run(&[
            "check",
            "floor.fsl",
            "--integer-division",
            "truncating",
            "--format",
            "json",
        ])?,
        20,
        "refusal",
    );
    let d = diagnostic(&value, "invalid_package");
    assert_eq!(d["cause"], "conflicting-definition");
    let message = d["message"].as_str().expect("message");
    for value in ["floor", "truncating"] {
        assert!(message.contains(value), "{d}");
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn unknown_law_names_refuse_before_source_io() -> Result<(), ContractError> {
    let case = case()?;
    for law in [
        "ceiling",
        "Floor",
        "filament.value.integer-division.floor/v1",
    ] {
        let value = document(
            &case.run(&[
                "check",
                "missing.fsl",
                "--integer-division",
                law,
                "--format",
                "json",
            ])?,
            20,
            "refusal",
        );
        let d = diagnostic(&value, "unknown_profile");
        assert_eq!(d["cause"], "unsupported-selection");
        let message = d["message"].as_str().expect("message");
        assert!(
            message.contains(law) && message.contains("--integer-division"),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2469
#[test]
fn repeated_single_valued_options_and_wrong_arity_write_usage() -> Result<(), ContractError> {
    let case = case()?;
    for args in [
        vec!["check"],
        vec!["check", "one", "two"],
        vec!["check", "a", "b", "c", "d"],
        vec![
            "check",
            "missing.fsl",
            "--integer-division",
            "floor",
            "--integer-division",
            "floor",
        ],
        vec![
            "check",
            "missing.fsl",
            "--dependent-bytes",
            "1",
            "--dependent-bytes",
            "2",
        ],
        vec!["check", "--markdown", "--markdown", "a", "b", "missing.md"],
        vec![
            "check",
            "missing.fsl",
            "--extensions",
            "one.json",
            "--extensions",
            "two.json",
        ],
    ] {
        argv_error(&case.run(&args)?, "usage: fsl check");
    }
    Ok(())
}

/// Trace: FR-002-AC-24, FR-002-AC-26
/// Provenance: QSL-2469
#[test]
fn foreign_options_are_refused_before_missing_source_is_read() -> Result<(), ContractError> {
    let case = case()?;
    for option in [
        "--bogus",
        "--version",
        "-V",
        "--color",
        "--engine",
        "--output",
        "--provider-config",
        "--input",
        "--clocks",
        "--request-bytes",
    ] {
        let value = document(
            &case.run(&["check", "missing.fsl", option, "--format", "json"])?,
            20,
            "refusal",
        );
        let d = diagnostic(&value, "invalid-request");
        assert!(
            d["message"].as_str().expect("message").contains(option),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn malformed_limits_and_byte_counts_are_not_silently_coerced() -> Result<(), ContractError> {
    let case = case()?;
    for value in ["-1", "many", "18446744073709551616"] {
        let value = document(
            &case.run(&[
                "check",
                "missing.fsl",
                "--dependent-bytes",
                value,
                "--format",
                "json",
            ])?,
            20,
            "refusal",
        );
        let d = diagnostic(&value, "invalid-request");
        assert!(
            d["message"]
                .as_str()
                .expect("message")
                .contains("--dependent-bytes"),
            "{value}"
        );
    }
    for limit in [
        "s1.nodes",
        "s1.nodes=-1",
        "s1.nodes=many",
        "unknown.budget=1",
        "s1.nodes=18446744073709551616",
    ] {
        let value = document(
            &case.run(&["check", "missing.fsl", "--limit", limit, "--format", "json"])?,
            20,
            "refusal",
        );
        diagnostic(&value, "invalid-request");
    }
    let value = document(
        &case.run(&[
            "check",
            "missing.fsl",
            "--limit",
            "s1.nodes=1",
            "--limit",
            "s1.nodes=2",
            "--format",
            "json",
        ])?,
        20,
        "refusal",
    );
    assert!(diagnostic(&value, "invalid-request")["message"]
        .as_str()
        .expect("message")
        .contains("s1.nodes"));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn frontend_limits_are_applied_and_distinct_limits_compose() -> Result<(), ContractError> {
    let case = case()?;
    for setting in [
        "s1.tokens",
        "s1.nodes",
        "s1.work_units",
        "s3.input_bytes",
        "s3.nodes",
        "s3.work_units",
    ] {
        let bound = format!("{setting}=0");
        let value = document(
            &case.run(&[
                "check",
                "eligible.fsl",
                "--limit",
                &bound,
                "--format",
                "json",
            ])?,
            22,
            "incomplete",
        );
        assert!(
            !value["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .is_empty(),
            "{setting}: {value}"
        );
    }
    success(&case.run(&[
        "check",
        "eligible.fsl",
        "--limit",
        "s1.nodes=50000",
        "--limit",
        "s1.tokens=100000",
        "--format",
        "json",
    ])?);
    Ok(())
}

/// Trace: FR-002-AC-1, FR-002-AC-24
/// Provenance: QSL-2469
#[test]
fn empty_extension_set_composes_with_division_limits_and_byte_options() -> Result<(), ContractError>
{
    let case = case()?;
    case.write("division.fsl", DIVISION.as_bytes())?;
    case.write(
        "empty-extensions.json",
        br#"{"format":"filament.extension-set/v1","extensions":[]}"#,
    )?;
    success(&case.run(&[
        "check",
        "division.fsl",
        "--extensions",
        "empty-extensions.json",
        "--integer-division",
        "floor",
        "--limit",
        "s1.nodes=50000",
        "--dependent-bytes",
        "8192",
        "--format",
        "json",
    ])?);
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn extension_reader_rejects_wrong_formats_unknown_fields_and_wrong_kinds(
) -> Result<(), ContractError> {
    let case = case()?;
    for payload in [
        json!({"format":"not-an-extension-set","extensions":[]}),
        json!({"format":"filament.extension-set/v1","extensions":[],"extra":true}),
        json!({"format":"filament.extension-set/v1","extensions":"not-an-array"}),
    ] {
        case.write("bad-extensions.json", &serde_json::to_vec(&payload)?)?;
        let value = document(
            &case.run(&[
                "check",
                "missing.fsl",
                "--extensions",
                "bad-extensions.json",
                "--format",
                "json",
            ])?,
            20,
            "refusal",
        );
        diagnostic(&value, "invalid-request");
    }
    Ok(())
}
