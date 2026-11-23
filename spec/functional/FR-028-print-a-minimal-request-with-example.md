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
request, and fixes what its missing-member and export refusals tell the user. The
accepted-format list for a wrong `format` is FSL's and is not specified here. The request
verbs are `run`, `compile`, `analyze`, `monitor`, `replay` and `export`
(`export --evidence <request-file>` reads the evidence request). `check`, `parse`, `format` and
`inspect` read no request file, so `--example` is an unknown option on them.

The request formats and their members belong to FSL. The driver prints the
example FSL returns and writes FSL's refusals unchanged (FR-002); it keeps
no copy of a request shape and adds no refusal text of its own. The refusal
text below is FSL's contract, pinned here so the binary a user runs can be
tested against it.

## Inputs

- `fsl <verb> --example` for a request verb, with no operand and no other
  option except `--format`. On `export`, `--example-evidence` in place of
  `--example` selects the evidence request.
- A request file whose `format` or members are wrong, given to a request
  verb.

## Outputs

- For `--example` and `--example-evidence`: one JSON request document on
  stdout, followed by one newline, and nothing on stderr.
- For a defective request: FSL's refusal document, `invalid-request`, exit 20, written as FR-002 routes refusals.

## Definitions

- Required member: a member the verb's FSL reader refuses the request for
  lacking, that has no default and is not in a one-of group.
- One-of group: members of which the reader requires exactly one, such as
  `program` or `package` for `run` (the reader refuses a request with neither
  as `a request selects exactly one of program and package`). A group is not a
  required member; a request lacking the whole group lacks the group.
- Missing-member pointer form: a missing required member is its JSON pointer,
  for example `/request/models`. A missing one-of group lists every member
  pointer of the group (for `run`, `/request/program` and `/request/package`)
  together as one entry that shows they are alternatives. FSL owns the exact
  join; this requirement does not fix it.
- Invalid member: a present member the FSL reader refuses because it is
  unknown to the format, has the wrong JSON kind, is outside its value
  domain, or conflicts with another member (a one-of group with two members).

## Behavior

- `--example` SHALL be an option in the option table entry of `run`,
  `compile`, `analyze`, `monitor`, `replay` and `export`, and
  `--example-evidence` SHALL be an option of `export` only. `--example-evidence`
  takes no value; the existing valued option `--evidence <request-file>` of
  FR-027 is unchanged. Each appears in its verb's `<verb> options:` section and
  in root help (FR-002-AC-24).
- `fsl <verb> --example` SHALL write the request document FSL's
  request-example entry (`fsl_command::command::request_example`, specified by
  the FSL requirement named in Dependencies) returns for that verb's request
  format, exactly as FSL returns it plus one newline, exit 0, and write nothing
  to stderr. It reads no file, starts no plugin and writes no
  `filament-outcome/1` document, because an example is a request, not an
  outcome. The bytes are the same with no `--format`, `--format text` and
  `--format json`.
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
  `invalid-request`, exit 20, and its diagnostic message SHALL list every
  missing required member and each missing one-of group at once, in the
  missing-member pointer form, in the order the verb's example lists them. A missing `format` or `request` is
  reported the same way. The message form is FSL's.
- If a request carries an invalid member, then FSL's refusal message SHALL
  contain the JSON pointer of that member.
- If `fsl export` reads a `native-run/1` request without an `export` member,
  then FSL's refusal SHALL be `invalid-request` whose member is `export` and
  whose reason names the accepted format and member and points to the
  example. It replaces the reason `the selected export invocation requires
  export`. For `fsl export --evidence <request-file>` the member is
  `export_evidence` and the pointer is to `fsl export --example-evidence`.
- The driver SHALL write each of these refusals unchanged (FR-002-AC-3).
- The driver SHALL NOT decide anything from the text of these refusals.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-028-AC-1 | For each of `run`, `compile`, `analyze`, `monitor` and `replay`, `fsl <verb> --example` exits 0, writes nothing to stderr, and writes stdout byte-equal to the document `fsl_command::command::request_example` returns for that verb's format plus one newline, which parses as an object whose members are exactly `format` and `request`, `format` being `native-run/1`, `native-compile/1`, `native-analyze/1`, `native-monitor/1` and `native-replay/1` respectively. The same invocation with `--format json` and `--format text` writes the same bytes, and no `filament-outcome/1` document. | Test (TC-050) |
| FR-028-AC-2 | `fsl export --example` writes a `native-run/1` request whose `request` has an `export` member and no `export_evidence` member; `fsl export --example-evidence` writes one with an `export_evidence` member and no `export` member; both exit 0 with empty stderr. `fsl export --evidence <file>` is unchanged: it still takes one value. | Test (TC-050) |
| FR-028-AC-3 | Over an empty directory, each verb's example saved to a file `R` and given to the same verb (`fsl export --evidence R` for the evidence example) is refused at stage `file` naming a file the example names, or succeeds; it is never refused at stage `envelope` or `request`. The `run` example carries exactly one of `program` and `package`, so it is never refused `a request selects exactly one of program and package`. | Test (TC-050) |
| FR-028-AC-4 | `fsl run --example R`, `fsl run --example --limit s1.nodes=1`, `fsl compile --example --output m.pkg` and `fsl export --example --example-evidence` each refuse `invalid-request`, exit 20, naming `--example` and the other argument, and read no file. `fsl run --example --help` writes help and exits 0; `fsl run --example --format bogus` refuses naming `--format` and `bogus`. `fsl check --example`, `fsl parse --example`, `fsl format --example`, `fsl inspect --example` and `fsl run --example-evidence` each refuse `invalid-request`, exit 20, naming the option as unknown. For every request verb, `--example` (and for `export`, `--example-evidence`) appears on its own line in the verb's `<verb> options:` section and in root help. | Test (TC-050) |
| FR-028-AC-5 | `fsl run` over `{}` and over `{"format":"native-run/1"}`, and each request verb over `{"format":"<its format>","request":{}}`, refuses `invalid-request`, exit 20, with a diagnostic message that contains, once each, the expected pointers and contains no line or column. The expected pointers are derived from the verb's own `--example` output, not from FSL's reader: for `{}` they are `/format` and `/request`; for the second `/request`; for the third, one pointer for each member of the example's `request`, in the example's order, where the example's member of a one-of group is replaced by every member pointer of the group, shown as one alternatives entry in FSL's join (for `run`: `/request/models`, and `/request/program` with `/request/package`). A reader that reports only the first missing member, or omits the group, fails. | Test (TC-050) |
| FR-028-AC-6 | `fsl run R` over `{"format":"native-run/1","request":{"models":"x"}}` refuses `invalid-request`, exit 20, with a diagnostic message that contains `/request/models`; the same holds for an unknown member `/request/bogus` and for a request with both `program` and `package`. | Test (TC-050) |
| FR-028-AC-7 | `fsl export R` over a `native-run/1` request with no `export` member refuses `invalid-request`, exit 20, with member `export` and a reason that contains `native-run/1`, `export` and `fsl export --example`; `fsl export --evidence R` over one with no `export_evidence` member refuses with member `export_evidence` and a reason that contains `native-run/1`, `export_evidence` and `fsl export --example-evidence`. Neither refusal contains `requires export`. | Test (TC-050) |
| FR-028-AC-8 | Each refusal of FR-028-AC-5 to FR-028-AC-7 is byte-equal to the document the FSL entry the driver calls for the verb under test returns for the same request: `run_with_providers_and_limits_and_overrides` for `run` (FR-002), `compile_outcome_with_overrides` for `compile` (`fsl-cli/src/cli.rs`; FSL FR-287; FR-024 names no entry), `analyze_with_providers_and_limits` for `analyze` (FR-022 names `analyze_with_providers`; the driver calls the `_and_limits` form), `monitor` for `monitor` (FR-025), `run_with_providers` for `replay` (FR-026) and `export_with_limits_and_overrides` for `export` (FR-027 names only "FSL's export entry"; the driver calls this one), exits 20 in both output modes, and in text mode and JSON mode writes only its outcome to stdout and nothing to stderr (FR-002). | Test (TC-050) |

## Dependencies

- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md): the option
  table, help, refusal routing and exit contract these criteria extend.
- [FR-027](FR-027-write-a-tla-or-lean-export-with-fsl-export.md): the export
  request (`native-run/1` with `export` or `export_evidence`) and the valued
  `--evidence` option.
- FSL FR-100, FR-620, FR-644, FR-862, FR-1015, FR-1022 and FR-027: the run,
  analyze, monitor, replay, export, export-evidence and compile request
  readers. FSL FR-287 lists the driver-facing verbs. FSL ticket QSL-2935
  (titan:QSL:code) owns the request-example entry
  `fsl_command::command::request_example`, the all-missing-members-with-pointers
  behaviour, the accepted-format list for a wrong `format`, and the export
  message text. The driver's `--example` calls that entry when it exists; this
  spec carries no copy of FSL's request shapes or text, and it does not
  change FSL FR-620-AC-9 (a `run`/`analyze` format mismatch stays
  `invalid-request` at stage `request`) or `run`'s dispatch on `format`. This
  requirement is not implementable until QSL-2935 lands; the entry is the
  oracle of AC-1 and AC-8.
- Listing a missing one-of group together with the missing required members in
  one refusal is new FSL reader behaviour. Today `required_program`
  (`fsl-command/src/command.rs`) runs only after serde accepts the request, so
  a request missing `models` is refused for `models` alone. QSL-2935
  must specify it.
- [TC-050](../test-cases/TC-050-print-a-request-example-and-name-request-defects.md).
