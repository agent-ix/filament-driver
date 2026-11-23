// SPDX-License-Identifier: AGPL-3.0-or-later
//! Argument and presentation failures are distinct from stream records.

use super::*;

/// Provenance: QSL-2473; FR-025 argument arity and monitor option contract
#[test]
fn invalid_arguments_refuse_before_consuming_a_valid_observation_stream() -> TestResult {
    let case = case()?;
    for args in [
        vec!["monitor"],
        vec!["monitor", "inputs/monitor.json", "extra.json"],
        vec!["monitor", "inputs/monitor.json", "--unknown-monitor-option"],
        vec!["monitor", "inputs/monitor.json", "--input"],
        vec![
            "monitor",
            "inputs/monitor.json",
            "--input",
            "a",
            "--input",
            "b",
        ],
        vec!["monitor", "inputs/monitor.json", "--format"],
        vec!["monitor", "inputs/monitor.json", "--format", "yaml"],
        vec!["monitor", "inputs/monitor.json", "--provider-config"],
        vec!["monitor", "inputs/monitor.json", "--provider-limit"],
        vec!["monitor", "inputs/monitor.json", "--limit", "fuel=1"],
    ] {
        let output = case.run_stdin(&args, &records(&[9], true))?;
        assert_eq!(output.0.status.code(), Some(20), "{args:?}: {output:?}");
        assert!(output.0.stderr.is_empty(), "{args:?}: {output:?}");
        assert!(
            !output.0.stdout.is_empty(),
            "{args:?}: missing usage diagnostic"
        );
    }
    Ok(())
}

/// Trace: FR-025-AC-7
#[test]
fn unopenable_record_input_in_text_mode_has_no_primary_output() -> TestResult {
    let case = case()?;
    for args in [
        vec![
            "monitor",
            "inputs/monitor.json",
            "--input",
            "absent-records",
        ],
        vec![
            "monitor",
            "inputs/monitor.json",
            "--input",
            "inputs",
            "--format",
            "text",
        ],
    ] {
        let output = case.run_stdin(&args, &records(&[9], true))?;
        assert_eq!(output.0.status.code(), Some(20), "{output:?}");
        assert!(output.0.stderr.is_empty(), "{output:?}");
        assert!(
            output.stdout_text()?.starts_with("monitor: refusal (exit 20)"),
            "{output:?}"
        );
    }
    Ok(())
}

/// Provenance: QSL-2473; FR-025 accepts provider configuration but rejects malformed transport settings
#[test]
fn invalid_provider_configuration_and_limits_never_become_temporal_verdicts() -> TestResult {
    let case = case()?;
    case.write("broken-provider.json", b"{")?;
    case.write("wrong-provider.json", b"{}")?;
    for args in [
        vec![
            "monitor",
            "inputs/monitor.json",
            "--provider-config",
            "missing-provider.json",
            "--format",
            "json",
        ],
        vec![
            "monitor",
            "inputs/monitor.json",
            "--provider-config",
            "broken-provider.json",
            "--format",
            "json",
        ],
        vec![
            "monitor",
            "inputs/monitor.json",
            "--provider-config",
            "wrong-provider.json",
            "--format",
            "json",
        ],
        vec![
            "monitor",
            "inputs/monitor.json",
            "--provider-limit",
            "deadline_ms=0",
            "--format",
            "json",
        ],
        vec![
            "monitor",
            "inputs/monitor.json",
            "--provider-limit",
            "deadline_ms=wrong",
            "--format",
            "json",
        ],
        vec![
            "monitor",
            "inputs/monitor.json",
            "--provider-limit",
            "unrecognized=5",
            "--format",
            "json",
        ],
    ] {
        let output = case.run_stdin(&args, &records(&[9], true))?;
        assert_eq!(output.0.status.code(), Some(20), "{args:?}: {output:?}");
        assert!(!output.stdout_text()?.contains("filament.monitor-event/1"));
        assert!(!output.stdout_text()?.contains("filament.monitor-summary/1"));
        assert!(!output.0.stdout.is_empty() || !output.0.stderr.is_empty());
    }
    Ok(())
}
