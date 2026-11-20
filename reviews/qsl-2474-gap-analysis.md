---
id: SR-1458
title: "Gap analysis of export CLI contracts"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver QSL-2474 PR203 9b06a773f238459757f4ce129728f899aea2b23a; cli-contracts/tests/export.rs; cli-contracts/tests/export_cases/{mod,intake,options,staged}.rs; Driver FR-027; FSL FR-1015 and FR-1039"
review_set: subset
---

## Summary

Compared the new export contracts with the driver's export behavior and the locked FSL export request and package-reference admission rules. Three reachable admission assertions remain missing; known blocked semantic successes remain correctly pending.

## Verdict

PASS — the three findings are addressed at 214968cf32a18301143a158eee510eaeabb1c036. Package admission remains a strict pending product oracle under QSL-2672, not runnable coverage; the original coder owns the single final component gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The documented missing-path refusal for --output has no test: current output cases always supply a path, so a parser accepting a trailing --output or reading the request first would escape this slice. Add a real invocation with missing request and trailing --output requiring exit 20, the option-naming diagnostic and no output. | cli-contracts/tests/export_cases/options.rs:98; spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md:78 |
| FND-002 | medium | Evidence conflict cases assert only --evidence in the diagnostic, while Driver FR-027 Behavior requires naming the other conflicting argument as well. A diagnostic omitting --output or the positional request currently passes. Assert both argument names in each existing case. | cli-contracts/tests/export_cases/options.rs:89; spec/functional/FR-027-write-a-tla-or-lean-export-with-fsl-export.md:80 |
| FND-003 | medium | Package export coverage is entirely ignored behind assembly QSL-1979, but FSL FR-1039 source_package admission must reject an absent, mistyped, empty or unknown reference member before opening any selected file. Add a table of those real malformed package requests against missing package paths, asserting exit 20 and the prescribed invalid_runtime_input cause/path; this admission does not require model export assembly. | cli-contracts/tests/export_cases/staged.rs:172; FSL FR-1039:The source package reference of a package export |

## Examined criteria

- Driver FR-027 Description and Behavior: separate export/evidence dispatch, option/request destination equivalence, existing-path preservation, conflicting or missing options and text rendering. AC-1 through AC-5 were examined against staged and active cases.
- Driver FR-002 AC-1, AC-13, AC-18, AC-24, AC-25 and AC-26: typed command routing, output mode context, foreign options and help. AC-13/18 enumerate other verbs; export text is directly owned by FR-027 Description/Behavior rather than expanding that enumeration.
- Driver FR-003 AC-2, AC-3, AC-7 and AC-8: unknown/malformed/duplicate limit admission and configured stage exhaustion.
- Locked FSL FR-1015 Behavior and AC-2, AC-6 and AC-9: closed export members, limit conflicts, ordered invalid obligations, precise invalid_runtime_input cause/path. The remaining FR-1015 text was context for export result and pending assembly assertions.
- Locked FSL FR-1039, The source package reference of a package export: required reference, exact retained identity/version and closed typed admission before selected-file I/O.
- Locked FSL FR-026 Behavior and FR-286 AC-23 were context for duplicate intake, read-once and the pending output failure oracle.

## Coverage

Plan completion: not assessed

This is the user-authorized diff-scoped gap analysis, including reverse ownership, stubs, test intent and coverage inflation checks for the five changed files. Existing matrix measurements are accepted; matrix recomputation and repository-wide inventory are explicitly out of scope. No tests, builds or product probes ran. Known pending cases are not counted as coverage, and real target/evidence success is not claimed. The final component gate remains with the original coder.

The authoritative FSL export rules were read from the locked dependency at /home/peter/.cargo/git/checkouts/filament-spec-language-1ba4ce0e71f23cf3/8877b38/spec/functional, because the separate local language clone lacks the later export requirements. No old-cohort result was used to claim a new product bug.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 adds a real missing-request invocation with trailing --output, requires the option-naming refusal before I/O, and compares the owned directory entries before and after. Coder reports the focused new test passed; its result was accepted without rerun. |
| FND-002 | fixed | Each conflict now asserts --evidence and the other argument role: --output or request file. The criterion does not require the literal positional pathname. The strengthened assertions passed according to the coder's accepted focused receipt. |
| FND-003 | fixed | A complete malformed source_package table asserts missing references/members, scalar/null/array types, nonstring identity/version, empty strings and unknown members before opening a missing selected package. Exact invalid_runtime_input causes, member paths, no result and no target remain strict. The actual early package-decoding defect has one owner QSL-2672 with blocks2474; the table is ignored under that owner and is not counted as coverage. The package assembly test names both admission QSL-2672 and subsequent QSL-1979. No known failure was rerun. |

Round 1 disposition inspected only the original-to-fix changed lines and accepted the focused and scoped-Clippy reports. No new findings, product probes, test runs or builds were introduced. Current totals are 16 active and nine strict pending cases.
