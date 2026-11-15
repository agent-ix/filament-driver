---
id: FR-002
title: "Expose the lifecycle through one driver CLI and outcome contract"
type: FR
relationships:
  - target: ix://agent-ix/filament-driver/US-001
    type: implements
  - target: ix://agent-ix/filament-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-285
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-286
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-287
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-298
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-301
    type: depends_on
  - target: ix://agent-ix/ix-cli-kit/FR-008
    type: depends_on
---
# FR-002: Expose the lifecycle through one driver CLI and outcome contract

## Description

The driver repository SHALL provide the one user-facing `fsl` binary. Its
lifecycle commands, output options, multi-item severity and process behavior SHALL
conform to FSL ADR-029 LC-1, CB-1, CB-4 and CB-5.

Each verb SHALL call its corresponding typed FSL or driver operation.
The CLI SHALL not make semantic decisions based on diagnostic text.

In machine mode, each verb SHALL write exactly one FSL-defined
`quire-outcome/1` JSON value to stdout and preserve the typed library outcome
without changing its schema or values, except when the CLI reports a reserved
exit-code mapping invariant failure or SIGINT/SIGTERM terminates an active
plugin session as specified below. The driver SHALL use FSL's single total
`Category::exit_code` function as the category-to-exit mapping; AC-6 specifies
the fallback when that function returns reserved value `5`.

## Inputs

- A lifecycle command and its options as defined by FSL ADR-029.
- Input documents and source files named by the options, or stdin where the
  verb accepts stdin.
- The typed outcome returned by the operation.

## Outputs

- One FSL-defined `quire-outcome/1` JSON value in machine mode, or the
  selected text rendering, except that a reserved-code mapping invariant
  failure or signal-terminated active plugin invocation writes no outcome
  document or text result. Until FSL supplies typed outcome text rendering,
  parse, check, compile and run text requests return the typed unsupported
  refusal specified in AC-13 and write no stdout.
- Root help text for a help invocation, as specified by AC-17; help does not
  produce a lifecycle outcome document.
- The exit status returned by FSL's category exit function, except that a
  reserved-code mapping invariant failure returns code `30`, and a
  signal-terminated active plugin invocation re-raises the signal after
  cleanup without an invented numeric exit result.

## Behavior

- The `fsl` binary SHALL be provided by the `fsl-cli` workspace package;
  the FSL workspace supplies no competing user binary. The `filament-driver`
  library SHALL NOT compose the downstream `filament-cache` package. FR-014
  specifies the separate CLI package boundary.
- The CLI SHALL preserve the library's FSL-defined `quire-outcome/1` value
  without adding, removing or changing outcome data, except in the two
  invocation cases specified by AC-6 and AC-7.
- When an invocation on Linux or macOS is terminated by SIGINT or SIGTERM
  during an active plugin session, after the plugin cleanup specified by
  FR-006 the CLI SHALL
  restore the default signal disposition and re-raise the received signal.
  That terminated invocation SHALL write no `quire-outcome/1` document, and
  the CLI SHALL make no FSL cancellation-cause claim.
- The CLI SHALL pass FSL's `Category::exit_code` result to
  `ix_cli_kit::exit::caller_exit_code`; it SHALL NOT remap accepted values
  through the kit's typed `Outcome`.
- If `caller_exit_code` returns `CallerExitCodeError` because FSL returned
  reserved result `5`, then the CLI SHALL write a plain-text diagnostic to
  stderr that identifies reserved code `5` and fallback code `30`. In this
  path, the CLI SHALL write no result or outcome document to stdout.
- If `caller_exit_code` returns `CallerExitCodeError` because FSL returned
  reserved result `5`, then the CLI SHALL return
  `Category::InternalFailure.exit_code()` (`30`).
- The diagnostic for the reserved result `5` SHALL not attribute a FSL cause
  to the mapping failure.
- When the CLI writes results and diagnostics, it SHALL use ix-cli-kit's
  `streams` module for stream discipline.
- When the CLI writes machine-mode outcomes, it SHALL use ix-cli-kit's `json`
  module for canonical JSON encoding.
- When writing or flushing primary command output to stdout fails with
  `BrokenPipe`, the CLI SHALL retain the category of the completed operation and use that
  category's normal process exit conversion. The invocation SHALL write nothing
  to stderr, including any pending operation diagnostics. It SHALL NOT
  reclassify the operation as an output failure or emit an `OutputFailure`
  diagnostic. This rule applies to successful and refused operations alike;
  the CLI SHALL defer operation diagnostics until it knows the primary write
  and flush disposition.
- When primary stdout writing and flushing succeed, the CLI SHALL retain its
  ordinary command diagnostics and category. Invocations that attempt no
  primary stdout write remain governed by their existing diagnostic and exit
  rules, including AC-6, AC-7 and AC-13; no closed-pipe probe or output artifact
  is introduced for them.
- When writing or flushing primary command output to stdout fails with an I/O
  error other than `BrokenPipe`, the CLI SHALL select
  `Category::InternalFailure` for process exit conversion and SHALL make a
  best-effort attempt to write an `OutputFailure` diagnostic to stderr that
  identifies the failed output. The CLI SHALL NOT promise that stdout is empty;
  a failed write can occur after some output bytes were written.
- When the user requests text output for parse, check, compile or run while
  FSL's typed outcome renderer is unavailable, the CLI SHALL refuse with FSL
  `Code::UnimplementedCapability` and SHALL NOT run the operation. Format
  continues to use FSL's formatter text.
- When the CLI resolves configuration values, it SHALL use ix-cli-kit's
  `config` module.
- When invoked with no arguments or with `--help`, `-h` or `help` as its sole
  argument, the CLI SHALL write root help to stdout without starting a
  lifecycle operation.
- The root help SHALL include the `fsl` invocation syntax, the recognized
  lifecycle verb names, the `--format text|json|dot` option with the `inspect`
  restriction on `dot`, and the `--version`/`-V` version options.
- The root help SHALL distinguish lifecycle verbs that execute operations
  from verbs that currently return an unsupported-capability result, and
  identify any default text-output restriction for executable verbs.
- When root help is written successfully, the CLI SHALL return FSL's success
  exit result and write no stderr output.
- When the user invokes `fsl --version`, the CLI SHALL report the semantic
  version declared by the `fsl-cli` Cargo package.
- When `fsl-cli` runs its built-binary version-agreement check, it SHALL
  compare `CARGO_PKG_VERSION` with the version reported by the built
  `fsl --version` command through
  `ix_cli_kit::version::Agreement::command`.
- The CLI SHALL keep its argument parser in the `fsl-cli` package.
- The CLI SHALL read only input paths named by its arguments and read no
  unnamed configuration file.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | The driver binary exposes the command and output surface defined by FSL ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation. | Test (TC-009) |
| FR-002-AC-2 | For each FSL category whose exit result is not reserved code `5`, the process status equals the value returned by FSL's `Category::exit_code`; no second category-to-exit mapping is applied. Reserved result `5` is governed by AC-6. | Test (TC-009, TC-018) |
| FR-002-AC-3 | In machine mode, each command writes one parseable `quire-outcome/1` value equal to the library outcome under FSL's schema; the CLI adds, removes or changes no outcome data. The mapping-invariant and signal exceptions are specified in AC-6 and AC-7. | Test (TC-009, TC-018, TC-028) |
| FR-002-AC-4 | The CLI passes FSL's `Category::exit_code` result through `ix_cli_kit::exit::caller_exit_code` and preserves each accepted numeric value without mapping it through the kit's typed `Outcome`. | Test (TC-009, TC-018) |
| FR-002-AC-5 | The CLI reads only input paths named by its arguments and reads no unnamed configuration file. | Test (TC-009) |
| FR-002-AC-6 | If FSL's category exit function returns reserved value `5`, the CLI writes a plain-text stderr diagnostic naming reserved code `5` and fallback code `30`, writes no text result or `quire-outcome/1` document to stdout, and returns `Category::InternalFailure.exit_code()` (`30`) without attributing a FSL cause. | Test (TC-009, TC-018) |
| FR-002-AC-7 | When a Linux or macOS invocation receives SIGINT or SIGTERM during an active plugin session, the CLI cleans up, restores the default disposition, and re-raises that signal without writing a result or outcome document or inventing a numeric exit value or FSL cause. | Test (TC-028) |
| FR-002-AC-8 | When the CLI writes results and diagnostics, it uses ix-cli-kit's `streams` module for stream discipline. | Test (TC-009, TC-018) |
| FR-002-AC-9 | When the CLI writes machine-mode outcomes, it uses ix-cli-kit's `json` module for canonical JSON encoding. | Test (TC-009, TC-018) |
| FR-002-AC-10 | When the CLI resolves configuration values, it uses ix-cli-kit's `config` module. | Test (TC-009) |
| FR-002-AC-11 | When invoked with `--version`, the CLI reports the semantic version declared by the `fsl-cli` Cargo package. | Test (TC-009) |
| FR-002-AC-12 | The `fsl-cli` built-binary version-agreement check uses `ix_cli_kit::version::Agreement::command` to compare `CARGO_PKG_VERSION` with the output of the built `fsl --version` command and requires agreement. | Test (TC-009) |
| FR-002-AC-13 | When text rendering is unavailable for parse, check, compile or run, the CLI returns the operation-specific FSL `UnimplementedCapability` result, writes no stdout, and does not execute the requested lifecycle operation. | Test (TC-009) |
| FR-002-AC-14 | For a `BrokenPipe` while writing or flushing primary stdout, the process status equals the completed operation's FSL category exit result and stderr is empty, including for a refused operation with nonempty diagnostics when stdout is writable. No output-failure category is selected and no `OutputFailure` diagnostic is emitted. | Test |
| FR-002-AC-15 | If writing or flushing primary stdout returns an I/O error other than `BrokenPipe`, the process exits with FSL's `Category::InternalFailure` exit result. The criterion makes no assertion that stdout is empty or complete. | Test (TC-018) |
| FR-002-AC-16 | If a non-`BrokenPipe` primary stdout write or flush fails and stderr is writable, stderr contains an `OutputFailure` diagnostic that identifies the failed output. | Test (TC-018) |
| FR-002-AC-17 | With no arguments or with each of `--help`, `-h` and `help` as the sole argument, the built `fsl` binary writes root help to stdout containing invocation syntax, recognized lifecycle verb names, `--format text\|json\|dot` with `dot` limited to `inspect`, and `--version`/`-V`. The help distinguishes executable verbs from verbs that currently return unsupported-capability results and identifies executable verbs whose default text output is unavailable. Successful help exits with FSL's success result, writes no stderr, and starts no lifecycle operation. | Test |

## Dependencies

- FSL ADR-029 CB-1, CB-4 and CB-5; FSL FR-285, FR-286 and FR-287.
- FSL FR-298 defines typed rendering; QSL-600/QSL-601 own its pending
  `fsl-inspect` implementation. AC-13 specifies the driver's behavior until
  that renderer is available.
- FSpec FR-300 and FR-301 for the CLI output and process contract.
- [ix-cli-kit FR-017](ix://agent-ix/ix-cli-kit/FR-017) defines the caller-owned
  exit-code pass-through; FSL ADR-029 remains authoritative for category
  outcomes and codes. IR-607 adopts the exit module first and retains the
  remaining required kit mechanics as open work.
- [ix-cli-kit FR-008](ix://agent-ix/ix-cli-kit/FR-008) defines the built-binary
  version agreement used by AC-12.
- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md).
- [TC-018](../test-cases/TC-018-binary-process-boundary.md), adding verification for AC-2 through AC-4 and AC-6.

AC-14 adopts FSL FR-287-AC-8's quiet behavior only when a primary stdout write
or flush actually reports `BrokenPipe`. AC-13's outputless text refusals do not
attempt that write, so this requirement does not qualify the whole FSL
criterion across output modes. It adds no file-descriptor probe or replacement
refusal envelope.
