// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rust-only provider fixture for process resource-limit tests.

use std::io;

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
    let mut reader = FrameReader::new(io::stdin().lock(), limits);
    let request = reader
        .read::<Envelope>()
        .map_err(io::Error::other)?
        .ok_or_else(|| io::Error::other("request missing"))?;
    match mode.as_str() {
        "memory" => {
            // Fail deterministically on RLIMIT_AS/DATA without asking the
            // allocator to abort or relying on the host's available RAM.
            let mut allocation = Vec::<u8>::new();
            if allocation.try_reserve_exact(256 * 1024 * 1024).is_err() {
                std::process::exit(71);
            }
        }
        "cpu" => loop {
            std::hint::black_box(1_u64.wrapping_mul(3));
        },
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
                    serde_json::json!({
                        "completed": true,
                        "memory_bytes": rlimit::Resource::AS.get_soft()?,
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
