// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-002-AC-23
/// FSL FR-665-AC-1/2: inspect reads the public compile artifact and preserves its bytes.
#[test]
fn a_generated_handoff_is_verified_without_changing_its_artifacts() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    let sidecar = selection(&case);
    let unit = sidecar["package"]["unit"].as_str().unwrap();
    let names = ["selection.json", "checked-package.json", unit];
    let before: Vec<_> = names
        .iter()
        .map(|name| std::fs::read(case.directory().join("handoff").join(name)).unwrap())
        .collect();
    let output = inspect(&case, "handoff")?;
    let value = verified(&output);
    let mut canonical = serde_json::to_vec(&value)?;
    canonical.push(b'\n');
    assert_eq!(output.0.stdout, canonical);
    for (name, bytes) in names.iter().zip(before) {
        assert_eq!(
            std::fs::read(case.directory().join("handoff").join(name)).unwrap(),
            bytes
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-23
/// FSL FR-665-AC-2: changing a counted node produces a selection-axis refusal.
#[test]
fn changing_a_selection_node_count_is_not_a_successful_inspection() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    let mut sidecar = selection(&case);
    let counts = sidecar["package"]["node_counts"].as_object_mut().unwrap();
    let count = counts
        .values_mut()
        .next()
        .expect("original function has counted nodes");
    *count = json!(count.as_u64().unwrap() + 1);
    case.write("handoff/selection.json", &serde_json::to_vec(&sidecar)?)?;
    let value = refusal(&inspect(&case, "handoff")?, "handoff-read-back-differs");
    assert!(
        value["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("package.nodes"),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn a_changed_package_byte_is_refused_by_the_strict_reader() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    let mut bytes = std::fs::read(case.directory().join("handoff/checked-package.json")).unwrap();
    bytes[0] = b'!';
    case.write("handoff/checked-package.json", &bytes)?;
    refusal(&inspect(&case, "handoff")?, "handoff-reader");
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn malformed_empty_and_non_utf8_selection_files_are_refused() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    for bytes in [
        b"".as_slice(),
        b"{",
        b"\xff",
        b"{\"format\":\"not-a-handoff\"}",
    ] {
        case.write("handoff/selection.json", bytes)?;
        refusal(&inspect(&case, "handoff")?, "handoff-sidecar");
    }
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn empty_and_non_utf8_package_files_are_refused() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    for bytes in [b"".as_slice(), b"\xff"] {
        case.write("handoff/checked-package.json", bytes)?;
        refusal(&inspect(&case, "handoff")?, "handoff-reader");
    }
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn removing_a_selected_package_is_an_input_read_refusal() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    std::fs::remove_file(case.directory().join("handoff/checked-package.json")).unwrap();
    let value = refusal(&inspect(&case, "handoff")?, "handoff-read");
    assert_eq!(
        value["diagnostics"][0]["locus"],
        json!({"kind":"input","member":"checked-package.json","operand":"handoff","pointer":null})
    );
    assert!(value["diagnostics"][0]["message"]
        .as_str()
        .unwrap()
        .contains("checked-package.json"));
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn a_selected_source_directory_or_symlink_cannot_be_read_as_a_regular_file(
) -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    let sidecar = selection(&case);
    let unit = sidecar["package"]["unit"].as_str().unwrap();
    let path = case.directory().join("handoff").join(unit);
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    refusal(&inspect(&case, "handoff")?, "handoff-label");
    std::fs::remove_dir(&path).unwrap();
    std::os::unix::fs::symlink(case.directory().join("reserve.fsl"), &path).unwrap();
    refusal(&inspect(&case, "handoff")?, "handoff-label");
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn an_oversized_selected_source_is_not_truncated_and_admitted() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    let sidecar = selection(&case);
    let unit = sidecar["package"]["unit"].as_str().unwrap();
    let path = case.directory().join("handoff").join(unit);
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .unwrap()
        .set_len(1_048_577)
        .unwrap();
    let value = refusal(&inspect(&case, "handoff")?, "handoff-read");
    assert!(
        value["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("not truncated"),
        "{value}"
    );
    Ok(())
}

fn exceed_file_budget(case: &CliCase, file: &str) -> Result<(), ContractError> {
    let ceiling = selection(case)["package"]["read_limits"]["bytes"]
        .as_u64()
        .unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(case.directory().join("handoff").join(file))
        .unwrap()
        .set_len(ceiling.checked_add(1).unwrap())
        .unwrap();
    let value = refusal(&inspect(case, "handoff")?, "handoff-read");
    let message = value["diagnostics"][0]["message"].as_str().unwrap();
    assert!(
        message.contains(file)
            && message.contains("not truncated")
            && message.contains(&ceiling.to_string()),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn oversized_package_bytes_are_refused_before_decoding() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    exceed_file_budget(&case, "checked-package.json")
}

/// Trace: FR-002-AC-23
#[test]
fn oversized_selection_bytes_are_refused_before_decoding() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    exceed_file_budget(&case, "selection.json")
}

/// Trace: FR-002-AC-23
/// FSL FR-665 verification reads only the artifact's contained regular input files.
#[test]
fn forged_parent_relative_and_absolute_source_labels_cannot_escape_the_handoff(
) -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    let original = selection(&case);
    for label in [
        "../reserve.fsl".to_owned(),
        case.directory()
            .join("reserve.fsl")
            .to_str()
            .unwrap()
            .to_owned(),
    ] {
        let mut sidecar = original.clone();
        sidecar["package"]["unit"] = json!(label);
        case.write("handoff/selection.json", &serde_json::to_vec(&sidecar)?)?;
        refusal(&inspect(&case, "handoff")?, "handoff-label");
    }
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn a_sidecar_with_unknown_members_is_not_admitted() -> Result<(), ContractError> {
    let case = handoff("handoff")?;
    let mut sidecar = selection(&case);
    sidecar["forged_admission"] = json!(true);
    case.write("handoff/selection.json", &serde_json::to_vec(&sidecar)?)?;
    refusal(&inspect(&case, "handoff")?, "handoff-sidecar");
    Ok(())
}

/// Trace: FR-002-AC-23
#[test]
fn invalid_selected_source_does_not_reuse_the_otherwise_valid_package() -> Result<(), ContractError>
{
    let case = handoff("handoff")?;
    let sidecar = selection(&case);
    let unit = sidecar["package"]["unit"].as_str().unwrap();
    case.write(format!("handoff/{unit}"), b"\xff")?;
    refusal(&inspect(&case, "handoff")?, "handoff-compile");
    Ok(())
}
