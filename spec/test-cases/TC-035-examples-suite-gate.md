---
id: TC-035
title: "Run the examples suite against the workspace-built fsl"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-021
    type: verifies
---
# TC-035: Run the examples suite against the workspace-built fsl

## Description

Verify that the driver gate builds `fsl` from this workspace, runs every
running and staged example case with it, and fails when any case changes or
the suite is missing.

Scope: FR-021-AC-1 through FR-021-AC-6.

## Test Procedure

1. `make examples-gate` fetches filament-examples `main` and runs
   `tests/examples_gate.rs` with `FSL_EXAMPLES_DIR` set.
2. Run `check` and `staged` over the whole suite with the built `fsl`; assert
   exit 0 and a summary of N passed, 0 failed with N above zero (AC-1, AC-2).
3. Copy the first system with a running case into a per-test folder under
   `CARGO_TARGET_TMPDIR`, edit that case's recorded `category`, and run
   `check` on the copy; assert exit 1 and a failure line naming the case
   (AC-3).
4. Copy the first system with a staged case, swap the case's `expect_exit` and
   `ready_exit`, and run `staged` on the copy; assert exit 1 and the promote
   message for the case (AC-4).
5. Check that the manifest reader used by steps 3 and 4 refuses an unexpected
   table, an inline comment on a read key, a repeated key, a line that is not
   `key = value`, a case without id or status, an absent case and a repeated
   case id, so an edit never lands on a misread line (AC-3, AC-4).
6. Resolve the checkout with the variable unset and with an absent folder,
   and run the runner on an empty root (AC-5).
7. Compare `fsl --version` with the `fsl-cli` version from `cargo metadata`
   and check that the binary lies under the workspace target directory; inspect
   the `ci` target (AC-6).

## Expected Results

- The whole suite passes on the workspace-built `fsl`.
- An edited running record or a staged case that starts working fails the
  gate, naming the case.
- A missing suite is an error, never a skip.
