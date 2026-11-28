# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

# Local gates run on native Linux. `make precommit` checks the staged tree.
# The merge gate runs the full checks and then the workspace tests.
# The license and reference selectors use the same complete repository policy
# test binary as the workspace test target.
#
# Gates beyond the workspace tests:
#   test-release  the raf-source tests again in release mode, because the parse
#                 stack cost of a nesting level differs between the profiles.
#   fuzz          fmt, clippy and deny on the fuzz harness, then a bounded run of
#                 each of its targets over its committed corpus. Needs cargo-fuzz.
# Standard checks execute and compile only for the native Linux host.

REUSE ?= uvx reuse@6.2.0
MSRV_RUN ?= rust-validate 1.98.1
export CARGO_NET_OFFLINE := false
# The SPDX text of the index that `docs-sync` creates when it is missing.
DOCS_COPYRIGHT ?= 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
DOCS_LICENCE ?= CC-BY-4.0
# Tree selected by the gate. The staged formatter and lints read a snapshot.
ROOT ?= .
# The pinned cargo-fuzz used by the local instrumented gate.
CARGO_FUZZ_VERSION ?= 0.13.2
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
.PHONY: help precommit cross-check fmt clippy clippy-all clippy-fuzz clippy-production registry-archives census workplan benches test test-release doc deny reuse msrv fuzz fuzz-check fuzz-run licenses issue-refs docs-sync check merge-gate hooks

help: ## List the targets.
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*## "} {printf "  %-12s %s\n", $$1, $$2}'

precommit: ## Check staged formatting, shipping lints, dependencies and document text.
	.githooks/check-staged

fmt: ## Check formatting of both locked workspaces.
	bash .githooks/check-format '$(ROOT)/Cargo.toml'
	bash .githooks/check-format '$(ROOT)/fuzz/Cargo.toml'

clippy: ## Check the shipping libraries and binaries with the production lint flags.
	.githooks/production-clippy '$(ROOT)' '$(CURDIR)/Cargo.toml' --check-only

clippy-all: ## Check all targets in both workspaces, warnings denied.
	cargo clippy --workspace --all-targets --locked -- -D warnings
	$(MAKE) clippy-fuzz

clippy-fuzz: ## Check all targets in the nested fuzz workspace.
	cargo clippy --manifest-path '$(ROOT)/fuzz/Cargo.toml' --all-targets --locked -- -D warnings

clippy-production: ## Check the shipping library and binary targets and record the compiler verdict.
	.githooks/production-clippy '$(ROOT)' '$(CURDIR)/Cargo.toml'

registry-archives: ## Fetch locked archives into the build directory and print their cache path.
	@build=$$(realpath -m -- "$${CARGO_TARGET_DIR:-$(CURDIR)/target}") && \
	  CARGO_TARGET_DIR="$$build" cargo run --quiet --locked -p rust-agentic-foundations --example prepare_census_archives -- '$(ROOT)' "$$build/census-archives" && \
	  printf '%s\n' "$$build/census-archives"

test: ## Run the workspace tests with fresh production Clippy evidence.
	@evidence=$$(mktemp -d) && trap 'rm -rf "$$evidence"' EXIT && \
	  export RAF_CENSUS_LINT_EVIDENCE="$$evidence/run.ttl" && \
	  archives=$$($(MAKE) --no-print-directory registry-archives) && \
	  export RAF_GATE_REGISTRY_CACHE="$$archives" && \
	  $(MAKE) --no-print-directory clippy-production && cargo test --workspace --locked

census: ## Check the locked repository ledger and debt.
	@evidence=$$(mktemp -d) && trap 'rm -rf "$$evidence"' EXIT && \
	  export RAF_CENSUS_LINT_EVIDENCE="$$evidence/run.ttl" && \
	  archives=$$($(MAKE) --no-print-directory registry-archives) && \
	  export RAF_GATE_REGISTRY_CACHE="$$archives" && \
	  $(MAKE) --no-print-directory clippy-production && \
	  cargo run --quiet --locked -p rust-agentic-foundations --example repository_census

workplan: ## Check the repository's documents, bindings and assessments.
	cargo test --locked -p raf-docs-contract --test docs_contract repository_workplan_is_clean_and_assessed -- --exact

benches: ## Build the workspace benchmarks.
	cargo bench --workspace --no-run --locked

test-release: ## Run the raf-source tests in release mode.
	cargo test -p raf-source --release --locked

cross-check: ## Compile raf-source for an Apple target, which has no parallel parser.
	cargo check --locked -p raf-source --target x86_64-apple-darwin

doc: ## Build the docs, warnings denied.
	RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked

deny: ## Check locked workspace supply-chain and licence policy.
	cargo deny --workspace --locked check
	cd '$(ROOT)/fuzz' && cargo deny --locked check

reuse: ## REUSE compliance lint (needs uvx or reuse).
	@command -v $(firstword $(REUSE)) >/dev/null 2>&1 || { \
	  echo "make reuse: required tool '$(firstword $(REUSE))' not found; install uv (uvx) or 'pipx install reuse', or set REUSE=reuse" >&2; exit 1; }
	$(REUSE) --root '$(ROOT)' lint

msrv: ## cargo check on the MSRV toolchain (1.98.1).
	@if test -n '$(strip $(MSRV_RUN))' && ! command -v $(firstword $(MSRV_RUN)) >/dev/null 2>&1; then \
	  echo "make msrv: required tool '$(firstword $(MSRV_RUN))' not found; install rust-validate, or run with MSRV_RUN= and RUSTUP_TOOLCHAIN=1.98.1" >&2; exit 1; fi
	cargo run --locked --example msrv_dependency_audit -p rust-agentic-foundations -- --check '$(ROOT)' $(MSRV_RUN)

fuzz: ## Fuzz harness: fmt, clippy, deny, then a bounded run (needs cargo-fuzz).
	bash .githooks/check-format '$(ROOT)/fuzz/Cargo.toml'
	$(MAKE) fuzz-check
	$(MAKE) fuzz-run

fuzz-check: ## Fuzz harness lint and dependency policy.
	$(MAKE) clippy-fuzz
	cd '$(ROOT)/fuzz' && cargo deny --locked check

fuzz-run: ## Run each instrumented target over its corpus and bounded mutations.
	@command -v cargo-fuzz >/dev/null 2>&1 || { \
	  echo "make fuzz: required tool 'cargo-fuzz' not found; install it with 'cargo install --locked cargo-fuzz@$(CARGO_FUZZ_VERSION)'" >&2; exit 1; }
	@work=$$(mktemp -d) && trap 'rm -rf "$$work"' EXIT && \
	  RAF_FUZZ_CARGO=$$(command -v cargo) && export RAF_FUZZ_CARGO && \
	  export PATH='$(CURDIR)/fuzz/tools/locked-cargo':"$$PATH" && \
	  for target in parse-source parse-document load-workspace census-normalize census-report-read; do \
	    mkdir "$$work/$$target" && \
	    cargo-fuzz fuzz run --no-cfg-fuzzing --codegen-units 16 "$$target" "$$work/$$target" "fuzz/corpus/$$target" -- \
	      -runs=$(FUZZ_RUNS) -seed=$(FUZZ_SEED) -max_len=$(FUZZ_MAX_LEN) \
	      -rss_limit_mb=$(FUZZ_RSS_LIMIT_MB) -malloc_limit_mb=$(FUZZ_MALLOC_LIMIT_MB) || exit 1; \
	  done

licenses: ## Verify the Mulan text digests and the zh-Hans source fingerprint.
	cargo test --locked --test repo_gates

issue-refs: ## Fail on issue/PR tokens and process wording in tracked files.
	cargo test --locked --test repo_gates

docs-sync: ## Rewrite the generated documentation blocks and create the capability index.
	cargo run --locked -q -p cargo-raf-workplan -- --root '$(ROOT)' docs sync --copyright '$(DOCS_COPYRIGHT)' --licence '$(DOCS_LICENCE)'

check: ## Run the static, compiler, policy and audit gates.
	$(MAKE) precommit
	$(MAKE) clippy-all
	$(MAKE) benches
	$(MAKE) census
	$(MAKE) workplan
	$(MAKE) test-release
	$(MAKE) cross-check
	$(MAKE) doc
	$(MAKE) deny
	$(MAKE) reuse
	$(MAKE) msrv
	$(MAKE) fuzz-run
	$(MAKE) licenses

merge-gate: ## Run the full checks and workspace tests on the exact merge candidate.
	+@common=$$(git rev-parse --path-format=absolute --git-common-dir) && \
	  target=$$(realpath -m -- "$${CARGO_TARGET_DIR:-$(CURDIR)/target}") && \
	  case "$$target" in \
	    "$${common%/*}"|"$${common%/*}/"*) \
	      echo 'make merge-gate: set CARGO_TARGET_DIR to an absolute directory outside the repository' >&2; exit 1;; \
	  esac && \
	  unset RAF_GATE_ROOT && export CARGO_TARGET_DIR="$$target" && \
	  $(MAKE) check ROOT='$(CURDIR)' && $(MAKE) test ROOT='$(CURDIR)'

hooks: ## Use .githooks for this clone (local config only).
	git config core.hooksPath .githooks
