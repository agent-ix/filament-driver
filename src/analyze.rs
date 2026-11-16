// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's negotiation of `analyze` items (FSL FR-281 "Negotiated
//! items", FR-620 "Resolving items").
//!
//! FSL's `analyze` never negotiates: the caller settles each resolved item
//! from the candidate set FSL computed over its registry, under FSpec
//! FR-290's candidate table, and FSL routes a `supported` item to its one
//! candidate. [`AnalyzeNegotiation`] is that caller-side settlement. It reads
//! the candidate outcome through `fsl_route::settle_candidates`, the one
//! statement of the table, and never chooses among several candidates.

use filament_semantic_value::catalog::CatalogCode;
use fsl_analyze::analyze::document::Negotiate;
use fsl_analyze::analyze::{ClaimItem, Diagnostic, Payload};
use fsl_route::routing::Disposition;
use fsl_route::{settle_candidates, CandidateOutcome, NegotiationCause, NegotiationSettlement};
use fsl_semantics::check::Capability;

/// The FSpec FR-290 settlement the driver gives each `analyze` item.
///
/// | Candidate outcome | Disposition | Diagnostic |
/// | --- | --- | --- |
/// | exactly one candidate | `supported` on it | none |
/// | no candidate | `unsupported` | `unsupported_projection`/`unsupported-requested-capability`, the item's kind |
/// | several candidates, no backend named | `invalid-request` | `invalid_capability`/`ambiguous-backend` |
/// | several candidates, a backend named | `invalid-request` | `invalid_capability`/`inconsistent-candidates`, the named backend |
/// | a named backend the registry does not hold | `invalid-request` | `invalid_capability`/`unknown-backend`, the backend |
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AnalyzeNegotiation;

impl Negotiate for AnalyzeNegotiation {
    fn negotiate(
        &self,
        item: &ClaimItem,
        candidates: &CandidateOutcome,
    ) -> (Disposition, Option<Diagnostic>) {
        settle(item.kind, candidates)
    }
}

/// The settlement of an item of `kind` over `candidates`, as the table on
/// [`AnalyzeNegotiation`] states.
fn settle(kind: Capability, candidates: &CandidateOutcome) -> (Disposition, Option<Diagnostic>) {
    let named = match candidates {
        CandidateOutcome::Candidates(set) => set.requested_backend(),
        CandidateOutcome::UnknownBackend(_) => None,
    };
    // FR-281 names CG's `negotiate_*` as the settlement point, but CG has no
    // arm for linked FSL engines yet (QSL-2286), so the FR-290 table is read
    // here without its extent rows.
    match settle_candidates(candidates, kind, named) {
        NegotiationSettlement::Single(backend) => (
            Disposition::Supported(fsl_route::Candidate::new(backend)),
            None,
        ),
        NegotiationSettlement::Inconsistent => (
            Disposition::InvalidRequest,
            Some(Diagnostic {
                code: CatalogCode::new("invalid_capability", "inconsistent-candidates"),
                payload: named.map_or(Payload::None, |id| Payload::Backend(id.clone())),
            }),
        ),
        NegotiationSettlement::Cause(cause) => {
            let code = cause.code();
            match cause {
                NegotiationCause::UnknownBackend(backend) => (
                    Disposition::InvalidRequest,
                    Some(Diagnostic {
                        code,
                        payload: Payload::Backend(backend),
                    }),
                ),
                NegotiationCause::AmbiguousBackend(_) => {
                    (Disposition::InvalidRequest, Some(Diagnostic::new(code)))
                }
                NegotiationCause::UnsupportedRequestedCapability(kind, _) => (
                    Disposition::Unsupported,
                    Some(Diagnostic {
                        code,
                        payload: Payload::Kind(kind),
                    }),
                ),
            }
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
    use fsl_foundation::BackendId;
    use fsl_route::{BackendDescriptor, Candidate, Mode, ProviderOrigin, Registry};
    use ix_trace_rs::trace;

    use super::*;

    const KIND: Capability = Capability::TemporalSatisfaction;

    /// A registry holding one unbounded temporal-satisfaction engine per id.
    fn registry(ids: &[&str]) -> Registry {
        let mut registry = Registry::new();
        for id in ids {
            let descriptor = BackendDescriptor::new(
                Candidate::new(BackendId::new(*id)),
                ProviderOrigin::Linked,
                [(KIND, Mode::Unbounded)],
                None,
                None,
            )
            .expect("an unbounded descriptor needs no domains");
            registry.register(descriptor).expect("a new identity");
        }
        registry
    }

    fn code(diagnostic: Option<&Diagnostic>) -> (&str, &str) {
        let diagnostic = diagnostic.expect("a diagnostic");
        (diagnostic.code.code(), diagnostic.code.cause())
    }

    /// FR-022-AC-8: one candidate is `supported` on that candidate, with no
    /// diagnostic.
    #[trace("TC-036", "FR-022-AC-8")]
    #[test]
    fn one_candidate_is_supported_on_it() {
        let (disposition, diagnostic) = settle(KIND, &registry(&["en-1"]).candidates(KIND, None));
        assert_eq!(
            disposition,
            Disposition::Supported(Candidate::new(BackendId::new("en-1")))
        );
        assert!(diagnostic.is_none());
    }

    /// FR-022-AC-8: no candidate is `unsupported`, naming the item's kind.
    #[trace("TC-036", "FR-022-AC-8")]
    #[test]
    fn no_candidate_is_unsupported_naming_the_kind() {
        let (disposition, diagnostic) = settle(KIND, &registry(&[]).candidates(KIND, None));
        assert_eq!(disposition, Disposition::Unsupported);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("unsupported_projection", "unsupported-requested-capability")
        );
        assert_eq!(diagnostic.map(|d| d.payload), Some(Payload::Kind(KIND)));
    }

    /// FR-022-AC-8: several candidates with no named backend are an
    /// ambiguous request; the driver never picks one.
    #[trace("TC-036", "FR-022-AC-8")]
    #[test]
    fn several_candidates_are_ambiguous() {
        let outcome = registry(&["en-1", "en-9"]).candidates(KIND, None);
        let (disposition, diagnostic) = settle(KIND, &outcome);
        assert_eq!(disposition, Disposition::InvalidRequest);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("invalid_capability", "ambiguous-backend")
        );
    }

    /// FR-022-AC-8: a named backend the registry does not hold is
    /// `unknown-backend`, naming it; a held named backend is supported.
    #[trace("TC-036", "FR-022-AC-8")]
    #[test]
    fn a_named_backend_settles_by_whether_it_is_held() {
        let held = registry(&["en-1", "en-9"]);
        let missing = BackendId::new("no-such-backend");
        let (disposition, diagnostic) = settle(KIND, &held.candidates(KIND, Some(&missing)));
        assert_eq!(disposition, Disposition::InvalidRequest);
        assert_eq!(
            code(diagnostic.as_ref()),
            ("invalid_capability", "unknown-backend")
        );
        assert_eq!(
            diagnostic.map(|d| d.payload),
            Some(Payload::Backend(missing))
        );
        let named = BackendId::new("en-9");
        let (disposition, diagnostic) = settle(KIND, &held.candidates(KIND, Some(&named)));
        assert_eq!(disposition, Disposition::Supported(Candidate::new(named)));
        assert!(diagnostic.is_none());
    }
}
