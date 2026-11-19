---
id: TC-046
title: "Run TL lowering and package requests with the fsl binary"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-002
    type: verifies
---
# TC-046: Run TL lowering and package requests with the fsl binary

## Description

Verify that `fsl run` routes a `tl_lowering` request (FSL FR-931) and a
package-selected request (FSL FR-1039) through FSL's executors, writes the
document FSL's in-process `run` writes, and applies `--limit` to the lowering.
Scope: FR-002-AC-20, FR-002-AC-21.

## Test Procedure

1. Write FSL FR-931-AC-1's `Low` unit, the ConfigVersion model and the request
   `R` with `tl_lowering` `{"clause": "Low"}` into a per-test folder.
2. Run `fsl run R --format json`; compare stdout with the canonical encoding
   of `fsl_command::command::run(R)` plus a newline, and read the record.
3. Run `fsl run R --limit s1.nodes=1 --format json`.
4. Run `fsl run R --limit s1.nodes=50000 --format json`.
5. Compile FSL FR-1039-AC-1's `half` unit with `integer_division` `floor`,
   write its checked package, remove the source, and write the request `P`
   selecting it as `package` `{file}` with the `call` `half`; run
   `fsl run P --format json` and compare with FSL's `run(P)`.
6. Add `integer_division` to `P` and run it again.

Test: `fsl-cli/tests/cli_contract.rs` `run_routes_tl_lowering_and_package_requests`.

## Expected Results

- Step 2: `disposition` `supported`, `correspondence` `mltl.closed-trace/v1`,
  `formula` `G[0,3] p0`, exit 0, byte-equal to FSL's document; no input
  refusal. Fails today: the driver refuses the route as "not run by the driver
  yet".
- Step 3: FSL's typed limit outcome naming `s1.nodes`, configured value 1,
  category `incomplete`, no lowering record, exit 22.
- Step 4: byte-equal to step 2, exit 0.
- Step 5: the call completes with `-4`, exit 0, byte-equal to FSL's document.
- Step 6: `invalid-request` naming `integer_division`, exit 20.
