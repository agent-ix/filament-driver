---
id: TC-048
title: "Verify a checked-package handoff with fsl inspect --handoff"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: verifies
---
# TC-048: Verify a checked-package handoff with fsl inspect --handoff

## Description

Verify that `fsl inspect --handoff <directory>` calls FSL's
`fsl_command::command::inspect_handoff`, writes the document it returns with
that document's exit result in JSON and text mode, refuses malformed
invocations before reading the directory, and leaves every other `inspect`
input not routed. Scope: FR-002-AC-23.

## Test Procedure

1. In a per-test folder, write FSL FR-665-AC-1's `native-compile/1` request
   for the spine fixture with the `handoff` member
   `{"directory": "handoff", "package_name": "spine", "native_join": {"work_units": 1000000, "storage": 1000000}}`
   and run `fsl compile <request> --format json`; call the written directory
   `H`.
2. Run `fsl inspect --handoff H --format json`; compare stdout with the
   canonical encoding of `inspect_handoff(H)` plus a newline.
3. Run `fsl inspect --handoff H` without `--format`; compare stdout with the
   FSL FR-865 text form of the same document.
4. Copy `H` to `H1` and raise one node count in `H1/selection.json` by one;
   copy `H` to `H2` and change one byte of `H2/checked-package.json`. Run
   `fsl inspect --handoff <copy> --format json` over each and compare with
   `inspect_handoff(<copy>)`.
5. Run `fsl inspect --handoff <path> --format json` with `<path>` a path that
   does not exist, an empty directory, and a regular file; compare each with
   `inspect_handoff(<path>)`.
6. Make `H` unreadable to the test's file probe (record every file the
   process opens, or remove read permission on `H` on Unix) and run
   `fsl inspect --handoff`, `fsl inspect --handoff H extra`,
   `fsl inspect --handoff H --handoff H` and
   `fsl inspect --handoff H --format dot`, each with `--format json` except
   the last.
7. Run `fsl inspect H --format json` without `--handoff`.

Test: `fsl-cli/tests/cli_contract.rs` `inspect_handoff_verifies_the_package_and_sidecar`.

## Expected Results

- Step 2: operation `inspect`, category success, byte-equal to FSL's
  document, exit 0. Fails today: `inspect` is not routed and exits 21.
- Step 3: the text form of the same document, exit 0.
- Step 4: `H1` refuses with the `SelectionRefusal` naming that count's axis,
  `H2` with the strict reader's typed refusal; each byte-equal to FSL's
  document, exit 20.
- Step 5: each byte-equal to `inspect_handoff`'s refusal for that path,
  exit 20.
- Step 6: each refuses `invalid-request`, operation `inspect`, exit 20,
  naming the missing operand, `extra`, `--handoff`, and `--format` with
  `dot` respectively; no run reads a file under `H`.
- Step 7: `unsupported` with cause `operation-not-routed` naming `inspect`,
  exit 21.
