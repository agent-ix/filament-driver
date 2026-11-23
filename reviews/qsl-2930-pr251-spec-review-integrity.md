---
id: SR-202611210022442313144108531
title: "spec-review integrity of filament-driver PR 251 FR-028 export hint amendment"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-driver; PR #251 at f7d1b09be017eee4a3645d29b39f481cbde9e00a; spec/functional/FR-028-print-a-minimal-request-with-example.md, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md (exception bullet, FR-002-AC-3), spec/test-cases/TC-050-print-a-request-example-and-name-request-defects.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: QSL-2930. Spec-only PR, scoped to `git diff origin/main...HEAD`. FSL facts were checked against FSL PR #3548 head 471324ac79b931fb509679341792e93987764f5e (FR-1047) and the FSL `filament-outcome/1` schema at FSL origin/main 9a2555a09 (`fsl-replay/schemas/filament-outcome-1.schema.json`). `quire validate` over the four changed files exits 0.

## Examined scope

- FR-028 `request_example(format: &str, selection: Option<ExportSelection>)` calls: match FR-1047's signature and its `None` / `Some(Ordinary)` / `Some(Evidence)` use. (examined)
- FR-028 wrong-format bullet and FR-028-AC-9: `invalid-request`, stage `request`, `details` `{"path":"/format","accepted":[...]}`, never `unknown_wire`; the accepted arrays for run, analyze and compile match the FR-1047 family table and FR-1047-AC-5. (examined)
- FR-028 Definitions "Missing entry" and FR-028-AC-5: pointer strings, one-of groups as arrays, depth-first example order, no line/column; the run array matches FR-1047-AC-1 exactly. (examined)
- FR-028-AC-7 `details.missing` `["/request/export"]` and `["/request/export_evidence"]`, no `requires export`: match FR-1047-AC-6; FR-1047's shape sentence contains no `--example`, so the AC-7 "message does not contain `--example`" clause is consistent. (examined)
- Withdrawn FR-028-AC-6: FR-028-AC-8 no longer cites it, TC-050 Description and step 5 drop it, tests.md notes it; no dangling reference found. (examined)
- FR-002 exception bullet and FR-002-AC-3 against FR-028-AC-7 and the FR-028 hint bullet. (examined)
- Hint text, `hint: ` prefix and `driver_hint` name: identical in FR-028 Behavior, FR-028-AC-7, FR-002 and TC-050. (examined)

## Verdict

Not mergeable: two high findings. The FR-1047 alignment, the AC-6 withdrawal and the hint wording are correct and consistent. FND-001 is a contradiction inside FR-028 (an export `{"request":{}}` refusal of AC-5 must and must not carry the hint). FND-002 is a contradiction between the owner-ruled `driver_hint` JSON member and FSL's published `filament-outcome/1` schema, which FR-002-AC-3 and FR-002 Outputs still require the machine-mode document to satisfy; it needs an owner decision, not only an author edit. FR-028 now `depends_on` FSL FR-1047, which exists only on open FSL PR #3548.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-028-AC-7's last sentence says AC-5 refusals carry no `hint:` line and no `driver_hint` member. AC-5 runs each request verb, export included, over `{"format":"<its format>","request":{}}`; for export FR-1047 gives `details.missing` `["/request/models",["/request/program","/request/package"],"/request/export"]`, which contains `/request/export`, so the FR-028 hint bullet ("when the export refusal's details.missing contains /request/export, the driver SHALL append the hint") and FR-002-AC-3 require the hint. The ACs contradict each other for that case | spec/functional/FR-028-print-a-minimal-request-with-example.md:175 |
| FND-002 | high | FR-002-AC-3 (and FR-002 Outputs) still require machine mode to write "one parseable `filament-outcome/1` value ... under FSL's schema", and now allow the FR-028-AC-7 `driver_hint` top-level member. FSL's published `filament-outcome/1` schema sets top-level `unevaluatedProperties: false` and has no `driver_hint` property, so the hinted JSON document is not a valid `filament-outcome/1` value. AC-3's schema clause and AC-7's member cannot both hold; FSL's schema must admit the member, or the driver spec must state that the hinted document is not schema-valid (owner decision, since the member placement is the ruling) | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:365 |
