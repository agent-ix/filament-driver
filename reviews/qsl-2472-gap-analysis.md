---
id: SR-1456
title: "Analyze contract gap review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver QSL-2472 PR202 d5edfde90ccc544d4a305bae80dd13991d2d6a3f; cli-contracts/README.md; cli-contracts/tests/analyze.rs; cli-contracts/tests/analyze_cases/{mod,options,packages,providers,requests,semantics,value_forms}.rs; spec/functional/FR-022-analyze-a-native-analyze-request.md; FR-006-AC-28/29"
review_set: subset
---

## Summary

Reviewed the new contracts against the driver's analyze acceptance criteria and provider-limit clauses, scoped to the PR as instructed. The available several-candidate ambiguity settlement has no new public-CLI contract; the two explicitly ignored byte-admission and hybrid cases preserve strict required oracles under named owners.

## Verdict

PASS — FND-001 is fixed at cddaced2b0855f56225b03d2c561aead329399b0. The two named owner-pending oracles remain excluded from passing coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-022-AC-8 requires each reachable candidate settlement row, but provider tests always name the backend and omit several candidates with no backend. A driver that arbitrarily picks one provider would pass this suite. Configure two value-validity/unbounded providers with distinct backend names; omit the item backend and require refusal20 plus invalid_capability/ambiguous-backend naming both, then select one explicitly and require dispatch only to it. The accepted helper already takes a backend argument, so this does not depend on the missing hybrid provider. | cli-contracts/tests/analyze_cases/providers.rs:7; spec/functional/FR-022-analyze-a-native-analyze-request.md |

## Coverage

Plan completion: not assessed

Scope is the nine-file PR diff, not a repository-wide assurance claim. Matrix recomputation, broad inventories and accepted evidence reruns are excluded by the user's efficiency instructions. Manual changed-test binding, reverse ownership, hollow-evidence and semantic checks were performed over the scoped public contract. No unspecified production implementation or test stubs were added. The missing named-backend-with-several-candidates row is explicitly unreachable under FR-022-AC-8 and is not requested.

FR-022-AC-1/2: finite proof/refutation, real counterexample and replay; AC-3: max-state bound and request deadline; AC-4: unknown and incapable backend; AC-5: request, file, stage and usage refusals; AC-7: text and canonical JSON framing; AC-8: registry, unique and no-candidate/unknown-backend rows are exercised, ambiguity remains above. AC-9: provider admission, transport, trusted proof, undecided/malformed response and native witness replay; AC-10: SIGTERM/SIGINT after actual dispatch; AC-11: strict ignored hybrid oracle under QSL-2234; AC-12: timed LTL output with non_zeno member. FR-006-AC-28/29 are contextual provider-limit contracts; analyze tests apply the FR-022 option delegation and do not establish every run-specific frame-size assertion. AC-6's exact in-process-to-CLI byte comparison is not claimed by the canonical framing test.

QSL-2662 oversized request admission and QSL-2234 hybrid provider positives remain ignored, visible and excluded from passing coverage. This review does not pronounce QSL-2472 exhaustive or Done while those owners remain pending. Existing baseline measurements and earlier contract deliveries are accepted without remeasurement.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at cddaced2b0855f56225b03d2c561aead329399b0 adds two eligible distinct providers. The unnamed item requires refusal20 and invalid_capability/ambiguous-backend, both names in candidates and registry, no dispatch to either and shutdown of both. A second run names one backend and requires trusted proof, dispatch to it and no dispatch to the alternate. FR-022-AC-8 requires the code/cause, without prescribing backend names inside the diagnostic; candidate/registry assertions are sufficient and no presentation-location oracle is imposed. |
