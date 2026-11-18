---
id: SR-17940486709564847047962
title: "Gap analysis of QSL-2190 compile limits"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; fsl-cli/src/cli.rs, fsl-cli/tests/cli_contract.rs; QSL-2190; PR #180 at 15e4dcdc1c406c18255b40c098cde59d79dc6759"
review_set: subset
evaluated_revision: "15e4dcdc1c406c18255b40c098cde59d79dc6759"
relationships:
  - {target: "ix://agent-ix/filament-driver/FR-003", type: references}
---

## Summary

Ticket: QSL-2190. Planless gap analysis scoped to the changed compile-limit behavior and its owning FR-003 requirements, with the pinned FSL FR-869 command contract checked manually. The computed `quoin matrix` on the exact PR head reports FR-003-AC-1 as `untagged` with no binders. The new compile test carries `QSL-2190`, not an FR-003 acceptance-criterion id, so the test does not discharge that matrix row. The same criterion was untagged on base `main`; this PR adds a new compile behavior test but leaves the formal evidence mapping unchanged. FR-869-AC-4 also requires compile to refuse the recognized-but-unavailable `admission.work_units` setting and list accepted settings; the added compile refusal test checks only an unknown name. The production change is covered by FR-003's caller-selected lifecycle limits requirement; no new code-to-spec gap or stub was found. The optional semantic review was skipped; no plan was supplied.

## Verdict

FAIL. FR-003-AC-1 remains untagged in the computed Test Matrix, and FR-869-AC-4's compile-specific unavailable-setting case lacks a focused assertion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The compile limit success behavior has no criterion-bound test in the computed matrix: `compile_accepts_limit_settings` is tagged with `QSL-2190`, leaving FR-003-AC-1 untagged. Bind the test to the formal criterion it verifies. | fsl-cli/tests/cli_contract.rs:693-738; spec/functional/FR-003-apply-caller-selected-limits.md:50-52 |
| FND-002 | medium | The compile limit test does not exercise FR-869-AC-4's recognized-but-unavailable `admission.work_units=5` refusal with the compile accepted-settings list. Add a compile assertion for that case. | fsl-cli/tests/cli_contract.rs:740-754 |

## Coverage

Plan completion: not assessed.

The computed matrix was run on PR head `15e4dcdc1c406c18255b40c098cde59d79dc6759` and compared with base `main`. FR-003-AC-1 is untagged on both revisions; the added test does not change its matrix status. Reverse-gap inspection found the new `compile --limit` parsing and propagation is owned by FR-003. Manual alignment against the FSL revision pinned in the driver lock found the compile unavailable-setting case from FR-869-AC-4 absent in the added tests. No stub or hollow assertion was found in the changed production/test lines. Semantic review was skipped because it was not explicitly requested.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The compile success test now carries the FR-003-AC-1 trace tag. |
| FND-002 | fixed | A compile test now asserts the unavailable admission setting and accepted settings list required by FR-869-AC-4. |

## Verdict (disposition pass 1)

PASS at `6b2a7b132e901a20b0ee34bcd7177f39ba39ddbd`. Both findings are fixed. The supplied focused test and rustfmt evidence was not rerun.
