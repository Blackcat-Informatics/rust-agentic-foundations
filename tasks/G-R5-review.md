# G-R5 review: project-document consistency

## Scope
Unstaged diff of CHANGELOG.md, AGENTS.md, CONTRIBUTING.md and .github/PULL_REQUEST_TEMPLATE.md against remediation-plan R5, gaps G15b/c/d and audit A1.

## Evidence
1. CHANGELOG claims checked against the tree. Pins and a concurrency group with cancel-in-progress on pull requests are in ci.yaml (SHA-pinned actions, cargo-deny 0.20.2, uv 0.13.0, reuse 6.2.0, MSRV assertion). .github/dependabot.yml covers github-actions. .githooks/pre-merge-commit execs pre-commit. tests/repo_gates.rs (40 tests) and tests/docs_contract.rs (112 tests) exist and share tests/support. SECURITY.md has the Trust model, containment, bounds and subprocess rules. 21 files are in docs/capabilities, 2 in docs/architecture, and the essay file is absent. README has the crate table and dev-dependency snippet; BRAND.md, the logo and the social preview exist. The cliff dedup is commit 8ff3859. No claim is false and no user-relevant item is missing.
2. No issue numbers or process wording found on reading. repo_gates passes.
3. AGENTS.md and CONTRIBUTING.md match the Makefile targets, the two hooks and the CI jobs (fmt, clippy, test, doc, deny, msrv, reuse; the old gates job is gone and neither doc mentions it).
4. AGENTS.md section 6 now agrees with docs/architecture/okf-profile.md. The document-contract test enforces today; the crates take over later; the inventory is pinned in INVENTORY. The okf-profile statement that symbol resolution is not checked does not conflict.
5. The PR template uses the two-line HTML-comment SPDX shape. An awk-style check over all Markdown without frontmatter found no deviations; the frontmatter files carry YAML-comment SPDX lines as AGENTS.md section 4 says. `reuse lint` is compliant (81/81).
6. With CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target: `cargo test --workspace --locked` passes (112 + 40 tests, 0 failed). `uvx reuse@6.2.0 lint` is compliant. `git cliff --unreleased` renders each of the 17 commits once.
   R5 Check greps: brand/logo/README, capabilit, document contract and docs_contract, repo_gates/repository gates, and essay all hit under [Unreleased]. `grep -n docs_contract AGENTS.md` returns at least 2 lines.

## Findings
Required: none.
Advisory:
- CONTRIBUTING.md and AGENTS.md `make hooks` comment says "install the pre-commit hook"; it also installs the pre-merge-commit hook. Cosmetic.
- The "Planning essay" bullet describes a removal, which is mildly history-flavoured. The plan asked for it and the gates accept it.

VERDICT: PASS
