---
id: SR-2026112002230002
title: "Gap analysis of filament-driver PR 196 driver option table"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; FR-002-AC-23..26 against fsl-cli/tests/help_contract.rs, cli_contract.rs, frontend_cli.rs and fsl-cli/src at 981693bcc6a7ca920b7a0966f0573404592ee8c3; QSL-2649 done-when; FSpec FR-562-AC-1/2; FSL FR-866-AC-5"
review_set: subset
evaluated_revision: "981693bcc6a7ca920b7a0966f0573404592ee8c3"
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: reviews
---

## Summary

Ticket: QSL-2602, with QSL-2601, QSL-2603 and QSL-2649. This is a manual mapping of acceptance criteria to tests over the diff; `quoin matrix` was not run, and plan completion was not assessed.

- FR-002-AC-23 (TC-048) maps to `inspect_handoff_verifies_the_package_and_sidecar`. It covers JSON and text byte equality, a raised node count, a changed byte, a missing path, an empty directory, a regular file and `./-odd`, the malformed forms with the directory made unreadable, `--format dot`, `--bogus` with and without `--handoff`, and not-routed without `--handoff`. Complete.
- FR-002-AC-24 maps to `help_lists_every_accepted_option`, `parser_refuses_options_outside_its_entry`, and the unit tests `removing_a_table_option_removes_it_from_parser_and_help` and `no_option_literal_appears_outside_the_table` (scope gap: SR-2026112002230001 FND-003).
- FR-002-AC-25 maps to `verb_help_writes_its_section` and `help_precedence`.
- FR-002-AC-26 maps to `color_is_refused_on_every_verb`.
- FSL FR-866-AC-5: the binary refusals of a bare file, `=file` and `id=` are in `check_accepts_the_model_operand_the_missing_import_message_names`. A duplicate identity is covered only by the unit-level parse test.

## Verdict

The criteria are covered except for two medium gaps: the two-model binary case that QSL-2649 requires, and part of FR-562-AC-2.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | QSL-2649's done-when asks for a binary test that checks a unit selecting two models with two `--model identity=file` operands. Only the unit-level parse test has two operands, so no test shows that the driver hands both selections to `check_outcome`. Fix: add a cli_contract test with a two-`model` unit and two model fixtures, expecting exit 0, plus each single-model run refusing `missing_import` for the other identity. | fsl-cli/tests/cli_contract.rs:2659 |
| FND-002 | medium | FSpec FR-562-AC-2 is only partly tested. Untested: a definition document that violates FR-467's `missing-member` rule, and a unit that selects no extension giving the same result with and without `--extensions`. FR-562-AC-1's form `check --extensions set.json unit.native` (one operand) is also untested; the test uses the three-operand form. Fix: add these three cases to `malformed_extension_sets_refuse_before_any_stage` and `extension_sets_supply_the_selected_definition_to_check_and_parse`. | fsl-cli/tests/frontend_cli.rs:2327-2412 |
| FND-003 | low | FSL FR-866-AC-5's duplicate-identity case (`--model example/x=m.json` given twice, exit 20, before reading `m.json`) is checked only by the parse unit test, not against the binary. Fix: add the duplicate to the `bad` loop in `check_accepts_the_model_operand_the_missing_import_message_names`, with a model path that does not exist. | fsl-cli/tests/cli_contract.rs:2697 |
