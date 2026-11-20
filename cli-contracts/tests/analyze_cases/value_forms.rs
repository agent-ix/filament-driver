// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-022-AC-1; FR-022-AC-2
/// FSpec FR-483/484: finite value validity is exhaustive, not core-certified.
#[test]
fn finite_boolean_functions_are_proved_or_refuted_at_their_actual_integer_domains(
) -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    case.write(
        "integers.fsl",
        br#"language "ix:native" edition "1-draft";
profile values = "filament.value.complete/v1";
function IncrementGrows using values(n: Int[0, 4]): Boolean pure { n + 1 > n }
function UnderFive using values(n: Int[0, 4]): Boolean pure { n + 1 < 5 }
"#,
    )?;
    case.write("integers.json",&serde_json::to_vec(&json!({"format":"native-analyze/1","request":{
        "models":[],"program":{"source":{"file":"integers.fsl","authority":"contracts","identity":"finite-arithmetic","document":"FiniteArithmetic","formal_revision":1}},
        "items":[{"claim":"IncrementGrows","kind":"value-validity"},{"claim":"UnderFive","kind":"value-validity"}]
    }}))?)?;
    let value = document(
        &case.run(&["analyze", "integers.json", "--format", "json"])?,
        10,
        "violation",
    );
    let proved = &value["items"][0];
    assert_eq!(proved["record"]["result"], "proved");
    assert_eq!(proved["record"]["backend"], "value-checker");
    assert_eq!(proved["record"]["basis"], json!({"type":"exhaustive"}));
    assert_eq!(proved["record"]["certification"], "uncertified");
    assert!(proved.get("certificate").is_none());
    let refuted = &value["items"][1];
    assert_eq!(refuted["record"]["result"], "refuted", "{refuted}");
    assert_eq!(refuted["replay"]["reproduced"], true);
    assert_eq!(refuted["counterexample"]["type"], "assignments");
    assert_eq!(
        refuted["counterexample"]["values"][0]["value"],
        json!({"type":"integer","value":"4"})
    );
    Ok(())
}

/// Trace: FR-022-AC-1; FR-022-AC-2
/// FSL FR-632: a timed subject uses EN-6 rather than discrete model steps.
#[test]
fn a_timed_capacity_window_is_certified_while_unforced_completion_is_refuted(
) -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    let timed = r#"
dimension Time;
unit second : Time = rational(1, 1);
time Queue dense unit second;
temporal TimedCapacity using timed over (q: Queue::Counter) clock "model-time" on origin {
  always[0 second, 2 second] holds(q.value <= 2)
}
temporal PromptlyFull using timed over (q: Queue::Counter) clock "model-time" on origin {
  eventually[0 second, 2 second] holds(q.value = 2)
}
"#;
    let source = SOURCE.replace(
        "profile values",
        "profile temporalLayer = \"filament.temporal.complete/v1\";\nprofile timed = \"filament.temporal.timed/v1\";\nprofile values",
    );
    case.write("data/workflow.fsl", format!("{source}\n{timed}").as_bytes())?;
    request["request"]["items"] = json!([{"claim":"TimedCapacity"},{"claim":"PromptlyFull"}]);
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        10,
        "violation",
    );
    let capacity = &value["items"][0];
    assert_eq!(capacity["record"]["result"], "proved", "{capacity}");
    assert_eq!(capacity["record"]["backend"], "en-6");
    assert_eq!(capacity["record"]["certification"], "certified");
    assert_eq!(capacity["record"]["basis"]["type"], "zone-certified");
    assert!(capacity.get("counterexample").is_none());
    let deadline = &value["items"][1];
    assert_eq!(deadline["record"]["result"], "refuted", "{deadline}");
    assert_eq!(deadline["record"]["backend"], "en-6");
    assert_eq!(deadline["replay"]["reproduced"], true);
    assert!(!deadline["counterexample"]["prefix"]
        .as_array()
        .unwrap()
        .is_empty());
    Ok(())
}

/// Trace: FR-022-AC-11
/// FSL FR-1019: the default CLI supplies its reference hybrid provider.
#[test]
#[ignore = "QSL-2234: default hybrid reference provider is absent from the frozen CLI registry"]
fn a_hybrid_reservoir_has_a_checked_bound_and_a_real_violation() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    case.write(
        "reservoir.fsl",
        br#"language "ix:native" edition "1-draft";
profile continuous = "filament.temporal.infinite-trace/v1";
hybrid Reservoir using continuous {
  mode Filling invariant { volume >= 1 and volume <= 3 }
    flow { volume' = 1; }
    transition to Draining when { volume = 3 } reset { volume = 3 };
  mode Draining invariant { volume >= 1 and volume <= 3 }
    flow { volume' = -1; }
    transition to Filling when { volume = 1 } reset { volume = 1 };
}
temporal BelowRim using continuous over (r: Reservoir) clock "model-time" on origin {
  always[0, 2] holds(r.volume < 4)
}
temporal BelowTwo using continuous over (r: Reservoir) clock "model-time" on origin {
  always[0, 2] holds(r.volume < 2)
}
"#,
    )?;
    case.write("reservoir.json",&serde_json::to_vec(&json!({"format":"native-analyze/1","request":{
        "models":[],"program":{"source":{"file":"reservoir.fsl","authority":"contracts","identity":"reservoir-cycle","document":"ReservoirCycle","formal_revision":1}},
        "items":[{"claim":"BelowRim"},{"claim":"BelowTwo"}]
    }}))?)?;
    let value = document(
        &case.run(&["analyze", "reservoir.json", "--format", "json"])?,
        10,
        "violation",
    );
    assert_eq!(value["items"][0]["record"]["result"], "proved");
    assert_eq!(
        value["items"][0]["record"]["backend"],
        "fsl-hybrid-reference"
    );
    assert_eq!(value["items"][0]["record"]["certification"], "certified");
    assert_eq!(value["items"][0]["certificate"]["accepted"], true);
    assert_eq!(value["items"][1]["record"]["result"], "refuted");
    assert_eq!(
        value["items"][1]["record"]["backend"],
        "fsl-hybrid-reference"
    );
    assert_eq!(value["items"][1]["replay"]["reproduced"], true);
    assert!(!value["items"][1]["counterexample"].is_null());
    assert!(value["registry"]
        .as_array()
        .unwrap()
        .iter()
        .any(|engine| engine["backend"] == "fsl-hybrid-reference"));
    Ok(())
}

/// Trace: FR-022-AC-1
/// Timed unbounded safety is a genuine three-state obligation, not a tautology.
#[test]
fn unbounded_timed_safety_keeps_its_certified_scope() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    let source=SOURCE.replace("profile values","profile temporalLayer = \"filament.temporal.complete/v1\";\nprofile timed = \"filament.temporal.timed/v1\";\nprofile values");
    let timed = r#"
dimension Time;
unit second : Time = rational(1, 1);
time Queue dense unit second;
temporal UnlimitedCapacity using timed over (q: Queue::Counter) clock "model-time" on origin {
  always holds(q.value <= 2)
}
"#;
    case.write("data/workflow.fsl", format!("{source}\n{timed}").as_bytes())?;
    request["request"]["items"] = json!([{"claim":"UnlimitedCapacity"}]);
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        0,
        "success",
    );
    let item = &value["items"][0];
    assert_eq!(item["record"]["result"], "proved");
    assert_eq!(item["record"]["backend"], "en-6");
    assert_eq!(item["record"]["certification"], "certified");
    assert!(item.get("counterexample").is_none());
    assert_eq!(value["aggregate"]["status"], "complete");
    Ok(())
}

/// Trace: FR-022-AC-12
/// The live antecedent at queue value1 makes this a nonvacuous timed LTL transport case.
#[test]
fn unbounded_timed_liveness_retains_the_non_zeno_record() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    let source=SOURCE.replace("profile values","profile temporalLayer = \"filament.temporal.complete/v1\";\nprofile timed = \"filament.temporal.timed/v1\";\nprofile values");
    let timed = r#"
dimension Time;
unit second : Time = rational(1, 1);
time Queue dense unit second;
temporal ActiveProgress using timed over (q: Queue::Counter) clock "model-time" on origin {
  always (holds(q.value = 1) implies eventually holds(q.value >= 1))
}
"#;
    case.write("data/workflow.fsl", format!("{source}\n{timed}").as_bytes())?;
    request["request"]["items"] = json!([{"claim":"ActiveProgress"}]);
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        0,
        "success",
    );
    let item = &value["items"][0];
    assert_eq!(item["record"]["result"], "proved");
    assert_eq!(item["record"]["backend"], "en-6");
    assert_eq!(item["record"]["certification"], "certified");
    assert!(
        ["not-required", "assumed", "checked"]
            .contains(&item["record"]["non_zeno"].as_str().unwrap_or("")),
        "{item}"
    );
    assert!(item.get("counterexample").is_none());
    assert_eq!(value["aggregate"]["status"], "complete");
    Ok(())
}
