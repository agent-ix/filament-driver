// SPDX-License-Identifier: AGPL-3.0-or-later
//! Resolves the FSL revision that `fsl --version` reports (FR-002-AC-11).
//!
//! Shared by `build.rs` and `tests/pinned_revision.rs` through `#[path]`, so
//! the parsing is plain functions a test can call. Nothing here panics or
//! spawns a process.

use std::{
    fs,
    path::{Path, PathBuf},
};

/// The `Cargo.lock` source of the `fsl-analyze` package.
#[derive(Debug, PartialEq, Eq)]
pub enum LockEntry {
    /// The package is absent from the lock.
    Missing,
    /// The package has no `source` line: a path dependency or `[patch]` path.
    NoSource,
    /// The package resolves from this `source` string.
    Source(String),
}

/// The lock entry for `fsl-analyze` in the text of a `Cargo.lock`.
pub fn lock_entry(lock: &str) -> LockEntry {
    let mut in_package = false;
    let mut found = false;
    let mut source = None;
    for line in lock.lines().map(str::trim) {
        if line == "[[package]]" {
            if found {
                break;
            }
            in_package = false;
        } else if line == "name = \"fsl-analyze\"" {
            in_package = true;
            found = true;
        } else if in_package {
            if let Some(value) = line
                .strip_prefix("source = \"")
                .and_then(|value| value.strip_suffix('"'))
            {
                source = Some(value.to_owned());
            }
        }
    }
    match (found, source) {
        (false, _) => LockEntry::Missing,
        (true, None) => LockEntry::NoSource,
        (true, Some(source)) => LockEntry::Source(source),
    }
}

/// The 40-character commit a git `source` pins, checked against its `rev`
/// query, or why it is not a pinned FSL git source.
pub fn pinned_commit(source: &str) -> Result<String, String> {
    let (repository, commit) = source
        .rsplit_once('#')
        .ok_or("fsl-analyze git source has no resolved commit")?;
    let pinned = repository
        .strip_prefix("git+https://github.com/agent-ix/filament-spec-language?rev=")
        .ok_or("fsl-analyze must resolve from the pinned FSL git repository")?;
    if pinned != commit {
        return Err("fsl-analyze Cargo.lock revision query and resolved commit differ".into());
    }
    if !is_sha(commit) {
        return Err(format!(
            "fsl-analyze source has an invalid resolved commit: {commit}"
        ));
    }
    Ok(commit.to_owned())
}

fn is_sha(text: &str) -> bool {
    text.len() == 40 && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// The `.git` directory holding `HEAD` of the checkout at `root`; follows a
/// worktree or submodule `.git` file to its `gitdir`.
fn git_dir(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let text = fs::read_to_string(&dot_git).ok()?;
    let target = text.trim().strip_prefix("gitdir:")?.trim();
    Some(root.join(target))
}

/// The ref `name` as a commit, from a loose ref file or `packed-refs`, in
/// `git_dir` or its common directory.
fn resolve_ref(git_dir: &Path, name: &str) -> Option<String> {
    let common = fs::read_to_string(git_dir.join("commondir"))
        .ok()
        .map(|text| git_dir.join(text.trim()));
    for dir in std::iter::once(git_dir).chain(common.as_deref()) {
        if let Ok(text) = fs::read_to_string(dir.join(name)) {
            let value = text.trim();
            if is_sha(value) {
                return Some(value.to_owned());
            }
        }
        if let Ok(packed) = fs::read_to_string(dir.join("packed-refs")) {
            let found = packed.lines().find_map(|line| {
                let (sha, reference) = line.split_once(' ')?;
                (reference == name && is_sha(sha)).then(|| sha.to_owned())
            });
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

/// The commit `HEAD` names in the git checkout at `root`, read from its `.git`
/// directory, or `None` when `root` is not a git checkout.
pub fn head_commit(root: &Path) -> Option<String> {
    let git_dir = git_dir(root)?;
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    match head.strip_prefix("ref:") {
        Some(name) => resolve_ref(&git_dir, name.trim()),
        None => is_sha(head).then(|| head.to_owned()),
    }
}

/// The text `fsl --version` reports for a local path build of the checkout at
/// `root`: `<sha> (local path)`, or `local path` when it is not a git checkout.
pub fn local_path_revision(root: &Path) -> String {
    match head_commit(root) {
        Some(sha) => format!("{sha} (local path)"),
        None => "local path".to_owned(),
    }
}
