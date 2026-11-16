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
    let report = fsl_replay::spine::run_clause(*request).expect("actual clause report");
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
    // The nonexistent executable proves this unsupported composition never starts a provider.
    let config_path = directory.0.join("providers.json");
    let config = serde_json::json!({"providers":[{"executable":directory.0.join("never-start-provider"),
        "argv":[],"deadline_ms":1024,"limits":{"max_frame_bytes":262144,"max_total_bytes":1048576,
            "max_stderr_bytes":4096,"cancellation_grace_ms":500}}]});
    std::fs::write(&config_path, config.to_string()).expect("provider config");
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request_path)
        .arg("--provider-config")
        .arg(&config_path)
        .args(["--format", "json"])
        .output()
        .expect("unsupported clause provider composition");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Unsupported.exit_code()))
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("typed unsupported document");
    assert_eq!(document["operation"], "monitor");
    assert_eq!(document["category"], "unsupported");
    assert_eq!(document["last_stage"], serde_json::Value::Null);
    assert_eq!(
        document["diagnostics"][0]["code"],
        "unimplemented_capability"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("clause"), "{stderr}");
    assert!(!stderr.contains("could not start"), "{stderr}");
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
        &filament_driver::AnalyzeNegotiation,
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
#[trace("TC-036", "FR-022-AC-8")]
#[test]
fn analyze_routes_through_engines_registry() {
    let request = AnalyzeRequest::new("registry", one_item("Bounded"), serde_json::json!({}));
    let (document, _) = request.json();
    let backends: Vec<&str> = document["registry"]
        .as_array()
        .unwrap_or_else(|| panic!("a registry member: {document}"))
        .iter()
        .filter_map(serde_json::Value::as_str)
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
    // The option FSL FR-287 gives `analyze` is refused, naming the missing
    // caller registry, before any file is read.
    let output = fsl(&["analyze", "missing.json", "--provider-config", "p.json"]);
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("`--provider-config` is not available for `analyze` yet"),
        "{stderr}"
    );
    assert!(stderr.contains("caller registry"), "{stderr}");
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
