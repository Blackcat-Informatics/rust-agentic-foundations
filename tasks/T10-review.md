# T10 review: Scenario A and Scenario B fixtures

Scope: the staged diff on top of 0668be1 (crates/raf-workplan/tests/scenarios.rs and
tests/support/{project,story,streaming,streaming_code,window,window_code}.rs, mod.rs),
read against plan.md (D2, D9, Task 10), T7-review.md, AGENTS.md and the Verification text
of docs/capabilities/workplan-complete-gate.md. Experiments ran in a scratch worktree with
the staged diff applied; it is removed and the staged tree is untouched.

## Evidence

Mutation results (each mutation applied, the scenario tests run, then reverted):

| Sentence / property | Mutation | Result |
| --- | --- | --- |
| A: no PROPOSED block on the reading path | block appended to the final document | FAILS (gate findings `proposed-missing-change`, `state-delivered-proposal`) |
| A: every link resolves | `[x](./nope.md)` appended | FAILS (`link-broken`) |
| A: no change file on the reading path | tracked `work/changes/zz.md` | FAILS (`stray-markdown`) |
| A: Explanation holds the rationale | decision heading and reason text altered | FAILS (`the Explanation lacks ...`) |
| A: reader need not reconstruct | superseded wording kept in final document | FAILS (`assert_superseded_wording_gone`) |
| A: chain words | "previously" appended | FAILS (chain scan) |
| A/B: complete succeeds with a leftover obligation | `Failure::Owed` dropped in `complete` | 3 of 4 tests FAIL |
| B: complete ignores PROPOSED blocks | condition 3 dropped | `planning_produces_the_documentation` FAILS (A not affected: its blocks are removed before `complete`) |
| B: post-deletion `check` ignored | `AfterRemoval` dropped | 2 tests FAIL |
| B: stub / thin end state | constraints, reference, usage code, matrix row, explanation reason, tense phrase each broken in the finished document | each yields exactly its own `Account::faults` entry and the test FAILS |

Lifecycle path: A and B use `NewChangeRequest`/`new_change`, `Lifecycle::reconcile`, `verify`
and `complete`, real `git` commits (each commit asserts the staged tree passes `check`),
a real cargo crate built per version, four redesigns whose code, tests, examples, bindings
and prose are five distinct texts each (asserted), a reversed decision (skip became error),
an amendment inside the last change (4096 to 65536) and a rename (`push`/`finish` to
`feed`/`close`). `landed` states are checked with `git show` and by gating a checkout of each.
B pins the refusal sets at five stages exactly; `state: delivered` is set by the author and
`state implementing` yields exactly `after-removal:no-open-record`. Negative variants: empty
sections (not loaded, error, nothing written), example binding to the old API, failing example,
thin document.

Runs: scenarios.rs debug x5: 16.0, 10.6, 8.9, 18.0, 26.6 s; release x3: 15.7, 25.5, 28.4 s; all
pass; no scratch directories left in the temp dir. Full `cargo test -p raf-workplan` debug and
release pass. `cargo clippy --workspace --all-targets -D warnings`, `cargo doc -Dwarnings`,
`cargo fmt --check`, docs_contract + repo_gates, `make reuse`, `make deny`, `make issue-refs`
all pass. Hygiene grep: no sleeps, tracker numbers, process wording, shell, `cfg(unix)` or
Unix paths in the new files; SPDX present; target dir is `CARGO_TARGET_TMPDIR`.

Hostile environment: with `HOME` and `XDG_CONFIG_HOME` pointing at a config that sets
`init.defaultBranch`, `commit.gpgsign`, `core.hooksPath`, `autocrlf`, `useConfigOnly`,
`LANG`/`LC_ALL` odd, and `GIT_CONFIG_GLOBAL` pointing at it, all 4 tests pass.

## Findings

REQUIRED

R1. The fixtures depend on the user's global git ignore file. `support::checkout::git`
(used by `Project::git`) sets `GIT_CONFIG_GLOBAL=/dev/null`, which removes `core.excludesFile`
but leaves git's default `$XDG_CONFIG_HOME/git/ignore` (or `~/.config/git/ignore`) in force.
With a global ignore of `*.md`, `work/`, `*.rs` there, `git add -A` skips the fixture files and
all four tests fail at the first commit (reproduced). Fix: add `-c core.excludesFile=` (empty)
to the fixture git invocation; I verified that with it all four tests pass under the same
hostile `XDG_CONFIG_HOME`. The helper is shared (`checkout.rs`), so the fix covers earlier
fixtures too. Add a one-line comment that the user's global ignore is neutralised.

ADVISORY

A1. Scenario B vs `examples: []`. The Contract (State paragraph, Constraints) requires a
delivered capability to bind "at least one code path and one test", and the empty-lists rule
covers only the fifth condition for `designed`/`considered`; nothing in the Contract or in
Scenario B's text says `complete` must refuse a delivered capability with tests but no
examples or a thin document ("A document that merely exists does not satisfy the scenario" is
a statement about what the fixture asserts, and the fixture does assert it through
`Account::faults`). I confirmed `complete` accepts the thin document with `examples: []`.
That is not a REQUIRED defect. The thin-document test does not pin the acceptance, so the gap
is silent; add one assertion that `complete` completes it (or a comment) so a later tightening
is a visible, deliberate change.

A2. `Project::show` (and `tracked_at` via `git()`) mixes isolated and plain `git`: `show`
runs `git` without removing `GIT_DIR`/`GIT_INDEX_FILE`, so the test would read the wrong
repository if it were ever run from inside a git hook. Route it through `git()` (it already
strips a trailing newline only; `show` wants the exact bytes, so add a raw variant).

A3. Some `Account::faults` guards have no negative fixture inside the suite (`candidate-text`
and `tense` appear only through the start-state check and the gate; `matrix`, `limitations`,
`api-reference`, `usage-runs`, `explanation` are exercised negatively only by the thin
document, which trips several at once). My single-fault mutations show each does fire, so
nothing is vacuous; a table-driven test that breaks the finished document one fault at a time
would keep it so.

A4. The Scenario A links/reading-path asserts are partly redundant with `Gate::check`, which
fires first (a PROPOSED block or broken link is reported by the gate before the explicit loops
run). Harmless; the explicit loops catch a gate regression.

A5. The chain-word list omits "was", "renamed from", "instead of"; the superseded-wording table
covers the concrete cases, so this is only a note.

A6. `core.hooksPath=/dev/null` and `GIT_CONFIG_GLOBAL=/dev/null` in `checkout.rs` are
Unix-spelled; Git for Windows maps `/dev/null`, so they work there, but `NUL` or an empty
value would be explicit. Not new in this task.

VERDICT: BLOCKED

## Re-review

Re-checked at HEAD 9f1ab69 in a scratch worktree (removed afterwards).

| Finding | Fix location | Evidence | Status |
| --- | --- | --- | --- |
| R1 user global git ignore breaks fixtures | `crates/raf-workplan/tests/support/checkout.rs:41-62` (`-c core.excludesFile=` plus HOME, USERPROFILE, XDG_CONFIG_HOME pointed at an empty directory, comment states the global ignore is neutralised); same in `crates/cargo-workplan/tests/support/mod.rs:468` and `crates/raf-workplan/tests/git.rs:77`; commit 9f1ab69 | With a hostile `$XDG_CONFIG_HOME/git/ignore` (`*.md`, `work/`, `*.rs`): HEAD passes all 32 `complete` tests; mutant with all hardening removed fails 10 of 32. Removing only the `-c core.excludesFile=` pair still passes because the empty XDG/HOME already neutralise the file, so that argument is belt and braces and the env isolation is what the fixtures rely on | resolved |

VERDICT: PASS
