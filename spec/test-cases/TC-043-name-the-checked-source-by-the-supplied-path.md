---
id: TC-043
title: "Name the checked source by the supplied path"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: verifies
---
# TC-043: Name the checked source by the supplied path

## Description

Verify that `fsl check` names a source it refuses by authority `filament` and
the path as given, and that the binary's refusal equals the library's.
Scope: FR-002-AC-19.

## Test Procedure

1. Write a source that does not parse at path `P`; run `fsl check P --format json`.
2. Parse the same bytes through FSL's `parse` under `{"authority": "filament", "identity": P}` and encode the check document of that refusal canonically.
3. Run `fsl compile` and `fsl run` over requests naming the same source, beside their library operations.

## Expected Results

- Step 1: a `check` refusal whose every diagnostic locus names `{"authority": "filament", "identity": P}`; no `fsl-command` or `check-source` in it.
- Step 2: step 1's stdout equals that encoding plus a newline; the exit statuses are equal.
- Step 3: each binary refusal equals its library refusal.

Test: `fsl-cli/tests/library_parity.rs`.
