---
id: SR-1465
title: "code-review of QSL-2478 help/version contracts"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-driver PR209 QSL-2478; cli-contracts/tests/help_version.rs, cli-contracts/tests/help_cases/mod.rs, cli-contracts/tests/help_cases/display.rs, cli-contracts/tests/help_cases/parser.rs; Driver FR-002-AC-12/17/24/25"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
---

## Summary

Confined review of PR209 at d3d823d92414a72517beb927b3d984822c4631c5. The new tests exercise actual CLI help/version streams and the documented same-repository production metadata seam; one exact help-contract assertion remains missing.

## Verdict

**CONDITIONAL** — fix the confined medium finding below before the final component gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Compare each verb's actual options-section bytes with root help; parsed-map equality currently accepts reordered option lines, violating the byte-equal contract. | cli-contracts/tests/help_cases/display.rs:125 |

## Detail

sections() stores options in a BTreeMap and normalizes the observed line representation. The test compares those maps, so reordering only standalone option lines leaves this assertion green. Keep the useful metadata comparisons, and additionally compare the actual complete options-section byte slices from root and standalone help.

## Coverage

- All four changed Rust files examined; private test helpers, SPDX headers, trace tags, actual output/status/stderr assertions and ignored-test ownership reviewed. No product, dependency, workflow or public API changes.
- Driver FR-002-AC-17, AC-24 and AC-25 examined; AC-12 existing kit Agreement mechanism is context only, with additive version-value checks reviewed. The current metadata table/scanner and documented path-integration seam were read only as needed for the new tests; delivered QSL-2602 was not re-audited.
- Reconciliation: confined direct criterion-to-test trace inspection; no matrix rerun under the user's explicit no-remeasurement policy. No repository-wide assurance claim.
- Plan completion: not assessed
- New tests: eight runnable focused passes and scoped Clippy pass accepted from the coder's receipt; one strict QSL-2642 pending test is ignored and is not coverage. Existing accepted help/kit cases were not rerun.
- No tests, builds, product probes or gates run by this reviewer. One final applicable component gate remains for the original coder after disposition.
- Reverse gaps, stubs, vendoring and hollow assertions inspected only in the changed code. No external content copied; production option metadata is referenced through its explicit same-repository seam.
- Exact runtime model identifier unavailable to this invocation; tracker marker records model=unknown.

## Examined units

- FR-002-AC-12 (context_only): The `fsl-cli` built-binary version-agreement check uses `ix_cli_kit::version::Agreement::command` to compare `CARGO_PKG_VERSION` with the output of the built `fsl --version` command and requires agreement.
- FR-002-AC-17 (examined): With no arguments or with each of `--help`, `-h` and `help` as the sole argument, the built `fsl` binary writes root help to stdout containing invocation syntax, recognized lifecycle verb names, `--format text\|json\|dot` with `dot` limited to `inspect`, and `--version`/`-V`. The help distinguishes executable verbs from verbs this build does not route, which return `unsupported_operation` results, and states that `text` is the default output format and `--format json` selects machine output. Successful help exits with FSL's success result, writes no stderr, and starts no lifecycle operation.
- FR-002-AC-24 (examined): For every verb that executes operations and every option `o` in its `fsl-cli` option table entry, `o` appears on its own line in the verb's `<verb> options:` section of root help and the built parser accepts `o` with a well-formed value (no `invalid-request` naming `o`). For every verb that executes operations and every option name `p` that is listed in another verb's entry or is one of `--version`, `-V`, `--color` and `--bogus`, but is not in this verb's entry, `fsl <verb> p` refuses `invalid-request`, exit 20, naming `p`. Removing one option from a copy of the table and running the parser an
- FR-002-AC-25 (examined): `fsl check --help`, `fsl check -h` and `fsl check nosuch.fsl --help`, from an empty working directory, each write exactly the following to stdout, exit 0, write no stderr and read no file: `Usage: fsl check [options] <source-path>` / `       fsl check [options] <authority> <identity> <source-path>` / (blank) / `check options:` / `  --markdown` / `  --model <identity>=<file> (repeatable)` / `  --library <identity>=<file> (repeatable)` / `  --dependent-bytes <bytes>` / `  --limit name=value (repeatable)` / `  --integer-division euclidean\|floor\|truncating` / `  --extensions <file>` / `  --forma
