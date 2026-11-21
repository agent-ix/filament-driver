---
id: SR-202611210022442313144108510
title: "Driver repository traceability during QSL-2378 review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-driver; spec/, all Rust source and tests; changed process resource behavior inspected directly"
review_set: subset
relationships:
  - {target: "ix://agent-ix/filament-driver", type: references}
---

## Summary

Repository-driven audit of computed criterion coverage, Rust source ownership and stubs during the QSL-2378 process-limit review. Existing coverage debt remains, and the new tests' comment markers do not bind under the active traceability model. Plan completion was not assessed.

## Verdict

FAIL for repository assurance: live criteria are untagged or ignored-only, and whole-repository reverse ownership is not established. The code review distinguishes the actionable PR findings from baseline debt. No full build or test gate ran.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed matrix reports 177 untagged criteria and five criteria bound only to ignored tests across the repository. | quoin matrix; FR-001-AC-2, FR-001-AC-3, FR-001-AC-4 |
| FND-002 | medium | All five new resource tests have comment-only criterion mentions that do not bind. Coverage reports seven unmatched mentions and names the required rust-trace-attribute marker. | filament-plugin-host/tests/resource_limits.rs; fsl-cli/tests/frontend_cli.rs |
| FND-003 | medium | The coverage report has 201 untracked symbols, 905 unmatched tags and 1166 diagnostics, including uncatalogued verification methods and missing target archetypes. | quire coverage; spec/functional/FR-001-orchestrate-the-driver-steps.md |
| FND-004 | high | Whole-repository reverse ownership cannot be established from the extracted tags and direct changed-surface inspection alone. | 78 Rust files under source and test package trees |

## Coverage

Reconciliation: quoin matrix, quoin 0.36.8; quire 0.52.0, engine 0.62.2. Both matrix and coverage commands exited 0. The audit scope is the driver repository, its spec root is spec/, and its identity is ix://agent-ix/filament-driver. No plan was supplied or selected.

Plan completion: not assessed

Engine criteria counts: tagged 125; untagged 177; tagged-by-ignored-test 5; method-without-symbol 10. All 317 criteria have no run evidence in the evidence store. Static tagging is not execution evidence. The separate coverage census reports 778 Rust candidates, 747 tagged and 246 bound under rust-trace-attribute. Comment markers are specifically rejected, including the seven new resource-test mentions.

Coverage stderr also reports symbol collisions for cfg-separated launch, signal run and invalid_os_string declarations. These extraction limitations are retained rather than interpreted as missing implementation. Raw outputs are /tmp/terra-codex2-driver-221-matrix.json and /tmp/terra-codex2-driver-221-coverage.json.

The source/test inventory contains 78 Rust files across src, tests, filament-plugin-host, fsl-cli, filament-cache and fsl-test-provider. A textual todo/unimplemented/FIXME scan of these trees found no candidates. Changed launcher, resource forwarding, process-death settlement, provider fixtures and install behavior were inspected directly against FR-006, FR-013 and the owning FSL FR-287 process-budget contract. None is a placeholder or a mocked unit under test. The workload duplication and partial-frame failure are recorded in code-review; the new tests exercise real child processes but do not yet have accepted criterion bindings. Complete reverse ownership of all existing behavior was not established and no clean repository claim is made.

Semantic review: skipped. No requirements, plans, tasks or test matrix were authored or edited.

## Reviewed units

```yaml
scope:
  - {id: repository-matrix, path: spec/, role: examined, excerpt: "tagged 125; untagged 177; tagged-by-ignored-test 5; method-without-symbol 10"}
  - {id: coverage-census, path: Rust source and tests, role: examined, excerpt: "778 candidates; 747 tagged; 246 bound; rust-trace-attribute"}
  - {id: new-resource-tests, path: filament-plugin-host/tests/resource_limits.rs, role: examined, excerpt: "trace-id-outside-marker"}
  - {id: cli-resource-tests, path: fsl-cli/tests/frontend_cli.rs, role: examined, excerpt: "trace-id-outside-marker"}
  - {id: changed-source-ownership, path: filament-plugin-host/src/launch.rs, role: examined, excerpt: "Apply process resource limits before replacing this process with a provider."}
```

## Dispositions — round 1

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred: baseline outside this PR | The existing repository gaps remain; this review does not expand the resource-limit ticket to own all repository coverage. |
| FND-002 | deferred: fix round 2 | CLI integration tests now use binding bare trace attributes. Fully qualified host attributes still do not bind, leaving new FR-013-AC-16 and AC-17 untagged despite substantive tests. |
| FND-003 | deferred: baseline outside this PR | Repository extraction and old tag debt remain, with source-level limitations explicitly retained. |
| FND-004 | deferred: baseline outside this PR | The changed resource behavior was inspected directly; this review does not claim whole-repository reverse ownership. |

## Follow-up coverage — c804a1f

Reconciliation reran with the same installed quoin/quire versions. Matrix and coverage exited 0. Fresh counts are tagged 128, untagged 177, tagged-by-ignored-test 5, method-without-symbol 10. All 320 criteria have no run evidence in the store. New FR-013-AC-16 and AC-17 are untagged; AC-18 is tagged by the two macOS CLI integration tests. The fully qualified host trace attributes appear as twelve unmatched criterion mentions, while the new CLI bare attributes bind. Raw follow-up outputs are /tmp/terra-codex2-driver-221-r1-matrix.json and /tmp/terra-codex2-driver-221-r1-coverage.json.

The follow-up source owns one shared CPU/memory workload, replacing the original duplicate. No new stub was found. Whole-repository baseline findings remain outside the bounded ticket; the outstanding new marker and enforcement-oracle issues are actionable within this PR. Plan completion: not assessed. Semantic review remains skipped.

## Dispositions — round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | The added host, CLI unit and CLI integration tests use binding bare trace attributes. New FR-013-AC-16, AC-17 and AC-18 are tagged, and the resource-test unmatched-tag set is empty. |

## Final changed-scope evidence

Fresh matrix and coverage commands exited 0 at source 83cfddd46adedac5ad57f0851820d0831f0380e9. Engine counts are tagged 133, untagged 172, tagged-by-ignored-test 5 and method-without-symbol 10. No runtime evidence is bound in the store. The new Linux memory criterion binds to the ordinary-memory, partial-memory and below-limit tests; the Unix CPU criterion binds to ordinary CPU death, request-write CPU death and rounded below-limit tests; the platform criterion binds to the host diagnostic/refusal tests and the CLI presence/refusal/default-notice tests. No resource-test tag remains unmatched.

Baseline repository assurance remains FAIL under the original deferred findings; the new resource behavior's tag and enum-oracle defects are fixed. This is not a new whole-repository audit or an expansion of the ticket. Plan completion: not assessed. Raw outputs are /tmp/terra-codex2-driver-221-r2-matrix.json and /tmp/terra-codex2-driver-221-r2-coverage.json.

## Tracker record

Ticket: QSL-2378. Linear has-review was already present. All posted comments passed read-back: the returned bodies matched, their YAML parsed with the installed yaml parser, finding IDs matched the retained artifact, and every fixed disposition retained a nonempty source excerpt.

- review: a94fe36d-8787-445d-9d5f-c241cc8ca48c
- dispositions round 1: e8440879-8355-4de0-974c-b5d383584fdd
- dispositions round 2: 54a7df6d-a1d5-4bc1-8f40-3809a41f7956


## Integration custody — 7eac980

Narrow integration disposition at driver `7eac980458ba93e0566b6dcf24793d3da3b3e2f0`, rebased from reviewed `e94b181e148a7edb8b1f08f6aab1b37d46af023f` onto the normal driver cohort `6b8f2a7`. This appends custody to the existing review cycle; it is not a fresh review, matrix run or baseline measurement.

The range-diff preserves the original launcher, typed process failures, dependency, shared Rust fixtures, partial-frame exit handling and request-write/remaining-grace fixes. The platform-policy commit changes only the placement of the unchanged memory-enforcement accessor into the existing implementation alongside hybrid exchange. Both methods remain present. The sole added integration commit adds `UnsupportedPlatform` to the hybrid transport error match. Explicit unsupported memory requests are still refused by `PluginSession::start` before process launch, so that defensive match arm does not turn an admitted platform refusal into an active exchange outcome. The normal manifest and lockfile agree on FSL `f5977125981288ea1bd36fb67f71780182055aac`; the adapter retains the observed Crash/NonzeroExit `ProcessFailed` mapping without inventing a resource measurement.

The author reports a passing focused locked CLI/frontend compilation at this head. This reviewer ran no Cargo or runtime tests. The earlier five Linux and four macOS host results remain evidence for their earlier source heads only. Current-head qualification remains conditional on the queued three CLI and two unit tests; no current-head runtime pass is claimed. No Linear operations were performed.

Bounded integration disposition: no new changed-scope finding. Original resource-test attributes and explicit platform enum assertions are unchanged. No matrix or coverage computation was performed. The historical whole-repository FAIL and deferred baseline findings remain unchanged; this custody note neither replaces nor refreshes those measurements. Plan completion: not assessed.

Structural validation of these four retained artifacts with the explicit driver repository scope exited 0 with no diagnostics.

## Integration runtime disposition — 7eac980

Parent-reported frozen-head qualification at `7eac980458ba93e0566b6dcf24793d3da3b3e2f0`: corrected same-lease prerequisite build exited 0; frontend selected 3 passed, 0 failed, 0 ignored; the two retained same-head unit selections each passed 1, 0 failed, 0 ignored. All five targeted CLI tests therefore pass, clearing the current-head runtime condition above.

The bounded integration disposition is CLEAR and ready for the parent to copy and merge. Historical Linux five and macOS four host results remain bound to the earlier reviewed source, not this integration head. The historical whole-repository gap FAIL is unchanged. The remaining macOS watchdog work keeps QSL-2378 In Progress and is not claimed complete by this merge. This reviewer ran no Cargo, measurements or Linear operations.
