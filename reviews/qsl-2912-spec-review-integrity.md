---
id: SR-206
title: "Integrity review — QSL-2912 monitor preparation limits"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-driver; spec/functional/FR-003-apply-caller-selected-limits.md, spec/test-cases/TC-010-caller-selected-limits.md"
review_set: subset
---

## Summary

Reviewed the FR-003 and TC-010 additions for completeness, consistency, atomicity, and testability. The new acceptance criteria are linked to TC-010, the test-case scope includes them, the FSL dependency is already declared, and the new limit names, defaults, units, and refusal behavior are internally consistent. No integrity defect was found.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-003-AC-12 | examined | `analyze`, `monitor` and `replay` expose the exact caller-limit sets assigned by FSL FR-255's “Lifecycle caller-limit coverage”; each accepted CLI operand reaches only its matching bound, and each recognized but operation-unavailable setting is refused before work. |
| FR-003-AC-13 | examined | With no override, `fsl monitor` passes 268435456 bytes as `MonitorPreparationLimits.storage` under `monitor.preparation_storage`. A real 2.9–3.2 MB monitor plan with a measured preparation read charge no greater than 126 MB completes under that default. With a lower explicit `--limit monitor.preparation_storage=<n>` that the plan exceeds, the typed refusal names `monitor.preparation_storage` and says it can be raised with `--limit monitor.preparation_storage=<n>`. |
| FR-003-AC-14 | examined | With no override, `fsl monitor` passes 16777216 as `MonitorPreparationLimits.work` under `monitor.preparation_work_units`; an explicit matching `--limit` changes only that work bound, and a reached bound names `monitor.preparation_work_units`. |
| TC-010 expected result | examined | `analyze`, `monitor` and `replay` accept exactly their operation-specific caller limits in FSL FR-255; recognized unsupported operation limits are refused before work. |
| TC-010 expected result | examined | The monitor plan is admitted under the 256 MiB default storage allowance; a reached storage allowance names `monitor.preparation_storage` and the matching `--limit` spelling. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
