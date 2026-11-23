---
id: FR-026
title: "Write the replay outcome document and its text with fsl replay"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-driver/FR-023
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-862
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-865
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-286
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-072
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-285
    type: depends_on
---
# FR-026: Write the replay outcome document and its text with fsl replay

## Description

`fsl replay <request-file>` SHALL hand one `native-replay/1` request
([FSL FR-862](ix://agent-ix/filament-spec-language/FR-862)) to FSL's replay
reader through `fsl_command::command::replay_with_limits`, and write the one
FR-286 outcome document FSL returns for it: unchanged in machine mode, and in
text mode as `fsl_inspect::render_outcome` renders that same document (FSL
FR-865). Both success and refusal outcomes go to stdout, with nothing to
stderr (FR-002). The process exits with the outcome's FR-285 exit code. FR-023 states
the request's `clocks` refusal; this requirement states the verb's output.

### Decision: one document in both modes, rendered by FSL

The replay's outcome document carries its result: FSL FR-286 gives a replay
that runs the `result` `{"kind": "replayed", "results": [...]}`, each entry a
row of the FSpec FR-323 `results` document with its arm, settlement, value
and charges, and FSL FR-865 renders each entry as one
`result <index>: <arm> <settlement>` line. So machine mode and text mode carry
the same document, as for every other verb (FR-002), and the driver adds no
line or member of its own: a reader of the text sees what a script reads
from the JSON, and a change to the result's text is made once, in FSL.

## Inputs

- One request-file operand; files the request names resolve against its
  folder (FSL FR-862).
- Zero or more `--limit name=value` options, resolved and applied as FR-003
  specifies.
- `--format json` (machine mode) or `--format text` (the default),
  `--provider-config` and `--provider-limit` as FR-006 states.

## Outputs

- Machine mode: the FR-286 outcome document with operation `replay`, in
  ix-cli-kit's canonical JSON followed by one newline, on stdout; for a replay
  that runs its `result` is FSL's `replayed` result, for a refused replay its
  `result` is `null` and it carries the refusal's diagnostic.
- Text mode: `render_outcome`'s text of that same document on stdout.
- The FR-285 exit code of the outcome's category, in both modes.

## Behavior

- The driver SHALL write FSL's outcome document unchanged in machine mode and
  exit with FSL's exit code; when that code is not the exit code of the
  document's category, the driver SHALL exit with the internal-failure code
  and a diagnostic naming both, rather than guess.
- In text mode the driver SHALL write exactly the bytes `render_outcome`
  returns for that document with `RenderForm::Text`, and SHALL write no line
  of its own.
- The driver SHALL write a replay FSL refuses (at stage `request`, at a
  recompile stage or at replay) the same way, as the refusal's outcome
  document, with the refusal's category code.
- The driver SHALL refuse a request file whose `format` is not
  `native-replay/1` before FSL runs, as FR-023 states, with an outcome
  document written the same way.
- The driver SHALL apply each accepted `--limit` override to the matching
  exposed replay bound before calling FSL. A caller value overrides the
  request value for that bound; other request values remain unchanged.
- Every `fsl replay` invocation SHALL write either a document to stdout in
  JSON mode or the selected text rendering of its outcome to stdout in text
  mode; it SHALL write nothing to stderr for an outcome refusal.

## Acceptance Criteria

The replay fixture is FR-023's: the function `entry` of a unit importing
`test/plant`, compiled with `fsl compile`, and a `native-replay/1` request of
its input arm.

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-026-AC-1 | `fsl replay <request> --format json` over the fixture writes stdout byte-equal to the canonical encoding of the outcome document `fsl_command::command::run` returns in process for the same request, followed by one newline: operation `replay`, category `success`, and `result` `{"kind": "replayed", "results": [R]}` with `R`'s `arm` `input` and `settlement` `reproduced-without-witness`; exit 0. | Test (TC-042) |
| FR-026-AC-2 | The same request without `--format` writes stdout byte-equal to `render_outcome`'s text of AC-1's document, whose lines are `replay: success (exit 0) at stage replay` and `result 0: input reproduced-without-witness`; exit 0. | Test (TC-042) |
| FR-026-AC-3 | The fixture request with `clocks` added writes in machine mode one outcome document on stdout with operation `replay`, category `refusal`, `result` `null` and an `invalid-request` diagnostic, and nothing to stderr; in text mode it writes the selected text rendering of that refusal outcome to stdout and nothing to stderr. Both exit 20. | Test (TC-042) |
| FR-026-AC-4 | A `native-run/1` request handed to `fsl replay` writes the selected text rendering of the refusal outcome to stdout in text mode and one refusal document to stdout in JSON mode, with no stderr in either mode; it exits 20 with no replay run. | Test (TC-042) |

## Dependencies

- FSL FR-862 owns the replay request reader and its refusals; FSL FR-286 the
  outcome document and its `replayed` result; FSL FR-072 the result rows; FSL
  FR-865 the text form.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) owns the CLI's
  stream and exit discipline; [FR-023](FR-023-supply-clock-configurations-with-clocks.md)
  the replay request's `clocks` refusal.
