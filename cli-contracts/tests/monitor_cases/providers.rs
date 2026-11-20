// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public provider transport is separate from independently evaluated trace clauses.

use super::*;

const BACKEND: &str = "monitor.transport-contract";

fn configured(result: &str, items: bool) -> Result<CliCase, ContractError> {
    let case = case()?;
    let mut input = request();
    if items {
        input["request"]["items"] =
            json!([{"claim":"Nonnegative","kind":"value-validity","backend":BACKEND}]);
    }
    write_request(&case, &input)?;
    let log = case.directory().join("transport-request.json");
    let shutdown = case.directory().join("transport-shutdown");
    let provider =
        std::path::PathBuf::from(std::env::var_os("FSL_TEST_PROVIDER").expect(
            "set FSL_TEST_PROVIDER to the selected cohort's authoritative transport helper",
        ));
    assert!(
        provider.is_absolute() && provider.is_file(),
        "select an existing absolute provider path"
    );
    case.write("transport.json", &serde_json::to_vec(&json!({"format":"filament.provider-config/v1","providers":[{"path":provider,"arguments":[BACKEND,"unbounded",result,log,shutdown]}]}))?)?;
    Ok(case)
}

fn arguments() -> [&'static str; 8] {
    [
        "monitor",
        "inputs/monitor.json",
        "--provider-config",
        "transport.json",
        "--provider-limit",
        "deadline_ms=4096",
        "--format",
        "json",
    ]
}

/// Trace: FR-025-AC-4
#[test]
fn selected_items_use_provider_transport_while_temporal_clauses_keep_their_real_verdicts(
) -> TestResult {
    let case = configured("proved", true)?;
    let output = case.run_stdin(&arguments(), &records(&[2, 9], true))?;
    let (events, summary) = stream(&output, 10)?;
    assert_eq!(
        events
            .iter()
            .filter(|event| event["settled"] == true)
            .count(),
        2
    );
    assert_eq!(
        summary_clause(&summary, "StaysLow")["category"],
        "violation"
    );
    assert_eq!(
        summary_clause(&summary, "DesiredSeen")["category"],
        "success"
    );
    let item = &summary["items"][0];
    assert_eq!(item["claim_name"], "Nonnegative");
    assert_eq!(item["record"]["result"], "proved");
    assert_eq!(item["record"]["backend"], BACKEND);
    assert_eq!(item["record"]["certification"], "trusted");
    assert!(item.get("certificate").is_none());
    assert!(summary["registry"]
        .as_array()
        .expect("registry")
        .iter()
        .any(|entry| entry["backend"] == BACKEND));
    write_request(&case, &request())?;
    let native = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &records(&[2, 9], true),
    )?;
    let (native_events, _) = stream(&native, 10)?;
    assert_eq!(
        events, native_events,
        "provider items must not alter temporal events"
    );
    let sent: Value =
        serde_json::from_slice(&fs::read(case.directory().join("transport-request.json"))?)?;
    assert_eq!(sent["request"]["deadline_ms"], 4096);
    let routed = sent["request"]["items"].as_array().expect("routed items");
    assert_eq!(routed.len(), 1);
    assert_eq!(routed[0]["backend"], BACKEND);
    assert_eq!(routed[0]["kind"], "value-validity");
    assert_eq!(
        fs::read(case.directory().join("transport-shutdown"))?,
        b"shutdown"
    );
    Ok(())
}

/// Trace: FR-025-AC-4
#[test]
fn configuring_transport_does_not_send_temporal_clauses_as_provider_items() -> TestResult {
    let case = configured("proved", false)?;
    let input = records(&[2, 9], true);
    let configured = case.run_stdin(&arguments(), &input)?;
    stream(&configured, 10)?;
    assert!(!case.directory().join("transport-request.json").exists());
    let native = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &input,
    )?;
    assert_eq!(configured.0.stdout, native.0.stdout);
    Ok(())
}

/// Trace: FR-025-AC-9
#[cfg(unix)]
#[test]
fn interruption_while_stdin_is_open_finishes_with_a_structured_incomplete_outcome() -> TestResult {
    for signal in ["term", "int"] {
        let case = configured(signal, true)?;
        let output = case.run_staged(&arguments(), b"", b"", |bytes| {
            bytes
                .split(|byte| *byte == b'\n')
                .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
                .any(|line| {
                    line["format"] == "filament-outcome/1" && line["category"] == "incomplete"
                })
        })?;
        assert_eq!(
            output.captured.0.status.code(),
            Some(22),
            "{}",
            output.captured.stdout_text()?
        );
        let before_eof = output
            .before_continuation
            .expect("interruption outcome arrives while stdin remains open");
        let lines: Vec<Value> = before_eof
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(serde_json::from_slice)
            .collect::<Result<_, _>>()?;
        let outcome = lines.last().expect("last structured outcome");
        assert_eq!(outcome["format"], "filament-outcome/1");
        assert_eq!(outcome["operation"], "monitor");
        assert_eq!(outcome["category"], "incomplete");
        assert_eq!(
            output.captured.json_lines()?.last(),
            Some(outcome),
            "the incomplete outcome must remain the last output record"
        );
        assert!(!outcome["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .is_empty());
        assert!(case.directory().join("transport-request.json").is_file());
    }
    Ok(())
}
