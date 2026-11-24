<!-- reviewer repo=agent-ix/filament-driver visibility=public quoin=0.36.10 module=spec-artifacts-process@v0.29.2 id=SR-202611210022442313144108534 method=gap-analysis lang=rust pr=filament-driver#260 model=gpt-6.2-sol run=CE19D777-6005-4659-8088-BBBA20783F7B -->

| Result | Scope |
| --- | --- |
| FAIL | The new host-allocation path is traced to external FSpec AC-17; the repository-wide matrix has 176 untagged criteria. Root-only pin updates also break `fsl-cli` compilation. |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FSpec-FR-330-AC-17, path: "tests/drive.rs", role: examined, excerpt: "When a package read's admitted allocation cannot be provided by the host, the reader returns `HostAllocationFailed` with its stage, locus, requested size and consumed count; the result creates no ContractPackage or `feature_accounting` record, and the tool reports a host error. The failure is not `body_incomplete` and does not change that record's closed shape."}
  - {id: FR-001-AC-1, path: "spec/functional/FR-001-orchestrate-the-driver-steps.md", role: examined, excerpt: "The driver's normal and build dependency set matches FSL ADR-011 §6.1 and the specified driver-layer exceptions: it contains exactly `fsl-analyze`, `fsl-foundation`, `fsl-package`, `fsl-replay`, `fsl-route`, `fsl-semantics`, `filament-exact`, `filament-contract-codegen`, `filament-contract-model`, `filament-contract-runtime`, `quire-canonical`, `filament-observation`, `filament-semantic-value`, `serde`, `serde_json` and `thiserror`; the driver's direct edge, CG's IR crate and FSL's `fsl-package` all resolve the same package id."}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-001-AC-1
    related_ids: []
    path: Cargo.toml
    lines: "59-66"
    excerpt: "The driver's normal and build dependency set matches FSL ADR-011 §6.1 and the specified driver-layer exceptions"
    finding: "Root and sibling workspace dependency revisions diverge, and a focused `fsl-cli` compile fails with 99 incompatible duplicate-type errors."
coverage:
  matrix: "quire matrix --scope . --format json; no evidence store"
  counts: {tagged: 151, untagged: 176, tagged_by_ignored_test: 5, method_without_symbol: 10}
  changed_behavior: "The injected `e5_host_allocation_failure_is_not_a_package_record` test is tagged FSpec-FR-330-AC-17 and verifies typed failure retention, Handoff stage, Fault kind, runtime_invariant code, message details, and no later upstream calls."
  reverse_gap: "No new unowned behavior found in changed production paths beyond the external FSpec criterion."
  stub_scan: "No TODO, FIXME, todo!, or unimplemented! in changed production/test paths."
  semantic_review: "Skipped; optional review was not requested."
  plan_completion: "Not assessed; no plan supplied."
```

+++

