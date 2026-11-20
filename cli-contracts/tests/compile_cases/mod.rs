// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original request/source programs for the compile command.

use std::error::Error;
use std::fs;

use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::{json, Value};

type TestResult = Result<(), Box<dyn Error>>;
const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
record Parcel { pieces: Int[0, 31]; }
function Total using v(pieces: Integer): Integer pure { pieces + 2 }
"#;

fn source(file: &str, identity: &str) -> Value {
    json!({"source":{"file":file,"authority":"compile-contracts","identity":identity,
        "document":"ParcelCompile","formal_revision":1}})
}

fn request(program: Value) -> Value {
    json!({"format":"native-compile/1","request":{"models":[],"program":program}})
}

fn write_request(case: &CliCase, path: &str, value: &Value) -> Result<(), ContractError> {
    case.write(path, &serde_json::to_vec(value)?)
}

fn parcel() -> Result<CliCase, ContractError> {
    let case = CliCase::from_environment()?;
    case.write("inputs/parcel.fsl", SOURCE.as_bytes())?;
    write_request(
        &case,
        "inputs/compile.json",
        &request(source("parcel.fsl", "parcel")),
    )?;
    Ok(case)
}

fn success(output: &Captured) -> Result<Value, ContractError> {
    assert_eq!(output.0.status.code(), Some(0), "{}", output.stdout_text()?);
    assert_eq!(output.stderr_text()?, "");
    let doc = output.json()?;
    assert_eq!(doc["format"], "filament-outcome/1", "{doc}");
    assert_eq!(doc["operation"], "package", "{doc}");
    assert_eq!(doc["category"], "success", "{doc}");
    assert_eq!(doc["last_stage"], "S4", "{doc}");
    assert_eq!(doc["artifacts"], json!([]));
    assert_eq!(doc["diagnostics"], json!([]));
    Ok(doc)
}

fn refusal(output: &Captured, exit: i32, code: &str) -> Result<Value, ContractError> {
    assert_eq!(
        output.0.status.code(),
        Some(exit),
        "{}",
        output.stdout_text()?
    );
    let doc = output.json()?;
    assert_eq!(doc["format"], "filament-outcome/1", "{doc}");
    assert_eq!(doc["operation"], "package", "{doc}");
    assert_eq!(
        doc["category"],
        match exit {
            21 => "unsupported",
            22 => "incomplete",
            _ => "refusal",
        }
    );
    assert_eq!(doc["artifacts"], json!([]));
    assert!(
        doc["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["code"] == code),
        "{doc}"
    );
    Ok(doc)
}

fn argument_refusal(output: &Captured, context: &str) -> Result<(), ContractError> {
    assert_eq!(output.0.status.code(), Some(20));
    let message = format!("{}{}", output.stdout_text()?, output.stderr_text()?);
    assert!(
        message.contains(context),
        "missing offending argument {context:?}: {message}"
    );
    if !output.0.stdout.is_empty() {
        let doc = output.json()?;
        assert_eq!(doc["operation"], "package", "{doc}");
        assert_eq!(doc["category"], "refusal", "{doc}");
    }
    Ok(())
}

fn package(case: &CliCase, path: &str, declarations: &[&str]) -> TestResult {
    let bytes = fs::read(case.directory().join(path))?;
    let artifact: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(
        artifact["format"], "filament.checked-package/v2",
        "{artifact}"
    );
    let content = artifact.to_string();
    for declaration in declarations {
        assert!(
            content.contains(declaration),
            "missing compiled declaration {declaration}: {artifact}"
        );
    }
    Ok(())
}

/// Trace: FR-024-AC-1, FR-002-AC-23
/// Provenance: QSL-2470; FSL FR-027-AC-5, FR-665
#[test]
fn native_source_emits_a_package_that_the_public_handoff_reader_verifies() -> TestResult {
    let case = parcel()?;
    let mut input = request(source("parcel.fsl", "parcel"));
    input["request"]["handoff"] = json!({"directory":"handoff","package_name":"parcel-contract",
        "native_join":{"work_units":100000,"storage":100000}});
    write_request(&case, "inputs/compile.json", &input)?;
    success(&case.run(&[
        "compile",
        "inputs/compile.json",
        "--output",
        "parcel.package.json",
        "--format",
        "json",
    ])?)?;
    package(&case, "parcel.package.json", &["Total", "Parcel"])?;
    assert_eq!(
        fs::read(case.directory().join("parcel.package.json"))?,
        fs::read(case.directory().join("inputs/handoff/checked-package.json"))?
    );
    let inspect = case.run(&["inspect", "--handoff", "inputs/handoff", "--format", "json"])?;
    assert_eq!(
        inspect.0.status.code(),
        Some(0),
        "{}",
        inspect.stdout_text()?
    );
    let inspected = inspect.json()?;
    assert_eq!(inspected["operation"], "inspect");
    assert_eq!(inspected["category"], "success");
    Ok(())
}

/// Trace: FR-024-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-5, FR-810-AC-1
#[test]
fn request_and_cli_output_paths_emit_equal_bytes_and_ignore_document_revision() -> TestResult {
    let case = parcel()?;
    let cli = success(&case.run(&[
        "compile",
        "inputs/compile.json",
        "--output",
        "cli.json",
        "--format",
        "json",
    ])?)?;
    let mut input = request(source("parcel.fsl", "parcel"));
    input["request"]["output"] = json!({"package":"member.json"});
    input["request"]["program"]["source"]["document"] = json!("ChangedAnnotation");
    input["request"]["program"]["source"]["formal_revision"] = json!(29);
    write_request(&case, "inputs/member.json", &input)?;
    let member = success(&case.run(&["compile", "inputs/member.json", "--format", "json"])?)?;
    assert_eq!(cli, member);
    assert_eq!(
        fs::read(case.directory().join("cli.json"))?,
        fs::read(case.directory().join("inputs/member.json"))?
    );
    package(&case, "cli.json", &["Total", "Parcel"])
}

/// Trace: FR-002-AC-1, FR-024-AC-1
/// Provenance: QSL-2470; FSL FR-287-AC-9
#[test]
fn absolute_request_from_another_cwd_resolves_dependencies_beside_the_request() -> TestResult {
    let owner = parcel()?;
    let relative = success(&owner.run(&["compile", "inputs/compile.json", "--format", "json"])?)?;
    let other = CliCase::from_environment()?;
    let path = owner.directory().join("inputs/compile.json");
    let absolute = success(&other.run(&[
        "compile".as_ref(),
        path.as_os_str(),
        "--output".as_ref(),
        "remote.json".as_ref(),
        "--format".as_ref(),
        "json".as_ref(),
    ])?)?;
    assert_eq!(relative, absolute);
    assert!(!owner.directory().join("remote.json").exists());
    package(&other, "remote.json", &["Total", "Parcel"])
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027 program unit list
#[test]
fn multiple_source_units_retain_both_declarations() -> TestResult {
    let case = parcel()?;
    case.write("inputs/offset.fsl", b"language \"ix:native\" edition \"1-draft\";\nprofile n = \"filament.value.complete/v1\";\nfunction Offset using n(): Integer pure { 5 }\n")?;
    let input = request(json!([
        source("parcel.fsl", "parcel"),
        source("offset.fsl", "offset")
    ]));
    write_request(&case, "inputs/multi.json", &input)?;
    success(&case.run(&[
        "compile",
        "inputs/multi.json",
        "--output",
        "multi.package.json",
        "--format",
        "json",
    ])?)?;
    package(&case, "multi.package.json", &["Total", "Offset"])
}

/// Trace: FR-002-AC-13
/// Provenance: QSL-2470
#[test]
fn default_and_explicit_text_describe_the_same_success_as_machine_mode() -> TestResult {
    let case = parcel()?;
    success(&case.run(&["compile", "inputs/compile.json", "--format", "json"])?)?;
    let default = case.run(&["compile", "inputs/compile.json"])?;
    let explicit = case.run(&["compile", "inputs/compile.json", "--format", "text"])?;
    assert_eq!(default.0.status.code(), Some(0));
    assert_eq!(default.0.stdout, explicit.0.stdout);
    assert_eq!(default.stderr_text()?, "");
    assert!(default
        .stdout_text()?
        .starts_with("package: success (exit 0) at S4"));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470
#[test]
fn named_request_is_used_even_when_stdin_contains_invalid_bytes() -> TestResult {
    let case = parcel()?;
    let file = case.run(&["compile", "inputs/compile.json", "--format", "json"])?;
    let piped = case.run_stdin(
        &["compile", "inputs/compile.json", "--format", "json"],
        &[0xff, 0, b'{'],
    )?;
    success(&file)?;
    success(&piped)?;
    assert_eq!(file.0.stdout, piped.0.stdout);
    Ok(())
}

/// Trace: FR-024-AC-2
/// Provenance: QSL-2470
#[test]
fn an_existing_output_file_is_preserved_without_compiling() -> TestResult {
    let case = parcel()?;
    case.write("keep.json", b"original owner bytes")?;
    let doc = refusal(
        &case.run(&[
            "compile",
            "inputs/compile.json",
            "--output",
            "keep.json",
            "--format",
            "json",
        ])?,
        20,
        "invalid-request",
    )?;
    assert!(doc["last_stage"].is_null(), "{doc}");
    assert!(doc.to_string().contains("keep.json"));
    assert_eq!(
        fs::read(case.directory().join("keep.json"))?,
        b"original owner bytes"
    );
    Ok(())
}

/// Trace: FR-024-AC-3
/// Provenance: QSL-2470
#[test]
fn request_output_and_cli_override_refuse_before_writing_either_target() -> TestResult {
    let case = parcel()?;
    let mut input = request(source("parcel.fsl", "parcel"));
    input["request"]["output"] = json!({"package":"member.package.json"});
    write_request(&case, "inputs/conflict.json", &input)?;
    let doc = refusal(
        &case.run(&[
            "compile",
            "inputs/conflict.json",
            "--output",
            "override.json",
            "--format",
            "json",
        ])?,
        20,
        "invalid-request",
    )?;
    assert!(doc["last_stage"].is_null());
    assert!(doc.to_string().contains("output") && doc.to_string().contains("--output"));
    assert!(!case.directory().join("override.json").exists());
    assert!(!case.directory().join("inputs/member.package.json").exists());
    Ok(())
}

/// Trace: FR-024-AC-6
/// Provenance: QSL-2470
#[test]
fn ill_typed_source_preserves_its_stage_and_writes_no_partial_output() -> TestResult {
    let case = parcel()?;
    case.write(
        "inputs/parcel.fsl",
        SOURCE.replace("pieces + 2", "true").as_bytes(),
    )?;
    let doc = refusal(
        &case.run(&[
            "compile",
            "inputs/compile.json",
            "--output",
            "refused.json",
            "--format",
            "json",
        ])?,
        20,
        "ill_typed",
    )?;
    assert_eq!(doc["last_stage"], "S3");
    assert!(doc.to_string().contains("type-mismatch"));
    assert!(doc.to_string().contains("compile-contracts") && doc.to_string().contains("parcel"));
    assert!(!case.directory().join("refused.json").exists());
    Ok(())
}

/// Trace: FR-002-AC-18
/// Provenance: QSL-2470
#[test]
fn absent_or_directory_requests_keep_the_io_refusal_in_json_and_text() -> TestResult {
    let case = CliCase::from_environment()?;
    case.write("folder/marker", b"directory input")?;
    for path in ["missing.json", "folder"] {
        let doc = refusal(
            &case.run(&["compile", path, "--format", "json"])?,
            20,
            "io-error",
        )?;
        assert!(doc["last_stage"].is_null());
        assert!(doc.to_string().contains(path));
        let text = case.run(&["compile", path])?;
        assert_eq!(text.0.status.code(), Some(20));
        assert!(text.stdout_text()?.contains("package: refusal (exit 20)"));
        assert!(text.stdout_text()?.contains("io-error"));
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-2
#[test]
fn malformed_request_bytes_never_become_a_successful_package() -> TestResult {
    let case = CliCase::from_environment()?;
    for (index, bytes) in [
        b"".as_slice(),
        b"{",
        &[0xff],
        b"[]",
        b"{}",
        b"{\"format\":\"native-compile/1\",\"request\":{}} trailing",
    ]
    .into_iter()
    .enumerate()
    {
        let path = format!("bad-{index}.json");
        case.write(&path, bytes)?;
        let doc = refusal(
            &case.run(&["compile", &path, "--format", "json"])?,
            20,
            "invalid-request",
        )?;
        assert!(doc["last_stage"].is_null(), "{doc}");
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-2, AC-7
#[test]
fn closed_compile_request_rejects_runtime_and_unknown_fields() -> TestResult {
    let case = parcel()?;
    for field in ["snapshots", "invocations", "selection", "package", "extra"] {
        let mut input = request(source("does-not-exist.fsl", "parcel"));
        input["request"][field] = json!([]);
        write_request(&case, "inputs/forbidden.json", &input)?;
        let doc = refusal(
            &case.run(&["compile", "inputs/forbidden.json", "--format", "json"])?,
            20,
            "invalid-request",
        )?;
        assert!(doc["last_stage"].is_null());
        assert!(doc.to_string().contains(field));
        assert!(!doc.to_string().contains("io-error"));
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-3, FR-001
#[test]
fn source_selection_requires_both_labels_before_reading_a_file() -> TestResult {
    let case = parcel()?;
    for label in ["authority", "identity"] {
        let mut input = request(source("does-not-exist.fsl", "parcel"));
        input["request"]["program"]["source"]
            .as_object_mut()
            .expect("source object")
            .remove(label);
        write_request(&case, "inputs/missing-label.json", &input)?;
        let doc = refusal(
            &case.run(&["compile", "inputs/missing-label.json", "--format", "json"])?,
            20,
            "invalid-request",
        )?;
        assert!(doc["last_stage"].is_null());
        assert!(doc.to_string().contains(label));
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-3, FR-001
#[test]
fn non_utf8_source_reports_encoding_and_no_package() -> TestResult {
    let case = parcel()?;
    case.write("inputs/parcel.fsl", &[0xff])?;
    let doc = refusal(
        &case.run(&[
            "compile",
            "inputs/compile.json",
            "--output",
            "never.json",
            "--format",
            "json",
        ])?,
        20,
        "invalid_utf8",
    )?;
    assert!(doc.to_string().contains("parcel"));
    assert!(!case.directory().join("never.json").exists());
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-3
#[test]
fn missing_or_directory_dependent_source_is_not_an_empty_program() -> TestResult {
    let case = parcel()?;
    for file in ["absent.fsl", "."] {
        write_request(
            &case,
            "inputs/unreadable.json",
            &request(source(file, "parcel")),
        )?;
        let doc = refusal(
            &case.run(&["compile", "inputs/unreadable.json", "--format", "json"])?,
            20,
            "io-error",
        )?;
        assert!(doc.to_string().contains(file));
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-7, AC-8
#[test]
fn empty_syntax_and_unsupported_source_fail_in_their_own_classes() -> TestResult {
    let case = parcel()?;
    for (bytes, exit, code) in [("",20,"invalid_syntax"),("not a native unit",20,"invalid_syntax"),("language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\nunknownDeclaration",21,"unsupported_construct")] {
        case.write("inputs/parcel.fsl", bytes.as_bytes())?;
        let doc = refusal(&case.run(&["compile", "inputs/compile.json", "--format", "json"])?, exit, code)?;
        assert!(!doc["diagnostics"].as_array().expect("diagnostics").is_empty());
    }
    Ok(())
}

/// Trace: FR-002-AC-1, FR-003-AC-3
/// Provenance: QSL-2470; FSL FR-027-AC-3
#[test]
fn oversized_request_refuses_at_intake_with_its_configured_limit() -> TestResult {
    let case = parcel()?;
    let mut bytes = serde_json::to_vec(&request(source("parcel.fsl", "parcel")))?;
    bytes.resize(1_048_577, b' ');
    case.write("inputs/oversized.json", &bytes)?;
    let doc = refusal(
        &case.run(&["compile", "inputs/oversized.json", "--format", "json"])?,
        22,
        "resource_exhausted",
    )?;
    assert!(doc["last_stage"].is_null());
    assert!(doc.to_string().contains("request_bytes") && doc.to_string().contains("1048576"));
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2470; FSL FR-027-AC-3 and FR-026 primary dependent accounting
#[test]
fn primary_program_bytes_count_toward_the_compile_dependent_budget() -> TestResult {
    let case = parcel()?;
    let mut bytes = SOURCE.as_bytes().to_vec();
    bytes.extend_from_slice(b"\n//");
    bytes.resize(8_388_609, b' ');
    case.write("inputs/parcel.fsl", &bytes)?;
    let doc = refusal(
        &case.run(&["compile", "inputs/compile.json", "--format", "json"])?,
        22,
        "resource_exhausted",
    )?;
    assert!(doc["last_stage"].is_null());
    assert!(doc.to_string().contains("dependent_bytes") && doc.to_string().contains("8388608"));
    Ok(())
}

/// Trace: FR-003-AC-3
/// Provenance: QSL-2470; FSL FR-869-AC-3
#[test]
fn caller_lowered_source_limit_refuses_and_a_fresh_default_call_succeeds() -> TestResult {
    let case = parcel()?;
    let doc = refusal(
        &case.run(&[
            "compile",
            "inputs/compile.json",
            "--limit",
            "s1.input_bytes=1",
            "--format",
            "json",
        ])?,
        22,
        "stage_limit_exceeded",
    )?;
    assert!(doc.to_string().contains("s1.input_bytes"));
    success(&case.run(&[
        "compile",
        "inputs/compile.json",
        "--limit",
        "s1.input_bytes=4096",
        "--format",
        "json",
    ])?)?;
    success(&case.run(&["compile", "inputs/compile.json", "--format", "json"])?)?;
    Ok(())
}

/// Trace: FR-002-AC-1, FR-003-AC-7, FR-024-AC-5
/// Provenance: QSL-2470; FSL FR-287-AC-3, AC-5
#[test]
fn invalid_arguments_name_the_operand_and_do_not_create_outputs() -> TestResult {
    for (args, context) in [
        (vec!["compile"], "compile"),
        (vec!["compile", "missing.json", "extra.json"], "compile"),
        (vec!["compile", "missing.json", "--bogus"], "--bogus"),
        (
            vec!["compile", "missing.json", "--format", "yaml"],
            "--format",
        ),
        (vec!["compile", "missing.json", "--format"], "--format"),
        (vec!["compile", "missing.json", "--output"], "--output"),
        (
            vec![
                "compile",
                "missing.json",
                "--output",
                "never.json",
                "--output",
                "other.json",
            ],
            "--output",
        ),
        (vec!["compile", "missing.json", "--clocks"], "--clocks"),
        (
            vec![
                "compile",
                "missing.json",
                "--clocks",
                "a.json",
                "--clocks",
                "b.json",
            ],
            "--clocks",
        ),
        (vec!["compile", "missing.json", "--limit"], "--limit"),
        (
            vec!["compile", "missing.json", "--limit", "s1.tokens"],
            "s1.tokens",
        ),
        (
            vec!["compile", "missing.json", "--limit", "s9.nodes=1"],
            "s9.nodes",
        ),
        (
            vec!["compile", "missing.json", "--limit", "s1.nodes=ten"],
            "s1.nodes",
        ),
        (
            vec![
                "compile",
                "missing.json",
                "--limit",
                "s1.nodes=5",
                "--limit",
                "s1.nodes=6",
            ],
            "s1.nodes",
        ),
        (
            vec![
                "compile",
                "missing.json",
                "--limit",
                "admission.work_units=5",
            ],
            "admission.work_units",
        ),
        (
            vec!["compile", "missing.json", "--integer-division", "floor"],
            "--integer-division",
        ),
    ] {
        let case = CliCase::from_environment()?;
        argument_refusal(&case.run(&args)?, context)?;
        assert!(!case.directory().join("never.json").exists());
        assert!(!case.directory().join("other.json").exists());
    }
    Ok(())
}

/// Trace: FR-023-AC-3, FR-023-AC-4, FR-023-AC-7
/// Provenance: QSL-2470
#[test]
fn clock_file_failures_are_bounded_typed_and_precede_the_request() -> TestResult {
    let case = parcel()?;
    for (name, bytes, exit, code, cause) in [
        (
            "array.json",
            b"[]".as_slice(),
            20,
            "invalid-request",
            "clocks-file-not-an-object",
        ),
        (
            "invalid.json",
            b"{".as_slice(),
            20,
            "invalid-request",
            "clocks-file-not-an-object",
        ),
        (
            "trailing.json",
            b"{} {}".as_slice(),
            20,
            "invalid-request",
            "clocks-file-not-an-object",
        ),
        (
            "encoding.json",
            &[0xff],
            20,
            "invalid-request",
            "clocks-file-not-an-object",
        ),
    ] {
        case.write(name, bytes)?;
        let doc = refusal(
            &case.run(&[
                "compile",
                "missing-request.json",
                "--clocks",
                name,
                "--format",
                "json",
            ])?,
            exit,
            code,
        )?;
        assert!(doc["last_stage"].is_null());
        assert!(doc.to_string().contains(cause));
    }
    let missing = refusal(
        &case.run(&[
            "compile",
            "missing-request.json",
            "--clocks",
            "absent-clocks.json",
            "--format",
            "json",
        ])?,
        20,
        "io-error",
    )?;
    assert!(missing.to_string().contains("clocks-file-unreadable"));
    case.write("huge-clocks.json", &vec![b' '; 1_048_577])?;
    let huge = refusal(
        &case.run(&[
            "compile",
            "missing-request.json",
            "--clocks",
            "huge-clocks.json",
            "--format",
            "json",
        ])?,
        22,
        "resource_exhausted",
    )?;
    assert!(huge["last_stage"].is_null());
    assert!(huge.to_string().contains("clocks-file-too-large"));
    Ok(())
}
