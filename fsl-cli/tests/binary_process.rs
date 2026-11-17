// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for CLI run options and FSL results.

#[cfg(all(unix, feature = "tc018-test-adapter"))]
use std::fs::File;
#[cfg(all(unix, feature = "tc018-test-adapter"))]
use std::io::Read;
#[cfg(all(unix, feature = "tc018-test-adapter"))]
use std::io::Write;
#[cfg(all(unix, feature = "tc018-test-adapter"))]
use std::process::Child;
use std::process::Command;
#[cfg(unix)]
use std::process::{ExitStatus, Stdio};
#[cfg(all(unix, feature = "tc018-test-adapter"))]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(all(unix, feature = "tc018-test-adapter"))]
use std::time::{Duration, Instant};

use filament_driver::{execute, ExecutionEngine, ExecutionRequest, ObjectEnvironment};
use filament_exact::Cancel;
use filament_semantic_value::catalog::Category;
use filament_semantic_value::supplied_stop::SuppliedLimits;
use fsl_foundation::diagnostic::Code;
use fsl_foundation::SourceIdentity;
#[cfg(all(unix, feature = "tc018-test-adapter"))]
use fsl_replay::spine::{check, parse, select, ParseRequest};
use fsl_replay::spine::{
    default_accounting, Call, DependencyInput, SpineLimits, DEFAULT_WORK_UNITS,
};
use fsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};

#[cfg(all(unix, feature = "tc018-test-adapter"))]
#[path = "../../tests/common/units.rs"]
mod shared_units;

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.fsl");

const RUN_REQUEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-run.json"
);

const STAMPED_PROFILE: &str = "filament.temporal.timestamped-event.finite-window/v1";

fn write_run_fixture(
    directory: &std::path::Path,
    source: &str,
    request: serde_json::Value,
) -> std::path::PathBuf {
    let source_path = directory.join("program.native");
    std::fs::write(&source_path, source).expect("write run program");
    let request_path = directory.join("run.json");
    std::fs::write(
        &request_path,
        serde_json::to_vec(&request).expect("encode run request"),
    )
    .expect("write run request");
    request_path
}

fn finite_window_request(
    directory: &std::path::Path,
    watermark: u64,
    complete: bool,
) -> std::path::PathBuf {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile t = \"{STAMPED_PROFILE}\";\n\
         record Reading {{ level: Int[0, 9]; }}\n\
         temporal Seen using t over (r: Reading) clock \"plant-ms\" on origin {{\n\
           eventually[0,5] holds(r.level > 0)\n\
         }}\n"
    );
    let clocks = serde_json::json!({
        "plant-ms": {"kind": "timestamped_event", "timestamp_unit": "ms"}
    });
    let stamps = if watermark >= 5 {
        vec![0_u64, 5]
    } else {
        vec![0_u64]
    };
    let observations = stamps
        .into_iter()
        .map(|stamp| {
            serde_json::json!({
                "position": {"sample": {"level": {"integer": "0"}}},
                "stamp": stamp,
            })
        })
        .collect::<Vec<_>>();
    let progress = if complete {
        serde_json::json!({
            "watermark": watermark,
            "completeness": {"complete": {"through": watermark}},
            "scope": "closed", "execution": "closed"
        })
    } else {
        serde_json::json!({
            "watermark": watermark,
            "completeness": "incomplete",
            "scope": "closed", "execution": "closed"
        })
    };
    let request = serde_json::json!({
        "format": "native-run/1",
        "request": {
            "models": [],
            "program": {"source": {
                "file": "program.native", "authority": "test", "identity": "test:finite-window",
                "document": "FiniteWindow", "formal_revision": 1
            }},
            "clocks": clocks,
            "clause": {
                "selection": {"temporal": {
                    "name": "Seen",
                    "trace": {"timestamped": {
                        "observations": observations,
                        "progress": progress,
                        "lower_boundary": "authoritative-origin",
                        "binding": {
                            "profile": STAMPED_PROFILE, "clock": "plant-ms",
                            "parameters": clocks["plant-ms"]
                        }
                    }}
                }},
                "observations": {}
            }
        }
    });
    write_run_fixture(directory, &source, request)
}

fn non_finite_rational_request(directory: &std::path::Path) -> std::path::PathBuf {
    let source = "language \"ix:native\" edition \"1-draft\";\n\
        profile v = \"filament.value.complete/v1\";\n\
        type R = Rational[-9, 9; 1, 9];\n\
        function as_rational using v(x: Float64[nearest-even]): R pure { convert<R>(x) }\n";
    let request = serde_json::json!({
        "format": "native-run/1",
        "request": {
            "models": [],
            "program": {"source": {
                "file": "program.native", "authority": "test", "identity": "test:non-finite",
                "document": "NonFinite", "formal_revision": 1
            }},
            "call": {"function": "as_rational", "arguments": [{
                "parameter": "x", "value": {"kind": "float64", "bits": "7ff8000000000000"}
            }]}
        }
    });
    write_run_fixture(directory, source, request)
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
static TEST_DIRECTORY_ID: AtomicUsize = AtomicUsize::new(0);

#[cfg(all(unix, feature = "tc018-test-adapter"))]
struct TestDirectory(std::path::PathBuf);

#[cfg(all(unix, feature = "tc018-test-adapter"))]
impl TestDirectory {
    fn new(label: &str) -> Self {
        let id = TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("fsl-cli-{label}-{}-{id}", std::process::id()));
        std::fs::create_dir(&path).expect("create isolated test directory");
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
struct ChildGuard(Option<Child>);

#[cfg(all(unix, feature = "tc018-test-adapter"))]
impl ChildGuard {
    fn new(child: Child) -> Self {
        Self(Some(child))
    }

    fn child(&mut self) -> &mut Child {
        self.0.as_mut().expect("child remains owned until reaped")
    }

    fn kill_and_wait(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.0 = None;
    }
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill_and_wait();
    }
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
fn wait_for_child_until(child: &mut ChildGuard, deadline: Instant) -> ExitStatus {
    loop {
        if let Some(status) = child.child().try_wait().expect("poll child status") {
            return status;
        }
        if Instant::now() >= deadline {
            child.kill_and_wait();
            panic!("child did not exit before the TC-018 deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
fn overflow_fixture_bytes() -> Vec<u8> {
    shared_units::overflow_unit().into_bytes()
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
fn assert_tc018_refused_range_observation(path: &std::path::Path, source_path: &str, bytes: &[u8]) {
    let observation: serde_json::Value = serde_json::from_slice(
        &std::fs::read(path).expect("the test-only operation adapter records its request"),
    )
    .expect("the adapter observation is JSON");
    assert_eq!(observation["case"], "refused-range");
    assert_eq!(observation["path"], source_path);
    assert_eq!(
        observation["source_identity"],
        serde_json::json!({
            "authority": "filament",
            "identity": source_path,
        })
    );
    assert_eq!(observation["source_bytes"], serde_json::json!(bytes));
    assert!(
        observation["profile"]
            .as_str()
            .is_some_and(|profile| profile.starts_with("profile v = ")),
        "the observation retains the selected profile: {observation}"
    );
    assert!(
        observation["limits"]
            .as_str()
            .is_some_and(|limits| limits.contains("SpineLimits")),
        "the observation retains the FSL limits: {observation}"
    );
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
fn expected_overflow_check_document(path: &str, bytes: &[u8]) -> Vec<u8> {
    let source = SourceIdentity::new("filament", path);
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let packages = std::collections::BTreeMap::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes,
        },
        limits.source,
        &cancel,
    )
    .and_then(|parsed| {
        select(parsed.value(), &packages, limits.model, &cancel).and_then(|models| {
            check(
                parsed.value(),
                path.to_owned(),
                models.value(),
                &DependencyInput::default(),
                limits,
                &cancel,
            )
        })
    });
    OutcomeDocument::from_check(&result)
        .to_bytes()
        .expect("FSL serializes the expected check refusal")
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
fn open_fifo_writer_until(
    path: &std::path::Path,
    child: &mut ChildGuard,
    deadline: Instant,
) -> File {
    use std::os::unix::fs::OpenOptionsExt;

    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(writer) => return writer,
            Err(error)
                if error.raw_os_error() == Some(libc::ENXIO)
                    || error.raw_os_error() == Some(libc::EINTR) =>
            {
                if let Some(status) = child.child().try_wait().expect("poll FIFO child") {
                    panic!("child exited before the source FIFO opened: {status:?}");
                }
                if Instant::now() >= deadline {
                    panic!("source FIFO writer did not open before the TC-018 deadline");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("open source FIFO writer: {error}"),
        }
    }
}

#[cfg(all(unix, feature = "tc018-test-adapter"))]
fn read_child_stderr(child: &mut ChildGuard) -> Vec<u8> {
    let mut stderr = Vec::new();
    child
        .child()
        .stderr
        .take()
        .expect("child stderr remains piped")
        .read_to_end(&mut stderr)
        .expect("read child stderr after exit");
    stderr
}

#[cfg(unix)]
fn status_with_unwritable_stderr(arguments: &[&str], reserved_exit: bool) -> ExitStatus {
    let mut command = Command::new(FSL);
    command.args(arguments).stdout(Stdio::null());
    if reserved_exit {
        command.env("TC018_CASE", "reserved-exit");
    }
    if let Ok(full) = std::fs::OpenOptions::new().write(true).open("/dev/full") {
        return command
            .stderr(Stdio::from(full))
            .status()
            .expect("run fsl with /dev/full as stderr");
    }

    let mut child = command
        .stderr(Stdio::piped())
        .spawn()
        .expect("run fsl with a pipe for stderr");
    drop(child.stderr.take());
    child.wait().expect("wait for fsl after closing stderr")
}

fn expected_document_bytes(code: Code, message: &str) -> Vec<u8> {
    let document =
        OutcomeDocument::new(Operation::Execute, None, code.category()).with_diagnostics(vec![
            OutcomeDiagnostic::new(None, code.as_str(), None, message),
        ]);
    let mut bytes =
        ix_cli_kit::json::encode_canonical(&document, false).expect("encode expected FSL document");
    bytes.push('\n');
    bytes.into_bytes()
}

/// Trace: QSL-2185; a violated finite-window clause exits through the binary as 10.
#[test]
fn violation_exits_ten() {
    let directory = tempfile::tempdir().expect("create finite-window fixture directory");
    let request = finite_window_request(directory.path(), 5, true);
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run finite-window violation");
    assert_eq!(
        output.status.code(),
        Some(10),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("violation document is JSON");
    assert_eq!(document["category"], "violation");
}

/// Trace: QSL-2185; converting a user-supplied IEEE NaN to Rational is undefined.
#[test]
fn undefined_exits_ten() {
    let directory = tempfile::tempdir().expect("create non-finite fixture directory");
    let request = non_finite_rational_request(directory.path());
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run non-finite rational conversion");
    assert_eq!(
        output.status.code(),
        Some(10),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("undefined document is JSON");
    assert_eq!(document["category"], "undefined", "{document}");
    assert_eq!(document["result"]["kind"], "undefined", "{document}");
    assert_eq!(
        document["result"]["reason"], "ieee-not-finite",
        "{document}"
    );
}

/// Trace: QSL-2185; a finite-window run that ends before its deadline exits 11.
#[test]
fn indeterminate_exits_eleven() {
    let directory = tempfile::tempdir().expect("create finite-window fixture directory");
    let request = finite_window_request(directory.path(), 5, false);
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run finite-window request before its deadline");
    assert_eq!(output.status.code(), Some(11));
}

/// Trace: QSL-2185; violation, refusal and incomplete categories keep distinct exit codes.
#[test]
fn exit_codes_are_distinct_per_category() {
    let directory = tempfile::tempdir().expect("create finite-window fixture directory");
    let request = finite_window_request(directory.path(), 5, true);
    let violation = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run finite-window violation");
    assert_eq!(violation.status.code(), Some(10));

    let refusal = Command::new(FSL)
        .args(["run", "missing-request.json", "--format", "json"])
        .output()
        .expect("run missing request");
    assert_eq!(refusal.status.code(), Some(20));

    let incomplete = Command::new(FSL)
        .args([
            "run",
            RUN_REQUEST,
            "--limit",
            "accounting.work_units=0",
            "--format",
            "json",
        ])
        .output()
        .expect("run request with an exhausted accounting budget");
    assert_eq!(incomplete.status.code(), Some(22));
    assert_ne!(refusal.status.code(), violation.status.code());
    assert_ne!(incomplete.status.code(), violation.status.code());
}

/// Trace: FR-004-AC-1, omitted engine selects the linked interpreter.
#[test]
fn run_without_engine_executes_with_the_interpreter() {
    let output = Command::new(FSL)
        .args(["run", RUN_REQUEST, "--format", "json"])
        .output()
        .expect("run fsl");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-002-AC-3, FR-002-AC-9; JSON is the unchanged FSL outcome encoded by the kit.
#[test]
fn machine_run_writes_the_canonical_fsl_outcome() {
    let output = Command::new(FSL)
        .args(["run", RUN_REQUEST, "--format", "json"])
        .output()
        .expect("run fsl in machine mode");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let packages = std::collections::BTreeMap::new();
    let dependencies = DependencyInput::default();
    let source_bytes = std::fs::read(SOURCE).expect("read source fixture");
    let request = ExecutionRequest {
        source: SourceIdentity::new("issuer", "document"),
        package_name: "document",
        path: SOURCE,
        bytes: &source_bytes,
        packages: &packages,
        dependencies: &dependencies,
        limits: SpineLimits::default(),
        call: Call {
            function: "yes".to_owned(),
            arguments: Vec::new(),
            accounting: default_accounting(DEFAULT_WORK_UNITS),
            supplied: SuppliedLimits::default(),
            backend: Default::default(),
            clocks: Default::default(),
            wire_arguments: Vec::new(),
            snapshot: None,
        },
        object_environment: &ObjectEnvironment::default(),
        cancel: Cancel::new(),
        engine: ExecutionEngine::Interpreter,
    };
    let result = execute(request).expect("obtain typed FSL result");
    let expected = ix_cli_kit::json::encode_canonical(&result.document, false)
        .expect("encode typed outcome through kit");
    assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
    let fsl_bytes = result
        .outcome_bytes()
        .expect("encode typed outcome through FSL");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse child outcome"),
        serde_json::from_slice::<serde_json::Value>(&fsl_bytes).expect("parse FSL outcome bytes")
    );
}

/// Trace: FR-002-AC-3, FR-002-AC-8, FR-002-AC-9; pre-FSL request refusals use FSL's catalog code.
#[test]
fn malformed_run_limit_emits_one_fsl_outcome_and_streams_diagnostic() {
    let output = Command::new(FSL)
        .args([
            "run",
            "missing-request.json",
            "--limit",
            "s1.input_bytes=invalid",
            "--format",
            "json",
        ])
        .output()
        .expect("run fsl with malformed limit");
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("s1.input_bytes=invalid"));
    assert!(stderr.ends_with('\n'));
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, stderr.trim_end())
    );
}

/// Trace: FR-002-AC-3, FR-002-AC-8, FR-002-AC-9; source I/O errors retain FSL's native code.
#[test]
fn missing_source_emits_fsl_io_error_document_and_diagnostic() {
    let output = Command::new(FSL)
        .args(["run", "missing-request.json", "--format", "json"])
        .output()
        .expect("run fsl with a missing source");
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot read missing-request.json:"));
    assert!(stderr.ends_with('\n'));
    let error = fsl_command::command::prepare_run(
        std::path::Path::new("missing-request.json"),
        fsl_command::command::RunLimits::default(),
    )
    .err()
    .expect("owner refuses missing request");
    let expected =
        ix_cli_kit::json::encode_canonical(&error.outcome_document(Operation::Execute), false)
            .expect("encode owner refusal");
    // OS error wording is owned by the same intake operation.
    assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
}

/// FSpec FR-561 and FR-006-AC-24: each refusal exits 20 with empty stdout,
/// in text and machine mode alike, and names the file, the member and the
/// code on stderr.
fn assert_provider_config_refusal(path: &std::path::Path, expected: &str) {
    for format in ["json", "text"] {
        let output = Command::new(FSL)
            .args(["run", RUN_REQUEST, "--provider-config"])
            .arg(path)
            .args(["--format", format])
            .output()
            .expect("run fsl with provider config");
        assert_eq!(output.status.code(), Some(20), "{format}");
        assert!(output.stdout.is_empty(), "{format}: stdout is empty");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("provider configuration"), "{stderr}");
        assert!(stderr.contains(expected), "{stderr}");
        assert!(
            stderr.contains(path.display().to_string().as_str()),
            "the refusal names `{}`: {stderr}",
            path.display()
        );
        let code = if expected == "unknown_wire/unsupported-wire" {
            "unknown_wire/unsupported-wire"
        } else {
            "invalid-request"
        };
        assert!(stderr.contains(code), "{stderr}");
    }
}

/// Trace: QSL-2192 / FR-006-AC-24; missing members and paths retain typed execute refusals.
#[test]
fn provider_config_missing_member_refused() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "fsl-cli-provider-config-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create isolated provider config directory");
    let missing = directory.join("missing.json");
    assert_provider_config_refusal(&missing, "could not read");

    let cases = [
        (
            "malformed.json",
            "{ malformed",
            "invalid provider configuration",
        ),
        ("missing-format.json", r#"{"providers":[]}"#, "format"),
        (
            "missing-providers.json",
            r#"{"format":"filament.provider-config/v1"}"#,
            "providers",
        ),
        (
            "unknown-top-level.json",
            r#"{"format":"filament.provider-config/v1","providers":[],"extra":1}"#,
            "unknown field",
        ),
        (
            "missing-path.json",
            r#"{"format":"filament.provider-config/v1","providers":[{}]}"#,
            "path",
        ),
        (
            "blank-path.json",
            r#"{"format":"filament.provider-config/v1","providers":[{"path":" "}]}"#,
            "blank path",
        ),
        (
            "bad-arguments.json",
            r#"{"format":"filament.provider-config/v1","providers":[{"path":"p","arguments":[1]}]}"#,
            "arguments",
        ),
        (
            "unsupported-format.json",
            r#"{"format":"filament.provider-config/v9","providers":[]}"#,
            "unknown_wire/unsupported-wire",
        ),
    ];
    for (name, contents, expected) in cases {
        let path = directory.join(name);
        std::fs::write(&path, contents).expect("write provider config fixture");
        assert_provider_config_refusal(&path, expected);
    }
    std::fs::remove_dir_all(directory).expect("remove isolated provider config directory");
}

/// Trace: FSpec FR-561 / FSL FR-287; the empty config document is valid.
#[test]
fn empty_provider_config_executes_without_starting_a_provider() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "fsl-cli-empty-provider-config-{}-{unique}.json",
        std::process::id()
    ));
    std::fs::write(
        &path,
        r#"{"format":"filament.provider-config/v1","providers":[]}"#,
    )
    .expect("write empty provider config");
    let output = Command::new(FSL)
        .args(["run", RUN_REQUEST, "--provider-config"])
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("run fsl with empty provider config");
    let _ = std::fs::remove_file(path);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(serde_json::from_slice::<serde_json::Value>(&output.stdout).is_ok());
}

/// Trace: FSpec FR-561-AC-2; a plugin path that does not exist refuses
/// naming it, exit 20 with empty stdout, before any plugin starts. QSL-2192.
#[test]
fn provider_config_naming_a_missing_plugin_is_refused() {
    let directory = tempfile::tempdir().expect("create provider config directory");
    let path = directory.path().join("providers.json");
    std::fs::write(
        &path,
        r#"{"format":"filament.provider-config/v1","providers":[{"path":"/must-not-start"}]}"#,
    )
    .expect("write valid provider config");
    assert_provider_config_refusal(&path, "`/must-not-start` does not exist");
}

const TEST_PROVIDER: &str = env!("CARGO_BIN_EXE_fsl-test-provider");
const PROVIDER_BACKEND: &str = "fsl.test-process-provider";

/// A native-run/1 request in `directory` calling `Even(3)` with
/// `call.requirements` naming the test provider (FSL FR-960), and the
/// FSpec FR-561 configuration starting `providers` test-provider processes,
/// each with its own arguments.
fn provider_run(
    directory: &std::path::Path,
    providers: &[Vec<String>],
) -> (std::path::PathBuf, std::path::PathBuf) {
    let source = directory.join("even.native");
    std::fs::write(
        &source,
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"filament.value.complete/v1\";\n\
         function Even using v(n: Integer): Boolean pure { n * 2 - n = n }\n",
    )
    .expect("write source fixture");
    let request = directory.join("run.json");
    std::fs::write(
        &request,
        serde_json::json!({"format": "native-run/1", "request": {"models": [],
            "program": {"source": {"file": source, "authority": "agent-ix",
                "identity": "test:provider-run", "document": "ProviderRun",
                "formal_revision": 1}},
            "call": {"function": "Even", "arguments": [{"parameter": "n", "value": 3}],
                "requirements": {"backend": PROVIDER_BACKEND}}}})
        .to_string(),
    )
    .expect("write run request");
    let config = directory.join("providers.json");
    let entries = providers
        .iter()
        .map(|arguments| serde_json::json!({"path": TEST_PROVIDER, "arguments": arguments}))
        .collect::<Vec<_>>();
    std::fs::write(
        &config,
        serde_json::json!({"format": "filament.provider-config/v1", "providers": entries})
            .to_string(),
    )
    .expect("write provider config");
    (request, config)
}

/// Trace: FR-006-AC-1, FR-006-AC-3, FR-006-AC-8; `fsl run` sends the call's
/// requirement item to the configured provider as one FR-331 request under
/// the run's deadline, settles it on the provider, and on normal completion
/// sends shutdown and reaps the provider. QSL-2192.
#[test]
fn configured_process_normal_exit_shuts_down_active_session() {
    let directory = tempfile::tempdir().expect("create isolated normal-exit test directory");
    let request_marker = directory.path().join("request.json");
    let shutdown_marker = directory.path().join("shutdown");
    let (request, config) = provider_run(
        directory.path(),
        &[vec![
            PROVIDER_BACKEND.to_owned(),
            "unbounded".to_owned(),
            "proved".to_owned(),
            request_marker.display().to_string(),
            shutdown_marker.display().to_string(),
        ]],
    );
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .arg("--provider-config")
        .arg(&config)
        .args(["--provider-limit", "deadline_ms=4096", "--format", "json"])
        .output()
        .expect("run production fsl binary");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let sent: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&request_marker).expect("the provider received the routed request"),
    )
    .expect("the request body is JSON");
    assert_eq!(sent["request"]["deadline_ms"], 4096, "{sent}");
    assert_eq!(
        sent["request"]["items"][0]["backend"], PROVIDER_BACKEND,
        "{sent}"
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("the execute outcome");
    let item = &document["items"][0];
    assert_eq!(item["record"]["result"], "proved", "{document}");
    assert_eq!(item["record"]["backend"], PROVIDER_BACKEND, "{document}");
    assert!(
        shutdown_marker.is_file(),
        "the CLI sent shutdown and waited for the provider to exit"
    );
}

/// Trace: FR-006-AC-25, FR-006-AC-1, FR-006-AC-3; both configured providers
/// start and are admitted before negotiation, each under its own admitted
/// identity; only the provider the call's requirements select receives a
/// request, its typed `proved` result is retained on the settled item, and
/// both sessions are shut down. QSL-2192.
#[test]
fn configured_run_invokes_only_the_selected_process_and_retains_its_typed_result() {
    let directory = tempfile::tempdir().expect("create isolated two-provider test directory");
    let identities = [PROVIDER_BACKEND, "fsl.test-process-provider-zeta"];
    let request_markers = [
        directory.path().join("request-selected"),
        directory.path().join("request-other"),
    ];
    let shutdown_markers = [
        directory.path().join("shutdown-selected"),
        directory.path().join("shutdown-other"),
    ];
    let providers = (0..2)
        .map(|index| {
            vec![
                identities[index].to_owned(),
                "unbounded".to_owned(),
                "proved".to_owned(),
                request_markers[index].display().to_string(),
                shutdown_markers[index].display().to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let (request, config) = provider_run(directory.path(), &providers);
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .arg("--provider-config")
        .arg(&config)
        .args(["--format", "json"])
        .output()
        .expect("run production fsl binary with two providers");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let sent: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&request_markers[0]).expect("the selected provider received a request"),
    )
    .expect("the request body is JSON");
    assert_eq!(
        sent["request"]["items"][0]["backend"], PROVIDER_BACKEND,
        "{sent}"
    );
    let manifest: Vec<&str> = sent["manifest"]
        .as_array()
        .expect("the request's registry snapshot")
        .iter()
        .filter_map(|entry| entry["backend"].as_str())
        .collect();
    assert!(
        identities
            .iter()
            .all(|identity| manifest.contains(identity)),
        "both admitted providers are registered before negotiation: {manifest:?}"
    );
    assert!(
        !request_markers[1].exists(),
        "the unselected provider records no invocation"
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("the execute outcome");
    let items = document["items"].as_array().expect("settled items");
    assert!(!items.is_empty(), "{document}");
    for item in items {
        assert_eq!(item["record"]["result"], "proved", "{document}");
        assert_eq!(item["record"]["backend"], PROVIDER_BACKEND, "{document}");
    }
    assert!(shutdown_markers.iter().all(|marker| marker.is_file()));
}

/// Trace: FR-006-AC-26; duplicate admitted identities refuse before any
/// request and close both sessions. QSL-2192.
#[test]
fn duplicate_provider_identities_refuse_before_negotiation_and_close_sessions() {
    let directory = tempfile::tempdir().expect("create isolated duplicate test directory");
    let request_markers = [
        directory.path().join("request-one"),
        directory.path().join("request-two"),
    ];
    let shutdown_markers = [
        directory.path().join("shutdown-one"),
        directory.path().join("shutdown-two"),
    ];
    let providers = (0..2)
        .map(|index| {
            vec![
                PROVIDER_BACKEND.to_owned(),
                "unbounded".to_owned(),
                "proved".to_owned(),
                request_markers[index].display().to_string(),
                shutdown_markers[index].display().to_string(),
            ]
        })
        .collect::<Vec<_>>();
    let (request, config) = provider_run(directory.path(), &providers);
    let output = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .arg("--provider-config")
        .arg(&config)
        .args(["--format", "json"])
        .output()
        .expect("run production fsl binary with duplicate identities");
    let message =
        format!("configured process providers repeat admitted identity `{PROVIDER_BACKEND}`");
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, &message)
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), message);
    assert!(request_markers.iter().all(|marker| !marker.exists()));
    assert!(shutdown_markers.iter().all(|marker| marker.is_file()));
}

/// Trace: FR-006-AC-4; the production binary cancels and reaps a selected provider before re-raising SIGTERM.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn configured_process_sigterm_cleans_active_session_before_reraise() {
    assert_configured_process_signal_cleans_active_session(
        signal_hook::consts::signal::SIGTERM,
        "term",
    );
}

/// Trace: FR-006-AC-4; the production binary cancels and reaps a selected provider before re-raising SIGINT.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn configured_process_sigint_cleans_active_session_before_reraise() {
    assert_configured_process_signal_cleans_active_session(
        signal_hook::consts::signal::SIGINT,
        "int",
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn assert_configured_process_signal_cleans_active_session(signal_number: i32, mode: &str) {
    use std::os::unix::process::ExitStatusExt;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let directory = tempfile::tempdir().expect("create isolated signal test directory");
    let request_marker = directory.path().join("request.json");
    let ready_marker = directory.path().join("descendant-ready");
    let descendant_marker = directory.path().join("descendant-leaked");
    let (request, config) = provider_run(
        directory.path(),
        &[vec![
            PROVIDER_BACKEND.to_owned(),
            "unbounded".to_owned(),
            mode.to_owned(),
            request_marker.display().to_string(),
            "-".to_owned(),
            ready_marker.display().to_string(),
            descendant_marker.display().to_string(),
        ]],
    );
    let started = Instant::now();
    let mut child = Command::new(FSL)
        .arg("run")
        .arg(&request)
        .arg("--provider-config")
        .arg(&config)
        .args(["--provider-limit", "grace_ms=250", "--format", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start production fsl binary");
    let deadline = started + Duration::from_secs(5);
    loop {
        if child.try_wait().expect("poll fsl child").is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .expect("collect timed-out fsl child");
            panic!(
                "production signal flow did not exit within its bound: status={:?}, stdout={}, stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let output = child
        .wait_with_output()
        .expect("collect signalled fsl child");

    assert!(
        request_marker.is_file(),
        "the provider received a real FR-331 request"
    );
    assert!(
        ready_marker.is_file(),
        "descendant reached its ready handshake"
    );
    assert_eq!(
        output.status.signal(),
        Some(signal_number),
        "the production CLI restores the default action and re-raises the signal"
    );
    assert!(
        output.stdout.is_empty(),
        "signal cleanup emits no outcome document"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "cleanup remains bounded"
    );
    std::thread::sleep(Duration::from_secs(4));
    assert!(
        !descendant_marker.exists(),
        "the process-group descendant was reaped before the CLI re-raised the signal"
    );
}

/// Trace: FR-002-AC-1, FR-002-AC-8; root argv refusals name the rejected argument.
#[test]
fn root_argv_refusal_names_invalid_format_without_result_output() {
    let output = Command::new(FSL)
        .args(["run", RUN_REQUEST, "--format", "dot"])
        .output()
        .expect("run fsl with an invalid root format");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("dot"));
}

/// Trace: FR-002-AC-1, FR-002-AC-8; run-only options are named when refused on another verb.
#[test]
fn non_run_verb_refuses_and_names_run_only_option() {
    for option in ["--engine", "--limit"] {
        let output = Command::new(FSL)
            .args(["check", option, "gpu"])
            .output()
            .expect("run fsl with a run-only option on check");
        assert_eq!(output.status.code(), Some(20), "{option}");
        assert!(output.stdout.is_empty(), "{option}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(option),
            "{option}"
        );
    }
}

/// Trace: FR-004-AC-3, unavailable engines refuse before reading a source.
#[test]
fn unavailable_engines_are_not_substituted() {
    for engine in ["aot", "jit"] {
        let output = Command::new(FSL)
            .args([
                "run",
                "missing-request.json",
                "--engine",
                engine,
                "--format",
                "json",
            ])
            .output()
            .expect("run fsl");
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::Unsupported.exit_code())),
            "{engine}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(engine),
            "typed unsupported refusal names {engine}"
        );
        let document: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("unsupported result is FSL JSON");
        assert_eq!(document["category"], "unsupported");
        assert_eq!(
            document["diagnostics"][0]["code"],
            "unimplemented_capability"
        );
    }
}

/// Trace: FR-004-AC-2, a value outside the engine set is refused.
#[test]
fn an_unknown_engine_is_a_refusal() {
    let output = Command::new(FSL)
        .args(["run", "missing-request.json", "--engine", "gpu"])
        .output()
        .expect("run fsl");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Refusal.exit_code()))
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("gpu"));
}

/// Trace: FR-003-AC-1, the exposed checking work bound reaches FSL unchanged.
#[test]
fn checking_work_limit_changes_the_call_outcome() {
    let exhausted = Command::new(FSL)
        .args([
            "run",
            RUN_REQUEST,
            "--limit",
            "s3.work_units=0",
            "--format",
            "json",
        ])
        .output()
        .expect("run with exhausted work limit");
    assert_eq!(
        exhausted.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );

    let sufficient = Command::new(FSL)
        .args([
            "run",
            RUN_REQUEST,
            "--limit",
            "s3.work_units=1000",
            "--format",
            "json",
        ])
        .output()
        .expect("run with sufficient work limit");
    assert_eq!(
        sufficient.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-003-AC-9, FR-003-AC-10. FSL-named accounting reaches FSL's call.
#[test]
fn named_accounting_work_units_limit_reaches_fsl_call_accounting() {
    let output = Command::new(FSL)
        .args([
            "run",
            RUN_REQUEST,
            "--limit",
            "accounting.work_units=0",
            "--format",
            "json",
        ])
        .output()
        .expect("run with a zero accounting budget");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("FSL writes an incomplete document");
    assert_eq!(document["category"], "incomplete");
    assert_eq!(document["result"]["kind"], "incomplete");
    assert_eq!(
        document["result"]["limit"]["setting"],
        "accounting.work_units"
    );
}

/// Trace: FR-003-AC-3 / TC-010, a lifecycle stage limit uses FSL's exit code.
#[test]
fn lifecycle_stage_limit_uses_incomplete_exit_code() {
    let output = Command::new(FSL)
        .args([
            "run",
            RUN_REQUEST,
            "--limit",
            "s1.input_bytes=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run with an exhausted source-byte limit");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );

    let source_bytes = std::fs::read(SOURCE).expect("read fixture to set exact source bound");
    let bound = format!("s1.input_bytes={}", source_bytes.len());
    let raised = Command::new(FSL)
        .args(["run", RUN_REQUEST, "--limit", &bound, "--format", "json"])
        .output()
        .expect("run with the source bound raised to the fixture length");
    assert_eq!(
        raised.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-003-AC-2, FR-003-AC-7, FR-003-AC-8.
#[test]
fn fsl_rejects_unknown_malformed_and_duplicate_limits_before_source_read() {
    for operand in [
        "work_units=0", // A bare counter is not a setting (FSL FR-255 Behavior 8).
        "normalization.work_units=1",
        "s1.input_bytes=invalid",
        "s1.input_bytes=1", // duplicated below using the same option twice
    ] {
        let mut command = Command::new(FSL);
        command.args(["run", "missing-request.json", "--limit", operand]);
        if operand == "s1.input_bytes=1" {
            command.args(["--limit", operand]);
        }
        let output = command.output().expect("run with invalid FSL limit");
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::Refusal.exit_code())),
            "{operand}"
        );
        assert!(output.stdout.is_empty(), "{operand}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(operand), "{operand}: {stderr}");
        if operand == "work_units=0" {
            assert!(stderr.contains("UnknownSetting"), "{stderr}");
        }
        assert!(
            !stderr.contains("could not read source"),
            "{operand}: {stderr}"
        );
    }
}

/// Trace: FR-003-AC-5, FSL-recognized settings absent from run bounds are distinct refusals.
#[test]
fn recognized_setting_unavailable_for_run_is_named_before_source_read() {
    let output = Command::new(FSL)
        .args([
            "run",
            "missing-request.json",
            "--limit",
            "explore.states=2",
            "--format",
            "json",
        ])
        .output()
        .expect("run with a setting unavailable to run");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Refusal.exit_code()))
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("explore.states"), "{stderr}");
    assert!(
        stderr.contains("unavailable for the `run` operation"),
        "{stderr}"
    );
    assert!(!stderr.contains("could not read source"), "{stderr}");
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, stderr.trim_end())
    );
}

/// Trace: FR-003-AC-2, parser refusals take precedence over operation bounds.
#[test]
fn invalid_operand_precedes_recognized_but_unavailable_setting() {
    let output = Command::new(FSL)
        .args([
            "run",
            "missing-request.json",
            "--limit",
            "explore.states=2",
            "--limit",
            "normalization.work_units=1",
        ])
        .output()
        .expect("run with an unknown operand and unavailable setting");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("normalization.work_units"), "{stderr}");
    assert!(
        !stderr.contains("unavailable for the `run` operation"),
        "{stderr}"
    );
    assert!(!stderr.contains("could not read source"), "{stderr}");
}

/// Trace: FR-002-AC-8; a full stderr device does not replace a refusal exit with a panic.
#[cfg(unix)]
#[test]
fn refusal_keeps_its_exit_code_when_stderr_is_unwritable() {
    let status = status_with_unwritable_stderr(&["run", RUN_REQUEST, "--format", "dot"], false);
    assert_eq!(status.code(), Some(20));
}

/// Trace: FR-002-AC-14; a broken primary stdout pipe preserves the FSL category result.
#[cfg(all(unix, feature = "tc018-test-adapter"))]
#[test]
fn fifo_broken_pipe_keeps_refusal_exit_without_output_failure_diagnostic() {
    let directory = TestDirectory::new("tc018-broken-pipe");
    let source_path = directory.path().join("driver-check.native");
    let source_text = source_path.to_str().expect("temporary path is UTF-8");
    let fifo_status = Command::new("mkfifo")
        .arg(&source_path)
        .status()
        .expect("create named-pipe source fixture");
    assert!(fifo_status.success(), "mkfifo exited with {fifo_status}");

    let bytes = overflow_fixture_bytes();
    let expected = expected_overflow_check_document(source_text, &bytes);
    let request_path = directory.path().join("tc018-request.json");
    assert!(
        expected
            .windows(b"undefined_expression".len())
            .any(|window| window == b"undefined_expression"),
        "the shared overflow fixture must retain its FSL refusal code"
    );
    assert!(
        expected
            .windows(b"unproved-range".len())
            .any(|window| window == b"unproved-range"),
        "the shared overflow fixture must retain its FSL refusal cause"
    );

    let child = Command::new(FSL)
        .args(["check", source_text, "--format", "json"])
        .env("TC018_CASE", "refused-range")
        .env("TC018_REQUEST_PATH", &request_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn fsl check with piped stdout");
    let mut child = ChildGuard::new(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut writer = open_fifo_writer_until(&source_path, &mut child, deadline);
    drop(child.child().stdout.take());
    writer
        .write_all(&bytes)
        .expect("write the shared overflow fixture into the source FIFO");
    drop(writer);

    let status = wait_for_child_until(&mut child, deadline);
    assert_tc018_refused_range_observation(&request_path, source_text, &bytes);
    let stderr_bytes = read_child_stderr(&mut child);
    let stderr = String::from_utf8_lossy(&stderr_bytes);
    assert_eq!(
        status.code(),
        Some(i32::from(Category::Refusal.exit_code())),
        "broken pipe preserves the independently computed FSL category exit"
    );
    assert!(
        !stderr.contains("OutputFailure"),
        "BrokenPipe must not emit OutputFailure: {stderr}"
    );
}

/// Trace: FR-002-AC-15, FR-002-AC-16; non-broken stdout I/O errors select InternalFailure and diagnose.
#[cfg(all(target_os = "linux", feature = "tc018-test-adapter"))]
#[test]
fn full_stdout_device_selects_internal_failure_and_output_failure_diagnostic() {
    let directory = TestDirectory::new("tc018-full-stdout");
    let source_path = directory.path().join("driver-check.native");
    let bytes = overflow_fixture_bytes();
    std::fs::write(&source_path, &bytes).expect("write the shared overflow source fixture");
    let source_text = source_path.to_str().expect("temporary path is UTF-8");
    let expected = expected_overflow_check_document(source_text, &bytes);
    let request_path = directory.path().join("tc018-request.json");
    assert!(
        expected
            .windows(b"undefined_expression".len())
            .any(|window| window == b"undefined_expression"),
        "the shared overflow fixture must retain its FSL refusal code"
    );

    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open Linux full device");
    let child = Command::new(FSL)
        .args(["check", source_text, "--format", "json"])
        .env("TC018_CASE", "refused-range")
        .env("TC018_REQUEST_PATH", &request_path)
        .stdout(Stdio::from(full))
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn fsl check with /dev/full as stdout");
    let mut child = ChildGuard::new(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = wait_for_child_until(&mut child, deadline);
    assert_tc018_refused_range_observation(&request_path, source_text, &bytes);
    let stderr_bytes = read_child_stderr(&mut child);
    let stderr = String::from_utf8_lossy(&stderr_bytes);

    assert_eq!(
        status.code(),
        Some(i32::from(Category::InternalFailure.exit_code()))
    );
    assert!(
        stderr.contains("OutputFailure: could not write outcome document"),
        "non-BrokenPipe failure is visibly classified and identifies the output: {stderr}"
    );
}

/// Trace: FR-002-AC-6, the reserved FSL result falls back without stdout output.
#[cfg(feature = "tc018-test-adapter")]
#[test]
fn reserved_exit_code_uses_internal_failure_fallback() {
    let version = Command::new(FSL)
        .arg("--version")
        .env("TC018_CASE", "reserved-exit")
        .output()
        .expect("run the test-built `fsl` binary with --version");
    assert_eq!(version.status.code(), Some(30));
    assert!(version.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&version.stderr),
        "driver exit mapping rejected result 5 (reserved code 5); using fallback code 30.\n"
    );

    for format in ["text", "json"] {
        let output = Command::new(FSL)
            .args(["run", RUN_REQUEST, "--format", format])
            .env("TC018_CASE", "reserved-exit")
            .output()
            .expect("run the test-built `fsl` binary with a valid verb");
        assert_eq!(output.status.code(), Some(30), "{format}");
        assert!(output.stdout.is_empty(), "{format}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("reserved code 5"), "{format}: {stderr}");
        assert!(stderr.contains("fallback code 30"), "{format}: {stderr}");
        assert!(
            stderr.contains(
                "driver exit mapping rejected result 5 (reserved code 5); using fallback code 30."
            ),
            "{format}: {stderr}"
        );
        if format == "json" {
            assert_eq!(
                stderr.as_ref(),
                "driver exit mapping rejected result 5 (reserved code 5); using fallback code 30.\n",
                "{format}"
            );
        } else {
            assert!(
                stderr.contains("unimplemented_capability"),
                "{format}: {stderr}"
            );
        }
    }

    #[cfg(unix)]
    {
        let status = status_with_unwritable_stderr(&["run", RUN_REQUEST, "--format", "json"], true);
        assert_eq!(status.code(), Some(30));
    }
}

/// The normal binary ignores TC-018's selector even when the caller sets it.
#[cfg(not(feature = "tc018-test-adapter"))]
#[test]
fn default_binary_does_not_enable_the_test_exit_seam() {
    let output = Command::new(FSL)
        .args(["check", SOURCE, "--format", "json"])
        .env("TC018_CASE", "reserved-exit")
        .output()
        .expect("run the default-feature fsl binary");
    assert_eq!(output.status.code(), Some(0));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check writes a FSL outcome");
    assert_eq!(document["operation"], "check");
    assert!(output.stderr.is_empty());
}
