---
id: TC-044
title: "Route fsl monitor request items to the configured providers"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-025
    type: verifies
---
# TC-044: Route fsl monitor request items to the configured providers

## Description

Verify that `fsl monitor` accepts `--provider-config` and `--provider-limit`,
starts the configured plugin, and has FSL route the request's `items` to it
while the clauses stay with the monitor session. Scope: FR-025-AC-4.

## Test Procedure

1. Write the monitor fixture unit with `function Even using v(n: Integer):
   Boolean pure { n * 2 - n = n }`, a request over `Reaches` and `Never`
   with the `value-validity` item `Even` on `fsl.test-recording-provider`,
   the same request without `items`, records 0, 1, 2, 1, 0 and a provider
   configuration starting `fsl-test-provider` answering `proved`
   (`fsl-cli/tests/monitor_cli.rs`).
2. Run `fsl monitor` with `--provider-config`, `--input` and
   `--format json`; run the request without `items` with no
   `--provider-config`.
3. Repeat step 2's first run with `--provider-limit deadline_ms=4096`.
4. Run the request without `items` with `--provider-config`.

## Expected Results

- Step 2: exit 10; the plugin received a request; the event lines equal
  the run without `items`; the summary's `items` are `proved` `trusted` on
  the test backend and its `registry` names it.
- Step 3: the plugin's request carries `deadline_ms` 4096.
- Step 4: the plugin receives no request and the lines equal the run
  without the option.
- No run refuses either option or names QSL-2132.
