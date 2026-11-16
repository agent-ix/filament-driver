// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strict caller-selected provider configuration from FSpec FR-561.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub(crate) const FORMAT: &str = "filament.provider-config/v1";

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

/// Runtime limits remain a caller option and are deliberately not part of FR-561.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
pub(crate) struct RuntimeProviderConfig {
    pub executable: PathBuf,
    pub argv: Vec<String>,
    pub deadline_ms: u64,
    pub limits: RuntimeProviderLimitsConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "provider execution awaits the provider-limit CLI ruling"
)]
pub(crate) struct RuntimeProviderLimitsConfig {
    pub max_frame_bytes: u64,
    pub max_total_bytes: u64,
    pub max_stderr_bytes: u64,
    pub cancellation_grace_ms: u64,
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
    #[error("`{path}` has invalid provider configuration: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("`{path}` has unsupported provider configuration format `{format}`")]
    UnsupportedFormat { path: String, format: String },
    #[error("`{path}` provider entry {index} has a blank path")]
    BlankPath { path: String, index: usize },
}

/// Read and validate only the caller-named FR-561 JSON document.
pub(crate) fn load(path: &Path) -> Result<ProviderConfig, ProviderConfigError> {
    let display_path = path.display().to_string();
    let contents = fs::read_to_string(path).map_err(|source| ProviderConfigError::Read {
        path: display_path.clone(),
        source,
    })?;
    let config: ProviderConfig =
        serde_json::from_str(&contents).map_err(|source| ProviderConfigError::Json {
            path: display_path.clone(),
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
    for provider in &mut config.providers {
        if provider.path.is_relative() {
            provider.path = config_dir.join(&provider.path);
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

    #[test]
    fn provider_paths_are_resolved_from_the_config_file_directory() {
        let directory = tempfile::tempdir().expect("create isolated config directory");
        let path = directory.path().join("providers.json");
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
