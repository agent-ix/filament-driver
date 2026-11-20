---
id: SR-1455
title: "Code review of analyze CLI contracts"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver QSL-2472 PR202 d5edfde90ccc544d4a305bae80dd13991d2d6a3f; cli-contracts/README.md; cli-contracts/tests/analyze.rs; cli-contracts/tests/analyze_cases/{mod,options,packages,providers,requests,semantics,value_forms}.rs"
review_set: subset
---

## Summary

Reviewed the changed Rust analyze contracts and README, including external-provider trust versus core certification, native witness replay, cancellation readiness, selected-package admission and semantic outcomes. One confined source-header omission remains; no production behavior, dependency, unsafe or CI changes are present.

## Verdict

PASS — FND-001 is fixed at cddaced2b0855f56225b03d2c561aead329399b0. The final component gate remains with the original coder.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The seven new analyze_cases child modules omit the repository SPDX header; rust-style Implementation requires the repository license header on every source file. | cli-contracts/tests/analyze_cases/mod.rs:1; cli-contracts/tests/analyze_cases/options.rs:1; cli-contracts/tests/analyze_cases/packages.rs:1; cli-contracts/tests/analyze_cases/providers.rs:1; cli-contracts/tests/analyze_cases/requests.rs:1; cli-contracts/tests/analyze_cases/semantics.rs:1; cli-contracts/tests/analyze_cases/value_forms.rs:1 |

## Coverage

All nine changed files listed in scope were examined. The root analyze.rs carries the correct AGPL header. New tests execute the supplied real FSL binary through the accepted bounded lifecycle harness; provider transport uses the authoritative same-repository helper selected by environment. Existing model data is referenced in place. No copied external model/schema/binary, fabricated semantic engine, hidden product-build fallback, changed gate or unsafe block was added.

Assertions distinguish real certified EN-1/EN-6 results from a transport provider's trusted result, reject malformed and false witnesses, require replay of a true n=0 violation, wait for a real dispatched request before provider signals its parent, and invalidate the source before admitting a public compile handoff. JSON assertions inspect values emitted by FSL; assertions and indexing failures are meaningful test failures, not product panic paths.

The coder's focused checks and changed-code Clippy results are accepted without rerun. The one final component gate is still due after disposition. No product builds, command probes, matrix recomputation, old-test reruns or repository-wide inventory were performed by this reviewer. The existing harness and earlier contracts are accepted peer deliveries. This is a diff review, not a repository assurance or completion claim.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at cddaced2b0855f56225b03d2c561aead329399b0 adds `// SPDX-License-Identifier: AGPL-3.0-or-later` to all seven child modules. Only the fix diff was inspected; no tests or builds were repeated. |
