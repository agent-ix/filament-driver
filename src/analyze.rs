// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's negotiation of `analyze` items (FSL FR-281 "Negotiated
//! items", FR-620 "Resolving items", FSpec FR-711).
//!
//! FSL's `analyze` never negotiates: the caller settles each resolved item
//! from the candidate set FSL computed over its registry. [`AnalyzeNegotiation`]
//! is that caller-side settlement. It holds the FR-331 manifest of FSL's
//! linked engines and settles every item through CG's one item entry,
//! `negotiate_item`, then reads CG's disposition back as the disposition and
//! diagnostic FSL takes. It applies no candidate rule of its own and never
//! chooses among several candidates.

use std::collections::{BTreeMap, BTreeSet};

use filament_contract_codegen as cg;
use filament_semantic_value::catalog::CatalogCode;
use fsl_analyze::analyze::document::Negotiate;
use fsl_analyze::analyze::{ClaimItem, Diagnostic, Engines, ItemForm, Payload};
use fsl_foundation::BackendId;
use fsl_route::routing::Disposition;
use fsl_route::{Candidate, CandidateOutcome, RegistrationRefusal};
use fsl_semantics::check::Capability;

/// The settlement the driver gives each `analyze` item: CG's `negotiate_item`
/// over the manifest of FSL's linked engines.
#[derive(Clone, Debug)]
pub struct AnalyzeNegotiation {
    manifest: Vec<cg::BackendDescriptor>,
}

impl AnalyzeNegotiation {
    /// The negotiation over the manifest of the engines FSL's `analyze` runs.
    ///
    /// # Errors
    ///
    /// The registry's refusal of an engine's own manifest.
    pub fn fsl() -> Result<Self, RegistrationRefusal> {
        let engines = Engines::fsl_with_reference_hybrid()?;
        Ok(Self {
            manifest: engines.registrations().map(manifest_entry).collect(),
        })
    }

    /// The negotiation over the manifest of FSL's engines and the caller's
    /// configured process providers, `descriptors` their admitted `Process`
    /// descriptors (FSL FR-635): the same registry FSL builds when the
    /// engines of [`run_engines`] are handed to `analyze_with_providers`.
    ///
    /// # Errors
    ///
    /// The registry's refusal of an engine's own manifest.
    pub fn with_providers<'a>(
        descriptors: impl IntoIterator<Item = &'a fsl_route::BackendDescriptor>,
    ) -> Result<Self, RegistrationRefusal> {
        let mut negotiation = Self::fsl()?;
        negotiation
            .manifest
            .extend(descriptors.into_iter().map(manifest_entry));
        Ok(negotiation)
    }

    /// The FR-331 manifest the negotiation settles over.
    pub fn manifest(&self) -> &[cg::BackendDescriptor] {
        &self.manifest
    }
}

/// The engines a run hands to FSL beside its own: the reference hybrid
/// engine, then `providers`. FSL registers them after the engines it links,
/// so its registry equals the manifest of [`AnalyzeNegotiation::fsl`] and
/// [`AnalyzeNegotiation::with_providers`].
///
/// # Errors
///
/// The registry's refusal of the hybrid engine's own manifest.
pub fn run_engines(
    providers: Vec<fsl_analyze::analyze::Engine>,
) -> Result<Vec<fsl_analyze::analyze::Engine>, RegistrationRefusal> {
    let mut engines = vec![fsl_analyze::analyze::Engine::hybrid_reference()?];
    engines.extend(providers);
    Ok(engines)
}

impl Negotiate for AnalyzeNegotiation {
    fn negotiate(
        &self,
        item: &ClaimItem,
        candidates: &CandidateOutcome,
    ) -> (Disposition, Option<Diagnostic>) {
        settle(&self.manifest, item, candidates)
    }
}

/// CG's manifest entry for one linked engine's registration.
fn manifest_entry(descriptor: &fsl_route::BackendDescriptor) -> cg::BackendDescriptor {
    let mut advertised: Vec<_> = descriptor
        .advertises()
        .filter_map(|(kind, mode)| {
            cg::CapabilityKind::from_label(kind.to_wire()).map(|kind| (kind, cg_mode(mode)))
        })
        .collect();
    advertised.sort();
    let profile = profile_of(descriptor);
    cg::BackendDescriptor {
        identity: descriptor.id().as_str().to_owned(),
        advertised,
        origin: match descriptor.origin() {
            fsl_route::ProviderOrigin::Linked => cg::ProviderOrigin::Linked,
            fsl_route::ProviderOrigin::Process => cg::ProviderOrigin::Process,
        },
        domains: descriptor.domains().cloned(),
        bounds: descriptor
            .max_cases()
            .map(|cases| BTreeMap::from([("max_cases".to_owned(), cases.get())])),
        profile,
    }
}

/// What the registration declares beyond its modes and domains, as CG reads it.
pub(crate) fn profile_of(descriptor: &fsl_route::BackendDescriptor) -> cg::BackendProfile {
    cg::BackendProfile {
        dynamics: descriptor
            .dynamics()
            .map(|kinds| {
                kinds
                    .iter()
                    .map(|kind| match kind {
                        fsl_route::DynamicsKind::Discrete => cg::Dynamics::Discrete,
                        fsl_route::DynamicsKind::Hybrid => cg::Dynamics::Hybrid,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        fairness_kinds: descriptor
            .fairness_kinds()
            .iter()
            .copied()
            .map(cg_fairness)
            .collect(),
        state_graph_forms: descriptor
            .state_graph_forms()
            .iter()
            .map(|form| match form {
                fsl_route::StateGraphFormKind::Possible => cg::StateGraphForm::Possible,
                fsl_route::StateGraphFormKind::AlwaysPossible => cg::StateGraphForm::AlwaysPossible,
                fsl_route::StateGraphFormKind::UniquePath => cg::StateGraphForm::UniquePath,
            })
            .collect(),
        reductions: descriptor
            .reductions()
            .iter()
            .copied()
            .map(cg_reduction)
            .collect(),
        obligation_forms: BTreeSet::new(),
        evidence_kinds: descriptor
            .evidence_kinds()
            .iter()
            .copied()
            .map(cg_evidence)
            .collect(),
    }
}

fn cg_mode(mode: fsl_route::Mode) -> cg::Mode {
    match mode {
        fsl_route::Mode::Bounded => cg::Mode::Bounded,
        fsl_route::Mode::Unbounded => cg::Mode::Unbounded,
    }
}

fn route_mode(mode: cg::Mode) -> fsl_route::Mode {
    match mode {
        cg::Mode::Bounded => fsl_route::Mode::Bounded,
        cg::Mode::Unbounded => fsl_route::Mode::Unbounded,
    }
}

fn cg_fairness(kind: filament_semantic_value::fairness::FairnessKind) -> cg::FairnessKind {
    match kind {
        filament_semantic_value::fairness::FairnessKind::Weak => cg::FairnessKind::Weak,
        filament_semantic_value::fairness::FairnessKind::Strong => cg::FairnessKind::Strong,
    }
}

fn cg_reduction(kind: fsl_foundation::ReductionKind) -> cg::Reduction {
    match kind {
        fsl_foundation::ReductionKind::Symmetry => cg::Reduction::Symmetry,
        fsl_foundation::ReductionKind::PartialOrder => cg::Reduction::PartialOrder,
        fsl_foundation::ReductionKind::StateConstraint => cg::Reduction::Constraint,
    }
}

fn cg_evidence(kind: fsl_route::EvidenceKind) -> cg::EvidenceKind {
    match kind {
        fsl_route::EvidenceKind::Statistical => cg::EvidenceKind::Statistical,
        fsl_route::EvidenceKind::Exact => cg::EvidenceKind::Exact,
    }
}

/// The request item CG settles for `item` over the candidates FSL computed.
fn request_item(item: &ClaimItem, candidates: &CandidateOutcome) -> cg::RequestItem {
    let (extent, finite_bound_available) = match item.extent {
        fsl_route::request::ExtentClassification::Bounded => (cg::Mode::Bounded, false),
        fsl_route::request::ExtentClassification::Unbounded {
            finite_bound_available,
        } => (cg::Mode::Unbounded, finite_bound_available),
    };
    let mut requirements = cg::ItemRequirements::default();
    let mut bounds = Vec::new();
    match &item.form {
        Some(ItemForm::Temporal(temporal)) => {
            bounds.clone_from(&temporal.proof_bounds);
            requirements.fairness = temporal
                .fairness
                .iter()
                .map(|constraint| cg_fairness(constraint.kind()))
                .collect();
            requirements.reductions = temporal
                .reductions
                .selected()
                .into_iter()
                .map(cg_reduction)
                .collect();
        }
        Some(ItemForm::Hybrid(hybrid)) => {
            bounds.clone_from(&hybrid.proof_bounds);
            requirements.hybrid_dynamics = true;
        }
        Some(ItemForm::FiniteHyper(_)) => {
            // The checked clause's bound limits its trace population, which
            // FSL evaluates with ItemOptions::traces. It is not a substituted
            // proof-domain bound or an additional CG manifest requirement.
            // Kind, extent and candidates still use the common projection.
        }
        Some(ItemForm::Probabilistic(probabilistic)) => {
            requirements.evidence = Some(cg::RequestedEvidence::Known(cg_evidence(
                probabilistic.evidence,
            )));
        }
        None => {}
    }
    let (named_backend, candidates) = match candidates {
        CandidateOutcome::Candidates(set) => (
            set.requested_backend().map(|id| id.as_str().to_owned()),
            cg::Candidates::Set(
                set.candidates()
                    .iter()
                    .map(|candidate| cg::Candidate {
                        identity: candidate.id().as_str().to_owned(),
                    })
                    .collect(),
            ),
        ),
        CandidateOutcome::UnknownBackend(id) => (
            Some(id.as_str().to_owned()),
            cg::Candidates::UnknownBackend(id.as_str().to_owned()),
        ),
    };
    cg::RequestItem {
        kind: cg::CapabilityKind::from_label(item.kind.to_wire()).map_or_else(
            || cg::RequestedKind::Unknown(item.kind.to_wire().to_owned()),
            cg::RequestedKind::Known,
        ),
        extent: Some(cg::ExtentClassification {
            extent,
            finite_bound_available,
            domains: Vec::new(),
            bounds,
        }),
        named_backend,
        candidates,
        kani_form: None,
        requirements,
    }
}

/// The settlement of `item` over `candidates` through CG's item entry.
fn settle(
    manifest: &[cg::BackendDescriptor],
    item: &ClaimItem,
    candidates: &CandidateOutcome,
) -> (Disposition, Option<Diagnostic>) {
    let request = request_item(item, candidates);
    match cg::negotiate_item(manifest, &request) {
        Ok(disposition) => read_back(manifest, item, &request, disposition),
        Err(_) => (Disposition::InvalidRequest, Some(inconsistent(&request))),
    }
}

fn inconsistent(request: &cg::RequestItem) -> Diagnostic {
    Diagnostic {
        code: CatalogCode::new("invalid_capability", "inconsistent-candidates"),
        payload: request
            .named_backend
            .as_deref()
            .map_or(Payload::None, |id| Payload::Backend(BackendId::new(id))),
    }
}

/// CG's disposition as the disposition and diagnostic FSL takes.
fn read_back(
    manifest: &[cg::BackendDescriptor],
    item: &ClaimItem,
    request: &cg::RequestItem,
    disposition: cg::Disposition,
) -> (Disposition, Option<Diagnostic>) {
    match disposition {
        cg::Disposition::Supported { backend } => (
            Disposition::Supported(Candidate::new(BackendId::new(backend))),
            None,
        ),
        cg::Disposition::RequiresBound { backend } => (
            Disposition::RequiresBound,
            Some(unbounded(manifest, item.kind, backend)),
        ),
        cg::Disposition::Unsupported { cause } => (
            Disposition::Unsupported,
            Some(cause_diagnostic(manifest, item, request, cause)),
        ),
        cg::Disposition::InvalidRequest { cause } => (
            Disposition::InvalidRequest,
            Some(cause_diagnostic(manifest, item, request, cause)),
        ),
    }
}

/// The `unbounded-extent` diagnostic: the kind, the candidate and the modes
/// its manifest advertises for the kind.
fn unbounded(manifest: &[cg::BackendDescriptor], kind: Capability, backend: String) -> Diagnostic {
    let modes = cg::CapabilityKind::from_label(kind.to_wire())
        .and_then(|kind| {
            manifest
                .iter()
                .find(|entry| entry.identity == backend)
                .map(|entry| {
                    entry
                        .advertised
                        .iter()
                        .filter(|(advertised, _)| *advertised == kind)
                        .map(|(_, mode)| route_mode(*mode))
                        .collect()
                })
        })
        .unwrap_or_default();
    Diagnostic {
        code: CatalogCode::new("unsupported_projection", "unbounded-extent"),
        payload: Payload::Unbounded {
            kind,
            candidate: BackendId::new(backend),
            modes,
        },
    }
}

/// The FR-272 code and payload CG's `cause` is reported under.
fn cause_diagnostic(
    manifest: &[cg::BackendDescriptor],
    item: &ClaimItem,
    request: &cg::RequestItem,
    cause: cg::Cause,
) -> Diagnostic {
    let unsupported = |cause| Diagnostic {
        code: CatalogCode::new("unsupported_projection", cause),
        payload: Payload::Kind(item.kind),
    };
    let invalid = |cause| Diagnostic::new(CatalogCode::new("invalid_capability", cause));
    match cause {
        cg::Cause::UnknownBackend { backend } => Diagnostic {
            code: CatalogCode::new("invalid_capability", "unknown-backend"),
            payload: Payload::Backend(BackendId::new(backend)),
        },
        cg::Cause::AmbiguousBackend { .. } => invalid("ambiguous-backend"),
        cg::Cause::InconsistentCandidates { .. } | cg::Cause::KaniFormInvalid { .. } => {
            inconsistent(request)
        }
        cg::Cause::AbsentKind => invalid("absent-kind"),
        cg::Cause::UnknownKind { .. } => invalid("unknown-kind"),
        cg::Cause::AbsentExtent => invalid("absent-extent"),
        cg::Cause::AbsentEvidenceKind => invalid("absent-evidence-kind"),
        cg::Cause::UnknownEvidenceKind { .. } => invalid("unknown-evidence-kind"),
        cg::Cause::UnboundedExtent { backend, .. } => unbounded(manifest, item.kind, backend),
        cg::Cause::UnsupportedRequestedCapability { .. }
        | cg::Cause::UnsupportedRequirement { .. }
        | cg::Cause::UnsupportedDomain { .. }
        | cg::Cause::UnsupportedKaniForm { .. }
        | cg::Cause::KaniFormRefused { .. }
        | cg::Cause::KaniStateFrameRefused { .. } => {
            unsupported("unsupported-requested-capability")
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use std::collections::BTreeSet;

    use filament_exact::{NodeKey, Origin, PathSegment, Role};
    use fsl_foundation::bound::DomainKind;
    use fsl_foundation::source::provenance::OccurrenceKey;
    use fsl_foundation::RequestIndex;
    use fsl_route::request::ExtentClassification;
    use fsl_route::{BackendDescriptor, Mode, ProviderOrigin, Registry};
    use ix_trace_rs::trace;

    use super::*;

    const KIND: Capability = Capability::TemporalSatisfaction;

    fn descriptor(id: &str, modes: &[Mode]) -> BackendDescriptor {
        BackendDescriptor::new(
            Candidate::new(BackendId::new(id)),
            ProviderOrigin::Linked,
            modes.iter().map(|mode| (KIND, *mode)),
            modes
                .contains(&Mode::Bounded)
                .then(|| BTreeSet::from([DomainKind::Collection])),
            None,
        )
        .expect("a descriptor with domains for a bounded mode")
    }

    /// A registry and its CG manifest holding one engine per `(id, modes)`.
    fn world(engines: &[(&str, &[Mode])]) -> (Registry, AnalyzeNegotiation) {
        let mut registry = Registry::new();
        for (id, modes) in engines {
            registry
                .register(descriptor(id, modes))
                .expect("a new identity");
        }
        let manifest = registry.descriptors().map(manifest_entry).collect();
        (registry, AnalyzeNegotiation { manifest })
    }

    fn item(extent: ExtentClassification, backend: Option<&str>) -> ClaimItem {
        let node = NodeKey::new("test", vec![PathSegment::Text("claim".to_owned())], 0)
            .expect("a valid node");
        ClaimItem {
            request_index: RequestIndex::new(0),
            claim: OccurrenceKey::new(node, Origin::new(Role::new("claim"), 0)),
            kind: KIND,
            extent,
            backend: backend.map(BackendId::new),
            candidates: Vec::new(),
            options: fsl_analyze::analyze::ItemOptions::default(),
            form: None,
        }
    }

    fn settle_over(
        registry: &Registry,
        negotiation: &AnalyzeNegotiation,
        item: &ClaimItem,
    ) -> (Disposition, Option<Diagnostic>) {
        let outcome = registry.candidates(KIND, item.backend.as_ref());
        negotiation.negotiate(item, &outcome)
    }

    fn code(diagnostic: Option<&Diagnostic>) -> (&str, &str) {
        let diagnostic = diagnostic.expect("a diagnostic");
        (diagnostic.code.code(), diagnostic.code.cause())
    }

    const UNBOUNDED_BOUNDABLE: ExtentClassification = ExtentClassification::Unbounded {
        finite_bound_available: true,
    };
    const UNBOUNDED_UNBOUNDABLE: ExtentClassification = ExtentClassification::Unbounded {
        finite_bound_available: false,
    };

    /// FR-022-AC-8: an engine advertising both modes serves a bounded and an
    /// unbounded item, `supported` on it with no diagnostic.
    #[trace("TC-1741", "FSpec-FR-711-AC-1")]
    #[test]
    fn a_both_mode_engine_serves_each_extent() {
        let (registry, negotiation) = world(&[("en-1", &[Mode::Bounded, Mode::Unbounded])]);
        for extent in [ExtentClassification::Bounded, UNBOUNDED_BOUNDABLE] {
            let (disposition, diagnostic) =
                settle_over(&registry, &negotiation, &item(extent, None));
            assert_eq!(
                disposition,
                Disposition::Supported(Candidate::new(BackendId::new("en-1")))
            );
            assert!(diagnostic.is_none());
        }
    }

    /// FSpec FR-290 extent row: an unbounded extent on a bounded-only engine
    /// with a finite bound available is `requires-bound`.
    #[trace("TC-1741", "FSpec-FR-711-AC-2")]
    #[test]
    fn an_unbounded_extent_with_a_bound_available_requires_a_bound() {
        let (registry, negotiation) = world(&[("en-1", &[Mode::Bounded])]);
        let (disposition, diagnostic) =
            settle_over(&registry, &negotiation, &item(UNBOUNDED_BOUNDABLE, None));
        assert_eq!(disposition, Disposition::RequiresBound);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("unsupported_projection", "unbounded-extent")
        );
        assert_eq!(
            diagnostic.map(|d| d.payload),
            Some(Payload::Unbounded {
                kind: KIND,
                candidate: BackendId::new("en-1"),
                modes: vec![Mode::Bounded],
            })
        );
    }

    /// FSpec FR-290 extent row: an unbounded extent on a bounded-only engine
    /// with no finite bound is `unsupported` `unbounded-extent`.
    #[trace("TC-1741", "FSpec-FR-711-AC-2")]
    #[test]
    fn an_unbounded_extent_with_no_bound_is_unsupported() {
        let (registry, negotiation) = world(&[("en-1", &[Mode::Bounded])]);
        let (disposition, diagnostic) =
            settle_over(&registry, &negotiation, &item(UNBOUNDED_UNBOUNDABLE, None));
        assert_eq!(disposition, Disposition::Unsupported);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("unsupported_projection", "unbounded-extent")
        );
    }

    /// FSpec FR-290 extent row: a bounded extent on a bounded-only engine is
    /// `supported`.
    #[trace("TC-1741", "FSpec-FR-711-AC-2")]
    #[test]
    fn a_bounded_extent_on_a_bounded_engine_is_supported() {
        let (registry, negotiation) = world(&[("en-1", &[Mode::Bounded])]);
        let (disposition, diagnostic) = settle_over(
            &registry,
            &negotiation,
            &item(ExtentClassification::Bounded, None),
        );
        assert_eq!(
            disposition,
            Disposition::Supported(Candidate::new(BackendId::new("en-1")))
        );
        assert!(diagnostic.is_none());
    }

    /// FSpec FR-290 extent row: an unbounded extent on an unbounded-only
    /// engine is `supported`; a bounded extent on it is `unsupported`
    /// `unsupported-requested-capability` naming the item's kind.
    #[trace("TC-1741", "FSpec-FR-711-AC-2")]
    #[test]
    fn an_unbounded_only_engine_settles_each_extent_by_the_mode_table() {
        let (registry, negotiation) = world(&[("en-1", &[Mode::Unbounded])]);
        let (disposition, _) =
            settle_over(&registry, &negotiation, &item(UNBOUNDED_UNBOUNDABLE, None));
        assert_eq!(
            disposition,
            Disposition::Supported(Candidate::new(BackendId::new("en-1")))
        );
        let (disposition, diagnostic) = settle_over(
            &registry,
            &negotiation,
            &item(ExtentClassification::Bounded, None),
        );
        assert_eq!(disposition, Disposition::Unsupported);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("unsupported_projection", "unsupported-requested-capability")
        );
        assert_eq!(diagnostic.map(|d| d.payload), Some(Payload::Kind(KIND)));
    }

    /// FR-022-AC-8: the candidate rows. No candidate is `unsupported` naming
    /// the kind; several candidates are ambiguous with no named backend and
    /// inconsistent with one; an unregistered named backend is
    /// `unknown-backend`.
    #[trace("TC-1741", "FSpec-FR-711-AC-1")]
    #[test]
    fn the_candidate_rows_settle_as_the_table_states() {
        let (empty, none) = world(&[]);
        let extent = ExtentClassification::Bounded;
        let (disposition, diagnostic) = settle_over(&empty, &none, &item(extent, None));
        assert_eq!(disposition, Disposition::Unsupported);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("unsupported_projection", "unsupported-requested-capability")
        );
        assert_eq!(diagnostic.map(|d| d.payload), Some(Payload::Kind(KIND)));

        let both = [Mode::Bounded, Mode::Unbounded];
        let (registry, negotiation) = world(&[("en-1", &both), ("en-9", &both)]);
        let (disposition, diagnostic) = settle_over(&registry, &negotiation, &item(extent, None));
        assert_eq!(disposition, Disposition::InvalidRequest);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("invalid_capability", "ambiguous-backend")
        );

        let missing = BackendId::new("no-such-backend");
        let (disposition, diagnostic) = settle_over(
            &registry,
            &negotiation,
            &item(extent, Some("no-such-backend")),
        );
        assert_eq!(disposition, Disposition::InvalidRequest);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("invalid_capability", "unknown-backend")
        );
        assert_eq!(
            diagnostic.map(|d| d.payload),
            Some(Payload::Backend(missing))
        );

        let (disposition, diagnostic) =
            settle_over(&registry, &negotiation, &item(extent, Some("en-9")));
        assert_eq!(
            disposition,
            Disposition::Supported(Candidate::new(BackendId::new("en-9")))
        );
        assert!(diagnostic.is_none());
    }

    /// FSpec FR-711-AC-5: for the real engines' manifest and each extent, the
    /// driver's disposition equals the one CG's item entry returns for the
    /// same manifest and request item.
    #[trace("TC-1741", "FSpec-FR-711-AC-5")]
    #[test]
    fn analyze_dispositions_equal_the_item_entrys() {
        let negotiation = AnalyzeNegotiation::fsl().expect("the engines register");
        let mut run = Engines::fsl().expect("engines");
        for engine in run_engines(Vec::new()).expect("the hybrid engine registers") {
            run.with(engine).expect("a new backend");
        }
        let run_ids: Vec<&str> = run.registrations().map(|d| d.id().as_str()).collect();
        let manifest_ids: Vec<&str> = negotiation
            .manifest()
            .iter()
            .map(|entry| entry.identity.as_str())
            .collect();
        assert_eq!(
            run_ids, manifest_ids,
            "the manifest names the backends of the registry a run builds"
        );
        let mut registry = Registry::new();
        for descriptor in run.registrations() {
            registry
                .register(descriptor.clone())
                .expect("a new identity");
        }
        let kinds = [
            Capability::TemporalSatisfaction,
            Capability::ValueValidity,
            Capability::OperationContract,
            Capability::ProbabilisticSatisfaction,
            Capability::Schedulability,
        ];
        let extents = [
            ExtentClassification::Bounded,
            UNBOUNDED_BOUNDABLE,
            UNBOUNDED_UNBOUNDABLE,
        ];
        for kind in kinds {
            for extent in extents {
                for named in [None, Some("en-1"), Some("no-such-backend")] {
                    let mut claim = item(extent, named);
                    claim.kind = kind;
                    let outcome = registry.candidates(kind, claim.backend.as_ref());
                    let (disposition, _) = negotiation.negotiate(&claim, &outcome);
                    let entry =
                        cg::negotiate_item(negotiation.manifest(), &request_item(&claim, &outcome))
                            .expect("no Kani form is read");
                    let expected = match entry {
                        cg::Disposition::Supported { backend } => {
                            Disposition::Supported(Candidate::new(BackendId::new(backend)))
                        }
                        cg::Disposition::RequiresBound { .. } => Disposition::RequiresBound,
                        cg::Disposition::Unsupported { .. } => Disposition::Unsupported,
                        cg::Disposition::InvalidRequest { .. } => Disposition::InvalidRequest,
                    };
                    assert_eq!(disposition, expected, "{kind:?} {extent:?} {named:?}");
                }
            }
        }
    }
}
