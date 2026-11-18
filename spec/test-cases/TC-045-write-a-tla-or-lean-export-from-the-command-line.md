---
id: TC-045
title: "Write a TLA+ or Lean export with fsl export"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-027
    type: verifies
---
# TC-045: Write a TLA+ or Lean export with fsl export

## Description

Verify that `fsl export` hands its request and `--output` path to FSL, writes FSL's `export` outcome document and its text, and routes and refuses `--evidence`. Scope: FR-027-AC-1 to FR-027-AC-5.

## Test Procedure

1. Write FR-756-AC-1's counter fixture (`fsl-cli/tests/fixtures/export-counter.*`) and run `fsl export <request> --output m.tla --format json`; run the request carrying `export.output` naming `m.tla` in a fresh directory.
2. Run step 1's command again with `m.tla` present; run the member request with `--output`; give `--output` twice.
3. Run with `--limit export.output_bytes=1`, then with `--limit export.output_bytes=16777216`.
4. Run step 1's command without `--format` in a fresh directory.
5. Run `fsl export --evidence e.json --format json`, `fsl export --evidence e.json --output m.tla`, `fsl export <request> --evidence e.json` and `fsl compile <request> --evidence e.json`.

Test: `fsl-cli/tests/export_cli.rs`.

## Expected Results

- Step 1: exit 0; `m.tla` equals `result.export.target_text`; operation `export`, category `success`, result kind `exported`; the two files are byte-identical.
- Step 2: `invalid-request` naming the path, exit 20, bytes unchanged; `invalid-request` naming `output` and `--output`, exit 20, no file; a refusal naming `--output`, exit 20.
- Step 3: exit 22 naming `export.output_bytes`, no file; then exit 0.
- Step 4: exactly the four FR-865 export lines; exit 0.
- Step 5: an outcome with operation `export`; two `invalid-request` refusals naming `--evidence`, exit 20; an unknown-option refusal, exit 20.
