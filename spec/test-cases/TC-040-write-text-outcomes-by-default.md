---
id: TC-040
title: "Write the text outcome by default for parse, check, compile and run"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: verifies
---
# TC-040: Write the text outcome by default for parse, check, compile and run

## Description

Verify that `parse`, `check`, `compile` and `run` write the FSL FR-865 text
form of their outcome document when no `--format` is given, with the text
header naming the invoked CLI command, that the exit status equals the
machine-mode status, and that a missing request file or an unknown format
value refuses with the same code in both modes.

Scope: FR-002-AC-13, FR-002-AC-18. Tickets: QSL-1973, QSL-2870; spec gap
STD-716.

## Test Procedure

1. For each of `check <file>`, `parse <authority> <identity> <file>`,
   `compile <request>` and `run <request>`, run the `fsl` binary over a
   succeeding input and over a refused input three times: without
   `--format`, with `--format text` and with `--format json`.
2. Render the `--format json` document through FSL's `render_outcome` with
   `RenderForm::Text` and the admitted sources of the same input. Assert that
   default and `--format text` stdout have the invoked command as the header
   operation, retain the rendered header category and exit code, and equal the
   FR-865 rendering after that header operation label; assert all three
   invocations exit with the same status (AC-13).
3. Assert the refused `check` text begins with the
   `check: refusal (exit 20) at <stage>` header, holds each diagnostic's
   `<path>:<line>:<column>: <code>: <message>` line and its `  cause:` line,
   and that the succeeding `run` text ends with its `result:` line (AC-13,
   AC-18).
4. Assert no invocation in steps 1 to 3 exits 21 or writes
   `unimplemented_capability` (AC-13).
5. For a `run` request whose JSON outcome has operation `monitor`, assert
   `--format json` preserves operation `monitor`, while default and
   `--format text` begin `run: <category> (exit <code>)`; assert the remaining
   text fields match the FR-865 rendering of that same outcome and the exit
   status is unchanged (AC-13).
6. Run `fsl run nosuch.json` and `fsl compile nosuch.json` without
   `--format` and with `--format json`; assert both exit 20 with the I/O
   failure's code, and that the default stdout is the text form of that
   refusal document (AC-18).
7. Run each of the four verbs with `--format bogus` over a path that does not
   exist; assert `invalid-request`, exit 20, naming `--format` and `bogus`,
   and that the missing path is not reported, because no file is opened
   (AC-18).

Test location: `fsl-cli/tests/cli_contract.rs` (`check_default_text_matches_json_exit`,
`parse_default_text_matches_json_exit`, `compile_default_text_matches_json_exit`)
and `fsl-cli/tests/frontend_cli.rs` (`run_default_text_matches_json_exit`,
`text_outcome_names_category_items_and_causes`). The earlier test
`unavailable_text_renderers_return_typed_unsupported_refusals` asserts the
superseded refusal and is removed.

## Expected Results

- Text is the default output of the four verbs. It retains FR-865's text
  rendering except that its header operation label names the invoked CLI
  command; the corresponding JSON document is unchanged. For a `run` request
  whose JSON operation is `monitor`, text begins `run:` and JSON retains
  `monitor`.
- Exit status never depends on the output format.
- A missing request file and an unknown `--format` value refuse with exit 20
  and their own codes in text mode, never exit 21.
