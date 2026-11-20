// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real-process FSL CLI contracts against one explicitly configured prebuilt binary.

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;
use thiserror::Error;

/// Stable distinctions for setup and capture failures, separate from FSL exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractCode {
    /// No executable was selected.
    MissingBinary,
    /// A fixture path escapes its case.
    FixturePath,
    /// A filesystem or process operation failed.
    Io,
    /// The child exceeded its execution deadline.
    Timeout,
    /// A captured stream exceeded its memory bound.
    OutputLimit,
    /// A streaming invocation emitted no record.
    EmptyJsonLines,
    /// Output was not valid JSON.
    Json,
    /// Output was not valid UTF-8.
    Utf8,
}

impl ContractCode {
    /// Stable code text, independent of an operating-system error message.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingBinary => "missing-binary",
            Self::FixturePath => "fixture-path",
            Self::Io => "io",
            Self::Timeout => "timeout",
            Self::OutputLimit => "output-limit",
            Self::EmptyJsonLines => "empty-json-lines",
            Self::Json => "invalid-json",
            Self::Utf8 => "invalid-utf8",
        }
    }

    /// Every declared contract code.
    pub const fn all() -> &'static [Self] {
        &[
            Self::MissingBinary,
            Self::FixturePath,
            Self::Io,
            Self::Timeout,
            Self::OutputLimit,
            Self::EmptyJsonLines,
            Self::Json,
            Self::Utf8,
        ]
    }

    /// Look up a declared code without coercing unknown text.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|item| item.as_str() == code)
    }
}

/// A setup, subprocess or captured-output error, distinct from an FSL refusal.
#[derive(Debug, Error)]
pub enum ContractError {
    /// The caller did not select the binary under test.
    #[error("set FSL to the prebuilt binary under test")]
    MissingBinary,
    /// A fixture path would leave the case's temporary working directory.
    #[error("fixture path must be relative without parent traversal: {0}")]
    FixturePath(PathBuf),
    /// A filesystem or subprocess operation failed.
    #[error("{operation}: {source}")]
    Io {
        /// The operation that failed.
        operation: &'static str,
        /// The original operating-system error.
        #[source]
        source: io::Error,
    },
    /// A child exceeded the bounded contract execution time and was reaped.
    #[error("fsl exceeded its {0:?} execution deadline")]
    Timeout(Duration),
    /// Capturing one stream exceeded the bounded in-memory capture size.
    #[error("{0} exceeded the 1 MiB contract capture limit")]
    OutputLimit(&'static str),
    /// A streaming invocation produced no JSON record.
    #[error("stdout contains no JSON Lines records")]
    EmptyJsonLines,
    /// Machine output was not the required JSON document or JSON Lines.
    #[error("stdout is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// A textual stream was not UTF-8.
    #[error("{stream} is not UTF-8: {source}")]
    Utf8 {
        /// The stream being decoded.
        stream: &'static str,
        /// The original UTF-8 error.
        #[source]
        source: std::str::Utf8Error,
    },
}

impl ContractError {
    /// Classify a failure without matching its human-readable message.
    pub const fn code(&self) -> ContractCode {
        match self {
            Self::MissingBinary => ContractCode::MissingBinary,
            Self::FixturePath(_) => ContractCode::FixturePath,
            Self::Io { .. } => ContractCode::Io,
            Self::Timeout(_) => ContractCode::Timeout,
            Self::OutputLimit(_) => ContractCode::OutputLimit,
            Self::EmptyJsonLines => ContractCode::EmptyJsonLines,
            Self::Json(_) => ContractCode::Json,
            Self::Utf8 { .. } => ContractCode::Utf8,
        }
    }
}

fn io_error(operation: &'static str, source: io::Error) -> ContractError {
    ContractError::Io { operation, source }
}

/// An isolated FSL working directory. It owns no executable and never builds one.
#[derive(Debug)]
pub struct CliCase {
    binary: PathBuf,
    directory: TempDir,
}

impl CliCase {
    /// Select the mandatory `FSL` executable and create an isolated working directory.
    pub fn from_environment() -> Result<Self, ContractError> {
        let binary = std::env::var_os("FSL").ok_or(ContractError::MissingBinary)?;
        let binary = fs::canonicalize(binary).map_err(|e| io_error("resolve FSL", e))?;
        let directory = tempfile::tempdir().map_err(|e| io_error("create CLI case", e))?;
        Ok(Self { binary, directory })
    }

    /// The case's working directory, for inspecting artifacts written by the CLI.
    pub fn directory(&self) -> &Path {
        self.directory.path()
    }

    /// Write an original input below the working directory, creating parent folders.
    pub fn write(&self, relative: impl AsRef<Path>, bytes: &[u8]) -> Result<(), ContractError> {
        let relative = relative.as_ref();
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(ContractError::FixturePath(relative.to_owned()));
        }
        let path = self.directory().join(relative);
        let mut component_path = self.directory().to_owned();
        for component in relative.components() {
            component_path.push(component.as_os_str());
            if component_path.is_symlink() {
                return Err(ContractError::FixturePath(relative.to_owned()));
            }
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| io_error("create fixture folder", e))?;
        }
        fs::write(path, bytes).map_err(|e| io_error("write fixture", e))
    }

    /// Execute the real CLI with closed stdin, preserving raw output and process status.
    pub fn run<S: AsRef<OsStr>>(&self, arguments: &[S]) -> Result<Captured, ContractError> {
        self.run_stdin(arguments, &[])
    }

    /// Execute with the supplied stdin bytes, closing stdin after writing them.
    ///
    /// Input and both output streams are handled concurrently. An early input
    /// refusal may close stdin with BrokenPipe; its real status/output still win.
    pub fn run_stdin<S: AsRef<OsStr>>(
        &self,
        arguments: &[S],
        input: &[u8],
    ) -> Result<Captured, ContractError> {
        let start = Instant::now();
        let mut child = Command::new(&self.binary)
            .args(arguments)
            .current_dir(self.directory())
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| io_error("spawn FSL", e))?;
        capture_process(&mut child, input, start, Duration::from_secs(30))
    }
}

// The caller creates a fresh process group before spawn. A timeout therefore
// terminates only this invocation and descendants that inherited its pipes.
struct Invocation<'a> {
    child: &'a mut std::process::Child,
    group: rustix::process::Pid,
    complete: bool,
}

impl Drop for Invocation<'_> {
    fn drop(&mut self) {
        if !self.complete {
            let _ = rustix::process::kill_process_group(self.group, rustix::process::Signal::KILL);
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn nonblocking(fd: &impl std::os::fd::AsFd) -> Result<(), ContractError> {
    let flags = rustix::fs::fcntl_getfl(fd).map_err(|e| io_error("read pipe flags", e.into()))?;
    rustix::fs::fcntl_setfl(fd, flags | rustix::fs::OFlags::NONBLOCK)
        .map_err(|e| io_error("set nonblocking pipe", e.into()))
}

#[derive(Default)]
struct StreamCapture {
    bytes: Vec<u8>,
    exceeded: bool,
}

impl StreamCapture {
    fn read(&mut self, stream: &mut Option<impl Read>) -> Result<(), ContractError> {
        let Some(reader) = stream.as_mut() else {
            return Ok(());
        };
        let mut bytes = [0; 16_384];
        match reader.read(&mut bytes) {
            Ok(0) => {
                *stream = None;
            }
            Ok(count) => {
                let remaining = 1_048_576 - self.bytes.len();
                self.bytes.extend_from_slice(&bytes[..count.min(remaining)]);
                self.exceeded |= count > remaining;
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(io_error("read FSL output", error)),
        }
        Ok(())
    }
}

fn capture_process(
    child: &mut std::process::Child,
    input: &[u8],
    start: Instant,
    budget: Duration,
) -> Result<Captured, ContractError> {
    let group = rustix::process::Pid::from_child(child);
    let invocation = Invocation {
        child,
        group,
        complete: false,
    };
    let mut invocation = invocation;
    let mut stdin = invocation.child.stdin.take();
    let mut stdout = invocation.child.stdout.take();
    let mut stderr = invocation.child.stderr.take();
    nonblocking(stdin.as_ref().expect("piped stdin"))?;
    nonblocking(stdout.as_ref().expect("piped stdout"))?;
    nonblocking(stderr.as_ref().expect("piped stderr"))?;
    let mut sent = 0;
    let mut out = StreamCapture::default();
    let mut err = StreamCapture::default();
    let mut status = None;
    loop {
        if start.elapsed() >= budget {
            return Err(ContractError::Timeout(budget));
        }
        if sent == input.len() {
            stdin = None;
        }
        if let Some(writer) = stdin.as_mut() {
            let end = input.len().min(sent.saturating_add(16_384));
            match writer.write(&input[sent..end]) {
                Ok(0) => return Err(io_error("write FSL stdin", io::ErrorKind::WriteZero.into())),
                Ok(count) => sent += count,
                Err(error) if error.kind() == io::ErrorKind::BrokenPipe => stdin = None,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(error) => return Err(io_error("write FSL stdin", error)),
            }
        }
        out.read(&mut stdout)?;
        err.read(&mut stderr)?;
        if status.is_none() {
            status = invocation
                .child
                .try_wait()
                .map_err(|e| io_error("wait for FSL", e))?;
        }
        if let Some(status) = status {
            if stdin.is_none() && stdout.is_none() && stderr.is_none() {
                invocation.complete = true;
                if out.exceeded {
                    return Err(ContractError::OutputLimit("stdout"));
                }
                if err.exceeded {
                    return Err(ContractError::OutputLimit("stderr"));
                }
                return Ok(Captured(Output {
                    status,
                    stdout: out.bytes,
                    stderr: err.bytes,
                }));
            }
        }
        let remaining = budget
            .saturating_sub(start.elapsed())
            .min(Duration::from_millis(10));
        let timeout = rustix::event::Timespec {
            tv_sec: 0,
            tv_nsec: i64::from(remaining.subsec_nanos()),
        };
        let mut descriptors = Vec::new();
        if let Some(fd) = stdin.as_ref() {
            descriptors.push(rustix::event::PollFd::new(
                fd,
                rustix::event::PollFlags::OUT,
            ));
        }
        if let Some(fd) = stdout.as_ref() {
            descriptors.push(rustix::event::PollFd::new(fd, rustix::event::PollFlags::IN));
        }
        if let Some(fd) = stderr.as_ref() {
            descriptors.push(rustix::event::PollFd::new(fd, rustix::event::PollFlags::IN));
        }
        match rustix::event::poll(&mut descriptors, Some(&timeout)) {
            Ok(_) | Err(rustix::io::Errno::INTR) => {}
            Err(error) => return Err(io_error("poll FSL pipes", error.into())),
        }
    }
}

/// The unmodified status, stdout and stderr of a completed real FSL invocation.
#[derive(Debug)]
pub struct Captured(pub Output);

impl Captured {
    /// Decode exactly one JSON value, refusing text or trailing non-JSON output.
    pub fn json(&self) -> Result<Value, ContractError> {
        Ok(serde_json::from_slice(&self.0.stdout)?)
    }

    /// Decode every line of a streaming command independently as JSON.
    pub fn json_lines(&self) -> Result<Vec<Value>, ContractError> {
        if self.0.stdout.is_empty() {
            return Err(ContractError::EmptyJsonLines);
        }
        self.stdout_text()?
            .lines()
            .map(|line| Ok(serde_json::from_str(line)?))
            .collect()
    }

    /// Decode stdout without replacing invalid UTF-8 bytes.
    pub fn stdout_text(&self) -> Result<&str, ContractError> {
        std::str::from_utf8(&self.0.stdout).map_err(|source| ContractError::Utf8 {
            stream: "stdout",
            source,
        })
    }

    /// Decode stderr independently, without mixing diagnostics into JSON stdout.
    pub fn stderr_text(&self) -> Result<&str, ContractError> {
        std::str::from_utf8(&self.0.stderr).map_err(|source| ContractError::Utf8 {
            stream: "stderr",
            source,
        })
    }
}

#[cfg(test)]
mod process_budget {
    use super::*;

    // These are OS pipe-lifecycle fixtures, not substitute FSL executables.
    fn fixture(script: &str) -> std::process::Child {
        Command::new("/bin/sh")
            .args(["-c", script])
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("private process-budget fixture")
    }

    /// Provenance: QSL-2468
    #[test]
    fn completed_io_retains_status_and_both_streams() -> Result<(), ContractError> {
        let mut child =
            fixture("IFS= read -r line; printf '%s' \"$line\"; printf 'diagnostic' >&2");
        let output = capture_process(
            &mut child,
            b"payload\n",
            Instant::now(),
            Duration::from_secs(2),
        )?;
        assert_eq!(output.0.status.code(), Some(0));
        assert_eq!(output.0.stdout, b"payload");
        assert_eq!(output.0.stderr, b"diagnostic");
        assert!(child.try_wait().expect("reaped child").is_some());
        Ok(())
    }

    /// Provenance: QSL-2468
    #[test]
    fn blocked_stdin_is_cancelled_and_the_child_is_reaped() {
        let mut child = fixture("exec sleep 5");
        let start = Instant::now();
        let result = capture_process(
            &mut child,
            &vec![b'x'; 131_072],
            start,
            Duration::from_millis(150),
        );
        assert!(
            matches!(result, Err(ContractError::Timeout(_))),
            "{result:?}"
        );
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "the complete I/O budget, not only child wait, must be bounded"
        );
        assert!(child.try_wait().expect("reaped child").is_some());
    }

    /// Provenance: QSL-2468
    #[test]
    fn pipes_retained_after_direct_child_exit_still_obey_the_deadline() {
        let mut child = fixture("sleep 5 & exit 0");
        let start = Instant::now();
        let result = capture_process(&mut child, b"", start, Duration::from_millis(150));
        assert!(
            matches!(result, Err(ContractError::Timeout(_))),
            "{result:?}"
        );
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "inherited stdout/stderr must not create an unbounded join"
        );
        assert!(
            child
                .try_wait()
                .expect("reaped parent")
                .expect("parent exited")
                .success(),
            "the direct child exited before its descendant released the pipes"
        );
    }
}
