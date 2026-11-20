use super::*;

/// Trace: FR-022-AC-1; FR-022-AC-2
/// FSL FR-850: selected packages are admitted without source compilation.
#[test]
fn a_public_compile_handoff_is_analyzed_without_reopening_source() -> Result<(), ContractError> {
    let (case, mut request) = setup()?;
    let compile = json!({"format":"native-compile/1","request":{
        "program":request["request"]["program"],"models":request["request"]["models"],
        "handoff":{"directory":"selected","package_name":"queue-workflow","native_join":{"work_units":1_000_000,"storage":1_000_000}}
    }});
    case.write("data/compile.json", &serde_json::to_vec(&compile)?)?;
    let output = case.run(&["compile", "data/compile.json", "--format", "json"])?;
    assert_eq!(output.0.status.code(), Some(0), "{output:?}");
    let compiled = output.json()?;
    assert_eq!(compiled["category"], "success", "{compiled}");
    assert!(case
        .directory()
        .join("data/selected/checked-package.json")
        .is_file());
    assert!(case
        .directory()
        .join("data/selected/identities.json")
        .is_file());
    request["request"]
        .as_object_mut()
        .unwrap()
        .remove("program");
    request["request"]["package"] = json!({"directory":"selected"});
    write_request(&case, &request)?;
    case.write(
        "data/workflow.fsl",
        b"this is deliberately not valid source",
    )?;
    let value = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        10,
        "violation",
    );
    assert_eq!(value["items"][0]["claim_name"], "WithinCapacity");
    assert_eq!(value["items"][0]["record"]["result"], "proved");
    assert_eq!(value["items"][0]["certificate"]["accepted"], true);
    assert_eq!(value["items"][1]["claim_name"], "NeverFull");
    assert_eq!(value["items"][1]["record"]["result"], "refuted");
    assert_eq!(value["items"][1]["replay"]["reproduced"], true);
    assert_eq!(value["items"][1]["counterexample"]["trace_position"], 2);
    // Mutate only the public artifact written for this case, never reconstruct it.
    case.write("data/selected/identities.json", b"{}")?;
    let rejected = document(
        &case.run(&["analyze", "data/request.json", "--format", "json"])?,
        20,
        "refusal",
    );
    assert!(rejected["items"].as_array().unwrap().is_empty());
    assert_eq!(rejected["diagnostics"][0]["code"], "invalid_package");
    assert!(
        rejected["diagnostics"][0]
            .to_string()
            .contains("identities.json"),
        "{rejected}"
    );
    Ok(())
}
