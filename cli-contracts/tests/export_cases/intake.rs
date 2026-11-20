// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// Trace: FR-002-AC-1, FSL-FR-1015-AC-9
/// Provenance: QSL-2474
#[test]
fn export_envelope_and_output_records_reject_unknown_and_invalid_fields(
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
    let mut empty_obligations = base.clone();
    empty_obligations["request"]["export"]["obligations"] = json!([]);
    let mut missing_obligations = base.clone();
    missing_obligations["request"]["export"]
        .as_object_mut()
        .expect("export")
        .remove("obligations");
    let mut wrong_fidelity = base.clone();
    wrong_fidelity["request"]["export"]["minimum_fidelity"] = json!(7);
    let serialized = base.to_string();
    for (input, member) in [
        (
            unknown.to_string(),
            Some(("/request/export/surprise", "unknown-member")),
        ),
        (
            wrong_output.to_string(),
            Some(("/request/export/output/file", "invalid-value")),
        ),
        (
            extra_output.to_string(),
            Some(("/request/export/output/extra", "unknown-member")),
        ),
        (
            missing_profile.to_string(),
            Some(("/request/export/profile", "missing-member")),
        ),
        (
            empty_obligations.to_string(),
            Some(("/request/export/obligations", "invalid-value")),
        ),
        (
            missing_obligations.to_string(),
            Some(("/request/export/obligations", "missing-member")),
        ),
        (
            wrong_fidelity.to_string(),
            Some(("/request/export/minimum_fidelity", "wrong-value-kind")),
        ),
        (format!("{serialized} {{}}"), None),
    ] {
        case.write("closed.json", input.as_bytes())?;
        let output = run(&case, "closed.json", false, &[])?;
        assert_eq!(
            output.0.status.code(),
            Some(20),
            "input {input}: {output:?}"
        );
        let value = outcome(&output, 20, "refusal");
        match member {
            Some((path, cause)) => {
                let d = diagnostic(&value, "invalid_runtime_input");
                assert_eq!(d["cause"], cause, "{value}");
                assert_eq!(d["details"]["path"], path, "{value}");
            }
            None => {
                diagnostic(&value, "invalid-request");
            }
        }
        assert!(value["last_stage"].is_null(), "{value}");
        assert!(!case.directory().join("never.tla").exists());
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2474
#[test]
#[ignore = "QSL-2669: export accepts duplicate models in the native-run envelope"]
fn duplicate_envelope_members_refuse_before_export_and_write_no_target(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let original = request().to_string();
    let duplicated = original.replacen("\"models\":[]", "\"models\":[],\"models\":[]", 1);
    assert_ne!(duplicated, original);
    case.write("duplicate.json", duplicated.as_bytes())?;
    let value = outcome(
        &run(
            &case,
            "duplicate.json",
            false,
            &["--output", "must-not-exist.tla"],
        )?,
        20,
        "refusal",
    );
    let d = diagnostic(&value, "invalid-request");
    assert!(
        d["message"].as_str().expect("message").contains("models"),
        "{value}"
    );
    assert!(value["last_stage"].is_null(), "{value}");
    assert!(value["result"].is_null(), "{value}");
    assert!(!case.directory().join("must-not-exist.tla").exists());
    Ok(())
}

/// Trace: FR-002-AC-1, FSL-FR-1015-AC-6
/// Provenance: QSL-2474
/// FSL FR-1015: undeclared obligations give refused records in request order.
#[test]
fn undeclared_obligations_produce_refused_export_records_and_no_target(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let mut input = request();
    input["request"]["export"]["obligations"] = json!(["AbsentOne", "AbsentTwo"]);
    write_json(&case, "absent.json", &input)?;
    let value = outcome(
        &run(&case, "absent.json", false, &["--output", "absent.tla"])?,
        21,
        "unsupported",
    );
    assert_eq!(value["last_stage"], "S4", "{value}");
    assert_eq!(value["result"]["kind"], "exported", "{value}");
    let export = &value["result"]["export"];
    assert_eq!(export["status"], "refused", "{value}");
    assert_eq!(
        export["refusal"]["code"], "unsupported_projection",
        "{value}"
    );
    assert_eq!(
        export["refusal"]["cause"], "below-fidelity-floor",
        "{value}"
    );
    assert!(export.get("target_text").is_none(), "{value}");
    let records = export["records"]
        .as_array()
        .expect("refused obligation records");
    assert_eq!(records.len(), 2, "{value}");
    for (record, name) in records.iter().zip(["AbsentOne", "AbsentTwo"]) {
        assert_eq!(record["obligation"], name, "{value}");
        assert_eq!(record["disposition"], "refused", "{value}");
        assert_eq!(
            record["causes"],
            json!([{"kind":"not-an-obligation","name":name}]),
            "{value}"
        );
    }
    assert!(!case.directory().join("absent.tla").exists());
    Ok(())
}

/// Trace: FR-002-AC-1, FSL-FR-1015-AC-2
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
#[ignore = "QSL-2667: export discards evidence mode before dispatch and requires export member"]
fn evidence_mode_rejects_an_ordinary_export_request() -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let missing = outcome(&run(&case, "export.json", true, &[])?, 20, "refusal");
    let d = diagnostic(&missing, "invalid-request");
    assert!(
        d["message"]
            .as_str()
            .expect("message")
            .contains("export_evidence"),
        "{missing}"
    );
    assert!(missing["result"].is_null());
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

/// Trace: FR-002-AC-1
/// Provenance: QSL-2474
/// FSL FR-026 requires a request to be read once before dispatch.
#[cfg(unix)]
#[test]
#[ignore = "QSL-2670: export preflight consumes the request stream before FSL reads it"]
fn explicit_stdin_file_is_read_once_and_keeps_the_absolute_program_selection(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let mut input = request();
    input["request"]["program"]["source"]["file"] = json!(case
        .directory()
        .join("unit.fsl")
        .to_str()
        .expect("UTF8 path"));
    let bytes = serde_json::to_vec(&input)?;
    let value = outcome(
        &case.run_stdin(&["export", "/dev/stdin", "--format", "json"], &bytes)?,
        21,
        "unsupported",
    );
    assert_eq!(value["last_stage"], "S4", "{value}");
    assert_eq!(value["result"]["kind"], "exported", "{value}");
    assert_eq!(
        value["result"]["export"]["records"][0]["causes"],
        json!([{"kind":"not-an-obligation","name":"Capacity"}]),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-1, FSL-FR-1039-AC-4
/// Provenance: QSL-2474
/// FSL FR-1039: source_package is checked before selected-file I/O.
#[test]
#[ignore = "QSL-2672: export rejects package before required source_package admission"]
fn package_source_reference_is_closed_and_typed_before_opening_the_package(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for (reference, cause, suffix) in [
        (None, "missing-member", ""),
        (Some(json!(null)), "wrong-value-kind", ""),
        (Some(json!(7)), "wrong-value-kind", ""),
        (Some(json!([])), "wrong-value-kind", ""),
        (Some(json!({})), "missing-member", "/identity"),
        (
            Some(json!({"identity":"contracts/bin"})),
            "missing-member",
            "/version",
        ),
        (Some(json!({"version":"1"})), "missing-member", "/identity"),
        (
            Some(json!({"identity":7,"version":"1"})),
            "wrong-value-kind",
            "/identity",
        ),
        (
            Some(json!({"identity":"contracts/bin","version":7})),
            "wrong-value-kind",
            "/version",
        ),
        (
            Some(json!({"identity":"","version":"1"})),
            "invalid-value",
            "/identity",
        ),
        (
            Some(json!({"identity":"contracts/bin","version":""})),
            "invalid-value",
            "/version",
        ),
        (
            Some(json!({"identity":"contracts/bin","version":"1","extra":true})),
            "unknown-member",
            "/extra",
        ),
    ] {
        let mut input = request();
        let inner = input["request"].as_object_mut().expect("request");
        inner.remove("program");
        inner.insert(
            "package".to_owned(),
            json!({"file":"selected-package-must-not-open.json"}),
        );
        if let Some(reference) = reference {
            input["request"]["package"]["source_package"] = reference;
        }
        input["request"]["export"]["output"] = json!({"file":"must-not-write.tla"});
        write_json(&case, "bad-reference.json", &input)?;
        let output = run(&case, "bad-reference.json", false, &[])?;
        assert_eq!(output.0.status.code(), Some(20), "{input}: {output:?}");
        let value = outcome(&output, 20, "refusal");
        let d = diagnostic(&value, "invalid_runtime_input");
        assert_eq!(d["cause"], cause, "{input}: {value}");
        assert_eq!(
            d["details"]["path"],
            format!("/request/package/source_package{suffix}"),
            "{input}: {value}"
        );
        assert_eq!(
            value["diagnostics"].as_array().expect("diagnostics").len(),
            1,
            "{value}"
        );
        assert!(value["last_stage"].is_null(), "{value}");
        assert!(value["result"].is_null(), "{value}");
        assert!(!case.directory().join("must-not-write.tla").exists());
    }
    Ok(())
}
