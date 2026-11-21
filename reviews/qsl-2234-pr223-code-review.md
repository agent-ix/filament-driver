---
id: SR-202611210022442313144108514
title: "QSL-2234 PR223 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; PR223; all 18 changed files; whole repository traceability baseline"
review_set: subset
relationships:
  - {target: ix://agent-ix/filament-driver, type: references}
---

## Summary

PASS: no code defects found. Final narrow fixture disposition and actual focused remote composition/Clippy evidence are complete; the inherited gap audit remains FAIL under the separate driver-specific ARCH acceptance. Ticket: QSL-2234. Reviewed frozen `f3711b7ee4408261ba29f0d69357fb791d31f1ee` against `260c35acc70783e4bbb3d580f582f2cd57c24454`.

## Verdict

PASS: no code defects found. Final narrow fixture disposition and actual focused remote composition/Clippy evidence are complete; the inherited gap audit remains FAIL under the separate driver-specific ARCH acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review evidence

Frozen PR #223: `f3711b7ee4408261ba29f0d69357fb791d31f1ee` against `260c35acc70783e4bbb3d580f582f2cd57c24454`. Ticket: QSL-2234. All 18 changed files examined. No owned spec/plan edits, so spec-review and its sub-analyses do not apply. No applicable AssuranceProfile was found. Optional additional semantic gap review was not requested and was not run.

The Rust lane applied API, implementation, testing, safety/concurrency and wire/persistence references, repository AGENTS.md/CLAUDE.md, contribution/lint/formatter/source-policy instructions and CI diff. No CI lane changes. No Tokio path changes. No source, branch, push, build or runtime mutation was performed by this reviewer.

Ownership: the actual FSL Braking source and request files are consumed in place from the declared fsl-replay dependency using Cargo metadata. No copied native program or private upstream API. The new shared resolver replaces two existing metadata resolver implementations in monitor_refusal and source_record_admission. Composition independently overlays the existing driver Counter library; the process fixture scripts external wire answers rather than replacing the reference solver. The strong Reservoir oracle is unchanged apart from removing its ignore.

Lifecycle: Engine::hybrid_process receives the admitted descriptor and opaque exchange over the existing HostProcessSession/PluginSession. Owner request composition, strict response reading, proof trust, certificate checks and native replay remain in FSL. Unknown response members survive driver transport. Mixed value/hybrid registration is the explicitly authorized temporary refusal with all admitted sessions retained for cleanup; this review does not claim combined-engine implementation or QSL-2234 closure. Private per-exchange timer tokens preserve the global parent token. Post-wait checks reject late successful bytes and preserve parent cancellation precedence; typed failures map into FSL HybridExchangeError. Real child abort/grace/reap remains the host lifecycle.

Local cheap checks: git diff --check and cargo fmt --all --check both exit 0. Supplied ROOTTEST-RUN AA71 evidence reports two production builds, three focused Clippy invocations and 25/26 runtime tests passing, zero ignored; the sole failure was the composition fixture's duplicate subject alias, then fixed in f3711b7. The actual production source is unchanged in f3711b7. Exact-head composition execution and frontend Clippy remain pending in this review; no final-head passing runtime claim is made. Remote ROOT owns all build/test/Clippy work. Full-crate/all-targets/feature sweeps, Kani/proof and broad gates were not authorized for this reviewer. No runtime result is inferred from matrix trace tags.

Module/tool provenance: quoin 0.36.8 at /home/peter/.npm-global/bin/quoin; quire 0.52.0 (engine 0.62.2) at /home/peter/.npm-global/bin/quire; spec-artifacts-process@v0.36.0. Both matrix commands and coverage commands exited 0. Coverage emitted two cfg symbol-id-collision notices (signals::run and frontend invalid_os_string); both are present in the exact base as well.

## Coverage

Repository reverse inventory: compilation/emission/routing/generation (FR-001/003/004), CLI/outcomes/run (FR-002), scalar execution (FR-009), providers/process framing/lifecycle (FR-005/006/013/014), caching (FR-007), replay (FR-026), inspection/diagnostic/source intake and monitor preparation (FR-010/011/012/015/017/019/020/021/023/024/025/027), and analyze/composition/hybrid rendering (FR-022). These behavior groups were inventoried over src/, fsl-cli/src/, filament-plugin-host/src/ and filament-cache/src/ and linked through their existing requirement/module descriptions; no confirmed new unowned behavior, source stub, test stub or hollow coverage claim was found. Intentional library reexports and external test fixture answers are not treated as stubs. This is an inventory and focused changed-code inspection, not a claim of exhaustive semantic verification of every historical implementation.

Plan completion: not assessed

Reconciliation: quoin matrix (quoin 0.36.8, quire 0.52.0 / engine 0.62.2), whole repository. 317 criteria: 127 tagged, 175 untagged, 5 tagged-by-ignored-test, 10 method-without-symbol; qualified-record 0, retired 0. All 317 have no run evidence in the quoin store. Execution evidence above was supplied separately, not imported or invented as engine evidence.

Exact-base comparison: 125 tagged, 177 untagged, 5 ignored-only, 10 method-without-symbol. The ONLY criterion-status changes are FR-002-AC-22 and FR-022-AC-11, both untagged to tagged. No new criterion gap. Coverage: 201 untracked symbols on both snapshots, 1160 diagnostics on both snapshots; bound symbols 246 to 262, tagged 742 to 760, candidates 773 to 791. Unmatched tags 898 to 903: the five new unmatched bindings are FR-013-AC-6/TC-025 on request_timeout_rejects_a_reply_arriving_inside_the_receive_wait and FR-013-AC-5/FR-013-AC-6/TC-025 on request_timeout_preserves_parent_cancellation_over_a_late_ready_reply. They use the documented `/// Trace:` form, but this active module binds only rust-trace-attribute. They therefore are not engine-backed bindings. This extraction limitation is explicitly retained; supplied actual passing host execution is a different kind of evidence. Existing unmatched/upstream/provenance tags are not silently counted as covered local criteria. No Test Matrix was written.

The 180 high findings below report the unchanged whole-repository assurance baseline, not 180 newly introduced PR regressions. No historical repair tickets or unrelated scope expansion was requested. The dispatching ROOT must obtain one driver-specific ARCH baseline decision before merging; the codegen-specific earlier ruling cannot be reused. The audit verdict remains FAIL even if that separate merge exception is approved.

## Examined units

```yaml
scope:
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: version = 4
- id: Cargo.toml
  path: Cargo.toml
  role: examined
  excerpt: '[package]'
- id: cli-contracts/tests/analyze_cases/value_forms.rs
  path: cli-contracts/tests/analyze_cases/value_forms.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: filament-cache/Cargo.toml
  path: filament-cache/Cargo.toml
  role: examined
  excerpt: '[package]'
- id: filament-plugin-host/src/lib.rs
  path: filament-plugin-host/src/lib.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: filament-plugin-host/tests/fixtures/echo-plugin.py
  path: filament-plugin-host/tests/fixtures/echo-plugin.py
  role: examined
  excerpt: '"""Small PL-4 process fixture used by the filament-plugin-host tests."""'
- id: fsl-cli/Cargo.toml
  path: fsl-cli/Cargo.toml
  role: examined
  excerpt: '[package]'
- id: fsl-cli/src/analyze_provider.rs
  path: fsl-cli/src/analyze_provider.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: fsl-cli/src/backend_provider.rs
  path: fsl-cli/src/backend_provider.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: fsl-cli/src/cli.rs
  path: fsl-cli/src/cli.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: fsl-cli/src/process_provider.rs
  path: fsl-cli/src/process_provider.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: fsl-cli/tests/cli_contract.rs
  path: fsl-cli/tests/cli_contract.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: fsl-cli/tests/frontend_cli.rs
  path: fsl-cli/tests/frontend_cli.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: fsl-test-provider/src/main.rs
  path: fsl-test-provider/src/main.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: src/analyze.rs
  path: src/analyze.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: tests/common/dependency_fixtures.rs
  path: tests/common/dependency_fixtures.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: tests/monitor_refusal.rs
  path: tests/monitor_refusal.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: tests/source_record_admission.rs
  path: tests/source_record_admission.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later'
- id: FR-022-AC-7
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: In text mode the AC-2 request prints `0 BelowThree refuted` at the start
    of its first line, `violation` as its last line, and exits 10; an item naming
    an unregistered backend prints `0 Bounded invalid-request invalid_capability/unknown-backend`.
- id: FR-022-AC-11
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'With no `--provider-config`, `fsl analyze` over FSL''s `examples/hybrid-braking/slow-enough.analyze.json`
    with `--format json` writes one item settled `proved` with certification `certified`
    and backend `fsl-hybrid-reference`, category `success`, exit 0, and over `too-slow.analyze.json`
    one item settled `refuted` with backend `fsl-hybrid-reference`, category `violation`,
    exit 10; neither writes `HybridProviderAbsent`, and the document''s `registry`
    lists `fsl-hybrid-reference` and `en-1`. The process-provider half depends on
    FSL''s `Engine::hybrid_process` (FSL FR-750): with `--provider-config` s'
- id: FR-002-AC-22
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '`fsl run Q --format json`, `Q` FSL FR-960-AC-1''s request calling `sq`
    with `x = 3` and `call.requirements` `{}`, with no `--provider-config`, writes
    stdout byte-equal to the canonical encoding of the document `run_with_providers_and_limits_and_overrides`
    returns for `Q` with the providers `[Engine::hybrid_reference()]` and default
    limits, plus a newline: value `{"integer": "16"}` and three settled requirement
    entries, exit 0. With `--provider-config` starting a fixture provider whose admitted
    backend is `fsl-hybrid-reference`, the run refuses `invalid-request` naming `providers`,
    exit 20, beca'
- id: FR-013-AC-5
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: A plugin's valid typed cancellation settlement received within grace is
    returned unchanged; no settlement by grace expiry terminates the child and descendants
    and returns cancellation within the one-second cleanup bound.
- id: FR-013-AC-6
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: A deadline expiring while a plugin does not read a request frame returns
    `TimedOut` within the 20-millisecond polling interval plus the one-second cleanup
    bound.
bindings:
- test_id: fsl-cli/tests/cli_contract.rs::run_routes_requirement_items_with_caller_spine_limits
  ac_id: FR-002-AC-22
  trace: correct
- test_id: fsl-cli/tests/cli_contract.rs::run_refuses_a_configured_provider_with_the_reference_backend_name
  ac_id: FR-002-AC-22
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::analyze_text_mode_prints_item_lines
  ac_id: FR-022-AC-7
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::composition_text_preserves_derived_outcomes_trust_basis_and_extents_from_json
  ac_id: FR-022-AC-7
  trace: correct
- test_id: fsl-cli/src/backend_provider.rs::tests::hybrid_hello_retains_dynamics_and_solver_bounds_through_fsl_admission
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/src/backend_provider.rs::tests::hybrid_hello_refuses_malformed_or_unrecognized_dynamics
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/src/backend_provider.rs::tests::hybrid_hello_preserves_missing_fairness_and_solver_bound_refusals
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/src/backend_provider.rs::tests::hybrid_hello_invalid_fairness_labels_retain_owner_refusals
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_braking_default_proof_and_refutation_have_real_evidence
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_selection_transports_fsl_request_and_keeps_trusted_proof
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_preserves_malformed_results_for_fsl_refusal
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_provider_unsuitable_hello_is_refused_and_started_sessions_are_closed
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_deadline_settles_incomplete_instead_of_accepting_a_late_proof
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_signal_cancels_the_open_item
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_refutation_requires_exact_native_replay
  ac_id: FR-022-AC-11
  trace: correct
- test_id: fsl-cli/tests/frontend_cli.rs::hybrid_process_refuses_nontrusted_plugin_certification
  ac_id: FR-022-AC-11
  trace: correct
```

## Final narrow disposition (second and final reviewer pass)

Reviewed exact latest head `9d57266af4f24c79f6f447e977a1854cf7cfe043` against initial reviewed head `f3711b7ee4408261ba29f0d69357fb791d31f1ee`. The only changed file is fsl-cli/tests/frontend_cli.rs: 187 insertions and 14 deletions. No production source, manifest, pin, original Reservoir oracle or CI change occurred. This section supersedes the initial review's pending-check statements; initial findings and evidence are preserved as history.

The composition fixture now supplies two distinct selected model subjects with two components, the existing driver-owned Counter schema/clauses instantiated for the meter subject, and one complete joint initialization world containing both populations. Both library scopes map both populations. This preserves the real native model route and required minimum-two-component composition grammar; no upstream native program is copied. The transformations consume existing driver fixture data, with no second checked-in semantic fixture definition. All previous three-item JSON/text outcomes, full proof basis, certification, per-premise extent, capability, backend and exit-21 assertions remain.

The former string rfind assertion is replaced by structural native counterexample inspection. It checks the declared population identity, package/node identity, reference type, object identity, original object type, record identity and integer value field. It ties the deciding meter reference to over_binding, asserts actual native replay reproduced=true, checks the full deciding trajectory [1,2,3], independently checks the peer Counter trajectory [0,0,0], and verifies both final values. No assertion is hidden behind a possibly unexecuted guard. This strengthens the CEX oracle and prevents a peer object's value from being mistaken for the deciding subject.

Actual remote evidence supplied by ROOT for this exact head: Titan simple223-9d57-c1 rc=0, selected=1, passed=1, failed=0, ignored=0; simple223-9d57-c2 focused frontend Clippy rc=0. ROOT reports the complete native JSON/text/certification/basis/extents/exit-21 and typed identity trajectory assertions executed successfully. The prior AA71 batch supplies the other 25 passing runtime checks, two passing production builds and three passing focused lint checks; all intervening edits are confined to this frontend test fixture. This is exact-head focused evidence plus preserved unchanged-production evidence, not a claim that the whole runtime batch reran at the latest head. No Cargo, runtime, repeat matrix, baseline or additional review method was run in this final pass.

ARCH explicitly accepts the inherited driver 317-criterion baseline conditionally for this bounded PR223/PR224 change with no new gap. The whole-repository audit remains FAIL: 175 untagged and five ignored-only criteria. The initial exact base/head comparison found no newly missing criterion and two improvements to tagged. No spec or test trace declarations changed in the final fixture diff, so there is no new traceability delta to recompute. The five documented-comment host trace tags remain an explicit active-module extraction limitation; no engine-backed PASS or historical gap repair is claimed.

Merge eligibility: source review and required focused actual evidence permit merge at `9d57266af4f24c79f6f447e977a1854cf7cfe043` under that bounded ARCH decision. QSL-2234's broader combined-provider acceptance is not declared complete. The lead/coder must retain the latest two SR artifacts; PR223 must use a merge commit to preserve stacked child PR224.

Tracker hold: Peter/ARCH forbids Terra Linear queries, comments and state calls until approximately 11:39 PT. This final pass made none. The initial private comments were previously posted and read back; the final disposition update is prepared for ROOT to post once after the hold, with read-back then. No final tracker write/read-back is claimed here.
