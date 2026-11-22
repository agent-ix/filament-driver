// SPDX-License-Identifier: AGPL-3.0-or-later
//! Configured process providers for `fsl analyze`.
//!
//! Value providers map admitted result assignments to FSL verdicts. Hybrid
//! providers carry FSL's opaque request and result bytes through the same
//! started session. FSL owns negotiation, certificate admission and replay;
//! the driver closes each shared session after the run.

use std::sync::{Arc, Mutex};

use filament_driver::{ProcessProviderRequest, ProcessProviderSession as _, ProviderManifest};
use filament_exact::{Cancel, Integer, NodeKey, Value};
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
        // An unreadable response leaves every item undecided, while an
        // observed process death is retained as a typed tool failure.
        // An exchange the run's cancellation
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
        let results = match session.invoke(ProcessProviderRequest {
            request,
            manifest: &self.manifest,
            cancel,
        }) {
            Ok(results) => results,
            Err(error) => {
                return process_failure(&error).map(|failure| vec![failure; request.items().len()])
            }
        };
        let backend = request.backend();
        Some(
            request
                .items()
                .iter()
                .zip(request.targets())
                .map(|(item, target)| match target {
                    ProviderTarget::Value(target) => verdict(
                        &results.items,
                        item.index(),
                        backend,
                        target,
                        request.package().bytes(),
                    )
                    .unwrap_or(ProviderVerdict::Unknown),
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

fn process_failure<A, V, F>(
    error: &crate::process_provider::AdapterError<A, V, F>,
) -> Option<ProviderVerdict> {
    let crate::process_provider::AdapterError::Request(error) = error else {
        return None;
    };
    let filament_plugin_host::PluginError::Exited(status) = error.primary_cause() else {
        return None;
    };
    if status.success() {
        return None;
    }
    // A signal or an ENOMEM exit observes the process failure, but does not
    // establish which resource limit caused it. Preserve only that evidence.
    Some(ProviderVerdict::ProcessFailed {
        cause: if status.code().is_some() {
            fsl_replay::ToolFailure::NonzeroExit
        } else {
            fsl_replay::ToolFailure::Crash
        },
    })
}

/// The verdict the response gives the item at `index`: `proved`, a
/// `refuted` with a readable assignment, or nothing.
fn verdict(
    results: &[filament_driver::ProviderItemResult<ProcessProviderOutcome, String>],
    index: RequestIndex,
    backend: &BackendId,
    target: &ValueTarget<'_>,
    package: &[u8],
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
            counterexample(index, backend, target, &outcome.response, package)
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
/// A provider that reports no entry has no refutation to read; an entry the
/// driver cannot map to the claim's parameters is reported on stderr and
/// leaves the item undecided, never silently dropped.
fn counterexample(
    index: RequestIndex,
    backend: &BackendId,
    target: &ValueTarget<'_>,
    response: &serde_json::Value,
    package: &[u8],
) -> Option<ProviderVerdict> {
    let wanted = u64::try_from(index.get()).ok()?;
    let entry = response
        .get("counterexamples")?
        .as_array()?
        .iter()
        .find(|entry| {
            entry
                .get("request_index")
                .and_then(serde_json::Value::as_u64)
                == Some(wanted)
                && entry.get("backend").and_then(serde_json::Value::as_str)
                    == Some(backend.as_str())
        })?;
    let parameters: Vec<NodeKey> = target
        .parameters()
        .iter()
        .map(fsl_replay::value_check::TargetParameter::wire)
        .collect();
    match assignment_values(entry, package, &parameters) {
        Ok(values) => Some(ProviderVerdict::Refuted {
            counterexample: values,
        }),
        // A malformed witness is no refutation and says nothing more (FSL FR-281).
        Err(Unreadable::Malformed) => None,
        Err(Unreadable::Unusable(reason)) => {
            crate::cli::emit_diagnostic(
                "ProviderDiagnostic",
                &format!(
                    "provider {backend} refuted request item {wanted} but its counterexample is not usable: {reason}",
                    backend = backend.as_str()
                ),
            );
            None
        }
    }
}

/// Why a provider's counterexample entry gives no refutation.
#[derive(Debug)]
enum Unreadable {
    /// The provider's own data is malformed: not a refutation, and nothing more is said.
    Malformed,
    /// Well-formed data the driver cannot map to the claim; the reason is reported.
    Unusable(String),
}

/// The runtime values of the provider's assignments, one per claim
/// `parameters` entry in order.
///
/// FSL writes node ids in the document-held form (`{"node": i}` index
/// segments into the package graph, FSpec FR-322) and the provider answers in
/// that form, while the assignment reader and the target parameters compare
/// structural keys, so each id is resolved against the package graph first.
///
/// # Errors
///
/// The reason the entry cannot be read: an unreadable package graph, an id
/// that does not resolve, an unreadable assignment, a claim parameter the
/// provider assigns no value, or a value that is not an `Integer` or `Boolean`.
fn assignment_values(
    entry: &serde_json::Value,
    package: &[u8],
    parameters: &[NodeKey],
) -> Result<Vec<Value>, Unreadable> {
    let entry = structural_entry(entry, package)?;
    let assignments =
        fsl_replay::read_plugin_assignments(&entry).map_err(|_| Unreadable::Malformed)?;
    parameters
        .iter()
        .map(|parameter| {
            let assignment = assignments
                .iter()
                .find(|assignment| &assignment.parameter == parameter)
                .ok_or_else(|| {
                    Unreadable::Unusable(
                        "no assignment names a parameter of the claim (a reported node is not one of its parameters)"
                            .to_owned(),
                    )
                })?;
            value(&assignment.value).ok_or(Unreadable::Malformed)
        })
        .collect()
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

/// `entry` with each assignment's `parameter` resolved from its document-held
/// form to the structural node key of the package graph.
fn structural_entry(
    entry: &serde_json::Value,
    package: &[u8],
) -> Result<serde_json::Value, Unreadable> {
    let graph =
        fsl_semantics::check::wire_id::WireGraph::of_package_bytes(package).map_err(|error| {
            Unreadable::Unusable(format!("the package graph is unreadable ({error})"))
        })?;
    let mut entry = entry.clone();
    if let Some(values) = entry
        .get_mut("counterexample")
        .and_then(|counterexample| counterexample.get_mut("values"))
        .and_then(serde_json::Value::as_array_mut)
    {
        for assignment in values {
            // A parameter that is no node id is the reader's to refuse as malformed.
            if let Some(parameter) = assignment.get_mut("parameter").filter(|p| p.is_object()) {
                *parameter = graph.structural_value(parameter).map_err(|error| {
                    Unreadable::Unusable(format!(
                        "a node id does not resolve in the package ({error})"
                    ))
                })?;
            }
        }
    }
    Ok(entry)
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use super::assignment_values;
    use filament_exact::{Cancel, Integer, NodeKey, Value};
    use fsl_foundation::SourceIdentity;
    use fsl_replay::spine::{check, parse, select, DependencyInput, ParseRequest, SpineLimits};
    use ix_trace_rs::trace;

    /// The emitted package of one function over two `Integer` parameters, with
    /// the package JSON's node ids of those parameters.
    fn package() -> (Vec<u8>, Vec<serde_json::Value>) {
        let source = "language \"ix:native\" edition \"1-draft\";\n\
            profile v = \"filament.value.complete/v1\";\n\
            function odd using v(n: Integer, m: Integer): Boolean pure { n = m }\n";
        let limits = SpineLimits::default();
        let cancel = Cancel::new();
        let identity = SourceIdentity::new("test", "held");
        let parsed = parse(
            &ParseRequest {
                source: &identity,
                path: "held.fsl",
                bytes: source.as_bytes(),
            },
            limits.source,
            &cancel,
        )
        .expect("parse")
        .into_value();
        let packages =
            fsl_semantics::model::intake::package_input(Vec::<&[u8]>::new()).expect("no models");
        let selected = select(&parsed, &packages, limits.model, &cancel)
            .expect("select")
            .into_value();
        let checked = check(
            &parsed,
            "held".to_owned(),
            &selected,
            &DependencyInput::default(),
            limits,
            &cancel,
        )
        .expect("check")
        .into_value();
        let emission =
            fsl_package::emit_checked(checked.package(), limits.package.emit()).expect("emit");
        let bytes = emission.package().bytes().to_vec();
        let wire: serde_json::Value = serde_json::from_slice(&bytes).expect("package JSON");
        let parameters: Vec<serde_json::Value> = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .filter(|node| node["semantic_form"] == "parameter")
            .map(|node| node["node_id"].clone())
            .collect();
        assert_eq!(parameters.len(), 2, "two parameter nodes");
        (bytes, parameters)
    }

    fn entry(parameter: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({"request_index": 0, "backend": "p", "counterexample": {
            "type": "assignments",
            "values": [{"parameter": parameter, "value": {"type": "integer", "value": "0"}}]}})
    }

    fn key_of(bytes: &[u8], id: &serde_json::Value) -> NodeKey {
        let graph = fsl_semantics::check::wire_id::WireGraph::of_package_bytes(bytes)
            .expect("package graph");
        let structural = graph.structural_value(id).expect("structural id");
        fsl_foundation::node_wire::node_key_from_json(&structural).expect("node key")
    }

    /// Trace: FR-022-AC-9; a provider assignment naming the parameter by its
    /// document-held id is read as the refutation's value.
    #[trace("TC-036", "FR-022-AC-9")]
    #[test]
    fn a_held_form_parameter_is_read_as_a_refutation() {
        let (bytes, parameters) = package();
        let claim = [key_of(&bytes, &parameters[0])];
        let values = assignment_values(&entry(&parameters[0]), &bytes, &claim)
            .expect("the assignment is read");
        assert!(
            matches!(values.as_slice(), [Value::Integer(value)] if *value == Integer::from(0_i64)),
            "{values:?}"
        );
    }

    /// Trace: FR-022-AC-9; an assignment that names a node which is not a
    /// parameter of the claim, or no node of the package, is refused with its
    /// reason, never dropped.
    #[trace("TC-036", "FR-022-AC-9")]
    #[test]
    fn an_assignment_naming_another_node_is_refused_with_a_reason() {
        let (bytes, parameters) = package();
        let claim = [key_of(&bytes, &parameters[0])];
        let reason = assignment_values(&entry(&parameters[1]), &bytes, &claim)
            .expect_err("another parameter is no assignment of the claim");
        let super::Unreadable::Unusable(reason) = reason else {
            panic!("{reason:?}")
        };
        assert!(reason.contains("not one of its parameters"), "{reason}");
        let reason = assignment_values(
            &entry(&serde_json::json!({"ordinal": 0, "package": "held", "path": ["value", {"node": 999_999}]})),
            &bytes,
            &claim,
        )
        .expect_err("an id outside the graph does not resolve");
        let super::Unreadable::Unusable(reason) = reason else {
            panic!("{reason:?}")
        };
        assert!(reason.contains("does not resolve"), "{reason}");
    }
}
