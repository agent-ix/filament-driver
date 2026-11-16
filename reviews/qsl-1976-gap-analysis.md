---
id: SR-2026111606100003
title: "Gap analysis of filament-driver PR 159 QSL-1976 acceptance criteria"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; spec/functional/FR-022-*.md, spec/test-cases/TC-036-*.md, fsl-cli/tests/frontend_cli.rs, fsl-cli/tests/cli_contract.rs, src/analyze.rs at fa6ea64c0a73d35fea2cfff60c425a1557b06ba4"
review_set: subset
evaluated_revision: "fa6ea64c0a73d35fea2cfff60c425a1557b06ba4"
relationships:
  - target: ix://agent-ix/filament-driver/FR-022
    type: reviews
---

## Summary

Ticket: QSL-1976. A manual check of each AC against its tests. The Quoin gap-analysis run was not used, because the brief limits this review to focused cargo runs. It covers the seven ticket ACs and FR-022-AC-1 to AC-8. The eight named tests exist, carry `#[trace("TC-036", "FR-022-AC-n")]`, and pass.

Ruling 6, recommendation only. The four language stubs ignored under QSL-1976 are `a_malformed_request_is_a_named_refusal`, `a_refused_program_refuses_at_its_stage`, `analyze_is_routed_and_never_refused_as_not_routed` and `a_binding_refusal_refuses_the_request`. They test FSL's command adapter for FR-624-AC-1 to AC-3 and FSpec FR-522-AC-2. The ticket's ACs are driver-side and do not name them, so they do not block Done by the letter. But their ignore reasons are now false. The driver's malformed-request test passes against the library's `outcome_document`. Leaving tests keyed to a Done ticket ignored also leaves FR-624-AC-2 and AC-3 with no running evidence anywhere. Recommendation: un-ignore all four in one small language PR as part of closing QSL-1976, and re-key any that still fails to its real ticket. They were not run, because the brief excludes the language checkout.

## Verdict

Ticket ACs: met. AC-1 holds, AC-2 refuted with replay, AC-3 malformed, AC-4 unsupported or unknown backend, AC-5 incomplete and deadline, AC-6 empty-output asserts removed, and AC-7 the registry. Cancellation is met only through the deadline; see the spec-review artifact, ruling 2. Two low test-strength gaps remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The deadline half of FR-022-AC-3 asserts only that the document text contains `deadline` somewhere. It never checks an item's record `result` `incomplete`, or a `cancelled`/`deadline` cause on the record or the diagnostic. A document that names `deadline_ms` in an echoed request member would pass. Fix: assert the item record (or, for a before-first-item settlement, the diagnostic) carries `incomplete` and the deadline cause, as FR-620-AC-6 states. | fsl-cli/tests/frontend_cli.rs:808-819 |
| FND-002 | low | FR-022 depends on FR-624, but only its malformed-request row is exercised through the binary. The check-stage refusal (FR-624-AC-2) and the binding refusal (FR-624-AC-3) have no driver test. Their text-mode `fsl analyze: <code>: <message>` line for a code other than `invalid-request` is also never seen. Fix: add one binary test for a program the checker refuses (`last_stage` `check`, empty `items`, exit 20, text line), or cover it through the language stubs of ruling 6. | fsl-cli/tests/frontend_cli.rs:902-944 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | `analyze_checker_refusal_refuses_at_check`, and its doc comment ("refuses the whole request at `check`"), name the stage `check`, while the test asserts `last_stage` `S3`. The name follows FR-624-AC-2's wording, which conflicts with FR-286-AC-6 (QSL-2288). Fix: rename the test to `..._refuses_at_s3` (or `_at_the_checker_stage`) and reword the comment. | fsl-cli/tests/frontend_cli.rs:841-845 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 608a922, the deadline case asserts `items[0].record.result` `incomplete` and `record.cause` `{"kind": "cancelled", "source": "deadline"}`. FR-022-AC-3 now names that cause. |
| FND-002 | fixed | `analyze_checker_refusal_refuses_at_check` (FR-624-AC-2: `last_stage` S3, empty `items`, a non-`invalid-request` code, and the `fsl analyze: <code>: ` text line) and `analyze_binding_refusal_refuses_the_request` (FR-624-AC-3) run the binary. Both pass. The four language stubs now also run, in filament-spec-language PR 2739. |

## Verdict (disposition pass 1)

APPROVE at 0fa4ea9. Every ticket AC and FR-022-AC-1 to AC-8 is covered by a passing tagged test (10 analyze tests). FND-003 is a naming nit.
