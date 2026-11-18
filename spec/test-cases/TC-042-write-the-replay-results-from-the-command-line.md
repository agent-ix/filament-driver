---
id: TC-042
title: "Write the replay results and its text outcome with fsl replay"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-026
    type: verifies
---
# TC-042: Write the replay results and its text outcome with fsl replay

## Description

Verify that `fsl replay` writes FSL's `results` document in machine mode and
the FR-865 outcome text plus one result line per row in text mode, and writes
a refusal's outcome in both modes. Scope: FR-026-AC-1 to FR-026-AC-4.

## Test Procedure

1. Compile FR-023's replay fixture and run `fsl replay <request> --format json`;
   compare stdout with the in-process `command::run` document.
2. Run the same request with no `--format`.
3. Run the request with `clocks` added in text and machine mode.
4. Hand a `native-run/1` request to `fsl replay` in text mode.

Test: `fsl-cli/tests/replay_cli.rs`.

## Expected Results

- Step 1: byte-equal canonical document plus newline; one `input` row,
  `reproduced-without-witness`; exit 0.
- Step 2: `replay: success (exit 0) at S8`, then
  `result 0: input reproduced-without-witness`; exit 0.
- Step 3: refusal header and `invalid-request` diagnostic, no result line; no
  `results` member; exit 20.
- Step 4: refusal header, no result line, exit 20.
