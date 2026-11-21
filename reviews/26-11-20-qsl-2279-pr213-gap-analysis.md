---
id: SR-17940486709564847047973
title: "gap-analysis of QSL-2279 driver integer-division law"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver PR213 QSL-2279 at af3230b9403ad314adb2f0be901352bcad31db97; diff origin/main...HEAD; FSL FR-767-AC-7 (TC-2189), driver FR-002-AC-24, FR-002-AC-25"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
---

## Summary

Ticket: QSL-2279. A planless gap analysis scoped to the PR diff. `quire matrix --scope <repo> --format json` (quire 0.52.0, engine 0.62.2) ran over the driver spec. FR-767-AC-7 is a filament-spec-language criterion, so the driver matrix cannot bind it. Its driver tests carry `#[trace("TC-2189", "FR-767-AC-7")]`, as TC-2189 directs. Both are present: `check_takes_the_integer_division_law_from_its_option` and `run_takes_the_integer_division_law_from_its_request`, with the latter's `#[ignore]` removed. Each AC-7 clause has an assertion in them. Repo-wide unbacked driver criteria (190 of 315) predate this PR and are out of this diff's scope.

## Verdict

**CONDITIONAL.** Every clause of AC-7 is tested. The reverse gap: the `prove` carve-out is behaviour that no driver requirement states, and it contradicts the stated unrouted-prove contract. One parity gap is low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `prove --integer-division` refusal is unspecified in the driver spec. FR-002-AC-25 and FSpec FR-522 state that unrouted `prove` answers `unsupported`/`operation-not-routed` exit 21, and FR-767-AC-7 requires exit 20 as an unknown option. The conflict is settled in code by a single-option special case, with no FR-002 criterion and no cli-contracts case stating which unknown prove options refuse and in what shape. | fsl-cli/src/cli.rs:2573-2582, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md |
| FND-002 | low | No library-side test passes `Some(law)` to `filament_driver::execute`. Every parity reference sets `integer_division: None`, or uses a fixture without the member. Only the binary test exercises a supplied law, so a library-only regression would be caught only through the CLI. | fsl-cli/tests/library_parity.rs:163, fsl-cli/tests/binary_process.rs:518, src/execute.rs:250, src/execute.rs:450 |

## Coverage

- Plan completion: not assessed
- FR-767-AC-7 clauses: check floor exit 0; no option missing-selection; ceiling unsupported-selection before read; option twice usage; header conflict conflicting-definition; run member -3; unknown option on run, compile, analyze, prove and monitor. Each has an assertion in frontend_cli.rs:1946-2096.
- Stubs and coverage inflation: none found in the diff.
- Semantic review: skipped (not requested).
