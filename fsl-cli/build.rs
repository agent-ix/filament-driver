use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR for build scripts"),
    );
    let lock_path = manifest_dir.join("../Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock_path.display());

    let lock = fs::read_to_string(&lock_path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", lock_path.display()));
    let mut current_package_is_fsl_analyze = false;
    let mut revision = None;
    for line in lock.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            current_package_is_fsl_analyze = false;
        } else if line == "name = \"fsl-analyze\"" {
            current_package_is_fsl_analyze = true;
        } else if current_package_is_fsl_analyze && line.starts_with("source = ") {
            let source = line
                .strip_prefix("source = \"")
                .and_then(|source| source.strip_suffix('\"'))
                .expect("fsl-analyze has a quoted Cargo.lock source");
            let (repository, commit) = source
                .rsplit_once('#')
                .expect("fsl-analyze git source has a resolved commit");
            let pinned_revision = repository
                .strip_prefix("git+https://github.com/agent-ix/filament-spec-language?rev=")
                .expect("fsl-analyze must resolve from the pinned FSL git repository");
            assert_eq!(
                pinned_revision, commit,
                "fsl-analyze Cargo.lock revision query and resolved commit differ"
            );
            assert!(
                commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "fsl-analyze source has an invalid resolved commit: {commit}"
            );
            revision = Some(commit);
            break;
        }
    }

    let revision = revision.expect("Cargo.lock contains the pinned fsl-analyze package");
    println!("cargo:rustc-env=FSL_PINNED_REVISION={revision}");
}
