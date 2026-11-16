// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for the initial command shell.
#![allow(
    clippy::expect_used,
    reason = "process launch failures should fail this integration test"
)]

use std::path::Path;
use std::process::Command;

use filament_exact::Cancel;
use filament_semantic_value::catalog::Category;
use fsl_cst::format::{format_with_limits, FormatLimits, FormatRefusal};
use fsl_foundation::diagnostic::{Code, Staged};
use fsl_foundation::SourceIdentity;
use fsl_replay::spine::{
    check, package, parse, select, DependencyInput, PackageLimits, ParseRequest, SpineLimits,
};
use fsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};
use ix_cli_kit::version::Agreement;
use ix_trace_rs::trace;

#[path = "common/requests.rs"]
mod requests;

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.fsl");
const INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/parse-invalid.fsl"
);
const CHECK_INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/check-invalid.fsl"
);

fn parse_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read parse fixture");
    let source = SourceIdentity::new("issuer", "document");
    let limits = SpineLimits::default();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
        },
        limits.source,
        &Cancel::new(),
    );
    OutcomeDocument::from_parse(&result)
        .to_bytes()
        .expect("FSL serializes its parse outcome")
}

fn check_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read check fixture");
    let source = SourceIdentity::new("filament", path);
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
        },
        limits.source,
        &cancel,
    )
    .and_then(|parsed| {
        select(parsed.value(), &Default::default(), limits.model, &cancel).and_then(|models| {
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
        .expect("FSL serializes its check outcome")
}

fn package_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read package fixture");
    let source = SourceIdentity::new("filament", path);
    let limits = SpineLimits::default();
    let dependencies = DependencyInput::default();
    let packages = std::collections::BTreeMap::new();
    let cancel = Cancel::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
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
                &dependencies,
                limits,
                &cancel,
            )
            .and_then(|checked| package(checked.value(), PackageLimits::default(), &cancel))
        })
    });
    OutcomeDocument::from_package(&result)
        .to_bytes()
        .expect("FSL serializes its package outcome")
}

fn format_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read format fixture");
    let source = SourceIdentity::new("issuer", "document");
    let limits = fsl_cst::Limits::default();
    let cancel = Cancel::new();
    let result = fsl_cst::parse_with_cancel(source, path, &bytes, limits, &cancel)
        .map_err(FormatRefusal::DiagnosedSource)
        .and_then(|parsed| format_with_limits(&parsed, FormatLimits::default()).map(Staged::new));
    let formatted_text = result.as_ref().ok().map(|staged| staged.value().clone());
    let document = OutcomeDocument::from_format(&result)
        .to_bytes()
        .expect("FSL serializes its format outcome");
    let Some(formatted_text) = formatted_text else {
        return document;
    };
    let mut value: serde_json::Value =
        serde_json::from_slice(&document).expect("FSL format outcome is JSON");
    value["result"] = serde_json::json!({"kind": "formatted", "text": formatted_text});
    serde_json::to_vec(&value).expect("encode expected format result")
}

fn argument_refusal_bytes(operation: Operation, message: &str) -> Vec<u8> {
    OutcomeDocument::new(operation, None, Code::InvalidRequest.category())
        .with_diagnostics(vec![OutcomeDiagnostic::new(
            None,
            Code::InvalidRequest.as_str(),
            None,
            message,
        )])
        .to_bytes()
        .expect("FSL serializes the argument refusal")
}

fn assert_fsl_document(cli_stdout: &[u8], fsl_bytes: &[u8]) {
    assert_eq!(
        cli_stdout.iter().filter(|&&byte| byte == b'\n').count(),
        1,
        "machine output has exactly one trailing line terminator"
    );
    let cli_document = cli_stdout
        .strip_suffix(b"\n")
        .expect("machine output ends with one line terminator");
    let cli_value: serde_json::Value =
        serde_json::from_slice(cli_document).expect("CLI emits one JSON document");
    let fsl_value: serde_json::Value =
        serde_json::from_slice(fsl_bytes).expect("FSL bytes encode one JSON document");
    assert_eq!(cli_value, fsl_value);
}

/// Trace: IR-607, one FSL exit function for unknown verbs.
#[test]
fn unknown_verb_uses_fsl_refusal_code() {
    let output = Command::new(FSL)
        .arg("not-an-fsl-verb")
        .output()
        .expect("run unknown command");
    assert_eq!(output.status.code(), Some(20));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check dispatches through FSL's typed check operation.
#[test]
fn check_emits_fsl_check_outcome_document() {
    let output = Command::new(FSL)
        .args(["check", SOURCE, "--format", "json"])
        .output()
        .expect("run check command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_fsl_document(&output.stdout, &check_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-5; the CLI reads only the input path supplied on argv.
#[test]
fn check_does_not_read_an_unnamed_config_from_the_working_directory() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "fsl-cli-no-implicit-config-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create isolated working directory");
    struct RemoveDirectory(std::path::PathBuf);
    impl Drop for RemoveDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = RemoveDirectory(directory.clone());

    // A config reader would reject this malformed file. The command receives
    // its source explicitly and must not inspect ambient working-directory data.
    std::fs::write(directory.join("fsl.toml"), "[malformed\n")
        .expect("write trap configuration file");

    let output = Command::new(FSL)
        .args(["check", SOURCE, "--format", "json"])
        .current_dir(&directory)
        .output()
        .expect("run check from isolated working directory");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_fsl_document(&output.stdout, &check_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check refusals retain FSL's typed outcome.
#[test]
fn check_refusal_emits_fsl_diagnostic_cause_and_locus() {
    let output = Command::new(FSL)
        .args(["check", INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run check refusal");
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(output.stderr, b"expected `)`\n");
    assert_fsl_document(&output.stdout, &check_outcome_bytes(INVALID_SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check source I/O failures use FSL's typed code and serializer.
#[test]
fn check_missing_source_emits_fsl_io_error_document() {
    let missing = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/missing-check-source.fsl"
    );
    let output = Command::new(FSL)
        .args(["check", missing, "--format", "json"])
        .output()
        .expect("run check with a missing source");
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = stderr.trim_end();
    assert!(message.starts_with(&format!("could not read source {missing}:")));
    assert!(stderr.ends_with('\n'));

    let expected = OutcomeDocument::new(Operation::Check, None, Code::IoError.category())
        .with_diagnostics(vec![OutcomeDiagnostic::new(
            None,
            Code::IoError.as_str(),
            None,
            message,
        )])
        .to_bytes()
        .expect("FSL serializes the check I/O outcome");
    assert_fsl_document(&output.stdout, &expected);
}

/// Trace: FR-002-AC-1, FR-002-AC-3; compile dispatches through FSL's package operation.
#[test]
fn compile_emits_fsl_package_outcome_document() {
    let request = requests::request(SOURCE, None);
    let output = Command::new(FSL)
        .args(["compile", request.as_str(), "--format", "json"])
        .output()
        .expect("run compile command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_fsl_document(&output.stdout, &package_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; compile refusals retain FSL's package outcome.
#[test]
fn compile_refusal_emits_fsl_package_outcome_document() {
    let request = requests::request(INVALID_SOURCE, None);
    let output = Command::new(FSL)
        .args(["compile", request.as_str(), "--format", "json"])
        .output()
        .expect("run compile refusal");
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(output.stderr, b"expected `)`\n");
    assert_fsl_document(&output.stdout, &package_outcome_bytes(INVALID_SOURCE));
}

/// Trace: QSL-2192; an empty compile program is an invalid package value.
#[test]
fn compile_empty_program_is_invalid_package() {
    let directory = tempfile::tempdir().expect("create compile request directory");
    let request = directory.path().join("empty-program.json");
    std::fs::write(
        &request,
        r#"{"format":"native-compile/1","request":{"models":[],"program":[]}}"#,
    )
    .expect("write empty compile request");

    let output = Command::new(FSL)
        .arg("compile")
        .arg(&request)
        .args(["--format", "json"])
        .output()
        .expect("run compile with an empty program");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI emits a refusal document");
    assert_eq!(document["category"], "refusal");
    assert_eq!(document["diagnostics"][0]["code"], "invalid_package");
    assert_eq!(document["diagnostics"][0]["cause"], "invalid-value");
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check semantic refusals retain FSL's checked outcome.
#[test]
fn check_semantic_refusal_emits_fsl_check_outcome_document() {
    let output = Command::new(FSL)
        .args(["check", CHECK_INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run check semantic refusal");
    assert_eq!(output.status.code(), Some(20));
    assert!(!output.stderr.is_empty());
    assert_fsl_document(&output.stdout, &check_outcome_bytes(CHECK_INVALID_SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; argument refusals retain the selected FSL operation.
#[test]
fn implemented_verb_argument_refusals_use_the_selected_operation() {
    let cases = [
        (
            "parse",
            Operation::Parse,
            "unknown parse option `--unknown`",
        ),
        (
            "check",
            Operation::Check,
            "unknown check option `--unknown`",
        ),
        (
            "compile",
            Operation::Package,
            "unknown compile option `--unknown`",
        ),
        (
            "format",
            Operation::Format,
            "unknown format option `--unknown`",
        ),
    ];

    for (verb, operation, message) in cases {
        let output = Command::new(FSL)
            .args([verb, "--unknown", "--format", "json"])
            .output()
            .expect("run command with an unknown operation option");
        assert_eq!(output.status.code(), Some(20), "{verb} exit status");
        assert_eq!(
            output.stderr,
            format!("{message}\n").as_bytes(),
            "{verb} stderr"
        );
        assert_fsl_document(&output.stdout, &argument_refusal_bytes(operation, message));
    }
}

/// Trace: FR-002-AC-1, FR-002-AC-3; parse serializes FSL's typed parse result.
#[test]
fn parse_emits_fsl_parse_outcome_document() {
    let output = Command::new(FSL)
        .args(["parse", "issuer", "document", SOURCE, "--format", "json"])
        .output()
        .expect("run parse command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_fsl_document(&output.stdout, &parse_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; a parse refusal retains FSL's cause and locus.
#[test]
fn parse_refusal_emits_fsl_diagnostic_cause_and_locus() {
    let output = Command::new(FSL)
        .args([
            "parse",
            "issuer",
            "document",
            INVALID_SOURCE,
            "--format",
            "json",
        ])
        .output()
        .expect("run parse refusal");
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(output.stderr, b"expected `)`\n");

    assert_fsl_document(&output.stdout, &parse_outcome_bytes(INVALID_SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; text mode writes FSL's formatted UTF-8 value.
#[test]
fn format_writes_fsl_formatted_text_without_editing_the_source() {
    let source = std::fs::read(SOURCE).expect("read format fixture");
    let parsed = fsl_cst::parse_with_cancel(
        SourceIdentity::new("issuer", "document"),
        SOURCE,
        &source,
        fsl_cst::Limits::default(),
        &Cancel::new(),
    )
    .expect("fixture is an admissible parsed source");
    let expected = format_with_limits(&parsed, FormatLimits::default())
        .expect("fixture formats under the default limit");

    let output = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE])
        .output()
        .expect("run format command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(
        std::fs::read(SOURCE).expect("source remains readable"),
        source
    );
}

/// Trace: FR-002-AC-1, FR-002-AC-3; format machine output is FSL's typed outcome.
#[test]
fn format_machine_mode_emits_fsl_format_outcome_document() {
    let output = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE, "--format", "json"])
        .output()
        .expect("run format in machine mode");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_fsl_document(&output.stdout, &format_outcome_bytes(SOURCE));
}

/// Trace: QSL-2192; machine format results carry the exact plain-text result.
#[test]
fn format_json_result_carries_text() {
    let text = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE])
        .output()
        .expect("format source as text");
    assert_eq!(text.status.code(), Some(0));
    assert!(text.stderr.is_empty());

    let json = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE, "--format", "json"])
        .output()
        .expect("format source as JSON");
    assert_eq!(json.status.code(), Some(0));
    assert!(json.stderr.is_empty());
    let document: serde_json::Value =
        serde_json::from_slice(&json.stdout).expect("format emits a JSON document");
    assert_eq!(document["result"]["kind"], "formatted");
    assert_eq!(
        document["result"]["text"].as_str().map(str::as_bytes),
        Some(text.stdout.as_slice())
    );
}

/// Trace: QSL-2192; formatting an already formatted file is idempotent.
#[test]
fn format_json_is_idempotent() {
    let first = Command::new(FSL)
        .args(["format", "issuer", "document", SOURCE])
        .output()
        .expect("format original source");
    assert_eq!(first.status.code(), Some(0));
    let directory = tempfile::tempdir().expect("create formatter directory");
    let formatted = directory.path().join("formatted.fsl");
    std::fs::write(&formatted, &first.stdout).expect("write canonical text");

    let second = Command::new(FSL)
        .arg("format")
        .arg("issuer")
        .arg("document")
        .arg(&formatted)
        .output()
        .expect("format canonical source");
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(second.stdout, first.stdout);
}

/// Trace: FR-002-AC-1, FR-002-AC-3; formatter refusals retain FSL diagnostics.
#[test]
fn format_refusal_emits_fsl_diagnostic_cause_and_locus() {
    let output = Command::new(FSL)
        .args([
            "format",
            "issuer",
            "document",
            INVALID_SOURCE,
            "--format",
            "json",
        ])
        .output()
        .expect("run format refusal");
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(output.stderr, b"expected `)`\n");

    assert_fsl_document(&output.stdout, &format_outcome_bytes(INVALID_SOURCE));
}

/// Trace: FR-002-AC-11, --version reports the Cargo package semantic version.
#[test]
fn version_command_reports_the_fsl_cli_package_version() {
    let output = Command::new(FSL)
        .arg("--version")
        .output()
        .expect("run fsl --version");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
    assert_eq!(
        output.stdout,
        format!("{}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    assert!(output.stderr.is_empty());
}

#[trace("TC-009", "FR-002-AC-17")]
#[test]
fn root_help_selectors_print_the_same_informative_help_without_an_operation() {
    let mut expected_help = None;
    for arguments in [&[][..], &["--help"][..], &["-h"][..], &["help"][..]] {
        let output = Command::new(FSL)
            .args(arguments)
            .output()
            .expect("run root help without a source or lifecycle operation");
        assert_eq!(output.status.code(), Some(0), "arguments: {arguments:?}");
        assert!(output.stderr.is_empty(), "arguments: {arguments:?}");
        let help = std::str::from_utf8(&output.stdout).expect("help is UTF-8");
        for required in [
            "Usage: fsl <command>",
            "--format text|json|dot",
            "dot is accepted only for inspect",
            "--version, -V",
        ] {
            assert!(help.contains(required), "missing {required:?} in {help:?}");
        }
        if let Some(expected) = &expected_help {
            assert_eq!(&output.stdout, expected, "arguments: {arguments:?}");
        } else {
            expected_help = Some(output.stdout);
        }
    }
}

#[trace("TC-009", "FR-002-AC-17")]
#[test]
fn root_help_identifies_current_command_availability_and_text_restrictions() {
    let output = Command::new(FSL)
        .arg("--help")
        .output()
        .expect("read root help");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let help = std::str::from_utf8(&output.stdout).expect("help is UTF-8");
    let command_line = |command| {
        help.lines()
            .find(|line| line.split_whitespace().next() == Some(command))
            .expect("every recognized lifecycle command has its own help row")
    };
    for command in ["parse", "check", "compile", "run"] {
        let line = command_line(command);
        assert!(line.contains("Executes with --format json"), "{line}");
        assert!(
            line.contains("default text output is unavailable"),
            "{line}"
        );
    }
    let format_line = command_line("format");
    assert!(format_line.contains("Executes source formatting"));
    assert!(format_line.contains("text output is the default"));
    let analyze_line = command_line("analyze");
    assert!(analyze_line.contains("native-analyze/1"), "{analyze_line}");
    // FSL FR-624 owes each unrouted verb a named outcome document, so this
    // fixes only the exit code, never empty output.
    let monitor_line = command_line("monitor");
    assert!(monitor_line.contains("native-monitor/1"), "{monitor_line}");
    for command in ["prove", "replay", "generate", "inspect", "render"] {
        assert!(command_line(command).contains("unsupported-capability"));
        let unavailable = Command::new(FSL)
            .args([command, "--format", "json"])
            .output()
            .expect("observe the advertised unsupported command");
        assert_eq!(
            unavailable.status.code(),
            Some(i32::from(Category::Unsupported.exit_code())),
            "{command}"
        );
    }
}

#[trace("TC-009", "FR-002-AC-17")]
#[test]
fn informative_help_does_not_replace_invalid_command_or_format_refusals() {
    for (arguments, diagnostic) in [
        (&["not-a-command"][..], "unknown verb `not-a-command`"),
        (
            &["check", "--format", "dot"][..],
            "the `dot` format is available only for `inspect`",
        ),
    ] {
        let output = Command::new(FSL)
            .args(arguments)
            .output()
            .expect("run an invalid command selection");
        assert_eq!(output.status.code(), Some(20), "{arguments:?}");
        assert!(output.stdout.is_empty(), "{arguments:?}");
        assert!(
            std::str::from_utf8(&output.stderr)
                .expect("diagnostics are UTF-8")
                .contains(diagnostic),
            "arguments: {arguments:?}"
        );
    }
}

/// Trace: FR-002-AC-12, the built binary agrees with its Cargo package version.
#[test]
fn built_binary_version_agrees_with_cargo_package_version() {
    let agreement = Agreement::new()
        .surface("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))
        .command("fsl --version", Path::new(FSL), &["--version"]);
    let result = agreement.check();
    assert!(result.is_ok(), "version agreement failed: {result:?}");
}
