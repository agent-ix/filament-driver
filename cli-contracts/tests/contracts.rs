// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original programs exercise the external-binary contract helpers themselves.

use fsl_cli_contracts::{CliCase, ContractError};
use serde_json::{json, Value};

const SEATS: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
function Unbooked using v(total: Integer, held: Integer): Integer pure {
    total - held
}
"#;

fn monitor_case() -> Result<CliCase, ContractError> {
    let case = CliCase::from_environment()?;
    case.write(
        "inputs/fuse.fsl",
        br#"language "ix:native" edition "1-draft";
profile t = "filament.temporal.infinite-trace/v1";
record Meter { amperes: Int[0, 9]; }
temporal FuseLimit using t over (r: Meter) clock "steps" on origin {
    always[0,1] holds(r.amperes <= 6)
}
"#,
    )?;
    let request = json!({"format":"native-monitor/1","request":{
        "models":[],"program":{"source":{"file":"fuse.fsl","authority":"contracts",
        "identity":"contracts:fuse","document":"FuseContract","formal_revision":1}},
        "clauses":[{"name":"FuseLimit"}],"trace":{"kind":"finite"}}});
    case.write("inputs/monitor.json", &serde_json::to_vec(&request)?)?;
    Ok(case)
}

fn observations(values: &[u8]) -> Vec<u8> {
    let mut lines = values
        .iter()
        .map(|value| {
            json!({"observation":{"sample":{"amperes":{"integer":value.to_string()}}}}).to_string()
        })
        .collect::<Vec<_>>();
    lines.push(json!({"end":true}).to_string());
    (lines.join("\n") + "\n").into_bytes()
}

mod file_contracts {
    use super::*;

    /// Trace: FR-002-AC-1
    /// Provenance: QSL-2468
    #[test]
    fn source_files_resolve_from_private_cwd_and_keep_json_streams_separate(
    ) -> Result<(), ContractError> {
        let case = CliCase::from_environment()?;
        case.write("source folder/seats.fsl", SEATS.as_bytes())?;
        let valid = case.run(&["check", "source folder/seats.fsl", "--format", "json"])?;
        assert_eq!(valid.0.status.code(), Some(0));
        assert_eq!(valid.stderr_text()?, "");
        let document = valid.json()?;
        assert_eq!(document["format"], "filament-outcome/1");
        assert_eq!(document["operation"], "check");
        assert_eq!(document["category"], "success");
        assert_eq!(document["last_stage"], "S4");
        assert_eq!(valid.json_lines()?, vec![document]);
        assert!(case.directory().join("source folder/seats.fsl").is_file());
        assert!(matches!(
            case.write("../outside.fsl", b"bad"),
            Err(ContractError::FixturePath(_))
        ));
        #[cfg(unix)]
        {
            let outside = tempfile::tempdir().expect("private fixture escape target");
            std::os::unix::fs::symlink(outside.path(), case.directory().join("redirect"))
                .expect("fixture symlink");
            assert!(matches!(
                case.write("redirect/new/input.fsl", b"bad"),
                Err(ContractError::FixturePath(_))
            ));
            assert!(
                !outside.path().join("new").exists(),
                "reject before creating anything outside the case"
            );
        }

        let invalid = SEATS.replace("total - held", "true");
        case.write("source folder/wrong-result.fsl", invalid.as_bytes())?;
        let refused = case.run(&[
            "check",
            "source folder/wrong-result.fsl",
            "--format",
            "json",
        ])?;
        assert_eq!(refused.0.status.code(), Some(20));
        let document = refused.json()?;
        assert_eq!(document["operation"], "check");
        assert_eq!(document["category"], "refusal");
        let diagnostics = document["diagnostics"]
            .as_array()
            .expect("diagnostic array");
        assert!(
            diagnostics.iter().any(|d| d["code"] == "ill_typed"),
            "{document}"
        );
        Ok(())
    }

    /// Trace: FR-002-AC-13
    /// Provenance: QSL-2468
    #[test]
    fn text_output_is_not_accepted_as_json() -> Result<(), ContractError> {
        let case = CliCase::from_environment()?;
        case.write("seats.fsl", SEATS.as_bytes())?;
        let text = case.run(&["check", "seats.fsl"])?;
        assert_eq!(text.0.status.code(), Some(0));
        assert!(text.stdout_text()?.starts_with("check: success (exit 0)"));
        assert_eq!(text.stderr_text()?, "");
        let error = text
            .json()
            .expect_err("text must not be silently admitted as JSON");
        assert_eq!(error.code().as_str(), "invalid-json");
        assert!(matches!(text.json_lines(), Err(ContractError::Json(_))));
        Ok(())
    }

    /// Trace: FR-025-AC-3
    /// Provenance: QSL-2468
    #[test]
    fn argument_refusals_write_their_text_outcome_to_stdout_only() -> Result<(), ContractError> {
        let case = CliCase::from_environment()?;
        let refused = case.run(&["monitor"])?;
        assert_eq!(refused.0.status.code(), Some(20));
        assert!(refused.0.stderr.is_empty(), "{refused:?}");
        assert!(refused.stdout_text()?.contains("usage: fsl monitor"));
        assert!(matches!(refused.json(), Err(ContractError::Json(_))));
        assert!(matches!(refused.json_lines(), Err(ContractError::Json(_))));
        Ok(())
    }
}

mod stdin_contracts {
    use super::*;

    /// Trace: FR-025-AC-1, FR-025-AC-7
    /// Provenance: QSL-2468
    #[test]
    fn finite_records_feed_monitor_and_preserve_verdicts() -> Result<(), ContractError> {
        let case = monitor_case()?;
        for (values, exit, category) in [([4, 6, 9], 0, "success"), ([4, 7, 0], 10, "violation")] {
            let records = observations(&values);
            case.write("inputs/records.jsonl", &records)?;
            let output = case.run_stdin(
                &["monitor", "inputs/monitor.json", "--format", "json"],
                &records,
            )?;
            assert_eq!(output.0.status.code(), Some(exit));
            assert_eq!(output.stderr_text()?, "");
            let file = case.run(&[
                "monitor",
                "inputs/monitor.json",
                "--input",
                "inputs/records.jsonl",
                "--format",
                "json",
            ])?;
            assert_eq!(file.0.status.code(), Some(exit));
            assert_eq!(
                file.0.stdout, output.0.stdout,
                "file and stdin must carry the same observations"
            );
            assert_eq!(file.0.stderr, output.0.stderr);
            let lines = output.json_lines()?;
            let (summary, events) = lines.split_last().expect("events and summary");
            assert_eq!(summary["format"], "filament.monitor-summary/1");
            assert!(!events.is_empty(), "stdin must reach the real monitor");
            assert!(events
                .iter()
                .all(|e| e["format"] == "filament.monitor-event/1"));
            assert!(
                events.iter().any(|e| e["clause"] == "FuseLimit"
                    && e["settled"] == true
                    && e["category"] == category),
                "{lines:?}"
            );
            if exit == 10 {
                assert!(
                    events
                        .iter()
                        .any(|e| e["record"] == 1 && e["category"] == "violation"),
                    "the second actual observation violates the bound: {lines:?}"
                );
            }
            assert!(
                matches!(output.json(), Err(ContractError::Json(_))),
                "a JSON stream is not one outcome value"
            );
        }
        Ok(())
    }

    /// Trace: FR-025-AC-3
    /// Provenance: QSL-2468
    #[test]
    fn early_request_refusal_preserves_output_even_when_stdin_closes() -> Result<(), ContractError>
    {
        let case = monitor_case()?;
        let mut request: Value = serde_json::from_slice(
            &std::fs::read(case.directory().join("inputs/monitor.json")).expect("fixture"),
        )?;
        request["request"]["clauses"][0]["name"] = json!("AbsentFuseLimit");
        case.write("inputs/unknown.json", &serde_json::to_vec(&request)?)?;
        // Larger than a pipe buffer: an early refusal may close stdin before
        // this finishes. The helper must retain FSL's refusal, not lose it to EPIPE.
        let input = observations(&[4, 5]).repeat(8_192);
        let output = case.run_stdin(
            &["monitor", "inputs/unknown.json", "--format", "json"],
            &input,
        )?;
        assert_eq!(output.0.status.code(), Some(20));
        let lines = output.json_lines()?;
        assert_eq!(lines.len(), 1, "request refused before observations");
        assert_eq!(lines[0]["operation"], "monitor");
        assert_eq!(lines[0]["category"], "refusal");
        assert!(!lines[0]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .is_empty());
        Ok(())
    }
}
