---
id: SR-2026112305304500000001
title: "spec-review integrity of filament-driver PR 248 FR-028 and TC-050"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-driver; PR #248 at 008807a584f4fe5db2292899c6f28b74b84c411d; spec/functional/FR-028-print-a-minimal-request-with-example.md, spec/test-cases/TC-050-print-a-request-example-and-name-request-defects.md, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md (one bullet), spec/spec.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: QSL-2930. Spec-only PR. Scope is `git diff origin/main...HEAD`. FSL facts were checked against a read-only clone of agent-ix/filament-spec-language at origin/main e642bea. Driver facts were checked against FR-002, FR-027 and `fsl-cli/src/options.rs` at the PR head.

## Examined scope

- The FR-028 claim that no `native-export/1` format exists: true. The string appears nowhere in FSL spec or code. The export request is `native-run/1` with an `export` member (FSL FR-1015 line 27, `fsl-command/src/command/export.rs:141-157`) or an `export_evidence` member (FSL FR-1022). Driver FR-027 lines 31 and 36 agree. (examined)
- The current-behaviour quotes in the FR-028 Description: true. `the selected export invocation requires {required}` (export.rs:155), `unsupported native command format` (command.rs:889, code `unknown_wire` at command.rs:1216, stage `envelope` at output.rs:195), and the serde `invalid request: ...` with line and column details (command.rs:738, output.rs:187). (examined)
- The monitor required members in FR-028-AC-5 (`models`, `program`, `clauses`, `trace`, in that order) match `MonitorRequest` in FSL `fsl-command/src/command/wire.rs:951`. (examined)
- The FR-028-AC-4 precedence agrees with FR-002 steps 1 to 4: a bad `--format` refuses first, an unknown `--example` on check, parse, format and inspect refuses in step 2, and help wins in step 3. `--limit` is in the run table entry, so `fsl run --example --limit ...` reaches the step-4 check that names both arguments. (examined)
- Consistency with FSL FR-287: the export and `--evidence` request forms and the "the driver writes the FSL outcome unchanged" rule agree, except for the `--evidence` value form in FND-001. (examined)
- The dependency on FSL entries that do not exist: FR-028 Dependencies says the request-example entry and the refusal text are not yet specified in FSL and that FR-028 is not implementable until they are. The tests.md row is 🚧 Planned and says the same. There is no request-example entry in FSL today. This is stated correctly. (examined)
- spec.md `contains` edges, index rows, tests.md counts and rows, and the TC-050 `verifies` edge are consistent. (examined)

## Verdict

Not mergeable. There are two high findings. FR-028 treats `--evidence` as a flag, but FR-002, FR-027 and FSL FR-287 make it a valued option. The "no optional member" rule makes the `run` and `export` examples invalid, which contradicts AC-3. The `native-export/1` claim, the quoted current messages, the FR-002 precedence and the statement of the unmet FSL dependency are all correct.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-028 treats `--evidence` as a flag, in Inputs, Behavior, AC-2 (`fsl export --evidence --example`, `fsl export --example --evidence`) and TC-050 step 2. The driver option table (`fsl-cli/src/options.rs` export entry: `valued(EVIDENCE, "<request-file>")`), FR-027 line 36 and FSL FR-287 (`fsl export --evidence <request>`) make it a valued option. So `--evidence --example` reads `--example` as the request-file path, and `--example --evidence` lacks a value. Both AC-2 cases are unsatisfiable without changing FR-002, FR-027 and FR-287, and FR-028 changes none of them | spec/functional/FR-028-print-a-minimal-request-with-example.md:128 |
| FND-002 | high | The rule that `request` "carries every member the verb's reader requires and no optional member" contradicts AC-3 for `run` and `export`. In FSL the only serde-required member of `native-run/1` is `models` (wire.rs:275). `program` and `package` are each optional, but exactly one is required: `required_program` refuses `invalid-request` with "a request selects exactly one of program and package" (command.rs:4964). A minimal example is therefore refused at stage `request`. The text does not define "required" for one-of groups, so AC-5's run message `missing required member /request/models` and the example's content can each be read two ways | spec/functional/FR-028-print-a-minimal-request-with-example.md:85 |
| FND-003 | medium | FR-028 says it "keeps no copy of a request shape", but AC-5 copies FSL's request shapes: monitor's required-member list and order (`models, program, clauses, trace`, mirroring FSL wire.rs:951) and run's. AC-7 copies the verb-to-format table, and AC-5 to AC-8 author the exact refusal text, labelled "FSL's contract, pinned here", before FSL specifies it. The driver spec becomes a second owner of FSL's wire contract, and it drifts when FSL adds a required member. AC-9 already gives the pass-through check (byte-equal to FSL's entry). The exact text and member lists belong in the FSL FRs that FR-028 depends on | spec/functional/FR-028-print-a-minimal-request-with-example.md:131 |
| FND-004 | medium | The exact text of AC-8 cannot be determined. The message holds backticks (`run `fsl export --example` for one`) inside a backtick code span, so Markdown splits it and readers cannot tell whether the backticks belong to the message. AC-8 also calls it "the message reason", while AC-5 to AC-7 say "diagnostic message". FSL's `ClauseRequest` has separate `member` and `reason` fields, so it is unclear which output field the exact string must equal | spec/functional/FR-028-print-a-minimal-request-with-example.md:134 |
| FND-005 | low | The `export --evidence` request reader, FSL FR-1022 (the `export_evidence` member), is missing from the `depends_on` edges and from the Dependencies list. FR-028 lists FR-1015 only for export, but AC-2, AC-3 and AC-8 constrain the evidence reader's example and refusal | spec/functional/FR-028-print-a-minimal-request-with-example.md:17-27 |

## New findings (disposition pass 1)

These were found at head f51257b499fc3a462dcfdb4434f3681bc3e4dd24.

The new `--example-evidence` option was judged against FR-002, FR-027 and FSL FR-287, and it is consistent with all three. It is a no-value entry in the export option table, so FR-002-AC-24 covers its help listing. `--evidence <request-file>` stays valued, as FR-027:36 and FSL FR-287 state. FR-287's `--evidence` refusal rules are untouched. FSpec FR-301 was not checked, because no clone was available.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | The new "Required member" definition counts each member of a one-of group as required ("each member of a one-of group that the reader requires exactly one of"). The example rule then says `request` "SHALL carry every required member, one member of each one-of group, and no other member". Read literally, that requires both `program` and `package`, which the "Invalid member" definition makes invalid. It is also unclear whether a run refusal over `"request":{}` must list both `/request/program` and `/request/package` as missing (AC-5), or one, or a group pointer | spec/functional/FR-028-print-a-minimal-request-with-example.md:73-77 |
| FND-007 | medium | AC-9 names `fsl_command::command::run` (or FSL's export entry) as the oracle for every refusal in AC-5 to AC-7. Those ACs cover `compile`, `analyze`, `monitor` and `replay` too, and FSL has separate entries for them (`compile_outcome`, `analyze`, `monitor`, ...) whose outcome `operation` differs. A correct driver would therefore fail byte-equality against `run`'s document. The oracle should be the FSL entry for the verb under test | spec/functional/FR-028-print-a-minimal-request-with-example.md:152 |
| FND-008 | low | `--example-evidence` is missing in two places. The FR-002 pointer bullet still says only "The request verbs take `--example`". TC-050 step 4's Expected Results omit the `fsl run --example-evidence` unknown-option result and the `--example-evidence` help-listing check that AC-4 requires | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:332 |
| FND-009 | low | The fix dropped the refusal code from the wrong-format case. AC-7 and its Behavior bullet now state only "refuses ... exit 20" with no code, where the previous text and FSL (`Code::UnknownWire`, command.rs:1216) give `unknown_wire`. A refusal under any code passes AC-7 | spec/functional/FR-028-print-a-minimal-request-with-example.md:150 |

## New findings (disposition pass 2)

These were found at head a664ab18dd737054431fc27985ed894216f250df.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | low | Two problems with the one-of group form. First, Dependencies does not name it as an FSL change. It says only that the request refusal text is not yet specified in FSL. Today FSL reports a missing one-of group (program or package) as a separate refusal: required_program, command.rs:4964, runs only after serde accepts the required members. Listing it together with the missing required members, in the form /request/program or /request/package, is new FSL reader behaviour, but nothing says so. Second, the driver Definitions fix the pointer form joined by ' or ', while the Behavior bullet says the message form is FSL's. The spec therefore authors part of FSL's message form while claiming not to | spec/functional/FR-028-print-a-minimal-request-with-example.md:79-81 |

## New findings (disposition pass 3)

These were found at head 27249bb081e1b12c6ba76902dcc64a2d020218e6.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | high | AC-9's reference documents for run and replay are FSL entries that execute, not refuse, the wrong-format requests AC-7 requires them to refuse. FSL run_with_providers and run_with_providers_and_limits_and_overrides both reach run_configured, which dispatches on format: a native-monitor/1 request is run as monitor (command.rs:2040) and a native-replay/1 request is replayed (command.rs:2042). So for fsl run over a native-replay/1 or native-monitor/1 request, and fsl replay over native-run/1 or native-monitor/1, the named entry returns a run outcome, not the unknown_wire refusal AC-7 requires. AC-7 and AC-9 cannot both hold. The replay wrong-format refusal is the driver's own today (fsl-cli/src/cli.rs:2015, FR-026-AC-4), which also contradicts the rule that the driver adds no refusal text. 'Every format FSL's reader for that verb accepts' is undefined when the entry accepts more formats than the verb reads. The FSL entry must be made to refuse by verb (a new FSL dependency), or AC-9 must exclude these pairings | spec/functional/FR-028-print-a-minimal-request-with-example.md:158 |
| FND-012 | high | AC-7 requires unknown_wire for every wrong-format pairing, including a native-analyze/1 request given to run, replay or export, and a native-run/1 request given to analyze. FSL FR-620-AC-9 specifies those pairings as invalid-request at the request stage, and FSL implements that at command.rs:3973-3980 (member format, 'is not read here'). FR-028 does not list FR-620-AC-9 as an FSL contract to change, so AC-7 contradicts an existing FSL requirement for those pairings | spec/functional/FR-028-print-a-minimal-request-with-example.md:156 |

## New findings (disposition pass 4)

These were found at head 2829a5cd5c64ff9c8835b0bad0e835ea73cc5800.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-013 | medium | AC-5 is ambiguous about which verbs its first two documents go to. Read the natural way (each of the three documents, given to each request verb), `{"format":"native-run/1"}` given to analyze, compile, monitor or replay would be a format-mismatch refusal: invalid-request on format for analyze, per FSL FR-620-AC-9, and unknown_wire for the others. It would not be the `/request` missing-member refusal AC-5 requires, so under that reading the AC is wrong for those verbs. TC-050 step 5 runs both documents only through `fsl run`. The AC should say `fsl run` for `{}` and `{"format":"native-run/1"}` | spec/functional/FR-028-print-a-minimal-request-with-example.md:146 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `--evidence <request-file>` is unchanged. The evidence example is now `fsl export --example-evidence`, a no-value option on export only, so AC-2 no longer conflicts with FR-002, FR-027 or FSL FR-287 |
| FND-002 | fixed | A Definitions section defines one-of groups and names `program` or `package` for run. AC-3 now requires the run example to carry exactly one of them. The leftover wording conflict is recorded as new finding FND-006 |
| FND-003 | fixed | The exact message strings and member lists were removed from AC-5, AC-7 and AC-8. These ACs now defer to FSL's reader and the FSL FRs. The message form is FSL's |
| FND-004 | fixed | AC-8 now asserts member `export` or `export_evidence` and reason substrings, with no nested backticks |
| FND-005 | fixed | FSL FR-1022 was added to the `depends_on` edges and to Dependencies |
| FND-006 | fixed | verified in fix commit a664ab18 |
| FND-007 | still-open | AC-9 now maps compile to fsl_command::command::run. The driver calls compile_outcome_with_overrides (fsl-cli/src/cli.rs:1124, FSL FR-287), not run, and FSL's run refuses a native-compile/1 request on format, so a correct driver still fails byte-equality for compile. Its claim that FR-002 says which entry each verb calls is also unsupported: FR-002 names run and inspect_handoff only. analyze, monitor and replay are named loosely (FSL's analyze entry, its monitor entry), and FR-026 says replay goes through run_with_providers. The run, replay and export mappings are acceptable |
| FND-008 | fixed | verified in fix commit a664ab18 |
| FND-009 | fixed | verified in fix commit a664ab18 |
| FND-007 | fixed | verified in fix commit 27249bb0; entry names match fsl-cli/src/cli.rs and FSL |
| FND-010 | fixed | verified in fix commit 27249bb0 |
| FND-011 | fixed | verified in 2829a5cd; the wrong-format AC was dropped and the AC-8 oracle now covers only the missing-member and export refusals |
| FND-012 | fixed | verified in 2829a5cd; the unknown_wire AC was removed and Dependencies keeps FSL FR-620-AC-9 unchanged |
