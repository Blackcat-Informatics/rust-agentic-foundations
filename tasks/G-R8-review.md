# G-R8 review: CI and hook hardening

## Scope
Unstaged diff of .github/workflows/ci.yaml and Makefile, plus new .githooks/pre-merge-commit and .github/dependabot.yml. Spec: remediation-plan R8 and its Check list.

## Evidence
- `git ls-remote ... refs/heads/master` prints e2a55d2ffb04f378e9626c28d38b36d230d1e12f, which equals the pin.
- `gh api compare/83d06109b7...e2a55d2ffb --jq .status` prints `ahead`.
- All 6 rust-toolchain uses pass `toolchain:`. Five pass `nightly`, matching rust-toolchain.toml. The msrv job passes 1.98.1.
- action.yml at the SHA passes the input through an env var and selects the toolchain as the rustup default. It has no toolchain-file override handling. The msrv job still sets `RUSTUP_TOOLCHAIN: 1.98.1` at job level, which outranks rust-toolchain.toml, and still asserts `rustc --version | grep '^rustc 1\.98\.1 '`. The design is intact.
- uv 0.13.0 release exists. setup-uv at bec219d... declares both the `version` and `enable-cache` inputs.
- cargo-deny 0.20.2 is the crates.io latest (`cargo search`). `cargo deny --offline check` prints advisories ok, bans ok, licenses ok, sources ok.
- `actionlint` rc 0. `zizmor` v1.30.1 reports no findings (13 suppressed, offline mode).
- Concurrency: `group: ci-${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`. The expression is valid (actionlint) and cancels only PR runs.
- Scratch clone with `make hooks` (core.hooksPath=.githooks) and normal signing:
  - `git merge --no-ff` of a branch adding a staged `#12` was refused with "pre-commit: gates failed ... Not committing merge", so the hook ran on a no-conflict merge.
  - A merge of a branch with a header-bearing clean file succeeded and produced a 2-parent commit.
  - A first clean-merge attempt failed because my test file lacked an SPDX header. That was the reuse gate working correctly, and the retry passed.
- `yq .` on dependabot.yml: rc 0. It has version 2, github-actions, directory "/", weekly. The SPDX header is present. `uvx reuse@6.2.0 lint` is compliant (81/81).
- `.githooks/pre-merge-commit` is executable. It execs the sibling pre-commit with `set -euo pipefail`.
- `git hook run pre-commit` on the worktree: 40 tests passed, rc 0.
- Makefile: `$(ROOT)` is used at 2 places (fmt, reuse), both single-quoted. `make fmt ROOT="/tmp/.../a b"` on a copy at a path with a space: rc 0, and `make reuse` with the same ROOT is compliant. The `deny` comment accurately describes the CI version, the fetching variant and the offline local run.

## Findings
Required: none.

Advisory:
1. The pin comment reads `# master, 2026-10-12`, where the plan text said 2026-10-09. This is fine because the plan allows pinning the current head, and today is 2026-10-12.
2. The `deny` comment pins 0.20.2 only in prose, so CI and the Makefile can drift silently. This is minor, and it matches the plan's "recorded beside the Makefile comment".
3. The pre-merge-commit hook checks the staged snapshot, as pre-commit does. That is the intended design.

VERDICT: PASS
