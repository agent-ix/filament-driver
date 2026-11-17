---
id: FR-025
title: "Stream observations through fsl monitor"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-driver/FR-014
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-644
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-643
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-285
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-287
    type: depends_on
---
# FR-025: Stream observations through fsl monitor

## Description

`fsl monitor <request-file>` SHALL read one `native-monitor/1` request
([FSL FR-644](ix://agent-ix/filament-spec-language/FR-644)) and a stream of
observation records, and have FSL check the records against the request's
clauses one at a time. It writes each `filament.monitor-event/1` event to
standard output as the record that caused it is read, then the
`filament.monitor-summary/1` object, and exits with the summary's FR-285 exit
code. It no longer exits 21 with no output. Ticket: QSL-2313, the driver half of QSL-1977.

The verb is the command-line encoding of FSL's in-process entry
`fsl_command::command::monitor`. The request document carries every monitoring
setting, so the driver reads none from options.

### Deviation: no `--provider-config` or `--provider-limit` for `monitor` (QSL-2132)

FSL FR-287 and FR-006 give `monitor`, as a provider-routing verb, the
`--provider-config <file>` and `--provider-limit <name>=<value>` options, and
FSL FR-644 passes the started process providers to `command::monitor` by FSL
FR-635's rule. The requirement stands; this deviation only defers it.
Configured providers are not built yet, and the library entry takes no
providers parameter, so until QSL-2132 lands the driver refuses both options
with exit 20 naming QSL-2132. When QSL-2132 lands, this deviation lifts and
`monitor` takes both options as FR-006 states.

### Decision: a signal ends the stream as a cancelled run

`fsl monitor` runs as one signal-reporting session, as `fsl analyze` does
([FR-022](FR-022-analyze-a-native-analyze-request.md)). SIGINT and SIGTERM
cancel the handle passed to FSL with cause `requested`. The record stream is
read on its own thread, so the cancellation ends the stream at once even while a
read of standard input blocks; no further record is read, and the run is FSL's
cancelled monitor run (FSL FR-624, exit 22), its outcome document the last line
written. The signal is consumed, not re-raised.

## Inputs

- One request-file operand, resolved as `fsl run` resolves its operand; files
  the request names resolve against the request file's folder.
- `--input <file>`: the record stream, JSON Lines (FSL FR-644). Without it the
  stream is standard input. At most once.
- `--format json` or `--format text`.

## Outputs

- Machine mode: one canonically encoded JSON line per event, in the order FSL
  hands them over, then one line for the summary.
- Text mode: one line per event, `<record> <clause> <category> settled|pending`,
  then one line per clause, `<clause> <category> settled|pending`, and
  `records <n> refused <m>`.
- The exit code of the summary (FR-285). A request refused before any record is
  read writes the command-error outcome document (machine mode) or its
  diagnostics on stderr (text mode), and exits 20.
- If `command::monitor` fails after events were written, the driver appends the
  `filament-outcome/1` document for that failure (machine mode) or its
  diagnostics on stderr (text mode) and exits with the failure's FR-285 code,
  20 for a refusal; the events already written stay.

## Behavior

- The driver SHALL write each event to standard output and flush it before it
  reads the next record, so a consumer sees a status change while the stream is
  still open.
- The driver SHALL add no event, no member and no reordering to what FSL hands
  it; a malformed record is FSL's `refusal` event and the stream continues.
- A missing or extra operand, an unknown option, `--input` repeated or without
  a path, an `--input` file that cannot be opened, and, until QSL-2132 lands,
  `--provider-config` and `--provider-limit` SHALL refuse with exit 20 before
  any record is read.
- A request that carries a `records` member is refused by FSL: `run` reads that
  batch form, `monitor` reads the stream it is given.

## Acceptance Criteria

The fixture is a record `Reading { level: Int[0, 9] }` with the infinite-trace
clauses `Reaches` (`eventually holds(r.level = 2)`) and `Never`
(`always holds(r.level < 2)`) over a `finite` trace.

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-025-AC-1 | `fsl monitor` over `Reaches` and `Never` with levels 0, 1, 2, 1 then `{"end": true}` writes event lines then one summary line and exits 10, with a `settled` `violation` event for `Never` at record 2; over `Never` alone with levels 0, 1, 0 it exits 0. | Test (TC-039) |
| FR-025-AC-2 | The lines the binary writes equal, byte for byte, the canonical encodings of the events and summary `command::monitor` hands back in process for the same request and records, and the exit code equals the summary's. | Test (TC-039) |
| FR-025-AC-3 | A request naming a clause the program lacks, a request that is not JSON, and a missing operand each exit 20 before any record is read; the first writes only the `monitor` command-error document. | Test (TC-039) |
| FR-025-AC-4 | Until QSL-2132 lands, `--provider-config` and `--provider-limit` each refuse with exit 20 and name QSL-2132; in machine mode stdout holds the `monitor` command-error document, in text mode stdout is empty and the message is on stderr. | Test (TC-039) |
| FR-025-AC-5 | A record with an unknown member between good records gives one `refusal` event naming that record, the summary counts it, the stream continues and the output equals `command::monitor`'s. | Test (TC-039) |
| FR-025-AC-6 | With standard input held open, the event that settles `Never` is on standard output before the next record is sent or the input is closed. | Test (TC-039) |
| FR-025-AC-7 | `--input <file>` gives the same bytes and exit code as the same records on standard input; an unopenable `--input` file refuses with exit 20, and in machine mode stdout holds the `monitor` command-error document, in text mode stdout is empty. | Test (TC-039) |
| FR-025-AC-8 | In text mode, over AC-1's fixture and records, stdout holds one event line `<record> <clause> <category> settled|pending` per event, then one `<clause> <category> settled|pending` line per clause and the line `records <n> refused <m>`, and the exit is 10. | Test (TC-039) |
| FR-025-AC-9 | SIGTERM or SIGINT while standard input is still open ends the run with no further record read: the last line written is an outcome document with category `incomplete`, and the process exits 22. | Test (TC-039) |

## Dependencies

- FSL FR-644 owns the request, the record stream, the events and the summary;
  FR-643 owns the session they come from.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md): the outcome and
  exit contract.
