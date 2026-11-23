---
id: TC-042
title: "Write the replay outcome document and its text with fsl replay"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-026
    type: verifies
---
# TC-042: Write the replay outcome document and its text with fsl replay

## Description

Verify that `fsl replay` writes FSL's replay outcome document in machine mode
and `render_outcome`'s text of that same document in text mode, for a replay
that runs and for a refusal. Scope: FR-026-AC-1 to FR-026-AC-4.

## Test Procedure

1. Compile FR-023's replay fixture and run `fsl replay <request> --format json`;
   compare stdout with the in-process `command::run` document.
2. Run the same request with no `--format`.
3. Run the request with `clocks` added in text and machine mode.
4. Hand a `native-run/1` request to `fsl replay` in text mode.
5. Run the replay fixture with a matching `--limit` override, then without
   it, and confirm the override reaches the exposed replay bound while other
   request values remain unchanged.

Test: `fsl-cli/tests/replay_cli.rs`.

## Expected Results

- Step 1: byte-equal canonical outcome document plus newline, `result`
  `replayed` with one `input` row, `reproduced-without-witness`; exit 0.
- Step 2: `replay: success (exit 0) at stage replay`, then
  `result 0: input reproduced-without-witness`; exit 0.
- Step 3: `result` `null` and an `invalid-request` diagnostic; text is the
  selected refusal rendering on stdout, with no stderr; exit 20.
- Step 4: refusal header, no result line, exit 20.
- Every refusal is written to stdout in the selected output format and never
  to stderr.
