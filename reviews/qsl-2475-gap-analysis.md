---
id: SR-1460
title: "gap-analysis — QSL-2475 inspect contracts"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; QSL-2475; cli-contracts/tests/inspect.rs and inspect_cases/{mod,artifacts,options,paths}.rs"
review_set: subset
date: 2026-11-20
---

## Summary

Reviewed PR205 at 1c370b11eb53108792766623df2dc573eb2bbfb2. Original real-CLI handoffs, read-back mutations, refusal classes and paths were inspected against Driver FR-002-AC-23/24/25 and locked FSL FR-665-AC-2. The coder's 23 passing focused contracts and scoped Clippy are accepted; no tests, builds, product probes or matrix measurements were repeated.

## Verdict

**PASS** — all recorded findings fixed at 664d1d84ff6c58d5f555e9a0a45b665a915735bb; no new finding in the confined fix diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The bare-path contract exercises JSON only; its required text and DOT not-routed outcomes have no case. | cli-contracts/tests/inspect_cases/options.rs:38 |
| FND-002 | medium | Inspect help checks selected substrings but never requires its options section to be byte-equal to root help. | cli-contracts/tests/inspect_cases/options.rs:171 |

## Required corrections

- FND-001: Add real bare-path invocations for default/explicit text and DOT; assert exit21 and unsupported operation-not-routed naming inspect, using the documented rendering rather than a fabricated graph.
- FND-002: Compare the actual root-help inspect options section with the verb-help section, retaining the existing usage/exit/stderr assertions. A missing or differing option line must fail.

## Coverage

Plan completion: not assessed

This is a user-scoped PR gap review, not a repository assurance audit. Existing harness, earlier contracts and peer evidence are accepted. Global matrix, repository inventory and product reruns were deliberately excluded by the user. Reverse traceability and stub checks were limited to the five changed test files; no fake proof package, external copied fixture, dead assertion or ignored coverage was found there. The conditional generic argument helper permits documented usage/refusal output forms; no extra machine-output location requirement is imposed. Null presentation values are not treated as a new defect without the exact schema obligation. One final component gate remains for the original coder after disposition.

Locked FSL specification directory: /home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/8877b38/spec.

## Examined units

```json
{
  "scope": [
    {
      "id": "FR-002-AC-23",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "`fsl inspect --handoff H --format dot` refuses `invalid-request`, exit 20, naming `--format` and `dot`"
    },
    {
      "id": "FR-002-AC-24",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "For every verb that executes operations and every option `o` in its `fsl-cli` option table entry, `o` appears on its own line in the verb's `<verb> options:` section of root help and the built parser accepts `o` with a well-formed value (no `invalid-request` naming `o`)."
    },
    {
      "id": "FR-002-AC-25",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "For every other verb that executes operations, `fsl <verb> --help` writes that verb's usage lines, a blank line and its `<verb> options:` section byte-equal to root help's, exit 0."
    },
    {
      "id": "FR-665-AC-2",
      "path": "/home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/8877b38/spec/functional/FR-665-write-and-verify-a-checked-package-handoff-through-the-command.md",
      "role": "examined",
      "excerpt": "`verify` over AC-1's directory returns category success with operation `inspect`; over a copy whose `selection.json` names one node count more it refuses with the `SelectionRefusal` axis of that count; over a copy whose `checked-package.json` has one changed byte it refuses with the strict reader's typed refusal."
    },
    {
      "id": "FR-002-AC-18",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "excerpt": "`--format bogus` on each of the four verbs refuses `invalid-request`, exit 20, naming `--format` and `bogus`, before any file is opened."
    }
  ],
  "bindings": [
    {
      "test_id": "a_generated_handoff_is_verified_without_changing_its_artifacts",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "changing_a_selection_node_count_is_not_a_successful_inspection",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "a_changed_package_byte_is_refused_by_the_strict_reader",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "malformed_empty_and_non_utf8_selection_files_are_refused",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "empty_and_non_utf8_package_files_are_refused",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "removing_a_selected_package_is_an_input_read_refusal",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "a_selected_source_directory_or_symlink_cannot_be_read_as_a_regular_file",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "an_oversized_selected_source_is_not_truncated_and_admitted",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "oversized_package_bytes_are_refused_before_decoding",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "oversized_selection_bytes_are_refused_before_decoding",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "forged_parent_relative_and_absolute_source_labels_cannot_escape_the_handoff",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "a_sidecar_with_unknown_members_is_not_admitted",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "invalid_selected_source_does_not_reuse_the_otherwise_valid_package",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "default_and_explicit_text_render_the_inspect_success_document",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "text_and_machine_refusals_keep_the_same_reader_code_and_exit",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "a_bare_inspect_path_is_the_documented_unrouted_form",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "missing_repeated_handoff_and_extra_operands_refuse_before_artifact_io",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "wrong_format_dot_handoff_and_foreign_options_are_refused",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "wrong_format_dot_handoff_and_foreign_options_are_refused",
      "ac_id": "FR-002-AC-24",
      "trace": "correct"
    },
    {
      "test_id": "inspect_help_is_available_without_opening_the_handoff",
      "ac_id": "FR-002-AC-25",
      "trace": "correct"
    },
    {
      "test_id": "invalid_options_are_not_hidden_by_help",
      "ac_id": "FR-002-AC-25",
      "trace": "correct"
    },
    {
      "test_id": "missing_empty_directory_and_regular_file_paths_are_real_refusals",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "unicode_spaces_absolute_paths_and_a_dot_prefixed_dash_directory_are_supported",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    },
    {
      "test_id": "stdin_does_not_replace_the_named_handoff_directory",
      "ac_id": "FR-002-AC-23",
      "trace": "correct"
    }
  ]
}
```

## Dispositions

Round 1, fixed head 664d1d84ff6c58d5f555e9a0a45b665a915735bb. Inspected only the options.rs fix diff; accepted three changed focused tests and scoped Clippy. No tests, build, product probe or full review repeated. Original coder owns the one final component gate.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | New real invocations cover default text, explicit text and DOT, exit21, empty stdout and the named unsupported-operation diagnostic. |
| FND-002 | fixed | The actual root-help inspect section is compared byte-for-byte with every verb-help section. |

```json
{
  "dispositions": [
    {
      "fnd": "FND-001",
      "outcome": "fixed",
      "reason": "New real invocations cover default text, explicit text and DOT, exit21, empty stdout and the named unsupported-operation diagnostic.",
      "after_excerpt": "for args in [\n        vec![\"inspect\", \"missing\"],\n        vec![\"inspect\", \"missing\", \"--format\", \"text\"],\n        vec![\"inspect\", \"missing\", \"--format\", \"dot\"],\n    ] {"
    },
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "reason": "The actual root-help inspect section is compared byte-for-byte with every verb-help section.",
      "after_excerpt": "assert_eq!(\n            inspect_options_section(printed).as_bytes(),\n            root_options.as_bytes()\n        );"
    }
  ]
}
```
