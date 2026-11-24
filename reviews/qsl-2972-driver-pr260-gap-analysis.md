---
id: SR-202611210022442313144108534
title: "Gap analysis of QSL-2972 filament-driver PR 260"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; repository matrix and PR #260 changed Rust/dependency paths; QSL-2972"
review_set: subset
relationships:
  - type: references
    target: "ix://agent-ix/filament-driver/spec/spec"
---

## Summary

The PR's new driver behavior is tagged to the externally owned `FSpec-FR-330-AC-17`, and its injected test exercises the host-allocation result through `drive`. The repository-wide matrix still reports substantial existing traceability gaps, and the dependency pin change breaks the local dependency-closure criterion when a workspace consumer is compiled.

## Verdict

FAIL. The full repository matrix has 176 untagged criteria, five tagged only by ignored tests, and ten method-without-symbol entries. The PR also introduces a compile failure for `fsl-cli` against the updated first-party API revisions.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Root dependency pins diverge from workspace consumer pins, causing `fsl-cli` compilation to fail and violating the shared dependency closure. | spec/functional/FR-001-orchestrate-the-driver-steps.md:321; Cargo.toml:59-66; fsl-cli/Cargo.toml:24-32,52-54; filament-cache/Cargo.toml:13,20 |

## Coverage

- Scope: whole-repository matrix (`quire matrix --scope . --format json`; no evidence store was available), changed dependency and production Rust paths, and the new focused test.
- Matrix rollup: 151 tagged, 176 untagged, 5 tagged-by-ignored-test, and 10 method-without-symbol criteria across 28 functional spec documents.
- Changed behavior: `tests/drive.rs::e5_host_allocation_failure_is_not_a_package_record` carries `FSpec-FR-330-AC-17`; it injects `CheckedPackageDispatchResult::HostAllocationFailed` and verifies the retained typed failure, `Handoff` stage, `Fault` kind, `runtime_invariant` code, host-error message details, and no later upstream calls. Production dispatch is handled in `src/handoff.rs`; the final-subject and source-record monitor read paths preserve the error as `Internal`.
- Reverse-gap review of changed behavior found no new unowned behavior beyond the externally owned FSpec criterion. Stub scan (`TODO`, `FIXME`, `todo!`, `unimplemented!`) across changed production/test paths found none.
- `fsl-cli` consumer compile: `CARGO_TARGET_DIR=$PWD/target cargo check --locked -j 8 -p fsl-cli --tests` failed with 99 errors caused by incompatible duplicate FSL public types from stale sibling manifest revisions.
- Semantic review: skipped (optional and not requested).
- Plan completion: not assessed (no plan supplied).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Updated fsl-cli and filament-cache pins now match the root FSL/codegen/IR API revisions; the focused fsl-cli compile passes. |

Round 1 after excerpt (fsl-cli/Cargo.toml): `fsl-analyze = { git = "https://github.com/agent-ix/filament-spec-language", rev = "7527601581610f825d5a3df0e138477f490cf08b" }`
