use super::*;

/// Trace: FR-002-AC-13, FR-002-AC-18
/// Provenance: QSL-2471
#[test]
fn default_and_explicit_text_match_json_status_and_preserve_value_and_refusal(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for (path, exit, category) in [("run.json", 0, "success"), ("missing.json", 20, "refusal")] {
        let machine = outcome(&run(&case, path, &[])?, exit, category);
        let default = case.run(&["run", path])?;
        let explicit = case.run(&["run", path, "--format", "text"])?;
        assert_eq!(default.0.status.code(), Some(exit));
        assert_eq!(explicit.0.status.code(), Some(exit));
        assert_eq!(default.0.stdout, explicit.0.stdout);
        assert!(default.0.stderr.is_empty());
        let text = default.stdout_text()?;
        assert!(
            text.starts_with(&format!("execute: {category} (exit {exit})")),
            "{text}"
        );
        if exit == 0 {
            assert!(text.contains("12"), "{text}");
        } else {
            for d in machine["diagnostics"].as_array().expect("diagnostics") {
                assert!(text.contains(d["code"].as_str().expect("code")), "{text}");
            }
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2471
#[test]
fn help_needs_no_request_and_repeated_single_options_write_only_usage(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for args in [
        vec!["run", "--help"],
        vec!["run", "-h"],
        vec!["run", "missing.json", "--help"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(0));
        assert!(output.0.stderr.is_empty());
        let text = output.stdout_text()?;
        assert!(text.starts_with("Usage: fsl run"));
        for name in [
            "--request-bytes",
            "--dependent-bytes",
            "--limit",
            "--engine",
            "--provider-config",
            "--provider-limit",
            "--clocks",
            "--format",
        ] {
            assert!(text.contains(name), "{text}");
        }
    }
    for (name, value) in [
        ("--request-bytes", "1024"),
        ("--dependent-bytes", "1024"),
        ("--engine", "interpreter"),
        ("--clocks", "clocks.json"),
        ("--provider-config", "providers.json"),
        ("--format", "json"),
    ] {
        let output = case.run(&["run", "missing.json", name, value, name, value])?;
        assert_eq!(output.0.status.code(), Some(20), "{output:?}");
        assert!(output.0.stdout.is_empty(), "{output:?}");
        assert!(
            output.stderr_text()?.contains("usage: fsl run"),
            "{output:?}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-24, FR-002-AC-26
/// Provenance: QSL-2471
#[test]
fn foreign_options_and_invalid_option_values_refuse_before_request_io(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for flag in [
        "--bogus",
        "--version",
        "-V",
        "--color",
        "--model",
        "--library",
        "--output",
        "--markdown",
        "--extensions",
        "--integer-division",
        "--input",
    ] {
        let value = outcome(&run(&case, "missing.json", &[flag])?, 20, "refusal");
        assert!(
            diagnostic(&value, "invalid-request")["message"]
                .as_str()
                .expect("message")
                .contains(flag),
            "{value}"
        );
    }
    for (flag, values) in [
        (
            "--request-bytes",
            vec!["-1", "many", "18446744073709551616"],
        ),
        (
            "--dependent-bytes",
            vec!["-1", "many", "18446744073709551616"],
        ),
        ("--engine", vec!["native", "Interpreter", ""]),
    ] {
        for operand in values {
            let value = outcome(
                &run(&case, "missing.json", &[flag, operand])?,
                20,
                "refusal",
            );
            assert!(
                diagnostic(&value, "invalid-request")["message"]
                    .as_str()
                    .expect("message")
                    .contains(flag),
                "{value}"
            );
        }
    }
    Ok(())
}

/// Trace: FR-003-AC-2, FR-003-AC-7, FR-003-AC-8
/// Provenance: QSL-2471
#[test]
fn malformed_unknown_and_duplicate_limit_operands_refuse() -> Result<(), Box<dyn std::error::Error>>
{
    let case = fixture()?;
    for setting in [
        "unknown.budget=1",
        "s1.nodes",
        "s1.nodes=-1",
        "s1.nodes=many",
        "work_units=1",
    ] {
        let value = outcome(
            &run(&case, "missing.json", &["--limit", setting])?,
            20,
            "refusal",
        );
        let d = diagnostic(&value, "invalid-request");
        assert!(
            d["message"]
                .as_str()
                .expect("message")
                .contains(setting.split('=').next().expect("name")),
            "{value}"
        );
    }
    let value = outcome(
        &run(
            &case,
            "missing.json",
            &["--limit", "s1.nodes=5", "--limit", "s1.nodes=6"],
        )?,
        20,
        "refusal",
    );
    assert!(
        diagnostic(&value, "invalid-request")["message"]
            .as_str()
            .expect("message")
            .contains("s1.nodes"),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-003-AC-1, FR-003-AC-3
/// Provenance: QSL-2471
#[test]
fn front_end_limits_cut_the_actual_run_and_distinct_sufficient_limits_compose(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for setting in ["s1.nodes", "s1.tokens", "s3.nodes"] {
        let operand = format!("{setting}=0");
        let value = outcome(
            &run(&case, "run.json", &["--limit", &operand])?,
            22,
            "incomplete",
        );
        assert!(value["result"].is_null(), "{value}");
        assert!(
            value["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .iter()
                .any(|d| d["message"].as_str().is_some_and(|m| m.contains(setting))),
            "{value}"
        );
    }
    completed(
        &run(
            &case,
            "run.json",
            &[
                "--engine",
                "interpreter",
                "--limit",
                "s1.nodes=50000",
                "--limit",
                "s1.tokens=100000",
            ],
        )?,
        json!({"integer":"12"}),
    );
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn empty_provider_configuration_runs_the_function_and_local_flags_do_not_silently_disappear(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    write_json(
        &case,
        "providers.json",
        &json!({"format":"filament.provider-config/v1","providers":[]}),
    )?;
    completed(
        &run(&case, "run.json", &["--provider-config", "providers.json"])?,
        json!({"integer":"12"}),
    );
    for (flag, value) in [
        ("--request-bytes", "10000"),
        ("--dependent-bytes", "10000"),
        ("--limit", "s1.nodes=50000"),
        ("--clocks", "clocks.json"),
    ] {
        let result = outcome(
            &run(
                &case,
                "run.json",
                &["--provider-config", "providers.json", flag, value],
            )?,
            20,
            "refusal",
        );
        let message = diagnostic(&result, "invalid-request")["message"]
            .as_str()
            .expect("message");
        assert!(
            message.contains(flag) && message.contains("--provider-config"),
            "{result}"
        );
    }
    Ok(())
}
