# =============================================================================
# filament-driver Makefile
# =============================================================================

CARGO ?= cargo
RUSTUP ?= rustup
NPM ?= npm

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make cli-contracts    - Test an explicit prebuilt FSL through real subprocesses"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make tools            - Install/check rustfmt, clippy, cargo-deny, and quire"
	@echo "  make deny             - cargo deny check licenses sources"
	@echo "  make spec             - quire validate over spec/"
	@echo "  make docs             - cargo doc, denying missing/broken doc links"
	@echo "  make examples-check   - Build fsl here and run the filament-examples suite (running + staged)"
	@echo "  make alpha-journey-gate - Install FSL and run the alpha tutorial journey in Docker"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + examples-check + deny + docs + spec)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets --features fsl-cli/tc018-test-adapter -- -D warnings
	$(CARGO) clippy -p filament-driver --test examples_gate -- -D warnings

.PHONY: test
test:
	$(CARGO) fetch --locked
	$(CARGO) test
	@set -eu; \
	run_tc018_test() { \
		name="$$1"; \
		output="$$($(CARGO) test --features tc018-test-adapter -p fsl-cli --test binary_process "$$name" -- --exact 2>&1)" || { \
			printf '%s\n' "$$output"; exit 1; \
		}; \
		printf '%s\n' "$$output"; \
		printf '%s\n' "$$output" | grep -Fq 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;' || { \
			echo "expected exactly one TC-018 test: $$name" >&2; exit 1; \
		}; \
	}; \
	run_tc018_test reserved_exit_code_uses_internal_failure_fallback; \
	run_tc018_test fifo_broken_pipe_keeps_refusal_exit_without_output_failure_diagnostic; \
	case "$$(uname -s)" in \
		Linux) run_tc018_test full_stdout_device_selects_internal_failure_and_output_failure_diagnostic ;; \
		Darwin) ;; \
		*) echo "TC-018 process tests require macOS or Linux" >&2; exit 1 ;; \
	esac

.PHONY: cli-contracts
cli-contracts:
	@test -n "$(FSL)" || { echo "set FSL=/absolute/path/to/the/prebuilt/fsl" >&2; exit 1; }
	FSL="$(FSL)" CARGO_TARGET_DIR="$(CURDIR)/target" $(CARGO) test --locked --manifest-path cli-contracts/Cargo.toml -- --test-threads=1

.PHONY: build
build:
	$(CARGO) build --release

# Install the host tools used by the repository's Make targets. rustup, Cargo,
# npm, and make themselves must already be available; this target bootstraps the
# rustfmt and clippy components, cargo-deny, and Quire on macOS and Linux.
.PHONY: tools
tools:
	@set -eu; \
	case "$$(uname -s)" in Darwin|Linux) ;; *) echo "make tools supports macOS and Linux" >&2; exit 1 ;; esac; \
	for tool in "$(RUSTUP)" "$(CARGO)" bash; do \
		if ! command -v "$$tool" >/dev/null 2>&1; then \
			echo "make tools requires $$tool on PATH; install rustup and bash first" >&2; exit 1; \
		fi; \
	done; \
	$(RUSTUP) component add rustfmt clippy; \
	if ! $(CARGO) deny --version >/dev/null 2>&1; then \
		$(CARGO) install --locked cargo-deny; \
	fi; \
	is_quire_cli() { \
		command -v quire >/dev/null 2>&1 || return 1; \
		quire --version 2>/dev/null | grep -Eq '^quire [^[:space:]]+ \(engine [^[:space:]]+\)$$'; \
	}; \
	if ! is_quire_cli; then \
		if ! command -v "$(NPM)" >/dev/null 2>&1; then \
			echo "make tools needs npm on PATH to install @agent-ix/quire-cli" >&2; exit 1; \
		fi; \
		$(NPM) install --global @agent-ix/quire-cli; \
	fi; \
	$(CARGO) deny --version; \
	quire_version="$$(quire --version 2>&1)" || { echo "could not run quire --version" >&2; exit 1; }; \
	if ! printf '%s\n' "$$quire_version" | grep -Eq '^quire [^[:space:]]+ \(engine [^[:space:]]+\)$$'; then \
		echo "expected the @agent-ix/quire-cli version output 'quire X (engine Y)'; ensure its npm global bin directory is on PATH" >&2; \
		exit 1; \
	fi; \
	printf '%s\n' "$$quire_version"

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses sources

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

# =============================================================================
# Documentation
# =============================================================================

# cargo doc is a separate lint pass from clippy: rustdoc-only lints such as
# rustdoc::broken_intra_doc_links only fire here, never under `make lint`.
.PHONY: docs
docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --all-features

# =============================================================================
# Specification
# =============================================================================

.PHONY: spec
spec:
	quire validate 'spec/**/*.md'

# =============================================================================
# Examples gate
# =============================================================================

# The filament-examples suite, following its main branch. tests/examples_gate.rs
# builds fsl from this workspace and runs the suite's runner with it.
EXAMPLES_REPO ?= https://github.com/agent-ix/filament-examples.git
EXAMPLES_DIR ?= $(CURDIR)/target/filament-examples

$(EXAMPLES_DIR)/.git:
	git clone --quiet --branch main $(EXAMPLES_REPO) $(EXAMPLES_DIR)

.PHONY: examples-fetch
examples-fetch: $(EXAMPLES_DIR)/.git
	git -C $(EXAMPLES_DIR) fetch --quiet $(EXAMPLES_REPO) main
	git -C $(EXAMPLES_DIR) checkout --quiet --detach FETCH_HEAD

.PHONY: examples-check
examples-check: examples-fetch
	FSL_EXAMPLES_DIR=$(EXAMPLES_DIR) $(CARGO) test --locked --test examples_gate

# Run the install page and checked command examples in tutorial pages in a
# disposable Ubuntu container. Pass the exact docs and examples checkouts under
# review; FSL_BINARY selects a local release candidate instead of a download.
FILAMENT_DOCS_DIR ?= $(CURDIR)/../filament-docs
FILAMENT_EXAMPLES_DIR ?= $(CURDIR)/../filament-examples
FSL_BINARY ?=
ALPHA_JOURNEY_BASE_IMAGE ?= ubuntu:22.04

.PHONY: alpha-journey-gate
alpha-journey-gate:
	$(CARGO) run --locked -p filament-driver --bin alpha-journey-gate -- --docs "$(FILAMENT_DOCS_DIR)" --examples "$(FILAMENT_EXAMPLES_DIR)" --base-image "$(ALPHA_JOURNEY_BASE_IMAGE)" $(if $(FSL_BINARY),--fsl-binary "$(FSL_BINARY)")

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test examples-check deny docs spec
