// SPDX-License-Identifier: AGPL-3.0-or-later
//! Driver-owned seam for caller-selected process providers.
//!
//! The wire authority is FSpec FR-331
//! (`agent-ix/filament-specification/spec/objects/interfaces/FR-331-backend-provider-envelope.md`)
//! and FR-462 (`filament-specification/spec/functional/tooling/FR-462-speak-the-plugin-wire.md`).
//! The seam forwards the FR-331 provider request FSL built
//! ([`fsl_analyze::provider::ProviderRequest`]) and returns the driver's
//! [`ProviderResults`]; it defines no FR-331 request of its own, holds no
//! package or routed item, and does not depend on the host crate.

use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use filament_exact::Cancel;
use fsl_route::BackendDescriptor;

use crate::{ProviderManifest, ProviderResults};

/// Caller-owned frame, stderr and cancellation limits, independent of the host crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessProviderLimits {
    /// Maximum encoded frame size.
    pub max_frame_bytes: u64,
    /// Maximum encoded bytes across the full plugin session; maps to the
    /// host's shared aggregate byte budget.
    pub max_total_bytes: u64,
    /// Maximum stderr bytes retained for diagnostics.
    pub max_stderr_bytes: u64,
    /// Maximum process memory in bytes on platforms with resource limits.
    pub max_memory_bytes: u64,
    /// Maximum process CPU time in milliseconds on supported platforms.
    pub max_cpu_ms: u64,
    /// Grace period for cancellation and shutdown.
    pub cancellation_grace: Duration,
}

/// One FSL provider request sent over an already-started process session.
pub struct ProcessProviderRequest<'a> {
    /// The FR-331 request FSL built after routing and negotiation: the
    /// package bytes, the routed items, the negotiated manifest entry and
    /// the FR-462 limits. Forwarded unchanged.
    pub request: &'a fsl_analyze::provider::ProviderRequest<'a>,
    /// Snapshot of all registered candidate manifests, from the one driver
    /// registry that admitted linked and process providers: the request
    /// stage's `manifest`.
    pub manifest: &'a [ProviderManifest],
    /// Caller-owned cancellation handle for this request.
    pub cancel: &'a Cancel,
}

/// A process provider whose hello has already been admitted by FSL.
///
/// `invoke` sends each request over this same session; implementations must
/// not restart the process, send another hello, or admit its descriptor again.
pub trait ProcessProviderSession {
    /// Typed result data returned by this provider.
    type ItemResult;
    /// Typed failure data returned by this provider.
    type ItemFailure;
    /// Typed adapter failure.
    type Error;

    /// Host diagnostics retained when graceful shutdown completes, including
    /// captured stderr and the process exit status when available.
    type Diagnostics;

    /// The descriptor returned by the one FSL admission of the hello manifest.
    fn descriptor(&self) -> &BackendDescriptor;

    /// The lossless driver-owned FR-331 manifest retained at admission.
    fn manifest(&self) -> &ProviderManifest;

    /// Send one routed request and return provider-specific typed results in
    /// the shared result carrier.
    fn invoke(
        &mut self,
        request: ProcessProviderRequest<'_>,
    ) -> Result<ProviderResults<Self::ItemResult, Self::ItemFailure>, Self::Error>;

    /// Request graceful shutdown and return the host's captured diagnostics.
    /// Callers should preserve or report these diagnostics. Dropping a session
    /// without calling `shutdown` relies on the host's `Drop` cleanup path.
    fn shutdown(self) -> Result<Self::Diagnostics, Self::Error>;
}

/// Start a named process provider and admit its hello before backend routing.
pub trait ProcessProvider {
    /// Started, hello-admitted session type.
    type Session: ProcessProviderSession<Error = Self::Error>;
    /// Typed adapter failure.
    type Error;

    /// Start only the caller-named executable, read one hello, and admit its
    /// manifest exactly once with caller-supplied `Process` origin.
    fn start(
        &self,
        executable: &Path,
        arguments: &[OsString],
        limits: ProcessProviderLimits,
        cancel: &Cancel,
    ) -> Result<Self::Session, Self::Error>;
}
