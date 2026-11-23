---
id: SR-204
title: "Base specification review — QSL-2912 monitor preparation limits"
type: SpecReview
analysis: base
scope: "agent-ix/filament-driver; spec/functional/FR-003-apply-caller-selected-limits.md, spec/test-cases/TC-010-caller-selected-limits.md"
review_set: subset
---

## Summary

Reviewed the new monitor preparation-limit requirements and TC-010 coverage. Two medium testability and decision-grounding gaps remain: the measured plan is not identified, and the new preparation-work default has no basis in the changed requirements.

## Verdict

**CONDITIONAL** — the storage decision is concrete and caller-configurable, but the two new requirements need reproducible evidence and a grounded work default before implementation.

## Assurance Context

No applicable AssuranceProfile exists in the specification scope; `spec/` contains no AssuranceProfile document for these paths.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-003-AC-12 | examined | `analyze`, `monitor` and `replay` expose the exact caller-limit sets assigned by FSL FR-255's “Lifecycle caller-limit coverage”; each accepted CLI operand reaches only its matching bound, and each recognized but operation-unavailable setting is refused before work. |
| FR-003-AC-13 | examined | With no override, `fsl monitor` passes 268435456 bytes as `MonitorPreparationLimits.storage` under `monitor.preparation_storage`. A real 2.9–3.2 MB monitor plan with a measured preparation read charge no greater than 126 MB completes under that default. With a lower explicit `--limit monitor.preparation_storage=<n>` that the plan exceeds, the typed refusal names `monitor.preparation_storage` and says it can be raised with `--limit monitor.preparation_storage=<n>`. |
| FR-003-AC-14 | examined | With no override, `fsl monitor` passes 16777216 as `MonitorPreparationLimits.work` under `monitor.preparation_work_units`; an explicit matching `--limit` changes only that work bound, and a reached bound names `monitor.preparation_work_units`. |
| TC-010 step 6, storage default | examined | For monitor, prepare a real tutorial-sized 2.9–3.2 MB plan whose measured read-storage charge is at most 126 MB. With no override, verify that `MonitorPreparationLimits.storage` is 268435456 bytes and the plan completes. |
| TC-010 step 6, work default | examined | Also set `--limit monitor.preparation_work_units=<n>` and verify only the preparation work bound changes; with no override, verify its default is 16777216 units. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The real-input storage criterion does not identify a reproducible tutorial plan or its measured byte charge, so independent runs can select different inputs. | spec/functional/FR-003-apply-caller-selected-limits.md:128; spec/test-cases/TC-010-caller-selected-limits.md:67 |
| FND-002 | medium | The new 16777216-unit preparation-work default is an ungrounded product-limit decision; the criterion checks the chosen value but provides no evidence that real monitor inputs fit it. | spec/functional/FR-003-apply-caller-selected-limits.md:93; spec/test-cases/TC-010-caller-selected-limits.md:74 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-010 now blocks the storage acceptance cases until the owner supplies a reproducible plan path and exact measured charge, and forbids substituting an unmeasured plan. |
| FND-002 | fixed | The unsupported 16777216-unit default was removed; the requirement names the caller setting and preserves the operation's existing default. |
