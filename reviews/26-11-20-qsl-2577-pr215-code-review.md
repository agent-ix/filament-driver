---
id: SR-17940486709564847047970
title: "code-review of QSL-2577 requirement-run caller limits"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver PR215 QSL-2577 at fb3984bace429f7e873227f3f50563894584dd57; fsl-cli/src/cli.rs, fsl-cli/tests/cli_contract.rs, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md, spec/test-cases/TC-046-run-tl-lowering-and-package-requests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
---

## Summary

Ticket: QSL-2577. Code review with the rust-review lane over the PR diff. The new `fsl run` route sends a function call carrying `call.requirements` through FSL's `run_with_providers_and_limits_and_overrides` with the caller's `SpineLimits`, clocks and intake ceilings, using `with_started_providers` and therefore `filament_driver::run_engines` (reference hybrid engine first) and `AnalyzeNegotiation::with_providers`. Probed with the built `target/debug/fsl` at this head: default run exit 0 with 3 settled items, `s1.nodes=1` exit 22 `incomplete` with `items: []`; the same request without requirements behaves as before.

## Verdict

**CONDITIONAL** — the routing is correct and the AC-27 oracle catches dropped spine limits (a dropped limit makes the `s1.nodes=1` run exit 0). Two medium gaps remain: the route loses source rendering in text mode, and it refuses `accounting.*` settings that the plain run applies, with no spec text stating the refusal. The bounded reader keeps the existing byte ceiling (`take(limit+1)`, refuse when longer) and returns `None` on malformed JSON exactly as before; `fsl run` has no stdin request operand, so the stdin concern does not apply.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Requirement route drops render sources: text-mode diagnostics print `agent-ix/t:p:82-90` instead of `unit.native:3:1` that the plain and package routes print | fsl-cli/src/cli.rs:2193-2198 |
| FND-002 | medium | Requirement route refuses `--limit accounting.work_units=N` (exit 20, "unavailable for the `run` operation") although the same request without `requirements` applies it per FR-003-AC-10; no AC states this refusal | fsl-cli/src/cli.rs:1086-1092 |
| FND-003 | low | AC-27 test's "configured value 1" check is `rendered.contains('1')`, which any outcome document satisfies | fsl-cli/tests/cli_contract.rs:2936 |
| FND-004 | low | `observation_limits` passed to the requirement route is always `None`: `apply_to_limits` on `SpineLimits` already refused any observation setting | fsl-cli/src/cli.rs:1086-1100 |
| FND-005 | low | Every function run now reads and parses the request file twice in the driver (extraction check, then requirements check after FSL's own prepare read); one `native_run_request` read could answer both, and `.cloned()` copies subtrees only to test presence | fsl-cli/src/cli.rs:1302-1332 |

## Detail

FND-001: `run_with_requirement_items` returns `CommandResult::document(document)` without `.with_sources(fsl_command::render_sources(result.sources()))`, which the Package and extraction routes attach. Reproduced: `fsl run req.json --limit s1.nodes=1` prints `agent-ix/t:p:82-90: stage_limit_exceeded: ...`, the request without requirements prints `unit.native:3:1: ...`. JSON output is unaffected. The pre-existing `run_with_providers` has the same omission.

FND-002: FSL `RequestOverrides` has no accounting member, so the route cannot carry accounting; arch dropped the accounting case for FSL #3206. The refusal is honest (nothing is silently dropped), but FR-003-AC-10 says an `accounting.<counter>` setting reaches the run, and FR-002-AC-27 is silent. Either state the refusal in FR-002-AC-27/FR-003 (with a test) or make the message say the requirements route takes no accounting settings. Reproduced: `req.json --limit accounting.work_units=1000000` exit 20; `noreq.json` same limit exit 0.

FND-003: AC-27 requires "naming `s1.nodes` and configured value 1". Assert the diagnostic's code `stage_limit_exceeded` and that its message contains `limit 1 reached`. The exit-22 and `items == []` assertions already make the limit-dropped case fail, so this is a precision gap, not a vacuous test.

FND-004: dead plumbing; remove the parameter or document it as a future override.

FND-005: efficiency and a small window where the file the driver inspected differs from the one FSL ran; the reader itself is correct.

