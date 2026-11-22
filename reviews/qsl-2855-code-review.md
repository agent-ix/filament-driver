---
id: SR-202611210022442313144108526
title: "Code review — filament-driver FSL pin cohort"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; Cargo.toml, Cargo.lock, fsl-cli/Cargo.toml, filament-cache/Cargo.toml; QSL-2855"
review_set: subset
---

## Summary

Reviewed the frozen QSL-2855 Cargo-only diff, including the dependency manifests and lockfile, using code-review and the Rust-review lane. The direct FSL dependencies now use `86acfde`; Contract IR and filament-exact resolve to their intended pinned commits, and `test-support` remains only in the root dev-dependency declaration.

## Verdict

**PASS** — no defects found in the manifest or lockfile diff. No build or tests were run on luna; Titan's requested workspace check is separate.

## Review Notes

- All 21 locked FSL package source entries resolve to `86acfdeedabe40674067342aaf2f639d48fc15f3`; the Contract IR packages resolve to `14c06b047b6a3b23a4da53723574ad7ce49ba246`, and filament-exact resolves to `a58474cc105a86f3307a11def3fc001e2459679f`.
- The Cargo.lock retains older FSL `rev` selectors in transitive source identifiers, but those entries resolve to the same locked `86acfde` commit. The source code revision therefore remains coherent in the lockfile.
- The root `filament-driver` manifest has no normal `fsl-semantics` dependency with `test-support`; the feature is present in `[dev-dependencies]` only. The CLI and cache manifests also use the new FSL revision.
- No build or tests were run, per the dispatch instructions.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | low | No findings (placeholder) | - |
