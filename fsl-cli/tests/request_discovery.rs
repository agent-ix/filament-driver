// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-028 (TC-050): `fsl <verb> --example` prints the minimal request FSL's
//! `request_example` returns, misuse of the option refuses before any file is
//! read, FSL's request refusals reach the user unchanged, and the one driver
//! addition is the text-mode export hint line.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]

use std::path::Path;
use std::process::{Command, Output, Stdio};

use fsl_command::command::{request_example, ExportSelection};
use ix_trace_rs::trace;
use serde_json::{json, Value};

const FSL: &str = env!("CARGO_BIN_EXE_fsl");

/// The request verbs with the format each reads.
const VERBS: [(&str, &str); 5] = [
    ("run", "native-run/1"),
    ("compile", "native-compile/1"),
    ("analyze", "native-analyze/1"),
    ("monitor", "native-monitor/1"),
    ("replay", "native-replay/1"),
];

fn fsl_in(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(FSL)
        .args(arguments)
        .current_dir(directory)
        .stdin(Stdio::null())
        .output()
        .expect("run fsl")
}

fn empty_directory() -> tempfile::TempDir {
    tempfile::tempdir().expect("empty working directory")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

fn document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("one document on stdout: {error}: {output:?}"))
}

fn message(document: &Value) -> String {
    document["diagnostics"][0]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("a diagnostic message: {document}"))
        .to_owned()
}

fn example_of(format: &str, selection: Option<ExportSelection>) -> String {
    request_example(format, selection).expect("FSL knows the request format")
}

/// FR-028-AC-1: each request verb writes FSL's example plus one newline, an
/// object of `format` and `request`, with no outcome document, no stderr and
/// the same bytes in every output mode.
#[test]
#[trace("TC-050", "FR-028-AC-1")]
fn each_request_verb_prints_fsls_example_in_every_output_mode() {
    let directory = empty_directory();
    for (verb, format) in VERBS {
        let expected = format!("{}\n", example_of(format, None));
        for extra in [&[][..], &["--format", "text"][..], &["--format", "json"][..]] {
            let mut arguments = vec![verb, "--example"];
            arguments.extend(extra);
            let output = fsl_in(directory.path(), &arguments);
            assert_eq!(output.status.code(), Some(0), "{arguments:?}: {output:?}");
            assert!(output.stderr.is_empty(), "{arguments:?}: {:?}", output.stderr);
            assert_eq!(stdout(&output), expected, "{arguments:?}");
        }
        let parsed: Value = serde_json::from_str(&expected).expect("the example is JSON");
        let members: Vec<&String> = parsed.as_object().expect("an object").keys().collect();
        assert_eq!(members, ["format", "request"], "{verb}");
        assert_eq!(parsed["format"], format, "{verb}");
        assert!(parsed.get("outcome").is_none() && parsed.get("operation").is_none());
    }
}

/// FR-028-AC-2: `export` prints the ordinary or the evidence reading of a
/// `native-run/1` request, and refuses both options together.
#[test]
#[trace("TC-050", "FR-028-AC-2")]
fn export_prints_its_ordinary_and_evidence_examples() {
    let directory = empty_directory();
    for (option, selection, present, absent) in [
        (
            "--example",
            ExportSelection::Ordinary,
            "export",
            "export_evidence",
        ),
        (
            "--example-evidence",
            ExportSelection::Evidence,
            "export_evidence",
            "export",
        ),
    ] {
        let output = fsl_in(directory.path(), &["export", option]);
        assert_eq!(output.status.code(), Some(0), "{option}: {output:?}");
        assert!(output.stderr.is_empty(), "{option}: {:?}", output.stderr);
        let expected = format!("{}\n", example_of("native-run/1", Some(selection)));
        assert_eq!(stdout(&output), expected, "{option}");
        let parsed: Value = serde_json::from_str(&expected).expect("the example is JSON");
        assert!(parsed["request"].get(present).is_some(), "{option}: {parsed}");
        assert!(parsed["request"].get(absent).is_none(), "{option}: {parsed}");
    }
    let both = fsl_in(
        directory.path(),
        &["export", "--example", "--example-evidence"],
    );
    assert_eq!(both.status.code(), Some(20), "{both:?}");
    let shown = stdout(&both);
    assert!(shown.contains("--example"), "{shown}");
    assert!(shown.contains("--example-evidence"), "{shown}");
}

/// FR-028-AC-3: the example of each verb, saved and given back to it over an
/// empty directory, is read as a request: its refusal, if any, is about a file
/// it names and never about the envelope or the request shape.
#[test]
#[trace("TC-050", "FR-028-AC-3")]
fn each_example_is_read_as_a_valid_request() {
    let directory = empty_directory();
    let mut cases: Vec<(String, String, Vec<&str>)> = VERBS
        .iter()
        .map(|(verb, format)| {
            (
                (*verb).to_owned(),
                example_of(format, None),
                vec![*verb],
            )
        })
        .collect();
    cases.push((
        "export".to_owned(),
        example_of("native-run/1", Some(ExportSelection::Ordinary)),
        vec!["export"],
    ));
    cases.push((
        "export --evidence".to_owned(),
        example_of("native-run/1", Some(ExportSelection::Evidence)),
        vec!["export", "--evidence"],
    ));
    for (name, example, prefix) in cases {
        let file = directory.path().join("example-request.json");
        std::fs::write(&file, &example).expect("save the example");
        let mut arguments = prefix.clone();
        arguments.extend([file.to_str().expect("UTF-8 path"), "--format", "json"]);
        let output = fsl_in(directory.path(), &arguments);
        let shown = stdout(&output);
        assert!(
            !shown.contains("missing required")
                && !shown.contains("is not read by")
                && !shown.contains("selects exactly one of program and package"),
            "{name}: refused as a malformed request: {shown}"
        );
        assert!(output.stderr.is_empty(), "{name}: {:?}", output.stderr);
        if output.status.code() != Some(0) {
            assert!(
                shown.contains("program.fsl")
                    || shown.contains("package.json")
                    || shown.contains("package")
                    || shown.contains("entry.fsl")
                    || shown.contains("records.jsonl")
                    || shown.contains("proof.txt")
                    || shown.contains("example"),
                "{name} did not name a file the example names: {shown}"
            );
        }
    }
}

/// FR-028-AC-4: `--example` refuses an operand, another option or the
/// evidence option, naming both, before any file is read; help wins, a bad
/// `--format` refuses first, and the verbs that read no request refuse the
/// option as unknown.
#[test]
#[trace("TC-050", "FR-028-AC-4")]
fn example_misuse_refuses_naming_both_arguments() {
    let directory = empty_directory();
    for (arguments, other) in [
        (vec!["run", "--example", "R.json"], "R.json"),
        (vec!["run", "--example", "--limit", "s1.nodes=1"], "--limit"),
        (
            vec!["compile", "--example", "--output", "m.pkg"],
            "--output",
        ),
        (
            vec!["export", "--example", "--example-evidence"],
            "--example-evidence",
        ),
    ] {
        let output = fsl_in(directory.path(), &arguments);
        assert_eq!(output.status.code(), Some(20), "{arguments:?}: {output:?}");
        assert!(output.stderr.is_empty(), "{arguments:?}: {:?}", output.stderr);
        let shown = stdout(&output);
        assert!(shown.contains("--example"), "{arguments:?}: {shown}");
        assert!(shown.contains(other), "{arguments:?}: {shown}");
        assert!(!directory.path().join("m.pkg").exists());
    }
    let help = fsl_in(directory.path(), &["run", "--example", "--help"]);
    assert_eq!(help.status.code(), Some(0), "{help:?}");
    assert!(stdout(&help).starts_with("Usage: fsl run"), "{help:?}");
    let bogus = fsl_in(directory.path(), &["run", "--example", "--format", "bogus"]);
    assert_eq!(bogus.status.code(), Some(20), "{bogus:?}");
    let shown = stdout(&bogus);
    assert!(shown.contains("--format") && shown.contains("bogus"), "{shown}");
    for arguments in [
        vec!["check", "--example"],
        vec!["parse", "--example"],
        vec!["format", "--example"],
        vec!["inspect", "--example"],
        vec!["run", "--example-evidence"],
    ] {
        let output = fsl_in(directory.path(), &arguments);
        assert_eq!(output.status.code(), Some(20), "{arguments:?}: {output:?}");
        let shown = stdout(&output);
        assert!(
            shown.contains(&format!("unknown {} option `--example", arguments[0])),
            "{arguments:?}: {shown}"
        );
    }
    let root = stdout(&fsl_in(directory.path(), &["--help"]));
    for (verb, _) in VERBS {
        let help = stdout(&fsl_in(directory.path(), &[verb, "--help"]));
        assert!(help.contains("\n  --example\n"), "{verb}: {help}");
        assert!(root.contains(&format!("{verb} options:\n")), "{verb}");
    }
    let export = stdout(&fsl_in(directory.path(), &["export", "--help"]));
    assert!(export.contains("\n  --example\n"), "{export}");
    assert!(export.contains("\n  --example-evidence\n"), "{export}");
    assert_eq!(
        root.matches("\n  --example\n").count(),
        VERBS.len() + 1,
        "{root}"
    );
    assert_eq!(root.matches("\n  --example-evidence\n").count(), 1, "{root}");
}

/// The `details.missing` of a refusal document.
fn missing(document: &Value) -> Value {
    document["diagnostics"][0]["details"]["missing"].clone()
}

/// The member names of `text`'s `request` object, in the order the example
/// prints them (FSL reports missing members in that order).
fn request_member_order(text: &str) -> Vec<String> {
    use serde::de::{Deserializer, IgnoredAny, MapAccess, Visitor};

    struct Keys(Vec<String>);
    impl<'de> Visitor<'de> for Keys {
        type Value = Vec<String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an object")
        }
        fn visit_map<A: MapAccess<'de>>(mut self, mut map: A) -> Result<Self::Value, A::Error> {
            while let Some(key) = map.next_key::<String>()? {
                map.next_value::<IgnoredAny>()?;
                self.0.push(key);
            }
            Ok(self.0)
        }
    }
    struct Outer;
    impl<'de> Visitor<'de> for Outer {
        type Value = Vec<String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an object")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut found = Vec::new();
            while let Some(key) = map.next_key::<String>()? {
                if key == "request" {
                    found = map.next_value_seed(KeysSeed)?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(found)
        }
    }
    struct KeysSeed;
    impl<'de> serde::de::DeserializeSeed<'de> for KeysSeed {
        type Value = Vec<String>;
        fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
            deserializer.deserialize_map(Keys(Vec::new()))
        }
    }
    serde_json::Deserializer::from_str(text)
        .deserialize_map(Outer)
        .expect("the example is a JSON object")
}

/// The `missing` array the verb's own example gives for `{"request": {}}`:
/// one element per top-level member of the example's `request`, a one-of
/// group's example member replaced by the array of the group's pointers
/// (FSL FR-1047's Definitions).
fn expected_missing(verb: &str, format: &str) -> Value {
    let groups = |name: &str| -> Option<Vec<&str>> {
        match (verb, name) {
            ("run", "program") | ("analyze", "program") => Some(vec!["program", "package"]),
            ("run", "call") => Some(vec!["call", "clause", "items"]),
            _ => None,
        }
    };
    Value::Array(
        request_member_order(&example_of(format, None))
            .iter()
            .map(|name| match groups(name.as_str()) {
                Some(group) => Value::Array(
                    group
                        .iter()
                        .map(|member| json!(format!("/request/{member}")))
                        .collect(),
                ),
                None => json!(format!("/request/{name}")),
            })
            .collect(),
    )
}

/// FR-028-AC-5: a request with no members names every missing one at once.
#[test]
#[trace("TC-050", "FR-028-AC-5")]
fn a_request_lacking_members_lists_all_of_them() {
    let directory = empty_directory();
    for (verb, format) in VERBS {
        for (name, body, expected) in [
            ("empty", "{}".to_owned(), json!(["/format", "/request"])),
            (
                "format only",
                format!("{{\"format\":\"{format}\"}}"),
                json!(["/request"]),
            ),
            (
                "empty request",
                format!("{{\"format\":\"{format}\",\"request\":{{}}}}"),
                expected_missing(verb, format),
            ),
        ] {
            let file = directory.path().join("request.json");
            std::fs::write(&file, body).expect("write the request");
            let output = fsl_in(
                directory.path(),
                &[verb, file.to_str().expect("UTF-8 path"), "--format", "json"],
            );
            assert_eq!(output.status.code(), Some(20), "{verb} {name}: {output:?}");
            assert!(output.stderr.is_empty(), "{verb} {name}: {:?}", output.stderr);
            let document = document(&output);
            assert_eq!(missing(&document), expected, "{verb} {name}: {document}");
            let shown = message(&document);
            assert!(
                !shown.contains(" line ") && !shown.contains(" column "),
                "{verb} {name}: {shown}"
            );
            assert!(document["diagnostics"][0]["details"].get("line").is_none());
        }
    }
}

/// An empty `native-run/1` request, saved as `request.json`.
fn run_request_without_members(directory: &Path) -> std::path::PathBuf {
    let file = directory.join("request.json");
    std::fs::write(&file, r#"{"format":"native-run/1","request":{}}"#).expect("write");
    file
}

/// FR-028-AC-7: an export refusal for the missing member names the member
/// in `details.missing`, and text mode alone ends with the hint line.
#[test]
#[trace("TC-050", "FR-028-AC-7")]
fn export_refusals_end_with_the_hint_in_text_mode_only() {
    let directory = empty_directory();
    let file = run_request_without_members(directory.path());
    let file = file.to_str().expect("UTF-8 path");
    for (arguments, member, hint) in [
        (
            vec!["export", file],
            "/request/export",
            "hint: run 'fsl export --example' to print a valid request\n",
        ),
        (
            vec!["export", "--evidence", file],
            "/request/export_evidence",
            "hint: run 'fsl export --example-evidence' to print a valid request\n",
        ),
    ] {
        let mut json_arguments = arguments.clone();
        json_arguments.extend(["--format", "json"]);
        let machine = fsl_in(directory.path(), &json_arguments);
        assert_eq!(machine.status.code(), Some(20), "{machine:?}");
        assert!(machine.stderr.is_empty(), "{:?}", machine.stderr);
        let refusal = document(&machine);
        let listed = missing(&refusal);
        assert!(
            listed
                .as_array()
                .expect("a missing array")
                .contains(&json!(member)),
            "{refusal}"
        );
        let shown = message(&refusal);
        assert!(
            !shown.contains("requires export") && !shown.contains("--example"),
            "{shown}"
        );
        assert!(!stdout(&machine).contains("hint:"), "JSON mode has no hint");

        let text = fsl_in(directory.path(), &arguments);
        assert_eq!(text.status.code(), Some(20), "{text:?}");
        assert!(text.stderr.is_empty(), "{:?}", text.stderr);
        let shown = stdout(&text);
        assert!(shown.ends_with(hint), "{shown}");
        let before = shown.strip_suffix(hint).expect("the hint is last");
        assert!(!before.contains("hint:"), "{shown}");
        assert!(before.starts_with("export: refusal (exit 20)"), "{shown}");
        assert!(!message(&refusal).contains("hint"));
    }
}

/// FR-028-AC-7: in text mode every export refusal is FSL's text rendering,
/// hint or not; only `--format json` prints the refusal document.
#[test]
#[trace("TC-050", "FR-028-AC-7")]
fn export_refusals_without_a_hint_print_the_text_rendering() {
    let directory = empty_directory();
    let file = directory.path().join("request.json");
    std::fs::write(&file, r#"{"format":"native-analyze/1","request":{}}"#).expect("write");
    let file = file.to_str().expect("UTF-8 path");
    for arguments in [vec!["export", file], vec!["export", "--evidence", file]] {
        let text = fsl_in(directory.path(), &arguments);
        assert_eq!(text.status.code(), Some(20), "{text:?}");
        assert!(text.stderr.is_empty(), "{:?}", text.stderr);
        let shown = stdout(&text);
        assert!(shown.starts_with("export: refusal (exit 20)"), "{shown}");
        assert!(!shown.starts_with('{') && !shown.contains("hint:"), "{shown}");

        let mut json_arguments = arguments.clone();
        json_arguments.extend(["--format", "json"]);
        let machine = fsl_in(directory.path(), &json_arguments);
        assert_eq!(machine.status.code(), Some(20), "{machine:?}");
        let refusal = document(&machine);
        assert!(!message(&refusal).is_empty(), "{refusal}");
    }
}

/// FR-028-AC-7 and FR-028-AC-5: a refusal that lacks the export members
/// carries no hint line.
#[test]
#[trace("TC-050", "FR-028-AC-7")]
fn other_refusals_carry_no_hint() {
    let directory = empty_directory();
    let file = directory.path().join("request.json");
    std::fs::write(&file, r#"{"format":"native-analyze/1","request":{}}"#).expect("write");
    for arguments in [
        vec!["run", file.to_str().expect("UTF-8 path")],
        vec!["analyze", "no-such-request.json"],
    ] {
        let output = fsl_in(directory.path(), &arguments);
        assert_eq!(output.status.code(), Some(20), "{output:?}");
        assert!(
            !stdout(&output).contains("run 'fsl export --example"),
            "{output:?}"
        );
    }
}

/// FR-028-AC-9: a request of another format is refused with the formats the
/// verb's family reads, never as an unknown wire.
#[test]
#[trace("TC-050", "FR-028-AC-9")]
fn a_request_of_another_format_names_the_formats_the_verb_reads() {
    let directory = empty_directory();
    for (verb, format, accepted) in [
        (
            "run",
            "native-analyze/1",
            json!(["native-run/1", "native-monitor/1", "native-replay/1"]),
        ),
        ("analyze", "native-run/1", json!(["native-analyze/1"])),
        ("compile", "native-run/1", json!(["native-compile/1"])),
    ] {
        let file = directory.path().join("request.json");
        std::fs::write(&file, format!("{{\"format\":\"{format}\",\"request\":{{}}}}"))
            .expect("write");
        let output = fsl_in(
            directory.path(),
            &[verb, file.to_str().expect("UTF-8 path"), "--format", "json"],
        );
        assert_eq!(output.status.code(), Some(20), "{verb}: {output:?}");
        let document = document(&output);
        assert_eq!(
            document["diagnostics"][0]["details"],
            json!({"path": "/format", "accepted": accepted}),
            "{verb}: {document}"
        );
        assert_ne!(document["diagnostics"][0]["code"], "unknown_wire", "{document}");
    }
}

/// FR-028-AC-7 and FR-002: an export refusal over a program that does not
/// parse renders its locus as `run` does.
#[test]
#[trace("TC-050", "FR-028-AC-7")]
fn an_export_syntax_error_refusal_renders_as_run_does() {
    let directory = empty_directory();
    let example: Value = serde_json::from_str(&example_of("native-run/1", Some(ExportSelection::Ordinary)))
        .expect("the export example is JSON");
    std::fs::write(directory.path().join("program.fsl"), "this is not a program\n")
        .expect("write the program");
    let file = directory.path().join("request.json");
    std::fs::write(&file, example.to_string()).expect("write the request");
    let output = fsl_in(directory.path(), &["export", file.to_str().expect("UTF-8 path")]);
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    let shown = stdout(&output);
    assert!(shown.starts_with("export: refusal (exit 20)"), "{shown}");
    let run = fsl_in(directory.path(), &["run", file.to_str().expect("UTF-8 path")]);
    let run = stdout(&run);
    // FSL renders the locus of a refusal it raised before any source opened
    // the way `run` does; the export refusal carries the same sources.
    assert_eq!(
        shown.split_once('\n').map(|(_, rest)| rest),
        run.split_once('\n').map(|(_, rest)| rest),
        "{shown}\n{run}"
    );
    assert!(shown.contains("invalid_syntax"), "{shown}");
}
