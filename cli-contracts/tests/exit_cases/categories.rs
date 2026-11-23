// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-002-AC-2, FR-002-AC-4
/// Provenance: QSL-2476; FSL FR-285 undefined row.
#[test]
fn undefined_evaluation_is_ten_and_keeps_its_distinct_label() -> Result<(), ContractError> {
    let case = fixture()?;
    case.write(
        "fraction.fsl",
        br#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
type Fraction = Rational[-8, 8; 1, 8];
function AsFraction using v(x: Float64[nearest-even]): Fraction pure { convert<Fraction>(x) }
"#,
    )?;
    write_json(
        &case,
        "undefined.json",
        &json!({"format":"native-run/1","request":{
            "models":[],"program":source_ref("fraction.fsl"),"call":{"function":"AsFraction",
            "arguments":[{"parameter":"x","value":{"type":"float64","bits":"7ff8000000000000"}}]}
        }}),
    )?;
    let value = outcome(
        &case.run(&["run", "undefined.json", "--format", "json"])?,
        "execute",
        "undefined",
    )?;
    assert_eq!(
        value["result"],
        json!({"kind":"undefined","reason":"ieee-not-finite"})
    );
    Ok(())
}

fn analysis(claim: &str) -> Value {
    json!({"format":"native-analyze/1","request":{
        "models":[],"program":source_ref("program.fsl"),"items":[{"claim":claim,"kind":"value-validity"}]
    }})
}

/// Trace: FR-002-AC-2, FR-002-AC-4, FR-022-AC-1, FR-022-AC-2
/// Provenance: QSL-2476; FSL FR-285 success/violation rows.
#[test]
fn proved_and_refuted_analysis_exit_zero_and_ten() -> Result<(), ContractError> {
    let case = fixture()?;
    for (claim, category, result) in [
        ("PositiveShift", "success", "proved"),
        ("BelowFour", "violation", "refuted"),
    ] {
        write_json(&case, "analyze.json", &analysis(claim))?;
        let value = outcome(
            &case.run(&["analyze", "analyze.json", "--format", "json"])?,
            "analyze",
            category,
        )?;
        assert_eq!(value["items"][0]["claim_name"], claim);
        assert_eq!(value["items"][0]["record"]["result"], result, "{value}");
        if category == "violation" {
            assert_eq!(value["items"][0]["replay"]["reproduced"], true, "{value}");
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-2, FR-002-AC-4, FR-022-AC-3, FR-022-AC-4
/// Provenance: QSL-2476; FSL FR-285 refusal/unsupported/incomplete rows.
#[test]
fn analysis_dispositions_follow_the_category_table() -> Result<(), ContractError> {
    let case = fixture()?;
    for (backend, deadline, category, result) in [
        (
            Some("no-such-exit-engine"),
            None,
            "refusal",
            "invalid-request",
        ),
        (Some("en-1"), None, "unsupported", "unsupported"),
        (None, Some(0), "incomplete", "incomplete"),
    ] {
        let mut input = analysis("PositiveShift");
        if let Some(backend) = backend {
            input["request"]["items"][0]["backend"] = json!(backend);
        }
        if let Some(deadline) = deadline {
            input["request"]["deadline_ms"] = json!(deadline);
        }
        write_json(&case, "disposition.json", &input)?;
        let value = outcome(
            &case.run(&["analyze", "disposition.json", "--format", "json"])?,
            "analyze",
            category,
        )?;
        let record = &value["items"][0]["record"];
        if category == "incomplete" {
            assert_eq!(record["result"], result, "{value}");
        } else {
            assert_eq!(record["settled_by"], "disposition", "{value}");
            assert_eq!(
                record["diagnostic"]["code"],
                if category == "refusal" {
                    "invalid_capability"
                } else {
                    "unsupported_projection"
                },
                "{value}"
            );
        }
        assert!(value["items"][0].get("certificate").is_none(), "{value}");
    }
    Ok(())
}

/// Trace: FR-002-AC-2, FR-002-AC-4, FR-022-AC-9
/// Provenance: QSL-2476; FSL FR-285 inconclusive row.
#[test]
fn an_unsettled_provider_claim_exits_twenty_two_not_pending_zero() -> Result<(), ContractError> {
    let case = fixture()?;
    case.write(
        "program.fsl",
        format!(
            "{SOURCE}\nfunction UnboundedShift using v(n: Integer): Boolean pure {{ n + 2 > n }}\n"
        )
        .as_bytes(),
    )?;
    let mut input = analysis("UnboundedShift");
    let backend = "exit-contracts.inconclusive";
    input["request"]["items"][0]["backend"] = json!(backend);
    write_json(&case, "inconclusive.json", &input)?;
    let provider =
        std::env::var_os("FSL_TEST_PROVIDER").expect("explicit authoritative test provider");
    let log = case.directory().join("sent.json");
    let shutdown = case.directory().join("shutdown");
    write_json(
        &case,
        "provider.json",
        &json!({"format":"filament.provider-config/v1","providers":[{
            "path":std::path::PathBuf::from(provider),"arguments":[backend,"unbounded","failed",log,shutdown]
        }]}),
    )?;
    let value = outcome(
        &case.run(&[
            "analyze",
            "inconclusive.json",
            "--provider-config",
            "provider.json",
            "--provider-limit",
            "deadline_ms=4000",
            "--format",
            "json",
        ])?,
        "analyze",
        "inconclusive",
    )?;
    assert_eq!(value["items"][0]["record"]["result"], "inconclusive");
    assert_eq!(
        value["items"][0]["record"]["cause"],
        json!({"kind":"solver-inconclusive"})
    );
    assert!(case.directory().join("sent.json").is_file());
    assert!(case.directory().join("shutdown").is_file());
    Ok(())
}

/// Trace: FR-002-AC-2, FR-002-AC-4
/// Provenance: QSL-2476; FSL FR-285-AC-12 and FR-501-AC-3/AC-5.
#[test]
fn pending_indeterminate_and_decisive_trace_rows_keep_zero_eleven_and_ten(
) -> Result<(), ContractError> {
    let case = fixture()?;
    case.write(
        "window.fsl",
        br#"language "ix:native" edition "1-draft";
profile t = "filament.temporal.timestamped-event.finite-window/v1";
record Reading { load: Int[0, 10]; }
temporal WindowSafe using t over (r: Reading) clock "window-ms" on origin {
    always[0,5] holds(r.load < 7)
}
"#,
    )?;
    for (watermark, completeness, load, category) in [
        (1, json!({"complete":{"through":1}}), 3, "pending"),
        (5, json!("incomplete"), 3, "indeterminate"),
        (5, json!({"complete":{"through":5}}), 3, "success"),
        (5, json!({"complete":{"through":5}}), 8, "violation"),
    ] {
        let input = json!({"format":"native-run/1","request":{
            "models":[],"program":source_ref("window.fsl"),
            "clocks":{"window-ms":{"kind":"timestamped_event","timestamp_unit":"ms"}},
            "clause":{"selection":{"temporal":{"name":"WindowSafe","trace":{"timestamped":{
                "observations":[{"position":{"sample":{"load":{"integer":load.to_string()}}},"stamp":0}],
                "progress":{"watermark":watermark,"completeness":completeness,"scope":if category == "pending" { "open" } else { "closed" },"execution":"closed"},
                "lower_boundary":"authoritative-origin","binding":{"profile":"filament.temporal.timestamped-event.finite-window/v1",
                    "clock":"window-ms","parameters":{"kind":"timestamped_event","timestamp_unit":"ms"}}
            }}}},"observations":{}}
        }});
        write_json(&case, "window.json", &input)?;
        let value = outcome(
            &case.run(&["run", "window.json", "--format", "json"])?,
            "monitor",
            category,
        )?;
        assert_eq!(value["items"].as_array().expect("items").len(), 1);
        assert_eq!(value["items"][0]["clause"], "WindowSafe");
    }
    Ok(())
}

/// Trace: FR-002-AC-2, FR-002-AC-4
/// Provenance: QSL-2476; Driver TC-009 stream-write failure and FSL FR-285 fault row.
#[test]
#[cfg(target_os = "linux")]
fn a_real_stdout_write_failure_exits_thirty_and_keeps_stderr(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let sink = std::fs::OpenOptions::new().write(true).open("/dev/full")?;
    let output = case.run_stdout_to_file(&["check", "program.fsl", "--format", "json"], sink)?;
    assert_exit(&output, "internal-failure");
    assert!(output.0.stdout.is_empty());
    assert!(
        output.stderr_text()?.contains("OutputFailure"),
        "{output:?}"
    );
    Ok(())
}

/// Trace: FR-002-AC-2, FR-002-AC-4, FR-022-AC-4
/// Provenance: QSL-2476; FSL FR-285 multi-item severity order.
#[test]
fn refusal_outranks_unsupported_and_violation_in_both_item_orders() -> Result<(), ContractError> {
    let case = fixture()?;
    let refuted = json!({"claim":"BelowFour","kind":"value-validity"});
    let unsupported = json!({"claim":"PositiveShift","kind":"value-validity","backend":"en-1"});
    let refused =
        json!({"claim":"PositiveShift","kind":"value-validity","backend":"absent-severity-engine"});
    for items in [
        json!([refuted, unsupported, refused]),
        json!([refused, unsupported, refuted]),
    ] {
        let mut input = analysis("PositiveShift");
        input["request"]["items"] = items;
        write_json(&case, "severity.json", &input)?;
        let value = outcome(
            &case.run(&["analyze", "severity.json", "--format", "json"])?,
            "analyze",
            "refusal",
        )?;
        let mut categories = value["items"]
            .as_array()
            .expect("items")
            .iter()
            .map(|item| item["category"].as_str().expect("item category"))
            .collect::<Vec<_>>();
        categories.sort();
        assert_eq!(categories, ["refusal", "unsupported", "violation"]);
    }
    Ok(())
}

/// Trace: FR-002-AC-2, FR-002-AC-4, FR-025-AC-1, FR-025-AC-2
/// Provenance: QSL-2476; FSL FR-285 trace success/violation rows.
#[test]
fn streamed_monitor_summaries_keep_the_real_zero_or_ten_exit() -> Result<(), ContractError> {
    let case = fixture()?;
    case.write(
        "stream.fsl",
        br#"language "ix:native" edition "1-draft";
profile t = "filament.temporal.infinite-trace/v1";
record Heat { degree: Int[0, 12]; }
temporal StayCool using t over (h: Heat) clock "steps" on origin {
  always[0,1] holds(h.degree < 8)
}
"#,
    )?;
    write_json(
        &case,
        "stream.json",
        &json!({"format":"native-monitor/1","request":{
            "models":[],"program":source_ref("stream.fsl"),"clauses":[{"name":"StayCool"}],
            "trace":{"kind":"finite"}
        }}),
    )?;
    for (last, category) in [(4, "success"), (9, "violation")] {
        let lines = [
            json!({"observation":{"sample":{"degree":{"integer":"3"}}}}),
            json!({"observation":{"sample":{"degree":{"integer":last.to_string()}}}}),
            json!({"end":true}),
        ]
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n")
            + "\n";
        let output = case.run_stdin(
            &["monitor", "stream.json", "--format", "json"],
            lines.as_bytes(),
        )?;
        assert_exit(&output, category);
        assert!(output.0.stderr.is_empty());
        let records = output.json_lines()?;
        let (summary, events) = records.split_last().expect("monitor records");
        assert_eq!(summary["format"], "filament.monitor-summary/1");
        let clauses = summary["clauses"].as_array().expect("summary clauses");
        assert_eq!(clauses.len(), 1, "{summary}");
        assert_eq!(clauses[0]["clause"], "StayCool", "{summary}");
        assert_eq!(clauses[0]["category"], category, "{summary}");
        assert!(
            events
                .iter()
                .any(|event| event["format"] == "filament.monitor-event/1"
                    && event["clause"] == "StayCool"
                    && event["settled"] == true
                    && event["category"] == category),
            "{records:?}"
        );
    }
    Ok(())
}
