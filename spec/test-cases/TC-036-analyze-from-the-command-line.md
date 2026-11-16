---
id: TC-036
title: "Analyze a native-analyze/1 request with the fsl binary"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-022
    type: verifies
---
# TC-036: Analyze a native-analyze/1 request with the fsl binary

## Description

Verify that `fsl analyze` reads a `native-analyze/1` request, settles each
item through the driver's negotiation, and writes FSL's `analyze` outcome
with its exit code, for each outcome a user meets: proved, refuted with a
replayable counterexample, incomplete under a limit or a deadline,
unsupported or refused per item, and a refused request.

Scope: FR-022-AC-1 through FR-022-AC-8.

## Test Procedure

1. Write the Counter request (`fsl-cli/tests/fixtures/analyze-counter.json`
   and the model, unit and snapshot beside it) into a per-test folder with
   the items each case needs.
2. Run `fsl analyze <request> --format json` and assert the category, the
   record `result`, the `aggregate` status and the exit code for the
   `Bounded` item (AC-1).
3. Run the `BelowThree` item and assert `refuted`, the counterexample entry,
   the replay target and `reproduced`, and exit 10 (AC-2).
4. Run `Bounded` with `temporal.limits.max_states` 1, and with
   `deadline_ms` 0; assert `incomplete`, the cause, no counterexample and
   exit 22 (AC-3).
5. Run an item naming `no-such-backend`, and an item naming backend `en-6`,
   which serves no temporal-satisfaction form; assert the per-item
   settlement and diagnostic, and the `registry` member (AC-4, AC-8).
6. Run a malformed request in machine and text mode, a program the checker
   refuses, a snapshot the subject binding refuses, and the verb with no
   operand, an unknown option and `--provider-config` (AC-5).
7. Call `fsl_command::command::analyze` in process with
   `filament_driver::AnalyzeNegotiation` for the AC-2 request and compare
   its canonical encoding with the binary's stdout (AC-6).
8. Run the AC-2 request and an unknown-backend item in text mode and assert
   their lines and exit codes (AC-7).
9. Unit-test the negotiation over each candidate-outcome shape (AC-8).

## Expected Results

- Every outcome is one document or one line, never empty output, with the
  FR-285 exit code of its category.
- The binary's machine-mode document equals the library's.
