---
id: SR-2026111601185103
title: "Code review of filament-driver PR 158 language-checkout gate build"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; Makefile, Cargo.toml, CLAUDE.md, tests/examples_gate.rs, tests/make_targets.rs at 3a2129544bc36d568248ba5f25bc3c8892bbda52"
review_set: subset
evaluated_revision: "3a2129544bc36d568248ba5f25bc3c8892bbda52"
---

## Summary

Ticket: QSL-2101. Code review with the rust-review lane (rust-style loaded) of `git diff origin/main...HEAD`. No cargo or make was run; the review is by reading.

## Verdict

REQUEST CHANGES. `language_patches` derives the patch set from every package id that `cargo metadata --locked` resolves from the language git source. Cargo.lock has one such source with 10 entries. A crate missing from the checkout panics. AC-7 asserts that no built artifact comes from the language git source, so no language package can be left on git main. The rename to `examples-check` has no stale references. Cargo.toml changes only comments, so the ADR-011 edges are unchanged. No workflow trigger is added, and there is no Python and no pin. One medium defect: the Cargo.lock restore is not interrupt-safe.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Cargo.lock is restored only in `LockRestore::drop`. A SIGINT, SIGKILL or abort during the patched `cargo build` leaves the driver checkout with a rewritten lock. Until the build finishes, any concurrent `cargo --locked` in the same tree also reads the rewritten lock. Fix: never rewrite the checkout's lock. Run the patched build in a temporary copy of the workspace manifests and lock, with CARGO_TARGET_DIR set to this workspace's target dir. At minimum, journal the snapshot to `target/` before the build and restore it at the start of every gate run. | tests/examples_gate.rs:187-206,227-233 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The patched build from the copy shares this workspace's target dir. Cargo puts the final binary at the same `target/debug/fsl` as a plain build, so a concurrent unpatched driver build in the same tree can swap the binary between the build and the gate run, after the marker line's package check has passed. Fix: pass `--target-dir <target_directory>/language-build`. The binary still lies under the workspace target directory, so AC-6 holds. | tests/examples_gate.rs:241-254 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 730c2ee7, the patched build runs from a copy under CARGO_TARGET_TMPDIR (`mirror`, `--manifest-path <copy>`). The checkout's Cargo.lock is never written, and AC-7 now asserts it is byte for byte unchanged. An interrupt can only leave the copy's lock rewritten. |
