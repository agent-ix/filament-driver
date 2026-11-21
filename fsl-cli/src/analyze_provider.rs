// SPDX-License-Identifier: AGPL-3.0-or-later
//! Configured process providers for `fsl analyze`.
//!
//! Value providers map admitted result assignments to FSL verdicts. Hybrid
//! providers carry FSL's opaque request and result bytes through the same
//! started session. FSL owns negotiation, certificate admission and replay;
//! the driver closes each shared session after the run.

use std::sync::{Arc, Mutex};

use filament_driver::{ProcessProviderRequest, ProcessProviderSession as _, ProviderManifest};
use filament_exact::{Cancel, Integer, Value};
use fsl_analyze::provider::{ExternalProvider, ProviderRequest, ProviderTarget, ProviderVerdict};
use fsl_foundation::{BackendId, RequestIndex};
use fsl_replay::value_check::ValueTarget;
use fsl_replay::WitnessValue;

use crate::process_provider::{
    Fr331ResultKind, ProcessProviderItemOutcome, ProcessProviderOutcome,
};

/// The one capability kind an external provider decides (FSL FR-281).
const VALUE_VALIDITY: &str = "value-validity";

/// One started session, shared between FSL's engine and the driver, which
/// shuts it down once the run ends.
pub(crate) type SharedSession = Arc<Mutex<Option<crate::cli::CliProviderSession>>>;

/// A started process provider as FSL's external decision procedure.
pub(crate) struct SessionProvider {
    pub(crate) session: SharedSession,
    /// The FR-331 manifest snapshot of the registry the run negotiates
    /// against: the linked engines and every configured provider.
    pub(crate) manifest: Arc<[ProviderManifest]>,
}

impl std::fmt::Debug for SessionProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SessionProvider")
            .finish_non_exhaustive()
    }
}

impl ExternalProvider for SessionProvider {
    fn settle(&self, request: &ProviderRequest<'_>, cancel: &Cancel) -> Vec<ProviderVerdict> {
        // A session that fails and a response FSL's request cannot be read
        // from leave every item undecided: FSL settles them `inconclusive`,
        // never `proved` (FSL FR-281). An exchange the run's cancellation
        // ended is charged to the handle, so FSL settles the items
        // `incomplete` with the cancellation's cause (FSL FR-635 "Budgets and
        // cancellation").
        self.exchange(request, cancel).unwrap_or_else(|| {
            let _ = cancel.poll();
            vec![ProviderVerdict::Unknown; request.items().len()]
        })
    }
}

impl SessionProvider {
    fn exchange(
        &self,
        request: &ProviderRequest<'_>,
        cancel: &Cancel,
    ) -> Option<Vec<ProviderVerdict>> {
        let mut guard = self.session.lock().ok()?;
        let session = guard.as_mut()?;
        let results = session
            .invoke(ProcessProviderRequest {
                request,
                manifest: &self.manifest,
                cancel,
            })
            .ok()?;
        let backend = request.backend();
        Some(
            request
                .items()
                .iter()
                .zip(request.targets())
                .map(|(item, target)| match target {
                    ProviderTarget::Value(target) => {
                        verdict(&results.items, item.index(), backend, target)
                            .unwrap_or(ProviderVerdict::Unknown)
                    }
                    // This adapter admits only value-validity verdicts; a
                    // contract target has no value-parameter assignment here.
                    ProviderTarget::Contract(_) => ProviderVerdict::Unknown,
                    ProviderTarget::Composition(_) => {
                        composition_verdict(&results.items, item.index())
                            .unwrap_or(ProviderVerdict::Unknown)
                    }
                })
                .collect(),
        )
    }
}

/// The verdict the response gives the item at `index`: `proved`, a
/// `refuted` with a readable assignment, or nothing.
fn verdict(
    results: &[filament_driver::ProviderItemResult<ProcessProviderOutcome, String>],
    index: RequestIndex,
    backend: &BackendId,
    target: &ValueTarget<'_>,
) -> Option<ProviderVerdict> {
    let outcome = results
        .iter()
        .find(|item| item.request_index == index.get())?
        .result
        .as_ref()
        .ok()?;
    // A result for another capability kind than the item's answers nothing
    // (FSpec FR-462-AC-4).
    match &outcome.item {
        ProcessProviderItemOutcome::Result {
            capability_kind,
            kind: Fr331ResultKind::Proved,
        } if capability_kind == VALUE_VALIDITY => Some(ProviderVerdict::Proved),
        ProcessProviderItemOutcome::Result {
            capability_kind,
            kind: Fr331ResultKind::Refuted,
        } if capability_kind == VALUE_VALIDITY => {
            counterexample(index, backend, target, &outcome.response)
        }
        _ => None,
    }
}

fn composition_verdict(
    results: &[filament_driver::ProviderItemResult<ProcessProviderOutcome, String>],
    index: RequestIndex,
) -> Option<ProviderVerdict> {
    let outcome = results
        .iter()
        .find(|item| item.request_index == index.get())?
        .result
        .as_ref()
        .ok()?;
    match &outcome.item {
        ProcessProviderItemOutcome::Result {
            capability_kind,
            kind,
        } if capability_kind == "composition" => match kind {
            Fr331ResultKind::Proved => Some(ProviderVerdict::Proved),
            // FSL owns the explicit no-replay settlement for composition.
            Fr331ResultKind::Refuted => Some(ProviderVerdict::Refuted {
                counterexample: Vec::new(),
            }),
            _ => None,
        },
        _ => None,
    }
}

/// The provider's reported assignment, in the claim's parameter order, from
/// the `counterexamples` entry of the item's request index and backend.
/// A refutation with no readable integer assignment is not a verdict.
fn counterexample(
    index: RequestIndex,
    backend: &BackendId,
    target: &ValueTarget<'_>,
    response: &serde_json::Value,
) -> Option<ProviderVerdict> {
    let index = u64::try_from(index.get()).ok()?;
    let entry = response
        .get("counterexamples")?
        .as_array()?
        .iter()
        .find(|entry| {
            entry
                .get("request_index")
                .and_then(serde_json::Value::as_u64)
                == Some(index)
                && entry.get("backend").and_then(serde_json::Value::as_str)
                    == Some(backend.as_str())
        })?;
    let assignments = fsl_replay::read_plugin_assignments(entry).ok()?;
    let values = target
        .parameters()
        .iter()
        .map(|parameter| {
            let wire = parameter.wire();
            let assignment = assignments
                .iter()
                .find(|assignment| assignment.parameter == wire)?;
            value(&assignment.value)
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ProviderVerdict::Refuted {
        counterexample: values,
    })
}

/// The runtime value of an assignment over the provider arm's `Integer`
/// and `Boolean` parameters (FSL FR-495).
fn value(value: &WitnessValue) -> Option<Value> {
    match value {
        WitnessValue::Integer(integer) => Some(Value::Integer(Integer::from(*integer))),
        WitnessValue::ExactInteger(integer) => integer.to_string().parse().ok().map(Value::Integer),
        WitnessValue::Boolean(boolean) => Some(Value::Boolean(*boolean)),
        _ => None,
    }
}

/// An opaque hybrid exchange sharing the driver's admitted session lifetime.
pub(crate) struct HybridSessionProvider {
    pub(crate) session: SharedSession,
}

impl fsl_analyze::analyze::HybridProcessExchange for HybridSessionProvider {
    fn exchange(
        &self,
        request: &[u8],
        cancel: &Cancel,
    ) -> Result<Vec<u8>, fsl_analyze::analyze::HybridExchangeError> {
        let crash = || {
            fsl_analyze::analyze::HybridExchangeError::ToolFailure(fsl_replay::ToolFailure::Crash)
        };
        let mut guard = self.session.lock().map_err(|_| crash())?;
        let session = guard.as_mut().ok_or_else(crash)?;
        session.exchange_hybrid(request, cancel)
    }
}

impl std::fmt::Debug for HybridSessionProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HybridSessionProvider")
            .finish_non_exhaustive()
    }
}
