---
id: FR-027
title: "Write a TLA+ or Lean export with fsl export --output and take in target evidence with --evidence"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-driver/FR-024
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-1015
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-1022
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-287
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-286
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-865
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-817
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-301
    type: references
---
# FR-027: Write a TLA+ or Lean export with fsl export --output and take in target evidence with --evidence

## Description

`fsl export <request-file>` SHALL hand one `native-run/1` request carrying an
`export` member ([FSL FR-1015](ix://agent-ix/filament-spec-language/FR-1015),
FSpec FR-817) to FSL's export entry and write the one FR-286 outcome document
with operation `export` that FSL returns: unchanged in machine mode, and in
FSL FR-865's text form in text mode. For `fsl export --evidence
<request-file>`, the driver SHALL hand one `native-run/1` request carrying an `export_evidence` member to
FSL's evidence intake ([FSL FR-1022](ix://agent-ix/filament-spec-language/FR-1022))
and write the outcome document it returns. The exit code is FSL FR-285's code
of the document's category.

`--output <file>` names the TLA+ or Lean target file. The driver hands the
path to FSL as the request override `output`
(`fsl_command::command::RequestOverrides`), as `fsl compile --output` does
(FR-024), and writes no target bytes of its own, so the option and the
request's own `export.output` member produce byte-identical files. FSL owns
the write: it writes only an assembled export, never over an existing path,
and leaves no partial file.

## Inputs

- The request-file operand, a relative path resolving against the working
  directory, or `--evidence <file>` in its place.
- `--output <file>` at most once, on `export` without `--evidence`.
- `--limit <name>=<value>`, repeatable, for the `export.*` settings and every
  setting `fsl compile` accepts (FSL FR-255, FR-869).
- `--format json|text`, default `text` (FR-002).

## Behavior

- The driver SHALL call FSL's export entry with the request path and the
  overrides and SHALL write the returned document; it adds, removes or
  reorders no member.
- If the target path already exists, then FSL refuses `invalid-request`,
  exit 20, naming the path, with `last_stage` null, and the driver writes
  nothing beyond FSL's document.
- A request whose `export` member carries `output` while `--output` is given
  is refused by FSL `invalid-request`, exit 20, naming `output` and
  `--output`, and no file is written. The driver adds no check of its own.
- If `--output` is given more than once or without a path, then `fsl` SHALL
  refuse the command line, exit 20, naming `--output`, and write no file.
- If `--evidence` is given with `--output` or with a request-file operand,
  then `fsl` SHALL refuse `invalid-request`, exit 20, naming `--evidence` and
  the other argument, before any file is read.
- `--evidence` on any verb other than `export` is an unknown option (FSpec
  FR-301), exit 20.
- In text mode the driver SHALL write `render_outcome`'s text of the same
  document: for an assembled export, the header, `target:`,
  `bytes written:` and `output:` lines of FSL FR-865.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-027-AC-1 | `fsl export <request> --output m.tla --format json` over FR-756-AC-1's counter request (`inc`, `value <= 5`) exits 0, writes `m.tla` whose bytes equal the document's `result.export.target_text`, and writes FSL's outcome document with operation `export`, category `success` and `result.kind` `exported`; the request carrying `export.output` naming `m.tla`, run with no option in a fresh directory, writes a byte-identical file. | Test (TC-045) |
| FR-027-AC-2 | `--output` naming an existing file refuses `invalid-request`, exit 20, `last_stage` null, naming the path, and the file's bytes are unchanged; a request carrying `export.output` with `--output` refuses naming `output` and `--output`, exit 20, and writes neither file; `--output` given twice refuses naming `--output`, exit 20. | Test (TC-045) |
| FR-027-AC-3 | `fsl export <request> --output m.tla --limit export.output_bytes=1` exits 22, names `export.output_bytes` and writes no file; with `--limit export.output_bytes=16777216` it exits 0. | Test (TC-045) |
| FR-027-AC-4 | `fsl export <request> --output m.tla` with no `--format` writes exactly the four lines `export: success (exit 0) at stage packaging`, `target: tla filament.output.tla2/v1`, `bytes written: <n>` and `output: m.tla`, and exits as in machine mode. | Test (TC-045) |
| FR-027-AC-5 | `fsl export --evidence e.json --format json` writes FSL's evidence outcome document with operation `export`; `fsl export --evidence e.json --output m.tla` and `fsl export <request> --evidence e.json` each refuse `invalid-request` naming `--evidence`, exit 20, with nothing written to the output path; `fsl compile <request> --evidence e.json` refuses as an unknown option, exit 20. | Test (TC-045) |

## Dependencies

- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md): the outcome and
  exit contract and the text default.
- [FR-024](FR-024-write-the-checked-package-with-output.md): the same
  request-override handoff for `fsl compile --output`.
- FSL FR-1015, FR-1022, FR-287, FR-286 and FR-865: the export entry, the
  evidence intake, the verb, the outcome document and its text.
- FSpec FR-817 and FR-301: the export request, outcome and CLI contract.
