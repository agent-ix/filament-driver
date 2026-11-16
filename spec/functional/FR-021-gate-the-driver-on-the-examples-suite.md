---
id: FR-021
title: "Gate the driver on the examples suite"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-driver/FR-014
    type: references
---
# FR-021: Gate the driver on the examples suite

## Description

The filament-examples suite records, for every example case, the exit code
and outcome structure the `fsl` binary gives. The driver's local gate SHALL
build `fsl` from this workspace, at the FSL language revision the workspace's
`Cargo.lock` resolves, and run the suite's Rust runner with that binary over
every running case and every staged case. A language or driver change that
alters any recorded outcome then fails the driver's gate instead of surfacing
first in the examples repository.

## Inputs

- The driver workspace and its `fsl-cli` package ([FR-014](FR-014-separate-the-user-cli-package.md)),
  which provides the `fsl` binary and its outcome contract ([FR-002](FR-002-expose-driver-cli-and-outcome-contract.md)).
- A checkout of filament-examples at the head of its `main` branch, fetched
  over https by `make examples-fetch` into `target/filament-examples` and
  named to the gate by `FSL_EXAMPLES_DIR`.
- The suite's runner crate `tools/fsl-examples` and its `check` and `staged`
  commands.

## Outputs

- A pass, or a failure that names each case whose outcome changed.

## Behavior

- `make examples-check` SHALL fetch the suite and run the
  `tests/examples_gate.rs` integration test; `make ci` SHALL run
  `examples-check`. A plain `cargo test` does not run the gate, because it needs
  the fetched suite.
- The gate SHALL build `fsl` with `cargo build --locked -p fsl-cli` in this
  workspace and run the runner with `FSL` set to that binary. It SHALL NOT use
  an installed or released `fsl`, and it needs no Node.js.
- When `FSL_LANGUAGE_DIR` names a filament-spec-language checkout,
  the gate SHALL instead build `fsl` with every package the driver graph
  resolves from the filament-spec-language git source patched, through cargo `--config`
  `patch` entries, to the same-named crate of that checkout. The patched
  build runs in a copy of this workspace under the test target's temporary
  folder, with this workspace's target directory, so the patch rewrites only
  the copy's `Cargo.lock`; the gate SHALL never write the checkout's
  `Cargo.lock`. Once the build's package sources are checked,
  the gate SHALL print `fsl built from language checkout <path>` with the checkout's
  absolute path. The language repository's
  `make examples-check` runs the gate this way, so a language change is gated
  on the suite before it reaches the driver's lock. A package the driver uses
  that the checkout lacks fails the gate.
- Every running case SHALL give its recorded exit and outcome structure.
- Every staged case SHALL still fail as recorded. A staged case that gives its
  ready exit SHALL fail the gate with the runner's promote message.
- An absent or incomplete checkout SHALL fail the gate, never pass as a skip
  or as a suite with zero cases.
- The gate SHALL follow the head of the suite's `main` branch, with no
  pinned commit.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-021-AC-1 | With the workspace-built `fsl`, the runner's `check` over the fetched suite exits 0 and reports a non-zero number of running cases passed and 0 failed. | Test (TC-035) |
| FR-021-AC-2 | With the same binary, the runner's `staged` over the fetched suite exits 0 and reports a non-zero number of staged cases passed and 0 failed. | Test (TC-035) |
| FR-021-AC-3 | In a copy of one system whose running case has one edited recorded outcome field, `check` exits 1 and its failure line names the case as `<system>/<case>`. | Test (TC-035) |
| FR-021-AC-4 | In a copy of one system whose staged case now gives its ready exit, `staged` exits 1 and prints `promote <system>/<case>` for that case. | Test (TC-035) |
| FR-021-AC-5 | An unset `FSL_EXAMPLES_DIR`, or one naming a folder without `examples/` and `tools/fsl-examples`, is an error naming the cause; the runner given a root with no `examples/` exits 2 rather than passing zero cases. | Test (TC-035) |
| FR-021-AC-6 | The binary the gate runs prints, for `fsl --version`, the version of this workspace's `fsl-cli` package and lies in this workspace's target directory; `make ci` runs `examples-check`. | Test (TC-035), Inspection |
| FR-021-AC-7 | Without `FSL_LANGUAGE_DIR`, the `fsl` build reports language packages from the filament-spec-language git source and no path package outside this workspace; with it, the build reports no package from that git source and at least one from the named checkout, the workspace copy's `Cargo.lock` differs from the checkout's, the gate prints `fsl built from language checkout <path>`, and in both modes the checkout's `Cargo.lock` is byte for byte what it was before the build. | Test (TC-035) |

## Dependencies

- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  outcome documents and exit codes the suite records.
- [FR-014](FR-014-separate-the-user-cli-package.md) places the `fsl` binary in
  the `fsl-cli` package that the gate builds.
- The filament-examples repository owns the cases, their records and the
  runner; this requirement owns only how the driver runs them.
