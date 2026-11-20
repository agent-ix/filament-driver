// SPDX-License-Identifier: AGPL-3.0-or-later
//! Request, filesystem and observation-record contracts.

use super::*;

fn request_refusal(output: &Captured, exit: i32) -> Result<Value, ContractError> {
    assert_eq!(
        output.0.status.code(),
        Some(exit),
        "{} {}",
        output.stdout_text()?,
        output.stderr_text()?
    );
    let document = output.json()?;
    assert_eq!(document["format"], "filament-outcome/1", "{document}");
    assert_eq!(document["operation"], "monitor", "{document}");
    assert_eq!(
        document["category"],
        if exit == 22 { "incomplete" } else { "refusal" },
        "{document}"
    );
    assert!(!document["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .is_empty());
    Ok(document)
}

/// Trace: FR-025-AC-3
/// Provenance: QSL-2473
#[test]
fn empty_malformed_non_utf8_and_directory_requests_do_not_read_observations() -> TestResult {
    for bytes in [b"".as_slice(), b"{", &[0xff], b"[]", b"{}", b"{} {}"] {
        let case = case()?;
        case.write("inputs/bad.json", bytes)?;
        let output = case.run_stdin(
            &["monitor", "inputs/bad.json", "--format", "json"],
            &records(&[9], true),
        )?;
        let document = request_refusal(&output, 20)?;
        assert!(document["last_stage"].is_null());
        assert!(document["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["code"] == "invalid-request"));
    }
    let case = case()?;
    for path in ["missing-request.json", "inputs"] {
        let output =
            case.run_stdin(&["monitor", path, "--format", "json"], &records(&[9], true))?;
        let document = request_refusal(&output, 20)?;
        assert!(document["last_stage"].is_null());
        assert!(document["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["code"] == "io-error"));
    }
    Ok(())
}

/// Trace: FR-025-AC-3
/// Provenance: QSL-2473
#[test]
fn unknown_or_empty_clause_selection_refuses_before_emitting_any_event() -> TestResult {
    let case = case()?;
    for clauses in [json!([]), json!([{"name":"NotDeclared"}])] {
        let mut input = request();
        input["request"]["clauses"] = clauses;
        write_request(&case, &input)?;
        let output = case.run_stdin(
            &["monitor", "inputs/monitor.json", "--format", "json"],
            &records(&[9], true),
        )?;
        request_refusal(&output, 20)?;
        assert!(!output.stdout_text()?.contains("filament.monitor-event/1"));
    }
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-644 closed monitor request and stream-only CLI form
#[test]
fn request_records_and_unknown_members_are_not_silently_used_as_a_batch() -> TestResult {
    let case = case()?;
    case.write("inputs/batch.jsonl", &records(&[9], true))?;
    for (field, value) in [
        ("records", json!("batch.jsonl")),
        ("unexpected", json!(true)),
    ] {
        let mut input = request();
        input["request"][field] = value;
        write_request(&case, &input)?;
        let document = request_refusal(
            &case.run_stdin(
                &["monitor", "inputs/monitor.json", "--format", "json"],
                &records(&[2], true),
            )?,
            20,
        )?;
        assert!(document.to_string().contains(field));
    }
    Ok(())
}

/// Trace: FR-025-AC-3
/// Provenance: QSL-2473; FSL FR-644 compile admission before record processing
#[test]
fn source_io_encoding_and_type_refusals_keep_their_original_classes() -> TestResult {
    let case = case()?;
    for (bytes, code) in [
        (&[0xff][..], "invalid_utf8"),
        (b"not a native program".as_slice(), "invalid_syntax"),
    ] {
        case.write("inputs/load.fsl", bytes)?;
        let document = request_refusal(
            &case.run_stdin(
                &["monitor", "inputs/monitor.json", "--format", "json"],
                &records(&[9], true),
            )?,
            20,
        )?;
        assert!(
            document["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .iter()
                .any(|d| d["code"] == code),
            "{document}"
        );
    }
    case.write(
        "inputs/load.fsl",
        SOURCE.replace("n * n >= 0", "n + 1").as_bytes(),
    )?;
    let document = request_refusal(
        &case.run_stdin(
            &["monitor", "inputs/monitor.json", "--format", "json"],
            &records(&[9], true),
        )?,
        20,
    )?;
    assert_eq!(document["last_stage"], "S3");
    assert!(document["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .any(|d| d["code"] == "ill_typed"));
    Ok(())
}

/// Trace: FR-025-AC-7
/// Provenance: QSL-2473
#[test]
fn missing_or_directory_record_files_refuse_instead_of_falling_back_to_stdin() -> TestResult {
    let case = case()?;
    case.write("record-directory/keep", b"directory sentinel")?;
    for path in ["missing-records.jsonl", "record-directory"] {
        let output = case.run_stdin(
            &[
                "monitor",
                "inputs/monitor.json",
                "--input",
                path,
                "--format",
                "json",
            ],
            &records(&[9], true),
        )?;
        request_refusal(&output, 20)?;
        assert!(!output.stdout_text()?.contains("filament.monitor-event/1"));
    }
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-644 request admission under RunLimits
#[test]
#[ignore = "QSL-2674: request-byte admission must refuse with exit 20 rather than incomplete/22"]
fn oversized_request_reports_its_intake_limit_without_consuming_the_stream() -> TestResult {
    let case = case()?;
    let mut input = serde_json::to_vec(&request())?;
    input.resize(1_048_577, b' ');
    case.write("inputs/huge.json", &input)?;
    let output = case.run_stdin(
        &["monitor", "inputs/huge.json", "--format", "json"],
        &records(&[9], true),
    )?;
    let document = request_refusal(&output, 20)?;
    assert!(document["last_stage"].is_null());
    assert!(
        document.to_string().contains("request_bytes") && document.to_string().contains("1048576")
    );
    Ok(())
}

/// Trace: FR-025-AC-7
/// Provenance: QSL-2473; native filesystem record operand
#[cfg(unix)]
#[test]
fn non_utf8_record_filename_preserves_file_input_binding() -> TestResult {
    use std::ffi::{OsStr, OsString};
    use std::os::unix::ffi::OsStringExt;
    let case = case()?;
    let name = OsString::from_vec(b"records-\xff.jsonl".to_vec());
    case.write(std::path::Path::new(&name), &records(&[2, 4, 9, 3], true))?;
    let output = case.run(&[
        OsStr::new("monitor"),
        OsStr::new("inputs/monitor.json"),
        OsStr::new("--input"),
        name.as_os_str(),
        OsStr::new("--format"),
        OsStr::new("json"),
    ])?;
    let (events, summary) = stream(&output, 10)?;
    decided_stream(&events, &summary);
    Ok(())
}

/// Trace: FR-025-AC-5
/// Provenance: QSL-2473; FSL FR-644 malformed records leave the session unchanged
#[test]
fn a_refused_observation_does_not_advance_the_trace_or_corrupt_later_decisions() -> TestResult {
    let case = case()?;
    let bad = json!({"observation":{"sample":{"bins":{"integer":"15"}}},"extra":true});
    let with_bad = encoded(&[
        observation(2),
        bad,
        observation(3),
        observation(9),
        json!({"end":true}),
    ]);
    let output = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &with_bad,
    )?;
    let (events, summary) = stream(&output, 10)?;
    let refused = events
        .iter()
        .find(|event| event["category"] == "refusal")
        .expect("one refused input record");
    assert_eq!(refused["record"], 1);
    assert!(refused["clause"].is_null());
    assert_eq!(refused["settled"], false);
    assert_eq!(refused["disposition"]["cause"], "invalid-request");
    assert_eq!(refused["disposition"]["member"], "extra");
    assert_eq!(summary["records"], 5);
    assert_eq!(summary["refused"], 1);
    let clean = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &records(&[2, 3, 9], true),
    )?;
    let (clean_events, clean_summary) = stream(&clean, 10)?;
    let adjusted = events
        .into_iter()
        .filter(|event| event["category"] != "refusal")
        .map(|mut event| {
            if event["record"].as_u64().expect("record") > 1 {
                event["record"] = json!(event["record"].as_u64().expect("record") - 1);
            }
            event
        })
        .collect::<Vec<_>>();
    assert_eq!(adjusted, clean_events);
    assert_eq!(summary["clauses"], clean_summary["clauses"]);
    for event in adjusted.iter().filter(|event| event["settled"] == true) {
        assert_eq!(event["position"], 2);
        assert_eq!(event["settlement"]["step"], "3");
    }
    Ok(())
}

/// Trace: FR-025-AC-5
/// Provenance: QSL-2473; FSL FR-644 malformed record forms and unchanged session
#[test]
fn malformed_records_are_refused_individually_and_processing_continues() -> TestResult {
    for bad in [
        b"{".as_slice(),
        &[0xff],
        b"[]",
        b"{}",
        b"{\"end\":\"closed\"}",
        b"{\"through\":20}",
        b"{\"progress\":{}}",
        b"{\"observation\":\"unbound-key\"}",
        b"{\"observation\":{\"sample\":{\"wrong\":{\"integer\":\"2\"}}}}",
    ] {
        let case = case()?;
        let mut input = bad.to_vec();
        input.push(b'\n');
        input.extend(records(&[9], true));
        let output = case.run_stdin(
            &["monitor", "inputs/monitor.json", "--format", "json"],
            &input,
        )?;
        let (events, summary) = stream(&output, 10)?;
        assert_eq!(events.len(), 3, "{events:?}");
        assert_eq!(events[0]["category"], "refusal", "{events:?}");
        assert_eq!(events[0]["record"], 0);
        assert!(events[0]["clause"].is_null());
        assert_eq!(events[1]["record"], 1);
        assert_eq!(events[1]["position"], 0);
        assert_eq!(events[1]["category"], "violation");
        assert_eq!(events[2]["clause"], "DesiredSeen");
        assert_eq!(events[2]["category"], "success");
        assert_eq!(summary["records"], 3);
        assert_eq!(summary["refused"], 1);
    }
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-644 explicit end is the final record
#[test]
fn explicit_end_prevents_later_observations_from_changing_the_prefix_reading() -> TestResult {
    let case = case()?;
    let input = encoded(&[
        observation(2),
        json!({"end":true}),
        observation(9),
        json!({"end":true}),
    ]);
    let output = case.run_stdin(
        &["monitor", "inputs/monitor.json", "--format", "json"],
        &input,
    )?;
    let (events, summary) = stream(&output, 0)?;
    assert_eq!(summary["records"], 2);
    assert_eq!(events.len(), 2);
    assert!(events
        .iter()
        .all(|event| event["settled"] == false && event["record"] == 0));
    assert_eq!(
        summary_clause(&summary, "DesiredSeen")["category"],
        "pending"
    );
    Ok(())
}

/// Provenance: QSL-2473; FSL FR-644 malformed closed-object records
#[test]
#[ignore = "QSL-2671: explicit blank records require a refusal event, not silent omission"]
fn blank_records_are_distinct_from_an_empty_trace() -> TestResult {
    for line in [b"".as_slice(), b"   "] {
        let case = case()?;
        let mut input = line.to_vec();
        input.push(b'\n');
        input.extend(records(&[9], true));
        let output = case.run_stdin(
            &["monitor", "inputs/monitor.json", "--format", "json"],
            &input,
        )?;
        let (events, summary) = stream(&output, 10)?;
        assert_eq!(events.len(), 3);
        assert_eq!(events[0]["category"], "refusal");
        assert!(events[0]["clause"].is_null());
        assert_eq!(events[0]["record"], 0);
        assert_eq!(summary["refused"], 1);
    }
    Ok(())
}
