---
id: SR-1443
title: "QSL-2468 native CLI contract harness code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; QSL-2468; Makefile, cli-contracts/Cargo.toml, cli-contracts/Cargo.lock, cli-contracts/README.md, cli-contracts/src/lib.rs, cli-contracts/tests/contracts.rs"
review_set: subset
---

## Summary

Reviewed PR #197 at `60c06a357aea612c0bfb65c1a351f1eb5967260a`, including its native Rust subprocess, capture, fixture and decoding API and its real-FSL examples. The explicit prebuilt binary and standalone dependency graph meet the assigned placement constraint. One process-lifecycle edge leaves the documented execution bound incomplete.

## Verdict

**PASS** — disposition round 1 resolves the capture-lifecycle finding at `82bf22d002fc23deb310a3c1368675de54b78fc9`. The lead's one final eight-test component gate remains pending.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The 30-second timeout only waits for the immediate child; the subsequent stdin/stdout/stderr joins are unbounded. If FSL exits or is killed while an engine/provider descendant retains a pipe, a reader or writer can remain blocked and the scoped join prevents `run_stdin` from returning, including after the stated timeout. Bound the complete I/O lifecycle and ensure owned processes and workers terminate rather than waiting indefinitely for descendant-held pipe EOF. | cli-contracts/src/lib.rs:215; cli-contracts/src/lib.rs:233; cli-contracts/README.md:17 | implementation-bug-despite-evidence |

## Coverage

Rust review covered the deliberate public API and typed errors, original fixture confinement, concurrent stdin/stdout/stderr handling, child wait/kill/reap paths, bounded retained output, JSON/JSONL and raw-text decoding, the Make target, manifest/lockfile and README. No product dependency, fallback executable, copied binary or alternate engine was introduced. The accepted five new real-binary checks and scoped Clippy result were used without repetition. The remaining final gate is not claimed as executed by this review.

No applicable AssuranceProfile was identified in the scoped repository context. No code or frozen-branch files were changed by the reviewer.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: The capture path no longer spawns I/O workers or joins them. Nonblocking stdin/stdout/stderr are polled in the caller thread under one elapsed-time budget; the invocation guard terminates its own process group and reaps the direct child on timeout/error. The reported three real OS lifecycle tests and current scoped Clippy passed; neither was rerun by the reviewer. |
