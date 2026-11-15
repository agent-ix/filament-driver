---
id: FR-011
title: "Cancel driver lifecycle stages through a caller-owned handle"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-driver/FR-001
    type: references
  - target: ix://agent-ix/filament-driver/TC-019
    type: references
  - target: ix://agent-ix/filament-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-276
    type: depends_on
---
# FR-011: Cancel driver lifecycle stages through a caller-owned handle

## Description

The driver library SHALL accept a caller-owned, cloneable FSL `Cancel` handle
for its compile-and-drive operation and pass that same handle to each FSL
lifecycle stage it invokes: `parse`, `select`, `check` and `package`. The
caller may cancel through any clone, including from another thread while one
of those stages is running. The driver SHALL propagate FSL's typed cancelled
failure to its caller as a distinct cancellation result carrying the FSL
stage and `CancelCause`. This cancellation guarantee covers those FSL calls;
it does not interrupt the driver's later routing or CG generation after
`package` has completed.

The public `DriverRefusal` type SHALL expose a distinct
`Cancelled { stage: SpineStage, cause: CancelCause }` variant.

When the driver classifies this cancellation, the driver SHALL use FSL's
category for `StageFailure::Cancelled(cause)`.

The driver SHALL NOT route this cancellation result through the
refusal-or-fault conversion for an unshared `Cancel` token.

The caller-visible category remains FSL-owned: a cancelled operation is
`incomplete` under ADR-029 CB-4. The binary obtains its process code from
FSL's one `Category::exit_code` function, as FR-002 specifies; this
requirement defines no exit table or mapping.

This requirement specifies the library API first so realtime and embedded
callers can cancel without a process signal or timer. A CLI signal/deadline
adapter is a separate thin frontend layer: it may own signal handling or a
timer and cancel the same handle with the corresponding FSL cause. That CLI
adapter is planned separately; this requirement claims no signal handler,
timeout, timer, or signal-to-cause mapping.

## Inputs

- The normal driver request and limits specified by FR-001.
- A caller-owned, cloneable FSL `Cancel` handle.

## Outputs

- The normal completed `Driven` result when all stages complete.
- A distinct typed cancellation result carrying the FSL stage and cause when
  cancellation stops a lifecycle stage; no completed `Driven` result or
  partial E4 package is returned for that failed call.

## Behavior

- The driver SHALL neither create a private replacement handle nor mask a
  cancellation failure from FSL.
- The public cancellation result SHALL identify the FSL `SpineStage` and
  preserve its `CancelCause`; its FSL category is `incomplete`.
- The driver SHALL NOT pass `StageFailure::Cancelled(CancelCause)` through
  the refusal-or-fault conversion that treats an unshared cancellation
  handle as an internal fault.
- FSL polls cancellation at its stage/work charges and stops within one
  charge; the driver does not poll independently or decide when to cancel.
- After FSL `package` completes, a later cancellation does not stop routing
  or CG generation under this requirement.
- The library does not read a clock. A caller or separate CLI frontend that
  owns a deadline requests cancellation using FSL's `Deadline` cause.
- The CLI's signal and timeout adapter remains planned and outside this
  library requirement's implementation scope.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-011-AC-1 | The public driver library accepts one caller-owned `Cancel` token and passes that same token to `parse`, `select`, `check` and `package` in order. | Test |
| FR-011-AC-2 | When the caller cancels the token during a FSL lifecycle stage, the public driver failure identifies that `SpineStage` and preserves the `CancelCause`; no `Driven` result or partial E4 package is returned. | Test |
| FR-011-AC-3 | The cancellation result has FSL's `incomplete` category, and the binary obtains the process code only through FSL `Category::exit_code`; the driver adds no category or exit conversion. | Test |
| FR-011-AC-4 | The driver does not route cancellation through the refusal-or-fault conversion for an unshared `Cancel` token. | Test |
| FR-011-AC-5 | CLI signal and deadline triggers are outside this specification slice; a later frontend adapter uses the caller's `Cancel` token to request cancellation. | Inspection |

## Dependencies

- FSL ADR-029 LC-3 cancellation and CB-4 outcome mapping.
- FSL FR-275 operation request shape, FR-276 cancellation, and FR-285
  category exit function.
- Driver FR-001 compile sequence and E5 hand-off.

## Status

Specified; implementation and CLI signal/deadline integration remain
planned.
