# FSL CLI contracts

This crate tests an explicitly selected FSL executable through real subprocesses. It has its own Cargo workspace and lockfile, so testing the process boundary does not build the driver, engines or code generators.

From the repository root:

```sh
make cli-contracts FSL=/absolute/path/to/fsl
```

`FSL` is mandatory. Select the binary from the build under test; the harness does not find a different binary or build one.

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
