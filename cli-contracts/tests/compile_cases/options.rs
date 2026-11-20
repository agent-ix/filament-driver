// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compile option interactions and additional native request forms.

use super::*;

/// Trace: FR-024-AC-1
/// Provenance: QSL-2470
#[test]
fn compile_flag_order_and_distinct_repeated_limits_preserve_the_program() -> TestResult {
    for args in [
        vec![
            "compile",
            "--format",
            "json",
            "--output",
            "combined.json",
            "--clocks",
            "clocks.json",
            "--limit",
            "s1.nodes=10000",
            "--limit",
            "s3.nodes=10000",
            "inputs/compile.json",
        ],
        vec![
            "compile",
            "--limit",
            "s1.nodes=10000",
            "inputs/compile.json",
            "--clocks",
            "clocks.json",
            "--output",
            "combined.json",
            "--format",
            "json",
        ],
        vec![
            "compile",
            "inputs/compile.json",
            "--output",
            "combined.json",
            "--format",
            "json",
            "--clocks",
            "clocks.json",
            "--limit",
            "s3.nodes=10000",
        ],
    ] {
        let case = parcel()?;
        case.write("clocks.json", b"{}")?;
        success(&case.run(&args)?)?;
        package(&case, "combined.json", &["Total", "Parcel"])?;
    }
    Ok(())
}

/// Trace: FR-023-AC-2
/// Provenance: QSL-2470; FSL FR-640
#[test]
fn timestamped_clock_file_and_request_member_compile_the_same_temporal_binding() -> TestResult {
    let case = parcel()?;
    case.write(
        "inputs/receipt.fsl",
        br#"language "ix:native" edition "1-draft";
profile ts = "filament.temporal.timestamped-event.finite-window/v1";
record Receipt { cleared: Boolean; }
temporal Clears using ts over (s: Receipt) clock "gate-ms" on each (pulse: Receipt) {
    eventually[0,12] holds(s.cleared)
}
"#,
    )?;
    let mut input = request(source("receipt.fsl", "receipt"));
    write_request(&case, "inputs/temporal.json", &input)?;
    let absent = refusal(
        &case.run(&["compile", "inputs/temporal.json", "--format", "json"])?,
        20,
        "invalid-request",
    )?;
    assert!(absent.to_string().contains("missing-clock-selection"));
    let clocks = json!({"gate-ms":{"kind":"timestamped_event","timestamp_unit":"ms"}});
    case.write("clock.json", &serde_json::to_vec(&clocks)?)?;
    let from_file = success(&case.run(&[
        "compile",
        "inputs/temporal.json",
        "--clocks",
        "clock.json",
        "--output",
        "clocked.package.json",
        "--format",
        "json",
    ])?)?;
    input["request"]["clocks"] = clocks;
    write_request(&case, "inputs/member-clock.json", &input)?;
    let from_member =
        success(&case.run(&["compile", "inputs/member-clock.json", "--format", "json"])?)?;
    assert_eq!(from_file, from_member);
    let artifact: Value =
        serde_json::from_slice(&fs::read(case.directory().join("clocked.package.json"))?)?;
    let bindings = artifact["temporal_bindings"]
        .as_array()
        .expect("compiled temporal bindings");
    assert!(
        bindings
            .iter()
            .any(|binding| binding["clock_name"] == "gate-ms"
                && binding["clock"]["kind"] == "timestamped_event"
                && binding["clock"]["timestamp_unit"] == "ms"),
        "{bindings:?}"
    );
    Ok(())
}

/// Trace: FR-023-AC-5
/// Provenance: QSL-2470
#[test]
fn a_clock_member_and_override_conflict_before_reading_the_source() -> TestResult {
    let case = parcel()?;
    case.write("clock.json", b"{}")?;
    let mut input = request(source("not-present.fsl", "parcel"));
    input["request"]["clocks"] = json!({});
    write_request(&case, "inputs/clock-conflict.json", &input)?;
    let doc = refusal(
        &case.run(&[
            "compile",
            "inputs/clock-conflict.json",
            "--clocks",
            "clock.json",
            "--format",
            "json",
        ])?,
        20,
        "invalid-request",
    )?;
    assert!(doc["last_stage"].is_null());
    assert!(doc.to_string().contains("--clocks") && doc.to_string().contains("clocks"));
    assert!(!doc.to_string().contains("io-error"));
    Ok(())
}

/// Trace: FR-023-AC-6
/// Provenance: QSL-2470
#[test]
fn an_invalid_clock_kind_is_refused_equally_from_file_and_member() -> TestResult {
    let case = parcel()?;
    let bad = json!({"unused":{"kind":"not-a-clock"}});
    case.write("clock.json", &serde_json::to_vec(&bad)?)?;
    let file = refusal(
        &case.run(&[
            "compile",
            "inputs/compile.json",
            "--clocks",
            "clock.json",
            "--format",
            "json",
        ])?,
        20,
        "invalid-request",
    )?;
    let mut input = request(source("parcel.fsl", "parcel"));
    input["request"]["clocks"] = bad;
    write_request(&case, "inputs/clock-member.json", &input)?;
    let member = refusal(
        &case.run(&["compile", "inputs/clock-member.json", "--format", "json"])?,
        20,
        "invalid-request",
    )?;
    assert_eq!(file["category"], member["category"]);
    assert_eq!(file["last_stage"], member["last_stage"]);
    assert_eq!(
        file["diagnostics"][0]["code"],
        member["diagnostics"][0]["code"]
    );
    assert_eq!(
        file["diagnostics"][0]["cause"],
        member["diagnostics"][0]["cause"]
    );
    assert!(file["last_stage"].is_null());
    for document in [file, member] {
        let message = document["diagnostics"][0]["message"]
            .as_str()
            .expect("refusal message");
        assert!(message.contains("not-a-clock") && message.contains("unused"));
    }
    Ok(())
}

/// Trace: FR-024-AC-2
/// Provenance: QSL-2470; FSL FR-810-AC-2
#[test]
fn output_directory_and_missing_parent_refuse_without_partial_files() -> TestResult {
    let case = parcel()?;
    case.write("existing/keep", b"preserve directory")?;
    let existing = refusal(
        &case.run(&[
            "compile",
            "inputs/compile.json",
            "--output",
            "existing",
            "--format",
            "json",
        ])?,
        20,
        "invalid-request",
    )?;
    assert!(existing["last_stage"].is_null());
    assert_eq!(
        fs::read(case.directory().join("existing/keep"))?,
        b"preserve directory"
    );
    let missing = refusal(
        &case.run(&[
            "compile",
            "inputs/compile.json",
            "--output",
            "absent/package.json",
            "--format",
            "json",
        ])?,
        20,
        "io-error",
    )?;
    assert!(missing.to_string().contains("absent/package.json"));
    assert!(!case.directory().join("absent").exists());
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-7
#[test]
#[ignore = "QSL-2658: compile must preserve the typed unknown_language refusal"]
fn unknown_language_preserves_its_documented_refusal_code() -> TestResult {
    let case = parcel()?;
    case.write(
        "inputs/parcel.fsl",
        SOURCE
            .replace("ix:native", "ix:another-language")
            .as_bytes(),
    )?;
    let doc = refusal(
        &case.run(&["compile", "inputs/compile.json", "--format", "json"])?,
        20,
        "unknown_language",
    )?;
    assert!(doc.to_string().contains("ix:another-language"));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-7
#[test]
fn unknown_edition_preserves_its_documented_refusal_code() -> TestResult {
    let case = parcel()?;
    case.write(
        "inputs/parcel.fsl",
        SOURCE.replace("1-draft", "7-draft").as_bytes(),
    )?;
    let doc = refusal(
        &case.run(&["compile", "inputs/compile.json", "--format", "json"])?,
        20,
        "unknown_edition",
    )?;
    assert!(doc.to_string().contains("7-draft"));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-001 source admission
#[test]
fn bom_and_nul_are_refused_instead_of_normalizing_the_source() -> TestResult {
    let case = parcel()?;
    for bytes in [
        [b"\xef\xbb\xbf".as_slice(), SOURCE.as_bytes()].concat(),
        [SOURCE.as_bytes(), b"\0"].concat(),
    ] {
        case.write("inputs/parcel.fsl", &bytes)?;
        let doc = refusal(
            &case.run(&["compile", "inputs/compile.json", "--format", "json"])?,
            20,
            "invalid_syntax",
        )?;
        assert!(doc.to_string().contains("parcel"));
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-2
#[test]
fn another_wire_format_and_an_empty_program_are_not_compile_successes() -> TestResult {
    let case = parcel()?;
    let mut wrong = request(source("absent.fsl", "parcel"));
    wrong["format"] = json!("native-run/1");
    write_request(&case, "inputs/wrong-wire.json", &wrong)?;
    let wire = refusal(
        &case.run(&["compile", "inputs/wrong-wire.json", "--format", "json"])?,
        20,
        "unknown_wire",
    )?;
    assert!(wire["last_stage"].is_null());
    write_request(&case, "inputs/empty-program.json", &request(json!([])))?;
    let empty = refusal(
        &case.run(&["compile", "inputs/empty-program.json", "--format", "json"])?,
        20,
        "invalid_package",
    )?;
    assert!(empty["last_stage"].is_null());
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-026 closed request admission
#[test]
fn duplicate_request_fields_and_program_clause_bindings_are_rejected() -> TestResult {
    let case = parcel()?;
    let program = source("absent.fsl", "parcel");
    let duplicate = format!("{{\"format\":\"native-compile/1\",\"request\":{{\"models\":[],\"models\":[],\"program\":{program}}}}}");
    case.write("inputs/duplicate.json", duplicate.as_bytes())?;
    let doc = refusal(
        &case.run(&["compile", "inputs/duplicate.json", "--format", "json"])?,
        20,
        "invalid-request",
    )?;
    assert!(doc.to_string().contains("duplicate") && doc.to_string().contains("models"));
    let mut clauses = request(program);
    clauses["request"]["program"]["clauses"] = json!([]);
    write_request(&case, "inputs/clauses.json", &clauses)?;
    let doc = refusal(
        &case.run(&["compile", "inputs/clauses.json", "--format", "json"])?,
        20,
        "invalid-request",
    )?;
    assert!(doc.to_string().contains("clauses"));
    assert!(doc["last_stage"].is_null());
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-001 blank-label admission
#[test]
fn blank_source_labels_report_the_exact_invalid_identity_field() -> TestResult {
    let case = parcel()?;
    for (field, value) in [("authority", "  "), ("identity", "")] {
        let mut input = request(source("parcel.fsl", "parcel"));
        input["request"]["program"]["source"][field] = json!(value);
        write_request(&case, "inputs/blank.json", &input)?;
        let doc = refusal(
            &case.run(&["compile", "inputs/blank.json", "--format", "json"])?,
            20,
            "invalid_source_identity",
        )?;
        assert!(doc.to_string().contains("blank-label"));
        assert!(doc.to_string().contains(field));
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-099 compile library selection
#[test]
fn a_supplied_import_is_compiled_into_the_dependency_closure() -> TestResult {
    let case = parcel()?;
    let importing = SOURCE.replace("record Parcel", "import \"rate-lib\";\nrecord Parcel");
    case.write("inputs/parcel.fsl", importing.as_bytes())?;
    let missing = refusal(
        &case.run(&["compile", "inputs/compile.json", "--format", "json"])?,
        20,
        "missing_import",
    )?;
    assert!(missing.to_string().contains("rate-lib"));
    case.write("inputs/rates.fsl", b"language \"ix:native\" edition \"1-draft\";\nprofile r = \"filament.value.complete/v1\";\nfunction Surcharge using r(): Integer pure { 3 }\n")?;
    let mut input = request(source("parcel.fsl", "parcel"));
    input["request"]["libraries"] =
        json!([{"identity":"rate-lib","source":source("rates.fsl","rates")["source"]}]);
    write_request(&case, "inputs/library.json", &input)?;
    success(&case.run(&[
        "compile",
        "inputs/library.json",
        "--output",
        "closure.json",
        "--format",
        "json",
    ])?)?;
    package(&case, "closure.json", &["Total", "Parcel"])?;
    let artifact: Value =
        serde_json::from_slice(&fs::read(case.directory().join("closure.json"))?)?;
    assert!(artifact["lock"]["dependency_selections"]
        .to_string()
        .contains("rate-lib"));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; explicit Unix stdin file operand
#[cfg(unix)]
#[test]
fn explicit_stdin_file_reads_the_request_without_replacing_absolute_source_selection() -> TestResult
{
    let case = parcel()?;
    let file = case.directory().join("inputs/parcel.fsl");
    let input = request(source(
        file.to_str().expect("temporary source path"),
        "parcel",
    ));
    let encoded = serde_json::to_vec(&input)?;
    let piped = case.run_stdin(
        &[
            "compile",
            "/dev/stdin",
            "--output",
            "stdin.package.json",
            "--format",
            "json",
        ],
        &encoded,
    )?;
    success(&piped)?;
    package(&case, "stdin.package.json", &["Total", "Parcel"])
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; invalid invocation of the explicit request-file form
#[test]
fn stdin_does_not_supply_a_missing_request_operand() -> TestResult {
    let case = parcel()?;
    let input = serde_json::to_vec(&request(source("inputs/parcel.fsl", "parcel")))?;
    let missing = case.run_stdin(&["compile", "--format", "json"], &input)?;
    let document = refusal(&missing, 20, "invalid-request")?;
    assert!(document["last_stage"].is_null());
    assert!(document.to_string().contains("compile"));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; native filesystem request operand
#[cfg(unix)]
#[test]
fn a_non_utf8_request_filename_is_preserved_as_an_os_path() -> TestResult {
    use std::ffi::{OsStr, OsString};
    use std::os::unix::ffi::OsStringExt;
    let case = parcel()?;
    let name = OsString::from_vec(b"compile-\xff.json".to_vec());
    let input = request(source("inputs/parcel.fsl", "parcel"));
    fs::write(case.directory().join(&name), serde_json::to_vec(&input)?)?;
    let args = [
        OsStr::new("compile"),
        name.as_os_str(),
        OsStr::new("--output"),
        OsStr::new("native-path.json"),
        OsStr::new("--format"),
        OsStr::new("json"),
    ];
    success(&case.run(&args)?)?;
    package(&case, "native-path.json", &["Total", "Parcel"])
}

/// Provenance: QSL-2470; FSL FR-810-AC-2
#[test]
fn malformed_output_records_refuse_before_writing_a_package() -> TestResult {
    let case = parcel()?;
    for output in [
        json!({"package":""}),
        json!({"path":"never.json"}),
        json!({"package":3}),
        json!({"package":"never.json","extra":true}),
    ] {
        let mut input = request(source("parcel.fsl", "parcel"));
        input["request"]["output"] = output;
        write_request(&case, "inputs/bad-output.json", &input)?;
        let doc = refusal(
            &case.run(&["compile", "inputs/bad-output.json", "--format", "json"])?,
            20,
            "invalid-request",
        )?;
        assert!(doc["last_stage"].is_null());
        assert!(!case.directory().join("inputs/never.json").exists());
    }
    Ok(())
}

/// Provenance: QSL-2470; FSL FR-810-AC-5
#[test]
fn an_output_inside_the_requested_handoff_is_refused_without_partial_artifacts() -> TestResult {
    let case = parcel()?;
    let mut input = request(source("parcel.fsl", "parcel"));
    input["request"]["output"] = json!({"package":"handoff/package.json"});
    input["request"]["handoff"] = json!({"directory":"handoff","package_name":"parcel-contract","native_join":{"work_units":100000,"storage":100000}});
    write_request(&case, "inputs/conflicting-handoff.json", &input)?;
    let doc = refusal(
        &case.run(&[
            "compile",
            "inputs/conflicting-handoff.json",
            "--format",
            "json",
        ])?,
        20,
        "invalid-request",
    )?;
    assert!(doc["last_stage"].is_null());
    assert!(doc.to_string().contains("output") && doc.to_string().contains("handoff"));
    assert!(!case.directory().join("inputs/handoff").exists());
    Ok(())
}

/// Provenance: QSL-2470; FSL FR-001 source admission ceiling
#[test]
fn default_source_size_limit_is_distinct_from_the_aggregate_dependent_budget() -> TestResult {
    let case = parcel()?;
    let mut bytes = SOURCE.as_bytes().to_vec();
    bytes.extend_from_slice(b"\n//");
    bytes.resize(1_048_577, b' ');
    case.write("inputs/parcel.fsl", &bytes)?;
    let document = refusal(
        &case.run(&["compile", "inputs/compile.json", "--format", "json"])?,
        22,
        "stage_limit_exceeded",
    )?;
    assert_eq!(document["last_stage"], "S1");
    assert!(document.to_string().contains("s1.input_bytes"));
    Ok(())
}

/// Trace: FR-003-AC-3
/// Provenance: QSL-2470; FSL FR-869-AC-2
#[test]
fn an_emission_byte_limit_refuses_without_publishing_partial_package_bytes() -> TestResult {
    let case = parcel()?;
    let document = refusal(
        &case.run(&[
            "compile",
            "inputs/compile.json",
            "--limit",
            "emit.output_bytes=1",
            "--output",
            "not-written.json",
            "--format",
            "json",
        ])?,
        22,
        "stage_limit_exceeded",
    )?;
    assert!(document.to_string().contains("emit.output_bytes"));
    assert!(!case.directory().join("not-written.json").exists());
    Ok(())
}
