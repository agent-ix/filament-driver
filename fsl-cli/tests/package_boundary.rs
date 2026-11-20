// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-014 (TC-028): keep the CLI executable and its frontend-owned files in
//! the separate `fsl-cli` workspace package.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "metadata and layout mismatches are the test failure report"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ix_trace_rs::trace;
use regex::Regex;
use serde_json::Value;

fn strip_rust_comments(source: &str) -> String {
    Regex::new(r"(?s)/\*.*?\*/|//[^\n]*")
        .expect("comment expression compiles")
        .replace_all(source, "")
        .into_owned()
}

fn has_cli_export(source: &str) -> bool {
    let source = strip_rust_comments(source);
    Regex::new(r"(?m)\bpub\s+(?:mod\s+cli\b(?:\s*\{|\s*;)|use\b[^;]*\bas\s+cli\b)")
        .expect("CLI export expression compiles")
        .is_match(&source)
}

fn has_parser_definition(source: &str) -> bool {
    let source = strip_rust_comments(source);
    Regex::new(r"(?m)\b(?:pub(?:\([^)]*\))?\s+)?fn\s+parse_invocation\b")
        .expect("parser expression compiles")
        .is_match(&source)
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("fsl-cli has a workspace root")
        .to_path_buf()
}

fn metadata() -> Value {
    let root = workspace_root();
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--offline",
            "--no-deps",
        ])
        .current_dir(&root)
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata is JSON")
}

fn direct_normal_dependencies(package: &Value) -> BTreeSet<&str> {
    package["dependencies"]
        .as_array()
        .expect("package dependencies")
        .iter()
        .filter(|dependency| dependency["kind"].is_null())
        .map(|dependency| dependency["name"].as_str().expect("dependency name"))
        .collect()
}

/// Trace: FR-014-AC-1, FR-014-AC-2
#[test]
fn cargo_target_and_dependencies_belong_to_fsl_cli() {
    let metadata = metadata();
    let packages = metadata["packages"].as_array().expect("workspace packages");
    let cli_packages: Vec<&Value> = packages
        .iter()
        .filter(|package| package["name"] == "fsl-cli")
        .collect();
    assert_eq!(cli_packages.len(), 1, "one fsl-cli workspace package");
    let cli = cli_packages[0];
    let default_features = cli["features"]["default"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        !default_features
            .iter()
            .any(|feature| feature == "tc018-test-adapter"),
        "TC-018's test-only feature must not be enabled by default"
    );

    let fsl_targets: Vec<(&Value, &Value)> = packages
        .iter()
        .flat_map(|package| {
            package["targets"]
                .as_array()
                .expect("package targets")
                .iter()
                .filter(move |target| {
                    target["name"] == "fsl"
                        && target["kind"]
                            .as_array()
                            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
                })
                .map(move |target| (package, target))
        })
        .collect();
    assert_eq!(fsl_targets.len(), 1, "one Cargo binary named fsl");
    assert_eq!(fsl_targets[0].0["name"], "fsl-cli");

    let dependencies = direct_normal_dependencies(cli);
    assert!(dependencies.contains("filament-driver"));
    assert!(dependencies.contains("filament-cache"));
    assert!(dependencies.contains("filament-plugin-host"));
    assert!(dependencies.contains("ix-cli-kit"));
    assert!(dependencies.contains("filament-semantic-value"));
}

/// Trace: FR-014-AC-4, FR-014-AC-5; test sources and non-executable fixtures
/// are owned by fsl-cli, and the library exports no CLI module.
#[test]
#[trace("TC-028", "FR-014-AC-4", "FR-014-AC-5")]
fn parser_and_cli_tests_live_in_the_cli_package() {
    let root = workspace_root();
    let cli_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parser_path = cli_root.join("src/cli.rs");
    let parser = fs::read_to_string(&parser_path).expect("read CLI parser source");
    assert!(has_parser_definition(&parser));
    assert!(parser.contains("enum Verb"));
    assert!(parser.contains("enum OutputFormat"));

    let main_path = cli_root.join("src/main.rs");
    let main = fs::read_to_string(&main_path).expect("read CLI entry point");
    assert!(main.contains("cli::entrypoint()"));
    assert!(!main.contains("fn parse_invocation("));

    let driver_lib = fs::read_to_string(root.join("src/lib.rs")).expect("read driver library");
    assert!(!has_cli_export(&driver_lib));

    for relative in ["tests/cli_contract.rs", "tests/binary_process.rs"] {
        assert!(
            cli_root.join(relative).is_file(),
            "missing fsl-cli/{relative}"
        );
        assert!(
            !root.join(relative).exists(),
            "CLI test remains in the filament-driver package: {relative}"
        );
    }
    let fixture = "tests/fixtures/ir608-run.fsl";
    assert!(
        cli_root.join(fixture).is_file(),
        "missing fsl-cli/{fixture}"
    );
    assert!(
        !root.join(fixture).exists(),
        "CLI fixture remains in filament-driver"
    );
}

/// Trace: FR-014-AC-7; the installable CLI target excludes its test provider.
#[test]
#[trace("TC-028", "FR-014-AC-7")]
fn source_install_package_has_only_fsl_and_separate_test_provider() {
    let metadata = metadata();
    let packages = metadata["packages"].as_array().expect("workspace packages");
    let cli_packages: Vec<&Value> = packages
        .iter()
        .filter(|package| package["name"] == "fsl-cli")
        .collect();
    let provider_packages: Vec<&Value> = packages
        .iter()
        .filter(|package| package["name"] == "fsl-test-provider")
        .collect();
    assert_eq!(cli_packages.len(), 1, "one fsl-cli package");
    assert_eq!(provider_packages.len(), 1, "one provider fixture package");

    let binary_names = |package: &Value| {
        package["targets"]
            .as_array()
            .expect("package targets")
            .iter()
            .filter(|target| {
                target["kind"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
            })
            .map(|target| {
                target["name"]
                    .as_str()
                    .expect("binary target name")
                    .to_owned()
            })
            .collect::<Vec<_>>()
    };

    let cli = cli_packages[0];
    assert_eq!(
        binary_names(cli),
        ["fsl".to_owned()],
        "fsl-cli installs only fsl"
    );
    let provider = provider_packages[0];
    assert_eq!(provider["publish"], serde_json::json!([]));
    assert_eq!(binary_names(provider), ["fsl-test-provider".to_owned()]);
    assert_ne!(cli["id"], provider["id"], "the fixture has its own package");
    assert!(
        metadata["workspace_members"]
            .as_array()
            .expect("workspace member ids")
            .iter()
            .any(|member| member == &provider["id"]),
        "the provider fixture is a workspace member"
    );
}

/// Comment examples do not count as definitions or public exports.
#[test]
fn source_matchers_ignore_comments_and_detect_cli_export_forms() {
    assert!(!has_parser_definition("// fn parse_invocation() {}\n"));
    assert!(has_parser_definition("fn parse_invocation() {}"));

    for declaration in [
        "pub mod cli;",
        "pub mod cli {}",
        "pub use crate::inner as cli;",
    ] {
        assert!(has_cli_export(declaration), "missed {declaration}");
    }
    assert!(!has_cli_export(
        "// pub mod cli;\n/* pub mod cli {} */\n/// pub use x as cli;"
    ));
}
