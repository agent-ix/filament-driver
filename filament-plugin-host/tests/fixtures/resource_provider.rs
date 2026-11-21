// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rust-only provider fixture for process resource-limit tests.

use std::io::{self, Write};

use filament_plugin_host::{Envelope, FrameLimits, FrameReader, FrameWriter, Kind, PROTOCOL};

fn main() -> io::Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let limits = FrameLimits {
        max_frame_bytes: 4096,
        max_total_bytes: 16384,
    };
    let mut writer = FrameWriter::new(io::stdout().lock(), limits);
    writer
        .write(&Envelope {
            protocol: PROTOCOL.to_owned(),
            id: 0,
            kind: Kind::Hello,
            body: serde_json::json!({}),
        })
        .map_err(io::Error::other)?;
    if mode == "cpu-before-request" {
        fsl_test_provider::consume_resource(fsl_test_provider::ResourceWorkload::Cpu);
    }
    let mut reader = FrameReader::new(io::stdin().lock(), limits);
    let request = reader
        .read::<Envelope>()
        .map_err(io::Error::other)?
        .ok_or_else(|| io::Error::other("request missing"))?;
    if mode.ends_with("-partial") {
        let mut output = io::stdout().lock();
        output.write_all(&64_u64.to_be_bytes())?;
        output.write_all(b"{\"")?;
        output.flush()?;
    }
    match mode.as_str() {
        "memory" | "memory-partial" => {
            fsl_test_provider::consume_resource(fsl_test_provider::ResourceWorkload::Memory)
        }
        "cpu" | "cpu-partial" => {
            fsl_test_provider::consume_resource(fsl_test_provider::ResourceWorkload::Cpu)
        }
        _ => {}
    }
    writer
        .write(&Envelope {
            protocol: PROTOCOL.to_owned(),
            id: request.id,
            kind: Kind::Result,
            body: {
                #[cfg(unix)]
                {
                    #[cfg(target_os = "linux")]
                    let memory_bytes = Some(rlimit::Resource::AS.get_soft()?);
                    #[cfg(not(target_os = "linux"))]
                    let memory_bytes: Option<u64> = None;
                    serde_json::json!({
                        "completed": true,
                        "memory_bytes": memory_bytes,
                        "cpu_seconds": rlimit::Resource::CPU.get_soft()?,
                    })
                }
                #[cfg(not(unix))]
                {
                    serde_json::json!({"completed": true})
                }
            },
        })
        .map_err(io::Error::other)?;
    let _ = reader.read::<Envelope>().map_err(io::Error::other)?;
    Ok(())
}
