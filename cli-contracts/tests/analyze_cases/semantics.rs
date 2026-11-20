use super::*;

/// Trace: FR-022-AC-1; FR-022-AC-2; FR-022-AC-8
/// FSL FR-621: proof and counterexample semantics, not agent assertions.
#[test]
fn a_reachable_full_queue_refutes_never_full_but_proves_capacity() -> Result<(), ContractError> {
    let (case, _) = setup()?;
    let output = case.run(&["analyze", "data/request.json", "--format", "json"])?;
    let value = document(&output, 10, "violation");
    eprintln!("{value}");
    assert_eq!(value["aggregate"]["status"], "complete");
    assert_eq!(value["items"].as_array().unwrap().len(), 2);
    assert_eq!(value["items"][0]["record"]["result"], "proved");
    assert_eq!(value["items"][1]["record"]["result"], "refuted");
    Ok(())
}
