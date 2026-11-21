// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed driver inputs for FSpec's backend-provider/v1 envelope.
//!
//! This module projects the path-addressed FR-331 contract into driver types;
//! it does not carry a copy of the schema or its vectors. Canonical descriptor
//! bytes are retained so optional manifest members survive admission and can
//! be included unchanged in a later request's registry snapshot.

use std::collections::BTreeMap;
use std::sync::Arc;

use filament_driver::ProviderManifest;
use filament_semantic_value::catalog::{CatalogCode, Category};
use fsl_foundation::backend::EmptyBackendIdentity;
use fsl_foundation::diagnostic::category_of;
use fsl_foundation::BackendId;
use fsl_route::{
    descriptor_from_manifest_with_dynamics, BackendDescriptor, Candidate, DynamicsMember,
    FairnessKindsMember, FairnessLabel, ProviderOrigin, SolverBounds,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CONTRACT_VERSION: &str = "filament.backend-provider/v1";
const CAPABILITY_VOCABULARY: &str = "filament.capability-kind/v1";
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderDescriptorShape {
    backend: String,
    advertises: Vec<AdvertisedPair>,
    #[serde(default, deserialize_with = "deserialize_present")]
    domains: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    dynamics: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    options: Option<BTreeMap<String, String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    bounds: Option<BTreeMap<String, u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    obligation_forms: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    reductions: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    evidence_kinds: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    fairness_kinds: Option<Vec<String>>,
    /// FSpec FR-331 "Backend tool": the code that decides the backend's
    /// results. FSL registers an external provider only with its tool.
    #[serde(default, deserialize_with = "deserialize_present")]
    tool: Option<ToolShape>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolShape {
    name: String,
    version: String,
}

fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdvertisedPair {
    kind: Option<String>,
    mode: Option<String>,
}

fn invalid_member(field: &'static str, detail: impl Into<String>) -> ManifestAdmissionError {
    ManifestAdmissionError::InvalidMember {
        field: field.to_owned(),
        detail: detail.into(),
    }
}

fn validate_shape_set(
    field: &'static str,
    values: Option<&[String]>,
    allowed: &[&str],
) -> Result<(), ManifestAdmissionError> {
    let Some(values) = values else {
        return Ok(());
    };
    if values.is_empty() {
        return Err(invalid_member(field, "must contain at least one value"));
    }
    let mut seen = std::collections::BTreeSet::new();
    if values
        .iter()
        .any(|value| !allowed.contains(&value.as_str()) || !seen.insert(value))
    {
        return Err(invalid_member(
            field,
            "contains an unknown or duplicate value",
        ));
    }
    Ok(())
}

fn validate_manifest(shape: &ProviderDescriptorShape) -> Result<(), ManifestAdmissionError> {
    if shape.advertises.is_empty() {
        return Err(invalid_member("advertises", "must not be empty"));
    }
    let mut advertised = std::collections::BTreeSet::new();
    if shape
        .advertises
        .iter()
        .any(|pair| !advertised.insert((&pair.kind, &pair.mode)))
    {
        return Err(invalid_member("advertises", "contains a duplicate pair"));
    }
    // Domain vocabulary and admission refusal belong to FSL. Keep the
    // typed value for FSL instead of pre-refusing it in the driver.
    if shape.options.as_ref().is_some_and(BTreeMap::is_empty) {
        return Err(invalid_member("options", "must not be empty when present"));
    }
    if let Some(bounds) = &shape.bounds {
        if bounds.is_empty()
            || bounds.iter().any(|(name, value)| {
                !name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                    || !name
                        .bytes()
                        .skip(1)
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
                    || *value > MAX_SAFE_INTEGER
            })
        {
            return Err(invalid_member(
                "bounds",
                "contains an invalid setting name or unsafe integer",
            ));
        }
    }
    // QSL-654 owns evidence-kind admission/refusal. Preserve that field
    // without applying a driver-side vocabulary rule.
    for (field, values, allowed) in [
        (
            "obligation_forms",
            shape.obligation_forms.as_deref(),
            &[
                "unbounded-quantification",
                "induction",
                "compositional-discharge",
                "mathematical-integers",
            ][..],
        ),
        (
            "reductions",
            shape.reductions.as_deref(),
            &["symmetry", "partial-order", "constraint"][..],
        ),
        (
            "fairness_kinds",
            shape.fairness_kinds.as_deref(),
            &["weak", "strong"][..],
        ),
    ] {
        validate_shape_set(field, values, allowed)?;
    }
    Ok(())
}

/// The FR-331 manifest-stage body received in the plugin's hello.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestStage {
    /// FR-331 contract identity.
    pub contract_version: String,
    /// FR-290 capability-vocabulary identity.
    #[serde(default)]
    pub capability_vocabulary: Option<String>,
    /// The plugin's losslessly retained provider-manifest descriptor.
    pub manifest: Vec<Arc<Value>>,
}

/// One successfully admitted process manifest and the exact wire snapshot
/// retained for later FR-331 requests.
#[derive(Clone, Debug)]
pub struct AdmittedManifest {
    descriptor: BackendDescriptor,
    manifest: ProviderManifest,
}

impl AdmittedManifest {
    /// The descriptor admitted once by FSL for this process hello.
    #[must_use]
    pub fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    /// The single driver-owned manifest carrier, including all optional
    /// FR-331 members. Its canonical wire bytes are a fresh retained encoding
    /// of the admitted hello value.
    #[must_use]
    pub fn manifest(&self) -> &ProviderManifest {
        &self.manifest
    }
}

/// An invalid FR-331 hello body or a typed FSL registration refusal.
#[derive(Debug, thiserror::Error)]
pub enum ManifestAdmissionError {
    /// The manifest-stage envelope identity did not match FR-331.
    #[error("plugin hello has an unsupported FR-331 contract identity")]
    ContractIdentity,
    /// The manifest-stage capability vocabulary is not recognized by FSL.
    #[error("{0}")]
    CapabilityVocabulary(CatalogCode),
    /// A provider hello did not carry exactly one descriptor.
    #[error("plugin hello must carry exactly one provider descriptor")]
    DescriptorCount,
    /// A descriptor member violated FR-331 wire shape or type requirements.
    #[error("plugin hello manifest has an invalid {field} member: {detail}")]
    InvalidMember {
        /// The member whose type or shape did not satisfy FR-331.
        field: String,
        /// Parser or semantic-validation detail for the rejected member.
        detail: String,
    },
    /// FSL refused the descriptor's backend identity.
    #[error(transparent)]
    EmptyBackendIdentity(#[from] EmptyBackendIdentity),
    /// FSL refused advertised labels or domain declarations with its native
    /// registration cause.
    #[error(transparent)]
    Registration(#[from] fsl_route::RegistrationRefusal),
    /// Canonical manifest hashing refused the input or exceeded its byte bound.
    #[error("plugin hello manifest could not be canonically encoded: {0}")]
    Canonical(#[from] quire_canonical::Error),
}

impl ManifestStage {
    /// Admit the hello's one manifest using the caller-supplied Process origin.
    ///
    /// The retained bytes use RFC 8785. FSL owns identity,
    /// capability, mode and domain admission; this method neither normalizes
    /// labels nor assigns an origin from wire data. Evidence-label admission
    /// remains FSL-owned without a driver-side vocabulary rule.
    pub fn admit_process(
        &self,
        max_frame_bytes: u64,
    ) -> Result<AdmittedManifest, ManifestAdmissionError> {
        if self.contract_version != CONTRACT_VERSION {
            return Err(ManifestAdmissionError::ContractIdentity);
        }
        if self.capability_vocabulary.as_deref() != Some(CAPABILITY_VOCABULARY) {
            let code = CatalogCode::new("invalid_capability", "unknown-vocabulary");
            debug_assert_eq!(category_of(&code), Some(Category::Refusal));
            return Err(ManifestAdmissionError::CapabilityVocabulary(code));
        }
        let [wire] = self.manifest.as_slice() else {
            return Err(ManifestAdmissionError::DescriptorCount);
        };
        let shape: ProviderDescriptorShape = serde_path_to_error::deserialize((**wire).clone())
            .map_err(|source| ManifestAdmissionError::InvalidMember {
                field: source.path().to_string(),
                detail: source.inner().to_string(),
            })?;
        validate_manifest(&shape)?;
        if shape
            .tool
            .as_ref()
            .is_some_and(|tool| tool.name.is_empty() || tool.version.is_empty())
        {
            return Err(invalid_member("tool", "names a nonempty name and version"));
        }
        let limits = quire_canonical::Limits::new(max_frame_bytes);
        let mut canonical_wire = Vec::new();
        quire_canonical::encode(&mut canonical_wire, &**wire, limits)?;
        let domain_labels = shape
            .domains
            .as_deref()
            .map(|domains| domains.iter().map(String::as_str).collect::<Vec<_>>());
        let fairness_labels = shape
            .fairness_kinds
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|label| FairnessLabel::Text(label))
            .collect::<Vec<_>>();
        let fairness_member = shape
            .fairness_kinds
            .as_ref()
            .map(|_| FairnessKindsMember::List(&fairness_labels));
        // FSL owns evidence-kind admission (QSL-654): hand it the labels.
        let evidence_labels = shape
            .evidence_kinds
            .as_deref()
            .map(|kinds| kinds.iter().map(String::as_str).collect::<Vec<_>>());
        let dynamics = shape
            .dynamics
            .as_deref()
            .map(|labels| labels.iter().map(String::as_str).collect::<Vec<_>>());
        let solver_bounds = shape.bounds.as_ref().and_then(|bounds| {
            let max_solver_steps = bounds.get("max_solver_steps").copied();
            let max_solver_time = bounds.get("max_solver_time").copied();
            (max_solver_steps.is_some() || max_solver_time.is_some()).then_some(SolverBounds {
                max_solver_steps,
                max_solver_time,
            })
        });
        let descriptor = descriptor_from_manifest_with_dynamics(
            Candidate::new(BackendId::from_wire(&shape.backend)?),
            ProviderOrigin::Process,
            shape
                .advertises
                .iter()
                .map(|pair| (pair.kind.as_deref(), pair.mode.as_deref())),
            fsl_route::ManifestMembers {
                domains: domain_labels.as_deref(),
                fairness_kinds: fairness_member,
                evidence_kinds: evidence_labels.as_deref(),
                ..fsl_route::ManifestMembers::default()
            },
            dynamics.as_deref().map(DynamicsMember::List),
            solver_bounds,
        )?;
        Ok(AdmittedManifest {
            descriptor,
            manifest: ProviderManifest {
                identity: shape.backend,
                advertises: shape
                    .advertises
                    .iter()
                    .map(|pair| (pair.kind.clone(), pair.mode.clone()))
                    .collect(),
                domains: shape.domains,
                bounds: shape.bounds,
                wire: Some(Arc::<[u8]>::from(canonical_wire)),
            },
        })
    }
}

/// One FR-331 request-stage envelope to serialize for a process provider:
/// the registry snapshot of retained manifests and the `request` member
/// FSL built ([`fsl_analyze::provider::ProviderRequest`]), forwarded as FSL
/// serialized it. The driver writes no request member of its own.
#[derive(Clone, Debug, Serialize)]
pub struct RequestStage {
    /// FR-331 contract identity.
    pub contract_version: &'static str,
    /// FR-290 capability-vocabulary identity.
    pub capability_vocabulary: &'static str,
    /// Snapshot of retained FR-331 manifests for the current FSL registry.
    pub manifest: Vec<Value>,
    /// FSL's FR-331 `request` member.
    pub request: Value,
}

/// A process request the driver cannot frame as an FR-331 request stage.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RequestStageError {
    /// FR-331 PositiveCount requires a nonzero safe integer deadline.
    #[error("deadline_ms must be in 1..=9007199254740991")]
    Deadline,
    /// FR-331 requires at least one retained manifest and one routed item.
    #[error("a process request requires at least one manifest and one item")]
    EmptyRequest,
    /// A retained manifest snapshot could not be reconstructed as JSON.
    #[error("a retained FR-331 manifest descriptor is invalid")]
    ManifestDescriptor,
    /// The routed provider is absent from the retained registry manifest.
    #[error("the routed backend is absent from the retained FR-331 manifest")]
    RoutedBackendMissing,
    /// FSL's request did not serialize.
    #[error("FSL's provider request did not serialize: {0}")]
    Request(String),
}

impl RequestStage {
    /// Frame FSL's request with the registry snapshot it was negotiated
    /// against.
    pub fn from_process_request(
        request: &filament_driver::ProcessProviderRequest<'_>,
    ) -> Result<Self, RequestStageError> {
        let deadline = request.request.deadline_ms();
        if deadline == 0 || deadline > MAX_SAFE_INTEGER {
            return Err(RequestStageError::Deadline);
        }
        if request.manifest.is_empty() || request.request.items().is_empty() {
            return Err(RequestStageError::EmptyRequest);
        }
        let backend = request.request.backend().as_str();
        if !request
            .manifest
            .iter()
            .any(|entry| entry.identity == backend)
        {
            return Err(RequestStageError::RoutedBackendMissing);
        }
        Ok(Self {
            contract_version: CONTRACT_VERSION,
            capability_vocabulary: CAPABILITY_VOCABULARY,
            manifest: request
                .manifest
                .iter()
                .map(manifest_descriptor)
                .collect::<Result<Vec<_>, _>>()?,
            request: serde_json::to_value(request.request)
                .map_err(|error| RequestStageError::Request(error.to_string()))?,
        })
    }
}

fn manifest_descriptor(manifest: &ProviderManifest) -> Result<Value, RequestStageError> {
    if let Some(wire) = &manifest.wire {
        let value: Value =
            serde_json::from_slice(wire).map_err(|_| RequestStageError::ManifestDescriptor)?;
        if value.get("backend").and_then(Value::as_str) != Some(manifest.identity.as_str()) {
            return Err(RequestStageError::ManifestDescriptor);
        }
        return Ok(value);
    }
    Err(RequestStageError::ManifestDescriptor)
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::*;

    fn hello(kind: Value, mode: Value) -> ManifestStage {
        ManifestStage {
            contract_version: CONTRACT_VERSION.to_owned(),
            capability_vocabulary: Some(CAPABILITY_VOCABULARY.to_owned()),
            manifest: vec![serde_json::json!({
                "backend": "agent-ix/provider/test",
                "advertises": [{ "kind": kind, "mode": mode }],
                "domains": ["integer"],
                "bounds": { "max_states": 8 }
            })
            .into()],
        }
    }

    #[trace("TC-047", "FR-022-AC-11")]
    #[test]
    fn hybrid_hello_retains_dynamics_and_solver_bounds_through_fsl_admission() {
        let wire = serde_json::json!({"backend": "hybrid-process",
            "advertises": [{"kind": "temporal-satisfaction", "mode": "unbounded"}],
            "dynamics": ["hybrid"], "bounds": {"max_solver_steps": 123, "max_solver_time": 2},
            "tool": {"name": "fixture", "version": "1"}});
        let stage = ManifestStage {
            contract_version: CONTRACT_VERSION.to_owned(),
            capability_vocabulary: Some(CAPABILITY_VOCABULARY.to_owned()),
            manifest: vec![Arc::new(wire.clone())],
        };
        let admitted = stage.admit_process(65_536).expect("hybrid hello admitted");
        assert!(admitted
            .descriptor()
            .dynamics()
            .expect("dynamics")
            .contains(&fsl_route::DynamicsKind::Hybrid));
        let bounds = admitted
            .descriptor()
            .solver_bounds()
            .expect("solver bounds");
        assert_eq!(bounds.max_solver_steps, Some(123));
        assert_eq!(bounds.max_solver_time, Some(2));
        assert_eq!(admitted.descriptor().origin(), ProviderOrigin::Process);
        let retained: Value =
            serde_json::from_slice(admitted.manifest().wire.as_deref().expect("retained wire"))
                .expect("wire descriptor");
        assert_eq!(retained, wire);
    }

    #[trace("TC-047", "FR-022-AC-11")]
    #[test]
    fn hybrid_hello_refuses_malformed_or_unrecognized_dynamics() {
        for dynamics in [
            serde_json::json!(null),
            serde_json::json!("hybrid"),
            serde_json::json!([]),
            serde_json::json!(["unknown"]),
            serde_json::json!(["hybrid", "hybrid"]),
        ] {
            let stage = ManifestStage {
                contract_version: CONTRACT_VERSION.to_owned(),
                capability_vocabulary: Some(CAPABILITY_VOCABULARY.to_owned()),
                manifest: vec![Arc::new(serde_json::json!({"backend": "hybrid-process",
                    "advertises": [{"kind": "temporal-satisfaction", "mode": "unbounded"}],
                    "dynamics": dynamics}))],
            };
            assert!(stage.admit_process(65_536).is_err(), "{dynamics}");
        }
    }

    fn process_manifest(identity: &str) -> ProviderManifest {
        ManifestStage {
            contract_version: CONTRACT_VERSION.to_owned(),
            capability_vocabulary: Some(CAPABILITY_VOCABULARY.to_owned()),
            manifest: vec![serde_json::json!({
                "backend": identity,
                "advertises": [{ "kind": "value-validity", "mode": "unbounded" }],
                "domains": ["integer"],
                "tool": {"name": "fsl-test-capture", "version": "1"}
            })
            .into()],
        }
        .admit_process(4096)
        .expect("FSL admits the process hello")
        .manifest()
        .clone()
    }

    /// Trace: FR-006-AC-1, FR-006-AC-5, FR-006-AC-8, FR-006-AC-9; the request
    /// stage carries the retained manifest snapshot and FSL's request member
    /// unchanged, and refuses a snapshot without the routed backend.
    #[test]
    fn request_stage_forwards_fsl_request_with_the_retained_snapshot() {
        use crate::test_support::{with_provider_request, CAPTURE_BACKEND};
        let (framed, expected, missing) = with_provider_request(|request| {
            let manifest = [process_manifest(CAPTURE_BACKEND)];
            let cancel = filament_exact::Cancel::new();
            let framed =
                RequestStage::from_process_request(&filament_driver::ProcessProviderRequest {
                    request,
                    manifest: &manifest,
                    cancel: &cancel,
                });
            let other = [process_manifest("agent-ix/provider/other")];
            let missing =
                RequestStage::from_process_request(&filament_driver::ProcessProviderRequest {
                    request,
                    manifest: &other,
                    cancel: &cancel,
                });
            (
                framed,
                serde_json::to_value(request).expect("FSL's request serializes"),
                missing,
            )
        });
        let framed = serde_json::to_value(framed.expect("the request frames"))
            .expect("the stage serializes");
        assert_eq!(framed["request"], expected);
        assert_eq!(framed["request"]["deadline_ms"], 600_000);
        assert!(framed["request"].get("domains").is_none());
        assert!(framed["request"].get("limits").is_none());
        assert_eq!(framed["manifest"][0]["backend"], CAPTURE_BACKEND);
        assert_eq!(framed["manifest"][0]["tool"]["name"], "fsl-test-capture");
        assert_eq!(missing.err(), Some(RequestStageError::RoutedBackendMissing));
    }

    /// Trace: FR-005-AC-15.
    #[test]
    fn hello_admits_process_descriptor_and_retains_full_manifest() {
        let stage = hello(
            Value::String("operation-contract".to_owned()),
            Value::String("bounded".to_owned()),
        );
        let source_wire = Arc::clone(&stage.manifest[0]);
        let admitted = stage
            .admit_process(1024 * 1024)
            .expect("known labels are admitted");

        assert_eq!(admitted.descriptor().origin(), ProviderOrigin::Process);
        assert_eq!(
            admitted.descriptor().id().as_str(),
            "agent-ix/provider/test"
        );
        let retained: Value = serde_json::from_slice(
            admitted
                .manifest()
                .wire
                .as_deref()
                .expect("canonical wire DTO retained"),
        )
        .expect("retained descriptor is JSON");
        assert_eq!(retained, *source_wire);
        assert_eq!(
            admitted.manifest().domains.as_deref(),
            Some(["integer".to_owned()].as_slice())
        );
        assert!(admitted
            .descriptor()
            .domains()
            .is_some_and(|domains| domains.contains(&fsl_foundation::bound::DomainKind::Integer)));
        assert_eq!(
            admitted.manifest().bounds.as_ref(),
            Some(&BTreeMap::from([("max_states".to_owned(), 8)]))
        );
    }

    #[test]
    fn unsupported_capability_vocabulary_uses_fsl_typed_refusal() {
        for capability_vocabulary in [None, Some("filament.capability-kind/v0".to_owned())] {
            let mut stage = hello(
                Value::String("operation-contract".to_owned()),
                Value::String("bounded".to_owned()),
            );
            stage.capability_vocabulary = capability_vocabulary;
            let error = stage
                .admit_process(1024)
                .expect_err("unknown vocabulary refuses");
            let ManifestAdmissionError::CapabilityVocabulary(code) = error else {
                panic!("vocabulary mismatch must retain FSL's typed refusal: {error:?}");
            };
            assert_eq!(code.code(), "invalid_capability");
            assert_eq!(code.cause(), "unknown-vocabulary");
            assert_eq!(category_of(&code), Some(Category::Refusal));
        }
    }

    #[test]
    fn manifest_descriptor_checks_wire_shape_constraints() {
        let base = || {
            serde_json::json!({
                "backend": "agent-ix/provider/test",
                "advertises": [{"kind":"operation-contract", "mode":"bounded"}]
            })
        };
        let mut cases = Vec::new();
        let mut invalid = base();
        invalid["options"] = serde_json::json!({});
        cases.push((invalid, "options"));
        let mut invalid = base();
        invalid["options"] = Value::Null;
        cases.push((invalid, "options"));
        let mut invalid = base();
        invalid["bounds"] = serde_json::json!({"Max_states": 2});
        cases.push((invalid, "bounds"));
        let mut invalid = base();
        invalid["bounds"] = serde_json::json!({"max_states": MAX_SAFE_INTEGER + 1});
        cases.push((invalid, "bounds"));
        let mut invalid = base();
        invalid["unknown_member"] = Value::Bool(true);
        cases.push((invalid, "unknown_member"));
        let mut invalid = base();
        invalid["advertises"] = serde_json::json!([]);
        cases.push((invalid, "advertises"));
        for (wire, field) in cases {
            let stage = ManifestStage {
                contract_version: CONTRACT_VERSION.to_owned(),
                capability_vocabulary: Some(CAPABILITY_VOCABULARY.to_owned()),
                manifest: vec![Arc::new(wire)],
            };
            let error = stage
                .admit_process(4096)
                .expect_err("schema-invalid descriptor refuses");
            assert!(
                matches!(error, ManifestAdmissionError::InvalidMember { .. }),
                "{error:?}"
            );
            assert!(error.to_string().contains(field), "{error}");
        }
    }

    #[test]
    fn absent_and_unknown_hello_labels_reach_fsl_admission() {
        for invalid in [
            (
                Value::Null,
                Value::String("bounded".to_owned()),
                "absent-kind",
            ),
            (
                Value::String("made-up-kind".to_owned()),
                Value::String("bounded".to_owned()),
                "unknown-kind",
            ),
            (
                Value::String("operation-contract".to_owned()),
                Value::String("made-up-mode".to_owned()),
                "unknown-mode",
            ),
            (
                Value::String("operation-contract".to_owned()),
                Value::Null,
                "unknown-mode",
            ),
        ] {
            let error = hello(invalid.0, invalid.1)
                .admit_process(1024 * 1024)
                .expect_err("FSL refuses malformed capability labels");
            let ManifestAdmissionError::Registration(refusal) = error else {
                panic!("malformed labels must be classified by FSL: {error:?}");
            };
            assert_eq!(refusal.cause().as_str(), invalid.2);
        }
    }

    /// Trace: FR-005-AC-18; domain vocabulary and refusal remain FSL-owned.
    #[test]
    fn invalid_domains_reach_fsl_admission_without_driver_reclassification() {
        let mut stage = hello(
            Value::String("operation-contract".to_owned()),
            Value::String("bounded".to_owned()),
        );
        let mut descriptor = (*stage.manifest[0]).clone();
        descriptor["domains"] = serde_json::json!(["quantity"]);
        stage.manifest[0] = Arc::new(descriptor);

        let error = stage
            .admit_process(1024 * 1024)
            .expect_err("FSL refuses a domain that cannot have a finite bound");
        let ManifestAdmissionError::Registration(refusal) = error else {
            panic!("domain admission must preserve FSL's typed refusal: {error:?}");
        };
        assert_eq!(refusal.catalog_code().code(), "invalid_capability");
        assert_eq!(refusal.catalog_code().cause(), "invalid-domains");
        assert!(matches!(
            refusal.cause(),
            fsl_route::RegistrationCause::InvalidDomains(_)
        ));
    }
}
