// SPDX-License-Identifier: AGPL-3.0-or-later
//! Keep the fixture executable available when the workspace test suite runs.

#[test]
fn provider_binary_is_built_for_workspace_tests() {
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_fsl-test-provider"));
    assert!(binary.is_file(), "{}", binary.display());
}
