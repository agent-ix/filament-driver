---
id: SR-1513
title: "Code and Rust review of text outcome closed pipes"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; cli-contracts/tests/text_streams.rs; FR-287-AC-8 text-outcome Unix cohort"
review_set: subset
---

## Summary

Ticket: QSL-2098. PR: filament-driver#210. Reviewed frozen commit f26a9f9d34e66349853d67cb8cf26c728c24ee85. The code-review and rust-review lanes examined the new test only, with the existing CliCase stdout seam as context. No production code changed.

## Verdict

PASS for the Unix text-outcome cohort. The real pipe reader is dropped before spawning the CLI, making the closed-reader condition independent of scheduling. Ownership transfers the actual writer to child stdout. The six parse/compile/run success and syntax-refusal invocations first establish rendered output, the expected operation/category, empty stderr and exit 0 or 20. The closed-pipe invocations independently require the same numeric status and empty stderr, exposing an output-I/O exit of 30, SIGPIPE termination, or leaked diagnostics. Errors propagate through Result; there are no mocks, sleeps, swallowed errors or skipped assertions.

The author reports the one active focused test, scoped clippy and direct rustfmt passing. Those receipts are accepted without repeating execution. The final affected-component gate remains with the author. Windows execution is unclaimed.

## Assurance Context

The owning language criterion is read from the locked FSL 8877b38 spec, not a later language interface. Existing operand, encoding, devfull, machine-pipe and producer-document facets are accepted peer-owned coverage and were not reread or retested.

## Examined Scope

- id: FR-287-AC-8
  role: examined
  path: /home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/8877b38/spec/functional/FR-287-reach-fsl-through-the-driver-cli.md
  excerpt: "When the reader of stdout closes the pipe before an attempted machine-outcome or formatted-source write, `quire parse`, `format`, `run` and `compile` end quietly, write nothing to stderr and exit with the outcome's FR-285 code, not 30. A text-mode write of the rendered outcome (FR-865) follows the same rule."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
