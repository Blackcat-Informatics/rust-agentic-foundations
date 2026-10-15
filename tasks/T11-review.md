# T11 review: reconcile, dogfood and deliver

## Scope

Commits `6708afb..92e4de8` (e42ff32, 0e999a1, ed5ec73, 41eba82, 92e4de8). Before-contracts
from `git show 6708afb:docs/capabilities/workplan-{reconcile,complete-gate,cli}.md` and
`6708afb:work/changes/workplan-*.md`. These are identical to `main`: no commit in
`main..6708afb` touches them. Experiments ran in throwaway worktrees at HEAD and at 6708afb,
and both have been removed. No tracked file was modified.

## Evidence

* Commit order matches Task 11 steps 1-4. 0e999a1 changes only state and bindings in the
  prose. ed5ec73 adds the PROPOSED blocks. 41eba82 resolves them. 92e4de8 changes state,
  removes the records, rewrites the `Changed by` rows and updates the catalogue, and nothing
  else in docs/.
* `cargo run -p cargo-workplan -- --root . verify`:
  * HEAD: 595 pass and 3 fail, exit 1. All 171 bindings of the three capabilities pass
    (61+67+43). The failures are the two `cfg(windows)` tests of `cap:source-index` and the
    `no_run` doc test of `raf_workplan::Gate` in `cap:workplan-document-model`.
  * 6708afb: 422 pass and 5 fail. The same three, plus `Workplan::sync` and
    `Workplan::preview` of `cap:workplan-generated-docs`, whose item paths were reported as
    `model::Workplan<'_>::sync|preview`.
  * So the 3 failures are pre-existing, and this issue fixed 2 of them (e42ff32).
* Gates at HEAD all exit 0: `cargo test --test docs_contract --test repo_gates`,
  `make reuse` (386/386), `make issue-refs`, `make licenses`, `make docs-sync` twice (no
  change either time, tree clean), `cargo fmt --all --check`, clippy `-D warnings`,
  `RUSTDOCFLAGS=-Dwarnings cargo doc`, and `make deny`.
* The binary produces the same index as the old example. In a scratch worktree,
  `docs sync --copyright ... --licence ...` after deleting `docs/capabilities/index.md`
  recreated it byte-identical. Without `--copyright` it exits 2 and names the option.
* Tense scan, equivalent to reconcile. A scratch record listing all three capabilities gives
  only `generated-stale` lines (the record is unstaged) and no `tense-mismatch`. Positive
  control: inserting "will" into the Purpose gives one `tense-mismatch` line. A grep for the
  PHRASES list finds only `PROPOSED` in capitals or inside code spans.
* e42ff32 mutation tests:
  * Removing the `file_module` override kills `documentation_tests_follow_the_rules_in_edition_{2021,2024}`
    (`Wrapper<T>::again` is unresolved).
  * Removing the `outer.is_empty() ||` clause of `holds` survives the bound tests and the lib
    unit tests (153 pass).

### Claims checked against code or the binary (31)

| # | Claim | Result |
| --- | --- | --- |
| 1 | No command is exit 2 | yes |
| 2 | `--format yaml` is exit 2 | yes |
| 3 | A slug `-rf` (with or without `--`) is a usage error, exit 2 | yes |
| 4 | An unknown slug is exit 2 and names the open changes | yes |
| 5 | `status` exits 0 with gaps; `-q status` prints nothing | yes |
| 6 | `check` exits 0 with `[]` in JSON; options are accepted after the command | yes |
| 7 | JSON objects start with `kind`; the 11 non-rule kinds exist in `render`/`change` | yes |
| 8 | `change new` writes the frontmatter keys, the title/description placeholders, `affects` defaulting to `#contract`, the template `> [!PROPOSED change:<slug>]`, the delivered-state guidance and the stage-then-sync steps | yes |
| 9 | `change new` refuses an existing file, a bad slug, an unknown capability, an `--affects` with an unlisted capability or an unknown section, and `--copyright` without `--licence` | all refused |
| 10 | `impact` prints the capabilities, sections, bindings, dependants and linking sections | yes |
| 11 | The obligation line format, `line <n>:` and JSON `obligation` fields | yes |
| 12 | Stub git 2.29 on PATH: `reconcile` and `complete` exit 2 after exactly one `--version` call; `impact` is not gated | yes |
| 13 | `verify` with stub git 2.29: no version gate; git is asked only `rev-parse --is-inside-work-tree` and `ls-files -z` | yes |
| 14 | The `complete` refusal text `refused: 2 obligations, 1 item without outcome, 1 stale generated block` plus one line per non-repeated failure; JSON `refused`/`refusal` with `condition` and `repeated` | yes |
| 15 | A bad `outcome` makes the record fail to load, exit 2 | yes |
| 16 | `cargo workplan check` works through real cargo dispatch; the root is found from a subdirectory | yes |
| 17 | A JSON run that cannot run leaves stdout empty | yes |
| 18 | 16 MiB per stream (`DEFAULT_OUTPUT_BOUND`) and 64 MiB for metadata | yes |
| 19 | 1800 s build timeout, 300 s for an example, 300 s for a query | yes |
| 20 | The 300 s example limit covers the `cargo run` build (`Executor` uses `Limits::example` for the single spawn) | yes |
| 21 | The 1 MiB blob bound (`MAX_TEXT_BYTES` in `blob_at`) | yes |
| 22 | The ref-name validator: `refs/`, the stated characters, no empty segment, no leading or trailing `.`, no `.lock`, no `..`, at most 1024 bytes | yes |
| 23 | The commit id is 40 or 64 lowercase hex digits | yes |
| 24 | The source path has no empty, `.` or `..` segment, and `\` is read as `/` in list output (`parse.rs:129`) | yes |
| 25 | CRLF is read as LF in the section digest (`untouched.rs:123`) | yes |
| 26 | The test proof needs exactly one listed entry equal to the name and `1 passed; 0 failed; 0 ignored` (`run.rs`) | yes |
| 27 | The doc-test proof: list the package, select by item part, run with the bare item, `no_run` or `compile_fail` refuses, every selected entry must be `ok`, and a substring neighbour's failure fails the exit status | yes |
| 28 | The tense exclusions (code, HTML comments, code spans, link destinations, PROPOSED and generated blocks, `PROPOSED` in capitals) and the 8 phrases | match `tense.rs` |
| 29 | The reconcile git command list and order (`untouched::stubs` test) | yes |
| 30 | The example-failing evidence text `` `<binding>` fails to build or run (exit code N) `` | yes |
| 31 | The count nouns `bound example failing` and `stale generated block` | yes |

## (1) Before/after audit (Contract, Constraints, Verification)

Method: each document was split into sentences per section, and every before-sentence was
looked for in the after text. Each missing or changed sentence is classified below:
(a) an edit the table permits, (b) a factual correction backed by evidence, or (c) unjustified.

### workplan-complete-gate

* "deletes ... and exits with 0. The deletion is the last step and the only write" became
  "rewrites the stale generated blocks, then deletes as the very last step; those are its
  only writes". Class (a), table row 1.
* "never edits a capability document" became "never edits prose; rewrites only the
  generated blocks". Class (a), the D9/C9 row.
* The list-output source path gained "no empty, `.` or `..` segment". Class (a), the Task 1
  row. It also gained "each `\` is read as `/`". Class (b): it is built (`parse.rs:129`),
  and without it every doc test is refused on Windows. It widens what is accepted and is
  not in the table (ADVISORY A2).
* The doc-test paragraph gains an inserted sentence on rustdoc naming a method by the module
  of its impl block. The rest is verbatim. This is additive and built in e42ff32.
* The Examples paragraph gains "includes the build that `cargo run` performs". This is the
  carried item T3 A1, and it is true.
* Additions: Order, the repository once the record is removed, Bounds, two further
  validators, the profile-refused outcome being exit 2, condition 3 counting unloaded
  documents, `verify` doing no git gate, the proof not being a defence (T3 A3), and the
  TOCTOU limitation. All are table rows, D9 or additive, and none removes a guarantee.
* Verification: "no other change" became "no other change than the rewrite of the generated
  blocks the removal stales". Class (a), row 1. Every other before-bullet is verbatim, and
  new bullets are only added.
* "Proof that the test ran" and the "list pass shows exactly one entry" rule are unchanged
  and built (`run.rs` `test()`). The record item "A bound test counts as passed only when
  exactly one test ran" is `implemented`.
* The dropped record claim "Doc tests are listed and run by their full libtest name" is
  untrue as written. The before Contract already said the run uses the bare item path,
  because "a full name ... can never act as one exact filter". The list pass lists the whole
  package. Full names are used only to judge `test <name> ... ok`. Dropping it removes no
  behaviour that the Contract or the issue (`issue.md` has no such text) promised. Class (b).

### workplan-reconcile

* `change new`: the sentences are extended (title/description, "or its document did not
  load", a repeat listed once). This is additive. `--affects` and the printed template with
  state guidance are class (a), the E8 row. The `--copyright/--licence` options are carried
  item T9 A1, whose "Contract-edit row added when built" was never added to the plan table
  (ADVISORY A3).
* The obligation line format and sort order went from "capability, section, kind" to
  subject/part/kind/line/evidence, with the change record and the index as subjects. This is
  a refinement that keeps the same order for capabilities. Class (b), plan D6.
* Merge base:
  * "heading span differs" became authored text with generated blocks blanked. Class (a),
    E3.
  * "current branch" became "`HEAD`": equivalent.
  * CRLF as LF, "no such section there counts as touched", "does not parse is an error" and
    the default-branch case are additive or come from D7. All are built.
* The command list gained `--version`, `rev-parse --show-prefix`, `--is-inside-work-tree`
  and `ls-files -z`, and `<prefix>`. The `--show-prefix` part is class (a), the T6 R1 row.
  The others are a factual completion (`Gate::open`, D4). **The ordinal references that
  follow were not updated; see R1.**
* `example-failing` gained "output rules". This is additive (capture row).
* Constraints:
  * "`reconcile` and `impact` never write files" became "never write a file in the
    repository". Class (b): the before document's own Constraint already said
    `example-failing` "still writes no file in the repository", and plan R16 says the same.
  * "Only `change new` writes" gained "and it writes one file". Additive.
* The tense Constraint went from "applies outside code fences and PROPOSED blocks" to also
  excluding code spans, HTML comments, link destinations, generated blocks and `PROPOSED` in
  capitals, with whole-word matching. Code spans and whole words are class (a), plan D6. The
  rest is class (b), with evidence:
  * the before documents themselves use "PROPOSED block" in Contract and Constraints prose,
    so the literal rule would refuse every lifecycle document;
  * HTML comments are already removed by `raf-okf` `prose_lines`, which predates this issue;
  * generated blocks are derived text.

  It is still a narrowing of a Constraint outside the table (ADVISORY A1).
* Verification: every before-bullet is verbatim or extended. The merge-base bullet gains
  the generated-only case and the prefix case.
* The "stub `git` and `cargo` on `PATH`" sentence is retained. It is realised by
  `untouched::stubs` (a recording git via `Programs::with_git`, asserting all 8 forms in
  order), `exec::stubs::a_stub_named_by_cargo_runs_for_cargo_and_a_stub_on_the_path_when_it_is_unset`
  (cargo and git resolved from the search path), and the cargo-workplan `process` tests
  (stub git and cargo really on PATH). The substance holds, but no single fixture has both
  stubs on PATH recording the reconcile forms (ADVISORY A4).
* The dropped record claim "`example-failing` requires proof that exactly one test ran" is
  untrue as written. `example-failing` never runs a test binding; that is pinned by
  `verify::test_bindings_are_not_run_by_reconcile_and_unresolved_examples_are_not_started`.
  A doc-test binding can select several entries, all of which must be `ok`. The behaviour
  the claim protected, that a binding which runs nothing fails, is kept in the Verification
  bullet ("zero tests run is reported as a failing binding") and pinned by
  `verify::a_binding_to_a_function_with_no_test_reports_zero_tests_as_failing`. Class (b).

### workplan-cli

* Every before-sentence is retained verbatim or extended (the cargo-subcommand form, root
  discovery order, sync/new/complete options and output, exit-2 cases, `status` exiting 0
  on gaps, the `-q` detail, JSON, the complete-write Constraint). These are covered by the
  E6/E14/E1 row or are additive.
* Verification "and the proposed text is present" became "the text of the blocks is
  present". This is a wording change to avoid the tense phrase. Same meaning, not weakened.

No class (c) removal was found.

## Findings

### REQUIRED

* **R1. `docs/capabilities/workplan-reconcile.md` Contract, Invocation (lines 197-201), is
  false as written.** "`<default-ref>` is the full ref name that **the first command**
  prints" and "resolved to a commit id by **the second command**" were true of the 4-command
  list at 6708afb. Task 11 prepended `git --version`, `git rev-parse --show-prefix`,
  `git rev-parse --is-inside-work-tree` and `git ls-files -z`. The first command is now
  `git --version` and the second is `git rev-parse --show-prefix`. Fix: name the commands
  ("that `git symbolic-ref` prints", "resolved ... by `git rev-parse --verify`").
  * "the last command" (cat-file) is still correct.
  * Completing this as a document edit to a delivered capability needs the normal change
    flow, or a direct prose fix that keeps `docs_contract` and `Gate` clean.

### ADVISORY

* **A1.** The tense-scope narrowing (HTML comments, link destinations, generated blocks,
  `PROPOSED` in capitals) is outside the Contract-edit table. It is justified (class (b),
  above), but per the plan's "put to the owner" rule, add it to the table and list it in the
  PR body's residual or readings section. The same applies to the `\`-to-`/` list-output
  normalisation (A2) and the "in the repository" scope of the reconcile write Constraint.
* **A2.** As A1, for the backslash normalisation of rustdoc list-output paths.
* **A3.** Carried item T9 A1 promised a Contract-edit row for `change new --copyright/--licence`.
  It is not in the plan table. The Contract statement is true and tested.
* **A4.** The reconcile Verification sentence "stub `git` and `cargo` on `PATH`" is met by
  three fixtures together, not by one. Consider rewording it to the built shape or adding
  the combined fixture.
* **A5.** e42ff32 has a test gap. The mutant that drops `outer.is_empty() ||` in `holds`
  survives. No fixture has a method whose impl block sits in an inline module of the crate
  root file (or of a file that contains the type's module). Likewise, the CHANGELOG and
  doc-comment claim "an impl in an inline module of another file ... is still refused" has
  no fixture. Every misnaming is fail-closed (the list pass selects nothing, so the binding
  is refused, never falsely passed), but the rule has no neighbour fixture as AGENTS asks.
  Add one inline-module case to `docs_lib`.
* **A6.** The pre-existing `verify` failures, not introduced by this issue, are left as they
  were:
  * `cap:source-index` binds two `cfg(windows)` tests that no Linux list pass shows, so a
    no-slug `verify` can never exit 0 on Linux or macOS;
  * `cap:workplan-document-model` binds the `no_run` doc test of `Gate` as an example, which
    the completion gate refuses by Contract.

  Both need a tracked home (no deferrals), and the PR body should name them. This issue
  improved the count from 5 to 3.
* **A7.** AGENTS.md and README now call the whole `raf-workplan` crate "delivered". Five
  `considered` workplan capabilities exist, one of which names `raf-workplan`. This matches
  the convention for the other crates (crate status is that of its built capabilities), but
  a reader could take it as covering them.

### Checks with no finding

* e42ff32 is correct for the sync/preview case. Accepted bindings keep their names (the
  override applies only when the file module does not hold the type's module), and the
  real-cargo fixture plus the mutation pin it.
* The records were completed with the binary. The recorded outputs match the final tree
  (92e4de8 is state plus the `Changed by` rows plus the catalogue). The records are deleted
  and the three states are `delivered`. `docs sync` writes nothing, and `docs_contract`
  and `Gate` are clean.
* Hygiene:
  * no tracker numbers, process wording, stage words or attribution in the diff or the
    5 commit messages;
  * SPDX is intact (`make reuse`);
  * the CHANGELOG entries are accurate (sync/preview naming confirmed against the base
    `verify` output; the `docs_sync` example still exists);
  * the AGENTS, README and SECURITY statements are true (the SECURITY `--show-prefix`
    order was seen in the stub log);
  * `make docs-sync` is idempotent and recreates the same index.

VERDICT: BLOCKED

## Re-review

Re-checked at HEAD 9f1ab69 in a scratch worktree (removed afterwards).

| Finding | Fix location | Evidence | Status |
| --- | --- | --- | --- |
| R1 reconcile Invocation text says first/second command | `docs/capabilities/workplan-reconcile.md` Invocation (about lines 195-203), commit a72c5f5: now "that `git symbolic-ref` prints" and "by `git rev-parse --verify`" | no "first command"/"second command" remain in the file; the commands match the code and tests (`crates/raf-workplan/src/git/tests.rs:305,388`); `cargo run -p cargo-workplan -- workplan check` exits clean | resolved |

VERDICT: PASS
