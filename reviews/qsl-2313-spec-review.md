---
id: SR-2026111614530003
title: "Spec review of filament-driver PR 162 FR-025 and TC-039"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-driver; spec/functional/FR-025-stream-observations-with-fsl-monitor.md, spec/test-cases/TC-039-stream-observations-with-fsl-monitor.md, spec/spec.md, spec/tests.md at 994f8131a59a55a6d79ac014633d3928d1b15015; language FR-644, FR-643, FR-287, FR-283 at origin/main b606ff9"
review_set: subset
evaluated_revision: "994f8131a59a55a6d79ac014633d3928d1b15015"
relationships:
  - target: ix://agent-ix/filament-driver/FR-025
    type: reviews
  - target: ix://agent-ix/filament-driver/TC-039
    type: reviews
---

## Summary

Ticket: QSL-2313. Spec integrity review of the new FR-025 and TC-039 and their index rows, against language FR-644 (request, stream, events, summary, driver verb), FR-643 (session), FR-287 (driver verbs, `--provider-config`) and FR-283 (batch monitor) at origin/main b606ff9. `quire validate --scope . spec/spec.md` fails.

FR-025 agrees with FR-644 on the operand, `--input` defaulting to stdin, one JSON line per event then the summary, flushing before the next record, refusal events for malformed records, and exit 20 for refusals before any record. Text mode is not defined by FR-644 or FR-287 (FR-287 only says every verb takes `--format json|text`), so FR-025 defining it is in scope for the driver. The `--provider-config` deviation names QSL-2132 and FSL FR-635 and matches the code.

## Verdict

REQUEST CHANGES, through FND-001 (high) and FND-002 and FND-003 (medium).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-025 and TC-039 index rows were inserted as lines 1-2 of spec/spec.md, above the `---` frontmatter fence. The frontmatter no longer parses: `quire validate` reports "required 'type' is missing from frontmatter" for spec/spec.md. Fix: move the FR-025 row after the FR-024 row (around line 249) and the TC-039 row after the TC-038 row (around line 255), and re-run `quire validate`. | spec/spec.md:1-2 |
| FND-002 | medium | FR-025-AC-4 ("writes nothing to stdout") and FR-025-AC-7 ("no stdout") hold only in text mode. With `--format json` the driver writes the `monitor` refusal `filament-outcome/1` document to stdout, which FR-025 Outputs itself describes. Reproduced: `fsl monitor req.json --format json --provider-config x` writes the document and exits 20. The tests run without `--format` (text default). Fix: scope both ACs ("in text mode nothing on stdout; in machine mode only the `monitor` command-error document") and add a json-mode assertion to each test. | spec/functional/FR-025-stream-observations-with-fsl-monitor.md (AC-4, AC-7), fsl-cli/tests/monitor_cli.rs:205-225, 297-326 |
| FND-003 | medium | The signal deviation cites QSL-2284, but QSL-2284 is "fsl analyze: wire SIGINT/SIGTERM to the analyze cancel handle" (Backlog) and does not mention `monitor`. Nothing tracks wiring signals to the monitor cancel handle. QSL-2132 also does not name `monitor`. Fix: add monitor scope to QSL-2284 (or file a monitor ticket and cite it), state in the FR-025 signal deviation body which ticket lifts it, and note `monitor` on QSL-2132. | spec/functional/FR-025-stream-observations-with-fsl-monitor.md (Deviation sections) |
