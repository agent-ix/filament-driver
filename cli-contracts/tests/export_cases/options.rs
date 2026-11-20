use super::*;

/// Trace: FR-002-AC-25
/// Provenance: QSL-2474
#[test]
fn help_describes_both_routes_and_needs_no_request_file() -> Result<(), Box<dyn std::error::Error>>
{
    let case = fixture()?;
    for args in [
        vec!["export", "--help"],
        vec!["export", "-h"],
        vec!["export", "missing.json", "--help"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(0), "{output:?}");
        assert!(output.0.stderr.is_empty());
        let text = output.stdout_text()?;
        assert!(
            text.starts_with("Usage: fsl export [options] <request-file>\n"),
            "{text}"
        );
        for form in [
            "fsl export --evidence <request-file>",
            "export options:",
            "--output <file>",
            "--evidence <request-file>",
            "--limit name=value (repeatable)",
            "--format text|json",
        ] {
            assert!(text.contains(form), "{text}");
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-24, FR-002-AC-26
/// Provenance: QSL-2474
#[test]
fn foreign_options_refuse_before_reading_the_named_request(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for name in [
        "--bogus",
        "--version",
        "-V",
        "--color",
        "--model",
        "--library",
        "--engine",
        "--provider-config",
        "--provider-limit",
        "--clocks",
        "--request-bytes",
        "--dependent-bytes",
        "--markdown",
        "--extensions",
        "--input",
    ] {
        invalid(&run(&case, "missing.json", false, &[name])?, &[name]);
    }
    Ok(())
}

/// Trace: FR-027-AC-5
/// Provenance: QSL-2474
#[test]
fn evidence_and_export_operands_are_mutually_exclusive_and_write_nothing(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for args in [
        vec![
            "export",
            "--evidence",
            "missing-evidence.json",
            "--output",
            "unused.tla",
            "--format",
            "json",
        ],
        vec![
            "export",
            "missing-export.json",
            "--evidence",
            "missing-evidence.json",
            "--format",
            "json",
        ],
    ] {
        invalid(&case.run(&args)?, &["--evidence"]);
        assert!(!case.directory().join("unused.tla").exists());
    }
    Ok(())
}

/// Trace: FR-027-AC-2
/// Provenance: QSL-2474
#[test]
fn output_override_repeated_or_conflicting_with_request_is_refused_without_files(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    invalid(
        &run(
            &case,
            "missing.json",
            false,
            &["--output", "first.tla", "--output", "second.tla"],
        )?,
        &["--output"],
    );
    let mut input = request();
    input["request"]["export"]["output"] = json!({"file":"member.tla"});
    write_json(&case, "twice.json", &input)?;
    invalid(
        &run(&case, "twice.json", false, &["--output", "override.tla"])?,
        &["output", "--output"],
    );
    for path in ["first.tla", "second.tla", "member.tla", "override.tla"] {
        assert!(!case.directory().join(path).exists(), "{path}");
    }
    Ok(())
}

/// Trace: FR-027-AC-2
/// Provenance: QSL-2474
#[test]
fn existing_file_and_directory_outputs_are_never_overwritten(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    case.write("existing.tla", b"original target bytes\n")?;
    case.write("existing-dir/anchor", b"preserve child")?;
    for path in ["existing.tla", "existing-dir"] {
        invalid(
            &run(&case, "export.json", false, &["--output", path])?,
            &[path],
        );
    }
    assert_eq!(
        std::fs::read(case.directory().join("existing.tla"))?,
        b"original target bytes\n"
    );
    assert_eq!(
        std::fs::read(case.directory().join("existing-dir/anchor"))?,
        b"preserve child"
    );
    Ok(())
}

/// Trace: FR-003-AC-2, FR-003-AC-7, FR-003-AC-8
/// Provenance: QSL-2474
#[test]
fn malformed_unknown_and_duplicate_limits_refuse_before_request_io(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for operand in [
        "export.output_bytes",
        "export.nodes=-1",
        "export.nodes=many",
        "unknown.budget=1",
    ] {
        invalid(
            &run(&case, "missing.json", false, &["--limit", operand])?,
            &[operand.split('=').next().expect("name")],
        );
    }
    invalid(
        &run(
            &case,
            "missing.json",
            false,
            &["--limit", "export.nodes=1", "--limit", "export.nodes=2"],
        )?,
        &["export.nodes"],
    );
    Ok(())
}

/// Trace: FR-003-AC-3
/// Provenance: QSL-2474
#[test]
fn front_end_limit_stops_before_export_semantics_and_leaves_no_artifact(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let value = outcome(
        &run(
            &case,
            "export.json",
            false,
            &["--limit", "s1.nodes=0", "--output", "bounded.tla"],
        )?,
        22,
        "incomplete",
    );
    let d = diagnostic(&value, "stage_limit_exceeded");
    assert_eq!(d["cause"], "node-count-exceeded", "{value}");
    let message = d["message"].as_str().expect("message");
    assert!(
        message.contains("limit 0 reached") && message.contains("s1.nodes"),
        "{value}"
    );
    assert!(!case.directory().join("bounded.tla").exists());
    Ok(())
}
