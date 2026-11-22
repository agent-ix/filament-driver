// SPDX-License-Identifier: AGPL-3.0-or-later
//! Process-level checks of the `fsl` frontend for `parse`, `format`, `run` and
//! `compile`: usage and arity refusals before I/O, invalid invocations, OS
//! encoding, stream write failures, closed pipes and path independence.

use std::process::{Command, Output, Stdio};

use filament_semantic_value::catalog::Category;
use ix_trace_rs::trace;

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.fsl");
const INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/parse-invalid.fsl"
);
const MISSING_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/missing-source.fsl"
);

#[path = "../../tests/common/dependency_fixtures.rs"]
mod dependency_fixtures;
#[path = "common/test_provider_path.rs"]
mod test_provider_path;
#[path = "common/timed_analyze.rs"]
mod timed_analyze;
const RUN_REQUEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-run.json"
);
const COMPILE_REQUEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-compile.json"
);
const RUN_REFUSED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-run-refused.json"
);
const COMPILE_REFUSED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-compile-refused.json"
);

fn refusal_code() -> i32 {
    i32::from(Category::Refusal.exit_code())
}

fn fsl(arguments: &[&str]) -> Output {
    Command::new(FSL).args(arguments).output().expect("run fsl")
}

/// Assert a refusal that wrote nothing to stdout and did not panic.
fn assert_refused_quietly(output: &Output, context: &str) {
    assert_eq!(output.status.code(), Some(refusal_code()), "{context}");
    assert!(output.stdout.is_empty(), "{context}: stdout must be empty");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "{context}: refusal names the problem");
    assert!(!stderr.contains("panicked"), "{context}: {stderr}");
}

#[trace("TC-1262", "FR-287-AC-3")]
#[test]
fn missing_and_extra_operands_are_refused_before_any_file_is_read() {
    let cases: [(&str, &[&str]); 8] = [
        ("run", &["run"]),
        ("run", &["run", "missing.json", "extra"]),
        ("run", &["run", "missing.fsl", "yes", "extra"]),
        ("compile", &["compile"]),
        ("compile", &["compile", "missing.fsl", "extra"]),
        ("parse", &["parse", "issuer", "document"]),
        (
            "parse",
            &["parse", "issuer", "document", "missing.fsl", "extra"],
        ),
        (
            "format",
            &["format", "issuer", "document", "missing.fsl", "extra"],
        ),
    ];
    for (verb, arguments) in cases {
        let output = fsl(arguments);
        assert_refused_quietly(&output, &format!("{arguments:?}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.starts_with(&format!("usage: fsl {verb} ")),
            "{arguments:?}: {stderr}"
        );
        assert!(
            !stderr.contains("could not read") && !stderr.contains("cannot read"),
            "{arguments:?} opened its path: {stderr}"
        );
    }
    let output = fsl(&["format", "issuer", "document"]);
    assert_refused_quietly(&output, "format without an operand");
    for verb in ["parse", "format"] {
        for (authority, identity, label) in [
            ("", "document", "authority"),
            ("issuer", "\u{3000}", "identity"),
            (" ", "", "authority"),
        ] {
            let output = fsl(&[
                verb,
                authority,
                identity,
                MISSING_SOURCE,
                "--format",
                "json",
            ]);
            assert_eq!(output.status.code(), Some(refusal_code()));
            assert!(!String::from_utf8_lossy(&output.stderr).contains("could not read"));
            let document: serde_json::Value =
                serde_json::from_slice(&output.stdout).expect("typed source identity refusal");
            let diagnostic = &document["diagnostics"][0];
            assert_eq!(diagnostic["code"], "invalid_source_identity");
            assert_eq!(diagnostic["cause"], "blank-label");
            assert_eq!(diagnostic["label"], label);
            assert_eq!(diagnostic["locus"], serde_json::Value::Null);
        }
    }
}

#[trace("TC-1262", "FR-287-AC-5")]
#[test]
fn invalid_invocations_exit_20_and_the_next_valid_call_runs() {
    type InvalidCase<'a> = (&'a [&'a str], &'a str);
    assert!(
        !std::path::Path::new(MISSING_SOURCE).exists(),
        "the invalid invocation cases require a nonexistent source"
    );
    let verbs: [(&[&str], &[InvalidCase<'_>]); 4] = [
        (
            &["parse", "issuer", "document", SOURCE, "--format", "json"],
            &[
                (
                    &["parse", "--bogus", MISSING_SOURCE],
                    "unknown parse option `--bogus`",
                ),
                (&["parse"], "usage: fsl parse "),
                (
                    &["parse", "issuer", "document", MISSING_SOURCE, "extra"],
                    "usage: fsl parse ",
                ),
                (
                    &["parse", MISSING_SOURCE, "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["parse", MISSING_SOURCE, "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
            ],
        ),
        (
            &["format", "issuer", "document", SOURCE],
            &[
                (
                    &["format", "--bogus", MISSING_SOURCE],
                    "unknown format option `--bogus`",
                ),
                (&["format"], "usage: fsl format "),
                (
                    &["format", "issuer", "document", MISSING_SOURCE, "extra"],
                    "usage: fsl format ",
                ),
                (
                    &["format", MISSING_SOURCE, "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["format", MISSING_SOURCE, "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
            ],
        ),
        (
            &["compile", COMPILE_REQUEST, "--format", "json"],
            &[
                (
                    &["compile", "--bogus", MISSING_SOURCE],
                    "unknown compile option `--bogus`",
                ),
                (&["compile"], "usage: fsl compile "),
                (&["compile", MISSING_SOURCE, "extra"], "usage: fsl compile "),
                (
                    &["compile", MISSING_SOURCE, "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["compile", MISSING_SOURCE, "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
            ],
        ),
        (
            &["run", RUN_REQUEST, "--format", "json"],
            &[
                (
                    &["run", "--bogus", MISSING_SOURCE, "yes"],
                    "unknown run option `--bogus`",
                ),
                (&["run"], "usage: fsl run "),
                (&["run", MISSING_SOURCE, "yes", "extra"], "usage: fsl run "),
                (
                    &["run", MISSING_SOURCE, "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["run", MISSING_SOURCE, "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
                (
                    &["run", MISSING_SOURCE, "--limit"],
                    "--limit requires name=value",
                ),
                (
                    &["run", MISSING_SOURCE, "--limit", "no-equals-sign"],
                    "NotAnAssignment",
                ),
                (
                    &["run", MISSING_SOURCE, "--engine", "bogus"],
                    "unknown engine `bogus`",
                ),
                (
                    &["run", MISSING_SOURCE, "--request-bytes", "-1"],
                    "--request-bytes requires a non-negative byte count",
                ),
                (
                    &["run", MISSING_SOURCE, "--dependent-bytes", "not-a-count"],
                    "--dependent-bytes requires a non-negative byte count",
                ),
            ],
        ),
    ];
    for (valid, invalid) in verbs {
        for (arguments, expected_error) in invalid {
            let output = fsl(arguments);
            assert_refused_quietly(&output, &format!("{arguments:?}"));
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains(expected_error), "{arguments:?}: {stderr}");
            assert!(
                !stderr.contains("could not read") && !stderr.contains("cannot read"),
                "{arguments:?} opened its path: {stderr}"
            );
            let output = fsl(valid);
            assert_eq!(
                output.status.code(),
                Some(i32::from(Category::Success.exit_code())),
                "{valid:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(!output.stdout.is_empty(), "{valid:?}");
        }
    }
}

#[cfg(unix)]
fn invalid_os_string() -> std::ffi::OsString {
    use std::os::unix::ffi::OsStringExt;
    std::ffi::OsString::from_vec(vec![0xff])
}

#[cfg(windows)]
fn invalid_os_string() -> std::ffi::OsString {
    use std::os::windows::ffi::OsStringExt;
    std::ffi::OsString::from_wide(&[0xd800])
}

#[cfg(any(unix, windows))]
#[trace("TC-1262", "FR-287-AC-6")]
#[test]
fn non_utf8_commands_and_labels_refuse() {
    let mut cases = vec![vec![invalid_os_string(), MISSING_SOURCE.into()]];
    for verb in ["parse", "format"] {
        cases.push(vec![
            verb.into(),
            invalid_os_string(),
            "document".into(),
            MISSING_SOURCE.into(),
        ]);
        cases.push(vec![
            verb.into(),
            "issuer".into(),
            invalid_os_string(),
            MISSING_SOURCE.into(),
        ]);
    }
    for arguments in cases {
        let output = Command::new(FSL)
            .args(&arguments)
            .output()
            .expect("run fsl with invalid command or label encoding");
        assert_refused_quietly(&output, &format!("{arguments:?}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("UTF-8"), "{arguments:?}: {stderr}");
        assert!(
            !stderr.contains("could not read") && !stderr.contains("cannot read"),
            "{arguments:?}: {stderr}"
        );
    }
}

// APFS refuses a file name that is not UTF-8, so macOS cannot create the source.
#[cfg(all(any(unix, windows), not(target_os = "macos")))]
#[trace("TC-1262", "FR-287-AC-6")]
#[test]
fn non_utf8_raw_file_paths_parse() {
    use std::ffi::OsString;

    let directory = TestDirectory(
        std::env::temp_dir().join(format!("filament-frontend-raw-path-{}", std::process::id())),
    );
    std::fs::create_dir_all(&directory.0).expect("create raw-path directory");
    let mut filename = OsString::from("source-");
    filename.push(invalid_os_string());
    filename.push(".fsl");
    let path = directory.0.join(filename);
    std::fs::copy(SOURCE, &path).expect("create source at raw OS path");
    let output = Command::new(FSL)
        .args(["parse", "issuer", "document"])
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("parse raw OS path");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("typed parse outcome");
    assert_eq!(document["category"], "success");
}

#[trace("TC-1262", "FR-287-AC-7")]
#[cfg(target_os = "linux")]
#[test]
fn stdout_write_failure_exits_30() {
    let cases: [&[&str]; 6] = [
        &["parse", "issuer", "document", SOURCE, "--format", "json"],
        &["format", "issuer", "document", SOURCE],
        &["format", "issuer", "document", SOURCE, "--format", "json"],
        &["compile", COMPILE_REQUEST, "--format", "json"],
        &["run", RUN_REQUEST, "--format", "json"],
        // A refused outcome document is written to the same stream.
        &[
            "parse",
            "issuer",
            "document",
            INVALID_SOURCE,
            "--format",
            "json",
        ],
    ];
    for arguments in cases {
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .expect("open /dev/full");
        let output = Command::new(FSL)
            .args(arguments)
            .stdout(Stdio::from(full))
            .output()
            .expect("run fsl with a full stdout");
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::InternalFailure.exit_code())),
            "{arguments:?}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("OutputFailure"),
            "{arguments:?}"
        );
    }
}

#[trace("TC-1262", "FR-287-AC-8")]
#[cfg(unix)]
#[test]
fn closed_stdout_pipe_keeps_the_outcome_exit_code_and_stays_quiet() {
    let success = i32::from(Category::Success.exit_code());
    let refusal = i32::from(Category::Refusal.exit_code());
    let cases: [(&[&str], i32); 9] = [
        (
            &["parse", "issuer", "document", SOURCE, "--format", "json"],
            success,
        ),
        (&["format", "issuer", "document", SOURCE], success),
        (
            &["format", "issuer", "document", SOURCE, "--format", "json"],
            success,
        ),
        (
            &[
                "format",
                "issuer",
                "document",
                INVALID_SOURCE,
                "--format",
                "json",
            ],
            refusal,
        ),
        (&["compile", COMPILE_REQUEST, "--format", "json"], success),
        (&["run", RUN_REQUEST, "--format", "json"], success),
        (&["run", RUN_REFUSED, "--format", "json"], refusal),
        (&["compile", COMPILE_REFUSED, "--format", "json"], refusal),
        // A refused outcome keeps its own non-zero code, distinguishing it from 30.
        (
            &[
                "parse",
                "issuer",
                "document",
                INVALID_SOURCE,
                "--format",
                "json",
            ],
            refusal,
        ),
    ];
    for (arguments, expected) in cases {
        // The reader is gone before the child exists, so its first write
        // always meets a closed pipe.
        let (reader, writer) = std::io::pipe().expect("create stdout pipe");
        drop(reader);
        let output = Command::new(FSL)
            .args(arguments)
            .stdout(Stdio::from(writer))
            .output()
            .expect("run fsl with a closed stdout reader");
        assert_eq!(output.status.code(), Some(expected), "{arguments:?}");
        assert!(
            output.stderr.is_empty(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Removes its directory when dropped, including after a failed assertion.
struct TestDirectory(std::path::PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Dispatch component: preserve the native model-based result and its process exit.
#[trace("FR-002-AC-1", "FR-002-AC-2")]
#[test]
fn model_based_test_run_matches_the_owner_result() {
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("fsl-model-based-run-{}", std::process::id())),
    );
    std::fs::create_dir_all(&directory.0).expect("request directory");
    let source = directory.0.join("unit.native");
    std::fs::write(
        &source,
        r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
predicate Square using v(x: Integer): Boolean { x * x >= 0 }
"#,
    )
    .expect("model-based source");
    let path = directory.0.join("request.json");
    let request = serde_json::json!({
        "format": "native-run/1",
        "request": {
            "models": [],
            "program": {
                "source": {
                    "file": source,
                    "authority": "test",
                    "identity": "model-based-run",
                    "document": "ModelBased",
                    "formal_revision": 1
                }
            },
            "model_based_test": {
                "clause": "Square", "seed": 7, "max_cases": 3, "max_value_bits": 8
            }
        }
    });
    std::fs::write(&path, request.to_string()).expect("model-based request");
    let negotiation = filament_driver::AnalyzeNegotiation::fsl().expect("owner negotiation");
    let library = fsl_command::command::run(&path, &negotiation).expect("owner model-based run");
    let expected: serde_json::Value =
        serde_json::from_str(&library.value.to_string()).expect("owner result document");
    let output = Command::new(FSL)
        .arg("run")
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("driver model-based run");
    assert_eq!(
        output.status.code(),
        Some(i32::from(library.exit_code)),
        "{output:?}"
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let actual: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one owner result document");
    assert_eq!(actual, expected);
    assert_eq!(actual["model_based_test"]["status"], "tested", "{actual}");
    assert_eq!(
        actual["model_based_test"]["cases"]["generated"], 3,
        "{actual}"
    );
}

/// A prepared transfer delegates execution and its typed selection refusal to
/// FSL, preserving the same outcome document and process category.
#[trace("FR-002-AC-1", "FR-002-AC-2")]
#[test]
fn refinement_transfer_run_preserves_the_owner_selection_refusal() {
    use fsl_command::command::{self, PreparedRun, RequestOverrides, RunCause, RunLimits};
    use fsl_replay::Operation;

    let directory = TestDirectory(
        std::env::temp_dir().join(format!("fsl-transfer-selection-{}", std::process::id())),
    );
    std::fs::create_dir_all(&directory.0).expect("request directory");
    let request = serde_json::json!({"format": "native-run/1", "request": {
        "models": [],
        "program": {"source": {
            "file": SOURCE, "authority": "test", "identity": "transfer-selection",
            "document": "TransferSelection", "formal_revision": 1,
        }},
        "refinement_transfer": {
            "refinement": "MissingRefinement",
            "abstract": {"model": "Spec", "initial": ["start"]},
            "concrete": {"model": "Impl", "initial": ["start"]},
            "clauses": ["MissingClause"],
            "observations": {"start": std::path::Path::new(ANALYZE_REQUEST)
                .parent().expect("fixture directory").join("analyze-counter-start.json")},
        },
    }});
    let path = directory.0.join("run.json");
    std::fs::write(&path, request.to_string()).expect("transfer request");
    let limits = RunLimits::default();
    let overrides = RequestOverrides::default();
    assert!(matches!(
        command::prepare_run_with_overrides(&path, limits, &overrides)
            .expect("the real source prepares a transfer"),
        PreparedRun::RefinementTransfer(_)
    ));
    let negotiation = filament_driver::AnalyzeNegotiation::fsl().expect("engine negotiation");
    let error = command::run_with_providers_and_limits_and_overrides(
        &path,
        limits,
        &overrides,
        &negotiation,
        filament_driver::run_engines(Vec::new()).expect("reference engine registration"),
        &fsl_analyze::provider::ProviderOptions::default(),
        &filament_exact::Cancel::new(),
    )
    .expect_err("the checked source declares no selected abstract model");
    match &error.cause {
        RunCause::TransferSelection {
            stage,
            member,
            cause,
            code,
            ..
        } => {
            assert_eq!(*stage, "select");
            assert_eq!(member, "/refinement_transfer/abstract/model");
            assert_eq!(*cause, "missing-name");
            assert_eq!(*code, fsl_foundation::Code::MissingDeclaration);
        }
        other => panic!("the native transfer selection must run: {other:?}"),
    }
    let expected = error.outcome_document(Operation::Execute);
    let output = Command::new(FSL)
        .arg("run")
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("driver transfer run");
    let actual: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one native transfer refusal document");
    assert_eq!(output.status.code(), Some(20), "{actual}");
    assert_eq!(
        output.status.code(),
        Some(i32::from(expected.category().exit_code()))
    );
    assert_eq!(
        actual,
        serde_json::to_value(expected).expect("owner document")
    );
}

#[trace("TC-1262", "FR-287-AC-9")]
#[test]
fn request_paths_are_independent_and_documents_equal_the_actual_producers() {
    use filament_driver::{execute, ExecutionEngine, ExecutionRequest, ObjectEnvironment};
    use fsl_command::command::{compile_outcome, prepare_run, PreparedRun, RunLimits};
    use fsl_replay::Operation;
    let fixtures = std::path::Path::new(RUN_REQUEST)
        .parent()
        .expect("fixture directory");
    let elsewhere = std::env::temp_dir();
    for (verb, request) in [
        ("compile", COMPILE_REQUEST),
        ("compile", COMPILE_REFUSED),
        ("run", RUN_REQUEST),
        ("run", RUN_REFUSED),
    ] {
        let document = if verb == "compile" {
            match compile_outcome(std::path::Path::new(request)) {
                Ok(outcome) => outcome.document().clone(),
                Err(error) => error.outcome_document(Operation::Package),
            }
        } else {
            let PreparedRun::Function(prepared) =
                prepare_run(std::path::Path::new(request), RunLimits::default())
                    .expect("prepare request")
            else {
                panic!("fixture selects a function");
            };
            let input = prepared.into_input();
            let result = execute(ExecutionRequest {
                source: input.source.identity().clone(),
                package_name: &input.package_name,
                path: input.source.path(),
                bytes: input.source.text().as_bytes(),
                packages: &input.packages,
                dependencies: &input.dependencies,
                limits: fsl_replay::spine::SpineLimits::default(),
                call: input.call,
                integer_division: input.integer_division,
                object_environment: &ObjectEnvironment::default(),
                cancel: filament_exact::Cancel::new(),
                engine: ExecutionEngine::Interpreter,
            });
            match result {
                Ok(result) => result.document,
                Err(error) => error.into_document(),
            }
        };
        let expected = format!(
            "{}\n",
            ix_cli_kit::json::encode_canonical(&document, false).expect("producer document")
        );
        let path = std::path::Path::new(request);
        let relative = Command::new(FSL)
            .current_dir(fixtures)
            .arg(verb)
            .arg(path.file_name().expect("request name"))
            .args(["--format", "json"])
            .output()
            .expect("relative request");
        let absolute = Command::new(FSL)
            .current_dir(&elsewhere)
            .args([verb, request, "--format", "json"])
            .output()
            .expect("absolute request");
        assert_eq!(
            relative.status.code(),
            Some(i32::from(document.category().exit_code())),
            "{request}"
        );
        assert_eq!(relative.stdout, expected.as_bytes(), "{request}");
        assert_eq!(relative.stdout, absolute.stdout, "{request}");
        assert_eq!(relative.status.code(), absolute.status.code(), "{request}");
        assert_eq!(relative.stderr, absolute.stderr, "{request}");
        let value: serde_json::Value =
            serde_json::from_slice(&relative.stdout).expect("outcome JSON");
        assert_eq!(value["format"], "filament-outcome/1");
    }
}

#[trace("TC-1262")]
#[test]
fn request_byte_options_reach_owner_limits_and_compile_accepts_none() {
    let source_bytes = std::fs::metadata(SOURCE).expect("source bytes").len();
    let request_bytes = std::fs::metadata(RUN_REQUEST).expect("request bytes").len();
    for (option, setting, size) in [
        ("--request-bytes", "request_bytes", request_bytes),
        ("--dependent-bytes", "dependent_bytes", source_bytes),
    ] {
        let too_small = (size - 1).to_string();
        let output = fsl(&["run", RUN_REQUEST, option, &too_small, "--format", "json"]);
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::Incomplete.exit_code()))
        );
        // In `--format json` mode the message is in the stdout document.
        let document = String::from_utf8_lossy(&output.stdout);
        assert!(document.contains(setting), "{document}");
        assert!(document.contains(&too_small), "{document}");
        assert!(document.contains(option), "{document}");
        let exact = size.to_string();
        let output = fsl(&["run", RUN_REQUEST, option, &exact, "--format", "json"]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let maximum = usize::MAX.to_string();
        let output = fsl(&["run", RUN_REQUEST, option, &maximum, "--format", "json"]);
        assert_eq!(output.status.code(), Some(0));
        let output = fsl(&[
            "compile",
            MISSING_SOURCE,
            option,
            &exact,
            "--format",
            "json",
        ]);
        assert_eq!(output.status.code(), Some(refusal_code()));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("unknown compile option"), "{stderr}");
        assert!(!stderr.contains("cannot read"), "{stderr}");
    }
}

#[trace("TC-1262", "FR-287-AC-9")]
#[test]
fn clause_requests_use_the_core_monitor_document_and_never_ignore_configured_providers() {
    use fsl_command::command::{prepare_run, PreparedRun, RunLimits};
    use fsl_replay::spine::ClauseRunSelection;
    use fsl_replay::{OutcomeDocument, OutcomeStage};
    let directory =
        TestDirectory(std::env::temp_dir().join(format!("filament-clause-{}", std::process::id())));
    std::fs::create_dir_all(&directory.0).expect("clause directory");
    let observation = serde_json::json!({"format":"filament.state.snapshot/v1",
        "identity":{"authority":"issuer","identity":"current"},"observation":"current"});
    std::fs::write(directory.0.join("current.json"), observation.to_string()).expect("observation");
    let request = serde_json::json!({"format":"native-run/1","request":{
        "models":[],"program":{"source":{"file":SOURCE,"authority":"issuer","identity":"document",
            "document":"CliClause","formal_revision":1}},
        "clause":{"selection":{"function":{"name":"yes","arguments":[],"snapshot":"current"}},
            "observations":{"current":"current.json"}}}});
    let request_path = directory.0.join("request.json");
    std::fs::write(&request_path, request.to_string()).expect("clause request");
    let PreparedRun::Clause(request) =
        prepare_run(&request_path, RunLimits::default()).expect("prepare actual clause")
    else {
        panic!("clause request must not become a function call");
    };
    let ClauseRunSelection::Function { ref name, .. } = request.selection else {
        panic!("function claim");
    };
    let name = name.clone();
    let report =
        fsl_replay::spine::run_clause(*request.into_request()).expect("actual clause report");
    let document =
        OutcomeDocument::from_clause_runs(Some(OutcomeStage::S6a), [(name.as_str(), &report)])
            .expect("core producer");
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request_path)
        .args(["--format", "json"])
        .output()
        .expect("run clause");
    let expected = format!(
        "{}\n",
        ix_cli_kit::json::encode_canonical(&document, false).expect("canonical monitor")
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(
        output.status.code(),
        Some(i32::from(document.category().exit_code()))
    );
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).expect("monitor output");
    assert_eq!(actual["operation"], "monitor");
    // QSL-2370: the configured provider is never ignored. An FSpec FR-561
    // document naming a plugin path that does not exist refuses naming it,
    // exit 20 with empty stdout, before any stage runs or any plugin starts
    // (FSpec FR-561-AC-2, FR-006-AC-24).
    let config_path = directory.0.join("providers.json");
    let never_start = directory.0.join("never-start-provider");
    let config = serde_json::json!({"format": "filament.provider-config/v1",
        "providers": [{"path": never_start, "arguments": []}]});
    std::fs::write(&config_path, config.to_string()).expect("provider config");
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request_path)
        .arg("--provider-config")
        .arg(&config_path)
        .args(["--provider-limit", "deadline_ms=1024"])
        .args(["--format", "json"])
        .output()
        .expect("clause run with a configured provider");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Refusal.exit_code()))
    );
    assert!(output.stdout.is_empty(), "the refusal writes no document");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid-request"), "{stderr}");
    assert!(
        stderr.contains(never_start.display().to_string().as_str()),
        "{stderr}"
    );
    assert!(stderr.contains("does not exist"), "{stderr}");
}

const ANALYZE_REQUEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/analyze-counter.json"
);

/// A `native-analyze/1` request over the Counter fixture with `items` and
/// the extra request members `extra`, written in its own folder; the files
/// it names are the fixtures, by absolute path.
struct AnalyzeRequest {
    directory: TestDirectory,
}

impl AnalyzeRequest {
    fn new(name: &str, items: serde_json::Value, extra: serde_json::Value) -> Self {
        let fixtures = std::path::Path::new(ANALYZE_REQUEST)
            .parent()
            .expect("fixture directory");
        let mut document: serde_json::Value = serde_json::from_slice(
            &std::fs::read(ANALYZE_REQUEST).expect("the Counter analyze request"),
        )
        .expect("the request is JSON");
        let request = &mut document["request"];
        request["program"]["source"]["file"] = fixtures
            .join("analyze-counter.fsl")
            .to_string_lossy()
            .into();
        request["models"][0]["source"]["file"] = fixtures
            .join("analyze-counter.semantic-ir.json")
            .to_string_lossy()
            .into();
        request["documents"][0]["file"] = fixtures
            .join("analyze-counter-start.json")
            .to_string_lossy()
            .into();
        request["items"] = items;
        for (member, value) in extra.as_object().expect("extra request members") {
            request[member] = value.clone();
        }
        let directory = TestDirectory(
            std::env::temp_dir().join(format!("fsl-analyze-{name}-{}", std::process::id())),
        );
        let _ = std::fs::remove_dir_all(&directory.0);
        std::fs::create_dir_all(&directory.0).expect("a request folder");
        std::fs::write(directory.0.join("analyze.json"), document.to_string())
            .expect("the request file");
        Self { directory }
    }

    /// Write `bytes` as `name` in the request folder and point the request
    /// member at JSON pointer `member` (a `file` member) at it.
    fn with_file(self, member: &str, name: &str, bytes: &[u8]) -> Self {
        let path = self.directory.0.join(name);
        std::fs::write(&path, bytes).expect("the replacement file");
        let mut document: serde_json::Value =
            serde_json::from_slice(&std::fs::read(self.path()).expect("the request"))
                .expect("the request is JSON");
        *document
            .pointer_mut(member)
            .unwrap_or_else(|| panic!("the request has {member}")) = path.to_string_lossy().into();
        std::fs::write(self.path(), document.to_string()).expect("rewrite the request");
        self
    }

    fn raw(name: &str, bytes: &[u8]) -> Self {
        let directory = TestDirectory(
            std::env::temp_dir().join(format!("fsl-analyze-{name}-{}", std::process::id())),
        );
        let _ = std::fs::remove_dir_all(&directory.0);
        std::fs::create_dir_all(&directory.0).expect("a request folder");
        std::fs::write(directory.0.join("analyze.json"), bytes).expect("the request file");
        Self { directory }
    }

    fn path(&self) -> std::path::PathBuf {
        self.directory.0.join("analyze.json")
    }

    fn run(&self, format: &str) -> Output {
        Command::new(FSL)
            .arg("analyze")
            .arg(self.path())
            .args(["--format", format])
            .output()
            .expect("run fsl analyze")
    }

    /// The machine-mode document and exit code.
    fn json(&self) -> (serde_json::Value, Option<i32>) {
        let output = self.run("json");
        let document = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "one outcome document ({error}); stderr {}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (document, output.status.code())
    }
}

fn one_item(claim: &str) -> serde_json::Value {
    serde_json::json!([{ "claim": claim }])
}

/// FR-022-AC-1, FSL FR-621-AC-1: a holding item prints its `proved` record
/// and exits 0.
#[trace("TC-036", "FR-022-AC-1")]
#[trace("TC-1581", "FR-621-AC-1")]
#[test]
fn analyze_holds_prints_result_and_exit_zero() {
    let request = AnalyzeRequest::new("holds", one_item("Bounded"), serde_json::json!({}));
    let (document, exit) = request.json();
    assert_eq!(exit, Some(0), "{document}");
    assert_eq!(document["operation"], "analyze");
    assert_eq!(document["category"], "success");
    assert_eq!(document["aggregate"]["status"], "complete");
    let item = &document["items"][0];
    assert_eq!(item["claim_name"], "Bounded");
    assert_eq!(item["record"]["result"], "proved", "{item}");
    assert_eq!(item["record"]["basis"]["type"], "exhaustive", "{item}");
    assert!(item.get("counterexample").is_none(), "{item}");
}

/// The checked finite-hyper clause reaches EN-1 with its original population
/// context and bound; negotiation does not replace them with proof bounds.
#[trace("TC-036", "FR-022-AC-8")]
#[test]
fn analyze_finite_hyper_preserves_population_results_and_declared_bound() {
    use serde_json::json;

    let source = include_str!("fixtures/analyze-counter.fsl").replace(
        "model M = \"test/counter\";",
        "profile ep = \"filament.temporal.event-position.false-extension/v1\";\n\
         model M = \"test/counter\";",
    ) + "\nrecord Obs { inp: Int[0, 3]; res: Int[0, 3]; }\n\
         hyper Det using ep over t in Obs bounded 3 \
         forall trace a forall trace b { (a.inp = b.inp) implies (a.res = b.res) }\n";
    let trace =
        |identity, result| json!({"identity": identity, "states": [{"inp": 0, "res": result}]});
    let fixtures = std::path::Path::new(ANALYZE_REQUEST)
        .parent()
        .expect("fixture directory");
    for (name, closure, traces, result, expected_exit) in [
        (
            "closed",
            "closed",
            json!([trace("a", 0), trace("b", 0)]),
            "proved",
            Some(0),
        ),
        (
            "refuted",
            "closed",
            json!([trace("a", 0), trace("b", 1)]),
            "refuted",
            Some(10),
        ),
        (
            "open",
            "open",
            json!([trace("a", 0), trace("b", 0)]),
            "inconclusive",
            Some(11),
        ),
        (
            "bound",
            "closed",
            json!([trace("a", 0), trace("b", 0), trace("c", 0), trace("d", 0)]),
            "requires-bound",
            None,
        ),
    ] {
        let population = json!({
            "format": "filament.trace-population/1",
            "record_type": "Obs", "closure": closure, "traces": traces,
        });
        let request = AnalyzeRequest::new(
            &format!("finite-hyper-{name}"),
            json!([{
                "claim": "Det", "kind": "temporal-satisfaction", "backend": "en-1",
                "traces": {"authority": "test", "identity": "population"},
            }]),
            json!({
                "clocks": {"t": {"kind": "event_position", "sequence_authority": "obs"}},
                "documents": [
                    {"file": fixtures.join("analyze-counter-start.json")},
                    {"file": "population.json", "identity": {"authority": "test", "identity": "population"}},
                ],
            }),
        )
        .with_file("/request/program/source/file", "finite-hyper.fsl", source.as_bytes())
        .with_file("/request/documents/1/file", "population.json", population.to_string().as_bytes());
        let (document, exit) = request.json();
        let items = document["items"].as_array().expect("one resolved item");
        assert_eq!(items.len(), 1, "{name}: {document}");
        let item = &items[0];
        assert_eq!(item["claim_name"], "Det", "{name}: {document}");
        if result == "requires-bound" {
            assert_ne!(exit, Some(0), "{document}");
            assert_eq!(item["record"]["settled_by"], "disposition", "{document}");
            let diagnostic = &item["record"]["diagnostic"];
            assert_eq!(diagnostic["code"], "requires_bound", "{document}");
            assert_eq!(diagnostic["cause"], "requires-bound", "{document}");
            assert_eq!(diagnostic["detail"]["bound"], 3, "{document}");
            assert_eq!(diagnostic["detail"]["found"], 4, "{document}");
        } else {
            assert_eq!(item["record"]["backend"], "en-1", "{name}: {document}");
            assert_eq!(item["record"]["result"], result, "{name}: {document}");
            assert_eq!(exit, expected_exit, "{name}: {document}");
            if result == "refuted" {
                assert_eq!(item["replay"]["reproduced"], true, "{document}");
                assert_eq!(
                    item["replay"]["target"]["hyperproperty"], "Det",
                    "{document}"
                );
                assert!(
                    item["counterexample"]["false_tuples"]
                        .as_array()
                        .is_some_and(|tuples| tuples.iter().any(|tuple| {
                            tuple["traces"] == json!(["a", "b"]) && tuple["position"] == 0
                        })),
                    "{document}"
                );
            }
        }
    }
}

/// FR-022-AC-2, FSL FR-621-AC-2 and AC-8: a refuted item carries a
/// counterexample and the replay target that reproduced it, and exits 10.
#[trace("TC-036", "FR-022-AC-2")]
#[trace("TC-1581", "FR-621-AC-2")]
#[trace("TC-2184", "FR-621-AC-8")]
#[test]
fn analyze_refuted_prints_counterexample() {
    let request = AnalyzeRequest::new("refuted", one_item("BelowThree"), serde_json::json!({}));
    let (document, exit) = request.json();
    assert_eq!(exit, Some(10), "{document}");
    assert_eq!(document["category"], "violation");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "refuted", "{item}");
    assert!(item["counterexample"].is_object(), "{item}");
    assert!(
        item["counterexample"]
            .to_string()
            .contains(r#"{"type":"integer","value":"3"}"#),
        "the counterexample reaches value 3: {item}"
    );
    assert_eq!(item["replay"]["reproduced"], true, "{item}");
    let target = item["replay"]["target"]["state_clause"]
        .as_str()
        .unwrap_or_else(|| panic!("a state-clause replay target: {item}"));
    assert!(target.ends_with("BelowThree"), "{target}");
}

/// FR-022-AC-3, FSL FR-621-AC-3 and FR-620-AC-6: an exhausted state budget
/// and a passed deadline each settle `incomplete`, exit 22.
#[trace("TC-036", "FR-022-AC-3")]
#[trace("TC-1581", "FR-621-AC-3")]
#[trace("TC-1580", "FR-620-AC-6")]
#[test]
fn analyze_budget_exhausted_is_incomplete() {
    let limited = AnalyzeRequest::new(
        "limit",
        serde_json::json!([{"claim": "Bounded", "temporal": {"limits": {"max_states": 1}}}]),
        serde_json::json!({}),
    );
    let (document, exit) = limited.json();
    assert_eq!(exit, Some(22), "{document}");
    assert_eq!(document["category"], "incomplete");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "incomplete", "{item}");
    assert!(
        item["record"]["cause"].to_string().contains("max_states"),
        "{item}"
    );
    assert!(item.get("counterexample").is_none(), "{item}");
    assert_eq!(document["aggregate"]["status"], "incomplete");

    let timed_out = AnalyzeRequest::new(
        "deadline",
        one_item("Bounded"),
        serde_json::json!({"deadline_ms": 0}),
    );
    let (document, exit) = timed_out.json();
    assert_eq!(exit, Some(22), "{document}");
    assert_eq!(document["category"], "incomplete");
    let record = &document["items"][0]["record"];
    assert_eq!(record["result"], "incomplete", "{document}");
    assert_eq!(
        record["cause"],
        serde_json::json!({"kind": "cancelled", "source": "deadline"}),
        "{document}"
    );
}

/// FR-022-AC-5, FSL FR-624-AC-2: a program the checker refuses refuses the
/// whole request at the checker stage, which FSL names `S3`, in both modes.
#[trace("TC-036", "FR-022-AC-5")]
#[trace("TC-1584", "FR-624-AC-2")]
#[test]
fn analyze_checker_refusal_refuses_at_s3() {
    let request = AnalyzeRequest::new(
        "check-refusal",
        serde_json::json!([]),
        serde_json::json!({}),
    )
    .with_file(
        "/request/program/source/file",
        "program.fsl",
        b"language \"ix:native\" edition \"1-draft\";\n\
              profile v = \"filament.value.complete/v1\";\n\
              model M = \"test/counter\";\n\
              function Bad using v(): Boolean pure { 1 }\n",
    );
    let (document, exit) = request.json();
    assert_eq!(exit, Some(20), "{document}");
    assert_eq!(document["operation"], "analyze");
    assert_eq!(document["category"], "refusal");
    // FSL names the checker stage `S3` in `last_stage`.
    assert_eq!(document["last_stage"], "S3", "{document}");
    assert_eq!(document["items"], serde_json::json!([]));
    let code = document["diagnostics"][0]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("a coded diagnostic: {document}"))
        .to_owned();
    assert_ne!(code, "invalid-request", "the request itself is well formed");
    let text = request.run("text");
    assert_eq!(text.status.code(), Some(20));
    assert!(text.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&text.stderr);
    assert!(
        stderr.starts_with(&format!("fsl analyze: {code}: ")),
        "{stderr}"
    );
}

/// Provenance: QSL-2153.
/// FR-723-AC-1..3: the CLI retains the front end's timed-model refusal
/// unchanged when a probabilistic timed-profile claim uses an untimed model.
#[trace("TC-1765", "FR-723-AC-1")]
#[trace("TC-1765", "FR-723-AC-2")]
#[trace("TC-1765", "FR-723-AC-3")]
#[test]
fn timed_event_profile_cause_removed() {
    let request = AnalyzeRequest::new(
        "timed-profile-untimed-model",
        one_item("Bounded"),
        serde_json::json!({}),
    )
    .with_file(
        "/request/program/source/file",
        "program.fsl",
        b"language \"ix:native\" edition \"1-draft\";\n\
          profile v = \"filament.value.complete/v1\";\n\
          profile timed = \"filament.temporal.timed/v1\";\n\
          model M = \"test/counter\";\n\
          probabilistic Bounded using timed on M over (c: M::Counter) under every scheduler on origin {\n\
            probability >= 0.9 [ eventually holds(c.value <= 3) ]\n\
          }\n",
    );
    let frontend_error = fsl_command::command::analyze_with_providers(
        &request.path(),
        &filament_driver::AnalyzeNegotiation::fsl().expect("the engines register"),
        filament_driver::run_engines(Vec::new()).expect("the hybrid engine registers"),
        &fsl_analyze::provider::ProviderOptions::default(),
        &filament_exact::Cancel::new(),
    )
    .expect_err("the frontend refuses model time over an untimed model");
    let frontend_document =
        serde_json::to_value(frontend_error.outcome_document(fsl_replay::Operation::Analyze))
            .expect("serialize the frontend refusal document");
    let (document, exit) = request.json();
    assert_eq!(exit, Some(20), "{document}");
    assert_eq!(document["operation"], "analyze");
    assert_eq!(document["category"], "refusal");
    assert_eq!(document["last_stage"], "S3", "{document}");
    assert_eq!(document["items"], serde_json::json!([]), "{document}");

    let diagnostics = document["diagnostics"]
        .as_array()
        .unwrap_or_else(|| panic!("diagnostic array: {document}"));
    assert_eq!(
        diagnostics.len(),
        1,
        "the driver adds no refusal causes: {document}"
    );
    let diagnostic = &diagnostics[0];
    let frontend_diagnostic = &frontend_document["diagnostics"][0];
    assert_eq!(
        document, frontend_document,
        "the CLI returns the frontend document unchanged"
    );
    assert_eq!(diagnostic["code"], "invalid_timed_model", "{diagnostic}");
    assert_eq!(diagnostic["code"], frontend_diagnostic["code"]);
    assert_eq!(
        diagnostic["cause"], "model-time-on-untimed-model",
        "{diagnostic}"
    );
    assert_eq!(diagnostic["cause"], frontend_diagnostic["cause"]);
    assert!(
        diagnostic["message"]
            .as_str()
            .is_some_and(|message| message.contains("`M`")),
        "the frontend refusal payload names the model: {diagnostic}"
    );
    assert_eq!(diagnostic["message"], frontend_diagnostic["message"]);
    assert_eq!(diagnostic["locus"]["kind"], "region", "{diagnostic}");
    assert_eq!(
        diagnostic["locus"]["source"],
        serde_json::json!({"authority": "test", "identity": "analyze-counter"}),
        "the frontend source locus is retained: {diagnostic}"
    );
    assert_eq!(diagnostic["locus"], frontend_diagnostic["locus"]);
    assert!(
        !document.to_string().contains("timed-event-profile"),
        "the driver does not add the old cause: {document}"
    );
}

/// FR-022-AC-5, FSL FR-624-AC-3: a snapshot whose population the model
/// does not declare refuses the whole request at the subject binding.
#[trace("TC-036", "FR-022-AC-5")]
#[trace("TC-1584", "FR-624-AC-3")]
#[test]
fn analyze_binding_refusal_refuses_the_request() {
    let snapshot = serde_json::json!({
        "format": "filament.state.snapshot/v1",
        "identity": {"authority": "test", "identity": "start"},
        "observation": "current", "model": {"identity": "test/counter"},
        "anchor": {"kind": "initialization", "name": "start"},
        "populations": [{"population": "ix://test/counter/nowhere", "complete": true,
            "objects": []}],
    });
    let request = AnalyzeRequest::new(
        "binding-refusal",
        one_item("Bounded"),
        serde_json::json!({}),
    )
    .with_file(
        "/request/documents/0/file",
        "start.json",
        snapshot.to_string().as_bytes(),
    );
    let (document, exit) = request.json();
    assert_eq!(exit, Some(20), "{document}");
    assert_eq!(document["category"], "refusal");
    assert_eq!(document["items"], serde_json::json!([]));
    let code = &document["diagnostics"][0]["code"];
    assert!(code.is_string(), "{document}");
    assert_ne!(code, "invalid-request", "the request is well formed");
}

/// FR-022-AC-4: an unknown backend is an `invalid-request` item and a kind
/// no engine serves is an `unsupported` item; neither is empty output.
#[trace("TC-036", "FR-022-AC-4")]
#[trace("TC-1595", "FR-635-AC-3")]
#[test]
fn analyze_unsupported_item_is_typed() {
    let unknown = AnalyzeRequest::new(
        "unknown-backend",
        serde_json::json!([{"claim": "Bounded", "backend": "no-such-backend"}]),
        serde_json::json!({}),
    );
    let (document, exit) = unknown.json();
    assert_eq!(exit, Some(20), "{document}");
    assert_eq!(document["items"][0]["category"], "refusal");
    let diagnostic = &document["items"][0]["record"]["diagnostic"];
    assert_eq!(diagnostic["code"], "invalid_capability", "{document}");
    assert_eq!(diagnostic["cause"], "unknown-backend", "{document}");
    assert!(
        diagnostic.to_string().contains("no-such-backend"),
        "{diagnostic}"
    );

    // EN-6 lists no temporal-satisfaction form, so the item no registered
    // engine serves settles `unsupported` (QSL-2122 made EN-1 serve the
    // derived deadlock-freedom item this case used before).
    let unserved = AnalyzeRequest::new(
        "unserved",
        serde_json::json!([{"claim": "Bounded", "backend": "en-6"}]),
        serde_json::json!({}),
    );
    let output = unserved.run("json");
    assert!(!output.stdout.is_empty(), "never empty output");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(output.status.code(), Some(21), "{document}");
    assert_eq!(document["category"], "unsupported");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "unsupported", "{item}");
    assert_eq!(
        item["record"]["diagnostic"]["code"], "unsupported_projection",
        "{item}"
    );
    assert_eq!(
        item["record"]["diagnostic"]["cause"], "unsupported-requested-capability",
        "{item}"
    );
    assert_eq!(item["request"]["temporal"]["item_kind"], "formula");
}

/// FR-022-AC-8: the registry FSL builds from its engine set is the one the
/// driver negotiates over, so the outcome lists FSL's registered engines.
/// Each registry entry is FSL FR-635's `{backend, tool}` object.
#[trace("TC-036", "FR-022-AC-8")]
#[test]
fn analyze_routes_through_engines_registry() {
    let request = AnalyzeRequest::new("registry", one_item("Bounded"), serde_json::json!({}));
    let (document, _) = request.json();
    let backends: Vec<&str> = document["registry"]
        .as_array()
        .unwrap_or_else(|| panic!("a registry member: {document}"))
        .iter()
        .filter_map(|entry| entry["backend"].as_str())
        .collect();
    assert!(backends.contains(&"en-1"), "{backends:?}");
    assert_eq!(
        document["items"][0]["request"]["candidates"]["backends"],
        serde_json::json!(["en-1"]),
        "{document}"
    );
}

/// FR-022-AC-5, FSL FR-624-AC-1: a malformed request is a typed refusal in
/// both modes; usage errors refuse before any file is read.
#[trace("TC-036", "FR-022-AC-5")]
#[trace("TC-1584", "FR-624-AC-1")]
#[test]
fn analyze_malformed_request_is_typed_refusal() {
    let request = AnalyzeRequest::raw(
        "malformed",
        br#"{"format": "native-analyze/1", "request": 7}"#,
    );
    let (document, exit) = request.json();
    assert_eq!(exit, Some(20), "{document}");
    assert_eq!(document["operation"], "analyze");
    assert_eq!(document["category"], "refusal");
    assert_eq!(document["items"], serde_json::json!([]));
    assert_eq!(document["diagnostics"][0]["code"], "invalid-request");

    let text = request.run("text");
    assert_eq!(text.status.code(), Some(20));
    assert!(text.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&text.stderr);
    assert!(
        stderr.starts_with("fsl analyze: invalid-request:"),
        "{stderr}"
    );

    for arguments in [
        &["analyze"][..],
        &["analyze", "missing.json", "extra"][..],
        &["analyze", "missing.json", "--engine", "aot"][..],
    ] {
        let output = fsl(arguments);
        assert_eq!(output.status.code(), Some(20), "{arguments:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("usage: fsl analyze") || stderr.contains("unknown analyze option"),
            "{arguments:?}: {stderr}"
        );
        assert!(
            !stderr.contains("could not read"),
            "{arguments:?}: {stderr}"
        );
    }
    // `--provider-config` takes one path, once, before any file is read.
    for arguments in [
        &["analyze", "missing.json", "--provider-config"][..],
        &[
            "analyze",
            "missing.json",
            "--provider-config",
            "a.json",
            "--provider-config",
            "b.json",
        ][..],
    ] {
        let output = fsl(arguments);
        assert_eq!(output.status.code(), Some(20), "{arguments:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("--provider-config"),
            "{arguments:?}: {stderr}"
        );
    }
}

const PROVIDER_BACKEND: &str = "fsl.test-process-provider";

/// The Counter program with the value-validity function `Even` added.
fn even_program() -> Vec<u8> {
    let fixtures = std::path::Path::new(ANALYZE_REQUEST)
        .parent()
        .expect("fixture directory");
    let mut program =
        std::fs::read_to_string(fixtures.join("analyze-counter.fsl")).expect("the Counter program");
    program.push_str("\nfunction Even using v(n: Integer): Boolean pure { n * 2 - n = n }\n");
    program.into_bytes()
}

/// A `--provider-config` file in `request`'s folder starting the test
/// provider with `arguments`.
fn provider_config(request: &AnalyzeRequest, arguments: &[&str]) -> std::path::PathBuf {
    let path = request.directory.0.join("providers.json");
    let config = serde_json::json!({"format": "filament.provider-config/v1",
        "providers": [{"path": test_provider_path::executable(), "arguments": arguments}]});
    std::fs::write(&path, config.to_string()).expect("the provider configuration");
    path
}

fn analyze_with_config(request: &AnalyzeRequest, config: &std::path::Path) -> Output {
    Command::new(FSL)
        .arg("analyze")
        .arg(request.path())
        .arg("--provider-config")
        .arg(config)
        .args(["--format", "json"])
        .output()
        .expect("run fsl analyze --provider-config")
}

/// FSL FR-287, FR-635: a `--provider-config` provider registers beside FSL's
/// engines before negotiation; an item naming it is sent to the started
/// process as one FR-331 request and settles `proved` with certification
/// `trusted`; the registry lists it; and an item that does not negotiate to
/// it is byte-equal to the run without it. QSL-2283.
#[trace("TC-036", "FR-022-AC-9")]
#[test]
fn analyze_provider_config_settles_an_item_on_the_process_provider() {
    let items = serde_json::json!([
        {"claim": "Even", "kind": "value-validity", "backend": PROVIDER_BACKEND},
        {"claim": "Bounded"}]);
    let request = AnalyzeRequest::new("provider-proved", items, serde_json::json!({})).with_file(
        "/request/program/source/file",
        "even.fsl",
        &even_program(),
    );
    let log = request.directory.0.join("request.json");
    let log_text = log.to_string_lossy().into_owned();
    let config = provider_config(
        &request,
        &[PROVIDER_BACKEND, "unbounded", "proved", &log_text],
    );
    let output = analyze_with_config(&request, &config);
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "a document; stderr {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "proved", "{document}");
    assert_eq!(item["record"]["certification"], "trusted", "{document}");
    let registry: Vec<&str> = document["registry"]
        .as_array()
        .expect("registry")
        .iter()
        .filter_map(|entry| entry["backend"].as_str())
        .collect();
    assert!(registry.contains(&PROVIDER_BACKEND), "{registry:?}");
    assert!(registry.contains(&"en-1"), "{registry:?}");
    let sent: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&log).expect("the provider saw a request"))
            .expect("the request body");
    assert_eq!(sent["request"]["items"][0]["request_index"], 0, "{sent}");
    assert_eq!(
        sent["request"]["items"][0]["backend"], PROVIDER_BACKEND,
        "{sent}"
    );
    let (without, _) = request.json();
    assert_eq!(document["items"][1], without["items"][1]);
}

/// FSL FR-281: a provider result that is not a verdict settles the item
/// `inconclusive`, never `proved`. QSL-2283.
#[trace("TC-036", "FR-022-AC-9")]
#[test]
fn analyze_provider_without_a_verdict_is_inconclusive() {
    let items = serde_json::json!([
        {"claim": "Even", "kind": "value-validity", "backend": PROVIDER_BACKEND}]);
    let request = AnalyzeRequest::new("provider-unknown", items, serde_json::json!({})).with_file(
        "/request/program/source/file",
        "even.fsl",
        &even_program(),
    );
    let config = provider_config(&request, &[PROVIDER_BACKEND, "unbounded", "failed"]);
    let output = analyze_with_config(&request, &config);
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("a document");
    assert_eq!(
        document["items"][0]["record"]["result"], "inconclusive",
        "{document}"
    );
}

#[cfg(unix)]
fn assert_provider_resource_breach(mode: &str, limit: &str, defect: &str) {
    let items = serde_json::json!([
        {"claim": "Even", "kind": "value-validity", "backend": PROVIDER_BACKEND},
        {"claim": "Bounded"}]);
    let request = AnalyzeRequest::new(mode, items, serde_json::json!({})).with_file(
        "/request/program/source/file",
        "even.fsl",
        &even_program(),
    );
    let config = provider_config(&request, &[PROVIDER_BACKEND, "unbounded", mode]);
    let child = Command::new(FSL)
        .arg("analyze")
        .arg(request.path())
        .arg("--provider-config")
        .arg(&config)
        .args(["--provider-limit", limit, "--format", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start process provider under resource limits");
    let pid = child.id();
    let (finished, waiting) = std::sync::mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        if waiting.recv_timeout(std::time::Duration::from_secs(20))
            == Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        {
            // The CLI handles TERM by cancelling and reaping its active
            // provider. A missing limit must fail this test without leaving
            // a spinning child behind.
            let _ = Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status();
        }
    });
    let output = child.wait_with_output();
    let _ = finished.send(());
    watchdog.join().expect("resource test watchdog finishes");
    let output = output.expect("run process provider under resource limits");
    let document: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| panic!("provider document; output {output:?}"));
    assert_eq!(output.status.code(), Some(30), "{document}");
    #[cfg(target_os = "macos")]
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("max_memory_bytes is not enforced on this platform"),
        "the CPU breach must retain the default-memory notice: {output:?}"
    );
    let record = &document["items"][0]["record"];
    assert_eq!(record["result"], "failed", "{document}");
    assert_eq!(
        record["cause"],
        serde_json::json!({"kind": "tool-failure", "defect": defect}),
        "{document}"
    );
    let (without, _) = request.json();
    assert_eq!(
        document["items"][1], without["items"][1],
        "unrelated item changed"
    );
}

#[trace("FR-006-AC-19", "FR-006-AC-23")]
#[cfg(target_os = "linux")]
#[test]
fn provider_resource_limits_memory_breach_settles_failed() {
    assert_provider_resource_breach("memory-breach", "max_memory_bytes=67108864", "nonzero-exit");
}

#[trace("FR-006-AC-19", "FR-006-AC-23")]
#[cfg(unix)]
#[test]
fn provider_resource_limits_cpu_breach_settles_failed() {
    assert_provider_resource_breach("cpu-breach", "max_cpu_ms=1", "crash");
}

#[trace("FR-006-AC-19", "FR-006-AC-23")]
#[cfg(target_os = "linux")]
#[test]
fn provider_resource_limits_partial_response_breach_settles_failed() {
    assert_provider_resource_breach(
        "memory-breach-partial",
        "max_memory_bytes=67108864",
        "nonzero-exit",
    );
}

#[trace("FR-013-AC-18")]
#[cfg(target_os = "macos")]
#[test]
fn provider_resource_limits_explicit_memory_refuses_before_launch_on_macos() {
    let request = AnalyzeRequest::new(
        "memory-platform-refusal",
        serde_json::json!([{"claim":"Bounded"}]),
        serde_json::json!({}),
    );
    let marker = request.directory.0.join("provider-started");
    let config = request.directory.0.join("providers.json");
    std::fs::write(
        &config,
        serde_json::json!({"format":"filament.provider-config/v1","providers":[{
            "path":"/usr/bin/touch", "arguments":[marker]
        }]})
        .to_string(),
    )
    .expect("provider launch marker configuration");
    for memory in ["max_memory_bytes=4294967296", "max_memory_bytes=67108864"] {
        for format in ["json", "text"] {
            let output = Command::new(FSL)
                .arg("analyze")
                .arg(request.path())
                .arg("--provider-config")
                .arg(&config)
                .args(["--provider-limit", memory, "--format", format])
                .output()
                .expect("run explicit unsupported memory limit");
            assert_eq!(
                output.status.code(),
                Some(i32::from(Category::Unsupported.exit_code())),
                "{output:?}"
            );
            assert!(
                output.stdout.is_empty(),
                "the pre-launch host refusal must not invent a FSL document"
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("unsupported-on-platform")
                    && stderr.contains("max_memory_bytes")
                    && stderr.contains("macos"),
                "{stderr}"
            );
            assert!(
                !stderr.contains("invalid-request"),
                "a platform limit is not an invalid value: {stderr}"
            );
            assert!(
                !marker.exists(),
                "an unsupported explicit limit must refuse before launching a provider"
            );
        }
    }
}

#[trace("FR-013-AC-18", "FR-022-AC-9")]
#[cfg(target_os = "macos")]
#[test]
fn provider_resource_limits_default_memory_reports_a_notice_and_keeps_machine_output() {
    let request = AnalyzeRequest::new(
        "memory-platform-default",
        serde_json::json!([
            {"claim":"Even","kind":"value-validity","backend":PROVIDER_BACKEND}
        ]),
        serde_json::json!({}),
    )
    .with_file("/request/program/source/file", "even.fsl", &even_program());
    let config = provider_config(&request, &[PROVIDER_BACKEND, "unbounded", "proved"]);
    let output = analyze_with_config(&request, &config);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("the producer-owned machine outcome");
    assert_eq!(document["format"], "filament-outcome/1");
    assert_eq!(document["items"][0]["record"]["result"], "proved");
    assert_eq!(document["items"][0]["record"]["certification"], "trusted");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr
            .matches("max_memory_bytes is not enforced on this platform")
            .count(),
        1,
        "{stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("memory_enforcement"),
        "platform diagnostics must stay outside the FSL document"
    );
}

/// FSL FR-624: SIGTERM or SIGINT during `fsl analyze` cancels the run with
/// cause `requested`: the open item settles `incomplete` with
/// `{"kind": "cancelled", "source": "requested"}`, the document is written,
/// and the process exits 22 rather than by the signal. The test provider
/// sends the signal while its request is open. QSL-2284.
#[trace("TC-036", "FR-022-AC-10")]
#[test]
fn analyze_signal_cancels_with_cause_requested() {
    for signal in ["term", "int"] {
        let items = serde_json::json!([
            {"claim": "Even", "kind": "value-validity", "backend": PROVIDER_BACKEND},
            {"claim": "Bounded"}]);
        let request =
            AnalyzeRequest::new(&format!("signal-{signal}"), items, serde_json::json!({}))
                .with_file("/request/program/source/file", "even.fsl", &even_program());
        let config = provider_config(&request, &[PROVIDER_BACKEND, "unbounded", signal]);
        let output = analyze_with_config(&request, &config);
        assert_eq!(output.status.code(), Some(22), "{signal}: {output:?}");
        let document: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("a document");
        for index in [0, 1] {
            let record = &document["items"][index]["record"];
            assert_eq!(record["result"], "incomplete", "{signal}: {document}");
            assert_eq!(
                record["cause"],
                serde_json::json!({"kind": "cancelled", "source": "requested"}),
                "{signal}: {document}"
            );
        }
    }
}

/// FSL FR-635-AC-4: a provider whose `hello` does not advertise
/// (`value-validity`, `unbounded`) is refused at registration, exit 20,
/// naming its backend, and no item runs. QSL-2283.
#[trace("TC-036", "FR-022-AC-9")]
#[test]
fn analyze_provider_config_refuses_a_misadvertising_provider() {
    let request = AnalyzeRequest::new(
        "provider-bounded",
        one_item("Bounded"),
        serde_json::json!({}),
    );
    let config = provider_config(&request, &[PROVIDER_BACKEND, "bounded", "proved"]);
    let output = analyze_with_config(&request, &config);
    assert_eq!(output.status.code(), Some(20));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains(PROVIDER_BACKEND) || stderr.contains(PROVIDER_BACKEND),
        "{stdout} {stderr}"
    );
}

/// FSL FR-287, FSpec FR-561: an unreadable or malformed `--provider-config`
/// refuses with exit 20 and empty stdout, naming the file, and starts no
/// provider. QSL-2283.
#[trace("TC-036", "FR-022-AC-9")]
#[test]
fn analyze_provider_config_refuses_an_invalid_file() {
    let request = AnalyzeRequest::new(
        "provider-invalid",
        one_item("Bounded"),
        serde_json::json!({}),
    );
    let missing = request.directory.0.join("absent.json");
    let malformed = request.directory.0.join("malformed.json");
    std::fs::write(&malformed, b"{\"providers\": 3}").expect("malformed configuration");
    for config in [missing, malformed] {
        let output = analyze_with_config(&request, &config);
        assert_eq!(output.status.code(), Some(20), "{}", config.display());
        assert!(output.stdout.is_empty(), "{}", config.display());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("provider configuration"), "{stderr}");
        assert!(
            stderr.contains(config.display().to_string().as_str()),
            "{stderr}"
        );
    }
}

/// FSL FR-287, FR-964: `--provider-limit deadline_ms` reaches the FR-331
/// request FSL sends the provider, and a malformed limit refuses naming
/// the option and argument, exit 20 with empty stdout, before any provider
/// starts. QSL-2192.
#[trace("TC-036", "FR-022-AC-9")]
#[test]
fn analyze_provider_limit_sets_the_request_deadline() {
    let items = serde_json::json!([
        {"claim": "Even", "kind": "value-validity", "backend": PROVIDER_BACKEND}]);
    let request = AnalyzeRequest::new("provider-limit", items, serde_json::json!({})).with_file(
        "/request/program/source/file",
        "even.fsl",
        &even_program(),
    );
    let log = request.directory.0.join("request.json");
    let log_text = log.to_string_lossy().into_owned();
    let config = provider_config(
        &request,
        &[PROVIDER_BACKEND, "unbounded", "proved", &log_text],
    );
    for (limits, deadline) in [(&[][..], 600_000), (&["deadline_ms=250"][..], 250)] {
        let mut command = Command::new(FSL);
        command
            .arg("analyze")
            .arg(request.path())
            .arg("--provider-config")
            .arg(&config);
        for limit in limits {
            command.args(["--provider-limit", limit]);
        }
        let output = command
            .args(["--format", "json"])
            .output()
            .expect("run fsl analyze");
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let sent: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&log).expect("the provider saw a request"))
                .expect("the request body");
        assert_eq!(sent["request"]["deadline_ms"], deadline, "{sent}");
    }
    let _ = std::fs::remove_file(&log);
    for format in ["json", "text"] {
        let output = Command::new(FSL)
            .arg("analyze")
            .arg(request.path())
            .arg("--provider-config")
            .arg(&config)
            .args(["--provider-limit", "deadline_ms=10s", "--format", format])
            .output()
            .expect("run fsl analyze");
        assert_eq!(output.status.code(), Some(20), "{format}");
        assert!(output.stdout.is_empty(), "{format}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("--provider-limit `deadline_ms=10s`"),
            "{stderr}"
        );
        assert!(!log.exists(), "no provider started");
    }
}

/// FR-022-AC-6, FSL FR-621-AC-5: the binary's machine-mode document is the
/// library's, byte for byte.
#[trace("TC-036", "FR-022-AC-6")]
#[trace("TC-1581", "FR-621-AC-5")]
#[test]
fn analyze_document_equals_the_library_document() {
    let request = AnalyzeRequest::new("parity", one_item("BelowThree"), serde_json::json!({}));
    let library = fsl_command::command::analyze_with_providers(
        &request.path(),
        &filament_driver::AnalyzeNegotiation::fsl().expect("the engines register"),
        filament_driver::run_engines(Vec::new()).expect("the hybrid engine registers"),
        &fsl_analyze::provider::ProviderOptions::default(),
        &filament_exact::Cancel::new(),
    )
    .unwrap_or_else(|_| panic!("the request is accepted"));
    let expected = format!(
        "{}\n",
        ix_cli_kit::json::encode_canonical(library.value.as_value(), false)
            .expect("canonical document")
    );
    let output = request.run("json");
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    assert_eq!(output.status.code(), Some(i32::from(library.exit_code)));
}

/// FR-022-AC-7, FSL FR-621-AC-6: text mode prints one line per item, then
/// the category, with the machine-mode exit code.
#[trace("TC-036", "FR-022-AC-7")]
#[trace("TC-1581", "FR-621-AC-6")]
#[test]
fn analyze_text_mode_prints_item_lines() {
    let request = AnalyzeRequest::new("text", one_item("BelowThree"), serde_json::json!({}));
    let output = request.run("text");
    assert_eq!(output.status.code(), Some(10));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines
            .first()
            .is_some_and(|line| line.starts_with("0 BelowThree refuted")),
        "{stdout}"
    );
    assert_eq!(lines.last(), Some(&"violation"), "{stdout}");

    // An `invalid-request` disposition record is labelled `invalid-request`.
    let refused = AnalyzeRequest::new(
        "text-invalid",
        serde_json::json!([{"claim": "Bounded", "backend": "no-such-backend"}]),
        serde_json::json!({}),
    );
    let output = refused.run("text");
    assert_eq!(output.status.code(), Some(20));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .lines()
            .next()
            .expect("item line")
            .starts_with("0 Bounded invalid-request invalid_capability/unknown-backend"),
        "{stdout}"
    );
    let (machine, _) = refused.json();
    let extent =
        ix_cli_kit::json::encode_canonical(&machine["items"][0]["request"]["extent"], false)
            .expect("canonical extent");
    assert!(
        stdout
            .lines()
            .next()
            .expect("item line")
            .contains(&format!("extent={extent}")),
        "{stdout}"
    );
}

/// Run a function of the driver’s sample-reading program through the CLI.
fn run_value(function: &str, arguments: serde_json::Value) -> serde_json::Value {
    let directory = tempfile::tempdir().expect("a request folder");
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/run-sample.fsl");
    let request = serde_json::json!({"format": "native-run/1", "request": {
        "models": [],
        "program": {"source": {"file": source, "authority": "issuer",
            "identity": "document", "document": "Sample", "formal_revision": 1}},
        "call": {"function": function, "arguments": arguments},
    }});
    let path = directory.path().join("run.json");
    std::fs::write(&path, request.to_string()).expect("the request file");
    let output = fsl(&[
        "run",
        path.to_str().expect("UTF-8 path"),
        "--format",
        "json",
    ]);
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(output.status.code(), Some(0), "{function}: {document}");
    assert_eq!(document["category"], "success", "{function}: {document}");
    document["result"].clone()
}

fn assert_run_value(function: &str, argument: serde_json::Value, expected: serde_json::Value) {
    assert_eq!(
        run_value(
            function,
            serde_json::json!([{"parameter": "input", "value": argument}])
        ),
        serde_json::json!({"kind": "completed", "value": expected}),
        "{function}"
    );
}

fn sample_field(sample: &serde_json::Value, name: &str) -> serde_json::Value {
    sample["fields"]
        .as_array()
        .expect("canonical record fields")
        .iter()
        .find(|field| field["name"] == name)
        .expect("the sample field")["value"]
        .clone()
}

/// Use the public writer's declaration names, as the owner's run-value spelling tests do.
fn sample() -> serde_json::Value {
    let result = run_value(
        "fixtureSample",
        serde_json::json!([
            {"parameter":"ratio", "value":{"type":"rational", "numerator":"4", "denominator":"6"}},
            {"parameter":"price", "value":{"type":"decimal", "coefficient":"150", "scale":"2"}},
            {"parameter":"gain", "value":{"type":"float64", "bits":"3ff0000000000000"}},
            {"parameter":"code", "value":{"type":"string", "value":"abc"}},
            {"parameter":"label", "value":{"type":"text", "value":"e\u{0301}"}}
        ]),
    );
    assert_eq!(result["kind"], "completed", "{result}");
    let mut sample = result["value"].clone();
    assert_eq!(sample["type"], "record", "{sample}");
    for field in sample["fields"]
        .as_array_mut()
        .expect("canonical record fields")
    {
        match field["name"].as_str().expect("a field name") {
            "ratio" => {
                field["value"]["numerator"] = serde_json::json!("4");
                field["value"]["denominator"] = serde_json::json!("6");
            }
            "label" => field["value"]["value"] = serde_json::json!("e\u{0301}"),
            "channels" => field["value"]["elements"]
                .as_array_mut()
                .expect("set elements")
                .reverse(),
            _ => {}
        }
    }
    sample
}

/// FSL FR-650-AC-1: Canonical Boolean arguments produce a Boolean result.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_boolean_result() {
    assert_run_value(
        "negate",
        serde_json::json!({"type":"boolean","value":false}),
        serde_json::json!({"type":"boolean","value":true}),
    );
    assert_run_value(
        "negate",
        serde_json::json!({"type":"boolean","value":true}),
        serde_json::json!({"type":"boolean","value":false}),
    );
}

/// FSL FR-650-AC-1, FR-650-AC-2: integer results retain their full magnitude.
#[trace("TC-1700", "FR-650-AC-1")]
#[trace("TC-1701", "FR-650-AC-2")]
#[test]
fn run_returns_integer_result() {
    assert_run_value(
        "successor",
        serde_json::json!({"type":"integer","value":"6"}),
        serde_json::json!({"type":"integer","value":"7"}),
    );
    assert_run_value(
        "successor",
        serde_json::json!({"type":"integer","value":"1267650600228229401496703205375"}),
        serde_json::json!({"type":"integer","value":"1267650600228229401496703205376"}),
    );
}

/// The canonical command boundary refuses legacy argument spellings before execution.
#[trace("FR-650-AC-18")]
#[test]
fn run_refuses_a_bare_integer_argument_with_the_owner_refusal() {
    let directory = tempfile::tempdir().expect("a request folder");
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/run-sample.fsl");
    let value = serde_json::json!(3);
    let path = directory.path().join("run.json");
    std::fs::write(
        &path,
        serde_json::json!({"format":"native-run/1", "request":{
            "models":[], "program":{"source":{"file":source,"authority":"issuer",
                "identity":"document","document":"Sample","formal_revision":1}},
            "call":{"function":"successor","arguments":[{"parameter":"input","value":value}]}
        }})
        .to_string(),
    )
    .expect("the untagged request");
    let error = fsl_command::command::prepare_run_with_overrides(
        &path,
        fsl_command::command::RunLimits::default(),
        &fsl_command::command::RequestOverrides::default(),
    )
    .err()
    .expect("the owner refuses a bare argument before preparing a call");
    let expected = error.outcome_document(fsl_replay::Operation::Execute);
    let output = fsl(&[
        "run",
        path.to_str().expect("UTF-8 path"),
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    assert_eq!(
        output.status.code(),
        Some(i32::from(expected.category().exit_code()))
    );
    assert_eq!(
        output.stdout,
        format!(
            "{}\n",
            ix_cli_kit::json::encode_canonical(&expected, false)
                .expect("the owner refusal document")
        )
        .as_bytes()
    );
}

/// FSL FR-650-AC-1: a Rational result is normalized.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_rational_result() {
    assert_run_value(
        "ratioOf",
        sample(),
        serde_json::json!({"type": "rational", "numerator": "2", "denominator": "3"}),
    );
}

/// FSL FR-650-AC-1: a Decimal result keeps its coefficient and scale.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_decimal_result() {
    let value = serde_json::json!({"type": "decimal", "coefficient": "150", "scale": "2"});
    assert_run_value("priceOf", sample(), value);
}

/// FSL FR-650-AC-1: a Float64 result is preserved by its bit encoding.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_float_result() {
    let value = serde_json::json!({"type": "float64", "bits": "3ff0000000000000"});
    assert_run_value("gainOf", sample(), value);
}

/// FSL FR-650-AC-1: a String result keeps its text.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_string_result() {
    let value = serde_json::json!({"type": "string", "value": "abc"});
    assert_run_value("codeOf", sample(), value);
}

/// FSL FR-650-AC-1: a Text result is normalized to NFC.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_text_result() {
    assert_run_value(
        "labelOf",
        sample(),
        serde_json::json!({"type": "text", "value": "\u{00e9}"}),
    );
}

/// FSL FR-650-AC-1: an enumeration result uses its producer-issued declaration name.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_enum_result() {
    let value = sample_field(&sample(), "light");
    assert_eq!(value["type"], "enum");
    assert_eq!(value["member"], "Green");
    assert_run_value("lightOf", sample(), value);
}

/// FSL FR-650-AC-1: a record result carries every field.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_record_result() {
    let mut value = sample_field(&sample(), "point");
    value["fields"] = serde_json::json!([
        {"name":"x","value":{"type":"integer","value":"1"}},
        {"name":"y","value":{"type":"integer","value":"2"}}
    ]);
    assert_run_value("pointOf", sample(), value);
}

/// FSL FR-650-AC-1: a tuple result carries each ordered component.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_tuple_result() {
    let mut value = sample_field(&sample(), "pair");
    value["components"] = serde_json::json!([
        {"type":"integer","value":"2"}, {"type":"integer","value":"1"}
    ]);
    assert_run_value("pairOf", sample(), value);
}

/// FSL FR-650-AC-1: a Sequence result keeps its order and repeats.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_sequence_result() {
    let value = serde_json::json!({"type":"sequence","elements":[{"type":"integer","value":"2"},{"type":"integer","value":"1"},{"type":"integer","value":"2"}]});
    assert_run_value("readingsOf", sample(), value);
}

/// FSL FR-650-AC-1: Set results use canonical order.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_set_result() {
    assert_run_value(
        "channelsOf",
        sample(),
        serde_json::json!({"type":"set","elements":[{"type":"integer","value":"1"},{"type":"integer","value":"3"}]}),
    );
}

/// FSL FR-650-AC-1: an Option result carries its value or null.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_option_result() {
    let present = serde_json::json!({"type":"option","value":{"type":"integer","value":"3"}});
    assert_run_value("firstOf", sample(), present);
    let absent = serde_json::json!({"type":"option","value":null});
    assert_run_value("lastOf", sample(), absent);
}

/// A unit whose `give` returns a collection over a population: FSL FR-650's
/// one result type with no run value form (FSpec FR-144).
const FORMLESS_RESULT_SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
model M = "driver/ir608-model";
function give using v(p: Population<M::Thing>[1]): Sequence<Population<M::Thing>[1]>[0, 1] pure { sequence[p] }
"#;

const POPULATION_MODEL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/population-model.semantic-ir.json"
);

/// FSL FR-650-AC-6: a result form unavailable to execution is refused during
/// checking, before call selection or evaluation.
#[trace("TC-1705", "FR-650-AC-6")]
#[test]
fn unsupported_result_refuses_during_check() {
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("fsl-run-formless-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&directory.0);
    std::fs::create_dir_all(&directory.0).expect("a request folder");
    std::fs::write(directory.0.join("formless.fsl"), FORMLESS_RESULT_SOURCE).expect("the source");
    let request = serde_json::json!({"format": "native-run/1", "request": {
        "models": [{"format": "semantic-ir/2.0.0", "source": {"file": POPULATION_MODEL,
            "authority": "test", "identity": "population-model", "document": "M",
            "formal_revision": 1}}],
        "program": {"source": {"file": "formless.fsl", "authority": "issuer",
            "identity": "document", "document": "Formless", "formal_revision": 1}},
        "call": {"function": "give", "arguments": [{"parameter": "p",
            "value": {"population": "ix://driver/ir608-model/Fleet"}}]},
    }});
    let path = directory.0.join("run.json");
    std::fs::write(&path, request.to_string()).expect("the request file");
    let output = Command::new(FSL)
        .arg("run")
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("run fsl run");
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(document["category"], "refusal", "{document}");
    assert_eq!(document["operation"], "execute", "{document}");
    assert!(document["result"].is_null(), "{document}");
    assert_eq!(document["last_stage"], "S3", "{document}");
    let diagnostics = document["diagnostics"].as_array().expect("diagnostics");
    assert_eq!(diagnostics.len(), 1, "{document}");
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["code"], "ill_typed", "{document}");
    assert_eq!(diagnostic["cause"], "operator-ineligible", "{document}");
    assert_eq!(diagnostic["locus"]["kind"], "region", "{document}");
}

/// Emit a supported sequence result and give its element the package’s population type.
/// This models an IR producer that can express a signature the source checker refuses.
fn population_sequence_package(directory: &std::path::Path, models: &serde_json::Value) {
    use fsl_semantics::check::wire_id::{resolve_operand_slots, write_operand_rows};
    use serde_json::{json, Value};

    let source = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
model M = "driver/ir608-model";
function gives using v(): Sequence<Integer>[0, 1] pure { sequence[1] }
function hasFleet using v(fleet: Population<M::Thing>[1]): Boolean pure { true }
"#;
    std::fs::write(directory.join("fleet.fsl"), source).expect("the package source");
    let request = directory.join("compile.json");
    let compile = json!({"format": "native-compile/1", "request": {
        "models": models,
        "program": {"source": {"file": "fleet.fsl", "authority": "issuer",
            "identity": "document", "document": "Fleet", "formal_revision": 1}}
    }});
    std::fs::write(&request, compile.to_string()).expect("the compile request");
    let bytes = fsl_command::command::compile(&request).expect("the supported source compiles");
    let mut package: Value = serde_json::from_slice(&bytes).expect("a checked package");
    resolve_operand_slots(&mut package).expect("graph references resolve");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("the node table");
    let population = nodes
        .iter()
        .find(|node| node["semantic_form"] == "model_population")
        .expect("the declared population type")["node_id"]
        .clone();
    let sequence = nodes
        .iter_mut()
        .find(|node| node["node_tag"] == "composite_type" && node["semantic_form"] == "sequence")
        .expect("the sequence type of gives");
    sequence["body"]["members"][0]["target"] = population.clone();
    sequence["dependencies"] = json!([population]);
    write_operand_rows(&mut package);
    std::fs::write(directory.join("fleet.package.json"), package.to_string())
        .expect("the checked package");
}

/// A selected checked package can declare a result outside the native run value grammar.
#[trace("TC-1705", "FR-650-AC-6")]
#[test]
#[ignore = "QSL-2390 owns package-only native-run selection at the call stage"]
fn unsupported_package_result_refuses_at_call() {
    use serde_json::{json, Value};

    let directory = tempfile::tempdir().expect("a request folder");
    let models = json!([{"format": "semantic-ir/2.0.0", "source": {"file": POPULATION_MODEL,
        "authority": "test", "identity": "population-model", "document": "M", "formal_revision": 1}}]);
    population_sequence_package(directory.path(), &models);
    let request = json!({"format": "native-run/1", "request": {
        "models": models, "package": {"file": "fleet.package.json"},
        "call": {"function": "gives", "arguments": []}
    }});
    let path = directory.path().join("run.json");
    std::fs::write(&path, request.to_string()).expect("the run request");
    let output = fsl(&[
        "run",
        path.to_str().expect("UTF-8 path"),
        "--format",
        "json",
    ]);
    let document: Value = serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(output.status.code(), Some(21), "{document}");
    assert_eq!(document["category"], "unsupported", "{document}");
    assert_eq!(document["last_stage"], "S6a", "{document}");
    let diagnostics = document["diagnostics"].as_array().expect("diagnostics");
    assert_eq!(diagnostics.len(), 1, "{document}");
    assert_eq!(
        diagnostics[0]["code"], "unknown_required_feature",
        "{document}"
    );
    assert_eq!(diagnostics[0]["cause"], "unsupported-feature", "{document}");
    assert_eq!(
        diagnostics[0]["details"],
        json!({"function": "gives"}),
        "{document}"
    );
}

#[trace("TC-2456", "FR-865-AC-3")]
#[trace("TC-2456", "FR-865-AC-4")]
#[test]
fn check_compile_and_run_default_to_the_text_form_of_the_json_document() {
    let cases: [(&str, &[&str]); 6] = [
        ("check", &[SOURCE]),
        ("check", &[INVALID_SOURCE]),
        ("compile", &[COMPILE_REQUEST]),
        ("compile", &[COMPILE_REFUSED]),
        ("run", &[RUN_REQUEST]),
        ("run", &[RUN_REFUSED]),
    ];
    for (verb, operands) in cases {
        let with = |format: &[&str]| {
            let mut arguments = vec![verb];
            arguments.extend_from_slice(operands);
            arguments.extend_from_slice(format);
            fsl(&arguments)
        };
        let context = format!("{verb} {operands:?}");
        let default = with(&[]);
        let json = with(&["--format", "json"]);
        let text = with(&["--format", "text"]);

        assert_eq!(default.status.code(), json.status.code(), "{context}");
        assert_ne!(default.status.code(), Some(21), "{context}");
        assert!(
            !String::from_utf8_lossy(&default.stderr).contains("text rendering"),
            "{context}"
        );
        assert_eq!(
            default.stdout, text.stdout,
            "{context}: text is the default"
        );
        assert_ne!(default.stdout, json.stdout, "{context}");

        let default_text = String::from_utf8(default.stdout.clone()).expect("text is UTF-8");
        assert!(!default_text.starts_with('{'), "{context}: {default_text}");
        assert!(
            !default_text.trim().is_empty(),
            "{context}: text output is written"
        );
        serde_json::from_slice::<serde_json::Value>(&default.stdout)
            .expect_err("the default output is not the JSON document");
    }
}

#[trace("TC-2456", "FR-865-AC-4")]
#[trace("TC-2456", "FR-865-AC-5")]
#[test]
fn text_mode_io_failures_keep_the_json_exit_code_and_locate_sources() {
    for (verb, operand) in [("run", "nosuch.json"), ("compile", "nosuch.json")] {
        let default = fsl(&[verb, operand]);
        let json = fsl(&[verb, operand, "--format", "json"]);
        assert_eq!(default.status.code(), Some(refusal_code()), "{verb}");
        assert_eq!(default.status.code(), json.status.code(), "{verb}");
        assert!(
            !String::from_utf8_lossy(&default.stderr).contains("unimplemented_capability"),
            "{verb}"
        );
    }
    // A refused check names the admitted source's path in its diagnostics.
    let output = fsl(&["check", INVALID_SOURCE]);
    let text = String::from_utf8(output.stdout).expect("text is UTF-8");
    assert!(text.contains("parse-invalid.fsl"), "{text}");
}

/// FR-865-AC-1, AC-2 and AC-3: `check`, `compile` and `run` without `--format` write the
/// language renderer's text: the `<operation>: <category> (exit <code>) at <stage>` header,
/// the item line with its cause, and the `result:` line.
#[trace("TC-2456", "FR-865-AC-3")]
#[test]
fn default_text_carries_the_fr_865_header_items_and_result() {
    let text = |arguments: &[&str]| {
        let output = fsl(arguments);
        (
            output.status.code(),
            String::from_utf8(output.stdout).expect("text is UTF-8"),
        )
    };

    let (code, check) = text(&["check", INVALID_SOURCE]);
    assert_eq!(code, Some(20), "{check}");
    assert!(
        check.starts_with("check: refusal (exit 20) at S1\n"),
        "{check}"
    );
    assert!(
        check.contains("parse-invalid.fsl:3:20: invalid_syntax"),
        "{check}"
    );
    assert!(check.contains("  cause: unexpected-token"), "{check}");
    let (code, check) = text(&["check", SOURCE]);
    assert_eq!(code, Some(0));
    assert_eq!(check, "check: success (exit 0) at S4\n");

    let (code, compile) = text(&["compile", COMPILE_REFUSED]);
    assert_eq!(code, Some(20), "{compile}");
    assert!(
        compile.starts_with("package: refusal (exit 20) at S1\n"),
        "{compile}"
    );
    assert!(compile.contains("  cause: unexpected-token"), "{compile}");
    let (code, compile) = text(&["compile", COMPILE_REQUEST]);
    assert_eq!(code, Some(0));
    assert_eq!(compile, "package: success (exit 0) at S4\n");

    let (code, run) = text(&["run", RUN_REQUEST]);
    assert_eq!(code, Some(0), "{run}");
    assert!(
        run.starts_with("execute: success (exit 0) at S6a\n"),
        "{run}"
    );
    assert!(
        run.contains("result: completed {\"value\":{\"boolean\":true}}"),
        "{run}"
    );
    let (code, run) = text(&["run", RUN_REFUSED]);
    assert_eq!(code, Some(20), "{run}");
    assert!(
        run.starts_with("execute: refusal (exit 20) at S6a\n"),
        "{run}"
    );
    assert!(run.contains("missing_declaration"), "{run}");
}

/// FSL FR-670-AC-1's unit, with the floor law header when `header` is set.
fn division_unit(directory: &std::path::Path, header: bool) -> std::path::PathBuf {
    let law = if header {
        "profile d = \"filament.value.integer-division.floor/v1\";\n"
    } else {
        ""
    };
    let path = directory.join("u.fsl");
    std::fs::write(
        &path,
        format!(
            "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"filament.value.complete/v1\";\n{law}\
             function q using v(a: Int[-7, 7], b: Int[1, 3]): Int[-7, 7] pure {{ a div b }}\n\
             function r using v(a: Int[-7, 7], b: Int[1, 3]): Int[-7, 7] pure {{ a rem b }}\n"
        ),
    )
    .expect("the division unit");
    path
}

fn diagnostic_codes(output: &Output) -> Vec<(String, String)> {
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("a check document");
    document["diagnostics"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|diagnostic| {
            (
                diagnostic["code"].as_str().unwrap_or_default().to_owned(),
                diagnostic["cause"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

/// FSL FR-767-AC-7: `fsl check --integer-division <law>` passes the admitted
/// law to the check; an unknown law refuses before the source is read; the
/// option given twice refuses with usage and no stdout byte; a header law
/// that disagrees refuses `conflicting-definition`; `run`, `compile`,
/// `analyze`, `prove` and `monitor` refuse the option as unknown. QSL-2279.
#[trace("TC-2189", "FR-767-AC-7")]
#[test]
fn check_takes_the_integer_division_law_from_its_option() {
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("fsl-integer-division-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&directory.0);
    std::fs::create_dir_all(&directory.0).expect("a unit folder");
    let unit = division_unit(&directory.0, false);
    let unit = unit.to_str().expect("UTF-8 path");

    let checked = fsl(&[
        "check",
        unit,
        "--integer-division",
        "floor",
        "--format",
        "json",
    ]);
    assert_eq!(checked.status.code(), Some(0), "{checked:?}");

    let without = fsl(&["check", unit, "--format", "json"]);
    assert_eq!(without.status.code(), Some(20), "{without:?}");
    assert!(diagnostic_codes(&without).contains(&(
        "missing_declaration".to_owned(),
        "missing-selection".to_owned()
    )));

    let absent = directory.0.join("absent.fsl");
    let absent = absent.to_str().expect("UTF-8 path");
    let unknown = fsl(&[
        "check",
        absent,
        "--integer-division",
        "ceiling",
        "--format",
        "json",
    ]);
    assert_eq!(unknown.status.code(), Some(20), "{unknown:?}");
    assert_eq!(
        diagnostic_codes(&unknown),
        [(
            "unknown_profile".to_owned(),
            "unsupported-selection".to_owned()
        )]
    );
    let unknown_document: serde_json::Value =
        serde_json::from_slice(&unknown.stdout).expect("unknown-law check document");
    let message = unknown_document["diagnostics"][0]["message"]
        .as_str()
        .expect("unknown-law diagnostic message");
    assert!(
        message.contains(
            "integer-division law `ceiling` supplied by `--integer-division` is not one of \
             `euclidean`, `floor`, `truncating`"
        ),
        "{message}"
    );
    assert!(!message.contains("could not read"), "{message}");
    assert!(unknown.stderr.is_empty(), "{unknown:?}");
    let near_miss = fsl(&["check", absent, "--integer-division", "Floor"]);
    assert_eq!(near_miss.status.code(), Some(20));

    let twice = fsl(&[
        "check",
        unit,
        "--integer-division",
        "floor",
        "--integer-division",
        "floor",
        "--format",
        "json",
    ]);
    assert_eq!(twice.status.code(), Some(20));
    assert!(twice.stdout.is_empty(), "no stdout byte");
    assert!(String::from_utf8_lossy(&twice.stderr).contains("usage: fsl check"));

    let headed = division_unit(&directory.0, true);
    let headed = headed.to_str().expect("UTF-8 path");
    let agreeing = fsl(&[
        "check",
        headed,
        "--integer-division",
        "floor",
        "--format",
        "json",
    ]);
    assert_eq!(agreeing.status.code(), Some(0), "{agreeing:?}");
    let conflicting = fsl(&[
        "check",
        headed,
        "--integer-division",
        "truncating",
        "--format",
        "json",
    ]);
    assert_eq!(conflicting.status.code(), Some(20), "{conflicting:?}");
    assert!(diagnostic_codes(&conflicting).contains(&(
        "invalid_package".to_owned(),
        "conflicting-definition".to_owned()
    )));

    for verb in ["run", "compile", "analyze", "prove", "monitor"] {
        let output = fsl(&[verb, "request.json", "--integer-division", "floor"]);
        assert_eq!(output.status.code(), Some(20), "{verb}: {output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("unknown") || stderr.contains("--integer-division"),
            "{verb}: {stderr}"
        );
        assert!(!stderr.contains("could not read"), "{verb}: {stderr}");
    }

    for flag in ["--integer-division", "--bogus"] {
        let output = fsl(&["prove", "request.json", flag, "floor", "--format", "json"]);
        assert_eq!(output.status.code(), Some(20), "{flag}: {output:?}");
        let document: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("an outcome document");
        assert_eq!(document["operation"], "prove", "{document}");
        assert_eq!(document["category"], "refusal", "{document}");
        assert!(
            document["diagnostics"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|diagnostic| diagnostic["code"] == "invalid-request"
                    && diagnostic["message"]
                        .as_str()
                        .is_some_and(|message| message.contains(flag))),
            "{flag}: {document}"
        );
    }
}

/// FSL FR-767-AC-7, run half: `fsl run` over a request with
/// `"integer_division": "floor"` calling `q(-7, 3)` exits 0 with value -3.
#[trace("TC-2189", "FR-767-AC-7")]
#[test]
fn run_takes_the_integer_division_law_from_its_request() {
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("fsl-integer-division-run-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&directory.0);
    std::fs::create_dir_all(&directory.0).expect("a unit folder");
    let unit = division_unit(&directory.0, false);
    let request = directory.0.join("run.json");
    std::fs::write(
        &request,
        serde_json::json!({"format": "native-run/1", "request": {"models": [],
            "integer_division": "floor",
            "program": {"source": {"file": unit, "authority": "test", "identity": "division",
                "document": "Division", "formal_revision": 1}},
            "call": {"function": "q", "arguments": [
                {"parameter": "a", "value": {"type":"integer","value":"-7"}}, {"parameter": "b", "value": {"type":"integer","value":"3"}}]}}})
        .to_string(),
    )
    .expect("the run request");
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run fsl");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one division outcome");
    assert_eq!(
        document["result"],
        serde_json::json!({
            "kind":"completed", "value":{"type":"integer","value":"-3"}
        })
    );
}

#[trace("TC-2456", "FR-865-AC-3")]
#[test]
fn parse_defaults_to_the_text_form_with_header_and_result_lines() {
    let source = SOURCE;
    let default = fsl(&["parse", "issuer", "document", source]);
    let text = fsl(&["parse", "issuer", "document", source, "--format", "text"]);
    let json = fsl(&["parse", "issuer", "document", source, "--format", "json"]);
    assert_eq!(default.status.code(), Some(0));
    assert_eq!(default.status.code(), json.status.code());
    assert_eq!(default.stdout, text.stdout, "text is the default");
    let body = String::from_utf8(default.stdout).expect("text is UTF-8");
    assert!(!body.starts_with('{'), "{body}");
    let mut lines = body.lines();
    assert_eq!(
        lines.next(),
        Some("parse: success (exit 0) at S2"),
        "{body}"
    );
    assert!(
        lines
            .next()
            .is_some_and(|line| line.starts_with("result: parsed ")),
        "{body}"
    );
    let document: serde_json::Value =
        serde_json::from_slice(&json.stdout).expect("json output is the document");
    assert_eq!(document["category"], "success");
}

#[trace("TC-2456", "FR-865-AC-4")]
#[test]
fn parse_help_row_names_text_as_the_default() {
    let output = fsl(&["--help"]);
    let help = String::from_utf8(output.stdout).expect("help is UTF-8");
    let line = help
        .lines()
        .find(|line| line.split_whitespace().next() == Some("parse"))
        .expect("parse help row");
    assert!(line.contains("text by default"), "{line}");
    assert!(!line.contains("unavailable"), "{line}");
}

fn simulation_item(settings: serde_json::Value) -> serde_json::Value {
    serde_json::json!([{"kind": "simulation", "simulation": settings}])
}

/// FSL FR-634-AC-3, FR-181: `fsl analyze` runs a sampled simulation item
/// and writes the traces; the same seed gives the same document twice, and
/// the seed is echoed. QSL-2132.
#[trace("TC-036", "FR-022-AC-9")]
#[trace("TC-1594", "FR-634-AC-3")]
#[test]
fn analyze_simulation_is_deterministic_for_seed() {
    let settings = serde_json::json!({"mode": "sample", "seed": 5, "samples": 3, "max_steps": 20});
    let request = AnalyzeRequest::new("sim-seed", simulation_item(settings), serde_json::json!({}));
    let (first, exit) = request.json();
    let (second, _) = request.json();
    assert_eq!(exit, Some(0), "{first}");
    assert_eq!(first, second);
    let item = &first["items"][0];
    assert_eq!(item["category"], "success", "{item}");
    assert_eq!(item["seed"], "5", "{item}");
    assert_eq!(item["traces"].as_array().map(Vec::len), Some(3), "{item}");
    assert!(
        item.get("record").is_none(),
        "a simulation settles no claim"
    );
}

/// FSL FR-634: enumeration through the binary reports the Counter's four
/// states and one trace. QSL-2132.
#[trace("TC-036", "FR-022-AC-9")]
#[trace("TC-1594", "FR-634-AC-1")]
#[test]
fn analyze_simulation_enumerates_the_counter() {
    let request = AnalyzeRequest::new(
        "sim-enumerate",
        simulation_item(serde_json::json!({"mode": "enumerate"})),
        serde_json::json!({}),
    );
    let (document, exit) = request.json();
    assert_eq!(exit, Some(0), "{document}");
    let item = &document["items"][0];
    assert_eq!(item["coverage"]["states"], "4", "{item}");
    assert_eq!(item["coverage"]["transitions"], "3", "{item}");
    assert_eq!(item["traces"].as_array().map(Vec::len), Some(1), "{item}");
}

/// FSL FR-634 "Refusals": a simulation with zero steps refuses the whole
/// request, exit 20. QSL-2132.
#[trace("TC-036", "FR-022-AC-9")]
#[trace("TC-1594", "FR-634")]
#[test]
fn analyze_simulation_with_zero_steps_is_refused() {
    let settings = serde_json::json!({"mode": "sample", "seed": 1, "samples": 1, "max_steps": 0});
    let request = AnalyzeRequest::new("sim-zero", simulation_item(settings), serde_json::json!({}));
    let (document, exit) = request.json();
    assert_eq!(exit, Some(20), "{document}");
    assert!(document.to_string().contains("max_steps"), "{document}");
}

/// FSL FR-635-AC-3: a `backend` the registry does not hold settles
/// `invalid-request` `unknown-backend`. QSL-2132.
#[trace("TC-036", "FR-022-AC-9")]
#[trace("TC-1595", "FR-635-AC-3")]
#[test]
fn analyze_unregistered_provider_is_refused_with_unknown_backend() {
    let items = serde_json::json!([
        {"claim": "Even", "kind": "value-validity", "backend": PROVIDER_BACKEND}]);
    let request = AnalyzeRequest::new("provider-none", items, serde_json::json!({})).with_file(
        "/request/program/source/file",
        "even.fsl",
        &even_program(),
    );
    let (document, exit) = request.json();
    assert_eq!(exit, Some(20), "{document}");
    assert!(
        document.to_string().contains("unknown-backend"),
        "{document}"
    );
}

/// FSL FR-281: a provider answering with an outcome that is not a FR-331
/// result settles the item `inconclusive`, never `proved`. QSL-2132.
#[trace("TC-036", "FR-022-AC-9")]
#[test]
fn analyze_provider_garbage_is_inconclusive() {
    let items = serde_json::json!([
        {"claim": "Even", "kind": "value-validity", "backend": PROVIDER_BACKEND}]);
    let request = AnalyzeRequest::new("provider-garbage", items, serde_json::json!({})).with_file(
        "/request/program/source/file",
        "even.fsl",
        &even_program(),
    );
    let config = provider_config(&request, &[PROVIDER_BACKEND, "unbounded", "garbage"]);
    let output = analyze_with_config(&request, &config);
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("a document");
    assert_eq!(
        document["items"][0]["record"]["result"], "inconclusive",
        "{document}"
    );
}

/// FSL FR-281, FR-635: a provider that reports a violation settles the item
/// `refuted` once the native replay reproduces its counterexample, and the
/// document carries that counterexample. QSL-2132.
#[trace("TC-036", "FR-022-AC-9")]
#[test]
fn analyze_external_provider_refutes_with_evidence() {
    let mut program = even_program();
    program.extend_from_slice(b"function Odd using v(n: Integer): Boolean pure { n = 1 }\n");
    let items = serde_json::json!([
        {"claim": "Odd", "kind": "value-validity", "backend": PROVIDER_BACKEND}]);
    let request = AnalyzeRequest::new("provider-refuted", items, serde_json::json!({})).with_file(
        "/request/program/source/file",
        "odd.fsl",
        &program,
    );
    // The provider spells the parameter as an expanded node identity, which the
    // document's own claim node holds as its third path segment.
    let proved = provider_config(&request, &[PROVIDER_BACKEND, "unbounded", "proved"]);
    let first: serde_json::Value =
        serde_json::from_slice(&analyze_with_config(&request, &proved).stdout).expect("a document");
    let parameter = first["items"][0]["request"]["claim"]["node"]["path"][2].to_string();
    let config = provider_config(&request, &[PROVIDER_BACKEND, "unbounded", "refuted"]);
    let output = Command::new(FSL)
        .arg("analyze")
        .arg(request.path())
        .arg("--provider-config")
        .arg(&config)
        .args(["--format", "json"])
        .env("FSL_TEST_PROVIDER_PARAMETER", parameter)
        .output()
        .expect("run fsl analyze --provider-config");
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("a document");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "refuted", "{document}");
    assert_eq!(item["record"]["backend"], PROVIDER_BACKEND, "{document}");
    assert_eq!(item["counterexample"]["type"], "assignments", "{document}");
    assert_eq!(
        item["counterexample"]["values"][0]["value"]["value"], "0",
        "{document}"
    );
}

const EXTENSION_HEADER: &str =
    "language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\n";
const EXTENSION_FORM: &str = "retention Audit { subject: \"audit\", days: 30 }";

/// Writes FSpec FR-467-AC-1's `retention` definition document, the set
/// document naming it with `entries`, and a unit selecting it.
fn extension_directory(entries: serde_json::Value) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary directory");
    let definition = serde_json::json!({
        "grammar_schema": {"productions": [
            {"keyword": "retention", "clauses": ["subject", "days"]}
        ]},
        "typed_node_schema": {"nodes": [
            {"keyword": "retention", "node_kind": "retention", "clauses": [
                {"name": "subject", "type": "Text"},
                {"name": "days", "type": "Int[1, 3650]"}
            ]}
        ]}
    });
    std::fs::write(
        directory.path().join("retention.json"),
        definition.to_string(),
    )
    .expect("write definition");
    std::fs::write(
        directory.path().join("set.json"),
        serde_json::json!({"format": "filament.extension-set/v1", "extensions": entries})
            .to_string(),
    )
    .expect("write set");
    std::fs::write(
        directory.path().join("unit.native"),
        format!("{EXTENSION_HEADER}profile r = \"ext.retention\";\n{EXTENSION_FORM}\n"),
    )
    .expect("write unit");
    directory
}

fn in_directory(directory: &tempfile::TempDir, arguments: &[&str]) -> Output {
    Command::new(FSL)
        .args(arguments)
        .current_dir(directory.path())
        .output()
        .expect("run fsl")
}

fn retention_entry() -> serde_json::Value {
    serde_json::json!({"authority": "agent-ix", "identity": "ext.retention", "file": "retention.json"})
}

/// FSpec FR-562-AC-1 (driver FR-002-AC-24, QSL-2603): `check --extensions`
/// and `parse --extensions` supply the definition a unit selects.
#[test]
#[trace("FSpec-TC-1242", "FSpec-FR-562-AC-1", "FR-002-AC-24")]
fn extension_sets_supply_the_selected_definition_to_check_and_parse() {
    let directory = extension_directory(serde_json::json!([retention_entry()]));
    for verb in ["check", "parse"] {
        let with = in_directory(
            &directory,
            &[
                verb,
                "--extensions",
                "set.json",
                "agent-ix",
                "unit",
                "unit.native",
            ],
        );
        assert_eq!(with.status.code(), Some(0), "{verb}: {with:?}");
        let without = in_directory(&directory, &[verb, "agent-ix", "unit", "unit.native"]);
        assert_ne!(without.status.code(), Some(0), "{verb}: {without:?}");
    }
    let without = in_directory(&directory, &["check", "agent-ix", "unit", "unit.native"]);
    assert!(String::from_utf8_lossy(&without.stdout).contains("unknown_profile"));
}

/// FSpec FR-562-AC-2 and FR-301-AC-5: a set document that cannot be read as
/// FR-562 states refuses with exit 20 before any stage runs.
#[test]
#[trace("FSpec-TC-1242", "FSpec-FR-562-AC-2", "FSpec-FR-301-AC-5")]
fn malformed_extension_sets_refuse_before_any_stage() {
    let mut unknown_member = retention_entry();
    unknown_member["version"] = serde_json::json!("1");
    let mut no_file = retention_entry();
    no_file.as_object_mut().expect("object").remove("file");
    for (entries, code) in [
        (serde_json::json!([unknown_member]), "invalid-request"),
        (serde_json::json!([no_file]), "invalid-request"),
        (
            serde_json::json!([retention_entry(), retention_entry()]),
            "conflicting-definition",
        ),
    ] {
        let directory = extension_directory(entries);
        for verb in ["check", "parse"] {
            let output = in_directory(
                &directory,
                &[
                    verb,
                    "--extensions",
                    "set.json",
                    "agent-ix",
                    "unit",
                    "unit.native",
                ],
            );
            assert_eq!(output.status.code(), Some(20), "{verb} {code}: {output:?}");
            let text = String::from_utf8_lossy(&output.stdout).into_owned()
                + &String::from_utf8_lossy(&output.stderr);
            assert!(text.contains(code), "{verb} {code}: {text}");
        }
    }
    let directory = extension_directory(serde_json::json!([retention_entry()]));
    let missing = in_directory(
        &directory,
        &[
            "check",
            "--extensions",
            "absent.json",
            "agent-ix",
            "unit",
            "unit.native",
        ],
    );
    assert_eq!(missing.status.code(), Some(20), "{missing:?}");
    let repeated = in_directory(
        &directory,
        &[
            "check",
            "--extensions",
            "set.json",
            "--extensions",
            "set.json",
            "agent-ix",
            "unit",
            "unit.native",
        ],
    );
    assert_eq!(repeated.status.code(), Some(20), "{repeated:?}");
}

/// FSpec FR-562-AC-1: the one-operand form `check --extensions set.json unit.native`.
#[test]
#[trace("FSpec-TC-1242", "FSpec-FR-562-AC-1", "FR-002-AC-24")]
fn check_with_extensions_takes_the_one_operand_form() {
    let directory = extension_directory(serde_json::json!([retention_entry()]));
    let with = in_directory(
        &directory,
        &["check", "--extensions", "set.json", "unit.native"],
    );
    assert_eq!(with.status.code(), Some(0), "{with:?}");
    let without = in_directory(&directory, &["check", "unit.native", "--format", "json"]);
    assert_eq!(without.status.code(), Some(20), "{without:?}");
    assert!(String::from_utf8_lossy(&without.stdout).contains("unknown_profile"));
}

/// FSpec FR-562-AC-2: a definition document that breaks FR-467's
/// `missing-member` rule refuses with exit 20, and a unit that selects no
/// extension gives the same result with and without the set.
#[test]
#[trace("FSpec-TC-1242", "FSpec-FR-562-AC-2", "FR-002-AC-24")]
fn extension_definitions_are_validated_and_unselected_sets_change_nothing() {
    let directory = extension_directory(serde_json::json!([retention_entry()]));
    std::fs::write(
        directory.path().join("retention.json"),
        serde_json::json!({"grammar_schema": {"productions": [
            {"keyword": "retention", "clauses": ["subject", "days"]}
        ]}})
        .to_string(),
    )
    .expect("write a definition without typed_node_schema");
    for verb in ["check", "parse"] {
        let output = in_directory(
            &directory,
            &[
                verb,
                "--extensions",
                "set.json",
                "agent-ix",
                "unit",
                "unit.native",
            ],
        );
        assert_eq!(output.status.code(), Some(20), "{verb}: {output:?}");
        let text = String::from_utf8_lossy(&output.stdout).into_owned()
            + &String::from_utf8_lossy(&output.stderr);
        assert!(text.contains("missing-member"), "{verb}: {text}");
    }

    let directory = extension_directory(serde_json::json!([retention_entry()]));
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/spine-compile.native"
        ),
        directory.path().join("plain.native"),
    )
    .expect("copy a unit that selects no extension");
    for verb in ["check", "parse"] {
        let mut without = vec![
            verb,
            "agent-ix",
            "plain",
            "plain.native",
            "--format",
            "json",
        ];
        let plain = in_directory(&directory, &without);
        without.splice(1..1, ["--extensions", "set.json"]);
        let with = in_directory(&directory, &without);
        assert_eq!(plain.status, with.status, "{verb}");
        assert_eq!(plain.stdout, with.stdout, "{verb}");
    }
}

fn hybrid_braking_request(name: &str, items: serde_json::Value) -> AnalyzeRequest {
    let request = AnalyzeRequest::new(
        name,
        items,
        serde_json::json!({"models": [], "documents": []}),
    );
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(request.path()).expect("request")).expect("JSON");
    document["request"]["program"]["source"] = serde_json::json!({
        "file": dependency_fixtures::fsl_example("hybrid-braking/braking.native"), "authority": "driver",
        "identity": "driver:configured-hybrid", "document": "ConfiguredHybrid", "formal_revision": 1
    });
    std::fs::write(request.path(), document.to_string())
        .expect("configured request referencing the owner source");
    request
}

fn provider_document(output: &Output, code: i32) -> serde_json::Value {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    serde_json::from_slice(&output.stdout).expect("provider outcome JSON")
}

#[trace("TC-047", "FR-022-AC-11")]
#[test]
fn hybrid_braking_default_proof_and_refutation_have_real_evidence() {
    for (request, code, result) in [
        ("slow-enough.analyze.json", 0, "proved"),
        ("too-slow.analyze.json", 10, "refuted"),
    ] {
        let output = Command::new(FSL)
            .arg("analyze")
            .arg(dependency_fixtures::fsl_example(&format!(
                "hybrid-braking/{request}"
            )))
            .args(["--format", "json"])
            .output()
            .expect("analyze the actual owner example request");
        let document = provider_document(&output, code);
        let item = &document["items"][0];
        assert_eq!(item["record"]["result"], result, "{document}");
        assert_eq!(
            item["record"]["backend"], "fsl-hybrid-reference",
            "{document}"
        );
        if result == "proved" {
            assert_eq!(item["record"]["certification"], "certified", "{document}");
            assert_eq!(item["certificate"]["accepted"], true, "{document}");
        } else {
            assert_eq!(item["replay"]["reproduced"], true, "{document}");
            assert!(!item["counterexample"].is_null(), "{document}");
        }
        let registry = document["registry"].as_array().expect("registry");
        for backend in ["fsl-hybrid-reference", "en-1"] {
            assert!(
                registry.iter().any(|entry| entry["backend"] == backend),
                "{document}"
            );
        }
    }
}

#[trace("TC-047", "FR-022-AC-11")]
#[test]
fn hybrid_process_selection_transports_fsl_request_and_keeps_trusted_proof() {
    let request = hybrid_braking_request(
        "hybrid-process",
        serde_json::json!([
            {"claim": "SlowEnough", "backend": PROVIDER_BACKEND},
            {"claim": "SlowEnough", "backend": "fsl-hybrid-reference"},
            {"claim": "SlowEnough"},
        ]),
    );
    let log = request.directory.0.join("hybrid-request.json");
    let shutdown = request.directory.0.join("shutdown");
    let config = provider_config(
        &request,
        &[
            PROVIDER_BACKEND,
            "hybrid",
            "unsat",
            log.to_str().expect("log path"),
            shutdown.to_str().expect("shutdown path"),
        ],
    );
    let document = provider_document(&analyze_with_config(&request, &config), 20);
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "proved", "{document}");
    assert_eq!(
        item["record"]["basis"]["type"], "bounded-solver",
        "{document}"
    );
    assert_eq!(
        item["record"]["basis"]["method"], "delta-complete",
        "{document}"
    );
    assert_eq!(item["record"]["certification"], "trusted", "{document}");
    assert_eq!(item["record"]["backend"], PROVIDER_BACKEND, "{document}");
    assert_eq!(
        document["items"][1]["record"]["result"], "proved",
        "{document}"
    );
    assert_eq!(
        document["items"][1]["record"]["backend"], "fsl-hybrid-reference",
        "{document}"
    );
    assert_eq!(
        document["items"][1]["record"]["certification"], "certified",
        "{document}"
    );
    assert!(
        document["items"][2]
            .to_string()
            .contains("ambiguous-backend"),
        "{document}"
    );
    assert_eq!(
        std::fs::read(shutdown).expect("orderly provider shutdown"),
        b"shutdown"
    );
    let sent: serde_json::Value = serde_json::from_slice(&std::fs::read(log).expect("request log"))
        .expect("FSL's hybrid request");
    assert_eq!(
        sent["contract_version"], "filament.backend-provider/v1",
        "{sent}"
    );
    assert_eq!(sent["request"]["items"][0]["request_index"], 0, "{sent}");
    assert_eq!(
        sent["request"]["items"][0]["kind"], "temporal-satisfaction",
        "{sent}"
    );
    assert!(sent.to_string().contains("hybrid"), "{sent}");
    assert_eq!(
        sent["request"]["items"][0]["requirements"],
        serde_json::json!(["hybrid-dynamics"]),
        "{sent}"
    );
    assert_eq!(sent["request"]["deadline_ms"], 600_000, "{sent}");
    let encoded = sent["request"]["package"].as_str().expect("base64 package");
    use base64::Engine as _;
    let package = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .expect("FSL emitted package bytes");
    assert!(
        String::from_utf8_lossy(&package).contains("SlowEnough"),
        "emitted package"
    );
    assert!(!sent["request"]["package"].is_null(), "{sent}");
}

#[trace("TC-047", "FR-022-AC-11")]
#[test]
fn hybrid_process_preserves_malformed_results_for_fsl_refusal() {
    for result in ["malformed", "unknown-member"] {
        let request = hybrid_braking_request(
            &format!("hybrid-{result}"),
            serde_json::json!([
                {"claim": "SlowEnough", "backend": PROVIDER_BACKEND},
            ]),
        );
        let shutdown = request.directory.0.join("shutdown");
        let config = provider_config(
            &request,
            &[
                PROVIDER_BACKEND,
                "hybrid",
                result,
                "-",
                shutdown.to_str().expect("shutdown path"),
            ],
        );
        let output = analyze_with_config(&request, &config);
        let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("outcome");
        assert_ne!(output.status.code(), Some(0), "{document}");
        assert_eq!(
            document["items"][0]["record"]["result"], "failed",
            "{document}"
        );
        assert_eq!(std::fs::read(shutdown).expect("shutdown"), b"shutdown");
        if result == "unknown-member" {
            assert!(
                document.to_string().contains("malformed-body"),
                "{document}"
            );
        }
    }
}

#[trace("TC-047", "FR-022-AC-11")]
#[test]
fn hybrid_provider_unsuitable_hello_is_refused_and_started_sessions_are_closed() {
    for mode in ["hybrid-bounded", "hybrid-no-dynamics"] {
        let request = hybrid_braking_request(
            &format!("hybrid-hello-{mode}"),
            serde_json::json!([
                {"claim": "SlowEnough", "backend": PROVIDER_BACKEND},
            ]),
        );
        let shutdown = request.directory.0.join("shutdown");
        let first_shutdown = request.directory.0.join("first-shutdown");
        let config = request.directory.0.join("providers.json");
        std::fs::write(&config, serde_json::json!({"format": "filament.provider-config/v1",
            "providers": [
                {"path": test_provider_path::executable(), "arguments": [
                    "first-hybrid", "hybrid", "unsat", "-", first_shutdown.to_str().expect("path")]},
                {"path": test_provider_path::executable(), "arguments": [
                    PROVIDER_BACKEND, mode, "unsat", "-", shutdown.to_str().expect("path")]}
            ]}).to_string()).expect("configuration");
        let output = analyze_with_config(&request, &config);
        assert_eq!(output.status.code(), Some(20), "{output:?}");
        assert_eq!(
            std::fs::read(first_shutdown).expect("earlier session shutdown"),
            b"shutdown"
        );
        // Admission may terminate an invalid hello immediately; a started,
        // admitted but unsuitable provider must receive orderly shutdown.
        if mode == "hybrid-bounded" {
            assert_eq!(
                std::fs::read(shutdown).expect("unsuitable session shutdown"),
                b"shutdown"
            );
        }
    }
}

#[trace("TC-047", "FR-022-AC-11")]
#[test]
fn hybrid_process_deadline_settles_incomplete_instead_of_accepting_a_late_proof() {
    let request = hybrid_braking_request(
        "hybrid-deadline",
        serde_json::json!([
            {"claim": "SlowEnough", "backend": PROVIDER_BACKEND},
            {"claim": "SlowEnough", "backend": "fsl-hybrid-reference"},
        ]),
    );
    let config = provider_config(&request, &[PROVIDER_BACKEND, "hybrid", "late"]);
    let output = Command::new(FSL)
        .arg("analyze")
        .arg(request.path())
        .arg("--provider-config")
        .arg(config)
        .args(["--provider-limit", "deadline_ms=500", "--format", "json"])
        .output()
        .expect("deadline hybrid run");
    let document = provider_document(&output, 22);
    assert_eq!(
        document["items"][0]["record"]["result"], "incomplete",
        "{document}"
    );
    assert!(document.to_string().contains("timed-out"), "{document}");
    assert_eq!(
        document["items"][1]["record"]["result"], "proved",
        "{document}"
    );
    assert_eq!(
        document["items"][1]["record"]["certification"], "certified",
        "{document}"
    );
    assert_eq!(
        document["items"][1]["certificate"]["accepted"], true,
        "{document}"
    );
}

#[trace("TC-047", "FR-022-AC-11")]
#[test]
fn hybrid_process_signal_cancels_the_open_item() {
    for signal in ["term", "int"] {
        let request = hybrid_braking_request(
            &format!("hybrid-signal-{signal}"),
            serde_json::json!([
                {"claim": "SlowEnough", "backend": PROVIDER_BACKEND},
            ]),
        );
        let config = provider_config(&request, &[PROVIDER_BACKEND, "hybrid", signal]);
        let document = provider_document(&analyze_with_config(&request, &config), 22);
        assert_eq!(
            document["items"][0]["record"]["result"], "incomplete",
            "{document}"
        );
        assert!(document.to_string().contains("cancelled"), "{document}");
    }
}

#[trace("TC-047", "FR-022-AC-11")]
#[test]
fn hybrid_process_refutation_requires_exact_native_replay() {
    for (result, terminal) in [("refuted", "refuted"), ("false-refutation", "inconclusive")] {
        let request = hybrid_braking_request(
            &format!("hybrid-replay-{result}"),
            serde_json::json!([
                {"claim": "TooSlow", "backend": PROVIDER_BACKEND},
            ]),
        );
        let config = provider_config(&request, &[PROVIDER_BACKEND, "hybrid", result]);
        let output = analyze_with_config(&request, &config);
        let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("outcome");
        let item = &document["items"][0];
        assert_eq!(item["record"]["result"], terminal, "{document}");
        if result == "refuted" {
            assert_eq!(output.status.code(), Some(10), "{document}");
            assert_eq!(item["replay"]["reproduced"], true, "{document}");
            assert!(!item["counterexample"].is_null(), "{document}");
        } else {
            assert_ne!(output.status.code(), Some(10), "{document}");
            assert!(item.to_string().contains("replay-parity"), "{document}");
            assert!(
                item.get("counterexample")
                    .is_none_or(serde_json::Value::is_null),
                "{document}"
            );
        }
    }
}

// Mixed capability support needs a combined FSL engine under one identity.
// This checks the current refusal and cleanup rather than claiming support.
#[trace("TC-047")]
#[test]
fn hybrid_mixed_capability_refusal_names_both_groups_and_closes_every_session() {
    let request = hybrid_braking_request(
        "hybrid-mixed-refusal",
        serde_json::json!([
            {"claim": "SlowEnough", "backend": PROVIDER_BACKEND},
        ]),
    );
    let first_shutdown = request.directory.0.join("first-shutdown");
    let mixed_shutdown = request.directory.0.join("mixed-shutdown");
    let final_shutdown = request.directory.0.join("final-shutdown");
    let request_log = request.directory.0.join("should-not-run");
    let config = request.directory.0.join("providers.json");
    std::fs::write(
        &config,
        serde_json::json!({"format": "filament.provider-config/v1",
        "providers": [
            {"path": test_provider_path::executable(), "arguments": [
                "first-value", "unbounded", "proved", "-", first_shutdown.to_str().expect("path")]},
            {"path": test_provider_path::executable(), "arguments": [
                PROVIDER_BACKEND, "hybrid-mixed", "unsat", request_log.to_str().expect("path"),
                mixed_shutdown.to_str().expect("path")]},
            {"path": test_provider_path::executable(), "arguments": [
                "final-hybrid", "hybrid", "unsat", "-", final_shutdown.to_str().expect("path")]}
        ]})
        .to_string(),
    )
    .expect("configuration");
    let output = analyze_with_config(&request, &config);
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for member in [
        PROVIDER_BACKEND,
        "value-validity",
        "temporal-satisfaction",
        "hybrid",
    ] {
        assert!(diagnostic.contains(member), "{diagnostic}");
    }
    assert!(!request_log.exists(), "refusal precedes item dispatch");
    for marker in [first_shutdown, mixed_shutdown, final_shutdown] {
        assert_eq!(
            std::fs::read(marker).expect("all admitted sessions close"),
            b"shutdown"
        );
    }
}

#[trace("TC-047")]
#[test]
fn hybrid_unbounded_only_provider_does_not_authorize_a_bounded_claim() {
    let request = hybrid_braking_request(
        "hybrid-unbounded-only",
        serde_json::json!([
            {"claim": "SlowEnough", "backend": PROVIDER_BACKEND},
        ]),
    );
    let log = request.directory.0.join("should-not-dispatch");
    let shutdown = request.directory.0.join("shutdown");
    let config = provider_config(
        &request,
        &[
            PROVIDER_BACKEND,
            "hybrid-unbounded",
            "unsat",
            log.to_str().expect("log path"),
            shutdown.to_str().expect("shutdown path"),
        ],
    );
    let document = provider_document(&analyze_with_config(&request, &config), 21);
    assert!(
        document["items"][0]
            .to_string()
            .contains("unsupported-requested-capability"),
        "{document}"
    );
    assert!(
        document["registry"]
            .as_array()
            .expect("registry")
            .iter()
            .any(|entry| entry["backend"] == PROVIDER_BACKEND),
        "{document}"
    );
    assert!(!log.exists(), "the driver must preserve advertised modes");
    assert_eq!(
        std::fs::read(shutdown).expect("admitted session shutdown"),
        b"shutdown"
    );
}

#[trace("TC-036", "FR-022-AC-7")]
#[test]
fn composition_text_preserves_derived_outcomes_trust_basis_and_extents_from_json() {
    let program = br#"language "ix:native" edition "1-draft";
profile audit = "filament.model.complete/v1";
import "driver/counter-invariants" as Invariants;
import "driver/meter-invariants" as MeterInvariants;
model M = "test/counter";
model Meter = "driver/audit-meter";
compose CounterBoundaryAudit rule interface {
  component Proven from Invariants using M;
  component Violated from MeterInvariants using Meter;
  guarantee Proven::Bounded;
  guarantee Violated::BelowThree;
}
"#;
    let request = AnalyzeRequest::new(
        "composition-text-trust",
        serde_json::json!([{"claim": "CounterBoundaryAudit", "kind": "composition"}]),
        serde_json::json!({}),
    )
    .with_file("/request/program/source/file", "counter-audit.fsl", program);
    let source = std::path::Path::new(ANALYZE_REQUEST)
        .parent()
        .expect("driver fixture directory")
        .join("analyze-counter.fsl");
    // Instantiate the existing driver schema and initial data for a second
    // selected model. Both components keep the original clause predicates.
    fn meter_identity(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(text) => {
                if text == "test/counter" {
                    *text = "driver/audit-meter".to_owned();
                } else if let Some(suffix) = text.strip_prefix("ix://test/counter/") {
                    *text = format!("ix://driver/audit-meter/{suffix}");
                }
            }
            serde_json::Value::Array(entries) => {
                for entry in entries {
                    meter_identity(entry);
                }
            }
            serde_json::Value::Object(members) => {
                for value in members.values_mut() {
                    meter_identity(value);
                }
            }
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
            }
        }
    }
    let fixtures = source.parent().expect("driver fixture directory");
    let mut meter_model: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixtures.join("analyze-counter.semantic-ir.json"))
            .expect("owned Counter schema"),
    )
    .expect("Counter model JSON");
    meter_identity(&mut meter_model);
    let model_path = request.directory.0.join("meter.semantic-ir.json");
    std::fs::write(&model_path, meter_model.to_string()).expect("independent meter model");
    let mut initial_world: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixtures.join("analyze-counter-start.json"))
            .expect("owned Counter initial data"),
    )
    .expect("Counter snapshot JSON");
    let mut meter_start = initial_world.clone();
    meter_identity(&mut meter_start);
    meter_start["populations"][0]["objects"][0]["key"] = "meter".into();
    // Every initialization document is a whole initial world. Supply both
    // real populations together, rather than two partial alternative worlds.
    initial_world["identity"] =
        serde_json::json!({"authority": "driver", "identity": "audit-world-start"});
    initial_world["populations"]
        .as_array_mut()
        .expect("initial world populations")
        .extend(
            meter_start["populations"]
                .as_array()
                .expect("meter population and initial object")
                .iter()
                .cloned(),
        );
    let start_path = request.directory.0.join("audit-world-start.json");
    std::fs::write(&start_path, initial_world.to_string())
        .expect("complete two-model initial world");
    let counter_source = std::fs::read_to_string(&source).expect("owned Counter clauses");
    // Each premise is checked in its own library scope. Both scopes must
    // map both populations present in the shared initialization world.
    let counter_library = counter_source.replace(
        "model M = \"test/counter\";",
        "model M = \"test/counter\";\nmodel Meter = \"driver/audit-meter\";",
    );
    let counter_library_path = request.directory.0.join("counter-invariants.fsl");
    std::fs::write(&counter_library_path, counter_library).expect("counter library model mappings");
    let meter_source = counter_source
        .replace(
            "model M = \"test/counter\";",
            "model Meter = \"driver/audit-meter\";\nmodel M = \"test/counter\";",
        )
        .replace("M::Counter", "Meter::Counter");
    let library_path = request.directory.0.join("meter-invariants.fsl");
    std::fs::write(&library_path, meter_source).expect("meter library model mappings");
    let mut input: serde_json::Value =
        serde_json::from_slice(&std::fs::read(request.path()).expect("request")).expect("JSON");
    input["request"]["libraries"] = serde_json::json!([
        {"identity": "driver/counter-invariants", "source": {"file": counter_library_path,
            "authority": "driver", "identity": "counter-fixture", "document": "CounterInvariants", "formal_revision": 1}},
        {"identity": "driver/meter-invariants", "source": {"file": library_path,
            "authority": "driver", "identity": "meter-fixture", "document": "MeterInvariants", "formal_revision": 1}}
    ]);
    input["request"]["models"].as_array_mut().expect("model selections").push(
        serde_json::json!({"format": "semantic-ir/2.0.0", "source": {"file": model_path,
            "authority": "driver", "identity": "meter-model", "document": "AuditMeter", "formal_revision": 1}}));
    input["request"]["documents"] = serde_json::json!([{"file": start_path}]);
    std::fs::write(request.path(), input.to_string()).expect("composition request");
    let (document, exit) = request.json();
    assert_eq!(exit, Some(21), "{document}");
    let items = document["items"].as_array().expect("ordered items");
    assert_eq!(items.len(), 3, "{document}");
    assert_eq!(items[0]["record"]["result"], "unsupported", "{document}");
    assert_eq!(
        items[0]["record"]["diagnostic"]["detail"]["kind"], "composition",
        "{document}"
    );
    assert_eq!(items[1]["record"]["result"], "proved", "{document}");
    assert_eq!(
        items[1]["record"]["certification"], "certified",
        "{document}"
    );
    assert!(
        items[1]["record"]["basis"].is_object(),
        "the proved premise carries its full trust basis: {document}"
    );
    assert_eq!(items[2]["record"]["result"], "refuted", "{document}");
    assert!(items[2]["record"]["certification"].is_null(), "{document}");
    let text = request.run("text");
    assert_eq!(text.status.code(), Some(21), "{text:?}");
    let text = String::from_utf8(text.stdout).expect("text output");
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.len(), 4, "{text}");
    assert!(lines[0].contains("capability=composition"), "{text}");
    assert_eq!(lines[3], "unsupported", "{text}");
    for (slot, item) in items[1..].iter().enumerate() {
        let line = lines[slot + 1];
        assert!(
            line.contains(item["record"]["result"].as_str().expect("outcome")),
            "{text}"
        );
        assert_eq!(item["record"]["backend"], "en-1", "{document}");
        assert!(line.contains("backend=en-1"), "{text}");
        assert_eq!(
            item["request"]["extent"]["classification"], "unbounded",
            "{document}"
        );
        assert_eq!(
            item["request"]["extent"]["classification"],
            items[0]["request"]["premises"][slot]["key"]["extent"]["classification"],
            "{document}"
        );
        let extent =
            ix_cli_kit::json::encode_canonical(&item["request"]["extent"], false).expect("extent");
        assert!(line.contains(&format!("extent={extent}")), "{text}");
        if let Some(basis) = item["record"].get("basis") {
            let basis = ix_cli_kit::json::encode_canonical(basis, false).expect("full basis");
            assert!(line.contains(&basis), "{text}");
        }
        if let Some(certification) = item["record"]["certification"].as_str() {
            assert!(
                line.contains(&format!("certification={certification}")),
                "{text}"
            );
        } else {
            assert!(!line.contains("certification="), "{text}");
        }
    }
    fn counter_field<'a>(
        state: &'a serde_json::Value,
        package: &str,
        object_identity: &str,
    ) -> (&'a serde_json::Value, &'a serde_json::Value) {
        assert_eq!(state["type"], "simulation-state");
        assert_eq!(state["semantic"]["type"], "map");
        let population_identity = serde_json::json!({
            "package": package, "node": format!("ix://{package}/counters")
        });
        let population = state["semantic"]["entries"]
            .as_array()
            .expect("population map")
            .iter()
            .find(|entry| {
                assert_eq!(entry["key"]["type"], "text");
                let identity: serde_json::Value = serde_json::from_str(
                    entry["key"]["value"]
                        .as_str()
                        .expect("population identity text"),
                )
                .expect("canonical population declaration identity");
                identity == population_identity
            })
            .expect("the declared population");
        assert_eq!(population["value"]["type"], "map");
        let type_identity = serde_json::json!({
            "package": package, "node": format!("ix://{package}/Counter")
        });
        let object = population["value"]["entries"]
            .as_array()
            .expect("object map")
            .iter()
            .find(|entry| {
                entry["key"]["type"] == "reference"
                    && entry["key"]["identity"] == object_identity
                    && entry["key"]["object_type"]["original"] == type_identity
            })
            .expect("the declared object identity and type");
        assert_eq!(object["value"]["type"], "record");
        let record_identity: serde_json::Value = serde_json::from_str(
            object["value"]["name"]
                .as_str()
                .expect("record declaration identity"),
        )
        .expect("canonical record declaration identity");
        assert_eq!(record_identity, type_identity);
        let field = object["value"]["fields"]
            .as_array()
            .expect("declared fields")
            .iter()
            .find(|field| field["name"] == "value")
            .expect("the Counter value field");
        assert_eq!(field["value"]["type"], "integer");
        (&field["value"], &object["key"])
    }
    let counterexample = &items[2]["counterexample"];
    assert_eq!(items[2]["replay"]["reproduced"], true, "{document}");
    let binding = &counterexample["over_binding"];
    assert_eq!(binding["type"], "reference", "{document}");
    assert_eq!(binding["identity"], "meter", "{document}");
    assert_eq!(
        binding["object_type"]["original"],
        serde_json::json!({
            "package": "driver/audit-meter", "node": "ix://driver/audit-meter/Counter"
        }),
        "{document}"
    );
    let prefix = counterexample["prefix"]
        .as_array()
        .expect("native counterexample trajectory");
    let mut deciding = Vec::new();
    let mut peer = Vec::new();
    for step in prefix {
        let state = &step["post_state"]["state"];
        let (value, reference) = counter_field(state, "driver/audit-meter", "meter");
        assert_eq!(
            reference, binding,
            "the witness follows the actual clause binding"
        );
        deciding.push(value["value"].as_str().expect("integer spelling"));
        let (value, _) = counter_field(state, "test/counter", "a");
        peer.push(value["value"].as_str().expect("peer integer spelling"));
    }
    assert_eq!(deciding, ["1", "2", "3"], "{document}");
    assert_eq!(peer, ["0", "0", "0"], "{document}");
    let final_state = &prefix.last().expect("a deciding final state")["post_state"]["state"];
    assert_eq!(
        counter_field(final_state, "driver/audit-meter", "meter").0,
        &serde_json::json!({"type": "integer", "value": "3"}),
        "{document}"
    );
    assert_eq!(
        counter_field(final_state, "test/counter", "a").0,
        &serde_json::json!({"type": "integer", "value": "0"}),
        "{document}"
    );
}

#[trace("TC-047", "FR-022-AC-11")]
#[test]
fn hybrid_process_refuses_nontrusted_plugin_certification() {
    for certification in ["uncertified", "certified"] {
        let request = hybrid_braking_request(
            &format!("hybrid-trust-{certification}"),
            serde_json::json!([{"claim": "SlowEnough", "backend": PROVIDER_BACKEND}]),
        );
        let shutdown = request.directory.0.join("shutdown");
        let config = provider_config(
            &request,
            &[
                PROVIDER_BACKEND,
                "hybrid",
                certification,
                "-",
                shutdown.to_str().expect("shutdown path"),
            ],
        );
        let document = provider_document(&analyze_with_config(&request, &config), 30);
        let item = &document["items"][0];
        assert_eq!(item["record"]["result"], "failed", "{document}");
        assert!(
            item.to_string().contains("certification-mismatch"),
            "{document}"
        );
        assert!(item.to_string().contains("/certification"), "{document}");
        assert_ne!(item["certificate"]["accepted"], true, "{document}");
        assert_eq!(
            std::fs::read(shutdown).expect("provider closed"),
            b"shutdown"
        );
    }
}
