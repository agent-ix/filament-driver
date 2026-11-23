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
                        "total",
                        json!({"type":"integer","value":capacity.to_string()}),
                    ),
                    (
                        "held",
                        json!({"type":"integer","value":reserved.to_string()}),
                    ),
                ],
            ),
        )?;
        completed(
            &run(&case, "run.json", &[])?,
            json!({"type":"integer","value":expected.to_string()}),
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
                &[("n", json!({"type":"integer","value":n.to_string()}))],
            ),
        )?;
        completed(
            &run(&case, "run.json", &[])?,
            json!({"type":"boolean","value":expected}),
        );
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
                ("held", json!({"type":"integer","value":"8"})),
                ("total", json!({"type":"integer","value":"21"})),
            ],
        ),
    )?;
    completed(
        &run(&case, "reversed.json", &[])?,
        json!({"type":"integer","value":"13"}),
    );
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
        &[("n", json!({"type":"integer","value":"4"}))],
    );
    let mut unknown = base.clone();
    unknown["request"]["call"]["function"] = json!("Absent");
    let mut missing = base.clone();
    missing["request"]["call"]["arguments"] = json!([]);
    let mut extra = base.clone();
    extra["request"]["call"]["arguments"]
        .as_array_mut()
        .expect("args")
        .push(json!({"parameter":"other","value":{"type":"integer","value":"2"}}));
    let mut duplicate = base.clone();
    duplicate["request"]["call"]["arguments"]
        .as_array_mut()
        .expect("args")
        .push(json!({"parameter":"n","value":{"type":"integer","value":"5"}}));
    let mut wrong = base.clone();
    wrong["request"]["call"]["arguments"][0]["value"] = json!({"type":"boolean","value":true});
    let mut untyped = base.clone();
    untyped["request"]["call"]["arguments"][0]["value"] = json!({"integer":"4"});
    {
        // FR-100-AC-4, FR-650: a value without the `type` form is of another kind.
        write_json(&case, "bad.json", &untyped)?;
        let value = outcome(&run(&case, "bad.json", &[])?, 20, "refusal");
        assert_eq!(value["last_stage"], "S6a", "{value}");
        assert_eq!(
            value["diagnostics"][0]["code"], "invalid_runtime_input",
            "{value}"
        );
        assert_eq!(
            value["diagnostics"][0]["cause"], "wrong-value-kind",
            "{value}"
        );
    }
    let mut out_of_domain = base;
    out_of_domain["request"]["call"]["arguments"][0]["value"] =
        json!({"type":"integer","value":"11"});
    for input in [unknown, missing, extra, duplicate, wrong, out_of_domain] {
        write_json(&case, "bad.json", &input)?;
        let value = outcome(&run(&case, "bad.json", &[])?, 20, "refusal");
        assert!(value["result"].is_null(), "{value}");
        assert!(
            value["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .iter()
                .all(|d| d["code"] != "invalid_syntax"),
            "argument validation must follow successful source parsing: {value}"
        );
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
            ("total", json!({"type":"integer","value":"17"})),
            ("held", json!({"type":"integer","value":"5"})),
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
    completed(
        &run(&case, "packaged.json", &[])?,
        json!({"type":"integer","value":"12"}),
    );
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

/// Trace: FR-002-AC-1
/// Provenance: QSL-2471
#[test]
fn non_finite_conversion_is_undefined_ten_with_its_actual_reason(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = fixture()?;
    case.write(
        "fraction.fsl",
        br#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
type Fraction = Rational[-4, 4; 1, 4];
function FractionOf using v(x: Float64[nearest-even]): Fraction pure { convert<Fraction>(x) }
"#,
    )?;
    for (bits, exit) in [("3ff0000000000000", 0), ("7ff8000000000000", 10)] {
        write_json(
            &case,
            "fraction.json",
            &request(
                "fraction.fsl",
                "FractionOf",
                &[("x", json!({"type":"float64","bits":bits}))],
            ),
        )?;
        let output = run(&case, "fraction.json", &[])?;
        if exit == 0 {
            completed(
                &output,
                json!({"type":"rational","numerator":"1","denominator":"1"}),
            );
        } else {
            let value = outcome(&output, 10, "undefined");
            assert_eq!(
                value["result"],
                json!({"kind":"undefined","reason":"ieee-not-finite"}),
                "{value}"
            );
        }
    }
    Ok(())
}
