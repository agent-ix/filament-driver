---
id: SR-2026111606100001
title: "Code review of filament-driver PR 159 fsl analyze verb"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; src/analyze.rs, src/lib.rs, fsl-cli/src/cli.rs, fsl-cli/tests/frontend_cli.rs, fsl-cli/tests/cli_contract.rs, fsl-cli/tests/fixtures/analyze-counter*, Cargo.lock at fa6ea64c0a73d35fea2cfff60c425a1557b06ba4"
review_set: subset
evaluated_revision: "fa6ea64c0a73d35fea2cfff60c425a1557b06ba4"
---

## Summary

Ticket: QSL-1976. Code review with the rust-review lane (rust-style baseline) of the diff from a2fdc28 (main, QSL-2101) to fa6ea64. One review across driver PR 159 and filament-examples PR 33. The examples half is in filament-examples `reviews/qsl-1976-code-review.md`. The spec half is in `reviews/qsl-1976-spec-review.md`.

Evidence: `cargo test -p fsl-cli --test frontend_cli analyze` gave 8 passed. `cargo test -p filament-driver --lib analyze` gave 4 passed. `cargo test -p fsl-cli --test cli_contract root_help` passed. Clippy and `make ci` were not run, because the lead runs the gate.

## Verdict

APPROVE for the code lane, with one low finding. `fsl analyze` calls `fsl_command::command::analyze` with `AnalyzeNegotiation` and a fresh `Cancel`. It writes FSL's document unchanged in machine mode, and the byte-equality test proves it. When the category and the exit code disagree, it fails closed with internal-failure. Usage refusals happen before any file is read. The removed empty-output asserts were the ones the ticket AC forbids. No remaining test asserts empty output on both streams. There is no Python, no logic-bearing shell, and no pin outside Cargo.lock. The Cargo.lock move is confined to the language dcf3165 packages and the crates that move with them. The settlement table's conformance questions (ruling 1) are spec findings, recorded in the spec-review artifact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Text mode labels an `invalid-request` disposition record with the item category (`refusal`), so `0 Bounded refusal invalid_capability/unknown-backend` is printed. FSL FR-621 gives a disposition record the label `invalid-request` (its aggregate `result` rule), and FR-621's text line is `<record label>`. Fix: when `record.result` is absent, use `invalid-request`, and amend FR-022's text-mode bullet ("the item category when a record has none") to match. | fsl-cli/src/cli.rs:1449-1455 |
