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

## Dispositions

Round 1 at `326e1fd45c6638eaa6d0fe07743105c3a2792869`. Original review text above is preserved unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in fix commits; see evidence below. |
| FND-002 | fixed | Verified in fix commits; see evidence below. |

### Round 1 evidence

FR-002-AC-27 now specifies the two unknown-option refusal cases and clean/help/version unsupported cases, and TC-049 includes AC-27. The cli-contracts case traces AC-27 and asserts exit, format, operation, category and invalid-request code/message. The library parity helper passes `Some(admitted)` for floor and truncating, compares canonical bytes, requires success and checks results -3 and -2.

Focused semantic check of the added AC found no remaining contradiction with AC-25: the unknown-option refusal precedes unsupported while clean/help/version invocations retain exit 21. This is a disposition check of the changed criterion, not a new full repository specification review. No new findings.

Round 1 original-finding assessment: resolved. Integration finding FND-003 below blocks merge pending rebase and renumbering, then focused verification. No Cargo, build or test was run by this reviewer. Plan completion: not assessed.

FND-001 after excerpt:

```rust
/// Trace: FR-002-AC-27
/// Provenance: FSL FR-767-AC-7; an unknown option on an unrouted verb refuses
/// exit 20 as an invalid-request outcome before the verb answers unsupported.
#[test]
fn unrouted_prove_refuses_an_unknown_option_before_answering_unsupported(
) -> Result<(), ContractError> {
```

FND-002 after excerpt:

```rust
let library = library_execute_with_law(
            &path,
            DIVISION_SOURCE.as_bytes(),
            "q",
            arguments.clone(),
            Some(admitted),
        );
```


## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The added prove criterion reuses FR-002-AC-27 already allocated on origin/main by PR #215 for requirements-route behavior. GitHub reports the PR conflicting. Rebase on current main and assign the prove criterion the next unused AC id, updating its trace and TC references. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:353, cli-contracts/tests/exit_cases/verbs.rs:114 |

### Integration disposition

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | still-open | Current branch and origin/main both define FR-002-AC-27 with different behavior. Requires rebase and criterion/trace renumbering before merge. |

Final round 1 verdict: **NOT MERGEABLE** until FND-003 is fixed and dispositioned, followed by green delegated focused checks on the final implementation head.
