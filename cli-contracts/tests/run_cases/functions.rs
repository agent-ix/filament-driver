use super::*;

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn integer_arguments_compute_results_and_each_invocation_has_fresh_inputs(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for (capacity, reserved, expected) in [(17, 5, 12), (0, 0, 0), (2, 9, -7), (-3, -8, 5)] {
        write_json(
            &case,
            "run.json",
            &request(
                "packing.fsl",
                "Remaining",
                &[
                    (
                        "capacity",
                        json!({"kind":"integer","value":capacity.to_string()}),
                    ),
                    (
                        "reserved",
                        json!({"kind":"integer","value":reserved.to_string()}),
                    ),
                ],
            ),
        )?;
        completed(
            &run(&case, "run.json", &[])?,
            json!({"integer":expected.to_string()}),
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn boolean_false_is_a_completed_value_and_does_not_become_a_violation(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    for (n, expected) in [(0, false), (5, false), (6, true), (10, true)] {
        write_json(
            &case,
            "run.json",
            &request(
                "packing.fsl",
                "IsPacked",
                &[("n", json!({"kind":"integer","value":n.to_string()}))],
            ),
        )?;
        completed(&run(&case, "run.json", &[])?, json!({"boolean":expected}));
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn argument_names_resolve_independently_of_array_order() -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    write_json(
        &case,
        "reversed.json",
        &request(
            "packing.fsl",
            "Remaining",
            &[
                ("reserved", json!({"kind":"integer","value":"8"})),
                ("capacity", json!({"kind":"integer","value":"21"})),
            ],
        ),
    )?;
    completed(&run(&case, "reversed.json", &[])?, json!({"integer":"13"}));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn unknown_function_missing_extra_duplicate_and_wrong_typed_arguments_refuse(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let base = request(
        "packing.fsl",
        "IsPacked",
        &[("n", json!({"kind":"integer","value":"4"}))],
    );
    let mut unknown = base.clone();
    unknown["request"]["call"]["function"] = json!("Absent");
    let mut missing = base.clone();
    missing["request"]["call"]["arguments"] = json!([]);
    let mut extra = base.clone();
    extra["request"]["call"]["arguments"]
        .as_array_mut()
        .expect("args")
        .push(json!({"parameter":"other","value":{"kind":"integer","value":"2"}}));
    let mut duplicate = base.clone();
    duplicate["request"]["call"]["arguments"]
        .as_array_mut()
        .expect("args")
        .push(json!({"parameter":"n","value":{"kind":"integer","value":"5"}}));
    let mut wrong = base.clone();
    wrong["request"]["call"]["arguments"][0]["value"] = json!({"kind":"boolean","value":true});
    let mut out_of_domain = base;
    out_of_domain["request"]["call"]["arguments"][0]["value"] =
        json!({"kind":"integer","value":"11"});
    for input in [unknown, missing, extra, duplicate, wrong, out_of_domain] {
        write_json(&case, "bad.json", &input)?;
        let value = outcome(&run(&case, "bad.json", &[])?, 20, "refusal");
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

/// Trace: FR-002-AC-21
/// Provenance: QSL-2471
#[test]
fn checked_package_runs_after_its_source_is_removed_and_rejects_restated_source(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    let original = request(
        "packing.fsl",
        "Remaining",
        &[
            ("capacity", json!({"kind":"integer","value":"17"})),
            ("reserved", json!({"kind":"integer","value":"5"})),
        ],
    );
    write_json(
        &case,
        "compile.json",
        &json!({"format":"native-compile/1","request":{
        "models":[],"program":original["request"]["program"],"output":{"package":"packing.package.json"}}}),
    )?;
    let compiled = case.run(&["compile", "compile.json", "--format", "json"])?;
    assert_eq!(compiled.0.status.code(), Some(0), "{compiled:?}");
    std::fs::remove_file(case.directory().join("packing.fsl"))?;
    let mut packaged = original;
    packaged["request"]
        .as_object_mut()
        .expect("request")
        .remove("program");
    packaged["request"]["package"] = json!({"file":"packing.package.json"});
    write_json(&case, "packaged.json", &packaged)?;
    completed(&run(&case, "packaged.json", &[])?, json!({"integer":"12"}));
    packaged["request"]["program"] = json!({"source":{"file":"missing.fsl","authority":"a","identity":"i","document":"D","formal_revision":1}});
    write_json(&case, "both.json", &packaged)?;
    let value = outcome(&run(&case, "both.json", &[])?, 20, "refusal");
    assert!(
        diagnostic(&value, "invalid-request")["message"]
            .as_str()
            .expect("message")
            .contains("program"),
        "{value}"
    );
    Ok(())
}
