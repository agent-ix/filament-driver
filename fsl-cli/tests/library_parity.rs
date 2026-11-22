// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `fsl` binary and the in-process library path agree for one
//! Value-family source: same disposition, identity, diagnostics and exit code.

use std::collections::BTreeMap;
use std::process::{Command, Output};

use filament_driver::{
    execute, ExecutionEngine, ExecutionFailure, ExecutionRequest, ObjectEnvironment,
};
use filament_exact::{Cancel, Integer, Value};
use filament_semantic_value::catalog::Category;
use filament_semantic_value::supplied_stop::SuppliedLimits;
use fsl_foundation::SourceIdentity;
use fsl_replay::spine::{
    check, default_accounting, package, parse, select, Call, CallArgument, DependencyInput,
    PackageLimits, ParseRequest, SpineLimits, DEFAULT_WORK_UNITS,
};
use fsl_replay::OutcomeDocument;
use ix_trace_rs::trace;

#[path = "common/requests.rs"]
mod requests;

const FSL: &str = env!("CARGO_BIN_EXE_fsl");

const VALUE_SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"filament.value.complete/v1\";\n\
    type Digit = Int[0, 9];\n\
    function p using v(x: Digit): Boolean pure { x < 10 }\n\
    function q using v(x: Digit): Boolean pure { x < 5 }\n";

const DIVISION_SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"filament.value.complete/v1\";\n\
    function q using v(a: Int[-7, 7], b: Int[1, 3]): Int[-7, 7] pure { a div b }\n";

const MALFORMED_SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"filament.value.complete/v1\";\n\
    function f using v(: Boolean pure { true }\n";

/// Removes its directory when dropped, including after a failed assertion.
struct TestDirectory(std::path::PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Write `contents` to a fresh source file and return its directory guard and path.
fn source_file(label: &str, contents: &str) -> (TestDirectory, String) {
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("filament-parity-{label}-{}", std::process::id())),
    );
    std::fs::create_dir_all(&directory.0).expect("create source directory");
    let path = directory.0.join("source.fsl");
    std::fs::write(&path, contents).expect("write source");
    let path = path.to_str().expect("UTF-8 temp path").to_owned();
    (directory, path)
}

/// What the binary must print: the kit's canonical encoding of the library
/// document plus a newline, with the exit code of the document's category.
fn expected(document: &OutcomeDocument) -> (Vec<u8>, i32) {
    let mut encoded =
        ix_cli_kit::json::encode_canonical(document, false).expect("encode library document");
    encoded.push('\n');
    (
        encoded.into_bytes(),
        i32::from(document.category().exit_code()),
    )
}

fn library_check(path: &str, bytes: &[u8]) -> OutcomeDocument {
    let source = SourceIdentity::new("filament", path);
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let packages = BTreeMap::new();
    let parsed = match parse(
        &ParseRequest {
            source: &source,
            path,
            bytes,
        },
        limits.source,
        &cancel,
    ) {
        Ok(parsed) => parsed,
        Err(failure) => return OutcomeDocument::from_check(&Err(failure)),
    };
    let models = match select(parsed.value(), &packages, limits.model, &cancel) {
        Ok(models) => models,
        Err(failure) => return OutcomeDocument::from_check(&Err(failure)),
    };
    OutcomeDocument::from_check(&check(
        parsed.value(),
        path.to_owned(),
        models.value(),
        &DependencyInput::default(),
        limits,
        &cancel,
    ))
}

fn library_package(path: &str, bytes: &[u8]) -> OutcomeDocument {
    let source = SourceIdentity::new("filament", path);
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let packages = BTreeMap::new();
    let parsed = match parse(
        &ParseRequest {
            source: &source,
            path,
            bytes,
        },
        limits.source,
        &cancel,
    ) {
        Ok(parsed) => parsed,
        Err(failure) => return OutcomeDocument::from_package(&Err(failure)),
    };
    let models = match select(parsed.value(), &packages, limits.model, &cancel) {
        Ok(models) => models,
        Err(failure) => return OutcomeDocument::from_package(&Err(failure)),
    };
    let checked = match check(
        parsed.value(),
        path.to_owned(),
        models.value(),
        &DependencyInput::default(),
        limits,
        &cancel,
    ) {
        Ok(checked) => checked,
        Err(failure) => return OutcomeDocument::from_package(&Err(failure)),
    };
    OutcomeDocument::from_package(&package(checked.value(), PackageLimits::default(), &cancel))
}

fn library_execute(
    path: &str,
    bytes: &[u8],
    function: &str,
    arguments: Vec<CallArgument>,
) -> OutcomeDocument {
    library_execute_with_law(path, bytes, function, arguments, None)
}

fn library_execute_with_law(
    path: &str,
    bytes: &[u8],
    function: &str,
    arguments: Vec<CallArgument>,
    integer_division: Option<fsl_replay::AdmittedIntegerDivision>,
) -> OutcomeDocument {
    let packages = BTreeMap::new();
    let dependencies = DependencyInput::default();
    let request = ExecutionRequest {
        source: SourceIdentity::new("filament", path),
        package_name: path,
        path,
        bytes,
        packages: &packages,
        dependencies: &dependencies,
        limits: SpineLimits::default(),
        call: Call {
            function: function.to_owned(),
            arguments,
            accounting: default_accounting(DEFAULT_WORK_UNITS),
            supplied: SuppliedLimits::default(),
            backend: Default::default(),
            clocks: Default::default(),
            live_occurrences: fsl_replay::spine::DEFAULT_LIVE_OCCURRENCES,
            wire_arguments: Vec::new(),
            canonical_values: true,
            snapshot: None,
        },
        integer_division,
        object_environment: &ObjectEnvironment::default(),
        cancel: Cancel::new(),
        engine: ExecutionEngine::Interpreter,
    };
    match execute(request) {
        Ok(result) => result.document,
        Err(ExecutionFailure::Run { document, .. }) => *document,
        Err(other) => panic!("unexpected library failure: {other}"),
    }
}

fn digit_argument(x: i64) -> Vec<CallArgument> {
    vec![CallArgument {
        parameter: "x".to_owned(),
        value: Value::Integer(Integer::from(x)),
    }]
}

fn fsl(arguments: &[&str]) -> Output {
    Command::new(FSL)
        .args(arguments)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run fsl")
}

fn assert_parity(output: &Output, document: &OutcomeDocument, context: &str) {
    let (bytes, code) = expected(document);
    assert_eq!(output.status.code(), Some(code), "{context}: exit code");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&bytes),
        "{context}: document"
    );
}

/// Trace: FR-299-AC-1, FR-299-AC-2; `fsl check`, `compile` and `run` print
/// the library outcome for the same source, byte for byte, with the library's
/// exit code, and a repeat run prints equal bytes.
#[test]
fn binary_outcomes_equal_the_library_outcomes_for_one_value_source() {
    let (_directory, path) = source_file("value", VALUE_SOURCE);
    let bytes = VALUE_SOURCE.as_bytes();
    let success = i32::from(Category::Success.exit_code());

    let checked = library_check(&path, bytes);
    assert_eq!(i32::from(checked.category().exit_code()), success);
    let output = fsl(&["check", &path]);
    assert_parity(&output, &checked, "check");
    assert_eq!(output.stdout, fsl(&["check", &path]).stdout);

    let compile_request = requests::request(&path, None);
    let p_request = requests::request(
        &path,
        Some((
            "p",
            serde_json::json!([{"parameter":"x","value":{"type":"integer","value":"3"}}]),
        )),
    );
    let q_request = requests::request(
        &path,
        Some((
            "q",
            serde_json::json!([{"parameter":"x","value":{"type":"integer","value":"7"}}]),
        )),
    );
    let packaged = library_package(&path, bytes);
    assert_eq!(i32::from(packaged.category().exit_code()), success);
    let output = fsl(&["compile", compile_request.as_str()]);
    assert_parity(&output, &packaged, "compile");
    assert_eq!(
        output.stdout,
        fsl(&["compile", compile_request.as_str()]).stdout
    );

    let executed = library_execute(&path, bytes, "p", digit_argument(3));
    assert_eq!(i32::from(executed.category().exit_code()), success);
    let output = fsl(&["run", p_request.as_str()]);
    assert_parity(&output, &executed, "run p x=3");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("run prints one JSON document");
    assert_eq!(
        document["result"]["value"],
        serde_json::json!({"type":"boolean","value":true})
    );
    assert_eq!(output.stdout, fsl(&["run", p_request.as_str()]).stdout);

    // The second function of the source takes the same path.
    let executed = library_execute(&path, bytes, "q", digit_argument(7));
    assert_parity(&fsl(&["run", q_request.as_str()]), &executed, "run q x=7");
}

/// Trace: FR-299-AC-1; a refused source gives the library's refusal document,
/// identity and exit code through `check`, `compile` and `run`.
#[test]
fn binary_refusals_equal_the_library_refusals_for_a_malformed_source() {
    let (_directory, path) = source_file("malformed", MALFORMED_SOURCE);
    let bytes = MALFORMED_SOURCE.as_bytes();
    let compile_request = requests::request(&path, None);
    let run_request = requests::request(&path, Some(("f", serde_json::json!([]))));

    let checked = library_check(&path, bytes);
    assert_ne!(
        i32::from(checked.category().exit_code()),
        i32::from(Category::Success.exit_code())
    );
    assert_parity(&fsl(&["check", &path]), &checked, "check");
    assert_parity(
        &fsl(&["compile", compile_request.as_str()]),
        &library_package(&path, bytes),
        "compile",
    );
    assert_parity(
        &fsl(&["run", run_request.as_str()]),
        &library_execute(&path, bytes, "f", Vec::new()),
        "run",
    );
}

/// FR-002-AC-19: `fsl check` names a source that does not parse by
/// authority `filament` and the path as given, never by a command-layer
/// identity, and its refusal equals the library's under that identity.
#[test]
#[trace("TC-043", "FR-002-AC-19")]
#[ignore = "QSL-2423: check_outcome names a malformed source fsl-command/check-source instead of filament and the path as given"]
fn a_malformed_check_source_is_named_by_the_supplied_path() {
    let (_directory, path) = source_file("malformed-identity", MALFORMED_SOURCE);
    let output = fsl(&["check", &path]);
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check prints one JSON document");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("the refusal carries diagnostics");
    assert!(!diagnostics.is_empty(), "{document}");
    for diagnostic in diagnostics {
        assert_eq!(
            diagnostic["locus"]["source"],
            serde_json::json!({"authority": "filament", "identity": path}),
            "{diagnostic}"
        );
    }
    let text = document.to_string();
    assert!(!text.contains("fsl-command"), "{text}");
    assert!(!text.contains("check-source"), "{text}");
    assert_parity(
        &output,
        &library_check(&path, MALFORMED_SOURCE.as_bytes()),
        "check",
    );
}

/// Trace: FR-767-AC-7; `fsl run` with a request's `integer_division` law prints
/// the library outcome `execute` gives for the same admitted law, byte for
/// byte, and the two laws give different values for `q(-7, 3)`.
#[test]
fn binary_run_equals_the_library_run_under_each_integer_division_law() {
    use fsl_replay::DefinitionRole;
    let (_directory, path) = source_file("division", DIVISION_SOURCE);
    let source_path = std::fs::canonicalize(&path).expect("existing source");
    let arguments = vec![
        CallArgument {
            parameter: "a".to_owned(),
            value: Value::Integer(Integer::from(-7_i64)),
        },
        CallArgument {
            parameter: "b".to_owned(),
            value: Value::Integer(Integer::from(3_i64)),
        },
    ];
    for (law, role, expected_value) in [
        ("floor", DefinitionRole::IntegerDivisionFloor, "-3"),
        (
            "truncating",
            DefinitionRole::IntegerDivisionTruncating,
            "-2",
        ),
    ] {
        let request = source_file(
            law,
            &serde_json::json!({"format": "native-run/1", "request": {"models": [],
                "integer_division": law,
                "program": {"source": {"authority": "filament", "identity": path,
                    "file": source_path, "document": "Division", "formal_revision": 1}},
                "call": {"function": "q", "arguments": [
                    {"parameter": "a", "value": {"type":"integer","value":"-7"}}, {"parameter": "b", "value": {"type":"integer","value":"3"}}]}}})
            .to_string(),
        );
        let admitted = fsl_replay::admit_integer_division(&[role.reference()], None)
            .expect("the law is admitted");
        let library = library_execute_with_law(
            &path,
            DIVISION_SOURCE.as_bytes(),
            "q",
            arguments.clone(),
            Some(admitted),
        );
        let output = fsl(&["run", &request.1]);
        assert_parity(&output, &library, law);
        assert_eq!(output.status.code(), Some(0), "{law}: {output:?}");
        let document: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("run prints one JSON document");
        assert_eq!(
            document["result"],
            serde_json::json!({"kind": "completed", "value": {"type":"integer","value": expected_value}}),
            "{law}: {document}"
        );
    }
}
