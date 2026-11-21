// SPDX-License-Identifier: AGPL-3.0-or-later
//! Process resource limits through the production host and safe launcher.
#![cfg(all(unix, feature = "test-support"))]

use std::convert::Infallible;
use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use filament_exact::Cancel;
use filament_plugin_host::{FrameLimits, PluginError, PluginSession, RunLimits, SessionStart};
use serde_json::Value;

fn start(mode: &str, memory: u64, cpu: u64) -> PluginSession<Value, Value, Infallible> {
    let arguments = [OsString::from(mode)];
    PluginSession::start(
        SessionStart {
            executable: Path::new(env!("CARGO_BIN_EXE_fsl-resource-test-provider")),
            arguments: &arguments,
            limits: RunLimits {
                frames: FrameLimits {
                    max_frame_bytes: 4096,
                    max_total_bytes: 16384,
                },
                max_stderr_bytes: 4096,
                max_memory_bytes: memory,
                max_cpu_ms: cpu,
                cancellation_grace: Duration::from_millis(500),
            },
            cancel: &Cancel::new(),
        },
        |_| Ok(()),
    )
    .expect("provider hello within resource limits")
}

#[ix_trace_rs::trace("FR-013-AC-4")]
#[test]
fn resource_limits_memory_breach_is_a_typed_process_failure() {
    let mut session = start("memory", 64 * 1024 * 1024, 60_000);
    let error = session
        .request::<_, Value, Infallible>(&(), &Cancel::new(), |_, _| Ok(()))
        .expect_err("allocation over the memory ceiling must fail");
    assert!(
        matches!(error.primary_cause(), PluginError::Exited(status) if status.code() == Some(71)),
        "{error:?}"
    );
}

#[ix_trace_rs::trace("FR-013-AC-4")]
#[test]
fn resource_limits_cpu_breach_is_a_typed_process_failure() {
    use std::os::unix::process::ExitStatusExt;

    let mut session = start("cpu", 64 * 1024 * 1024, 1);
    let cancel = Cancel::new();
    let deadline = cancel.clone();
    let (finished, waiting) = std::sync::mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        if waiting.recv_timeout(Duration::from_secs(20)).is_err() {
            deadline.cancel(filament_exact::CancelCause::Deadline);
        }
    });
    let error = session
        .request::<_, Value, Infallible>(&(), &cancel, |_, _| Ok(()))
        .expect_err("CPU loop must be stopped by the OS");
    let _ = finished.send(());
    watchdog.join().expect("watchdog finishes");
    assert!(
        matches!(error.primary_cause(), PluginError::Exited(status) if status.signal() == Some(nix::libc::SIGKILL) || status.signal() == Some(nix::libc::SIGXCPU)),
        "{error:?}"
    );
}

#[ix_trace_rs::trace("FR-013-AC-3")]
#[test]
fn resource_limits_provider_below_limits_completes_with_rounded_cpu_ceiling() {
    let mut session = start("ordinary", 64 * 1024 * 1024, 1_001);
    let result: Value = session
        .request::<_, _, Infallible>(&(), &Cancel::new(), |_, _| Ok(()))
        .expect("provider below limits completes");
    assert_eq!(
        result,
        serde_json::json!({
            "completed": true, "memory_bytes": 64 * 1024 * 1024, "cpu_seconds": 2,
        })
    );
    session.shutdown().expect("orderly shutdown");
}

#[ix_trace_rs::trace("FR-013-AC-4", "FR-013-AC-10")]
#[test]
fn resource_limits_partial_response_preserves_the_memory_failure() {
    let mut session = start("memory-partial", 64 * 1024 * 1024, 60_000);
    let error = session
        .request::<_, Value, Infallible>(&(), &Cancel::new(), |_, _| Ok(()))
        .expect_err("partial response cannot hide the process failure");
    assert!(
        matches!(error.primary_cause(), PluginError::Exited(status) if status.code() == Some(71)),
        "{error:?}"
    );
}
