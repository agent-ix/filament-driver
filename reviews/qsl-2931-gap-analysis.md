---
id: SR-202611210022442313144108531
title: "gap-analysis of filament-driver PR #249 help and refusal messages"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; FR-002-AC-29..FR-002-AC-32, FR-002-AC-23, FR-003-AC-9 against git diff origin/main...cf598ead5af9c6f8b352e9a75569facefd1aa5d9"
review_set: subset
---

## Summary

Ticket: QSL-2931. Planless gap analysis scoped to the PR diff. `quire matrix --scope .`
(quire 0.38.3) reports FR-002-AC-30, AC-31 and AC-32 as `tagged`, each bound only to
`cli-contracts/tests/messages.rs` tests, and those tests pass (8/8) against the built
binary. There are no stubs and no coverage inflation; the assertions are exact
`assert_eq!` on full message text. The gaps are a partly evidenced statement and two
unowned behaviours.

## Verdict

CONDITIONAL. No untagged criterion and no high finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-002 Outputs bullet "an unknown `--limit` setting refuses naming the operand once and the settings the command accepts, grouped by stage" covers every verb, but AC-31 and its test pin only `check`; on run/analyze/monitor/replay/export the code lists all FSL settings, so the statement is unverified there and the code contradicts it. | FR-002-AC-31; cli-contracts/tests/messages.rs:81-93; fsl-cli/src/cli.rs:807-827 |
| FND-002 | low | Reverse gap: `fsl help <unrouted verb>` (root help, exit 0), `fsl help <verb> <extra>` ("help takes at most one verb", exit 20) and `fsl help <non-verb>` (unknown-verb refusal) are code paths no FR-002 statement or AC owns. | fsl-cli/src/cli.rs:397-419 |

## Coverage

- FR-002-AC-30: tagged; `a_non_utf8_argument_after_a_verb_is_refused_on_stdout_a_non_utf8_verb_on_stderr`, exact stdout, empty stderr, exit 20, plus the first-argument stderr case.
- FR-002-AC-31: tagged; four tests with exact text for unknown verb, nearest verb, unknown option with nearest and list (check, run, compile), engine list, and the check limit list.
- FR-002-AC-32: tagged; `help <verb>` byte-equal to `<verb> --help` for all ten executing verbs, root-help verb lines, and the exact `Operands:`/`Exit codes:` tail.
- FR-002-AC-23: the new bare-inspect test adds the no-operand forms, which existing `inspect_cases/options.rs` tests (which pass a path) do not cover.
- Semantic review: skipped (not requested).
- Plan completion: not assessed
