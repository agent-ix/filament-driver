---
id: FR-026
title: "Write the replay results document and its text outcome with fsl replay"
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
  - target: ix://agent-ix/filament-spec-language/FR-072
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-285
    type: depends_on
---
# FR-026: Write the replay results document and its text outcome with fsl replay

## Description

`fsl replay <request-file>` SHALL hand one `native-replay/1` request
([FSL FR-862](ix://agent-ix/filament-spec-language/FR-862)) to FSL's replay
reader through `fsl_command::command::run_with_providers`, and write what FSL
returns: in machine mode the FSpec FR-323 `results` document of the replay,
and in text mode the FSL FR-865 text of the replay's outcome document followed
by one line per result. The process exits with the outcome's FR-285 exit code.
FR-023 states the request's `clocks` refusal; this requirement states the
verb's output.

### Decision: the results document in machine mode, the outcome text in text mode

FSL returns two forms for a replay that runs: the `results` document, which
holds FSL FR-072's typed result per row (`arm`, `settlement`, `category`,
`value`, `charges` and the arm's other members), and the FR-286 outcome
document with operation `replay`, which holds the category and the last stage
but no result member. The `results` document is the replay's value, the one a
script reads, as `fsl run` writes a run's native result; so machine mode
writes it unchanged. The outcome document is what FR-865 renders for a
reader, but its text carries no result line for a replay, so text mode adds
one line per row naming its arm and settlement, which a reader needs to know
whether the counterexample reproduced. A refused replay has no `results`
document, so both modes write the refusal's outcome document, as every other
verb does.

## Inputs

- One request-file operand; files the request names resolve against its
  folder (FSL FR-862).
- `--format json` (machine mode) or `--format text` (the default),
  `--provider-config` and `--provider-limit` as FR-006 states.

## Outputs

- Machine mode, a replay that runs: the `{"results": [...]}` document FSL
  returns, in ix-cli-kit's canonical JSON followed by one newline, on stdout.
- Machine mode, a refused replay: the FR-286 outcome document with operation
  `replay`, its category, its `last_stage` and one diagnostic, on stdout.
- Text mode, a replay that runs: FSL FR-865's text of the outcome document
  (`replay: <category> (exit <code>) at S8`), then per `results` row, in
  order, `result <index>: <arm> <settlement>`, on stdout.
- Text mode, a refused replay: FSL FR-865's text of the refusal's outcome
  document, its header and its diagnostic line, on stdout.
- The FR-285 exit code of the outcome's category, in both modes.

## Behavior

- The driver SHALL write FSL's `results` document unchanged in machine mode
  and exit with FSL's exit code; when that code is not the exit code of the
  outcome document's category, the driver SHALL exit with the
  internal-failure code and a diagnostic naming both, rather than guess.
- In text mode the driver SHALL render the outcome document through FSL's
  `render_outcome` (FSL FR-865) and then write one
  `result <index>: <arm> <settlement>` line per `results` row, `index` the
  row's zero-based position and `arm` and `settlement` the row's members.
- The driver SHALL write a replay FSL refuses (at stage `request`, at a
  recompile stage or at replay) as its outcome document in machine mode and its
  FR-865 text in text mode, with the refusal's category code; the driver
  writes no `results` member and no result line for it.
- The driver SHALL refuse a request file whose `format` is not
  `native-replay/1` before FSL runs, as FR-023 states, with an outcome document in machine mode
  and its FR-865 text in text mode.
- No `fsl replay` invocation SHALL exit without writing a document or a
  line to stdout.

## Acceptance Criteria

The replay fixture is FR-023's: the function `entry` of a unit importing
`test/plant`, compiled with `fsl compile`, and a `native-replay/1` request of
its input arm.

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-026-AC-1 | `fsl replay <request> --format json` over the fixture writes stdout byte-equal to the canonical encoding of the `results` document `fsl_command::command::run` returns in process for the same request, followed by one newline: one row with `arm` `input` and `settlement` `reproduced-without-witness`, and exits 0. | Test (TC-042) |
| FR-026-AC-2 | The same request without `--format` writes the line `replay: success (exit 0) at S8` followed by `result 0: input reproduced-without-witness`, and exits 0. | Test (TC-042) |
| FR-026-AC-3 | The fixture request with `clocks` added writes in text mode the line `replay: refusal (exit 20)` followed by one diagnostic line whose code is `invalid-request`, and no `result` line; in machine mode an outcome document with operation `replay`, category `refusal` and no `results` member; both exit 20. | Test (TC-042) |
| FR-026-AC-4 | A `native-run/1` request handed to `fsl replay` writes in text mode a `replay: refusal (exit 20)` line and no `result` line, and exits 20, with no replay run. | Test (TC-042) |

## Dependencies

- FSL FR-862 owns the replay request reader and its refusals; FSL FR-072 the
  result; FSL FR-865 the text form of an outcome document.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) owns the CLI's
  stream and exit discipline; [FR-023](FR-023-supply-clock-configurations-with-clocks.md)
  the replay request's `clocks` refusal.
