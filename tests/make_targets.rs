// SPDX-License-Identifier: AGPL-3.0-or-later
//! The local gate targets of the Makefile: `make ci` runs the examples gate.

use ix_trace_rs::trace;

const MAKEFILE: &str = include_str!("../Makefile");

/// The prerequisites of the rule `target:` in the Makefile.
fn prerequisites(target: &str) -> Vec<&'static str> {
    let head = format!("{target}:");
    let mut rules = MAKEFILE
        .lines()
        .filter_map(|line| line.strip_prefix(head.as_str()));
    let rule = rules
        .next()
        .unwrap_or_else(|| panic!("the Makefile has no `{target}:` rule"));
    assert!(
        rules.next().is_none(),
        "the Makefile has two `{target}:` rules"
    );
    rule.split_whitespace().collect()
}

/// The recipe lines of the rule `target:`.
fn recipe(target: &str) -> Vec<&'static str> {
    let head = format!("{target}:");
    MAKEFILE
        .lines()
        .skip_while(|line| !line.starts_with(head.as_str()))
        .skip(1)
        .take_while(|line| line.starts_with('\t'))
        .collect()
}

#[trace("FR-021-AC-6")]
#[test]
fn ci_includes_examples_check() {
    assert!(
        prerequisites("ci").contains(&"examples-check"),
        "`make ci` does not run examples-check: {:?}",
        prerequisites("ci")
    );
    assert_eq!(prerequisites("examples-check"), ["examples-fetch"]);
    let recipe = recipe("examples-check");
    assert!(
        recipe
            .iter()
            .any(|line| line.contains("FSL_EXAMPLES_DIR=") && line.contains("--test examples_gate")),
        "examples-check does not run the examples gate: {recipe:?}"
    );
}
