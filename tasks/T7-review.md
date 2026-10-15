# T7 review: `complete` (D9)

Scope: the staged diff on top of 3f6cc3b (`lifecycle/complete.rs`, `lifecycle/completion.rs`,
`staged.rs`, `Opened::workplan_after`, `LifecycleError::{Sync,Rewrite,Remove}`, `tests/complete.rs`,
CHANGELOG, doc edits in `lib.rs` / `lifecycle.rs`). Checked against plan D9, Task 7, R10/R14/R15/R20/R21,
the Contract-edit table, AGENTS.md, SECURITY.md and the complete-gate and reconcile documents.
Experiments ran in a throwaway worktree with the staged diff applied (removed afterwards); the review
worktree was not modified.

## Evidence

Gates (all green): `cargo test -p raf-workplan` (debug and `--release`), `cargo test --workspace --locked`,
`make fmt`, `make clippy` (workspace, -D warnings), `make doc` (-Dwarnings), `docs_contract` (26) and
`repo_gates` (57), `make reuse` (361/361), `make deny`, `make issue-refs`, `.deficiencies` holds only its notice.
`tests/complete.rs`: 26 tests, five consecutive runs 2.6 to 3.0 s each, no flake; the read-only-directory
tests really ran (uid 1000, not skipped).

(1) Conditions. Each of the five has a refusal fixture plus passing neighbour; the loop over
`ObligationKind::ALL` is guarded both ways (`the_cases_cover_every_kind...`) and asserts record and tree
byte-identical and the repair completing. My own attempts to make `complete` succeed wrongly, all refused or
errored: a listed capability left `designed`, `considered` or `implementing` with no other record
(`no-open-record`); `delivered` beside a second open record (`state-delivered-open-record`);
`implementing` plus a second record with empty example list (`binding-missing`); PROPOSED block of the change
in an unlisted loaded capability (condition 3) and in an unloaded document (tested); stale block in an
unlisted capability (tested); a listed `cap:ghost` (error, record kept); bad `outcome` values
(`moved:change:`, wrong case, trailing space, `done`) (error, record kept); a doc-test binding to an
undocumented function (`binding-missing` plus `binding-wrong-role`); a prose link to the record from another
capability (refused, see A1); a record that is a symlink (link removed, target kept).

(2) Post-deletion model. Sound as a mechanism: `Staged` hides the record from read, read_unseen, list and
entry, shows the rewritten text, and `workplan_after` drops the record from the tracked set, so a dangling
link to it is a `link-broken` finding. The sync is computed on the model without the record and the check is
run on the model with its edits, so blocks the removal stales are not a fault and blocks that were already
stale are caught earlier by condition 4 (judged on the pre-removal model), so the rewrite cannot hide a
pre-existing stale block. Findings are deduplicated only against identical condition-4 findings or the
condition-3 proposal at the same file and line; nothing else is dropped. Compared with D9: D9 says "no
`check` finding on the record or a listed capability", the code fails on every finding in the repository.
Verified: a broken link in an unrelated `designed` capability refuses completion with one
`once the record is removed, ...[link-broken]` line. This is stricter than D9 but cannot let a bad
completion through and cannot make a legitimate one impossible in a repository that passes its own gate
(`docs_contract` runs `Gate` on every commit); what it does is make an unrelated defect, including a broken
second record, block this change. Judged ADVISORY (A2).

(3) Write order. `Workplan::write` stages every file first (any staging error discards all temporaries,
nothing renamed), then renames; a failing rename returns before `remove_file`. Verified by the read-only
`docs/capabilities` fixture (record present, nothing written, neighbour completes) and by the read-only
`work/changes` fixture (`LifecycleError::Remove` names the rewritten files, record left). Mutant: swap to
remove-then-rewrite is killed by both. A partial rename across several files leaves the record in place;
re-running `docs sync` restores the rows and `complete` again finishes, which is what the error text says.
A record or document changed between the second evaluation and the write is not detected; inherent, no
compare-and-swap exists in `Workplan::write` (A3).

(4) TOCTOU. `Opened` freezes the source index but reads documents live through `DiskTree`. Mutating the
re-check to reuse `self.opened` is killed only by the renamed-test fixture, which is the case the claim
needs: the second `evaluate` re-reads documents either way, and the re-open re-reads source. Removing the
second evaluation, the binding-set comparison, and the "after.failures" refusal are each killed. The
"partial survival" the implementer reports is the document-edit fixture, which passes under both openings
because documents are not frozen; the CHANGELOG and docs claim ("the repository is opened again ... because
the documents and the source can change") is accurate. The run-key comparison covers (capability, kind,
path) only, so a binding that stays resolvable but moves to another target is not noticed (A4, minor).

(5) Static refusal spends no process: recorded-command fixture shows only `git --version` and
`git rev-parse --show-prefix` for an item-without-outcome refusal and for hostile bindings; with
`contract_effect: changes` read-only `git` calls for the touched check also run, which is not `cargo`.

(6) Display. `refused: 2 obligations, 1 item without outcome, 1 stale generated block` and
`refused: 1 obligation, 1 bound example failing` plus the obligation line are asserted equal in shape to
the two Contract examples; `completed: removed <path>` is followed by one `rewrote <path>` line per file,
which the planned Contract edit covers.

(7) Happy path: snapshot diff equals exactly the capability file and the record; `git status` shows only
those two; `Gate::workspace(root).assert_clean()` and an empty `sync` afterwards.

(8) Mutants (own, 13 on `complete.rs`, 4 on `staged.rs`/`gate.rs`): no static gate, remove-first order, no
re-check, same-`Opened` re-check, no proposals, check without the rewritten text, no run-key comparison,
no after-removal findings, no item condition, no generated condition, dropped obligations, ignored run
failures, rewrite dropped, ignored second-pass failures, staged `entry`/`read` not hiding: all killed.
Survivors: staged `list` no-retain did not compile (not a mutant); filtering the record out of the tracked
paths in `workplan_after` survives, 26 of 26 pass (A1).

(10) Hygiene: SPDX headers present on the four new files, REUSE clean, no tracker numbers or process
wording (`make issue-refs`), no capability document touched, no `#![feature]`, no dependency added.

## Findings

REQUIRED

R1. No refusal fixture guards the dangling-link half of the post-deletion model. In `Opened::workplan_after`
the line `paths.iter().filter(|path| *path != hidden)` can be removed and all 26 tests plus the unit tests
still pass, while a capability whose prose links to `work/changes/<slug>.md` then completes and leaves a
broken link (I reproduced both: refused with the filter, completed without). The behaviour is right; the
rule has no negative fixture, which the repository requires of every rule. Add a scripted fixture to
`tests/complete.rs`: a second capability whose prose links `../../work/changes/shape.md`; expect a refusal
with an `AfterRemoval` `link-broken` finding, record and tree unchanged, and the neighbour (link removed)
completing.

ADVISORY

A1. (same fixture as R1 covers it.)

A2. The post-deletion check fails on every `check` finding in the repository, where D9 says the record or a
listed capability. Keep the stricter reading (it is fail-hard and sound) but say so in the Task 11 edit of the
complete-gate document and in the `complete` rustdoc ("any finding, including in capabilities the change
does not list"), so the stated contract and the behaviour agree.

A3. The write window between the second evaluation and the rename is not closed (no compare of the text
`Workplan::write` replaces); acceptable for a trusted workspace, worth one sentence in SECURITY.md or the
rustdoc of `complete`, which currently says only that the repository is judged "immediately before" writing.

A4. An `outcome` that is not one of the three values makes the record not load, so `complete` returns
`LifecycleError::Change(NotLoaded)` (an error, exit 2 in the CLI) rather than a condition 2 refusal (exit 1)
as the Contract's "every item has an outcome of implemented, dropped or moved" suggests. This matches how
`reconcile` treats an unloaded record; decide the exit mapping in Task 9 and state it in the document.
The run-key comparison (see 4) does not include the resolved target; minor.

VERDICT: BLOCKED

## Re-review

Re-checked at HEAD 9f1ab69 in a scratch worktree (removed afterwards).

| Finding | Fix location | Evidence | Status |
| --- | --- | --- | --- |
| R1 no negative fixture for after-removal link-broken | `crates/raf-workplan/tests/complete.rs:1667-1740`, test `prose_that_links_to_the_record_refuses_because_the_link_is_broken_once_it_is_removed` (refusal with one `AfterRemoval` `link-broken` finding on the capability, record and tree snapshot unchanged, then the neighbour with the link removed completes) | passes at HEAD; mutant removing `.filter(|path| *path != hidden)` at `crates/raf-workplan/src/gate.rs:400` makes this test FAIL | resolved |

VERDICT: PASS
