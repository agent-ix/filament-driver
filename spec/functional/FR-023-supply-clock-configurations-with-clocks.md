---
id: FR-023
title: "Supply clock configurations to compile and run with --clocks"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-640
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-283
    type: references
---
# FR-023: Supply clock configurations to compile and run with --clocks

## Description

`fsl compile` and `fsl run` SHALL take `--clocks <json-file>`, a file whose
whole content is the `clocks` object of
[FSL FR-640](ix://agent-ix/filament-spec-language/FR-640), and SHALL supply
it as the request's `clocks` member. A unit whose timestamped-event,
event-position or fixed-sample clauses read a clock the program does not
define then compiles and runs from a request file with no `clocks` member,
and gives the outcome the same request carrying the member gives.

The driver checks that the file is readable and one JSON object and hands it to
FSL as the request override `clocks` (`fsl_command::command::RequestOverrides`).
FSL reads it exactly as it reads a request's own member: every refusal FSL
FR-640 gives a malformed or misfitting member applies to the file's members
unchanged, and FSL itself refuses a request that carries `clocks` as well.

## Inputs

- `--clocks <json-file>` after the request operand, at most once, on `compile`
  and `run`.

## Behavior

- An unreadable `--clocks` file SHALL refuse with code `io-error`, cause
  `clocks-file-unreadable`, naming the path, with no stage reached.
- A `--clocks` file that is not UTF-8 JSON whose whole content is one object
  SHALL refuse with code `invalid-request`, cause `clocks-file-not-an-object`,
  naming `--clocks`, with no stage reached.
- A request document that carries a `clocks` member while `--clocks` is given
  is refused by FSL (FSL FR-640) `invalid-request` at stage `request`, naming
  `clocks` and `--clocks`; the driver adds no check of its own.
- A replay request document has no `clocks` member (FSL FR-640-AC-8): replay
  reads its clocks from the proving package. A driver reader of a replay
  request document SHALL refuse a `clocks` member there `invalid-request` at
  stage `request`. The driver has no replay verb yet, so this obligation has
  no code path and no test until that verb lands.
- A `--clocks` file over the request byte limit SHALL refuse
  `invalid-request` with cause `clocks-file-too-large`.
- A refusal records no stage because none was reached: the outcome document's
  `stage` is null, as for every other refusal before FSL runs.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-023-AC-1 | `fsl run <request> --clocks <file>` on a unit with a timestamped-event clause on clock `plant-ms` exits with the outcome and writes the document the same request carrying the file's object as its `clocks` member gives. | Test (TC-037) |
| FR-023-AC-2 | `fsl compile <request> --clocks <file>` on that unit writes the document the request carrying the member gives, and the same request with no clocks refuses `missing-clock-selection`. | Test (TC-037) |
| FR-023-AC-3 | An unreadable `--clocks` file refuses `io-error`, cause `clocks-file-unreadable`, stage null, exit 20 on `run` and `compile`. | Test (TC-037) |
| FR-023-AC-4 | A `--clocks` file that is an array, not JSON, or has trailing content refuses `invalid-request`, cause `clocks-file-not-an-object`, naming `--clocks`, stage null. | Test (TC-037) |
| FR-023-AC-5 | A request carrying `clocks` with `--clocks` refuses `invalid-request` (FSL's refusal), naming `clocks` and `--clocks`, exit 20, on `run` and `compile`. | Test (TC-037) |
| FR-023-AC-6 | A member FSL refuses (unknown `kind`) in the file refuses as the same member in the request does, and a repeated member name in the file reaches FSL and refuses. | Test (TC-037) |

## Dependencies

- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md): the outcome and
  exit contract.
- FSL FR-640 and FR-283: the `clocks` member and its replay and monitor use.
