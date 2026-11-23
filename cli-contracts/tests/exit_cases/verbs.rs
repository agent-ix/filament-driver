// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-002-AC-1, FR-002-AC-2, FR-002-AC-4
/// Provenance: QSL-2476; FSL FR-285 category-to-exit table.
#[test]
fn routed_source_verbs_preserve_success_in_their_own_operation() -> Result<(), ContractError> {
    let case = fixture()?;
    for (verb, input, operation) in [
        ("parse", "program.fsl", "parse"),
        ("format", "program.fsl", "format"),
        ("check", "program.fsl", "check"),
        ("compile", "compile.json", "package"),
        ("run", "run.json", "execute"),
    ] {
        let args = if matches!(verb, "parse" | "format") {
            vec![
                verb,
                "exit-contracts",
                "exit-program",
                input,
                "--format",
                "json",
            ]
        } else {
            vec![verb, input, "--format", "json"]
        };
        let value = outcome(&case.run(&args)?, operation, "success")?;
        if verb == "run" {
            assert_eq!(
                value["result"],
                json!({"kind":"completed","value":{"type":"integer","value":"11"}})
            );
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-1, FR-002-AC-2, FR-002-AC-4
/// Provenance: QSL-2476; FSL FR-285 refusal row.
#[test]
fn every_routed_verb_preserves_twenty_for_missing_input() -> Result<(), ContractError> {
    let case = fixture()?;
    for (verb, operation, extra) in [
        ("parse", "parse", None),
        ("format", "format", None),
        ("check", "check", None),
        ("compile", "package", None),
        ("run", "execute", None),
        ("analyze", "analyze", None),
        ("monitor", "monitor", None),
        ("replay", "replay", None),
        ("export", "export", Some("--output")),
        ("inspect", "inspect", Some("--handoff")),
    ] {
        let args = match extra {
            Some("--output") => vec![
                verb,
                "missing.json",
                "--output",
                "target.tla",
                "--format",
                "json",
            ],
            Some(option) => vec![verb, option, "missing", "--format", "json"],
            None if matches!(verb, "parse" | "format") => vec![
                verb,
                "exit-contracts",
                "missing",
                "missing.json",
                "--format",
                "json",
            ],
            None => vec![verb, "missing.json", "--format", "json"],
        };
        let value = outcome(&case.run(&args)?, operation, "refusal")?;
        assert!(
            !value["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .is_empty(),
            "{value}"
        );
    }
    assert!(!case.directory().join("target.tla").exists());
    Ok(())
}

/// Trace: FR-002-AC-1, FR-002-AC-2
/// Provenance: QSL-2476; FSL FR-624-AC-4.
#[test]
fn recognized_unrouted_verbs_use_twenty_one_and_name_the_actual_verb() -> Result<(), ContractError>
{
    let case = fixture()?;
    for verb in ["prove", "generate", "render"] {
        let value = outcome(
            &case.run(&[verb, "program.fsl", "--format", "json"])?,
            verb,
            "unsupported",
        )?;
        let diagnostics = value["diagnostics"].as_array().expect("diagnostics");
        assert!(
            diagnostics
                .iter()
                .any(|d| d["code"] == "unsupported_operation"
                    && d["cause"] == "operation-not-routed"
                    && d.to_string().contains(verb)),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-28
/// Provenance: FSL FR-767-AC-7; an unknown option on an unrouted verb refuses
/// exit 20 as an invalid-request outcome before the verb answers unsupported.
#[test]
fn unrouted_prove_refuses_an_unknown_option_before_answering_unsupported(
) -> Result<(), ContractError> {
    let case = fixture()?;
    for flag in ["--integer-division", "--bogus"] {
        let output = case.run(&["prove", "program.fsl", flag, "floor", "--format", "json"])?;
        assert_eq!(output.0.status.code(), Some(20), "{output:?}");
        let value = output.json()?;
        assert_eq!(value["format"], "filament-outcome/1", "{value}");
        assert_eq!(value["operation"], "prove", "{value}");
        assert_eq!(value["category"], "refusal", "{value}");
        let diagnostics = value["diagnostics"].as_array().expect("diagnostics");
        assert!(
            diagnostics.iter().any(|d| d["code"] == "invalid-request"
                && d["message"].as_str().is_some_and(|m| m.contains(flag))),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-2, FR-002-AC-4, FR-002-AC-23
/// Provenance: QSL-2476; FSL FR-285 success row.
#[test]
fn inspect_success_comes_from_the_checked_handoff_not_help() -> Result<(), ContractError> {
    let case = fixture()?;
    let mut request: Value = serde_json::from_slice(
        &std::fs::read(case.directory().join("compile.json")).expect("fixture"),
    )?;
    request["request"]["handoff"] = json!({"directory":"handoff","package_name":"exit-program",
        "native_join":{"work_units":100000,"storage":100000}});
    write_json(&case, "handoff.json", &request)?;
    outcome(
        &case.run(&["compile", "handoff.json", "--format", "json"])?,
        "package",
        "success",
    )?;
    let value = outcome(
        &case.run(&["inspect", "--handoff", "handoff", "--format", "json"])?,
        "inspect",
        "success",
    )?;
    assert_eq!(value["diagnostics"], json!([]), "{value}");
    assert!(case.directory().join("handoff").is_dir());
    Ok(())
}
