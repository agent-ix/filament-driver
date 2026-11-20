// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original load-monitor streams and independent temporal verdict oracles.

use std::error::Error;
use std::fs;

use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::{json, Value};

type TestResult = Result<(), Box<dyn Error>>;
const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile t = "filament.temporal.infinite-trace/v1";
profile v = "filament.value.complete/v1";
record Load { bins: Int[0, 15]; }
temporal DesiredSeen using t over (s: Load) clock "steps" on origin {
    eventually holds(s.bins = 9)
}
temporal StaysLow using t over (s: Load) clock "steps" on origin {
    always holds(s.bins <= 6)
}
function Nonnegative using v(n: Integer): Boolean pure { n * n >= 0 }
"#;

fn request() -> Value {
    json!({"format":"native-monitor/1","request":{
        "models":[],"program":{"source":{"file":"load.fsl","authority":"monitor-contracts",
        "identity":"load","document":"LoadMonitor","formal_revision":1}},
        "clauses":[{"name":"StaysLow"},{"name":"DesiredSeen"}],"trace":{"kind":"finite"}
    }})
}

fn write_request(case: &CliCase, value: &Value) -> Result<(), ContractError> {
    case.write("inputs/monitor.json", &serde_json::to_vec(value)?)
}

fn case() -> Result<CliCase, ContractError> {
    let case = CliCase::from_environment()?;
    case.write("inputs/load.fsl", SOURCE.as_bytes())?;
    write_request(&case, &request())?;
    Ok(case)
}

fn observation(value: u8) -> Value {
    json!({"observation":{"sample":{"bins":{"integer":value.to_string()}}}})
}

fn records(values: &[u8], end: bool) -> Vec<u8> {
    let mut lines: Vec<_> = values.iter().copied().map(observation).collect();
    if end {
        lines.push(json!({"end":true}));
    }
    encoded(&lines)
}

fn encoded(lines: &[Value]) -> Vec<u8> {
    if lines.is_empty() {
        return Vec::new();
    }
    (lines
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n")
        .into_bytes()
}

fn stream(output: &Captured, exit: i32) -> Result<(Vec<Value>, Value), ContractError> {
    assert_eq!(
        output.0.status.code(),
        Some(exit),
        "stdout: {}\nstderr: {}",
        output.stdout_text()?,
        output.stderr_text()?
    );
    assert_eq!(output.stderr_text()?, "");
    let mut lines = output.json_lines()?;
    let summary = lines.pop().expect("one final summary");
    assert_eq!(summary["format"], "filament.monitor-summary/1", "{summary}");
    for event in &lines {
        assert_eq!(event["format"], "filament.monitor-event/1", "{event}");
        assert_eq!(
            event.get("settlement").is_some(),
            event["settled"] == true,
            "{event}"
        );
    }
    let canonical = encoded(
        &lines
            .iter()
            .cloned()
            .chain([summary.clone()])
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        output.0.stdout, canonical,
        "one canonical JSON value per complete line"
    );
    Ok((lines, summary))
}

fn summary_clause<'a>(summary: &'a Value, name: &str) -> &'a Value {
    summary["clauses"]
        .as_array()
        .expect("summary clauses")
        .iter()
        .find(|item| item["clause"] == name)
        .expect("requested clause")
}

fn decided_stream(events: &[Value], summary: &Value) {
    let expected = [
        (0, "StaysLow", "pending", false),
        (0, "DesiredSeen", "pending", false),
        (2, "StaysLow", "violation", true),
        (2, "DesiredSeen", "success", true),
    ];
    let actual: Vec<_> = events
        .iter()
        .map(|event| {
            (
                event["record"].as_u64().expect("record index"),
                event["clause"].as_str().expect("clause"),
                event["category"].as_str().expect("category"),
                event["settled"].as_bool().expect("settled"),
            )
        })
        .collect();
    assert_eq!(actual, expected);
    for event in events {
        assert_eq!(
            event["position"], event["record"],
            "event position records the latest observed sample: {event}"
        );
        if event["category"] == "violation" {
            assert_eq!(
                event["disposition"]["temporal"]["position"], 0,
                "failing activation is at origin: {event}"
            );
        }
        assert!(!event["disposition"].to_string().contains("\"proved\""));
    }
    for (name, category) in [("StaysLow", "violation"), ("DesiredSeen", "success")] {
        let item = summary_clause(summary, name);
        assert_eq!(item["category"], category);
        assert_eq!(item["settled"], true);
        assert_eq!(item["settlement"]["step"], "3");
        let expected_position = if name == "StaysLow" {
            json!("0")
        } else {
            Value::Null
        };
        assert_eq!(item["settlement"]["position"], expected_position);
        assert_eq!(item["settlement"]["horizon"], "2");
        let decided = events
            .iter()
            .find(|event| event["clause"] == name && event["settled"] == true)
            .expect("deciding event");
        assert_eq!(item["disposition"], decided["disposition"]);
        assert_eq!(item["settlement"], decided["settlement"]);
        assert_eq!(item["settlement"]["trigger"], "decided");
    }
    assert_eq!(summary["records"], 5);
    assert_eq!(summary["refused"], 0);
}

/// Trace: FR-025-AC-1
/// Provenance: QSL-2473; FSL FR-644 status changes and origin readings
#[test]
fn ordered_status_changes_bind_each_new_observation_and_keep_decided_reports_final() -> TestResult {
    let case = case()?;
    let captured = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &records(&[2, 4, 9, 3], true),
    )?;
    let (events, summary) = stream(&captured, 10)?;
    decided_stream(&events, &summary);
    Ok(())
}

/// Trace: FR-025-AC-7
/// Provenance: QSL-2473
#[test]
fn an_input_file_replaces_stdin_and_preserves_the_complete_ordered_stream() -> TestResult {
    let case = case()?;
    let input = records(&[2, 4, 9, 3], true);
    case.write("records.jsonl", &input)?;
    let stdin = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &input,
    )?;
    let file = case.run_stdin(
        &[
            "monitor",
            "--format",
            "json",
            "--input",
            "records.jsonl",
            "inputs/monitor.json",
        ],
        &records(&[0], true),
    )?;
    let (events, summary) = stream(&file, 10)?;
    decided_stream(&events, &summary);
    assert_eq!(file.0.stdout, stdin.0.stdout);
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-643 Trace end, infinite-trace prefixes
#[test]
fn a_good_finite_prefix_does_not_become_an_infinite_trace_proof_at_eof() -> TestResult {
    let case = case()?;
    for end in [false, true] {
        let output = case.run_stdin(
            &["monitor", "inputs/monitor.json", "--format", "json"],
            &records(&[2, 4, 3], end),
        )?;
        let (_, summary) = stream(&output, 0)?;
        for name in ["StaysLow", "DesiredSeen"] {
            let item = summary_clause(&summary, name);
            assert_eq!(item["category"], "pending", "{item}");
            assert_eq!(item["settled"], false);
            assert_eq!(item["settlement"]["trigger"], "finish");
            assert!(!item.to_string().contains("\"proved\""));
        }
        assert_eq!(summary["records"], if end { 4 } else { 3 });
    }
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-643 Trace end, legitimate empty traces
#[test]
fn empty_trace_is_a_pending_summary_not_an_empty_request_refusal() -> TestResult {
    let case = case()?;
    for end in [false, true] {
        let output = case.run_stdin(
            &["monitor", "inputs/monitor.json", "--format", "json"],
            &records(&[], end),
        )?;
        let (events, summary) = stream(&output, 0)?;
        assert!(events.is_empty());
        assert_eq!(summary["records"], if end { 1 } else { 0 });
        assert_eq!(summary["refused"], 0);
        for name in ["StaysLow", "DesiredSeen"] {
            let item = summary_clause(&summary, name);
            assert_eq!(item["category"], "pending", "{item}");
            assert_eq!(item["settled"], false);
            assert_eq!(item["settlement"], json!({"step":"0","trigger":"finish"}));
        }
    }
    Ok(())
}

/// Trace: FR-025-AC-8
/// Provenance: QSL-2473
#[test]
fn text_mode_renders_every_event_then_every_clause_and_final_record_counts() -> TestResult {
    let case = case()?;
    let input = records(&[2, 4, 9, 3], true);
    let machine = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &input,
    )?;
    let (events, summary) = stream(&machine, 10)?;
    let mut expected = events
        .iter()
        .map(|event| {
            format!(
                "{} {} {} {}",
                event["record"],
                event["clause"].as_str().expect("clause"),
                event["category"].as_str().expect("category"),
                if event["settled"] == true {
                    "settled"
                } else {
                    "pending"
                }
            )
        })
        .collect::<Vec<_>>();
    expected.extend(
        summary["clauses"]
            .as_array()
            .expect("clauses")
            .iter()
            .map(|item| {
                format!(
                    "{} {} {}",
                    item["clause"].as_str().expect("clause"),
                    item["category"].as_str().expect("category"),
                    if item["settled"] == true {
                        "settled"
                    } else {
                        "pending"
                    }
                )
            }),
    );
    expected.push("records 5 refused 0".to_owned());
    let expected = (expected.join("\n") + "\n").into_bytes();
    for args in [
        vec!["monitor", "inputs/monitor.json"],
        vec!["monitor", "inputs/monitor.json", "--format", "text"],
    ] {
        let output = case.run_stdin(&args, &input)?;
        assert_eq!(output.0.status.code(), Some(10));
        assert_eq!(output.0.stdout, expected);
        assert_eq!(output.stderr_text()?, "");
    }
    Ok(())
}

/// Trace: FR-025-AC-6
/// Provenance: QSL-2473
#[test]
fn a_decisive_event_is_flushed_before_later_records_or_stdin_eof() -> TestResult {
    let case = case()?;
    let prefix = records(&[2, 4, 9], false);
    let remainder = records(&[3], true);
    let staged = case.run_staged(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &prefix,
        &remainder,
        |bytes| {
            bytes
                .split(|byte| *byte == b'\n')
                .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
                .any(|event| {
                    event["format"] == "filament.monitor-event/1"
                        && event["clause"] == "StaysLow"
                        && event["record"] == 2
                        && event["settled"] == true
                        && event["category"] == "violation"
                })
        },
    )?;
    let observed = staged
        .before_continuation
        .as_ref()
        .expect("deciding event observed while stdin stayed open");
    let lines = observed
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice::<Value>)
        .collect::<Result<Vec<_>, _>>()?;
    assert!(lines.iter().any(|event| event["record"] == 2
        && event["clause"] == "StaysLow"
        && event["settled"] == true));
    assert!(lines
        .iter()
        .all(|event| event["format"] == "filament.monitor-event/1"));
    assert!(lines
        .iter()
        .all(|event| event["record"].as_u64().expect("record") <= 2));
    let (events, summary) = stream(&staged.captured, 10)?;
    decided_stream(&events, &summary);
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-644 Outputs/Published schemas (QSL-2477)
#[test]
#[ignore = "QSL-2477: finite summary must omit ledger and unrequested item/registry members"]
fn finite_summary_omits_unapplicable_optional_members() -> TestResult {
    let case = case()?;
    let output = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &records(&[2, 4, 9, 3], true),
    )?;
    let (_, summary) = stream(&output, 10)?;
    assert!(summary.get("ledger").is_none());
    assert!(summary.get("items").is_none());
    assert!(summary.get("registry").is_none());
    Ok(())
}

mod continuations;
mod inputs;
mod options;
mod providers;
