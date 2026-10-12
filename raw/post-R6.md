### Stage 2 fix R6 (`2faa697`): licence and issue-reference gates ported to Rust

**User decision applied (verbatim):** "Port gates to Rust (Recommended)".

**Gaps closed (G7, G9, G16-F7, Q-M5, Q-L1)**
- **`tests/repo_gates.rs`** (std-only, 40 tests):
  - Mulan digests and the zh-Hans fingerprint, checked with a hand-written SHA-256 that is verified against NIST vectors;
  - issue/PR tokens and process wording over **every tracked file**;
  - CITATION.cff version agreement with `[workspace.package]`.
- **Fixtures.** Every rule has a named negative fixture.
- **`tests/support`** is the single definition site for the patterns and the index-derived file list (read under `RAF_GATE_ROOT`). `docs_contract.rs` uses both, so its duplicate patterns and walker are removed.
- **G9.** `fill="#000"`, `stroke='#abc'`, `color: #1f2`, `&#38;` and `#123456` pass; `see #123`, `(#1234)` and `Per #12` fail.
- **New forms.** `issue 12`, `GH-12` and `pull request 11` are now caught, and near-miss prose passes.
- **Binary files.** Only a NUL byte marks a file as binary. Latin-1 text is scanned.
- **Gate wiring.**
  - `make licenses` and `make issue-refs` run the full Rust gate binary, so no zero-match filter is possible. The shell recipes are deleted.
  - The pre-commit hook runs both Rust test binaries with `--no-fail-fast` against the staged snapshot, so it now enforces the document contract too (Q-M5).
  - The CI `gates` job is removed; `cargo test --workspace` covers it.

**Checks run**
- `cargo test`: docs_contract 46/46, repo_gates 40/40.
- `make clippy` (`-D warnings`), `make fmt`, `make msrv` (1.98.1), reuse 79/79, actionlint.
- `git hook run pre-commit` passes, with 0.57 s warm wall time.
- In a scratch clone using real signed commits, the hook refused:
  - a staged `#12`;
  - an altered `LICENSE-MULAN`;
  - a capability missing `## Usage`;
  - a CITATION-only version bump.

**Review:** `G-R6-review`. The first pass BLOCKED because invalid UTF-8 was silently skipped. That is fixed, and the zero-match filter and fail-fast advisories were adopted; the recheck gave PASS.
