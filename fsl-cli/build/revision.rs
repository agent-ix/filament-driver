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

/// What reading a local checkout produced.
#[derive(Debug, PartialEq, Eq)]
pub struct LocalRevision {
    /// The text `fsl --version` reports after `FSL revision: `.
    pub revision: String,
    /// Every path the resolver read or would read next; a build script emits
    /// `cargo:rerun-if-changed` for each so a moved checkout is never reported
    /// with a stale commit.
    pub watched: Vec<PathBuf>,
}

/// The outcome of resolving a checkout's `HEAD`.
enum Head {
    /// No `.git` entry: not a git checkout.
    NotGit,
    /// A git checkout whose `HEAD` cannot be read as a 40-hex commit.
    Unknown,
    Sha(String),
}

/// The text `fsl --version` reports for a local path build of the checkout at
/// `root` (FR-002-AC-11): `<sha> (local path)`; `local path` when `root` has no
/// `.git`; `unknown (local path)` when `.git`, `HEAD` or the ref it names is
/// unreadable or is not a 40-hex commit (a garbage loose ref is unknown and is
/// never replaced by a `packed-refs` value; a 64-hex SHA-256 commit is unknown).
pub fn local_path_revision(root: &Path) -> LocalRevision {
    let mut watched = Vec::new();
    let head = resolve_head(root, &mut watched);
    let revision = match head {
        Head::Sha(sha) => format!("{sha} (local path)"),
        Head::NotGit => "local path".to_owned(),
        Head::Unknown => "unknown (local path)".to_owned(),
    };
    LocalRevision { revision, watched }
}

fn resolve_head(root: &Path, watched: &mut Vec<PathBuf>) -> Head {
    let dot_git = root.join(".git");
    watched.push(dot_git.clone());
    let git_dir = if dot_git.is_dir() {
        dot_git
    } else if dot_git.exists() {
        // A worktree or submodule: `.git` is a file naming the real directory.
        let target = fs::read_to_string(&dot_git).ok().and_then(|text| {
            text.trim()
                .strip_prefix("gitdir:")
                .map(|t| t.trim().to_owned())
        });
        match target {
            Some(target) if !target.is_empty() => root.join(target),
            _ => return Head::Unknown,
        }
    } else {
        return Head::NotGit;
    };
    let head_path = git_dir.join("HEAD");
    let commondir_path = git_dir.join("commondir");
    watched.extend([
        head_path.clone(),
        commondir_path.clone(),
        git_dir.join("packed-refs"),
    ]);
    let mut dirs = vec![git_dir.clone()];
    if let Ok(text) = fs::read_to_string(&commondir_path) {
        let common = git_dir.join(text.trim());
        watched.push(common.join("packed-refs"));
        dirs.push(common);
    }
    let Ok(head) = fs::read_to_string(&head_path) else {
        return Head::Unknown;
    };
    let head = head.trim();
    let Some(name) = head.strip_prefix("ref:").map(str::trim) else {
        return if is_sha(head) {
            Head::Sha(head.to_owned())
        } else {
            Head::Unknown
        };
    };
    for dir in &dirs {
        watched.push(dir.join(name));
    }
    for dir in &dirs {
        if let Ok(text) = fs::read_to_string(dir.join(name)) {
            let value = text.trim();
            return if is_sha(value) {
                Head::Sha(value.to_owned())
            } else {
                Head::Unknown
            };
        }
    }
    for dir in &dirs {
        if let Ok(packed) = fs::read_to_string(dir.join("packed-refs")) {
            let found = packed.lines().find_map(|line| {
                let (sha, reference) = line.split_once(' ')?;
                (reference == name && is_sha(sha)).then(|| sha.to_owned())
            });
            if let Some(sha) = found {
                return Head::Sha(sha);
            }
        }
    }
    Head::Unknown
}
