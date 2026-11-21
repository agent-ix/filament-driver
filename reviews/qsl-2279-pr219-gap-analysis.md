---
id: SR-202611210022442313144108505
title: "QSL-2279 PR219 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; QSL-2279 PR219; Cargo.toml, Cargo.lock, fsl-cli/Cargo.toml, filament-cache/Cargo.toml, src/monitor/lowering/program.rs, src/monitor/projection.rs, src/monitor_refusal.rs, tests/monitor_refusal.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-driver
    type: references
---

## Summary

Manual pin-scoped gap check, as assigned; engine matrix used for the selected criteria, plus changed-code ownership/stub checks. This is not a full repository gap audit. One high-severity consumer adaptation gap blocks compilation; existing owner contracts apply, and no new stub or hollow assertion is introduced. Whole-repository matrix baseline is outside this review, including unrelated untagged criteria. Optional full semantic analysis was not run. Plan completion: not assessed

## Verdict

FAIL — not mergeable. Adapt the new native FiniteHyper item explicitly and faithfully before rerunning the exact-head production CLI build and focused checks. No test PASS or waiver is inferred.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FSL pin introduces ItemForm::FiniteHyper but request_item does not handle it, so the exact candidate fails to compile with E0004 before any focused test or fresh CLI can run. | src/analyze.rs:211 |

## Coverage

Frozen PR: https://github.com/agent-ix/filament-driver/pull/219. Ticket: QSL-2279. Head `45fad94f9a66a78733f6c6124833709b16416e26`; base `a34862bf3cc3c6a04e6468d87f1155bd3fe86f8f`. All eight changed files inspected. No applicable AssuranceProfile found under spec/. No CI workflow, architecture, spec, fixture source, assertion, reader ceiling or oracle change.

Every shared owner in the existing locked/offline metadata resolves to one authorized revision: FSL 529282e9, IR/model 20a4dfbf, CG f327415b, RT 75688ead, SV 6e809434, eval/checked 89573102, observation 1e356f99, Exact b26a8e4c. Manifest and lock source identities agree. Registry package records are byte-equivalent as parsed TOML values. No FSpec package resolves in this graph; FSpec247 is not claimed as a direct driver dependency. No ahead-of-window owner SHA is introduced.

The four Binding patterns remain borrowed name/value inspections. Native IR's binder field belongs to control.let bindings, whereas these three production sites inspect function parameters/body or monitor named members. The added `..` neither drops nor reconstructs the retained admitted package. Existing source-record admission still maps immutable producer structural/source/declaration/opaque-key/member identities into the native join and retains its strict-reader package. The complete-row assertion compares every original producer row/member without filtering.

The monitor test bytes equal the base exactly after removing the single QSL-2279 ignore and accepting the new Binding field. Its real fixture still executes check -> emit_checked -> emit_monitor_plan_over -> strict read with domain evidence; no JSON/package repair supplies success. Nested record/tuple expressions and budgets are unchanged. The target-premise test checks all eight named values, final child zero, original buffer owner, actual generated occurrence/None behavior and unchanged graph/source map. Native FSL emit.rs fixes body-location origin.function during final rekeying; IR body_locations.rs follows record/tuple value references. Their producer/reader implementations remain upstream.

Validation observed here: cargo fmt --check exit 0; git diff --check exit 0; existing locked metadata parsed and checked; deterministic fixture-diff and registry-record comparisons passed; quire matrix exit 0, quire 0.52.0 / engine 0.62.2. All examined criteria are tagged; FR-016-AC-3 now binds the enabled test at tests/monitor_refusal.rs:248. Tag status is not runtime qualification. No Cargo build, test, Clippy, feature sweep, or upstream retest was run by this reviewer under the assigned execution policy. The code lead reports Draco's exact-head production CLI build failed E0004 at src/analyze.rs:211 for missing Some(ItemForm::FiniteHyper(_)); all remaining 11 commands, including the monitor test, were NOT RUN. The remote raw log is draco:/private/tmp/qsl-2279-driver45fa-0.log. This reviewer independently verified the selected native enum and the non-exhaustive consumer match statically; no local build or runtime PASS is asserted.

QSL-2423's unrelated library_parity.rs:297 ignore is retained; it cannot count as a passed parity test. No QSL-2739 composition-text, issue #3287, complete monitor preparation, cumulative budget, clock admission or whole-repository assurance claim is made.


## Compile blocker evidence

Selected FSL fsl-analyze/src/analyze/request.rs declares FiniteHyper(Box<FiniteHyperItem>). The driver's request_item match covers Temporal, Hybrid, Probabilistic and None only. Updating the selected FSL revision makes the ordinary driver/CLI compilation fail for every invocation, before any input is processed. Add the explicit producer-to-CG projection required by the native finite-hyper contract and a real regression, preserving typed bounds/requirements; no wildcard, panic, todo or fabricated unsupported result. This is a consuming adapter fix, not permission to change architecture or pins.

## Examined scope

```yaml
- id: FR-001-AC-1
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: "The driver's normal and build dependency set matches FSL ADR-011 \xA7\
    6.1 and the specified driver-layer exceptions: it contains exactly `fsl-analyze`,\
    \ `fsl-foundation`, `fsl-package`, `fsl-replay`, `fsl-route`, `fsl-semantics`,\
    \ `filament-exact`, `filament-contract-codegen`, `filament-contract-model`, `filament-contract-runtime`,\
    \ `quire-canonical`, `filament-observation`, `filament-semantic-value`, `serde`,\
    \ `serde_json` and `thiserror`; `serde_json` is required for production projection\
    \ of FR-331's retained optional `bounds` member and RT supplies FR-015/FR-017's\
    \ native retained program/arena type"
- id: FR-016-AC-1
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: For an actual strictly admitted node, the locus retains the supplied selector
    and exactly its package-resolved structural key as distinct values, plus the original
    path and optional occurrence; its owned fields remain readable after package/input
    destruction.
- id: FR-016-AC-3
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: On a genuinely admitted monitor root, Every TargetPremise resolves the
    value of its own actual named binding, including the final Binding.value child
    zero, without comparing or changing that value. Uncertainty resolves its actual
    uncertainty value path; BufferCapacity resolves the chosen capacity value path,
    including its final child zero and original row owner.
- id: FR-016-AC-6
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: An actual admitted role/ordinal is retained exactly; a nonmember occurrence
    returns OccurrenceMismatch. A genuine generated root succeeds with caller-supplied
    None even when it retains a Generated occurrence, without selecting a source-map
    row or inventing a selected occurrence.
- id: FR-016-AC-7
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Construction retains typed error distinctions and returns no partial diagnostic
    on failure. Neither constructor changes the admitted package, target values, operation
    records or source correspondence.
- id: FR-018-AC-1
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: Genuine ordinary source records emitted by public FSL and strictly admitted
    by IR retain all complete joined native rows and opaque record keys through driver
    admission/projection, including required, optional and Option fields and recursive/shared
    types where the owners admit them.
- id: FR-018-AC-3
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: The public API accepts genuine immutable Emission authority and binds its
    rows to its own strict reader output; caller-created row structs cannot enter
    production admission. Native reader refusal/incompleteness remain distinct.
- id: FR-018-AC-4
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: Projection uses the retained package and native identities with its independent
    limit, retaining original native type and complete source_record_view without
    schema reconstruction or invented member identities.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: "source = \"git+https://github.com/agent-ix/filament-core-data?rev=424ba49ebca649906dd9d0f5f3cd2f8e7f3e0194#424ba49ebca649906dd9d0f5f3cd2f8e7f3e0194\"\
    \n \"agent-ix-semantic-ir\",\nsource = \"git+https://github.com/agent-ix/filament-core-data?rev=424ba49ebca649906dd9d0f5f3cd2f8e7f3e0194#424ba49ebca649906dd9d0f5f3cd2f8e7f3e0194\"\
    \nsource = \"git+https://github.com/agent-ix/filament-eval?rev=89573102cc309941786e3693b2f71e254dddc91c#89573102cc309941786e3693b2f71e254dddc91c\"\
    \nsource = \"git+https://github.com/agent-ix/filament-contract-codegen?rev=f327415b488d3c91c4b7384eff1e81a53fed8bfd#f327415b488d3c91c4b7384"
- id: Cargo.toml
  path: Cargo.toml
  role: examined
  excerpt: 'filament-semantic-value = { version = "0.1.5", git = "https://github.com/agent-ix/filament-semantic-value",
    rev = "6e809434d5fd7bde4c5b88ec40cd809efcca595d" }

    fsl-analyze = { git = "https://github.com/agent-ix/filament-spec-language", rev
    = "529282e9dc28cc6be8b382ae634636e3b4fe4a14" }

    fsl-foundation = { git = "https://github.com/agent-ix/filament-spec-language",
    rev = "529282e9dc28cc6be8b382ae634636e3b4fe4a14" }

    fsl-package = { git = "https://github.com/agent-ix/filament-spec-language", rev
    = "529282e9dc28cc6be8b382ae634636e3b4fe4a14" }

    fsl-replay = { git = "https://github.com/agent-ix/filamen'
- id: filament-cache/Cargo.toml
  path: filament-cache/Cargo.toml
  role: examined
  excerpt: 'fsl-foundation = { git = "https://github.com/agent-ix/filament-spec-language",
    rev = "529282e9dc28cc6be8b382ae634636e3b4fe4a14" }

    fsl-foundation = { git = "https://github.com/agent-ix/filament-spec-language",
    rev = "529282e9dc28cc6be8b382ae634636e3b4fe4a14" }'
- id: fsl-cli/Cargo.toml
  path: fsl-cli/Cargo.toml
  role: examined
  excerpt: 'filament-semantic-value = { version = "0.1.5", git = "https://github.com/agent-ix/filament-semantic-value",
    rev = "6e809434d5fd7bde4c5b88ec40cd809efcca595d" }

    fsl-analyze = { git = "https://github.com/agent-ix/filament-spec-language", rev
    = "529282e9dc28cc6be8b382ae634636e3b4fe4a14" }

    fsl-command = { git = "https://github.com/agent-ix/filament-spec-language", rev
    = "529282e9dc28cc6be8b382ae634636e3b4fe4a14", features = ["quire-extraction"]
    }

    fsl-inspect = { git = "https://github.com/agent-ix/filament-spec-language", rev
    = "529282e9dc28cc6be8b382ae634636e3b4fe4a14" }

    fsl-foundation = { git = "h'
- id: src/monitor/lowering/program.rs
  path: src/monitor/lowering/program.rs
  role: examined
  excerpt: '        CheckedTerm::Binding { name, value, .. } => Some((name.kind(),
    value)),'
- id: src/monitor/projection.rs
  path: src/monitor/projection.rs
  role: examined
  excerpt: '        if let CheckedTerm::Binding { name, value, .. } = member {'
- id: src/monitor_refusal.rs
  path: src/monitor_refusal.rs
  role: examined
  excerpt: '                ..'
- id: tests/monitor_refusal.rs
  path: tests/monitor_refusal.rs
  role: examined
  excerpt: '            ..'
- id: FR-001
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: 'Between steps 2 and 3, the E5 hand-off: IR''s checked-package reader

    (`read_checked_package`) reads the emitted v2 bytes, and the driver accepts

    the admitted package that E4 emitted.'
- id: FR-022-AC-1
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: '`fsl analyze` over the Counter request with the item `{"claim": "Bounded"}`
    and `--format json` writes one document with `operation` `analyze`, category `success`,
    one item whose record is `proved`, and `aggregate.status` `complete`, and exits
    0.'
- id: src/analyze.rs::request_item
  path: src/analyze.rs
  role: examined
  excerpt: match &item.form {
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | The new CLI test wraps expected_exit twice: json() returns Option<i32> and the loop already supplies Option<i32>, so comparing exit to Some(expected_exit) fails to compile. Compare exit directly to expected_exit. | fsl-cli/tests/frontend_cli.rs:927 |
| FND-003 | low | The new foreign FSL-FR-898 trace strings normalize to unmatched local FR-898-AC-3 and FR-898-AC-4 in Quire coverage. Keep the local FR-022-AC-8 binding and express upstream partial coverage as provenance or use a supported declared foreign binding. | fsl-cli/tests/frontend_cli.rs:844 |
| FND-004 | high | The selected FSL PreparedRun enum includes RefinementTransfer, but the frozen driver CLI match omits it, causing E0004 at fsl-cli/src/cli.rs:1085 and preventing a fresh CLI build before the remaining checks can run. | fsl-cli/src/cli.rs:1085 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The new transfer test provenance docstring emits unmatched FR-1031 and trace-id-outside-marker in fresh Quire coverage; retain the valid local bindings while avoiding an unsupported foreign ID token in the scanned test comment. | fsl-cli/tests/frontend_cli.rs:514 |

## Dispositions

### Round 1

Reviewed frozen `dda27605300a7298ce42c9a6cf2638b790669a9d` against original `45fad94f9a66a78733f6c6124833709b16416e26`. Original finding text remains unchanged. Explicit native FiniteHyper matching fixes FND-001's production source defect. Native FiniteHyperItem contains the checked clause, not CG proof bounds; FSL owns population intake, bound checks and evaluation. CG ItemRequirements has no separate finite-hyper carrier. The common projection still carries kind/extent/finite_bound_available/candidates, with no wildcard, invented unsupported result or proof-bound substitution.

The added CLI test uses the real process and source/population documents. Closed proof/refutation, open inconclusive, replay false tuples and exact bound/found assertions match the native owner behavior statically. Runtime qualification remains pending. The doubled Option at line 927 prevents compiling this test; fresh Quire coverage also identifies two new unmatched trace IDs. The test is partial upstream evidence, not full FR-898 AC-3/AC-4 qualification (three-trace/position-one and quantifier-order controls are upstream).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Explicit FiniteHyper arm covers the selected native enum variant and preserves the existing common CG request projection. |
| FND-002 | still-open | Round 1 introduced a noncompiling Option<Option<i32>> assertion in the new frontend test; compare its existing Option<i32> values directly. |
| FND-003 | still-open | Fresh quire coverage reports both new foreign-looking IDs as unmatched local IDs; use the valid local binding and upstream provenance. |
| FND-004 | still-open | Frozen dda276 CLI match omits PreparedRun::RefinementTransfer; delegate explicitly to the native FSL executor preserving owner results and limits. |

No local Cargo build, tests or Clippy were run. Static git diff --check and quire coverage exited 0. Earlier exact-45fad CLI build failed and its remaining 11 checks were NOT RUN. Draco's exact-dda276 production CLI build now failed E0004 for missing PreparedRun::RefinementTransfer at fsl-cli/src/cli.rs:1085; the production driver library compiled past the earlier FiniteHyper error. All remaining commands were NOT RUN; no fresh CLI or monitor result exists. Raw log reported by the code lead: draco:/private/tmp/qsl-2279-driverdda-0.log. The selected native FSL execute_prepared delegates this variant to transfer::execute; the driver must preserve that owner executor rather than fabricate unsupported output. No runtime PASS is asserted. No pin, monitor fixture, budgets or original assertions changed. The unrelated QSL-2423 ignore remains excluded from parity success claims. No full repository gap/semantic audit was run. Plan completion: not assessed.

Round 1 verdict: FAIL, NOT MERGEABLE while FND-002/FND-003/FND-004 remain open; production compilation and all delegated focused checks must be verified on the corrected head.

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: "Some(ItemForm::FiniteHyper(_)) => {\n            // The checked\
    \ clause's bound limits its trace population, which\n            // FSL evaluates\
    \ with ItemOptions::traces. It is not a substituted\n            // proof-domain\
    \ bound or an additional CG manifest requirement.\n            // Kind, extent\
    \ and candidates still use the common projection.\n        }"
- fnd: FND-002
  outcome: still-open
  reason: New test assertion nests Option; compare exit directly to expected_exit.
- fnd: FND-003
  outcome: still-open
  reason: The new foreign-looking tags are unmatched local IDs in fresh Quire coverage.
- fnd: FND-004
  outcome: still-open
  reason: Frozen dda276 CLI match omits PreparedRun::RefinementTransfer; delegate
    explicitly to the native FSL executor preserving owner results and limits.
```

### Round 2

Frozen `da01aa94cd4f65a8e9661f6af6d514556af4825e` reviewed against `dda27605300a7298ce42c9a6cf2638b790669a9d`. FND-001 remains fixed from round 1; no new disposition row is needed for it. Direct Option comparison fixes FND-002. The explicit RefinementTransfer pattern fixes FND-004 and uses the existing owner executor with unchanged request path, intake limits, overrides, negotiation, reference engines, provider options, render-source retention and native result/error conversion. Native execute_prepared delegates transfer to transfer::execute, whose selected-name resolution produces the asserted typed missing-name refusal. The new real CLI regression first proves native transfer preparation, then asserts exact refusal stage/member/cause/code, exit 20, owner category parity and complete owner/CLI document equality. This demonstrates delegation/refusal only, with no successful refinement proof claim.

FND-003 remains open: removing the attributes is insufficient because the engine also scans the replacement provenance docstring. Fresh Quire coverage reports unmatched FR-898-AC-3 at frontend_cli.rs:927 and trace-id-outside-marker. A separate new transfer provenance token similarly reports unmatched FR-1031 at line 514 (FND-005). Local FR-022-AC-8 binds the finite-hyper test at line 930; FR-002-AC-1/AC-2 bind the transfer test at line 517. Both tests are tracked locally, but the new unmatched tokens remain. The native population proof/refutation/open/bound/replay assertions remain unchanged except the corrected Option comparison. No runtime result exists yet.

Round 2 dispositions (append-only continuation of Dispositions):

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Compares exit directly to the existing Option<i32> expected_exit. |
| FND-003 | still-open | Foreign-looking attribute tags are gone, but provenance still emits unmatched FR-898-AC-3 and trace-id-outside-marker. |
| FND-004 | fixed | Explicit native transfer variant delegates through the existing FSL owner execution branch. |
| FND-005 | still-open | New transfer provenance docstring emits unmatched FR-1031 and trace-id-outside-marker. |

Round 2 verdict: NOT MERGEABLE while FND-003/FND-005 remain open. Both authorized fix rounds are used; the lead must route the remaining low findings through the owner/architecture decision rather than silently waive them or start an automatic third round. All production compile blockers are statically corrected, subject to actual compilation. Exact-da01aa 15-command Draco batch is pending; prior two CLI builds failed and their subsequent checks were NOT RUN. No local Cargo build/test/Clippy or runtime PASS is asserted. git diff --check and fresh quire coverage exited 0. No pins, original monitor source/assertions/budgets or architecture changed; the single QSL-2279 monitor ignore remains removed. No whole-repository assurance PASS. Plan completion: not assessed.

```yaml
dispositions:
- fnd: FND-002
  outcome: fixed
  after_excerpt: 'assert_eq!(exit, expected_exit, "{name}: {document}");'
- fnd: FND-003
  outcome: still-open
  reason: Unsupported foreign attribute strings were removed, but the finite-hyper
    test provenance docstring still emits unmatched FR-898-AC-3 and trace-id-outside-marker
    in fresh Quire coverage.
- fnd: FND-004
  outcome: fixed
  after_excerpt: "PreparedRun::Package(_)\n        | PreparedRun::ModelBasedTest(_)\n\
    \        | PreparedRun::RefinementTransfer(_) => {"
- fnd: FND-005
  outcome: still-open
  reason: New transfer provenance emits unmatched FR-1031 and trace-id-outside-marker;
    valid local FR-002 bindings are present.
```
