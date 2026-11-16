// SPDX-License-Identifier: AGPL-3.0-or-later
//! Configured process providers for `fsl analyze` (FSL FR-635, FR-287).
//!
//! The driver starts each `--provider-config` entry, admits its `hello`
//! manifest with `Process` origin, and hands FSL one
//! [`Engine::external`](fsl_analyze::analyze::Engine::external) per provider
//! through `fsl_command::command::analyze_with_providers`. FSL negotiates;
//! an item settled on a provider calls [`SessionProvider::check`], which
//! writes the FSpec FR-331 request item, sends it over the started session,
//! and reads the provider's verdict. FSL replays a reported counterexample
//! before the item can settle `refuted` (FSL FR-281).

use std::sync::{Arc, Mutex};

use filament_driver::{ProcessProviderSession as _, ProviderManifest};
use filament_exact::{Cancel, Integer, Value};
use fsl_analyze::external_provider::{ExternalProvider, ProviderRequest, ProviderVerdict};
use fsl_replay::WitnessValue;

use crate::process_provider::{Fr331ResultKind, ProcessProviderItemOutcome};

/// The emitted-package ceiling of one provider request: the default
/// `emit.output_bytes` the driver uses for every package it hands on.
const PACKAGE_BYTES: u64 = 64 * 1024 * 1024;

/// One started session, shared between FSL's engine and the driver, which
/// shuts it down once the run ends.
pub(crate) type SharedSession = Arc<Mutex<Option<crate::cli::CliProviderSession>>>;

/// A started process provider as FSL's external decision procedure.
pub(crate) struct SessionProvider {
    pub(crate) session: SharedSession,
    /// The FR-331 manifest snapshot of the registry the run negotiates
    /// against: the linked engines and every configured provider.
    pub(crate) manifest: Arc<[ProviderManifest]>,
    /// The entry's provider deadline (FSpec FR-462).
    pub(crate) deadline_ms: u64,
}

impl std::fmt::Debug for SessionProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SessionProvider")
            .field("deadline_ms", &self.deadline_ms)
            .finish_non_exhaustive()
    }
}

impl ExternalProvider for SessionProvider {
    fn check(&self, request: &ProviderRequest<'_, '_>, cancel: &Cancel) -> ProviderVerdict {
        // A request the driver cannot write, a session that fails, and a
        // result that is not a verdict all leave the claim undecided: FSL
        // settles them `inconclusive`, never `proved` (FSL FR-281). An
        // exchange the run's cancellation ended is charged to the handle, so
        // FSL settles the item `incomplete` with the cancellation's cause
        // (FSL FR-635 "Budgets and cancellation").
        self.exchange(request, cancel).unwrap_or_else(|| {
            let _ = cancel.poll();
            ProviderVerdict::Unknown
        })
    }
}

impl SessionProvider {
    fn exchange(
        &self,
        request: &ProviderRequest<'_, '_>,
        cancel: &Cancel,
    ) -> Option<ProviderVerdict> {
        let body = self.request_stage(request)?;
        let mut guard = self.session.lock().ok()?;
        let session = guard.as_mut()?;
        let results = session.exchange(&body, cancel).ok()?;
        let index = request.index().get();
        let outcome = results
            .items
            .into_iter()
            .find(|item| item.request_index == index)?
            .result
            .ok()?;
        match outcome.item {
            ProcessProviderItemOutcome::Result {
                kind: Fr331ResultKind::Proved,
                ..
            } => Some(ProviderVerdict::Proved),
            ProcessProviderItemOutcome::Result {
                kind: Fr331ResultKind::Refuted,
                ..
            } => counterexample(request, &outcome.response),
            _ => Some(ProviderVerdict::Unknown),
        }
    }

    /// The FR-331 request stage for the one item FSL negotiated to this
    /// provider: the package's v2 bytes, the registry's manifest snapshot,
    /// and the item at FSL's request index with the routed backend as its
    /// one candidate.
    fn request_stage(
        &self,
        request: &ProviderRequest<'_, '_>,
    ) -> Option<crate::backend_provider::RequestStage> {
        let package = request.package();
        let emission =
            fsl_package::emit_checked(package, fsl_package::EmitLimits::new(PACKAGE_BYTES)).ok()?;
        let session = self.session.lock().ok()?;
        let descriptor = session.as_ref()?.descriptor().clone();
        drop(session);
        let mut registry = fsl_route::Registry::new();
        registry.register(descriptor).ok()?;
        let item = fsl_route::request::items_from_requirements(
            package.graph().requirements(),
            &registry,
            Some(request.backend()),
        )
        .into_iter()
        .find(|item| item.occurrence() == request.claim())?;
        let mut item =
            crate::backend_provider::RequestItem::from_fsl(&item, request.backend()).ok()?;
        item.request_index = u64::try_from(request.index().get()).ok()?;
        crate::backend_provider::RequestStage::from_emitted(
            emission.package(),
            &self.manifest,
            self.deadline_ms,
            vec![item],
        )
        .ok()
    }
}

/// The provider's reported assignment, in the claim's parameter order, from
/// the `counterexamples` entry of the item's request index and backend.
/// A refutation with no readable integer assignment is not a verdict.
fn counterexample(
    request: &ProviderRequest<'_, '_>,
    response: &serde_json::Value,
) -> Option<ProviderVerdict> {
    let index = u64::try_from(request.index().get()).ok()?;
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
                    == Some(request.backend().as_str())
        })?;
    let assignments = fsl_replay::read_plugin_assignments(entry).ok()?;
    let values = request
        .target()
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
