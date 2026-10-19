# Plan completeness review

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/27-raf-source-the-registry-scan-example/.stage/raf-source-the-registry-scan-example/plan.md`

Inputs read in full: `issue.md` (body, 0 comments), `issue-analysis.md` (R1-R6, 5 NEEDS-JUDGMENT),
`prior-art-assessment.md` (3 NEEDS-JUDGMENT), `plan.md`. Source checked:
`crates/raf-source/examples/registry_scan.rs`, `tests/repo_gates.rs` (`clock_rule`,
`SOURCE_CLOCK_WORDS`, `time_in_crates_tracked_files_name_no_clock`,
`time_in_crates_exempts_the_examples_and_paths_outside_every_rule`), `crates/raf-source/Cargo.toml`,
`crates/raf-source/src/options.rs` (`LoadOptions`: byte and token budgets only, no step budget,
which confirms the plan's claim), `Makefile` (`clippy` is `--workspace --all-targets`, `test` is
`cargo test --workspace`), `AGENTS.md` L152-168, `CONTRIBUTING.md` L90-97, `CHANGELOG.md`.

## Requirement → task → check

| R | Requirement (source) | Contract row | Task | Check | Status |
|---|---|---|---|---|---|
| R1 | Replace the per-crate timeout and the sleep poll "with a bound on counted work", or remove the example if the differential test serves its purpose (body L11) | Reworded to "a bound that is not a clock" | 1 | Checks (1)-(5) of Task 1 | **SILENT CUT / substitution** (F1). The plan does neither option the issue offers and replaces the timer with a kernel CPU-time limit. The cpu-limit outcome is shown only on fabricated statuses and a hand-written shell (F3) |
| R2 | Extend the gate to cover `examples/` once the example is clean, or say in the gate why it stays exempt (body L12) | R2 | 2 | `cargo test --test repo_gates` with no carve-out; a planted `Instant` fails in a scratch clone | MAPPED, falsifiable |
| R3 | `grep -nE '...' crates/raf-source/examples` is empty, or the example is gone (body L15) | R3 | 1 | "the grep above" | **UNFALSIFIABLE** (F2). Without `-r`, grep on a directory prints nothing to stdout, so the check passes on today's tree |
| R4 | The repository gate passes with `examples/` in scope (body L16) | R4 | 2 | `cargo test --locked --test repo_gates`, `make clippy` without the allow | MAPPED, falsifiable |
| R5 | "time-based testing and wall-clock measurement are not allowed" (body L8) | **none** | (1) | none | **UNMAPPED** (F4) |
| R6 | "a repository gate keeps clock spellings out of the crate's source and tests" (body L8) | **none** | (2) | none | **UNMAPPED** (F4) |

NEEDS-JUDGMENT dispositions: issue-analysis 1-5 are all addressed (1 and 4 as "neither", see F1;
2, 3 and 5 as "R4 governs", which is sound). Prior-art 1-3 are all addressed (examples dir
exists, licence not in question, the related closed issue is precedent). Coverage of
dispositions is complete. Only the substance of 1/4 is a defect.

Structure: Task 1 onward, no Task 0, and the final task creates the PR and posts the plan. **No
task has a commit, push, independent task review or issue update** (F5).

## Findings

### F1 — BLOCKING — SILENT CUT: R1's requested bound is replaced by one the issue did not offer, with no user decision quoted

The issue offers two ways to meet R1: "a bound on counted work (the index already reports `Usage`
and refuses at its budgets)" or "remove the example if its purpose is served by the differential
test against rustc". The plan does neither and says so:

> "Considered and declined: removing the example ... a counted-work bound in the index ..."

It then rewrites the requirement in the contract so that its own substitute fits:

> "| R1 | Replace the per-crate timeout and the sleep poll with a bound that is not a clock | 1 | ..."

`ulimit -t` (RLIMIT_CPU) is a limit in CPU seconds. It is still a time bound, although not a wall
clock, and the plan itself declines the pidfd-plus-timer route as "still a clock". The declined
counted-work route rests on "settling cost without a budget is tracked separately as superlinear
glob settling", but the plan names no tracked item. Neither the brief's forge search nor the
inputs show one. The planner's reason for not removing the example is plausible: the oracle runs
generated programs, not real crates. Even so, R1 as written is not delivered, and the
substitution is backed by no quote from the user. NEEDS-JUDGMENT 1/4 needed a choice between the
issue's two options, not permission to invent a third.

REWRITE TO (contract row, R1):
`| R1 | "Replace the per-crate timeout and the sleep poll with a bound on counted work (the index already reports \`Usage\` and refuses at its budgets), or remove the example if its purpose is served by the differential test against rustc." | 1 | Task 1 checks (1)-(6) |`

REWRITE TO (NEEDS-JUDGMENT disposition 1, 4), choosing exactly one of the two:

- (a) If the user has approved the CPU-time limit:
  `1, 4: the user decided "<verbatim quote of the user's message approving a kernel CPU-time limit in place of a counted-work bound>"; the example stays and the kernel's CPU-time limit replaces the timer. The step budget the index lacks is tracked as <forge item URL/number recorded in the stage directory only>.`
- (b) If that approval is not obtained, deliver the issue's counted-work option:
  `1, 4: counted work. The child loads through \`Workspace::load_with\` with \`LoadOptions\` whose byte and token budgets come from \`--max-total-bytes\`, \`--max-glob-bytes\` and \`--max-file-tokens\` (defaults from \`LoadOptions::default()\`). A child the index refuses prints \`ERR Budget :: ...\`. The per-crate timer, the poll and the sleep are removed and \`run_child\` blocks on \`child.wait()\`. A load whose settling has no budget is bounded by a step budget added to the index in this change (\`Budget::SettleSteps\`, reported in \`Usage\` and printed as \`steps=N\` on the OK line), so no part of the load is unbounded.`

Until (a) carries a real quote, only (b) delivers R1.

### F2 — BLOCKING — UNFALSIFIABLE: the acceptance grep passes vacuously as written

Task 1 says: "Acceptance (executable): (1) the acceptance grep of the issue is empty". The
issue's command is `grep -nE '...' crates/raf-source/examples` with no `-r`. Grep run on a
directory writes "Is a directory" to stderr, prints nothing to stdout and exits 2, so stdout is
"empty" on today's unmodified tree, which still has `Instant` at L64, `Duration` at L72 and
`thread::sleep` at L572. The check can never fail.

REWRITE TO (Task 1 acceptance (1)):
`(1) \`grep -rnE 'std::time|Instant|Duration|sleep|recv_timeout|SystemTime' crates/raf-source/examples\` prints nothing and exits 1. Run it first on the unchanged tree and record in validation.md that it prints the hits at registry_scan.rs:64, :72, :75 and :572, so the recursive form is shown to bite before it is shown clean.`

### F3 — BLOCKING — TEST-ONLY: the `cpu-limit` outcome is never shown through the shipped example

The new behavior is the scan reporting `ERR cpu-limit` for a child the limit stopped. It is shown
only by:
- check (2): `classify` over "crafted statuses (`ExitStatusExt::from_raw`)", which is fabricated
  internal input;
- check (4): "run the child wrapper by hand (`sh -c 'ulimit -t 1; exec sh -c "while :; do :; done"'`)",
  which skips `run_child`, its real wrapper string (`ulimit -v ...; ulimit -t ...; exec "$0" "$@"`)
  and the example binary.

Check (3) only exercises the `OK` path. No check runs the example's `cargo run --example
registry_scan` entry point and sees a child stopped by the limit. If the wrapper drops or
misquotes `ulimit -t`, or composes it badly with `ulimit -v`, all five checks still pass.

REWRITE TO (add Task 1 acceptance (6)):
`(6) the shipped entry point stops a real child at the limit: \`cargo run --release -p raf-source --example registry_scan -- <scratch-registry> --cpu-limit 1 --jobs 1\`, where the scratch registry holds one copy of a real registry crate whose load takes many CPU seconds (the crate with the largest \`total=\` from a full scan of the local registry), prints \`<name>@<version> | ERR cpu-limit :: stopped after 1 CPU seconds\` and \`# err cpu-limit=1\`, and exits 0. Only the outcome line is read; no time is measured or asserted. The output is recorded in validation.md.`

### F4 — BLOCKING — UNMAPPED: R5 and R6 have no contract rows

The method requires every R-id from the ledger in the contract. R5 ("time-based testing and
wall-clock measurement are not allowed") and R6 ("a repository gate keeps clock spellings out of
the crate's source and tests") are missing. Task 2 edits `clock_rule`, which today routes
raf-source `src`/`tests` to `SOURCE_RULE`, so R6 needs a check that the existing coverage
survives the edit.

REWRITE TO (add contract rows):
`| R5 | "time-based testing and wall-clock measurement are not allowed" | 1 | Task 1 check (1) and the new example unit tests read no clock; \`cargo test --locked --test repo_gates\` passes with the example judged by the raf-source rule (which also refuses \`elapsed\`, \`wait_timeout\` and \`core::time\`) |`
`| R6 | "a repository gate keeps clock spellings out of the crate's source and tests" | 2 | the existing fixtures that report a planted \`Duration\`/\`elapsed\` in \`crates/raf-source/src/a.rs\` and a planted \`Instant\` in \`crates/raf-source/tests/a.rs\` still pass unchanged after the \`clock_rule\` edit; scratch-clone mutant: a planted \`Instant::now()\` in \`crates/raf-source/src/lib.rs\` fails \`cargo test --locked --test repo_gates\` |`

### F5 — BLOCKING — STRUCTURE: no per-task commit, push, independent review or issue update

Tasks 1 and 2 end at their acceptance lines. Neither has an independent task review, a commit,
a push or an issue update. Task 3 is the only point where anything leaves the worktree.

REWRITE TO (append to each of Task 1 and Task 2):
`Close-out: record the acceptance output in validation.md; independent task review of the diff against this task's acceptance; commit (no attribution trailers, no issue numbers or process wording in committed text); push the branch; post a progress comment on the issue naming the commit and the acceptance results.`

### F6 — ADVISORY — Task 1 check (3) does not name its command or crate

"the example run over a one-crate scratch registry copy of a small real crate prints the `OK`
line and exits 0" names neither the invocation nor the crate.

REWRITE TO:
`(3) \`cargo run --release -p raf-source --example registry_scan -- <scratch-registry>\`, with the scratch registry holding one copy of a small real registry crate (named in validation.md), prints \`<name>@<version> | OK items=...\`, \`# ok=1\`, and exits 0; the same command with \`--timeout 5\` exits 1 with \`unknown \`--timeout\`\`.`

### F7 — ADVISORY — the user-visible rename has no CHANGELOG entry

The `--timeout` flag and the `timeout` outcome kind are renamed, and the plan lists this as a
visible cost. `CHANGELOG.md` `[Unreleased]` already records the raf-source clock work (L39-41).

REWRITE TO (Task 2 Files):
`Files: \`tests/repo_gates.rs\`, \`clippy.toml\` unchanged, \`AGENTS.md\`, \`CONTRIBUTING.md\`, \`CHANGELOG.md\` (the \`raf-source\` entry: the registry scan example reads no clock, bounds a child by CPU time with \`--cpu-limit\`, reports \`cpu-limit\` in place of \`timeout\`, and the gate now judges the crate's examples).`

### F8 — ADVISORY — the lost wall-clock stop should be a stated residual in shipped text

"a child that blocks without using CPU is not stopped" is disclosed in the plan only. With a
blocking `child.wait()`, such a child hangs the whole scan.

REWRITE TO (Task 1, the module doc sentence):
`Update the module doc (usage line, the \`cpu-limit\` outcome kind, the account of the child start, and one sentence that a child which blocks without using CPU is not stopped and holds its job).`
Also add this as a residual in the Task 3 PR body.

VERDICT: NEEDS-REVISION
