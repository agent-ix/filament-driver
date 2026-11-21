---
id: SR-17940486709564847047971
title: "gap-analysis of QSL-2577 requirement-run caller limits"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver PR215 QSL-2577 at fb3984bace429f7e873227f3f50563894584dd57; FR-002-AC-27, TC-046, fsl-cli/src/cli.rs, fsl-cli/tests/cli_contract.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
---

## Summary

Ticket: QSL-2577. Planless gap analysis scoped to the PR diff. `quire matrix --scope . --format json` reports FR-002-AC-27 `tagged`, bound to `run_routes_requirement_items_with_caller_spine_limits` (fsl-cli/tests/cli_contract.rs:2864). `quire validate --scope . "spec/**/*.md"` exits 0 (pre-existing EARS warnings on FR-019 only). Reverse gap: the new code (`has_call_requirements`, `native_run_request`, `run_with_requirement_items`) is owned by FR-002-AC-27 and FSL FR-961; no stubs. Semantic review not run (not opted in). Repository-wide untagged criteria outside this diff (for example FR-002-AC-20..22) are pre-existing and not assessed here.

## Verdict

**CONDITIONAL** — AC-27 is tagged by a real subprocess test whose exit-22 and `items == []` assertions fail when the route drops spine limits, and whose 3-item default assertion fails when the route is not taken. The criterion has one gap: the refusal of accounting settings on the requirements route has no owning criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No criterion owns the requirements route's refusal of `accounting.*` settings, which contradicts FR-003-AC-10 for calls carrying `requirements`; behaviour is unspecified and untested | fsl-cli/src/cli.rs:1086-1092 |
| FND-002 | low | AC-27's first sentence compares `fsl run Q --format json` with "the run without `--limit`", which is the same command, so the clause cannot fail | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:353 |

## Coverage

- FR-002-AC-27: tagged, 1 binder, method Test (TC-046 step 9).
- TC-046: Test line now names the AC-27 test only; steps 1-8 have no named test, and the previous name did not exist in the tree.
- Plan completion: not assessed
