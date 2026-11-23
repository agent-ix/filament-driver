//! Reports the FSL revision of this build as `FSL_PINNED_REVISION` (FR-002-AC-11).

use std::{env, fs, path::PathBuf};

#[path = "build/revision.rs"]
mod revision;

use revision::{local_path_revision, lock_entry, pinned_commit, LockEntry};

/// The variable naming the FSL checkout a local path build patches in; the
/// examples gate sets it (driver FR-021).
const LANGUAGE_VAR: &str = "FSL_LANGUAGE_DIR";
const UNKNOWN: &str = "unknown (local path)";

fn main() {
    let lock_path = env::var_os("CARGO_MANIFEST_DIR")
        .map(|dir| PathBuf::from(dir).join("../Cargo.lock"))
        .unwrap_or_else(|| PathBuf::from("../Cargo.lock"));
    println!("cargo:rerun-if-changed={}", lock_path.display());
    println!("cargo:rerun-if-env-changed={LANGUAGE_VAR}");

    let lock = fs::read_to_string(&lock_path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", lock_path.display()));
    let revision = match lock_entry(&lock) {
        // A git-pinned build that fails validation fails the build.
        LockEntry::Source(source) => {
            pinned_commit(&source).unwrap_or_else(|message| panic!("{message}"))
        }
        LockEntry::NoSource => match env::var_os(LANGUAGE_VAR).filter(|v| !v.is_empty()) {
            Some(dir) => {
                let local = local_path_revision(&PathBuf::from(dir));
                for path in &local.watched {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
                if local.revision == UNKNOWN {
                    warn(&format!(
                        "could not read the git HEAD of the {LANGUAGE_VAR} checkout"
                    ));
                }
                local.revision
            }
            None => {
                warn(&format!(
                    "fsl-analyze is a path dependency and {LANGUAGE_VAR} is not set"
                ));
                UNKNOWN.to_owned()
            }
        },
        LockEntry::Missing => {
            warn("Cargo.lock has no fsl-analyze package");
            UNKNOWN.to_owned()
        }
    };
    println!("cargo:rustc-env=FSL_PINNED_REVISION={revision}");
}

fn warn(message: &str) {
    println!("cargo:warning={message}");
}
