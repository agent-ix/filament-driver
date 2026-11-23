---
id: TC-050
title: "Print a request example and name every request defect"
type: TC
relationships:
  - target: ix://agent-ix/filament-driver/FR-028
    type: verifies
---
# TC-050: Print a request example and name every request defect

## Description

Verify that each request verb prints a valid minimal request with `--example`, refuses misuse of that option, and that FSL's request refusals reach the user listing every missing member in `details.missing`, and that the driver appends the export hint outside FSL's document. Scope: FR-028-AC-1 to FR-028-AC-9 (AC-6 is withdrawn).

## Test Procedure

1. For each of `run`, `compile`, `analyze`, `monitor` and `replay`, run `fsl <verb> --example`, then with `--format json` and `--format text`; capture stdout, stderr and the exit status.
2. Run `fsl export --example`, `fsl export --example-evidence` and `fsl export --example --example-evidence`.
3. In an empty directory save each example to a file and give it to its verb.
4. Run `fsl run --example R`, `fsl run --example --limit s1.nodes=1`, `fsl compile --example --output m.pkg`, `fsl run --example --help`, `fsl run --example --format bogus`, `fsl run --example-evidence`, and `--example` on `check`, `parse`, `format` and `inspect`; read each request verb's help and root help for `--example` (and `--example-evidence` on `export`).
5. Run `fsl run` over `{"format":"native-run/1","request":{}}`, `{}` and `{"format":"native-run/1"}`; run `fsl monitor` over `{"format":"native-monitor/1","request":{}}`, then over FR-644-AC-17's otherwise valid request with an unknown member inside `target`.
6. Run `fsl export` and `fsl export --evidence` over a `native-run/1` request without the member, in text and JSON mode; run `fsl run` over a `native-analyze/1` request, `fsl analyze` and `fsl compile` over a `native-run/1` request.
7. Compare each refusal of steps 5 and 6 with the document FSL's own entry returns, in both output modes.

Test: `fsl-cli/tests/request_discovery.rs`.

## Expected Results

- Step 1: exit 0, empty stderr, stdout equal to FSL's example plus a newline, object with `format` and `request`, no outcome document; the three output modes give equal bytes.
- Step 2: the `export` and `export_evidence` members as stated; the third refuses exit 20 naming both options.
- Step 3: each is refused at stage `file` or succeeds, never at `envelope` or `request`.
- Step 4: `fsl run --example-evidence` refuses naming the unknown option; help and root help list `--example` on each request verb and `--example-evidence` on `export`; exit 20 naming `--example` and the other argument with no file read; help exit 0; `--format` refusal first; unknown option on the four non-request verbs; `--example` listed on each request verb.
- Step 5: each `details.missing` equals the array derived from that verb's `--example` output (a one-of group as an array of pointers), with no line or column.
- Step 6: `details.missing` `["/request/export"]` and `["/request/export_evidence"]`, no message containing `requires export` or `--example`; the `hint:` last line in text mode as FR-028-AC-7 states and no other refusal-line hint, and in JSON mode FSL's document byte-equal with no hint; the format refusals carry `details` `path` and `accepted` as FR-028-AC-9 states.
- Step 7: documents byte-equal to FSL's in JSON mode and in text mode except for the hint line, exit 20, stdout only. The invalid-target monitor case carries FSL's `details.path` `/request/target` and exact message unchanged in both formats, with no hint line.
