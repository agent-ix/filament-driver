---
id: SR-202611210022442313144108512
title: "QSL-2378 launcher and failure-priority integrity review"
type: SpecReview
analysis: integrity
scope: "spec/functional/FR-013-host-process-plugins.md; new resource launcher behavior, process-failure precedence and platform policy"
review_set: subset
relationships:
  - {target: "ix://agent-ix/filament-driver/FR-013", type: references}
---

## Summary

Integrity review of FR-013's new resource and platform policy against its retained acceptance criteria. The architecture ruling is reflected truthfully, but two older criteria in the changed requirement still state stricter behavior than the new launcher and EOF precedence allow.

## Verdict

CONDITIONAL: align AC-1 and AC-7 with the revised Behavior section. No unrelated requirement expansion is requested.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Behavior now permits the bundled launcher before exec, but AC-1 still says the host launches only the caller-named executable. Clarify that exact provider execution is preserved while the bundled launcher may start first. | FR-013-AC-1; FR-013 Behavior |
| FND-002 | medium | Behavior and AC-16 now prioritize an unsuccessful process exit over a truncated frame, but AC-7 still unconditionally requires a truncated frame's specific typed failure. State the same observed-exit exception in AC-7. | FR-013-AC-7; FR-013-AC-16; FR-013 Behavior |

## Coverage

The new contract distinguishes Linux memory enforcement from Unix CPU enforcement, preserves the explicit-presence bit even at the default numeric value, and defines observable unsupported-platform behavior before launch. The macOS default path states both the memory limitation and its separate diagnostic channel. These choices are consistent with the architecture ruling supplied by the parent and avoid silently calling an unenforced memory budget enforced.

The new clauses are externally observable through process resource outcomes, typed session diagnostics, pre-launch refusal and CLI output. Their actor and trigger meanings are concrete. The host remains outside the checked core, and FSL retains ownership of the machine document and failure vocabulary. No duplicate resource policy is introduced in another specification, and no new requirement delegates a live authenticated API, unbounded lookup or paginated service.

Only the contradictions at the two retained criteria are actionable scope here. The document-level base review and matrix separately record test binding and evidence limits. Existing whole-repository assurance debt is not part of this integrity finding set. No code or specification was edited by the reviewer.

## Dispositions — round 2

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-1 preserves exact provider execution and explicitly permits the bundled Unix resource launcher before replacing itself with that provider. |
| FND-002 | fixed | AC-7 now states the same observed-unsuccessful-exit precedence for truncated EOF as Behavior and AC-16. |

## Final review state

PASS for the changed scope after independently inspecting the criterion corrections at 83cfddd46adedac5ad57f0851820d0831f0380e9. No further integrity finding was found in the final disposition pass. The final code and evidence limitations remain recorded by the separate method artifacts.

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

- review: 71b72588-d698-4a43-bcf5-8c5951bdc718
- dispositions round 2: f507a865-7ec4-4545-acb6-fa925c24a361
