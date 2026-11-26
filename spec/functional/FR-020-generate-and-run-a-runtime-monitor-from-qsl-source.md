---
id: FR-020
title: "Generate and run a runtime monitor from QSL source"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-002
    type: implements
  - target: ix://agent-ix/quire-driver/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-017
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-019
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-280
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-283
    type: references
  - target: ix://agent-ix/quire-contract-runtime/FR-276
    type: depends_on
  - target: ix://agent-ix/quire-contract-runtime/interface_012
    type: depends_on
---
# FR-020: Generate and run a runtime monitor from QSL source

## Description

When a caller invokes `quire generate` with a request that selects exactly one
QSL runtime-monitor plan and names an event stream, the driver SHALL build and
run the monitor by passing the source through the QSL generate path, admitting
and lowering its monitor plan through Contract IR and the driver monitor
preparation path, and constructing the monitor through RT.

The caller SHALL supply the event stream using `--events <path|->`. A path
names the event stream file; `-` names standard input. The stream SHALL contain
newline-delimited JSON records that each encode one RT `Event` or RT
`Watermark` using the fields and value encodings of those RT types. When the
CLI reads a complete record, it SHALL decode the record and invoke the
corresponding RT operation in stream order.

## Inputs

- One request file using the existing `native-compile/1` envelope (`format`
  and `request`), with its existing `models`, `program.source`, and optional
  `libraries` as the source and dependency inputs. The generate request SHALL
  add a closed `monitor` member inside that same `request`; it SHALL NOT define
  a second source, model, library, or compile-request encoding.
- `monitor` SHALL identify the QSL producer inputs and the independent
  admission selection. Its `producer` object SHALL contain `source` (the
  `SourceIdentity` authority and identity), `path`, `package_name`, `clause`,
  `target` (`counter`, `unit`, `width_bits`, `uncertainty`, optional
  `max_event_rate`, `max_reading_gap`, and `max_activations`) and
  `settings` (`TimingObligationSettings`). The source bytes and model package
  inputs SHALL be taken from the enclosing native-compile request; the driver
  SHALL call QSL `emit_plan_root` with these inputs, the native-compile source
  and dependencies, and `SpineLimits` from the existing request limits.
- `monitor.selection` SHALL independently supply every input required by
  `admit_monitor_final_subject`, `admit_monitor_clock_selection`, and
  `prepare_monitor`: the selected package/root and package lock; root and
  four entry content keys (`init`, `on_event`, `on_watermark`, `on_gap`);
  clause, subject and full source-map occurrence; temporal profile; optional
  expected clock binding; counter, tick-unit node, width, maximum reading
  gap, exact maximum event rate, exact uncertainty, ordered buffers and
  activation capacity; selected producer fact rows; base Time dimension;
  authority selection, optional progress view and validation caps; and finite
  work/storage limits. These values SHALL be decoded into the existing
  `MonitorPreparationRequest`, `PlanAuthoritySelection`,
  `PlanAuthorityValidationCaps`, and `MonitorPreparationLimits` API types.
  The selected fact rows SHALL be retained from the QSL final emission and
  SHALL NOT be reconstructed from caller-authored copies.
- The request SHALL include the native checked-package evidence and read
  limits required by subject admission. It SHALL preserve each canonical
  identifier, rational, occurrence, selection and witness in its owner type's
  existing wire encoding; a display name alone SHALL NOT stand in for an
  admitted value.
- `--events <path|->`, containing RT `Event` and `Watermark` records.

## Outputs

- One final QSL `quire-outcome/1` document for the invocation. Its runtime
  result SHALL preserve one RT step result per consumed event or watermark in
  input order.
- Typed QSL or RT failure information when request preparation or an input
  record is refused.

## Behavior

- When the CLI receives a valid request, the driver SHALL resolve the QSL
  request, require exactly one selected monitor plan, prepare the complete
  monitor program under FR-019, and construct the RT monitor before consuming
  event records. RT construction SHALL run the monitor's `init` entry once.
- For each valid record, the driver SHALL call the matching RT `event` or
  `watermark` operation exactly once and append its `MonitorStep` results to
  the final outcome without changing their values or order.
- When the stream reaches EOF, the driver SHALL finish the invocation without
  synthesizing a final event or watermark and without claiming a temporal
  verdict that RT did not return.
- If monitor selection, preparation, program construction or initialization
  fails, the CLI SHALL return the typed failure before reading any event
  record.
- If a record is malformed or rejected by RT, the CLI SHALL stop at that
  record, return its typed failure, retain earlier completed step results and
  consume no following record.
- If the event stream is empty, the CLI SHALL return the successful initialized
  monitor result with no event or watermark step results.
- When EOF or the first failure occurs, machine mode SHALL write one final
  `quire-outcome/1` document and SHALL NOT emit a separate outcome document for
  each record.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-020-AC-1 | Given a request selecting one QSL runtime-monitor plan and a stream containing two RT `Event` records followed by one RT `Watermark`, `quire generate` constructs the monitor from the requested source, calls RT once per record in stream order, and returns the same ordered step results as the corresponding direct RT calls. | Test |
| FR-020-AC-2 | Given a request that selects zero or more than one runtime-monitor plan, or a monitor preparation, RT program construction or initialization refusal, `quire generate` returns the same typed refusal and consumes no event record. | Test |
| FR-020-AC-3 | Given one valid record, one malformed or RT-rejected record, and a later valid record, `quire generate` returns the typed failure at the middle record, preserves the first record's completed result, and does not consume the later record. | Test |
| FR-020-AC-4 | Given an empty event stream, `quire generate` runs RT initialization once, returns no step results, and reports no event-derived temporal verdict. | Test |
| FR-020-AC-5 | Given the same record stream through a named `--events` path and through `--events -` on standard input, the runtime step results and their order are equal. | Test |
| FR-020-AC-6 | Given EOF after any number of valid records, `quire generate` adds no implicit RT `Watermark` or `Event` and makes no terminal temporal claim not returned by RT. | Test |

## Dependencies

- [FR-002](./FR-002-expose-driver-cli-and-outcome-contract.md) owns the
  user-facing CLI and outcome document contract.
- [FR-015](./FR-015-prepare-an-admitted-monitor-program.md),
  [FR-017](./FR-017-lower-and-retain-admitted-monitor-programs.md), and
  [FR-019](./FR-019-prepare-the-complete-monitor-program-under-one-budget.md)
  own monitor selection, lowering, and preparation.
- QSL [FR-280](ix://agent-ix/quire-spec-language/FR-280) defines the generate
  target boundary; QSL [FR-283](ix://agent-ix/quire-spec-language/FR-283)
  defines monitor semantics that share the trace corpus.
- RT [FR-276](ix://agent-ix/quire-contract-runtime/FR-276) and
  [interface-012](ix://agent-ix/quire-contract-runtime/interface-012) define
  the event, watermark, step and failure behavior consumed by the driver.
- [TC-034](../test-cases/TC-034-generate-and-run-a-runtime-monitor.md)
  verifies this requirement.
