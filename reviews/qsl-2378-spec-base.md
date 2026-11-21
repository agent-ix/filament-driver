---
id: SR-202611210022442313144108511
title: "QSL-2378 platform memory policy base review"
type: SpecReview
analysis: base
scope: "spec/functional/FR-013-host-process-plugins.md; platform behavior changes and FR-013-AC-16 through AC-18"
review_set: subset
relationships:
  - {target: "ix://agent-ix/filament-driver/FR-013", type: references}
---

## Summary

Base review of the new Linux-memory, Unix-CPU and explicit unsupported-platform policy in FR-013. The new criteria are concrete and preserve platform limitations, but accepted source-test bindings are still missing for AC-16 and AC-17.

## Verdict

CONDITIONAL: complete the two criterion bindings and the explicit enforcement-status oracle identified by code review. This is a scoped review of the changed contract, not a new baseline cleanup request.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The computed matrix has no bound tests for new AC-16 and AC-17: their substantive host tests use fully qualified attributes that the active trace model does not bind. AC-16's Enforced output also needs an explicit value oracle rather than same-getter consistency. | FR-013-AC-16; FR-013-AC-17; code-review FND-004, FND-006 |

## Coverage

Review selection: base plus integrity, as requested for this bounded platform-policy extension. No applicable AssuranceProfile was found. No enforced review workflow or human acceptance claim is made. Optional EARS analysis was not selected; the deterministic validation reported one grammar-clean document and zero grammar findings.

The three new IDs are unique and contiguous after AC-15, and their Verification cells name Test. The document retains its US-001 implementing edge, FR-006 composition reference and FSL protocol dependency. Existing references resolve in the repository. The changed requirements describe platform-specific behavior, explicit-selection versus default state, typed refusal, pre-launch containment, CPU rounding, partial-response process failures and diagnostic channel separation. No new hash, pin, inventory or version-tracking requirement is introduced.

Linux memory and Unix CPU cases exercise real Rust processes, including a partial response. macOS explicit-limit tests check both a different value and the exact default value, both output formats, exit 21, empty stdout and absence of a launch marker. Default-memory tests check continued provider success and a single stderr notice without a diagnostic member in FSL output. The criterion-binding issue and enum oracle remain outstanding. Runtime execution is requested separately and not established by this review; the CLI dependency cohort update remains pending.

Validation: quire validate with the explicit repository scope and the changed FR-013 path exited 0; one document was grammar-clean, with zero grammar findings. Fresh matrix commands exited 0. The separate integrity artifact records remaining wording conflicts within the changed contract.

## Dispositions — round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The engine now binds all three new criteria through bare trace attributes, and the below-limit test explicitly asserts Enforced on Linux and NotEnforcedOnPlatform on other platforms. |

## Final review state

PASS for the changed contract at 83cfddd46adedac5ad57f0851820d0831f0380e9. The two inconsistent retained criteria identified by integrity review were aligned as well. Runtime checks and coordinated CLI compilation remain external pending evidence; this specification review does not claim those results or human acceptance. No new analysis cycle or unrelated baseline review was started.

## Final reviewed units

```yaml
scope:
  - id: FR-013-AC-1
    path: spec/functional/FR-013-host-process-plugins.md
    role: examined
    excerpt: Given a bare executable name, the host refuses to spawn it; given an
      explicit path, it executes exactly that provider with the supplied
      arguments and no shell. On Unix, the bundled resource launcher applies
      supported limits before replacing itself with the provider.
  - id: FR-013-AC-7
    path: spec/functional/FR-013-host-process-plugins.md
    role: examined
    excerpt: Truncated, malformed, non-canonical, oversized, unexpected, and wrongly
      correlated frames produce their specific typed failures without allocation
      above configured limits; when EOF truncates a frame and an unsuccessful
      process exit is observed, that process failure takes precedence.
  - id: FR-013-AC-16
    path: spec/functional/FR-013-host-process-plugins.md
    role: examined
    excerpt: On Linux, a Rust provider whose allocation exceeds its configured
      process memory ceiling returns an observed typed process failure,
      including when it wrote a partial response before exiting; a provider
      below the ceiling completes and reports memory enforcement as `Enforced`.
  - id: FR-013-AC-17
    path: spec/functional/FR-013-host-process-plugins.md
    role: examined
    excerpt: On Unix, a Rust provider consuming CPU until stopped is terminated by
      its configured CPU-time ceiling; when the inherited hard ceiling permits
      it, a setting of 1001 milliseconds applies a two-second OS ceiling, and a
      below-limit provider completes normally.
  - id: FR-013-AC-18
    path: spec/functional/FR-013-host-process-plugins.md
    role: examined
    excerpt: On macOS, a default provider completes with CPU enforcement and typed
      `NotEnforcedOnPlatform` memory diagnostics, and the CLI reports the memory
      limitation on stderr without changing the machine document. An explicit
      memory setting, including the default numeric value, is refused as typed
      `UnsupportedPlatform` before launch and exits 21.
```

## Tracker record

Ticket: QSL-2378. Linear has-review was already present. All posted comments passed read-back: the returned bodies matched, their YAML parsed with the installed yaml parser, finding IDs matched the retained artifact, and every fixed disposition retained a nonempty source excerpt.

- review: 524ed968-9d4c-46bd-8ff0-3baf7e89bbec
- dispositions round 2: 330fb412-20c4-4240-86fc-13cc277bc9fd


## Integration custody — 7eac980

Narrow integration disposition at driver `7eac980458ba93e0566b6dcf24793d3da3b3e2f0`, rebased from reviewed `e94b181e148a7edb8b1f08f6aab1b37d46af023f` onto the normal driver cohort `6b8f2a7`. This appends custody to the existing review cycle; it is not a fresh review, matrix run or baseline measurement.

The range-diff preserves the original launcher, typed process failures, dependency, shared Rust fixtures, partial-frame exit handling and request-write/remaining-grace fixes. The platform-policy commit changes only the placement of the unchanged memory-enforcement accessor into the existing implementation alongside hybrid exchange. Both methods remain present. The sole added integration commit adds `UnsupportedPlatform` to the hybrid transport error match. Explicit unsupported memory requests are still refused by `PluginSession::start` before process launch, so that defensive match arm does not turn an admitted platform refusal into an active exchange outcome. The normal manifest and lockfile agree on FSL `f5977125981288ea1bd36fb67f71780182055aac`; the adapter retains the observed Crash/NonzeroExit `ProcessFailed` mapping without inventing a resource measurement.

The author reports a passing focused locked CLI/frontend compilation at this head. This reviewer ran no Cargo or runtime tests. The earlier five Linux and four macOS host results remain evidence for their earlier source heads only. Current-head qualification remains conditional on the queued three CLI and two unit tests; no current-head runtime pass is claimed. No Linear operations were performed.

Bounded base disposition: CLEAR for the integration delta. FR-013 and its resource acceptance criteria are unchanged from the reviewed source. Existing round-two binding and enum-oracle dispositions remain fixed; no additional requirement or verification promise is introduced. Current runtime acceptance remains conditional as stated above.

Structural validation of these four retained artifacts with the explicit driver repository scope exited 0 with no diagnostics.

## Integration runtime disposition — 7eac980

Parent-reported frozen-head qualification at `7eac980458ba93e0566b6dcf24793d3da3b3e2f0`: corrected same-lease prerequisite build exited 0; frontend selected 3 passed, 0 failed, 0 ignored; the two retained same-head unit selections each passed 1, 0 failed, 0 ignored. All five targeted CLI tests therefore pass, clearing the current-head runtime condition above.

The bounded integration disposition is CLEAR and ready for the parent to copy and merge. Historical Linux five and macOS four host results remain bound to the earlier reviewed source, not this integration head. The historical whole-repository gap FAIL is unchanged. The remaining macOS watchdog work keeps QSL-2378 In Progress and is not claimed complete by this merge. This reviewer ran no Cargo, measurements or Linear operations.
