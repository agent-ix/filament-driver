---
id: SR-1464
title: "Gap analysis — QSL-2476 real CLI exit contracts"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; QSL-2476; cli-contracts/src/lib.rs additive stdout sink; cli-contracts/tests/exit_codes.rs and exit_cases/"
review_set: subset
---

## Summary

Ticket: QSL-2476. Reviewed PR207 frozen at 10ef9a1aafa004c99b97f2f70db914723bc614cb.

## Verdict

**PASS** — no findings in the changed surface.

The changed tests account for all ten category labels and all seven documented numeric exits across thirteen recognized verbs. Category cases include genuine undefined evaluation, proof/refutation, incomplete and inconclusive analysis, supplied-trace pending/indeterminate, streamed success/violation and a real internal output failure. Mixed refusal ordering exercises severity rather than numeric maximum. No ignored exit cases or stubbed semantic engine is introduced. This is a changed-surface gap check under the user's explicit scope, not a repository-wide matrix claim. Plan completion: not assessed

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Review confined to PR207 at 10ef9a1aafa004c99b97f2f70db914723bc614cb. The staged-stdin change 43292c1 (same-repository cherry-pick 817c40c) was already reviewed and delivered in PR206 and was accepted without another review. No tests, product builds, matrix, repository inventory or product probes ran. Coder focused evidence (12 CLI tests plus one OS helper; scoped Clippy) is accepted. The single final applicable component gate remains with the original coder after this review. Linux /dev/full exercises non-BrokenPipe stdout failure; this review makes no macOS exit-30 claim.

Plan completion: not assessed

## Examined scope

- cli-contracts/src/lib.rs (examined), cli-contracts/src/lib.rs:     pub fn run_stdout_to_file<S: AsRef<OsStr>>(
- cli-contracts/tests/exit_cases/mod.rs (examined), cli-contracts/tests/exit_cases/mod.rs: const CATEGORY_EXITS: [(&str, i32); 10] = [     ("success", 0),     ("violation", 10),     ("undefined", 10),     ("refusal", 20),     ("unsupported", 21),     ("incomplete", 22),     ("inconclusive", 22),     ("pending", 0),     ("indeterminate", 11),     ("internal-failure", 30), ];
- cli-contracts/tests/exit_cases/categories.rs (examined), cli-contracts/tests/exit_cases/categories.rs:     let sink = std::fs::OpenOptions::new().write(true).open("/dev/full")?;
- cli-contracts/tests/exit_cases/verbs.rs (examined), cli-contracts/tests/exit_cases/verbs.rs: fn every_routed_verb_preserves_twenty_for_missing_input() -> Result<(), ContractError>
- cli-contracts/tests/exit_codes.rs (examined), cli-contracts/tests/exit_codes.rs: mod exit_cases;
- Driver FR-002-AC-2 (examined), spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md: For each FSL category whose exit result is not reserved code `5`, the process status equals the value returned by FSL's `Category::exit_code`; no second category-to-exit mapping is applied. Reserved result `5` is governed by AC-6.
- Driver FR-002-AC-4 (examined), spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md: The CLI passes FSL's `Category::exit_code` result through `ix_cli_kit::exit::caller_exit_code` and preserves each accepted numeric value without mapping it through the kit's typed `Outcome`.
- Driver FR-002-AC-15 (examined), spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md: If writing or flushing primary stdout returns an I/O error other than `BrokenPipe`, the process exits with FSL's `Category::InternalFailure` exit result. The criterion makes no assertion that stdout is empty or complete.
- Driver FR-002-AC-16 (examined), spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md: If a non-`BrokenPipe` primary stdout write or flush fails and stderr is writable, stderr contains an `OutputFailure` diagnostic that identifies the failed output.
- FSL FR-285 (examined), /home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/8877b38/spec/functional/FR-285-map-every-outcome-category-to-one-exit-code.md: A multi-item outcome exits with the most severe code present, in the order 30, 20, 21, 22, 10, 11, 0 (FSpec FR-301's order with 11 added below 10).
- FSL FR-285-AC-12 (examined), /home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/8877b38/spec/functional/FR-285-map-every-outcome-category-to-one-exit-code.md: A finite-window `Pending`, `Indeterminate`, `Completed(true)`, `Completed(false)` and `Incomplete` outcome has the label and exit `pending`/0, `indeterminate`/11, `success`/0, `violation`/10 and `incomplete`/22.
- Driver FR-002-AC-1 (examined), spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md: The driver binary exposes the command and output surface defined by FSL ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation.
- Driver FR-002-AC-23 (examined), spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md: Over a handoff directory `H` written by `fsl compile` from a `native-compile/1` request carrying FSL FR-665-AC-1's `handoff` member, `fsl inspect --handoff H --format json` writes stdout byte-equal to the canonical encoding of the document `fsl_command::command::inspect_handoff(H)` returns, plus a newline: operation `inspect`, category success, exit 0; without `--format` it writes that document's FSL FR-865 text form, exit 0. Over a copy of `H` whose `selection.json` names one node count more, and over a copy whose `checked-package.json` has one changed byte, each run refuses with the `Selecti
- Driver FR-022-AC-1 (examined), spec/functional/FR-022-analyze-a-native-analyze-request.md: `fsl analyze` over the Counter request with the item `{"claim": "Bounded"}` and `--format json` writes one document with `operation` `analyze`, category `success`, one item whose record is `proved`, and `aggregate.status` `complete`, and exits 0.
- Driver FR-022-AC-2 (examined), spec/functional/FR-022-analyze-a-native-analyze-request.md: The item `{"claim": "BelowThree"}` gives an item whose record is `refuted`, with a `counterexample` entry and `replay` `{"target": {"state_clause": Q}, "reproduced": true}`, `Q` ending in `BelowThree`, category `violation`, and exit 10.
- Driver FR-022-AC-3 (examined), spec/functional/FR-022-analyze-a-native-analyze-request.md: A request whose item sets `temporal.limits.max_states` to 1 gives an `incomplete` record whose cause names `max_states`, with no `counterexample`, and exit 22; a request with `deadline_ms` 0 gives an `incomplete` record whose cause is `{"kind": "cancelled", "source": "deadline"}`, and exit 22.
- Driver FR-022-AC-4 (examined), spec/functional/FR-022-analyze-a-native-analyze-request.md: An item naming a backend the registry does not hold settles with category `refusal` and the record diagnostic `invalid_capability`/`unknown-backend` naming the backend, exit 20; an item over the Counter fixture naming backend `en-6`, which lists no temporal-satisfaction form, settles `unsupported` with `unsupported_projection`/`unsupported-requested-capability`, exit 21 (the item kind is incidental: EN-1 serves derived deadlock-freedom since QSL-2122, and a state-graph claim over a timed subject stays unsupported until QSL-2305); neither writes empty output.
- Driver FR-022-AC-9 (examined), spec/functional/FR-022-analyze-a-native-analyze-request.md: With `--provider-config` starting a process provider whose `hello` advertises (`value-validity`, `unbounded`), an item naming its backend is sent to it as one FR-331 request at the item's request index and settles `proved` with certification `trusted` when the provider answers `proved`, and `inconclusive` when it answers anything else; the document's `registry` lists the provider and `en-1`; another item is unchanged by it. A provider advertising only `bounded`, an unreadable configuration and a malformed one each refuse with exit 20.
- Driver FR-025-AC-1 (examined), spec/functional/FR-025-stream-observations-with-fsl-monitor.md: `fsl monitor` over `Reaches` and `Never` with levels 0, 1, 2, 1 then `{"end": true}` writes event lines then one summary line and exits 10, with a `settled` `violation` event for `Never` at record 2; over `Never` alone with levels 0, 1, 0 it exits 0.
- Driver FR-025-AC-2 (examined), spec/functional/FR-025-stream-observations-with-fsl-monitor.md: The lines the binary writes equal, byte for byte, the canonical encodings of the events and summary `command::monitor` hands back in process for the same request and records, and the exit code equals the summary's.
- FSL FR-624-AC-4 (examined), /home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/8877b38/spec/functional/FR-624-name-every-analyze-refusal-and-every-unrouted-verb.md: For each of `generate`, `prove`, `replay`, `inspect`, `render` and `monitor` invoked in a build that does not route it, the driver binary writes a document with that `operation`, category `unsupported`, code `unsupported_operation`, cause `operation-not-routed` and payload `verb` naming it, and exits 21; stdout is never empty.
