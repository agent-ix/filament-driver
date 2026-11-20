// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-022-AC-7
#[test]
fn text_and_json_keep_order_verdicts_and_canonical_json_framing() -> Result<(), ContractError> {
    let (case, _) = setup()?;
    for options in [
        vec!["analyze", "data/request.json"],
        vec!["analyze", "--format", "text", "data/request.json"],
    ] {
        let output = case.run(&options)?;
        assert_eq!(output.0.status.code(), Some(10));
        assert!(output.0.stderr.is_empty());
        let text = output.stdout_text()?;
        let lines: Vec<_> = text.lines().collect();
        assert!(lines[0].starts_with("0 WithinCapacity proved"), "{text}");
        assert!(lines[1].starts_with("1 NeverFull refuted"), "{text}");
        assert_eq!(lines.last(), Some(&"violation"));
    }
    let output = case.run(&["analyze", "--format", "json", "data/request.json"])?;
    let value = document(&output, 10, "violation");
    let mut encoded = serde_json::to_vec(&value)?;
    encoded.push(b'\n');
    assert_eq!(
        output.0.stdout, encoded,
        "canonical sorted-key JSON with one newline"
    );
    Ok(())
}

/// Trace: FR-022-AC-5
#[test]
fn missing_extra_and_unknown_options_refuse_before_reading_a_request() -> Result<(), ContractError>
{
    let case = CliCase::from_environment()?;
    for args in [
        vec!["analyze"],
        vec!["analyze", "absent.json", "extra.json"],
        vec!["analyze", "absent.json", "--unknown"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(20), "{args:?}: {output:?}");
        let all = format!("{}{}", output.stdout_text()?, output.stderr_text()?);
        let expected = if args.contains(&"--unknown") {
            "--unknown"
        } else {
            "usage: fsl analyze"
        };
        assert!(all.contains(expected), "{args:?}: {all}");
        assert!(
            !all.contains("could not read"),
            "argv is rejected before file access"
        );
    }
    Ok(())
}

/// Trace: FR-022-AC-5
#[test]
fn missing_repeated_provider_config_and_wrong_format_name_the_option() -> Result<(), ContractError>
{
    let case = CliCase::from_environment()?;
    for args in [
        vec!["analyze", "absent.json", "--provider-config"],
        vec![
            "analyze",
            "absent.json",
            "--provider-config",
            "a",
            "--provider-config",
            "b",
        ],
        vec!["analyze", "absent.json", "--format", "xml"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(20), "{args:?}: {output:?}");
        let all = format!("{}{}", output.stdout_text()?, output.stderr_text()?);
        let expected = if args.contains(&"xml") {
            "format"
        } else {
            "--provider-config"
        };
        assert!(all.contains(expected), "{args:?}: {all}");
        assert!(
            !all.contains("could not read"),
            "argv is rejected before file access"
        );
    }
    Ok(())
}

/// Trace: FR-022-AC-5
#[test]
fn analyze_help_does_not_read_the_operand() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for args in [
        vec!["analyze", "--help"],
        vec!["analyze", "-h"],
        vec!["analyze", "absent.json", "--help"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(0), "{output:?}");
        assert!(output.0.stderr.is_empty());
        let text = output.stdout_text()?;
        assert!(text.starts_with("Usage: fsl analyze"));
        for flag in ["--format", "--provider-config", "--provider-limit"] {
            assert!(text.contains(flag), "{text}");
        }
    }
    Ok(())
}

/// Trace: FR-022-AC-5; FR-006-AC-28
#[test]
fn empty_provider_configuration_and_limits_combine_with_json_analysis() -> Result<(), ContractError>
{
    let (case, mut request) = setup()?;
    request["request"]["items"] = json!([{"claim":"WithinCapacity"}]);
    write_request(&case, &request)?;
    case.write(
        "providers.json",
        br#"{"format":"filament.provider-config/v1","providers":[]}"#,
    )?;
    for config in [false, true] {
        let mut args = vec![
            "analyze",
            "data/request.json",
            "--format",
            "json",
            "--provider-limit",
            "deadline_ms=30000",
            "--provider-limit",
            "max_frame_bytes=1048576",
        ];
        if config {
            args.extend(["--provider-config", "providers.json"]);
        }
        let value = document(&case.run(&args)?, 0, "success");
        assert_eq!(value["items"][0]["record"]["result"], "proved");
        assert_eq!(value["items"][0]["record"]["certification"], "certified");
    }
    Ok(())
}

/// Trace: FR-022-AC-5; FR-006-AC-29
#[test]
fn invalid_provider_limit_values_refuse_without_opening_a_request() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for value in [
        "max_frames=4",
        "deadline_ms",
        "deadline_ms=0",
        "deadline_ms=10s",
        "deadline_ms=010",
        "max_total_bytes=9007199254740992",
    ] {
        for format in ["text", "json"] {
            let output = case.run(&[
                "analyze",
                "absent.json",
                "--provider-limit",
                value,
                "--format",
                format,
            ])?;
            assert_eq!(output.0.status.code(), Some(20));
            assert!(output.0.stdout.is_empty());
            let text = output.stderr_text()?;
            assert!(
                text.contains("--provider-limit") && text.contains(value),
                "{text}"
            );
        }
    }
    let repeated = case.run(&[
        "analyze",
        "absent.json",
        "--provider-limit",
        "deadline_ms=5",
        "--provider-limit",
        "deadline_ms=5",
    ])?;
    assert_eq!(repeated.0.status.code(), Some(20));
    assert!(repeated.stderr_text()?.contains("already given"));
    Ok(())
}

/// Trace: FR-022-AC-9
#[test]
fn provider_configuration_refusals_precede_request_admission() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for (path,bytes) in [("malformed.json",b"{".as_slice()),("wrong.json",br#"{"format":"another/v1","providers":[]}"#),("unknown.json",br#"{"format":"filament.provider-config/v1","providers":[],"extra":0}"#),("missing-executable.json",br#"{"format":"filament.provider-config/v1","providers":[{"path":"not-an-executable"}]}"#)] {
        case.write(path,bytes)?;
    }
    for path in [
        "absent.json",
        "malformed.json",
        "wrong.json",
        "unknown.json",
        "missing-executable.json",
    ] {
        let output = case.run(&[
            "analyze",
            "missing-request.json",
            "--provider-config",
            path,
            "--format",
            "json",
        ])?;
        assert_eq!(output.0.status.code(), Some(20), "{output:?}");
        assert!(output.0.stdout.is_empty());
        assert!(output.stderr_text()?.contains(path), "{output:?}");
    }
    Ok(())
}
