# Completion audit: raf-workplan change lifecycle + cargo-workplan CLI

Branch `paudley/8-raf-workplan-change-lifecycle-cargo`, HEAD `a72c5f5`, compared with
`origin/main`. Every experiment ran in a detached copy (`git worktree add /tmp/c8 HEAD`,
`CARGO_TARGET_DIR=/tmp/c8-target`, `CARGO_BUILD_JOBS=6`), removed at the end. Nothing in the
review worktree was staged, stashed, reset or edited apart from this file. No make target ran.

## How the entry point was driven

`cargo build --locked -p cargo-workplan` built `/tmp/c8-target/debug/cargo-workplan`. I made a
scratch repository `/tmp/e2e` in the fixture style: package `demo` with `add`, `real`, a
passing lib test, `tests/it.rs`, and the examples `good` (exits 0) and `exiting` (exits 3).
It has the capabilities `alpha` and `shape`, both delivered, both with generated blocks. The
index was created by `docs sync --copyright/--licence`. I committed it on `main`, set
`origin/main` and `origin/HEAD`, and branched `feature`. Every result below is the output and
exit code of the binary. Refusal fixtures were one copy of the clean state per fault, each
with a sha256 snapshot of the tree taken before and after.

## Requirements (R1-R24)

| R | Verdict | Evidence (binary output / exit) |
| --- | --- | --- |
| R1 `change new` | PASS (owner reading) | `change new reshape --capability cap:shape --affects cap:shape#contract --affects cap:shape#usage --copyright .. --licence ..` exit 0. It writes one file, `work/changes/reshape.md`, with SPDX lines, `contract_effect: changes` and `items: []`. It **prints** the PROPOSED template, the state guidance (`delivered` -> set `implementing`) and the `git add` then `docs sync` steps. Following the guidance gave `check` exit 0. Placing the printed block then gave exactly one `unresolved-proposal` line. The command refuses an existing file, `Bad_Slug`, `cap:nope`, `#nonsense` and a missing `--capability`, each with exit 2 and no file written. |
| R18 affects named at planning | PASS | The `affects` values were written in order. `-q change new tidy` without `--affects` wrote the default `cap:<slug>#contract` (also covered by `tests/new.rs`). `impact` reports those sections. |
| R2 `change impact` | PASS | It prints the capabilities, sections, the bindings with `resolves`, `cap:alpha depends on cap:shape` and `cap:alpha#usage links to cap:shape#contract`. Exit 0. The sha256 tree snapshot was identical before and after. |
| R3/R19/R23 obligation list, evidence | PASS | Each line carries the subject, part, kind, `line n:` and the evidence needed. Two runs were byte-identical. Exit 1 when the list is non-empty and 0 when it is empty. |
| R4 unresolved-proposal | PASS | `cap:shape contract unresolved-proposal line 26: the block is replaced by plain contract text, or removed because the claim was dropped` |
| R5 item-without-outcome | PASS | `change:reshape item 1 item-without-outcome an outcome of implemented, dropped or moved:change:<slug>`. With `moved:change:nowhere` the line adds "names no change record". |
| R6 binding-missing | PASS | `cap:shape verification binding-missing ... bindings.code \`demo::missing\` does not resolve` |
| R7 section-untouched (git CLI) | PASS | `cap:shape usage section-untouched a diff of that section against the merge base ...` (usage left as at the merge base). The edited contract is not reported. Without `origin/HEAD` the result is exit 2, naming `git remote set-head origin --auto`. |
| R8 generated-stale | PASS | `cap:shape verification generated-stale line 51: a docs sync run (the \`verification-matrix\` block is stale)` and the index `catalogue` line. A sync clears both. |
| R9 tense-mismatch | PASS | `cap:shape contract tense-mismatch line 29: ... (the text says "will")`. A PROPOSED block in a delivered capability is also reported as tense-mismatch. |
| seventh kind example-failing | PASS | `cap:shape examples example-failing ... (\`demo::example_crate::exiting\` fails to build or run (exit code 3))` |
| R10 `complete` five conditions + deletion | PASS | With all seven faults present: `refused: 7 obligations, 1 item without outcome, 1 unresolved PROPOSED block, 2 stale generated blocks, 2 findings once the record is removed`, exit 1, record kept. One copy per fault, each exit 1, with the tree sha256 unchanged and the record present: C1 tense, C1 section-untouched, C1 binding-missing, C2 no outcome, C2 moved-to-nowhere, C3 PROPOSED block, C4 stale matrix (an unsynced binding, and a hand-edited row), C5 failing bound test (`refused: 1 bound test failing`, with the `cargo test --frozen --package=demo --lib -- --exact -- tests::passes` argv and its counts), C5 failing example (shown under C1 and C5), and a capability left `implementing` (`no-open-record` once the record is removed). `contract_effect: none` gave reconcile exit 0 with no Usage/Examples edits, where the same record with `changes` lists two `section-untouched` lines. It still refused an item with no outcome (exit 1) and completed once that was fixed (exit 0, record gone). |
| R15/R20/R21 happy path, file gone | PASS | `completed: removed work/changes/reshape.md` / `rewrote docs/capabilities/shape.md`, exit 0. The record is gone. Afterwards `check` and `docs sync` exit 0 with nothing written, and `status` lists 0 open changes. The only writes were the Changed-by rewrite and the removal. |
| R11 `verify [<slug>]` | PASS | `verify` (all) and `verify prev` exit 0 and print one line per binding with proof counts. A failing test gives a `fail ...` line and exit 1. A snapshot showed nothing written. Over the repository copy, `verify` gave 595 pass and 3 fail, all the known bindings of other capabilities (see risks). |
| R12 `docs preview` | PASS | It prints the Contract with the PROPOSED markers removed and the proposed text present, exit 0. The tree sha256 was unchanged. |
| R13 binary, clap derive, 0/1/2 | PASS | clap 4.6.7 `derive` (`cli.rs` uses `Parser`/`Subcommand`). Every command in the issue is present. Exit 0 for clean runs. Exit 1 for findings (reconcile, complete refusal, failing verify). Exit 2 for: missing slug, a slug `-x`, an unknown subcommand, an unknown record, an existing record, no workspace, a bad `--root`, and a Git 2.29 stub (below). |
| R13 Git 2.29 stub | PASS | A PATH stub answering `git version 2.29.2` gave exit 2 for `change reconcile` and `change complete`, with `--version` as the only git call logged. The same stub at 2.30.0 proceeds (exit 1 with the obligation). `check` is not gated (exit 0), as documented. |
| R13 `-q` / `--format json` | PASS | `-q change reconcile` keeps the obligation lines (exit 1). `-q docs sync` and `-q status` print nothing (exit 0), as the cli Contract states. JSON: `verify`, `change reconcile` and `change impact` each print one array with `kind` first and `null` where a field does not apply. |
| R13 both argv shapes | PASS | `cargo workplan <cmd>` (real cargo, the binary on PATH), `cargo-workplan workplan <cmd>` and `cargo-workplan <cmd>` gave byte-identical output and the same exit for `check`, `status`, `change impact`, `change reconcile` and `docs preview`. |
| R14 fixture per kind + complete refuses | PASS | Driven above. In the suite: `obligations.rs` and `untouched.rs` per kind, `complete.rs::each_kind_of_obligation_makes_complete_refuse_...` and `the_cases_cover_every_kind_of_obligation_in_both_directions`. |
| R16 CLI integration tests | PASS (see REQUIRED 2) | `crates/cargo-workplan/tests/*.rs` (13 files) run `CARGO_BIN_EXE_cargo-workplan`. The full `cargo test --workspace --locked` in the copy exited 0 with 0 failed. |
| R17 capability docs reconciled | PASS | In the repository copy, `cargo-workplan check` exits 0 and `docs sync` writes nothing (`git status` clean). The three records were completed with the shipped binary (validation.md). Their deletion is in `92e4de8`. |
| R22 three capabilities delivered | PASS | `state: delivered` for workplan-reconcile, workplan-complete-gate and workplan-cli, each with resolving bindings (82/80/49 entries) and a generated verification matrix. `status`: delivered 7. |
| R24 builds on the merged model | PASS | It uses `Gate`, `Workplan`, `sync`, `preview` and `Roadmap` from origin/main. |

## Repository rules

* **Tracker numbers, process wording, attribution**: none. I grepped every added line of
  `git diff origin/main..HEAD` (36101 lines) for tracker references, issue or PR numbers,
  stage, task and reviewer wording, deferral words, TODO/FIXME and agent names; the only hit
  was the false positive "A cursor over the bytes". All 17 commit messages are clean.
* **`.deficiencies`**: holds only its notice and marker.
* **Delivered documents state built behaviour**: I grepped the three documents for forward
  words (will, planned, future, not yet, later and the like). The only hits are the
  tense-phrase list itself, "would rewrite nothing" (a condition), "every later command"
  (ordering), "evidence that would settle it" and the `status` table row "planned,
  implementing with gaps, delivered". That row is pre-existing and loose: the output says
  designed and considered (advisory). The T11 R1 defect ("the first/second command") is
  fixed in `a72c5f5`.
* **CI**: `actionlint .github/workflows/ci.yaml` exits 0. The `bound::` steps are quoted,
  which fixes T3 R1.
* **Contract-edit table**: all 10 rows are present in the documents. The complete-gate
  rows are writes, order, Bounds (1800 s and 300 s including the build), never-edits-prose,
  16 MiB and the validators. The reconcile rows are touched-ignores-generated, `--affects`
  with the template and guidance, `--show-prefix` and 16 MiB. The cli rows are
  `--copyright/--licence`, JSON, the cargo-subcommand form and the complete-writes
  Constraint. The cli does not restate 16 MiB, but it spawns only `cargo metadata`
  (64 MiB, stated).
  Edits **outside the table** are listed under REQUIRED 3. None removes a guarantee that
  the code does not keep, but the plan forbids weakening a Constraint outside the table
  without putting it to the owner.

## Plan tasks: artifact and commit

| Task | Commit | Review | Status |
| --- | --- | --- | --- |
| 1 exec | fefadb2 | T1 PASS | ok |
| 2 git, Gate::open | 8c5eeba | T2 PASS | ok |
| 3 bound executors | b6a367a | T3 **BLOCKED** (R1 yaml, R2 Windows `\`) | fixes in tree (ci.yaml quoted, `parse.rs:129` + `a_doc_list_entry_with_the_platform_separator_...`); no re-review recorded |
| 4 new/impact | 4c97695 | T4 PASS | ok |
| 5 static obligations | 80c303e | T5 PASS | ok |
| 6 untouched/verify | 3f6cc3b | T6 PASS | ok |
| 7 complete | c7791e5, a83a1a0 | T7 **BLOCKED** (R1 dangling-link fixture) | fix in tree (`prose_that_links_to_the_record_refuses_...`, complete.rs:1667); no re-review recorded |
| 8 crate, check/status/docs | c0685a4 | T8 PASS | ok |
| 9 lifecycle CLI | 0668be1 | T9 **BLOCKED** (R1 `git add` guidance) | fix in tree (seen in my e2e output); no re-review recorded |
| 10 scenarios | 6708afb | T10 **BLOCKED** (R1 global git ignore) | fixed in `raf-workplan/tests/support/checkout.rs` only; **the same defect remains in two other helpers** (REQUIRED 2) |
| 11 reconcile and deliver | e42ff32, 0e999a1, ed5ec73, 41eba82, 92e4de8, a72c5f5 | T11 **BLOCKED** (R1 first/second command) | fixed in a72c5f5; no re-review recorded |

## Findings

### REQUIRED

1. **Five task reviews end `VERDICT: BLOCKED` with no recorded re-review** (T3, T7, T9, T10,
   T11). I confirmed that each listed REQUIRED fix is in the tree, except as noted in item 2.
   But the plan's per-task artifact, a review with PASS, does not exist for these tasks.
   Append a re-review with its evidence and `VERDICT: PASS` to each file once item 2 is done.
2. **The T10 R1 defect class is still present in two git fixture helpers.** The helpers set
   `GIT_CONFIG_GLOBAL=/dev/null`, but git's default `$XDG_CONFIG_HOME/git/ignore` stays in
   force. With `XDG_CONFIG_HOME` pointing at a `git/ignore` that holds `*.md` and `work/`:
   * `cargo test -p cargo-workplan`: `tests/complete.rs` fails 10 of 11 tests.
   * `cargo test -p raf-workplan --no-fail-fast`: `tests/git.rs` fails 16 of 22. Every other
     raf-workplan binary passes, because it uses the fixed `checkout.rs`.

   Fix: add `"-c", "core.excludesFile="` and the one-line comment to
   `crates/cargo-workplan/tests/support/mod.rs` (`fn git`, line 458) and to
   `crates/raf-workplan/tests/git.rs` (line 67), matching `support/checkout.rs:56`. Then
   re-run both packages under the hostile `XDG_CONFIG_HOME`.
3. **Document edits outside the Contract-edit table must be put to the owner before the PR
   is created** (plan lines 24 and 242; T11 A1-A3).
   * The reconcile Constraint "the list applies outside code fences and PROPOSED blocks"
     now also excludes code spans, HTML comments, link destinations and generated blocks,
     and does not count `PROPOSED` in capitals. Code spans come from D6. The rest narrow a
     Constraint.
   * The complete-gate relative-source-path pattern now reads `\` as `/` in rustdoc list
     output, which widens a validator, although it only reads output.
   * "reconcile and impact never write files" became "never write a file in the
     repository".
   * `change new --copyright/--licence` has no row in the table.

   Add these rows to the plan's table and list them in the PR body as owner readings
   needing acceptance. Do not create the PR on the implementer's own judgement.

### ADVISORY

* The cli `status` row says "planned, implementing with gaps, delivered", while the output
  prints delivered, implementing, designed, considered and open changes. The row predates
  this branch, but it is now a delivered statement.
* AGENTS.md: "the records of this repository are completed with it" is a rule, while 14
  records stay open. That is fine as a rule; D2 says dogfooding the other records is not
  claimed.
* Under `-q`, `change new` prints nothing, not even the `git add` and `docs sync` steps.
  This is documented, and JSON `sync_needed` carries it (T9 A3).
* validation.md "Gates after the last commit" names `92e4de8`, but HEAD is `a72c5f5` (a
  doc-only change). The gate run on HEAD is in progress in the worktree. Record its result.

## Residual risks and what a PR reviewer should know

* **`change new` prints the PROPOSED template and does not write it into the capability.**
  The issue's "scaffolds ... a PROPOSED block" is read this way because the CLI Constraint
  fixes a one-file write (D8). The PR body must state this reading.
* **All three capabilities are delivered here**, with Scenario A and Scenario B built as
  fixtures (`raf-workplan/tests/scenarios.rs`) (D2). Open issue 9 ("Acceptance scenarios A
  & B + workplan self-gate") overlaps with this. The reviewer should know the scenario part
  is built here and only the self-gate remains there.
* **The rustix dependency is an exception**: `raf-workplan` gains `rustix` (unix, `process`
  + `std`, the version raf-source uses) for the process-group SIGKILL. It is recorded in the
  CHANGELOG. Windows has no group kill (T3 A2).
* **The `docs_sync` example changed behaviour.** It now loads through `Gate::open`. A root
  inside a git work tree without its own `.git` entry now lists the tracked paths. A root
  with no capability document is now exit 2 instead of a sync with nothing to write. Plan D4
  called this behaviour-preserving. The CHANGELOG states the change, and no generated-docs
  statement contradicts it. `make docs-sync` now runs the binary.
* **Failing verify bindings are tracked separately** in open issue 17. They are the
  `cap:source-index` `cfg(windows)` tests, not listed on Linux, and the `no_run` doc test of
  `raf_workplan::Gate` bound as an example of `cap:workplan-document-model`. Because of
  them, a no-slug `verify` exits 1 on Linux and macOS (reproduced: 595 pass, 3 fail).
* **Cross-check evidence comes from hosted CI**, by the owner's decision "Hosted CI counts
  (Recommended)". CI runs only on `pull_request`/`push` to main, so there is no hosted run
  of this branch yet. The macOS and Windows jobs and the backslash handling of T3 R2 have
  never run. Merge only after the hosted cross-check, macOS and Windows jobs are green, and
  quote the decision in the PR body.
* **Known accepted limits**: the TOCTOU window between the second judgement and the rename
  is documented as sound only because the workspace is trusted input. Result-shaped output
  from trusted test code is read as reported. The post-removal `check` refuses on any
  finding in the repository, not only in the listed capabilities.
* **Cost**: `complete` runs one `cargo` invocation per binding (E7 declined). The three
  dogfood completions took 40-102 s.

VERDICT: BLOCKED

REQUIRED fixes:
1. Append a re-review with evidence and `VERDICT: PASS` to T3, T7, T9, T10 and T11.
2. Add `-c core.excludesFile=` to `crates/cargo-workplan/tests/support/mod.rs` `fn git` and
   to `crates/raf-workplan/tests/git.rs` (line 67). Re-run both packages under a hostile
   `XDG_CONFIG_HOME/git/ignore`.
3. Put the out-of-table document edits to the owner before PR creation: the tense-scope
   narrowing, the `\`-to-`/` list-output normalisation, the "in the repository" write
   scope, and the `change new --copyright/--licence` row. Add them to the plan's
   Contract-edit table and the PR body's owner readings.
