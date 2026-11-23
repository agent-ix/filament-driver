---
id: SR-202611210022442313144108533
title: "spec-review criterion strength of filament-driver PR 251 FR-028 export hint amendment"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/filament-driver; PR #251 at f7d1b09be017eee4a3645d29b39f481cbde9e00a; spec/functional/FR-028-print-a-minimal-request-with-example.md (FR-028-AC-1, AC-2, AC-5, AC-6, AC-7, AC-8, AC-9), spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md (FR-002-AC-3)"
review_set: subset
---

## Summary

Ticket: QSL-2930. Jev is not available on this machine (no `jev` executable; the skill states the client does not exist), so criterion strength was judged by hand over the spec text. Each changed AC was checked for whether it pins exact text, exit code, stream and JSON member, so a wrong implementation fails it.

## Examined scope

- FR-028-AC-1, AC-2: byte-equal to `request_example(...)` plus one newline, exit 0, empty stderr; can fail. (examined)
- FR-028-AC-5: exact `details.missing` arrays, stage, exit, no line/column. (examined)
- FR-028-AC-6: withdrawn, verification `None`; asserts nothing by design. (examined)
- FR-028-AC-7: exact `details.missing`, negative message substrings, exact hint line, `driver_hint` value, empty stderr, no hint on other refusals. (examined)
- FR-028-AC-8: equality to the named FSL entry except the hint. (examined)
- FR-028-AC-9: exact `details` path and accepted arrays per verb, never `unknown_wire`; can fail. (examined)
- FR-002-AC-3: exception scoped to the export missing-member refusal. (examined)

## Verdict

Mergeable on this lens only after the two medium findings are fixed; no AC is unfalsifiable. AC-1, AC-2, AC-9 and the hint text of AC-7 are strong. AC-5 cannot be evaluated for analyze and export from the driver spec alone, and the byte-level comparison of AC-7/AC-8 in JSON mode is undefined.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-028-AC-5 derives each verb's expected array from its `--example` output, replacing "a one-of group's example member" by the group's pointers, but the example shows only the group's first member, so the group membership is not derivable from it. The AC names the groups only for `run`; for `analyze` (`program`/`package`) and `export` (`program`/`package`) the expected array is undetermined unless the test copies FSL FR-1047's group table, which the driver spec says it does not keep | spec/functional/FR-028-print-a-minimal-request-with-example.md:173 |
| FND-002 | medium | FR-028-AC-7 and AC-8 compare the JSON-mode hinted document to FSL's "byte-equal except for the `driver_hint` member", but neither fixes where the member is placed or how it is removed for comparison. A driver that parses and reserializes FSL's document (changing member order or whitespace) passes a parsed-value comparison and fails a textual one, so the AC does not decide whether that implementation fails | spec/functional/FR-028-print-a-minimal-request-with-example.md:175 |
