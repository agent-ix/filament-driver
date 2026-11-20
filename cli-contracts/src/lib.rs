// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real-process FSL CLI contracts against one explicitly configured prebuilt binary.

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;

use serde_json::Value;
use tempfile::TempDir;
use thiserror::Error;
use wait_timeout::ChildExt;

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
    /// A capture worker panicked.
    Worker,
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
            Self::Worker => "worker",
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
            Self::Worker,
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
    /// A subprocess I/O worker panicked.
    #[error("{0} worker panicked")]
    Worker(&'static str),
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
            Self::Worker(_) => ContractCode::Worker,
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
        let mut child = Command::new(&self.binary)
            .args(arguments)
            .current_dir(self.directory())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| io_error("spawn FSL", e))?;
        let mut stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let deadline = Duration::from_secs(30);
        thread::scope(|scope| {
            let writer = scope.spawn(move || stdin.write_all(input));
            let stdout = scope.spawn(move || capture(stdout, "stdout"));
            let stderr = scope.spawn(move || capture(stderr, "stderr"));
            let status = match child.wait_timeout(deadline) {
                Ok(Some(status)) => Ok(status),
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    Err(ContractError::Timeout(deadline))
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    Err(io_error("wait for FSL", error))
                }
            };
            let input = writer.join().map_err(|_| ContractError::Worker("stdin"))?;
            let stdout = stdout.join().map_err(|_| ContractError::Worker("stdout"))?;
            let stderr = stderr.join().map_err(|_| ContractError::Worker("stderr"))?;
            let status = status?;
            if let Err(error) = input {
                if error.kind() != io::ErrorKind::BrokenPipe {
                    return Err(io_error("write FSL stdin", error));
                }
            }
            Ok(Captured(Output {
                status,
                stdout: stdout?,
                stderr: stderr?,
            }))
        })
    }
}

fn capture(mut stream: impl Read, name: &'static str) -> Result<Vec<u8>, ContractError> {
    const LIMIT: u64 = 1_048_576;
    let mut bytes = Vec::new();
    (&mut stream)
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| io_error("read FSL output", e))?;
    if bytes.len() as u64 > LIMIT {
        // Keep draining so a noisy child cannot deadlock on a full pipe before
        // its deadline. Its bytes are never retained beyond the capture bound.
        io::copy(&mut stream, &mut io::sink()).map_err(|e| io_error("drain FSL output", e))?;
        return Err(ContractError::OutputLimit(name));
    }
    Ok(bytes)
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
