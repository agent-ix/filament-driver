---
id: SR-202611210022442313144108512
title: spec-review/scope-boundary review of driver PR 222 provider trust amendment
type: SpecReview
analysis: scope-boundary
scope: agent-ix/filament-driver; spec/functional/FR-022-analyze-a-native-analyze-request.md;
  spec/test-cases/TC-047-decide-hybrid-claims-with-the-reference-provider.md; QSL-2234
review_set: subset
---

## Summary

Ticket: QSL-2234. Exact reviewed head: fc501383b3a925962dc5b24d5f705c1c81658801; base: df280fbec1fd07f1934bc5e5673aa38d1484391f. Driver owns process construction and transport (infrastructure); FSL owns strict admission, verdict settlement, replay and certificate checking (core). FSL behavior is an assumed external dependency for this static review, identified through FR-290/750/751; no runtime contract-test guarantee is claimed. The in-process fixture and first-party reference proof remain distinct from the out-of-process plugin.

## Scope

```yaml
scope:
- id: FR-022-Behavior-hybrid
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: "  A third-party out-of-process proof SHALL carry certification `trusted`,\n  never `certified` or `uncertified` (FSL FR-290). The `uncertified` fixture\n  of FSL FR-750-AC-1 is an in-process `Engine::hybrid` provider; the reference\n  provider's accepted proof remains `certified`. The proof coverage, replay\n  and certificate acceptance checks of FSL FR-750 and FR-751 still apply."
- id: FR-022-AC-1
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: '`fsl analyze` over the Counter request with the item `{"claim": "Bounded"}` and `--format json` writes one document with `operation` `analyze`, category `success`, one item whose record is `proved`, and `aggregate.status` `complete`, and exits 0.'
- id: FR-022-AC-2
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: 'The item `{"claim": "BelowThree"}` gives an item whose record is `refuted`, with a `counterexample` entry and `replay` `{"target": {"state_clause": Q}, "reproduced": true}`, `Q` ending in `BelowThree`, category `violation`, and exit 10.'
- id: FR-022-AC-3
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: 'A request whose item sets `temporal.limits.max_states` to 1 gives an `incomplete` record whose cause names `max_states`, with no `counterexample`, and exit 22; a request with `deadline_ms` 0 gives an `incomplete` record whose cause is `{"kind": "cancelled", "source": "deadline"}`, and exit 22.'
- id: FR-022-AC-4
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: 'An item naming a backend the registry does not hold settles with category `refusal` and the record diagnostic `invalid_capability`/`unknown-backend` naming the backend, exit 20; an item over the Counter fixture naming backend `en-6`, which lists no temporal-satisfaction form, settles `unsupported` with `unsupported_projection`/`unsupported-requested-capability`, exit 21 (the item kind is incidental: EN-1 serves derived deadlock-freedom since QSL-2122, and a state-graph claim over a timed subject stays unsupported until QSL-2305); neither writes empty output.'
- id: FR-022-AC-5
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: 'A malformed `native-analyze/1` document gives, in machine mode, a document with `operation` `analyze`, category `refusal`, empty `items` and one `invalid-request` diagnostic, and in text mode one stderr line starting `fsl analyze: invalid-request:`; both exit 20. A program the checker refuses gives a document whose `last_stage` is the checker stage FSL names (`S3`), empty `items` and that refusal''s code, and the text line `fsl analyze: <code>: `; a snapshot naming a population the model does not declare gives the binding refusal''s code with empty `items`; each exits 20. A missing operand, an u'
- id: FR-022-AC-6
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: The machine-mode stdout for the AC-2 request is byte-equal to the canonical encoding of the document `fsl_command::command::analyze` returns in process for the same request, followed by one newline.
- id: FR-022-AC-7
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: In text mode the AC-2 request prints `0 BelowThree refuted` at the start of its first line, `violation` as its last line, and exits 10; an item naming an unregistered backend prints `0 Bounded invalid-request invalid_capability/unknown-backend`.
- id: FR-022-AC-8
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: Each settlement row of the table holds for the driver's negotiation over a candidate outcome of that shape, except the several-candidates-with-a-named-backend row, which FSL's candidate computation does not produce; the `registry` member of an outcome lists the engines FSL registers (`en-1` among them), and the `Bounded` item's candidates are `["en-1"]`.
- id: FR-022-AC-9
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: With `--provider-config` starting a process provider whose `hello` advertises (`value-validity`, `unbounded`), an item naming its backend is sent to it as one FR-331 request at the item's request index and settles `proved` with certification `trusted` when the provider answers `proved`, and `inconclusive` when it answers anything else; the document's `registry` lists the provider and `en-1`; another item is unchanged by it. A provider advertising only `bounded`, an unreadable configuration and a malformed one each refuse with exit 20.
- id: FR-022-AC-10
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: 'SIGTERM or SIGINT while an item runs gives a document whose open and later items are `incomplete` with cause `{"kind": "cancelled", "source": "requested"}`, and exit 22.'
- id: FR-022-AC-11
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'The process-provider half depends on FSL''s `Engine::hybrid_process` (FSL FR-750): with `--provider-config` starting a fixture process provider whose `hello` advertises (`temporal-satisfaction`, `unbounded`) with `dynamics` `["hybrid"]` and that answers `delta-complete` `unsat` covering horizon 10 with certification `trusted`, the slow-enough item rewritten to name that provider''s backend is sent to it as one FR-447 request and settles `proved`, `bounded-solver`, `trusted`, naming the fixture backend, exit 0, with no `ambiguous-backend`; the item naming `fsl-hybrid-reference` settles `prov'
- id: FR-022-AC-12
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: With no `--provider-config`, `fsl analyze` over a timed Counter subject with one item named `TimedLiveness`, whose clause is the unbounded `always (holds(c.value = 0) implies eventually holds(c.value = 0))` on `model-time`, writes one item settled `proved` by `en-6`; the record includes FR-632's `non_zeno` member, the document category is `success`, and the process exits 0.
- id: FR-022-mixed-provider-note
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: context_only
  excerpt: 'The current QSL-2234 consuming slice temporarily refuses registration of a

    process provider whose `hello` advertises both (`value-validity`, `unbounded`)

    and (`temporal-satisfaction`, `unbounded`) with `dynamics` `["hybrid"]`.

    This is a typed registration refusal before any item runs: its diagnostic

    names the provider and both capability groups, every started provider session

    is shut down, and neither capability group is silently dropped. This records

    incomplete implementation of the full contract below, which still admits

    such mixed manifests; it does not narrow their eligibility. A combined '
- id: TC-047-step-4
  path: spec/test-cases/TC-047-decide-hybrid-claims-with-the-reference-provider.md
  role: examined
  excerpt: "- Step 4: the fixture-named item is sent to the provider as one FR-447 request\n  and settles `proved`, `bounded-solver`, `uncertified`, naming the fixture\n  backend, exit 0, with no `ambiguous-backend`;\n  the reference-named item settles `proved`, `certified`, exit 0; the unnamed\n  item settles `invalid-request`, `invalid_capability`/`ambiguous-backend`,\n  naming both backends, exit 20; the `bounded`-only fixture refuses with exit\n  20 before any item runs."
- id: FSL-FR-290-producer-table
  path: spec/functional/FR-290-settle-plugin-results-as-typed-terminal-records.md
  role: context_only
  excerpt: '| Producer of the `proved` result | Settled by | Certification | The record also names |

    | --- | --- | --- | --- |

    | An `analyze` engine whose certificate a qualified-core checker accepted (FR-281, FR-282) | FSL, `fsl-replay` | `certified` | the checker that accepted it |

    | A native `analyze` engine with no qualified-core certificate checker (FR-281) | FSL, `fsl-replay` | `uncertified` | the engine |

    | A third-party out-of-process plugin | FSL, `fsl-replay` | `trusted` | the plugin''s `BackendId` |

    | The Kani backend, whose prove path is part of the qualified core (ADR-029 CB-2) | CG, C-09 | `c'
- id: FSL-FR-290-Certification
  path: spec/functional/FR-290-settle-plugin-results-as-typed-terminal-records.md
  role: context_only
  excerpt: "`read_plugin_certification` accepts exactly the JSON string `trusted` (FSpec\nFR-331 \"Certification\", FR-331-AC-23: a plugin proof is `trusted`). It returns\nno value, because an accepted entry's record carries `trusted`\n(`TerminalRecord::from_plugin_proof`). It checks in this order:\n\n1. The member is absent: `MissingMember` at `/certification`.\n2. The member is not a string: `Malformed` at `/certification`.\n3. The string is not one of FR-331's three values `certified`,\n   `uncertified` and `trusted`: `Malformed` at `/certification`.\n4. The string is `certified` or `uncertified`, which only the "
- id: FSL-FR-750-AC-1
  path: spec/functional/FR-750-run-a-hybrid-item-through-its-negotiated-hybrid-provider.md
  role: context_only
  excerpt: '| FR-750-AC-1 | With an in-process fixture provider registered through `Engine::hybrid`, `SlowEnough` over `Braking` is dispatched once to it, and its `delta-complete` `unsat` covering horizon 10 settles `proved`, `bounded-solver`, `uncertified`, naming the fixture backend. | Test (TC-2100) |'
- id: FSL-FR-750-Dispatch
  path: spec/functional/FR-750-run-a-hybrid-item-through-its-negotiated-hybrid-provider.md
  role: context_only
  excerpt: "- Every returned certificate that the verdict rests on SHALL be checked by\n  FR-751 before the record is written: the certificate of a `proved` entry\n  that FR-250 and FSpec FR-193 coverage settle `proved`, and the\n  `validated-witness` of a trace FR-450's exact replay returns `NotExact`\n  for. Any other returned certificate, on a proof settled\n  `insufficient-horizon` or with a trace exact replay decides, whose replay\n  is refused or whose replay the work meter stops, is recorded unchecked\n  (FR-621) and changes no settlement. Every returned certificate is still\n  read by `read_entry`, so a c"
- id: FSL-FR-751-certificate-acceptance
  path: spec/functional/FR-751-check-hybrid-provider-certificates-in-the-core.md
  role: context_only
  excerpt: "- An accepted certificate SHALL set the proof's certification to\n  `Certified`; a rejected one SHALL keep FR-250's certification and record\n  `certificate-rejected`."
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for this method and the bounded static amendment only. Driver owns process construction and transport (infrastructure); FSL owns strict admission, verdict settlement, replay and certificate checking (core). FSL behavior is an assumed external dependency for this static review, identified through FR-290/750/751; no runtime contract-test guarantee is claimed. The in-process fixture and first-party reference proof remain distinct from the out-of-process plugin.

## Evidence and limits

FSL evidence: d0ac266b85e21dc67997b740dcab480749c2bf93; the FR-290/750/751 files are identical at locally available origin/main e3ed0df2ce01b03dc663ed0910698ed5c75ce3cb. FR-290 producer table lines 87–93 and strict Certification lines 240–253; FR-750 in-process AC-1, Dispatch coverage/replay/check rules and AC-16; FR-751 certificate acceptance rules. No AssuranceProfile was found under this repository spec tree. No enforcement workflow is required by a discovered profile. The dispatch authorizes the review subset and architecture ruling.

Ran focused Quire validation of FR-022 with installed spec-artifacts-iso: exit 0; grammar-clean 1/1, zero grammar findings. git diff --check over the exact candidate range: exit 0. Quoin 0.36.8 and Quire 0.52.0 (engine 0.62.2), resolved under /home/peter/.npm-global/bin. No builds, runtime tests, matrix, broad coverage audit or fetched state. Existing runtime completion and unrelated inherited spec issues are outside scope. QSL-2234 remains open.

Methods selected: code-review, base, failure-domain, scope-boundary and ears-conformance. Rust/React/Python lanes and gap-analysis do not apply to this spec-only diff. Integrity structural checks, dependency edges, evidence-method changes, object/security catalog changes, risk-complexity and criterion-strength were not selected: this amendment adds none of those structures or methods; its trust boundary is examined under failure-domain and scope-boundary. No full matrix or aggregate gate PASS is claimed.

## Verdict (disposition pass 1)

PASS / CLEAR for the bounded static spec amendment at exact head 10a9a17224cb736f2bfd17140f0a0f094d9094af. Independently inspected the fix diff from fc501383b3a925962dc5b24d5f705c1c81658801: only TC-047 changes, adding explicit trusted certification to its fixture procedure and replacing uncertified with trusted in its process proof expectation. FR-022 is byte-unchanged from the reviewed candidate. The default reference remains certified, refuted outcomes and ambiguity/refusal expectations remain unchanged, and the FSL proof coverage, replay and certificate checks are unaffected. This method had no original findings and adds none in disposition 1. The latest finding state is clear across all five methods. Historical Findings, original scope and review verdict are preserved above.

Focused validation of FR-022 and TC-047 using the ISO module exited 0; git diff --check for the exact fix range exited 0; worktree is clean. No builds, runtime tests, fetch, broad coverage audit or aggregate gate. QSL-2234 remains open.

### Scope examined in disposition 1

```yaml
scope:
- id: TC-047-step-4-procedure
  path: spec/test-cases/TC-047-decide-hybrid-claims-with-the-reference-provider.md
  role: examined
  excerpt: "   ignored until it lands. Write a `--provider-config` starting a fixture\n   process provider whose `hello` advertises (`temporal-satisfaction`,\n   `unbounded`) with `dynamics` `[\"hybrid\"]` and that answers `delta-complete`\n   `unsat` covering horizon 10 with certification `trusted`. Run three variants\n   of the slow-enough request with it: the item naming the fixture's backend, the item naming\n   `fsl-hybrid-reference`, and the item naming no backend. Then run the\n   request with a fixture whose `hello` advertises only\n   (`temporal-satisfaction`, `bounded`).\n"
- id: TC-047-step-4
  path: spec/test-cases/TC-047-decide-hybrid-claims-with-the-reference-provider.md
  role: examined
  excerpt: "- Step 4: the fixture-named item is sent to the provider as one FR-447 request\n  and settles `proved`, `bounded-solver`, `trusted`, naming the fixture\n  backend, exit 0, with no `ambiguous-backend`;\n  the reference-named item settles `proved`, `certified`, exit 0; the unnamed\n  item settles `invalid-request`, `invalid_capability`/`ambiguous-backend`,\n  naming both backends, exit 20; the `bounded`-only fixture refuses with exit\n  20 before any item runs."
- id: FR-022-AC-11
  path: spec/functional/FR-022-analyze-a-native-analyze-request.md
  role: examined
  excerpt: 'The process-provider half depends on FSL''s `Engine::hybrid_process` (FSL FR-750): with `--provider-config` starting a fixture process provider whose `hello` advertises (`temporal-satisfaction`, `unbounded`) with `dynamics` `["hybrid"]` and that answers `delta-complete` `unsat` covering horizon 10 with certification `trusted`'
```
