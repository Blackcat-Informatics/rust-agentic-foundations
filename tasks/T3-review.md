# T3 review: gates (Makefile, pre-commit hook, CI)

## Scope
Makefile, .githooks/pre-commit, .github/workflows/ci.yaml, REUSE.toml edit, staged deletion of INITIAL_PLANNING_PLAN.md (all uncommitted). Contract: plan Task 3, A2, A3, A14, A15/adv.6, adv.9, adv.12, issue-refs row, Execution adjustment; checks R5, R6, R7, R36.

## Evidence (command -> rc)
- `make -n check` -> 0; order fmt clippy test doc deny reuse msrv licenses issue-refs.
- `make fmt licenses issue-refs reuse` -> 0.
- `make reuse REUSE=nonexist-tool` -> 2, names tool and alternatives. `make msrv MSRV_RUN=nonexist` -> 2, names tool.
- `uvx reuse@6.2.0 lint` -> 0 (14/14 files) with the REUSE.toml annotation removed.
- `shellcheck .githooks/pre-commit` -> 0. `actionlint` -> 0.
- `git config --get core.hooksPath` -> `.githooks`.
- No `--no-verify` or bypass text in Makefile, hook or CI (grep).
- issue-refs self-match: `git grep --no-index` of both pattern sets over Makefile, hook, ci.yaml -> rc 1 (no hits) for all.
- Pattern behaviour via `make issue-refs` in a /tmp clone with the temp file tracked: `Per #12`, `/issues/3`, `PR #4`, `CHANGE-17`, `this issue`, `Task 3` -> rc 2 each; `#111214`, `#fbbc05` -> rc 0. Clean tree -> rc 0.
- licenses in the /tmp clone: clean rc 0; appending to docs/LICENSING.zh-Hans.md -> rc 2 naming file and hashes; altering LICENSES/MulanPSL-2.0.txt -> rc 2.
- Hook run directly in /tmp clone (snapshot via checkout-index, ROOT swap, `git grep --cached`): clean -> 0; staged-bad fmt / working-good -> 1 ("fmt failed"); staged-good / working-bad -> 0; staged `Per #12` with working file removed -> 1 (issue-refs); untracked-only bad working file -> 0. The index, not the working tree, is read.
- CI jobs present: fmt, clippy, test, doc, deny (`cargo deny --locked check`, fetching), msrv (env RUSTUP_TOOLCHAIN 1.98.1, toolchain 1.98.1, `rustc --version | grep '^rustc 1\.98\.1 '`), reuse (setup-uv + `uvx reuse@6.2.0 lint`), gates (`make licenses issue-refs`). permissions contents: read; persist-credentials false.
- Action SHAs resolve via `gh api`: actions/checkout 9c091bb... == tag v7.0.0; astral-sh/setup-uv bec219d... == tag v10.1.0; dtolnay/rust-toolchain 29eef33... resolves but is NOT tagged v1 (see R1).
- SPDX headers present on all three files.

## Findings
### REQUIRED
- R1. ci.yaml pins `dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8 # v1` (6 uses). That commit is the tip of the action's `stable` branch ("toolchain: stable", 2026-03-27), not tag `v1` (tag v1 = e2a55d2ffb04f378e9626c28d38b36d230d1e12f; the two commits have diverged). The version comment is false, which defeats auditing and automated pin updates. Fix: either pin e2a55d2ffb04f378e9626c28d38b36d230d1e12f `# v1`, or keep the SHA and change the comment to the real ref (`# stable branch, 2026-03-27`). Every use passes `toolchain:` explicitly, so behaviour is unaffected either way.

### ADVISORY
- A1. The `make msrv` failure text suggests `MSRV_RUN=` plus `RUSTUP_TOOLCHAIN=1.98.1`; on this machine a PreToolUse hook blocks that override. Also `make msrv MSRV_RUN=` (empty) passes the `command -v` check and silently runs `cargo check` on nightly (verified rc 0, not an MSRV check). Consider a guard or wording that makes the empty override CI-only.
- A2. `make issue-refs` and the hook's `--cached` search only cover tracked/staged files; a brand-new untracked file is unchecked until staged (by design, verified). Makefile, hook and ci.yaml were untracked during this review, so self-non-matching was checked with `--no-index`.
- A3. The hook has no tool-missing preflight of its own, but the reused `reuse` target hard-fails, so there is no silent skip.
- A4. The `deny` CI job builds cargo-deny from source (`cargo install --locked`); slow but acceptable.

Initial verdict (superseded by the Recheck below): BLOCKED (one REQUIRED finding, R1: mislabelled dtolnay/rust-toolchain pin; all functional checks pass)

## Recheck
- R1: all 6 dtolnay/rust-toolchain pins now carry `# stable branch, 2026-03-27` (accurate: the SHA is the stable-branch tip). The one remaining `# v1` match is `setup-uv # v10.1.0`. Resolved.
- `actionlint` -> 0.
- `make msrv` -> 0 (runs `rust-validate 1.98.1 cargo check ...`).
- Advisory A1 fixed: `make msrv MSRV_RUN=` -> 2 (got nightly, version assertion fails); `MSRV_RUN=nonexist` -> 2 (tool missing); `MSRV_RUN='rust-validate 1.97.1'` -> 2 (expected 1.98.1, got 1.97.1). Cosmetic leftover: the missing-tool message still suggests a toolchain env override that the local machine hook blocks (usable in CI only).
- `make fmt licenses issue-refs reuse` -> 0; the Makefile still does not match its own issue-refs patterns.

VERDICT: PASS
