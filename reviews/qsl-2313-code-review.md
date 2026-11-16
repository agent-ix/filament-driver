---
id: SR-2026111614530001
title: "Code review of filament-driver PR 162 fsl monitor"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; fsl-cli/src/cli.rs, fsl-cli/tests/monitor_cli.rs, fsl-cli/tests/cli_contract.rs, Cargo.lock at 994f8131a59a55a6d79ac014633d3928d1b15015; language fsl-command monitor.rs at b606ff9"
review_set: subset
evaluated_revision: "994f8131a59a55a6d79ac014633d3928d1b15015"
relationships:
  - target: ix://agent-ix/filament-driver/FR-025
    type: reviews
---

## Summary

Ticket: QSL-2313 (driver half of QSL-1977). Code review with the rust-review lane (rust-style baseline) of the diff from origin/main to 994f8131a59a55a6d79ac014633d3928d1b15015. Evidence: `cargo test -p fsl-cli --test monitor_cli --test cli_contract --test frontend_cli` gave 7, 19 and 17 passed. The built `fsl` binary was run by hand in text and json mode, with `--provider-config`, a missing operand, an unknown option, an unknown clause, a closed stdout pipe, and a checkpoint record whose folder does not exist. `make ci` and clippy were not run; the lead runs the gate.

Streaming: `command::monitor` hands each event to the sink before it pulls the next line (fsl-command monitor.rs:538-590 at b606ff9), and the sink calls `streams::write_primary_stdout`, which writes and flushes under the stdout lock (ix-cli-kit 7f3b58b streams.rs:386-390). The record source is `BufRead::split` over `--input` or stdin, so no record is read ahead of the one being processed beyond what the OS pipe already holds. The AC-6 test has no fixed sleeps, only 60-second receive bounds, and asserts the settling event arrives while stdin is still open. It is not timing-sensitive under load.

Record source: FR-644 Behavior states "The driver's `monitor` verb SHALL take the request file operand and `--input <file>` (default standard input)". FR-644 Inputs gives `records` to `command::run` only. fsl-command monitor.rs:446-451 refuses a `records` member on the stream path ("a records file is read by run; monitor reads the stream it is given"). The coder's claim holds.

Exit codes: the summary's `exit_code` is mapped back to a category through `Category::ALL`, so the process exits with FSL's code. A closed pipe keeps that code (reproduced: exit 10). Refusals before any record use FSL's command-error document, exit 20. Driver argument refusals use `input_refusal`, exit 20.

Cargo.lock: only the ten fsl-* packages move, from f1db1f3 to b606ff9 (language origin/main). filament-contract-model stays at 80e3f67f. No Python and no logic-bearing shell in the diff. The `cli_contract` change moves `monitor` out of the unsupported-verb loop and asserts its help line names native-monitor/1; the other five verbs keep the exit-21 assertion.

## Verdict

APPROVE for the code, with two low findings. The overall PR verdict is REQUEST CHANGES because of the spec-review SR (SR-2026111614530003, FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | When `command::monitor` returns `Err` after events were written (reproduced with a `{"checkpoint": "nodir/x.json"}` record: two events, then a `filament-outcome/1` io-error document, exit 20), the driver appends the command-error document to the same JSON Lines stream. FR-025 Outputs only describes refusals before any record. Fix: state in FR-025 Outputs that a failure after the first record ends the stream with FSL's command-error document and its exit code. | fsl-cli/src/cli.rs:1762-1767 |
| FND-002 | low | `fsl_with_stdin` calls `.expect("write records")` after spawn. The AC-3 and AC-4 cases refuse before reading stdin, so if the child exits before the parent writes, the write fails with EPIPE and the test panics. Not reproduced in 150 runs, but nothing prevents it under load. Fix: ignore `ErrorKind::BrokenPipe` on that write, or pass `Stdio::null()` for the refusal cases. | fsl-cli/tests/monitor_cli.rs:72-87 |
