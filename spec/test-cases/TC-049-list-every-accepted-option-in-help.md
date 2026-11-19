---
id: TC-049
title: "List every accepted option in root and per-verb help"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: verifies
---
# TC-049: List every accepted option in root and per-verb help

## Description

Verify that root help and each verb's help are generated from the one
`fsl-cli` option table that the verb parsers read, in both directions; that
`fsl <verb> --help` writes the verb's section of root help; that `--version`
is a root option only; and that `--color` is refused everywhere.
Scope: FR-002-AC-24, FR-002-AC-25, FR-002-AC-26.

## Test Procedure

1. Read the option table from `fsl-cli` (a test-visible accessor of the same
   table the parsers read). For each routed verb and each option `o` in its
   entry, run `fsl <verb> <operands> o <well-formed value> --format json`
   over a fixture valid for the verb and record whether the outcome refuses
   naming `o`.
2. Run `fsl --help`. Split it into verb sections. For each routed verb,
   collect every option-shaped token (a word starting with `--` or a single
   `-` and a letter) in its section, and run step 1's probe with it.
3. Check that the `check` entry holds `--integer-division` and `--markdown`,
   the `parse` and `check` entries hold `--extensions <file>`, the `run`
   entry holds `--provider-limit name=value`, the `inspect` entry holds
   `--handoff <directory>`, and the `analyze`, `monitor`, `replay` and
   `export` entries hold every option their parsers accept.
4. For each verb in the recognized verb set, from an empty working directory,
   run `fsl <verb> --help`, `fsl <verb> -h` and
   `fsl <verb> nosuch.json --help`, recording files opened.
5. For each recognized verb, run `fsl <verb> --version` and `fsl <verb> -V`.
6. For each routed verb, run it over its fixture with `--color always`, then
   `--color never`; record files opened.
7. Run every invocation of steps 1 to 6 once with `NO_COLOR=1` and once
   without, and scan stdout and stderr for the byte `0x1B`.

Test: `fsl-cli/tests/help_contract.rs` `help_lists_every_accepted_option`,
`verb_help_writes_the_root_help_section`, `color_is_refused_on_every_verb`.

## Expected Results

- Steps 1 and 2: every table option appears in its verb's section and no
  probe refuses naming the option; every token in a section is accepted.
  Fails today: root help omits `check --integer-division`,
  `check --markdown`, `run --provider-limit` and every `analyze`, `monitor`
  and `export` option.
- Step 3: each listed option is present. Fails today: neither parser accepts
  `--extensions`.
- Step 4: stdout equals the verb's section of root help, exit 0, empty
  stderr, no file opened. Fails today: `--help` after a verb falls into the
  verb's option parser.
- Step 5: `invalid-request`, exit 20, naming the option; no file read.
- Step 6: `invalid-request`, exit 20, naming `--color`, before any named
  file is read; no help section contains `--color`. Fails today: `run`
  accepts and ignores `--color`.
- Step 7: no `0x1B` byte in any stream under either setting.
