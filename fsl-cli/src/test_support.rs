// SPDX-License-Identifier: AGPL-3.0-or-later
//! Helpers for tests that must exercise a real FSL FR-331 provider request.

/// The backend FSL negotiates the [`with_provider_request`] item to.
pub const CAPTURE_BACKEND: &str = "fsl.test-capture";

type CaptureBody = Box<dyn FnOnce(&fsl_analyze::provider::ProviderRequest<'_>) + Send>;

/// An external engine that hands the first FR-331 request FSL builds for it
/// to a test body and leaves every item undecided.
struct Capture(std::sync::Mutex<Option<CaptureBody>>);

impl std::fmt::Debug for Capture {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Capture")
    }
}

impl fsl_analyze::provider::ExternalProvider for Capture {
    fn settle(
        &self,
        request: &fsl_analyze::provider::ProviderRequest<'_>,
        _cancel: &filament_exact::Cancel,
    ) -> Vec<fsl_analyze::provider::ProviderVerdict> {
        let body = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(body) = body {
            body(request);
        }
        vec![fsl_analyze::provider::ProviderVerdict::Unknown; request.items().len()]
    }
}

/// Run `body` over a real FSL FR-331 provider request: FSL analyzes the
/// Counter fixture's `Even` value-validity item on the external backend
/// [`CAPTURE_BACKEND`] and passes the request it builds to that backend.
pub fn with_provider_request<T: Send + 'static>(
    body: impl FnOnce(&fsl_analyze::provider::ProviderRequest<'_>) -> T + Send + 'static,
) -> T {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let directory = tempfile::tempdir().expect("a request folder");
    let mut program =
        std::fs::read_to_string(fixtures.join("analyze-counter.fsl")).expect("the Counter program");
    program.push_str("\nfunction Even using v(n: Integer): Boolean pure { n * 2 - n = n }\n");
    let program_path = directory.path().join("even.fsl");
    std::fs::write(&program_path, program).expect("the program");
    let mut document: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixtures.join("analyze-counter.json")).expect("the Counter request"),
    )
    .expect("the request is JSON");
    let request = &mut document["request"];
    request["program"]["source"]["file"] = program_path.to_string_lossy().into();
    request["models"][0]["source"]["file"] = fixtures
        .join("analyze-counter.semantic-ir.json")
        .to_string_lossy()
        .into();
    request["documents"][0]["file"] = fixtures
        .join("analyze-counter-start.json")
        .to_string_lossy()
        .into();
    request["items"] = serde_json::json!([
        {"claim": "Even", "kind": "value-validity", "backend": CAPTURE_BACKEND}]);
    let request_path = directory.path().join("analyze.json");
    std::fs::write(&request_path, document.to_string()).expect("the request file");

    let hello: crate::backend_provider::ManifestStage = serde_json::from_value(serde_json::json!({
        "contract_version": "filament.backend-provider/v1",
        "capability_vocabulary": "filament.capability-kind/v1",
        "manifest": [{
            "backend": CAPTURE_BACKEND,
            "advertises": [{"kind": "value-validity", "mode": "unbounded"}],
            "domains": ["integer"],
        }],
    }))
    .expect("the capture hello");
    let descriptor = hello
        .admit_process(1 << 20)
        .expect("the capture hello is admitted")
        .descriptor()
        .clone();
    let output = std::sync::Arc::new(std::sync::Mutex::new(None));
    let sink = std::sync::Arc::clone(&output);
    let capture = Capture(std::sync::Mutex::new(Some(Box::new(move |request| {
        let value = body(request);
        *sink
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(value);
    }))));
    let engine = fsl_analyze::analyze::Engine::external(
        descriptor.clone(),
        fsl_analyze::analyze::BackendTool::new("fsl-test-capture", "1").expect("a tool"),
        Box::new(capture),
    )
    .expect("the capture engine advertises unbounded value validity");
    let negotiation = filament_driver::AnalyzeNegotiation::with_providers([&descriptor])
        .expect("the engines register");
    fsl_command::command::analyze_with_providers(
        &request_path,
        &negotiation,
        vec![engine],
        &fsl_analyze::provider::ProviderOptions::default(),
        &filament_exact::Cancel::new(),
    )
    .expect("FSL analyzes the request");
    let value = output
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    value.expect("FSL settled the item on the capture backend")
}
