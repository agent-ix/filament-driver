---
id: SR-1453
title: "code-review review of QSL-2471 run contracts"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver; QSL-2471; cli-contracts/tests/run.rs; cli-contracts/tests/run_cases/"
review_set: subset
---

## Summary

Independent diff-only code and Rust review of the new public run contract cases. 2 medium test-oracle findings need correction.

## Verdict

CONDITIONAL — strengthen the listed assertions in the same change. No product bug is claimed by this review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The text-result oracle accepts an incorrect computed result such as 112 because it only searches for the substring 12; assert the documented result line/value and compare both text modes against the JSON result. | cli-contracts/tests/run_cases/options.rs:22 |
| FND-002 | medium | Malformed provider configurations are accepted as evidence without asserting invalid-request or the offending member; an io-error naming bad-provider.json passes all current assertions. Assert the required code and member for each fixture. | cli-contracts/tests/run_cases/options.rs:342 |

## Coverage

Frozen candidate f144c347afccec52268b7b3cdb441f3d9cf44955; only run.rs and run_cases new files. Accepted peer evidence: 23 active focused tests and scoped Clippy passed; 2 strict pending tests name QSL-2348 and QSL-2661 and are not counted as coverage. No tests, builds, CLI probes or repository-wide matrix rerun performed. The lead owns the final gate. Helpers and prior check/compile cases are excluded. No production code or CI changes are in the reviewed slice.

## Examined units

```json
[
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
    "id": "FR-002-AC-20",
    "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
    "role": "examined",
    "excerpt": "`fsl run R --format json`, `R` FSL FR-931-AC-1's request `{\"clause\": \"Low\"}` over the ConfigVersion model, writes the `tl_lowering` record with `disposition` `supported`, `correspondence` `mltl.closed-trace/v1` and `formula` `G[0,3] p0`, exit 0, and its stdout is byte-equal to the canonical encoding of the document FSL's `fsl_command::command::run` returns for `R`, plus a newline; it never writes the \"not run by the driver\" input refusal. `fsl run R --limit s1.nodes=1 --format json` writes FSL's typed limit outcome naming `s1.nodes` with configured value 1, category `incomplete`, no lowering r"
  },
  {
    "id": "FR-002-AC-21",
    "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
    "role": "examined",
    "excerpt": "`fsl run P --format json`, `P` FSL FR-1039-AC-1's request selecting the `half` unit's checked package as `package` `{file}` with the `call` `half`, run with the source file removed, completes with `-4`, exit 0, byte-equal to FSL's `run` for `P` plus a newline; the same request with `integer_division` gives FSL's `invalid-request` refusal naming `integer_division`, exit 20."
  },
  {
    "id": "FR-002-AC-24",
    "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
    "role": "examined",
    "excerpt": "For every verb that executes operations and every option `o` in its `fsl-cli` option table entry, `o` appears on its own line in the verb's `<verb> options:` section of root help and the built parser accepts `o` with a well-formed value (no `invalid-request` naming `o`). For every verb that executes operations and every option name `p` that is listed in another verb's entry or is one of `--version`, `-V`, `--color` and `--bogus`, but is not in this verb's entry, `fsl <verb> p` refuses `invalid-request`, exit 20, naming `p`. Removing one option from a copy of the table and running the parser an"
  },
  {
    "id": "FR-002-AC-25",
    "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
    "role": "examined",
    "excerpt": "`fsl check --help`, `fsl check -h` and `fsl check nosuch.fsl --help`, from an empty working directory, each write exactly the following to stdout, exit 0, write no stderr and read no file: `Usage: fsl check [options] <source-path>` / `       fsl check [options] <authority> <identity> <source-path>` / (blank) / `check options:` / `  --markdown` / `  --model <identity>=<file> (repeatable)` / `  --library <identity>=<file> (repeatable)` / `  --dependent-bytes <bytes>` / `  --limit name=value (repeatable)` / `  --integer-division euclidean\\|floor\\|truncating` / `  --extensions <file>` / `  --forma"
  },
  {
    "id": "FR-002-AC-26",
    "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
    "role": "examined",
    "excerpt": "`fsl run R --color always`, `fsl run R --color never` and `--color` on every other verb that executes operations refuse `invalid-request`, exit 20, naming `--color`, before `R` or any other named file is read, and no help section contains `--color`. Over inputs that contain no `0x1B` byte, no `fsl` invocation of TC-049 writes the byte `0x1B` to stdout or stderr, with or without `NO_COLOR` set."
  },
  {
    "id": "FR-003-AC-1",
    "path": "spec/functional/FR-003-apply-caller-selected-limits.md",
    "role": "examined",
    "excerpt": "Setting one FSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values."
  },
  {
    "id": "FR-003-AC-2",
    "path": "spec/functional/FR-003-apply-caller-selected-limits.md",
    "role": "examined",
    "excerpt": "A setting name not resolved by FSL's operand parser is refused before any stage with code `invalid-request` and names the offending operand."
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
    "id": "FR-003-AC-8",
    "path": "spec/functional/FR-003-apply-caller-selected-limits.md",
    "role": "examined",
    "excerpt": "A duplicate `--limit` setting is refused before any stage with code `invalid-request` and identifies the duplicate setting."
  },
  {
    "id": "FR-006-AC-24",
    "path": "spec/functional/FR-006-host-provider-plugins.md",
    "role": "examined",
    "excerpt": "The `--provider-config` file accepts only FSpec FR-561's `filament.provider-config/v1` shape (`format`, `providers`, entries with `path` and optional `arguments`); a missing or unreadable file refuses naming the path; a non-JSON document, a missing `format` or `providers`, an unknown member at any object level (including an entry member `deadline_ms` or `limits`), an entry with no or a blank `path`, and `arguments` that are not strings refuse `invalid-request` naming the file and the member; `format` `filament.provider-config/v2` refuses `unknown_wire`/`unsupported-wire`; each exits 20 with em"
  },
  {
    "id": "FR-006-AC-28",
    "path": "spec/functional/FR-006-host-provider-plugins.md",
    "role": "examined",
    "excerpt": "`fsl run --provider-config providers.json` with no `--provider-limit` sends the fixture plugin a request whose `deadline_ms` is 600000 and holds it to a 67108864-byte frame limit; with `--provider-limit deadline_ms=250 --provider-limit max_frame_bytes=1048576` the request carries `deadline_ms` 250 and a 1048577-byte result frame settles the item `failed` with the frame-length tool failure naming limit 1048576; `--provider-limit` is accepted without `--provider-config`."
  },
  {
    "id": "FR-006-AC-29",
    "path": "spec/functional/FR-006-host-provider-plugins.md",
    "role": "examined",
    "excerpt": "`fsl run` with `--provider-limit max_frames=4`, `deadline_ms`, `deadline_ms=0`, `deadline_ms=10s`, `deadline_ms=010`, `max_total_bytes=9007199254740992`, or `deadline_ms=5` given twice each refuses `invalid-request` naming `--provider-limit` and the argument, exits 20 with empty stdout under both `--format text` and `--format json`, and starts no plugin; `fsl parse --provider-limit deadline_ms=5` and `fsl parse --provider-config providers.json` refuse the option as unknown, exit 20."
  },
  {
    "id": "FR-023-AC-1",
    "path": "spec/functional/FR-023-supply-clock-configurations-with-clocks.md",
    "role": "examined",
    "excerpt": "`fsl run <request> --clocks <file>` on a unit with a timestamped-event clause on clock `plant-ms` exits with the outcome and writes the document the same request carrying the file's object as its `clocks` member gives."
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
    "id": "FR-023-AC-7",
    "path": "spec/functional/FR-023-supply-clock-configurations-with-clocks.md",
    "role": "examined",
    "excerpt": "A `--clocks` file over the request byte limit refuses `resource_exhausted`, cause `clocks-file-too-large`, stage null, exit 22, naming `--clocks` and the limit: `--request-bytes` on `run`, 1,048,576 bytes on `compile`."
  },
  {
    "id": "cli-contracts/tests/run.rs",
    "path": "cli-contracts/tests/run.rs",
    "role": "examined",
    "excerpt": "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Real run requests exercise the frozen external FSL executable.\nmod run_cases;\n"
  },
  {
    "id": "cli-contracts/tests/run_cases/functions.rs",
    "path": "cli-contracts/tests/run_cases/functions.rs",
    "role": "examined",
    "excerpt": "use super::*;\n\n/// Trace: FR-002-AC-1\n/// Provenance: QSL-2471\n#[test]\nfn integer_arguments_compute_results_and_each_invocation_has_fresh_inputs(\n) -> Result<(), Box<dyn std::error::Error>> {\n    let case = fixture()?;\n    for (capacity, reserved, expected) in [(17, 5, 12), (0, 0, 0), (2, 9, -7), (-3, -8, 5)] {\n        write_json(\n            &case,\n            \"run.json\",\n            &request(\n                \"packing.fsl\",\n                \"Remaining\",\n                &[\n                    (\"total\", json!({\"integer\":capacity.to_string()})),\n                    (\"held\", json!({\"integer\":reser"
  },
  {
    "id": "cli-contracts/tests/run_cases/inputs.rs",
    "path": "cli-contracts/tests/run_cases/inputs.rs",
    "role": "examined",
    "excerpt": "use super::*;\n\n/// Trace: FR-002-AC-1\n/// Provenance: QSL-2471\n#[test]\nfn request_relative_absolute_and_parent_paths_keep_the_request_base(\n) -> Result<(), Box<dyn std::error::Error>> {\n    let case = fixture()?;\n    case.write(\"requests/nested/anchor\", b\"\")?;\n    for path in [\n        \"../../packing.fsl\".to_owned(),\n        case.directory()\n            .join(\"packing.fsl\")\n            .to_str()\n            .expect(\"UTF8\")\n            .to_owned(),\n    ] {\n        write_json(\n            &case,\n            \"requests/nested/run.json\",\n            &request(\n                &path,\n                "
  },
  {
    "id": "cli-contracts/tests/run_cases/mod.rs",
    "path": "cli-contracts/tests/run_cases/mod.rs",
    "role": "examined",
    "excerpt": "use fsl_cli_contracts::{Captured, CliCase, ContractError};\nuse serde_json::{json, Value};\n\nmod functions;\nmod inputs;\nmod options;\nmod temporal;\n\nconst SOURCE: &str = r#\"language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\nfunction Remaining using v(total: Integer, held: Integer): Integer pure {\n    total - held\n}\nfunction IsPacked using v(n: Int[0, 10]): Boolean pure { n >= 6 }\n\"#;\n\nfn request(source: &str, name: &str, args: &[(&str, Value)]) -> Value {\n    json!({\"format\":\"native-run/1\",\"request\":{\n        \"models\":[],\"program\":{\"source\":{\"file\":source,\"authority"
  },
  {
    "id": "cli-contracts/tests/run_cases/options.rs",
    "path": "cli-contracts/tests/run_cases/options.rs",
    "role": "examined",
    "excerpt": "use super::*;\n\n/// Trace: FR-002-AC-13, FR-002-AC-18\n/// Provenance: QSL-2471\n#[test]\nfn default_and_explicit_text_match_json_status_and_preserve_value_and_refusal(\n) -> Result<(), Box<dyn std::error::Error>> {\n    let case = fixture()?;\n    for (path, exit, category) in [(\"run.json\", 0, \"success\"), (\"missing.json\", 20, \"refusal\")] {\n        let machine = outcome(&run(&case, path, &[])?, exit, category);\n        let default = case.run(&[\"run\", path])?;\n        let explicit = case.run(&[\"run\", path, \"--format\", \"text\"])?;\n        assert_eq!(default.0.status.code(), Some(exit));\n        assert_e"
  },
  {
    "id": "cli-contracts/tests/run_cases/temporal.rs",
    "path": "cli-contracts/tests/run_cases/temporal.rs",
    "role": "examined",
    "excerpt": "use super::*;\n\nconst STAMPED: &str = r#\"language \"ix:native\" edition \"1-draft\";\nprofile v = \"filament.value.complete/v1\";\nprofile t = \"filament.temporal.timestamped-event.finite-window/v1\";\nrecord Pressure { level: Int[0, 10]; }\ntemporal SafePressure using t over (p: Pressure) clock \"pump-ms\" on origin {\n    always[0,2] holds(p.level <= 6)\n}\nfunction Constant using v(): Integer pure { 19 }\n\"#;\nfn clocked(case: &CliCase) -> Result<Value, ContractError> {\n    case.write(\"pressure.fsl\", STAMPED.as_bytes())?;\n    Ok(request(\"pressure.fsl\", \"Constant\", &[]))\n}\nfn clocks() -> Value {\n    json!({\"pum"
  }
]
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Both text modes parse exactly one completed result and compare its complete value with the independently asserted JSON result 12. |
| FND-002 | fixed | Active extra/path cases assert code and actual member. The arguments case retains the strict member oracle under tracked QSL-2663 and is not counted as coverage. |

## Disposition round 1 verdict

PASS — all original test-oracle findings are fixed at 5cfdcb4c006a9e01218977b0c513aff18e28bc14. No additional findings in the fix diff. Current count is 26 tests: 23 active and 3 strict ignored cases (QSL-2348, QSL-2661, QSL-2663), which provide no active coverage. Accepted peer evidence: the three affected focused filters and scoped Clippy passed. No tests, lint, builds, probes or matrix reruns were performed during this disposition. The lead owns the final gate.
