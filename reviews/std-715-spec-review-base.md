---
id: SR-202611210022442313144108521
title: "base review of FR-002-AC-11 and TC-009 version output"
type: SpecReview
analysis: base
scope: "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md (FR-002-AC-11); spec/test-cases/TC-009-cli-outcome-and-exit-contract.md"
review_set: subset
profile_disposition: "No applicable AssuranceProfile exists in spec/assurance or the active project configuration; no enforced workflow applies."
---

## Summary

Base checklist review of the changed version-output requirement and its TC-009 coverage. FAIL under the checklist's mandatory hash/pin antipattern rule because the criterion asks for a pinned source commit SHA. This conflicts directly with the user's explicit alpha release requirement; preserve the requested behavior and treat this finding as a policy conflict requiring owner disposition, not as an actionable recommendation to remove the line.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Mandatory checklist classifies the required FSL source revision SHA in CLI version output as a P0 hash/pin antipattern and prescribes removal. The user explicitly requires `FSL revision: <sha>` for candidate identification, so this checklist disposition conflicts with the governing user requirement and cannot safely be applied as a spec correction. | FR-002-AC-11; TC-009 step 7; TC-009 Expected Results | wrong-requirement |
