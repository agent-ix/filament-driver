// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-002-AC-23
#[test]
fn default_and_explicit_text_render_the_inspect_success_document() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    for args in [
        vec!["inspect", "--handoff", "handoff"],
        vec!["inspect", "--format", "text", "--handoff", "handoff"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(0), "{output:?}");
        assert!(output.0.stderr.is_empty());
        assert_eq!(output.stdout_text()?, "inspect: success (exit 0)\n");
    }
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn text_and_machine_refusals_keep_the_same_reader_code_and_exit() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    let machine = refusal(&inspect(&case, "missing")?, "handoff-read");
    let text = case.run(&["inspect", "--handoff", "missing", "--format", "text"])?;
    assert_eq!(text.0.status.code(), Some(20));
    assert!(text.0.stderr.is_empty());
    let printed = text.stdout_text()?;
    assert!(
        printed.starts_with("inspect: refusal (exit 20)\n"),
        "{printed}"
    );
    assert!(printed.contains(machine["diagnostics"][0]["code"].as_str().unwrap()));
    assert!(printed.contains("missing"));
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn a_bare_inspect_path_is_the_documented_unrouted_form() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    let value = document(
        &case.run(&["inspect", "missing", "--format", "json"])?,
        21,
        "unsupported",
    );
    assert_eq!(value["last_stage"], Value::Null);
    assert_eq!(value["diagnostics"][0]["code"], "unsupported_operation");
    assert_eq!(value["diagnostics"][0]["cause"], "operation-not-routed");
    assert!(value["diagnostics"][0].to_string().contains("inspect"));
    Ok(())
}

fn argv_refusal(output: &Captured, named: &str) {
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    let printed = format!(
        "{}{}",
        output.stdout_text().unwrap(),
        output.stderr_text().unwrap()
    );
    assert!(printed.contains(named), "{printed}");
    assert!(
        !printed.contains("handoff-read"),
        "arguments must be rejected before opening a handoff"
    );
    if !output.0.stdout.is_empty() {
        let value = output.json().expect("machine-mode argument document");
        assert_eq!(value["operation"], "inspect");
        assert_eq!(value["category"], "refusal");
        assert_eq!(value["diagnostics"][0]["code"], "invalid-request");
    }
}

/// Trace: FR-002-AC-23
#[test]
fn missing_repeated_handoff_and_extra_operands_refuse_before_artifact_io(
) -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for (args, named) in [
        (vec!["inspect", "--handoff"], "--handoff"),
        (
            vec!["inspect", "--handoff", "--format", "json"],
            "--handoff",
        ),
        (
            vec![
                "inspect",
                "--handoff",
                "missing",
                "extra",
                "--format",
                "json",
            ],
            "extra",
        ),
        (
            vec![
                "inspect",
                "--handoff",
                "missing",
                "--handoff",
                "missing",
                "--format",
                "json",
            ],
            "--handoff",
        ),
    ] {
        argv_refusal(&case.run(&args)?, named);
    }
    Ok(())
}

/// Trace: FR-002-AC-23; FR-002-AC-24
#[test]
fn wrong_format_dot_handoff_and_foreign_options_are_refused() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for (args, named) in [
        (
            vec!["inspect", "--handoff", "missing", "--format", "dot"],
            "dot",
        ),
        (
            vec!["inspect", "--handoff", "missing", "--format", "xml"],
            "xml",
        ),
        (
            vec![
                "inspect",
                "--handoff",
                "missing",
                "--bogus",
                "--format",
                "json",
            ],
            "--bogus",
        ),
        (
            vec!["inspect", "missing", "--bogus", "--format", "json"],
            "--bogus",
        ),
        (
            vec![
                "inspect",
                "--handoff",
                "missing",
                "--provider-config",
                "absent.json",
                "--format",
                "json",
            ],
            "--provider-config",
        ),
        (
            vec![
                "inspect",
                "--handoff",
                "missing",
                "--version",
                "--format",
                "json",
            ],
            "--version",
        ),
    ] {
        argv_refusal(&case.run(&args)?, named);
    }
    Ok(())
}

/// Trace: FR-002-AC-25
#[test]
fn inspect_help_is_available_without_opening_the_handoff() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for args in [
        vec!["inspect", "--help"],
        vec!["inspect", "-h"],
        vec!["inspect", "--handoff", "missing", "--help"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(0), "{output:?}");
        assert!(output.0.stderr.is_empty());
        let printed = output.stdout_text()?;
        assert!(printed.starts_with("Usage: fsl inspect"));
        assert!(printed.contains("inspect options:"));
        assert!(printed.contains("--handoff <directory>"));
        assert!(printed.contains("--format"));
    }
    Ok(())
}
