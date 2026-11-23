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
