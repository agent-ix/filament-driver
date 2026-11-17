---
id: FR-014
title: "Place the user CLI in a separate workspace package"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-001
    type: references
  - target: ix://agent-ix/filament-driver/FR-002
    type: references
  - target: ix://agent-ix/filament-driver/FR-012
    type: depends_on
  - target: ix://agent-ix/filament-driver/FR-013
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-287
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-561
    type: depends_on
---
# FR-014: Place the user CLI in a separate workspace package

## Description

The driver repository SHALL provide its user-facing `fsl` executable from a
workspace package named `fsl-cli`, separate from the `filament-driver` library
package. The `fsl-cli` package SHALL own the binary argument parser, CLI
tests and their fixtures. It SHALL compose the driver library, downstream
cache, and process-provider adapter using `filament-plugin-host`.

## Inputs

- The driver repository Cargo workspace and its package manifests.
- The `filament-driver` and `filament-cache` packages.
- The `filament-plugin-host` package implementing FR-013.

## Outputs

- A workspace package named `fsl-cli` that provides the `fsl` executable.
- The binary argument parser and CLI tests/fixtures owned by `fsl-cli`.
- A Cargo dependency graph in which the CLI composes the driver, cache and
  ix-cli-kit.
- A Cargo dependency graph in which `fsl-cli` composes the plugin host while
  the `filament-driver` dependency closure excludes both the host and ix-cli-kit.

## Behavior

- The driver repository's Cargo workspace SHALL include a separate
  `fsl-cli` package.
- Exactly one workspace package, `fsl-cli`, SHALL provide the `fsl` binary
  target.
- `fsl-cli` SHALL have direct normal dependencies on `filament-driver`,
  `filament-cache`, `filament-plugin-host` and `ix-cli-kit`.
- The `ix-cli-kit` Git dependency in `fsl-cli/Cargo.toml` SHALL specify
  `branch = "main"`; the consumer's `Cargo.lock` SHALL record the resolved
  commit, with no `rev` field in the dependency declaration.
- The workspace root SHALL list `fsl-cli` in `default-members`.
- The `fsl-cli` package SHALL own the argument parser in
  `fsl-cli/src/cli.rs`, with the binary entry point in
  `fsl-cli/src/main.rs`.
- The `fsl-cli` composition in `fsl-cli/src/cli.rs` SHALL own the
  process-provider configuration and provider-limit inputs passed into FR-006
  composition. The `run` command, like every provider-routing verb (FSL
  FR-287), SHALL accept `--provider-config <path>` as the sole source of the
  process plugins it starts, a plugins-only FSpec FR-561 document with the
  shape and refusals FR-006 states, and `--provider-limit <name>=<value>` as
  the sole source of the run's provider limits, including `deadline_ms`.
  When `--provider-config` is omitted, the command SHALL register no process
  providers; when a limit is not given, FSL FR-287's default applies. The
  command SHALL NOT search a default location or read either input from
  ambient environment state or another file.
- CLI contract and binary process-boundary tests and their fixtures SHALL
  reside in the `fsl-cli` package.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-014-AC-1 | Exactly one workspace package, `fsl-cli`, provides the `fsl` binary target. | Test (TC-028) |
| FR-014-AC-2 | `fsl-cli` has direct normal dependency edges to `filament-driver`, `filament-cache`, `filament-plugin-host` and `ix-cli-kit`; the `filament-driver` closure has no normal or build edge to `filament-plugin-host` or `ix-cli-kit`. | Test (TC-005, TC-028) |
| FR-014-AC-3 | The workspace root lists `fsl-cli` in `default-members`. | Test (TC-005) |
| FR-014-AC-4 | CLI contract and binary process-boundary tests and their fixtures reside in the `fsl-cli` package. | Test (TC-009, TC-018, TC-028) |
| FR-014-AC-5 | The CLI parser definition resides in `fsl-cli/src/cli.rs`, and the binary entry point is in `fsl-cli/src/main.rs`; the `filament-driver` package's `src/lib.rs` does not export a `cli` module. | Test (TC-028) |
| FR-014-AC-6 | The `fsl run` command accepts `--provider-config <path>` as its sole process-plugin source, a plugins-only document whose entries carry no deadline or limit, and `--provider-limit <name>=<value>` as the sole source of the run's limits, including `deadline_ms`; omitting `--provider-config` registers no process providers, omitting a limit applies FSL FR-287's default (`deadline_ms` 600000), and no default, ambient or file configuration is read. | Test (TC-028, TC-041) |

## Dependencies

- FSL ADR-029 CB-3 package-boundary guidance.
- FR-001-AC-11 and FR-012-AC-13 require the driver closure to exclude
  `filament-cache`; TC-005 verifies those criteria.
- [FR-001](./FR-001-orchestrate-the-driver-steps.md) keeps the driver core
  independent of downstream cache implementation.
- [FR-002](./FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  binary's command and outcome contract.
- [ix-cli-kit FR-017](ix://agent-ix/ix-cli-kit/FR-017) provides the caller-owned
  exit-code pass-through consumed by the CLI; the kit does not own argv parsing.
- [FR-012](./FR-012-implement-the-downstream-content-addressed-cache.md)
  defines the separate cache package.
- [TC-005](../test-cases/TC-005-dependency-edges.md) and
  [TC-028](../test-cases/TC-028-cli-package-boundary.md) verify the graph.
