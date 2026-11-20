use super::*;

fn fenced(heading: &str, source: &str) -> String {
    format!("### {heading}\n\n```ix:native\n{source}```\n\n")
}

fn check(case: &CliCase, file: &str, extra: &[&str]) -> Result<Captured, ContractError> {
    let mut args = vec![
        "check",
        "--markdown",
        "contracts",
        "checks",
        file,
        "--format",
        "json",
    ];
    args.extend_from_slice(extra);
    case.run(&args)
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn each_native_fence_is_checked_and_other_fence_languages_are_ignored() -> Result<(), ContractError>
{
    let case = case()?;
    let source = format!(
        "# Rule checks\n\n{}\n```text\nnot a native program {{\n```\n\n{}",
        fenced("Ready", UNIT),
        fenced("Other", &UNIT.replace("Eligible", "AnotherEligible"))
    );
    case.write("checks.md", source.as_bytes())?;
    let value = document(&check(&case, "checks.md", &[])?, 0, "success");
    let items = value["items"].as_array().expect("native fence items");
    assert_eq!(items.len(), 2);
    for (item, clause) in items.iter().zip(["Ready", "Other"]) {
        assert_eq!(item["category"], "success", "{value}");
        assert!(item["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .is_empty());
        assert_eq!(item["unit"]["clause"], clause, "{value}");
        assert_eq!(
            item["unit"]["body"]["identity"],
            format!("checks#{clause}"),
            "{value}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn an_ill_typed_fence_does_not_stop_later_fences() -> Result<(), ContractError> {
    let case = case()?;
    let broken = UNIT.replace("n >= 3", "n + 3");
    let source = format!(
        "# Checks\n\n{}{}{}",
        fenced("Before", UNIT),
        fenced("Wrong", &broken),
        fenced("After", &UNIT.replace("Eligible", "Afterwards"))
    );
    case.write("mixed.md", source.as_bytes())?;
    let value = document(&check(&case, "mixed.md", &[])?, 20, "refusal");
    let items = value["items"].as_array().expect("fence items");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["category"], "success");
    assert_eq!(items[1]["category"], "refusal");
    assert_eq!(items[2]["category"], "success");
    assert!(
        items[1]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["code"] == "ill_typed"),
        "{value}"
    );
    assert_eq!(items[1]["unit"]["clause"], "Wrong");
    assert_eq!(items[2]["unit"]["clause"], "After");
    Ok(())
}

/// Trace: FR-002-AC-1
/// Provenance: QSL-2469
#[test]
fn duplicate_clause_fences_and_missing_clause_names_are_located_refusals(
) -> Result<(), ContractError> {
    let case = case()?;
    let duplicate =
        format!("# Duplicate\n\n### Same\n\n```ix:native\n{UNIT}```\n\n```ix:native\n{UNIT}```\n");
    case.write("duplicate.md", duplicate.as_bytes())?;
    let value = document(&check(&case, "duplicate.md", &[])?, 20, "refusal");
    let items = value["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    for item in items {
        assert_eq!(item["category"], "refusal");
        let d = item["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .find(|d| d["code"] == "extraction-clause-count")
            .expect("duplicate-fence refusal");
        assert_eq!(d["details"]["actual"], 2);
    }
    case.write(
        "unowned.md",
        format!("```ix:native\n{UNIT}```\n").as_bytes(),
    )?;
    let value = document(&check(&case, "unowned.md", &[])?, 20, "refusal");
    let items = value["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert!(
        items[0]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["code"] == "missing_declaration" && d["cause"] == "missing-name"),
        "{value}"
    );
    Ok(())
}

/// Trace: FR-002-AC-1, FR-002-AC-24
/// Provenance: QSL-2469
#[test]
fn division_selection_limits_and_extensions_reach_every_native_fence() -> Result<(), ContractError>
{
    let case = case()?;
    let divided = UNIT.replace("n >= 3", "n div 3 >= 1");
    case.write(
        "division.md",
        format!(
            "# Quotients\n\n{}{}",
            fenced("First", &divided),
            fenced("Second", &divided.replace("Eligible", "SecondEligible"))
        )
        .as_bytes(),
    )?;
    case.write(
        "extensions.json",
        br#"{"format":"filament.extension-set/v1","extensions":[]}"#,
    )?;
    let value = document(
        &check(
            &case,
            "division.md",
            &[
                "--integer-division",
                "floor",
                "--limit",
                "s1.nodes=50000",
                "--extensions",
                "extensions.json",
            ],
        )?,
        0,
        "success",
    );
    let items = value["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert!(
        items.iter().all(|item| item["category"] == "success"),
        "{value}"
    );
    let value = document(&check(&case, "division.md", &[])?, 20, "refusal");
    let items = value["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert!(
        items.iter().all(|item| item["category"] == "refusal"
            && item["diagnostics"].as_array().is_some_and(|ds| ds
                .iter()
                .any(|d| d["code"] == "missing_declaration" && d["cause"] == "missing-selection"))),
        "{value}"
    );
    Ok(())
}
