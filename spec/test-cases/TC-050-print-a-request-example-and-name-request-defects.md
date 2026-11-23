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

Verify that each request verb prints a valid minimal request with `--example`, refuses misuse of that option, and that FSL's request refusals reach the user listing every missing member, the accepted formats and the export request form. Scope: FR-028-AC-1 to FR-028-AC-9.

## Test Procedure

1. For each of `run`, `compile`, `analyze`, `monitor` and `replay`, run `fsl <verb> --example`, then with `--format json` and `--format text`; capture stdout, stderr and the exit status.
2. Run `fsl export --example`, `fsl export --evidence --example` and `fsl export --example --evidence`.
3. In an empty directory save each example to a file and give it to its verb.
4. Run `fsl run --example R`, `fsl run --example --limit s1.nodes=1`, `fsl compile --example --output m.pkg`, `fsl run --example --help`, `fsl run --example --format bogus`, and `--example` on `check`, `parse`, `format` and `inspect`; read each verb's help for `--example`.
5. Run `fsl run` over `{"format":"native-run/1","request":{}}`, `{}`, `{"format":"native-run/1"}` and `{"format":"native-run/1","request":{"models":"x"}}`; run `fsl monitor` over `{"format":"native-monitor/1","request":{}}`.
6. Run each request verb over a request of a format it does not read, and `fsl export` over `{"format":"native-export/1","request":{}}`.
7. Run `fsl export` and `fsl export --evidence` over a `native-run/1` request without the member.
8. Compare each refusal of steps 5 to 7 with the document FSL's own entry returns, in both output modes.

Test: `fsl-cli/tests/request_discovery.rs`.

## Expected Results

- Step 1: exit 0, empty stderr, stdout equal to FSL's example plus a newline, object with `format` and `request`, no outcome document; the three output modes give equal bytes.
- Step 2: the `export` and `export_evidence` members as stated; both flag orders give equal bytes.
- Step 3: each is refused at stage `file` or succeeds, never at `envelope` or `request`.
- Step 4: exit 20 naming `--example` and the other argument with no file read; help exit 0; `--format` refusal first; unknown option on the four non-request verbs; `--example` listed on each request verb.
- Step 5: the exact messages of FR-028-AC-5, and a message containing `/request/models` for the wrong-typed member.
- Step 6: `unsupported native command format "<format>"; this verb reads <accepted>`, exit 20, for every pairing.
- Step 7: the two export messages of FR-028-AC-8, none containing `requires export`.
- Step 8: byte-equal documents, exit 20, stdout only.
