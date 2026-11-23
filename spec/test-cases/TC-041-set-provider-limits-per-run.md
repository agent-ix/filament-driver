---
id: TC-041
title: "Set the provider limits once per run with --provider-limit"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-006
    type: verifies
  - target: ix://agent-ix/filament-driver/FR-014
    type: verifies
---
# TC-041: Set the provider limits once per run with --provider-limit

## Description

Verify that `fsl run` takes its provider limits only from `--provider-limit`,
with FSL FR-287's names and defaults, applies one limit set to every provider
of the run, reads no limit from the plugins-only `--provider-config`
document, and refuses a malformed limit before any plugin starts.

Scope: FR-006-AC-8, FR-006-AC-24, FR-006-AC-28, FR-006-AC-29, FR-014-AC-6.
Ticket: QSL-2335.

## Test Procedure

1. Write a `filament.provider-config/v1` document naming two recording
   fixture plugins by paths relative to the document, one with `arguments`
   and one without. Run `fsl run --provider-config providers.json` with no
   `--provider-limit`; assert each request the fixtures record carries
   `deadline_ms` 600000 and that a 67108865-byte frame is refused as over the
   67108864-byte default (AC-8, AC-28).
2. Repeat with `--provider-limit deadline_ms=250 --provider-limit
   max_frame_bytes=1048576`; assert both fixtures receive `deadline_ms` 250,
   and that a 1048577-byte result frame settles the item `failed` with the
   frame-length tool failure naming limit 1048576 and value 1048577 (AC-8,
   AC-28).
3. Run with `--provider-limit deadline_ms=250` and no `--provider-config`;
   assert it is accepted and starts no plugin (AC-28, FR-014-AC-6).
4. Run each refused `--provider-limit` argument of FR-006-AC-29 under both
   `--format text` and `--format json`; assert `invalid-request` naming
   `--provider-limit` and the argument, exit 20, the refusal outcome on
   stdout in the selected format, empty stderr, and no fixture start marker.
   Run `fsl parse` with `--provider-limit` and with `--provider-config` under
   both formats and assert each unknown-option refusal follows the same
   output rule (AC-29).
5. Run configuration documents whose entry holds a `deadline_ms` or a
   `limits` member, a document with no `format`, a document with `format`
   `filament.provider-config/v2`, an entry with a blank `path`, and
   `arguments` holding a number; assert each refusal FR-006-AC-24 states,
   exit 20, the refusal outcome on stdout in the selected format, empty
   stderr, and no fixture start marker (AC-24). With a malformed config and
   `--format bogus`, assert the invalid-format refusal is rendered as text on
   stdout and no stderr is written, before reading the config or starting a
   plugin.

Test location: `fsl-cli/tests/binary_process.rs` (production binary) and the
`fsl-cli/src/cli.rs` configured-run composition tests.

## Expected Results

- The run's limits come only from `--provider-limit`, with FSL FR-287's
  defaults, and every provider of the run receives the same set.
- The configuration document lists plugins only; a deadline or limit member
  in it is an unknown member.
- Each malformed limit or configuration refuses with exit 20 and its outcome
  on stdout in the selected output format, with no stderr, before any plugin
  starts. If output mode cannot be selected, it uses the text rendering.
