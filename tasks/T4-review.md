# T4 review: commits 7a3a5f2, b3e6b7d, 01799a9, da83cdc (HEAD da83cdc)

VERDICT: BLOCKED.

The T3 fixes (B1-B4) are closed and the default mode is byte-identical. The gates
pass. But the literal touch rule `contract-effect-understated` has four false
negatives that contradict the capability text (R1-R4 below), and one false positive
against its "a reformatting does not" sentence (R5). The reconcile Contract
assessment lost most of its watch (R6). Four of my mutants of the new rule survive
the whole suite (R7). The git reads of the rule are not pinned (R8).

## How the review was done

* I did not edit the worktree. I made in-repo scratch clones under
  `.stage/raf-workplan-strict-mode-force-coverage/scratch/`, each with
  `git clone --no-hardlinks` and a `pwd` check:
  * `t4review` at da83cdc, for the gates;
  * `t4old` at 7d35393;
  * `t4adv` at da83cdc, for the probes;
  * `t4mut` at da83cdc, for the mutants.
* Builds went only to the target directories the wrapper assigned:
  `/opt/.cargo/target/{t4review-2f781a04,t4old-808dcc17,t4adv-4e87b847,t4mut-50a514da}`.
* There were no timing runs.
* Logs are in `scratch/t4review-logs/`. Tools are `scratch/t4review-tools/mutants.py`
  and `scratch/t4review-logs/cmp.sh`.
* Baseline: `cargo test -p raf-okf -p raf-workplan -p cargo-workplan --locked
  --no-fail-fast` in `t4review` passed 1486 tests, failed 0, exit 0.

## Required fixes

**R1. Narrowing the visibility of a bound public item completes under strict.**
`crates/raf-workplan/src/lifecycle/touched.rs:256-267`.

* The removal check skips every merge-base unit whose name still exists in the
  working tree, at any visibility.
* The candidates are only the items of today's surface.
* So an item that leaves the surface is neither touched nor removed.

Probes (`t4adv/crates/raf-workplan/tests/complete.rs` `zz_probe`, log
`probes1.log`), each run as reconcile plus `complete` under strict:

* P04 `pub fn mul` -> `fn mul`, bound `fx_life::mul`: `completed`.
* P05 `pub mod shapes` -> `mod shapes`, bound module: `completed`.
* P29 `pub fn push` -> `fn push` in `impl Frame`, `Frame` bound: `completed`.
* P03 `pub` -> `pub(crate)` was refused only by accident: the fixture's doc test
  failed to compile. The rule raised nothing.

This contradicts `docs/capabilities/workplan-strict.md:243` ("an edit of ... a
visibility ... touches it") and plan E3 ("removes ... it").

Fix: a merge-base unit that the rule counts as public, whose name in the working tree
is no longer public, is touched. Fixtures: P04, P05 and P29 refuse under strict and
complete under default.

**R2. A change to the header of an inherent impl block of a bound type completes.**
`touched.rs:417-423` gives the impl block no name and no candidate. A member's span
does not include the impl header.

* P02: `impl<T: Clone> Wrap<T>` -> `impl<T: Clone + Default>`, with `Wrap` bound.
  Reconcile owes nothing, and `complete` gives `completed`.
* P30: `#[cfg(any())]` on `impl Frame` removes its methods. Reconcile owes nothing.

This is the "generic bound change" case. A bound bound on a free function is caught
(P01), but a bound on the impl block is not, although it changes the signature of
every bound method. The literal rule (plan E3: "alters an item covered below a bound
one") and `workplan-strict.md:242-243` (impl blocks; "a signature") cover it.

Fix: compare each member together with the tokens of its impl header (attributes,
generics, self type, trait, where clause), or make the inherent impl of a covered
type a candidate. Fixtures: P02 and P30.

**R3. An edit is hidden when its result equals another item of the same file.**
`touched.rs:235-242`. A candidate is tested for membership in the set of the streams
of every merge-base unit of its file, whatever its name.

* P13: `impl A { pub fn get(&self) -> u32 { 2 } }` beside `impl B { ...{ 1 } }`.
  Edit `A::get` to `{ 1 }`. Reconcile owes nothing, and `complete` gives `completed`.
* This is a body change of a bound public item under `none`.
* It is realistic for the inherent methods of sibling types (`new`, `len`,
  `is_empty`).

Fix: compare by name and stream. An item moved within its file keeps its name, and
`now.units` already names each unit. Fixture: P13.

**R4. A removed member of an impl block in another module than its bound type is
never found, and the Constraints justify this wrongly.**

* P11c: `pub fn push` removed from `impl crate::Frame` in `src/imp.rs`, with `Frame`
  bound. `complete` gives `completed`.
* P11d: `src/imp.rs` and its `mod imp;` are deleted. `complete` gives `completed`.

The edit of such a method is caught (P11a, P11b), but its removal is not.
`workplan-strict.md:566-570` justifies the miss for deleted files: "the binding that
names it no longer resolves, which `binding-missing` and the check report". That is
false when the binding names the self type or a parent module. `Frame` still
resolves, and nothing is reported.

Fix: name removed impl members by the self type's path, which the working-tree index
resolves from the impl's module (`touched.rs:268-271`, `self_type_key` at `:521`).
Or state the limit plainly and correct the binding-missing sentence. Either way, add
a fixture that pins it.

**R5. A rustfmt reflow that adds a trailing comma refuses under strict, against
"a reformatting ... does not".**

* P25: `pub fn mul(a: u32, b: u32) -> u32 { a * b }` reflowed by rustfmt to one
  parameter per line adds `b: u32,`. The rule owes `contract-effect-understated`.
* P20: a trailing space added inside a doc comment also refuses.

Both fail closed, which is safe. But `workplan-strict.md:244` and `touched.rs:22`
promise that a reformatting is free. The unit test at `touched.rs:609` even pins the
trailing comma as different.

Do not canonicalize the comma (`(a,)` and `(a)` differ). Narrow the sentence
instead: whitespace, line breaks and plain comments are free; a reformatting that
adds or removes a token (a trailing comma, a merged `derive`) or changes doc-comment
text touches the item.

**R6. The reconcile Contract assessment now watches 5 symbols instead of 28.**

* Source: `work/assessments/workplan-reconcile-contract.md`.
* Cause: the resolved Contract (`docs/capabilities/workplan-reconcile.md:416-425`)
  names `obligation::suppressed` and `Workplan::reconcile` strongly, so the
  assessment rule (`crates/raf-workplan/src/assessment.rs:13-21`) switches from
  every binding to the named ones.
* Effect: the Contract states the table of kinds and their evidence, yet it no longer
  watches these bindings: `raf_workplan::obligation` (a module binding, which stands
  for the whole module), `ObligationKind`, `Needed`, `Reconciled`, `lifecycle::Impact`
  and `git::blob_at`. A future change to the kinds would not stale it.
* Every other changed assessment kept its count.

This is mechanically "documented behaviour", but it weakens the gate on the section
T4 rewrote. Fix: name the obligation bindings strongly in the Contract (for example
`obligation::ObligationKind`, `obligation::Needed`, `raf_workplan::obligation`) and
assess again.

**R7. Four mutants of the new rule survive the full suite**
(`cargo test -p raf-okf -p raf-workplan -p cargo-workplan --no-fail-fast`, 1486
passed each; log `mutants.log`):

* MA `touched.rs:223`: a file absent at the merge base touches nothing. The rule is
  stated at `workplan-strict.md:251-252`, and probe P28 shows that it works today.
* MB `touched.rs:265`: a private merge-base item counts as removed. The rule is stated
  at `workplan-strict.md:247-248`.
* MC `touched.rs:419`: trait-impl members count as public for removal.
* MD `touched.rs:143-149`: a surface that cannot be listed is skipped instead of being
  `LifecycleError::Surface`. Probe P27 shows the error today: "the public surface of
  `fx_life` ... cannot be listed". No test produces `Surface`.

Fix: one fixture each, with its neighbour. Use a new file holding a bound item, a
private helper of a bound module removed, a trait-impl member of a bound type
removed, and the two-glob `dup` surface (as `cases.rs:1402`) under strict `none`,
with exit 2 in the CLI.

**R8. The git reads of the rule are neither counted nor pinned.**

* `touched.rs:67-108` reads one `git cat-file blob` for every file of every lib/bin
  target of each package that holds a bound item.
* On this repository a strict `none` reconcile asked 137 source blobs (GIT_TRACE, log
  `gittrace.txt`; 143 `cat-file` in all).
* `Lifecycle::complete` evaluates twice (`complete.rs:193,208`), so a completion asks
  about 274.
* The count is linear and bounded by the source index's file limit, so it is not
  unbounded. But no tally or test pins it.
* Repository practice pins process calls: `tests/evidence.rs:800-808` asserts one
  `cat-file`, and `tests/untouched.rs:674-702` lists the calls.

Fix: add a recording-runner test that asserts exactly one `cat-file` per file read
(none for test-only files), and none under `default` or `contract_effect: changes`.

## (1) T3 fixes B1-B4: PASS

* I copied the T3 reviewer's probe file to `t4adv/crates/raf-workplan/tests/zz_adv.rs`
  and added the B scenarios (log `claims-probes.log`).
* B1, keys:
  * Both orders: `foo, foo-3, foo-2` and `foo-2, foo, foo-3`.
  * A chain `Foo, Foo, Foo 3, Foo, Foo 2` gives `foo, foo-4, foo-3, foo-5, foo-2`.
  * Three identical Constraints items give `c-…`, `c-…-2` and `c-…-3`.
  * The keys are distinct in every case.
* B2, separators:
  * These lines are no claim: `* * *`, `- - -`, `---`, `___`, `_ _ _`, `-`, `- `,
    `1.`, `2) ` and `+`.
  * The neighbour `- neighbour` is one claim, and uncovered.
  * A break directly after an item ends that item.
* B3:
  * `Para one.`, a marker, then `Para two…` gives two units, the second uncovered.
  * `Para A.`, `<!-- a note -->`, `Para B.` gives two units.
  * A comment line ends a table.
  * A bold lead directly after `####`, `#####` or `######` starts a clause.
* B4 and the mutants (log `mutants.log`):
  * The report fixtures carry `cap:alpha`.
  * MR1 (sort by section and line before capability) was killed by
    `claim_coverage::the_report_lists_every_claim_of_every_delivered_capability_sorted_and_the_same_each_run`
    and by `claims::the_report_sorts_by_capability_first_and_capability_prints_the_one_it_names`.
  * MR2 (`whole.capability(id)` -> `whole`) was killed by the latter.

## (2) Literal touch rule: FAIL (R1-R5)

These probes behave as the text says:

* P01, a generic bound on a fn: touched.
* P06 and P06b, `pub(crate)` -> `pub`, in a bound module and bound directly: touched.
* P07, two bound items reordered: nothing.
* P08, an item moved to another module file and re-exported: touched (stated limit).
* P10, a rename with the binding updated: touched.
* P11a and P11b, the body or signature of a method in an impl in another module:
  touched.
* P12, an impl beside its type: touched.
* P14, the body of a bound `#[macro_export]` macro: touched.
* P16, a file below a bound module deleted with its `pub mod` line: removed.
* P17a and P17b, non-ASCII identifiers (`fläche`, `bär` after `pub const Ä` on the
  same line): no false positive, and only `bär` is touched.
* P18a-c, CRLF: a comment and a CRLF-to-LF rewrite give nothing, and a body edit is
  touched.
* P19, a BOM: nothing.
* P21, `///` against `#[doc = "..."]`: equal.
* P24, a plain comment between a doc comment and its fn: nothing.
* P26, a tab and no-space reflow: nothing.
* Every refusal left the tree byte-identical.

Advisories (not blocking; state them in Constraints):

* P22: the members of trait impls of a bound type are not judged.
* P23: a bound `pub use` retargeted to another item completes.
* P15: a `macro_rules!` edit that changes a pub fn generated in a bound module
  completes.
* P09: moving an item into an inline module, re-exported, with the module unbound,
  completes. This is consistent with "moved within its file".

## (3) Reasons, Usage/Examples and the conditions that already hold: PASS

* `outcome-without-reason`:
  * It is owed for `dropped` and `moved:change:other`.
  * It is not owed for `implemented` with a reason, nor for a reason given without an
    outcome.
  * It is owed under strict only.
  * The headline is `refused: 1 obligation, 1 item without a reason`.
  * The with-reason neighbour completes.
* Under strict `none`, an untouched Usage or Examples is owed, and the Contract in
  the same `affects` stays suppressed. Under default both complete.
* In the CLI, strict gives 1, the edit gives 0, and no flag gives 0.
* A failing bound test and a `tense-mismatch` give `Completion` values equal under
  strict and default.
* My mutant MI (an `implemented` item owes a reason) was killed by 18 tests.
* Advisory: `reason: "two\nlines"` is accepted, although the text says "a non-empty
  line of text". `claim` is accepted the same way.

## (4) Default-mode identity: PASS

* Library: I dumped every `all_cases()` case under Default (`zz_dump.rs`, logs
  `dump-old.txt` and `dump-head.txt`).
  * There are 195 cases at 7d35393 and 196 at da83cdc. The new one is
    `okf-item-reason-empty`.
  * One case changed: `okf-item-unknown-key`, whose fix text now names `reason`. This
    is the accepted change.
* The 7d35393 versions of `raf-workplan` `tests/complete.rs` and `tests/obligations.rs`
  and of `cargo-workplan` `tests/reconcile.rs` and `tests/complete.rs` were run
  against the HEAD code (log `old-tests-on-head.log`).
  * Only the three expected tests fail: the T2 strict test (`Unscripted git
    symbolic-ref`), the 13-kind both-directions list, and the static-kind fixture
    list.
  * Every default-mode lifecycle test passes.
* CLI: I ran the 7d35393 binary and the HEAD binary on both clean trees, in text and
  JSON. The commands were `check`, `-q check`, `status`, `coverage code|tests|claims`,
  `evidence claims`, `change impact|reconcile workplan-strict`, `change reconcile
  nosuch`, `docs preview` and a bad command. All 48 comparisons were SAME in stdout,
  stderr and exit code (log `cmp2.txt`).

## (5) Kinds, errors, refusals: PASS except R7

* `ObligationKind::ALL` has 15 kinds, in rank order. `is_static` and `is_strict` are
  pinned.
* `the_cases_cover_every_kind_of_obligation_in_both_directions`,
  `each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes`
  and `each_kind_..._is_a_reconcile_line_...` hold both new kinds and pass.
* `LifecycleError::Surface` is fail-hard by probe, but untested (R7 MD).
* Reconcile exits 2 for a merge-base blob that does not parse
  (`under_strict_a_bound_source_file_that_does_not_parse_at_the_merge_base_is_exit_two`).
  That test does not assert that the worktree is unchanged (minor).

## (6) Changed and renamed tests, and the assessment

* The T2 test `the_check_once_the_record_is_removed_is_judged_with_the_strictness_floor_of_the_gate`
  is legitimate. The strict `none` change now asks `git` for the merge base, so the
  FakeRunner gains `at_merge_base` with the unchanged library. Every assertion is
  kept.
* The three renamed tests are strengthened, not weakened: 13 to 15 kinds, 3 to 4
  process kinds, and the suppression is now checked under both strictnesses with
  `changes` never lighter.
* The narrowed reconcile Contract assessment is a weakening (R6).

## (7) Cost: see R8

## (8) Own mutants

| Mutant | Result |
| --- | --- |
| MA new file not touched | SURVIVED |
| MB private removal counted | SURVIVED |
| MC trait-impl members public | SURVIVED |
| MD surface error skipped | SURVIVED |
| ME CRLF fold removed | SURVIVED, equivalent: proc_macro2 already reads `\r\n` as LF |
| MG `self_type_key` -> None | killed by `lifecycle::touched::tests::the_items_of_modules_impls_and_traits_are_named_below_the_file_and_a_private_one_is_not_counted` |
| MH inline modules not read | killed by the same test and `complete::under_strict_an_item_renamed_below_a_bound_module_refuses_and_under_default_it_completes` |
| MI `implemented` owes a reason | killed by 18 tests, among them `complete::under_strict_an_item_moved_within_its_file_completes` and `complete::under_strict_an_untouched_usage_or_examples_of_a_none_change_refuses_and_under_default_it_completes` |

`git diff --exit-code` was clean after each run.

## (9) Docs: PASS

* The PROPOSED blocks of `workplan-reconcile`, `workplan-complete-gate` and
  `okf-profile` were resolved into contract text in the commits that deliver them.
* No remaining PROPOSED block holds T4 content. Its two `reason` hits are the T6
  exception reasons.
* `okf-profile.md` states the `items` rule.
* The CHANGELOG lists the breaks (`suppressed`, `Needed`, `LifecycleError::Surface`,
  `ChangeItem::reason`, `Rule`).
* The bindings are complete: `cargo workplan check` exits 0.
* `cap:workplan-strict` stays `implementing`.
* Added lines (`git diff 7d35393 da83cdc`) hold no issue or PR number, no process
  wording and no hash. The only hex run is the test constant `COMMIT`.

## (10) Gates in `t4review`: PASS

* `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`
  exits 0.
* `make fmt clippy doc deny reuse issue-refs` all exit 0 (reuse 549/549, issue-refs
  68/68).
* `cargo workplan check` exits 0.
* `make docs-sync` rewrites nothing.
* The three crates pass 1486 tests.

## Left in place

* In the repository (scratch only): `scratch/t4review` (clean), `scratch/t4old`
  (clean), `scratch/t4mut` (clean) and `scratch/t4adv`. `t4adv` holds the probe
  edits to `crates/raf-workplan/tests/complete.rs` and `crates/raf-okf/tests/profile.rs`
  and the untracked `zz_adv.rs`, `zz_old_complete.rs`, `zz_old_obligations.rs` (in
  raf-workplan) and `zz_old_complete.rs`, `zz_old_reconcile.rs` (in cargo-workplan).
  Also `scratch/t4review-logs/` and `scratch/t4review-tools/`.
* Outside the repository:
  * the wrapper-assigned build directories `/opt/.cargo/target/t4review-2f781a04`,
    `/opt/.cargo/target/t4old-808dcc17`, `/opt/.cargo/target/t4adv-4e87b847` and
    `/opt/.cargo/target/t4mut-50a514da`, plus the wrapper's shared caches;
  * `make reuse` ran through `uvx`, which uses its own cache.

  I created nothing in `/tmp`. The `/tmp/raf-source-unit-*` directories present are
  raf-source unit-test scratch, which none of my commands ran (the concurrent T5
  work).
