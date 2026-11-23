# FSL CLI contracts

This crate tests an explicitly selected FSL executable through real subprocesses. It has its own Cargo workspace and lockfile, so testing the process boundary does not build the driver, engines or code generators.

From the repository root:

```sh
make cli-contracts FSL=/absolute/path/to/fsl
```

`FSL` is mandatory. Select the binary from the build under test; the harness does not find a different binary or build one.

The build under test is one directory holding `fsl`, `fsl-plugin-launch` (the process-provider launcher, which `fsl` finds beside itself) and `fsl-test-provider`. `make cli-contracts` passes `FSL_TEST_PROVIDER` as `fsl-test-provider` beside `FSL` unless the variable is set, so the analyze, monitor and exit-code provider cases run.

Each `CliCase` owns a temporary working directory. Write original inputs with `write`, then use `run` or `run_stdin` to execute the public command. `Captured` preserves the actual process status and raw stdout/stderr; `json` requires one JSON value and `json_lines` decodes a streaming command. Assert the command's semantic fields independently of the helper.

The examples cover a valid source and a result-type refusal, text versus machine output, raw argument-error diagnostics, and holding/violating observation streams over both files and stdin. A larger stream also exercises an early request refusal without losing its output to a closed stdin pipe.

On Linux and macOS, the 30-second execution budget covers the child and all pipe I/O, with a 1 MiB capture bound per stream. Nonblocking pipes are polled in one thread, so no blocked capture worker can outlive the budget. On timeout the harness closes its pipes, kills only its invocation process group, and reaps the direct child. The temporary working directory is removed when the case drops.

Run only the relevant group while authoring:

```sh
FSL=/absolute/path/to/fsl CARGO_TARGET_DIR="$PWD/target" \
  cargo test --locked --manifest-path cli-contracts/Cargo.toml \
  --test contracts file_contracts:: -- --test-threads=1
```

## Check contracts

The `check` integration target adds original native files, source-byte boundaries, model and library selections, division laws, extension sets and Markdown fences. It asserts actual typed outcomes and semantic controls: a renamed model type must stop resolving, an ill-typed library body must fail, and a refused Markdown fence must not stop later fences. Stdin tests verify that the named source file stays authoritative.

Run one new family during authoring:

```sh
FSL=/absolute/path/to/fsl CARGO_TARGET_DIR="$PWD/target" \
  cargo test --locked --manifest-path cli-contracts/Cargo.toml \
  --test check check_cases::dependencies:: -- --test-threads=1
```

Use the delivered CLI build that accepts identity-qualified model/library options. Model documents are original caller-owned semantic-IR inputs. The test executes the public check with them.

The strict model/library-only dependent-byte boundary case is ignored under QSL-2657: the current check incorrectly charges the primary source to that ceiling. Its expected success and exact library-byte boundary remain executable required oracles, not passing coverage. The actionable invalid-format diagnostic work remains QSL-2642. Native model-package authoring remains with QSL-1980.

## Analyze contracts

The `analyze` target adds an original finite queue workflow, integer value
validity, timed windows and unbounded timed LTL. It checks real proof trust,
current counterexamples and replay, request-relative inputs, stage-specific
refusals, budgets, options and machine/text rendering. A checked handoff is
produced through public compile and analyzed after the source becomes invalid;
corrupting its identity sidecar must refuse admission.

Process-provider cases use the repository's authoritative `fsl-test-provider`
executable from the same locked product build as `FSL`. Select it explicitly;
the contract workspace does not build or substitute either executable:

```sh
FSL=/absolute/path/to/fsl FSL_TEST_PROVIDER=/absolute/path/to/fsl-test-provider \
  CARGO_TARGET_DIR="$PWD/cli-contracts/target" \
  cargo test --locked --manifest-path cli-contracts/Cargo.toml \
  --test analyze analyze_cases::providers:: -- --test-threads=1
```

The provider fixture exercises transport, trust labels and signal reporting.
A public native replay must reject its false candidate assignment and accept
its actual violation. Its `trusted` proof is distinct from a core-certified
proof. SIGINT/SIGTERM are sent only by the owned provider to its FSL parent
after it receives a request; the existing harness bounds and reaps the process.

Two required oracles remain ignored and uncounted: QSL-2662 owns oversized
analyze admission returning incomplete/22 instead of refusal/20; QSL-2234 owns
the missing default hybrid reference provider. The hybrid source preserves
its required proof and reproduced counterexample rather than accepting the
current unsupported outcome.

## Inspect contracts

The `inspect` target generates an original handoff through public compile and
verifies it with `inspect --handoff`. It asserts JSON/text outcomes and confirms
that inspection preserves the package, selection and selected source bytes.
Changed package bytes and node counts must refuse through the strict reader
and selection comparison. Missing, empty, malformed, non-UTF8 and oversized
artifact files, escaping source labels, directories and symlinks have concrete
refusal checks.

```sh
FSL=/absolute/path/to/fsl CARGO_TARGET_DIR="$PWD/cli-contracts/target" \
  cargo test --locked --manifest-path cli-contracts/Cargo.toml \
  --test inspect inspect_cases:: -- --test-threads=1
```

The directory operand remains authoritative when stdin contains other bytes.
The supported handoff form accepts text and JSON; `dot` with `--handoff`
refuses, and the separate bare-path form remains explicitly unsupported.
