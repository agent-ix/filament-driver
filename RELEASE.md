# FSL alpha release runbook

Use this runbook to tag and verify the FSL alpha release. The release identity is `v0.2.0`; alpha describes its quality, not a version suffix. Run the release gates on one release host that has Docker and the pinned Rust toolchain (Titan, Luna or Terra). The release owner pushes each tag only after checking that repository's latest tag and the exact commit to tag.

## Release order

Tag in dependency order. Stop if a repository already has a later tag than the one proposed below and choose the next unused repository-native version.

| Order | Repository | Proposed tag | Consumer role |
| --- | --- | --- | --- |
| 1 | `filament-specification` | `v0.1.2` | FSL standard |
| 2 | `filament-contract-ir` | `v0.1.6` | FSL and codegen model |
| 3 | `filament-spec-language` | `v0.3.0` | Language crates |
| 4 | `filament-contract-codegen` | `v0.1.1` | Driver backend |
| 5 | `filament-driver` | `v0.2.0` | Installed CLI and library |
| 6 | `filament-examples` | `v0.2.0` | Tutorials and toolchain image |
| 7 | `filament-project-template` | `v0.2.0` | New-project starter |
| 8 | `filament-docs` | `v0.2.0` | Published install and tutorials |

Only the driver tag creates downloadable `fsl` release archives. The examples tag carries tutorials and the image workflow; the template and docs tags identify consumer content. Supporting crates such as `filament-exact` and `filament-semantic-value` are consumed at Cargo-pinned commits and do not need separate alpha tags for this install path.

Before the first tag, merge and freeze the release pin updates. The driver, codegen, FSL and runtime pins must form one cohort: the driver `Cargo.toml` and `Cargo.lock` pin the same FSL revision as codegen, the same `filament-contract-model` (contract IR) revision as FSL and codegen, the same `filament-semantic-value` and `filament-eval` revisions as FSL, and the codegen and runtime revisions that were merged for this release. `Cargo.lock` must hold one entry each for `filament-semantic-value`, `filament-contract-model` and `filament-contract-runtime`, and every `fsl-*` crate at the one FSL revision; a second copy breaks type identity. Tag each dependency repository at the commit the verified candidate resolves (the revision in the driver `Cargo.lock`; for `filament-specification`, the revision that FSL's `Cargo.toml` pins at that FSL commit), not at a later `main`. Tag examples and docs at the commits the candidate gates ran against, the template at its reviewed commit, and the driver at `DRIVER_SHA`. Do not tag a dependent repository until its pin update is merged and the release candidate has passed its required gate.

## One-pass candidate verification

After all pin moves are merged, freeze the driver candidate commit. Record the full 40-character SHA as `DRIVER_SHA`; paste that exact value into the release-host handoff and the source image build. Do not substitute a branch name or move the candidate after freezing it.

```sh
cd /home/peter/dev/filament-driver
test -z "$(git status --porcelain)"
git status --short
git rev-parse HEAD
```

On the release host, check out that frozen commit. Run the workspace compile first, then the named alpha CLI proofs (named tests only, never the whole suite):

```sh
cd /home/peter/dev/filament-driver
git fetch origin main
export DRIVER_SHA=PASTE_FROZEN_40_CHARACTER_SHA_HERE
git checkout "$DRIVER_SHA"
test "$(git rev-parse HEAD)" = "$DRIVER_SHA"
CARGO_BUILD_JOBS=2 cargo check --locked --workspace --all-targets
CARGO_BUILD_JOBS=2 cargo test --locked -p fsl-cli --test cli_contract run_paid_short_json_contains_the_state_rule_witness -- --exact
CARGO_BUILD_JOBS=2 cargo test --locked -p fsl-cli --test cli_contract run_charge_changes_total_json_contains_the_frame_witness -- --exact
CARGO_BUILD_JOBS=2 cargo test --locked -p fsl-cli --test frontend_cli timed_analyze::analyze_zone_deadline_holds -- --exact
CARGO_BUILD_JOBS=2 cargo test --locked -p fsl-cli --test frontend_cli timed_analyze::analyze_zone_deadline_refuted -- --exact
```

Then run the named-test sweep on the final candidate, before any tag. For every area of the driver that changed since its previous release tag (`v0.1.4` for this release; list the areas with `git diff --stat v0.1.4..$DRIVER_SHA`), run name-filtered tests: a test-name filter or an exact test name, never the whole suite. Give each command one test target (`--lib` or `--test <binary>`) so that it prints one `test result:` line, and fail the sweep if that line reads `ok. 0 passed`, which is what cargo prints, with exit status 0, when a filter matches no test. For example:

```sh
CARGO_BUILD_JOBS=2 cargo test --locked -p <package> --lib <module>::<filter>
CARGO_BUILD_JOBS=2 cargo test --locked -p <package> --test <binary> <filter>
```

Build the candidate pair with the same `dist` profile used by the release workflow:

```sh
CARGO_BUILD_JOBS=2 cargo build --locked --profile dist -p fsl-cli --bin fsl
CARGO_BUILD_JOBS=2 cargo build --locked --profile dist -p filament-plugin-host --bin fsl-plugin-launch
```

Run the candidate-mode journey with the Ubuntu 26.04 baseline. `FSL_BINARY` points to the just-built candidate; leave `FSL_RELEASE_BINARY` unset. Candidate mode skips the released-artifact check. In both modes the gate skips tutorial setup blocks that start with `git clone` or `cd`, and reports them as skipped, not failed.

```sh
CARGO_BUILD_JOBS=2 make alpha-journey-gate \
  FILAMENT_DOCS_DIR=/home/peter/dev/filament-docs \
  FILAMENT_EXAMPLES_DIR=/home/peter/dev/filament-examples \
  FSL_BINARY="$PWD/target/dist/fsl" \
  ALPHA_JOURNEY_BASE_IMAGE=ubuntu:26.04
```

From the examples checkout, build and smoke a local source-mode image on the release host. Set `DRIVER_SHA` to the frozen full SHA. This local Docker build does not publish an image; the Dockerfile applies its own bounded release profile settings to the source build.

```sh
cd /home/peter/dev/filament-examples
docker build --pull -f image/Dockerfile \
  --build-arg UBUNTU=26.04 \
  --build-arg FSL_SOURCE=source \
  --build-arg FSL_REVISION="$DRIVER_SHA" \
  --build-arg IMAGE_VERSION=0.2.0 \
  -t filament-toolchain:source-candidate .
docker run --rm filament-toolchain:source-candidate smoke.sh
```

After the release-host gates pass, stop for Draco's explicit go-ahead. Only then tag the repositories in the release order above, one repository at a time. Recheck each latest tag and candidate commit immediately before that tag push; verify the remote tag before proceeding. A driver tag publishes the GitHub release, and the examples image workflow publishes its requested image tags. Do not create either tag or publish either artifact before the go-ahead.

## Tag and verify one repository at a time

For each row, enter that repository's checkout and substitute its tag. Check the latest tag immediately before tagging. Confirm the reviewed commit is checked out, then create and push the tag:

```sh
git fetch origin --tags
git tag --list 'v*' --sort=-version:refname | head -1
git rev-parse HEAD
git status --short
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin refs/tags/v0.1.2
git ls-remote --tags --refs origin refs/tags/v0.1.2
```

Replace `v0.1.2` with the row's proposed tag. Confirm the remote tag resolves to the intended commit before moving to the next row. For annotated tags, also verify the peeled commit with `git ls-remote origin 'refs/tags/v0.1.2^{}'`.

## Version sources

For this release, keep these authoritative values aligned with `0.2.0` where applicable:

| Repository | Version source | Required value or action |
| --- | --- | --- |
| Driver | [manifest](Cargo.toml), [lockfile](Cargo.lock) | `0.2.0`; regenerate lockfile |
| Driver | Release workflow | Tag matches package version |
| Examples | Dockerfile and image workflow | Defaults use `0.2.0` |
| Examples | Image README | Keep examples aligned with defaults |
| Examples | Toolchain npm version sources | `image/package.json` and its lockfile |
| Examples | Quire CLI and Quoin versions | `0.52.1` and `0.36.10` |
| Examples | Image installation command | Docker uses `npm ci` |
| Examples | Image license records and notices | Generated from installed tools and Cargo metadata |
| Docs | [release notes](https://github.com/agent-ix/filament-docs/blob/main/src/content/docs/fsl/releases/v0.2.0.md) | Release page for this identity |
| Docs | [setup guide](https://github.com/agent-ix/filament-docs/blob/main/src/content/docs/fsl/guides/set-up-your-environment.md) | Show CLI version and full FSL SHA |
| Template | No package version field | Tag the reviewed template commit |

`filament-spec-language` has package version `0.3.0`; repository tag `v0.3.0` matches it. The standard uses dynamic package versioning. Contract IR and codegen repository tags are independent repository tags; do not change their Cargo package versions solely to make them equal the driver version. Before a later alpha, search these files and the image configuration for the selected version, update only the authoritative values, then regenerate Cargo.lock for manifest changes.

The toolchain image's pinned npm tools are separate from the driver's `fsl` version. The examples image sets `@agent-ix/quire-cli` to `0.52.1` and `@agent-ix/quoin` to `0.36.10` in `image/package.json`. `image/package-lock.json` records the exact npm versions and integrity values, and the Docker build installs them with `npm ci`. During the image build, `cargo metadata --locked` runs against `/tmp/driver-source` at the exact release revision, `/tmp/licences/tools/fsl-examples` at the image revision, `/tmp/quire-cli-source` from `agent-ix/quire-cli` tag `v0.52.1`, and `/tmp/quoin-source` from `agent-ix/quoin` tag `v0.36.10`. The npm inventory comes from installed `/opt/npm/node_modules` and `/opt/npm/package-lock.json`. The generated notices ship in the image under `/usr/share/licenses/filament-toolchain`; generated license records and notices are not checked into the repository. These npm versions describe the toolchain image, not the driver's version.

## Driver release workflow

Only `.github/workflows/release.yml` builds the versioned driver archives. It builds `fsl` and `fsl-plugin-launch` for Linux x86_64 and aarch64 on Ubuntu 26.04, plus macOS arm64 and x86_64. It packages each pair as `fsl-v<VERSION>-<TARGET>.tar.gz`, checks the packaged CLI version, and uploads workflow artifacts. Linux checks inspect both packaged ELF files, reject `GLIBC_PRIVATE`, and require the maximum numeric GLIBC symbol version to be at most 2.43. A tag push runs the publish job, which checks tag/version agreement, writes `SHA256SUMS.txt`, and creates the GitHub Release.

On macOS, `fsl` defaults provider memory limits to zero, enforces CPU time only, and writes a memory-not-enforced notice to stderr. With a configured provider, explicitly setting `--provider-limit max_memory_bytes=<bytes>` is refused before provider start for `run`, `analyze`, `monitor`, and `replay` (exit 21, unsupported on this platform). The launcher itself refuses a nonzero memory argument on macOS; zero allows provider execution without a memory cap. The release workflow therefore keeps archive, executable, and version checks on all targets but runs its nonzero-memory launcher probe only on Linux.

The manual release workflow is a non-publishing build when dispatched from `main` with `publish=false`. The release owner coordinates this dry run with this command; do not dispatch it from this runbook or use it as a substitute for checking the final tag's artifacts:

```sh
gh workflow run release.yml --repo agent-ix/filament-driver --ref main -f publish=false
```

After the driver tag, download the actual release assets and verify their checksums:

```sh
gh release view v0.2.0 --repo agent-ix/filament-driver
gh release download v0.2.0 --repo agent-ix/filament-driver --dir release
(cd release && sha256sum -c SHA256SUMS.txt)
```

The examples image workflow is different: its manual dispatch builds and pushes the multi-platform `ghcr.io/agent-ix/filament-toolchain` image. It has no non-publishing dry-run. It builds the Ubuntu 26.04 image for Linux/amd64 and Linux/arm64, generates notices from the exact tools and source metadata included in the image, runs `image/smoke.sh`, then publishes the requested image tags. After the driver release tag and examples tag `v0.2.0` are ready, hand off to the release host to dispatch from the tagged examples source:

```sh
gh workflow run image.yml --repo agent-ix/filament-examples --ref v0.2.0 \
  -f tags=0.2.0 -f fsl_version=0.2.0 -f fsl_source=release
```

After publication, verify the built image on the release host:

```sh
IMAGE=ghcr.io/agent-ix/filament-toolchain:0.2.0
docker pull "$IMAGE"
docker run --rm "$IMAGE" smoke.sh
docker run --rm --entrypoint sh "$IMAGE" -ec '
  test -s /usr/share/licenses/filament-toolchain/THIRD-PARTY.md
  test -s /usr/share/licenses/filament-toolchain/THIRD-PARTY-LICENSES.txt
'
```

Inspect the generated notices under `/usr/share/licenses/filament-toolchain`. Confirm they identify the tagged driver and runner sources and the npm tools at `0.52.1` and `0.36.10`.

## Released-mode journey gate

Run this on the release host after the driver release assets, docs tag, and examples tag are available. First complete the release download and full checksum verification above; reuse that `release/` directory here. Prerequisites: Cargo and the pinned Rust toolchain, Docker running, host `readelf` from binutils, matching local docs and examples checkouts, and network access to pull Ubuntu 26.04. Use the actual released Linux GNU `fsl` binary, not a local candidate:

```sh
tar -xzf release/fsl-v0.2.0-x86_64-unknown-linux-gnu.tar.gz -C release fsl
make alpha-journey-gate \
  FILAMENT_DOCS_DIR=/home/peter/dev/filament-docs \
  FILAMENT_EXAMPLES_DIR=/home/peter/dev/filament-examples \
  FSL_RELEASE_BINARY="$PWD/release/fsl" \
  ALPHA_JOURNEY_BASE_IMAGE=ubuntu:26.04
```

The gate runs the install and checked tutorial journey in a disposable Ubuntu 26.04 container. Released mode verifies the binary is Linux ELF, checks its GLIBC requirement ceiling and version output, then exercises the published install route and tutorial commands. Use the matching aarch64 archive and binary on an ARM64 release host if that is the release platform being verified. Record the gate result. The acceptance result is a passing journey against published artifacts.
