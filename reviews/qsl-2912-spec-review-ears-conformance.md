---
id: SR-205
title: "EARS review — QSL-2912 monitor preparation limits"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/filament-driver; spec/functional/FR-003-apply-caller-selected-limits.md"
review_set: subset
---

## Summary

Reviewed the new FR-003 requirement statements against the EARS grammar and semantic checks. The changed statements identify the driver operation and provide observable defaults, override operands, and refusal behavior; no EARS defect was found. The full-spec grammar check reported two pre-existing warnings in FR-019, not in the changed FR-003 statements.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-003 monitor caller limits | examined | For `analyze`, `monitor` and `replay`, the operation's caller-configurable bound set SHALL be the operation-specific set in FSL FR-255's “Lifecycle caller-limit coverage”. The driver SHALL pass each accepted override to that matching FSL bound and SHALL refuse a recognized setting absent from the operation's set before doing work. In particular, `monitor` SHALL expose `monitor.preparation_work_units` and `monitor.preparation_storage` for plan preparation; the latter configures `MonitorPreparationLimits.storage`. |
| FR-003 preparation-work default | examined | The `fsl monitor` default for `monitor.preparation_work_units` SHALL be 16777216 work units, and a caller MAY replace it with `--limit monitor.preparation_work_units=<n>`. A reached work bound SHALL name `monitor.preparation_work_units` in its refusal. |
| FR-003 preparation-storage default | examined | For real inputs, `fsl monitor` SHALL use a default read-storage allowance of 268435456 bytes (256 MiB) for `monitor.preparation_storage`. This fixed default is derived from the measured upper charge of 126 MB for a 2.9–3.2 MB plan (IR-1584), with approximately 2.1 times headroom, and gives the same allowance independent of package size. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
