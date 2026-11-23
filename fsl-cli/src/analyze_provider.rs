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
use fsl_analyze::provider::{
    ExternalProvider, ProviderRequest, ProviderTarget, ProviderVerdict, UnusableCounterexample,
};
use fsl_foundation::{BackendId, RequestIndex};
use fsl_replay::value_check::{ParameterDomain, ValueTarget};
use fsl_replay::WitnessValue;
use fsl_semantics::check::wire_id::WireGraph;

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
        let package = &PackageGraph::new(request.package().bytes());
        Some(
            request
                .items()
                .iter()
                .zip(request.targets())
                .map(|(item, target)| match target {
                    ProviderTarget::Value(target) => {
                        verdict(&results.items, item.index(), backend, target, package)
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
    package: &PackageGraph<'_>,
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

/// The package graph of one exchange, parsed on first use and shared by every
/// refuted item of the exchange.
struct PackageGraph<'a> {
    bytes: &'a [u8],
    graph: std::cell::OnceCell<Result<WireGraph, ()>>,
}

impl<'a> PackageGraph<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            graph: std::cell::OnceCell::new(),
        }
    }

    fn get(&self) -> Result<&WireGraph, Unreadable> {
        self.graph
            .get_or_init(|| WireGraph::of_package_bytes(self.bytes).map_err(|_| ()))
            .as_ref()
            .map_err(|()| Unreadable::Unusable(UnusableCounterexample::PackageGraphUnreadable))
    }
}

/// The provider's reported assignment, in the claim's parameter order, from
/// the `counterexamples` entry of the item's request index and backend.
///
/// A provider that reports no entry has no refutation to read, and a
/// malformed witness is no refutation (FSL FR-281): neither says more. A
/// well-formed entry the driver cannot map to the claim's parameters is handed
/// to FSL as a candidate that is no assignment of the target, which FSL
/// settles `inconclusive` with cause `replay-parity` in the result document
/// (FSL `confirm_reported`); no text of the provider's reaches any stream.
fn counterexample(
    index: RequestIndex,
    backend: &BackendId,
    target: &ValueTarget<'_>,
    response: &serde_json::Value,
    package: &PackageGraph<'_>,
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
    let parameters: Vec<ClaimParameter> = target
        .parameters()
        .iter()
        .map(|parameter| ClaimParameter {
            key: parameter.wire(),
            kind: ParameterKind::of(parameter.domain()),
        })
        .collect();
    match assignment_values(entry, package, &parameters) {
        Ok(values) => Some(ProviderVerdict::Refuted {
            counterexample: values,
        }),
        Err(Unreadable::Malformed) => None,
        // FSL settles the item `inconclusive`/`replay-parity` without replay
        // and writes the typed reason as the item's diagnostic (FSL FR-290).
        Err(Unreadable::Unusable(reason)) => Some(ProviderVerdict::RefutedUnusable { reason }),
    }
}

/// One parameter of the claim, as the assignment mapping checks it.
struct ClaimParameter {
    key: NodeKey,
    kind: ParameterKind,
}

/// The kind of value a claim parameter takes from a provider's assignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParameterKind {
    Boolean,
    Integer,
    /// A domain the value-validity arm does not read an assignment for.
    Other,
}

impl ParameterKind {
    fn of(domain: &ParameterDomain) -> Self {
        match domain {
            ParameterDomain::Boolean => Self::Boolean,
            ParameterDomain::Integers(_) | ParameterDomain::Unbounded => Self::Integer,
            _ => Self::Other,
        }
    }

    fn admits(self, value: &Value) -> bool {
        matches!(
            (self, value),
            (Self::Boolean, Value::Boolean(_))
                | (Self::Integer, Value::Integer(_))
                | (Self::Other, _)
        )
    }
}

/// Why a provider's counterexample entry gives no refutation.
#[derive(Debug)]
enum Unreadable {
    /// The provider's own data is malformed: not a refutation, nothing more.
    Malformed,
    /// Well-formed data the driver cannot map to the claim. The reason is
    /// FSL's closed FR-290 set; it never carries text the provider supplied.
    Unusable(UnusableCounterexample),
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
/// [`Unreadable::Malformed`] for an unreadable assignment or a value that is
/// not an `Integer` or `Boolean`; [`Unreadable::Unusable`] for an unreadable
/// package graph, an id that does not resolve, a node that is not a parameter
/// of the claim, or a claim parameter the provider assigns no value.
fn assignment_values(
    entry: &serde_json::Value,
    package: &PackageGraph<'_>,
    parameters: &[ClaimParameter],
) -> Result<Vec<Value>, Unreadable> {
    let entry = structural_entry(entry, package)?;
    let assignments =
        fsl_replay::read_plugin_assignments(&entry).map_err(|_| Unreadable::Malformed)?;
    if assignments.iter().any(|assignment| {
        !parameters
            .iter()
            .any(|parameter| parameter.key == assignment.parameter)
    }) {
        return Err(Unreadable::Unusable(
            UnusableCounterexample::AssignmentNotClaimParameter,
        ));
    }
    let mut assigned = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let assignment = assignments
            .iter()
            .find(|assignment| assignment.parameter == parameter.key)
            .ok_or(Unreadable::Unusable(
                UnusableCounterexample::ClaimParameterUnassigned,
            ))?;
        assigned.push((parameter.kind, &assignment.value));
    }
    assigned
        .into_iter()
        .map(|(kind, assigned)| {
            value(assigned)
                .filter(|value| kind.admits(value))
                .ok_or(Unreadable::Unusable(
                    UnusableCounterexample::ParameterValueKindMismatch,
                ))
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
    package: &PackageGraph<'_>,
) -> Result<serde_json::Value, Unreadable> {
    let mut entry = entry.clone();
    if let Some(values) = entry
        .get_mut("counterexample")
        .and_then(|counterexample| counterexample.get_mut("values"))
        .and_then(serde_json::Value::as_array_mut)
    {
        for assignment in values {
            // A parameter that is no node id is the reader's to refuse as malformed.
            if let Some(parameter) = assignment.get_mut("parameter").filter(|p| p.is_object()) {
                *parameter = package.get()?.structural_value(parameter).map_err(|_| {
                    Unreadable::Unusable(UnusableCounterexample::NodeIdUnresolved)
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
    use super::{
        assignment_values, ClaimParameter, PackageGraph, ParameterKind, Unreadable,
        UnusableCounterexample,
    };
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

    fn entry(parameters: &[&serde_json::Value]) -> serde_json::Value {
        let values: Vec<_> = parameters
            .iter()
            .map(|parameter| {
                serde_json::json!({"parameter": parameter,
                    "value": {"type": "integer", "value": "0"}})
            })
            .collect();
        serde_json::json!({"request_index": 0, "backend": "p", "counterexample": {
            "type": "assignments", "values": values}})
    }

    fn key_of(bytes: &[u8], id: &serde_json::Value) -> NodeKey {
        let graph = fsl_semantics::check::wire_id::WireGraph::of_package_bytes(bytes)
            .expect("package graph");
        let structural = graph.structural_value(id).expect("structural id");
        fsl_foundation::node_wire::node_key_from_json(&structural).expect("node key")
    }

    fn claim(keys: &[NodeKey]) -> Vec<ClaimParameter> {
        keys.iter()
            .map(|key| ClaimParameter {
                key: key.clone(),
                kind: ParameterKind::Integer,
            })
            .collect()
    }

    fn unusable(result: Result<Vec<Value>, Unreadable>) -> UnusableCounterexample {
        match result {
            Err(Unreadable::Unusable(reason)) => reason,
            other => panic!("expected an unusable counterexample, got {other:?}"),
        }
    }

    /// Trace: FR-022-AC-9; a provider assignment naming the parameter by its
    /// document-held id is read as the refutation's value.
    #[trace("TC-036", "FR-022-AC-9")]
    #[test]
    fn a_held_form_parameter_is_read_as_a_refutation() {
        let (bytes, parameters) = package();
        let graph = PackageGraph::new(&bytes);
        let claim = claim(&[key_of(&bytes, &parameters[0])]);
        let values = assignment_values(&entry(&[&parameters[0]]), &graph, &claim)
            .expect("the assignment is read");
        assert!(
            matches!(values.as_slice(), [Value::Integer(value)] if *value == Integer::from(0_i64)),
            "{values:?}"
        );
    }

    /// Trace: FR-022-AC-13; an assignment naming a node that is not a
    /// parameter of the claim, or no node of the package, is unusable with
    /// FSL's typed reason.
    #[trace("TC-036", "FR-022-AC-13")]
    #[test]
    fn an_assignment_naming_another_node_is_unusable_with_a_reason() {
        let (bytes, parameters) = package();
        let graph = PackageGraph::new(&bytes);
        let claim = claim(&[key_of(&bytes, &parameters[0])]);
        let reason = unusable(assignment_values(&entry(&[&parameters[1]]), &graph, &claim));
        assert_eq!(reason, UnusableCounterexample::AssignmentNotClaimParameter);
        let outside = serde_json::json!({"ordinal": 0, "package": "held", "path": ["value", {"node": 999_999}]});
        let reason = unusable(assignment_values(&entry(&[&outside]), &graph, &claim));
        assert_eq!(reason, UnusableCounterexample::NodeIdUnresolved);
    }

    /// Trace: FR-022-AC-13; a claim parameter the provider assigns no value is
    /// named apart from an assignment to a node that is no parameter.
    #[trace("TC-036", "FR-022-AC-13")]
    #[test]
    fn a_claim_parameter_without_an_assignment_is_unusable_with_its_own_reason() {
        let (bytes, parameters) = package();
        let graph = PackageGraph::new(&bytes);
        let claim = claim(&[
            key_of(&bytes, &parameters[0]),
            key_of(&bytes, &parameters[1]),
        ]);
        let reason = unusable(assignment_values(&entry(&[&parameters[0]]), &graph, &claim));
        assert_eq!(reason, UnusableCounterexample::ClaimParameterUnassigned);
    }

    /// Trace: FR-022-AC-13; an assigned value of another kind than its
    /// parameter is unusable with its own reason, after an unassigned
    /// parameter by FSL FR-290's priority.
    #[trace("TC-036", "FR-022-AC-13")]
    #[test]
    fn a_value_of_the_wrong_kind_is_unusable_with_its_own_reason() {
        let (bytes, parameters) = package();
        let graph = PackageGraph::new(&bytes);
        let keys = [
            key_of(&bytes, &parameters[0]),
            key_of(&bytes, &parameters[1]),
        ];
        let boolean = |parameter: &serde_json::Value| {
            serde_json::json!({"parameter": parameter, "value": {"type": "boolean", "value": true}})
        };
        let entry = serde_json::json!({"request_index": 0, "backend": "p", "counterexample": {
            "type": "assignments", "values": [
                boolean(&parameters[0]),
                serde_json::json!({"parameter": parameters[1],
                    "value": {"type": "integer", "value": "0"}}),
            ]}});
        let reason = unusable(assignment_values(&entry, &graph, &claim(&keys)));
        assert_eq!(reason, UnusableCounterexample::ParameterValueKindMismatch);
        let mixed = [
            ClaimParameter {
                key: keys[0].clone(),
                kind: ParameterKind::Boolean,
            },
            ClaimParameter {
                key: keys[1].clone(),
                kind: ParameterKind::Integer,
            },
        ];
        let values = assignment_values(&entry, &graph, &mixed).expect("each kind matches");
        assert_eq!(values.len(), 2);
        let both_missing = unusable(assignment_values(
            &self::entry(&[&parameters[1]]),
            &graph,
            &mixed,
        ));
        assert_eq!(both_missing, UnusableCounterexample::ClaimParameterUnassigned);
    }

    /// Trace: FR-022-AC-13; a provider id carrying an ESC byte and ten
    /// thousand characters reaches no reason: the reason is one of FSL's
    /// closed spellings.
    #[trace("TC-036", "FR-022-AC-13")]
    #[test]
    fn provider_text_never_reaches_a_reason() {
        let (bytes, parameters) = package();
        let graph = PackageGraph::new(&bytes);
        let claim = claim(&[key_of(&bytes, &parameters[0])]);
        let hostile = format!("held\u{1b}[31m{}", "A".repeat(10_000));
        let ids = [
            serde_json::json!({"ordinal": 0, "package": hostile, "path": ["value", {"node": 999_999}]}),
            serde_json::json!({"ordinal": 0, "package": "held", "path": ["value", {"node": 999_999}, hostile]}),
        ];
        for id in &ids {
            let reason = unusable(assignment_values(&entry(&[id]), &graph, &claim));
            assert_eq!(reason, UnusableCounterexample::NodeIdUnresolved);
            assert_eq!(reason.as_str(), "node-id-unresolved");
        }
    }
}
