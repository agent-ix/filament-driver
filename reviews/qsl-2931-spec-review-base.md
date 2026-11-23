---
id: SR-202611210022442313144108532
title: "spec-review (base, with EARS check) of FR-002, TC-049 and tests.md edits in filament-driver PR #249"
type: SpecReview
analysis: base
scope: "agent-ix/filament-driver; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md; spec/test-cases/TC-049-list-every-accepted-option-in-help.md; spec/tests.md"
review_set: subset
---

## Summary

Ticket: QSL-2931. Base checklist plus an EARS check of the new statements, scoped to the
diff. `quire validate --scope .` on the three files exits 0. AC-30, AC-31 and AC-32 can
each fail: they fix the stream, the exit code and the exact text, and the bound tests
use `assert_eq!` on full bytes. The edits agree with FR-002's one-stream rule. AC-29 is
narrowed only in the Outputs bullet ("its first argument"), which matches AC-29's
existing text. The decisions are sound. QSL-2929: a non-UTF-8 argument after a
recognized verb stays on stdout. FR-002's Behavior text and AC-25/AC-28 give `prove`,
`generate` and `render` `--help` exit 21. Bare `inspect` remains not routed, exit 21,
per AC-23 and FSpec FR-522. So QSL-2931 item 5 and QSL-2933 rightly need a spec change
before any code change. The `--limit` unknown-setting text still names the operand and
its cause, so it meets FSL FR-869's Behavior bullet and FR-869-AC-4 (FR-255 Behavior 6's
usage refusal naming the operand). The PR body's citation of FSL FR-255-AC-6 is wrong:
that AC covers defaults. Driver FR-002-AC-13 holds because text and JSON carry the same
message.

## Verdict

CONDITIONAL. One medium (shared with the code review) and four low defects in wording
and layout.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new Outputs bullet says an unknown `--limit` setting refuses naming "the settings the command accepts, grouped by stage" for every verb; the implementation does so only for parse/check/compile/format, and AC-31 tests only `check`, so the bullet is false for run/analyze/monitor/replay/export and unverified. | FR-002-AC-31; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:101-103 |
| FND-002 | low | TC-049 step 13 is indented four spaces, so it renders as a continuation of step 12 instead of its own numbered step. | spec/test-cases/TC-049-list-every-accepted-option-in-help.md:68 |
| FND-003 | low | TC-049 says `cli-contracts/tests/messages.rs` covers "steps 11 to 15"; step 11 (AC-29, including `fsl not-a-verb --format json`) is not in messages.rs and is bound elsewhere. The line should say steps 13 to 15. | spec/test-cases/TC-049-list-every-accepted-option-in-help.md:77 |
| FND-004 | low | AC-31 nests backticks inside single-backtick code spans (`unknown verb `bogus`; ...`, `; did you mean `check``), so the exact pinned text renders broken and reads ambiguously. Use double-backtick spans. | FR-002-AC-31 |
| FND-005 | low | EARS: `help <verb>`, nearest-match refusals and the `Operands:`/`Exit codes:` sections are stated only as Outputs bullets. The Behavior SHALL statements still say root help for `help` "as its sole argument" and "Root help SHALL keep its current layout" without the new sections, so no SHALL statement owns AC-31/AC-32, and `help <unrouted verb>` and `help a b` stay unspecified. | FR-002-AC-32; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:291-330 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | FR-002 Behavior still opens with "Help applies only to verbs that execute operations:" and then says `prove`, `generate` and `render` write the not-available help for `--help`/`-h`, so the lead clause contradicts the rule that follows it. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:354-358 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The Behavior statement now says "the settings that verb accepts", and gives `monitor`/`replay` their "accepts no --limit settings" text. AC-31 lists every verb's set, and the code matches. |
| FND-002 | fixed | TC-049 step 13 is now a top-level numbered step. |
| FND-003 | fixed | The TC-049 test line now names steps 8, 13, 14, 15 and 16. messages.rs covers step 8's unrouted-help part, and help_contract.rs covers the rest. |
| FND-004 | fixed | AC-31 now uses double-backtick spans around text that contains backticks. |
| FND-005 | fixed | The new behaviour moved from Outputs bullets to Behavior SHALL statements (help word, unknown verb, unknown option/engine/limit, Operands/Exit codes, unrouted help). The root-help statement now says `help` "alone", so AC-31 and AC-32 have owning EARS statements. |
