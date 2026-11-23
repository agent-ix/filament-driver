---
id: TC-038
title: "Write the checked package with fsl compile --output"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-024
    type: verifies
---
# TC-038: Write the checked package with fsl compile --output

## Description

Verify that `--output` makes `fsl compile` write the library's checked-package
bytes to a new file, and that each refusal of the flag records its code with
`last_stage` null and writes no file.

Scope: FR-024-AC-1 through FR-024-AC-6. This is the driver half of FSL
FR-810-AC-4 (FSL TC-2297 step 4).

## Test Procedure

1. Write a unit, a request without `output` and the same request with an
   `output` member into a per-test folder.
2. Run `fsl compile --output cli.package.json` in that folder; compare the file
   with the member's file and with `command::compile`, and the two outcome
   documents (AC-1).
3. Run with an existing target (AC-2), then with the member request and
   `--output` (AC-3).
4. Run `fsl run`, `fsl check` and `fsl analyze` with `--output` in text and
   JSON mode (AC-4).
5. Run `fsl compile` with `--output` twice and with no path, with no format
   and with `--format json` (AC-5).
6. Run `fsl compile --output` over a unit the checker refuses (AC-6).

## Expected Results

- The `--output` file, the member's file and the library bytes are equal.
- Each refusal exits 20 with its code and writes no file; an existing target
  keeps its bytes. Every refusal outcome is written to stdout in the selected
  format and nothing is written to stderr.
