---
id: SR-17940486709564847047968
title: "QSL-2129 driver manual-ac-to-tests"
type: SpecReview
analysis: evidence
scope: "agent-ix/filament-driver; QSL-2129; fsl-cli/tests/common/timed_analyze.rs; fsl-cli/tests/frontend_cli.rs"
review_set: subset
---

## Summary

Manual review of the three driver acceptance criteria found one medium coverage gap: the nominal two-clock timer is observationally a one-clock timer. Runtime validation remains pending.

## Verdict

**CONDITIONAL** — one medium gap requires lead disposition; execution remains pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The two-clock deadline fixture never distinguishes x from y: both start at zero, advance together, and are never reset, so aliasing both clocks to one clock leaves every deadline assertion unchanged. | fsl-cli/tests/common/timed_analyze.rs:38 |

## Assessment

Automated quoin gap-analysis is inapplicable to this test-only diff (no changed production implementation to audit). Quoin is available; this is a scoped manual AC-to-tests assessment, not a repository-wide gap-analysis claim. No spec/plan change calls for spec-review subanalyses; React is inapplicable.

| Ticket criterion | Test and assessment |
| --- | --- |
| QSL-2129-AC-2: two-clock holding deadline | analyze_zone_deadline_holds asserts exit 0, EN-6, proved, certified zone basis. Pending state has x>=3 completion guard and y<=3 invariant. Both named clocks participate syntactically, but their equality is invariant; see FND-001. |
| QSL-2129-AC-3: larger delay misses and exact run replays | analyze_zone_deadline_refuted changes delay to 4 against deadline 3; exit 10, refuted EN-6, nonempty timed prefix, 4/1 first delay and reproduced replay are asserted. This is meaningful semantic evidence if the queued run passes, with the same clock-distinction gap. |
| QSL-2129-AC-4: Zeno through time-lock item | analyze_zeno_refuted requests time-lock-freedom; x is reset on every inc, invariant x<=0 prevents any positive delay, and the always-enabled loop has no divergent continuation. It asserts refuted EN-6, time-lock kind, reproduced replay and final delay 0/1. Static alignment passes; execution pending. |

FND-001 scenario: a regression maps both model clock identities to the same live clock slot. For the deadline branch neither clock is modified, so x=y=t forever and the guard/invariant remain exactly x>=delay and x<=delay. All current deadline results and delays can remain unchanged. This does not establish distinct two-clock handling. Use a staged operation/reset so the clocks acquire different values and both affect the deadline outcome; optionally assert a one-clock budget stops the two-live-clock request. This is a coverage finding established by fixture inspection, not an executed mutation or a claim that production currently aliases clocks.

The Zeno case verifies a global time trap with an enabled loop, rather than merely a local dead-end. It does not independently cover the entire FR-632 non-Zeno-liveness premise, and this slice is not claimed to do so.

## Examined Scope

```yaml
[
  {
    "id": "QSL-2129-AC-1",
    "path": "Linear QSL-2129 description / Acceptance criteria",
    "role": "context_only",
    "excerpt": "Request items for zone reachability, timed liveness, time-lock/Zeno, and MITL claim."
  },
  {
    "id": "QSL-2129-AC-2",
    "path": "Linear QSL-2129 description / Acceptance criteria",
    "role": "examined",
    "excerpt": "Driver end-to-end, holding: a two-clock model meets a deadline bound (`analyze_zone_deadline_holds`, `filament-driver/fsl-cli/tests/frontend_cli.rs`)."
  },
  {
    "id": "QSL-2129-AC-3",
    "path": "Linear QSL-2129 description / Acceptance criteria",
    "role": "examined",
    "excerpt": "Driver end-to-end, refuted: the same model with a larger delay misses the deadline and returns a concretised timed run with exact rational delays that replays (`analyze_zone_deadline_refuted`)."
  },
  {
    "id": "QSL-2129-AC-4",
    "path": "Linear QSL-2129 description / Acceptance criteria",
    "role": "examined",
    "excerpt": "Zeno behaviour is reported by the time-lock item (`analyze_zeno_refuted`)."
  },
  {
    "id": "QSL-2129-AC-5",
    "path": "Linear QSL-2129 description / Acceptance criteria",
    "role": "context_only",
    "excerpt": "Negative: a wrong-dimension clock bound is refused at check (probe t3 behaviour); a zone-count budget returns incomplete."
  },
  {
    "id": "QSL-2129-AC-6",
    "path": "Linear QSL-2129 description / Acceptance criteria",
    "role": "context_only",
    "excerpt": "MITL breadth: open, closed and punctual bounds and nested until each have a test in `fsl-analyze/tests/it/zone_mitl.rs` (05 test 25)."
  },
  {
    "id": "QSL-2129-AC-7",
    "path": "Linear QSL-2129 description / Acceptance criteria",
    "role": "context_only",
    "excerpt": "Example cases: analyses/timed-zone-search gains holding and refuted run cases."
  },
  {
    "id": "timer_request",
    "path": "fsl-cli/tests/common/timed_analyze.rs",
    "role": "examined",
    "excerpt": "fn timer_request(name: &str, delay: u32, zeno: bool) -> AnalyzeRequest"
  },
  {
    "id": "analyze_zone_deadline_holds",
    "path": "fsl-cli/tests/common/timed_analyze.rs",
    "role": "examined",
    "excerpt": "A two-clock timer is proved to finish within its three-millisecond bound."
  },
  {
    "id": "analyze_zone_deadline_refuted",
    "path": "fsl-cli/tests/common/timed_analyze.rs",
    "role": "examined",
    "excerpt": "Delaying the same timer to four milliseconds produces an exact timed counterexample that the native evaluator reproduces."
  },
  {
    "id": "analyze_zeno_refuted",
    "path": "fsl-cli/tests/common/timed_analyze.rs",
    "role": "examined",
    "excerpt": "A reset loop under a zero clock invariant cannot advance time and is reported as a replayed time lock, rather than as a liveness witness."
  },
  {
    "id": "timed_analyze-module",
    "path": "fsl-cli/tests/frontend_cli.rs:22",
    "role": "examined",
    "excerpt": "#[path = \"common/timed_analyze.rs\"]\nmod timed_analyze;"
  },
  {
    "id": "FR-022-AC-1",
    "path": "spec/functional/FR-022-analyze-a-native-analyze-request.md",
    "role": "context_only",
    "excerpt": "`fsl analyze` over the Counter request with the item `{\"claim\": \"Bounded\"}` and `--format json` writes one document with `operation` `analyze`, category `success`, one item whose record is `proved`, and `aggregate.status` `complete`, and exits 0."
  },
  {
    "id": "FR-022-AC-2",
    "path": "spec/functional/FR-022-analyze-a-native-analyze-request.md",
    "role": "context_only",
    "excerpt": "The item `{\"claim\": \"BelowThree\"}` gives an item whose record is `refuted`, with a `counterexample` entry and `replay` `{\"target\": {\"state_clause\": Q}, \"reproduced\": true}`, `Q` ending in `BelowThree`, category `violation`, and exit 10."
  },
  {
    "id": "FR-632-AC-1",
    "path": "FSL pinned 13b0b11/spec/functional/FR-632-request-timed-zone-items.md",
    "role": "context_only",
    "excerpt": "A timer subject whose operation fires within 2 time units settles the MITL claim `eventually[0,2]` `proved` with basis `zone-certified`, certification `certified`, and `eventually[0,1]` `refuted` with a counterexample whose delays are exact rationals and replay."
  },
  {
    "id": "FR-626-AC-3",
    "path": "FSL pinned 13b0b11/spec/functional/FR-626-request-deadlock-freedom-terminal-states-and-time-lock-freedom-items.md",
    "role": "context_only",
    "excerpt": "`time-lock-freedom` over the untimed Counter subject settles `invalid-request` `inapplicable-kind`; over a timed subject whose invariant forbids time passing in a reachable location it settles `refuted` with a concretised counterexample."
  }
]
```

## Checks and limitations

Reviewed origin/main...HEAD at afbd47170179d392f2b08bbbb945e8e4e0568a99, PR filament-driver#192 (head resolved through GitHub). Only two test files changed; no CI, product, dependency or spec changes. git diff --check passed.

The one authorized name-filtered command is cargo test --locked -p fsl-cli --test frontend_cli timed_analyze:: -- --nocapture, owned by existing session 93483, with /tmp/qsl-2129-timed-tests-unsandboxed.log. Latest observed log contains only occupied-slot lockf messages; no compilation completion, test results or exit status. This reviewer cannot attach to that session (Unknown process id 93483); no duplicate build was started. The earlier sandbox FETCH_HEAD write failure is infrastructure evidence, not a missing revision or semantic result.

No cargo commands were launched by this review. Full tests, fmt/clippy/deny gates and aggregate sweeps were omitted under Peter's explicit targeted-only/no-gate override. Required run environment remains nice -n 19 /Users/peter/dev/_lead-bin/qsl-slot; jobs=2; CARGO_INCREMENTAL=0; CARGO_PROFILE_DEV_DEBUG=line-tables-only; FSPEC_DIR=/Users/peter/dev/filament-specification; target in this same tree. Lead owns validation completion and the once-per-PR gate. No product/test edits, pushes, GitHub findings, relay messages or additional worktrees.

## Partial ticket scope

This is partial QSL-2129 delivery. Newest comments identify language routing #2930 as landed; driver tests and examples remain. The examples path analyses/timed-zone-search maps to filament-docs and rover fixtures per the review brief, outside this driver diff. Request-family breadth, dimension refusal, zone budget and MITL interval breadth are context only here, not missing driver files demanded by this review. No claim that the entire ticket is done. Plan completion: not assessed.

## Disposition pass 1

**Current verdict: PASS (static); FND-001 fixed; execution pending.**

Reviewed fixed head 2e2aa1ba67e14dc98281b397b27777ae28dd594e in the sole existing tree. Local and PR #192 heads match. Original scratch artifacts and committed reviews/ copies were byte-identical before this pass. Original Findings text remains unchanged.

Current disposition: FND-001 fixed by code inspection. No unresolved findings and no new defects found in this pass. Both methods have a current static PASS; this is not runtime acceptance or merge authorization.

Semantics checked against actual source, not coder prose: BeforeArm y<=1 plus Arm y>=1 forces the first operation at time 1. The inc frame modifies value and x only, so resetting x preserves y=1. State value=1 has x=t-1 and y=t. With delay=3, finish requires x>=2 and pending y<=3, forcing completion at t=3. With delay=4, x>=3 plus y<=4 forces completion at t=4, violating eventually[0,3]. The prefix assertions require at least two steps and exact delays 1/1 then 3/1. Idle is enabled only at value=2 and leaves clock values unchanged, so completion has a time-divergent continuation.

The new analyze_zone_deadline_distinguishes_reset_and_global_clocks test uses the same delay=3 model and rewrites every self.y occurrence to self.x before calling the real fsl binary. Arm remains forced at t=1; afterward x resets to zero and pending x<=3 now permits finishing at t=4. A refuted deadline with reproduced replay is therefore a valid expected result. Unlike a setup-failure check, its exit 10, refuted record and reproduced=true assertions require semantic analysis. Paired with the original holding case, a collapsed-clock implementation cannot satisfy both expected verdicts. This is a behavioral adversarial fixture, not an executed production mutation.

Zeno branch remains unchanged: an enabled reset loop under x<=0 cannot admit any positive delay or divergent continuation. It still requires EN-6 time-lock refutation and reproduced replay. Real binary routing, EN-6 assertions in the principal hold/refute/Zeno cases, exact rational encodings, unique request directories and request lifetime were rechecked. No production, dependency, CI or specification change was introduced; reviews/ additions preserve review history.

Validation: git diff --check origin/main...HEAD passed. Existing single authorized command is cargo test --locked -p fsl-cli --test frontend_cli timed_analyze:: -- --nocapture, session 93483, log /tmp/qsl-2129-timed-tests-unsandboxed.log. Latest log still contains occupied-slot lockf messages, with no completed compilation, test totals or exit status. No cargo command was launched or duplicated. Targeted-only/no aggregate/no-gate and max-two-fix-round overrides remain in force. The new fourth test falls under the same timed_analyze:: filter when that queued command eventually starts; actual compiled-head evidence must be confirmed by the lead.

GitHub reports MERGEABLE with mergeStateStatus BLOCKED. Lead owns completion of targeted validation, once-per-PR gate, dispositions custody and final merge decision. QSL-2129 remains partial delivery: examples/docs in filament-docs and rover fixtures are outside this driver slice.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: staged x reset preserves global y; collapse control requires a replayed refutation. Static fix verified; execution pending. |

Exact after_excerpt, fsl-cli/tests/common/timed_analyze.rs:57–60:

```rust
            "pre Arm using v on M::Counter::inc {{ self.value = 0 and self.y >= 1 ms }}\n\
             post Armed using v on M::Counter::inc {{ self.value = 1 and self.x = 0 ms }}\n\
             pre Ready using v on M::Counter::finish {{ \
                 self.value = 1 and self.x >= {remaining} ms }}\n\
```
