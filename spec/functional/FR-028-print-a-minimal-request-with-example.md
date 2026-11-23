---
id: FR-028
title: "Print a minimal valid request with --example and make request refusals name every defect"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-driver/FR-024
    type: references
  - target: ix://agent-ix/filament-driver/FR-027
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-100
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-620
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-644
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-862
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-1015
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-1022
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-027
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-287
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-1047
    type: depends_on
---
# FR-028: Print a minimal valid request with --example and make request refusals name every defect

## Description

A user who has only the `fsl` binary and its output cannot learn what a
request file holds. Today `fsl run` over `{"format":"native-run/1","request":{}}`
exits 20 with `invalid request: missing field 'models' at line 1 column 2`,
one missing member per attempt and no member path; and `fsl export` over a run
request reports `the selected export invocation requires export`, which
names the missing thing and nothing else.

This requirement gives each request verb a way to print a minimal valid
request, and fixes what its missing-member, wrong-format and export refusals
tell the user. FSL is a library: its refusals carry structured `details` and
no CLI text, so the driver derives the `fsl export --example` hint from
`details.missing` and appends it outside FSL's document. The request
verbs are `run`, `compile`, `analyze`, `monitor`, `replay` and `export`
(`export --evidence <request-file>` reads the evidence request). `check`, `parse`, `format` and
`inspect` read no request file, so `--example` is an unknown option on them.

The request formats and their members belong to FSL (FSL FR-1047). The driver
prints the example FSL returns and writes FSL's refusal document unchanged
(FR-002); it keeps no copy of a request shape. Its one addition is the
export hint below, written outside FSL's document. The refusal contract below
is FSL's, pinned here so the binary a user runs can be tested against it.

## Inputs

- `fsl <verb> --example` for a request verb, with no operand and no other
  option except `--format`. On `export`, `--example-evidence` in place of
  `--example` selects the evidence request.
- A request file whose `format` or members are wrong, given to a request
  verb.

## Outputs

- For `--example` and `--example-evidence`: one JSON request document on
  stdout, followed by one newline, and nothing on stderr.
- For a request with missing members or a wrong `format`: FSL's refusal document, `invalid-request`, exit 20, written as FR-002 routes refusals.
- For an `export` refusal of a missing `export` or `export_evidence` member:
  that document plus the driver's export hint (Behavior).

## Definitions

- Required member: a member the verb's FSL reader refuses the request for
  lacking, that has no default and is not in a one-of group.
- One-of group: members of which the reader requires exactly one, such as
  `program` or `package` for `run` (the reader refuses a request with neither
  as `a request selects exactly one of program and package`). A group is not a
  required member; a request lacking the whole group lacks the group.
- Missing entry: an element of the refusal's `details.missing` (FSL
  FR-1047): the JSON pointer string of a missing required member, for example
  `/request/models`, or the array of the member pointers of a missing one-of
  group (for `run` `["/request/program","/request/package"]`). The elements
  are in the depth-first order of the verb's request example, and `details`
  carries no line or column.

## Behavior

- `--example` SHALL be an option in the option table entry of `run`,
  `compile`, `analyze`, `monitor`, `replay` and `export`, and
  `--example-evidence` SHALL be an option of `export` only. `--example-evidence`
  takes no value; the existing valued option `--evidence <request-file>` of
  FR-027 is unchanged. Each appears in its verb's `<verb> options:` section and
  in root help (FR-002-AC-24).
- `fsl <verb> --example` SHALL write the request document FSL's
  request-example entry returns for that verb's request format, exactly as
  FSL returns it plus one newline, exit 0, and write nothing
  to stderr. It reads no file, starts no plugin and writes no
  `filament-outcome/1` document, because an example is a request, not an
  outcome. The bytes are the same with no `--format`, `--format text` and
  `--format json`.
- The driver SHALL call `fsl_command::command::request_example(format: &str,
  selection: Option<ExportSelection>)` (FSL FR-1047) with `None` for `run`,
  `compile`, `analyze`, `monitor` and `replay`, with format `native-run/1` and
  `Some(ExportSelection::Ordinary)` for `fsl export --example`, and with
  `Some(ExportSelection::Evidence)` for `fsl export --example-evidence`.
- The example for `run` SHALL be a `native-run/1` request, for `compile`
  `native-compile/1`, for `analyze` `native-analyze/1`, for `monitor`
  `native-monitor/1`, for `replay` `native-replay/1`, and for `export` a
  `native-run/1` request that carries an `export` member.
  `fsl export --example-evidence` SHALL write a `native-run/1` request that
  carries an `export_evidence` member.
- Each example SHALL be one object whose members are `format` and `request`;
  its `request` SHALL carry every required member, exactly one member of each
  one-of group, and no other member.
- An example SHALL be a valid request for its verb: saved to a file and given
  to the same verb (`fsl export --evidence <file>` for the evidence example),
  it is not refused at stage `envelope` or `request`. The files an example
  names are placeholders that may not exist, so a refusal at stage `file`
  naming one of them is the expected outcome.
- If `--example` or `--example-evidence` is given with an operand or an option
  other than `--format`, then `fsl` SHALL refuse `invalid-request`, exit 20,
  naming it and the other argument, before any file is read. `--example` and
  `--example-evidence` given together refuse the same way. Option precedence
  is FR-002's: `--help` (step 3) wins over them, and a bad `--format` refuses
  first (step 1).
- If a request lacks required members, then FSL's refusal SHALL be
  `invalid-request` at stage `request`, exit 20, whose `details.missing`
  lists every missing required member and each missing one-of group at once,
  in the order the verb's example lists them. A missing `format` or `request`
  is listed the same way. Message and `details` carry no line or column.
- If a request's `format` is not read by the verb's FSL entry family, then
  FSL's refusal SHALL be `invalid-request` at stage `request`, exit 20, with
  `details` `{"path": "/format", "accepted": [...]}` listing the formats that
  family reads.
- If `fsl export` reads a `native-run/1` request without an `export` member,
  then FSL's refusal SHALL be `invalid-request` with `details.missing` equal
  to `["/request/export"]`, and the message states the member's shape; for
  `fsl export --evidence <request-file>` and `export_evidence` it is
  `["/request/export_evidence"]`. The message replaces the reason `the
  selected export invocation requires export`.
- FSL's refusal carries no CLI option text. When the export refusal's
  `details.missing` contains `/request/export`, the driver SHALL append the
  hint `run 'fsl export --example' to print a valid request`; when it contains
  `/request/export_evidence`, the hint `run 'fsl export --example-evidence' to
  print a valid request`. In text mode the hint is one further line of stdout
  after FSL's rendered document, `hint: ` followed by the hint and a newline.
  In JSON mode no hint is written: stdout is FSL's `filament-outcome/1` document, byte-equal, with no further member. The hint is outside FSL's document and message: FSL's
  message SHALL NOT contain `--example`. The driver derives the hint only from
  `details.missing`, never from message text, and writes no hint for any other
  refusal. Stderr stays empty. This text-mode hint line is the one addition to FSL's
  output that FR-002 permits for these refusals.
- The driver SHALL write every other refusal of this requirement unchanged
  (FR-002-AC-3).
- Present-but-invalid members (an unknown member, a member of the wrong JSON
  kind, a one-of group with two members) are outside FSL FR-1047 and this
  requirement; the driver writes FSL's refusal for them unchanged and this
  requirement specifies no text or pointer for them.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-028-AC-1 | For each of `run`, `compile`, `analyze`, `monitor` and `replay`, `fsl <verb> --example` exits 0, writes nothing to stderr, and writes stdout byte-equal to the string `fsl_command::command::request_example(<format>, None)` returns plus one newline, which parses as an object whose members are exactly `format` and `request`, `format` being `native-run/1`, `native-compile/1`, `native-analyze/1`, `native-monitor/1` and `native-replay/1` respectively. The same invocation with `--format json` and `--format text` writes the same bytes, and no `filament-outcome/1` document. | Test (TC-050) |
| FR-028-AC-2 | `fsl export --example` writes the string `request_example("native-run/1", Some(ExportSelection::Ordinary))` returns plus one newline, whose `request` has an `export` member and no `export_evidence` member; `fsl export --example-evidence` writes the `Some(ExportSelection::Evidence)` string plus one newline, with an `export_evidence` member and no `export` member; both exit 0 with empty stderr. `fsl export --evidence <file>` is unchanged: it still takes one value. | Test (TC-050) |
| FR-028-AC-3 | Over an empty directory, each verb's example saved to a file `R` and given to the same verb (`fsl export --evidence R` for the evidence example) is refused at stage `file` naming a file the example names, or succeeds; it is never refused at stage `envelope` or `request`. The `run` example carries exactly one of `program` and `package`, so it is never refused `a request selects exactly one of program and package`. | Test (TC-050) |
| FR-028-AC-4 | `fsl run --example R`, `fsl run --example --limit s1.nodes=1`, `fsl compile --example --output m.pkg` and `fsl export --example --example-evidence` each refuse `invalid-request`, exit 20, naming `--example` and the other argument, and read no file. `fsl run --example --help` writes help and exits 0; `fsl run --example --format bogus` refuses naming `--format` and `bogus`. `fsl check --example`, `fsl parse --example`, `fsl format --example`, `fsl inspect --example` and `fsl run --example-evidence` each refuse `invalid-request`, exit 20, naming the option as unknown. For every request verb, `--example` (and for `export`, `--example-evidence`) appears on its own line in the verb's `<verb> options:` section and in root help. | Test (TC-050) |
| FR-028-AC-5 | `fsl run` over `{}` and over `{"format":"native-run/1"}`, and each request verb over `{"format":"<its format>","request":{}}`, refuses `invalid-request`, exit 20, stage `request`, with `details.missing` exactly the expected array and no `line` or `column` in the message or `details`. The expected array is derived from the verb's own `--example` output, not from FSL's reader: for `{}` it is `["/format","/request"]`; for the second `["/request"]`; for the third, one element for each member of the example's `request` in the example's order, where a one-of group's example member is replaced by the array of every member pointer of that group as FSL FR-1047's Definitions list the groups (the oracle, cited by reference and not copied here; for `run`: `"/request/models"`, `["/request/program","/request/package"]`, `["/request/call","/request/clause","/request/items"]`). A reader that reports only the first missing member, or omits a group, fails. | Test (TC-050) |
| FR-028-AC-6 | Withdrawn. FSL FR-1047 covers missing members and wrong formats only, not present-but-invalid members, so a wrong-typed `models` expecting `/request/models`, an unknown member `/request/bogus` and a request carrying both `program` and `package` have no FSL contract to test; no criterion is asserted for them until FSL specifies one. | None |
| FR-028-AC-7 | `fsl export R` over a `native-run/1` request with no `export` member refuses `invalid-request`, exit 20, stage `request`, with `details.missing` equal to `["/request/export"]` and a message that does not contain `requires export` and does not contain `--example`. `fsl export --evidence R` over one with no `export_evidence` member refuses likewise with `details.missing` `["/request/export_evidence"]`. In text mode (the default and `--format text`) stdout ends with exactly one line `hint: run 'fsl export --example' to print a valid request` (`... 'fsl export --example-evidence' ...` for the evidence request), and what precedes it equals FSL's text rendering of the refusal; in JSON mode (`--format json`) stdout is byte-equal to FSL's refusal document, with no hint and no further member. Stderr is empty in both modes. `fsl export` over `{"format":"native-run/1","request":{}}` (an AC-5 case) lists `/request/export` in `details.missing` and therefore carries the text-mode hint line too; every refusal whose `details.missing` lacks `/request/export` and `/request/export_evidence` (all other AC-5 cases, and AC-9) has no `hint:` line. | Test (TC-050) |
| FR-028-AC-8 | Each refusal of FR-028-AC-5, FR-028-AC-7 and FR-028-AC-9 equals the document the FSL entry the driver calls for the verb under test returns for the same request, byte-equal in JSON mode, and in text mode byte-equal except for the AC-7 `hint:` line: `run_with_providers_and_limits_and_overrides` for `run` (FR-002), `compile_outcome_with_overrides` for `compile` (`fsl-cli/src/cli.rs`; FSL FR-287; FR-024 names no entry), `analyze_with_providers_and_limits` for `analyze` (FR-022 names `analyze_with_providers`; the driver calls the `_and_limits` form), `monitor` for `monitor` (FR-025), `run_with_providers` for `replay` (FR-026) and `export_with_limits_and_overrides` for `export` (FR-027 names only "FSL's export entry"; the driver calls this one), exits 20 in both output modes, and in text mode and JSON mode writes only its outcome to stdout and nothing to stderr (FR-002). | Test (TC-050) |
| FR-028-AC-9 | `fsl run` over a `native-analyze/1` request, `fsl analyze` over a `native-run/1` request and `fsl compile` over a `native-run/1` request each refuse `invalid-request`, exit 20, stage `request`, never `unknown_wire`, with `details` `{"path":"/format","accepted":[...]}` where `accepted` is `["native-run/1","native-monitor/1","native-replay/1"]` for `run`, `["native-analyze/1"]` for `analyze` and `["native-compile/1"]` for `compile`. | Test (TC-050) |

## Dependencies

- This change lands after FSL PR 3548 (FSL FR-1047) merges; its criteria are written against FR-1047 as that PR states it.

- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md): the option
  table, help, refusal routing and exit contract these criteria extend.
- [FR-027](FR-027-write-a-tla-or-lean-export-with-fsl-export.md): the export
  request (`native-run/1` with `export` or `export_evidence`) and the valued
  `--evidence` option.
- FSL FR-100, FR-620, FR-644, FR-862, FR-1015, FR-1022 and FR-027: the run,
  analyze, monitor, replay, export, export-evidence and compile request
  readers. FSL FR-287 lists the driver-facing verbs. FSL FR-1047 (FSL ticket QSL-2935, PR #3548) owns the request-example entry
  `fsl_command::command::request_example`, `details.missing`, the accepted-format
  `details` for a wrong `format`, and the export message text. The driver's
  `--example` calls that entry; this spec carries no copy of FSL's request
  shapes or text, and it does not change `run`'s dispatch on `format`. This
  requirement is not implementable until QSL-2935 lands; the entry is the
  oracle of AC-1 and AC-8.
- [TC-050](../test-cases/TC-050-print-a-request-example-and-name-request-defects.md).
