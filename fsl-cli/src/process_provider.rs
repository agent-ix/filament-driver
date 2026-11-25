// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generic `filament-plugin-host` adapter for the driver's process-provider seam.

use std::convert::Infallible;
use std::ffi::OsString;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use filament_driver::{
    ProcessProvider, ProcessProviderLimits, ProcessProviderRequest, ProcessProviderSession,
    ProviderItemResult, ProviderResults,
};
use filament_exact::Cancel;
use filament_plugin_host::{
    FrameLimits, PluginError, PluginSession, RunLimits, SessionFinished, SessionStart,
};
use fsl_route::{BackendDescriptor, ProviderOrigin};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

/// Closed FR-331 result values retained before a FSL mapping is available.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Fr331ResultKind {
    Proved,
    Refuted,
    Tested,
    Measured,
    Inconclusive,
    Unsupported,
    Declined,
    Incomplete,
    Failed,
}

/// Closed FR-331 negotiation dispositions retained without terminal mapping.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Fr331DispositionKind {
    Supported,
    RequiresBound,
    Unsupported,
    InvalidRequest,
}

/// One result or disposition for a routed item, plus its exact received envelope.
#[derive(Clone, Debug)]
pub(crate) struct ProcessProviderOutcome {
    pub(crate) item: ProcessProviderItemOutcome,
    pub(crate) response: Arc<serde_json::Value>,
}

/// Build one request body from retained driver and FSL values.
pub(crate) fn encode_fr331_request(
    request: ProcessProviderRequest<'_>,
) -> Result<crate::backend_provider::RequestStage, String> {
    crate::backend_provider::RequestStage::from_process_request(&request)
        .map_err(|error| error.to_string())
}

/// Validate an FR-331 outcome against the one admitted hello identity.
pub(crate) fn validate_fr331_response(
    manifest: &crate::backend_provider::ManifestStage,
    response: &serde_json::Value,
) -> Result<(), String> {
    let admitted_identity = manifest
        .manifest
        .first()
        .and_then(|descriptor| descriptor.get("backend"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "admitted FR-331 hello has no backend identity".to_owned())?;
    let results = retain_fr331_response(response.clone())?;
    if results
        .items
        .iter()
        .any(|item| item.backend.identity != admitted_identity)
    {
        return Err("FR-331 outcome backend differs from the admitted provider".to_owned());
    }
    let mut seen = std::collections::BTreeSet::new();
    if results
        .items
        .iter()
        .any(|item| !seen.insert(item.request_index))
    {
        return Err("FR-331 outcome repeats a request index".to_owned());
    }
    Ok(())
}

/// Retain the full FR-331 outcome without selecting FSL terminals.
pub(crate) fn convert_fr331_response(
    response: serde_json::Value,
) -> Result<ProviderResults<ProcessProviderOutcome, String>, String> {
    retain_fr331_response(response)
}

/// One FR-331 item state: a result, with the capability kind it answers,
/// or a disposition, which settles no claim.
#[derive(Clone, Debug)]
pub(crate) enum ProcessProviderItemOutcome {
    Result {
        capability_kind: String,
        kind: Fr331ResultKind,
    },
    Disposition,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fr331ResponseItem {
    request_index: u64,
    backend: String,
    kind: String,
    #[serde(default)]
    result: Option<Fr331ResultKind>,
    #[serde(default)]
    disposition: Option<Fr331DispositionKind>,
    /// The item's other FR-331 members, accepted and left in the retained
    /// response.
    #[serde(flatten)]
    _members: serde_json::Map<String, serde_json::Value>,
}

fn retain_fr331_response(
    response: serde_json::Value,
) -> Result<ProviderResults<ProcessProviderOutcome, String>, String> {
    let contract = response
        .get("contract_version")
        .and_then(serde_json::Value::as_str);
    let vocabulary = response
        .get("capability_vocabulary")
        .and_then(serde_json::Value::as_str);
    if contract != Some("filament.backend-provider/v1")
        || vocabulary != Some("filament.capability-kind/v1")
    {
        return Err("plugin result has an unsupported FR-331 identity".to_owned());
    }
    let response = Arc::new(response);
    let mut items = Vec::new();
    for (member, is_result) in [("results", true), ("dispositions", false)] {
        let entries = response
            .get(member)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("plugin result is missing the `{member}` array"))?;
        for value in entries {
            let parsed: Fr331ResponseItem = serde_json::from_value(value.clone())
                .map_err(|error| format!("invalid FR-331 {member} item: {error}"))?;
            if parsed.result.is_some() != is_result || parsed.disposition.is_some() == is_result {
                return Err(format!("FR-331 {member} item has the wrong state member"));
            }
            let request_index = usize::try_from(parsed.request_index)
                .map_err(|_| "FR-331 request index does not fit this platform".to_owned())?;
            let outcome = match parsed.result {
                Some(kind) => ProcessProviderItemOutcome::Result {
                    capability_kind: parsed.kind,
                    kind,
                },
                None => ProcessProviderItemOutcome::Disposition,
            };
            items.push(ProviderItemResult {
                request_index,
                backend: filament_driver::Candidate {
                    identity: parsed.backend,
                },
                result: Ok(ProcessProviderOutcome {
                    item: outcome,
                    response: Arc::clone(&response),
                }),
            });
        }
    }
    items.sort_by_key(|item| item.request_index);
    Ok(ProviderResults { items })
}

type ProviderWireTypes<M, Q, R, F, A, V, I, E> = fn() -> (M, Q, R, F, A, V, I, E);
type SessionWireTypes<Q, R, V, I, E> = fn() -> (Q, R, V, I, E);

/// Why hello admission did not provide a caller-admitted process descriptor.
#[derive(Debug, thiserror::Error)]
pub enum AdmissionFailure<E> {
    /// The caller's FSL admission boundary rejected the manifest.
    #[error("caller rejected plugin hello: {0}")]
    Caller(E),
    /// The returned descriptor did not carry the caller-supplied Process origin.
    #[error("caller-admitted descriptor does not have Process origin")]
    WrongOrigin,
}

/// Error returned by the generic host adapter.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError<A, V, F> {
    /// Starting the explicit-path process or admitting its hello failed.
    #[error("plugin session start failed: {0}")]
    Start(PluginError<AdmissionFailure<A>, Infallible, F>),
    /// The typed FR-331 request could not be projected from retained driver inputs.
    #[error("plugin request projection failed: {0}")]
    Encode(V),
    /// The FR-331 result could not be retained in the shared provider carrier.
    #[error("plugin result conversion failed: {0}")]
    Convert(V),
    /// Sending a request, validating its result, or process cleanup failed.
    #[error("plugin request failed: {0}")]
    Request(PluginError<AdmissionFailure<A>, V, F>),
    /// Graceful shutdown or diagnostic capture failed.
    #[error("plugin shutdown failed: {0}")]
    Shutdown(PluginError<AdmissionFailure<A>, Infallible, F>),
    /// The host reported successful admission without invoking its admitter.
    #[error("plugin host completed admission without returning a descriptor")]
    MissingAdmittedDescriptor,
}

/// A process provider adapter parameterized by caller-owned FR-331 types and
/// conversion functions. This type defines no manifest, request, result or
/// fault wire schema.
pub struct HostProcessProvider<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert> {
    admit: Arc<Admit>,
    encode: Arc<Encode>,
    validate: Arc<Validate>,
    convert: Arc<Convert>,
    signals: crate::signals::SignalContext,
    // This function-pointer marker keeps the wire types covariant without
    // imposing their Send/Sync bounds on the adapter itself.
    #[allow(clippy::type_complexity)]
    _wire_types: PhantomData<ProviderWireTypes<M, Q, R, F, A, V, I, E>>,
}

impl<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert>
    HostProcessProvider<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert>
{
    /// Build the adapter from FSL admission and caller-owned typed FR-331
    /// request/result conversions.
    pub fn new(
        signals: crate::signals::SignalContext,
        admit: Admit,
        encode: Encode,
        validate: Validate,
        convert: Convert,
    ) -> Self {
        Self {
            admit: Arc::new(admit),
            encode: Arc::new(encode),
            validate: Arc::new(validate),
            convert: Arc::new(convert),
            signals,
            _wire_types: PhantomData,
        }
    }
}

impl<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert> ProcessProvider
    for HostProcessProvider<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert>
where
    M: DeserializeOwned,
    Q: Serialize,
    R: DeserializeOwned,
    F: DeserializeOwned,
    Admit: Fn(&M) -> Result<(BackendDescriptor, filament_driver::ProviderManifest), A> + 'static,
    Encode: for<'a> Fn(ProcessProviderRequest<'a>) -> Result<Q, V> + 'static,
    Validate: Fn(&M, &R) -> Result<(), V> + 'static,
    Convert: Fn(R) -> Result<ProviderResults<I, E>, V> + 'static,
{
    type Session = HostProcessSession<M, Q, R, F, A, V, I, E, Encode, Validate, Convert>;
    type Error = AdapterError<A, V, F>;

    fn start(
        &self,
        executable: &Path,
        arguments: &[OsString],
        limits: ProcessProviderLimits,
        cancel: &Cancel,
    ) -> Result<Self::Session, Self::Error> {
        let active = self.signals.activate(cancel.clone());
        let mut admitted = None;
        let host = PluginSession::start(
            SessionStart {
                executable,
                arguments,
                limits: RunLimits {
                    frames: FrameLimits {
                        max_frame_bytes: limits.max_frame_bytes,
                        max_total_bytes: limits.max_total_bytes,
                    },
                    max_stderr_bytes: limits.max_stderr_bytes,
                    max_memory_bytes: limits.max_memory_bytes,
                    memory_limit_requested: limits.memory_limit_requested,
                    max_cpu_ms: limits.max_cpu_ms,
                    cancellation_grace: limits.cancellation_grace,
                },
                cancel,
            },
            |manifest| {
                let (descriptor, retained_manifest) =
                    (self.admit)(manifest).map_err(AdmissionFailure::Caller)?;
                if descriptor.origin() != ProviderOrigin::Process {
                    return Err(AdmissionFailure::WrongOrigin);
                }
                admitted = Some((descriptor, retained_manifest));
                Ok(())
            },
        )
        .map_err(AdapterError::Start)?;
        let (descriptor, manifest) = admitted.ok_or(AdapterError::MissingAdmittedDescriptor)?;
        Ok(HostProcessSession {
            host,
            descriptor,
            manifest,
            signals: self.signals.clone(),
            encode: Arc::clone(&self.encode),
            validate: Arc::clone(&self.validate),
            convert: Arc::clone(&self.convert),
            active,
            _wire_types: PhantomData,
        })
    }
}

/// An admitted host session implementing the driver-owned process seam.
pub struct HostProcessSession<M, Q, R, F, A, V, I, E, Encode, Validate, Convert> {
    host: PluginSession<M, F, AdmissionFailure<A>>,
    descriptor: BackendDescriptor,
    manifest: filament_driver::ProviderManifest,
    signals: crate::signals::SignalContext,
    encode: Arc<Encode>,
    validate: Arc<Validate>,
    convert: Arc<Convert>,
    active: crate::signals::ActiveSession,
    _wire_types: PhantomData<SessionWireTypes<Q, R, V, I, E>>,
}

impl<M, Q, R, F, A, V, I, E, Encode, Validate, Convert>
    HostProcessSession<M, Q, R, F, A, V, I, E, Encode, Validate, Convert>
where
    M: DeserializeOwned,
    F: DeserializeOwned,
{
    /// Transport an opaque FSL request through the same bounded session.
    /// FSL alone reads the hybrid result and admits its semantic evidence.
    pub(crate) fn exchange_hybrid(
        &mut self,
        request: &[u8],
        cancel: &Cancel,
    ) -> Result<Vec<u8>, fsl_analyze::analyze::HybridExchangeError> {
        use fsl_analyze::analyze::HybridExchangeError;
        use fsl_replay::ToolFailure;

        let _request_active = self.signals.activate(cancel.clone());
        let body: serde_json::Value = serde_json::from_slice(request)
            .map_err(|_| HybridExchangeError::ToolFailure(ToolFailure::MalformedBody))?;
        let deadline_ms = body
            .pointer("/request/deadline_ms")
            .and_then(serde_json::Value::as_u64)
            .ok_or(HybridExchangeError::ToolFailure(ToolFailure::MalformedBody))?;
        let result: serde_json::Value = self
            .host
            .request_with_timeout(
                &body,
                cancel,
                std::time::Duration::from_millis(deadline_ms),
                |_, _: &serde_json::Value| Ok::<_, Infallible>(()),
            )
            .map_err(|error| {
                if let Some(source) = cancel.cause() {
                    let _ = cancel.poll();
                    HybridExchangeError::Cancelled { source }
                } else if matches!(error.primary_cause(), PluginError::TimedOut) {
                    HybridExchangeError::TimedOut
                } else {
                    HybridExchangeError::ToolFailure(hybrid_transport_failure(
                        error.primary_cause(),
                    ))
                }
            })?;
        // The host admits only exact canonical frames. Re-encoding their
        // opaque body preserves every member while the host owns byte limits.
        quire_canonical::to_vec(&result, quire_canonical::Limits::new(u64::MAX))
            .map_err(|_| HybridExchangeError::ToolFailure(ToolFailure::MalformedBody))
    }
    /// The admitted host's actual process-memory enforcement status.
    pub(crate) fn memory_enforcement(&self) -> filament_plugin_host::MemoryEnforcement {
        self.host.memory_enforcement()
    }
}

fn hybrid_transport_failure<A, V, F>(error: &PluginError<A, V, F>) -> fsl_replay::ToolFailure {
    use filament_plugin_host::FrameError;
    use fsl_replay::ToolFailure;

    match error {
        PluginError::Frame(FrameError::FrameTooLarge { actual, maximum }) => {
            ToolFailure::FrameLength {
                limit: *maximum,
                value: *actual,
            }
        }
        PluginError::Frame(FrameError::RunTooLarge { actual, maximum }) => {
            ToolFailure::TotalBytes {
                limit: *maximum,
                value: *actual,
            }
        }
        PluginError::Frame(FrameError::NonCanonical) => ToolFailure::NoncanonicalFrame,
        PluginError::Frame(FrameError::WrongProtocol(_)) => ToolFailure::UnknownProtocol,
        PluginError::UnexpectedId { .. } => ToolFailure::UnknownRequestId,
        PluginError::UnexpectedKind { .. } => ToolFailure::UnknownFrameKind,
        PluginError::Exited(status) if status.code().is_none() => ToolFailure::Crash,
        PluginError::Exited(status) if !status.success() => ToolFailure::NonzeroExit,
        PluginError::MissingFrame | PluginError::Exited(_) => ToolFailure::MissingTerminalRecord,
        PluginError::SessionClosed => ToolFailure::Crash,
        PluginError::Fault(_) | PluginError::ResultValidation(_) => ToolFailure::MalformedBody,
        PluginError::WithDiagnostics { cause, .. } => hybrid_transport_failure(cause),
        PluginError::BareExecutableName(_)
        | PluginError::UnsupportedPlatform { .. }
        | PluginError::ShellScriptExecutable(_)
        | PluginError::MissingManifest
        | PluginError::Spawn { .. }
        | PluginError::MissingPipe(_)
        | PluginError::Wait(_)
        | PluginError::RequestIdsExhausted
        | PluginError::UnexpectedFrame
        | PluginError::GraceExpired
        | PluginError::ShutdownTimeout
        | PluginError::ProcessTreeCleanupTimeout
        | PluginError::Cancelled
        | PluginError::TimedOut
        | PluginError::ManifestAdmission(_)
        | PluginError::Json(_)
        | PluginError::TimerThread
        | PluginError::ReaderThread
        | PluginError::WriterThread
        | PluginError::WriterUnresponsive
        | PluginError::WorkerJoinTimeout(_)
        | PluginError::WorkerSpawn { .. }
        | PluginError::CaptureThread
        | PluginError::Stderr(_)
        | PluginError::Frame(_) => ToolFailure::MalformedFrame,
    }
}

impl<M, Q, R, F, A, V, I, E, Encode, Validate, Convert> ProcessProviderSession
    for HostProcessSession<M, Q, R, F, A, V, I, E, Encode, Validate, Convert>
where
    M: DeserializeOwned,
    Q: Serialize,
    R: DeserializeOwned,
    F: DeserializeOwned,
    Encode: for<'a> Fn(ProcessProviderRequest<'a>) -> Result<Q, V>,
    Validate: Fn(&M, &R) -> Result<(), V>,
    Convert: Fn(R) -> Result<ProviderResults<I, E>, V>,
{
    type Error = AdapterError<A, V, F>;
    type ItemResult = I;
    type ItemFailure = E;
    type Diagnostics = SessionFinished<M>;

    fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    fn manifest(&self) -> &filament_driver::ProviderManifest {
        &self.manifest
    }

    fn invoke(
        &mut self,
        request: ProcessProviderRequest<'_>,
    ) -> Result<ProviderResults<Self::ItemResult, Self::ItemFailure>, Self::Error> {
        let cancel = request.cancel;
        let _request_active = self.signals.activate(cancel.clone());
        let body = (self.encode)(request).map_err(AdapterError::Encode)?;
        let result = self
            .host
            .request(&body, cancel, |manifest, result| {
                (self.validate)(manifest, result)
            })
            .map_err(AdapterError::Request)?;
        (self.convert)(result).map_err(AdapterError::Convert)
    }

    fn shutdown(self) -> Result<Self::Diagnostics, Self::Error> {
        let Self { host, active, .. } = self;
        let diagnostics = host.shutdown().map_err(AdapterError::Shutdown)?;
        drop(active);
        Ok(diagnostics)
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "an adapter fixture mismatch is the test failure report"
)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use filament_driver::{
        build_linked_providers, ProcessProvider, ProcessProviderLimits, ProcessProviderRequest,
        ProcessProviderSession, ProviderItemResult, ProviderManifest, ProviderResults,
    };
    use filament_exact::Cancel;
    use fsl_foundation::BackendId;
    use fsl_route::{descriptor_from_manifest, BackendDescriptor, Candidate, ProviderOrigin};
    use serde::Deserialize;

    use super::HostProcessProvider;
    use crate::test_support::with_provider_request;

    #[derive(Debug, Deserialize)]
    struct Hello {
        identity: String,
        advertises: Vec<(Option<String>, Option<String>)>,
        #[serde(default)]
        domains: Option<Vec<String>>,
    }

    fn admit_hello(
        manifest: &Hello,
        origin: ProviderOrigin,
    ) -> Result<(BackendDescriptor, ProviderManifest), String> {
        let identity =
            BackendId::from_wire(&manifest.identity).map_err(|error| error.to_string())?;
        let descriptor = descriptor_from_manifest(
            Candidate::new(identity),
            origin,
            manifest
                .advertises
                .iter()
                .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
            fsl_route::ManifestMembers {
                domains: manifest
                    .domains
                    .as_deref()
                    .map(|domains| domains.iter().map(String::as_str).collect::<Vec<_>>())
                    .as_deref(),
                ..fsl_route::ManifestMembers::default()
            },
        )
        .map_err(|error| format!("{error:?}"))?;
        let retained = ProviderManifest {
            identity: manifest.identity.clone(),
            advertises: manifest.advertises.clone(),
            domains: manifest.domains.clone(),
            bounds: None,
            wire: None,
        };
        Ok((descriptor, retained))
    }

    fn fixture_executable() -> PathBuf {
        std::env::current_exe().expect("test fixture executable")
    }

    fn is_fixture_child(test_name: &str) -> bool {
        let arguments = std::env::args().collect::<Vec<_>>();
        arguments
            .windows(2)
            .any(|pair| pair[0] == "--exact" && pair[1] == test_name)
    }

    fn fixture_arguments(test_name: &str) -> [std::ffi::OsString; 4] {
        [
            "--quiet".into(),
            "--exact".into(),
            format!("process_provider::tests::{test_name}").into(),
            "--nocapture".into(),
        ]
    }

    fn limits() -> ProcessProviderLimits {
        ProcessProviderLimits {
            max_frame_bytes: 4096,
            max_total_bytes: 16_384,
            max_stderr_bytes: 1024,
            max_memory_bytes: 4_294_967_296,
            memory_limit_requested: false,
            max_cpu_ms: 600_000,
            cancellation_grace: Duration::from_millis(500),
        }
    }

    fn write_fixture_frame(value: &serde_json::Value) {
        use std::io::Write;

        let payload = serde_json::to_vec(value).expect("serialize fixture frame");
        let mut output = std::io::stdout().lock();
        let length = u64::try_from(payload.len()).expect("fixture frame length fits u64");
        output
            .write_all(&length.to_be_bytes())
            .and_then(|()| output.write_all(&payload))
            .and_then(|()| output.flush())
            .expect("write fixture frame");
    }

    fn read_fixture_frame() -> serde_json::Value {
        use std::io::Read;

        let mut input = std::io::stdin().lock();
        let mut prefix = [0; 8];
        input
            .read_exact(&mut prefix)
            .expect("read fixture frame length");
        let length = usize::try_from(u64::from_be_bytes(prefix)).expect("frame length fits usize");
        let mut payload = vec![0; length];
        input.read_exact(&mut payload).expect("read fixture frame");
        serde_json::from_slice(&payload).expect("parse fixture frame")
    }

    fn wire_frame(id: u64, kind: &str, body: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "protocol": "filament.plugin-wire/v1",
            "id": id,
            "kind": kind,
            "body": body,
        })
    }

    fn run_plugin_fixture(kind: &str) {
        const ECHO: &str = "agent-ix/provider/test";
        const FR331: &str = "agent-ix/provider/fr331-fixture";
        let (identity, hello_body) = match kind {
            "echo" | "wrong-result" => (
                ECHO,
                serde_json::json!({"identity": ECHO, "advertises": []}),
            ),
            "fr331" => (
                FR331,
                serde_json::json!({
                    "contract_version": "filament.backend-provider/v1",
                    "capability_vocabulary": "filament.capability-kind/v1",
                    "manifest": [{
                        "backend": FR331,
                        "advertises": [{"kind": "operation-contract", "mode": "bounded"}],
                        "domains": ["integer"],
                        "bounds": {"max_states": 8}
                    }]
                }),
            ),
            "signal" => (
                "agent-ix/provider/signal-test",
                serde_json::json!({"identity": "agent-ix/provider/signal-test", "advertises": []}),
            ),
            _ => panic!("unknown Rust plugin fixture kind"),
        };

        if kind == "signal"
            && std::env::var("FILAMENT_PLUGIN_SIGNAL_STAGE")
                .expect("signal stage")
                .starts_with("hello")
        {
            run_signal_hello();
        }
        write_fixture_frame(&wire_frame(0, "hello", hello_body));
        loop {
            let request = read_fixture_frame();
            match request["kind"].as_str().expect("request kind") {
                "shutdown" => {
                    if kind == "signal" {
                        run_signal_shutdown();
                    }
                    break;
                }
                "cancel" => continue,
                "request" => {
                    if kind == "signal" {
                        run_signal_request();
                    }
                    if kind == "wrong-result" {
                        write_fixture_frame(&wire_frame(
                            request["id"].as_u64().expect("request id"),
                            "result",
                            serde_json::json!({"identity": "agent-ix/provider/other", "items": []}),
                        ));
                        break;
                    }
                    let mut body = serde_json::json!({"identity": identity, "items": []});
                    if kind == "echo" || kind == "fr331" {
                        body["items"] = request["body"]["items"].clone();
                    }
                    write_fixture_frame(&wire_frame(
                        request["id"].as_u64().expect("request id"),
                        "result",
                        body,
                    ));
                }
                other => panic!("unexpected fixture request kind: {other}"),
            }
        }
        if kind == "echo" {
            eprintln!("opaque provider diagnostic");
        } else if kind == "fr331" {
            eprintln!("FR-331 fixture diagnostic");
        }
        // The fixture is running inside libtest. Exit before the harness writes its
        // test-result line to stdout, which is the provider protocol stream.
        std::process::exit(0);
    }

    fn signal_fixture_paths() -> (
        String,
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
    ) {
        use std::path::PathBuf;

        let stage = std::env::var("FILAMENT_PLUGIN_SIGNAL_STAGE").expect("signal stage");
        let ready =
            PathBuf::from(std::env::var_os("FILAMENT_PLUGIN_SIGNAL_READY").expect("ready path"));
        let release = PathBuf::from(
            std::env::var_os("FILAMENT_PLUGIN_SIGNAL_RELEASE").expect("release path"),
        );
        let descendant = PathBuf::from(
            std::env::var_os("FILAMENT_PLUGIN_SIGNAL_DESCENDANT").expect("descendant path"),
        );
        let plugin_marker = PathBuf::from(
            std::env::var_os("FILAMENT_PLUGIN_SIGNAL_PLUGIN_MARKER").expect("plugin marker path"),
        );
        (stage, ready, release, descendant, plugin_marker)
    }

    fn signal_parent(stage: &str) {
        let signal = if stage.ends_with("-int") {
            "-INT"
        } else {
            "-TERM"
        };
        let parent =
            std::env::var("FILAMENT_PLUGIN_SIGNAL_PARENT_PID").expect("signal test parent pid");
        let status = std::process::Command::new("/bin/kill")
            .args([signal, &parent])
            .status()
            .expect("run system signal utility");
        assert!(status.success(), "send lifecycle signal to test process");
    }

    fn wait_signal_release(release: &std::path::Path) {
        use std::thread;
        use std::time::{Duration, Instant};

        let deadline = Instant::now() + Duration::from_secs(5);
        while !release.is_file() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        assert!(release.is_file(), "signal fixture released");
    }

    fn spawn_signal_descendant(descendant: &std::path::Path) {
        let executable = std::env::current_exe().expect("fixture test executable");
        std::process::Command::new(executable)
            .args([
                "--exact",
                "process_provider::tests::run_descendant_plugin_fixture",
                "--nocapture",
            ])
            .env("FILAMENT_PLUGIN_SIGNAL_DESCENDANT_CHILD", descendant)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn process-group descendant");
    }

    fn run_signal_hello() {
        use std::thread;
        use std::time::Duration;

        let (stage, ready, release, _, plugin_marker) = signal_fixture_paths();
        std::fs::write(&ready, b"ready").expect("write hello readiness");
        wait_signal_release(&release);
        signal_parent(&stage);
        thread::sleep(Duration::from_secs(2));
        std::fs::write(plugin_marker, b"leaked").expect("write late plugin marker");
    }

    fn run_signal_request() {
        use std::thread;
        use std::time::Duration;

        let (stage, ready, release, descendant, plugin_marker) = signal_fixture_paths();
        spawn_signal_descendant(&descendant);
        std::fs::write(&ready, b"ready").expect("write request readiness");
        wait_signal_release(&release);
        signal_parent(&stage);
        let cancel = read_fixture_frame();
        assert_eq!(cancel["kind"], "cancel");
        thread::sleep(Duration::from_secs(2));
        std::fs::write(plugin_marker, b"leaked").expect("write late plugin marker");
    }

    fn run_signal_shutdown() {
        use std::thread;
        use std::time::Duration;

        let (stage, ready, release, descendant, plugin_marker) = signal_fixture_paths();
        spawn_signal_descendant(&descendant);
        std::fs::write(&ready, b"ready").expect("write shutdown readiness");
        wait_signal_release(&release);
        signal_parent(&stage);
        thread::sleep(Duration::from_secs(2));
        std::fs::write(plugin_marker, b"leaked").expect("write late plugin marker");
    }

    #[test]
    fn run_echo_plugin_fixture() {
        if is_fixture_child("process_provider::tests::run_echo_plugin_fixture") {
            run_plugin_fixture("echo");
        }
    }

    #[test]
    fn run_fr331_plugin_fixture() {
        if is_fixture_child("process_provider::tests::run_fr331_plugin_fixture") {
            run_plugin_fixture("fr331");
        }
    }

    #[test]
    fn run_wrong_result_plugin_fixture() {
        if is_fixture_child("process_provider::tests::run_wrong_result_plugin_fixture") {
            run_plugin_fixture("wrong-result");
        }
    }

    #[test]
    fn run_signal_plugin_fixture() {
        if is_fixture_child("process_provider::tests::run_signal_plugin_fixture") {
            run_plugin_fixture("signal");
        }
    }

    #[test]
    fn run_descendant_plugin_fixture() {
        if is_fixture_child("process_provider::tests::run_descendant_plugin_fixture") {
            let path = std::env::var_os("FILAMENT_PLUGIN_SIGNAL_DESCENDANT_CHILD")
                .expect("descendant marker path");
            std::thread::sleep(Duration::from_millis(600));
            std::fs::write(path, b"leaked").expect("write descendant marker");
        }
    }

    /// Trace: FR-005-AC-6.
    #[test]
    fn host_adapter_returns_admitted_descriptor_and_shutdown_diagnostics() {
        let conversion_called = Arc::new(AtomicBool::new(false));
        let converted = Arc::clone(&conversion_called);
        let admissions = Arc::new(AtomicUsize::new(0));
        let admission_count = Arc::clone(&admissions);
        let provider = HostProcessProvider::<
            Hello,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            String,
            String,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            crate::signals::SignalContext::default(),
            move |manifest: &Hello| {
                admission_count.fetch_add(1, Ordering::Relaxed);
                admit_hello(manifest, ProviderOrigin::Process)
            },
            |request: ProcessProviderRequest<'_>| {
                assert!(!request.request.package().bytes().is_empty());
                assert_eq!(request.request.deadline_ms(), 600_000);
                assert!(!request.manifest.is_empty());
                Ok(serde_json::json!({
                    "items": request.request.items().iter().map(|item| item.index().get()).collect::<Vec<_>>()
                }))
            },
            |manifest: &Hello, result: &serde_json::Value| {
                if result["identity"] == manifest.identity {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            move |result: serde_json::Value| {
                converted.store(result["items"].as_array().is_some(), Ordering::Release);
                let identity = result["identity"]
                    .as_str()
                    .expect("echo fixture returns an identity")
                    .to_owned();
                Ok(ProviderResults::<String, String> {
                    items: vec![ProviderItemResult {
                        request_index: 23,
                        backend: filament_driver::Candidate {
                            identity: identity.clone(),
                        },
                        result: Ok(identity),
                    }],
                })
            },
        );
        let cancel = Cancel::new();
        let arguments = fixture_arguments("run_echo_plugin_fixture");
        let mut session = provider
            .start(&fixture_executable(), &arguments, limits(), &cancel)
            .expect("host starts and admits hello");
        assert_eq!(session.descriptor().origin(), ProviderOrigin::Process);
        assert_eq!(session.descriptor().id().as_str(), "agent-ix/provider/test");
        assert_eq!(admissions.load(Ordering::Relaxed), 1);

        let mut linked = build_linked_providers(&[]).expect("empty linked registry builds");
        linked
            .register_admitted([(session.descriptor().clone(), session.manifest().clone())])
            .expect("caller-admitted Process descriptor joins same registry");
        assert_eq!(
            linked
                .registry()
                .descriptors()
                .next()
                .expect("one descriptor registered")
                .origin(),
            ProviderOrigin::Process
        );

        let manifest = linked.manifest_snapshot();
        let (session, results) = with_provider_request(move |request| {
            let results = session.invoke(ProcessProviderRequest {
                request,
                manifest: &manifest,
                cancel: &cancel,
            });
            (session, results)
        });
        let results = results.expect("host returns typed result");
        assert!(conversion_called.load(Ordering::Acquire));
        assert_eq!(results.items.len(), 1);
        assert_eq!(results.items[0].request_index, 23);
        assert_eq!(results.items[0].backend.identity, "agent-ix/provider/test");
        assert_eq!(
            results.items[0].result.as_deref(),
            Ok("agent-ix/provider/test")
        );
        let finished = session.shutdown().expect("host shuts down cleanly");
        assert_eq!(finished.manifest.identity, "agent-ix/provider/test");
        assert_eq!(finished.stderr, b"opaque provider diagnostic\n");
        assert!(!finished.stderr_truncated);
    }

    /// Trace: FR-005-AC-5, FR-005-AC-12, FR-005-AC-13, FR-005-AC-14, FR-005-AC-15.
    #[test]
    fn fr331_hello_is_admitted_once_and_retained_for_registry_snapshot() {
        use crate::backend_provider::{ManifestAdmissionError, ManifestStage};

        let admissions = Arc::new(AtomicUsize::new(0));
        let admission_count = Arc::clone(&admissions);
        let provider = HostProcessProvider::<
            ManifestStage,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            ManifestAdmissionError,
            String,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            crate::signals::SignalContext::default(),
            move |manifest: &ManifestStage| {
                admission_count.fetch_add(1, Ordering::Relaxed);
                manifest
                    .admit_process(limits().max_frame_bytes)
                    .map(|admitted| (admitted.descriptor().clone(), admitted.manifest().clone()))
            },
            |_: ProcessProviderRequest<'_>| Ok(serde_json::json!({"items": []})),
            |manifest: &ManifestStage, result: &serde_json::Value| {
                if result["identity"] == manifest.manifest[0]["backend"] {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            |_| Ok(ProviderResults::<serde_json::Value, String>::default()),
        );
        let cancel = Cancel::new();
        let arguments = fixture_arguments("run_fr331_plugin_fixture");
        let mut session = provider
            .start(&fixture_executable(), &arguments, limits(), &cancel)
            .expect("typed FR-331 hello is admitted");
        assert_eq!(admissions.load(Ordering::Relaxed), 1);
        assert_eq!(session.descriptor().origin(), ProviderOrigin::Process);
        assert_eq!(
            session.descriptor().id().as_str(),
            "agent-ix/provider/fr331-fixture"
        );

        let mut linked = build_linked_providers(&[]).expect("one registry builds");
        linked
            .register_admitted([(session.descriptor().clone(), session.manifest().clone())])
            .expect("no second admission is performed");
        assert_eq!(
            linked
                .registry()
                .descriptors()
                .next()
                .expect("one process descriptor is registered")
                .origin(),
            ProviderOrigin::Process
        );
        let manifest = linked.manifest_snapshot();
        assert_eq!(manifest.len(), 1);
        let retained_wire = manifest[0]
            .wire
            .as_deref()
            .expect("retained hello descriptor has canonical bytes");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(retained_wire)
                .expect("retained bytes are valid JSON"),
            serde_json::json!({
                "backend": "agent-ix/provider/fr331-fixture",
                "advertises": [{"kind": "operation-contract", "mode": "bounded"}],
                "domains": ["integer"],
                "bounds": {"max_states": 8}
            })
        );

        let (session, invoked) = with_provider_request(move |request| {
            let invoked = session.invoke(ProcessProviderRequest {
                request,
                manifest: &manifest,
                cancel: &cancel,
            });
            (session, invoked)
        });
        invoked.expect("fixture responds to request");
        let finished = session.shutdown().expect("session ends cleanly");
        assert_eq!(finished.manifest.manifest.len(), 1);
        assert_eq!(finished.manifest.manifest[0]["domains"][0], "integer");
        assert_eq!(finished.manifest.manifest[0]["bounds"]["max_states"], 8);
    }

    #[test]
    fn adapter_preserves_result_validation_rejection() {
        use filament_plugin_host::PluginError;

        let provider = HostProcessProvider::<
            Hello,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            String,
            String,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            crate::signals::SignalContext::default(),
            |manifest: &Hello| admit_hello(manifest, ProviderOrigin::Process),
            |_: ProcessProviderRequest<'_>| Ok(serde_json::json!({"items": []})),
            |manifest: &Hello, result: &serde_json::Value| {
                if result["identity"] == manifest.identity {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            |_: serde_json::Value| Ok(ProviderResults::<serde_json::Value, String>::default()),
        );
        let cancel = Cancel::new();
        let arguments = fixture_arguments("run_wrong_result_plugin_fixture");
        let mut session = provider
            .start(&fixture_executable(), &arguments, limits(), &cancel)
            .expect("fixture hello is admitted");
        let error = with_provider_request(move |request| {
            session.invoke(ProcessProviderRequest {
                request,
                manifest: &[],
                cancel: &cancel,
            })
        });
        let rejected = match error {
            Err(super::AdapterError::Request(PluginError::ResultValidation(_))) => true,
            Err(super::AdapterError::Request(PluginError::WithDiagnostics { cause, .. })) => {
                matches!(*cause, PluginError::ResultValidation(_))
            }
            _ => false,
        };
        assert!(
            rejected,
            "adapter preserves typed result validation refusal"
        );
    }

    #[test]
    fn caller_admission_rejects_linked_origin_descriptor() {
        use filament_plugin_host::PluginError;

        let provider = HostProcessProvider::<
            Hello,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            String,
            String,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            crate::signals::SignalContext::default(),
            |manifest: &Hello| admit_hello(manifest, ProviderOrigin::Linked),
            |_: ProcessProviderRequest<'_>| Ok(serde_json::json!({})),
            |_: &Hello, _: &serde_json::Value| Ok::<(), String>(()),
            |_: serde_json::Value| Ok(ProviderResults::<serde_json::Value, String>::default()),
        );
        let arguments = fixture_arguments("run_echo_plugin_fixture");
        let error = provider.start(&fixture_executable(), &arguments, limits(), &Cancel::new());
        let refused_for_origin = match error {
            Err(super::AdapterError::Start(PluginError::ManifestAdmission(
                super::AdmissionFailure::WrongOrigin,
            ))) => true,
            Err(super::AdapterError::Start(PluginError::WithDiagnostics { cause, .. })) => {
                matches!(
                    *cause,
                    PluginError::ManifestAdmission(super::AdmissionFailure::WrongOrigin)
                )
            }
            _ => false,
        };
        assert!(refused_for_origin, "linked-origin hello is rejected");
    }
}
