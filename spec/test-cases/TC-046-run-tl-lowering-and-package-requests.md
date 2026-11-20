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
Also verify that a run settling requirement items registers FSL's reference
hybrid provider and passes caller spine limits to the provider-backed run
entry. Scope: FR-002-AC-20, FR-002-AC-21, FR-002-AC-22, FR-002-AC-27.

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
7. Write FSL FR-960-AC-1's request `Q` calling `sq` with `x = 3` and
   `call.requirements` `{}`; run `fsl run Q --format json` and compare with
   `run_with_providers_and_limits_and_overrides(Q)` given the providers
   `[Engine::hybrid_reference()]` and default limits.
8. Run `Q` with a `--provider-config` starting a fixture provider whose
   `hello` names the backend `fsl-hybrid-reference`.
9. Run `Q` without `--limit`, with `--limit s1.nodes=50000`, and with
   `--limit s1.nodes=1`; compare the first two documents and inspect the
   third outcome and its settled requirement items.

Test: `fsl-cli/tests/cli_contract.rs` `run_routes_requirement_items_with_caller_spine_limits`.

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
- Step 7: byte-equal to FSL's document, value `{"integer": "16"}`, three
  settled requirement entries, exit 0.
- Step 8: `invalid-request` naming `providers`, exit 20. Fails today: the
  driver registers no reference provider, so the fixture is admitted.
- Step 9: the default and `s1.nodes=50000` documents are byte-equal and
  settle three requirement items; `s1.nodes=1` writes FSL's typed
  `incomplete` outcome naming the setting and configured value 1, exit 22,
  with no requirement item settled. A provider call that uses default spine
  limits fails this step.
