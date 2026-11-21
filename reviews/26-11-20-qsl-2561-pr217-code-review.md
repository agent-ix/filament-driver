---
id: SR-202612
title: "code-review of QSL-2561 driver pin wave step 2"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver PR217 QSL-2561 at 141e46c9d67e98e7497b8c42bf0f2db9ac151d8f; Cargo.toml, Cargo.lock, fsl-cli/Cargo.toml, filament-cache/Cargo.toml, src/analyze.rs, fsl-cli/src/test_support.rs, fsl-cli/tests/export_cli.rs; Driver FR-022"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-022
    type: references
---

## Summary

Ticket: QSL-2561. Code review with the rust-review lane folded in. The PR moves every filament-spec-language pin to 8f71444d, the contract IR pins to af45a983 and the codegen pin to 556f47c3. It also switches `AnalyzeNegotiation::fsl()` to `Engines::fsl_with_reference_hybrid()` and adapts one `.expect` message to the new `Engine::external` error type.

## Verdict

**PASS with one medium finding.** The lockfile is coherent. It holds one revision each of FSL (8f71444d, 13 packages), contract IR (af45a983, ir + model) and codegen (556f47c3). filament-exact b26a8e4c, semantic-value f5c913bd, observation 39b3ebb1 and checked/eval 2184793a match the pins in FSL 8f71444d's own Cargo.toml and codegen 556f47c3's. The `Engine::external` error-path change is an improvement. FSL now accepts any non-empty advertisement and refuses only absent-kind or value-validity-without-unbounded, and `fsl-cli/src/cli.rs:1896` formats the error through `Display` with no source change. The export_cli.rs import order is correct rustfmt for edition 2021. The hybrid change is only half-wired (FND-001). It changes no CLI-observable output today, so no fixture needed updating.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The negotiation manifest now includes fsl-hybrid-reference, but no driver entry passes `Engine::hybrid_reference()` to analyze_with_providers or run_with_providers. FSL builds its registry from `Engines::fsl()` plus the providers it is given, so the driver negotiates over a registry FSL never builds. This breaks the FR-022 invariant "the same registry FSL computes the candidates from" and the `with_providers` doc. The FR-711-AC-5 test registry was moved to the same unbuilt set. | src/analyze.rs:38, src/analyze.rs:44-55, src/analyze.rs:612-618, fsl-cli/src/cli.rs:1866, fsl-cli/src/cli.rs:2037-2048 |

## Detail

FND-001. FSL 8f71444d `fsl-command/src/command/analyze.rs:479` (`engines_with`) builds `Engines::fsl()` and then adds only the caller's `providers`. FR-1019 and driver FR-022 "Behavior" both require the driver to pass `Engine::hybrid_reference()` first among those providers. In `with_started_providers` (fsl-cli/src/cli.rs:2037-2048), `engines` holds only the started process providers, and `StartedProviders.engines` starts empty at cli.rs:1866. The result is that a hybrid item's candidates, computed from FSL's registry, are still empty, so it settles `unsupported` as before. The negotiation manifest meanwhile advertises a backend that FSL never registered. No output changes today. That is why the change is invisible, and also why the `fsl_with_reference_hybrid` switch achieves nothing.

Failure scenario: the driver negotiates over a manifest that differs from the candidates' registry. CG's FR-711 "function of manifest and item" is evaluated over a phantom entry, and the doc comment at src/analyze.rs:44-47 is now false. The FR-711-AC-5 test (src/analyze.rs:610-618) builds its registry from `fsl_with_reference_hybrid` too. It therefore checks agreement over a registry no `fsl analyze` run uses, and cannot detect this skew.

Fix: either (a) prepend `Engine::hybrid_reference()` to the providers handed to analyze_with_providers and run_with_providers, as FR-022 specifies, or (b) keep `Engines::fsl()` here and leave the whole hybrid default to QSL-2234, whose FR-022-AC-11 test is ignored at cli-contracts/tests/analyze_cases/value_forms.rs:93.

Measured: I rebuilt `fsl` at 141e46c and ran that ignored test with `--include-ignored`. It fails with exit 20 before negotiation: `invalid_package/invalid-value at /temporal_clause_names`, raised by FSL 8f71444d fsl-analyze/src/analyze.rs:795. So wiring (a) alone will not turn AC-11 green. Hybrid package admission in FSL is a second blocker on QSL-2234. The other four value_forms cases pass.

## Coverage

- Cargo.toml, fsl-cli/Cargo.toml, filament-cache/Cargo.toml: every FSL dependency moved; none left on 374fe2a8. The dev-dependencies moved as well.
- Cargo.lock: one source revision per FSL, IR and codegen package. The repeated `agent-ix-semantic-ir`, `agent-ix-semantic-schema` and `quire-rs` names were already there and this diff does not touch them. fsl-hybrid-reference gained serde_json, which comes from upstream.
- fsl-cli/src/test_support.rs:97: an expect message only. It is accurate for the new refusal type.
- fsl-cli/src/cli.rs:1891-1896 (not in the diff): it compiles against `ProviderRegistrationRefusal`, and the message now renders `BackendId("…"): <cause>`. The refusal branch had no test before this PR and has none now.
- Tests run by the reviewer: `cargo test -p fsl-cli --bin fsl -- provider signals`, 25 passed. These are the in-crate `with_provider_request` and `Engine::external` users. Also cli-contracts analyze value_forms with `--include-ignored`: 4 passed, and the 1 ignored QSL-2234 case fails as described above.
- Known reds (2 lib tests on qualification kernel_version) are excluded. I confirmed the cause cheaply: codegen 556f47c3 docs/qualification/ir1530-general-route-scalar-table.json carries kernel_version 1.
- Rust lane: no new unsafe, panics, API or unwraps. The only `expect` is in test support.

## New findings (disposition pass 1)

Re-checked at 3f8b20fbc9dcb82866738635b34528cfefd8af33 (fix commit 3f8b20f).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | `fsl run` with no `--provider-config` or `--provider-limit` still skews. The package and extracted-clause paths negotiate with `AnalyzeNegotiation::fsl()`, whose manifest includes fsl-hybrid-reference, but call FSL's `run_with_limits_and_overrides`, which builds `Engines::fsl()` with `providers: Vec::new()`. FR-022 says `fsl run` registers the reference provider whenever it settles requirement items. Fix: call `run_with_providers_and_limits_and_overrides` with `run_engines(Vec::new())`. | fsl-cli/src/cli.rs:1038-1052, fsl-cli/src/cli.rs:1085-1099 |
| FND-003 | low | The test helper `with_provider_request` negotiates over `AnalyzeNegotiation::with_providers([&descriptor])`, which includes the hybrid manifest, but hands FSL only `vec![engine]`. That is the same skew in test support, so provider tests run a registry the CLI no longer builds. Fix: hand FSL `run_engines(vec![engine])`. | fsl-cli/src/test_support.rs:98-104 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 3f8b20f adds `run_engines` (src/analyze.rs:68-82), which returns `Engine::hybrid_reference()` followed by the providers. `with_started_providers` (fsl-cli/src/cli.rs:2048-2054) routes the started engines through it, so analyze, `run_with_providers` and the other `with_started_providers` users (cli.rs:1971, 2074, 2129, 2467) now hand FSL the same registry the manifest names. The FR-711-AC-5 test now builds `Engines::fsl()` plus `run_engines` the way FSL's `engines_with` does, and asserts that the backend ids equal the manifest ids before comparing dispositions, so the test now catches this skew. The `with_providers` doc is corrected. The `fsl run` paths without provider flags were outside FND-001's named entries and are recorded as FND-002. |

Verification in this pass: I ran `cargo test --locked -p fsl-cli --test cli_contract` at 3f8b20f before the disk hold removed the target. Result: 57 passed, 0 failed, 1 ignored, the same as before the fix. The cli-contracts `analyze_cases` rerun did not happen: no build is allowed under the disk hold, and the code lead will run it before merge. I did not rerun the driver lib tests; the coder's fix1.log reports 124 passed and the 2 known reds.

## New findings (disposition pass 2)

Re-checked at 11b747fd14d2562150564c26e21ceca944b76ffd by reading the code only. The PR was merged as e858b6e8 while this pass ran, and no build or test ran in this pass.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The fsl-cli parity tests build their library-side reference over a different registry from the one the CLI now runs. The references call `command::analyze` or `command::monitor` with `AnalyzeNegotiation::fsl()` and no providers, so FSL builds `Engines::fsl()`. The CLI they are compared against now runs `run_engines(..)`, which includes fsl-hybrid-reference. Either the parity assertions now fail, or they compare runs over different registries without noticing. These tests are frontend_cli and monitor_cli, which the coder's checks did not include; this pass did not run them either. Fix: hand the library side `run_engines(Vec::new())` through `analyze_with_providers` and `MonitorProviders.providers`. | fsl-cli/tests/frontend_cli.rs:912-916, fsl-cli/tests/frontend_cli.rs:1394-1398, fsl-cli/tests/monitor_cli.rs:113-117 |

## Dispositions (round 2)

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Both `fsl run` sites without provider flags (fsl-cli/src/cli.rs:1047-1062 and 1105-1120) now call `run_with_providers_and_limits_and_overrides` with `run_engines(Vec::new())`. In FSL 8f71444d, `run_with_limits_and_overrides` is that same entry with no providers, `ProviderOptions::default()` and a fresh `Cancel`, so the only change is the added hybrid engine. A `run_engines` error returns an InternalFailure result instead of panicking. |
| FND-003 | fixed | `with_provider_request` (fsl-cli/src/test_support.rs:103) hands FSL `run_engines(vec![engine])`, which matches its `with_providers([&descriptor])` negotiation. |
