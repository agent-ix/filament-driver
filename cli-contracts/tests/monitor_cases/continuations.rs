// SPDX-License-Identifier: AGPL-3.0-or-later
//! State resumption, progress and timed completeness contracts.

use super::*;

/// Provenance: QSL-2473; FSL FR-644-AC-3 checkpoint/resume
#[test]
#[ignore = "QSL-2673: resumed events reset observation position instead of preserving the checkpoint prefix"]
fn checkpoint_resumption_preserves_trace_positions_deciding_prefix_and_finality() -> TestResult {
    let case = case()?;
    case.write("inputs/checkpoints/keep", b"owned state directory")?;
    let state = case.directory().join("inputs/checkpoints/load.json");
    let first = encoded(&[
        observation(2),
        observation(4),
        json!({"checkpoint":"checkpoints/load.json"}),
    ]);
    let prefix = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &first,
    )?;
    let (_, prefix_summary) = stream(&prefix, 0)?;
    assert_eq!(prefix_summary["refused"], 0, "{}", prefix.stdout_text()?);
    assert!(
        state.is_file(),
        "checkpoint not written: {}",
        prefix.stdout_text()?
    );
    let state_bytes = fs::read(&state)?;
    let checkpoint: Value = serde_json::from_slice(&state_bytes)?;
    assert_eq!(checkpoint["format"], "filament.monitor-state/1");
    let mut resumed = request();
    resumed["request"]["state"] = json!("checkpoints/load.json");
    case.write("inputs/resumed.json", &serde_json::to_vec(&resumed)?)?;
    let continued = case.run_stdin(
        &["monitor", "inputs/resumed.json", "--format", "json"],
        &records(&[9, 3], true),
    )?;
    let (events, summary) = stream(&continued, 10)?;
    assert_eq!(events.len(), 2);
    assert!(
        events.iter().all(|event| event["record"] == 0
            && event["position"] == 2
            && event["settled"] == true),
        "{}",
        continued.stdout_text()?
    );
    assert!(events
        .iter()
        .all(|event| event["settlement"]["step"] == "3" && event["settlement"]["horizon"] == "2"));
    assert_eq!(summary["records"], 3);
    let whole = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &records(&[2, 4, 9, 3], true),
    )?;
    let (_, uninterrupted) = stream(&whole, 10)?;
    assert_eq!(summary["clauses"], uninterrupted["clauses"]);
    assert_eq!(
        fs::read(&state)?,
        state_bytes,
        "resuming must not rewrite its input state"
    );
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-644 resume refusals occur before records
#[test]
fn unreadable_or_malformed_resume_state_never_emits_observation_events() -> TestResult {
    let case = case()?;
    for (path, bytes) in [
        ("missing-state.json", None),
        ("bad-state.json", Some(b"{".as_slice())),
        ("wrong-state.json", Some(b"{}".as_slice())),
    ] {
        if let Some(bytes) = bytes {
            case.write(path, bytes)?;
        }
        let mut input = request();
        input["request"]["state"] = json!(case.directory().join(path));
        write_request(&case, &input)?;
        let output = case.run_stdin(
            &["monitor", "inputs/monitor.json", "--format", "json"],
            &records(&[9], true),
        )?;
        assert_eq!(output.0.status.code(), Some(20));
        let document = output.json()?;
        assert_eq!(document["format"], "filament-outcome/1");
        assert_eq!(document["operation"], "monitor");
        assert_eq!(document["category"], "refusal");
        assert!(!output.stdout_text()?.contains("filament.monitor-event/1"));
    }
    Ok(())
}

fn clocked(
    source: &str,
    profile: &str,
    kind: &str,
    configuration: Value,
) -> Result<CliCase, ContractError> {
    let case = case()?;
    case.write("inputs/load.fsl", source.as_bytes())?;
    let mut input = request();
    input["request"]["clauses"] = json!([{"name":"Due"}]);
    input["request"]["clocks"] = json!({"ticks":configuration});
    input["request"]["trace"] = if kind == "timed" {
        json!({"kind":"timed","lower_boundary":"authoritative-origin"})
    } else {
        json!({"kind":kind,"lower_boundary":"authoritative-origin","binding":{"profile":profile,"clock":"ticks","parameters":configuration}})
    };
    write_request(&case, &input)?;
    Ok(case)
}

const SAMPLED: &str = r#"language "ix:native" edition "1-draft";
profile p = "filament.temporal.fixed-sample.false-extension/v1";
record Pulse { active: Boolean; }
temporal Due using p over (s: Pulse) clock "ticks" on origin { eventually[0,2] holds(s.active) }
"#;
fn sampled_clock() -> Value {
    json!({"kind":"fixed_sample","epoch":{"kind":"integer","decimal":"0"},"period":{"kind":"integer","decimal":"1"},"unit":"tick"})
}
fn active(value: bool) -> Value {
    json!({"observation":{"sample":{"active":{"boolean":value}}}})
}
fn progress(complete: bool) -> Value {
    json!({"progress":{"watermark":2,"completeness":if complete {json!({"complete":{"through":2}})}else{json!("incomplete")},"scope":"open","execution":"open"}})
}

/// Provenance: QSL-2473; FSL FR-643 settling on authoritative fixed-sample progress
#[test]
fn complete_progress_can_decide_a_window_without_inventing_an_observation() -> TestResult {
    let case = clocked(
        SAMPLED,
        "filament.temporal.fixed-sample.false-extension/v1",
        "progressed",
        sampled_clock(),
    )?;
    let input = encoded(&[active(false), progress(true), json!({"end":true})]);
    let output = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &input,
    )?;
    let (events, summary) = stream(&output, 10)?;
    let decided = events
        .iter()
        .find(|event| event["settled"] == true)
        .expect("progress decides the covered window");
    assert_eq!(decided["record"], 1);
    assert_eq!(decided["category"], "violation");
    assert_eq!(decided["settlement"]["horizon"], "2");
    assert_eq!(decided["settlement"]["step"], "2");
    assert_eq!(
        summary_clause(&summary, "Due")["settlement"],
        decided["settlement"]
    );
    assert_eq!(summary["ledger"]["format"], "filament.progress-ledger/1");
    assert_eq!(summary["records"], 3);
    Ok(())
}

/// Provenance: QSL-2473; incomplete progress cannot prove absent samples
#[test]
fn incomplete_progress_does_not_turn_missing_observations_into_a_decisive_verdict() -> TestResult {
    let case = clocked(
        SAMPLED,
        "filament.temporal.fixed-sample.false-extension/v1",
        "progressed",
        sampled_clock(),
    )?;
    let output = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &encoded(&[active(false), progress(false), json!({"end":true})]),
    )?;
    let (events, summary) = stream(&output, 11)?;
    assert!(events.iter().all(|event| event["settled"] == false));
    let item = summary_clause(&summary, "Due");
    assert_eq!(item["category"], "indeterminate");
    assert_eq!(item["settled"], false);
    assert_eq!(item["settlement"]["trigger"], "finish");
    Ok(())
}

const TIMED: &str = r#"language "ix:native" edition "1-draft";
profile p = "filament.temporal.timed/v1";
dimension Time;
unit tick : Time = rational(1, 1);
record Pulse { active: Boolean; }
temporal Due using p over (s: Pulse) clock "ticks" on origin { eventually[0,4] holds(s.active) }
"#;
fn stamped(value: bool, stamp: Value) -> Value {
    let mut record = active(value);
    record["stamp"] = stamp;
    record
}

/// Provenance: QSL-2473; FSL FR-644-AC-6 timed completeness
#[test]
#[ignore = "QSL-1848: timed trace execution is not built in the public monitor route"]
fn a_through_assertion_settles_the_timed_deadline_on_the_assertion_record() -> TestResult {
    let case = clocked(
        TIMED,
        "filament.temporal.timed/v1",
        "timed",
        json!({"kind":"timed","unit":"tick"}),
    )?;
    let input = encoded(&[
        stamped(false, json!("0")),
        json!({"through":"4"}),
        json!({"end":true}),
    ]);
    let output = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &input,
    )?;
    let (events, summary) = stream(&output, 10)?;
    let decided = events
        .iter()
        .find(|event| event["settled"] == true)
        .expect("through decides deadline");
    assert_eq!(decided["record"], 1);
    assert_eq!(decided["category"], "violation");
    assert_eq!(decided["settlement"]["through"], "4");
    assert_eq!(decided["settlement"]["horizon"], "4");
    assert_eq!(
        summary_clause(&summary, "Due")["settlement"],
        decided["settlement"]
    );
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-642 timed witness is evaluated from real samples
#[test]
#[ignore = "QSL-1848: timed trace execution is not built in the public monitor route"]
fn a_real_timed_witness_settles_before_later_completeness() -> TestResult {
    let case = clocked(
        TIMED,
        "filament.temporal.timed/v1",
        "timed",
        json!({"kind":"timed","unit":"tick"}),
    )?;
    let input = encoded(&[
        stamped(false, json!("0")),
        stamped(true, json!("3")),
        json!({"through":"4"}),
        json!({"end":true}),
    ]);
    let output = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &input,
    )?;
    let (events, summary) = stream(&output, 0)?;
    let decided = events
        .iter()
        .find(|event| event["settled"] == true)
        .expect("real witness");
    assert_eq!(decided["record"], 1);
    assert_eq!(decided["category"], "success");
    assert_eq!(decided["settlement"]["horizon"], "3");
    assert!(decided["settlement"].get("through").is_none());
    assert_eq!(
        summary_clause(&summary, "Due")["settlement"],
        decided["settlement"]
    );
    Ok(())
}

/// Provenance: QSL-2473; FR-644 timestamped progress closes an observed finite window
#[test]
fn timestamped_completeness_settles_the_window_at_the_progress_record() -> TestResult {
    let source = r#"language "ix:native" edition "1-draft";
profile p = "filament.temporal.timestamped-event.finite-window/v1";
record Pulse { active: Boolean; }
temporal Due using p over (s: Pulse) clock "ticks" on origin { eventually[0,4] holds(s.active) }
"#;
    let case = clocked(
        source,
        "filament.temporal.timestamped-event.finite-window/v1",
        "timestamped",
        json!({"kind":"timestamped_event","timestamp_unit":"ms"}),
    )?;
    let input = encoded(&[
        stamped(false, json!(0)),
        json!({"progress":{"watermark":4,"completeness":{"complete":{"through":4}},"scope":"open","execution":"open"}}),
        json!({"end":true}),
    ]);
    let output = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &input,
    )?;
    let (events, summary) = stream(&output, 10)?;
    let decided = events
        .iter()
        .find(|event| event["settled"] == true)
        .expect("covered window decides");
    assert_eq!(decided["record"], 1);
    assert_eq!(decided["category"], "violation");
    assert_eq!(decided["settlement"]["horizon"], "4");
    assert_eq!(
        summary_clause(&summary, "Due")["settlement"],
        decided["settlement"]
    );
    assert_eq!(summary["refused"], 0);
    Ok(())
}
