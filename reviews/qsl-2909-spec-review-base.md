---
id: SR-202611210022442313144108527
title: "Base specification review — QSL-2909 driver text output"
type: SpecReview
analysis: base
scope: "agent-ix/filament-driver; FR-002, FR-022 and TC-036 changes"
review_set: subset
---

## Summary

Reviewed the driver output requirement and CLI acceptance test for clarity and coverage. One medium ambiguity remains in FR-022-AC-13's statement that machine-mode bytes and exit status are unchanged by text rendering.

## Verdict

**CONDITIONAL** — the separate text and JSON contracts are clear, but the final clause in FR-022-AC-13 has no defined comparison or invocation.

## Assurance Context

No applicable `AssuranceProfile` exists in this repository's specification scope.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-022 diagnostic behavior | examined | `For an item with FR-290's typed unusable-provider-counterexample diagnostic, the driver SHALL print one indented line immediately after that item's line` |
| FR-022-AC-13 | examined | The AC requires JSON item diagnostic and fixed text rendering for each reason. |
| FR-002 output behavior | examined | Analyze text mode renders the FSL diagnostic according to FR-022. |
| TC-036 step 11 | examined | JSON assertions and text phrase assertions cover five reasons and mapped replay. |
| TC-036 expected results | examined | Text output uses fixed phrase and omits provider text. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-022-AC-13 does not define what machine-mode output is compared against | spec/functional/FR-022-analyze-a-native-analyze-request.md:242 |

The clause `machine-mode bytes and exit status are unchanged by text rendering` is ambiguous: the AC does not state which two invocations are compared, and text mode necessarily uses a different output representation from JSON mode. TC-036 does not assert this clause. Specify the unchanged invariant against a named baseline or remove the clause.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-022-AC-13 removes the undefined machine-mode bytes and exit-status comparison. |
