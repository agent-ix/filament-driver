// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rust process fixtures for the `fsl-cli` process-provider tests.

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let Some(kind) = std::env::args().nth(1) else {
        return;
    };
    if kind == "descendant" {
        run_descendant();
    } else if matches!(kind.as_str(), "echo" | "fr331" | "wrong-result" | "signal") {
        run_plugin_fixture(&kind);
    }
}

fn write_frame(value: &Value) {
    let payload = serde_json::to_vec(value).expect("serialize fixture frame");
    let length = u64::try_from(payload.len()).expect("fixture frame length fits u64");
    let mut output = std::io::stdout().lock();
    output
        .write_all(&length.to_be_bytes())
        .and_then(|()| output.write_all(&payload))
        .and_then(|()| output.flush())
        .expect("write fixture frame");
}

fn read_frame() -> Value {
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

fn wire_frame(id: u64, kind: &str, body: Value) -> Value {
    json!({
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
        "echo" | "wrong-result" => (ECHO, json!({"identity": ECHO, "advertises": []})),
        "fr331" => (
            FR331,
            json!({
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
            json!({"identity": "agent-ix/provider/signal-test", "advertises": []}),
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
    write_frame(&wire_frame(0, "hello", hello_body));
    loop {
        let request = read_frame();
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
                    write_frame(&wire_frame(
                        request["id"].as_u64().expect("request id"),
                        "result",
                        json!({"identity": "agent-ix/provider/other", "items": []}),
                    ));
                    break;
                }
                let mut body = json!({"identity": identity, "items": []});
                if kind == "echo" || kind == "fr331" {
                    body["items"] = request["body"]["items"].clone();
                }
                write_frame(&wire_frame(
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
}

fn signal_fixture_paths() -> (String, PathBuf, PathBuf, PathBuf, PathBuf) {
    let stage = std::env::var("FILAMENT_PLUGIN_SIGNAL_STAGE").expect("signal stage");
    let ready =
        PathBuf::from(std::env::var_os("FILAMENT_PLUGIN_SIGNAL_READY").expect("ready path"));
    let release =
        PathBuf::from(std::env::var_os("FILAMENT_PLUGIN_SIGNAL_RELEASE").expect("release path"));
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
    let status = Command::new("/bin/kill")
        .args([signal, &parent])
        .status()
        .expect("run system signal utility");
    assert!(status.success(), "send lifecycle signal to test process");
}

fn wait_for_release(release: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !release.is_file() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
    }
    assert!(release.is_file(), "signal fixture released");
}

#[allow(
    clippy::zombie_processes,
    reason = "The fixture deliberately detaches a descendant so CLI signal tests can verify process-group cancellation."
)]
fn spawn_descendant(marker: &Path) {
    let executable = std::env::current_exe().expect("fixture executable");
    Command::new(executable)
        .arg("descendant")
        .env("FILAMENT_PLUGIN_SIGNAL_DESCENDANT_CHILD", marker)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn process-group descendant");
}

fn run_signal_hello() {
    let (stage, ready, release, _, plugin_marker) = signal_fixture_paths();
    std::fs::write(ready, b"ready").expect("write hello readiness");
    wait_for_release(&release);
    signal_parent(&stage);
    thread::sleep(Duration::from_secs(2));
    std::fs::write(plugin_marker, b"leaked").expect("write late plugin marker");
}

fn run_signal_request() {
    let (stage, ready, release, descendant, plugin_marker) = signal_fixture_paths();
    spawn_descendant(&descendant);
    std::fs::write(ready, b"ready").expect("write request readiness");
    wait_for_release(&release);
    signal_parent(&stage);
    let cancel = read_frame();
    assert_eq!(cancel["kind"], "cancel");
    thread::sleep(Duration::from_secs(2));
    std::fs::write(plugin_marker, b"leaked").expect("write late plugin marker");
}

fn run_signal_shutdown() {
    let (stage, ready, release, descendant, plugin_marker) = signal_fixture_paths();
    spawn_descendant(&descendant);
    std::fs::write(ready, b"ready").expect("write shutdown readiness");
    wait_for_release(&release);
    signal_parent(&stage);
    thread::sleep(Duration::from_secs(2));
    std::fs::write(plugin_marker, b"leaked").expect("write late plugin marker");
}

fn run_descendant() {
    let marker = std::env::var_os("FILAMENT_PLUGIN_SIGNAL_DESCENDANT_CHILD")
        .expect("descendant marker path");
    thread::sleep(Duration::from_millis(600));
    std::fs::write(marker, b"leaked").expect("write descendant marker");
}
