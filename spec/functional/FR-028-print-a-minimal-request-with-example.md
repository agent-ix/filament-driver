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
one missing member per attempt and no member path; `fsl export` over a run
request reports `the selected export invocation requires export`, which
names the missing thing and nothing else; and a request whose `format` the
verb does not read reports `unsupported native command format` with no list
of what the verb reads.

This requirement gives each request verb a way to print a minimal valid
request, and fixes what its request refusals tell the user. The request
verbs are `run`, `compile`, `analyze`, `monitor`, `replay` and `export`
(`export --evidence` reads its own request). `check`, `parse`, `format` and
`inspect` read no request file, so `--example` is an unknown option on them.

The request formats and their members belong to FSL. The driver prints the
example FSL returns and writes FSL's refusals unchanged (FR-002); it keeps
no copy of a request shape and adds no refusal text of its own. The refusal
text below is FSL's contract, pinned here so the binary a user runs can be
tested against it.

## Inputs

- `fsl <verb> --example` for a request verb, with no operand and no other
  option except `--format`, and, on `export`, `--evidence`.
- A request file whose `format` or members are wrong, given to a request
  verb.

## Outputs

- For `--example`: one JSON request document on stdout, followed by one
  newline, and nothing on stderr.
- For a defective request: FSL's refusal document, `invalid-request` or
  `unknown_wire`, exit 20, written as FR-002 routes refusals.

## Behavior

- `--example` SHALL be an option in the option table entry of `run`,
  `compile`, `analyze`, `monitor`, `replay` and `export`, so it appears in
  that verb's `<verb> options:` section and in root help (FR-002-AC-24).
- `fsl <verb> --example` SHALL write the request document FSL's
  request-example entry returns for that verb's request format, exactly as
  FSL returns it plus one newline, exit 0, and write nothing to stderr. It
  reads no file, starts no plugin and writes no `filament-outcome/1`
  document, because an example is a request, not an outcome. The bytes are
  the same with no `--format`, `--format text` and `--format json`.
- The example for `run` is a `native-run/1` request, for `compile`
  `native-compile/1`, for `analyze` `native-analyze/1`, for `monitor`
  `native-monitor/1`, for `replay` `native-replay/1`, and for `export` a
  `native-run/1` request that carries an `export` member. `fsl export
  --evidence --example` writes a `native-run/1` request that carries an
  `export_evidence` member. Each is one object whose members are `format` and
  `request`, and `request` carries every member the verb's reader requires
  and no optional member.
- An example SHALL be a valid request for its verb: saved to a file and given
  to the same verb (with `--evidence` for the evidence example), it is not
  refused at stage `envelope` or `request`. The files an example names are
  placeholders that may not exist, so a refusal at stage `file` naming one of
  them is the expected outcome.
- `--example` SHALL refuse `invalid-request`, exit 20, naming `--example` and
  the other argument, before any file is read, when the invocation also has an
  operand or an option other than `--format` and, on `export`, `--evidence`.
  Option precedence is FR-002's: `--help` (step 3) wins over `--example`,
  and a bad `--format` refuses first (step 1).
- When a request is not valid JSON of the verb's shape because required
  members are absent, FSL's refusal SHALL be `invalid-request`, exit 20, with
  a diagnostic message that lists every missing required member at once, each
  as its JSON pointer from the document root, in the order the verb's example
  lists them: `invalid request: missing required member <pointer>` for one and
  `invalid request: missing required members <pointer>, <pointer>[, ...]` for
  several. A missing `format` or `request` is reported the same way.
- When a request member is present but invalid, the refusal's message SHALL
  contain the JSON pointer of that member.
- When a request's `format` is not one the verb reads, FSL's refusal SHALL
  name the format it read and list every format the verb reads, as
  `unsupported native command format "<format>"; this verb reads <format>[,
  <format>...]`. The lists are `native-run/1` for `run` and `export`,
  `native-compile/1` for `compile`, `native-analyze/1` for `analyze`,
  `native-monitor/1` for `monitor` and `native-replay/1` for `replay`.
- When `fsl export` reads a `native-run/1` request without an `export`
  member, FSL's refusal SHALL name the accepted format and member and point
  to the example: `fsl export reads a native-run/1 request that carries an
  export member; run `fsl export --example` for one`. For `--evidence` the
  member is `export_evidence` and the pointer is `fsl export --evidence
  --example`. This replaces `the selected export invocation requires export`.
  No `native-export/1` format exists; the export request is the `native-run/1`
  request FR-027 specifies, and no message names `native-export/1` as accepted.
- The driver SHALL write each of these refusals unchanged (FR-002-AC-3).
- The driver SHALL NOT decide anything from the text of these refusals.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-028-AC-1 | For each of `run`, `compile`, `analyze`, `monitor` and `replay`, `fsl <verb> --example` exits 0, writes nothing to stderr, and writes stdout byte-equal to the document FSL's request-example entry returns for that verb's format plus one newline, which parses as an object whose members are exactly `format` and `request`, `format` being `native-run/1`, `native-compile/1`, `native-analyze/1`, `native-monitor/1` and `native-replay/1` respectively. The same invocation with `--format json` and `--format text` writes the same bytes, and no `filament-outcome/1` document. | Test (TC-050) |
| FR-028-AC-2 | `fsl export --example` writes a `native-run/1` request whose `request` has an `export` member and no `export_evidence` member; `fsl export --evidence --example` writes one with an `export_evidence` member and no `export` member; both exit 0 with empty stderr, and `fsl export --example --evidence` writes the same bytes as `fsl export --evidence --example`. | Test (TC-050) |
| FR-028-AC-3 | Over an empty directory, each verb's example saved to a file `R` and given to the same verb (`fsl export --evidence R` for the evidence example) exits with a refusal whose stage is `file` naming a file the example names, or a success; it is never refused at stage `envelope` or `request`, so no example is missing a required member. | Test (TC-050) |
| FR-028-AC-4 | `fsl run --example R`, `fsl run --example --limit s1.nodes=1` and `fsl compile --example --output m.pkg` each refuse `invalid-request`, exit 20, naming `--example` and the other argument (`R`, `--limit`, `--output`), and read no file. `fsl run --example --help` writes help and exits 0; `fsl run --example --format bogus` refuses naming `--format` and `bogus`. `fsl check --example`, `fsl parse --example`, `fsl format --example` and `fsl inspect --example` each refuse `invalid-request`, exit 20, naming `--example` as an unknown option. For every request verb, `--example` appears on its own line in the verb's `<verb> options:` section and in root help. | Test (TC-050) |
| FR-028-AC-5 | `fsl run R` over `{"format":"native-run/1","request":{}}` refuses `invalid-request`, exit 20, with the diagnostic message `invalid request: missing required member /request/models`; `fsl monitor R` over `{"format":"native-monitor/1","request":{}}` with the message `invalid request: missing required members /request/models, /request/program, /request/clauses, /request/trace`; `fsl run R` over `{}` with `invalid request: missing required members /format, /request`; and `fsl run R` over `{"format":"native-run/1"}` with `invalid request: missing required member /request`. No message names a line or column in place of a pointer, and each lists all of its missing members in one refusal. | Test (TC-050) |
| FR-028-AC-6 | `fsl run R` over `{"format":"native-run/1","request":{"models":"x"}}` refuses `invalid-request`, exit 20, with a diagnostic message that contains `/request/models`. | Test (TC-050) |
| FR-028-AC-7 | `fsl run R` over `{"format":"native-compile/1","request":{}}` refuses `unknown_wire` with the message `unsupported native command format "native-compile/1"; this verb reads native-run/1`, exit 20; the same message form, naming that verb's accepted format from the list above, comes from `compile`, `analyze`, `monitor`, `replay` and `export` over a `native-run/1`, `native-compile/1`, `native-analyze/1`, `native-monitor/1` or `native-replay/1` request that the verb does not read. `fsl export R` over `{"format":"native-export/1","request":{}}` refuses with `unsupported native command format "native-export/1"; this verb reads native-run/1`. | Test (TC-050) |
| FR-028-AC-8 | `fsl export R` over a `native-run/1` request with no `export` member refuses `invalid-request`, exit 20, with the message reason `fsl export reads a native-run/1 request that carries an export member; run `fsl export --example` for one`; `fsl export --evidence R` over one with no `export_evidence` member refuses with `fsl export --evidence reads a native-run/1 request that carries an export_evidence member; run `fsl export --evidence --example` for one`. Neither message contains `requires export`. | Test (TC-050) |
| FR-028-AC-9 | Each refusal of FR-028-AC-5 to FR-028-AC-8 is, byte for byte, the document FSL's own entry returns for the same request, exits 20 in both output modes, and in text mode and JSON mode writes only its outcome to stdout and nothing to stderr (FR-002). | Test (TC-050) |

## Dependencies

- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md): the option
  table, help, refusal routing and exit contract these criteria extend.
- [FR-027](FR-027-write-a-tla-or-lean-export-with-fsl-export.md): the export
  request (`native-run/1` with `export` or `export_evidence`).
- FSL FR-100, FR-620, FR-644, FR-862, FR-1015 and FR-027: the run, analyze,
  monitor, replay, export and compile request readers. The request-example
  entry and the refusal text above are FSL changes to those readers and are
  not yet specified in FSL; this requirement is not implementable until they
  are (QSL-2930).
- [TC-050](../test-cases/TC-050-print-a-request-example-and-name-request-defects.md).
