# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

# Local gates. `make check` runs every gate and stops at the first failure.
# CI runs the same gates; `.githooks/pre-commit` runs the fast subset on the
# staged index (fmt, reuse, licenses, issue-refs).

REUSE ?= uvx reuse@6.2.0
MSRV_RUN ?= rust-validate 1.98.1
# Tree the file-based gates read (the hook points it at a snapshot of the index).
ROOT ?= .
# Search command for issue-refs; the hook swaps in `git grep --cached -InE`.
ISSUE_REFS_GREP ?= git grep -InE

MULAN_SHA256 := eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c

.PHONY: help fmt clippy test doc deny reuse msrv licenses issue-refs check hooks

help: ## List the targets.
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*## "} {printf "  %-12s %s\n", $$1, $$2}'

fmt: ## Check formatting.
	cargo fmt --all --check --manifest-path $(ROOT)/Cargo.toml

clippy: ## Clippy over the workspace, warnings denied.
	cargo clippy --workspace --all-targets --locked -- -D warnings

test: ## Run the workspace tests.
	cargo test --workspace --locked

doc: ## Build the docs, warnings denied.
	RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked

# CI runs the fetching variant (`cargo deny --locked check`). The local machine
# hook blocks fetching the advisory database, so this runs offline. Never bypass.
deny: ## Supply-chain and licence check (offline).
	cargo deny --offline check

reuse: ## REUSE compliance lint (needs uvx or reuse).
	@command -v $(firstword $(REUSE)) >/dev/null 2>&1 || { \
	  echo "make reuse: required tool '$(firstword $(REUSE))' not found; install uv (uvx) or 'pipx install reuse', or set REUSE=reuse" >&2; exit 1; }
	$(REUSE) --root $(ROOT) lint

msrv: ## cargo check on the MSRV toolchain (1.98.1).
	@command -v $(firstword $(MSRV_RUN)) >/dev/null 2>&1 || { \
	  echo "make msrv: required tool '$(firstword $(MSRV_RUN))' not found; install rust-validate, or run with MSRV_RUN= and RUSTUP_TOOLCHAIN=1.98.1" >&2; exit 1; }
	@v=$$($(MSRV_RUN) rustc --version); case "$$v" in "rustc 1.98.1 "*) ;; *) \
	  echo "make msrv: expected rustc 1.98.1, got '$$v'; set MSRV_RUN to a command that runs the 1.98.1 toolchain" >&2; exit 1;; esac
	$(MSRV_RUN) cargo check --workspace --all-targets --locked

licenses: ## Verify the Mulan text digests and the zh-Hans source fingerprint.
	@set -e; root='$(ROOT)'; fail=0; \
	for f in LICENSE-MULAN LICENSES/MulanPSL-2.0.txt; do \
	  got=$$(sha256sum "$$root/$$f" | cut -d' ' -f1); \
	  if [ "$$got" != "$(MULAN_SHA256)" ]; then \
	    echo "licenses: $$f sha256 $$got != $(MULAN_SHA256)" >&2; fail=1; fi; \
	done; \
	want=$$(grep -m1 'Source SHA-256' "$$root/docs/LICENSING-BACKTRANSLATION.md" | grep -oE '[0-9a-f]{64}' | head -n1); \
	got=$$(sha256sum "$$root/docs/LICENSING.zh-Hans.md" | cut -d' ' -f1); \
	if [ -z "$$want" ] || [ "$$got" != "$$want" ]; then \
	  echo "licenses: docs/LICENSING.zh-Hans.md sha256 $$got != recorded '$$want'" >&2; fail=1; fi; \
	exit $$fail

# Every pattern is written so that it does not match its own text. grep exits
# 0 when it finds a hit (a failure here), 1 when clean, 2 or more on error.
issue-refs: ## Fail on issue/PR tokens and process wording in tracked files.
	@bad=0; \
	$(ISSUE_REFS_GREP) -e '#[0-9]{1,5}\b' -e '/issues/[0-9]+' -e '/pull/[0-9]+' \
	  -e '\bPR ?#?[0-9]+' -e 'CHANGE[-][0-9]+'; rc=$$?; \
	if [ $$rc -ne 1 ]; then bad=1; fi; \
	$(ISSUE_REFS_GREP) -i -e 'this[ ]issue' -e 'the[ ]plan.s' -e 'per[ ]the[ ]plan' \
	  -e 'this[ ]branch' -e 'Task [0-9]+' -e '\bAC[0-9]+\b'; rc=$$?; \
	if [ $$rc -ne 1 ]; then bad=1; fi; \
	if [ $$bad -ne 0 ]; then echo "issue-refs: hits above (or grep error); remove issue/PR tokens and process wording" >&2; fi; \
	exit $$bad

check: ## Run every gate, stopping at the first failure.
	$(MAKE) fmt
	$(MAKE) clippy
	$(MAKE) test
	$(MAKE) doc
	$(MAKE) deny
	$(MAKE) reuse
	$(MAKE) msrv
	$(MAKE) licenses
	$(MAKE) issue-refs

hooks: ## Use .githooks for this clone (local config only).
	git config core.hooksPath .githooks
