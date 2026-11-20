---
id: SR-1514
title: "Scoped gap review of text outcome closed pipes"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; cli-contracts/tests/text_streams.rs; FR-287-AC-8 text-outcome Unix cohort"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-driver", type: references }
---

## Summary

Ticket: QSL-2098. PR: filament-driver#210. Reviewed frozen commit f26a9f9d34e66349853d67cb8cf26c728c24ee85. This is the assigned diff-scoped gap analysis of the additional Unix text-outcome cohort, not a repository-wide matrix or platform-completion audit.

## Verdict

PASS within the assigned cohort. The active test traces FR-287-AC-8 and exercises real OS pipes for parse, compile and run with both success and syntax refusal. The normally captured baseline ensures each exact request reaches a rendered text outcome; refusal additionally requires invalid_syntax. Closing the only reader before execution reliably drives the intended write failure, while exit-code and stderr assertions distinguish quiet outcome preservation from I/O failure or signal termination. No new unowned production behavior, stub, fake outcome, or coverage inflation was introduced.

## Examined Scope

- id: FR-287-AC-8
  role: examined
  path: /home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/8877b38/spec/functional/FR-287-reach-fsl-through-the-driver-cli.md
  excerpt: "When the reader of stdout closes the pipe before an attempted machine-outcome or formatted-source write, `quire parse`, `format`, `run` and `compile` end quietly, write nothing to stderr and exit with the outcome's FR-285 code, not 30. A text-mode write of the rendered outcome (FR-865) follows the same rule."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. No plan was supplied. Matrix computation was not run under the explicit no-matrix assignment; this verdict records the bounded diff review and does not claim a fresh engine-backed whole-repository gap audit. Existing frontend facets and prior driver/L3 reports are accepted peer reports. No test, build, lint or gate was repeated. The focused active test and scoped formatting/lint receipts are accepted. The optional whole-repository semantic review was not run; the assigned test-intent review is confined to the six new cases. Windows text-pipe execution remains unclaimed.
