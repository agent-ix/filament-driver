<!-- reviewer repo=agent-ix/filament-driver visibility=public quoin=0.36.10 module=spec-artifacts-process@v0.29.2 id=SR-202611210022442313144108533 method=code-review lang=rust pr=filament-driver#260 model=gpt-6.2-sol run=CE19D777-6005-4659-8088-BBBA20783F7B -->

| FND | Severity | Finding |
| --- | --- | --- |
| FND-001 | high | Root dependency pins diverge from workspace consumer pins, causing `fsl-cli` compilation to fail. |

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
    method: code-review
    check: code-bug
    artifact_id: FR-001-AC-1
    related_ids: []
    path: Cargo.toml
    lines: "59-66"
    excerpt: "fsl-foundation = { git = \"https://github.com/agent-ix/filament-spec-language\", rev = \"7527601581610f825d5a3df0e138477f490cf08b\" }"
    finding: "Updating only root dependency pins leaves `fsl-cli` and `filament-cache` on incompatible FSL/codegen/IR revisions; `cargo check --locked -j 8 -p fsl-cli --tests` fails with 99 errors, including duplicate incompatible `fsl_analyze` and `fsl_foundation` types. Align the workspace manifests and lockfile."
```

+++

