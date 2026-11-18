---
id: SR-17940486709564847047961
title: "Code and Rust review of QSL-2190 compile limits"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; fsl-cli/src/cli.rs, fsl-cli/tests/cli_contract.rs; QSL-2190; PR #180 at 15e4dcdc1c406c18255b40c098cde59d79dc6759"
review_set: subset
evaluated_revision: "15e4dcdc1c406c18255b40c098cde59d79dc6759"
relationships:
  - {target: "ix://agent-ix/filament-driver/FR-003", type: references}
---

## Summary

Ticket: QSL-2190. Read-only code review with the Rust lane of the diff from `main` (`af759f1bc4b8f72f89b5cde1fac2cac415fd38b2`) to PR #180 head `15e4dcdc1c406c18255b40c098cde59d79dc6759`. The change adds `compile --limit`, parses operands against `SpineLimits`, applies them through `RequestOverrides`, and adds a compile success/refusal regression. Repository `CLAUDE.md` specifies Rust 1.98.1, 100-column rustfmt, and standard workspace gates; no CI workflow or gate changed. The supplied exact-head focused evidence reports the requested parse, default-bound, check, compile, and invalid-setting cases passing. No tests or builds were rerun. FR-869 at the FSL revision pinned by the driver lock is the relevant cross-repository command contract: it requires a compile refusal for recognized but unavailable `admission.work_units`.

## Verdict

REQUEST CHANGES. The implementation uses the existing typed limit helpers, but the new tests do not bind the compile success case to the formal criterion or assert the recognized-but-unavailable compile refusal specified by FR-869-AC-4.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The compile limit regression uses `#[trace("QSL-2190")]`, which is not the formal criterion id. `quoin matrix` therefore reports FR-003-AC-1 as untagged with no binder; tag the test with the criterion it verifies. | fsl-cli/tests/cli_contract.rs:693-696 |
| FND-002 | medium | The compile limit test checks success and an unknown setting only; it does not assert the recognized-but-unavailable `admission.work_units=5` refusal and accepted-setting list required by FR-869-AC-4. Add a compile case for that operand. | fsl-cli/tests/cli_contract.rs:740-754 |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-003-AC-1 | examined | Setting one FSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values. |
| FR-003-AC-2 | examined | A setting name not resolved by FSL's operand parser is refused before any stage with code `invalid-request` and names the offending operand. |
| FR-003 | context_only | When a caller supplies one or more `--limit` options, the driver SHALL use FSL's operand parser and typed caller limits to apply those values to the selected lifecycle operation. |
| FR-869-AC-4 | examined | `fsl parse a b x.fsl --limit s3.nodes=5`, `fsl check x.fsl --limit accounting.work_units=5`, `fsl check x.fsl --limit emit.output_bytes=5` and `fsl compile req.json --limit admission.work_units=5` each exit 20 `invalid-request` naming the setting and the command's accepted settings, before reading a file; `--limit s9.nodes=1`, `--limit s1.nodes=ten` and `--limit s1.nodes=5 --limit s1.nodes=6` each refuse with FR-255 Behavior 6's usage refusal naming the operand. |

The source uses the existing `LimitOverrides` parser/application helper and forwards the resulting `SpineLimits` through `RequestOverrides`. The CLI help and usage text include the new option. The test exercises a deep compile with all exposed bounds raised and checks refusal for an unknown setting. No duplication, panic path, gate narrowing, or unrelated production gap was found. Rust-style API, implementation, boundary, safety, and testing guidance was applied. The focused test output was provided by the owner; no package-wide tests or aggregate gates were run.
