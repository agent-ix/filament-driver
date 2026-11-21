// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owner examples resolved through the driver's declared FSL dependency.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

/// Locate an example in the exact FSL repository selected by Cargo.lock.
/// The artifact remains in its owner's checkout and is never copied here.
pub fn fsl_example(relative: &str) -> PathBuf {
    static ROOT: LazyLock<PathBuf> = LazyLock::new(|| {
        let output = Command::new(env!("CARGO"))
            .args(["metadata", "--locked", "--format-version", "1"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("resolve the declared FSL dependency artifacts");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("Cargo metadata");
        let package = metadata["packages"]
            .as_array()
            .expect("resolved packages")
            .iter()
            .find(|package| {
                package["name"] == "fsl-replay"
                    && package["source"].as_str().is_some_and(|source| {
                        source.starts_with("git+https://github.com/agent-ix/filament-spec-language")
                    })
            })
            .expect("the declared owner fsl-replay git dependency");
        let manifest = Path::new(package["manifest_path"].as_str().expect("owner manifest"));
        manifest
            .parent()
            .and_then(Path::parent)
            .expect("owner repository root")
            .to_path_buf()
    });
    let artifact = ROOT.join("examples").join(relative);
    assert!(
        artifact.is_file(),
        "missing owner artifact {}",
        artifact.display()
    );
    artifact
}
