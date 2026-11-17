---
id: FR-006
title: "Route negotiated items to process providers"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/filament-driver/FR-013
    type: depends_on
  - target: ix://agent-ix/filament-driver/FR-014
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-331
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-290
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-069
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-287
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-964
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-561
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-777
    type: depends_on
---
# FR-006: Route negotiated items to process providers

## Description

When backend negotiation selects an out-of-process provider, the driver SHALL
dispatch only that provider's negotiated items through its driver-owned
process-provider seam, using the executable path named by the caller. The
`fsl-cli` composition SHALL implement that seam with `filament-plugin-host`.
The driver SHALL feed returned typed provider results into the same proof
settlement path used for first-party providers.

## Inputs

- The caller-named plugin executable path.
- Explicit process-provider configuration supplied to the `fsl-cli`
  composition by `--provider-config <path>`: FSpec FR-561's
  `filament.provider-config/v1` document, which lists plugins only, each
  entry an executable `path` and optional `arguments`.
- The run's one provider limit set, resolved from `--provider-limit
  <name>=<value>` options with FSL FR-287's names and defaults and carried
  as `ProviderOptions.provider_limits` (FSL FR-964): `deadline_ms` and the
  transport limits `max_frame_bytes`, `max_total_bytes`, `grace_ms`,
  `max_stderr_bytes`, `max_memory_bytes` and `max_cpu_ms`.
- The admitted provider manifest, negotiated provider identity and routed
  request items.
- The emitted checked-package bytes, frame and session-byte limits, stderr
  retention limit, and cancellation-grace interval.
- The caller-owned FSL cancellation handle for the active run.
- The process-provider seam supplied by the `fsl-cli` composition.

## Outputs

- The typed provider results returned for the routed items, consumed by the
  driver's proof settlement path.
- Bounded plugin process-tree cleanup before normal CLI termination or exit
  after handled SIGINT/SIGTERM on platforms covered by this requirement.

## Behavior

- The driver SHALL expose and invoke a process-provider seam that receives
  the named executable, admitted provider identity, one `EmittedPackage` value
  (including its bytes), caller-supplied deadline,
  the driver's registered-candidate manifest snapshot, negotiated items and
  caller limits. The driver package SHALL NOT depend on `filament-plugin-host`.
- The `fsl-cli` composition SHALL implement the process-provider seam with
  `filament-plugin-host`, using FSL ADR-029 PL-1 through PL-7 as the authoritative
  lifecycle and failure contract. It SHALL use FR-331's provider envelope and
  FSL-owned typed terminal values; it SHALL NOT define a second wire format or
  infer terminal categories from provider-specific evidence or diagnostics.
- Every provider-routing verb of `fsl-cli` (`analyze`, `prove`, `monitor`,
  `run`, `replay`, `generate`; FSL FR-287, FSpec FR-561) SHALL accept
  `--provider-config <path>` and `--provider-limit <name>=<value>`. `run`
  and `analyze` meet this requirement now. `monitor` refuses both options
  under the deviation FR-025 (QSL-2132) states, which lifts when its ticket
  lands. A verb that routes no items SHALL refuse both options as
  unknown, exit 20 (FSpec FR-301).
- The configuration file SHALL be FSpec FR-561's
  `filament.provider-config/v1` document: a JSON object with exactly
  `format` (the string `filament.provider-config/v1`) and `providers` (an
  array, empty for no plugin). Each entry SHALL have a required non-blank
  `path`, the plugin executable, resolved against the directory of the
  configuration document when relative, and an optional `arguments` array of
  strings passed to the plugin in order, `[]` when absent. The document lists
  plugins only: it SHALL hold no deadline and no limit, so one configuration
  serves runs of any size, and an entry member such as `deadline_ms` or
  `limits` is an unknown member.
- The `fsl-cli` composition SHALL read the configuration document before any
  stage runs and before any plugin starts, and SHALL refuse as FSpec FR-561
  states: a missing or unreadable file names the path; a document that is
  not JSON, a missing `format` or `providers`, an unknown member at any
  object level, an entry with no or a blank `path`, or `arguments` that are
  not an array of strings refuses `invalid-request` naming the file and the
  member; another `format` refuses `unknown_wire`/`unsupported-wire`. Each
  refusal exits 20 with empty stdout in text and machine mode alike (FSpec
  FR-561), with the diagnostic on stderr, and starts no provider.
- `--provider-limit` SHALL set one limit of the run's limit set by FSL
  FR-287's names, units and defaults, may be given once per name, and is
  accepted with or without `--provider-config`. The composition SHALL read
  every `--provider-limit` before any stage runs and before any plugin
  starts. An argument with no `=`, an unknown name, a name given twice, or a
  value that is empty, zero, signed, has a leading zero, a non-digit or a
  unit suffix, or exceeds 9,007,199,254,740,991 SHALL refuse
  `invalid-request` naming `--provider-limit` and the argument, exit 20 with
  empty stdout in text and machine mode alike (FSL FR-287), and start no
  plugin.
- The limit set is one per run (FSpec FR-777): every provider of the run,
  linked or process, works under the same set. The composition SHALL read no
  provider limit from the configuration document, a provider's registration,
  an environment variable or another file; a limit the caller does not give
  takes FSL FR-287's default.
- For each configured entry, the `fsl-cli` composition SHALL start the named
  executable and admit its hello before negotiation. The identity in that
  once-admitted hello descriptor SHALL be the sole identity used to register
  and route that process provider. The composition SHALL pass the entry's
  resolved `path` and `arguments` unchanged, and the run's limit set, to the
  process-provider seam, and SHALL hold each plugin session to the set's
  transport limits.
- The `fsl-cli` composition SHALL start and admit every configured entry
  before beginning negotiation.
- If distinct configured process sessions return the same identity from their
  admitted hellos, the `fsl-cli` composition SHALL stop before negotiation and
  close all those sessions. It SHALL preserve any FSL registration refusal
  unchanged; if FSL reports identical repeated admission as idempotent, the
  composition SHALL return the existing CLI `InvalidRequest` input refusal
  rather than route an ambiguous identity.
- If a configured provider fails to start or its hello fails before all
  providers are prepared, the `fsl-cli` composition SHALL begin no
  negotiation, close every earlier active session, and preserve the original
  typed host failure.
- The `fsl-cli` composition SHALL NOT read process-provider configuration
  from ambient environment state, a default file, or a search path. When
  `--provider-config` is omitted, the run SHALL register no process providers;
  when the file contains an empty `providers` array, it SHALL also register no
  process providers.
- The `fsl-cli` adapter SHALL start the host session with the caller-named
  executable path, pass the once-admitted descriptor returned by the host hello
  admitter to the driver's existing registry without re-admitting it or
  building a second registry, and send only the items negotiated to that
  provider.
- The driver registry record SHALL retain one lossless FR-331 manifest DTO for
  each registered candidate: process DTOs are retained at hello admission and
  linked-provider DTOs are retained when their descriptors are built. The
  driver SHALL compose the request's manifest snapshot from those records for
  the complete registered candidate set. It SHALL NOT reconstruct the
  snapshot from FSL registry descriptors, which do not retain optional FR-331
  fields.
- The `fsl-cli` adapter SHALL serialize the supplied manifest snapshot
  without reconstructing it or adding members.
- The `fsl-cli` adapter SHALL build each request's `package` from the emitted
  package's exact bytes. The request carries no `package_id`.
- The `fsl-cli` adapter SHALL copy the run limit set's `deadline_ms` (the
  `--provider-limit deadline_ms` value, or FSL FR-287's default 600000) into
  the process-provider request and copy that same value unchanged into the
  FR-331 request. It SHALL NOT derive a deadline from the configuration
  document, ambient state, or a provider's registration. It SHALL NOT add
  request-level `domains` or `limits`.
- The `fsl-cli` adapter SHALL serialize each item's request index, claim
  locus, kind, extent/bounds and candidate set from the corresponding retained
  FSL `RequirementItem`/`CandidateOutcome`, using the same request data used to
  build typed CG negotiation. CG's routed output SHALL supply the selected
  backend. The adapter SHALL NOT treat the routed output as the source of the
  FSL-owned item fields.
- The `fsl-cli` adapter SHALL place temporal proof bounds under
  `items[].temporal.subject.proof_bounds`. The current CG routed output does
  not retain all these fields; their projection through production composition
  remains incomplete. CG negotiation owns the PV-4 domain-kind
  comparison defined by FSpec FR-290-AC-13. The adapter SHALL NOT implement or
  restate that comparison.
  FR-331 specifies the proof-bound `kind`; QSL-654 owns FSL emitter support for
  writing that member, which remains pending. Other request projection fields
  are exercised by the selected-provider composition test.
- The driver SHALL pass a validated plugin result to the shared settlement
  path only when FSL supplies a terminal mapping for it. A result without a
  currently supported terminal mapping SHALL remain typed, unmapped provider
  data; it SHALL NOT be assigned a fabricated terminal.
- The driver's shared settlement path SHALL map a FR-331 `declined` result to
  FSL `TerminalValue::Declined { code }` using its typed code, and SHALL keep
  any response without a specified FSL terminal mapping typed and unmapped.
- The driver's shared settlement path SHALL put the FR-272 cause, diagnostic
  code and available locus and message into a FSL `OutcomeDiagnostic` in the
  outcome document. It SHALL NOT parse free-form message or `detail` text to
  choose a terminal; terminal mappings must be defined by FSL.
- Given an FR-331 response for non-canonical package spelling, the `fsl-cli`
  adapter SHALL preserve each routed item identity and the typed FR-272
  diagnostic (`invalid_runtime_input` code with `invalid-value` cause).
- For either preserved response, the driver's shared settlement path SHALL map
  each failed item to FSL `TerminalValue::Failed` and carry the diagnostic
  separately in the outcome document. Results for unrelated providers SHALL
  remain unchanged.
- The driver SHALL synthesize provider failures only for host/protocol failures
  defined by ADR-029 PL-7; it SHALL NOT attribute a request-body refusal to
  FSL, which has no process request-body reader.
- The `fsl-cli` adapter SHALL request graceful shutdown of every active
  plugin session before normal command termination. On Linux and macOS, if
  graceful shutdown exceeds the run limit set's `grace_ms`, it SHALL terminate
  the process tree and await cleanup within the one-second bound in FR-013.
  On Windows, the merged host's forced cleanup path terminates the Job Object
  through `terminate_tree`/`terminate_job`; native Windows evidence remains
  planned. The separate Job Object kill-on-close mechanism is planned under
  IR-674 and specified by FR-013-AC-14.
- When `fsl-cli` on Linux or macOS receives SIGINT or SIGTERM while a plugin
  session is starting, running or shutting down, it SHALL request cancellation
  through the caller-owned FSL handle, allow the run limit set's `grace_ms` to
  expire or the session to settle, terminate any remaining
  plugin process tree, and await cleanup within the one-second bound in
  FR-013 before exiting. The CLI SHALL restore the default signal disposition
  and re-raise the received signal; this requirement makes no claim about a
  FSL cancellation cause for the signal. If no plugin session is active, the
  CLI SHALL retain the platform's default signal disposition. This guarantee
  does not cover an uncatchable host kill on Unix or a plugin descendant that
  deliberately leaves the host's process group.
- Windows console control events, including Ctrl+C and console close, retain
  the platform's default disposition; this requirement makes no plugin cleanup
  guarantee for those events. Windows process-tree cleanup, if supported by
  the host, is specified only by [FR-013-AC-14](./FR-013-host-process-plugins.md).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | When negotiation selects a process provider, the driver invokes its process-provider seam with the configuration entry associated with that provider's once-admitted hello identity, one `EmittedPackage` value, the run limit set's `deadline_ms` and transport limits, the registered-candidate manifest snapshot and exactly that provider's negotiated items; `fsl-cli` implements the seam with `filament-plugin-host` and passes the once-admitted hello descriptor to the driver's existing registry. | Partial (TC-013, TC-028; production dispatch and typed-result retention are tested; complete field projection and later-start failure cleanup remain partial) |
| FR-006-AC-2 | Validated typed results returned through the process-provider seam enter the same driver proof settlement path as first-party provider results when FSL defines their terminal mapping; results without a current FSL mapping remain typed and unmapped, while unrelated routed items retain their own results. | Planned (TC-013; no end-to-end claim from an injected boundary; IR-672, IR-677, QSL-490 and QSL-655) |
| FR-006-AC-3 | Before normal CLI command termination, the adapter requests graceful shutdown of every active plugin session; on Linux and macOS, if the run limit set's `grace_ms` expires first, the process tree terminates and cleanup completes within one second. The merged Windows host force-terminates the Job Object through `terminate_tree`/`terminate_job`; native Windows evidence remains planned. Kill-on-close is separately planned under IR-674. | Partial (TC-013, TC-028; normal host shutdown and production signal cleanup are covered, but the one-second forced-cleanup guarantee lacks a production binary assertion) |
| FR-006-AC-4 | On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned FSL handle, waits for settlement or the run limit set's `grace_ms` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal. No FSL cancellation cause is claimed; with no active session, the platform's default signal disposition is retained. | Test |
| FR-006-AC-5 | The driver composes the request manifest snapshot from the lossless FR-331 DTO retained in each registered-candidate registry record, covering linked and process providers; the `fsl-cli` adapter serializes that snapshot without reconstructing or adding members. | Planned (TC-013) |
| FR-006-AC-6 | The `fsl-cli` adapter serializes `package` as the unpadded base64url encoding of the emitted package's exact bytes. | Planned (TC-013) |
| FR-006-AC-7 | Retired: the request carries no `package_id`; the `fsl-cli` adapter serializes the package bytes from the `EmittedPackage` value. | Not applicable |
| FR-006-AC-8 | The `fsl-cli` adapter serializes `deadline_ms` equal to the run limit set's `deadline_ms`: the `--provider-limit deadline_ms=<value>` value unchanged, or 600000 when the option is not given; two configured providers of one run receive equal `deadline_ms`. | Test (TC-013, TC-028) |
| FR-006-AC-9 | The `fsl-cli` adapter serializes a request without top-level `domains` or `limits` members. | Planned (TC-013) |
| FR-006-AC-10 | The `fsl-cli` adapter serializes each item's request index, claim locus and kind from the corresponding retained FSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation. | Partial (TC-013; production composition checks index, claim and kind; full field coverage remains incomplete) |
| FR-006-AC-11 | The `fsl-cli` adapter serializes each item's selected backend from CG's routed output and its candidate set from the corresponding retained FSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation. | Test (TC-013) |
| FR-006-AC-12 | The `fsl-cli` adapter serializes each item's extent and bounds from the corresponding retained FSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation. | Partial (TC-013; extent is asserted; proof-bound kind emission remains pending FSL support) |
| FR-006-AC-13 | The `fsl-cli` adapter serializes each item's temporal proof bounds under `items[].temporal.subject.proof_bounds`. | Partial (TC-013; production projection exists, but temporal-bound output is not yet asserted) |
| FR-006-AC-14 | The driver's shared settlement path preserves a plugin `declined` result's typed code in FSL `TerminalValue::Declined { code }`. | Planned (TC-013) |
| FR-006-AC-15 | A validated result without a specified FSL terminal mapping remains typed and unmapped in the driver's shared settlement path. | Planned (TC-013; IR-677 and FSL result-mapping APIs) |
| FR-006-AC-16 | The driver's shared settlement path carries the FR-272 cause, diagnostic code and available locus and message in a FSL `OutcomeDiagnostic`; free-form message or `detail` text does not select a terminal. | Planned (TC-013) |
| FR-006-AC-17 | Retired: the package-id mismatch case no longer exists (the request carries no `package_id`); the id is not reused. | Not applicable |
| FR-006-AC-18 | Given an FR-331 response for non-canonical package spelling, the `fsl-cli` adapter preserves each routed item identity and the FR-272 diagnostic with code `invalid_runtime_input` and cause `invalid-value`. | Planned (TC-013) |
| FR-006-AC-19 | The driver synthesizes failed provider results only for host/protocol failures defined by ADR-029 PL-7; it does not attribute a process request-body refusal to FSL. | Planned (TC-013) |
| FR-006-AC-20 | For a preserved package-mismatch response, the driver's shared settlement path maps each failed routed item to FSL `TerminalValue::Failed`. | Planned (TC-013; IR-677 and FSL settlement API) |
| FR-006-AC-21 | For a preserved non-canonical-package response, the driver's shared settlement path maps each failed routed item to FSL `TerminalValue::Failed`. | Planned (TC-013; IR-677 and FSL settlement API) |
| FR-006-AC-22 | For package-mismatch and non-canonical-package responses, the driver's shared settlement path carries each typed FR-272 diagnostic separately in FSL `OutcomeDiagnostic`. | Planned (TC-013; IR-677 and FSL settlement API) |
| FR-006-AC-23 | Settling a failed process-provider response leaves unrelated provider results unchanged. | Planned (TC-013; IR-677 and FSL settlement API) |
| FR-006-AC-24 | The `--provider-config` file accepts only FSpec FR-561's `filament.provider-config/v1` shape (`format`, `providers`, entries with `path` and optional `arguments`); a missing or unreadable file refuses naming the path; a non-JSON document, a missing `format` or `providers`, an unknown member at any object level (including an entry member `deadline_ms` or `limits`), an entry with no or a blank `path`, and `arguments` that are not strings refuse `invalid-request` naming the file and the member; `format` `filament.provider-config/v2` refuses `unknown_wire`/`unsupported-wire`; each exits 20 with empty stdout under both `--format text` and `--format json` and starts no provider. A relative `path` resolves against the document's directory. | Test (TC-028) |
| FR-006-AC-25 | The `fsl-cli` composition starts and admits every configured provider before negotiation, and uses each admitted hello descriptor as the sole provider identity for subsequent selection and dispatch. | Partial (TC-028; selected-provider production dispatch is tested; later-start/hello failure cleanup remains untested) |
| FR-006-AC-26 | If distinct configured sessions return the same admitted identity, the composition begins no negotiation and closes all sessions; it preserves a FSL registration refusal, or returns the existing CLI `InvalidRequest` input refusal when FSL treats repeated identical admission as idempotent. | Test (TC-028; idempotent duplicate admission path) |
| FR-006-AC-27 | If a configured provider fails to start or admit its hello, the composition begins no negotiation, closes all earlier active sessions, and preserves the original typed host failure. | Planned (TC-028) |
| FR-006-AC-28 | `fsl run --provider-config providers.json` with no `--provider-limit` sends the fixture plugin a request whose `deadline_ms` is 600000 and holds it to a 67108864-byte frame limit; with `--provider-limit deadline_ms=250 --provider-limit max_frame_bytes=1048576` the request carries `deadline_ms` 250 and a 1048577-byte result frame settles the item `failed` with the frame-length tool failure naming limit 1048576; `--provider-limit` is accepted without `--provider-config`. | Test (TC-041) |
| FR-006-AC-29 | `fsl run` with `--provider-limit max_frames=4`, `deadline_ms`, `deadline_ms=0`, `deadline_ms=10s`, `deadline_ms=010`, `max_total_bytes=9007199254740992`, or `deadline_ms=5` given twice each refuses `invalid-request` naming `--provider-limit` and the argument, exits 20 with empty stdout under both `--format text` and `--format json`, and starts no plugin; `fsl parse --provider-limit deadline_ms=5` and `fsl parse --provider-config providers.json` refuse the option as unknown, exit 20. | Test (TC-041) |

## Dependencies

- FSL ADR-029 PL-1 through PL-7 define the plugin protocol, host lifecycle,
  manifest admission and typed failure behavior; PV-1 through PV-4 define
  provider registration and negotiation. [FR-013](./FR-013-host-process-plugins.md)
  specifies the separate host implementation. FR-014 specifies its
  composition by the `fsl-cli` package.
- [FR-331](ix://agent-ix/filament-specification/FR-331) owns the
  `filament.backend-provider/v1` envelope; FSpec FR-290-AC-13 owns the PV-4
  domain-kind rule; FSL owns typed terminal values and
  `filament-outcome/1` serialization.
- QSL-654 specifies the proof-bound `kind` member; FSL emitter support for
  writing it remains pending. FSpec FR-290-AC-13 defines the PV-4 domain-kind
  comparison rule. CG's separate `bounds[].kind` work is
  outside this request adapter.
- QSL-490 and QSL-655 are prerequisites for a plugin `proved` result's trusted
  `ProofBasis` and for replay settlement of plugin counterexample candidates.
  Until those APIs land, these result forms are not mapped to terminals.
- FR-005 defines the first-party `Provider` interface and shared manifest
  admission.
- CG IR-629 specifies the process-origin projection under CG FR-019 and
  FR-022. Its typed mapping is available; selected-provider production
  composition is implemented, with remaining request-field and failure-path
  coverage identified in the acceptance rows.
- [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md).
- FSL FR-287 names `--provider-limit`, its limits and defaults; FSL FR-964 carries them as `ProviderOptions.provider_limits`; FSpec FR-561 defines the plugins-only configuration document; FSpec FR-777 makes the limit set one per run.
- [TC-041](../test-cases/TC-041-set-provider-limits-per-run.md), verifying AC-28 and AC-29.
