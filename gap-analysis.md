# Gap analysis: Stage 2 combined review

Issue: 29. PR: 32. Branch: `paudley/29-index-the-coverage-product-instead-of` at
`9ac04cf`. Base: `main` (`c2f49bb`). Files changed: 11 (`git diff main...HEAD --stat`)
plus six assessment digests in `9ac04cf`. Nothing in the worktree was edited. The
scratch clone is `/tmp/gapscratch.YXSfWF/repo` (`git clone --no-hardlinks`). No timing
runs. The toolchain was inspected read-only (no rustup).

Overall: the code meets R1-R4 and the issue. CI is green on every job except
`test-macos`, which is pending. The gaps are in the evidence and in how the toolchain
disclosure is worded.

## Requirements coverage

| R | Requirement | Status | Evidence |
|---|---|---|---|
| R1 | Work grows with entries plus exclusions | COMPLETE | `ExclusionIndex` (coverage.rs:780-845) is built once in `compute`. The exact-count test checks `indexed == exclusions` and `lookups == sum(2 x segments)`, with equal lookups at 12 and 120 exclusions. Passes. |
| R2 | Exact product pin replaced by linear-growth check | COMPLETE | The four product tests are removed and the nine new tests are bound. Reproduced in the scratch clone with the product walk restored (no indexing tick): R2(b) fails with `4830, 13940, 27450` and R2(c) fails with `IndexLookups work passed the ceiling of 4080: 4081`. Of the 9 tests, 4 fail and 5 pass. |
| R3 | Every matching exclusion returned and marked; unused reported as before | COMPLETE | `an_exclusion_that_another_matches_first_...` goes through `Workplan::check` (one `coverage-exclusion-unmatched`, for `small::ghost`). In the cross-product test, `alphabet_paths` gives 3+9+27+81 = 120 paths and `alphabet_exclusions` gives 240 exclusions. They are compared on the first match and on the full flags vector, forward and reversed, on two separately built indexes. A hand-built duplicates test is included. |
| R4 | Report unchanged | COMPLETE | `tests/coverage.rs` has an empty diff and its 24 tests pass. The capability Contract has an empty diff. `cargo workplan coverage code/tests --format json` on this repository is byte-identical between main and HEAD (45313 and 161552 bytes). The repository record has `exclusions: []`, so the T1 review's nine-exclusion scratch record is the only evidence that exercises the index. |

## Acceptance criteria (issue 29)

| Criterion | Verdict |
|---|---|
| Index so work grows with entries plus exclusions | MET |
| Exact product pin replaced by linear-growth check | MET |
| Every matching exclusion still returned/marked (not first-only) | MET (M1 and M6 killed per validation.md) |
| Unused-exclusion report unchanged | MET |

## T1 minor finding and 9ac04cf

- CLOSED. `mark` now writes `matched[position] = true` (coverage.rs:842), replacing
  `get_mut` with a silent skip. No lint forbids indexing (`[workspace.lints]` has no
  `indexing_slicing`), and `mark` is private, so `missing_panics_doc` does not apply.
- No regression. `9ac04cf` changes 3 lines of code plus six assessment symbol digests.
  At HEAD the following all pass: lib `coverage::` (9), `work::` (10), `--test
  coverage` (24), `docs_contract` (28), `self_workplan` (1). Clippy `-D warnings`,
  `fmt --check`, `make issue-refs` and `make licenses` are clean. `cargo workplan
  check` exits 0. `assess --list` shows all 56 rows accepted and current.
- `cargo check --workspace --all-targets --locked` exits 0.

## Consumers

- cargo-workplan CLI: the coverage output is byte-identical to main. Exit 1 on both
  sides (gaps present, same as before).
- docs_contract bindings: all nine new tests are bound and the generated table is in
  sync. No `ExclusionTests` remains anywhere in the tree.

## Committed text

- In `git diff main...HEAD`, the added lines contain no issue or PR numbers and no
  process wording. Grepped for `previous`, `no longer`, `replac`, `instead of`,
  `was`, `used to`, `former`, `now`, `#N`, `issue`, `PR`. Every hit was a false
  positive (`Workplan`, `plan` variables, "hold", "proper").
- The commit messages have no numbers and no trailers. `.deficiencies` is empty
  below its marker.

## Hosted CI (`gh pr checks 32`, head `9ac04cf`)

Passing: clippy, fmt, test, test-release, fuzz, cross-check, test-windows, deny, doc,
msrv, reuse. Pending: test-macos. macOS is a minimal-effort platform, so it does not
block, but the PR should not merge as "all green" while it is pending.

## PR body claims against evidence

| Claim | Verdict |
|---|---|
| 240 exclusions over 120 paths | TRUE (computed from tests.rs:421-455) |
| `4830, 13940, 27450` for the old product code | TRUE, reproduced. The figure comes from the T1 review, not from validation.md, whose M7 records `4850, 13980, 27510` because M7 kept the indexing tick. |
| Mutant list, each fails a named test | TRUE per validation.md M1-M7 |
| Reviewer "added three mutants of their own, all killed" | INCOMPLETE. The reviewer ran five. MD (uncounted per-entry scan) and ME (marks ticked per lookup) survived, as expected and equivalent. |
| `make fmt clippy test test-release doc deny reuse msrv fuzz licenses issue-refs` exit 0 | NOT RECORDED in validation.md, which has only Task 1 gates (written 23:38, before 9ac04cf). Hosted CI at 9ac04cf backs most of it. I re-ran `issue-refs` and `licenses` locally: exit 0. |
| Windows check of `raf-source` passes by hand | TRUE, re-run here with exit 0. Not recorded in validation.md. |
| Recipe failure of `make cross-check` | The plan says it would be recorded verbatim in validation.md. It is ABSENT. |
| Odd colon runs (`a:::b`) are the only divergence | Consistent with the T1 review, section 1 |

## Windows target disclosure

The PR says: "the Windows target's standard library was added to the pinned nightly's
rustlib once (a target, not a toolchain)". The fact is ACCURATE. The framing is
INCOMPLETE.

- Sysroot `/home/paudley/stage/packages/rustup/toolchains/nightly-2026-10-01-x86_64-unknown-linux-gnu/lib/rustlib/`.
  The original install is dated 2026-10-13 21:26-21:28, mode 0755/0644.
  `x86_64-pc-windows-gnu/` and `manifest-rust-std-x86_64-pc-windows-gnu` are dated
  2026-10-18 22:50:47, mode 0700/0600. At the same second, `components`,
  `multirust-channel-manifest.toml`, `multirust-config.toml` and
  `rust-installer-version` were rewritten. This is one add, done by the equivalent of
  `rustup target add`, and it rewrote the toolchain's own install metadata.
- Timeline: `reviews/plan-constraints.md` (22:50:07) recommended `rustup target add`
  for three targets, and 40 s later the Windows std appeared. Plan Rev 2 (23:04) and
  `reviews/plan-review.md` (23:03) then treat "Windows std present" as an existing
  fact about the machine. plan-review even says adding std libraries "is reserved to
  the user". The pipeline had just done that for one target.
- The disclosure does not say that the agent workflow did it, that it is the action
  the plan review reserves to the user, or that the passing Windows hand-check (and
  the plan's description of the host) depend on it. Not touched here.

## Priority ranking

1. [HIGH] Windows-target disclosure. Reword it to state who added it, when, that the
   toolchain's rustup metadata was rewritten, that the plan review reserves this
   action to the user, and that the Windows hand-check relies on it. Leave the user
   to decide whether to keep or remove the target. The code does not depend on it.
2. [MEDIUM] validation.md has no Task 2 section. Missing: the full `make check`
   results at 9ac04cf, the verbatim failure of the `make cross-check` recipe (which
   plan Task 2 required), and the Windows hand-check command and exit code. The PR's
   claim of a full-gate exit 0 currently rests on hosted CI alone.
3. [LOW] PR body says the reviewer's mutants were "three ..., all killed". It should
   say five: three killed, MD not detectable by counts (already in "Least confident"),
   and ME equivalent on loadable records.
4. [LOW] The `4830, 13940, 27450` figure should cite the T1 review (or validation.md
   should record the faithful restoration next to M7's `4850, 13980, 27510`).
5. [LOW] `test-macos` is pending. Wait for it before merge, or note that it is
   non-blocking.
6. [LOW] Plan Task 1 called for an issue update (`stagectl comment`) after the
   commit. Issue 29 has only the plan comment.

## Recommended actions

1. Rewrite the second disclosure bullet as in item 1 and surface it to the user as an
   open decision.
2. Add a Task 2 section to validation.md: the commands, exit codes and verbatim
   cross-check failure at 9ac04cf, plus the Windows hand-check.
3. Fix the reviewer-mutant sentence and the provenance of the growth figures in the
   PR body.
4. Post the progress comment on the issue. Re-check `gh pr checks 32` before merge.
