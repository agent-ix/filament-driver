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
`fsl-cli` option table that the verb parsers read, in both directions and
under mutation of the table; that the root-help layout keeps the sections
the docs generator reads; that `fsl <verb> --help` writes the verb's usage
lines and options section with the specified precedence; that `--version`
is a root option only; and that `--color` is refused everywhere.
Scope: FR-002-AC-24, FR-002-AC-25, FR-002-AC-26, FR-002-AC-28, FR-002-AC-29,
FR-002-AC-30, FR-002-AC-31, FR-002-AC-32.

## Test Procedure

1. Read the option table from `fsl-cli` (a test-visible accessor of the same
   table the parsers read). For each verb that executes operations and each
   option `o` in its entry, run `fsl <verb> <operands> o <well-formed value>
   --format json` over a fixture valid for the verb and record whether the
   outcome refuses naming `o`.
2. Run `fsl --help`. Check it holds the `Commands:`, `Source operands:` and
   `Options:` sections, that each verb this build does not route has a
   `Commands:` description starting `Recognized;`, and that after `Options:`
   one `<verb> options:` section per verb that executes operations follows in
   `Commands:` order. For each such section, check its lines equal the
   verb's table options in order, rendered as FR-002 states, and run step 1's
   probe on every option-shaped token in it.
3. For each verb that executes operations, take the union of all other
   verbs' entries plus `--version`, `-V`, `--color` and `--bogus`, drop the
   names in this verb's entry, and run `fsl <verb> p` with each remaining
   name `p` in text and JSON mode; check stdout and stderr.
4. Mutation probe, in the `fsl-cli` unit tests: for each verb, build a copy
   of the table with one option removed, run the verb's parser over an
   argument list using that option and run the help generator over the
   copy.
5. Scan `fsl-cli`'s parsing source for string literals that start with `-`
   outside the table definition.
6. Check that the `check` entry holds `--integer-division`, `--markdown` and
   `--limit name=value`; the `analyze`, `monitor` and `replay` entries also
   hold repeatable `--limit name=value`; check that
   the `parse` and `check` entries hold `--extensions <file>`, the `run`
   entry holds `--provider-limit name=value`, the `inspect` entry holds
   `--handoff <directory>`, and the `analyze`, `monitor`, `replay` and
   `export` entries hold every option their parsers accept.
7. From an empty working directory, run `fsl check --help`, `fsl check -h`
   and `fsl check nosuch.fsl --help`, recording files opened; then
   `fsl <verb> --help` for every other verb that executes operations.
8. Run `fsl prove --help`, `fsl generate --help`, `fsl render --help`,
   `fsl run --format bogus --help` and `fsl run --bogus --help`.
9. For each verb that executes operations, run `fsl <verb> --version` and
   `fsl <verb> -V`.
10. For each verb that executes operations, run it over its fixture with
    `--color always`, then `--color never`; record files opened.
11. Run `fsl not-a-verb`, `fsl not-a-verb --format json` and `fsl` with a
    first argument that is not valid UTF-8; check stderr holds one line,
    stdout is empty and the exit status is 20 (AC-29).
12. Run steps 1, 2, 7 and 10 once with `NO_COLOR=1` and once without, over
    fixtures that contain no `0x1B` byte, and scan stdout and stderr for the
    byte `0x1B`.
13. Run `fsl run` with the single argument byte `0xFF`, then `fsl` with that
    byte as its first argument; check the first writes the I/O refusal to
    stdout with empty stderr and exit 20, the second one stderr line (AC-30).
14. Run `fsl bogus`, `fsl chek`, `fsl check --modle x`, `fsl run --engine
    fast` and `fsl check --limit foo=1`, `--limit foo=1` on `run`, `export`, `analyze`,
    `monitor` and `replay`, `fsl help --format json` and `fsl help run extra`;
    check the exact messages of AC-31.
15. Run `fsl help <verb>` and `fsl <verb> --help` for every verb that
    executes operations, and `fsl --help`; check the bytes agree and root
    help ends with the Operands and Exit codes sections (AC-32).
    Include `fsl help prove`, `fsl help generate` and `fsl help render`.
16. Run `fsl inspect`, `fsl inspect --format dot` and `fsl inspect --handoff
    missing --format dot`; check the first two write the not-routed refusal
    ending in the `--handoff` form, exit 21, and the third refuses exit 20
    (AC-23).

Test: `cli-contracts/tests/messages.rs` for steps 8, 13, 14, 15 and 16 (AC-25, AC-30..AC-32, AC-23).

Test: `fsl-cli/tests/help_contract.rs` `help_lists_every_accepted_option`,
`parser_refuses_options_outside_its_entry`, `verb_help_writes_its_section`,
`help_precedence`, `color_is_refused_on_every_verb`; `fsl-cli/src/cli.rs`
unit test `removing_a_table_option_removes_it_from_parser_and_help`.

## Expected Results

- Steps 1 and 2: the layout is as stated; every table option appears in its
  verb's section and no probe refuses naming it. Fails today: root help has
  no per-verb sections and omits `check --integer-division`,
  `check --markdown`, `run --provider-limit` and every `analyze`, `monitor`
  and `export` option.
- Step 3: each `p` refuses `invalid-request`, exit 20, naming `p`; in both
  modes the refusal outcome is written to stdout in the selected format and
  stderr is empty. Fails today: `run` accepts `--color`.
- Step 4: the parser refuses the removed option, exit 20 naming it, and the
  generated help omits it.
- Step 5: no such literal.
- Step 6: each listed option is present. Fails today: neither parser accepts
  `--extensions`.
- Step 7: the three `check` runs write exactly the text FR-002-AC-25 gives,
  exit 0, empty stderr, no file opened; each other verb writes its usage
  lines, a blank line and its section byte-equal to root help's, exit 0.
  Fails today: `--help` after a verb falls into the verb's option parser.
- Step 8: each of the three unrouted verbs writes its not-available help
  (first line `fsl <verb> is not available in this build; running it exits 21
  (unsupported_operation).`), exit 0, no stderr; `--format bogus` refuses `invalid-request`,
  exit 20, naming `--format` and `bogus`; `--bogus` refuses
  `invalid-request`, exit 20, naming `--bogus`.
- Step 9: `invalid-request`, exit 20, naming the option; no file read.
- Step 10: `invalid-request`, exit 20, naming `--color`, before any named
  file is read; no help section contains `--color`.
- Step 11: no `0x1B` byte in any stream under either setting.
- Step 13: the argument form writes the text I/O refusal to stdout, exit 20,
  nothing to stderr; the first-argument form writes one stderr line, exit 20.
- Step 14: every message equals the text AC-31 gives; `help --format json`
  suggests no verb; no `--limit` list names a setting the verb does not accept.
- Step 15: `help <verb>` and `<verb> --help` agree byte for byte, exit status
  included, for all thirteen verbs; root help ends with the two sections.
- Step 16: the not-routed refusals end ``; use `fsl inspect --handoff
  <directory>` ``, exit 21; the dot-with-handoff form refuses, exit 20.
