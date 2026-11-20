// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::BTreeMap;
use std::ffi::OsString;

use fsl_cli_contracts::{Captured, CliCase, ContractError};
use serde_json::Value;

use crate::production_options as options;

mod display;
mod parser;

#[derive(Debug, Eq, PartialEq)]
struct HelpOption {
    value: Option<String>,
    repeatable: bool,
}

fn sections(text: &str) -> BTreeMap<String, BTreeMap<String, HelpOption>> {
    let mut result = BTreeMap::new();
    let mut selected = None;
    for line in text.lines() {
        if let Some(verb) = line.strip_suffix(" options:") {
            assert!(
                !verb.is_empty() && !verb.contains(char::is_whitespace),
                "invalid section: {line}"
            );
            assert!(
                result.insert(verb.to_owned(), BTreeMap::new()).is_none(),
                "duplicate section: {line}"
            );
            selected = Some(verb.to_owned());
        } else if line.is_empty() {
            selected = None;
        } else if let Some(verb) = &selected {
            assert!(line.starts_with("  --"), "invalid option line: {line}");
            let content = &line[2..];
            let repeatable = content.ends_with(" (repeatable)");
            let content = content.strip_suffix(" (repeatable)").unwrap_or(content);
            let (name, value) = match content.split_once(' ') {
                Some((name, value)) => (name, Some(value.to_owned())),
                None => (content, None),
            };
            assert!(!name.is_empty(), "empty option name");
            assert!(
                result
                    .get_mut(verb)
                    .expect("selected section")
                    .insert(name.to_owned(), HelpOption { value, repeatable })
                    .is_none(),
                "duplicate option {name} in {verb}"
            );
        }
    }
    result
}

fn successful_text(output: &Captured) -> Result<String, ContractError> {
    assert_eq!(output.0.status.code(), Some(0), "{output:?}");
    assert!(output.0.stderr.is_empty(), "{output:?}");
    let text = output.stdout_text()?.to_owned();
    assert!(
        !text.contains('\u{1b}'),
        "help/version must not contain terminal controls"
    );
    assert!(text.ends_with('\n'), "{text}");
    Ok(text)
}

fn option_value(spec: &options::OptionSpec) -> Option<String> {
    spec.value.map(|form| match form {
        "<bytes>" => "1048576".to_owned(),
        "<identity>=<file>" => "contracts/selected=unread-model.json".to_owned(),
        "name=value" if spec.name == options::PROVIDER_LIMIT => "deadline_ms=1000".to_owned(),
        "name=value" => "s1.nodes=50000".to_owned(),
        form if form.starts_with('<') => "unread-selection".to_owned(),
        form => form
            .split('|')
            .next()
            .expect("nonempty value syntax")
            .to_owned(),
    })
}

fn option_arguments(spec: &options::OptionSpec) -> Vec<OsString> {
    let mut args = vec![OsString::from(spec.name)];
    if let Some(value) = option_value(spec) {
        args.push(OsString::from(value));
    }
    args
}

fn invalid_option(output: &Captured, name: &str) -> Value {
    assert_eq!(output.0.status.code(), Some(20), "{name}: {output:?}");
    let value = output.json().expect("one invalid-option document");
    assert_eq!(value["category"], "refusal", "{name}: {value}");
    let diagnostics = value["diagnostics"].as_array().expect("diagnostics");
    assert!(
        diagnostics.iter().any(|d| d["code"] == "invalid-request"
            && d["message"].as_str().is_some_and(|m| m.contains(name))),
        "{name}: {value}"
    );
    assert!(value["last_stage"].is_null(), "{value}");
    assert!(value["result"].is_null(), "{value}");
    value
}
