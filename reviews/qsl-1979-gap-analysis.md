---
id: SR-17940486709564847047968
title: "Gap analysis — fsl-cli export route"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; FR-027; repository Test Matrix"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-027
    type: references
---

## Summary

Audited the QSL-1979 export requirement-to-test-to-code slice on PR #176. The computed matrix marks every FR-027 acceptance criterion as covered only by an ignored test, leaving no runnable test evidence for the delivered route.

## Verdict

**FAIL** — FR-027-AC-1 through FR-027-AC-5 have no runnable test evidence.

## Coverage

Plan completion: not assessed.

`quoin matrix --repo /tmp/qsl-1979-pr176-review --json` reported 305 repository criteria: 106 tagged, 179 untagged, 10 tagged only by ignored tests, and 10 with no test symbol; all 305 had no run evidence. In this feature slice, FR-027-AC-1 through FR-027-AC-5 each had static status `tagged-by-ignored-test`, bound to `export_cli.rs` tests that retain QSL-2359 ignore reasons. This finding is limited to the QSL-1979 export slice; the other repository-wide gaps predate and are outside this PR's changed behavior. Semantic review was skipped (not requested).

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-027-AC-1 | examined | `fsl export <request> --output m.tla --format json` writes FSL's exported document and target file; the request's own output member writes identical bytes. |
| FR-027-AC-2 | examined | Existing targets, conflicting output destinations and repeated `--output` are refused without changing or writing a file. |
| FR-027-AC-3 | examined | The `export.output_bytes` limit bounds target text and refuses too-small limits without writing. |
| FR-027-AC-4 | examined | Default text output has the four FR-865 export lines. |
| FR-027-AC-5 | examined | `--evidence` routes to FSL evidence intake and invalid combinations refuse; other verbs reject the flag. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-027-AC-1 has no runnable test; its only bound test is ignored | fsl-cli/tests/export_cli.rs:83 |
| FND-002 | high | FR-027-AC-2 has no runnable test; its only bound test is ignored | fsl-cli/tests/export_cli.rs:122 |
| FND-003 | high | FR-027-AC-3 has no runnable test; its only bound test is ignored | fsl-cli/tests/export_cli.rs:175 |
| FND-004 | high | FR-027-AC-4 has no runnable test; its only bound test is ignored | fsl-cli/tests/export_cli.rs:212 |
| FND-005 | high | FR-027-AC-5 has no runnable test; its only bound test is ignored | fsl-cli/tests/export_cli.rs:242 |
