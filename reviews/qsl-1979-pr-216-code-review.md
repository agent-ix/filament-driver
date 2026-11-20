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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The replacement no longer verifies exact `source_package` retention on a successful selected-package export. | fsl-cli/tests/export_cli.rs:154-200 |

The new test now exercises the real counter package and observes its current-pin unsupported-capability refusal, but it does not produce a successful package export or inspect the result's `package.source_package`. The request's `source_package` is only input data. This drops the earlier passing assertion for exact identity/version retention, which is part of QSL-2672 and FSL FR-1039-AC-4. Keep a separate successful selected-package assertion for the current pin's supported value-only path, or otherwise cover the valid-reference result through this CLI route.

## Dispositions

Round 1, fix commit `a08046afa4b4ef4be672b51d324944e633bbd720`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The replacement uses the real counter checked package and equality handoff, removes the source file, and asserts the precise current-pin `model/object_type` lowering refusal, exit 21 at S4, zero output bytes and no target file. With the package as the only program input, this establishes the selected-package route reaches the package's export semantics; QSL-2647 is named as the ticket that flips this expected refusal to positive parity. |
| FND-002 | still-open | The selected-package request now ends at the expected lowering refusal and no longer asserts a successful result retaining the exact caller-supplied `source_package` identity/version. Preserve a separate successful CLI case or add another direct assertion for this supported current-pin behavior. |

The current pin remains `374fe2a8`; this PR intentionally records its refusal. The successful AC-4 parity test reported in the latest FSL worktree does not change the driver's pinned behavior and is not evidence that this PR produces a successful export.

Round 2, fix commit `7ccecd6b8c1f16fb76ca9012619e7ddfd8540d7a`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | A separate successful value-only selected-package CLI case now removes the source file, asserts exact `source_package` identity/version retention, and compares the written target file bytes with the returned `target_text`. The counter/equality refusal case remains separate and unchanged. |

FND-002 after excerpt:

```rust
    assert_eq!(
        value["result"]["export"]["package"]["source_package"], source_package,
        "FSL must retain the selected package's caller-supplied source reference"
    );
    let text = value["result"]["export"]["target_text"]
        .as_str()
        .expect("target text from selected package");
    assert_eq!(
        std::fs::read(directory.path().join("selected.tla")).expect("the target file"),
        text.as_bytes()
    );
```

No new findings. The focused tests were not rerun by this reviewer; the coder reports both selected-package tests passed.
