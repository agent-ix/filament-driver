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
  - target: ix://agent-ix/filament-spec-language/FR-865
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-866
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-931
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-1039
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-960
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-718
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-301
    type: depends_on
  - target: ix://agent-ix/filament-spec-language/FR-665
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-522
    type: depends_on
  - target: ix://agent-ix/filament-specification/FR-562
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
`filament-outcome/1` JSON value to stdout and preserve the typed library outcome
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

- One FSL-defined `filament-outcome/1` JSON value in machine mode, or the
  selected text rendering, except that a reserved-code mapping invariant
  failure or signal-terminated active plugin invocation writes no outcome
  document or text result. Text is the default output format: without
  `--format`, successful parse, check, compile and run operations write the
  FSL FR-865 text form of the same `filament-outcome/1` document that
  `--format json` writes, with the header naming the invoked command, and
  analyze and monitor write the successful text forms FR-022 and FR-025 give
  them. Refusals follow the mode-specific routing rule below. The
  header departs from FSL FR-865, whose header names the operation
  (`execute` for `fsl run`); the JSON `operation` is unchanged.
- `fsl analyze` text mode renders FR-290's typed
  unusable-provider-counterexample diagnostic immediately after its item's
  line using the exact code, cause, typed reason and fixed message form
  FR-022 specifies; it adds no provider-supplied text.
- Root help text for a help invocation, as specified by AC-17 and AC-24, and
  one verb's section of that help for `fsl <verb> --help`, as specified by
  AC-25; help does not produce a lifecycle outcome document.
- For `fsl inspect --handoff <directory>`, the `filament-outcome/1` document
  with operation `inspect` that FSL's `inspect_handoff` returns, or its text
  form (AC-23).
- When the command line names no recognized verb, or is not valid UTF-8, there
  is no operation to name in an outcome: the driver writes one diagnostic line
  to standard error, nothing to standard output, and exits 20.
- The exit status returned by FSL's category exit function, except that a
  reserved-code mapping invariant failure returns code `30`, and a
  signal-terminated active plugin invocation re-raises the signal after
  cleanup without an invented numeric exit result.

## Behavior

- The `fsl` binary SHALL be provided by the `fsl-cli` workspace package;
  the FSL workspace supplies no competing user binary. The `filament-driver`
  library SHALL NOT compose the downstream `filament-cache` package. FR-014
  specifies the separate CLI package boundary.
- The CLI SHALL preserve the library's FSL-defined `filament-outcome/1` value
  without adding, removing or changing outcome data, except in the two
  invocation cases specified by AC-6 and AC-7 and the text-mode export hint line of
  FR-028-AC-7; machine mode is unchanged.
- When an invocation on Linux or macOS is terminated by SIGINT or SIGTERM
  during an active plugin session, after the plugin cleanup specified by
  FR-006 the CLI SHALL
  restore the default signal disposition and re-raise the received signal.
  That terminated invocation SHALL write no `filament-outcome/1` document, and
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
- When the CLI begins processing a verb invocation, it SHALL resolve the
  refusal output mode before validating other verb options or reading inputs.
  No `--format` or one valid `--format text`
  selects text; one valid `--format json` selects JSON. If `--format` is
  missing its value, repeated or names an unsupported value, the CLI SHALL
  render that refusal as text. Any refused `inspect` invocation carrying
  `--format dot` also uses the text refusal rendering because DOT does not
  encode an outcome.
- In `--format json` mode the diagnostics of a document result are carried in
  the document and nothing is written to stderr.
- When the CLI writes a refusal in JSON mode, it SHALL write its
  `filament-outcome/1` refusal document to stdout and nothing to stderr,
  whether the refusal comes from option validation, input admission,
  configuration, or the FSL operation.
- When the CLI writes a refusal in text mode, it SHALL write the selected
  text rendering of that refusal outcome to stdout and nothing to stderr.
  This applies uniformly to every lifecycle verb; a streaming verb may retain
  event lines already written before the refusal, then writes its text refusal
  outcome to stdout.
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
  rules, including AC-6 and AC-7; no closed-pipe probe or output artifact
  is introduced for them.
- When writing or flushing primary command output to stdout fails with an I/O
  error other than `BrokenPipe`, the CLI SHALL select
  `Category::InternalFailure` for process exit conversion and SHALL make a
  best-effort attempt to write an `OutputFailure` diagnostic to stderr that
  identifies the failed output. The CLI SHALL NOT promise that stdout is empty;
  a failed write can occur after some output bytes were written.
- The `--format` option of every lifecycle verb SHALL default to `text`
  when omitted; `--format json` selects machine mode. Text is the default so that
  the first command a reader types, `fsl check rule.fsl`, prints a readable
  result without a flag (FSpec FR-718).
- When parse, check, compile or run runs in text mode, the CLI SHALL build
  the outcome document exactly as in machine mode, render it through FSL
  FR-298's `render_outcome` with `RenderForm::Text` and a `RenderSources`
  built from the admitted sources the command read (FSL FR-865), and write
  the rendered bytes to stdout as its primary output. The rendered text names
  the operation, category, exit code, last stage when known, every
  diagnostic's code, cause and locus, every item's result, category and
  cause, and the result. None of these verbs SHALL refuse text mode as
  `UnimplementedCapability`.
- `fsl check <file>` SHALL name the source it checks by the file the caller
  supplied: authority `filament` and identity the path exactly as given on
  the command line, on every path, a source that does not parse included.
  Every diagnostic locus and the admitted sources of its outcome SHALL carry
  that identity, never an authority or identity of the command layer, so a
  diagnostic locates the file the caller wrote. The binary's refusal of a
  source SHALL be byte-equal, with the same exit status, to the document the
  library parse and check give for the same bytes under that identity, as
  FSL FR-866 states for `check_outcome`.
- The CLI SHALL take the source values for `RenderSources` from the admitted
  sources exposed by FSL's compile, run and check outcomes and `RunError`,
  and SHALL NOT read a source a second time to render it.
- The process exit status of an invocation SHALL be equal in text and machine
  mode; the format changes only the bytes on stdout.
- A missing request file or a malformed request SHALL refuse in text mode
  with the same FSL code and exit status as in machine mode; text mode writes
  the text form of that refusal document.
- If `--format` names a value other than `text`, `json`, or `dot` on
  `inspect`, the CLI SHALL refuse `invalid-request`, exit 20, naming
  `--format` and the value, before reading any file.
- `fsl inspect --handoff <directory>` SHALL call FSL's
  `fsl_command::command::inspect_handoff` with the directory exactly as given
  and write the `filament-outcome/1` document it returns, operation
  `inspect`, unchanged under `--format json` and in its FSL FR-865 text form
  under `--format text` or without `--format`, with that document's category
  exit result. FSpec FR-301 and FSL FR-665 put this form in the default build:
  it is how a user verifies a checked-package handoff that `fsl compile`
  wrote. Success, exit 0, means the strict reader admitted the package and it
  agrees with the selection sidecar; any other result is the reader's typed
  refusal or the `SelectionRefusal`, exit 20.
- The driver SHALL make no decision of its own about the handoff directory: a
  missing path, a file in place of a directory, an empty directory and a
  changed byte all reach `inspect_handoff`, and the CLI writes the refusal it
  returns. The CLI adds no existence probe that could disagree with the
  reader.
- `--handoff` SHALL take the next argument verbatim as its directory, even
  when it starts with `-` (`fsl inspect --handoff ./-odd` names `./-odd`),
  except when that argument is itself an option name the `inspect` entry of
  the option table lists: then the directory operand is missing.
- The handoff form SHALL take exactly one directory operand after `--handoff`
  and no other operand. A missing directory operand, a second operand or a
  repeated `--handoff` SHALL refuse `invalid-request`, operation `inspect`,
  exit 20, naming the missing operand, the extra operand or `--handoff`,
  before any file is read.
- `--format dot` together with `--handoff` SHALL refuse `invalid-request`,
  operation `inspect`, exit 20, naming `--format` and `dot`, before any file
  is read: a handoff verification result is an outcome document, not a graph,
  so it has no DOT rendering.
- An `inspect` invocation without `--handoff` stays a verb form this build
  does not route: `unsupported` with cause `operation-not-routed` naming
  `inspect` (FSpec FR-522), exit 21, under every accepted `--format` value
  including `dot`, whatever positional operands it carries.
- The CLI SHALL decide an `inspect` invocation in this order, and the first
  step that refuses decides the result: (1) the root `--format` checks of
  AC-18 (exit 20); (2) an option the `inspect` entry of the option table does
  not list, with or without `--handoff` (`invalid-request` naming it, exit
  20); (3) `--help` or `-h` (AC-25, exit 0); (4) the handoff operand shape
  and `--format dot` with `--handoff` (exit 20); (5) with `--handoff`, the
  call to `inspect_handoff`, and without it the not-routed result (exit
  21). An unknown option is a mistake in the invocation whatever form it
  meant, so it refuses before the not-routed result.
- Analyze and monitor keep the text forms FR-022 and FR-025 specify. Format
  continues to write FSL's formatted source as its text output.
- `fsl run` SHALL route every prepared run FSL's `prepare_run_with_overrides`
  returns, and refuse none of them as not run by the driver: a function run, a
  clause run, a TL lowering (FSL FR-931) and a package run (FSL FR-1039). Each
  arm runs through FSL's own executor for it, so `fsl run` gives the document
  FSL's in-process `run` gives for the same request.
- For a TL lowering, the driver SHALL apply its `--limit` overrides to the
  lowering's clause request through `PreparedLowering::clause_request_mut`
  with the same clause-run application FR-003 uses for a clause run, then call
  `execute_tl_lowering`. It renders the result as it renders a function run's
  result (the result's outcome document when FSL returns one, else the native
  result with its exit code) and a `RunError` through
  `RunError::outcome_document` for the execute operation. A TL lowering
  therefore takes the same caller limits and the same outcome contract as
  every other prepared run (FR-003).
- For a package run, the driver SHALL call `execute_package_run` on the
  prepared package and render its result and `RunError` the same way.
- When `fsl run` settles requirement items, the driver SHALL run the request
  through `fsl_command::command::run_with_providers_and_limits_and_overrides`
  with `Engine::hybrid_reference()` followed by the configured providers.
  `fsl run` settles requirement items when it is given `--provider-config` or
  `--provider-limit` (FR-006), or when the request's `call` carries
  `requirements` (FSL FR-960).
- When `fsl run` settles requirement items, the driver SHALL negotiate over
  `Engines::fsl_with_reference_hybrid()` plus the configured providers, as
  FR-022 does for `fsl analyze`. A requirement item over hybrid
  dynamics is then decided by the same providers whichever verb carries it
  (FSL FR-960 routes run items by analyze's rule).
- When the CLI resolves configuration values, it SHALL use ix-cli-kit's
  `config` module.
- When invoked with no arguments or with `--help`, `-h` or `help` as its sole
  argument, the CLI SHALL write root help to stdout without starting a
  lifecycle operation.
- The root help SHALL include the `fsl` invocation syntax, the recognized
  lifecycle verb names, the `--format text|json|dot` option with the `inspect`
  restriction on `dot`, and the `--version`/`-V` version options.
- The root help SHALL distinguish lifecycle verbs that execute operations
  from verbs the build accepts but does not route, which return an
  `unsupported_operation` result, and state that `text` is the default output
  format and `--format json` selects machine output.
- When root help is written successfully, the CLI SHALL return FSL's success
  exit result and write no stderr output.
- The `fsl-cli` package SHALL hold one option table. For each verb that
  executes operations it records the verb's usage lines (operand forms) and,
  in a fixed order, every option the verb's parser accepts, with the option's
  value form (for example `--limit name=value`,
  `--engine interpreter|aot|jit`) and whether the option may repeat.
- Each verb's option parser SHALL derive the options it accepts only from
  its table entry: the parser and the help generator take the table as an
  argument, and no option name appears as a literal in `fsl-cli`'s parsing
  code outside the table. The parser SHALL refuse every option-shaped
  argument (one starting with `-`, other than a value an option consumes)
  that its entry does not list, as `invalid-request` with the verb's
  operation, exit 20, naming the option, before any file is read. The CLI
  generates root help and verb help from that same table. With no second
  list of options, the help cannot name an option the parser refuses and the
  parser cannot accept an option the help omits.
- Root help SHALL keep its current layout, which the filament-docs reference
  generator reads: the `Usage:` lines, then a `Commands:` section with one
  line per recognized verb (a verb this build does not route keeps the
  description that starts `Recognized;`), then a `Source operands:` section
  with each routed verb's usage lines from the table (including
  `inspect [options] --handoff <directory>`), then the `Options:`
  section of root options. Root help SHALL add, for each verb that
  executes operations in `Commands:` order and after `Options:`, one section headed
  `<verb> options:` followed by one line per table option in table order:
  two spaces, the option name, one space and its value form when it takes a
  value, and ` (repeatable)` when it may repeat. Sections are separated by
  one blank line. `inspect` is listed among the verbs that execute
  operations, through its `--handoff` form.
- `fsl <verb> --help` SHALL print the verb's usage lines, each starting
  `Usage: fsl <verb> ` for the first and `       fsl <verb> ` for the rest,
  then one blank line, then the verb's `<verb> options:` section exactly as
  root help prints it. A user who types `fsl run --help` learns the run
  options without reading the whole root help.
- The CLI SHALL decide an invocation of a verb that executes operations in
  this order, and the first step that refuses decides the result: (1) the
  root `--format` checks of AC-18 and the `dot` restriction (exit 20), even
  when `--help` is present; (2) an option-shaped argument the verb's table
  entry does not list (`invalid-request`, exit 20); (3) `--help` or `-h` in
  option position (help, exit 0); (4) the verb's operand and value checks;
  (5) the operation. Help applies only to verbs that execute operations: for
  `prove`, `generate` and `render`, `--help` is not a help request and the
  invocation gives FSpec FR-522's `unsupported`/`operation-not-routed`
  result, exit 21.
- When help is written in step 3, the CLI SHALL return FSL's success exit
  result, write no stderr, read no file and start no lifecycle operation.
- `--version` and `-V` are root options only. No verb's table entry lists
  them, so after a verb each refuses in step 2 as `invalid-request`, exit 20,
  naming the option.
- The request verbs take `--example` (and `export` also `--example-evidence`), and the text of their request refusals is
  specified by [FR-028](FR-028-print-a-minimal-request-with-example.md); the CLI
  writes those refusals unchanged under AC-3.
- The CLI SHALL accept no `--color` option on any verb.
- The CLI SHALL emit no ANSI escape sequence of its own on stdout or
  stderr: its text output
  and diagnostics are plain text, so no option selects colour and the
  `NO_COLOR` environment variable has nothing to switch off. Bytes the CLI
  echoes from its input, such as formatted source or a source line quoted in
  a diagnostic, are the caller's bytes and are exempt. `--color` is an
  option no table entry lists and refuses in step 2. AD-001 plans the
  adoption of ix-cli-kit's `streams` module, which handles colour; coloured
  output, with an option to select it, is a later requirement with its own
  specification, not part of this contract.
- When the user invokes `fsl --version`, the CLI SHALL report the semantic
  version declared by the `fsl-cli` Cargo package and the 40-character commit
  SHA of the FSL source revision pinned for that build. It SHALL print the
  revision on a separate line in the form `FSL revision: <sha>`.
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
| FR-002-AC-3 | In machine mode, each command writes one parseable `filament-outcome/1` value equal to the library outcome under FSL's schema; the CLI adds, removes or changes no outcome data. The mapping-invariant and signal exceptions are specified in AC-6 and AC-7; the one further exception is the text-mode export hint line of FR-028-AC-7 (machine mode is unchanged), on a missing-`export` or missing-`export_evidence` refusal of `fsl export` only. | Test (TC-009, TC-018, TC-028) |
| FR-002-AC-4 | The CLI passes FSL's `Category::exit_code` result through `ix_cli_kit::exit::caller_exit_code` and preserves each accepted numeric value without mapping it through the kit's typed `Outcome`. | Test (TC-009, TC-018) |
| FR-002-AC-5 | The CLI reads only input paths named by its arguments and reads no unnamed configuration file. | Test (TC-009) |
| FR-002-AC-6 | If FSL's category exit function returns reserved value `5`, the CLI writes a plain-text stderr diagnostic naming reserved code `5` and fallback code `30`, writes no text result or `filament-outcome/1` document to stdout, and returns `Category::InternalFailure.exit_code()` (`30`) without attributing a FSL cause. | Test (TC-009, TC-018) |
| FR-002-AC-7 | When a Linux or macOS invocation receives SIGINT or SIGTERM during an active plugin session, the CLI cleans up, restores the default disposition, and re-raises that signal without writing a result or outcome document or inventing a numeric exit value or FSL cause. | Test (TC-028) |
| FR-002-AC-8 | When the CLI writes results and diagnostics, it uses ix-cli-kit's `streams` module for stream discipline. | Test (TC-009, TC-018) |
| FR-002-AC-9 | When the CLI writes machine-mode outcomes, it uses ix-cli-kit's `json` module for canonical JSON encoding. | Test (TC-009, TC-018) |
| FR-002-AC-10 | When the CLI resolves configuration values, it uses ix-cli-kit's `config` module. | Test (TC-009) |
| FR-002-AC-11 | When invoked with `--version`, the CLI reports the semantic version declared by the `fsl-cli` Cargo package and the FSL source revision pinned for that build. The revision appears on a separate line as `FSL revision: <revision>`. For a build pinned to an FSL git commit, `<revision>` is that 40-character commit SHA. For a build that patches the FSL packages to a local path checkout named by `FSL_LANGUAGE_DIR`, `<revision>` is the `HEAD` commit of that checkout followed by ` (local path)`, or `local path` alone when the directory has no `.git`, or `unknown (local path)` when its git metadata is unreadable (a `.git` file, `HEAD`, or the ref it names that is missing or is not a 40-hexadecimal commit; a malformed loose ref is unknown and is not replaced by a `packed-refs` value, and a 64-hexadecimal SHA-256 commit is unknown), with a build warning and without failing the build. A lock entry for the FSL packages that has a git source but is not the pinned FSL repository commit fails the build instead. The build re-reads the checkout whenever `HEAD`, the ref it names, `packed-refs` or the `.git` entry changes. | Test (TC-009) |
| FR-002-AC-12 | The `fsl-cli` built-binary version-agreement check uses `ix_cli_kit::version::Agreement::command` to compare `CARGO_PKG_VERSION` with the output of the built `fsl --version` command and requires agreement. | Test (TC-009) |
| FR-002-AC-13 | `fsl check <file>`, `fsl parse <authority> <identity> <file>`, `fsl compile <request>` and `fsl run <request>` without `--format`, and with `--format text`, each over a succeeding and a refused input, write the FR-865 text form of the document `--format json` writes for the same invocation, except that the text header names the invoked CLI command (`check`, `parse`, `compile` or `run`) instead of the document's operation. The process exit status is unchanged between modes, and no refusal writes to stderr. None returns `UnimplementedCapability` or exit 21. | Test (TC-040) |
| FR-002-AC-14 | For a `BrokenPipe` while writing or flushing primary stdout, the process status equals the completed operation's FSL category exit result and stderr is empty, including for a refused operation with nonempty diagnostics when stdout is writable. No output-failure category is selected and no `OutputFailure` diagnostic is emitted. | Test |
| FR-002-AC-15 | If writing or flushing primary stdout returns an I/O error other than `BrokenPipe`, the process exits with FSL's `Category::InternalFailure` exit result. The criterion makes no assertion that stdout is empty or complete. | Test (TC-018) |
| FR-002-AC-16 | If a non-`BrokenPipe` primary stdout write or flush fails and stderr is writable, stderr contains an `OutputFailure` diagnostic that identifies the failed output. | Test (TC-018) |
| FR-002-AC-17 | With no arguments or with each of `--help`, `-h` and `help` as the sole argument, the built `fsl` binary writes root help to stdout containing invocation syntax, recognized lifecycle verb names, `--format text\|json\|dot` with `dot` limited to `inspect`, and `--version`/`-V`. The help distinguishes executable verbs from verbs this build does not route, which return `unsupported_operation` results, and states that `text` is the default output format and `--format json` selects machine output. Successful help exits with FSL's success result, writes no stderr, and starts no lifecycle operation. | Test |
| FR-002-AC-18 | `fsl run nosuch.json` and `fsl compile nosuch.json` without `--format` refuse with the I/O failure's FSL code and exit 20, as under `--format json`, never `UnimplementedCapability` or exit 21; both modes write the refusal outcome on stdout in their selected format and nothing to stderr. The text of a refused `check` names each diagnostic's code, cause and source position; `--format bogus` on each of the four verbs refuses `invalid-request`, exit 20, naming `--format` and `bogus`, before any file is opened, with the refusal outcome on stdout in the selected format and no stderr. | Test (TC-040) |
| FR-002-AC-19 | `fsl check P --format json` over a source at path `P` that does not parse writes the `check` refusal whose every diagnostic locus names `{"authority": "filament", "identity": P}`, with `P` exactly as given, and that contains neither `fsl-command` nor `check-source`; its stdout is byte-equal to the canonical encoding of the document FSL's `parse` gives for the same bytes under that identity, plus a newline, with that document's exit status, and `compile` and `run` over requests naming the same source refuse equally to their library operations. | Test (TC-043) |
| FR-002-AC-20 | `fsl run R --format json`, `R` FSL FR-931-AC-1's request `{"clause": "Low"}` over the ConfigVersion model, writes the `tl_lowering` record with `disposition` `supported`, `correspondence` `mltl.closed-trace/v1` and `formula` `G[0,3] p0`, exit 0, and its stdout is byte-equal to the canonical encoding of the document FSL's `fsl_command::command::run` returns for `R`, plus a newline; it never writes the "not run by the driver" input refusal. `fsl run R --limit s1.nodes=1 --format json` writes FSL's typed limit outcome naming `s1.nodes` with configured value 1, category `incomplete`, no lowering record, exit 22; `--limit s1.nodes=50000` gives the unlimited run's bytes, exit 0. | Test (TC-046) |
| FR-002-AC-21 | `fsl run P --format json`, `P` FSL FR-1039-AC-1's request selecting the `half` unit's checked package as `package` `{file}` with the `call` `half`, run with the source file removed, completes with `-4`, exit 0, byte-equal to FSL's `run` for `P` plus a newline; the same request with `integer_division` gives FSL's `invalid-request` refusal naming `integer_division`, exit 20. | Test (TC-046) |
| FR-002-AC-22 | `fsl run Q --format json`, `Q` FSL FR-960-AC-1's request calling `sq` with `x = 3` and `call.requirements` `{}`, with no `--provider-config`, writes stdout byte-equal to the canonical encoding of the document `run_with_providers_and_limits_and_overrides` returns for `Q` with the providers `[Engine::hybrid_reference()]` and default limits, plus a newline: value `{"integer": "16"}` and three settled requirement entries, exit 0. With `--provider-config` starting a fixture provider whose admitted backend is `fsl-hybrid-reference`, the run refuses `invalid-request` naming `providers`, exit 20, because that backend is already registered. | Test (TC-046) |
| FR-002-AC-23 | Over a handoff directory `H` written by `fsl compile` from a `native-compile/1` request carrying FSL FR-665-AC-1's `handoff` member, `fsl inspect --handoff H --format json` writes stdout byte-equal to the canonical encoding of the document `fsl_command::command::inspect_handoff(H)` returns, plus a newline: operation `inspect`, category success, exit 0; without `--format` it writes that document's FSL FR-865 text form, exit 0. Over a copy of `H` whose `selection.json` names one node count more, and over a copy whose `checked-package.json` has one changed byte, each run refuses with the `SelectionRefusal` or strict-reader refusal `inspect_handoff` returns, byte-equal to it, exit 20. A path that does not exist, an empty directory, a regular file, and a directory named `-odd` given as `./-odd` each yield the document `inspect_handoff` returns for that path exactly as given (refusal, exit 20, for the first three). `fsl inspect --handoff` with no further argument, `fsl inspect --handoff --format json`, `fsl inspect --handoff H extra` and `fsl inspect --handoff H --handoff H` each refuse `invalid-request`, operation `inspect`, exit 20, naming the missing operand, the missing operand, `extra` and `--handoff`; `fsl inspect --handoff H --format dot` refuses `invalid-request`, exit 20, naming `--format` and `dot`; `fsl inspect --handoff H --bogus` and `fsl inspect H --bogus` each refuse `invalid-request`, exit 20, naming `--bogus`; none of these reads a file under `H`. `fsl inspect H --format json` without `--handoff` writes `unsupported`/`operation-not-routed` naming `inspect`, exit 21. | Test (TC-048) |
| FR-002-AC-24 | For every verb that executes operations and every option `o` in its `fsl-cli` option table entry, `o` appears on its own line in the verb's `<verb> options:` section of root help and the built parser accepts `o` with a well-formed value (no `invalid-request` naming `o`). For every verb that executes operations and every option name `p` that is listed in another verb's entry or is one of `--version`, `-V`, `--color` and `--bogus`, but is not in this verb's entry, `fsl <verb> p` refuses `invalid-request`, exit 20, naming `p`; in both output modes its refusal outcome is the only command result written to stdout and nothing is written to stderr. Removing one option from a copy of the table and running the parser and the help generator over that copy makes the parser refuse that option, exit 20 naming it, and makes the help omit it; a parser that accepts an option outside its table entry, or a help line that names one, fails this criterion. No option name (a string literal starting `-`) appears in `fsl-cli`'s parsing code outside the table. The entries include `--integer-division` and `--markdown` on `check`, `--extensions <file>` on `parse` and `check` (FSpec FR-562), repeatable `--limit name=value` on `analyze`, `monitor` and `replay` (FR-003), every option of `analyze`, `monitor`, `replay` and `export`, `--provider-limit name=value` on `run`, and `--handoff <directory>` on `inspect`. Root help keeps the `Commands:`, `Source operands:` and `Options:` sections and the `Recognized;` description of each verb this build does not route. | Test (TC-049) |
| FR-002-AC-25 | `fsl check --help`, `fsl check -h` and `fsl check nosuch.fsl --help`, from an empty working directory, each write exactly the following to stdout, exit 0, write no stderr and read no file: `Usage: fsl check [options] <source-path>` / `       fsl check [options] <authority> <identity> <source-path>` / (blank) / `check options:` / `  --markdown` / `  --model <identity>=<file> (repeatable)` / `  --library <identity>=<file> (repeatable)` / `  --dependent-bytes <bytes>` / `  --limit name=value (repeatable)` / `  --integer-division euclidean\|floor\|truncating` / `  --extensions <file>` / `  --format text\|json`, each line ending in a newline; the same `check options:` section appears in root help. The `--model` and `--library` operands are FSL FR-866's `<identity>=<file>`, each repeatable, and `fsl check` refuses a bare file, an empty identity or file, and a repeated identity as `invalid-request`, exit 20, before any file is read; an identity no `model` declaration or `import` selects refuses after the source parses and before its file is read. An option the table does not mark repeatable, given twice, refuses exit 20 with the verb's usage in the selected output format on stdout and no stderr (FSL FR-767). For every other verb that executes operations, `fsl <verb> --help` writes that verb's usage lines, a blank line and its `<verb> options:` section byte-equal to root help's, exit 0. `fsl prove --help` writes FSpec FR-522's `unsupported`/`operation-not-routed` result naming `prove`, exit 21, and so do `generate` and `render`. `fsl run --format bogus --help` refuses `invalid-request`, exit 20, naming `--format` and `bogus`, and `fsl run --bogus --help` refuses `invalid-request`, exit 20, naming `--bogus`. `fsl <verb> --version` and `fsl <verb> -V` for each verb that executes operations refuse `invalid-request`, exit 20, naming the option, before any file is read. | Test (TC-049) |
| FR-002-AC-26 | `fsl run R --color always`, `fsl run R --color never` and `--color` on every other verb that executes operations refuse `invalid-request`, exit 20, naming `--color`, before `R` or any other named file is read, and no help section contains `--color`. Over inputs that contain no `0x1B` byte, no `fsl` invocation of TC-049 writes the byte `0x1B` to stdout or stderr, with or without `NO_COLOR` set. | Test (TC-049) |
| FR-002-AC-27 | For FSL FR-960-AC-1's native-run request `Q` whose call carries `requirements`, `fsl run Q --format json` writes FR-002-AC-22's document (value `{"integer": "16"}`, three settled requirement entries), exit 0. `fsl run Q --limit s1.nodes=50000 --format json` writes the same document. FSL's provider run entry carries stage limits only, so `fsl run Q --limit accounting.work_units=1000000` and `fsl run Q --limit observation.input_bytes=1000000` each refuse `invalid-request`, exit 20, naming the setting and `requirements`, and run nothing. `fsl run Q --limit s1.nodes=1 --format json` writes FSL's typed `incomplete` outcome naming `s1.nodes` and configured value 1, exits 22, and settles no requirement item, and in text mode its diagnostic names the authored source locus, not FSL's internal identity; a driver that runs the provider call under default spine limits fails this criterion. | Test (TC-046) |
| FR-002-AC-28 | `fsl prove R --integer-division floor --format json` and `fsl prove R --bogus --format json` refuse `invalid-request`, exit 20, as the only command result written to stdout: one `invalid-request` outcome document with operation `prove` naming the option, and no stderr output, before the verb answers unsupported; in text mode the same refusals write only their selected text outcome to stdout. `fsl prove R --format json`, `fsl prove --help` and `fsl prove --version` answer unsupported, exit 21 (AC-25 governs help and version on a verb the build does not route). | Test (TC-049) |
| FR-002-AC-29 | `fsl not-a-verb`, `fsl not-a-verb --format json` and a first argument that is not valid UTF-8 each write one diagnostic line to stderr, nothing to stdout, and exit 20; no outcome document is written. | Test (TC-049) |

## Dependencies

- FSL ADR-029 CB-1, CB-4 and CB-5; FSL FR-285, FR-286 and FR-287.
- FSL FR-298 defines typed rendering and `render_outcome`; FSL FR-865
  defines the text form of an outcome document and the driver's default
  text mode; FSpec FR-718 makes text the CLI's default output.
- FSpec FR-300 and FR-301 for the CLI output and process contract.
- [ix-cli-kit FR-017](ix://agent-ix/ix-cli-kit/FR-017) defines the caller-owned
  exit-code pass-through; FSL ADR-029 remains authoritative for category
  outcomes and codes. IR-607 adopts the exit module first and retains the
  remaining required kit mechanics as open work.
- [ix-cli-kit FR-008](ix://agent-ix/ix-cli-kit/FR-008) defines the built-binary
  version agreement used by AC-12.
- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md).
- [TC-018](../test-cases/TC-018-binary-process-boundary.md), adding verification for AC-2 through AC-4 and AC-6.
- [TC-040](../test-cases/TC-040-write-text-outcomes-by-default.md), verifying AC-13 and AC-18.
- [TC-043](../test-cases/TC-043-name-the-checked-source-by-the-supplied-path.md), verifying AC-19.
- FSL FR-866 defines the identity `check_outcome` gives the checked source.
- FSL FR-931 defines the `tl_lowering` request, its record and
  `PreparedLowering::clause_request_mut`; FSL FR-1039 defines the package run.
  [TC-046](../test-cases/TC-046-run-tl-lowering-and-package-requests.md)
  verifies AC-20, AC-21 and AC-22; FSL FR-960 defines `call.requirements`.
- FSL FR-665 defines `inspect_handoff` and the handoff directory; FSpec FR-301
  gives `inspect --handoff` its exit contract and FSpec FR-522 the not-routed
  refusal. [TC-048](../test-cases/TC-048-verify-a-checked-package-handoff-with-fsl-inspect.md)
  verifies AC-23.
- FSpec FR-562 defines `--extensions` on `parse` and `check`. The
  filament-docs CLI reference generator reads root help's `Commands:`,
  `Source operands:` and `Options:` sections and the `Recognized;` prefix,
  which is why AC-24 keeps that layout and adds the per-verb sections after
  it.
- [AD-001](../assurance/AD-001-driver-architecture.md) plans ix-cli-kit
  `streams` adoption, whose colour handling would arrive under its own
  requirement with a colour option; AC-26 governs until then.
  [TC-049](../test-cases/TC-049-list-every-accepted-option-in-help.md)
  verifies AC-24, AC-25 and AC-26.

AC-14 adopts FSL FR-287-AC-8's quiet behavior only when a primary stdout write
or flush actually reports `BrokenPipe`. A text-mode write of the rendered
outcome is a primary stdout write and follows the same rule (FSL
FR-287-AC-8). It adds no file-descriptor probe or replacement refusal
envelope.
