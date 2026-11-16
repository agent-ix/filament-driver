---
id: FR-022
title: "Analyze a native-analyze/1 request from the command line"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-driver/FR-014
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-620
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-621
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-624
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-281
    type: depends_on
---
# FR-022: Analyze a native-analyze/1 request from the command line

## Description

`fsl analyze <request-file>` SHALL read one `native-analyze/1` request
document ([FSL FR-620](ix://agent-ix/filament-spec-language/FR-620)), have
FSL analyze its items, and write the outcome FSL writes for it: the
`analyze` document of
[FSL FR-621](ix://agent-ix/filament-spec-language/FR-621), or the
whole-request refusal of
[FSL FR-624](ix://agent-ix/filament-spec-language/FR-624). The process exits
with that outcome's FR-285 exit code.

The verb is the command-line encoding of FSL's in-process entry
`fsl_command::command::analyze`, as `fsl run` is the encoding of a
`native-run/1` request. The request document carries every analysis
setting: the program, models, supplied documents, subject, items with their
engines, fairness, reductions and limits, the shared limits and the deadline.
The driver therefore reads no analysis setting from options; `--format`
selects the output mode, and `--provider-config` is refused until the
deviation below is closed.

### Decision: the driver negotiates, FSL computes the candidates

FSL's `analyze` never negotiates (FSL FR-281 "Negotiated items"). FSL builds
the engine registry from `Engines::fsl()`, computes each resolved item's
candidate set from it, and asks the caller to settle the item. The driver
settles it under FSpec FR-290's candidate table, read through
`fsl_route::settle_candidates`, and never chooses among several candidates.
Because the registry is the one FSL builds from its own engine set, an engine
FSL registers is reachable through `fsl analyze` with no driver change.

### Deviation: settlement through `settle_candidates`, not CG `negotiate_*`

FSL FR-281 and FSpec AD-016 name CG's `negotiate_*` as the single
negotiation point. CG's `negotiate_backend_provider` settles a candidate only
through a `BackendKind` arm, and its arms are `Kani` and `Process`: a linked
FSL engine such as `en-1` has no arm and settles `unknown-backend`, and each
`RequestItem` requires a Kani-specific `KaniForm` an analyze item does not
have. Until CG offers a negotiation arm for FSL's linked analysis engines
(QSL-2286), the driver settles through `fsl_route::settle_candidates`. That settlement reads
the candidate count only, so FSpec FR-290's extent rows (`requires-bound`, or
`unsupported` with `unbounded-extent` for an unbounded extent on a
bounded-only candidate) are not applied by the driver. Such an item reaches
FSL as `supported` on its one candidate, and FSL refuses it with
`invalid_capability`/`inconsistent-candidates` (FR-281 rule 4), so the user
sees that refusal rather than `requires-bound` or `unbounded-extent`.

### Deviation: no `--provider-config` for `analyze` (QSL-2283)

FSL FR-287 gives `analyze` the `--provider-config <file>` option, so that
configured providers register before negotiation (FR-635).
`fsl_command::command::analyze` builds its registry from `Engines::fsl()` and
takes no caller registry, so the driver cannot register a configured
provider. QSL-2283 adds a language analyze entry that takes the caller's
registry; until then the driver refuses the option with exit 20 and a
message naming the missing registry.

## Inputs

- One request-file operand, resolved as `fsl run` resolves its operand; files
  the request names resolve against the request file's folder (FSL FR-026).
- `--format json` (machine mode) or `--format text` (the default).

## Outputs

- Machine mode: one `filament-outcome/1` document with `operation`
  `analyze` on stdout, encoded with ix-cli-kit's canonical JSON
  ([FR-002](FR-002-expose-driver-cli-and-outcome-contract.md)).
- Text mode: FSL FR-621's item lines on stdout, or FSL FR-624's one refusal
  line on stderr.
- The FR-285 exit code of the outcome's category.

## Behavior

- The driver SHALL call `fsl_command::command::analyze` with the request
  path, the driver's negotiation and a fresh cancellation handle.
- Only the request's `deadline_ms` cancels that handle. SIGINT and SIGTERM are
  not cancellation triggers for `fsl analyze` yet (QSL-2284), so FSL FR-624's
  `cancelled` cause `requested` cannot be reached from the CLI; a signal ends
  the process by its default disposition and writes no document.
- For an accepted request the driver SHALL write FSL's document unchanged in
  machine mode, and exit with FSL's exit code. When the document's `category`
  is not the category that exit code names, the driver SHALL exit with the
  internal-failure code and a diagnostic naming both, rather than guess.
- For a whole-request refusal the driver SHALL write the refusal's
  `analyze` outcome document in machine mode, and in text mode the line
  `fsl analyze: <code>: <message>` on stderr, and exit with the refusal's
  category code.
- In text mode the driver SHALL print, per item in request order,
  `<request_index> <claim_name> <label>`, where `label` is `invalid-request`
  for an `invalid-request` disposition record (one with no `result`, or whose
  cause code is `invalid-request`) and the record's `result` otherwise, and `claim_name` is `-`
  when the item has no name, followed by the first present of the record's
  diagnostic `code/cause`, its cause and its proof-basis type; then one line
  with the document category.
- The driver SHALL settle each item FSL hands it as the table below states.
  The settlement is a pure function of the candidate outcome FSL computed.

  | Candidate outcome | Disposition | Diagnostic |
  | --- | --- | --- |
  | exactly one candidate | `supported` on that candidate | none |
  | no candidate | `unsupported` | `unsupported_projection`/`unsupported-requested-capability`, payload the item's kind |
  | several candidates, no backend named | `invalid-request` | `invalid_capability`/`ambiguous-backend` |
  | several candidates, a backend named | `invalid-request` | `invalid_capability`/`inconsistent-candidates`, payload the named backend |
  | a named backend the registry does not hold | `invalid-request` | `invalid_capability`/`unknown-backend`, payload the backend |

- A missing or extra operand, or an option other than `--format`, SHALL
  refuse with exit 20 before any file is read, naming the usage;
  `--provider-config` is refused naming the missing caller registry.
- No `fsl analyze` invocation SHALL exit without writing a document to stdout
  in machine mode or a line to stdout or stderr in text mode.

## Acceptance Criteria

The Counter fixture is FSL FR-620's: domain package `test/counter`, the
`inc` contract, the clauses `Bounded` (`always holds(c.value <= 3)`) and
`BelowThree` (`always holds(c.value <= 2)`), and one initialization snapshot
with counter `a` at 0.

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-022-AC-1 | `fsl analyze` over the Counter request with the item `{"claim": "Bounded"}` and `--format json` writes one document with `operation` `analyze`, category `success`, one item whose record is `proved`, and `aggregate.status` `complete`, and exits 0. | Test (TC-036) |
| FR-022-AC-2 | The item `{"claim": "BelowThree"}` gives an item whose record is `refuted`, with a `counterexample` entry and `replay` `{"target": {"state_clause": Q}, "reproduced": true}`, `Q` ending in `BelowThree`, category `violation`, and exit 10. | Test (TC-036) |
| FR-022-AC-3 | A request whose item sets `temporal.limits.max_states` to 1 gives an `incomplete` record whose cause names `max_states`, with no `counterexample`, and exit 22; a request with `deadline_ms` 0 gives an `incomplete` record whose cause is `{"kind": "cancelled", "source": "deadline"}`, and exit 22. | Test (TC-036) |
| FR-022-AC-4 | An item naming a backend the registry does not hold settles with category `refusal` and the record diagnostic `invalid_capability`/`unknown-backend` naming the backend, exit 20; an item over the Counter fixture naming backend `en-6`, which lists no temporal-satisfaction form, settles `unsupported` with `unsupported_projection`/`unsupported-requested-capability`, exit 21 (the item kind is incidental: EN-1 serves derived deadlock-freedom since QSL-2122, and a state-graph claim over a timed subject stays unsupported until QSL-2305); neither writes empty output. | Test (TC-036) |
| FR-022-AC-5 | A malformed `native-analyze/1` document gives, in machine mode, a document with `operation` `analyze`, category `refusal`, empty `items` and one `invalid-request` diagnostic, and in text mode one stderr line starting `fsl analyze: invalid-request:`; both exit 20. A program the checker refuses gives a document whose `last_stage` is the checker stage FSL names (`S3`), empty `items` and that refusal's code, and the text line `fsl analyze: <code>: `; a snapshot naming a population the model does not declare gives the binding refusal's code with empty `items`; each exits 20. A missing operand, an unknown option, or `--provider-config` refuses with exit 20 naming the usage or the missing caller registry, before any file is read. | Test (TC-036) |
| FR-022-AC-6 | The machine-mode stdout for the AC-2 request is byte-equal to the canonical encoding of the document `fsl_command::command::analyze` returns in process for the same request, followed by one newline. | Test (TC-036) |
| FR-022-AC-7 | In text mode the AC-2 request prints `0 BelowThree refuted` at the start of its first line, `violation` as its last line, and exits 10; an item naming an unregistered backend prints `0 Bounded invalid-request invalid_capability/unknown-backend`. | Test (TC-036) |
| FR-022-AC-8 | Each settlement row of the table holds for the driver's negotiation over a candidate outcome of that shape, except the several-candidates-with-a-named-backend row, which FSL's candidate computation does not produce; the `registry` member of an outcome lists the engines FSL registers (`en-1` among them), and the `Bounded` item's candidates are `["en-1"]`. | Test (TC-036) |

## Dependencies

- FSL FR-620 owns the request document and its reading; FSL FR-621 owns the
  outcome document, its exit code and its text rendering; FSL FR-624 owns the
  refusal table.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) owns the CLI's
  stream and exit discipline, which this verb follows.
