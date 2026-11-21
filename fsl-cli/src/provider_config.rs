// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strict caller-selected provider configuration from FSpec FR-561.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use fsl_analyze::provider::ProviderLimits;
use serde::Deserialize;

use crate::options::{self, Scanned};

pub(crate) const FORMAT: &str = "filament.provider-config/v1";

/// The `--provider-config` path and `--provider-limit` set of one
/// provider-routing verb (FSL FR-287, FR-964; FR-006). Every limit the
/// caller does not give keeps FR-287's default.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProviderArguments {
    /// The FSpec FR-561 document the caller named, if any.
    pub config: Option<PathBuf>,
    /// The run's one limit set (FSpec FR-777).
    pub limits: ProviderLimits,
    /// The limit names the caller gave, each at most once.
    named: BTreeSet<String>,
}

/// A verb option the verb cannot accept.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OptionRefusal {
    /// A usage refusal written as the verb's typed `invalid-request`
    /// document, such as `--provider-config` without a path or given twice.
    Usage(String),
    /// A refusal with empty stdout and the message on stderr, exit 20: a
    /// `--provider-limit` FSL FR-287 refuses (FR-006), or a repeated
    /// `--integer-division` (FSL FR-767).
    Quiet(String),
}

impl From<String> for OptionRefusal {
    fn from(message: String) -> Self {
        Self::Usage(message)
    }
}

impl ProviderArguments {
    /// Whether the caller gave either provider option.
    pub(crate) fn given(&self) -> bool {
        self.config.is_some() || !self.named.is_empty()
    }

    /// Whether `max_memory_bytes` appeared, even when its value equals the default.
    pub(crate) fn memory_limit_requested(&self) -> bool {
        self.named.contains("max_memory_bytes")
    }

    /// Read every provider option the scan found: `--provider-config` once
    /// and each `--provider-limit` (FSL FR-287, FR-964; FR-006).
    pub(crate) fn take_all(&mut self, scanned: &Scanned) -> Result<(), OptionRefusal> {
        for (name, value) in &scanned.options {
            let Some(value) = value else { continue };
            match *name {
                options::PROVIDER_CONFIG => {
                    self.config = Some(PathBuf::from(value));
                }
                options::PROVIDER_LIMIT => {
                    let argument = value.to_string_lossy().into_owned();
                    self.set_limit(&argument).map_err(|reason| {
                        OptionRefusal::Quiet(format!(
                            "invalid-request: {name} `{argument}` {reason}"
                        ))
                    })?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Set one FR-287 limit from `name=value`.
    fn set_limit(&mut self, argument: &str) -> Result<(), &'static str> {
        let (name, value) = argument
            .split_once('=')
            .ok_or("is not of the form name=value")?;
        let value = limit_value(value)?;
        let slot = self
            .limits
            .named_mut(name)
            .ok_or("names no FR-287 provider limit")?;
        if !self.named.insert(name.to_owned()) {
            return Err("names a limit already given");
        }
        *slot = value;
        Ok(())
    }
}

/// An FR-287 limit value: a decimal count in `1..=2^53-1` with no sign,
/// leading zero or unit.
fn limit_value(value: &str) -> Result<u64, &'static str> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("has a value that is not a decimal count");
    }
    if value.starts_with('0') {
        return Err("has a value that is zero or has a leading zero");
    }
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value <= ProviderLimits::MAX_VALUE)
        .ok_or("has a value above 9007199254740991")
}

/// The complete provider document named by `--provider-config`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProviderConfig {
    pub format: String,
    pub providers: Vec<ProviderEntry>,
}

/// One configured executable and its verbatim argument vector.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProviderEntry {
    pub path: PathBuf,
    #[serde(default)]
    pub arguments: Vec<String>,
}

/// Why an explicitly selected provider configuration could not be loaded.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ProviderConfigError {
    #[error("could not read `{path}` as UTF-8 JSON: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("`{path}` has invalid provider configuration at `{member}`: {source}")]
    Json {
        path: String,
        member: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("`{path}` has unsupported provider configuration format `{format}`")]
    UnsupportedFormat { path: String, format: String },
    #[error("`{path}` provider entry {index} has a blank path at `providers[{index}].path`")]
    BlankPath { path: String, index: usize },
    #[error("`{path}` provider entry {index} path `{executable}` does not exist")]
    MissingExecutable {
        path: String,
        index: usize,
        executable: String,
    },
}

/// Read and validate only the caller-named FR-561 JSON document.
pub(crate) fn load(path: &Path) -> Result<ProviderConfig, ProviderConfigError> {
    let display_path = path.display().to_string();
    let contents = fs::read_to_string(path).map_err(|source| ProviderConfigError::Read {
        path: display_path.clone(),
        source,
    })?;
    let mut deserializer = serde_json::Deserializer::from_str(&contents);
    let config: ProviderConfig =
        serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
            ProviderConfigError::Json {
                path: display_path.clone(),
                member: error.path().to_string(),
                source: error.into_inner(),
            }
        })?;
    deserializer
        .end()
        .map_err(|source| ProviderConfigError::Json {
            path: display_path.clone(),
            member: "<document>".to_owned(),
            source,
        })?;
    if config.format != FORMAT {
        return Err(ProviderConfigError::UnsupportedFormat {
            path: display_path,
            format: config.format,
        });
    }
    for (index, provider) in config.providers.iter().enumerate() {
        if provider.path.to_string_lossy().trim().is_empty() {
            return Err(ProviderConfigError::BlankPath {
                path: display_path,
                index,
            });
        }
    }
    let config_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut config = config;
    for (index, provider) in config.providers.iter_mut().enumerate() {
        if provider.path.is_relative() {
            provider.path = config_dir.join(&provider.path);
        }
        // FSpec FR-561-AC-2: a path that does not exist refuses naming it,
        // before any plugin starts.
        if !provider.path.exists() {
            return Err(ProviderConfigError::MissingExecutable {
                path: display_path,
                index,
                executable: provider.path.display().to_string(),
            });
        }
    }
    Ok(config)
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    reason = "invalid fixture data should fail the test"
)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    const VALID: &str = r#"{
        "format":"filament.provider-config/v1",
        "providers":[{"path":"/tmp/provider", "arguments":["--mode", "test"]}]
    }"#;

    #[test]
    fn parses_optional_arguments_and_empty_provider_list() {
        let config: ProviderConfig = serde_json::from_str(VALID).expect("valid config");
        assert_eq!(config.format, FORMAT);
        assert_eq!(config.providers[0].path, PathBuf::from("/tmp/provider"));
        assert_eq!(config.providers[0].arguments, ["--mode", "test"]);
        let no_arguments: ProviderConfig = serde_json::from_str(
            r#"{"format":"filament.provider-config/v1","providers":[{"path":"p"}]}"#,
        )
        .expect("arguments are optional");
        assert!(no_arguments.providers[0].arguments.is_empty());
        let empty: ProviderConfig =
            serde_json::from_str(r#"{"format":"filament.provider-config/v1","providers":[]}"#)
                .expect("empty providers are valid");
        assert!(empty.providers.is_empty());
    }

    #[test]
    fn rejects_missing_members_unknown_members_bad_paths_and_wrong_arguments() {
        for malformed in [
            r#"{"providers":[]}"#,
            r#"{"format":"filament.provider-config/v1"}"#,
            r#"{"format":"filament.provider-config/v1","providers":[],"extra":1}"#,
            r#"{"format":"filament.provider-config/v1","providers":[{}]}"#,
            r#"{"format":"filament.provider-config/v1","providers":[{"path":"p","arguments":[1]}]}"#,
            r#"{"format":"filament.provider-config/v1","providers":[{"path":"p","extra":true}]}"#,
            "{ malformed",
        ] {
            assert!(
                serde_json::from_str::<ProviderConfig>(malformed).is_err(),
                "{malformed}"
            );
        }
    }

    #[test]
    fn named_file_loader_reports_missing_malformed_and_unsupported_format() {
        let directory = tempfile::tempdir().expect("create isolated config directory");
        let missing = load(&directory.path().join("missing.json"))
            .expect_err("missing named file is refused");
        assert!(missing.to_string().contains("could not read"));

        let path = directory.path().join("config.json");
        fs::write(&path, "{ malformed").expect("write malformed JSON");
        assert!(load(&path)
            .expect_err("malformed data is refused")
            .to_string()
            .contains("invalid"));

        fs::write(&path, r#"{"format":"future","providers":[]}"#)
            .expect("write a different format");
        assert!(load(&path)
            .expect_err("different format is refused")
            .to_string()
            .contains("unsupported"));
    }

    fn take_limits(values: &[&str]) -> Result<ProviderArguments, OptionRefusal> {
        let mut arguments = ProviderArguments::default();
        let argv = values
            .iter()
            .flat_map(|value| {
                [
                    OsString::from(options::PROVIDER_LIMIT),
                    OsString::from(value),
                ]
            })
            .collect::<Vec<_>>();
        let entry = options::entry("analyze").expect("analyze has an option table entry");
        let Ok(options::Scan::Options(scanned)) = options::scan(entry, &argv) else {
            panic!("provider limits scan");
        };
        arguments.take_all(&scanned)?;
        Ok(arguments)
    }

    /// Trace: FR-006-AC-8, FR-006-AC-28; FSL FR-287's defaults and names.
    #[test]
    fn provider_limits_default_to_fr287_and_set_each_named_limit() {
        let defaults = take_limits(&[]).expect("no limit given");
        assert!(!defaults.given());
        assert_eq!(defaults.limits.max_frame_bytes, 67_108_864);
        assert_eq!(defaults.limits.max_total_bytes, 1_073_741_824);
        assert_eq!(defaults.limits.deadline_ms, 600_000);
        assert_eq!(defaults.limits.grace_ms, 5_000);
        assert_eq!(defaults.limits.max_stderr_bytes, 1_048_576);
        assert_eq!(defaults.limits.max_memory_bytes, 4_294_967_296);
        assert_eq!(defaults.limits.max_cpu_ms, 600_000);
        let set = take_limits(&["deadline_ms=250", "max_frame_bytes=1048576"]).expect("valid");
        assert!(set.given());
        assert_eq!(set.limits.deadline_ms, 250);
        assert_eq!(set.limits.max_frame_bytes, 1_048_576);
        assert_eq!(set.limits.grace_ms, 5_000);
    }

    /// Explicitly requested memory remains distinct from a default.
    #[ix_trace_rs::trace("FR-013-AC-18")]
    #[test]
    fn provider_memory_limit_presence_survives_an_explicit_default_value() {
        let defaults = take_limits(&[]).expect("default limits");
        assert!(!defaults.memory_limit_requested());
        let unrelated = take_limits(&["max_cpu_ms=1"]).expect("CPU limit");
        assert!(!unrelated.memory_limit_requested());
        for memory in ["max_memory_bytes=4294967296", "max_memory_bytes=67108864"] {
            let supplied = take_limits(&[memory]).expect("explicit memory limit");
            assert!(supplied.memory_limit_requested(), "{memory}");
        }
    }

    /// Trace: FR-006-AC-29; each malformed limit refuses naming the option and argument.
    #[test]
    fn provider_limits_refuse_malformed_unknown_and_repeated_arguments() {
        for refused in [
            &["max_frames=4"][..],
            &["deadline_ms"],
            &["deadline_ms=0"],
            &["deadline_ms=10s"],
            &["deadline_ms=010"],
            &["deadline_ms=-5"],
            &["deadline_ms="],
            &["max_total_bytes=9007199254740992"],
            &["deadline_ms=5", "deadline_ms=5"],
        ] {
            let error = take_limits(refused).expect_err("the limit is refused");
            assert!(
                matches!(error, OptionRefusal::Quiet(_)),
                "{refused:?} is a limit refusal: {error:?}"
            );
            let (OptionRefusal::Quiet(message) | OptionRefusal::Usage(message)) = error;
            assert!(message.contains("--provider-limit"), "{message}");
            assert!(
                message.contains(refused[refused.len() - 1]),
                "{message} names the argument"
            );
        }
        let accepted = take_limits(&["max_total_bytes=9007199254740991"]).expect("the maximum");
        assert_eq!(accepted.limits.max_total_bytes, ProviderLimits::MAX_VALUE);
    }

    #[test]
    fn provider_paths_are_resolved_from_the_config_file_directory() {
        let directory = tempfile::tempdir().expect("create isolated config directory");
        let path = directory.path().join("providers.json");
        fs::create_dir(directory.path().join("bin")).expect("create plugin directory");
        fs::write(directory.path().join("bin/provider"), b"").expect("write plugin");
        fs::write(
            &path,
            r#"{"format":"filament.provider-config/v1","providers":[{"path":"bin/provider"}]}"#,
        )
        .expect("write config");
        let config = load(&path).expect("load config");
        assert_eq!(
            config.providers[0].path,
            directory.path().join("bin/provider")
        );
    }
}
