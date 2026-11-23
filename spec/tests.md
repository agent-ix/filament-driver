---
id: TM-001
title: "Driver command and execution test matrix"
type: TestMatrix
---
# TM-001: Driver binary, provider and execution test matrix

## Overview

This authored verification inventory integrates FR-001 and SP-D1 through
SP-D9 (FR-002 through FR-010) from IR-527, FR-011 from IR-616, FR-012 for
the downstream cache component, FR-013 for the process plugin host, and FR-014
for the CLI package boundary, and planned FR-015 for admitted monitor program
preparation. It covers FR-001 through FR-028 and TC-001 through TC-050.
Discrete FR and TC artifacts remain authoritative;
this matrix is their index, not a computed implementation-coverage report.

`✅` means the named verification is implemented in the listed artifact;
`🚧` means some or all verification remains planned, as described in the row.
These markers do not claim a fresh test run or compatibility with current
upstream APIs.

## Verification Ownership

| Scope | Verification owner and location |
|---|---|
| FR-001 / TC-001..TC-008 | Driver orchestration tests in `tests/drive.rs`, `tests/dependency_edges.rs` and `src/refusal.rs`, as named by the TC artifacts; TC-006 includes the IR-757 KaniForm projection and occurrence-context regression. |
| FR-001-AC-9..10, FR-011 / TC-019 | Driver library cancellation matrix and source-provision lifetime tests; API/lifetime review. |
| FR-002..FR-009 / TC-009..TC-016, TC-018 | FSL-backed execute and available proof-terminal serializers have unit coverage; `fsl-cli` process tests cover selected-engine and limit exit behavior; full outcome-contract and prove/replay composition tests remain planned. |
| Plugin wire, cache policy, execution semantics and AOT/interpreter parity | FSL ADR-029 owns the protocol and policy. Driver `filament-plugin-host` conformance is specified by TC-023..TC-026 under FR-013; implementation tests cover the listed session, typed-failure, cancellation/deadline, stderr and cleanup cases, with remaining cases identified in the matrix rows. TC-013, TC-014 and TC-015 specify composition boundaries; `filament-cache` behavior is verified by TC-020..TC-022 under FR-012. |
| FR-010 / TC-017 | Planned FSL-owned cross-repository test in `filament-integration`, QSL-514; FSL AR2 is the prerequisite. |
| FR-015 / TC-029 | Planned FSL producer, strict IR admission, complete IR FR-357 generated-source projection, driver lowering and RT build/execution evidence. The twelve complete-preparation FR-015 criteria remain planned and untagged; TC-029 supplies the procedure, not execution credit. |
| G5b acceptance evidence / IR-528 | `filament-integration` only: CLI/library parity across the eight outcome kinds with identical outcome bytes and exit codes; provider/plugin installation; cache hit/miss/never-store; AOT/interpreter parity; absence of a FSL user binary; and prove execution, replay and FR-331 records. End-to-end evidence is required on every OS declared supported at gate freeze; one platform's result does not establish acceptance on another. |

The driver supplies the one user-facing CLI. CG FR-005's CLI-facing
conformance is exercised through that binary; CG keeps its local generator
and atomic publication tests. CG's cross-backend parity and IR-337's
outcome-per-backend and bound-suite coverage feed the G5b evidence rather
than a second driver or CG gate. The driver and FSL suites supply local
verification; they do not substitute for IR-528 acceptance evidence.

## Requirements Traceability

### Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1..FR-001-AC-16 | TC-001, TC-002, TC-003, TC-004, TC-005, TC-006, TC-007, TC-008, TC-019 | 🚧 Partial: `tests/drive.rs::tc_006_rr16_preserves_each_occurrence_conversion_context` covers typed-form projection and distinct occurrence-context roots. Production FSL Call binding, request-index sidecar retention, and forwarding through generation (AC-13..AC-16) remain planned under IR-879. Configured Process-provider invocation is exercised under TC-013 and TC-028. |
| FR-001 | FR-001-AC-17..FR-001-AC-21 | TC-017 | 🚧 `tests/drive.rs` covers the export partition, whole-export refusal, unknown driver item and relation-less skip; cross-repository composition remains in `filament-integration` PR 16. |
| FR-001 | FR-001-AC-22 | TC-004 | ✅ `src/refusal.rs` tests retain all eight typed refusal fields in both present and absent forms. `DriverRefusal::from` constructs a typed FR-272 diagnostic with the code, cause, category and `package_read` stage; it maps the CG path to an FR-274 `JsonPath` location and retains the `CheckedNodeId` locus. Tests cover the five permitted incomplete limits, the `Diagnostics` invariant fault, and each unchanged CG projection. `src/drive.rs` converts the CG negotiation error through that constructor. FR-460/FR-461 reader-limit registration remains deferred. |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-5 | TC-009, TC-018, TC-028 | 🚧 Partial: `fsl-cli/tests/cli_contract.rs` compares `compile`, `check`, `parse`, and `format` machine documents with FSL-built outcomes; `compile` uses FSL's parse/select/check/package path. Its working-directory trap test verifies `check` reads the explicitly named source without reading malformed `fsl.toml` (AC-5). Other verbs remain explicitly unsupported. `binary_process.rs` checks successful outcome bytes, `invalid-request` and `io-error` documents against FSL-built documents, plus named root-argv refusals. Check-stage semantic refusal cases and `ExecutionFailure` documents remain planned; Linux/macOS signal behavior is tracked separately under AC-7. |
| FR-002 | FR-002-AC-4 | TC-009, TC-018 | 🚧 `TC-018` step 1 inspects the production call through `ix_cli_kit::exit::caller_exit_code`; the wrapper unit test checks FSL's category results, and the kit's exhaustive `u8` contract is tested in ix-cli-kit. Full CLI outcome composition remains planned. |
| FR-002 | FR-002-AC-6 | TC-009, TC-018 | ✅ `fsl-cli/tests/binary_process.rs::reserved_exit_code_uses_internal_failure_fallback` exercises the test-feature seam in text and machine modes; status `30`, empty stdout, and stderr naming reserved `5` and fallback `30` are asserted. |
| FR-002 | FR-002-AC-7 | TC-028 | 🚧 Linux/macOS cleanup and signal re-raise are exercised through the adapter; full CLI dispatch integration remains planned. |
| FR-002 | FR-002-AC-8 | TC-009, TC-018 | 🚧 Partial: diagnostics use `ix_cli_kit::streams::Record::write` and machine results use `write_primary_stdout`; unit and process-level tests cover failed stderr writes without replacing refusal/fallback statuses. |
| FR-002 | FR-002-AC-14..FR-002-AC-16 | TC-018 | 🚧 Partial: `fsl-cli/tests/binary_process.rs` verifies the refusal exit and absence of `OutputFailure` for a FIFO `BrokenPipe` on Unix, plus status `30` and an `OutputFailure` diagnostic for Linux `/dev/full`. Windows broken-pipe evidence and non-Linux non-broken stdout-I/O-error evidence remain open. |
| FR-002 | FR-002-AC-9 | TC-009, TC-018 | 🚧 Partial: binary-process tests compare successful outcomes and request-refusal/I/O-error documents with FSL-built canonical documents. The four TC-018 test-adapter cases (refused-range, profile-gated, cancelled and fault) remain planned; `ExecutionFailure` documents await QSL-653. |
| FR-002 | FR-002-AC-10 | TC-009 | 🚧 No config reader exists. If configuration is added, it must use ix-cli-kit `config`; unnamed configuration reads remain prohibited. |
| FR-002 | FR-002-AC-11 | TC-009 | ✅ `fsl-cli/tests/cli_contract.rs::version_command_reports_the_fsl_cli_package_version` checks `--version` against the `fsl-cli` Cargo package version. |
| FR-002 | FR-002-AC-12 | TC-009 | ✅ `fsl-cli/tests/cli_contract.rs::built_binary_version_agrees_with_cargo_package_version` checks the built binary with `ix_cli_kit::version::Agreement::command`. |
| FR-002 | FR-002-AC-13, FR-002-AC-18 | TC-040 | 🚧 Planned (QSL-1973): default text output through FSL FR-865 `render_outcome`, equal exit status in both modes, missing-file and `--format bogus` refusals. The superseded `unavailable_text_renderers_return_typed_unsupported_refusals` test is removed with the code change. |
| FR-002 | FR-002-AC-20..FR-002-AC-22 | TC-046 | 🚧 Planned (QSL-2348, QSL-2234): `fsl run` routes TL lowering and package requests through FSL's executors, applies `--limit` to the lowering's clause request, and registers the reference hybrid provider when it settles requirement items. |
| FR-003 | FR-003-AC-1, FR-003-AC-3 | TC-010 | ✅ `src/limits.rs` applies each exposed value through FSL bounds and `CallerLimits::set_bound`; `src/execute.rs::caller_source_limit_preserves_configured_bound_and_field` checks typed metadata through the driver wrapper; `binary_process.rs` checks incomplete stage-limit status and completion after raising the bound. |
| FR-003 | FR-003-AC-2, FR-003-AC-7, FR-003-AC-8 | TC-010 | ✅ `src/limits.rs` asserts FSL parser refusal causes; `binary_process.rs::fsl_rejects_unknown_malformed_and_duplicate_limits_before_source_read` checks operands and pre-stage refusal. |
| FR-003 | FR-003-AC-4 | TC-010 | ✅ Inspection: `src/limits.rs` uses FSL `parse_operands`, selected-operation `bounds()`, and `CallerLimits::set_bound`; the driver grammar/table is removed. |
| FR-003 | FR-003-AC-5 | TC-010 | ✅ `src/limits.rs::recognized_but_unavailable_setting_has_operation_specific_error` distinguishes a FSL-recognized setting unavailable to `run` from an unknown name. |
| FR-003 | FR-003-AC-6 | TC-010 | ✅ `src/limits.rs::every_exposed_setting_updates_only_its_fsl_bound` checks each exposed setting changes only its own bound. |
| FR-003 | FR-003-AC-9, FR-003-AC-10 | TC-010 | ✅ `src/limits.rs::each_named_accounting_setting_updates_only_call_accounting` checks all FSL accounting names update only `Call.accounting`; `fsl-cli/tests/binary_process.rs::named_accounting_work_units_limit_reaches_fsl_call_accounting` verifies `accounting.work_units=0` reaches FSL and returns the typed incomplete outcome. |
| FR-004 | FR-004-AC-1..FR-004-AC-10 | TC-011 | 🚧 Partial: CLI parser and binary-process tests cover interpreter selection, unavailable-engine refusal, and unknown engine values. `caller_object_context_controls_lookup_result`, `absent_undefined_lookup_matches_direct_fsl`, and `unresolved_population_identity_matches_direct_fsl_refusal` compare driver results/documents with direct FSL using the same object environments. `caller_object_environment_is_forwarded_unchanged` proves reference forwarding, and the CLI explicitly supplies its empty-environment policy. The private `execute_using` callback can characterize exact typed FSL `RunRefusal::UnsupportedResult` and preparation-refusal forwarding. Its invocation count cannot detect direct engine calls outside that callback. Full AC-4 evidence remains blocked on a second production engine or upstream-supported shared dispatch boundary that observes every engine call; the test must assert exact failure, one selected invocation, and zero alternative invocations. |
| FR-005 | FR-005-AC-1..FR-005-AC-4 | TC-012 | 🚧 The Linked Kani Provider slice covers FSL manifest admission/lookup, caller-supplied Linked origin, distinct lookup, and identical-manifest idempotence, plus Kani preflight refusals, explicit `ProofCeilings` propagation and mismatch refusal, typed CG evidence fields, and CG terminal mapping. Process-origin registration/duplicate behavior and full routed invocation remain planned. |
| FR-005 | FR-005-AC-5 | TC-012, TC-028 | 🚧 Partial: `tests/drive.rs` and the FR-331 host fixture cover caller-supplied Process origin, one hello admission and registration into the same FSL registry with its retained DTO. TC-012 now verifies CG's origin-aware kind conversion; the configured composition test exercises production dispatch. |
| FR-005 | FR-005-AC-12..FR-005-AC-15 | TC-013 | 🚧 Partial: `fsl-cli/src/process_provider.rs::fr331_hello_is_admitted_once_and_retained_for_registry_snapshot` checks one FSL hello admission, registration into the same registry, and lossless carrier retention. |
| FR-005 | FR-005-AC-16 | TC-012 | ✅ `src/registry.rs` checks canonical linked bytes, absent optional members, byte-for-byte preservation of supplied wire, and typed refusal of empty advertisements; `fsl-cli/src/backend_provider.rs::request_stage_refuses_missing_manifest_wire_without_reconstruction` checks the adapter refuses instead of reconstructing missing bytes. |
| FR-005 | FR-005-AC-18..FR-005-AC-19 | TC-013 | 🚧 Partial: the process hello adapter passes domains to FSL and preserves its typed `invalid-domains` refusal; absent/unknown kind and mode refusals also remain FSL-typed. Evidence-kind admission is FSL-owned and awaits QSL-654; fairness, obligation-form and reduction values are checked as FR-331 wire shape by the driver and interpreted at negotiation. |
| FR-005 | FR-005-AC-17 | TC-012 | ✅ `src/drive.rs::backend_kind_conversion_uses_admitted_origin` verifies the driver calls CG's descriptor conversion, preserves Process origin for the same identity, and still refuses an unknown Linked identity. |
| FR-005 | FR-005-AC-6 | TC-012, TC-013 | 🚧 Partial: `src/provider.rs::shared_provider_carrier_preserves_non_kani_data_and_failures` verifies generic success/failure data, request indexes and backend identity; the host adapter test verifies a non-Kani carrier survives conversion. End-to-end routed invocation remains planned under IR-672. |
| FR-005 | FR-005-AC-7 | TC-013 | 🚧 Planned under IR-672 and QSL-655: retain a plugin counterexample as a non-terminal candidate until FSL S6a replay reproduces it; do not emit `Refuted` before replay. |
| FR-005 | FR-005-AC-8 | TC-012 | 🚧 Partial: TC-012 exercises CG's authoritative `run_terminal_value` mapping; the driver passes the original evidence and optional settlement, and implements no second mapping. |
| FR-005 | FR-005-AC-20..FR-005-AC-21 | TC-012 | ✅ `src/provider.rs::kani_request_forwards_only_the_explicit_home_directory` forwards a known path and keeps unknown `None` with a decoy child-process `HOME`; `provider_result_preserves_optional_cg_diagnostics_and_failure_cause` retains CG's optional fields unchanged. |
| FR-005 | FR-005-AC-22 | TC-012 | ✅ `src/provider.rs::general_route_scalar_harness_is_refused_before_any_kani_launch` submits a driver-generated general-route scalar harness through the provider's production run path with a nonexistent launcher and crate and observes CG's typed `V2ClauseNotTractable` row before any launch; `native_route_scalar_harness_still_reaches_execution` generates a genuine native-route harness with CG's `generate_routed_native` and observes it is not refused: CG's execution refuses it as `HarnessNotInCrate` (the crate does not exist). Known gap: a `NotTractable` item has no terminal record, so `ProviderResults::outcome_bytes` fails the whole batch, as for any `Execution` refusal; CG ships no terminal map for `DecidedRunRefusal` and the driver adds none. |
| FR-005 | FR-005-AC-9..FR-005-AC-11 | TC-012, TC-013 | 🚧 Partial: generic non-Kani success/failure carriers are tested under IR-677. Plugin trusted `proved` mapping waits on QSL-490 and QSL-655; replay-only claims become tool failures only after QSL-655 amends ADR-029 PL-7. |
| FR-006 | FR-006-AC-1 | TC-013, TC-028 | 🚧 Partial: `fsl-cli/tests/binary_process.rs::configured_run_invokes_only_the_selected_process_and_retains_its_typed_result` starts two real FR-331 fixtures, admits both into one registry, runs FSL/CG composition and sends one selected request through the production host adapter. Full field-projection coverage and later-start cleanup remain incomplete. |
| FR-006 | FR-006-AC-8 | TC-013, TC-028, TC-041 | 🚧 Planned (QSL-2335): `deadline_ms` comes from the run limit set (`--provider-limit`, default 600000), not from the configuration entry; the existing configured-run test asserts the superseded per-entry source and is updated with the code change. |
| FR-006 | FR-006-AC-24 | TC-028, TC-041 | 🚧 Planned (QSL-2335): the plugins-only FSpec FR-561 `filament.provider-config/v1` document and its refusals replace the earlier `executable`/`argv`/`deadline_ms`/`limits` shape. |
| FR-006 | FR-006-AC-25 | TC-028 | 🚧 Partial: `fsl-cli/tests/binary_process.rs` `configured_run_invokes_only_the_selected_process_and_retains_its_typed_result` runs the `fsl` binary with two FR-561 providers, checks both are admitted into the request's registry snapshot before negotiation, that only the selected provider receives a request, and that its typed `proved` result is retained on the settled items. Later-start/hello failure cleanup with preservation of the original typed host failure remains untested. |
| FR-006 | FR-006-AC-26 | TC-028 | ✅ `fsl-cli/tests/binary_process.rs::duplicate_provider_identities_refuse_before_negotiation_and_close_sessions` checks machine `InvalidRequest`, no request markers, and shutdown of both sessions for FSL's idempotent repeated admission path. The composition-level test also checks typed refusal and cleanup diagnostics. |
| FR-006 | FR-006-AC-28, FR-006-AC-29 | TC-041 | 🚧 Planned (QSL-2335): `--provider-limit` names, defaults and refusals per FSL FR-287. |
| FR-006 | FR-006-AC-2 | TC-013 | 🚧 The process adapter preserves generic result carriers; no end-to-end settlement claim is made from an injected boundary. Production routing and typed-result retention are exercised by the configured composition; settlement remains pending FSL result-mapping APIs, including QSL-655 for AC-14..AC-23. |
| FR-006 | FR-006-AC-3 | TC-028 | 🚧 Partial: `fsl-cli/tests/binary_process.rs::configured_process_normal_exit_shuts_down_active_session` verifies normal production shutdown, and the production signal subprocess verifies signal cleanup. The production one-second forced-cleanup bound remains unasserted. Windows native evidence remains planned. |
| FR-006 | FR-006-AC-4 | TC-028 | 🚧 Partial: `fsl-cli/tests/binary_process.rs::configured_process_sigterm_cleans_active_session_before_reraise` and `configured_process_sigint_cleans_active_session_before_reraise` exercise production SIGTERM and SIGINT during a running request, verifying bounded cleanup, empty stdout and signal termination. Production start/shutdown-wait states remain untested; Windows console control is outside the guarantee. |
| FR-006 | FR-006-AC-5 | TC-013 | 🚧 Partial: `src/registry.rs::linked_manifest_snapshot_is_independent_of_registration_order` checks that the retained registered-candidate snapshot is sorted independently of registration order; production composition retains and serializes that snapshot, while linked-wire population remains IR-688. |
| FR-006 | FR-006-AC-6..FR-006-AC-13 | TC-013, TC-028 | 🚧 Partial: the production fixture checks the selected request's index, kind, claim, backend, candidate set, extent and unchanged deadline. Exact package bytes/id, forbidden top-level members, temporal proof bounds and proof-bound `kind` remain unverified; FSL proof-bound `kind` support awaits QSL-654. |
| FR-006 | FR-006-AC-14 | TC-013 | 🚧 Unclaimed: typed `declined` result mapping remains pending QSL-655; no terminal mapping is inferred. |
| FR-006 | FR-006-AC-15 | TC-013, TC-028 | 🚧 Partial: the production fixture verifies a `Tested` result remains typed in `CommandResult` and is not fabricated as a FSL terminal; the shared settlement path remains pending QSL-655 and IR-677. |
| FR-006 | FR-006-AC-16..FR-006-AC-23 | TC-013 | 🚧 Unclaimed: diagnostic conversion and settlement remain pending QSL-655 and the required FSL result-mapping APIs; no terminal mappings are inferred. |
| FR-007 | FR-007-AC-1..FR-007-AC-6 | TC-014 | 🚧 AC-1, AC-2 and AC-6 remain planned until provider dispatch is integrated; the helper tests cover cache-call semantics AC-3..AC-5 |
| FR-008 | FR-008-AC-1, FR-008-AC-2 | TC-015 | 🚧 Planned |
| FR-009 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5, FR-009-AC-10 | TC-016 | 🚧 Execute and available proved-provider terminals serialize through FSL `OutcomeDocument`; per-item terminal coverage applies to requests reaching settlement. Predicate/function replay and scalar parity use their distinct FSL/CG contracts; scalar parity never settles `refuted`. Full replay mapping and per-item accounting remain planned. |
| FR-009 | FR-009-AC-6, FR-009-AC-7, FR-009-AC-8, FR-009-AC-9, FR-009-AC-11, FR-009-AC-12, FR-009-AC-13 | TC-006, TC-016 | 🚧 FSL FR-367 and IR FR-049 APIs are available; production call-to-occurrence binding and the request-index sidecar through generation remain planned under IR-879 and are exercised by TC-006. CG request/meter/returned `ScalarLimits` attestation remains planned under IR-857. Refusal passthrough follows IR-847. Native receipt, replay conversion, limits mismatch, report-claim conversion, and per-item settlement remain planned under IR-773/IR-631/IR-610 and their CG API dependencies. Claim mismatch must use CG's typed converter when available; scalar parity never settles as `refuted`. |
| FR-001, FR-009 | FR-001-AC-23..FR-001-AC-24, FR-009-AC-23..FR-009-AC-24 | TC-016 | 🚧 Planned until IR-1121 supplies CG’s current-V2 function harness and proving-run identity, then IR-610 retains the exact FSL source/dependency lock, selected declaration and occurrence, admitted package, generated identity and provider evidence for E9. Missing or mismatched joins fault before provider/replay; upstream typed refusals keep their owner. |
| FR-009 | FR-009-AC-17..FR-009-AC-19 | TC-016 | 🚧 Planned: the driver projects FSL's exact typed `OperationSite.frame_operation` value to the CG item joined by admitted package identity and request index. FSL FR-121 defines the pair and owner resolution; FSpec FR-323 defines replay-target/request shape. AC-18 preserves FSL typed refusals; AC-19 reserves `runtime_invariant` for a driver-detected inconsistent join. Process/state-machine operations and state-clause projection remain out of scope. |
| FR-009 | FR-009-AC-14..FR-009-AC-15 | TC-016 | ✅ `src/provider.rs::falsified_scalar_does_not_guess_a_replay_settlement` passes the optional settlement to CG's map; `cg_terminal_mapping_reads_original_failure_cause_not_diagnostics` verifies the actual failure cause drives the map and optional diagnostics do not classify it. |
| FR-009 | FR-009-AC-26 | TC-016 | ✅ `src/drive.rs::scalar_native_tractability_tests::general_route_scalar_is_refused_before_the_paired_native_run` drives a general-route scalar through `drive` with `ScalarNativeInput` and a nonexistent launcher and observes `DriverRefusal::ScalarNotTractable` carrying CG's row (reaching the runner would be `ScalarNative`); `native_route_scalar_reaches_the_paired_native_run` observes a `generate_routed_native` harness reaching the runner (`ScalarNative`). |
| FR-009 | FR-009-AC-16 | TC-017 | 🚧 `tests/drive.rs` checks the bound-only generation indexes; terminal-record attribution remains in the TC-017 composition. |
| FR-010 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4, FR-010-AC-5 | TC-017 | 🚧 `tests/drive.rs` checks the partition and all-refused paths; `filament-integration` PR 16 checks the cross-repository bound-only path. |
| FR-011 | FR-011-AC-1, FR-011-AC-2, FR-011-AC-3, FR-011-AC-4, FR-011-AC-5 | TC-019 | ✅ Cancellation matrix and API mapping tests present; CLI adapter remains out of scope |
| FR-012 | FR-012-AC-1..FR-012-AC-5 | TC-020 | ✅ Key members, target shape, excluded metadata projection and meaning-input changes are tested with `Trace:` tags |
| FR-012 | FR-012-AC-6..FR-012-AC-8 | TC-021 | ✅ Matrix binds AC-6, AC-7, and AC-8 to the typed hit/miss, excluded-value, and deterministic incomplete-value test at `filament-cache/src/lib.rs:942` |
| FR-012 | FR-012-AC-9..FR-012-AC-12 | TC-022 | ✅ Entry bounds, temp cleanup, rename errors, ownership-filtered eviction, concurrent writes and eviction-callsite NotFound are tested with `Trace:` tags |
| FR-012 | FR-012-AC-13 | TC-005 | ✅ `tests/dependency_edges.rs::tc_005_driver_edges_match_the_adr_011_allow_list` carries a `Trace:` tag |
| FR-013 | FR-013-AC-1..FR-013-AC-3 | TC-023 | 🚧 Partial: tests refuse bare names and cover hello admission, rejected protocol, and zero, one and multiple requests; the explicit no-request-frame assertion remains planned. |
| FR-013 | FR-013-AC-4, FR-013-AC-7 | TC-024 | 🚧 Partial: tests cover invalid typed result, wrong request id, fault, non-zero exit, truncated hello, non-canonical/oversized frames and unexpected extra frame; malformed JSON and truncated-body cases remain planned. |
| FR-013 | FR-013-AC-5..FR-013-AC-6 | TC-025 | 🚧 Partial: valid settlement and fault during cancellation grace, grace-expiry descendant cleanup within one second, and blocked-write deadline are tested. |
| FR-013 | FR-013-AC-9..FR-013-AC-10 | TC-026 | 🚧 Partial: stderr truncation and EOF exit-status preservation, descendant cleanup and bounded reader joins are tested. |
| FR-013 | FR-013-AC-11 | TC-005, TC-028 | 🚧 `tests/dependency_edges.rs` and `fsl-cli/tests/package_boundary.rs` already assert the direct fsl-cli-to-host edge and host-free driver closure; full TC-028 adapter composition remains planned for IR-672. |
| FR-013 | FR-013-AC-12 | TC-027 | 🚧 Makefile `test` already runs `cargo fetch --locked`; clean-cache fetch, offline metadata and full-test evidence remain planned |
| FR-013 | FR-013-AC-13 | TC-027 | ✅ Python 3 prerequisite test emits an actionable installation message |
| FR-013 | FR-013-AC-14 | TC-028 | 🚧 Deferred to IR-674; this PR has no kill-on-close implementation or test. Native Windows evidence is required when the follow-up is completed. |
| FR-013 | FR-013-AC-15 | TC-028 | 🚧 Planned analysis/inspection of `filament-plugin-host/src/lib.rs`; runtime failure evidence requires a distinguishable or injectable setup-failure seam and native Windows runner. |
| FR-014 | FR-014-AC-1 | TC-028 | ✅ `fsl-cli/tests/package_boundary.rs::cargo_target_and_dependencies_belong_to_fsl_cli` asserts the sole `fsl` binary target is in `fsl-cli`. |
| FR-014 | FR-014-AC-2 | TC-005, TC-028 | ✅ `tests/dependency_edges.rs` asserts the direct CLI edges and that the driver closure excludes the kit; `fsl-cli/tests/package_boundary.rs` checks the direct kit edge. |
| FR-014 | FR-014-AC-3 | TC-005 | ✅ TC-005 asserts `fsl-cli` workspace default-member status |
| FR-014 | FR-014-AC-4 | TC-009, TC-018, TC-028 | ✅ `fsl-cli/tests/package_boundary.rs::parser_and_cli_tests_live_in_the_cli_package` asserts CLI test-source and non-executable fixture ownership; TC-028 allows the test-only provider binary in its separate unpublished workspace package. CLI behavior remains tracked separately by TC-009/TC-018. |
| FR-014 | FR-014-AC-5 | TC-028 | ✅ `fsl-cli/tests/package_boundary.rs::parser_and_cli_tests_live_in_the_cli_package` asserts parser and binary entry point locations. |
| FR-014 | FR-014-AC-6 | TC-028, TC-041 | 🚧 Planned (QSL-2335): `--provider-config` is the sole plugin source and `--provider-limit` the sole limit source for `fsl run`. |
| FR-014 | FR-014-AC-7 | TC-028 | ✅ Empty-prefix `cargo install --path fsl-cli --locked` installs only `fsl`; the installed version, model-free check and sample run succeed. The focused provider process lane builds `fsl-test-provider` before running the `fsl-cli` provider tests. |
| FR-015 | FR-015-AC-1..FR-015-AC-12 | TC-029 | 🚧 Planned: producer/read/lower/RT evidence in TC-029 has not executed. QSL-1631 source-owned witnesses, complete IR FR-357 projection and driver preparation remain open; the first witnessed IR subset does not complete a `TypeEnvironment` or RT `ProgramParts`. No criterion is accepted by this inventory row. |
| FR-016 | FR-016-AC-1..FR-016-AC-12 | TC-030 | 🚧 Planned for the one-budget accounting of FR-016-AC-10/11; locus-retention evidence as recorded in TC-030. |
| FR-017 | FR-017-AC-1..FR-017-AC-8 | TC-031 | 🚧 Planned: full lowering on the caller budget; scalar helper evidence as recorded in TC-031. |
| FR-018 | FR-018-AC-1..FR-018-AC-5 | TC-032 | 🚧 Evidence as recorded in TC-032. |
| FR-019 | FR-019-AC-1..FR-019-AC-10 | TC-033 | 🚧 Planned: one-budget preparation from a genuine plan root; producer criteria run in `filament-integration`. |
| FR-020 | FR-020-AC-1..FR-020-AC-15 | TC-034 | ✅ AC-1 to AC-9 and AC-11 to AC-15 run, tagged, in `tests/drive.rs` and `src/refusal.rs` against CG main; AC-14's guard-node case uses FSL's `test-support` constructors. AC-10 is retired by CG IR-304 slice (b). Driver ticket IR-1494. |
| FR-021 | FR-021-AC-1..FR-021-AC-7 | TC-035 | ✅ `tests/examples_gate.rs`, run by `make examples-check` from `make ci` against a fetched filament-examples checkout, and by the language repository's `make examples-check` with `FSL_LANGUAGE_DIR` set; `tests/make_targets.rs` checks the `ci` target. |
| FR-022 | FR-022-AC-1..FR-022-AC-12 | TC-036, TC-047 | 🚧 `fsl-cli/tests/frontend_cli.rs` runs the `fsl` binary over the Counter analyze fixture; `src/analyze.rs` unit-tests the negotiation table and the FSpec FR-711-AC-5 parity of its dispositions with CG `negotiate_item` (TC-1741); `analyze_provider_config_*` and `analyze_provider_without_a_verdict_is_inconclusive` start the Rust `fsl-test-provider` process (AC-9), `analyze_signal_cancels_with_cause_requested` has it signal `fsl` mid-request (AC-10), and `analyze_timed_liveness_reports_non_zeno` checks AC-12 through the real binary. |
| FR-023 | FR-023-AC-1..FR-023-AC-7 | TC-037 | ✅ `fsl-cli/tests/clocks_flag.rs` runs the `fsl` binary with `--clocks`; `fsl-cli/tests/replay_cli.rs` runs `fsl replay` over a compiled package and checks the replay-document `clocks` refusal (FSL FR-640-AC-8). |
| TC-038 | Write the checked package with fsl compile --output | Integration | FR-024-AC-1..FR-024-AC-6 | ✅ See the FR-024 row. |
| FR-024 | FR-024-AC-1..FR-024-AC-6 | TC-038 | ✅ `fsl-cli/tests/output_flag.rs` runs the `fsl` binary with `--output`; the write itself is FSL's (FSL FR-810). |
| TC-039 | Stream observations with the fsl monitor verb | Integration | FR-025-AC-1..FR-025-AC-8 | ✅ See the FR-025 row. |
| TC-040 | Write the text outcome by default for parse, check, compile and run | Integration | FR-002-AC-13, FR-002-AC-18 | 🚧 Planned (QSL-1973). See the FR-002-AC-13 row. |
| TC-041 | Set the provider limits once per run with --provider-limit | Integration | FR-006-AC-8, FR-006-AC-24, FR-006-AC-28, FR-006-AC-29, FR-014-AC-6 | 🚧 Planned (QSL-2335). See the FR-006-AC-28 row. |
| FR-025 | FR-025-AC-1..FR-025-AC-9 | TC-039, TC-044 | 🚧 `fsl-cli/tests/monitor_cli.rs` runs the `fsl` binary over a Reaches/Never stream; `provider_config_routes_the_request_items_to_the_started_plugin` routes the request items to a started plugin (AC-4, TC-044); `a_signal_ends_an_open_stream_as_a_cancelled_run` signals an open stream (AC-9). Ticket QSL-2313 (driver half of QSL-1977). |
| TC-042 | Write the replay outcome document and its text with fsl replay | Integration | FR-026-AC-1..FR-026-AC-4 | 🚧 See the FR-026 row. |
| FR-026 | FR-026-AC-1..FR-026-AC-4 | TC-042 | 🚧 `fsl-cli/tests/replay_cli.rs` stubs, ignored until the text result lines land (QSL-2292). |
| TC-045 | Write a TLA+ or Lean export with fsl export | Integration | FR-027-AC-1..FR-027-AC-5 | 🚧 See the FR-027 row. |
| FR-027 | FR-027-AC-1..FR-027-AC-5 | TC-045 | 🚧 `fsl-cli/tests/export_cli.rs` stubs, ignored until `fsl export` and FSL's export entry land (QSL-2359). |
| TC-046 | Run TL lowering and package requests with the fsl binary | Integration | FR-002-AC-20..FR-002-AC-22 | 🚧 See the FR-002-AC-20 row. |
| TC-047 | Decide hybrid claims with FSL's reference provider through fsl analyze | Integration | FR-022-AC-11 | 🚧 See the FR-022-AC-11 row. |
| FR-022 | FR-022-AC-11 | TC-047 | 🚧 Planned (QSL-2234): `fsl analyze` registers FSL's reference hybrid provider beside the configured providers. The process-provider half (TC-047 step 4) also depends on FSL's `Engine::hybrid_process` (QSL-2532). |
| FR-002 | FR-002-AC-23 | TC-048 | ✅ `fsl-cli/tests/cli_contract.rs::inspect_handoff_verifies_the_package_and_sidecar` compiles the spine fixture with a `handoff` member and compares `fsl inspect --handoff` with `inspect_handoff` in JSON and text, over a raised node count, a changed package byte, a missing path, an empty directory, a regular file and `./-odd`, refuses the malformed forms with the directory unreadable, and checks that `inspect` without `--handoff` stays not routed (QSL-2601). |
| TC-048 | Verify a checked-package handoff with fsl inspect --handoff | Integration | FR-002-AC-23 | ✅ See the FR-002-AC-23 row. |
| FR-002 | FR-002-AC-24..FR-002-AC-26, FR-002-AC-28, FR-002-AC-30..FR-002-AC-32 | TC-049 | ✅ AC-30..AC-32 (QSL-2929, QSL-2931, QSL-2932): `cli-contracts/tests/messages.rs` pins the exact unknown-verb, unknown-option, engine and limit messages, `help <verb>`, the root-help verb lines, operands and exit codes, and the non-UTF-8 argument on stdout. `fsl-cli/src/options.rs` is the one option table; `fsl-cli/tests/help_contract.rs` runs `help_lists_every_accepted_option`, `parser_refuses_options_outside_its_entry`, `verb_help_writes_its_section`, `help_precedence` and `color_is_refused_on_every_verb` over the built binary, and the `fsl-cli` unit tests `removing_a_table_option_removes_it_from_parser_and_help` and `no_option_literal_appears_outside_the_table` check the table-mutation and no-literal rules (QSL-2602). `--extensions <file>` on `parse` and `check` (QSL-2603) is covered by `fsl-cli/tests/frontend_cli.rs` `extension_sets_supply_the_selected_definition_to_check_and_parse` and `malformed_extension_sets_refuse_before_any_stage`. The filament-docs generator update for the per-verb sections is QSL-2608. |
| TC-049 | List every accepted option in root and per-verb help | Integration | FR-002-AC-24..FR-002-AC-26, FR-002-AC-28, FR-002-AC-30..FR-002-AC-32 | ✅ See the FR-002-AC-24 row. |
| FR-028 | FR-028-AC-1..FR-028-AC-8 | TC-050 | 🚧 Planned (QSL-2930): `fsl-cli/tests/request_discovery.rs`. Depends on FSL's request-example entry and refusal behaviour (QSL-2935). |
| TC-050 | Print a request example and name every request defect | Integration | FR-028-AC-1..FR-028-AC-8 | 🚧 See the FR-028 row. |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-001 | The steps run in ADR-011 T-13 order | Integration | FR-001-AC-4 | ✅ Tests present |
| TC-002 | An unknown BackendId refuses the run before negotiation | Integration | FR-001-AC-2 | ✅ Tests present |
| TC-003 | route runs after negotiation, for supported items only | Integration | FR-001-AC-3 | ✅ Tests present |
| TC-004 | Command-level refusals are typed | Integration | FR-001-AC-5, FR-001-AC-22 | ✅ command-level refusal tests and focused CG reader-projection unit tests present |
| TC-005 | The dependency edges match ADR-011 §6.1 and §7.1 | Integration | FR-001-AC-1, FR-001-AC-11, FR-012-AC-13, FR-013-AC-11, FR-014-AC-2, FR-014-AC-3 | ✅ Driver/cache edges, host-free and kit-free driver closure, direct CLI-to-host/kit edges and CLI default membership are asserted. |
| TC-006 | E5 admits the emitted package and CG generates for the routed items | Integration | FR-001-AC-6, FR-001-AC-7, FR-001-AC-8, FR-001-AC-12..FR-001-AC-16 | 🚧 Existing tests verify typed-form projection and two same-node occurrences’ distinct conversion roots. The production FSL Call/binder-to-CG composition, ordered operand sidecars, full request-position vector, and typed absence/mismatch cases remain planned under IR-879. This generation-path regression is distinct from TC-016 native receipt authentication and replay. |
| TC-007 | Generation refusals are typed | Integration | FR-001-AC-5, FR-001-AC-6 | ✅ Tests present |
| TC-008 | A missing import or an unknown header profile refuses at compile | Integration | FR-001-AC-5 | ✅ Tests present |
| TC-009 | Exercise CLI verbs, outcome documents and exit codes | Integration | FR-002-AC-1..FR-002-AC-6, FR-002-AC-8..FR-002-AC-12, FR-014-AC-4 | 🚧 Partial: `fsl-cli/tests/cli_contract.rs` checks FSL-equivalent machine documents for `compile`, `check`, `parse`, and `format`; default text mode is verified by TC-040. Other verbs remain explicitly unsupported. The working-directory trap covers AC-5. Stream write failures, check-stage semantic refusal cases, and `ExecutionFailure` documents remain planned. No configuration reader exists; AC-10 remains unimplemented. Fixtures belong to `fsl-cli`. |
| TC-010 | Apply explicit lifecycle limits and preserve limit outcomes | Integration | FR-003-AC-1..FR-003-AC-10 | ✅ FSL parser/application, stage-bound independence, unavailable-stage-setting refusal, unknown/malformed/duplicate refusals, typed stage-limit results and raised-bound completion are tested. FSL's `accounting.<counter>` setting names route through `AccountingLimits`, while the separate bare-name adverse case retains FSL's unknown-setting refusal; focused unit and process tests assert stage limits remain unchanged and FSL returns an incomplete accounting-limit outcome. |
| TC-011 | Select only the requested execution engine | Integration | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4, FR-004-AC-5, FR-004-AC-6, FR-004-AC-7, FR-004-AC-8, FR-004-AC-9, FR-004-AC-10 | 🚧 `src/execute.rs` verifies interpreter selection, unchanged environment-reference forwarding, direct-FSL parity for populated/empty object contexts, the FSL-defined absent-lookup `Undefined` result, unresolved-population refusal parity, unavailable-engine refusal and preparation-refusal parity. Callback-injected cases can verify exact `RunRefusal::UnsupportedResult` and typed preparation-refusal forwarding, but callback counts cannot detect a direct alternate invocation. Full AC-4 remains Partial until a second production engine or upstream-supported shared boundary makes every invocation observable; the future oracle must assert exact results, one selected invocation, zero alternative invocations, and fail if any engine call bypasses the boundary. |
| TC-012 | Register and invoke a first-party provider through Provider | Integration | FR-005-AC-1..FR-005-AC-11, FR-005-AC-16 | 🚧 `src/registry.rs` covers distinct lookup, Linked-origin FR-005-AC-1, identical-manifest idempotence without order-dependent selection, canonical linked-manifest bytes with absent optional members omitted, byte-for-byte retention of supplied wire, and refusal of empty advertisements; `fsl-cli/src/backend_provider.rs` verifies the adapter refuses `wire: None` instead of reconstructing. `tests/drive.rs` tests the Linked/Process projection and caller ceilings in the generated harness identity; `src/provider.rs` covers cancellation, backend mismatch, missing limits/harness, execution-ceiling mismatch, typed evidence carriers and CG terminal mappings; `tests/drive.rs` exercises public `Provider::run`. Generic non-Kani carriers are tested; real Kani execution/result forwarding remains planned. Configured process-provider invocation is exercised under TC-013 and TC-028. |
| TC-013 | Dispatch process-provider results through the driver | Integration | FR-005-AC-12..FR-005-AC-19, FR-006-AC-1..FR-006-AC-2, FR-006-AC-5..FR-006-AC-23 | 🚧 Partial: the production fixture covers call-aware selected-item dispatch, typed result retention, request projection, hello admission/registry retention, FSL-owned domain admission/refusals, and generic result carriers. Evidence-kind admission/refusals and proof-bound `kind` emission await QSL-654; fairness, obligation-form and reduction values are enforced as FR-331 wire shape and used at negotiation. Settlement remains pending FSL result-mapping APIs. The 13 Kani rows from FSpec FR-331-AC-20, FR-272 diagnostics, and replay/proved settlement remain held. Replay-only claims remain unhandled until QSL-655 amends PL-7. |
| TC-014 | Exercise the cache-aware provider-call helper through a recording cache interface | Unit | FR-007-AC-1..FR-007-AC-6 | 🚧 Helper tests cover cache-call semantics AC-3..AC-5; admission-to-dispatch integration AC-1, AC-2 and AC-6 remains planned |
| TC-015 | Invoke AOT with current checked request inputs | Integration | FR-008-AC-1, FR-008-AC-2 | 🚧 Planned |
| TC-016 | Settle one terminal record per item after replay | Integration | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5, FR-009-AC-6, FR-009-AC-7, FR-009-AC-8, FR-009-AC-9, FR-009-AC-10, FR-009-AC-11, FR-009-AC-12, FR-009-AC-13, FR-009-AC-17, FR-009-AC-18, FR-009-AC-19, FR-009-AC-20..FR-009-AC-24, FR-001-AC-23..FR-001-AC-24 | 🚧 FSL/IR binding APIs exist; TC-006/IR-879 owns the still-planned production generation composition. TC-016 specifies native receipt authentication, replay, typed conversion, terminal settlement, and frame-target projection; these remain planned under IR-773/IR-631/IR-610 and the FSL/CG API dependencies; function proof production also waits on IR-1121. Predicate/function reproduction, frame projection, and scalar-parity conversion remain distinct; scalar parity never settles `refuted`. |
| TC-017 | Send only abstraction-relation-bound items to code generation | Integration | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4, FR-010-AC-5, FR-001-AC-17..FR-001-AC-21, FR-009-AC-16 | 🚧 `tests/drive.rs` exercises the driver partition, typed refusal and index mapping; `filament-integration` PR 16 exercises FSL/driver/CG composition. The full specified terminal outcome remains open. |
| TC-018 | Preserve the FSL outcome at the binary process boundary | Integration | FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-6, FR-002-AC-8, FR-002-AC-9, FR-002-AC-14, FR-002-AC-15, FR-002-AC-16, FR-014-AC-4 | 🚧 Partial: binary-process tests cover the code-5 fallback, machine-outcome bytes, request/I/O refusals, stream diagnostics, the FIFO broken-pipe case on Unix, and the Linux `/dev/full` case. The four test-adapter cases (refused-range, profile-gated, cancelled and fault), full locus-rendered diagnostic comparison, Windows broken-pipe coverage, non-Linux `/dev/full` equivalent, and `ExecutionFailure` documents (QSL-653) remain planned. Fixtures belong to `fsl-cli`. |
| TC-019 | Propagate library cancellation and retain bounded checked-unit source | Integration | FR-001-AC-9, FR-001-AC-10, FR-011-AC-1..FR-011-AC-5 | ✅ Cancellation/source-lifetime tests are present with explicit comma-separated Trace IDs for FR-001-AC-9, FR-001-AC-10, and FR-011-AC-1..FR-011-AC-4. |
| TC-020 | Derive cache identity from meaning inputs | Unit | FR-012-AC-1..FR-012-AC-5 | 🚧 Key, target, and excluded-field projection tests are present; `Trace:` tags identify covered criteria |
| TC-021 | Reject invalid cache entries and excluded values | Unit | FR-012-AC-6..FR-012-AC-8 | ✅ Quire matrix binds AC-6, AC-7, and AC-8 to `filament-cache/src/lib.rs:942`; typed invalid-entry readers also bind AC-6 |
| TC-022 | Keep cache writes atomic and within the caller's size limit | Unit | FR-012-AC-9..FR-012-AC-12 | ✅ Size, rename boundary and rename NotFound, temp cleanup, ownership-filtered eviction, concurrency and removal-race tests carry `Trace:` tags |
| TC-023 | Start a plugin session and admit hello before requests | Integration | FR-013-AC-1..FR-013-AC-3 | 🚧 Partial: executable-name refusal, hello admission, rejected protocol, and zero/one/two requests are tested; the explicit no-request-frame assertion remains planned. |
| TC-024 | Enforce plugin framing and preserve typed failures | Integration | FR-013-AC-4, FR-013-AC-7 | 🚧 Partial: invalid result, wrong request id, plugin fault, non-zero exit, truncated hello, non-canonical/oversized frames, and unexpected extra frame are tested; malformed JSON and truncated-body cases remain planned. |
| TC-025 | Preserve plugin settlement during cancellation and bound deadlines | Integration | FR-013-AC-5..FR-013-AC-6 | 🚧 Partial: cancellation settlement/fault, grace-expiry descendant cleanup within one second, and blocked-write deadline tests exist. |
| TC-026 | Bound plugin stderr and preserve cleanup | Integration | FR-013-AC-9..FR-013-AC-10 | 🚧 Partial: stderr truncation and EOF exit-status preservation, descendant cleanup and bounded reader joins are tested. |
| TC-027 | Run plugin-host tests and dependency checks from a clean cache | Integration | FR-013-AC-12, FR-013-AC-13 | 🚧 Python prerequisite message is tested; clean-cache fetch, offline metadata, and full gate evidence remain planned. |
| TC-028 | Keep the CLI package separate from the driver and compose cache and process host | Integration | FR-002-AC-3, FR-002-AC-7, FR-006-AC-1, FR-006-AC-3..FR-006-AC-4, FR-006-AC-24..FR-006-AC-27, FR-013-AC-11, FR-013-AC-14..FR-013-AC-15, FR-014-AC-1, FR-014-AC-2, FR-014-AC-4..FR-014-AC-7 | 🚧 Partial: `fsl-cli` tests cover explicit-option parsing, the empty configuration, strict invalid-file/deadline outcomes, selected-provider composition with typed-result retention, production duplicate-identity refusal, and active-request SIGTERM cleanup. TC-028 adds empty-prefix install/version/check/sample evidence and documents building `fsl-test-provider` before focused provider process tests. Later-start/hello failure cleanup remains untested. Package-boundary tests cover target ownership, layout and the direct CLI-to-host edge. Adapter lifecycle/signal tests are partial; AC-14 kill-on-close is deferred to IR-674 and has no implementation or test in this PR; it requires native Windows evidence when resumed. |
| TC-034 | Project FSL requirement records into CG's typed input | Integration | FR-020-AC-1..FR-020-AC-15 | ✅ See the FR-020 row. |
| TC-035 | Run the examples suite against the workspace-built fsl | Integration | FR-021-AC-1..FR-021-AC-7 | ✅ See the FR-021 row. |
| TC-036 | Analyze a native-analyze/1 request with the fsl binary | Integration | FR-022-AC-1..FR-022-AC-10, FR-022-AC-12 | 🚧 See the FR-022 row. |
| TC-037 | Supply clock configurations with --clocks on compile and run | Integration | FR-023-AC-1..FR-023-AC-7 | 🚧 See the FR-023 row. |

## Integration Test Matrix

| Purpose | Target | Type | Test Cases |
|---|---|---|---|
| Verify cache calls carry eligible requests and preserve typed hits | `filament-driver` cache interface with a recording implementation | component | TC-014 |
| Verify content identity, typed reads, exclusion policy and bounded atomic storage | Downstream `filament-cache` package and caller-named temporary directory | component | TC-020, TC-021, TC-022 |
| Verify plugin sessions, protocol, typed outcomes, cancellation, deadlines, stderr and cleanup | `filament-plugin-host` with deterministic process fixtures | component | TC-023, TC-024, TC-025, TC-026, TC-027 |
| Verify CLI process-provider composition and cleanup on exit and handled signals | `fsl-cli` adapter and `filament-plugin-host` with a descendant-spawning fixture | component | TC-028 |
| Verify admitted monitor program preparation and execution from real emitted entries | FSL producer, strict IR reader, driver preparation and RT program | integration | TC-029 (planned) |

The separate clock-admission component has narrow FR-015-AC-13
traces for native clause/role/ordinal and work-refusal payloads, plus
`MissingClock` payload classification. An admitted non-Timed no-clock driver
entry is still needed for the no-token `MissingClock` obligation.
The original partial selection controls retain their unclocked meaning.
Contract CG PR #749 removes the previous production compilation blocker.
Actual production component execution passes the filament-integration `monitor_preparation` and the driver
`monitor_lowering` suites. The clock-admission controls
exercise reached FR-015-AC-13 native refusal payloads, exact requested
occurrences and remaining allowance, plus original pre-cancellation with no
capability or mutation. This is component evidence, not a final gate or a
complete preparation result.
The authentic Timed PlanRoot now provides a native model-time clock receipt,
extra independent configured-row refusal and successful cumulative work
accounting under FR-015-AC-14. Configured-profile row comparison and full
criterion qualification remain Planned.

This component does not qualify FR-015-AC-1 through FR-015-AC-12 or whole
TC-029 execution. FSL PR #2395 supplies the final subject-occurrence owner
lookup; its driver consumer has separate FR-015-AC-15 controls. Admitted
order/progress association, authentic final source-union producer identities,
complete generated record closure and full Program preparation remain missing
facts. Contract IR PR #905 supplies the native union seam and Contract IR
PR #908 supplies the selected clock read. FSL PR #2385 supplies final
source-record/member rows and Contract IR PR #910 supplies native record
admission/projection; their driver join and complete closure qualification
remain separate work. The clock component retains the direct native refusal
when those available record rows are not joined.

The native final-subject API retains the genuine `MonitorSubject` and final
`Emission`, strictly authenticates the complete native package, and consumes
the original-key/ordinal owner declaration lookup. Native acquisition, full
package equality and the owner's unbounded linear lookup are outside the
driver ledger; declaration `len()` is not a work receipt. The driver prefix
is preserved through the mapped token's actual native clock continuation.
This component reads the healthy native Timed model-time source but supplies no
complete preparation qualification. Source-record driver joining and full
closure remain separate.

Production filament-integration `monitor_final_subject` executes eight FR-015-AC-15
controls: two-clause owner mapping and native model-time source reading;
another declaration that maps in the same emission; absent lookup and edited
node/role/name/package; full native package and instance mismatches; independent
clause/full occurrence/subject changes; native acquisition payloads/limits; and
original cancellation with exact/one-under driver prefix continuation. The
preparation controls also exercise native model-time admission. These receipts
qualify final-subject selection and its bounded native source read, not whole
TC-029 preparation.

The additional early-identity boundary control requires zero-work refusal
before wrong producer package/node/role/ordinal selection. Exact package-name
material and both native keys are charged before comparison. Native borrowed
key unit controls cover nested UTF-8 material, exact/one-under work and scratch
release on work, storage and cancellation stops. Both integration suites use
one shared native producer/request builder with explicit source shapes.
