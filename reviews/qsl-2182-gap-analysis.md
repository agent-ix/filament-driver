---
id: SR-17940486709564847047966
title: QSL-2182 driver CLI result fixtures gap-analysis
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/filament-driver; fsl-cli/tests/frontend_cli.rs; QSL-2182, PR #189'
review_set: subset
evaluated_revision: aa9ed4984210a580d32996433cbf7f2bcd5accd6
relationships:
- target: ix://agent-ix/filament-spec-language/FR-650
  type: references
- target: ix://agent-ix/filament-driver/spec/spec.md
  type: references
---

## Summary

PR #189 adds driver CLI tests for four result encodings and activates a source-check refusal test.

## Verdict

CONDITIONAL. The source refusal has evidence. The selected-package refusal remains unverified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The activated source test proves the S3 refusal (exit 20). The checked-package result branch of FR-650-AC-6 still lacks executable evidence for exit 21, unknown_required_feature/unsupported-feature and details.function. At the driver pin, the owning language test is ignored and its named fixture is absent. Keep the source test and record this branch as pending until the package path and its oracle run. | fsl-cli/tests/frontend_cli.rs:1661 |

## Coverage

Quoin 0.36.7 computed 305 criteria across 27 driver requirements. The driver-only matrix contains no FR-650 rows because the language repository owns FR-650. The owner’s contract and test sources were checked at the exact locked revision a2f4ae873c6f0eb8628bb1e79541f6af46ddfa17. The four value encodings and source refusal match that contract. tests/it/run_value_spellings.rs:550 at the pin explicitly ignores the selected-package test. git ls-tree at the pin contains no tests/fixtures/population-collection.checked-package.json. The corrected ticket criteria assign the unsupported result behavior to QSL-2182. No production paths were added. Optional semantic review was not requested.

Plan completion: not assessed

```yaml
scope:
- id: RUN_VALUE_FIXTURE
  path: fsl-cli/tests/frontend_cli.rs
  role: examined
  excerpt: 'type F = Float64[nearest-even];

    type T = Text[0, 8; nfc];

    enum Color { Red, Green }

    record P { x: Int[0, 9]; y: Int[0, 9]; }

    tuple Tp(Int[0, 5], Int[0, 5]);'
- id: run_returns_float_result
  path: fsl-cli/tests/frontend_cli.rs
  role: examined
  excerpt: "fn run_returns_float_result() {\n    let value = serde_json::json!({\"kind\": \"float64\", \"bits\": \"3ff0000000000000\"});\n    assert_run_value(\"\
    id_flt\", value.clone(), value);\n}"
- id: run_returns_text_result
  path: fsl-cli/tests/frontend_cli.rs
  role: examined
  excerpt: "fn run_returns_text_result() {\n    assert_run_value(\n        \"id_txt\",\n        serde_json::json!({\"kind\": \"text\", \"text\": \"e\\u{0301}\"\
    }),\n        serde_json::json!({\"kind\": \"text\", \"text\": \"\\u{00e9}\"}),\n    );\n}"
- id: run_returns_tuple_result
  path: fsl-cli/tests/frontend_cli.rs
  role: examined
  excerpt: "fn run_returns_tuple_result() {\n    let value = serde_json::json!({\"tuple\": {\"type\": \"Tp\", \"components\": [\n        {\"integer\": \"\
    2\"}, {\"integer\": \"1\"}\n    ]}});\n    assert_run_value(\"id_tup\", value.clone(), value);\n}"
- id: run_returns_set_result
  path: fsl-cli/tests/frontend_cli.rs
  role: examined
  excerpt: "/// FSL FR-650-AC-1: Set results use canonical order and remove duplicates.\n#[trace(\"TC-1700\", \"FR-650-AC-1\")]\n#[test]\nfn run_returns_set_result()\
    \ {\n    assert_run_value(\n        \"id_set\",\n        serde_json::json!({\"set\": [{\"integer\": \"3\"}, {\"integer\": \"1\"}]}),\n        serde_json::json!({\"\
    set\": [{\"integer\": \"1\"}, {\"integer\": \"3\"}]}),\n    );"
- id: unsupported_result_refuses_during_check
  path: fsl-cli/tests/frontend_cli.rs
  role: examined
  excerpt: "assert!(document[\"result\"].is_null(), \"{document}\");\n    assert_eq!(document[\"last_stage\"], \"S3\", \"{document}\");\n    let diagnostics\
    \ = document[\"diagnostics\"].as_array().expect(\"diagnostics\");\n    assert_eq!(diagnostics.len(), 1, \"{document}\");\n    let diagnostic = &diagnostics[0];\n\
    \    assert_eq!(diagnostic[\"code\"], \"ill_typed\", \"{document}\");\n    assert_eq!(diagnostic[\"cause\"], \"operator-ineligible\", \"{document}\"\
    );\n    assert_eq!(diagnostic[\"locus\"][\"kind\"], \"region\", \"{document}\");"
- id: FR-650-AC-1
  path: filament-spec-language/spec/functional/FR-650-carry-every-argument-and-result-kind-of-a-run-in-the-native-value-grammar.md
  role: examined
  excerpt: 'Through `run`, each row of the identity-fixture table, as a native-run/1 `call` of its function with its argument `value`, returns one `spine-run-result/1`
    document whose `outcome` is `{"kind": "completed", "value": V}` with `V` the row''s completed value, exit 0.'
- id: FR-650-AC-3
  path: filament-spec-language/spec/functional/FR-650-carry-every-argument-and-result-kind-of-a-run-in-the-native-value-grammar.md
  role: examined
  excerpt: '`id_set` with `[{"integer": "1"}, {"integer": "1"}]`, `duplicate-member`, `/set/1`; `id_seq` with four elements, `cardinality_out_of_bound`/`above-maximum`,
    `""`; `id_tup` with one component, `invalid-value`, `/tuple/components`; `id_bool` with `2`, `invalid_runtime_input`/`invalid-value`, `""`.'
- id: FR-650-AC-6
  path: filament-spec-language/spec/functional/FR-650-carry-every-argument-and-result-kind-of-a-run-in-the-native-value-grammar.md
  role: examined
  excerpt: 'A native-run/1 request whose `package` member selects the fixture package `tests/fixtures/population-collection.checked-package.json` (the compiled
    ConfigVersion package with the result type of its function `gives`, and the parameter type of its function `takes`, rewritten to that collection type,
    as an IR producer can write them) refuses `unknown_required_feature`/`unsupported-feature` at stage `call`, exit 21, before any call, with `details`
    `{"function": "gives"}` for `gives` and `{"position": 0}` for `takes`. A function whose parameter or result type is `Quantity<s>`, `Map<String, Int[0,
    9'
```

## Dispositions

Round 1 at `4d22ae4bb9ff415009a69cd4f6f0e68cfce52f23`.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | QSL-2390 owns native-run selection of checked-package/v2 artifacts. The staged driver test asserts exit 21 at S6a, unknown_required_feature/unsupported-feature and details.function = gives. Explicit execution returned exit 20 at request intake because program was missing. An unmodified compiler-emitted package returned the same diagnostic. Enable and rerun the staged assertion after the driver consumes the selection path. |

## Verdict (disposition pass 1)

PASS for this PR with the checked-package result obligation deferred to QSL-2390. The current intake error occurs before the package bytes are read, so neither package admission nor the S6a refusal has been demonstrated. The focused result-filter log records 17 passed and 1 ignored. The narrower gate passed the 13 result-kind tests and the source-check refusal. The staged package probe and independent intake probe both returned exit 20.
