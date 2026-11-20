use super::*;

const STAMPED: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
profile t = "filament.temporal.timestamped-event.finite-window/v1";
record Pressure { level: Int[0, 10]; }
temporal SafePressure using t over (p: Pressure) clock "pump-ms" on origin {
    always[0,2] holds(p.level <= 6)
}
function Constant using v(): Integer pure { 19 }
"#;
fn clocked(case: &CliCase) -> Result<Value, ContractError> {
    case.write("pressure.fsl", STAMPED.as_bytes())?;
    Ok(request("pressure.fsl", "Constant", &[]))
}
fn clocks() -> Value {
    json!({"pump-ms":{"kind":"timestamped_event","timestamp_unit":"ms"}})
}

/// Trace: FR-023-AC-1, FR-023-AC-5
/// Provenance: QSL-2471
#[test]
fn external_clock_configuration_matches_inline_and_rejects_both(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let mut input = clocked(&case)?;
    write_json(&case, "clocks.json", &clocks())?;
    write_json(&case, "external.json", &input)?;
    let external = completed(
        &run(&case, "external.json", &["--clocks", "clocks.json"])?,
        json!({"integer":"19"}),
    );
    input["request"]["clocks"] = clocks();
    write_json(&case, "inline.json", &input)?;
    let inline = completed(&run(&case, "inline.json", &[])?, json!({"integer":"19"}));
    assert_eq!(external, inline);
    let refused = outcome(
        &run(&case, "inline.json", &["--clocks", "clocks.json"])?,
        20,
        "refusal",
    );
    assert!(refused["last_stage"].is_null());
    let message = diagnostic(&refused, "invalid-request")["message"]
        .as_str()
        .expect("message");
    assert!(
        message.contains("clocks") && message.contains("--clocks"),
        "{refused}"
    );
    Ok(())
}

/// Trace: FR-023-AC-3, FR-023-AC-4, FR-023-AC-7
/// Provenance: QSL-2471
#[test]
fn clock_file_intake_refuses_missing_non_objects_non_utf8_and_oversize(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let missing = outcome(
        &run(&case, "run.json", &["--clocks", "missing-clocks.json"])?,
        20,
        "refusal",
    );
    assert_eq!(
        diagnostic(&missing, "io-error")["cause"],
        "clocks-file-unreadable"
    );
    for bytes in [b"[]".as_slice(), b"null", b"not json", b"{} {}", &[0xff]] {
        case.write("bad-clocks.json", bytes)?;
        let value = outcome(
            &run(&case, "run.json", &["--clocks", "bad-clocks.json"])?,
            20,
            "refusal",
        );
        assert_eq!(
            diagnostic(&value, "invalid-request")["cause"],
            "clocks-file-not-an-object"
        );
        assert!(value["last_stage"].is_null());
    }
    case.write("large-clocks.json", &vec![b' '; 2049])?;
    let value = outcome(
        &run(
            &case,
            "run.json",
            &["--clocks", "large-clocks.json", "--request-bytes", "2048"],
        )?,
        22,
        "incomplete",
    );
    assert_eq!(
        diagnostic(&value, "resource_exhausted")["cause"],
        "clocks-file-too-large"
    );
    assert!(value["last_stage"].is_null());
    Ok(())
}

/// Trace: FR-002-AC-20
/// Provenance: QSL-2471
#[test]
fn tl_lowering_preserves_the_requested_correspondence_and_actual_formula(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let mut input = clocked(&case)?;
    input["request"]
        .as_object_mut()
        .expect("request")
        .remove("call");
    input["request"]["clocks"] = clocks();
    input["request"]["tl_lowering"] =
        json!({"clause":"SafePressure","correspondence":"mltl.closed-trace/v1"});
    write_json(&case, "lower.json", &input)?;
    let value = outcome(&run(&case, "lower.json", &[])?, 0, "success");
    let lowering = &value["result"]["tl_lowering"];
    assert_eq!(lowering["disposition"], "supported", "{value}");
    assert_eq!(
        lowering["correspondence"], "mltl.closed-trace/v1",
        "{value}"
    );
    assert_eq!(lowering["formula"], "G[0,2] p0", "{value}");
    Ok(())
}
