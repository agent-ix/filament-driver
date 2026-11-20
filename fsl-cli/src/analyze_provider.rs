// SPDX-License-Identifier: AGPL-3.0-or-later
//! Configured process providers for `fsl analyze` (FSL FR-635, FR-287).
//!
//! The driver starts each `--provider-config` entry, admits its `hello`
//! manifest with `Process` origin, and hands FSL one
//! [`Engine::external`](fsl_analyze::analyze::Engine::external) per provider
//! through `fsl_command::command::analyze_with_providers`. FSL routes,
//! negotiates and builds the FSpec FR-331 request for the items it settled on
//! a provider; [`SessionProvider::settle`] only transports it: it forwards
//! the request through [`filament_driver::ProcessProviderSession::invoke`] over the started
//! session and maps the FR-331 response back to one verdict per item. The
//! driver holds no package or routed item of its own. FSL replays a reported
//! counterexample before the item can settle `refuted` (FSL FR-281).

use std::sync::{Arc, Mutex};

use filament_driver::{ProcessProviderRequest, ProcessProviderSession as _, ProviderManifest};
use filament_exact::{Cancel, Integer, Value};
use fsl_analyze::provider::{ExternalProvider, ProviderRequest, ProviderVerdict};
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
                .map(|(item, target)| {
                    verdict(&results.items, item.index(), backend, target)
                        .unwrap_or(ProviderVerdict::Unknown)
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
