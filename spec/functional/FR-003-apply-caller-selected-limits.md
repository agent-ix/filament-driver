---
id: FR-003
title: "Apply caller-selected lifecycle limits"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-277
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-255
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-275
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-300
    type: depends_on
  - target: ix://agent-ix/filament-driver/FR-002
    type: depends_on
---
# FR-003: Apply caller-selected lifecycle limits

## Description

When a caller supplies one or more `--limit` options, the driver SHALL use
FSL's operand parser and typed caller limits to apply those values to the
selected lifecycle operation. When a caller omits a limit setting, the
operation SHALL use that setting's published FSL default. Every
FSL-enforced bound SHALL be caller-configurable through the canonical FSL
setting interface. When a caller invokes `run`, `check`, `analyze`, `monitor`
or `replay` with repeatable `--limit name=value` options, the driver SHALL
override each matching exposed bound for that operation; request-supplied
bounds and omitted settings retain their operation-defined values. The driver
supports 64-bit hosts. On a 32-bit host, FSL owns its documented bound
application behavior, including any clamp; the driver adds no platform
overflow rule. When an operation reaches a bound, it SHALL return FSL's typed
limit outcome and report no success.

## Inputs

- Zero or more `--limit` options containing a FSL setting operand and value.
- The lifecycle operation's published default limits.

## Outputs

- The operation's result under the effective limits, or its typed
  `LimitExceeded` outcome when a bound is reached.

## Behavior

- When the CLI receives `--limit` operands, the driver SHALL call
  `fsl_foundation::setting::parse_operands` to resolve them as FSL
  `SettingName` values.
- The option table SHALL list `--limit name=value` as repeatable for `run`,
  `check`, `analyze`, `monitor` and `replay`. The driver SHALL apply these
  overrides to each verb's corresponding FSL operation before it starts the
  operation; an operation that does not expose a resolved bound SHALL refuse
  that setting before starting any stage.
- When FSL resolves a stage setting, the driver SHALL check it against the
  selected operation's `SettingLimits::bounds()`.
- When a resolved stage setting is exposed by those bounds, the driver SHALL
  apply it with FSL's `CallerLimits::set_bound`.
- When FSL resolves an accounting setting, the driver SHALL apply it with
  FSL's `AccountingLimits::set_bound` to the selected `Call.accounting` without
  checking stage bounds, except on a `fsl run` whose call carries
  `requirements`, which refuses accounting settings (FR-002-AC-27).
- The driver SHALL NOT reimplement FSL's setting-name grammar, duplicate
  detection, value parsing or overflow handling, or add aliases.
- The driver SHALL use FSL FR-255's accounting setting names
  `accounting.<counter>` (for example, `accounting.work_units`) through
  FSL's `parse_operands` name resolution. A bare counter such as
  `work_units` SHALL retain FSL's unknown-setting refusal.
- When FSL resolves an accounting setting, the driver SHALL apply it with
  `AccountingLimits::set_bound` to `Call.accounting` without checking stage
  bounds or adding a driver-defined namespace or aliases, except on a
  `fsl run` whose call carries `requirements`, which refuses accounting
  settings (FR-002-AC-27).
- If a resolved stage setting is absent from the selected operation's bounds,
  then the CLI SHALL refuse it before any stage with code `invalid-request`
  and a diagnostic that distinguishes an operation-unavailable setting from
  an unknown name.
- The driver SHALL pass the computed limits values to the selected operation.
- The driver SHALL preserve each explicit caller value when building the
  computed limits values.
- For `analyze`, `monitor` and `replay`, the operation's caller-configurable
  bound set SHALL be the operation-specific set in FSL FR-255's “Lifecycle
  caller-limit coverage”. The driver SHALL pass each accepted override to
  that matching FSL bound and SHALL refuse a recognized setting absent from
  the operation's set before doing work. In particular, `monitor` SHALL
  expose `monitor.preparation_work_units` and
  `monitor.preparation_storage` for plan preparation; the latter configures
  `MonitorPreparationLimits.storage`.
- A caller MAY set `monitor.preparation_work_units` with
  `--limit monitor.preparation_work_units=<n>`. This requirement assigns the
  setting name to the preparation work bound but does not change the
  operation's existing default. A reached work bound SHALL name
  `monitor.preparation_work_units` in its refusal.
- For real inputs, `fsl monitor` SHALL use a default read-storage allowance
  of 268435456 bytes (256 MiB) for `monitor.preparation_storage`. It provides
  approximately 2.1 times headroom over QSL-2912's reported upper charge of
  126 MB and gives a predictable allowance independent of package size; it is
  not calculated from the input package size. A caller MAY replace it with the
  repeatable CLI operand `--limit monitor.preparation_storage=<n>`; the
  explicit value is the effective allowance. When this bound is reached, the
  refusal SHALL name `monitor.preparation_storage` and state that the caller
  can raise it with `--limit monitor.preparation_storage=<n>`. This storage
  bound is separate from `monitor.preparation_work_units`.
- A reached limit SHALL retain FSL's typed cause, limit kind, configured
  value and field identity in the machine outcome.
- A limit outcome SHALL retain FSL's incomplete category and use FSL's
  category exit function.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-003-AC-1 | Setting one FSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values. | Test (TC-010) |
| FR-003-AC-2 | A setting name not resolved by FSL's operand parser is refused before any stage with code `invalid-request` and names the offending operand. | Test (TC-010) |
| FR-003-AC-3 | An input that reaches a caller-lowered bound returns typed `LimitExceeded` with the configured value and field name and FSL's incomplete category; the process status equals FSL's category exit function. Raising the same bound to fit the input allows the operation to complete. | Test (TC-010) |
| FR-003-AC-4 | The driver uses FSL `parse_operands`, checks each resolved stage setting against the selected operation's `SettingLimits::bounds()`, and applies an exposed stage setting with `CallerLimits::set_bound`; accounting settings bypass stage bounds and use `AccountingLimits::set_bound` on `Call.accounting`, and the driver maintains no duplicate grammar, alias table, or name-to-field mapping. | Inspection (TC-010) |
| FR-003-AC-5 | A stage setting parsed by FSL but absent from the selected operation's `SettingLimits::bounds()` is refused before any stage with code `invalid-request` and a distinct diagnostic naming the setting as unavailable for that operation. | Test (TC-010) |
| FR-003-AC-6 | Applying an accepted setting through FSL `CallerLimits::set_bound` changes only the matching bound; every other operation limit retains its prior value. | Test and Inspection (TC-010) |
| FR-003-AC-7 | A malformed `--limit` operand is refused before any stage with code `invalid-request` and identifies that operand. | Test (TC-010) |
| FR-003-AC-8 | A duplicate `--limit` setting is refused before any stage with code `invalid-request` and identifies the duplicate setting. | Test (TC-010) |
| FR-003-AC-9 | The driver accepts FSL FR-255's `accounting.<counter>` setting names through FSL's operand APIs and retains FSL's unknown-setting refusal for a bare counter such as `work_units`, without adding a driver-defined namespace or aliases. | Test (TC-010) |
| FR-003-AC-10 | When FSL resolves an `accounting.<counter>` setting name, the driver applies its value through FSL `AccountingLimits::set_bound` to `Call.accounting`; the stage `SpineLimits` remain unchanged and the accounting value reaches `spine::run`, except on a `fsl run` whose call carries `requirements`, which refuses accounting settings (FR-002-AC-27). | Test (TC-010) |
| FR-003-AC-11 | The `run`, `check`, `analyze`, `monitor` and `replay` option-table entries accept repeatable `--limit name=value`; each operation receives accepted overrides to its exposed limits, retains request-supplied bounds and defaults for omitted settings, and refuses an operation-unavailable setting before work begins. | Test |
| FR-003-AC-12 | `analyze`, `monitor` and `replay` expose the exact caller-limit sets assigned by FSL FR-255's “Lifecycle caller-limit coverage”; each accepted CLI operand reaches only its matching bound, and each recognized but operation-unavailable setting is refused before work. | Test (TC-010) |
| FR-003-AC-13 | With no caller override, `fsl monitor` passes 268435456 bytes as `MonitorPreparationLimits.storage` for `monitor.preparation_storage`. | Test (TC-010) |
| FR-003-AC-14 | The recorded owner-identified monitor-plan fixture has a measured preparation read charge M no greater than 268435456 bytes. | Test (TC-010) |
| FR-003-AC-15 | The owner-identified monitor-plan fixture completes when run with the 268435456-byte default read-storage allowance. | Test (TC-010) |
| FR-003-AC-16 | When the owner-identified fixture is run with an explicit `monitor.preparation_storage` value n less than its measured read charge M, `fsl monitor` returns a typed refusal naming `monitor.preparation_storage` and how to raise it. | Test (TC-010) |

## Dependencies

- FSL ADR-029 LC-4, FR-255 and FR-277; FSpec FR-300's caller-supplied
  limits.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point and exit function.
- [TC-010](../test-cases/TC-010-caller-selected-limits.md).
