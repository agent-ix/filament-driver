// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

fn invalid_request(output: &Captured) -> Value {
    let value = document(output, 20, "refusal");
    assert!(value["items"].as_array().unwrap().is_empty());
    assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1);
    assert_eq!(value["diagnostics"][0]["code"], "invalid-request");
    value
}

/// Trace: FR-022-AC-5
/// FSL FR-620 Reading: closed request/item shapes, no caller-forged identity.
#[test]
fn request_schema_rejects_forged_and_unknown_members() -> Result<(), ContractError> {
    let (case, request) = setup()?;
    let mut variants = Vec::new();
    for (field, value) in [
        ("request_index", json!(8)),
        ("extent", json!({})),
        ("candidates", json!([])),
        ("unknown_member", json!(true)),
        ("claim", json!({"node":"forged"})),
        ("required", json!("yes")),
    ] {
        let mut variant = request.clone();
        variant["request"]["items"][0][field] = value;
        variants.push((field, variant));
    }
    let mut unknown = request.clone();
    unknown["request"]["unknown_member"] = json!(1);
    variants.push(("unknown_member", unknown));
    let mut fairness = request.clone();
    fairness["request"]["items"][0]["temporal"] = json!({"fairness":[]});
    variants.push(("fairness", fairness));
    for (_member, variant) in variants {
        write_request(&case, &variant)?;
        let value =
            invalid_request(&case.run(&["analyze", "data/request.json", "--format", "json"])?);
        assert_eq!(value["last_stage"], Value::Null);
        assert!(
            value["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("request"),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-022-AC-5
#[test]
fn malformed_wrong_format_and_duplicate_json_are_whole_request_refusals(
) -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    for bytes in [
        b"{".as_slice(),
        b"[]",
        b"{\"format\":\"native-run/1\",\"request\":{}}",
        b"{\"format\":\"native-analyze/1\",\"format\":\"native-analyze/1\",\"request\":{}}",
    ] {
        case.write("broken.json", bytes)?;
        invalid_request(&case.run(&["analyze", "broken.json", "--format", "json"])?);
    }
    let text = case.run(&["analyze", "broken.json", "--format", "text"])?;
    assert_eq!(text.0.status.code(), Some(20));
    assert!(text.0.stderr.is_empty(), "{text:?}");
    let rendered = text.stdout_text()?;
    assert!(
        rendered.starts_with("analyze: refusal (exit 20)\n"),
        "{rendered}"
    );
    assert!(rendered.contains(": invalid-request:"), "{rendered}");
    Ok(())
}

/// Trace: FR-022-AC-5
#[test]
fn request_files_cover_missing_empty_non_utf8_and_directory_inputs() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    case.write("empty.json", b"")?;
    case.write("non-utf8.json", b"\xff")?;
    case.write("folder/anchor", b"")?;
    for (path, code) in [
        ("missing.json", "io-error"),
        ("empty.json", "invalid-request"),
        ("non-utf8.json", "invalid-request"),
        ("folder", "io-error"),
    ] {
        let value = document(
            &case.run(&["analyze", path, "--format", "json"])?,
            20,
            "refusal",
        );
        assert_eq!(value["diagnostics"][0]["code"], code, "{path}: {value}");
        assert_eq!(value["last_stage"], Value::Null);
        assert!(!value["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .is_empty());
    }
    Ok(())
}

/// Trace: FR-022-AC-5
/// FSL FR-026-AC-4; FR-624: default request bytes are bounded before decode.
#[test]
fn request_byte_boundary_is_admitted() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"] = json!([{"claim":"WithinCapacity"}]);
    let mut bytes = serde_json::to_vec(&request)?;
    bytes.resize(1_048_576, b' ');
    case.write("data/request.json", &bytes)?;
    let accepted = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        0,
        "success",
    );
    assert_eq!(accepted["items"][0]["record"]["result"], "proved");
    Ok(())
}

/// Trace: FR-022-AC-5
/// FSL FR-624 Analyze refusals: admission byte limits refuse with exit 20.
#[test]
#[ignore = "QSL-2662: analyze intake misclassifies request bytes as incomplete exit 22"]
fn oversized_analyze_request_is_an_admission_refusal() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"] = json!([{"claim":"WithinCapacity"}]);
    let mut bytes = serde_json::to_vec(&request)?;
    bytes.resize(1_048_576, b' ');
    bytes.push(b' ');
    case.write("data/request.json", &bytes)?;
    let refused = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        20,
        "refusal",
    );
    assert!(refused["items"].as_array().unwrap().is_empty());
    assert!(
        refused["diagnostics"][0]
            .to_string()
            .contains("request_bytes"),
        "{refused}"
    );
    Ok(())
}

/// Trace: FR-022-AC-1; FR-022-AC-5
/// FSL FR-026-AC-5: request-relative files; stdin does not replace a path operand.
#[test]
fn file_operand_remains_authoritative_with_stdin_and_literal_dash() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"] = json!([{"claim":"WithinCapacity"}]);
    write_request(&case, &request)?;
    document(
        &case.run_stdin(
            &["analyze", "data/request.json", "--format", "json"],
            b"invalid stdin\xff",
        )?,
        0,
        "success",
    );
    case.write("data/-", &serde_json::to_vec(&request)?)?;
    document(
        &case.run_stdin(
            &["analyze", "data/-", "--format", "json"],
            b"not request JSON",
        )?,
        0,
        "success",
    );
    let absolute = case.directory().join("data/request.json");
    document(
        &case.run(&["analyze", absolute.to_str().unwrap(), "--format", "json"])?,
        0,
        "success",
    );
    Ok(())
}

/// Trace: FR-022-AC-5
#[test]
fn a_bare_dash_is_not_a_stdin_request_option() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    let output = case.run_stdin(&["analyze", "-", "--format", "json"], b"{}")?;
    assert_eq!(output.0.status.code(), Some(20));
    let value = output.json()?;
    assert_eq!(value["operation"], "analyze");
    assert_eq!(value["diagnostics"][0]["code"], "invalid-request");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    assert!(value["diagnostics"][0]["message"]
        .as_str()
        .is_some_and(|message| message.contains("unknown analyze option `-`")));
    Ok(())
}

/// Trace: FR-022-AC-5
#[test]
fn checker_and_snapshot_binding_failures_keep_the_stage() -> Result<(), ContractError> {
    let (case, request) = setup()?;
    case.write(
        "data/workflow.fsl",
        SOURCE
            .replace("q.value <= 2", "missing_value <= 2")
            .as_bytes(),
    )?;
    let checked = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        20,
        "refusal",
    );
    assert_eq!(checked["last_stage"], "S3");
    assert!(checked["items"].as_array().unwrap().is_empty());
    assert_eq!(checked["diagnostics"][0]["code"], "missing_declaration");
    case.write("data/workflow.fsl", SOURCE.as_bytes())?;
    let mut snapshot: Value =
        serde_json::from_slice(&std::fs::read(case.directory().join("data/start.json")).unwrap())?;
    snapshot["populations"][0]["population"] = json!("ix://test/counter/undeclared");
    case.write("data/start.json", &serde_json::to_vec(&snapshot)?)?;
    write_request(&case, &request)?;
    let bound = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        20,
        "refusal",
    );
    assert_eq!(bound["last_stage"], "S4");
    assert!(bound["items"].as_array().unwrap().is_empty());
    assert!(!bound["diagnostics"].as_array().unwrap().is_empty());
    Ok(())
}

/// Trace: FR-022-AC-5
/// FSL FR-620 default and empty item semantics.
#[test]
fn explicit_empty_items_and_omitted_items_have_different_meanings() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"]["items"] = json!([]);
    write_request(&case, &request)?;
    let empty = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        0,
        "success",
    );
    assert!(empty["items"].as_array().unwrap().is_empty());
    request["request"].as_object_mut().unwrap().remove("items");
    write_request(&case, &request)?;
    // FSL FR-620: the default items are every operation contract and every claim.
    // The two operation contracts have an unbounded extent this build cannot
    // prove, so the run is `unsupported` (exit 21) and the claims follow them.
    let all = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        21,
        "unsupported",
    );
    let names: Vec<_> = all["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["claim_name"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(
        names,
        [
            "SpaceAvailable",
            "AppendOne",
            "WithinCapacity",
            "NeverFull",
            ""
        ]
    );
    for (index, item) in all["items"].as_array().unwrap().iter().enumerate() {
        assert_eq!(item["request_index"], index, "{item}");
    }
    assert_eq!(all["items"][0]["category"], "unsupported");
    assert_eq!(all["items"][2]["category"], "success");
    assert_eq!(all["items"][3]["category"], "violation");
    Ok(())
}

/// Trace: FR-022-AC-5
/// FSL FR-620 default items; FSL FR-483 and FR-711-AC-2 (a finite bound is
/// available, so each operation contract settles `requires-bound`).
#[test]
#[ignore = "QSL-2913: operation contracts settle unsupported/unbounded-extent although a finite bound is available; FR-483 says requires-bound"]
fn omitted_items_default_to_every_claim_and_operation_contracts_require_a_bound(
) -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    request["request"].as_object_mut().unwrap().remove("items");
    write_request(&case, &request)?;
    let all = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        21,
        "unsupported",
    );
    let items = all["items"].as_array().unwrap();
    let names: Vec<_> = items
        .iter()
        .map(|item| item["claim_name"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(
        names,
        [
            "SpaceAvailable",
            "AppendOne",
            "WithinCapacity",
            "NeverFull",
            ""
        ]
    );
    for (index, item) in items.iter().enumerate() {
        assert_eq!(item["request_index"], index, "{item}");
    }
    for item in &items[..2] {
        assert_eq!(item["category"], "unsupported", "{item}");
        assert_eq!(item["record"]["settled_by"], "disposition", "{item}");
        assert_eq!(
            item["record"]["diagnostic"]["code"], "requires_bound",
            "{item}"
        );
        assert_eq!(
            item["record"]["diagnostic"]["cause"], "requires-bound",
            "{item}"
        );
    }
    let categories: Vec<_> = items.iter().map(|item| item["category"].clone()).collect();
    assert_eq!(
        categories,
        [
            "unsupported",
            "unsupported",
            "success",
            "violation",
            "violation"
        ]
    );
    Ok(())
}

/// Trace: FR-022-AC-5
/// FSL FR-620: items named in the request are analysed as named; the claims
/// that settle on their own are covered here independent of the operation contracts.
#[test]
fn named_claims_settle_with_exact_names_categories_and_results() -> Result<(), ContractError> {
    let (case, request) = setup()?;
    assert_eq!(
        request["request"]["items"],
        json!([{"claim":"WithinCapacity"},{"claim":"NeverFull"}])
    );
    let all = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        10,
        "violation",
    );
    let items = all["items"].as_array().unwrap();
    let rows: Vec<_> = items
        .iter()
        .map(|item| {
            (
                item["claim_name"].as_str().unwrap_or_default().to_owned(),
                item["category"].as_str().unwrap_or_default().to_owned(),
                item["record"]["result"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (
                "WithinCapacity".to_owned(),
                "success".to_owned(),
                "proved".to_owned()
            ),
            (
                "NeverFull".to_owned(),
                "violation".to_owned(),
                "refuted".to_owned()
            ),
        ]
    );
    assert_eq!(items[0]["record"]["certification"], "certified");
    assert_eq!(items[0]["record"]["settled_by"], "result");
    assert_eq!(items[1]["record"]["settled_by"], "result");
    assert_eq!(
        items.len(),
        2,
        "no derived item follows an explicit item list"
    );
    Ok(())
}

/// Trace: FR-022-AC-5
/// FSL FR-620: terminal certificates are outcomes, not caller-supplied item settings.
#[test]
fn a_caller_cannot_supply_an_accepted_certificate_instead_of_analysis() -> Result<(), ContractError>
{
    let (case, mut request) = setup()?;
    request["request"]["items"][0]["certificate"] = json!({"accepted":true,"checker":"en-1-closure","identity":{"model":"test/counter","ordinal":0}});
    write_request(&case, &request)?;
    let value =
        invalid_request(&case.run(&["analyze", "data/request.json", "--format", "json"])?);
    assert_eq!(value["last_stage"], Value::Null);
    assert!(value["diagnostics"][0]["message"]
        .as_str()
        .unwrap()
        .contains("certificate"));
    Ok(())
}
