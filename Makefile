# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

# Local gates. `make check` runs every gate and stops at the first failure.
# CI runs the same gates. `licenses` and `issue-refs` are thin selectors over
# the Rust gates in `tests/repo_gates.rs`, so they are a subset of `make test`.
# Both run the whole `repo_gates` binary (under a second) with no name filter,
# so a renamed test can never leave a target running zero tests.
# `.githooks/pre-commit` runs fmt and reuse on a checkout of the staged index,
# plus the `repo_gates`, `docs_contract` and `self_workplan` test binaries over that
# checkout.
#
# Gates beyond the workspace tests:
#   test-release  the raf-source tests again in release mode, because the parse
#                 stack cost of a nesting level differs between the profiles.
#   fuzz          fmt, clippy and deny on the fuzz harness, then a bounded run of
#                 each of its targets over its committed corpus. Needs cargo-fuzz.
#   cross-check   cargo check of every raf-source target (the library, its unit
#                 tests and its test binaries), warnings denied, for the Apple,
#                 FreeBSD and Windows targets. Needs the three targets installed
#                 (rustup target add).

REUSE ?= uvx reuse@6.2.0
MSRV_RUN ?= rust-validate 1.98.1
# The SPDX text of the index that `docs-sync` creates when it is missing.
DOCS_COPYRIGHT ?= 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
DOCS_LICENCE ?= CC-BY-4.0
# Tree that `fmt` and `reuse` read (the hook points it at a snapshot of the index).
ROOT ?= .
# The pinned cargo-fuzz that CI installs; keep local and CI versions in step.
CARGO_FUZZ_VERSION ?= 0.13.2
# CI sets this to `--locked`, which lets cargo-deny fetch the advisory database.
FUZZ_DENY ?= --offline
FUZZ_RUNS ?= 5000
FUZZ_SEED ?= 1
# Inputs are cut to this length, which every corpus file fits, so a run reaches
# inputs as large as the chains that once overflowed the parse stack.
FUZZ_MAX_LEN ?= 400000
# The runs build without `cfg(fuzzing)`: under it `proc-macro2` drops the
# positions of what it parses, and the source index refuses every item without
# them, so a load would never reach the index.
# The memory limits of every run, in MiB: the resident set, and one allocation.
FUZZ_RSS_LIMIT_MB ?= 2048
FUZZ_MALLOC_LIMIT_MB ?= 512
CROSS_TARGETS ?= x86_64-apple-darwin x86_64-unknown-freebsd x86_64-pc-windows-gnu

.PHONY: help fmt clippy test test-release doc deny reuse msrv fuzz cross-check licenses issue-refs docs-sync check hooks

help: ## List the targets.
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*## "} {printf "  %-12s %s\n", $$1, $$2}'

fmt: ## Check formatting.
	cargo fmt --all --check --manifest-path '$(ROOT)/Cargo.toml'
	cargo fmt --check --manifest-path '$(ROOT)/fuzz/Cargo.toml'

clippy: ## Clippy over the workspace, warnings denied.
	cargo clippy --workspace --all-targets --locked -- -D warnings

test: ## Run the workspace tests.
	cargo test --workspace --locked

test-release: ## Run the raf-source tests in release mode.
	cargo test -p raf-source --release --locked

doc: ## Build the docs, warnings denied.
	RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked

# CI installs cargo-deny 0.20.2 and runs the fetching variant
# (`cargo deny --workspace --locked check`); keep local and CI versions in step.
# The local machine hook blocks fetching the advisory database, so this runs
# offline. Never bypass.
deny: ## Supply-chain and licence check (offline).
	cargo deny --offline --workspace check

reuse: ## REUSE compliance lint (needs uvx or reuse).
	@command -v $(firstword $(REUSE)) >/dev/null 2>&1 || { \
	  echo "make reuse: required tool '$(firstword $(REUSE))' not found; install uv (uvx) or 'pipx install reuse', or set REUSE=reuse" >&2; exit 1; }
	$(REUSE) --root '$(ROOT)' lint

msrv: ## cargo check on the MSRV toolchain (1.98.1).
	@command -v $(firstword $(MSRV_RUN)) >/dev/null 2>&1 || { \
	  echo "make msrv: required tool '$(firstword $(MSRV_RUN))' not found; install rust-validate, or run with MSRV_RUN= and RUSTUP_TOOLCHAIN=1.98.1" >&2; exit 1; }
	@v=$$($(MSRV_RUN) rustc --version); case "$$v" in "rustc 1.98.1 "*) ;; *) \
	  echo "make msrv: expected rustc 1.98.1, got '$$v'; set MSRV_RUN to a command that runs the 1.98.1 toolchain" >&2; exit 1;; esac
	$(MSRV_RUN) cargo check --workspace --all-targets --locked
	$(MSRV_RUN) cargo check --manifest-path '$(ROOT)/fuzz/Cargo.toml' --all-targets --locked

fuzz: ## Fuzz harness: fmt, clippy, deny, then a bounded run (needs cargo-fuzz).
	@command -v cargo-fuzz >/dev/null 2>&1 || { \
	  echo "make fuzz: required tool 'cargo-fuzz' not found; install it with 'cargo install --locked cargo-fuzz@$(CARGO_FUZZ_VERSION)'" >&2; exit 1; }
	cargo fmt --check --manifest-path '$(ROOT)/fuzz/Cargo.toml'
	cargo clippy --manifest-path '$(ROOT)/fuzz/Cargo.toml' --locked -- -D warnings
	cd '$(ROOT)/fuzz' && cargo deny $(FUZZ_DENY) check
	@work=$$(mktemp -d) && trap 'rm -rf "$$work"' EXIT && \
	  for target in parse-source parse-document load-workspace census-normalize census-report-read; do \
	    mkdir "$$work/$$target" && \
	    cargo fuzz run --no-cfg-fuzzing "$$target" "$$work/$$target" "fuzz/corpus/$$target" -- \
	      -runs=$(FUZZ_RUNS) -seed=$(FUZZ_SEED) -max_len=$(FUZZ_MAX_LEN) \
	      -rss_limit_mb=$(FUZZ_RSS_LIMIT_MB) -malloc_limit_mb=$(FUZZ_MALLOC_LIMIT_MB) || exit 1; \
	  done

cross-check: ## cargo check every raf-source target for the Apple, FreeBSD and Windows targets, warnings denied.
	@for target in $(CROSS_TARGETS); do \
	  rustup target list --installed | grep -qx "$$target" || { \
	    echo "make cross-check: target '$$target' is not installed; run 'rustup target add $$target'" >&2; exit 1; }; \
	  RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target "$$target" || exit 1; \
	done

licenses: ## Verify the Mulan text digests and the zh-Hans source fingerprint.
	cargo test --locked --test repo_gates

issue-refs: ## Fail on issue/PR tokens and process wording in tracked files.
	cargo test --locked --test repo_gates

docs-sync: ## Rewrite the generated documentation blocks and create the capability index.
	cargo run --locked -q -p cargo-workplan -- --root '$(ROOT)' docs sync --copyright '$(DOCS_COPYRIGHT)' --licence '$(DOCS_LICENCE)'

check: ## Run every gate, stopping at the first failure.
	$(MAKE) fmt
	$(MAKE) clippy
	$(MAKE) test
	$(MAKE) test-release
	$(MAKE) doc
	$(MAKE) deny
	$(MAKE) reuse
	$(MAKE) msrv
	$(MAKE) fuzz
	$(MAKE) cross-check
	$(MAKE) licenses
	$(MAKE) issue-refs

hooks: ## Use .githooks for this clone (local config only).
	git config core.hooksPath .githooks
