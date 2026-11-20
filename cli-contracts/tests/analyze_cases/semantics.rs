// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-022-AC-1; FR-022-AC-2; FR-022-AC-8
/// FSL FR-621-AC-8: a refuted required item leaves an aggregate proof gap.
#[test]
fn a_reachable_full_queue_refutes_never_full_but_proves_capacity() -> Result<(), ContractError> {
    let (case, _) = setup()?;
    let output = case.run(&["analyze", "data/request.json", "--format", "json"])?;
    let value = document(&output, 10, "violation");
    assert_eq!(
        value["aggregate"],
        json!({"status":"incomplete","gaps":[{"reason":"not-proved","request_index":1,"result":"refuted"}]})
    );
    assert_eq!(value["last_stage"], "S8");
    assert_eq!(value["items"].as_array().unwrap().len(), 2);
    let proved = &value["items"][0];
    proof(proved);
    let refuted = &value["items"][1];
    assert_eq!(refuted["claim_name"], "NeverFull");
    assert_eq!(refuted["request_index"], 1);
    assert_eq!(refuted["record"]["result"], "refuted");
    assert_eq!(refuted["record"]["backend"], "en-1");
    assert_eq!(refuted["record"]["settled_by"], "result");
    assert_eq!(
        refuted["replay"],
        json!({"reproduced":true,"target":{"state_clause":"NeverFull"}})
    );
    let counterexample = &refuted["counterexample"];
    assert_eq!(counterexample["trace"], "model");
    assert_eq!(counterexample["trace_position"], 2);
    assert_eq!(counterexample["over_binding"]["identity"], "queue-east");
    let steps = counterexample["prefix"].as_array().expect("witness steps");
    assert_eq!(steps.len(), 2);
    for (step, expected) in steps.iter().zip(["1", "2"]) {
        assert_eq!(
            step["transition"]["operation"]["node"],
            "ix://test/counter/Counter/inc"
        );
        assert_eq!(step["transition"]["receiver"]["identity"], "queue-east");
        let state =
            &step["post_state"]["state"]["semantic"]["entries"][0]["value"]["entries"][0]["value"];
        assert_eq!(state["fields"][0]["name"], "value");
        assert_eq!(
            state["fields"][0]["value"],
            json!({"type":"integer","value":expected})
        );
    }
    Ok(())
}

fn proof(item: &Value) {
    assert_eq!(item["claim_name"], "WithinCapacity");
    assert_eq!(item["record"]["result"], "proved");
    assert_eq!(item["record"]["certification"], "certified");
    assert_eq!(item["record"]["basis"], json!({"type":"exhaustive"}));
    assert_eq!(item["record"]["backend"], "en-1");
    assert_eq!(item["certificate"]["accepted"], true);
    assert_eq!(item["certificate"]["checker"], "en-1-closure");
    assert_eq!(
        item["certificate"]["identity"]["node_id"],
        item["request"]["claim"]["node"]
    );
    assert_eq!(item["certificate"]["identity"]["model"], "test/counter");
    assert_eq!(item["exploration"]["states"], "3");
    assert_eq!(item["exploration"]["transitions"], "2");
    assert!(item.get("counterexample").is_none());
    assert_eq!(item["request"]["candidates"]["backends"], json!(["en-1"]));
}

/// Trace: FR-022-AC-1; FR-022-AC-8
#[test]
fn complete_certified_proof_has_no_aggregate_gap() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"] = json!([{"claim":"WithinCapacity"}]);
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        0,
        "success",
    );
    assert_eq!(value["aggregate"], json!({"status":"complete","gaps":[]}));
    assert_eq!(value["last_stage"], "S6c");
    proof(&value["items"][0]);
    assert!(value["registry"]
        .as_array()
        .unwrap()
        .iter()
        .any(|engine| engine["backend"] == "en-1"));
    Ok(())
}

/// Trace: FR-022-AC-3
#[test]
fn exploration_budget_cannot_be_reported_as_proof() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"] =
        json!([{"claim":"WithinCapacity","temporal":{"limits":{"max_states":1}}}]);
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        22,
        "incomplete",
    );
    let item = &value["items"][0];
    assert_eq!(item["record"]["result"], "incomplete");
    assert!(
        item["record"]["cause"].to_string().contains("max_states"),
        "{item}"
    );
    assert!(item.get("counterexample").is_none());
    assert!(item.get("certificate").is_none());
    assert_eq!(value["aggregate"]["status"], "incomplete");
    assert_eq!(item["request"]["temporal"]["limits"]["max_states"], 1);
    Ok(())
}

/// Trace: FR-022-AC-3
#[test]
fn an_expired_whole_request_deadline_is_incomplete() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["deadline_ms"] = json!(0);
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        22,
        "incomplete",
    );
    assert_eq!(value["last_stage"], Value::Null);
    assert_eq!(value["items"].as_array().unwrap().len(), 2, "{value}");
    for item in value["items"].as_array().unwrap() {
        assert_eq!(item["record"]["result"], "incomplete", "{item}");
        assert_eq!(
            item["record"]["cause"],
            json!({"kind":"cancelled","source":"deadline"}),
            "{item}"
        );
        assert!(item.get("certificate").is_none());
        assert!(item.get("counterexample").is_none());
    }
    Ok(())
}

/// Trace: FR-022-AC-4; FR-022-AC-7
#[test]
fn unknown_backend_keeps_the_other_item_and_precise_disposition() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"][1]["backend"] = json!("contract-missing-engine");
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        20,
        "refusal",
    );
    proof(&value["items"][0]);
    let item = &value["items"][1];
    assert_eq!(item["request_index"], 1);
    assert_eq!(item["record"]["settled_by"], "disposition");
    assert!(item["record"].get("backend").is_none());
    assert_eq!(item["record"]["diagnostic"]["code"], "invalid_capability");
    assert_eq!(item["record"]["diagnostic"]["cause"], "unknown-backend");
    assert!(item["record"]["diagnostic"]
        .to_string()
        .contains("contract-missing-engine"));
    let text = case.run(&["analyze", "data/request.json"])?;
    assert_eq!(text.0.status.code(), Some(20));
    assert!(text
        .stdout_text()?
        .contains("1 NeverFull invalid-request invalid_capability/unknown-backend"));
    Ok(())
}

/// Trace: FR-022-AC-4
#[test]
fn an_existing_engine_without_the_requested_form_is_unsupported() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"] = json!([{"claim":"WithinCapacity","backend":"en-6"}]);
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        21,
        "unsupported",
    );
    let record = &value["items"][0]["record"];
    assert_eq!(record["result"], "unsupported");
    assert_eq!(record["diagnostic"]["code"], "unsupported_projection");
    assert_eq!(
        record["diagnostic"]["cause"],
        "unsupported-requested-capability"
    );
    Ok(())
}

/// Trace: FR-022-AC-5
/// FSL FR-620: per-item name/kind refusals do not suppress other items.
#[test]
fn unknown_claim_and_inapplicable_kind_are_local_dispositions() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    for (item, cause) in [
        (json!({"claim":"UnknownQueueRule"}), "unknown-claim"),
        (
            json!({"claim":"NeverFull","kind":"value-validity"}),
            "inapplicable-kind",
        ),
    ] {
        request["request"]["items"] = json!([item,{"claim":"WithinCapacity"}]);
        write_request(&case, &request)?;
        let value = document(
            &case.run(&["analyze", "data/request.json", "--format", "json"])?,
            20,
            "refusal",
        );
        let refused = &value["items"][0];
        assert_eq!(refused["request_index"], 0);
        assert_eq!(refused["record"]["settled_by"], "disposition");
        assert_eq!(
            refused["record"]["diagnostic"]["code"],
            "invalid_capability"
        );
        assert_eq!(refused["record"]["diagnostic"]["cause"], cause, "{refused}");
        assert!(refused["record"].get("backend").is_none());
        assert_eq!(value["items"][1]["request_index"], 1);
        proof(&value["items"][1]);
    }
    Ok(())
}

/// Trace: FR-022-AC-2
/// FSL FR-621 aggregate considers required items; violation remains a violation.
#[test]
fn an_optional_refutation_does_not_claim_incomplete_required_proofs() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"][1]["required"] = json!(false);
    write_request(&case, &request)?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        10,
        "violation",
    );
    assert_eq!(value["aggregate"], json!({"status":"complete","gaps":[]}));
    assert_eq!(value["items"][1]["record"]["result"], "refuted");
    assert_eq!(value["items"][1]["replay"]["reproduced"], true);
    Ok(())
}
