---
id: SR-17940486709564847047964
title: "Gap analysis of QSL-2019 Markdown CLI slice"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; fsl-cli/src/cli.rs, fsl-cli/tests/cli_contract.rs, fsl-cli/Cargo.toml, Cargo.lock; QSL-2019; PR #181 at 56fe07ed4f9dc7b15d99eb9d7b7534af1b449e18"
review_set: subset
evaluated_revision: "56fe07ed4f9dc7b15d99eb9d7b7534af1b449e18"
relationships:
  - {target: "ix://agent-ix/filament-driver/FR-002", type: references}
---

## Summary

Ticket: QSL-2019. Planless gap analysis scoped to the changed Markdown CLI slice and its driver outcome contract. The added `check --markdown` tests exercise the upstream FSL FR-662 paths; the run-extraction acceptance test is present but cannot pass with locked FSL revision `6f5d277`, which rejects `program.clauses`. That is a blocked cross-repository criterion, not evidence of a driver implementation gap.

## Verdict

CONDITIONAL. The new check regression does not assert FR-662-AC-1's exact `ill_typed` diagnostic code, leaving that part of the CLI evidence open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `check_markdown_checks_each_named_native_fence` verifies only that the refusal code is nonempty; it does not verify the required `ill_typed` code at the CLI boundary. Assert the exact code so a changed or rewritten code fails this regression. | fsl-cli/tests/cli_contract.rs:274-279; FR-662-AC-1 |

## Coverage

Plan completion: not assessed.

The computed `quoin matrix --repo . --json` ran on exact head `56fe07ed4f9dc7b15d99eb9d7b7534af1b449e18`; it reports 26 driver requirements and contains no local FR-662 or FSpec FR-563 criterion. Those trace ids belong to upstream repositories, so this driver matrix cannot discharge or classify them. Manual alignment against FSL FR-662-AC-1 and AC-2 found the CLI tests cover healthy/refused fence identities, refusal categories, spans and original coordinates, duplicate/headless refusal details, and deterministic output. The exact error code remains weakly asserted as recorded above. The default-build extracted-run test covers the driver invocation but is blocked by locked FSL's `program.clauses` decoder; it remains In Progress under QSL-2019 and is not counted as a defect in this driver diff. No new stub or additional unowned behavior was found in the changed slice. Semantic review was skipped; no plan was supplied. The supplied focused check test passed (2 tests); no broader tests were run.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The CLI regression now asserts the exact `ill_typed` code for the Broken native fence. |

## Verdict (disposition pass 1)

PASS at `697f66b2ecec89f6b0930954ded1b32db623ee1d`. The exact-code coverage gap is fixed. The extracted-run test remains blocked by locked FSL and is tracked separately, not as a driver gap. The coder reports three focused CLI tests and changed-file rustfmt passing; I did not rerun them.

## Verdict (disposition pass 2)

PASS at `14d2519b22bef22c6206a4eba102011d83f5d4b1`. The exact-code assertion remains present after the rebase and lockfile refresh; no new gap was found in the updated PR diff. The coder reports three focused CLI tests and changed-file rustfmt passing; I did not rerun them. The extracted-run criterion remains blocked pending FSL #2891 and is tracked separately.
