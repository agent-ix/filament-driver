---
id: SR-1521
title: "Code and Rust review of QSL-2289 cancellation synchronization"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; filament-plugin-host/src/lib.rs; FR-013-AC-5; FR-013-AC-9; TC-025"
review_set: subset
---

## Summary

Reviewed the test-only cancellation synchronization diff with code-review and rust-review. No defects found; production timing policy and the real OS child fixture are unchanged.

## Verdict

**PASS** for the frozen diff. Admission completes before the watcher starts, while the unchanged fixture emits its marker only after consuming the request and starting its descendant. Cancellation therefore exercises active-request grace rather than racing hello admission. The timestamp precedes cancellation publication, so waiting a full configured grace cannot become a tight lower-bound race. The upper bound allows the full one-second cleanup budget beyond grace, and is looser than the previous one-second total assertion. Reader/writer join diagnostics, process-tree cleanup diagnostics and Linux nonrunning-state checks preserve substantive cleanup evidence rather than accepting only the Cancelled enum.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Frozen source: `905e8381ff6f924dd18f618e99409893b86c9230`. Ticket: QSL-2289. PR: agent-ix/filament-driver#211 (public).

- Examined: `filament-plugin-host/src/lib.rs:1599-1616`, the watcher returning its cancellation timestamp, and `1831-1894`, the grace-expiry test. Context: request cancellation/grace path (`550-645`, `764-818`), abort diagnostics (`696-735`), Linux process sampling (`1120-1141`), and unchanged fixture (`tests/fixtures/failure-plugin.py:167,264-271`).
- FR-013-AC-5 (examined): “A plugin's valid typed cancellation settlement received within grace is returned unchanged; no settlement by grace expiry terminates the child and descendants and returns cancellation within the one-second cleanup bound.” Review covers the changed no-settlement branch only.
- FR-013-AC-9 (examined): “Stderr beyond the session retention limit is drained but not retained; stderr is available on failures, and reader/writer workers are joined within one second after termination begins.” Review covers the changed cleanup assertions only; stderr behavior is context outside the diff.
- TC-025 (context_only): “Use synchronization handshakes rather than sleeps”; the existing positive ready-file observation remains the synchronization oracle.

Reported author evidence accepted without rerun: focused changed test PASS (1 passed, 22 filtered, one run); scoped clippy `--tests -- -D warnings` PASS; fmt PASS. No reviewer runtime checks. Final affected-component gate remains the lead's post-review action.
