---
id: SR-17940486709564847047969
title: "Code review — selected-package export CLI test"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver PR216 QSL-1979/QSL-2672; fsl-cli/tests/export_cli.rs; Driver FR-027-AC-1; FSL FR-1039-AC-4"
review_set: subset
---

## Summary

Reviewed PR #216 at `4c181664603cdb61c064d0b6afc615e1d99b4afa`, the test-only selected-package CLI export change. The process test invokes `fsl export`, removes the source file, checks the supplied `source_package` in the outcome, and compares the written target file to `target_text`; however its request does not prove that package contents drive the export or meet the selected-package criterion it claims.

## Verdict

**CONDITIONAL** — one medium test-coverage finding remains; no build or test was run under the dispatch restriction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The selected-package test allows a successful export without proving the selected package supplied the export records or target text. | fsl-cli/tests/export_cli.rs:155-185 |

## Detail

`FR-1039-AC-4` specifies the counter package with its equality handoff and requires the package export's records and `target_text` to match the source export. This test instead compiles a value-only function, selects `obligations: ["not-a-claim"]`, and sets `minimum_fidelity: "any"`. The unknown obligation becomes a refused record, while that floor permits an assembled export; the test never checks the records or compares the target with a source-route result. It only checks that the caller's `source_package` was retained and that some `target_text` bytes match the selected output file. A route that accepts the wrapper and echoes `source_package` while omitting selected-package semantics could satisfy these assertions. Use the criterion's counter/equality fixture and compare the selected-package records and target text to the source route, or otherwise assert package-derived output.

## Coverage

- PR diff: `fsl-cli/tests/export_cli.rs` only; no production code, dependency pin, or workflow change. The added test invokes the built CLI process with a selected `package.file`, removes `counter.fsl` before export, asserts success and exact `source_package`, requires the output file to exist by reading it, and compares its bytes to the reported `target_text`.
- QSL-2672's valid-reference behavior was checked: the test supplies an identity/version pair and verifies exact retention. Its invalid/missing-reference-before-file-IO cases are outside this test's asserted behavior.
- Test failure behavior checked against FSL's obligation rules: `not-a-claim` can produce a refused obligation record, and `minimum_fidelity: any` allows assembly, so exit 0 does not establish that a valid package claim was exported.
- Driver FR-027-AC-1 was examined as the local CLI output-file criterion; FSL FR-1039-AC-4 was examined as the selected-package record/text criterion; FSpec FR-817-AC-9 was context for the unknown-obligation case.
- Static review only. The coder reports the focused test passed; this reviewer ran no tests, builds, probes, matrix or gates, per dispatch.
- Reverse-gap, stubs, and duplication checks were limited to the changed test. No gap-analysis method applies because the PR changes tests only, not production code, spec, or plan.
- Plan completion: not assessed.

## Examined units

- FR-027-AC-1 (context_only): `fsl export <request> --output m.tla --format json` over the specified counter request exits 0, writes `m.tla` byte-equal to `result.export.target_text`, and returns an export success outcome; the request-member output route writes the same bytes.
- FR-1039-AC-4 (examined): `FR-1015-AC-4`'s counter compiled to a package with its equality handoff and explicit `source_package`, exported with the source removed, writes the exact source reference and otherwise the source export's `records` and `target_text`.
- FSpec FR-817-AC-9 (context_only): An unknown obligation name produces a refused `not-an-obligation` record; the default fidelity floor refuses, while `minimum_fidelity: any` permits assembly.
