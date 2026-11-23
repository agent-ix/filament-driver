---
id: SR-2026112305304500000002
title: "spec-review EARS conformance of filament-driver PR 248 FR-028"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/filament-driver; PR #248 at 008807a584f4fe5db2292899c6f28b74b84c411d; spec/functional/FR-028-print-a-minimal-request-with-example.md (Behavior), spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md (one added bullet)"
review_set: subset
---

## Summary

Ticket: QSL-2930. This review checks the Behavior statements of the new FR-028, and the one added FR-002 bullet, against EARS grammar.

## Examined scope

- FR-028 Behavior: the `--example` option-table bullet, the stdout bullet, the valid-example bullet and the misuse-refusal bullet. Each is a ubiquitous or event-driven SHALL statement with one response. (examined)
- FR-028 Behavior: the missing-member, wrong-format and export-member `When ... SHALL` bullets. These are event-driven and conform. (examined)
- FR-028 Behavior: the two driver `SHALL` and `SHALL NOT` pass-through bullets. These are ubiquitous and conform. (examined)
- FR-028 Behavior: the "The example for `run` is ..." bullet (lines 79-86). (examined)
- FR-028 Behavior: the "present but invalid" bullet (line 104). (examined)
- The FR-002 added bullet is an informative pointer to FR-028 and states no new obligation. (examined)

## Verdict

Mostly conformant. One medium finding: the trigger "present but invalid" is undefined. One low finding: one bullet sets the content of each example but has no SHALL.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The trigger "When a request member is present but invalid" is undefined. It is unclear whether it covers an unknown member (FSL readers are `deny_unknown_fields`), a wrong JSON kind, a value-level violation, or a one-of conflict such as `program` together with `package`. Two implementers would put the pointer obligation on different refusals, and AC-6 tests only a wrong-typed `models` | spec/functional/FR-028-print-a-minimal-request-with-example.md:104 |
| FND-002 | low | The bullet "The example for `run` is a `native-run/1` request, ... `request` carries every member the verb's reader requires and no optional member" has no SHALL and no EARS form. It is the only statement that sets the format and content of each example, so that obligation is descriptive, not normative | spec/functional/FR-028-print-a-minimal-request-with-example.md:79 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | A Definitions section defines "Invalid member" (unknown, wrong kind, out of domain, one-of conflict). The bullet is now "If a request carries an invalid member, then ... SHALL", and AC-6 adds the unknown-member and one-of-conflict cases |
| FND-002 | fixed | The bullet now reads "The example for `run` SHALL be ...", and the member rule is a separate SHALL bullet |
