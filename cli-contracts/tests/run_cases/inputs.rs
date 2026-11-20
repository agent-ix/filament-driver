use super::*;

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn request_relative_absolute_and_parent_paths_keep_the_request_base(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    case.write("requests/nested/anchor", b"")?;
    for path in [
        "../../packing.fsl".to_owned(),
        case.directory()
            .join("packing.fsl")
            .to_str()
            .expect("UTF8")
            .to_owned(),
    ] {
        write_json(
            &case,
            "requests/nested/run.json",
            &request(
                &path,
                "Remaining",
                &[
                    ("total", json!({"integer":"20"})),
                    ("held", json!({"integer":"7"})),
                ],
            ),
        )?;
        completed(
            &run(&case, "requests/nested/run.json", &[])?,
            json!({"integer":"13"}),
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn named_request_ignores_stdin_and_dash_is_an_ordinary_path(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    completed(
        &case.run_stdin(&["run", "run.json", "--format", "json"], b"bad JSON\0")?,
        json!({"integer":"12"}),
    );
    let bytes = std::fs::read(case.directory().join("run.json"))?;
    case.write("-", &bytes)?;
    completed(
        &case.run_stdin(&["run", "./-", "--format", "json"], b"not the request")?,
        json!({"integer":"12"}),
    );
    Ok(())
}

/// Trace: FR-002-AC-18
/// Provenance: QSL-2471
#[test]
fn missing_directory_empty_malformed_and_non_utf8_requests_are_typed_refusals(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    case.write("empty.json", b"")?;
    case.write("broken.json", b"{\"format\":")?;
    case.write("non-utf8.json", &[0xff])?;
    case.write("directory/anchor", b"")?;
    for path in [
        "missing.json",
        "directory",
        "empty.json",
        "broken.json",
        "non-utf8.json",
    ] {
        let value = outcome(&run(&case, path, &[])?, 20, "refusal");
        assert!(value["result"].is_null(), "{value}");
        assert!(
            !value["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .is_empty(),
            "{path}: {value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn closed_request_envelopes_reject_unknown_duplicate_and_wrong_kind_members(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let valid = std::fs::read_to_string(case.directory().join("run.json"))?;
    let parsed: Value = serde_json::from_str(&valid)?;
    let mut unknown = parsed.clone();
    unknown["request"]["surprise"] = json!(true);
    let mut wrong_format = parsed.clone();
    wrong_format["format"] = json!("native-run/99");
    let mut missing_label = parsed.clone();
    missing_label["request"]["program"]["source"]
        .as_object_mut()
        .expect("source")
        .remove("authority");
    let duplicate = valid.replacen("\"models\":[]", "\"models\":[],\"models\":[]", 1);
    assert_ne!(duplicate, valid, "duplicate-key fixture must differ");
    for bytes in [
        unknown.to_string(),
        wrong_format.to_string(),
        missing_label.to_string(),
        duplicate,
        "[]".to_owned(),
        "null".to_owned(),
        format!("{valid} {{}}"),
    ] {
        case.write("closed.json", bytes.as_bytes())?;
        let value = outcome(&run(&case, "closed.json", &[])?, 20, "refusal");
        assert!(value["last_stage"].is_null(), "{value}");
        assert!(value["result"].is_null(), "{value}");
        assert!(
            !value["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .is_empty(),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn exact_request_and_dependency_budgets_are_inclusive_and_reset(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let bytes = std::fs::read(case.directory().join("run.json"))?;
    let request_len = bytes.len().to_string();
    let source_len = SOURCE.len().to_string();
    completed(
        &run(
            &case,
            "run.json",
            &[
                "--request-bytes",
                &request_len,
                "--dependent-bytes",
                &source_len,
            ],
        )?,
        json!({"integer":"12"}),
    );
    for (flag, field, size) in [
        ("--request-bytes", "request_bytes", bytes.len()),
        ("--dependent-bytes", "dependent_bytes", SOURCE.len()),
    ] {
        let lower = (size - 1).to_string();
        let value = outcome(&run(&case, "run.json", &[flag, &lower])?, 22, "incomplete");
        let d = diagnostic(&value, "resource_exhausted");
        let message = d["message"].as_str().expect("message");
        assert!(
            message.contains(field) && message.contains(&lower),
            "{value}"
        );
        assert!(value["result"].is_null(), "{value}");
    }
    completed(&run(&case, "run.json", &[])?, json!({"integer":"12"}));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn default_request_ceiling_reports_oversize_before_json_parse(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    case.write("oversize.json", &vec![b' '; 1_048_577])?;
    let value = outcome(&run(&case, "oversize.json", &[])?, 22, "incomplete");
    let d = diagnostic(&value, "resource_exhausted");
    let message = d["message"].as_str().expect("message");
    assert!(
        message.contains("request_bytes") && message.contains("1048576"),
        "{value}"
    );
    Ok(())
}
