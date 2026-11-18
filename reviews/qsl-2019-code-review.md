---
id: SR-17940486709564847047963
title: "Code and Rust review of QSL-2019 Markdown CLI"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; fsl-cli/src/cli.rs, fsl-cli/tests/cli_contract.rs, fsl-cli/Cargo.toml, Cargo.lock; QSL-2019; PR #181 at 56fe07ed4f9dc7b15d99eb9d7b7534af1b449e18"
review_set: subset
evaluated_revision: "56fe07ed4f9dc7b15d99eb9d7b7534af1b449e18"
relationships:
  - {target: "ix://agent-ix/filament-driver/FR-002", type: references}
---

## Summary

Ticket: QSL-2019. Read-only review of PR #181 from base `5aad4bb0c7a177101ad76191c80e780edb8d18fd` to exact head `56fe07ed4f9dc7b15d99eb9d7b7534af1b449e18`. The change adds Markdown check dispatch and CLI outcome tests; the supplied focused test and rustfmt checks passed. The extracted-run test is blocked by the locked FSL revision rejecting `program.clauses`; that cross-repository dependency is recorded as In Progress, not as a driver finding.

## Verdict

CONDITIONAL. Two medium findings affect source-read identity retention and the exact diagnostic-code assertion in the new CLI regression.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `Source::read_typed` failures are discarded and retried through `command::check_outcome`, which cannot receive the requested document identity; invalid UTF-8, BOM/NUL, or blank labels can therefore produce a fallback diagnostic under a different source identity. Preserve the typed source-read refusal with the requested identity. | fsl-cli/src/cli.rs:1218-1234 |
| FND-002 | medium | The CLI regression only requires a nonempty diagnostic code, so it passes if the command changes the required `ill_typed` code to another code. Assert the expected code at the CLI boundary. | fsl-cli/tests/cli_contract.rs:274-279 |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-002-AC-3 | examined | In machine mode, each command writes one parseable `filament-outcome/1` value equal to the library outcome under FSL's schema; the CLI adds, removes or changes no outcome data. |
| FR-662-AC-1 | examined | A Markdown document with a healthy and ill-typed native fence gives a healthy result and a `Broken` refusal `ill_typed`, retaining the unit identity, body span, and original Markdown line and column. |
| FR-662-AC-2 | examined | Duplicate and headless native fences refuse independently, other fence outcomes remain unchanged, repeated checks are equal, and a cancelled API call returns `StageFailure::Cancelled`. |
| FR-662-AC-3 | context_only | The FSL command-owner default build runs a selected extracted clause; the driver PR adds a CLI test for this path, but its execution is blocked by the locked FSL revision. |

The rust review inspected only the four changed files and did not run tests or broad gates. The dispatch supplied `cargo test -p fsl-cli --test cli_contract check_markdown -- --nocapture` (2 passed) and rustfmt success. No CI workflow or aggregate/package gate changed. The computed driver matrix has no local FR-662 row; FR-662 is owned by the FSL repository. The run-extraction test remains blocked because locked FSL revision `6f5d277` rejects `program.clauses` as an unknown field; this is not attributed to the driver diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `check_markdown_source` now maps `Source::read_typed`'s `SourceReadRefusal` directly through `source_refusal` and serializes that diagnostic, preserving the requested source identity. The new invalid-UTF-8 CLI regression compares the output with the typed refusal and checks `agent-ix/doc`. |
| FND-002 | fixed | The CLI regression now asserts `diagnostic["code"] == "ill_typed"`. |

## Verdict (disposition pass 1)

PASS at `697f66b2ecec89f6b0930954ded1b32db623ee1d`. Both findings are fixed, and no new driver defect was found in the disposition diff. The coder reports three focused CLI tests and changed-file rustfmt passing; I did not rerun them. The extracted-run test remains blocked by locked FSL and is tracked separately, not as a driver finding.

## Verdict (disposition pass 2)

PASS at `14d2519b22bef22c6206a4eba102011d83f5d4b1`. Rebase and lockfile refresh leave both prior fixes intact: source-read refusals retain their typed identity, and the CLI regression asserts `ill_typed`. No new driver finding was found in the updated PR diff. The coder reports three focused CLI tests and changed-file rustfmt passing; I did not rerun them. The extracted-run criterion remains blocked pending FSL #2891 and is tracked separately.
