---
id: FR-024
title: "Write the checked package to a file with fsl compile --output"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-810
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-287
    type: references
---
# FR-024: Write the checked package to a file with fsl compile --output

## Description

`fsl compile` takes `--output <file>`, which makes the compile write the
checked package, the `filament.checked-package/v2` bytes, to that file. A
consumer that needs the package as a file then gets it from the command line
with no library call.

The driver hands the path to FSL as the request override `output`
(`fsl_command::command::RequestOverrides`) and writes no package bytes of its
own, so the option and the request's own `output` member
([FSL FR-810](ix://agent-ix/filament-spec-language/FR-810)) produce
byte-identical files. FSL owns the write: it never overwrites an existing path,
leaves no partial file, and writes nothing when the compile refuses.

## Inputs

- `--output <file>` after the request operand, at most once, on `compile` only.
  A relative path resolves against the working directory.

## Behavior

- If the target path already exists, then `fsl` SHALL refuse with code
  `invalid-request`, exit 20, naming the path, with no stage reached, and SHALL
  leave the existing path untouched.
- A request document that carries an `output` member while `--output` is given
  is refused by FSL (FSL FR-810) `invalid-request`, exit 20, naming `output` and
  `--output`, and writes no file. The outcome document's `last_stage` is null.
  The driver adds no check of its own.
- If `--output` is given more than once or without a path, then `fsl` SHALL
  refuse the command line, exit 20, naming `--output`, and write no file.
- If the compile refuses at any stage, then no file is written.
- `--output` on any verb other than `compile` and `export` is an unknown
  option (FSpec FR-301); `fsl export --output` writes the TLA+ or Lean
  target file through the same request override (FR-027).
- The outcome document of a compile that writes the file is the document the
  same compile gives without it: `artifacts` stays `[]`.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-024-AC-1 | `fsl compile <request> --output <file>` over a request without `output` exits 0, writes a file whose bytes equal those `fsl_command::command::compile` returns and those the request carrying the same path as its `output` member writes, resolves a relative path against the working directory, and gives an outcome with `artifacts` `[]` equal to the member's. | Test (TC-038) |
| FR-024-AC-2 | `--output` naming an existing file refuses `invalid-request`, exit 20, `last_stage` null, naming the path, and the file's bytes are unchanged. | Test (TC-038) |
| FR-024-AC-3 | A request carrying `output` with `--output` refuses `invalid-request` (FSL's refusal, FR-810), exit 20, naming `output` and `--output`, `last_stage` null, and neither file is written. | Test (TC-038) |
| FR-024-AC-4 | `--output` on `run`, `check` and `analyze` is refused as an unknown option, exit 20, with nothing on stdout and no file written. | Test (TC-038) |
| FR-024-AC-5 | `--output` given twice, or without a path, refuses the command line naming `--output`, exit 20, and writes no file. | Test (TC-038) |
| FR-024-AC-6 | A compile that the checker refuses with `--output` exits 20 with the checker's code and writes no file. | Test (TC-038) |

## Dependencies

- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md): the outcome and
  exit contract.
- FSL FR-810 and FR-287: the output file and the driver's verbs and options.
