---
id: TC-009
title: "Exercise CLI verbs, outcome documents and exit codes"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: verifies
  - target: ix://agent-ix/filament-driver/FR-014
    type: verifies
---
# TC-009: Exercise CLI verbs, outcome documents and exit codes

## Description

Verify that the driver CLI exposes the specified verbs, preserves typed
outcomes in `filament-outcome/1` documents and uses FSL's single exit function.

Scope: FR-002-AC-1 through FR-002-AC-6 and FR-002-AC-8 through FR-002-AC-12,
FR-002-AC-17, plus FR-014-AC-4. TC-040 verifies FR-002-AC-13 and FR-002-AC-18.

## Test Procedure

1. Invoke every listed verb with its valid format option, including `inspect`
   with `dot`.
2. Invoke an unknown verb, option, profile and provider; capture stdout,
   stderr, exit status and whether a lifecycle stage ran.
3. Exercise representative outcomes for every O-16 category, including
   `undefined`, a pending monitor clause, an inconclusive proof and a
   multi-item outcome containing multiple exit codes. Compare each process
   status to FSL's category exit function.
4. Inspect the driver boundary and verify it passes FSL's category exit result
   to `ix_cli_kit::exit::caller_exit_code` without routing through the kit's
   `Outcome`. Exercise all six current FSL category exit values (`0`, `10`,
   `20`, `21`, `22`, `30`) through the driver path and compare the process
   result with FSL's function. Do not repeat the kit's own exhaustive FR-017
   test over all `u8` values. For the code-5 fallback, verify the diagnostic
   and stdout behavior specified by FR-002-AC-6 and TC-018.
5. Run one operation in JSON mode and compare the serialized value with the
   typed library outcome. Repeat with stdout/stderr write failure and a broken
   pipe.
6. Run from a directory containing an unnamed configuration file and verify
   that the CLI does not read it.
7. Invoke `fsl --version` and verify that it reports the semantic version
   declared by the `fsl-cli` Cargo package. Run
   `ix_cli_kit::version::Agreement::command` on the built `fsl` executable
   with `--version`, paired with the `CARGO_PKG_VERSION` surface. Require the
   agreement check to pass.
8. Inspect the CLI output paths and verify results and diagnostics are written
   through ix-cli-kit's `streams` module. Exercise its stdout/stderr selection,
   write-failure and broken-pipe behavior through the CLI.
9. Inspect machine-mode serialization and verify the CLI calls ix-cli-kit's
   `json` module to encode the unchanged FSL `filament-outcome/1` value.
10. Invoke format with omitted output format and verify it writes
    FSL-formatted source. Default text output of parse, check, compile and
    run is verified by TC-040.

11. From an empty working directory, invoke the built `fsl` binary with no
    arguments and separately with each of `--help`, `-h` and `help` as its
    sole argument. Capture stdout, stderr and process status. Verify the help
    content specified by FR-002-AC-17, including the distinction between
    executable verbs and recognized verbs this build does not route and that return `unsupported_operation`
    results. Compare the advertised commands and format restrictions with the
    parser and dispatch surface; verify that these invocations do not read
    source input or dispatch a lifecycle operation.

Test location: `fsl-cli/tests/cli_contract.rs`; CLI-specific test fixtures
reside under `fsl-cli/tests/fixtures/`. The parser and command construction
under test are owned by `fsl-cli/src/cli.rs`.

## Expected Results

- Valid verbs and format options reach their corresponding operations.
- Invalid arguments are refused before any stage with FSL's refusal category
  and name the refused argument.
- Each normal category and multi-item result exits with the exact code selected
  by FSL's single exit function; no second mapping changes the result. The
  reserved-code invariant path follows FR-002-AC-6 and does not write the
  result to stdout.
- Exit conversion uses `ix_cli_kit::exit::caller_exit_code` without mapping
  FSL categories through the kit's typed `Outcome` values; the driver tests
  only the FSL category outputs, not the kit's entire `u8` surface. The code-5
  behavior is specified by AC-6 and checked at the process boundary.
- The parser remains owned by `fsl-cli`. Stream, JSON, conditional config,
  and version behavior have separate criteria under IR-607. This case verifies
  the stream and JSON module call sites and behavior, plus the package-version
  output and agreement check.
- JSON mode emits one FSL-defined `filament-outcome/1` value equal to the typed
  library outcome. A stream write failure returns the internal-failure
  category through FSL's exit function; a broken pipe ends quietly with the
  outcome's code.
- An unnamed configuration file is not read.
- Format writes FSL-formatted source in text mode.
- The version output reports the `fsl-cli` package semver; the kit's
  built-binary agreement check compares it with `CARGO_PKG_VERSION` and
  passes.
- Every root help invocation writes informative help to stdout, exits with
  FSL's success result and writes no stderr. Its command and output guidance
  agrees with the parser and dispatch behavior; no source input is required
  and no lifecycle operation starts.
