---
id: SR-17940486709564847047965
title: "Code and Rust review of QSL-2214 driver dependency pins"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; QSL-2214; PR #191 at 38234ed3c4e05c7d6f7b51df77a3e304a3aad91f; Cargo.toml, Cargo.lock, filament-cache/Cargo.toml, fsl-cli/Cargo.toml, tests/frame_replay.rs"
review_set: subset
evaluated_revision: "38234ed3c4e05c7d6f7b51df77a3e304a3aad91f"
---

## Summary

Ticket: QSL-2214. Read-only Rust review of PR #191 at the exact supplied head. Every FSL git dependency in the driver, cache, and CLI manifests resolves to `13b0b11e7b00e7412976d7a4b55cc93aed528c38`; every contract-codegen manifest dependency resolves to `ed0b05ee9cdd025fe74f3120b64725a02b529a7e`; Cargo.lock records those exact revisions. The sole Rust source change moves the frame replay fixture's `ObligationIdentity` import to its public owner, `fsl_package`. The shared manifests do not overlap PR #189's changed test/fixture files. The PR title has no ticket key and its body contains `Refs QSL-2214`.

## Verdict

PASS. No findings. The focused library/binary check, frame replay test (5/5), dependency edge test (1/1), PR #189 result regression tests (9/9), rustfmt check, and diff check pass. A focused `cargo check -p fsl-cli --tests` reproduces the PR-documented unrelated E0061 in unchanged `fsl-cli/src/cli.rs:2806`; `write_result_diagnostics` at line 878 requires an `OutputFormat` argument. The failure is accurately documented, and no repair to that file is in this diff. No aggregate gate was run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| PR-191-pin | examined | Pin all driver workspace and member FSL dependencies to `13b0b11e7b00e7412976d7a4b55cc93aed528c38` and contract-codegen to `ed0b05ee9cdd025fe74f3120b64725a02b529a7e`; refresh Cargo.lock. |
| PR-191-repair | examined | The frame replay integration test imports `ObligationIdentity` through `fsl-package`, its public owner in the selected FSL revision. |
| PR-191-compatibility | examined | Focused verification covers the affected frame replay and dependency graph paths; no aggregate gate is run. |
| PR-189-result-regressions | examined | PR #189 adds FSL run-result coverage for Boolean, integer, Rational, Decimal, Text/String, enum, record, Sequence, Option, unsupported result, and related CLI behavior. |
| PR-191-ticket-reference | examined | Title: `Pin driver to matching FSL and codegen revisions`; body begins `Refs QSL-2214`. |
| FSL dependency closure | examined | All direct FSL dependencies in Cargo.toml, filament-cache/Cargo.toml, and fsl-cli/Cargo.toml use revision `13b0b11e7b00e7412976d7a4b55cc93aed528c38`; corresponding lock entries use that revision and commit hash. |
| Codegen dependency closure | examined | Direct contract-codegen dependencies in Cargo.toml and fsl-cli/Cargo.toml use revision `ed0b05ee9cdd025fe74f3120b64725a02b529a7e`; Cargo.lock records that revision and hash. |
| Compile diagnostic | examined | `fsl-cli/src/cli.rs:2806` is unchanged and calls `write_result_diagnostics` with two arguments while its local signature requires three, including `OutputFormat`. |

Rust style baseline and repository conventions were read; the changed Rust code adds no API, unsafe code, implementation branch, or test behavior. The only compile repair is the owner-correct import made necessary by the selected pin. PR #189's `frontend_cli` `run_returns` tests passed (9/9) under these pins, with no file overlap between its changes and this PR. The documented all-target failure was reproduced with the focused `-p fsl-cli --tests` check, confirming its location and argument mismatch.
