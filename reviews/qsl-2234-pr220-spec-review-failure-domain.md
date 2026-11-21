---
id: SR-202611210022442313144108508
title: "spec-review/failure-domain of QSL-2234 mixed-provider status note"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/filament-driver; spec/functional/FR-022-analyze-a-native-analyze-request.md:105-117"
review_set: subset
---

## Summary

Ticket: QSL-2234. PR: https://github.com/agent-ix/filament-driver/pull/220. Frozen head: 38144b224e299b0ffd14b641f7424abbd3fe5fe1; base: 2e92e4f23d566bf67573a27176df5ace5790c6cf. Only the added current-behavior note at FR-022:105-117 is reviewed; the normative routing and refusal clauses are context. This is a specification-stage review, not runtime acceptance of the pending consuming implementation.

The note preserves both-group manifest eligibility and the original SHALL clauses and ACs. It records temporary refusal before any item, names the provider and both groups, shuts down every started session, and forbids silent capability loss. It expressly retains the combined engine under one backend identity as unfinished FSL-owned API work under open QSL-2234. This agrees with owner ruling 5076cfdd-2343-4182-9574-94a0dd8c109b.

No AssuranceProfile applies: no AssuranceProfile or review_policy declaration exists in this repository's spec tree. No enforced review run is required. The dispatch selects base plus the bounded failure-domain lens; no new requirements, criteria, tracker issues, matrix or assurance records are authored.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. Refusal is typed and precedes every item; diagnostics identify the provider and both capability groups; cleanup covers every started provider session. Neither group may be dropped. Shared backend identity is explicitly retained as the unresolved combined-engine problem. Purity and graph topology are not changed.

## Reviewed units

- FR-022 at spec/functional/FR-022-analyze-a-native-analyze-request.md:107-109; role: examined; excerpt:

```text
The current QSL-2234 consuming slice temporarily refuses registration of a
process provider whose `hello` advertises both (`value-validity`, `unbounded`)
and (`temporal-satisfaction`, `unbounded`) with `dynamics` `["hybrid"]`.
```

- FR-022 at spec/functional/FR-022-analyze-a-native-analyze-request.md:110-112; role: examined; excerpt:

```text
This is a typed registration refusal before any item runs: its diagnostic
names the provider and both capability groups, every started provider session
is shut down, and neither capability group is silently dropped. This records
```

- FR-022 at spec/functional/FR-022-analyze-a-native-analyze-request.md:113-117; role: examined; excerpt:

```text
incomplete implementation of the full contract below, which still admits
such mixed manifests; it does not narrow their eligibility. A combined engine
carrying the value provider and hybrid exchange under one backend identity
needs an FSL-owned API first. That combined-provider admission remains open
under QSL-2234.
```

- FR-022 at spec/functional/FR-022-analyze-a-native-analyze-request.md:145-150; role: context_only; excerpt:

```text
- For a configured provider whose admitted `hello` advertises
  (`temporal-satisfaction`, `unbounded`) with `dynamics` `["hybrid"]`, the
  driver SHALL make its engine with FSL's `Engine::hybrid_process(descriptor,
  tool, exchange)` (FSL FR-750), the exchange carrying FSL's FR-447 request
  frame bytes to the process and its FR-331 result frame bytes back, and SHALL
  put that descriptor in the negotiation manifest. FSL then settles an item
```

- FR-022 at spec/functional/FR-022-analyze-a-native-analyze-request.md:151-154; role: context_only; excerpt:

```text
  routed to it by FR-750 "Dispatch": its `delta-complete` `unsat` covering the
  item's horizon settles `proved`, `bounded-solver`, `uncertified`; a
  `refuted` settles `refuted` only after exact replay. A provider advertising
  (`value-validity`, `unbounded`) keeps the external-engine route above.
```

- FR-022 at spec/functional/FR-022-analyze-a-native-analyze-request.md:158-162; role: context_only; excerpt:

```text
- A `--provider-config` file that cannot be read or does not match its shape,
  a provider that does not start, and a provider whose manifest advertises
  neither (`value-validity`, `unbounded`) nor (`temporal-satisfaction`,
  `unbounded`) with `dynamics` `["hybrid"]` SHALL refuse with exit 20 before
  any item runs; every started provider is shut down.
```

## Method selection and limits

Code-review applies to every diff and is limited here to documentation integrity, duplication, and spec faithfulness. Base spec-review checks the new note and contextual normative statements. Failure-domain examines the refusal boundary, cleanup completeness and shared backend identity. EARS is skipped because no requirement statement changes; integrity because there is no structural requirement/AC/frontmatter edit; dependency because no relationship changes; object-review because no domain object changes; evidence and criterion-strength because no AC/evidence method changes; risk-complexity and scope-boundary because no new risk allocation or eligibility boundary is introduced. Rust/React/Python lanes and gap-analysis are skipped because no code or tests change. Existing FR-022 defects outside the inserted note are not reopened. Criterion coverage/matrix and runtime gates are inapplicable to this status note; no source/spec-byte tests, local Cargo, builds, or full gate run.
