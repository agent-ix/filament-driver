---
id: SR-202613
title: "gap-analysis of QSL-2561 driver pin wave step 2"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver PR217 QSL-2561 at 141e46c9d67e98e7497b8c42bf0f2db9ac151d8f; src/analyze.rs, fsl-cli/src/test_support.rs; Driver FR-022"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-022
    type: references
---

## Summary

Ticket: QSL-2561. A diff-scoped gap audit of the one production behaviour change, the hybrid negotiation default in src/analyze.rs, against driver FR-022's reference-hybrid decision and Behavior bullet.

## Verdict

**One spec-to-code gap, recorded as FND-001 in SR-202612. No separate defect here.** FR-022 "Behavior" says the driver SHALL hand `Engine::hybrid_reference()` to analyze_with_providers and SHALL negotiate over the manifest of `Engines::fsl_with_reference_hybrid()` plus the started providers. The PR implements only the second half. FR-022-AC-11 (TC-047) is still ignored under QSL-2234 and stays red when run, for an FSL admission reason as well. That gap was already open before this PR; the new element is the manifest/registry skew already reported in the code review. No stubs, no newly untagged tests and no inflated coverage in the diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-022 "Decision: the driver always registers FSL's reference hybrid provider" and Behavior bullet 1: examined.
- FR-022-AC-11: examined. Its ignored test was run with `--include-ignored` and fails with exit 20 `invalid_package` /temporal_clause_names.
- FSpec FR-711-AC-5 test `analyze_dispositions_equal_the_item_entrys`: examined. Its trace tag is unchanged and the oracle compares the driver against CG over the same manifest. The registry change is covered under SR-202612 FND-001.
- The quoin matrix was not rerun. The scope is a pin wave with one default change.
- Plan completion: not assessed

## Dispositions

No findings to dispose; the placeholder row has no outcome. Re-checked at 3f8b20f: FR-022 Behavior bullet 1 is now met for `fsl analyze` and for `fsl run` with providers. It is not met for `fsl run` without provider flags, which is recorded as SR-202612 FND-002.

## Dispositions (round 2)

No findings to dispose. Re-checked at 11b747f: FR-022 Behavior bullet 1 is now met for `fsl analyze`, and for `fsl run` with and without provider flags. FR-022-AC-11 remains ignored under QSL-2234, which is blocked by FSL's hybrid package admission refusal measured in the review pass.
