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

use revision::{local_path_revision, lock_entry, pinned_commit, LockEntry};

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

fn revision(root: &std::path::Path) -> String {
    local_path_revision(root).revision
}

fn checkout() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join(".git/refs/heads")).expect("refs");
    dir
}

/// Trace: FR-002-AC-11, a local path build reports the checkout HEAD and says so.
#[trace("FR-002-AC-11")]
#[test]
fn a_local_path_build_reports_the_checkout_head_commit() {
    let dir = checkout();
    let git = dir.path().join(".git");
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    // A ref nobody holds is unknown.
    assert_eq!(revision(dir.path()), "unknown (local path)");
    fs::write(
        git.join("packed-refs"),
        format!("# pack-refs\n{OTHER} refs/heads/main\n"),
    )
    .expect("packed");
    assert_eq!(revision(dir.path()), format!("{OTHER} (local path)"));
    fs::write(git.join("refs/heads/main"), format!("{SHA}\n")).expect("loose");
    assert_eq!(revision(dir.path()), format!("{SHA} (local path)"));
    fs::write(git.join("HEAD"), format!("{OTHER}\n")).expect("detached");
    assert_eq!(revision(dir.path()), format!("{OTHER} (local path)"));
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
    let local = local_path_revision(&work);
    assert_eq!(local.revision, format!("{SHA} (local path)"));
    for expected in [
        work.join(".git"),
        admin.join("HEAD"),
        admin.join("commondir"),
        admin.join("packed-refs"),
        admin.join("../..").join("packed-refs"),
        admin.join("../..").join("refs/heads/topic"),
    ] {
        assert!(
            local.watched.contains(&expected),
            "{expected:?} is not watched"
        );
    }
}

/// Trace: FR-002-AC-11, a relative `gitdir:` resolves against the checkout root.
#[trace("FR-002-AC-11")]
#[test]
fn a_relative_gitdir_file_resolves_against_the_checkout() {
    let dir = tempfile::tempdir().expect("tempdir");
    let real = dir.path().join("real-git");
    fs::create_dir_all(&real).expect("gitdir");
    fs::write(real.join("HEAD"), format!("{SHA}\n")).expect("HEAD");
    fs::write(dir.path().join(".git"), "gitdir: real-git\n").expect(".git");
    assert_eq!(revision(dir.path()), format!("{SHA} (local path)"));
}

/// Trace: FR-002-AC-11, a directory without `.git` reports `local path` only.
#[trace("FR-002-AC-11")]
#[test]
fn a_directory_that_is_not_a_git_checkout_reports_local_path_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let local = local_path_revision(dir.path());
    assert_eq!(local.revision, "local path");
    // The `.git` entry is watched so a later `git init` is noticed.
    assert_eq!(local.watched, vec![dir.path().join(".git")]);
}

/// Trace: FR-002-AC-11, unreadable git metadata reports `unknown (local path)`.
#[trace("FR-002-AC-11")]
#[test]
fn unreadable_git_metadata_reports_unknown() {
    let sha256 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    // Garbage HEAD.
    let dir = checkout();
    fs::write(dir.path().join(".git/HEAD"), "not a head\n").expect("HEAD");
    assert_eq!(revision(dir.path()), "unknown (local path)");
    // Missing HEAD.
    let dir = checkout();
    assert_eq!(revision(dir.path()), "unknown (local path)");
    // Garbage `.git` file.
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join(".git"), "garbage\n").expect(".git");
    assert_eq!(revision(dir.path()), "unknown (local path)");
    // A garbage loose ref is unknown even when packed-refs holds a value.
    let dir = checkout();
    let git = dir.path().join(".git");
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    fs::write(git.join("refs/heads/main"), "garbage\n").expect("loose");
    fs::write(
        git.join("packed-refs"),
        format!("{OTHER} refs/heads/main\n"),
    )
    .expect("packed");
    assert_eq!(revision(dir.path()), "unknown (local path)");
    // A 64-hex SHA-256 commit is not the 40-character revision FR-002-AC-11 prints.
    let dir = checkout();
    fs::write(dir.path().join(".git/HEAD"), format!("{sha256}\n")).expect("HEAD");
    assert_eq!(revision(dir.path()), "unknown (local path)");
}

/// Trace: FR-002-AC-11, the watched list covers every file the resolver reads.
#[trace("FR-002-AC-11")]
#[test]
fn the_watched_paths_cover_head_the_ref_and_packed_refs() {
    let dir = checkout();
    let git = dir.path().join(".git");
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    // Not yet resolvable, the candidate paths are still watched.
    let local = local_path_revision(dir.path());
    assert_eq!(local.revision, "unknown (local path)");
    for expected in [
        git.clone(),
        git.join("HEAD"),
        git.join("commondir"),
        git.join("packed-refs"),
        git.join("refs/heads/main"),
    ] {
        assert!(
            local.watched.contains(&expected),
            "{expected:?} is not watched"
        );
    }
}
