// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-003-AC-11 and FR-002's refusal routing: `analyze`, `monitor` and
//! `replay` take repeatable `--limit name=value`, and every refusal on every
//! verb is written to stdout in the selected output mode with nothing on
//! stderr, the mode being resolved before any other option is validated.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]

use std::path::Path;
use std::process::{Command, Output, Stdio};

use ix_trace_rs::trace;
use serde_json::Value;

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
const CHECK_UNKNOWN_BOGUS: &str = "unknown check option `--bogus`; check accepts --markdown, --model, --library, --dependent-bytes, --limit, --integer-division, --extensions, --format";

fn fsl_in(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(FSL)
        .args(arguments)
        .current_dir(directory)
        .stdin(Stdio::null())
        .output()
        .expect("run fsl")
}

/// An empty working directory, so a named file is absent.
fn empty_directory() -> tempfile::TempDir {
    tempfile::tempdir().expect("empty working directory")
}

fn document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("one outcome document on stdout: {error}: {output:?}"))
}

fn message(document: &Value) -> String {
    document["diagnostics"][0]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("a diagnostic message: {document}"))
        .to_owned()
}

/// The `analyze-counter` request asking for the refuted claim `BelowThree`,
/// whose counterexample FSL replays, in its own directory.
fn replaying_request() -> (tempfile::TempDir, &'static str) {
    let directory = empty_directory();
    for file in [
        "analyze-counter.fsl",
        "analyze-counter.semantic-ir.json",
        "analyze-counter-start.json",
    ] {
        std::fs::copy(Path::new(FIXTURES).join(file), directory.path().join(file))
            .expect("copy fixture");
    }
    let request = std::fs::read_to_string(Path::new(FIXTURES).join("analyze-counter.json"))
        .expect("read request fixture")
        .replace("\"Bounded\"", "\"BelowThree\"");
    std::fs::write(directory.path().join("request.json"), request).expect("write request");
    (directory, "request.json")
}

/// FR-003-AC-1, FR-003-AC-3, FR-003-AC-11: a caller `--limit` on `analyze`
/// overrides the matching bound FSL exposes (`replay.input_bytes`); a bound
/// small enough to starve the replay read turns the refutation into FSL's
/// typed limit outcome naming the setting, and a larger one reproduces the
/// unlimited document byte for byte, so no other request value moved.
#[test]
#[trace("TC-010", "FR-003-AC-11")]
#[trace("TC-036", "FR-022-AC-5")]
fn analyze_limit_overrides_the_replay_read_bound() {
    let (directory, request) = replaying_request();
    let unlimited = fsl_in(directory.path(), &["analyze", request, "--format", "json"]);
    assert_eq!(unlimited.status.code(), Some(10), "{unlimited:?}");
    assert!(unlimited.stderr.is_empty(), "{:?}", unlimited.stderr);

    let starved = fsl_in(
        directory.path(),
        &[
            "analyze",
            request,
            "--limit",
            "replay.input_bytes=1",
            "--format",
            "json",
        ],
    );
    assert_eq!(starved.status.code(), Some(22), "{starved:?}");
    assert!(starved.stderr.is_empty(), "{:?}", starved.stderr);
    let starved = document(&starved);
    let cause = &starved["items"][0]["record"]["cause"];
    assert_eq!(cause["code"], "stage_limit_exceeded", "{starved}");
    assert_eq!(cause["setting"], "replay.input_bytes", "{starved}");
    assert_eq!(cause["bound"], "1", "{starved}");

    let raised = fsl_in(
        directory.path(),
        &[
            "analyze",
            request,
            "--limit",
            "replay.input_bytes=1048576",
            "--format",
            "json",
        ],
    );
    assert_eq!(raised.status.code(), Some(10), "{raised:?}");
    assert_eq!(
        raised.stdout, unlimited.stdout,
        "the raised bound changes nothing else"
    );

    let text = fsl_in(
        directory.path(),
        &["analyze", request, "--limit", "replay.input_bytes=1"],
    );
    assert_eq!(text.status.code(), Some(22), "{text:?}");
    assert!(text.stderr.is_empty(), "{:?}", text.stderr);
    let text = String::from_utf8_lossy(&text.stdout);
    assert!(text.contains("replay.input_bytes"), "{text}");
}

/// FR-003-AC-2, FR-003-AC-5, FR-003-AC-7, FR-003-AC-8, FR-003-AC-11: a
/// setting an operation does not expose, an unknown name, a malformed operand
/// and a duplicate each refuse before the request file is read, in both
/// modes, naming the setting and the operation; the stream rule holds.
#[test]
#[trace("TC-010", "FR-003-AC-11")]
fn unavailable_unknown_malformed_and_duplicate_limits_refuse_before_work() {
    let directory = empty_directory();
    // Each operation's recognized-but-unavailable setting (FSL FR-255).
    for (verb, unavailable_setting) in [
        ("analyze", "monitor.preparation_storage"),
        ("monitor", "replay.input_bytes"),
        ("replay", "monitor.preparation_storage"),
    ] {
        let unavailable_operand = format!("{unavailable_setting}=5");
        let cases: [(&[&str], &str); 5] = [
            (&["--limit", &unavailable_operand], "unavailable for the `"),
            (&["--limit", "nosuch.setting=1"], "nosuch.setting"),
            (&["--limit", "s1.nodes"], "s1.nodes"),
            (&["--limit", "s1.nodes=x"], "s1.nodes=x"),
            (
                &["--limit", "s1.nodes=1", "--limit", "s1.nodes=2"],
                "s1.nodes",
            ),
        ];
        for (limit_arguments, expected) in cases {
            for format in ["text", "json"] {
                let mut arguments = vec![verb, "nosuch-request.json"];
                arguments.extend(limit_arguments);
                arguments.extend(["--format", format]);
                let output = fsl_in(directory.path(), &arguments);
                let context = format!("{arguments:?}");
                assert_eq!(output.status.code(), Some(20), "{context}: {output:?}");
                assert!(output.stderr.is_empty(), "{context}: {:?}", output.stderr);
                let shown = if format == "json" {
                    let document = document(&output);
                    assert_eq!(document["operation"], verb, "{context}: {document}");
                    message(&document)
                } else {
                    let text = String::from_utf8_lossy(&output.stdout).into_owned();
                    assert!(
                        text.starts_with(&format!("{verb}: refusal (exit 20)")),
                        "{context}: {text}"
                    );
                    text
                };
                assert!(shown.contains(expected), "{context}: {shown}");
                assert!(
                    !shown.contains("nosuch-request.json"),
                    "{context} read its request: {shown}"
                );
            }
        }
        let unavailable = fsl_in(
            directory.path(),
            &[
                verb,
                "nosuch-request.json",
                "--limit",
                &unavailable_operand,
                "--format",
                "json",
            ],
        );
        let shown = message(&document(&unavailable));
        assert!(
            shown.starts_with(&format!(
                "limit setting `{unavailable_setting}` is unavailable for the `{verb}` operation; settings this command accepts, by stage: "
            )),
            "{shown}"
        );
        assert!(!shown.contains("nosuch-request.json"), "{shown}");
    }
}

/// FR-003-AC-12: `replay.input_bytes` is exposed by `analyze` and `replay`,
/// whose requests are then read, and is unavailable to `monitor`.
#[test]
#[trace("TC-010", "FR-003-AC-12")]
fn replay_input_bytes_is_accepted_by_analyze_and_replay_not_monitor() {
    let directory = empty_directory();
    for verb in ["analyze", "replay"] {
        let output = fsl_in(
            directory.path(),
            &[
                verb,
                "nosuch-request.json",
                "--limit",
                "replay.input_bytes=1",
                "--format",
                "json",
            ],
        );
        assert_eq!(output.status.code(), Some(20), "{output:?}");
        assert!(
            message(&document(&output)).contains("nosuch-request.json"),
            "{verb}: {output:?}"
        );
    }
    let output = fsl_in(
        directory.path(),
        &[
            "monitor",
            "nosuch-request.json",
            "--limit",
            "replay.input_bytes=1",
            "--format",
            "json",
        ],
    );
    assert!(output.stderr.is_empty(), "{output:?}");
    assert!(
        message(&document(&output)).starts_with(
            "limit setting `replay.input_bytes` is unavailable for the `monitor` operation"
        ),
        "{output:?}"
    );
}

/// FR-003-AC-12: each of `analyze`, `monitor` and `replay` exposes exactly
/// FSL's `CommandLimits` set for its operation (less `monitor`'s two
/// preparation settings until QSL-2986) and every accounting counter; each accepted operand reads the request file, and a setting
/// outside the set refuses before it.
#[test]
#[trace("TC-010", "FR-003-AC-12")]
fn each_operation_accepts_its_exact_command_limit_set() {
    use fsl_command::command::{CommandLimits, MonitorPreparationLimits};
    use fsl_foundation::{AccountingSetting, Setting, SettingName};

    let directory = empty_directory();
    let monitor = CommandLimits::monitor(MonitorPreparationLimits {
        work: 1,
        storage: 1,
    });
    let operations = [
        ("analyze", CommandLimits::analyze()),
        ("monitor", monitor),
        ("replay", CommandLimits::replay()),
    ];
    for (verb, limits) in &operations {
        let names = Setting::ALL
            .iter()
            .map(|setting| SettingName::Stage(*setting))
            .chain(
                AccountingSetting::ALL
                    .iter()
                    .map(|setting| SettingName::Accounting(*setting)),
            );
        for name in names {
            let operand = format!("{}=7", name.name());
            let output = fsl_in(
                directory.path(),
                &[
                    verb,
                    "nosuch-request.json",
                    "--limit",
                    &operand,
                    "--format",
                    "json",
                ],
            );
            let shown = message(&document(&output));
            // `fsl monitor` refuses the preparation settings until QSL-2986
            // enforces them.
            let accepted = match name {
                SettingName::Stage(setting) => {
                    limits.exposes(setting)
                        && !(*verb == "monitor"
                            && matches!(
                                setting,
                                Setting::MonitorPreparationWorkUnits
                                    | Setting::MonitorPreparationStorage
                            ))
                }
                SettingName::Accounting(_) => true,
            };
            assert_eq!(output.status.code(), Some(20), "{verb} {operand}: {shown}");
            if accepted {
                assert!(
                    shown.contains("nosuch-request.json"),
                    "{verb} {operand}: {shown}"
                );
            } else {
                assert!(
                    shown.starts_with(&format!(
                        "limit setting `{}` is unavailable for the `{verb}` operation",
                        name.name()
                    )),
                    "{verb} {operand}: {shown}"
                );
            }
        }
    }
}

/// FR-002-AC-18, FR-002-AC-24, FR-002-AC-28, FR-006-AC-24, FR-006-AC-29: a
/// missing request file, an unrouted verb, an unknown option, a provider
/// configuration and a provider limit refusal each write their outcome to
/// stdout in the selected mode and nothing to stderr.
#[test]
#[trace("TC-040", "FR-002-AC-18")]
#[trace("TC-049", "FR-002-AC-24")]
#[trace("TC-041", "FR-006-AC-29")]
fn every_refusal_is_written_to_stdout_in_the_selected_mode() {
    let directory = empty_directory();
    // (arguments, exit status, document `operation`, text header)
    let cases: Vec<(Vec<&str>, i32, &str, &str)> = vec![
        (
            vec!["run", "nosuch.json"],
            20,
            "execute",
            "run: refusal (exit 20)",
        ),
        (
            vec!["analyze", "nosuch.json"],
            20,
            "analyze",
            "analyze: refusal (exit 20)",
        ),
        (
            vec!["replay", "nosuch.json"],
            20,
            "replay",
            "replay: refusal (exit 20)",
        ),
        (
            vec!["monitor", "nosuch.json"],
            20,
            "monitor",
            "monitor: refusal (exit 20)",
        ),
        (
            vec!["prove", "R"],
            21,
            "prove",
            "prove: unsupported (exit 21)",
        ),
        (
            vec!["run", "--bogus"],
            20,
            "execute",
            "run: refusal (exit 20)",
        ),
        (
            vec!["check", "--bogus"],
            20,
            "check",
            "check: refusal (exit 20)",
        ),
        (
            vec!["analyze", "--bogus"],
            20,
            "analyze",
            "analyze: refusal (exit 20)",
        ),
        (
            vec!["monitor", "--bogus"],
            20,
            "monitor",
            "monitor: refusal (exit 20)",
        ),
        (
            vec!["replay", "--bogus"],
            20,
            "replay",
            "replay: refusal (exit 20)",
        ),
        (
            vec!["prove", "R", "--bogus"],
            20,
            "prove",
            "prove: refusal (exit 20)",
        ),
        (
            vec![
                "run",
                "nosuch.json",
                "--provider-config",
                "absent-providers.json",
            ],
            20,
            "execute",
            "run: refusal (exit 20)",
        ),
        (
            vec![
                "analyze",
                "nosuch.json",
                "--provider-config",
                "absent-providers.json",
            ],
            20,
            "analyze",
            "analyze: refusal (exit 20)",
        ),
        (
            vec![
                "monitor",
                "nosuch.json",
                "--provider-config",
                "absent-providers.json",
            ],
            20,
            "monitor",
            "monitor: refusal (exit 20)",
        ),
        (
            vec![
                "replay",
                "nosuch.json",
                "--provider-config",
                "absent-providers.json",
            ],
            20,
            "replay",
            "replay: refusal (exit 20)",
        ),
        (
            vec!["run", "nosuch.json", "--provider-limit", "deadline_ms=0"],
            20,
            "execute",
            "run: refusal (exit 20)",
        ),
        (
            vec![
                "analyze",
                "nosuch.json",
                "--provider-limit",
                "deadline_ms=0",
            ],
            20,
            "analyze",
            "analyze: refusal (exit 20)",
        ),
        (
            vec![
                "monitor",
                "nosuch.json",
                "--provider-limit",
                "deadline_ms=0",
            ],
            20,
            "monitor",
            "monitor: refusal (exit 20)",
        ),
        (
            vec!["replay", "nosuch.json", "--provider-limit", "deadline_ms=0"],
            20,
            "replay",
            "replay: refusal (exit 20)",
        ),
    ];
    for (arguments, status, operation, header) in cases {
        let default_mode = fsl_in(directory.path(), &arguments);
        let mut explicit = arguments.clone();
        explicit.extend(["--format", "text"]);
        let text_mode = fsl_in(directory.path(), &explicit);
        let mut machine = arguments.clone();
        machine.extend(["--format", "json"]);
        let json_mode = fsl_in(directory.path(), &machine);
        for (label, output) in [("default", &default_mode), ("text", &text_mode)] {
            let context = format!("{arguments:?} ({label})");
            assert_eq!(output.status.code(), Some(status), "{context}: {output:?}");
            assert!(output.stderr.is_empty(), "{context}: {:?}", output.stderr);
            let text = String::from_utf8_lossy(&output.stdout);
            assert!(text.starts_with(header), "{context}: {text}");
            assert!(serde_json::from_str::<Value>(&text).is_err(), "{context}");
        }
        assert_eq!(default_mode.stdout, text_mode.stdout, "{arguments:?}");
        let context = format!("{arguments:?} (json)");
        assert_eq!(
            json_mode.status.code(),
            Some(status),
            "{context}: {json_mode:?}"
        );
        assert!(
            json_mode.stderr.is_empty(),
            "{context}: {:?}",
            json_mode.stderr
        );
        let document = document(&json_mode);
        assert_eq!(document["operation"], operation, "{context}: {document}");
        assert_eq!(
            json_mode
                .stdout
                .iter()
                .filter(|byte| **byte == b'\n')
                .count(),
            1,
            "{context}: exactly one document"
        );
    }
}

/// FR-002 mode resolution, FR-006-AC-24 and TC-041: the output mode is
/// resolved before any other option or input is validated, and a missing,
/// repeated or unsupported `--format` renders its own refusal as text.
#[test]
#[trace("TC-041", "FR-006-AC-24")]
#[trace("TC-049", "FR-002-AC-24")]
fn output_mode_is_resolved_before_options_are_validated() {
    let directory = empty_directory();
    let json = fsl_in(directory.path(), &["check", "--bogus", "--format", "json"]);
    assert_eq!(json.status.code(), Some(20), "{json:?}");
    assert!(json.stderr.is_empty(), "{:?}", json.stderr);
    let json = document(&json);
    assert_eq!(json["operation"], "check", "{json}");
    assert_eq!(message(&json), CHECK_UNKNOWN_BOGUS);

    // The mode is selected whether `--format` comes before or after the bad option.
    let before = fsl_in(directory.path(), &["check", "--format", "json", "--bogus"]);
    assert_eq!(before.status.code(), Some(20), "{before:?}");
    assert!(before.stderr.is_empty(), "{:?}", before.stderr);
    assert_eq!(document(&before)["operation"], "check");
    assert_eq!(message(&document(&before)), CHECK_UNKNOWN_BOGUS);

    let text = fsl_in(directory.path(), &["check", "--bogus", "--format", "text"]);
    assert!(text.stderr.is_empty(), "{:?}", text.stderr);
    assert!(
        String::from_utf8_lossy(&text.stdout).starts_with("check: refusal (exit 20)"),
        "{text:?}"
    );

    for arguments in [
        vec!["check", "--bogus", "--format", "xml"],
        vec!["check", "--bogus", "--format"],
        vec!["check", "--bogus", "--format", "json", "--format", "text"],
        vec!["check", "--bogus", "--format", "json", "--format", "json"],
        vec!["check", "--bogus", "--format", "dot"],
        vec!["inspect", "--bogus", "--format", "dot"],
        vec!["inspect", "--handoff", "nosuch", "--format", "dot"],
        vec![
            "analyze",
            "nosuch.json",
            "--provider-config",
            "--format",
            "xml",
        ],
        vec![
            "run",
            "nosuch.json",
            "--provider-limit",
            "deadline_ms=0",
            "--format",
            "xml",
        ],
    ] {
        let output = fsl_in(directory.path(), &arguments);
        let context = format!("{arguments:?}");
        assert_eq!(output.status.code(), Some(20), "{context}: {output:?}");
        assert!(output.stderr.is_empty(), "{context}: {:?}", output.stderr);
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(
            text.starts_with(&format!("{}: refusal (exit 20)", arguments[0])),
            "{context}: {text}"
        );
        assert!(serde_json::from_str::<Value>(&text).is_err(), "{context}");
    }
}

/// FR-002-AC-18, FR-002-AC-28: a missing file and an unreadable one (a
/// directory named as the file) refuse on `parse`, `format`, `compile`,
/// `export` and `inspect` with exactly the text outcome on stdout in text
/// mode and one document in JSON mode, and nothing on stderr.
#[test]
#[trace("TC-040", "FR-002-AC-18")]
fn unreadable_inputs_refuse_on_stdout_for_the_source_and_package_verbs() {
    const HINT: &str =
        "  hint: Check that the path exists and that the process can read or write it.\n";
    let directory = empty_directory();
    std::fs::create_dir(directory.path().join("adir")).expect("a directory");
    std::fs::write(directory.path().join("afile"), b"x").expect("a file");
    let missing = "No such file or directory (os error 2)";
    let is_directory = "Is a directory (os error 21)";
    // (arguments, operation in the document, text header, text diagnostic line)
    let cases: Vec<(Vec<&str>, &str, &str, String)> = vec![
        (
            vec!["parse", "a", "b", "nosuch.fsl"],
            "parse",
            "parse",
            format!("<no-source>: io-error: could not read source nosuch.fsl: {missing}\n{HINT}"),
        ),
        (
            vec!["parse", "a", "b", "adir"],
            "parse",
            "parse",
            format!("<no-source>: io-error: could not read source adir: {is_directory}\n{HINT}"),
        ),
        (
            vec!["format", "a", "b", "nosuch.fsl"],
            "format",
            "format",
            format!("<no-source>: io-error: could not read source nosuch.fsl: {missing}\n{HINT}"),
        ),
        (
            vec!["format", "a", "b", "adir"],
            "format",
            "format",
            format!("<no-source>: io-error: could not read source adir: {is_directory}\n{HINT}"),
        ),
        (
            vec!["compile", "nosuch.json"],
            "package",
            "package",
            format!("<no-source>: io-error: cannot access nosuch.json: {missing}\n{HINT}"),
        ),
        (
            vec!["compile", "adir"],
            "package",
            "package",
            format!("<no-source>: io-error: cannot access adir: {is_directory}\n{HINT}"),
        ),
        (
            vec!["export", "nosuch.json"],
            "export",
            "export",
            format!("<no-source>: io-error: cannot access nosuch.json: {missing}\n{HINT}"),
        ),
        (
            vec!["export", "adir"],
            "export",
            "export",
            format!("<no-source>: io-error: cannot access adir: {is_directory}\n{HINT}"),
        ),
        (
            vec!["inspect", "--handoff", "nosuch"],
            "inspect",
            "inspect",
            format!("<no-source>: handoff-read: cannot read nosuch: {missing}\n"),
        ),
        (
            vec!["inspect", "--handoff", "afile"],
            "inspect",
            "inspect",
            "<no-source>: handoff-read: cannot read afile: Not a directory (os error 20)\n"
                .to_owned(),
        ),
    ];
    for (arguments, operation, header, body) in cases {
        let context = format!("{arguments:?}");
        let text = fsl_in(directory.path(), &arguments);
        assert_eq!(text.status.code(), Some(20), "{context}: {text:?}");
        assert!(text.stderr.is_empty(), "{context}: {:?}", text.stderr);
        assert_eq!(
            String::from_utf8_lossy(&text.stdout),
            format!("{header}: refusal (exit 20)\n{body}"),
            "{context}"
        );
        let mut machine = arguments.clone();
        machine.extend(["--format", "json"]);
        let json = fsl_in(directory.path(), &machine);
        assert_eq!(json.status.code(), Some(20), "{context}: {json:?}");
        assert!(json.stderr.is_empty(), "{context}: {:?}", json.stderr);
        let document = document(&json);
        assert_eq!(document["operation"], operation, "{context}: {document}");
        assert_eq!(document["category"], "refusal", "{context}: {document}");
    }
}

/// FR-002-AC-29: with no recognized verb, or a command that is not valid
/// UTF-8, there is no operation to name in an outcome: one diagnostic line on
/// stderr, nothing on stdout, exit 20.
#[test]
#[trace("TC-049", "FR-002-AC-29")]
fn an_unrecognized_command_writes_one_stderr_line_and_no_outcome() {
    let directory = empty_directory();
    for arguments in [
        vec!["not-a-verb"],
        vec!["not-a-verb", "--format", "json"],
        vec!["not-a-verb", "--format", "text"],
    ] {
        let output = fsl_in(directory.path(), &arguments);
        assert_eq!(output.status.code(), Some(20), "{arguments:?}");
        assert!(
            output.stdout.is_empty(),
            "{arguments:?}: {:?}",
            output.stdout
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(stderr.lines().count(), 1, "{arguments:?}: {stderr}");
        assert!(stderr.contains("unknown verb `not-a-verb`"), "{stderr}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let output = Command::new(FSL)
            .arg(std::ffi::OsStr::from_bytes(&[0xff]))
            .current_dir(directory.path())
            .stdin(Stdio::null())
            .output()
            .expect("run fsl with a non-UTF-8 command");
        assert_eq!(output.status.code(), Some(20));
        assert!(output.stdout.is_empty(), "{:?}", output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(stderr.lines().count(), 1, "{stderr}");
        assert!(stderr.contains("valid UTF-8"), "{stderr}");
    }
}
