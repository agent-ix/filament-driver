---
id: SR-202611210022442313144108516
title: "code-review — PR224 export admission"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; QSL-2667/QSL-2669/QSL-2670; fsl-cli/src/cli.rs; fsl-cli/tests/export_cli.rs; cli-contracts/tests/export_cases/intake.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-027
    type: references
---

## Summary

Reviewed only `aa71f20dd30bab9ae2ca66580ca407e434164cca..4aa1ae4dd0bd3106633ecd1fc452bac7fc5410b1` for PR224 (stacked on PR223), QSL-2667/QSL-2669/QSL-2670. No spec, dependency pin, lockfile, parser, or CI change belongs to this delta. No applicable AssuranceProfile was found. The frozen source was not edited.

Rust lane folded into code-review.

## Verdict

**PASS** — second/final narrow disposition found no new defect; focused runtime results and stacked rebase custody are satisfied. Mergeable after artifact-only custody.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

The ordinary/evidence enum survives argument parsing. Early CLI conflicts precede file access. The driver delegates one path, selection, default byte ceilings and output/stage overrides to the pinned FSL export entry, with no pre-read, fallback, duplicate parser or driver target write. Pinned FSL source confirms one bounded request read, duplicate-preserving raw admission, closed envelope/required mode checks before dependent reads, request-parent path resolution and lifecycle output. Both output-overrides errors and typed intake errors retain FSL's document/category. Existing source rendering behavior is not changed by this diff.

The restored regressions retain strict ordinary-in-evidence refusal, equal/unequal duplicate rejection ahead of missing-source I/O, exact undeclared-obligation 21 and absolute source selection, plus file/stream byte parity and bounded request failures. CliCase concurrently drains bounded pipes and reaps children on timeout/error. The package regression selects real equality bytes emitted with a byte-identical producer package; its source is removed, and equality bytes remain unchanged. No hand-built equality/package proof substitute appears. The evidence test first requires successful real export, takes its package object, then checks the exact null-stage/null-result pending capability and malformed-tool path. It does not claim proof/tool execution or binding. FSL FR-1022-AC-3 authorizes that current pending boundary; AC-1/2 remain future work.

Local rustfmt --edition 2021 --check over all three changed Rust files and git diff --check passed. No local build, runtime, Clippy, broad gate, proof tool, install, fetch or push was run. Supplied service evidence for unchanged production at 7b: product build/Clippy pass; intake 13 passed and one intentional QSL-2672 ignore; options 8 passed. That earlier unit selection did not compile, and export_cli had two failures. Head 4aa fixes tests only. The four unit tests, two previously failed export tests, and targeted Clippy at this exact head remain PENDING; earlier failures are not reported as passes.

Merge also requires parent223's composition QA, merge with ancestry preserved, child retarget/rebase and verification of the resulting frozen head and relevant tests/review/custody. All three tickets stay open until driver QA and merge. Five source-model staged ignores and QSL-2672 remain excluded future work, not passing coverage.

## Examined units

```json
{
  "clean": true,
  "scope": [
    {
      "id": "FR-002-AC-1",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "The driver binary exposes the command and output surface defined by FSL ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation."
    },
    {
      "id": "FR-002-AC-13",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "`fsl check <file>`, `fsl parse <authority> <identity> <file>`, `fsl compile <request>` and `fsl run <request>` without `--format`, and with `--format text`, each over a succeeding and a refused input, write to stdout the FSL FR-865 text form of the document `--format json` writes for the same invocation (header `<operation>: <category> (exit <code>)`, diagnostic lines with cause and locus, item lines, result line) and exit with the same status as under `--format json`; none returns `UnimplementedCapability` or exit 21."
    },
    {
      "id": "FR-002-AC-18",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "`fsl run nosuch.json` and `fsl compile nosuch.json` without `--format` refuse with the I/O failure's FSL code and exit 20, as under `--format json`, never `UnimplementedCapability` or exit 21, and stdout holds the text form of that refusal document; the text of a refused `check` names each diagnostic's code, cause and source position; `--format bogus` on each of the four verbs refuses `invalid-request`, exit 20, naming `--format` and `bogus`, before any file is opened."
    },
    {
      "id": "FR-027-AC-1",
      "path": "spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md",
      "role": "examined",
      "excerpt": "`fsl export <request> --output m.tla --format json` over FR-756-AC-1's counter request (`inc`, `value <= 5`) exits 0, writes `m.tla` whose bytes equal the document's `result.export.target_text`, and writes FSL's outcome document with operation `export`, category `success` and `result.kind` `exported`; the request carrying `export.output` naming `m.tla`, run with no option in a fresh directory, writes a byte-identical file."
    },
    {
      "id": "FR-027-AC-2",
      "path": "spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md",
      "role": "examined",
      "excerpt": "`--output` naming an existing file refuses `invalid-request`, exit 20, `last_stage` null, naming the path, and the file's bytes are unchanged; a request carrying `export.output` with `--output` refuses naming `output` and `--output`, exit 20, and writes neither file; `--output` given twice refuses naming `--output`, exit 20."
    },
    {
      "id": "FR-027-AC-3",
      "path": "spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md",
      "role": "examined",
      "excerpt": "`fsl export <request> --output m.tla --limit export.output_bytes=1` exits 22, names `export.output_bytes` and writes no file; with `--limit export.output_bytes=16777216` it exits 0."
    },
    {
      "id": "FR-027-AC-4",
      "path": "spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md",
      "role": "examined",
      "excerpt": "`fsl export <request> --output m.tla` with no `--format` writes exactly the four lines `export: success (exit 0) at S4`, `target: tla filament.output.tla2/v1`, `bytes written: <n>` and `output: m.tla`, and exits as in machine mode."
    },
    {
      "id": "FR-027-AC-5",
      "path": "spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md",
      "role": "examined",
      "excerpt": "`fsl export --evidence e.json --format json` writes FSL's evidence outcome document with operation `export`; `fsl export --evidence e.json --output m.tla` and `fsl export <request> --evidence e.json` each refuse `invalid-request` naming `--evidence`, exit 20, with nothing written to the output path; `fsl compile <request> --evidence e.json` refuses as an unknown option, exit 20."
    },
    {
      "id": "fsl-cli/src/cli.rs",
      "path": "fsl-cli/src/cli.rs",
      "role": "examined",
      "excerpt": "selection: ExportSelection,"
    },
    {
      "id": "fsl-cli/tests/export_cli.rs",
      "path": "fsl-cli/tests/export_cli.rs",
      "role": "examined",
      "excerpt": "fn export_evidence_routes_to_the_intake_and_refuses_other_arguments() {"
    },
    {
      "id": "cli-contracts/tests/export_cases/intake.rs",
      "path": "cli-contracts/tests/export_cases/intake.rs",
      "role": "examined",
      "excerpt": "fn oversized_streams_match_file_byte_limits_on_both_routes("
    },
    {
      "id": "FSL-FR-1022-AC-3",
      "path": "/home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/f597712/spec/functional/FR-1022-take-in-tla-and-lean-results-as-trusted-evidence.md",
      "role": "examined",
      "excerpt": "Until original generated-package binding and target-result verification are implemented, a well-shaped `export_evidence` input carrying an assembled export's `package`, a nonempty obligation, a declared tool and a nonempty artifact path is admitted and then refused with `unimplemented_capability` / `unimplemented-meaning`, `details.capability` `export_evidence`, category `unsupported`, exit `21`, with null `last_stage` and `result`. No artifact is read or tool probed or run; no proof, counterexample or package binding is accepted. Malformed evidence is refused at its own member before this cap"
    },
    {
      "id": "FSL-FR-026",
      "path": "/home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/f597712/spec/functional/FR-026-run-standalone-native-workflow.md",
      "role": "context_only",
      "excerpt": "For a run request, the command shall open each file once and cap reads before\nparsing."
    }
  ],
  "findings": [],
  "bindings": [
    {
      "test_id": "export_evidence_routes_to_the_intake_and_refuses_other_arguments",
      "ac_id": "FR-027-AC-5",
      "trace": "correct"
    }
  ],
  "reviewed_head": "4aa1ae4dd0bd3106633ecd1fc452bac7fc5410b1",
  "reviewed_base": "aa71f20dd30bab9ae2ca66580ca407e434164cca",
  "run": "56490d1c-518c-4bad-a737-f9fcabcf7c0d"
}
```

## Dispositions

Second and final narrow disposition at `c2edfbe1a294e136fc26a5d46c10a7f0d82f72b4`, based on merged parent223 `891ca755cf742712d7ff4f4adfa3208b775948f6`. Initial findings remain unchanged below; this disposition updates their current status and replaces colliding provisional artifact IDs using the installed reviewer skill's numeric repo-plus-scratch scan, with Python integers. No Quoin ID allocator exists or was needed.

Reviewed only the test corrections from 4aa to 2b0c and the mechanical rebase. The complete binary export diff `aa71..2b0c` equals `891ca..c2edfbe` (cmp exit 0); both SHA-256 values are `f999f2fa2b4c553e0d701c3fc17fb2e92037c452d4cc6c80c21a9dfcc7c6affd`. Exactly three export files remain in scope. Production, pins and lockfile have no correction-round changes. Parent223's source and its review artifacts are now inherited through the merged parent graph.

The corrected selected-package test requires actual exit 0, S4, exported/assembled success with no diagnostics; it preserves the caller's source identity/version, executable target semantics, truthful per-route native provenance comments, name-map graph references checked against the owning reader's resolved graph, and represented definition regions. It compares every non-provenance target line exactly and validates the provenance differences rather than erasing them. Target bytes equal the full unmodified returned text; output path and byte count match; producer package/equality bytes remain intact after source removal. This is a legitimate contract-grounded success oracle, not a synthetic package/equality or weakened unsupported check.

Accepted actual service results supplied by root: Titan simple224-2b0c-c1 selected test rc0, 1 passed / 0 failed / 0 ignored; c2 focused export Clippy rc0. Retained earlier 27 other passing checks plus four unit tests and one pending-evidence test give 33 passed, with one intentional QSL-2672 ignore. These are selected checks, not a full-suite claim. No runtime/build/lint/matrix baseline was rerun by this reviewer. First-review pending/failure evidence remains historical, not retroactively relabeled.

The bounded parent223/PR224 exception is accepted because the original exact-base comparison proved no new criterion or qualified-binding gap, and the narrow test corrections remove no driver criterion binding. The renamed package test retains its existing cross-repository trace attribute. The whole-repository matrix stays FAIL: 175 untagged and five ignored-only criteria, plus the recorded doc-Trace extraction limitation; no engine-backed PASS or ignored-test pass is claimed.

All source/test findings are clear. **Mergeable after custody of these latest two validated artifacts**, with the exact reviewed source head above and an artifact-only trailing custody commit. No further semantic review round is authorized or needed. Tracker closing/state changes remain deferred under the Peter/ARCH hold; root owns the six prepared single updates afterward and ticket completion only after merge. This reviewer made no Linear call in the final disposition.

Initial verdict (historical): **CONDITIONAL** — no new source or test defect found in the exact delta; exact-head runtime reruns and stacked merge prerequisites remain pending.

No source/test finding required a fix. The corrected stale selected-package oracle was checked as the final narrow disposition; no third round was started.

Final reviewed source head: `c2edfbe1a294e136fc26a5d46c10a7f0d82f72b4`. Final reviewed parent: `891ca755cf742712d7ff4f4adfa3208b775948f6`.
