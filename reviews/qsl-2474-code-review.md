---
id: SR-1457
title: "Code review of export CLI contracts"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver QSL-2474 PR203 9b06a773f238459757f4ce129728f899aea2b23a; cli-contracts/tests/export.rs; cli-contracts/tests/export_cases/{mod,intake,options,staged}.rs"
review_set: subset
---

## Summary

Reviewed all five changed Rust files, their real public CLI invocations, parsed diagnostics and artifact assertions. Four child modules omit the required source license header; no product, dependency, lifecycle helper or CI behavior changes are present.

## Verdict

CONDITIONAL — add the four source headers. The original coder owns the single final component gate after disposition.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The four export_cases child modules omit the repository SPDX header required by rust-style Implementation on every source file. | cli-contracts/tests/export_cases/mod.rs:1; cli-contracts/tests/export_cases/intake.rs:1; cli-contracts/tests/export_cases/options.rs:1; cli-contracts/tests/export_cases/staged.rs:1 |

## Coverage

All files in scope were examined. Original source, model and snapshot data exercise the real external FSL binary through the accepted bounded process harness. Test helpers are private, errors propagate, file destinations live in owned temporary directories and assertions inspect actual subprocess status, JSON, text and file effects. There is no copied external model/schema/binary, fabricated proof, unsafe code, dependency change, fallback build or narrowed gate in the diff.

The source-model assembly, target limit, exact successful text, selected checked-package success and output-write-failure cases remain strict ignored under QSL-1979. Evidence dispatch, duplicate-envelope and stdin cases remain strict ignored under QSL-2667, QSL-2669 and QSL-2670. Those tests are not runnable coverage. Active semantic negatives require ordered not-an-obligation records and absence of a target; they do not accept an unrelated mapper refusal as generation success.

Existing native harness deliveries and the coder's focused and Clippy results are accepted without rerunning them. No build, product probe, test execution, matrix recomputation or repository inventory was performed. A filename search that included historical reviews was stopped; no historical review was inspected from that search. This is a diff review, not a repository-wide assurance claim.
