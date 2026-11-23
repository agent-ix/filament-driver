---
id: TC-010
title: "Apply explicit lifecycle limits and preserve limit outcomes"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-003
    type: verifies
---
# TC-010: Apply explicit lifecycle limits and preserve limit outcomes

## Description

Verify that FSL parses CLI `--limit` operands, applies exposed settings to the
selected operation, omitted values keep their defaults, and a reached limit
remains a typed incomplete outcome. FSL's `parse_operands` owns name
resolution and returns typed `SettingName` values. The driver checks stage settings against the selected
operation's bounds and applies them with `CallerLimits::set_bound`. It applies
accounting settings with `AccountingLimits::set_bound` to `Call.accounting`;
accounting settings do not pass through stage bounds. The driver adds no
namespace or aliases. The driver supports 64-bit hosts; it adds no platform
overflow rule, and FSL owns bound application behavior on 32-bit hosts.

Scope: FR-003-AC-1 through FR-003-AC-16.

## Test Procedure

Prerequisite for FR-003-AC-14 through FR-003-AC-16: blocked until the owner supplies a reproducible
monitor-plan fixture path and its exact measured preparation read charge. The
current repository and issue reference do not identify this evidence; do not
substitute an unmeasured plan.

1. Inspect the parser and application path: the driver calls
   `fsl_foundation::setting::parse_operands` and branches on the returned
   `SettingName`. It checks only stage settings against operation bounds,
   applies those with `CallerLimits::set_bound`, and applies accounting
   settings with `AccountingLimits::set_bound` to `Call.accounting`. Confirm
   the driver has no duplicate grammar, duplicate-detection or name-to-field
   rules.
2. Read the selected operation's FSL `SettingLimits::bounds()`. For every
   returned stage setting, pass its exact FSL name with a distinct valid value
   through `--limit`; observe that only that exposed bound changes at the
   operation boundary. Confirm a recognized stage setting absent from
   `bounds()` is refused with `invalid-request` and an operation-unavailable
   diagnostic, distinct from an unknown-name refusal.
3. Pass the former driver-only name `normalization.work_units`, a malformed
   operand and a duplicate setting. Observe each refusal before any stage with
   code `invalid-request`; unknown-name, malformed and duplicate refusals
   identify their respective causes. When multiple operands are invalid,
   confirm FSL parser refusals are reported before checking whether recognized
   settings are available to the operation. Pass the `accounting.<counter>` setting names
   defined by FSL FR-255 and verify FSL's operand API resolves them to
   `SettingName::Accounting` values.
4. Apply a FSL-named accounting operand such as `accounting.work_units=0`. Inspect that FSL
   `AccountingLimits::set_bound` changes `Call.accounting` while the stage
   limits remain unchanged, then run an input that exhausts that accounting
   budget and verify FSL reports the accounting limit and setting
   `accounting.work_units`. Separately, pass bare `work_units=0` with a
   missing source: FSL refuses the unknown setting before any source read,
   preserving its refusal category rather than treating it as an alias.
   Separately, run an
   input that exceeds a chosen stage limit. Through the driver's
   typed execution wrapper, inspect the returned `LimitExceeded` configured
   bound and `setting()`; then repeat with the limit raised to fit the input.
5. Run the CLI fixtures for `analyze`, `monitor` and `replay` with a matching
   `--limit` override and without it. Verify that the caller's value reaches
   the matching operation bound, the request's other values are preserved,
   and a setting unavailable to that operation refuses before work begins.
6. For `analyze`, `monitor` and `replay`, exercise each operation-specific
   setting listed in FSL FR-255's “Lifecycle caller-limit coverage”; inspect
   that each accepted spelling changes its matching bound and that an
   unavailable recognized spelling is refused before work. For monitor,
   use the exact measured monitor-plan fixture identified by the owner before
   implementation; record its repository path and exact preparation
   read-storage charge M. Verify M is no greater than 268435456. With no
   override, verify that `MonitorPreparationLimits.storage` is 268435456 bytes
   and the fixture completes. Repeat with an explicit
   `--limit monitor.preparation_storage=<n>` where n is less than M; verify the typed refusal names
   `monitor.preparation_storage` and says to raise it with
   `--limit monitor.preparation_storage=<n>`. Also set
   `--limit monitor.preparation_work_units=<n>` and verify only the preparation
   work bound changes; do not infer a new default from this test.

Test location: `src/limits.rs`, `src/execute.rs`, `fsl-cli/src/cli.rs`, and
`fsl-cli/tests/binary_process.rs`.

## Expected Results

- The explicit value replaces only its field; omitted fields retain defaults.
- Every exposed stage setting is accepted under its exact name through FSL's
  operand API and changes only the matching stage bound.
- A FSL-resolved accounting setting changes `Call.accounting` through FSL's
  `AccountingLimits`; it leaves stage limits unchanged and reaches `spine::run`.
- Names not resolved by the pinned FSL parser, malformed operands and
  duplicates are refused before any stage with code `invalid-request`.
- A FSL-recognized stage setting absent from the operation's bounds is refused with
  `invalid-request` and an operation-unavailable diagnostic, not the
  unknown-name diagnostic.
- The driver adds no name parser, driver-defined namespace or aliases. FSL
  owns the `accounting.<counter>` names defined by FR-255 and refuses bare
  counter names as unknown settings.
- FSL owns platform-sized bound application; the driver adds no overflow rule.
- The low limit returns FSL's typed `LimitExceeded`; the driver wrapper
  preserves its configured bound and `setting()` and has category
  `Incomplete`. The process status is selected by FSL's category exit
  function. Raising the source-byte bound to the fixture's byte length allows
  the operation to complete.
- `run`, `check`, `analyze`, `monitor` and `replay` accept the repeatable
  option; each forwards only accepted matching overrides and preserves other
  request values.
- `analyze`, `monitor` and `replay` accept exactly their operation-specific
  caller limits in FSL FR-255; recognized unsupported operation limits are
  refused before work.
- The owner-identified monitor fixture and measured read charge are recorded;
  the fixture's run/refusal results match its measured charge and the fixed
  256 MiB default.
