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

/// Trace: FR-002-AC-23
/// FSL FR-624: non-machine unrouted forms print the named diagnostic, not a graph.
#[test]
fn a_bare_inspect_path_reports_unsupported_in_default_text_explicit_text_and_dot(
) -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for args in [
        vec!["inspect", "missing"],
        vec!["inspect", "missing", "--format", "text"],
        vec!["inspect", "missing", "--format", "dot"],
    ] {
        let output = case.run(&args)?;
        assert_eq!(output.0.status.code(), Some(21), "{output:?}");
        assert!(output.0.stderr.is_empty(), "{output:?}");
        let text = output.stdout_text()?;
        assert!(text.starts_with("inspect: unsupported (exit 21)\n"), "{text}");
        assert!(
            text.contains("fsl inspect: this build does not route the inspect operation (unsupported_operation)"),
            "{text}"
        );
        assert!(!text.contains("digraph"), "unrouted DOT must not fabricate a graph");
    }
    Ok(())
}

fn argv_refusal(output: &Captured, named: &str) {
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    let printed = output.stdout_text().unwrap();
    assert!(printed.contains(named), "{printed}");
    assert!(
        !printed.contains("handoff-read"),
        "arguments must be rejected before opening a handoff"
    );
    if output.0.stdout.starts_with(b"{") {
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

/// Trace: FR-002-AC-23
#[test]
fn refused_handoff_formats_name_both_the_format_option_and_its_value() -> Result<(), ContractError>
{
    let case = CliCase::from_environment()?;
    for format in ["dot", "xml"] {
        let output = case.run(&["inspect", "--handoff", "missing", "--format", format])?;
        argv_refusal(&output, format);
        let diagnostic = output.stdout_text()?;
        assert!(diagnostic.contains("--format"), "{diagnostic}");
    }
    Ok(())
}

fn inspect_options_section(help: &str) -> &str {
    let start = help
        .find("inspect options:\n")
        .expect("inspect options section");
    let remaining = &help[start..];
    match remaining.find("\n\n") {
        Some(end) => &remaining[..=end],
        None => remaining,
    }
}

/// Trace: FR-002-AC-25
#[test]
fn inspect_help_is_available_without_opening_the_handoff() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    let root = case.run(&["--help"])?;
    assert_eq!(root.0.status.code(), Some(0));
    assert!(root.0.stderr.is_empty());
    let root_text = root.stdout_text()?;
    let root_options = inspect_options_section(root_text);
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
        assert_eq!(
            inspect_options_section(printed).as_bytes(),
            root_options.as_bytes()
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-25
#[test]
fn invalid_options_are_not_hidden_by_help() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for (args, named) in [
        (vec!["inspect", "--format", "xml", "--help"], "xml"),
        (
            vec!["inspect", "--bogus", "--help", "--format", "json"],
            "--bogus",
        ),
    ] {
        argv_refusal(&case.run(&args)?, named);
    }
    Ok(())
}
