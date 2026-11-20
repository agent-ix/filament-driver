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
