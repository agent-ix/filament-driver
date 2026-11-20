// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-002-AC-17
/// Provenance: QSL-2478
#[test]
fn root_help_aliases_list_executable_and_unrouted_commands_with_output_policy(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    let canonical = successful_text(&case.run(&[])?)?;
    for args in [vec!["--help"], vec!["-h"], vec!["help"]] {
        assert_eq!(successful_text(&case.run(&args)?)?, canonical);
    }
    for line in [
        "Usage: fsl <command>",
        "Commands:",
        "Source operands:",
        "Options:",
        "--format text|json|dot",
        "default: text",
        "only for inspect",
        "--version",
        "-V",
    ] {
        assert!(canonical.contains(line), "missing {line}: {canonical}");
    }
    let commands = canonical
        .split("Commands:\n")
        .nth(1)
        .expect("command section")
        .split("\nSource operands:")
        .next()
        .expect("commands");
    let advertised = commands
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next().expect("command name");
            let description = words.collect::<Vec<_>>().join(" ");
            (name.to_owned(), description)
        })
        .collect::<BTreeMap<_, _>>();
    for entry in options::table() {
        let description = advertised
            .get(entry.verb)
            .unwrap_or_else(|| panic!("missing executable {}", entry.verb));
        assert!(
            !description.starts_with("Recognized;"),
            "{}: {description}",
            entry.verb
        );
    }
    for verb in ["prove", "generate", "render"] {
        assert!(
            advertised
                .get(verb)
                .expect("recognized verb")
                .starts_with("Recognized;"),
            "{canonical}"
        );
    }
    for verb in advertised.keys() {
        assert!(
            options::entry(verb).is_some()
                || ["prove", "generate", "render"].contains(&verb.as_str()),
            "unclassified advertised command {verb}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-12
/// Provenance: QSL-2478
#[test]
fn root_version_aliases_equal_the_cli_package_version_and_have_no_lifecycle_output(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    let manifest = include_str!("../../../fsl-cli/Cargo.toml");
    let package = manifest
        .strip_prefix("[package]\n")
        .expect("explicit package manifest")
        .split("\n[")
        .next()
        .expect("package section");
    let version = package
        .lines()
        .find_map(|line| line.strip_prefix("version = "))
        .expect("declared package version");
    let version: String = serde_json::from_str(version)?;
    assert!(!version.is_empty());
    for flag in options::VERSION_OPTIONS {
        let output = case.run(&[flag])?;
        assert_eq!(successful_text(&output)?, format!("{version}\n"));
        assert!(
            output.json().is_err(),
            "version is plain text, not an outcome document"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-24, FR-002-AC-25
/// Provenance: QSL-2478
#[test]
fn every_executable_help_alias_has_exactly_its_production_option_metadata(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    let root = successful_text(&case.run(&["--help"])?)?;
    let root_sections = sections(&root);
    for entry in options::table() {
        let canonical = successful_text(&case.run(&[entry.verb, "--help"])?)?;
        assert_eq!(
            successful_text(&case.run(&[entry.verb, "-h"])?)?,
            canonical,
            "{}",
            entry.verb
        );
        assert_eq!(
            successful_text(&case.run(&[entry.verb, "unread-input", "--help"])?)?,
            canonical,
            "{}",
            entry.verb
        );
        let standalone = sections(&canonical);
        assert_eq!(standalone.len(), 1, "{canonical}");
        let observed = standalone.get(entry.verb).expect("verb option section");
        assert_eq!(
            root_sections.get(entry.verb),
            Some(observed),
            "{}",
            entry.verb
        );
        for spec in &entry.options {
            let line = observed
                .get(spec.name)
                .unwrap_or_else(|| panic!("help omits {} {}", entry.verb, spec.name));
            assert_eq!(
                line.value.as_deref(),
                spec.value,
                "{} {}",
                entry.verb,
                spec.name
            );
            assert_eq!(
                line.repeatable, spec.repeatable,
                "{} {}",
                entry.verb, spec.name
            );
        }
        for (name, observed_spec) in observed {
            let spec = entry
                .option(name)
                .unwrap_or_else(|| panic!("help advertises unaccepted {} {name}", entry.verb));
            assert_eq!(observed_spec.value.as_deref(), spec.value);
            assert_eq!(observed_spec.repeatable, spec.repeatable);
        }
        let expected_first = format!("Usage: fsl {}", entry.usage[0]);
        assert_eq!(canonical.lines().next(), Some(expected_first.as_str()));
    }
    for verb in root_sections.keys() {
        assert!(
            options::entry(verb).is_some(),
            "help invents option section {verb}"
        );
    }
    Ok(())
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2478
#[test]
fn unrouted_help_and_version_flags_never_impersonate_supported_help_or_version(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    for verb in ["prove", "generate", "render"] {
        for flag in ["--help", "-h", "--version", "-V"] {
            let output = case.run(&[verb, flag, "--format", "json"])?;
            assert_eq!(
                output.0.status.code(),
                Some(21),
                "{verb} {flag}: {output:?}"
            );
            let value = output.json()?;
            assert_eq!(value["category"], "unsupported", "{value}");
            assert_eq!(value["operation"], verb, "{value}");
            assert!(
                value["diagnostics"]
                    .as_array()
                    .expect("diagnostics")
                    .iter()
                    .any(|d| d["code"] == "unsupported_operation"
                        && d["cause"] == "operation-not-routed"
                        && d["message"].as_str().is_some_and(|m| m.contains(verb))),
                "{value}"
            );
            assert!(value["result"].is_null());
            assert!(value["last_stage"].is_null());
        }
    }
    Ok(())
}
