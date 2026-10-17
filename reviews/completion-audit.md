# Completion audit: issue 10 (rest of the planning essay), HEAD a51977f

## Scope and method

* Source reviewed: branch `paudley/10-m2-rest-of-the-planning-essay-entity` at `a51977f` against
  `origin/main` (37 commits, 204 files). Plan: `plan.md` revision 4 (completeness contract R1-R13,
  D1-D19, carried items). Task reviews `tasks/T1..T10-review.md`. Task 11 evidence:
  `validation.md` sections 11A, 11B, 11C, 11D1-11D3, 11E, 11F1-11F3.
* Execution surface: a detached worktree `/tmp/c10` at `a51977f`, with
  `CARGO_TARGET_DIR=/tmp/c10-target` and `CARGO_BUILD_JOBS=6`. The binary was built once
  (`cargo build -p cargo-workplan`). Mutating experiments ran in an independent clone
  `/tmp/c10x`. That clone had its own `.git` and its own `origin/HEAD`, and its commits were
  throwaway commits there only. All scratch copies were removed afterwards. Nothing in the
  worktree was modified, staged, stashed or reset. No toolchain change was made.
* Not run: hosted CI. No PR exists yet, and `ci.yaml` triggers only on `pull_request` and on
  `main` pushes. Also not run: a whole-repository `verify` with no slug (about 1400 bound
  tests), and `make check` as one chain. The validation records the gates as run one at a
  time. I re-ran `make issue-refs`, `make reuse` and `docs_contract` myself.

## Criteria

| # | Requirement | Demonstration (real entry point) | Observation | Status |
|---|---|---|---|---|
| R6 | Each capability reaches `delivered` through `change complete` | `git show <commit> -- docs/capabilities/<slug>.md`; validation 11F1-11F3; `cargo-workplan status` | Each of the 12 delivery commits flips `state: implementing -> delivered` in the same commit that deletes the record. Validation shows `change reconcile` empty (exit 0) and then `completed: removed work/changes/<slug>.md` (exit 0) for all 12, one at a time. `status`: delivered 12, implementing 0, considered 0; open changes = the 9 census records only | demonstrated |
| R7 | Record deleted on completion; history in Git | `git log --diff-filter=D -- work/changes/<slug>.md` for all 12 | One deletion commit for each: ef62049, a8bd498, 6a132df, 0b88c66, e22ac7d, ba34ce9, 11ce02d, d561881, fcd69d1, 7b8ae50, 68a3342, 023df72. Each has 4 files (the capability document, the deleted record, the index, the INVENTORY test). Author is the user; no trailer. The five records existed on `origin/main` and the five capabilities were `considered` there (R13 holds) | demonstrated |
| R8 | No deferrals; anything leaving is `moved:` | each deleted record read at `<commit>^` | Every item of all 12 records is `outcome: implemented`. None is moved, none dropped. Rewordings of items are listed in validation 11C | demonstrated |
| R9 | Lifecycle run with the tool | validation 11F1-11F3 commands | `change reconcile` and then `change complete` through the built `cargo-workplan` | demonstrated |
| R1 | Entity history (`--follow`, `-L`) | `history cap:workplan-reconcile` (exit 0, 20 entries, document and symbol sources); `history arch:normal-form` (exit 0, 3 commits); `--since 2026-10-15/16` narrows; bad date, `-x`, `--x` and an unknown id each exit 2 before git; JSON `history-entry`; shallow clone exit 2 with the stated message; removed binding (scratch: add fn + bind, then delete fn) reports `history-removed`, the removing commit and its parent as last resolved; a function binding lists the commit that edited it | Works for item bindings. **Findings F1 and F2 below** | **failing (F1)** |
| R2 | Evidence and claims | `evidence record -- /bin/sh -c 'echo peak=41'` with `--limit peak=50`: exit 0, the record holds command, values, limits, input digests, `by`, `for`. Limit 40: exit 1, `fail`. `peak=1e5`: exit 2, nothing written. Exit 3: `fail`, exit 1. Program that cannot start: exit 2, no record. Never-printed limit, twice-printed name, non-number, 1025 values, duplicate limit, bad `--revision`, `--copyright` alone, unknown `--from-binding`, capability with no code binding, replace without `--update`: each exit 2, nothing written. `-q` hides pass and keeps fail. `evidence claims` and `evidence for` (text and JSON) work. Scratch claim + base commit as merge base: `--update` to `peak=80` gives `claim-unsupported` x2 and `evidence-changed` x2, naming value and limit. Removed record gives `claim-unsupported` + `evidence-changed`, and `check` gives `claim-evidence-missing`. A comment inside bound `evidence_for` gives `evidence-stale` for the fn and the module | as stated | demonstrated |
| R3 | Prose assessment | `assess ...#usage --context` (section, bound symbols, evidence; the output is byte-identical with `--by human` and `--by agent`). `--verdict rejected --by human` gives `replaced:`, `--list` shows `rejected human current`, reconcile shows `assessment-rejected`. Editing the section gives `--list` `stale: the section ... has changed since`. Re-accepting makes it current. Deleting the record gives `assessment-missing`. Bound-symbol edit and evidence-input changes give `assessment-stale` naming the input. `change complete` refuses (exit 1, record kept, no cargo run) with "3 sections without a current accepted assessment" | as stated | demonstrated |
| R4 | Two-way coverage | `coverage code` (whole workspace) exit 1: 177 raf-okf + 35 raf-source gaps. `coverage tests` exit 1: 138 raf-okf, 216 raf-source, 85 root-package gaps. `--crate raf-workplan` and `--crate cargo-workplan`, both directions: exit 0, no gap. `--crate nope`: exit 2. `--crate raf_okf` maps to the package. JSON `coverage-gap` with `in_scope`. `check` on HEAD: exit 0. Scratch: an unbound `pub fn` gives `coverage-item-unbound`; an unbound `cfg(test)` test gives `coverage-test-unbound`; no record gives `coverage-record-missing`; an empty reason gives `okf-empty-reason`; `raf_workplan::*` gives `okf-crate-root-exclusion`; an unmatched exclusion gives `coverage-exclusion-unmatched` | as stated | demonstrated |
| R5 | Current rationale | Scratch `check`: removing `## Conditions to revisit` from normal-form gives `decision-conditions-missing`; an empty list gives `decision-conditions-missing` (no list item); an unlinked new page gives `architecture-page-unlinked`; a link only inside a code span does not count; an Explanation `### Decision` without `### Conditions to revisit` gives `explanation-conditions-missing`. On HEAD both architecture pages have `## Decision` and conditions, and `check` is clean | as stated (a setext "Decision" is not a decision page, as the Contract says) | demonstrated |
| reconcile 13 kinds | `ObligationKind::ALL` = the 13-row Contract table | Seen live in scratch: unresolved-proposal, item-without-outcome, binding-missing, section-untouched, generated-stale, tense-mismatch, claim-unsupported, evidence-stale, evidence-changed, assessment-missing, assessment-stale, assessment-rejected (12/13). `example-failing` is an earlier kind, not changed here; only the validation records it | 12/13 live, 1 by existing tests |
| (3) | Contract true; no forward-looking text | A scratch change lists all 12 capabilities, then `change reconcile` | 0 `tense-mismatch` lines. 31 sampled Contract claims (argv rows, exit codes 0/1/2, bounds 1024 values / 1e-3 refusal / commit-id pattern / date range, finding kinds, JSON kinds, `-q`, `--update`) match behaviour, except F1 (module binding history) and F2 (`--since` date basis) | partly failing |
| (4) | No PROPOSED, no Open questions | `grep -rn 'PROPOSED change:' docs`; `grep -i 'open questions'` over the five | The only hits are fenced examples (`example-slug`, `incremental-parsing`, both inside ```` ```markdown ```` fences, the second already present on main) and a code-span syntax mention. No Open questions | demonstrated |
| (5) | Owner decisions | `work/coverage/default.md`; `evidence record` runs; CI | `scope: [raf-workplan, cargo-workplan]`, `exclusions: []`, `check` clean, reports cover the whole workspace. `evidence record` RUNS the argv after `--`. Hosted CI has not run (no PR); it must be green before merge | demonstrated / CI pending (Task 12) |
| (6) | Hygiene | `make issue-refs` (57 passed), `make reuse` (482/482 compliant), `docs_contract` (28 passed); diff and commit-message grep for tracker numbers, process wording and attribution; `.deficiencies` (worktree and root) | All clean. Both `.deficiencies` files hold only the notice and marker. `.stage` is ignored. No dependency change (`Cargo.lock` untouched; `rustix` was already a raf-workplan dependency). No `#![feature]`, no new `unsafe` | demonstrated |
| (7) | Tasks 1-11 reviewed and committed | `tail` of T1..T10 reviews; `git log` | T1..T10 each end `VERDICT: PASS`. One commit for each of tasks 1-10 (6e75ac7 .. 21499bc). Task 11 = 3bd9e56 .. a51977f, with validation sections 11A-11F3. The roadmap guard is restored, and a negative check is recorded | demonstrated |

## Open findings

**F1 (blocking): the history of a file-module binding is the history of its `mod` line only.**
`raf_workplan::history` runs `git log -L` over `item.span` (`history.rs` around line 675). For a
module in its own file, that span is the declaration `pub mod x;` in the parent file. Meanwhile
`digest.rs` (lines 93-99, `module_files`) and coverage treat a module binding as standing for the
whole content of its files. Demonstrated: `history cap:workplan-current-rationale`, whose only
code binding is `raf_workplan::rationale`, lists one symbol commit, c54db19, with path
`crates/raf-workplan/src/lib.rs` and not `rationale.rs`. d6f8804, which edited `rationale.rs`, is
listed as `document` only. In scratch, a commit that edited the body of `evidence.rs` was listed
for the `Workplan::evidence_for` binding and not for the `raf_workplan::evidence` binding. 16
code bindings are file modules. They include the main code binding of each of the five
capabilities this issue delivers (`history`, `evidence`, `assessment`, `coverage`, `rationale`).
So "the history of a capability and its bindings" is close to empty for the code of exactly
these capabilities (VACUOUS OUTPUT / SILENT NARROWING of R1). The entity-history Contract says
"the report lists the commits that changed that symbol". It says nothing about modules, and the
rest of the system defines a module binding as its files.

REQUIRED fix: make the history of a module binding cover the files the digest's
`module_files` reads for it, through the same validated path forms, with `--follow` or a
whole-file `-L`. Add a fixture: a commit editing a file module's body is listed for the module
binding, and an edit of a sibling module is not. State this in the entity-history Contract.
Because `cap:workplan-entity-history` is now `delivered`, carry the fix through the tool:
`change new`, then `implementing`, then the PROPOSED block, then reconcile, assess, complete.
If the owner instead chooses to state the current behaviour as a limit, that is a narrowing:
the Contract must say it explicitly, and the module-content history must be `moved:` to a
tracked change. It cannot be reworded away silently.

**F2 (required, small): `--since` filters on the committer date while entries show and order
by the author date.** Git's `--since` uses the committer date. The Contract (and the `--help`
text "Only the commits from 00:00:00 UTC of this day on") says entries "hold ... its date" and
are "ordered by the instant of their author date", and is silent on which date `--since`
reads. Demonstrated in scratch: a commit whose author date was set to 2026-01-01 was listed
under `--since 2026-10-15` with the date 2026-01-01. Rebased commits hit this routinely.
REQUIRED: either state in the Contract, the help text and the Constraints that `--since`
selects by committer date, or filter by author date. Add the fixture with a commit whose two
dates differ. This rides with the F1 change.

Advisory, not blocking:

* Text dates are shown in the author's offset, so a commit made at 00:53 UTC on 10-17 shows as
  2026-10-16 under `--since 2026-10-17`. This is consistent with the Contract, but worth one
  sentence there.
* A closed pipe (`| head`) makes `cargo-workplan` exit 2 with "cannot write the report: Broken
  pipe".

## Scope notes and owner readings for the PR body

* Rewordings of delivered and proposed sentences, item rewordings and the "removed: nothing"
  statements are itemised in validation 11A (okf-profile 5, source-index 3,
  document-model 6, generated-docs 0), 11B (reconcile 4+, complete-gate 6, cli 6), 11C (five
  capabilities, each with its item rewordings), the 11D text-fix lists (fixes 1-7 for 11D1,
  and the per-section fix tables of 11D2/11D3, 38 fixes in 11D3) and 11E. The PR body must
  carry these lists.
* Owner readings to list:
  * `evidence record` RUNS the argv after `--` through `Program::Trusted`: no shell, stdin
    closed, 1800 s, 16 MiB.
  * `DocType`, `RafDocument`, `Rule::ALL` and `ErrorKind::ALL` growth is not source-compatible
    for exhaustive matches. CHANGELOG states it.
  * No new dependency. `rustix` already existed.
  * A module binding hashes its whole module. Any edit there, comments included, stales the
    evidence and assessments that depend on it; demonstrated with a comment.
  * Condition 6 was added to `change complete`, and `contract_effect: none` does not suppress
    the assessment kinds (reconcile Contract line 404 ff.; complete-gate line 319).
  * The `--since YYYY-MM-DD` form counts from 00:00:00 UTC and is placed before the final `--`
    (subject to F2).
  * Coverage: `check` enforces scope = the two workplan crates with `exclusions: []`, and the
    reports cover the whole workspace.
  * All 54 assessments are `by: agent`. D8 lets an agent assessment satisfy the gate, and no
    human assessed any section.
* Reopened and re-delivered: okf-profile, source-index, workplan-document-model,
  workplan-generated-docs (reopened in Task 7, not in the original D16 list), workplan-reconcile,
  workplan-complete-gate and workplan-cli. That is 7 records, plus the 5 considered
  capabilities, for 12 deliveries.
* Issue 17: this branch fixes its three failing bindings: the two `#[cfg(windows)]` tests bound
  in `cap:source-index`, now unbound with Linux siblings bound and a Constraints bullet, and the
  `no_run` `raf_workplan::Gate` doc test in `cap:workplan-document-model`, now runnable. Issue
  17's other scope (verify exit 0 on macOS/Windows, the skip-rule decision) is not done here.
  The PR must not say it closes 17.

## Residual risks for the PR reviewer

* Hosted CI, including the Windows and cross-check jobs, has not run on this branch. The
  standing decision makes it the cross-check evidence, so it must be green before merge.
* Module-granularity digests make dependent assessments and evidence brittle: any edit in a
  bound module, including test code, reopens them.
* The removed-binding lookup is a `git log -S` heuristic. Its limits are stated in the
  Constraints.
* Setext headings are invisible to the rationale rules, as stated.
* Long completion runs (up to 367 s for each record) and the whole-repository `verify` was not
  run.

VERDICT: BLOCKED

REQUIRED fixes:
1. F1: make the history of a file-module binding cover the files of the module (the set
   `digest::module_files` hashes). Add a fixture with a negative neighbour and update the
   entity-history Contract. Carry it through `change new` / reconcile / assess / `change
   complete` on `cap:workplan-entity-history`, or, only on an explicit owner decision, state
   the limit and `moved:` the module-content history to a tracked change.
2. F2: make `--since` and the displayed/ordering date agree (filter by author date), or state
   in the Contract, the help text and the Constraints that `--since` selects by committer date.
   Add a fixture with differing author and committer dates. Same change as fix 1.
3. After 1-2: re-run `change reconcile` (empty), re-assess the moved sections, run
   `change complete`, and run `make issue-refs`, `make reuse` and `check`. Then open the PR
   with the lists above, and get hosted CI green before merge.
