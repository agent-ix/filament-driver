---
id: SR-202611210022442313144108530
title: "code-review (with rust-review lane) of filament-driver PR #249 help and refusal messages"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; git diff origin/main...cf598ead5af9c6f8b352e9a75569facefd1aa5d9: fsl-cli/src/cli.rs, fsl-cli/src/options.rs, src/execute.rs, cli-contracts/tests/messages.rs, fsl-cli/tests/binary_process.rs, fsl-cli/tests/cli_contract.rs, fsl-cli/tests/stream_rule.rs"
review_set: subset
---

## Summary

Ticket: QSL-2931 (batch with QSL-2932, QSL-2933, QSL-2929). Reviewed the Rust diff of
PR #249 with the rust-review lane (rust-style loaded). The help and message changes are
small and readable: `nearest`/`edit_distance` is a correct Levenshtein with a bounded
two-edit threshold, `grouped_settings` preserves table order, and every check, verb and
option message the tickets name has an exact-text cli-contracts case. The one material
defect: on `run`, `analyze`, `monitor`, `replay` and `export` the unknown-`--limit`
refusal still lists every FSL stage and accounting setting (about 150 names, now grouped)
instead of the settings the command accepts, contrary to the FR-002 Outputs bullet this
PR adds. Only `check` is pinned.

Gates (local, reused worktree target): rustfmt --check clean on every changed file
(two pre-existing fmt diffs elsewhere, not in this diff). `cargo clippy -p fsl-cli
--all-targets [--features fsl-cli/tc018-test-adapter] -D warnings` fails on a
pre-existing dead_code error at fsl-cli/src/analyze_provider.rs:263 (file not in the
diff). cli-contracts clippy on `messages` is clean. Tests: after building
`fsl-plugin-launch`, cli-contracts messages 8/8, analyze 39/39, monitor 28/28,
exit_codes 13/13; fsl-cli bin units 58/58, binary_process 28/28, cli_contract 60/60,
help_contract 5/5, stream_rule 7/7, frontend_cli 81/82. The one failure,
`analyze_finite_hyper_preserves_population_results_and_declared_bound`, is an FSL
package-admission refusal (`invalid_package/timed-node-reference`) on a path this PR
does not touch and with an unchanged lockfile; it is pre-existing.

Provider-spawn NotFound cause: `fsl` finds the process-provider launcher
`fsl-plugin-launch` beside itself (filament-plugin-host/src/lib.rs:63-76). It is a
`[[bin]]` of `filament-plugin-host`, so `cargo test -p fsl-cli` and `cargo build --bin
fsl` never build it. Running `cargo build -p filament-plugin-host --bin
fsl-plugin-launch` first makes every provider test pass. No failure is caused by this PR.

## Verdict

CONDITIONAL. No high findings. FND-001 (medium) should be fixed in this PR, either by
passing each verb's accepted set or by narrowing the FR-002 bullet. The rest are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Unknown `--limit` setting on run/analyze/monitor/replay/export calls `limit_usage_message(&error, &[])`, which lists every FSL stage plus accounting setting (~150 names, e.g. accounting.*, observation.*, model_check.* on `fsl run R --limit foo=1`), not "the settings the command accepts" as the new FR-002 Outputs bullet states; run's provider path refuses accounting/observation (FR-002-AC-27) and analyze/monitor/replay have narrower FR-003-AC-12 sets. Only `check` is pinned. | fsl-cli/src/cli.rs:807-827; fsl-cli/src/cli.rs:313; fsl-cli/src/cli.rs:889; fsl-cli/src/cli.rs:1584; fsl-cli/src/cli.rs:2320 |
| FND-002 | low | `fsl help <anything-not-a-verb>` changed from root help (exit 0) to an unknown-verb refusal, and `fsl help --format json` now says "unknown verb `--format`; did you mean `format`", suggesting a verb for an option. Unspecified and untested. | fsl-cli/src/cli.rs:397-419 |
| FND-003 | low | `fsl help prove` (and generate, render) writes root help with exit 0, while `fsl prove --help` answers unsupported exit 21 per FR-002-AC-25/AC-28; the `Some(_) if arguments.len() == 2` arm is unspecified and untested. | fsl-cli/src/cli.rs:415 |
| FND-004 | low | Changed messages without an exact-text case: "help takes at most one verb"; the "settings this command accepts, by stage:" suffix (only `contains` checks in cli_contract.rs); the unknown-setting text on run/analyze/monitor/replay/export. | fsl-cli/src/cli.rs:416; fsl-cli/src/cli.rs:839; fsl-cli/src/cli.rs:849; fsl-cli/tests/cli_contract.rs:2031-2042 |
| FND-005 | low | New cli-contracts tests bind with deprecated `/// Trace:` doc lines; the project rule (2026-10-17) is ix-trace-rs `#[trace]`. cli-contracts has no ix-trace-rs dependency, so this follows the crate's existing 14 tags, but propagates the deprecated form. | cli-contracts/tests/messages.rs:29; cli-contracts/tests/messages.rs:166; cli-contracts/tests/messages.rs:190 |
| FND-006 | low | The bare-inspect test asserts the `inspect --handoff missing --format dot` refusal only by `contains("--format")`, which also matches a usage line, and its name ("...text_and_dot_and_dot_with_handoff...") is garbled. | cli-contracts/tests/messages.rs:190-215 |
| FND-007 | low | Root help `Operands:` row `--model <identity>=<file>` puts its description at column 29, while every other row starts at column 26, so the section is misaligned. | fsl-cli/src/options.rs:538 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | low | `request_setting_names` hard-codes `analyze`'s accepted list as `&["replay.input_bytes"]`, a second copy of the match in `analyze_run_limits`. If a setting is added to one and not the other, the refusal text drifts from what analyze applies, and only the `messages.rs` literal would catch it. | fsl-cli/src/cli.rs:865-870; fsl-cli/src/cli.rs:1952-1966 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix 8a5efea passes each verb's set (`run_setting_names`, `export_setting_names`, `request_setting_names`, `monitor` empty), and messages.rs pins exact text for check, run, export, analyze, monitor and replay. |
| FND-002 | fixed | `unknown_verb` suggests no verb for a word starting with `-`; `help <non-verb>` refusal is now specified in FR-002 Behavior and pinned by `help_with_a_non_verb_or_extra_words_refuses_on_stderr`. |
| FND-003 | fixed | `help <verb>` now routes every recognized verb to `<verb> --help`; prove/generate/render print the not-available help, exit 0, the same bytes for `--help`, `-h` and `help <verb>` (owner decision a). |
| FND-004 | fixed | "help takes at most one verb", the compile "settings this command accepts, by stage:" refusal and every verb's unknown-setting text now have exact `assert_eq!` cases. |
| FND-005 | accepted-no-change | cli-contracts is a standalone test-only workspace with its own lockfile and no ix-trace-rs dependency; its 14 existing tags use `/// Trace:` and the matrix binds them. Converting the crate is crate-wide work, not this PR's. Acceptable here; a crate-wide conversion ticket is the right follow-up. |
| FND-006 | fixed | Dot-with-handoff refusal is now an exact `assert_eq!` and the test is renamed. |
| FND-007 | fixed | The `--model <identity>=<file>` row now puts its description on the next line at the shared column. |
