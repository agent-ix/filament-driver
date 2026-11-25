---
id: TC-051
title: "Settle temporal SMT results with checker evidence through the driver"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-009
    type: verifies
---
# TC-051: Settle temporal SMT results with checker evidence through the driver

## Description

Verify that the driver's temporal SMT path forwards a certificate-bearing
codegen outcome to FSL's production settlement writer and emits the accepted
checker result through the driver CLI.

## Test Procedure

Invoke the driver CLI on the temporal SMT example using its existing source and
request files at their repository paths. Do not copy either file. The `Counter`
item carries FR-314-AC-1's accepted `BoundedComplete{depth: 3}` certificate.
Read the emitted outcome document and inspect that item's record and certificate.

## Expected Results

The CLI emits the outcome document produced by FSL's writer. The `Counter` item
is `proved`, its record certification is `certified`, and its certificate has
`checker` `smt-proof` and `accepted` `true`, matching FR-314's `Verified` result.
The source and request files used by the invocation are the existing repository
files; the test creates no copied example files.
