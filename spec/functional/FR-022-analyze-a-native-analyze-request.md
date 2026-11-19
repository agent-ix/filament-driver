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
  - target: ix://agent-ix/filament-spec-language/FR-1019
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-750
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
selects the output mode, `--provider-config` names the process providers
registered beside FSL's engines (FSL FR-287, FR-635; FSpec FR-561), and
`--provider-limit` sets the run's FSpec FR-462 limits (FSL FR-287, FR-964).

### Decision: the driver negotiates, FSL computes the candidates

FSL's `analyze` never negotiates (FSL FR-281 "Negotiated items"). FSL builds
the engine registry from `Engines::fsl()` and the providers the caller hands it,
computes each resolved item's candidate set from it, and asks the caller to
settle the item. The driver settles it through CG's one item entry,
`negotiate_item` (FSpec FR-711), over the FR-331 manifest of that same
registry, and never chooses among several candidates.
Because the registry is the one FSL builds from its own engine set, an engine
FSL registers is reachable through `fsl analyze` with no driver change.

### Decision: the driver always registers FSL's reference hybrid provider

FSL's own engine set `Engines::fsl()` holds no hybrid provider, so a library
caller chooses whether hybrid claims are decided (FSL FR-1019, FR-750-AC-3).
The `fsl` binary is the user's entry, and a user with no external provider
installed should still get a verdict on a hybrid claim from FSL itself. The
driver therefore SHALL always pass FSL's first-party reference hybrid provider
`Engine::hybrid_reference()` among the providers it hands
`analyze_with_providers`, beside the providers `--provider-config` starts, and
negotiate over that one registry: `Engines::fsl_with_reference_hybrid()` plus
the configured providers. `Engines::fsl()` and FSL FR-750-AC-3's
`HybridProviderAbsent` settlement are unchanged; they describe the library
route without the reference provider. A configured process provider whose
`hello` advertises (`temporal-satisfaction`, `unbounded`) with `dynamics`
`["hybrid"]` is a second hybrid provider: the driver starts it as a hybrid
process engine (below), an item that names its backend routes to it, and an
item that names no backend while it is configured settles `ambiguous-backend`,
naming both, as FSL FR-1019-AC-5 states, because the driver never chooses
among candidates. `fsl run` registers the reference provider the same way
whenever it settles requirement items (FR-002).

### Decision: configured providers are started by the driver

FSL's analyze entry reads no provider configuration (FSL FR-635 "One rule for
every provider-routing entry"). The driver reads `--provider-config`, starts
each entry's executable as a `filament.plugin-wire/v1` process, admits its
`hello` manifest with origin `Process`, and hands FSL one external engine per
provider through `fsl_command::command::analyze_with_providers`, with the run's
one limit set from `--provider-limit` as `ProviderOptions` (FSL FR-964). FSL
routes, negotiates and builds the FR-331 request for the items it settles on a
provider (FSL FR-635); the driver only transports it to that process under
the set's transport limits and maps the response back. The provider's `proved`
settles `proved` (certification `trusted`); its `refuted` carries its
`counterexamples` assignment, which FSL replays before the item can settle
`refuted`; any other result, or an exchange that fails, leaves the claim
undecided and settles `inconclusive`, never `proved`.

### Decision: a signal is reported, not re-raised

`fsl analyze` runs as one signal-reporting session. SIGINT and SIGTERM cancel
the run's handle with cause `requested`; FSL settles the open and later items
`incomplete` with `{"kind": "cancelled", "source": "requested"}` (FSL FR-624),
and the driver writes that document and exits 22. Unlike a configured `fsl
run` (FR-006-AC-4), the signal is consumed once the run ends rather than
re-raised, because FSL FR-624 gives the cancellation an outcome.

## Inputs

- One request-file operand, resolved as `fsl run` resolves its operand; files
  the request names resolve against the request file's folder (FSL FR-026).
- `--provider-config <path>`, at most once: FSpec FR-561's provider
  configuration in the driver's FR-014 shape, read before the request.
- `--format json` (machine mode) or `--format text` (the default).

## Outputs

- Machine mode: one `filament-outcome/1` document with `operation`
  `analyze` on stdout, encoded with ix-cli-kit's canonical JSON
  ([FR-002](FR-002-expose-driver-cli-and-outcome-contract.md)).
- Text mode: FSL FR-621's item lines on stdout, or FSL FR-624's one refusal
  line on stderr.
- The FR-285 exit code of the outcome's category.

## Behavior

- The driver SHALL call `fsl_command::command::analyze_with_providers` with
  the request path, the driver's negotiation, the providers
  (`Engine::hybrid_reference()` followed by the started providers, so only
  the reference provider without `--provider-config`) and the run's
  cancellation handle. The driver's negotiation SHALL read the manifest of
  `Engines::fsl_with_reference_hybrid()` plus the started providers, the same
  registry FSL computes the candidates from.
- For a configured provider whose admitted `hello` advertises
  (`temporal-satisfaction`, `unbounded`) with `dynamics` `["hybrid"]`, the
  driver SHALL make its engine with FSL's `Engine::hybrid_process(descriptor,
  tool, exchange)` (FSL FR-750), the exchange carrying FSL's FR-447 request
  frame bytes to the process and its FR-331 result frame bytes back, and SHALL
  put that descriptor in the negotiation manifest. FSL then settles an item
  routed to it by FR-750 "Dispatch": its `delta-complete` `unsat` covering the
  item's horizon settles `proved`, `bounded-solver`, `uncertified`; a
  `refuted` settles `refuted` only after exact replay. A provider advertising
  (`value-validity`, `unbounded`) keeps the external-engine route above.
- The request's `deadline_ms`, SIGINT and SIGTERM cancel that handle; a signal
  cancels it with cause `requested` and the driver writes the document FSL
  returns.
- A `--provider-config` file that cannot be read or does not match its shape,
  a provider that does not start, and a provider whose manifest advertises
  neither (`value-validity`, `unbounded`) nor (`temporal-satisfaction`,
  `unbounded`) with `dynamics` `["hybrid"]` SHALL refuse with exit 20 before
  any item runs; every started provider is shut down.
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
- The driver SHALL settle each item FSL hands it through CG's `negotiate_item`
  over the manifest of that registry, and apply no candidate rule
  of its own. The settlement is a pure function of that manifest and the
  candidate outcome FSL computed. The table below states the rows that entry
  settles by candidate outcome; an unbounded extent on a bounded-only candidate
  settles `requires-bound`, or `unsupported` with `unsupported_projection`/`unbounded-extent`
  when no finite bound is available.

  | Candidate outcome | Disposition | Diagnostic |
  | --- | --- | --- |
  | exactly one candidate | `supported` on that candidate | none |
  | no candidate | `unsupported` | `unsupported_projection`/`unsupported-requested-capability`, payload the item's kind |
  | several candidates, no backend named | `invalid-request` | `invalid_capability`/`ambiguous-backend` |
  | several candidates, a backend named | `invalid-request` | `invalid_capability`/`inconsistent-candidates`, payload the named backend |
  | a named backend the registry does not hold | `invalid-request` | `invalid_capability`/`unknown-backend`, payload the backend |

- When an operand is missing or extra, an option is none of `--format`,
  `--provider-config` and `--provider-limit`, or `--provider-config` has no
  path or is given twice, the driver SHALL refuse with exit 20 before any file
  is read, naming the usage.
- The driver SHALL read and refuse the `--provider-config` document and every
  `--provider-limit` as FR-006 states, before any stage runs and before any
  plugin starts.
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
| FR-022-AC-5 | A malformed `native-analyze/1` document gives, in machine mode, a document with `operation` `analyze`, category `refusal`, empty `items` and one `invalid-request` diagnostic, and in text mode one stderr line starting `fsl analyze: invalid-request:`; both exit 20. A program the checker refuses gives a document whose `last_stage` is the checker stage FSL names (`S3`), empty `items` and that refusal's code, and the text line `fsl analyze: <code>: `; a snapshot naming a population the model does not declare gives the binding refusal's code with empty `items`; each exits 20. A missing operand, an unknown option, or a `--provider-config` with no path or given twice refuses with exit 20 naming the usage, before any file is read. | Test (TC-036) |
| FR-022-AC-6 | The machine-mode stdout for the AC-2 request is byte-equal to the canonical encoding of the document `fsl_command::command::analyze` returns in process for the same request, followed by one newline. | Test (TC-036) |
| FR-022-AC-7 | In text mode the AC-2 request prints `0 BelowThree refuted` at the start of its first line, `violation` as its last line, and exits 10; an item naming an unregistered backend prints `0 Bounded invalid-request invalid_capability/unknown-backend`. | Test (TC-036) |
| FR-022-AC-8 | Each settlement row of the table holds for the driver's negotiation over a candidate outcome of that shape, except the several-candidates-with-a-named-backend row, which FSL's candidate computation does not produce; the `registry` member of an outcome lists the engines FSL registers (`en-1` among them), and the `Bounded` item's candidates are `["en-1"]`. | Test (TC-036) |
| FR-022-AC-9 | With `--provider-config` starting a process provider whose `hello` advertises (`value-validity`, `unbounded`), an item naming its backend is sent to it as one FR-331 request at the item's request index and settles `proved` with certification `trusted` when the provider answers `proved`, and `inconclusive` when it answers anything else; the document's `registry` lists the provider and `en-1`; another item is unchanged by it. A provider advertising only `bounded`, an unreadable configuration and a malformed one each refuse with exit 20. | Test (TC-036) |
| FR-022-AC-10 | SIGTERM or SIGINT while an item runs gives a document whose open and later items are `incomplete` with cause `{"kind": "cancelled", "source": "requested"}`, and exit 22. | Test (TC-036) |
| FR-022-AC-11 | With no `--provider-config`, `fsl analyze` over FSL's `examples/hybrid-braking/slow-enough.analyze.json` with `--format json` writes one item settled `proved` with certification `certified` and backend `fsl-hybrid-reference`, category `success`, exit 0, and over `too-slow.analyze.json` one item settled `refuted` with backend `fsl-hybrid-reference`, category `violation`, exit 10; neither writes `HybridProviderAbsent`, and the document's `registry` lists `fsl-hybrid-reference` and `en-1`. The process-provider half depends on FSL's `Engine::hybrid_process` (FSL FR-750): with `--provider-config` starting a fixture process provider whose `hello` advertises (`temporal-satisfaction`, `unbounded`) with `dynamics` `["hybrid"]` and that answers `delta-complete` `unsat` covering horizon 10, the slow-enough item rewritten to name that provider's backend is sent to it as one FR-447 request and settles `proved`, `bounded-solver`, `uncertified`, naming the fixture backend, exit 0, with no `ambiguous-backend`; the item naming `fsl-hybrid-reference` settles `proved`, `certified`, exit 0; the item naming neither settles `invalid-request` with `invalid_capability`/`ambiguous-backend` naming both backends, exit 20 (FSL FR-1019-AC-5); a provider whose `hello` advertises neither (`value-validity`, `unbounded`) nor (`temporal-satisfaction`, `unbounded`) with `dynamics` `["hybrid"]` refuses with exit 20 before any item runs. | Test (TC-047) |

## Dependencies

- FSL FR-620 owns the request document and its reading; FSL FR-621 owns the
  outcome document, its exit code and its text rendering; FSL FR-624 owns the
  refusal table.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) owns the CLI's
  stream and exit discipline, which this verb follows.
- FSL FR-1019 defines the reference hybrid provider, `Engine::hybrid_reference()`
  and `Engines::fsl_with_reference_hybrid()`, and the hybrid-braking examples;
  FSL FR-750 defines hybrid routing and the `HybridProviderAbsent` settlement of
  the library route without it.
- The filament-examples suite records each `hybrid-systems-*` case's verdict
  through the `fsl` binary (FR-021). Its expected verdicts change with this
  decision from `HybridProviderAbsent` to the reference provider's verdicts;
  updating those records is a code change made with the driver change, not a
  requirement of this FR.
- [TC-047](../test-cases/TC-047-decide-hybrid-claims-with-the-reference-provider.md)
  verifies AC-11.
