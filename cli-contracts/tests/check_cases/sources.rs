use super::*;

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn content_not_filename_extension_and_labelled_operands_select_native_source(
) -> Result<(), ContractError> {
    let case = case()?;
    for path in ["notes.txt", "sub folder/café.fsl"] {
        case.write(path, UNIT.as_bytes())?;
        success(&case.run(&["check", path, "--format", "json"])?);
        success(&case.run(&[
            "check",
            "contracts",
            "labelled-unit",
            path,
            "--format",
            "json",
        ])?);
    }
    let absolute = case.directory().join("eligible.fsl");
    success(&case.run(&[
        "check",
        absolute.to_str().expect("UTF8 temporary path"),
        "--format",
        "json",
    ])?);
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn file_operand_stays_authoritative_when_stdin_contains_different_bytes(
) -> Result<(), ContractError> {
    let case = case()?;
    success(&case.run_stdin(
        &["check", "eligible.fsl", "--format", "json"],
        b"not native\0\xff",
    )?);
    case.write("-", UNIT.as_bytes())?;
    success(&case.run_stdin(&["check", "./-", "--format", "json"], b"invalid stdin")?);
    Ok(())
}

/// Trace: FR-002-AC-19
/// Provenance: QSL-2469
#[test]
fn malformed_source_keeps_the_callers_path_and_matches_public_parse_diagnostics(
) -> Result<(), ContractError> {
    let case = case()?;
    let path = "source dir/échec.fsl";
    let broken = UNIT.replace("n: Int[0, 12]", ": Int[0, 12]");
    case.write(path, broken.as_bytes())?;
    let checked = case.run(&["check", path, "--format", "json"])?;
    let value = document(&checked, 20, "refusal");
    let diagnostics = value["diagnostics"].as_array().expect("diagnostics");
    assert!(!diagnostics.is_empty());
    for diagnostic in diagnostics {
        source_locus(diagnostic, path);
    }
    assert!(!checked.stdout_text()?.contains("fsl-command"));
    assert!(!checked.stdout_text()?.contains("check-source"));
    let parsed = case.run(&["parse", "filament", path, path, "--format", "json"])?;
    assert_eq!(parsed.0.status.code(), checked.0.status.code());
    let parsed = parsed.json()?;
    assert_eq!(parsed["category"], value["category"]);
    assert_eq!(
        parsed["diagnostics"], value["diagnostics"],
        "same bytes and labels have the same canonical diagnostic values"
    );
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn invalid_utf8_bom_and_nul_keep_exact_source_byte_regions() -> Result<(), ContractError> {
    let case = case()?;
    let prefix = "// café\n".as_bytes();
    let mut utf8 = prefix.to_vec();
    utf8.push(0xff);
    let mut nul = prefix.to_vec();
    nul.push(0);
    nul.extend_from_slice(UNIT.as_bytes());
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(UNIT.as_bytes());
    for (path, bytes, code, start, end) in [
        (
            "invalid-utf8.fsl",
            utf8,
            "invalid_utf8",
            prefix.len(),
            prefix.len(),
        ),
        ("bom.fsl", bom, "invalid_syntax", 0, 3),
        (
            "nul.fsl",
            nul,
            "invalid_syntax",
            prefix.len(),
            prefix.len() + 1,
        ),
    ] {
        case.write(path, &bytes)?;
        let value = document(
            &case.run(&["check", path, "--format", "json"])?,
            20,
            "refusal",
        );
        let d = diagnostic(&value, code);
        source_locus(d, path);
        assert_eq!(d["locus"]["start"], start);
        assert_eq!(d["locus"]["end"], end);
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn missing_empty_and_directory_inputs_are_real_refusals() -> Result<(), ContractError> {
    let case = case()?;
    case.write("empty.fsl", b"")?;
    case.write("directory/anchor", b"")?;
    for path in ["missing.fsl", "empty.fsl", "directory"] {
        let value = document(
            &case.run(&["check", path, "--format", "json"])?,
            20,
            "refusal",
        );
        assert!(
            !value["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .is_empty(),
            "{path}: {value}"
        );
    }
    Ok(())
}

fn padded_source(size: usize) -> Vec<u8> {
    let mut bytes = b"//".to_vec();
    bytes.resize(size - UNIT.len() - 1, b'x');
    bytes.push(b'\n');
    bytes.extend_from_slice(UNIT.as_bytes());
    assert_eq!(bytes.len(), size);
    bytes
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn default_source_byte_ceiling_admits_the_boundary_and_reports_one_byte_over(
) -> Result<(), ContractError> {
    let case = case()?;
    const LIMIT: usize = 1_048_576; // FSL FR-255 s1.input_bytes default.
    for size in [LIMIT - 1, LIMIT] {
        case.write("boundary.fsl", &padded_source(size))?;
        success(&case.run(&["check", "boundary.fsl", "--format", "json"])?);
    }
    case.write("over.fsl", &padded_source(LIMIT + 1))?;
    let value = document(
        &case.run(&["check", "over.fsl", "--format", "json"])?,
        22,
        "incomplete",
    );
    assert!(
        value["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["code"] == "stage_limit_exceeded" || d["code"] == "resource_exhausted"),
        "{value}"
    );
    assert!(
        value["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .all(|d| d["locus"].is_null()),
        "source ceiling has no invented byte-zero region: {value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn a_trailing_declaration_form_is_unsupported_not_a_syntax_refusal() -> Result<(), ContractError> {
    let case = case()?;
    case.write("trailing.fsl", format!("{UNIT}stray_name\n").as_bytes())?;
    let value = document(
        &case.run(&["check", "trailing.fsl", "--format", "json"])?,
        21,
        "unsupported",
    );
    let d = diagnostic(&value, "unsupported_construct");
    assert_eq!(d["cause"], "declaration-form");
    source_locus(d, "trailing.fsl");
    Ok(())
}
