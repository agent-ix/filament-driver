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

Verify that the driver's temporal SMT path forwards codegen's typed outcome to
FSL's production settlement writer and emits the FR-621 checker evidence through
the driver CLI.

## Test Procedure

Invoke the driver CLI on the temporal SMT example using its existing source and
request files at their repository paths. Do not copy either file. Read the
emitted outcome document and inspect the settled item for the checker member.

## Expected Results

The CLI emits the outcome document produced by FSL's writer, and the settled item
contains FR-621 `CertificateChecker::SmtProof` evidence. The source and request
files used by the invocation are the existing repository files; the test creates
no copied example files.
