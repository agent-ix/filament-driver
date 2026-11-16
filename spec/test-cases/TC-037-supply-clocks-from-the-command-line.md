---
id: TC-037
title: "Supply clock configurations with --clocks on compile and run"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-023
    type: verifies
---
# TC-037: Supply clock configurations with --clocks on compile and run

## Description

Verify that `--clocks` supplies the request's `clocks` member on `fsl run` and
`fsl compile`, and that each refusal of the flag records its code and cause
with `last_stage` null.

Scope: FR-023-AC-1 through FR-023-AC-7.

## Test Procedure

1. Write a unit with a timestamped-event clause on clock `plant-ms`, a
   request without `clocks`, the same request with the member, and a clocks
   file holding the same object, into a per-test folder.
2. Run `fsl run` with the member and with `--clocks`; compare stdout and exit
   code (AC-1). Run `fsl compile` the same way, and without clocks (AC-2).
3. Run with a missing file, an array file, a non-JSON file and a file with
   trailing content (AC-3, AC-4).
4. Run with a request that carries `clocks` and `--clocks` (AC-5).
5. Run with an unknown `kind` as a file and as the request member, and with a repeated member name (AC-6).
6. Run `fsl run --request-bytes 30` with a larger file, and `fsl compile` with a file over 1,048,576 bytes (AC-7).

## Expected Results

- `--clocks` and the request member give equal documents and exit codes.
- Each refusal exits 20 (22 for an oversize file) with its code and cause, and `last_stage` null.
