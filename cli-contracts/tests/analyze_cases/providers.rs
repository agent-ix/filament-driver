use super::*;

const BACKEND: &str = "contracts.queue-proof-provider";
const IDENTITY: &str =
    "function ShiftPreservesOrder using values(n: Integer): Boolean pure { n + 3 > n }\n";

fn provider_case(result: &str, mode: &str) -> Result<(CliCase, Value), ContractError> {
    let (case, mut request) = setup()?;
    case.write(
        "data/workflow.fsl",
        format!("{SOURCE}\n{IDENTITY}").as_bytes(),
    )?;
    request["request"]["items"] = json!([
        {"claim":"ShiftPreservesOrder","kind":"value-validity","backend":BACKEND},
        {"claim":"WithinCapacity"}
    ]);
    write_request(&case, &request)?;
    let provider = std::env::var_os("FSL_TEST_PROVIDER")
        .expect("set FSL_TEST_PROVIDER to the authoritative frozen-cohort helper");
    assert!(
        std::path::Path::new(&provider).is_file(),
        "the explicitly selected provider must exist"
    );
    let log = case.directory().join("provider-request.json");
    let shutdown = case.directory().join("provider-shutdown");
    case.write("providers.json",&serde_json::to_vec(&json!({
        "format":"filament.provider-config/v1","providers":[{
            "path":std::path::PathBuf::from(provider),"arguments":[BACKEND,mode,result,log.to_str().unwrap(),shutdown.to_str().unwrap()]
        }]
    }))?)?;
    Ok((case, request))
}

fn analyze_provider(case: &CliCase) -> Result<Captured, ContractError> {
    case.run(&[
        "analyze",
        "data/request.json",
        "--provider-config",
        "providers.json",
        "--provider-limit",
        "deadline_ms=4000",
        "--format",
        "json",
    ])
}

fn sent(case: &CliCase) -> Value {
    serde_json::from_slice(
        &std::fs::read(case.directory().join("provider-request.json"))
            .expect("provider received the real request"),
    )
    .expect("provider request JSON")
}

/// Trace: FR-022-AC-9
/// FSL FR-281 Process-provider proofs are trusted, never core-certified.
#[test]
fn a_selected_process_receives_the_real_item_and_reports_explicit_trust(
) -> Result<(), ContractError> {
    let (case, _) = provider_case("proved", "unbounded")?;
    let value = document(&analyze_provider(&case)?, 0, "success");
    let item = &value["items"][0];
    assert_eq!(item["claim_name"], "ShiftPreservesOrder");
    assert_eq!(item["record"]["result"], "proved");
    assert_eq!(item["record"]["backend"], BACKEND);
    assert_eq!(item["record"]["certification"], "trusted");
    assert!(
        item.get("certificate").is_none(),
        "a provider assertion is not a core-checked certificate"
    );
    // Independently real finite semantics remain on EN-1 with its accepted certificate.
    assert_eq!(value["items"][1]["record"]["backend"], "en-1");
    assert_eq!(value["items"][1]["record"]["result"], "proved");
    assert_eq!(value["items"][1]["certificate"]["accepted"], true);
    let request = sent(&case);
    assert_eq!(request["request"]["deadline_ms"], 4000);
    assert_eq!(request["request"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(request["request"]["items"][0]["request_index"], 0);
    assert_eq!(request["request"]["items"][0]["backend"], BACKEND);
    assert_eq!(request["request"]["items"][0]["kind"], "value-validity");
    assert!(value["registry"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["backend"] == BACKEND));
    assert_eq!(
        std::fs::read(case.directory().join("provider-shutdown")).unwrap(),
        b"shutdown"
    );
    Ok(())
}

/// Trace: FR-022-AC-9
#[test]
fn a_process_without_a_verdict_is_not_a_proof() -> Result<(), ContractError> {
    let (case, _) = provider_case("failed", "unbounded")?;
    let value = document(&analyze_provider(&case)?, 22, "inconclusive");
    let item = &value["items"][0];
    assert_eq!(item["record"]["result"], "inconclusive", "{item}");
    assert_eq!(
        item["record"]["cause"],
        json!({"kind":"solver-inconclusive"})
    );
    assert!(item.get("certificate").is_none());
    assert!(item.get("counterexample").is_none());
    assert!(case.directory().join("provider-shutdown").is_file());
    Ok(())
}

/// Trace: FR-022-AC-9
/// FSL FR-281: a malformed provider witness cannot become a refuted record.
#[test]
fn a_malformed_process_witness_is_not_an_accepted_refutation() -> Result<(), ContractError> {
    let (case, _) = provider_case("refuted", "unbounded")?;
    let value = document(&analyze_provider(&case)?, 22, "inconclusive");
    let item = &value["items"][0];
    assert_eq!(item["record"]["result"], "inconclusive", "{item}");
    assert_eq!(
        item["record"]["cause"],
        json!({"kind":"solver-inconclusive"})
    );
    assert!(
        item.get("counterexample").is_none(),
        "unvalidated provider data is not an accepted witness"
    );
    assert!(item.get("certificate").is_none());
    Ok(())
}

fn cancellation(signal: &str) -> Result<(), ContractError> {
    let (case, _) = provider_case(signal, "unbounded")?;
    let value = document(&analyze_provider(&case)?, 22, "incomplete");
    assert_eq!(
        sent(&case)["request"]["items"][0]["request_index"],
        0,
        "signal sent only after the owned provider receives its item"
    );
    assert_eq!(value["items"].as_array().unwrap().len(), 2);
    for item in value["items"].as_array().unwrap() {
        assert_eq!(item["record"]["result"], "incomplete", "{item}");
        assert_eq!(
            item["record"]["cause"],
            json!({"kind":"cancelled","source":"requested"})
        );
        assert!(item.get("certificate").is_none());
        assert!(item.get("counterexample").is_none());
    }
    Ok(())
}

/// Trace: FR-022-AC-10
/// Authoritative provider signals only its own FSL parent after request readiness.
#[test]
fn sigterm_cancels_open_and_later_items_and_reports_exit_22() -> Result<(), ContractError> {
    cancellation("term")
}

/// Trace: FR-022-AC-10
#[test]
fn sigint_cancels_open_and_later_items_and_reports_exit_22() -> Result<(), ContractError> {
    cancellation("int")
}

/// Trace: FR-022-AC-9
#[test]
fn a_bounded_only_process_manifest_is_refused_before_any_item() -> Result<(), ContractError> {
    let (case, _) = provider_case("proved", "bounded")?;
    let output = analyze_provider(&case)?;
    assert_eq!(output.0.status.code(), Some(20), "{output:?}");
    let diagnostic = format!("{}{}", output.stdout_text()?, output.stderr_text()?);
    assert!(diagnostic.contains(BACKEND), "{diagnostic}");
    assert!(!case.directory().join("provider-request.json").exists());
    Ok(())
}

/// Trace: FR-022-AC-9
/// FSL FR-281: a well-formed assignment counts only when native replay violates the goal.
#[test]
fn native_replay_accepts_a_real_external_witness_and_rejects_a_false_one(
) -> Result<(), ContractError> {
    let (case, _) = provider_case("proved", "unbounded")?;
    let control = document(&analyze_provider(&case)?, 0, "success");
    assert_eq!(control["items"][0]["record"]["certification"], "trusted");
    // Read the parameter from FSL's actual public request, not a synthesized package/node.
    let parameter = &control["items"][0]["request"]["claim"]["node"]["path"][3];
    assert_eq!(parameter["path"][0], "value");
    assert_eq!(parameter["path"][1], "parameter");
    let config_path = case.directory().join("providers.json");
    let mut config: Value = serde_json::from_slice(&std::fs::read(config_path).unwrap())?;
    let mut arguments = config["providers"][0]["arguments"]
        .as_array()
        .unwrap()
        .clone();
    arguments[2] = json!("refuted");
    arguments.insert(0, config["providers"][0]["path"].clone());
    arguments.insert(0, json!(format!("FSL_TEST_PROVIDER_PARAMETER={parameter}")));
    // The published helper takes its assignment through an environment variable.
    // Existing OS env executes that exact helper; no custom provider or SDK is copied.
    config["providers"][0]["path"] = json!("/usr/bin/env");
    config["providers"][0]["arguments"] = json!(arguments);
    case.write("providers.json", &serde_json::to_vec(&config)?)?;
    let false_witness = document(&analyze_provider(&case)?, 22, "inconclusive");
    let rejected = &false_witness["items"][0];
    assert_eq!(rejected["record"]["result"], "inconclusive");
    assert_eq!(
        rejected["record"]["cause"],
        json!({"kind":"replay-parity","parity":{"proved":"violation","reason":"verdicts","replayed":"success"}}),
        "{rejected}"
    );
    assert!(rejected.get("counterexample").is_none());
    case.write(
        "data/workflow.fsl",
        format!("{SOURCE}\n{}", IDENTITY.replace("n + 3 > n", "n > 0")).as_bytes(),
    )?;
    let actual = document(&analyze_provider(&case)?, 10, "violation");
    let refuted = &actual["items"][0];
    assert_eq!(refuted["record"]["result"], "refuted");
    assert_eq!(refuted["record"]["backend"], BACKEND);
    assert_eq!(refuted["counterexample"]["type"], "assignments");
    assert_eq!(
        refuted["counterexample"]["values"][0]["value"],
        json!({"type":"integer","value":"0"})
    );
    assert_eq!(
        refuted["counterexample"]["values"][0]["parameter"],
        *parameter
    );
    assert_eq!(refuted["replay"]["reproduced"], true);
    Ok(())
}
