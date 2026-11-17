// SPDX-License-Identifier: AGPL-3.0-or-later
//! A `filament.plugin-wire/v1` process provider for the `fsl analyze
//! --provider-config` tests (FSL FR-635): it sends one `hello` whose FR-331
//! manifest advertises (`value-validity`, `<mode>`) over `integer`, then
//! answers every request item with `<result>` until `shutdown`.
//!
//! Usage: `fsl-test-provider <backend> <mode> <result> [<request-log>|-]
//! [<shutdown-marker>|-] [<descendant-ready> <descendant-leaked>]`.
//! When `<request-log>` is given, each request body is written to it; when
//! `<shutdown-marker>` is given, it is written once `shutdown` arrives. The
//! results `term` and `int` instead signal the parent `fsl` process while
//! the request is open (Linux and macOS), first starting a descendant that
//! writes `<descendant-ready>` and, if it survives three seconds,
//! `<descendant-leaked>`.

use std::io::{self, Read, Write};

use serde_json::{json, Value};

const PROTOCOL: &str = "filament.plugin-wire/v1";
const CONTRACT: &str = "filament.backend-provider/v1";
const VOCABULARY: &str = "filament.capability-kind/v1";

fn read_frame(input: &mut impl Read) -> Option<Value> {
    let mut prefix = [0_u8; 8];
    input.read_exact(&mut prefix).ok()?;
    let length = usize::try_from(u64::from_be_bytes(prefix)).ok()?;
    let mut payload = vec![0_u8; length];
    input.read_exact(&mut payload).ok()?;
    serde_json::from_slice(&payload).ok()
}

fn write_frame(output: &mut impl Write, value: &Value) -> io::Result<()> {
    let payload = serde_json::to_vec(value)?;
    let length = u64::try_from(payload.len()).map_err(io::Error::other)?;
    output.write_all(&length.to_be_bytes())?;
    output.write_all(&payload)?;
    output.flush()
}

/// `descendant <ready> <leaked>`: write `<ready>`, then write `<leaked>`
/// three seconds later unless the process is killed first.
fn descendant(ready: &str, leaked: &str) -> io::Result<()> {
    std::fs::write(ready, b"ready")?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    std::fs::write(leaked, b"leaked")
}

fn main() -> io::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let [kind, ready, leaked] = arguments.as_slice() {
        if kind == "descendant" {
            return descendant(ready, leaked);
        }
    }
    let [backend, mode, result, rest @ ..] = arguments.as_slice() else {
        return Err(io::Error::other(
            "usage: fsl-test-provider <backend> <mode> <result> [<request-log>|-] \
             [<shutdown-marker>|-] [<descendant-ready> <descendant-leaked>]",
        ));
    };
    let log = rest.first().filter(|log| log.as_str() != "-");
    let shutdown_marker = rest.get(1).filter(|marker| marker.as_str() != "-");
    // With `term` or `int`, a descendant in the provider's process group
    // that the host must reap before `fsl` re-raises the signal.
    let descendant_markers = rest.get(2).zip(rest.get(3));
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    write_frame(
        &mut output,
        &json!({
            "protocol": PROTOCOL, "id": 0, "kind": "hello",
            "body": {
                "contract_version": CONTRACT,
                "capability_vocabulary": VOCABULARY,
                "manifest": [{
                    "backend": backend,
                    "advertises": [{"kind": "value-validity", "mode": mode}],
                    "domains": ["integer"],
                    "tool": {"name": "fsl-test-provider", "version": env!("CARGO_PKG_VERSION")},
                }],
            },
        }),
    )?;
    while let Some(envelope) = read_frame(&mut input) {
        match envelope["kind"].as_str() {
            Some("shutdown") => {
                if let Some(marker) = shutdown_marker {
                    std::fs::write(marker, b"shutdown")?;
                }
                break;
            }
            Some("request") => {}
            _ => return Err(io::Error::other("unexpected frame")),
        }
        let body = &envelope["body"];
        if let Some(log) = log {
            std::fs::write(log, body.to_string())?;
        }
        // `term` and `int` send that signal to `fsl` while the request is
        // open, then wait for the host's `cancel` and stop answering.
        if let Some(signal) = match result.as_str() {
            "term" => Some("-TERM"),
            "int" => Some("-INT"),
            _ => None,
        } {
            if let Some((ready, leaked)) = descendant_markers {
                std::process::Command::new(std::env::current_exe()?)
                    .args(["descendant", ready, leaked])
                    .spawn()?;
                while !std::path::Path::new(ready).exists() {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
            let parent = std::os::unix::process::parent_id().to_string();
            std::process::Command::new("kill")
                .args([signal, &parent])
                .status()?;
            let _ = read_frame(&mut input);
            return Ok(());
        }
        let results: Vec<Value> = body["request"]["items"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|item| {
                json!({
                    "request_index": item["request_index"],
                    "backend": backend,
                    "kind": item["kind"],
                    "result": result,
                })
            })
            .collect();
        write_frame(
            &mut output,
            &json!({
                "protocol": PROTOCOL, "id": envelope["id"], "kind": "result",
                "body": {
                    "contract_version": CONTRACT,
                    "capability_vocabulary": VOCABULARY,
                    "results": results,
                    "dispositions": [],
                },
            }),
        )?;
    }
    Ok(())
}
