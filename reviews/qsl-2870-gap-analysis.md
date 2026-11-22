---
id: SR-202611210022442313144108527
title: "Gap analysis — QSL-2870 CLI run text header"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; fsl-cli/src/cli.rs, fsl-cli/tests/frontend_cli.rs; QSL-2870"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
---

## Summary

Audited the QSL-2870 delta at `21ba0da20ce1d2ccd91ae9793394ee49d9215eef` against base `adae2d29981b5977a81526f1b6596ca651989274`. The change is owned by FR-002-AC-13 and the added test exercises the clause-run path, verifies JSON operation `monitor`, then checks the text header `run`; it also checks first lines for check, analyze, and monitor. The implementation and test agree with QSL-2870. They do not agree with the current local FR-002-AC-13 text-rendering contract, which remains unamended.

## Verdict

**CONDITIONAL** — the delta adds one contract mismatch against FR-002-AC-13. The static matrix is unchanged from base. Plan completion: not assessed

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The changed `run` text header is not the FSL FR-865 rendering of the JSON document as FR-002-AC-13 requires. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:341; fsl-cli/src/cli.rs:2524-2529 |

## Coverage

The planless static matrix was computed on archived base and head trees with `quoin matrix --repo <tree> --json`. Both contain 320 criteria: 169 untagged, 136 tagged, 10 method-without-symbol and 5 tagged-by-ignored-test. No criterion status delta was found. The new test's TC-2456 / FR-865 trace tags refer to the FSL contract; its additional FR-022-AC-7 and FR-025-AC-8 bindings are correct for the analyze and monitor text assertions. No source/test stub or unowned new behavior was found in the changed code. The changed run-header behavior is covered by FR-002-AC-13, but contradicts that criterion's current wording; this mismatch is the only delta finding. Semantic review is included for the changed behavior. Reverse-gap and stub review were bounded to the changed source and test. No tests/builds were run because the execution lane is Titan-only.

Plan completion: not assessed

## Findings Detail

```json
{
  "clean": false,
  "scope": [
    {
      "id": "FR-002-AC-13",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "`fsl check <file>`, `fsl parse <authority> <identity> <file>`, `fsl compile <request>` and `fsl run <request>` without `--format`, and with `--format text`, each over a succeeding and a refused input, write to stdout the FSL FR-865 text form of the document `--format json` writes for the same invocation (header `<operation>: <category> (exit <code>)`, diagnostic lines with cause and locus, item lines, result line) and exit with the same status as under `--format json`; none returns `UnimplementedCapability` or exit 21."
    },
    {
      "id": "FR-022-AC-7",
      "path": "spec/functional/FR-022-analyze-a-native-analyze-request.md",
      "role": "examined",
      "excerpt": "In text mode the AC-2 request prints `0 BelowThree refuted` at the start of its first line, `violation` as its last line, and exits 10; an item naming an unregistered backend prints `0 Bounded invalid-request invalid_capability/unknown-backend`."
    },
    {
      "id": "FR-025-AC-8",
      "path": "spec/functional/FR-025-stream-observations-with-fsl-monitor.md",
      "role": "examined",
      "excerpt": "In text mode, over AC-1's fixture and records, stdout holds one event line `<record> <clause> <category> settled|pending` per event, then one `<clause> <category> settled|pending` line per clause and the line `records <n> refused <m>`, and the exit is 10."
    },
    {
      "id": "fsl-cli/src/cli.rs",
      "path": "fsl-cli/src/cli.rs",
      "role": "examined",
      "excerpt": "if let Some((_, rest)) = rendered.split_once(\": ") { rendered = format!(\"{command_name}: {rest}\"); }"
    },
    {
      "id": "fsl-cli/tests/frontend_cli.rs",
      "path": "fsl-cli/tests/frontend_cli.rs",
      "role": "examined",
      "excerpt": "fn text_command_first_lines_cover_run_check_analyze_and_monitor()"
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "medium",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "soundness",
      "artifact_id": "FR-002-AC-13",
      "related_ids": [],
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "lines": "341",
      "excerpt": "`fsl check <file>`, `fsl parse <authority> <identity> <file>`, `fsl compile <request>` and `fsl run <request>` without `--format`, and with `--format text`, each over a succeeding and a refused input, write to stdout the FSL FR-865 text form of the document `--format json` writes for the same invocation (header `<operation>: <category> (exit <code>)`, diagnostic lines with cause and locus, item lines, result line) and exit with the same status as under `--format json`; none returns `UnimplementedCapability` or exit 21.",
      "finding": "The new run clause path has JSON operation `monitor`, but the text renderer's first header is rewritten to `run`. That is the requested user-facing result, yet it is no longer the FSL FR-865 text form of the same JSON document that FR-002-AC-13 requires. Update the local contract and TC-040 to define this command-label translation, or the product must preserve the FSL-rendered operation label."
    }
  ],
  "bindings": [
    {"test_id": "text_command_first_lines_cover_run_check_analyze_and_monitor", "ac_id": "FR-025-AC-8", "trace": "correct"},
    {"test_id": "text_command_first_lines_cover_run_check_analyze_and_monitor", "ac_id": "FR-022-AC-7", "trace": "correct"}
  ]
}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-002-AC-13 and TC-040 now define the command-label translation, including the `run`/`monitor` JSON-versus-text case. |

After excerpt (verbatim from amended FR-002-AC-13):

> the text header names the invoked CLI command (`check`, `parse`, `compile` or `run`) instead of the document's operation; the header category and exit code and the diagnostic lines with cause and locus, item lines and result line remain as specified by FR-865. The JSON document and exit status are unchanged.

Disposition reviewed on head `d8106f583b628c0ddf72652e5946c5d510649348`.
