---
id: SR-202611210022442313144108518
title: "Code review — filament-driver dependency pin cohort"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; Cargo.toml, fsl-cli/Cargo.toml, filament-cache/Cargo.toml, Cargo.lock; QSL-2855, QSL-2835"
review_set: subset
---

## Summary

Reviewed the frozen manifest and lockfile changes for QSL-2855 and QSL-2835. The direct FSL and Contract IR pins moved and `test-support` was removed from normal dependencies, but the resolved dependency graph still carries stale and inconsistent FSL/IR sources, and the consumer manifest guard required by QSL-2835 is absent. No build or tests were run on luna; the owner assigned those checks to titan.

## Verdict

**CONDITIONAL** — two medium findings remain before merge: the pin cohort is still split through the pinned codegen dependency, and the consumer guard acceptance is not implemented.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | medium | The lockfile still resolves a split FSL/Contract IR cohort through the pinned codegen dependency, and records old FSL rev queries with the new commit hash. | Cargo.toml:60; Cargo.lock:467-476, 529-531, 862, 1072 |
| FND-002 | medium | The driver removes `test-support` from normal dependencies but adds no guard that fails when a consumer manifest enables it again. | Cargo.toml:59-75 |

## Review Notes

- `rust-review` was applied to the manifest and lockfile diff. Rust compile, lint, and test gates were not run because the release owner requires compilation and tests on titan; no compile/test was run on luna.
- `gap-analysis` was not applicable because this diff contains no production Rust code or spec/plan changes. Acceptance criteria were checked directly against the manifest and lockfile diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The driver now pins codegen at c6fc722e, and Cargo.lock resolves the codegen dependency graph to FSL 3a81f1e8 and Contract IR 14c06b04. |
| FND-002 | fixed | Graph::load now rejects normal fsl-semantics/test-support edges from Cargo metadata; the new focused test covers both normal and dev-only edges. |
