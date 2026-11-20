// SPDX-License-Identifier: AGPL-3.0-or-later
//! Locate the separate workspace test-provider binary beside Cargo's test targets.

use std::path::PathBuf;

/// The `fsl-test-provider` executable built by the workspace's provider package.
#[must_use]
pub fn executable() -> PathBuf {
    let test_binary = std::env::current_exe().expect("the integration-test executable path");
    let target_profile = test_binary
        .parent()
        .and_then(std::path::Path::parent)
        .expect("the Cargo target profile directory");
    target_profile.join(format!("fsl-test-provider{}", std::env::consts::EXE_SUFFIX))
}
