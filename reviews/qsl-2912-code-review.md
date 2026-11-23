---
id: SR-203
title: "Code review — driver monitor preparation-limit specification"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; spec/functional/FR-003-apply-caller-selected-limits.md, spec/test-cases/TC-010-caller-selected-limits.md"
review_set: subset
---

## Summary

Reviewed the spec-only change for QSL-2912 and the matched FR-003 caller-limit additions. No code or test implementation is part of this PR; no code-level defect is claimed for the intentionally pending implementation.

## Verdict

**PASS** — the new contract maps the CLI override and reached-limit refusal to the named monitor preparation storage field, and the test case describes the intended checks.

## Assurance Context

No applicable AssuranceProfile exists in the specification scope; `spec/` contains no AssuranceProfile document for these paths.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-003-AC-12 | examined | `analyze`, `monitor` and `replay` expose the exact caller-limit sets assigned by FSL FR-255's “Lifecycle caller-limit coverage”; each accepted CLI operand reaches only its matching bound, and each recognized but operation-unavailable setting is refused before work. |
| FR-003-AC-13 | examined | With no override, `fsl monitor` passes 268435456 bytes as `MonitorPreparationLimits.storage` under `monitor.preparation_storage`. A real 2.9–3.2 MB monitor plan with a measured preparation read charge no greater than 126 MB completes under that default. With a lower explicit `--limit monitor.preparation_storage=<n>` that the plan exceeds, the typed refusal names `monitor.preparation_storage` and says it can be raised with `--limit monitor.preparation_storage=<n>`. |
| FR-003-AC-14 | examined | With no override, `fsl monitor` passes 16777216 as `MonitorPreparationLimits.work` under `monitor.preparation_work_units`; an explicit matching `--limit` changes only that work bound, and a reached bound names `monitor.preparation_work_units`. |
| TC-010 step 6, storage default | examined | For monitor, prepare a real tutorial-sized 2.9–3.2 MB plan whose measured read-storage charge is at most 126 MB. With no override, verify that `MonitorPreparationLimits.storage` is 268435456 bytes and the plan completes. |
| TC-010 step 6, storage override | examined | Repeat with `--limit monitor.preparation_storage=1` (or another measured lower value the plan exceeds); verify the typed refusal names `monitor.preparation_storage` and says to raise it with `--limit monitor.preparation_storage=<n>`. |
| TC-010 step 6, work limit | examined | Also set `--limit monitor.preparation_work_units=<n>` and verify only the preparation work bound changes; with no override, verify its default is 16777216 units. Repeat with a work value below the plan's measured preparation charge and verify the refusal names `monitor.preparation_work_units`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
