---
id: SR-1452
title: "gap-analysis of QSL-2470 compile contracts"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; QSL-2470; cli-contracts/tests/compile.rs, cli-contracts/tests/compile_cases/mod.rs, cli-contracts/tests/compile_cases/options.rs, cli-contracts/COMPILE.md"
review_set: subset
---

## Summary

Reviewed only the four added compile-contract files at 3cf43ca72fc5ab688b7cd176b669836e466615d4. Existing harness/check coverage is outside scope. Accepted peer evidence: 39 active compile cases passed focused checks and scoped Clippy passed. No tests, builds, probes, lint, or aggregate gates were repeated. The final aggregate gate remains the lead’s responsibility. QSL-2658 is a strict ignored pending case, excluded from active coverage. The compile intake-override normative gap remains pending; this review does not claim exhaustive flags completion.

## Verdict

CONDITIONAL — correct the four trace-binding groups before counting their criteria coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Flag-order success does not establish effective limit propagation or preservation of other defaults: all supplied limits are ample, so ignoring every --limit operand still passes. Drop FR-003-AC-1 from this ordering test or add a behavioral differential that observes the selected bound and an omitted bound. | cli-contracts/tests/compile_cases/options.rs:6-54 |
| FND-002 | medium | Default request/source ceiling tests claim the caller-lowered-bound criterion without supplying --limit or demonstrating successful completion after raising that bound. They still pass if caller limit overrides are broken. Bind these cases to default intake/source admission instead; retain the separate real caller-lowered source test. | cli-contracts/tests/compile_cases/mod.rs:536-551 |
| FND-003 | medium | Malformed output records and output/handoff containment tests claim the request-output-plus-CLI-output conflict criterion, but neither invocation supplies --output. Both pass if dual-output conflict detection is removed. Use their actual request/handoff admission ownership instead; retain the existing real dual-output conflict case. | cli-contracts/tests/compile_cases/options.rs:457-504 |
| FND-004 | medium | Output-filesystem failures and an exit-22 emission limit claim the checker-refusal exit-20 criterion despite compiling a well-typed source. These pass if checker-refusal output suppression breaks. Remove the checker-refusal binding from these cases and preserve their actual output/limit ownership; the ill-typed-source case already asserts the correct criterion. | cli-contracts/tests/compile_cases/options.rs:188-224,526-547 |

## Coverage

Plan completion: not assessed

Diff-only manual requirement-to-test binding, reverse ownership, stub and assertion inspection; no repository-wide matrix or inventory was run, as directed. This is not a repository-wide gap-audit claim. Real subprocess outputs, graph references, output bytes and public handoff inspection provide meaningful oracles. Original fixtures are created privately, bounded by the existing harness. No product source or CI gate changed.

## Examined units

```json
{
  "clean": false,
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
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "medium",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "trace",
      "artifact_id": "FR-003-AC-1",
      "related_ids": [],
      "path": "cli-contracts/tests/compile_cases/options.rs",
      "lines": "6-54",
      "excerpt": "Setting one FSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values.",
      "finding": "Flag-order success does not establish effective limit propagation or preservation of other defaults: all supplied limits are ample, so ignoring every --limit operand still passes. Drop FR-003-AC-1 from this ordering test or add a behavioral differential that observes the selected bound and an omitted bound.",
      "test_id": "compile_flag_order_and_distinct_repeated_limits_preserve_the_program",
      "claimed_ac": "FR-003-AC-1",
      "correct_ac": "FR-002-AC-1"
    },
    {
      "fnd": "FND-002",
      "severity": "medium",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "trace",
      "artifact_id": "FR-003-AC-3",
      "related_ids": [],
      "path": "cli-contracts/tests/compile_cases/mod.rs",
      "lines": "536-551",
      "excerpt": "An input that reaches a caller-lowered bound returns typed `LimitExceeded` with the configured value and field name and FSL's incomplete category; the process status equals FSL's category exit function. Raising the same bound to fit the input allows the operation to complete.",
      "finding": "Default request/source ceiling tests claim the caller-lowered-bound criterion without supplying --limit or demonstrating successful completion after raising that bound. They still pass if caller limit overrides are broken. Bind these cases to default intake/source admission instead; retain the separate real caller-lowered source test.",
      "test_id": "oversized_request_refuses_at_intake_with_its_configured_limit, default_source_size_limit_is_distinct_from_the_aggregate_dependent_budget",
      "claimed_ac": "FR-003-AC-3",
      "correct_ac": "FR-002-AC-1"
    },
    {
      "fnd": "FND-003",
      "severity": "medium",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "trace",
      "artifact_id": "FR-024-AC-3",
      "related_ids": [],
      "path": "cli-contracts/tests/compile_cases/options.rs",
      "lines": "457-504",
      "excerpt": "A request carrying `output` with `--output` refuses `invalid-request` (FSL's refusal, FR-810), exit 20, naming `output` and `--output`, `last_stage` null, and neither file is written.",
      "finding": "Malformed output records and output/handoff containment tests claim the request-output-plus-CLI-output conflict criterion, but neither invocation supplies --output. Both pass if dual-output conflict detection is removed. Use their actual request/handoff admission ownership instead; retain the existing real dual-output conflict case.",
      "test_id": "malformed_output_records_refuse_before_writing_a_package, an_output_inside_the_requested_handoff_is_refused_without_partial_artifacts",
      "claimed_ac": "FR-024-AC-3",
      "correct_ac": "FR-002-AC-1"
    },
    {
      "fnd": "FND-004",
      "severity": "medium",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "trace",
      "artifact_id": "FR-024-AC-6",
      "related_ids": [],
      "path": "cli-contracts/tests/compile_cases/options.rs",
      "lines": "188-224,526-547",
      "excerpt": "A compile that the checker refuses with `--output` exits 20 with the checker's code and writes no file.",
      "finding": "Output-filesystem failures and an exit-22 emission limit claim the checker-refusal exit-20 criterion despite compiling a well-typed source. These pass if checker-refusal output suppression breaks. Remove the checker-refusal binding from these cases and preserve their actual output/limit ownership; the ill-typed-source case already asserts the correct criterion.",
      "test_id": "output_directory_and_missing_parent_refuse_without_partial_files, an_emission_byte_limit_refuses_without_publishing_partial_package_bytes",
      "claimed_ac": "FR-024-AC-6",
      "correct_ac": "FR-002-AC-1"
    }
  ],
  "bindings": [
    {
      "test_id": "native_source_emits_a_package_that_the_public_handoff_reader_verifies",
      "ac_id": "FR-024-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "native_source_emits_a_package_that_the_public_handoff_reader_verifies",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "request_and_cli_output_paths_emit_equal_bytes_and_ignore_document_revision",
      "ac_id": "FR-024-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "absolute_request_from_another_cwd_resolves_dependencies_beside_the_request",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "absolute_request_from_another_cwd_resolves_dependencies_beside_the_request",
      "ac_id": "FR-024-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "multiple_source_units_retain_both_declarations",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "default_and_explicit_text_describe_the_same_success_as_machine_mode",
      "ac_id": "FR-002-AC-13",
      "trace": "correct"
    },
    {
      "test_id": "named_request_is_used_even_when_stdin_contains_invalid_bytes",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "an_existing_output_file_is_preserved_without_compiling",
      "ac_id": "FR-024-AC-2",
      "trace": "correct"
    },
    {
      "test_id": "request_output_and_cli_override_refuse_before_writing_either_target",
      "ac_id": "FR-024-AC-3",
      "trace": "correct"
    },
    {
      "test_id": "ill_typed_source_preserves_its_stage_and_writes_no_partial_output",
      "ac_id": "FR-024-AC-6",
      "trace": "correct"
    },
    {
      "test_id": "absent_or_directory_requests_keep_the_io_refusal_in_json_and_text",
      "ac_id": "FR-002-AC-18",
      "trace": "correct"
    },
    {
      "test_id": "malformed_request_bytes_never_become_a_successful_package",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "closed_compile_request_rejects_runtime_and_unknown_fields",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "source_selection_requires_both_labels_before_reading_a_file",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "non_utf8_source_reports_encoding_and_no_package",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "missing_or_directory_dependent_source_is_not_an_empty_program",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "empty_syntax_and_unsupported_source_fail_in_their_own_classes",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "oversized_request_refuses_at_intake_with_its_configured_limit",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "oversized_request_refuses_at_intake_with_its_configured_limit",
      "ac_id": "FR-003-AC-3",
      "trace": "wrong"
    },
    {
      "test_id": "primary_program_bytes_count_toward_the_compile_dependent_budget",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "caller_lowered_source_limit_refuses_and_a_fresh_default_call_succeeds",
      "ac_id": "FR-003-AC-3",
      "trace": "correct"
    },
    {
      "test_id": "invalid_arguments_name_the_operand_and_do_not_create_outputs",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "invalid_arguments_name_the_operand_and_do_not_create_outputs",
      "ac_id": "FR-003-AC-7",
      "trace": "correct"
    },
    {
      "test_id": "invalid_arguments_name_the_operand_and_do_not_create_outputs",
      "ac_id": "FR-024-AC-5",
      "trace": "correct"
    },
    {
      "test_id": "clock_file_failures_are_bounded_typed_and_precede_the_request",
      "ac_id": "FR-023-AC-3",
      "trace": "correct"
    },
    {
      "test_id": "clock_file_failures_are_bounded_typed_and_precede_the_request",
      "ac_id": "FR-023-AC-4",
      "trace": "correct"
    },
    {
      "test_id": "clock_file_failures_are_bounded_typed_and_precede_the_request",
      "ac_id": "FR-023-AC-7",
      "trace": "correct"
    },
    {
      "test_id": "compile_flag_order_and_distinct_repeated_limits_preserve_the_program",
      "ac_id": "FR-024-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "compile_flag_order_and_distinct_repeated_limits_preserve_the_program",
      "ac_id": "FR-003-AC-1",
      "trace": "wrong"
    },
    {
      "test_id": "timestamped_clock_file_and_request_member_compile_the_same_temporal_binding",
      "ac_id": "FR-023-AC-2",
      "trace": "correct"
    },
    {
      "test_id": "a_clock_member_and_override_conflict_before_reading_the_source",
      "ac_id": "FR-023-AC-5",
      "trace": "correct"
    },
    {
      "test_id": "an_invalid_clock_kind_is_refused_equally_from_file_and_member",
      "ac_id": "FR-023-AC-6",
      "trace": "correct"
    },
    {
      "test_id": "output_directory_and_missing_parent_refuse_without_partial_files",
      "ac_id": "FR-024-AC-2",
      "trace": "correct"
    },
    {
      "test_id": "output_directory_and_missing_parent_refuse_without_partial_files",
      "ac_id": "FR-024-AC-6",
      "trace": "wrong"
    },
    {
      "test_id": "unknown_language_preserves_its_documented_refusal_code",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "unknown_edition_preserves_its_documented_refusal_code",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "bom_and_nul_are_refused_instead_of_normalizing_the_source",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "another_wire_format_and_an_empty_program_are_not_compile_successes",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "duplicate_request_fields_and_program_clause_bindings_are_rejected",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "blank_source_labels_report_the_exact_invalid_identity_field",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "a_supplied_import_is_compiled_into_the_dependency_closure",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "explicit_stdin_file_reads_the_request_without_replacing_absolute_source_selection",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "stdin_does_not_supply_a_missing_request_operand",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "a_non_utf8_request_filename_is_preserved_as_an_os_path",
      "ac_id": "FR-002-AC-1",
      "trace": "correct"
    },
    {
      "test_id": "malformed_output_records_refuse_before_writing_a_package",
      "ac_id": "FR-024-AC-3",
      "trace": "wrong"
    },
    {
      "test_id": "an_output_inside_the_requested_handoff_is_refused_without_partial_artifacts",
      "ac_id": "FR-024-AC-3",
      "trace": "wrong"
    },
    {
      "test_id": "default_source_size_limit_is_distinct_from_the_aggregate_dependent_budget",
      "ac_id": "FR-003-AC-3",
      "trace": "wrong"
    },
    {
      "test_id": "an_emission_byte_limit_refuses_without_publishing_partial_package_bytes",
      "ac_id": "FR-003-AC-3",
      "trace": "correct"
    },
    {
      "test_id": "an_emission_byte_limit_refuses_without_publishing_partial_package_bytes",
      "ac_id": "FR-024-AC-6",
      "trace": "wrong"
    }
  ]
}
```


## Dispositions

Round 1, head fa1cabfdf1bf0a20b98d85dd4348e6f31f6ead3a. Only the seven changed trace-comment lines were inspected; assertions and fixtures are unchanged. No tests, lint, builds, gates, or framework review repeated.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Removed the unrelated criterion binding; retained actual assertions and provenance. |
| FND-002 | fixed | Removed the unrelated criterion binding; retained actual assertions and provenance. |
| FND-003 | fixed | Removed the unrelated criterion binding; retained actual assertions and provenance. |
| FND-004 | fixed | Removed the unrelated criterion binding; retained actual assertions and provenance. |

### Round 1 verdict

PASS for the reviewed binding fixes. All four findings are fixed. The original findings remain unchanged above. QSL-2658 and the intake-option normative gap remain pending, outside completion claims. The final component gate remains pending with the lead.

```json
{
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "after_excerpt": "/// Trace: FR-024-AC-1\n/// Provenance: QSL-2470\n#[test]\nfn compile_flag_order_and_distinct_repeated_limits_preserve_the_program() -> TestResult"
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "after_excerpt": "/// Trace: FR-002-AC-1\n/// Provenance: QSL-2470; FSL FR-027-AC-3\n#[test]\nfn oversized_request_refuses_at_intake_with_its_configured_limit() -> TestResult\n\n/// Provenance: QSL-2470; FSL FR-001 source admission ceiling\n#[test]\nfn default_source_size_limit_is_distinct_from_the_aggregate_dependent_budget() -> TestResult"
    },
    {
      "fnd": "FND-003",
      "outcome": "fixed",
      "after_excerpt": "/// Provenance: QSL-2470; FSL FR-810-AC-2\n#[test]\nfn malformed_output_records_refuse_before_writing_a_package() -> TestResult\n\n/// Provenance: QSL-2470; FSL FR-810-AC-5\n#[test]\nfn an_output_inside_the_requested_handoff_is_refused_without_partial_artifacts() -> TestResult"
    },
    {
      "fnd": "FND-004",
      "outcome": "fixed",
      "after_excerpt": "/// Trace: FR-024-AC-2\n/// Provenance: QSL-2470; FSL FR-810-AC-2\n#[test]\nfn output_directory_and_missing_parent_refuse_without_partial_files() -> TestResult\n\n/// Trace: FR-003-AC-3\n/// Provenance: QSL-2470; FSL FR-869-AC-2\n#[test]\nfn an_emission_byte_limit_refuses_without_publishing_partial_package_bytes() -> TestResult"
    }
  ]
}
```
