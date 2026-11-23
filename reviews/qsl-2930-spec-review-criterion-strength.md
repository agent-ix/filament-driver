---
id: SR-2026112305304500000003
title: "spec-review criterion strength of filament-driver PR 248 FR-028"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/filament-driver; PR #248 at 008807a584f4fe5db2292899c6f28b74b84c411d; spec/functional/FR-028-print-a-minimal-request-with-example.md (FR-028-AC-1..AC-9), spec/test-cases/TC-050-print-a-request-example-and-name-request-defects.md"
review_set: subset
---

## Summary

Ticket: QSL-2930. Jev was not available on this machine (no `jev` executable), so the reviewer judged criterion strength by hand over the spec text. Each AC was checked to see whether it states the exact text, exit code and stream, so that a wrong implementation fails it.

## Examined scope

- FR-028-AC-1: exit 0, empty stderr, byte-equal stdout plus a newline, exact member set and format per verb, and equal bytes across output modes. Each part can fail. (examined)
- FR-028-AC-2: the presence and absence of members, and order independence. These can fail, subject to the `--evidence` value-form defect recorded in the integrity review. (examined)
- FR-028-AC-3: forbids a stage `envelope` or `request` refusal. This can fail. (examined)
- FR-028-AC-4: exact invocations, `invalid-request`, exit 20, the named arguments, no file read, help precedence, and the help-listing check. These can fail. (examined)
- FR-028-AC-5: exact messages, code and exit 20. These can fail. (examined)
- FR-028-AC-6: requires the pointer substring. This can fail. (examined)
- FR-028-AC-7: exact message, code `unknown_wire` and exit 20. These can fail. (examined)
- FR-028-AC-8: exact message, plus a negative check for `requires export`. This can fail, subject to the quoting defect recorded in the integrity review. (examined)
- FR-028-AC-9: byte equality to FSL's output, exit 20 in both modes, stdout only and empty stderr. These can fail. (examined)
- TC-050 steps 1-8 map one-to-one to AC-1..AC-9. (examined)

## Verdict

All nine ACs can fail, and each states its code, exit status and stream. One low finding: AC-1's and AC-9's oracle is an FSL entry that has no name yet. The Dependencies section states this correctly.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The oracle in AC-1 ("the document FSL's request-example entry returns") and in AC-9 ("the document FSL's own entry returns") is an unnamed FSL entry that does not exist yet. A test author cannot bind these byte-equality checks until FSL names the entry. The Dependencies section records that FR-028 is not implementable until then, but the AC does not say which symbol to call | spec/functional/FR-028-print-a-minimal-request-with-example.md:127 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | AC-5's oracle is circular. The message must contain "the JSON pointer of every required member the verb's FSL reader reports missing", and "the set compared is FSL's reader's". A reader that reports only the first missing member then defines the set as that one member and passes, which contradicts the AC's own claim that it fails. The expected set should come from an independent source, for example the members of the verb's `request_example` `request` object | spec/functional/FR-028-print-a-minimal-request-with-example.md:148 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-1 now names `fsl_command::command::request_example` as the oracle, and Dependencies records that FSL must specify it |
| FND-002 | fixed | verified in fix commit a664ab18 |
