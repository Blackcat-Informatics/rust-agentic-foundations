# Squash notes DRAFT (Stage 2). Not final: Stage 3 seeds `${WORKTREE}.squash_notes` after its binding gates.

feat: scaffold the RAF workspace, project documents, brand and spec-first capability docs

## Behaviour delivered
- **Workspace.** A non-virtual root package with workspace lints, a floating nightly toolchain, MSRV 1.98.1 (owner decision), and rustfmt, clippy and cargo-deny configuration.
- **Gates.** `make check` runs fmt, clippy `-D warnings`, test, doc, deny, reuse, the msrv build, the licence digests and the issue-reference gates. The last two are Rust tests in `tests/repo_gates.rs`.
- **Hooks.** The pre-commit and pre-merge-commit hooks run the fast gates and both Rust test binaries against the staged index.
- **CI.** It has pinned actions and tools, a concurrency group and Dependabot.
- **Licensing.**
  - `MIT OR Apache-2.0 OR MulanPSL-2.0` for code and CC-BY-4.0 for documentation.
  - REUSE-compliant.
  - Byte-pinned Mulan text.
  - A zh-Hans explanation with an independent backtranslation record.
- **Project documents.** Adapted from purrdf: AGENTS, CLAUDE, CONTRIBUTING, Code of Conduct, SECURITY (with the trust model and the PGP fingerprint), templates, CITATION, cliff with per-release dedup, and CHANGELOG.
- **Brand.** The RAF brand, logo, social preview and README.
- **Capability documents.** 21 of them: 16 designed and 5 considered. They come with 21 change records and 2 architecture pages, and they absorb the planning essay.
- **Document contract.** `tests/docs_contract.rs` (114 tests) enforces the OKF profile, including the capability inventory.

## Owner decisions
- MSRV 1.98.1 instead of 1.97.1.
- `.goals`, `.baseline` and `.stage/` are never committed.
- Census code may be ported from any copy without notice.
- G3: "(1) but add an issue for (2)". The Gate owns the census path, and a separate enhancement issue covers the CLI running repository rules.
- Trust model: "Workspace = build input (Recommended)".
- Gates: "Port gates to Rust (Recommended)".
- Parser overflow: "Fail-closed, honest (Recommended)".
- Exit gate: `reviews/completion-audit-stage2-exit.md` pass 3 PASS (CI half of AC1 externally blocked).

## Evidence
- `validation.md` is the index.
- `gap-analysis.md`.
- `reviews/*`.
- `tasks/G-R*-review.md`, one per remediation group, all PASS.
- Final full `make check`: `raw/make-check-80c14d7.log`, rc 0 on 80c14d7.
- Hosted CI: BLOCKED by organisation billing; never executed.

Defect-Class: none
