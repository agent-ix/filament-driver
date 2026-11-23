// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-002-AC-23
#[test]
fn missing_empty_directory_and_regular_file_paths_are_real_refusals() -> Result<(), ContractError> {
    let case = CliCase::from_environment()?;
    case.write("regular.json", b"{}")?;
    case.write("empty/anchor", b"")?;
    // The reader names the first member it could not read: none for a path that is
    // not a directory, `selection.json` for a directory without one.
    for (directory, member) in [
        ("missing", Value::Null),
        ("empty", json!("selection.json")),
        ("regular.json", Value::Null),
    ] {
        let value = refusal(&inspect(&case, directory)?, "handoff-read");
        assert_eq!(
            value["diagnostics"][0]["locus"],
            json!({"kind":"input","member":member,"operand":directory,"pointer":null})
        );
        assert!(
            value["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains(directory),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn unicode_spaces_absolute_paths_and_a_dot_prefixed_dash_directory_are_supported(
) -> Result<(), ContractError> {
    let case = handoff("handoff é reserve")?;
    verified(&inspect(&case, "handoff é reserve")?);
    let absolute = case.directory().join("handoff é reserve");
    verified(&inspect(&case, absolute.to_str().unwrap())?);
    std::fs::rename(absolute, case.directory().join("-odd")).unwrap();
    verified(&inspect(&case, "./-odd")?);
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn stdin_does_not_replace_the_named_handoff_directory() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    verified(&case.run_stdin(
        &["inspect", "--handoff", "handoff", "--format", "json"],
        b"not a handoff\0\xff",
    )?);
    let output = case.run_stdin(
        &["inspect", "--handoff", "missing", "--format", "json"],
        br#"{"format":"filament.checked-handoff.selection/1"}"#,
    )?;
    refusal(&output, "handoff-read");
    Ok(())
}
