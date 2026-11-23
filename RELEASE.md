# FSL alpha release runbook

Use this runbook to tag and verify the FSL alpha release. The release identity is `v0.2.0`; alpha describes its quality, not a version suffix. Run the release gates on Titan. The release owner pushes each tag only after checking that repository's latest tag and the exact commit to tag.

## Release order

Tag in dependency order. The proposed tags below are the 2026-11-22 snapshot; stop if a repository already has a later tag and choose the next unused repository-native version.

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

Before the first tag, merge and freeze the release pin updates. FSL main at `5275e5086fec403c4d1a6bce25ab7c3660ea5716` uses specification `01f8dd31b8cc540418a4858543aed7231a1ed1ee` and contract IR `14c06b047b6a3b23a4da53723574ad7ce49ba246`. Codegen PR #888 merged as `ecbc3a4c05a0e76152872fce1bd4c0c7af96a1cb`, moving its FSL dependencies from `7713d248be656715cee7ecdac50106991f0158b4` to the FSL revision. Driver PR #244 merged as `2f5c2a228ee0a273498997db97d3b20b2d645727`, moving its FSL pin from `86acfdeedabe40674067342aaf2f639d48fc15f3` to that same SHA. The driver must pin codegen at `ecbc3a4c05a0e76152872fce1bd4c0c7af96a1cb`; its Cargo.lock must resolve the same dependency cohort. Do not tag a dependent repository until its pin update is merged and the release candidate has passed its required gate.

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

The toolchain image's pinned npm tools are separate from the driver's `fsl` version. Examples PR [#104](https://github.com/agent-ix/filament-examples/pull/104), head `971b698d3ccc514297539fa394ee99da80b781aa`, sets `@agent-ix/quire-cli` to `0.52.1` and `@agent-ix/quoin` to `0.36.10` in `image/package.json`. `image/package-lock.json` records the exact npm versions and integrity values, and the Docker build installs them with `npm ci`. During the image build, license records and notices are generated from the installed npm tree and the exact driver and runner sources using Cargo metadata. The generated notices ship in the image under `/usr/share/licenses/filament-toolchain`; generated license records and notices are not checked into the repository. These npm versions describe the toolchain image, not the driver's version.

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

The examples image workflow is different: its manual dispatch builds and pushes the multi-platform `ghcr.io/agent-ix/filament-toolchain` image. It has no non-publishing dry-run. It builds the Ubuntu 26.04 image for Linux/amd64 and Linux/arm64, generates notices from the exact tools and source metadata included in the image, runs `image/smoke.sh`, then publishes the requested image tags. After the driver release tag and examples source tag are ready, hand off to Titan to dispatch the workflow with `tags=0.2.0`, `fsl_source=release`, and `fsl_version=0.2.0`. After publication, verify the built image on Titan:

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

Run this on Titan after the driver release assets, docs tag, and examples tag are available. First complete the release download and full checksum verification above; reuse that `release/` directory here. Prerequisites: Cargo and the pinned Rust toolchain, Docker running, host `readelf` from binutils, matching local docs and examples checkouts, and network access to pull Ubuntu 26.04. Use the actual released Linux GNU `fsl` binary, not a local candidate:

```sh
tar -xzf release/fsl-v0.2.0-x86_64-unknown-linux-gnu.tar.gz -C release fsl
make alpha-journey-gate \
  FILAMENT_DOCS_DIR=/home/peter/dev/filament-docs \
  FILAMENT_EXAMPLES_DIR=/home/peter/dev/filament-examples \
  FSL_RELEASE_BINARY="$PWD/release/fsl" \
  ALPHA_JOURNEY_BASE_IMAGE=ubuntu:26.04
```

The gate runs the install and checked tutorial journey in a disposable Ubuntu 26.04 container. Released mode verifies the binary is Linux ELF, checks its GLIBC requirement ceiling and version output, then exercises the published install route and tutorial commands. Use the matching aarch64 archive and binary on an ARM64 Titan if that is the release platform being verified. Record the gate result and the exact docs, examples, driver, and FSL commits used. The acceptance result is a passing journey against published artifacts.
