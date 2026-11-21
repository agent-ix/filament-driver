---
id: SR-17940486709564847047972
title: "code-review of QSL-2279 driver integer-division law"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver PR213 QSL-2279 at af3230b9403ad314adb2f0be901352bcad31db97; src/execute.rs, fsl-cli/src/cli.rs, fsl-cli/tests/frontend_cli.rs, fsl-cli/tests/library_parity.rs, fsl-cli/tests/binary_process.rs, fsl-cli/tests/monitor_cli.rs; FSL FR-767-AC-7, driver FR-002-AC-24"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
---

## Summary

Ticket: QSL-2279. Code review with the rust-review lane folded in. The PR adds `ExecutionRequest.integer_division`, passes the `native-run/1` member from `FunctionRunInput` through `execute`, drops the old refusal of a law-bearing run request, aligns the analyze/monitor parity references on `run_engines(Vec::new())` (FND-004 of the PR 217 review) and special-cases `fsl prove --integer-division` in `dispatch`.

## Verdict

**FAIL: one high finding.** The run half is correct. An absent member is `None`, which reaches `fsl_replay::spine::run` and checks under the header law alone. No default law is substituted, as FR-767 requires ("There is no default law"). A present member reaches `run_with_profile_catalog_and_integer_division`. The run test's oracle is strong: `q(-7, 3)` gives -3 under floor, -2 under truncating, and a missing-selection refusal if the law is dropped. The check option and the header-law conflict path are unchanged on this branch, and the test asserts `invalid_package`/`conflicting-definition` exit 20. The FND-004 parity change matches production: with no provider config, the CLI's `with_started_providers` hands `run_engines(empty)`, the hybrid engine only, to `analyze_with_providers` and `monitor`. The library side now uses the same set, so it hides no real difference. The `prove` carve-out is the defect (FND-001, FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The `prove` carve-out spells the option literal `"--integer-division"` in fsl-cli production code outside the option table. This violates FR-002-AC-24 ("No option name ... appears in fsl-cli's parsing code outside the table"). It also fails the unit test `no_option_literal_appears_outside_the_table`, whose regex `"--?[A-Za-z]` matches cli.rs:2576, which comes before the first `#[cfg(test)]` at line 2846. The coder's logs never ran the fsl-cli lib tests. | fsl-cli/src/cli.rs:2573-2582, fsl-cli/src/cli.rs:3449 |
| FND-002 | medium | The prove refusal does not have the same shape as every other verb's unknown-option refusal. It returns `CommandResult::failure(Category::Refusal, ..)`, while `options::scan` refusals go through `input_refusal(operation, ..)`, an `invalid-request` outcome document naming the option, as FR-002-AC-24 and cli-contracts run_cases/options.rs:131-141 assert. It matches only the exact `--integer-division` argument: `fsl prove r.json --bogus` still answers unsupported exit 21. No cli-contracts fixture covers the new CLI-observable prove change (exit 21 to exit 20). The frontend_cli loop's oracle, `stderr.contains("unknown") \|\| stderr.contains("--integer-division")`, cannot see the shape difference. | fsl-cli/src/cli.rs:2573-2582, fsl-cli/tests/frontend_cli.rs:2046-2055 |
| FND-003 | low | `execute` builds an empty `DefinitionCatalog` only to reach the law-bearing FSL entry point. That switches law-bearing runs from `front_end::parse` to `parse_selecting_extensions`, and the 10-argument call is duplicated across the two arms. Behaviour is the same for units that select no extension, but the dummy argument is a workaround for a missing FSL API: a `run_with_integer_division` without a catalog, or an `Option` law on `run`. | src/execute.rs:140-176 |

## Detail

FND-001 failure scenario: `cargo test -p fsl-cli --lib no_option_literal_appears_outside_the_table` panics with "cli.rs:2576 spells an option outside the table". It was traced by reading the test's regex and file split against the head, and not run, under the brief's build limits. Fix: give `prove` a table entry, or route the unknown-option scan for unrouted verbs through the table, so the name comes from `options.rs`.

FND-002 failure scenario: `fsl prove r.json --integer-division floor --format json` produces a plain refusal, while `fsl run r.json --integer-division floor --format json` writes an `invalid-request` document naming `--integer-division`. Fix: use the same `input_refusal(Operation, "unknown prove option ...")` path that `options::scan` produces, decide it for every unknown prove option (not one literal), and add a cli-contracts case.

Observation outside this diff (FSL 8f71444d, not a finding of this PR): `fsl-command` `execute_prepared` maps `(None catalog, Some(law))` to plain `spine::run`, which drops the law on FSL's own command path. The driver's `execute` path does not share this, because it passes the law whenever one is present.
