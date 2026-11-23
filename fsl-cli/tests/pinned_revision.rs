// SPDX-License-Identifier: AGPL-3.0-or-later
//! The FSL revision `fsl --version` reports, for git-pinned and local path builds.
#![allow(
    clippy::expect_used,
    reason = "test setup failures should fail the test"
)]

use std::fs;

use ix_trace_rs::trace;

#[path = "../build/revision.rs"]
mod revision;

use revision::{head_commit, local_path_revision, lock_entry, pinned_commit, LockEntry};

const SHA: &str = "86acfdeedabe40674067342aaf2f639d48fc15f3";
const OTHER: &str = "0123456789abcdef0123456789abcdef01234567";

fn git_source(commit: &str) -> String {
    format!("git+https://github.com/agent-ix/filament-spec-language?rev={SHA}#{commit}")
}

/// Trace: FR-002-AC-11, a git-pinned lock entry yields the full commit.
#[trace("FR-002-AC-11")]
#[test]
fn a_git_source_lock_entry_yields_the_pinned_commit() {
    let lock = format!(
        "[[package]]\nname = \"other\"\nsource = \"x\"\n\n[[package]]\nname = \"fsl-analyze\"\nversion = \"0.1.0\"\nsource = \"{}\"\n",
        git_source(SHA)
    );
    let LockEntry::Source(source) = lock_entry(&lock) else {
        panic!("expected a source");
    };
    assert_eq!(pinned_commit(&source).as_deref(), Ok(SHA));
    assert!(pinned_commit(&git_source(OTHER)).is_err());
    assert!(pinned_commit("registry+https://example.test").is_err());
}

/// Trace: FR-002-AC-11, a path or patched entry has no source and is not an error.
#[trace("FR-002-AC-11")]
#[test]
fn a_lock_entry_without_a_source_is_a_path_dependency() {
    let lock = "[[package]]\nname = \"fsl-analyze\"\nversion = \"0.1.0\"\ndependencies = []\n\n[[package]]\nname = \"z\"\nsource = \"s\"\n";
    assert_eq!(lock_entry(lock), LockEntry::NoSource);
    assert_eq!(
        lock_entry("[[package]]\nname = \"z\"\n"),
        LockEntry::Missing
    );
}

/// Trace: FR-002-AC-11, a local path build reports the checkout HEAD and says so.
#[trace("FR-002-AC-11")]
#[test]
fn a_local_path_build_reports_the_checkout_head_commit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let git = dir.path().join(".git");
    fs::create_dir_all(git.join("refs/heads")).expect("refs");
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    // Unresolvable ref: still a checkout, with no commit to report.
    assert_eq!(head_commit(dir.path()), None);
    assert_eq!(local_path_revision(dir.path()), "local path");
    // Packed ref.
    fs::write(
        git.join("packed-refs"),
        format!("# pack-refs\n{OTHER} refs/heads/main\n"),
    )
    .expect("packed");
    assert_eq!(
        local_path_revision(dir.path()),
        format!("{OTHER} (local path)")
    );
    // Loose ref wins.
    fs::write(git.join("refs/heads/main"), format!("{SHA}\n")).expect("loose");
    assert_eq!(
        local_path_revision(dir.path()),
        format!("{SHA} (local path)")
    );
    // Detached HEAD.
    fs::write(git.join("HEAD"), format!("{OTHER}\n")).expect("detached");
    assert_eq!(head_commit(dir.path()).as_deref(), Some(OTHER));
}

/// Trace: FR-002-AC-11, a linked worktree resolves refs through its common directory.
#[trace("FR-002-AC-11")]
#[test]
fn a_linked_worktree_checkout_resolves_through_the_common_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let common = dir.path().join("main/.git");
    let admin = common.join("worktrees/w");
    fs::create_dir_all(common.join("refs/heads")).expect("common");
    fs::create_dir_all(&admin).expect("admin");
    fs::write(common.join("refs/heads/topic"), format!("{SHA}\n")).expect("ref");
    fs::write(admin.join("HEAD"), "ref: refs/heads/topic\n").expect("HEAD");
    fs::write(admin.join("commondir"), "../..\n").expect("commondir");
    let work = dir.path().join("w");
    fs::create_dir_all(&work).expect("work");
    fs::write(work.join(".git"), format!("gitdir: {}\n", admin.display())).expect(".git");
    assert_eq!(local_path_revision(&work), format!("{SHA} (local path)"));
}

/// Trace: FR-002-AC-11, a directory that is not a git checkout reports `local path` only.
#[trace("FR-002-AC-11")]
#[test]
fn a_directory_that_is_not_a_git_checkout_reports_local_path_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(local_path_revision(dir.path()), "local path");
}
