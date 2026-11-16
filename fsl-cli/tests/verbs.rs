// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verbs the build accepts but does not route (FSpec FR-522, FSL FR-624): each
//! prints a typed `unsupported_operation` document or a named stderr line and
//! exits 21, an unknown verb is a usage error, and no verb exits silently.

use std::process::{Command, Output};

use filament_semantic_value::catalog::Category;
use ix_trace_rs::trace;
use serde_json::Value;

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const UNROUTED: [&str; 5] = ["prove", "replay", "generate", "inspect", "render"];
const ALL_VERBS: [&str; 12] = [
    "parse", "format", "check", "compile", "run", "analyze", "monitor", "prove", "replay",
    "generate", "inspect", "render",
];

fn unsupported_code() -> i32 {
    i32::from(Category::Unsupported.exit_code())
}

fn fsl(arguments: &[&str]) -> Output {
    Command::new(FSL).args(arguments).output().expect("run fsl")
}

fn document(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

fn assert_typed_unsupported(output: &Output, verb: &str) {
    assert_eq!(output.status.code(), Some(unsupported_code()), "{verb}");
    let document = document(output);
    assert_eq!(document["operation"], verb);
    assert_eq!(document["category"], "unsupported", "{verb}");
    let diagnostic = &document["diagnostics"][0];
    assert_eq!(diagnostic["code"], "unsupported_operation", "{verb}");
    assert_eq!(diagnostic["cause"], "operation-not-routed", "{verb}");
    assert_eq!(diagnostic["detail"]["verb"], verb);
    assert!(
        diagnostic["message"]
            .as_str()
            .is_some_and(|message| message.contains(verb)),
        "{verb}: {diagnostic}"
    );
}

#[trace("TC-1082", "FSpec-FR-522-AC-1")]
#[trace("TC-1082", "FSpec-FR-301-AC-4")]
#[trace("TC-1584", "FR-624-AC-4")]
#[test]
fn unbuilt_verb_prints_typed_unsupported() {
    for verb in UNROUTED {
        let output = fsl(&[verb, "--format", "json"]);
        assert_typed_unsupported(&output, verb);
    }
}

#[trace("TC-1082", "FSpec-FR-522-AC-1")]
#[trace("TC-1584", "FR-624-AC-4")]
#[test]
fn unbuilt_verb_without_operands_is_typed() {
    for verb in UNROUTED {
        assert_typed_unsupported(&fsl(&[verb, "--format", "json"]), verb);
        assert_typed_unsupported(
            &fsl(&[verb, "missing-operand.json", "--format", "json"]),
            verb,
        );
    }
}

#[trace("TC-1082", "FSpec-FR-522-AC-1")]
#[trace("TC-1584", "FR-624-AC-5")]
#[test]
fn unbuilt_verb_text_mode_names_the_verb_on_stderr() {
    for verb in UNROUTED {
        for arguments in [vec![verb], vec![verb, "--format", "text"]] {
            let output = fsl(&arguments);
            assert_eq!(
                output.status.code(),
                Some(unsupported_code()),
                "{arguments:?}"
            );
            assert!(output.stdout.is_empty(), "{arguments:?}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            let expected =
                format!("fsl {verb}: this build does not route the {verb} operation (unsupported_operation)");
            assert_eq!(stderr.lines().count(), 1, "{arguments:?}: {stderr}");
            assert!(stderr.contains(&expected), "{arguments:?}: {stderr}");
        }
    }
}

#[trace("TC-1082", "FSpec-FR-522-AC-1")]
#[test]
fn inspect_with_dot_format_is_not_silent() {
    let output = fsl(&["inspect", "--format", "dot"]);
    assert_eq!(output.status.code(), Some(unsupported_code()));
    assert!(String::from_utf8_lossy(&output.stderr).contains("inspect"));
}

#[trace("TC-1082", "FSpec-FR-301-AC-4")]
#[test]
fn unknown_verb_is_usage_error() {
    let output = fsl(&["not-a-verb", "--format", "json"]);
    let code = output.status.code();
    assert_eq!(code, Some(i32::from(Category::Refusal.exit_code())));
    assert_ne!(code, Some(unsupported_code()));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown verb `not-a-verb`"));
}

#[trace("TC-1082", "FSpec-FR-522-AC-2")]
#[trace("TC-1082", "FSpec-FR-301-AC-4")]
#[test]
fn no_verb_exits_silently() {
    for verb in ALL_VERBS {
        for format in ["json", "text"] {
            let output = fsl(&[verb, "--format", format]);
            let code = output.status.code();
            assert!(code.is_some(), "{verb} {format}: killed by a signal");
            assert!(
                !output.stdout.is_empty() || !output.stderr.is_empty(),
                "{verb} {format}: exit {code:?} with no output"
            );
            if format == "json" && output.status.code() != Some(0) {
                assert!(!output.stdout.is_empty(), "{verb} json: empty stdout");
            }
        }
    }
}
