---
id: SR-202611210022442313144108533
title: "Code review of QSL-2972 filament-driver PR 260"
type: SpecReview
analysis: base
scope: "agent-ix/filament-driver; Cargo.toml, Cargo.lock, src/handoff.rs, src/refusal.rs, src/monitor/error.rs, src/monitor/final_subject.rs, src/monitor/source_record.rs, tests/drive.rs, tests/frame_replay.rs, tests/source_record_admission.rs; QSL-2972"
review_set: subset
---

## Summary

Reviewed PR #260 at `c348f0097af1ccd05ba7a35e171c25e521b3d211`, including the Rust paths handling checked-package host allocation failure and the injected driver test. The host failure is represented distinctly as a handoff fault, but the root dependency revision update leaves workspace consumers on incompatible first-party revisions and breaks `fsl-cli` compilation.

## Verdict

CONDITIONAL. Do not merge until the workspace dependency revisions are aligned and `fsl-cli` compiles against the updated public types.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Updating only root dependency pins leaves workspace crates on incompatible FSL/codegen/IR revisions, so `fsl-cli` no longer compiles. | Cargo.toml:59-66; fsl-cli/Cargo.toml:24-32,52-54; filament-cache/Cargo.toml:13,20 |

### FND-001 details

`cargo check --locked -j 8 -p fsl-cli --tests` fails with 99 errors after the root manifest moves FSL to `7527601581610f825d5a3df0e138477f490cf08b`, codegen to `7959d66196befe3a2ff60994db44c0deaf5fb419`, and Contract IR/model to `729a6641222da57f2e6f722af7bbd47b613b2241`. `fsl-cli/Cargo.toml` still pins FSL at `4b743f6760dae7c5d49a363c5754b21a0dc02f5c`, codegen at `c4e851ed8c072969e18128ab2da80698f09d2ca3`, and IR/model at `ddbc69ef7738df1618df8a5dc6cf5a01705f61b8`; `filament-cache/Cargo.toml` also keeps FSL at the old revision. The compiler reports duplicate incompatible `fsl_analyze` and `fsl_foundation` types. Align the sibling manifests with the root API revisions and refresh the lockfile.

Reviewed the `FSpec-FR-330-AC-17` injected test and handoff/refusal projection: the failure retains its typed stage/locus/requested/consumed data, maps to `Fault`/`runtime_invariant` at `Handoff`, and does not become a package refusal or limit. No defect found in those reviewed paths.

### Review scope

- `FSpec-FR-330-AC-17` (external FSpec criterion): “When a package read's admitted allocation cannot be provided by the host, the reader returns `HostAllocationFailed` with its stage, locus, requested size and consumed count; the result creates no ContractPackage or `feature_accounting` record, and the tool reports a host error. The failure is not `body_incomplete` and does not change that record's closed shape.” — examined in `tests/drive.rs` and `src/handoff.rs`, `src/refusal.rs`, and monitor admission paths.
- `FR-001-AC-1` (local dependency closure criterion): “The driver's normal and build dependency set matches FSL ADR-011 §6.1 and the specified driver-layer exceptions: it contains exactly `fsl-analyze`, `fsl-foundation`, `fsl-package`, `fsl-replay`, `fsl-route`, `fsl-semantics`, `filament-exact`, `filament-contract-codegen`, `filament-contract-model`, `filament-contract-runtime`, `quire-canonical`, `filament-observation`, `filament-semantic-value`, `serde`, `serde_json` and `thiserror`; `serde_json` is required for production projection of FR-331's retained optional `bounds` member and RT supplies FR-015/FR-017's native retained program/arena types. RT's normal/build closure contains no FSL package, IR/model, driver or codegen backedge. The driver's `filament-contract-model` is the one CG's IR crate depends on, CG's FSL edges are exactly `fsl-package` and `fsl-replay`, and the lock holds exactly one `filament-contract-model` package, so the driver's direct edge, CG's IR crate and FSL's `fsl-package` all resolve the same package id.” — examined in root and workspace manifests and via a focused consumer compile.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Updated fsl-cli and filament-cache pins now match the root FSL/codegen/IR API revisions; the focused fsl-cli compile passes. |

Round 1 after excerpt (fsl-cli/Cargo.toml): `fsl-analyze = { git = "https://github.com/agent-ix/filament-spec-language", rev = "7527601581610f825d5a3df0e138477f490cf08b" }`
