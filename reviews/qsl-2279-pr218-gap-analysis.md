---
id: SR-17940486709564847047975
title: "QSL-2279 PR218 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; QSL-2279 PR218; Cargo manifests/lock, execute, monitor material/types/source_record, changed tests"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver
    type: references
---

## Summary

Independent static gap-analysis of the final consuming pin change. Two low-severity trace-binding defects; no scoped production defect found. Runtime checks are pending.

## Verdict

CONDITIONAL — fix the two new unmatched test tags and obtain the delegated focused results before merge. No whole-repository assurance PASS or runtime-green conclusion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new temporal_binding_declaring_source_is_charged_before_inspection_completes test traces only FR-019, which Quire reports as an unmatched tag; use an existing criterion ID for the asserted component behavior and retain the explicit limitation on full monitor qualification. | src/monitor/budget/tests.rs:22 |
| FND-002 | low | The new clock_record_member_refuses_without_a_substitute_runtime_type test traces only FR-019, which Quire reports as an unmatched tag; use an existing criterion ID for the asserted component behavior and retain the explicit limitation on full monitor qualification. | src/monitor/types/tests.rs:11 |

## Coverage

Reviewed frozen 279e3e82f0f887f46a2175c3e524ded11dec45b6 against fccbe3d03f7f2fc11ad52bcd1555447948fb86d1. All 13 changed files inspected. No spec, CI lane, IR-1568 table or future reader pin changes. No applicable AssuranceProfile found.

The single optional-law spine call removes the empty catalog and duplicated call. All manifest/lock FSL, CG and IR/model surfaces consistently adopt the assigned revisions; lock diff adds only FSL replay num-traits beyond source changes. FR-002 AC27 requirements routing and AC28 prove behavior remain separate and unchanged. Existing two-law parity checks independently expect floor -3 and truncating -2. The enabled dependent-byte contract checks primary-source exclusion, exact dependency-byte success and one-under refusal.

The FSL record adapter preserves native data fields as Some and supplies no clock metadata. Native IR documents value_type None exactly for clock record/tuple entries; union clock_payload is checked before runtime type construction. The unsupported payload uses the original admitted package/node and existing locus helper. Its regression compiles/admit a real value-only source, then supplies counterfactual clock-shaped input to the private conversion seam: defensive refusal evidence only, no genuine clock witness admission, no QSL-2347 completion and no full monitor preparation credit. The temporal material test measures 20 additional work units (record, two strings, 17 UTF-8 bytes), tests exact success and one-short denial with the paid prefix retained; no fresh complete-budget claim.

Runtime: no Cargo, build, lint, test or all-target gate run by this reviewer. Delegated 12-command batch pending; no runtime-green claim. Static git diff --check passed. Whole-repository baseline assurance is not asserted.

Reconciliation: quoin matrix 0.36.8 (quire 0.52.0, engine 0.62.2); exits 0. {'tagged': 121, 'untagged': 181, 'tagged-by-ignored-test': 5, 'method-without-symbol': 10}. All 317 criteria have no run evidence. quire coverage exits 0; Rust census candidates 770, tagged 739, bound 241. It reports cfg-alternative symbol collisions for existing signals::run and frontend_cli::invalid_os_string; baseline diagnostics include 1151 trace-id-outside-marker, six uncatalogued-verification-method, two absent optional archetypes and one catch-all-universal. These whole-repository baseline results are outside this PR finding set and prevent a repository assurance PASS. New unmatched tags are the two findings. No grep fallback used.

Reverse-gap inventory: execute plus monitor admission, type conversion and borrowed material accounting map to existing FR-001/FR-018/FR-019/FR-017 behaviors; CLI dependent-byte semantics map to FR-002. All changed production behaviors have an owner; zero newly introduced source/test stubs or replacement production doubles found. Wider public APIs and source TODO/unimplemented scans were inspected; no exhaustive repository reverse-gap assurance is claimed. Optional repository-wide semantic fan-out was not run; targeted test-oracle review was authorized by dispatch.

Plan completion: not assessed

## Examined scope

```yaml
[
  {
    "id": "FR-018-AC-1",
    "path": "spec/functional/FR-018-admit-source-record-witnesses.md",
    "role": "examined",
    "excerpt": "Genuine ordinary source records emitted by public FSL and strictly admitted by IR retain all complete joined native rows and opaque record keys through driver admission/projection, including required, optional and Option fields and recursive/shared types where the owners admit them."
  },
  {
    "id": "FR-018-AC-3",
    "path": "spec/functional/FR-018-admit-source-record-witnesses.md",
    "role": "examined",
    "excerpt": "The public API accepts genuine immutable Emission authority and binds its rows to its own strict reader output; caller-created row structs cannot enter production admission. Native reader refusal/incompleteness remain distinct."
  },
  {
    "id": "FR-018-AC-4",
    "path": "spec/functional/FR-018-admit-source-record-witnesses.md",
    "role": "examined",
    "excerpt": "Projection uses the retained package and native identities with its independent limit, retaining original native type and complete source_record_view without schema reconstruction or invented member identities."
  },
  {
    "id": "FR-017-AC-4",
    "path": "spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md",
    "role": "examined",
    "excerpt": "Native IR refusal, unsupported source operation, operating stop, cancellation, runtime ProgramRefusal and driver invariant failure remain distinct, with original source node/path and native payload. No failure becomes a temporal outcome or a partial successful program."
  },
  {
    "id": "FR-002-AC-27",
    "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
    "role": "examined",
    "excerpt": "For FSL FR-960-AC-1's native-run request `Q` whose call carries `requirements`, `fsl run Q --format json` writes FR-002-AC-22's document (value `{\"integer\": \"16\"}`, three settled requirement entries), exit 0. `fsl run Q --limit s1.nodes=50000 --format json` writes the same document. FSL's provider run entry carries stage limits only, so `fsl run Q --limit accounting.work_units=1000000` and `fsl run Q --limit observation.input_bytes=1000000` each refuse `invalid-request`, exit 20, naming the setting and `requirements`, and run nothing. `fsl run Q --limit s1.nodes=1 --format json` writes FSL's t"
  },
  {
    "id": "FR-002-AC-28",
    "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
    "role": "examined",
    "excerpt": "`fsl prove R --integer-division floor --format json` and `fsl prove R --bogus --format json` refuse `invalid-request`, exit 20, as an `invalid-request` outcome document with operation `prove` naming the option, before the verb answers unsupported; `fsl prove R --format json`, `fsl prove --help` and `fsl prove --version` answer unsupported, exit 21 (AC-25 governs help and version on a verb the build does not route)."
  },
  {
    "id": "FR-019-AC-2",
    "path": "spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md",
    "role": "context_only",
    "excerpt": "Let `C(p, r)` be the cumulative consumption of resource `r` before phase `p` plus `p`'s first charge of `r`. For every phase and every resource it charges, a budget with `r` limited to `C(p, r) - 1` and the other resource at its measured total refuses in that phase with code `work` or `storage` and the budget's first unpaid kind, amount and phase locus, no program and earlier consumption retained; the measured totals succeed exactly. Status: D2 is deferred; the code keeps the private ledger and `MonitorPreparationLimits` for selection, lowering and the recursion bound, and does not yet meet th"
  },
  {
    "id": "src/monitor/budget/tests.rs::temporal_binding_declaring_source_is_charged_before_inspection_completes",
    "path": "src/monitor/budget/tests.rs",
    "role": "examined",
    "excerpt": "#[ix_trace_rs::trace(\"FR-019\")]"
  },
  {
    "id": "src/monitor/types/tests.rs::clock_record_member_refuses_without_a_substitute_runtime_type",
    "path": "src/monitor/types/tests.rs",
    "role": "examined",
    "excerpt": "#[ix_trace_rs::trace(\"FR-019\")]"
  }
]
```
