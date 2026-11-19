---
id: SR-17940486709564847047967
title: "Code review — fsl-cli export route"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; fsl-cli/src/cli.rs, fsl-cli/tests/replay_cli.rs; FR-027, FR-024"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-027
    type: references
---

## Summary

Reviewed PR #176 at `5d3917e75c2d60f894819e6319359bd335777a51` against current driver main, including the FR-027 export contract and affected CLI implementation. The route does not implement the specified `--output` handoff, and text mode and limit parsing also diverge from the contract.

## Verdict

**FAIL** — the required export command path is incompatible with FR-027 and cannot satisfy its main acceptance criteria.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-027-AC-1 | examined | `fsl export <request> --output m.tla --format json` over FR-756-AC-1's counter request (`inc`, `value <= 5`) exits 0, writes `m.tla` whose bytes equal the document's `result.export.target_text`, and writes FSL's outcome document with operation `export`, category `success` and `result.kind` `exported`; the request carrying `export.output` naming `m.tla`, run with no option in a fresh directory, writes a byte-identical file. |
| FR-027-AC-2 | examined | `--output` naming an existing file refuses `invalid-request`, exit 20, `last_stage` null, naming the path, and the file's bytes are unchanged; a request carrying `export.output` with `--output` refuses naming `output` and `--output`, exit 20, and writes neither file; `--output` given twice refuses naming `--output`, exit 20. |
| FR-027-AC-3 | examined | `fsl export <request> --output m.tla --limit export.output_bytes=1` exits 22, names `export.output_bytes` and writes no file; with `--limit export.output_bytes=16777216` it exits 0. |
| FR-027-AC-4 | examined | `fsl export <request> --output m.tla` with no `--format` writes exactly the four lines `export: success (exit 0) at S4`, `target: tla filament.output.tla2/v1`, `bytes written: <n>` and `output: m.tla`, and exits as in machine mode. |
| FR-027-AC-5 | examined | `fsl export --evidence e.json --format json` writes FSL's evidence outcome document with operation `export`; `fsl export --evidence e.json --output m.tla` and `fsl export <request> --evidence e.json` each refuse `invalid-request` naming `--evidence`, exit 20, with nothing written to the output path; `fsl compile <request> --evidence e.json` refuses as an unknown option, exit 20. |
| FR-024-AC-1 | context_only | `fsl compile <request> --output <file>` writes bytes equal to the library result and to the request member's output, resolving relative paths against the working directory. |
| FR-024-AC-3 | context_only | A request carrying `output` with `--output` is refused by FSL, naming `output` and `--output`, with no file written. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Export CLI uses `--out` and writes target bytes itself instead of passing `--output` to FSL as the request override | fsl-cli/src/cli.rs:259, 1232, 1262 |
| FND-002 | medium | Default text mode serializes the result as pretty JSON instead of FR-865's four-line export text | fsl-cli/src/cli.rs:1388-1396 |
| FND-003 | medium | Export limit parser permits only one `export.output_bytes` value, rejecting the repeatable export and compile limits FR-027 accepts | fsl-cli/src/cli.rs:283-300 |

FND-001: FR-027 defines `fsl export <request> [--output <file>]` and requires the driver to pass the output path to FSL through `RequestOverrides`, with FSL owning the file write. This implementation instead requires `--out`, routes the request through generic `command::run`, and writes `target_text` in the driver. A conforming invocation using `--output` is rejected, so the primary command contract cannot run.

FND-002: `export_result` sets `text_output` to pretty-printed JSON for every non-JSON format. Thus a successful default text invocation cannot produce the required FR-865 header, target, byte count, and output path lines.

FND-003: FR-027 allows repeatable `--limit` values for `export.*` and settings accepted by `fsl compile`; this parser rejects every setting except `export.output_bytes` and rejects a second `--limit`.

## Verification

- Focused command: `cargo test --locked -p fsl-cli --bin fsl export -- --nocapture`, through `locked-build.sh`, with debug info and incremental compilation disabled. Result: PASS, 5 passed, 0 failed, 39 filtered; target cleaned after the green gate.
- Aggregate and package-wide gates were not run under the standing targeted-check instruction.
- No assurance profile in `spec/` applies to this CLI change.
