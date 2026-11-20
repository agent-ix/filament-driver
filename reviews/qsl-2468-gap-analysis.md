---
id: SR-1444
title: "QSL-2468 CLI helper acceptance and trace review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; QSL-2468; cli-contracts/tests/contracts.rs, cli-contracts/src/lib.rs, cli-contracts/README.md, Makefile; FR-002-AC-1, FR-002-AC-13, FR-002-AC-19, FR-025-AC-1, FR-025-AC-3, FR-025-AC-7"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/filament-driver/FR-002
  - type: references
    target: ix://agent-ix/filament-driver/FR-025
---

## Summary

Reviewed the new helper examples against the supplied ticket acceptance and the exact criteria named by their trace tags. The real subprocess cases exercise success/refusal, raw stderr, text-versus-JSON rejection, bounded monitor verdicts and file/stdin parity. One tag claims parser-refusal identity/canonical-output evidence that the tagged test does not assert.

## Verdict

**CONDITIONAL** — correct the `FR-002-AC-19` binding. This is a diff-scoped acceptance and trace review; it does not certify the whole repository or every branch of the broad tagged criteria. Final focused testing remains pending the lead's post-review gate.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | `source_files_resolve_from_private_cwd_and_keep_json_streams_separate` is tagged `FR-002-AC-19`, but its programs parse: the negative program changes an Integer function body to `true` and asserts an `ill_typed` refusal. The test never asserts a non-parsing source's diagnostic authority/identity or canonical-byte parity with `parse`, which are AC-19's obligations. Remove this unrelated tag or add the actual malformed-source identity/canonical-output assertions; retain the applicable command/output binding. | cli-contracts/tests/contracts.rs:48; cli-contracts/tests/contracts.rs:85; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:345 | wrong-requirement |

## Coverage

Examined all five new test functions and their declared trace bindings. `FR-002-AC-1` has command/output examples; `FR-002-AC-13` has a text-output check example; `FR-025-AC-3` has missing-operand and unknown-clause refusals; `FR-025-AC-1` has concrete settled holding/violation monitor behavior; `FR-025-AC-7` has actual file/stdin byte and exit parity. These are component examples, not an exhaustive discharge of each broad criterion. The new API is grounded in QSL-2468's supplied helper acceptance; no stubs or fake FSL implementation were found in the scoped change.

The deadline lifecycle defect is recorded once in SR-1443. Accepted in-lane test and Clippy results were not re-run. Whole-repository matrix/audit work was excluded by the user's instructions and dispatch scope; the binding analysis above is manual and limited to the changed tests and exact cited requirement rows.

Plan completion: not assessed
