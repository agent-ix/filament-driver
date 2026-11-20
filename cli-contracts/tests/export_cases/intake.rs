use super::*;

/// Trace: FR-027-AC-5, FR-002-AC-1
/// Provenance: QSL-2474
#[test]
fn each_route_reports_missing_directory_empty_non_utf8_and_malformed_requests(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    case.write("empty.json", b"")?;
    case.write("directory/anchor", b"")?;
    case.write("non-utf8.json", &[0xff])?;
    case.write("malformed.json", b"{\"format\":")?;
    for evidence in [false, true] {
        for path in [
            "missing.json",
            "directory",
            "empty.json",
            "non-utf8.json",
            "malformed.json",
        ] {
            let value = outcome(&run(&case, path, evidence, &[])?, 20, "refusal");
            assert!(value["result"].is_null(), "{path}: {value}");
            assert!(value["last_stage"].is_null(), "{path}: {value}");
            assert!(
                !value["diagnostics"]
                    .as_array()
                    .expect("diagnostics")
                    .is_empty(),
                "{path}: {value}"
            );
            if path == "missing.json" || path == "directory" {
                diagnostic(&value, "io-error");
            }
        }
    }
    Ok(())
}

/// Trace: FR-027-AC-5, FR-002-AC-1
/// Provenance: QSL-2474
#[test]
fn both_routes_bound_request_bytes_before_decoding_an_oversized_document(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    case.write("oversize.json", &vec![b' '; 1_048_577])?;
    for evidence in [false, true] {
        let value = outcome(
            &run(&case, "oversize.json", evidence, &[])?,
            22,
            "incomplete",
        );
        let message = diagnostic(&value, "resource_exhausted")["message"]
            .as_str()
            .expect("message");
        assert!(
            message.contains("request_bytes") && message.contains("1048576"),
            "{value}"
        );
        assert!(value["result"].is_null());
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2474
#[test]
fn an_execute_request_is_not_silently_run_by_export() -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let mut input = request();
    input["request"]
        .as_object_mut()
        .expect("request")
        .remove("export");
    input["request"]["call"] = json!({"function":"Ready","arguments":[]});
    write_json(&case, "call.json", &input)?;
    invalid(&run(&case, "call.json", false, &[])?, &["export"]);
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2474
#[test]
fn export_envelope_and_output_records_reject_unknown_duplicate_and_wrong_kind_fields(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let base = request();
    let mut unknown = base.clone();
    unknown["request"]["export"]["surprise"] = json!(true);
    let mut wrong_output = base.clone();
    wrong_output["request"]["export"]["output"] = json!({"file":17});
    let mut extra_output = base.clone();
    extra_output["request"]["export"]["output"] = json!({"file":"never.tla","extra":true});
    let mut missing_profile = base.clone();
    missing_profile["request"]["export"]
        .as_object_mut()
        .expect("export")
        .remove("profile");
    let mut wrong_obligations = base.clone();
    wrong_obligations["request"]["export"]["obligations"] = json!("Capacity");
    let serialized = base.to_string();
    let duplicate = serialized.replacen("\"models\":[]", "\"models\":[],\"models\":[]", 1);
    assert_ne!(duplicate, serialized);
    for input in [
        unknown.to_string(),
        wrong_output.to_string(),
        extra_output.to_string(),
        missing_profile.to_string(),
        wrong_obligations.to_string(),
        duplicate,
        format!("{serialized} {{}}"),
    ] {
        case.write("closed.json", input.as_bytes())?;
        let value = outcome(&run(&case, "closed.json", false, &[])?, 20, "refusal");
        diagnostic(&value, "invalid-request");
        assert!(value["last_stage"].is_null(), "{value}");
        assert!(!case.directory().join("never.tla").exists());
    }
    Ok(())
}

/// Trace: FR-027-AC-3
/// Provenance: QSL-2474
#[test]
fn request_export_limits_are_closed_and_conflicts_do_not_pick_one_value(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for limits in [
        json!({"export.unknown":1}),
        json!({"export.nodes":-1}),
        json!({"export.output_bytes":"5"}),
    ] {
        let mut input = request();
        input["request"]["export"]["limits"] = limits;
        write_json(&case, "limits.json", &input)?;
        let value = outcome(&run(&case, "limits.json", false, &[])?, 20, "refusal");
        diagnostic(&value, "invalid-request");
        assert!(value["last_stage"].is_null());
    }
    let mut input = request();
    input["request"]["export"]["limits"] = json!({"export.nodes":8});
    write_json(&case, "limits.json", &input)?;
    invalid(
        &run(&case, "limits.json", false, &["--limit", "export.nodes=9"])?,
        &["limits", "export.nodes"],
    );
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2474
#[test]
fn named_request_is_authoritative_and_file_named_dash_is_addressable(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let mut input = request();
    input["request"]
        .as_object_mut()
        .expect("request")
        .remove("export");
    write_json(&case, "no-export.json", &input)?;
    write_json(&case, "-", &input)?;
    let stdin = serde_json::to_vec(&request())?;
    for path in ["no-export.json", "./-"] {
        invalid(
            &case.run_stdin(&["export", path, "--format", "json"], &stdin)?,
            &["export"],
        );
    }
    Ok(())
}

/// Trace: FR-027-AC-5
/// Provenance: QSL-2474
#[test]
fn evidence_requires_its_own_member_and_closed_shape_instead_of_exporting(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let missing = outcome(&run(&case, "export.json", true, &[])?, 20, "refusal");
    diagnostic(&missing, "invalid-request");
    assert!(missing["result"].is_null());
    for evidence in [
        json!({}),
        json!({"package":null,"obligation":"Capacity","tool":"tlc","artifact":"never.cfg"}),
        json!({"package":{},"obligation":"Capacity","tool":"pretend-tool","artifact":"never.cfg","extra":true}),
    ] {
        write_json(
            &case,
            "evidence.json",
            &json!({"format":"native-run/1","request":{"export_evidence":evidence}}),
        )?;
        let value = outcome(&run(&case, "evidence.json", true, &[])?, 20, "refusal");
        diagnostic(&value, "invalid-request");
        assert!(value["result"].is_null());
    }
    Ok(())
}

/// Trace: FR-002-AC-13, FR-002-AC-18
/// Provenance: QSL-2474
#[test]
fn default_and_explicit_text_preserve_the_json_refusal_on_each_route(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for evidence in [false, true] {
        let json = outcome(&run(&case, "missing.json", evidence, &[])?, 20, "refusal");
        let mut args = vec!["export"];
        if evidence {
            args.push("--evidence");
        }
        args.push("missing.json");
        let default = case.run(&args)?;
        args.extend_from_slice(&["--format", "text"]);
        let explicit = case.run(&args)?;
        assert_eq!(default.0.status.code(), Some(20));
        assert_eq!(explicit.0.status.code(), Some(20));
        assert_eq!(default.0.stdout, explicit.0.stdout);
        let text = default.stdout_text()?;
        assert!(text.starts_with("export: refusal (exit 20)"), "{text}");
        for d in json["diagnostics"].as_array().expect("diagnostics") {
            assert!(text.contains(d["code"].as_str().expect("code")), "{text}");
        }
    }
    Ok(())
}
