// SPDX-License-Identifier: AGPL-3.0-or-later
//! `fsl monitor` (FR-025, TC-039): the verb streams one event per clause status
//! change as each record arrives, then the summary, and exits with FSL's code.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use filament_exact::Cancel;
use fsl_command::command::{monitor, RunLimits};
use ix_trace_rs::trace;
use serde_json::{json, Value};

#[path = "common/test_provider_path.rs"]
mod test_provider_path;

const FSL: &str = env!("CARGO_BIN_EXE_fsl");
const INFINITE: &str = "filament.temporal.infinite-trace/v1";

struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("fsl-monitor-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a test folder");
        let unit = format!(
            "language \"ix:native\" edition \"1-draft\";\n\
             profile i = \"{INFINITE}\";\n\
             record Reading {{ level: Int[0, 9]; }}\n\
             temporal Reaches using i over (r: Reading) clock \"steps\" on origin {{ eventually holds(r.level = 2) }}\n\
             temporal Never using i over (r: Reading) clock \"steps\" on origin {{ always holds(r.level < 2) }}\n"
        );
        std::fs::write(path.join("unit.native"), unit).expect("the unit");
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// A `native-monitor/1` request over the given clauses, written as `name`.
    fn request(&self, name: &str, clauses: &[&str]) -> PathBuf {
        let clauses: Vec<Value> = clauses.iter().map(|name| json!({"name": name})).collect();
        let request = json!({"format": "native-monitor/1", "request": {
            "models": [],
            "program": {"source": {"file": "unit.native", "authority": "agent-ix",
                "identity": "test:monitor-cli", "document": "MonitorCli",
                "formal_revision": 1}},
            "clauses": clauses, "trace": {"kind": "finite"}}});
        let path = self.path(name);
        std::fs::write(&path, request.to_string()).expect("a request");
        path
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn level(value: i64) -> String {
    json!({"observation": {"sample": {"level": {"integer": value.to_string()}}}}).to_string()
}

fn records(levels: &[i64]) -> Vec<String> {
    let mut lines: Vec<String> = levels.iter().copied().map(level).collect();
    lines.push(json!({"end": true}).to_string());
    lines
}

fn fsl_with_stdin(arguments: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(FSL)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn fsl");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .or_else(|error| {
            if error.kind() == std::io::ErrorKind::BrokenPipe {
                Ok(())
            } else {
                Err(error)
            }
        })
        .expect("write records");
    child.wait_with_output().expect("fsl output")
}

fn json_lines(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|error| panic!("{error}: {line}")))
        .collect()
}

/// The events and summary `command::monitor` gives in process, canonically encoded.
fn library_lines(request: &Path, lines: &[String]) -> (Vec<String>, u8) {
    let mut encoded = Vec::new();
    let summary = monitor(
        request,
        lines.iter().map(|line| Ok(line.clone().into_bytes())),
        &mut |event| encoded.push(canonical(event.as_value())),
        fsl_command::command::MonitorProviders {
            negotiate: &filament_driver::AnalyzeNegotiation::fsl().expect("the engines register"),
            providers: filament_driver::run_engines(Vec::new())
                .expect("the hybrid engine registers"),
            options: &fsl_analyze::provider::ProviderOptions::default(),
        },
        RunLimits::default(),
        &Cancel::new(),
    )
    .expect("the stream runs");
    encoded.push(canonical(summary.as_value()));
    (encoded, summary.exit_code)
}

fn canonical(value: &Value) -> String {
    ix_cli_kit::json::encode_canonical(value, false).expect("canonical JSON")
}

fn stdout_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[trace("TC-039", "FR-025-AC-1")]
#[test]
fn a_recorded_trace_streams_events_then_the_summary_with_the_verdict_exit() {
    let folder = Folder::new("verdict");
    let violated = folder.request("violated.json", &["Reaches", "Never"]);
    let output = fsl_with_stdin(
        &["monitor", violated.to_str().unwrap(), "--format", "json"],
        &records(&[0, 1, 2, 1]).join("\n"),
    );
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    let lines = json_lines(&output);
    let (summary, events) = lines.split_last().expect("lines");
    assert_eq!(summary["format"], "filament.monitor-summary/1", "{summary}");
    assert!(!events.is_empty());
    assert!(events
        .iter()
        .all(|event| event["format"] == "filament.monitor-event/1"));
    assert!(
        events.iter().any(|event| event["clause"] == "Never"
            && event["record"] == 2
            && event["settled"] == true
            && event["category"] == "violation"),
        "{events:?}"
    );

    let holding = folder.request("holding.json", &["Never"]);
    let output = fsl_with_stdin(
        &["monitor", holding.to_str().unwrap(), "--format", "json"],
        &records(&[0, 1, 0]).join("\n"),
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let lines = json_lines(&output);
    assert_eq!(
        lines.last().unwrap()["format"],
        "filament.monitor-summary/1"
    );
}

#[trace("TC-039", "FR-025-AC-2")]
#[test]
fn the_events_and_summary_equal_command_monitor_byte_for_byte() {
    let folder = Folder::new("parity");
    let request = folder.request("request.json", &["Reaches", "Never"]);
    let lines = records(&[0, 1, 2, 1]);
    let (expected, expected_exit) = library_lines(&request, &lines);
    let output = fsl_with_stdin(
        &["monitor", request.to_str().unwrap(), "--format", "json"],
        &lines.join("\n"),
    );
    assert_eq!(stdout_lines(&output), expected);
    assert_eq!(output.status.code(), Some(i32::from(expected_exit)));
}

#[trace("TC-039", "FR-025-AC-3")]
#[test]
fn a_bad_request_is_refused_before_any_record_with_exit_20() {
    let folder = Folder::new("refused");
    let unknown = folder.request("unknown.json", &["Absent"]);
    let output = fsl_with_stdin(
        &["monitor", unknown.to_str().unwrap(), "--format", "json"],
        &records(&[0, 1]).join("\n"),
    );
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    let lines = json_lines(&output);
    assert_eq!(lines.len(), 1, "only the envelope: {lines:?}");
    assert_eq!(lines[0]["operation"], "monitor", "{}", lines[0]);
    assert_eq!(lines[0]["category"], "refusal", "{}", lines[0]);

    std::fs::write(folder.path("garbage.json"), "not json").unwrap();
    let output = fsl_with_stdin(
        &["monitor", folder.path("garbage.json").to_str().unwrap()],
        "",
    );
    assert_eq!(output.status.code(), Some(20), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.starts_with("monitor: refusal (exit 20)"), "{text}");

    let missing = fsl_with_stdin(&["monitor"], "");
    assert_eq!(missing.status.code(), Some(20));
    assert!(missing.stderr.is_empty(), "{missing:?}");
    assert!(String::from_utf8_lossy(&missing.stdout).contains("usage: fsl monitor"));
}

const PROVIDER_BACKEND: &str = "fsl.test-recording-provider";

/// FR-025-AC-4 (TC-044, FSL FR-644-AC-8 and AC-11): `fsl monitor` accepts
/// `--provider-config` and `--provider-limit`, starts the configured test
/// plugin, and FSL routes the request's `items` to it before the first
/// record: the events equal the run's without `items` and without the
/// option, the summary carries each item `proved` by the plugin, `trusted`,
/// and the registry naming it. The limit reaches the plugin's request, a
/// request without `items` sends the plugin nothing, and no option is
/// refused.
#[trace("TC-044", "FR-025-AC-4")]
#[test]
fn provider_config_routes_the_request_items_to_the_started_plugin() {
    let folder = Folder::new("providers");
    let unit = std::fs::read_to_string(folder.path("unit.native")).expect("the unit");
    // A profile line precedes the declarations.
    let mut unit = unit.replacen(
        "profile i = ",
        "profile v = \"filament.value.complete/v1\";\nprofile i = ",
        1,
    );
    unit.push_str("function Even using v(n: Integer): Boolean pure { n * 2 - n = n }\n");
    std::fs::write(folder.path("unit.native"), unit).expect("the unit with Even");
    let plain = folder.request("plain.json", &["Reaches", "Never"]);
    let mut request: Value =
        serde_json::from_slice(&std::fs::read(&plain).expect("the request")).expect("json");
    request["request"]["items"] =
        json!([{"claim": "Even", "kind": "value-validity", "backend": PROVIDER_BACKEND}]);
    let routed = folder.path("monitor.json");
    std::fs::write(&routed, request.to_string()).expect("the routed request");
    let log = folder.path("requests.log");
    let config = folder.path("providers.json");
    std::fs::write(
        &config,
        json!({"format": "filament.provider-config/v1", "providers": [{
            "path": test_provider_path::executable(),
            "arguments": [PROVIDER_BACKEND, "unbounded", "proved", log.to_str().unwrap()]}]})
        .to_string(),
    )
    .expect("the provider configuration");
    let input = folder.path("records.jsonl");
    std::fs::write(&input, records(&[0, 1, 2, 1, 0]).join("\n")).expect("the records");
    let run = |request: &Path, options: &[&str]| {
        Command::new(FSL)
            .arg("monitor")
            .arg(request)
            .args(options)
            .arg("--input")
            .arg(&input)
            .args(["--format", "json"])
            .output()
            .expect("run fsl monitor")
    };
    let config_arguments = ["--provider-config", config.to_str().unwrap()];
    let bare = run(&plain, &[]);
    assert_eq!(bare.status.code(), Some(10), "{bare:?}");
    let bare_lines = json_lines(&bare);
    let (_, bare_events) = bare_lines.split_last().expect("a summary line");

    let output = run(&routed, &config_arguments);
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("QSL-2132"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("QSL-2132"));
    let lines = json_lines(&output);
    let (summary, events) = lines.split_last().expect("a summary line");
    assert_eq!(events, bare_events);
    let items = summary["items"]
        .as_array()
        .unwrap_or_else(|| panic!("a summary `items` member: {summary}"));
    assert!(!items.is_empty(), "{summary}");
    for item in items {
        assert_eq!(item["record"]["result"], "proved", "{item}");
        assert_eq!(item["record"]["backend"], PROVIDER_BACKEND, "{item}");
        assert_eq!(item["record"]["certification"], "trusted", "{item}");
    }
    assert!(
        summary["registry"]
            .as_array()
            .is_some_and(|registry| registry
                .iter()
                .any(|row| row["backend"] == PROVIDER_BACKEND)),
        "{summary}"
    );
    assert!(!std::fs::read_to_string(&log)
        .expect("the plugin logged its request")
        .is_empty());

    std::fs::remove_file(&log).expect("the request log");
    let limited = run(
        &routed,
        &[
            "--provider-config",
            config.to_str().unwrap(),
            "--provider-limit",
            "deadline_ms=4096",
        ],
    );
    assert_eq!(limited.status.code(), Some(10), "{limited:?}");
    let logged = std::fs::read_to_string(&log).expect("the plugin logged its request");
    let sent: Value = serde_json::from_str(&logged).expect("a logged request body");
    assert_eq!(sent["request"]["deadline_ms"], 4096, "{sent}");

    std::fs::remove_file(&log).expect("the request log");
    let without = run(&plain, &config_arguments);
    assert_eq!(without.status.code(), Some(10), "{without:?}");
    assert!(
        !std::fs::read_to_string(&log).is_ok_and(|requests| !requests.is_empty()),
        "a request without items sent the plugin a request"
    );
    assert_eq!(json_lines(&without), bare_lines);
}

#[trace("TC-039", "FR-025-AC-5")]
#[test]
fn a_bad_record_becomes_a_refusal_event_and_the_stream_continues() {
    let folder = Folder::new("badrecord");
    let request = folder.request("request.json", &["Reaches", "Never"]);
    let mut lines = records(&[0, 1, 2]);
    lines.insert(
        1,
        json!({"observation": {"sample": {"level": {"integer": "0"}}}, "bogus": 1}).to_string(),
    );
    let output = fsl_with_stdin(
        &["monitor", request.to_str().unwrap(), "--format", "json"],
        &lines.join("\n"),
    );
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    let all = json_lines(&output);
    let refusal = all
        .iter()
        .find(|event| event["category"] == "refusal")
        .unwrap_or_else(|| panic!("a refusal event: {all:?}"));
    assert_eq!(refusal["record"], 1, "{refusal}");
    let summary = all.last().unwrap();
    assert_eq!(summary["refused"], 1, "{summary}");
    let (expected, _) = library_lines(&request, &lines);
    assert_eq!(stdout_lines(&output), expected);
}

#[trace("TC-039", "FR-025-AC-6")]
#[test]
fn each_event_is_written_before_the_next_record_is_sent() {
    let folder = Folder::new("live");
    let request = folder.request("request.json", &["Never"]);
    let mut child = Command::new(FSL)
        .args(["monitor", request.to_str().unwrap(), "--format", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn fsl");
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let wait = Duration::from_secs(60);
    // Record 1 settles `Never`; its event must arrive while stdin is still open.
    for line in [level(0), level(2)] {
        writeln!(stdin, "{line}").expect("send a record");
    }
    let mut settled = None;
    while settled.is_none() {
        let line = receiver
            .recv_timeout(wait)
            .expect("an event before stdin closes");
        let event: Value = serde_json::from_str(&line).expect("an event line");
        if event["clause"] == "Never" && event["settled"] == true {
            settled = Some(event);
        }
    }
    drop(stdin);
    let summary: Value =
        serde_json::from_str(&receiver.recv_timeout(wait).expect("the summary")).expect("json");
    assert_eq!(summary["format"], "filament.monitor-summary/1");
    assert_eq!(child.wait().expect("exit").code(), Some(10));
}

#[trace("TC-039", "FR-025-AC-7")]
#[test]
fn input_file_and_standard_input_give_the_same_stream() {
    let folder = Folder::new("input");
    let request = folder.request("request.json", &["Reaches", "Never"]);
    let lines = records(&[0, 2]);
    let file = folder.path("records.jsonl");
    std::fs::write(&file, lines.join("\n")).unwrap();
    let from_file = Command::new(FSL)
        .args([
            "monitor",
            request.to_str().unwrap(),
            "--format",
            "json",
            "--input",
        ])
        .arg(&file)
        .output()
        .expect("run fsl");
    let from_stdin = fsl_with_stdin(
        &["monitor", request.to_str().unwrap(), "--format", "json"],
        &lines.join("\n"),
    );
    assert_eq!(from_file.stdout, from_stdin.stdout);
    assert_eq!(from_file.status.code(), from_stdin.status.code());
    let missing = Command::new(FSL)
        .args(["monitor", request.to_str().unwrap(), "--input"])
        .arg(folder.path("absent.jsonl"))
        .output()
        .expect("run fsl");
    assert_eq!(missing.status.code(), Some(20));
    assert!(missing.stderr.is_empty(), "no stderr: {missing:?}");
    let text = String::from_utf8_lossy(&missing.stdout);
    assert!(text.starts_with("monitor: refusal (exit 20)"), "{text}");
    assert!(text.contains("absent.jsonl"), "{text}");

    let missing = Command::new(FSL)
        .args([
            "monitor",
            request.to_str().unwrap(),
            "--format",
            "json",
            "--input",
        ])
        .arg(folder.path("absent.jsonl"))
        .output()
        .expect("run fsl");
    assert_eq!(missing.status.code(), Some(20));
    let lines = json_lines(&missing);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0]["operation"], "monitor");
    assert_eq!(lines[0]["category"], "refusal");
    assert!(
        lines[0].to_string().contains("absent.jsonl"),
        "{}",
        lines[0]
    );
}

#[trace("TC-039", "FR-025-AC-8")]
#[test]
fn text_mode_writes_event_lines_then_clause_lines_and_the_counts() {
    let folder = Folder::new("text");
    let request = folder.request("request.json", &["Reaches", "Never"]);
    let output = fsl_with_stdin(
        &["monitor", request.to_str().unwrap(), "--format", "text"],
        &records(&[0, 1, 2, 1]).join("\n"),
    );
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    assert_eq!(
        stdout_lines(&output),
        [
            "0 Reaches pending pending",
            "0 Never pending pending",
            "2 Reaches success settled",
            "2 Never violation settled",
            "Reaches success settled",
            "Never violation settled",
            "records 5 refused 0",
        ]
    );
}

/// FR-025, FSL FR-624: SIGTERM or SIGINT while the record stream is still
/// open ends the run at once, with no further record read, as FSL's
/// cancelled monitor run: its outcome document is the last line and the
/// process exits 22 rather than by the signal. QSL-2284.
#[trace("TC-039", "FR-025-AC-9")]
#[test]
fn a_signal_ends_an_open_stream_as_a_cancelled_run() {
    for signal in ["-TERM", "-INT"] {
        let folder = Folder::new(&format!("signal{signal}"));
        let request = folder.request("request.json", &["Never"]);
        let mut child = Command::new(FSL)
            .args(["monitor", request.to_str().unwrap(), "--format", "json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn fsl");
        let mut stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let wait = Duration::from_secs(60);
        for line in [level(0), level(2)] {
            writeln!(stdin, "{line}").expect("send a record");
        }
        // The first event shows the run is live; stdin stays open.
        receiver.recv_timeout(wait).expect("an event");
        let killed = Command::new("kill")
            .args([signal, &child.id().to_string()])
            .status()
            .expect("send the signal");
        assert!(killed.success());
        let status = child.wait().expect("exit");
        drop(stdin);
        assert_eq!(status.code(), Some(22), "{signal}");
        let last = receiver
            .iter()
            .last()
            .expect("an outcome document after the signal");
        let document: Value = serde_json::from_str(&last).expect("json");
        assert_eq!(document["category"], "incomplete", "{signal}: {document}");
    }
}
