// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// Trace: FR-002-AC-24
/// Provenance: QSL-2478
#[test]
fn advertised_options_are_consumed_by_the_production_parser_and_real_cli(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    for entry in options::table() {
        for spec in &entry.options {
            let args = option_arguments(spec);
            let scanned = match options::scan(entry, &args) {
                Ok(options::Scan::Options(scanned)) => scanned,
                other => panic!("{} {} not admitted: {other:?}", entry.verb, spec.name),
            };
            assert!(scanned.positionals.is_empty());
            assert_eq!(scanned.options.len(), 1);
            assert_eq!(scanned.options[0].0, spec.name);
            assert_eq!(
                scanned.options[0].1.as_deref(),
                args.get(1).map(OsString::as_os_str)
            );
            let mut actual = vec![entry.verb.to_owned()];
            actual.extend(
                args.iter()
                    .map(|arg| arg.to_str().expect("UTF8 fixture").to_owned()),
            );
            actual.push("--help".to_owned());
            let output = case.run(&actual.iter().map(String::as_str).collect::<Vec<_>>())?;
            let text = successful_text(&output)?;
            assert!(
                sections(&text)
                    .get(entry.verb)
                    .expect("verb section")
                    .contains_key(spec.name),
                "{} {}: {text}",
                entry.verb,
                spec.name
            );
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-24
/// Provenance: QSL-2478
#[test]
fn foreign_table_options_and_reserved_unknowns_refuse_even_when_help_is_requested(
) -> Result<(), Box<dyn std::error::Error>> {
    let case = CliCase::from_environment()?;
    let mut all = BTreeMap::new();
    for entry in options::table() {
        for spec in &entry.options {
            all.entry(spec.name).or_insert(spec);
        }
    }
    for entry in options::table() {
        for (&name, spec) in &all {
            if entry.option(name).is_some() {
                continue;
            }
            let mut args = vec![
                entry.verb.to_owned(),
                "--format".to_owned(),
                "json".to_owned(),
            ];
            args.extend(
                option_arguments(spec)
                    .iter()
                    .map(|arg| arg.to_str().expect("UTF8 fixture").to_owned()),
            );
            args.push("--help".to_owned());
            invalid_option(
                &case.run(&args.iter().map(String::as_str).collect::<Vec<_>>())?,
                name,
            );
        }
        for name in ["--bogus", "--color"] {
            invalid_option(
                &case.run(&[entry.verb, "--format", "json", name, "--help"])?,
                name,
            );
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2478
#[test]
fn version_flags_are_root_only_for_every_executable_verb() -> Result<(), Box<dyn std::error::Error>>
{
    let case = CliCase::from_environment()?;
    for entry in options::table() {
        for flag in options::VERSION_OPTIONS {
            invalid_option(
                &case.run(&[entry.verb, "unread-input", flag, "--format", "json"])?,
                flag,
            );
            invalid_option(
                &case.run(&[entry.verb, flag, "--format", "json", "--help"])?,
                flag,
            );
        }
    }
    Ok(())
}

/// Trace: FR-002-AC-24
/// Provenance: QSL-2478
#[test]
fn removing_an_option_changes_the_real_parser_and_help_generator_together() {
    for entry in options::table() {
        for spec in &entry.options {
            let reduced = entry.without(spec.name);
            let args = option_arguments(spec);
            let error = options::scan(&reduced, &args).expect_err("removed option must not parse");
            assert!(matches!(error, options::Refusal::Usage(_)));
            assert!(error.message().contains(spec.name), "{error:?}");
            let rendered = options::root_help(
                std::slice::from_ref(&reduced),
                &[(entry.verb, "contract fixture")],
            );
            let observed = sections(&rendered);
            assert!(
                !observed
                    .get(entry.verb)
                    .expect("remaining section")
                    .contains_key(spec.name),
                "{rendered}"
            );
            for other in &entry.options {
                if other.name != spec.name {
                    assert!(
                        observed
                            .get(entry.verb)
                            .expect("remaining section")
                            .contains_key(other.name),
                        "removing {} also removed {}",
                        spec.name,
                        other.name
                    );
                }
            }
        }
    }
}

/// Trace: FR-002-AC-25
/// Provenance: QSL-2478
#[test]
#[ignore = "QSL-2642: unsupported output-format diagnostic omits required --format context"]
fn malformed_format_does_not_get_hidden_by_a_help_request() -> Result<(), Box<dyn std::error::Error>>
{
    let case = CliCase::from_environment()?;
    let output = case.run(&["run", "--format", "bogus", "--help"])?;
    let value = invalid_option(&output, "--format");
    assert!(
        value["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["message"]
                .as_str()
                .is_some_and(|message| message.contains("bogus"))),
        "{value}"
    );
    Ok(())
}
