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
            text.starts_with(&format!("run: {category} (exit {exit})")),
            "{text}"
        );
        if exit == 0 {
            assert_eq!(
                machine["result"],
                json!({"kind":"completed","value":{"type":"integer","value":"12"}})
            );
            for rendered in [default.stdout_text()?, explicit.stdout_text()?] {
                let results = rendered
                    .lines()
                    .filter_map(|line| line.strip_prefix("result: completed "))
                    .collect::<Vec<_>>();
                assert_eq!(results.len(), 1, "{rendered}");
                let result: Value = serde_json::from_str(results[0])?;
                assert_eq!(
                    result,
                    json!({"value":machine["result"]["value"]}),
                    "{rendered}"
                );
            }
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
    ] {
        let output = case.run(&["run", "missing.json", name, value, name, value])?;
        assert_eq!(output.0.status.code(), Some(20), "{name}: {output:?}");
        assert!(output.0.stderr.is_empty(), "{name}: {output:?}");
        assert!(
            output.stdout_text()?.contains("usage: fsl run"),
            "{output:?}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2471
#[test]
fn repeated_output_format_refuses_usage_before_request_io_or_execution(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for path in ["missing.json", "run.json"] {
        let output = case.run(&["run", path, "--format", "json", "--format", "json"])?;
        assert_eq!(output.0.status.code(), Some(20), "{output:?}");
        assert!(output.0.stderr.is_empty(), "{output:?}");
        let text = output.stdout_text()?;
        assert!(text.starts_with("run: refusal (exit 20)\n"), "{text}");
        assert!(text.contains("usage: fsl run"), "{text}");
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
            let field = if flag == "--engine" { "engine" } else { flag };
            let value = outcome(
                &run(&case, "missing.json", &[flag, operand])?,
                20,
                "refusal",
            );
            assert!(
                diagnostic(&value, "invalid-request")["message"]
                    .as_str()
                    .expect("message")
                    .contains(field),
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
        let d = diagnostic(&value, "stage_limit_exceeded");
        assert_eq!(
            d["cause"],
            if setting == "s1.tokens" {
                "token-count-exceeded"
            } else {
                "node-count-exceeded"
            },
            "{value}"
        );
        let message = d["message"].as_str().expect("limit message");
        assert!(
            message.contains(setting) && message.contains("limit 0 reached"),
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
        json!({"type":"integer","value":"12"}),
    );
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn empty_provider_configuration_runs_the_function_without_a_process_plugin(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    write_json(
        &case,
        "providers.json",
        &json!({"format":"filament.provider-config/v1","providers":[]}),
    )?;
    completed(
        &run(&case, "run.json", &["--provider-config", "providers.json"])?,
        json!({"type":"integer","value":"12"}),
    );
    Ok(())
}

/// Trace: FR-006-AC-28, FR-006-AC-29
/// Provenance: QSL-2471
#[test]
fn provider_limits_accept_distinct_names_and_refuse_bad_counts_before_request_io(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    completed(
        &run(
            &case,
            "run.json",
            &[
                "--provider-limit",
                "deadline_ms=1000",
                "--provider-limit",
                "grace_ms=100",
                "--provider-limit",
                "max_frame_bytes=1048576",
                "--provider-limit",
                "max_total_bytes=4194304",
                "--provider-limit",
                "max_stderr_bytes=1024",
                "--provider-limit",
                "max_memory_bytes=1048576",
                "--provider-limit",
                "max_cpu_ms=1000",
            ],
        )?,
        json!({"type":"integer","value":"12"}),
    );
    for operand in [
        "deadline_ms",
        "unknown=1",
        "deadline_ms=",
        "deadline_ms=0",
        "deadline_ms=-1",
        "deadline_ms=+1",
        "deadline_ms=01",
        "deadline_ms=2ms",
        "deadline_ms=9007199254740992",
    ] {
        let output = case.run(&[
            "run",
            "missing.json",
            "--provider-limit",
            operand,
            "--format",
            "json",
        ])?;
        assert_eq!(output.0.status.code(), Some(20), "{operand}: {output:?}");
        assert!(output.0.stderr.is_empty(), "{operand}: {output:?}");
        let stderr = output.json()?["diagnostics"][0]["message"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        assert!(
            stderr.contains("invalid-request")
                && stderr.contains("--provider-limit")
                && stderr.contains(operand),
            "{stderr}"
        );
    }
    let output = case.run(&[
        "run",
        "missing.json",
        "--provider-limit",
        "deadline_ms=1",
        "--provider-limit",
        "deadline_ms=2",
        "--format",
        "json",
    ])?;
    assert_eq!(output.0.status.code(), Some(20));
    assert!(output.0.stderr.is_empty(), "{output:?}");
    assert!(output.stdout_text()?.contains("deadline_ms"));
    Ok(())
}

/// Trace: FR-006-AC-24
/// Provenance: QSL-2471
#[test]
fn provider_configuration_is_closed_and_refuses_before_missing_request(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for (input, member) in [
        (
            json!({"format":"filament.provider-config/v1","providers":[],"extra":true}),
            "extra",
        ),
        (
            json!({"format":"filament.provider-config/v1","providers":[{"path":"","arguments":[]}]}),
            "path",
        ),
    ] {
        assert_provider_member_refusal(&case, &input, member)?;
    }
    Ok(())
}

fn assert_provider_member_refusal(
    case: &CliCase,
    input: &Value,
    member: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json(case, "bad-provider.json", input)?;
    let output = case.run(&[
        "run",
        "missing.json",
        "--provider-config",
        "bad-provider.json",
        "--format",
        "json",
    ])?;
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    let stderr = output.json()?["diagnostics"][0]["message"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(stderr.starts_with("invalid-request:"), "{stderr}");
    assert!(stderr.contains(member), "{stderr}");
    assert!(stderr.contains("bad-provider.json"), "{stderr}");
    assert!(!stderr.contains("cannot access missing.json"), "{stderr}");
    Ok(())
}

/// Trace: FR-006-AC-24
/// Provenance: QSL-2471
#[test]
#[ignore = "QSL-2663: nested provider argument type diagnostic omits arguments member"]
fn provider_argument_kind_refusal_names_the_arguments_member(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    assert_provider_member_refusal(
        &case,
        &json!({"format":"filament.provider-config/v1","providers":[{"path":"never-start","arguments":[1]}]}),
        "arguments",
    )
}
