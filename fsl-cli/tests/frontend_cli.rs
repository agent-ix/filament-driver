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
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(setting), "{stderr}");
        assert!(stderr.contains(&too_small), "{stderr}");
        assert!(stderr.contains(option), "{stderr}");
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
        OutcomeDocument::from_clause_runs(Some(OutcomeStage::S4), [(name.as_str(), &report)])
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
    let frontend_error = fsl_command::command::analyze(
        &request.path(),
        &filament_driver::AnalyzeNegotiation::fsl().expect("the engines register"),
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

const TEST_PROVIDER: &str = env!("CARGO_BIN_EXE_fsl-test-provider");
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
        "providers": [{"path": TEST_PROVIDER, "arguments": arguments}]});
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
    let library = fsl_command::command::analyze(
        &request.path(),
        &filament_driver::AnalyzeNegotiation::fsl().expect("the engines register"),
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
    assert_eq!(
        stdout.lines().next(),
        Some("0 Bounded invalid-request invalid_capability/unknown-backend"),
        "{stdout}"
    );
}

/// FSL FR-650's identity fixture, cut to the kinds the driver's `run` checks
/// here: each function returns its argument, so one call checks the argument
/// form and the result form of one kind.
const RUN_VALUE_FIXTURE: &str = r#"language "ix:native" edition "1-draft";
profile v = "filament.value.complete/v1";
type Big = Int[0, 1267650600228229401496703205376];
type R = Rational[0, 9; 1, 9];
type D = Decimal[0, 1000; 0, 2; exact];
enum Color { Red, Green }
record P { x: Int[0, 9]; y: Int[0, 9]; }
function id_bool using v(a: Boolean): Boolean pure { a }
function id_big using v(a: Big): Big pure { a }
function id_rat using v(a: R): R pure { a }
function id_dec using v(a: D): D pure { a }
function id_str using v(a: String): String pure { a }
function id_enum using v(a: Color): Color pure { a }
function id_rec using v(a: P): P pure { a }
function id_seq using v(a: Sequence<Int[0, 9]>[0, 3]): Sequence<Int[0, 9]>[0, 3] pure { a }
function id_opt using v(a: Option<Int[0, 9]>): Option<Int[0, 9]> pure { a }
"#;

/// Run `function` of [`RUN_VALUE_FIXTURE`] through `fsl run --format json`
/// with `arguments`, from a request written in its own folder.
fn run_value_fixture(label: &str, function: &str, arguments: serde_json::Value) -> Output {
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("fsl-run-value-{label}-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&directory.0);
    std::fs::create_dir_all(&directory.0).expect("a request folder");
    std::fs::write(directory.0.join("run-values.fsl"), RUN_VALUE_FIXTURE).expect("the source");
    let request = serde_json::json!({"format": "native-run/1", "request": {
        "models": [],
        "program": {"source": {"file": "run-values.fsl", "authority": "issuer",
            "identity": "document", "document": "RunValues", "formal_revision": 1}},
        "call": {"function": function, "arguments": arguments},
    }});
    let path = directory.0.join("run.json");
    std::fs::write(&path, request.to_string()).expect("the request file");
    Command::new(FSL)
        .arg("run")
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("run fsl run")
}

/// Assert `function(a = argument)` completes with `expected`, exit 0.
fn assert_run_value(function: &str, argument: serde_json::Value, expected: serde_json::Value) {
    let output = run_value_fixture(
        function,
        function,
        serde_json::json!([{"parameter": "a", "value": argument}]),
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "{function}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(document["category"], "success", "{document}");
    assert_eq!(
        document["result"],
        serde_json::json!({"kind": "completed", "value": expected}),
        "{function}: {document}"
    );
}

/// FSL FR-650-AC-1: a Boolean result is `{"boolean": b}`, from the Boolean
/// form and from the bare integer `1`.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_boolean_result() {
    let expected = serde_json::json!({"boolean": true});
    assert_run_value(
        "id_bool",
        serde_json::json!({"boolean": true}),
        expected.clone(),
    );
    assert_run_value("id_bool", serde_json::json!(1), expected);
}

/// FSL FR-650-AC-1, FR-650-AC-2: an integer result is `{"integer": "<decimal>"}`
/// at any magnitude, from a bare JSON integer too.
#[trace("TC-1700", "FR-650-AC-1")]
#[trace("TC-1701", "FR-650-AC-2")]
#[test]
fn run_returns_integer_result() {
    assert_run_value(
        "id_big",
        serde_json::json!(7),
        serde_json::json!({"integer": "7"}),
    );
    let big = serde_json::json!({"integer": "1267650600228229401496703205376"});
    assert_run_value("id_big", big.clone(), big);
}

/// FSL FR-650-AC-1: a Rational result is normalized.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_rational_result() {
    assert_run_value(
        "id_rat",
        serde_json::json!({"kind": "rational", "numerator": "4", "denominator": "6"}),
        serde_json::json!({"kind": "rational", "numerator": "2", "denominator": "3"}),
    );
}

/// FSL FR-650-AC-1: a Decimal result keeps its coefficient and scale.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_decimal_result() {
    let value = serde_json::json!({"kind": "decimal", "coefficient": "150", "scale": "2"});
    assert_run_value("id_dec", value.clone(), value);
}

/// FSL FR-650-AC-1: a String result keeps its text.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_string_result() {
    let value = serde_json::json!({"kind": "string", "text": "abc"});
    assert_run_value("id_str", value.clone(), value);
}

/// FSL FR-650-AC-1: an enumeration result is the variant form.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_enum_result() {
    let value = serde_json::json!({"variant": {"type": "Color", "case": "Green"}});
    assert_run_value("id_enum", value.clone(), value);
}

/// FSL FR-650-AC-1: a record result carries every field.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_record_result() {
    let value = serde_json::json!({"record": {"type": "P", "fields": {
        "x": {"integer": "1"}, "y": {"integer": "2"}}}});
    assert_run_value("id_rec", value.clone(), value);
}

/// FSL FR-650-AC-1: a Sequence result keeps its order and repeats.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_sequence_result() {
    let value =
        serde_json::json!({"sequence": [{"integer": "2"}, {"integer": "1"}, {"integer": "2"}]});
    assert_run_value("id_seq", value.clone(), value);
}

/// FSL FR-650-AC-1: an Option result is `present` or `absent`.
#[trace("TC-1700", "FR-650-AC-1")]
#[test]
fn run_returns_option_result() {
    let present = serde_json::json!({"present": {"integer": "3"}});
    assert_run_value("id_opt", present.clone(), present);
    let absent = serde_json::json!({"absent": {}});
    assert_run_value("id_opt", absent.clone(), absent);
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

/// FSL FR-650-AC-6: a result type with no run value form refuses
/// `unknown_required_feature`/`unsupported-feature` with exit 21, a non-null
/// cause and no result, before any call. FSL FR-286's `filament-outcome/1`
/// diagnostic has no `details` member, so the driver document names the
/// function in its message.
#[trace("TC-1705", "FR-650-AC-6")]
#[test]
#[ignore = "QSL-2182: needs a formless result reachable from source (collection over population)"]
fn unsupported_result_names_function_and_cause() {
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
    assert_eq!(output.status.code(), Some(21), "{output:?}");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(document["category"], "unsupported", "{document}");
    assert_eq!(document["operation"], "execute", "{document}");
    assert!(document["result"].is_null(), "{document}");
    let diagnostics = document["diagnostics"].as_array().expect("diagnostics");
    assert_eq!(diagnostics.len(), 1, "{document}");
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["code"], "unknown_required_feature", "{document}");
    assert_eq!(diagnostic["cause"], "unsupported-feature", "{document}");
    let message = diagnostic["message"].as_str().expect("a message");
    assert!(message.contains("function give"), "{message}");
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
/// `analyze` and `monitor` refuse the option as unknown. QSL-2279.
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
    let stderr = String::from_utf8_lossy(&unknown.stderr);
    assert!(
        stderr.contains(
            "integer-division law `ceiling` supplied by `--integer-division` is not one of \
             `euclidean`, `floor`, `truncating`"
        ),
        "{stderr}"
    );
    assert!(!stderr.contains("could not read"), "{stderr}");
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

    // `prove` is not routed by this build: it answers every invocation with
    // its `unsupported_operation` result (FSpec FR-522).
    for verb in ["run", "compile", "analyze", "monitor"] {
        let output = fsl(&[verb, "request.json", "--integer-division", "floor"]);
        assert_eq!(output.status.code(), Some(20), "{verb}: {output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("unknown") || stderr.contains("--integer-division"),
            "{verb}: {stderr}"
        );
        assert!(!stderr.contains("could not read"), "{verb}: {stderr}");
    }
}

/// FSL FR-767-AC-7, run half: `fsl run` over a request with
/// `"integer_division": "floor"` calling `q(-7, 3)` exits 0 with value -3.
#[trace("TC-2189", "FR-767-AC-7")]
#[test]
#[ignore = "QSL-1989: FSL's native-run/1 reader refuses `integer_division` as an unknown field (FR-767 run half not in language main)"]
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
                {"parameter": "a", "value": -7}, {"parameter": "b", "value": 3}]}}})
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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"-3\""),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
