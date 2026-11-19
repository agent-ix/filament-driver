---
id: TC-047
title: "Decide hybrid claims with FSL's reference provider through fsl analyze"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-022
    type: verifies
---
# TC-047: Decide hybrid claims with FSL's reference provider through fsl analyze

## Description

Verify that `fsl analyze` always registers FSL's reference hybrid provider, so
a hybrid claim gets FSL's own verdict with no external provider installed, and
that a configured external hybrid provider still routes when an item names it.
Scope: FR-022-AC-11.

## Test Procedure

1. Copy FSL's `examples/hybrid-braking` folder (`braking.native`,
   `slow-enough.analyze.json`, `too-slow.analyze.json`) into a per-test folder.
2. Run `fsl analyze slow-enough.analyze.json --format json` with no
   `--provider-config`.
3. Run `fsl analyze too-slow.analyze.json --format json` with no
   `--provider-config`.
4. Depends on FSL's `Engine::hybrid_process` (FSL FR-750; tracked by QSL-2532);
   ignored until it lands. Write a `--provider-config` starting a fixture
   process provider whose `hello` advertises (`temporal-satisfaction`,
   `unbounded`) with `dynamics` `["hybrid"]` and that answers `delta-complete`
   `unsat` covering horizon 10. Run three variants of the slow-enough request
   with it: the item naming the fixture's backend, the item naming
   `fsl-hybrid-reference`, and the item naming no backend. Then run the
   request with a fixture whose `hello` advertises only
   (`temporal-satisfaction`, `bounded`).

Test: `fsl-cli/tests/cli_contract.rs`
`analyze_decides_hybrid_claims_with_the_reference_provider`.

## Expected Results

- Step 2: one item `proved`, certification `certified`, backend
  `fsl-hybrid-reference`, category `success`, exit 0; `registry` lists
  `fsl-hybrid-reference` and `en-1`. Fails today: the driver registers only
  `Engines::fsl()`, so the item settles `unsupported`, `HybridProviderAbsent`,
  exit 21.
- Step 3: one item `refuted`, backend `fsl-hybrid-reference`, category
  `violation`, exit 10.
- Step 4: the fixture-named item is sent to the provider as one FR-447 request
  and settles `proved`, `bounded-solver`, `uncertified`, naming the fixture
  backend, exit 0, with no `ambiguous-backend`;
  the reference-named item settles `proved`, `certified`, exit 0; the unnamed
  item settles `invalid-request`, `invalid_capability`/`ambiguous-backend`,
  naming both backends, exit 20; the `bounded`-only fixture refuses with exit
  20 before any item runs.
