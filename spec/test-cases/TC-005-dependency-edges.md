---
id: TC-005
title: "The dependency edges match ADR-011 §6.1 and §7.1"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-001
    type: verifies
  - target: ix://agent-ix/filament-driver/FR-012
    type: verifies
  - target: ix://agent-ix/filament-driver/FR-013
    type: verifies
  - target: ix://agent-ix/filament-driver/FR-014
    type: verifies
---
# TC-005: The dependency edges match ADR-011 §6.1 and §7.1

## Description

Verify the driver's place in the crate graph from `cargo metadata`.

Scope: FR-001-AC-1, FR-001-AC-11, FR-012-AC-13, FR-013-AC-11,
FR-014-AC-2 and FR-014-AC-3.

## Test Procedure

1. Run `cargo metadata --locked --offline` and read the resolved graph and
   `workspace_default_members`. Count normal and build edges. Identify FSL
   packages by their source,
   `git+https://github.com/agent-ix/filament-spec-language`. Compare the driver's
   complete direct normal/build inventory and its separate direct development
   inventory with exact-set equality. Retain exactly six normal FSL layer
   edges; `fsl-analyze` is a normal edge ([FR-019](../functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) Dependencies).
2. Traverse the complete normal/build dependency closure from `filament-driver`.
   Identify `filament-cache` and inspect its direct dependency edges. Existing
   `tests/dependency_edges.rs` coverage verifies the driver/cache boundary and
   host-free driver closure.
3. Inspect the direct normal dependency edges from `fsl-cli` and confirm it
   depends on `filament-plugin-host` and `ix-cli-kit`; confirm the driver closure
   remains host-free and kit-free. The CLI-to-host edge is already asserted by
   `tests/dependency_edges.rs` and `fsl-cli/tests/package_boundary.rs`.
4. Confirm the authorised direct runtime edge and traverse RT's complete
   normal/build closure. It must contain no FSL-source package, IR/model,
   driver or codegen backedge. This dependency direction check alone is not
   evidence that monitor preparation or execution has succeeded.

Test: `tests/dependency_edges.rs`, `tc_005_driver_edges_match_the_adr_011_allow_list`.

## Expected Results

- The driver's normal and build dependencies are exactly `fsl-analyze`,
  `fsl-foundation`, `fsl-package`, `fsl-replay`, `fsl-route`, `fsl-semantics`,
  `filament-exact`, `filament-contract-codegen`, `filament-contract-model`, `filament-contract-runtime`,
  `quire-canonical`, `filament-observation`, `filament-semantic-value`, `serde`, `serde_json` and `thiserror`. `serde_json` is a
  normal edge for production projection of FR-331's retained optional
  `bounds` member. The direct `filament-semantic-value` normal edge supplies
  existing shared catalog owner types after their ownership moved out of
  FSL; this is not a re-export shim or an additional driver API. The
  dependency assertion remains exact-set equality. The FSL root crate and `filament-cache` are not direct edges.
  The RT edge supplies the native program/arena types authorised by FR-015
  and FR-017, with no FSL/IR/model/driver/codegen backedge in RT's complete
  normal/build closure.
  `filament-plugin-host` is not in the driver's normal or build dependency
  closure. No package in the driver's transitive normal/build dependency
  closure is `filament-cache`.
- The driver's direct development dependencies are exactly `ix-trace-rs`,
  and `filament-checked`. `fsl-analyze` is not a development edge: FR-019 takes
  it as a normal dependency for `PlanRoot` and `AdmittedPlanAuthority`.
  `filament-observation` is a normal edge because FR-019's request field
  `progress` has type `filament_observation::authority::progress::View`, issued
  under filament-observation FR-004 (AC-7). `filament-checked` is test-only and supplies the existing
  `DeclarationKey` owner type relocated from FSL; it adds no normal/build
  edge or driver API. `serde_json` remains a normal dependency used by tests.
- The driver's `filament-contract-model` is the package CG's
  `filament-contract-ir` depends on.
- If the workspace has a `filament-cache` package, it is separate from
  `filament-driver` and depends on the driver-owned interface. No transitive
  normal/build path leads from `filament-driver` to `filament-cache`.
- The `workspace_default_members` array includes `fsl-cli`.
- When the workspace contains `fsl-cli`, the graph includes its driver
  interface and cache edges, plus direct normal edges to `filament-plugin-host`
  and `ix-cli-kit`. The direct CLI-to-host edge is already asserted by the
  current dependency test together with the CLI-to-kit edge. `ix-cli-kit`
  remains outside the driver's closure. The driver's closure
  remains cache-free and host-free.
- CG's one normal or build edge into FSL is `fsl-replay`.
- The graph holds one copy of `filament-contract-model`: the driver's own
  direct dependency, CG's `filament-contract-ir` and FSL's `fsl-package` all
  resolve the same package id (by identity, not just by revision string).
