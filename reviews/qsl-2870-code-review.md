---
id: SR-202611210022442313144108526
title: "Code review — QSL-2870 CLI run text header"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; fsl-cli/src/cli.rs, fsl-cli/tests/frontend_cli.rs; QSL-2870"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
---

## Summary

Reviewed PR #233 at `21ba0da20ce1d2ccd91ae9793394ee49d9215eef` against base `adae2d29981b5977a81526f1b6596ca651989274`. The CLI's `run` verb now relabels only the first text header, and its JSON path returns before the text-only transformation. The added integration test exercises a clause request whose JSON document says `operation: monitor`, then asserts `run:` in text, plus first-line expectations for check, analyze, and monitor. No build or tests were run; they belong on Titan.

## Verdict

**CONDITIONAL** — one medium contract mismatch remains: FR-002-AC-13 still requires the FSL FR-865 text form of the same JSON document, while this change intentionally rewrites its operation header.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The run-only text header rewrite no longer emits the text form of the JSON outcome document required by FR-002-AC-13. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:341; fsl-cli/src/cli.rs:2524-2529 |

## Review Notes

- Rust review covered the dispatch, format helper, and new CLI integration test. The JSON branch returns before rendering or changing the outcome document; no machine document mutation was found.
- The run test uses a `clause.selection.function` request. It first asserts JSON `operation == "monitor"`, exercising the reported clause path rather than a function-call request.
- The named test asserts the first line for run, check, analyze, and monitor. It also checks run success and the machine operation before text assertions.
- The repository requires builds/tests on Titan. No tests, builds, or other gates were run by this reviewer.

## Examined units

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
      "method": "code-review",
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
| FND-001 | fixed | FR-002-AC-13 now explicitly defines the CLI-command header override and states that the JSON document is unchanged. |

After excerpt (verbatim from amended FR-002-AC-13):

> the text header names the invoked CLI command (`check`, `parse`, `compile` or `run`) instead of the document's operation; the header category and exit code and the diagnostic lines with cause and locus, item lines and result line remain as specified by FR-865. The JSON document and exit status are unchanged.

Disposition reviewed on head `d8106f583b628c0ddf72652e5946c5d510649348`.
