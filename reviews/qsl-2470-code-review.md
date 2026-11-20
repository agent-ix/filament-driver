---
id: SR-1451
title: "code-review of QSL-2470 compile contracts"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; QSL-2470; cli-contracts/tests/compile.rs, cli-contracts/tests/compile_cases/mod.rs, cli-contracts/tests/compile_cases/options.rs, cli-contracts/COMPILE.md"
review_set: subset
---

## Summary

Reviewed only the four added compile-contract files at 3cf43ca72fc5ab688b7cd176b669836e466615d4. Existing harness/check coverage is outside scope. Accepted peer evidence: 39 active compile cases passed focused checks and scoped Clippy passed. No tests, builds, probes, lint, or aggregate gates were repeated. The final aggregate gate remains the lead’s responsibility. QSL-2658 is a strict ignored pending case, excluded from active coverage. The compile intake-override normative gap remains pending; this review does not claim exhaustive flags completion.

## Verdict

PASS — no code/Rust defects found in the bounded diff; known pending behavior stays explicit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Diff-only manual requirement-to-test binding, reverse ownership, stub and assertion inspection; no repository-wide matrix or inventory was run, as directed. This is not a repository-wide gap-audit claim. Real subprocess outputs, graph references, output bytes and public handoff inspection provide meaningful oracles. Original fixtures are created privately, bounded by the existing harness. No product source or CI gate changed.

## Examined units

```json
{
  "clean": true,
  "scope": [
    {
      "id": "cli-contracts/tests/compile.rs",
      "path": "cli-contracts/tests/compile.rs",
      "role": "examined",
      "excerpt": "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Public compile contracts against the explicitly selected prebuilt FSL.\n\nmod compile_cases;\n"
    },
    {
      "id": "cli-contracts/tests/compile_cases/mod.rs",
      "path": "cli-contracts/tests/compile_cases/mod.rs",
      "role": "examined",
      "excerpt": "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Original request/source programs for the compile command.\n\nuse std::error::Error;\nuse std::fs;\n\nuse fsl_cli_contracts::{Captured, CliCase, ContractError};\nuse serde_json::{json, Value};\n\ntype TestResult = Result<(), Box<dyn Error>>;\nconst SOURCE: &str = r#\"language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\nrecord Parcel { pieces: Int[0, 31]; }\nfunction Total using v(pieces: Integer): Integer pure { pieces + 2 }\n\"#;\n\nfn source(file: &str, identity: &str) -> Value {\n"
    },
    {
      "id": "cli-contracts/tests/compile_cases/options.rs",
      "path": "cli-contracts/tests/compile_cases/options.rs",
      "role": "examined",
      "excerpt": "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Compile option interactions and additional native request forms.\n\nuse super::*;\n\n/// Trace: FR-024-AC-1, FR-003-AC-1\n/// Provenance: QSL-2470\n#[test]\nfn compile_flag_order_and_distinct_repeated_limits_preserve_the_program() -> TestResult {\n    for args in [\n        vec![\n            \"compile\",\n            \"--format\",\n            \"json\",\n            \"--output\",\n            \"combined.json\",\n            \"--clocks\",\n            \"clocks.json\",\n            \"--limit\",\n            \"s1.nodes=10000\",\n    "
    },
    {
      "id": "cli-contracts/COMPILE.md",
      "path": "cli-contracts/COMPILE.md",
      "role": "examined",
      "excerpt": "---\ntype: reference\n---\n# Compile contracts\n\nThe compile cases run an explicitly selected, prebuilt FSL through `CliCase`.\nThey create original sources and requests in private directories, then check\noutcomes, written packages and side effects.\n\nRun the compile group from the repository root:\n\n```sh\nFSL=/absolute/path/to/fsl CARGO_TARGET_DIR=\"$PWD/target\" \\\n  cargo test --locked --manifest-path cli-contracts/Cargo.toml \\\n  --test compile compile_cases:: -- --test-threads=1\n```\n\nThe cases cover single and multiple source units, library selection"
    },
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
      "id": "FR-002-AC-23",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "Over a handoff directory `H` written by `fsl compile` from a `native-compile/1` request carrying FSL FR-665-AC-1's `handoff` member, `fsl inspect --handoff H --format json` writes stdout byte-equal to the canonical encoding of the document `fsl_command::command::inspect_handoff(H)` returns, plus a newline: operation `inspect`, category success, exit 0; without `--format` it writes that document's FSL FR-865 text form, exit 0. Over a copy of `H` whose `selection.json` names one node count more, and over a copy whose `checked-package.json` has one changed byte, each run refuses with the `Selecti"
    },
    {
      "id": "FR-003-AC-1",
      "path": "spec/functional/FR-003-apply-caller-selected-limits.md",
      "role": "examined",
      "excerpt": "Setting one FSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values."
    },
    {
      "id": "FR-003-AC-3",
      "path": "spec/functional/FR-003-apply-caller-selected-limits.md",
      "role": "examined",
      "excerpt": "An input that reaches a caller-lowered bound returns typed `LimitExceeded` with the configured value and field name and FSL's incomplete category; the process status equals FSL's category exit function. Raising the same bound to fit the input allows the operation to complete."
    },
    {
      "id": "FR-003-AC-7",
      "path": "spec/functional/FR-003-apply-caller-selected-limits.md",
      "role": "examined",
      "excerpt": "A malformed `--limit` operand is refused before any stage with code `invalid-request` and identifies that operand."
    },
    {
      "id": "FR-023-AC-2",
      "path": "spec/functional/FR-023-supply-clock-configurations-with-clocks.md",
      "role": "examined",
      "excerpt": "`fsl compile <request> --clocks <file>` on that unit writes the document the request carrying the member gives, and the same request with no clocks refuses `missing-clock-selection`."
    },
    {
      "id": "FR-023-AC-3",
      "path": "spec/functional/FR-023-supply-clock-configurations-with-clocks.md",
      "role": "examined",
      "excerpt": "An unreadable `--clocks` file refuses `io-error`, cause `clocks-file-unreadable`, stage null, exit 20 on `run` and `compile`."
    },
    {
      "id": "FR-023-AC-4",
      "path": "spec/functional/FR-023-supply-clock-configurations-with-clocks.md",
      "role": "examined",
      "excerpt": "A `--clocks` file that is an array, not JSON, or has trailing content refuses `invalid-request`, cause `clocks-file-not-an-object`, naming `--clocks`, stage null."
    },
    {
      "id": "FR-023-AC-5",
      "path": "spec/functional/FR-023-supply-clock-configurations-with-clocks.md",
      "role": "examined",
      "excerpt": "A request carrying `clocks` with `--clocks` refuses `invalid-request` (FSL's refusal, FR-640), naming `clocks` and `--clocks`, exit 20, with the outcome's `last_stage` null (FR-286, FR-287-AC-11), on `run` and `compile`."
    },
    {
      "id": "FR-023-AC-6",
      "path": "spec/functional/FR-023-supply-clock-configurations-with-clocks.md",
      "role": "examined",
      "excerpt": "A member FSL refuses (unknown `kind`) in the file refuses with the code, cause, stage and message the same object gives as the request's `clocks` member, and a repeated member name in the file reaches FSL and is refused naming the clock."
    },
    {
      "id": "FR-023-AC-7",
      "path": "spec/functional/FR-023-supply-clock-configurations-with-clocks.md",
      "role": "examined",
      "excerpt": "A `--clocks` file over the request byte limit refuses `resource_exhausted`, cause `clocks-file-too-large`, stage null, exit 22, naming `--clocks` and the limit: `--request-bytes` on `run`, 1,048,576 bytes on `compile`."
    },
    {
      "id": "FR-024-AC-1",
      "path": "spec/functional/FR-024-write-the-checked-package-with-output.md",
      "role": "examined",
      "excerpt": "`fsl compile <request> --output <file>` over a request without `output` exits 0, writes a file whose bytes equal those `fsl_command::command::compile` returns and those the request carrying the same path as its `output` member writes, resolves a relative path against the working directory, and gives an outcome with `artifacts` `[]` equal to the member's."
    },
    {
      "id": "FR-024-AC-2",
      "path": "spec/functional/FR-024-write-the-checked-package-with-output.md",
      "role": "examined",
      "excerpt": "`--output` naming an existing file refuses `invalid-request`, exit 20, `last_stage` null, naming the path, and the file's bytes are unchanged."
    },
    {
      "id": "FR-024-AC-3",
      "path": "spec/functional/FR-024-write-the-checked-package-with-output.md",
      "role": "examined",
      "excerpt": "A request carrying `output` with `--output` refuses `invalid-request` (FSL's refusal, FR-810), exit 20, naming `output` and `--output`, `last_stage` null, and neither file is written."
    },
    {
      "id": "FR-024-AC-5",
      "path": "spec/functional/FR-024-write-the-checked-package-with-output.md",
      "role": "examined",
      "excerpt": "`--output` given twice, or without a path, refuses the command line naming `--output`, exit 20, and writes no file."
    },
    {
      "id": "FR-024-AC-6",
      "path": "spec/functional/FR-024-write-the-checked-package-with-output.md",
      "role": "examined",
      "excerpt": "A compile that the checker refuses with `--output` exits 20 with the checker's code and writes no file."
    }
  ],
  "findings": [],
  "bindings": []
}
```


## Round 1 follow-up

At fa1cabfdf1bf0a20b98d85dd4348e6f31f6ead3a, inspected only the seven trace-comment changes addressing SR-1452. Code/Rust PASS remains; no new code behavior changed, and no tests/builds/lint were repeated. The final component gate remains the lead’s responsibility.
