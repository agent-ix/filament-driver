---
id: TC-039
title: "Stream observations with the fsl monitor verb"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-025
    type: verifies
---
# TC-039: Stream observations with the fsl monitor verb

## Description

Verify that `fsl monitor` writes FSL's events as records arrive, then the
summary, exits with the summary's code, and refuses a bad request, a bad
option and a bad record as FR-025 states.

Scope: FR-025-AC-1 through FR-025-AC-7. Ticket: QSL-2313, the driver half of QSL-1977.

## Test Procedure

1. Write the unit and a `native-monitor/1` request over `Reaches` and `Never`
   into a per-test folder (`fsl-cli/tests/monitor_cli.rs`).
2. Pipe levels 0, 1, 2, 1 and `{"end": true}` to `fsl monitor --format json`;
   assert the lines and exit 10; repeat for `Never` alone and exit 0 (AC-1).
3. Call `fsl_command::command::monitor` in process over the same records and
   compare its canonical lines with the binary's stdout (AC-2).
4. Run an unknown-clause request, a non-JSON request and no operand (AC-3).
5. Run with `--provider-config` (AC-4).
6. Insert a record with an unknown member and compare with the library (AC-5).
7. Keep stdin open, send levels 0 and 2 and read the settling event before
   closing stdin (AC-6).
8. Run with `--input` and with stdin, then with a missing `--input` file (AC-7).

## Expected Results

- The binary's lines equal the library's; each refusal exits 20; events arrive
  before the stream ends.
