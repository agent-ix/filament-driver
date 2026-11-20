---
id: SR-2026112002230003
title: "Spec review of filament-driver PR 196 FR-002-AC-25"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-driver; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md (AC-25 row), spec/tests.md (TC-048, TC-049 rows) at 981693bcc6a7ca920b7a0966f0573404592ee8c3; FSL FR-866 at origin/main"
review_set: subset
evaluated_revision: "981693bcc6a7ca920b7a0966f0573404592ee8c3"
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: reviews
---

## Summary

Ticket: QSL-2602, with QSL-2601, QSL-2603 and QSL-2649. The PR changes the FR-002-AC-25 row: the check help line is now `--model <identity>=<file> (repeatable)`, and a sentence cites FSL FR-866 for the operand and its refusals, as the QSL-2649 arch ruling requires. In spec/tests.md, the TC-048 and TC-049 rows move from planned to done and name the tests that exist.

## Verdict

The `--model` amendment follows the arch ruling. One medium conflict with FSL FR-866 remains in the same row.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-002-AC-25 still fixes the check help line `--library <file> (repeatable)`. FSL FR-866, which the same row now cites as owning the operand, defines `--library <identity>=<file>`, repeatable, split at the first `=`, with duplicate and unselected-identity refusals. This is the same disagreement QSL-2649 fixed for `--model`. Fix: change the line to `--library <identity>=<file> (repeatable)` and extend the FR-866 sentence to cover `--library`. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:351 |
