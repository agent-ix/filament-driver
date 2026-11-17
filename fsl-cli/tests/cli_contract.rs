// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for the initial command shell.
#![allow(
    clippy::expect_used,
    reason = "process launch failures should fail this integration test"
)]

use std::path::{Path, PathBuf};
use std::process::Command;

use filament_exact::Cancel;
use filament_semantic_value::catalog::Category;
use fsl_command::command::{self, CheckInputs};
use fsl_cst::format::{format_with_limits, FormatLimits, FormatRefusal};
use fsl_foundation::diagnostic::{Code, Staged};
use fsl_foundation::{SettingLimits, Source, SourceIdentity};
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
const CHECK_MODEL_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/check-with-model.fsl"
);
const CHECK_LIBRARY_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/check-with-library.fsl"
);
const MODEL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/population-model.semantic-ir.json"
);

struct TemporaryDirectory(std::path::PathBuf);

impl TemporaryDirectory {
    fn new(label: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("fsl-cli-{label}-{}-{unique}", std::process::id()));
        std::fs::create_dir(&path).expect("create isolated test directory");
        Self(path)
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
const PAID_SHORT_REQUEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/paid-short/requests/paid-short.json"
);
const CHARGE_CHANGES_TOTAL_REQUEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/charge-changes-total/requests/charge-changes-total.json"
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
    command::check_outcome(Path::new(path), &CheckInputs::default())
        .document()
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
    let document = match parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
        },
        limits.source,
        &cancel,
    ) {
        Err(failure) => OutcomeDocument::from_package(&Err(failure)),
        Ok(parsed) => match select(parsed.value(), &packages, limits.model, &cancel) {
            Err(failure) => OutcomeDocument::from_package(&Err(failure)),
            Ok(models) => match check(
                parsed.value(),
                path.to_owned(),
                models.value(),
                &dependencies,
                limits,
                &cancel,
            ) {
                Err(failure) => OutcomeDocument::from_package(&Err(failure)),
                Ok(checked) => OutcomeDocument::from_package(&package(
                    checked.value(),
                    PackageLimits::default(),
                    &cancel,
                )),
            },
        },
    };
    document
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
    OutcomeDocument::from_format(&result)
        .to_bytes()
        .expect("FSL serializes its format outcome")
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

fn deep_parenthesis_source(depth: usize) -> String {
    let expression = format!("{}1{}", "(".repeat(depth), ")".repeat(depth));
    format!(
        concat!(
            "language \"ix:native\" edition \"1-draft\";\n",
            "profile v = \"filament.value.complete/v1\";\n",
            "function deep using v(): Integer pure {{ {expression} }}\n"
        ),
        expression = expression
    )
}

fn write_deep_source(directory: &TemporaryDirectory, depth: usize) -> PathBuf {
    let path = directory.0.join("deep.fsl");
    std::fs::write(&path, deep_parenthesis_source(depth)).expect("write deep source fixture");
    path
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

/// Trace: TC-1743, FR-662-AC-1, FSpec-TC-1243, FSpec-FR-563-AC-1, FSpec-FR-563-AC-2.
/// Provenance: QSL-2019.
#[test]
#[trace(
    "TC-1743",
    "FR-662-AC-1",
    "FSpec-TC-1243",
    "FSpec-FR-563-AC-1",
    "FSpec-FR-563-AC-2"
)]
fn check_markdown_checks_each_named_native_fence() {
    let directory = TemporaryDirectory::new("check-markdown");
    let healthy = std::fs::read_to_string(SOURCE).expect("read native source fixture");
    let broken = format!("{healthy}function Bad using v (x: Integer): Boolean pure {{ x }}\n");
    let document = format!(
        "### Healthy\n\n```ix:native\n{healthy}```\n\n### Broken\n\n```ix:native\n{broken}```\n"
    );
    std::fs::write(directory.0.join("spec.md"), &document).expect("write Markdown fixture");
    let output = Command::new(FSL)
        .current_dir(&directory.0)
        .args([
            "check",
            "--markdown",
            "agent-ix",
            "doc",
            "spec.md",
            "--format",
            "json",
        ])
        .output()
        .expect("run Markdown check");
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let outcome: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(outcome["operation"], "check", "{outcome}");
    assert_eq!(outcome["category"], "refusal", "{outcome}");
    let items = outcome["items"].as_array().expect("fence result items");
    assert_eq!(items.len(), 2, "{outcome}");
    assert_eq!(items[0]["category"], "success", "{outcome}");
    assert_eq!(items[0]["unit"]["document"]["authority"], "agent-ix");
    assert_eq!(items[0]["unit"]["document"]["identity"], "doc");
    assert_eq!(items[0]["unit"]["path"], "spec.md");
    assert_eq!(items[0]["unit"]["clause"], "Healthy");
    assert_eq!(items[0]["unit"]["body"]["identity"], "doc#Healthy");
    assert_eq!(items[0]["diagnostics"], serde_json::json!([]));

    let broken_item = &items[1];
    assert_eq!(broken_item["category"], "refusal", "{outcome}");
    let unit = &broken_item["diagnostics"][0]["unit"];
    assert_eq!(unit["document"]["authority"], "agent-ix");
    assert_eq!(unit["document"]["identity"], "doc");
    assert_eq!(unit["path"], "spec.md");
    assert_eq!(unit["clause"], "Broken");
    assert_eq!(unit["body"]["authority"], "agent-ix");
    assert_eq!(unit["body"]["identity"], "doc#Broken");

    let diagnostic = &broken_item["diagnostics"][0];
    assert!(!diagnostic["cause"].is_null(), "{outcome}");
    assert_eq!(diagnostic["code"], "ill_typed", "{outcome}");
    let body_start = broken.rfind(" x }").expect("ill-typed expression") + 1;
    assert_eq!(diagnostic["locus"]["kind"], "region", "{outcome}");
    assert_eq!(diagnostic["locus"]["source"]["identity"], "doc#Broken");
    assert_eq!(diagnostic["locus"]["start"], body_start);
    assert_eq!(diagnostic["locus"]["end"], body_start + 1);

    let original_start = document.rfind(" x }").expect("ill-typed expression") + 1;
    let prefix = &document[..original_start];
    let expected_line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let expected_column = prefix
        .rsplit('\n')
        .next()
        .expect("last source line")
        .chars()
        .count()
        + 1;
    let original = &diagnostic["original_regions"][0];
    assert_eq!(original["kind"], "located", "{outcome}");
    assert_eq!(original["file"], "spec.md");
    assert_eq!(original["span"]["start"]["line"], expected_line);
    assert_eq!(original["span"]["start"]["column"], expected_column);

    let without_markdown = Command::new(FSL)
        .current_dir(&directory.0)
        .args(["check", "agent-ix", "doc", "spec.md", "--format", "json"])
        .output()
        .expect("run native check against Markdown bytes");
    assert_eq!(
        without_markdown.status.code(),
        Some(20),
        "{without_markdown:?}"
    );
    let plain_document: serde_json::Value =
        serde_json::from_slice(&without_markdown.stdout).expect("one native check outcome");
    assert_eq!(plain_document["operation"], "check");
    assert_eq!(plain_document["category"], "refusal");
    assert!(
        plain_document.to_string().contains("invalid_syntax"),
        "{plain_document}"
    );
}

/// Trace: FR-002-AC-3.
/// Provenance: QSL-2019, FND-001.
#[test]
#[trace("FR-002-AC-3")]
fn check_markdown_source_read_refusal_keeps_requested_identity() {
    let directory = TemporaryDirectory::new("check-markdown-source-refusal");
    let path = directory.0.join("spec.md");
    std::fs::write(&path, [0xff]).expect("write invalid UTF-8 source fixture");
    let output = Command::new(FSL)
        .current_dir(&directory.0)
        .args([
            "check",
            "--markdown",
            "agent-ix",
            "doc",
            "spec.md",
            "--format",
            "json",
        ])
        .output()
        .expect("run Markdown check on invalid UTF-8");
    assert_eq!(output.status.code(), Some(20), "{output:?}");

    let limits = SpineLimits::default();
    let refusal = Source::read_typed(
        SourceIdentity::new("agent-ix", "doc"),
        "spec.md",
        &[0xff],
        limits.source.source_bytes,
    )
    .expect_err("invalid UTF-8 must be refused at source admission");
    let diagnostic = fsl_cst::diagnostic::source_refusal(*refusal, limits.source.source_bytes);
    let expected = OutcomeDocument::new(Operation::Check, None, diagnostic.code.category())
        .with_diagnostics(vec![OutcomeDiagnostic::from_source_diagnostic(&diagnostic)])
        .to_bytes()
        .expect("serialize typed source refusal");
    assert_fsl_document(&output.stdout, &expected);

    let outcome: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(outcome["category"], "refusal", "{outcome}");
    assert_eq!(outcome["diagnostics"][0]["code"], "invalid_utf8");
    assert_eq!(
        outcome["diagnostics"][0]["locus"]["source"]["authority"],
        "agent-ix"
    );
    assert_eq!(
        outcome["diagnostics"][0]["locus"]["source"]["identity"],
        "doc"
    );
}

/// Trace: FSpec-TC-1243, FSpec-FR-563-AC-2.
/// Provenance: QSL-2019.
#[test]
#[trace("FSpec-TC-1243", "FSpec-FR-563-AC-2")]
#[ignore = "QSL-2443 waits for FSL Markdown clause extraction support"]
fn run_executes_a_selected_markdown_clause_in_the_default_build() {
    let directory = TemporaryDirectory::new("run-markdown-extraction");
    let body = std::fs::read_to_string(SOURCE).expect("read native source fixture");
    let markdown = format!("### Healthy\n\n```ix:native\n{body}```\n");
    std::fs::write(directory.0.join("spec.md"), markdown).expect("write Markdown fixture");
    std::fs::write(
        directory.0.join("current.json"),
        serde_json::json!({
            "format": "filament.state.snapshot/v1",
            "identity": {"authority": "agent-ix", "identity": "current"},
            "observation": "current"
        })
        .to_string(),
    )
    .expect("write observation fixture");
    let request = serde_json::json!({
        "format": "native-run/1",
        "request": {
            "models": [],
            "program": {
                "source": {
                    "file": "spec.md",
                    "authority": "agent-ix",
                    "identity": "doc",
                    "document": "Spec",
                    "formal_revision": 1
                },
                "extraction": {
                    "body": {"authority": "agent-ix", "identity": "doc#Healthy", "document": "Spec"}
                },
                "clauses": [{"clause": "Healthy", "package": "doc"}]
            },
            "clause": {
                "selection": {
                    "function": {"name": "yes", "arguments": [], "snapshot": "current"}
                },
                "observations": {"current": "current.json"}
            }
        }
    });
    let request_path = directory.0.join("request.json");
    std::fs::write(&request_path, request.to_string()).expect("write run request");

    let output = Command::new(FSL)
        .current_dir(&directory.0)
        .arg("run")
        .arg("request.json")
        .args(["--format", "json"])
        .output()
        .expect("run extracted clause");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let outcome: serde_json::Value = serde_json::from_slice(&output.stdout).expect("outcome JSON");
    assert_eq!(outcome["format"], "filament-outcome/1", "{outcome}");
    assert_eq!(outcome["operation"], "monitor", "{outcome}");
    assert_eq!(outcome["category"], "success", "{outcome}");
    assert_eq!(
        outcome["items"].as_array().map(Vec::len),
        Some(1),
        "{outcome}"
    );
    assert!(outcome.to_string().contains("Healthy"), "{outcome}");
}

/// Trace: TC-1743, FR-662-AC-2.
/// Provenance: QSL-2019.
#[test]
#[trace("TC-1743", "FR-662-AC-2")]
fn check_markdown_refuses_doubled_and_headless_fences() {
    let directory = TemporaryDirectory::new("check-markdown-refusals");
    let healthy = std::fs::read_to_string(SOURCE).expect("read native source fixture");
    let document = format!(
        "```ix:native\n{healthy}```\n\n### Twice\n\n```ix:native\n{healthy}```\n\n```ix:native\n{healthy}```\n\n### Stable\n\n```ix:native\n{healthy}```\n"
    );
    std::fs::write(directory.0.join("spec.md"), &document).expect("write Markdown fixture");

    let run_check = || {
        Command::new(FSL)
            .current_dir(&directory.0)
            .args([
                "check",
                "--markdown",
                "agent-ix",
                "doc",
                "spec.md",
                "--format",
                "json",
            ])
            .output()
            .expect("run Markdown check")
    };
    let output = run_check();
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let repeated = run_check();
    assert_eq!(repeated.status.code(), Some(20), "{repeated:?}");
    assert_eq!(
        output.stdout, repeated.stdout,
        "fence report is deterministic"
    );

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one outcome document");
    assert_eq!(report["operation"], "check", "{report}");
    assert_eq!(report["category"], "refusal", "{report}");
    let items = report["items"].as_array().expect("fence result items");
    assert_eq!(items.len(), 4, "{report}");

    let headless = &items[0];
    assert_eq!(headless["category"], "refusal", "{headless}");
    assert_eq!(headless["unit"]["clause"], serde_json::Value::Null);
    assert_eq!(headless["diagnostics"][0]["cause"], "missing-name");
    assert_eq!(headless["diagnostics"][0]["code"], "missing_declaration");
    assert_eq!(headless["first_line"], 2);

    for doubled in [&items[1], &items[2]] {
        assert_eq!(doubled["category"], "refusal", "{doubled}");
        assert_eq!(doubled["unit"]["clause"], "Twice");
        assert_eq!(
            doubled["diagnostics"][0]["cause"],
            "extraction-clause-count"
        );
        assert_eq!(doubled["diagnostics"][0]["code"], "extraction-clause-count");
        assert_eq!(doubled["details"]["actual"], 2);
    }

    let stable = &items[3];
    assert_eq!(stable["category"], "success", "{stable}");
    assert_eq!(stable["unit"]["clause"], "Stable");
    assert_eq!(stable["unit"]["body"]["identity"], "doc#Stable");
    assert_eq!(stable["diagnostics"], serde_json::json!([]));
}

/// Trace: QSL-1972 / FR-866-AC-1; check supplies the selected domain model to FSL.
#[test]
fn check_with_model_checks_importing_unit() {
    let directory = TemporaryDirectory::new("check-model");
    let model = directory.0.join("model.json");
    std::fs::copy(MODEL, &model).expect("copy model fixture");
    let output = Command::new(FSL)
        .arg("check")
        .arg(CHECK_MODEL_SOURCE)
        .arg("--model")
        .arg(&model)
        .arg("--dependent-bytes")
        .arg("8192")
        .arg("--format")
        .arg("json")
        .output()
        .expect("check unit with model");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a JSON outcome");
    assert_eq!(document["operation"], "check");
    assert_eq!(document["category"], "success");
}

/// Trace: QSL-1972 / FR-866-AC-2; missing models preserve the selected identity and supplying flag.
#[test]
fn check_without_model_names_supplying_option() {
    let output = Command::new(FSL)
        .args(["check", CHECK_MODEL_SOURCE, "--format", "json"])
        .output()
        .expect("check importing unit without its model");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a refusal outcome");
    let serialized = document.to_string();
    assert!(serialized.contains("driver/ir608-model"), "{serialized}");
    assert!(serialized.contains("--model"), "{serialized}");
}

/// Trace: QSL-1972 / FR-866-AC-4; a model on a source with no model selection is refused.
#[test]
fn check_model_option_misuse_refused() {
    let output = Command::new(FSL)
        .arg("check")
        .arg(SOURCE)
        .arg("--model")
        .arg(MODEL)
        .arg("--format")
        .arg("json")
        .output()
        .expect("check source with unused model");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a refusal outcome");
    assert!(document.to_string().contains("--model"));
}

/// Trace: QSL-1972 / FR-866-AC-5; dependent-byte options bound supplied model files.
#[test]
fn check_dependent_byte_limit_is_applied() {
    let output = Command::new(FSL)
        .args([
            "check",
            CHECK_MODEL_SOURCE,
            "--model",
            MODEL,
            "--dependent-bytes",
            "0",
            "--format",
            "json",
        ])
        .output()
        .expect("check unit with zero dependent-byte allowance");
    assert_eq!(output.status.code(), Some(22));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a refusal outcome");
    assert!(document.to_string().contains("dependent_bytes"));
}

/// Trace: QSL-1972 / FR-866-AC-5; each library path is offered for a source-selected identity.
#[test]
fn check_library_path_is_read_for_selected_import() {
    let directory = TemporaryDirectory::new("check-library");
    let missing_library = directory.0.join("library.native");
    let output = Command::new(FSL)
        .arg("check")
        .arg(CHECK_LIBRARY_SOURCE)
        .arg("--library")
        .arg(&missing_library)
        .arg("--format")
        .arg("json")
        .output()
        .expect("check unit with selected library path");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits a refusal outcome");
    let serialized = document.to_string();
    assert!(
        serialized.contains(&missing_library.display().to_string()),
        "{serialized}"
    );
}

/// Trace: QSL-1972 / FR-866-AC-1; check links a supplied library selected by the source.
#[test]
fn check_with_library_checks_importing_unit() {
    let directory = TemporaryDirectory::new("check-library-success");
    let library = directory.0.join("library.native");
    std::fs::write(
        &library,
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"filament.value.complete/v1\";\n\
         predicate Ready using v (ready: Boolean): Boolean { ready }\n",
    )
    .expect("write library fixture");
    let output = Command::new(FSL)
        .arg("check")
        .arg(CHECK_LIBRARY_SOURCE)
        .arg("--library")
        .arg(&library)
        .arg("--format")
        .arg("json")
        .output()
        .expect("check unit with library");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
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

/// Trace: QSL-1994; `fsl run` preserves the FSL state-rule witness in JSON.
#[test]
fn run_paid_short_json_contains_the_state_rule_witness() {
    let output = Command::new(FSL)
        .args(["run", PAID_SHORT_REQUEST, "--format", "json"])
        .output()
        .expect("run the paid-short state-rule example");
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI emits one JSON outcome");
    assert_eq!(document["category"], "violation", "{document}");
    let item = &document["items"][0];
    assert_eq!(item["clause"], "PaidInFull", "{item}");
    assert_eq!(item["category"], "violation", "{item}");
    let witness = item
        .get("witness")
        .filter(|witness| witness.is_object())
        .expect("the state-rule violation carries its witness");
    assert_eq!(
        witness["value_path"]["root"]["subject"]["object_identity"], "o1",
        "{witness}"
    );
    assert!(
        witness["value_path"]["steps"]
            .as_array()
            .is_some_and(|steps| {
                steps
                    .iter()
                    .any(|step| step["name"] == "charged" || step["name"] == "total")
            }),
        "the witness identifies the deciding field: {witness}"
    );
    assert!(
        ["4000", "4200"].contains(
            &witness["deciding_element"]["decimal"]
                .as_str()
                .unwrap_or("")
        ),
        "the witness carries the decisive paid/total value: {witness}"
    );
}

/// Trace: QSL-2005; a frame violation reports the changed field and its values.
#[test]
fn run_charge_changes_total_json_contains_the_frame_witness() {
    let output = Command::new(FSL)
        .args(["run", CHARGE_CHANGES_TOTAL_REQUEST, "--format", "json"])
        .output()
        .expect("run the charge-changes-total frame example");
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI emits one JSON outcome");
    assert_eq!(document["category"], "violation", "{document}");
    let item = &document["items"][0];
    assert_eq!(item["clause"], "Shop::Order::charge", "{item}");
    assert_eq!(item["category"], "violation", "{item}");
    assert_eq!(item["cause"]["code"], "frame_violation", "{item}");
    assert_eq!(item["cause"]["kind"], "unauthorized-change", "{item}");

    let witness = item
        .get("witness")
        .and_then(serde_json::Value::as_object)
        .expect("the frame violation carries its user-facing witness");
    assert_eq!(witness["object_identity"], "o1", "{witness}");
    assert_eq!(witness["field"], "total", "{witness}");
    assert_eq!(witness["before"]["integer"], "4200", "{witness}");
    assert_eq!(witness["after"]["integer"], "9000", "{witness}");
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

/// Trace: FR-986-AC-1; a `null` expression reaches S3 and exits as a refusal.
#[trace("FR-986-AC-1")]
#[test]
fn null_expression_refusal_exits_twenty_at_s3() {
    let directory = tempfile::tempdir().expect("create null expression source directory");
    let source = directory.path().join("null-expression.fsl");
    std::fs::write(
        &source,
        concat!(
            "language \"ix:native\" edition \"1-draft\";\n",
            "profile v = \"filament.value.complete/v1\";\n",
            "function bad using v(): Integer pure { null }\n"
        ),
    )
    .expect("write null expression source");

    let output = Command::new(FSL)
        .arg("check")
        .arg(&source)
        .args(["--format", "json"])
        .output()
        .expect("check null expression source");

    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI emits a check outcome");
    assert_eq!(document["last_stage"], "S3", "{document}");
    assert_eq!(document["category"], "refusal", "{document}");
    assert_eq!(
        document["diagnostics"][0]["code"], "ill_typed",
        "{document}"
    );
    // FR-986 and FSpec FR-143-AC-20: `null` is `ill_typed` with cause `type-mismatch`,
    // located at the `null` token (bytes 121..125); the message text is language-owned.
    assert_eq!(
        document["diagnostics"][0]["cause"], "type-mismatch",
        "{document}"
    );
    assert_eq!(
        document["diagnostics"][0]["locus"]["start"], 121,
        "{document}"
    );
    assert_eq!(
        document["diagnostics"][0]["locus"]["end"], 125,
        "{document}"
    );
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

/// Trace: QSL-2190; a caller may raise the parser's node bound from the CLI.
#[trace("QSL-2190")]
#[test]
fn parse_accepts_raised_node_limit() {
    let directory = TemporaryDirectory::new("parse-node-limit");
    let source = write_deep_source(&directory, 6_000);
    let output = Command::new(FSL)
        .arg("parse")
        .arg("issuer")
        .arg("deep")
        .arg(&source)
        .args(["--limit", "s1.nodes=1000000", "--format", "json"])
        .output()
        .expect("parse a deep source with a raised node limit");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse emits an outcome document");
    assert_eq!(document["category"], "success");
}

/// Trace: QSL-2190; the default parser bound remains enforced.
#[trace("QSL-2190")]
#[test]
fn default_node_limit_still_stops_deep_nest() {
    let directory = TemporaryDirectory::new("default-node-limit");
    let source = write_deep_source(&directory, 6_000);
    let output = Command::new(FSL)
        .arg("parse")
        .arg("issuer")
        .arg("deep")
        .arg(&source)
        .arg("--format")
        .arg("json")
        .output()
        .expect("parse a deep source under the default node limit");
    assert_eq!(output.status.code(), Some(22));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse emits an incomplete document");
    assert_eq!(document["category"], "incomplete");
}

/// Trace: QSL-2190; check passes caller-raised source limits to FSL.
#[trace("QSL-2190")]
#[test]
fn check_accepts_raised_node_limit() {
    let directory = TemporaryDirectory::new("check-node-limit");
    let source = write_deep_source(&directory, 6_000);
    let mut command = Command::new(FSL);
    command.arg("check").arg(&source);
    for (setting, _) in SpineLimits::default().bounds() {
        command
            .arg("--limit")
            .arg(format!("{}=1000000", setting.name()));
    }
    let output = command
        .args(["--format", "json"])
        .output()
        .expect("check a deep source with caller-raised limits");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check emits an outcome document");
    assert_eq!(document["category"], "success");
}

/// Trace: QSL-2190, FR-003-AC-1; compile applies caller limits from FSL's bounds.
#[trace("QSL-2190", "FR-003-AC-1")]
#[test]
fn compile_accepts_limit_settings() {
    let directory = TemporaryDirectory::new("compile-node-limit");
    let source = write_deep_source(&directory, 6_000);
    let request = directory.0.join("compile.json");
    let document = serde_json::json!({
        "format": "native-compile/1",
        "request": {
            "models": [],
            "program": {"source": {
                "file": source,
                "authority": "issuer",
                "identity": "deep",
                "document": "Deep",
                "formal_revision": 1
            }}
        }
    });
    std::fs::write(
        &request,
        serde_json::to_vec(&document).expect("encode compile request"),
    )
    .expect("write compile request");

    let single_override = Command::new(FSL)
        .arg("compile")
        .arg(&request)
        .args(["--limit", "s1.nodes=1000000", "--format", "json"])
        .output()
        .expect("compile with one raised bound and other defaults");
    assert_eq!(
        single_override.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&single_override.stdout)
    );
    let single_result: serde_json::Value =
        serde_json::from_slice(&single_override.stdout).expect("compile emits an outcome document");
    assert_eq!(single_result["category"], "success");

    let mut command = Command::new(FSL);
    command.arg("compile").arg(&request);
    for (setting, _) in SpineLimits::default().bounds() {
        command
            .arg("--limit")
            .arg(format!("{}=1000000", setting.name()));
    }
    let output = command
        .args(["--format", "json"])
        .output()
        .expect("compile a deep source with caller-raised limits");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("compile emits an outcome document");
    assert_eq!(result["category"], "success");

    let invalid = Command::new(FSL)
        .arg("compile")
        .arg(&request)
        .args(["--limit", "unknown.limit=7", "--format", "json"])
        .output()
        .expect("compile with an unknown limit setting");
    assert_eq!(invalid.status.code(), Some(20));
    let refusal: serde_json::Value =
        serde_json::from_slice(&invalid.stdout).expect("invalid limit emits a refusal document");
    let message = refusal["diagnostics"][0]["message"]
        .as_str()
        .expect("limit refusal has a message");
    assert!(message.contains("unknown.limit"), "{message}");
    assert!(message.contains("s1.nodes"), "{message}");
    assert!(message.contains("s3.nodes"), "{message}");
}

/// Trace: QSL-2190, FR-003-AC-5; compile refuses recognized settings it cannot apply.
/// Provenance: FSL FR-869-AC-4 requires the accepted settings list on this refusal.
#[trace("QSL-2190", "FR-003-AC-5")]
#[test]
fn compile_refuses_recognized_unavailable_limit_setting() {
    let directory = TemporaryDirectory::new("compile-unavailable-limit");
    let missing_request = directory.0.join("missing.json");
    let output = Command::new(FSL)
        .arg("compile")
        .arg(&missing_request)
        .args(["--limit", "admission.work_units=5", "--format", "json"])
        .output()
        .expect("compile with a recognized but unavailable limit");

    assert_eq!(output.status.code(), Some(20));
    let refusal: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("unavailable limit emits a refusal");
    assert_eq!(refusal["category"], "refusal");
    assert_eq!(refusal["diagnostics"][0]["code"], "invalid-request");
    let message = refusal["diagnostics"][0]["message"]
        .as_str()
        .expect("limit refusal has a message");
    assert!(message.contains("admission.work_units"), "{message}");
    assert!(
        message.contains("unavailable for the `compile` operation"),
        "{message}"
    );
    let accepted = message
        .split_once("settings accepted by this command: ")
        .map(|(_, accepted)| accepted)
        .expect("refusal lists the command's accepted settings");
    assert!(accepted.contains("s1.nodes"), "{accepted}");
    assert!(accepted.contains("s3.nodes"), "{accepted}");
    assert!(!accepted.contains("admission.work_units"), "{accepted}");
}

/// Trace: QSL-2190; unknown settings identify the valid settings for the verb.
#[trace("QSL-2190")]
#[test]
fn frontend_limit_unknown_setting_names_accepted_settings() {
    let output = Command::new(FSL)
        .args([
            "parse",
            "issuer",
            "deep",
            SOURCE,
            "--limit",
            "unknown.limit=7",
            "--format",
            "json",
        ])
        .output()
        .expect("parse with an unknown limit setting");
    assert_eq!(output.status.code(), Some(20));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid limit emits a refusal document");
    let message = document["diagnostics"][0]["message"]
        .as_str()
        .expect("limit refusal has a message");
    assert!(message.contains("unknown.limit"), "{message}");
    assert!(message.contains("s1.nodes"), "{message}");
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

/// Trace: QSL-2192; the library owns formatted text and the CLI passes it through unchanged.
#[test]
#[ignore = "QSL-2192: language FR-286-AC-16 OutcomeDocument::from_format result not implemented"]
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
    assert_fsl_document(&json.stdout, &format_outcome_bytes(SOURCE));
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
    // FR-865: text is the default for check, compile, parse and run.
    for command in ["check", "compile", "parse", "run"] {
        let line = command_line(command);
        assert!(line.contains("text by default"), "{line}");
        assert!(line.contains("--format json selects JSON"), "{line}");
        assert!(!line.contains("unavailable"), "{line}");
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
    let replay_line = command_line("replay");
    assert!(replay_line.contains("native-replay/1"), "{replay_line}");
    for command in ["prove", "generate", "inspect", "render"] {
        assert!(command_line(command).contains("unsupported_operation"));
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
/// Trace: FR-255-AC-1, `--limit` is accepted by check and parse for S1 ceilings.
#[test]
fn check_and_parse_accept_raised_s1_limits_for_deep_source() {
    let source_path =
        std::env::temp_dir().join(format!("fsl-qsl-2014-deep-{}.native", std::process::id()));
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile Complete = \"filament.value.complete/v1\";\nfunction f using Complete (x: Boolean): Boolean pure {{ {}true }}\n",
        "not ".repeat(6_000)
    );
    std::fs::write(&source_path, source).expect("write deep source fixture");
    let path = source_path.to_string_lossy().into_owned();
    for arguments in [
        vec![
            "parse",
            "issuer",
            "deep",
            path.as_str(),
            "--limit",
            "s1.nodes=1000000",
            "--format",
            "json",
        ],
        vec![
            "check",
            path.as_str(),
            "--limit",
            "s1.nodes=1000000",
            "--format",
            "json",
        ],
    ] {
        let output = Command::new(FSL)
            .args(arguments)
            .output()
            .expect("run source command with raised S1 node limit");
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(output.stderr.is_empty());
    }
    std::fs::remove_file(source_path).expect("remove deep source fixture");
}

/// Trace: FR-255-AC-2, compile accepts a named limit operand.
#[test]
fn compile_accepts_limit_operand() {
    let request = requests::request(SOURCE, None);
    let output = Command::new(FSL)
        .args([
            "compile",
            request.as_str(),
            "--limit",
            "s1.nodes=1000000",
            "--format",
            "json",
        ])
        .output()
        .expect("run compile with a named limit");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
}

/// Trace: FR-255-AC-4, the format source-byte refusal names the accepted setting.
#[test]
fn format_limit_message_names_the_source_byte_setting() {
    let source_path =
        std::env::temp_dir().join(format!("fsl-qsl-2014-format-{}.native", std::process::id()));
    std::fs::write(&source_path, b"source bytes").expect("write format source fixture");
    let path = source_path.to_string_lossy().into_owned();
    let output = Command::new(FSL)
        .args([
            "format",
            "issuer",
            "identity",
            path.as_str(),
            "--limit",
            "s1.input_bytes=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run format with a low source-byte limit");
    assert_eq!(output.status.code(), Some(22));
    let document = String::from_utf8_lossy(&output.stdout);
    assert!(document.contains("s1.input_bytes"), "{document}");
    std::fs::remove_file(source_path).expect("remove format source fixture");
}

/// Trace: FR-255-AC-6, unknown limits keep FSL's accepted-setting list.
#[test]
fn unknown_limit_setting_lists_accepted_settings() {
    let output = Command::new(FSL)
        .args([
            "parse",
            "issuer",
            "document",
            SOURCE,
            "--limit",
            "not.a.setting=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run parse with an unknown limit setting");
    assert_eq!(output.status.code(), Some(20));
    let document = String::from_utf8_lossy(&output.stdout);
    assert!(document.contains("not.a.setting"), "{document}");
    assert!(document.contains("s1.nodes"), "{document}");
    assert!(document.contains("format.output_bytes"), "{document}");
}

/// Trace: FR-255-AC-3, run limit diagnostics omit request-stage settings.
#[test]
fn run_limit_message_only_offers_the_cli_setting() {
    let request = requests::request(SOURCE, None);
    let output = Command::new(FSL)
        .args([
            "run",
            request.as_str(),
            "--limit",
            "s1.tokens=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run with a low token limit");
    assert_eq!(output.status.code(), Some(22));
    let document = String::from_utf8_lossy(&output.stdout);
    assert!(document.contains("s1.tokens"), "{document}");
    assert!(!document.contains("stage_limits"), "{document}");
}

/// Trace: FR-002-AC-5; the CLI reads only the input path supplied on argv.
#[test]
