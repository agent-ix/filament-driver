---
id: SR-2026112002230001
title: "Code review of filament-driver PR 196 driver option table"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; fsl-cli/src/options.rs, fsl-cli/src/cli.rs, fsl-cli/src/extension_set.rs, fsl-cli/src/provider_config.rs, fsl-cli/src/main.rs, fsl-cli/tests/help_contract.rs, fsl-cli/tests/cli_contract.rs, fsl-cli/tests/frontend_cli.rs at 981693bcc6a7ca920b7a0966f0573404592ee8c3; language FR-866 and FSpec FR-562 at origin/main"
review_set: subset
evaluated_revision: "981693bcc6a7ca920b7a0966f0573404592ee8c3"
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: reviews
---

## Summary

Ticket: QSL-2602, with QSL-2601, QSL-2603 and QSL-2649 (one PR). Code review with the rust-review lane (rust-style baseline) of `git diff origin/main...HEAD` at 981693b.

One table: `fsl-cli/src/options.rs` holds every routed verb's usage lines and options. `options::scan` and `options::root_help` take the table or an entry as an argument, and every verb parser (`*Options::from_scanned`) reads the scan result. The old per-verb `USAGE` constants, hand-written root help, `clocks_operand`/`output_operand` and `ProviderArguments::take` are deleted. No shim remains. The only option literals left in `fsl-cli/src` outside the table are in `clocks.rs` (FND-003) and in test code.

Per-verb help: `dispatch` returns `entry.help()` for `--help`/`-h` once the unknown-option check passes. `prove`, `generate` and `render` have no entry and stay not routed. `--version`/`-V` and `--color` are unknown options on every verb. `help_contract.rs` checks all of this against the built binary. I ran `fsl inspect --help` by hand: exit 0 and the table text.

`--model <identity>=<file>`: `identity_operands` splits at the first `=` (`example/two=other=model.json` gives the file `other=model.json`). It refuses a missing `=`, an empty identity, an empty file and a repeated identity before any file is read. The bare form is gone from code, tests, help, and driver spec text (`git grep -- --model` finds no bare form). No filament-examples manifest uses `--model`.

The changed assertion in `root_help_identifies_current_command_availability_and_text_restrictions` is a correct update: FR-002 now lists `inspect` among the routed verbs. The test still runs `inspect --format json` and expects the not-routed result.

No Python, no `#[ignore]`, no removed assertions, no Cargo or lockfile change. Formatting moves toward the pinned 1.98.1 rustfmt; cli.rs drift went from 7 hunks on main to 3. Evidence: the two pre-existing `frontend_cli` failures were reproduced at this head (see the verdict). The other suites were not rerun; the coder ran them.

## Verdict

APPROVE: no high finding. The two medium findings are a regression on repeated `--markdown` and the `--library` operand form, which still contradicts FSL FR-866. The second needs a follow-up ticket if it is not fixed here.

`request_byte_options_reach_owner_limits_and_compile_accepts_none` (frontend_cli.rs:543) and `check_takes_the_integer_division_law_from_its_option` (frontend_cli.rs:1985) fail at this head as on main. They assert a stderr line under `--format json`. FR-002 says machine mode writes one `filament-outcome/1` document that carries the diagnostics, and `write_result_diagnostics` writes to stderr only for driver input refusals. I reproduced `fsl check absent.fsl --integer-division ceiling --format json`: stdout has the message in `diagnostics[0].message`, stderr is empty, exit 20. So the tests are wrong and the driver is right. Fix: read the message from the stdout JSON document's `diagnostics[].message` in both places. That is a separate ticket, because both failures are pre-existing on main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A repeated `--markdown` is no longer refused. `scanned.values(MARKDOWN)` drops options with no value, so for a flag `count()` is always 0. `fsl check --markdown --markdown a b x.md` now reaches the source read (reproduced: io-error) instead of the usage refusal main gave. Fix: count `scanned.options` entries named `--markdown`. Better: enforce `repeatable: false` in `options::scan` for every option, so the table decides repetition and the per-verb `once()` calls go. | fsl-cli/src/cli.rs:602 |
| FND-002 | medium | `--library` takes a bare `<file>` and binds it by position to the unit's imports, with `unused-library-N` for any extra. FSL FR-866 defines `--library <identity>=<file>`, repeatable, with the same malformed, duplicate and unselected-identity refusals as `--model`. Fix: parse it with `identity_operands(scanned, LIBRARY)`, pass the pairs to `CheckInputs.libraries`, delete the index binding, and update the table value form and the tests (see the Q4 list). | fsl-cli/src/cli.rs:1456-1470 |
| FND-003 | low | `no_option_literal_appears_outside_the_table` scans only cli.rs, provider_config.rs, extension_set.rs and main.rs, but `clocks.rs` spells `--clocks` in three messages, which FR-002-AC-24's no-literal rule covers. Fix: use `options::CLOCKS` in clocks.rs, and have the test scan every file under `src/` except options.rs. | fsl-cli/src/cli.rs:3400; fsl-cli/src/clocks.rs:47 |
| FND-004 | low | The non-Markdown check path drops an extension-catalog refusal with `.ok()` and parses without the catalog. The parse and `--markdown` paths return `set_refusal` for the same error. Fix: return `set_refusal(Operation::Check, refusal)` on `Err`, as the other two paths do. | fsl-cli/src/cli.rs:1407 |
| FND-005 | low | The `inspect` usage line in the table is `inspect --handoff <directory>`, but FR-002 Behavior says root help's `Source operands:` includes `inspect [options] --handoff <directory>`. The filament-docs generator reads that section. Fix: make the table line and the FR-002 sentence the same. | fsl-cli/src/options.rs:293 |
