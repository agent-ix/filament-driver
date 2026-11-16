---
id: SR-2026111606100002
title: "Spec review of filament-driver PR 159 FR-022 and TC-036"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-driver; spec/functional/FR-022-analyze-a-native-analyze-request.md, spec/test-cases/TC-036-analyze-from-the-command-line.md, spec/spec.md, spec/tests.md at fa6ea64c0a73d35fea2cfff60c425a1557b06ba4; language FR-281, FR-287, FR-620, FR-621, FR-622, FR-624 at dcf3165"
review_set: subset
evaluated_revision: "fa6ea64c0a73d35fea2cfff60c425a1557b06ba4"
relationships:
  - target: ix://agent-ix/filament-driver/FR-022
    type: reviews
  - target: ix://agent-ix/filament-driver/TC-036
    type: reviews
---

## Summary

Ticket: QSL-1976. Spec review, with the integrity and dependency sub-analyses, of the new driver FR-022 (8 ACs) and TC-036 against language FR-281, FR-287, FR-620, FR-621, FR-622 and FR-624 at dcf3165. `PATH= quire validate --scope . 'spec/**/*.md'` exits 0 from the driver root. It reports no diagnostic for FR-022 or TC-036. The only warnings are the existing FR-019 EARS warnings and the module's duplicate-archetype notices. FR-022 and TC-036 are indexed in spec.md and in TM-001.

Rulings. (1) Negotiating through `fsl_route::settle_candidates` deviates from FR-281: see FND-001 and FND-002. (2) No spec requires signal cancellation for analyze. FR-620 makes the driver's `&Cancel` the handle. FR-624 lists `cancelled`/`requested` without saying what triggers it. Driver spec.md section 5 keeps CLI signal triggers planned. Driver FR-002-AC-7 makes a signal during a plugin session re-raise with no document. Deadline cancellation meets the ticket AC's "timeout and cancellation" through FR-624's "cancellation or deadline" row. FND-004 asks for that decision to be recorded in FR-022 rather than only in the PR body.

## Verdict

REQUEST CHANGES. FR-022 is well formed, and its ACs are testable and traced. Three medium spec defects remain. FR-022 writes down a settlement point that FR-281 does not name. That settlement drops FR-290's extent rows. And FR-022 forbids the `--provider-config` option that FSL FR-287 gives `analyze`. Each needs either a change here or a language-side amendment through a spec ticket. The coder must not guess which.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Deviation, not conformance. FSL FR-281 "Negotiated items" names CG's `negotiate_*` as the single negotiation point (FSpec AD-016): the caller "settles each item with `negotiate_*`". FR-022's Decision instead makes `fsl_route::settle_candidates` the settlement, which is a second statement of the FR-290 table. The driver already calls CG `negotiate_contract_package` for `run` (src/upstream.rs:213). Fix: either settle each analyze item through CG's negotiation, as drive.rs does, or have FR-281 amended in the language repo, through a spec ticket, to name the `Negotiate` seam's `settle_candidates` settlement as conformant. Until one lands, mark FR-022's Decision as a deviation naming that ticket. | spec/functional/FR-022-analyze-a-native-analyze-request.md:41-51; src/analyze.rs:1-10 |
| FND-002 | medium | The settlement ignores the item's extent and mode. FR-281 (FSpec FR-290-AC-6) settles an unbounded extent on a candidate that advertises only `bounded` as `requires-bound`, or as `unsupported` with `unbounded-extent`. `settle()` reads only the candidate count and returns `Supported`. FSL then refuses with `inconsistent-candidates`, a different record. FR-022's table has no extent rows. Fix: add the two extent rows to FR-022's table, and in `settle()` read `item.extent` against the candidate's advertised modes, with a unit test per row. Alternatively, resolve this with FND-001 by settling through CG's `negotiate_*`. | spec/functional/FR-022-analyze-a-native-analyze-request.md:84-94; src/analyze.rs:43-48 |
| FND-003 | medium | FSL FR-287 requires `--provider-config <file>` on `analyze` (FSpec FR-561), so that configured plugins register before negotiation (FR-635). FR-022 states "the driver adds no option of its own beyond `--format`", and `AnalyzeOptions::parse` refuses `--provider-config` as an unknown option. `command::analyze` takes no registry or engine set, so the driver cannot honour the option today. Fix: record the gap in FR-022 as a deviation naming a ticket for the FSL API (an analyze entry that takes the caller's registry or providers), and keep the refusal until then. Do not state that no other option exists. | spec/functional/FR-022-analyze-a-native-analyze-request.md:35-39; fsl-cli/src/cli.rs:1348-1359 |
| FND-004 | low | FR-022 says the driver passes "a fresh cancellation handle" but does not say that nothing cancels it except `deadline_ms`. So FR-624's `cancelled`/`requested` cause cannot be reached from the CLI, and SIGINT ends the process by its default disposition with no document. The PR body records this deviation, but the spec does not. Fix: add one Behavior bullet stating that SIGINT and SIGTERM are not cancellation triggers for `fsl analyze` yet (spec.md section 5, CLI signal triggers planned), and naming the ticket that will wire them. | spec/functional/FR-022-analyze-a-native-analyze-request.md:68-69 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The `settle_candidates` deviation names the two extent rows not applied (`requires-bound`, and `unsupported` with `unbounded-extent`). It does not say what a user gets instead. The driver settles such an item `Supported`, and FSL's analyze then refuses it `invalid-request` with `invalid_capability`/`inconsistent-candidates` (FR-281 rule 4). The TM-001 row also says "awaits a CG arm" without naming QSL-2286. Fix: add the resulting record to the deviation paragraph, and name QSL-2286 in the tests.md FR-022 row. | spec/functional/FR-022-analyze-a-native-analyze-request.md:53-64; spec/tests.md:131 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Lead ruling: CG's negotiation has only `Kani` and `Process` `BackendKind` arms and needs a `KaniForm`, so it cannot settle an analyze item for a linked FSL engine. FR-022 now carries a Deviation section naming QSL-2286, and src/analyze.rs:48-50 says the same. The wording is accurate. |
| FND-002 | accepted-no-change | Accepted as part of the same deviation (QSL-2286, "negotiate_item with extent/mode"). FR-022 names both FR-290 extent rows the driver does not apply. The resulting record is not named; see FND-005. |
| FND-003 | fixed | FR-022 replaces "no option beyond `--format`" with a Deviation section naming QSL-2283. `AnalyzeOptions::parse` refuses `--provider-config` with exit 20 and a message naming the missing caller registry, and `analyze_malformed_request_is_typed_refusal` asserts it. |
| FND-004 | fixed | FR-022 Behavior now says only `deadline_ms` cancels the handle, that SIGINT and SIGTERM are not triggers yet (QSL-2284), that `requested` cannot be reached, and that a signal writes no document. |

## Verdict (disposition pass 1)

APPROVE at 0fa4ea9. The deviations are recorded with real tickets: QSL-2283, QSL-2284 and QSL-2286 exist in project "FSL analyze from the command line". `PATH= quire validate --scope . 'spec/**/*.md'` still exits 0. FND-005 is a low wording gap and does not block.
