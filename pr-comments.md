# PR #26 — comments and review threads

4 comment(s). **Every one needs a disposition** in
`review-debt.md`: addressed (with the commit hash) or declined (with the
reason posted in reply). Bot comments count.

## 1. paudley — 

# Plan: issue 14, rustc visibility rules for private globs in raf-source

## Issue summary
`raf-source` settles `use` paths with rustc's import-accessibility rule. Two rustc rules are not modelled and are listed as a Constraints bullet of `cap:source-index`: (1) an explicit `pub use` of a binding its module reaches only through a private glob must stay no more visible than that binding, so a module that globs the re-export cannot read a name rustc hides from it; (2) a name the globs of a module bring in ambiguously, one item through a glob the reading module cannot access, must be refused (E0659, sometimes E0573) and not answered with the other item. The issue reports 12 wrong queries in 10 of 5,522 generated programs. Scope: model both in the settling of `use` paths, independent of declaration order; an every-order fixture for each; re-run the rustc-oracle fuzz with private globs with no new disagreement; remove the two sentences from the Contract's Constraints.

## Standing goals and decisions
CLAUDE.md rules apply (no timing runs, hooks on, no issue numbers or process words in committed files, no deferrals). Owner instruction (verbatim): "In your next PR, make sure to update README.md to the current status." Capability changes go through the workplan lifecycle: reopen `source-index` with `change new`, edit its Contract and Constraints, re-assess its sections, complete with `change complete`.

## Prior art
No prior attempt (0 commits, branches, PRs). The nearest work is the settling tables in `crates/raf-source/src/settle.rs` and `glob_scope.rs`, and the every-order tests in `crates/raf-source/tests/use_resolution.rs`. No rustc-oracle harness exists in the repository or its history (verified by grep and `git log --grep`): the issue's fuzz was a throwaway. Recurrence: none counted.

## NEEDS-JUDGMENT dispositions
* "rustc-oracle fuzz" has no harness: Task 1 builds one as a committed test, so the "re-run" is repeatable and not a claim.
* Which error the every-order fixture asserts (E0659 or E0573): the index answers by a typed result (not found / ambiguous), not by rustc codes; the oracle compares accept/reject plus the item chosen, and fixtures assert the index result for the case rustc refuses as ambiguity; the rare E0573 case gets its own fixture whose expectation is "refused".
* The Contract sentences to remove are the Constraints bullet quoted above (`docs/capabilities/source-index.md`, "Two visibility rules of rustc are not modelled"); the Contract text that already describes the other rules is extended to state these two.

## Completeness contract
| Requirement | Task | Acceptance |
| --- | --- | --- |
| rustc-oracle fuzz with private globs exists and is re-runnable | 1 | `cargo test -p raf-source --test rustc_oracle` generates a fixed seed list of programs with private globs, asks real rustc and the index the same questions, and reports every disagreement; before Tasks 2 and 3 it reports the known disagreements (a fixed-seed failing case is captured as a named regression input) |
| model rule 1 independent of declaration order | 2 | the oracle has no rule-1 disagreement; the every-order fixture passes in all permutations of declaration order |
| model rule 2 independent of declaration order | 3 | same for rule 2, including the E0573 shape |
| every-order fixture for each rule, with a passing neighbour | 2, 3 | fixtures named in the Verification list of `source-index`; neighbours (accessible glob, unambiguous glob) still resolve |
| re-run the oracle: no new disagreements | 4 | full oracle run at the final head with a larger seed list; the count of disagreements is zero; results recorded |
| remove the two sentences from the Constraints | 5 | `grep -c "visibility rules of rustc are not modelled" docs/capabilities/source-index.md` is 0; the Contract states both rules; capability reconciled and completed with the tool |
| README current (owner instruction) | 6 | README status text matches `cargo workplan status` and the crate/command lists (history, evidence, assess, coverage, rationale; delivered counts; census still designed) |

## Task 1: the rustc-oracle test
`crates/raf-source/tests/rustc_oracle.rs` plus a small generator in `tests/support/`: a deterministic PRNG (fixed seeds, no clock) builds a library crate with a few modules, private/pub/pub(crate) globs, named and glob `pub use`, and shadowing; for each program it picks the names to read, writes the crate to a temp directory, and asks `rustc --crate-type lib` whether `use` probes of each name compile (one probe item per query, in one compile with error codes per query where possible), then asks the index what the name resolves to for the probing module. A disagreement is a probe rustc accepts that the index resolves to a different item or rejects, or a probe rustc rejects that the index resolves. The default test uses a fixed list of seeds and a fixed program count (kept small enough for CI), with the count raised by an environment variable for the full re-run. No timing assertions or sleeps. Skips with an explicit failure message when `rustc` is unavailable (it is always available under cargo).

## Task 2: rule 1 in the settling
In `settle.rs` / `glob_scope.rs`: when a named `pub use` re-exports a binding that the importing module reads only through a private glob, the new binding carries the visibility of the weaker of the import and the binding it re-exports. Order independence follows from computing visibility as a meet in the existing fixpoint. Fixture in `use_resolution.rs` in every declaration order and the neighbour (the same re-export of a binding reached by an accessible glob stays as visible as the import).

## Task 3: rule 2 in the settling
When the globs of a module bring a name in ambiguously and one of the items arrives through a glob the reading module cannot access, the reader gets rustc's answer (refused as ambiguity; the E0573 shape refused as not found), not the other item. Fixture in every order and a neighbour (ambiguity resolved because only one glob is accessible but the other item is not a candidate by rustc's rule stays resolved).

## Task 4: the full oracle run
Run the oracle with a large seed list at the final head (outside CI, results in the stage notes and the PR) and require zero disagreements; any disagreement is a new gap fixed in the settling with a fixture.

## Task 5: capability documents
Reopen `source-index` (`change new`), remove the Constraints bullet, state both rules in the Contract, bind the new tests, `docs sync`, re-assess the changed sections, `change complete`.

## Task 6: README
Update `README.md` status to the current state (what `raf-workplan` and `cargo-workplan` deliver now; census still designed; the crates are not published) and check every claim against `cargo workplan status`.

## Task 7: handoff
Open the PR (`Closes #14`), post the plan and the least-confident note.

## Declined
* A CI job running the oracle at full size: it compiles thousands of programs with rustc; the default seed list in the test is the CI guard, the full run is recorded per change.
* Modelling the rest of rustc's name resolution: out of scope.

# Revision 1 (supersedes the tasks and contract above where they differ)

Integrates reviews/plan-{completeness,constraints,design,enhancement,adversary}.md. Every task ends with: focused validation, an independent task review (`tasks/TN-review.md`), a commit through the hooks with explicit files, a push, and an issue update. No wall-clock timeouts, sleeps, recorded durations or timing assertions anywhere, including on `rustc` child processes.

## Corrected evidence and decisions
* A second Constraints bullet (`docs/capabilities/source-index.md` around "Of the two visibility rules above that the index does not model") also describes both rules as unmodelled and rule 2 as reaching `Workspace::exported_items`. Task 5 rewrites it to the delivered behaviour; `exports.rs` judges routes by declared `Vis::Public`, so rule 1 reaches `exports.rs` with an `exported_items.rs` fixture (Task 2) and rule 2 gets one (Task 3).
* Rule 2 stance: the index answers `Ambiguous(all candidates)` for both rustc shapes (E0659 and the E0573 shape). Views only grow in the settling and the repository already took this stance for the neighbouring case (`tests/use_resolution.rs`, the comment near line 3066); "not found" is retracted. The oracle compares accept/reject plus the chosen item; an ambiguity answer counts as rustc's refusal.
* Rule 1 cap: the cap is normalised to an ancestor of the importing module (`Inherited`, `Restricted("crate")` or `Restricted("in <path>")`), the ancestor chain is pre-seeded in `vis_of`, and the cap is computed from the origin of the hit's last binding where a glob-reached origin exists (none for associated members and module-ending paths). The named binding no longer takes the declaration's visibility verbatim.
* Rule 2 homes: `seen_by`, `export_ids`, the duplicate loop in `make_view`, and `arrive` all apply one rule: export the full candidate set when it has more than one item and the reading class sees at least one of them (monotone). `make_view` calls `export_ids` so the filter has one definition. `glob.rs` (workspace-member patterns) is untouched.
* Declined: the scope-chain re-formalisation of the whole settling (`Scope = Public | Module(depth)`, absorbing ambiguity state). Cost of declining: the two rules stay two local rules on the existing class model. Reason: it rewrites a 2940-line settling whose behaviour about 1000 existing every-order tests pin, for two rules whose minimal form is bounded; revisit if the minimal form cannot satisfy the oracle.

## Completeness contract (replaces the table above)
| Requirement | Task | Acceptance |
| --- | --- | --- |
| rustc-oracle with private globs, re-runnable and able to find both bugs | 1 | `cargo test -p raf-source --test rustc_oracle` fails when `rustc` is missing (message names the tool) and asserts a non-zero program count and non-zero probe counts; over the full run of at least 5,522 programs the committed generator yields at least one divergence per rule on the unfixed resolver (shape counts printed per run); each is committed as a minimised replayable input under `fuzz/corpus/rustc-oracle/` with its expected classification; the default test replays the corpus plus a fixed seed list |
| the comparator is falsifiable | 1 | a committed negative fixture feeds the comparator a program on which the index is made to answer wrongly and requires the divergence to be reported |
| model rule 1, every order | 2 | `Workspace::resolve` (or the viewer-aware entry named in Task 1) on a module that globs `m::c`'s `pub use super::A` where `A` reaches `m` only through a private glob returns `NotFound` for the reader and the item for `m`, in every permutation of declaration order (`one_outcome_in_every_order`); neighbour: the same re-export of an accessible-glob binding still returns the item; `exported_items` fixture: an item reached only through such a re-export is not exported; the oracle corpus entry for rule 1 now agrees |
| model rule 2, every order | 3 | a reader whose globs bring a name from two items, one through an inaccessible glob, gets `ResolveError::Ambiguous` listing both, in every permutation; the E0573 shape gets the same; neighbour: a name brought in by two globs of which only one is accessible and the other brings in a different, non-candidate name still resolves; `exported_items` fixture likewise; the corpus entry for rule 2 agrees |
| no new disagreement | 4 | full run (at least 5,522 programs, seeds and counts printed) at the final head: zero divergences outside the committed expected list, and the expected list is empty for the two rules; counts recorded in the PR |
| Constraints sentences removed, Contract states the rules | 5 | `grep -c "visibility rules of rustc are not modelled" docs/capabilities/source-index.md` is 0 and `grep -c "do not model\|does not model" ` finds no sentence about these rules; both Constraints bullets are rewritten; the capability goes `implementing` with PROPOSED blocks and a capability-named change record, `INVENTORY` edited in the same commit, new tests bound, `docs sync` twice with no diff, sections re-assessed, `change reconcile` empty, `change complete` removes the record, `check` exit 0 |
| README current (owner instruction) | 6 | each Status cell of the crate table equals `cargo workplan status` for its capabilities, and the command list equals `cargo workplan --help` (check, status, docs sync/preview, change new/impact/reconcile/complete, verify, history, evidence, assess, coverage) |
| gate before handoff | 7 | `make check` with hooks installed exits 0 except the wall-clock probe, which this machine does not run (no timing runs); the default oracle seed list passes in `test-release` and `cross-check` |

## Task 1 (revised): differential harness
Under `crates/raf-source/tests/` with its generator in `tests/support/`: SPDX headers; names by program shape. Generation: deterministic PRNG, fixed seeds. Each program is a library crate; each probe is a `use` of a name inside a chosen module. rustc side: two-phase compile, JSON error format, `--emit=metadata`, `-A warnings` (warnings count as accept), line-mapped probes, and unique tag associated constants so an accepted probe reveals which item it resolved to. Index side: the oracle asks the question from the reading module (the `use` walk with a viewer); if no public entry point asks from a module, one is added to `Workspace`, documented in the Contract in Task 5 and fixtured. A divergence classifier maps each divergence to rule 1, rule 2 or unclassified; unclassified fails the test. Known divergences live in the committed corpus with their classification and the test asserts each still diverges (so a fixed rule must remove its entries in the same commit). Seed count: the default list is small; `RAF_ORACLE_PROGRAMS` raises it for the full run and the run prints the seed list and counts. Failure of `rustc` to start is a test failure.

## Task 7: handoff
Run `make check`; open the PR (`Closes #14`) with the full-run counts and the README change; post the plan and the least-confident note.

# Revision 2 (answers reviews/plan-review.md; supersedes Revision 1 where they differ)

* **Rule 2 is derived from rustc, not from the issue's prose.** The committed fixture `tests/use_resolution.rs` (the shape around lines 2837-2860: a private glob of one `Y` and a pub glob of another `Y` in `m2`, read from outside module `m3`; the comment there says rustc accepts it) answers the pub-glob item in all orders. Task 3 first compiles each existing fixture crate that holds an inaccessible-glob collision with the Task 1 harness, in every declaration order, and records which way rustc goes. The rule is then the exact condition under which rustc refuses (the reader can see the colliding glob binding, or the collision is made ambiguous by an inaccessible glob only where rustc says so); fixtures that rustc contradicts are corrected with the reason in their comment, and fixtures it confirms stay as the passing neighbours. No rule is written before the oracle shows rustc's answer for its shape.
* **Documented divergence class.** The classifier has a `documented` class tied to the Contract sentence about the glob re-export that rustc only warns about (`ambiguous_glob_reexports`) and resolves while the index answers `Ambiguous`. rustc runs with `-A warnings`, then `-W` for the two relevant lints so the warning is observed and classified, not hidden. `unclassified` fails the test; `documented` is counted and printed. The generator is not narrowed to avoid the shapes under test.
* **Corpus lifecycle (one story).** An expected-divergence corpus entry exists from Task 1 until the task that fixes its rule; that task deletes the entry in the same commit (the test asserts each remaining entry still diverges and that no unlisted entry diverges). After Tasks 2 and 3 the corpus holds only `documented` examples and regression inputs that now agree (kept as passing replay inputs).
* **Viewer-aware entry (decided).** Add `Workspace::resolve_from(module, path)` on the existing `Heads::Use` walk (workspace.rs entry points have no viewer today). It is documented in the Contract, fixtured with a passing neighbour and bound in Task 5. The oracle uses only public entry points.
* **Gates (decided).** `make check` stops at the first failure and `test-release` runs the cost probe `validate_cost`, which asserts elapsed time and is forbidden to run here. The handoff therefore lists the gates individually: `make fmt clippy doc deny reuse msrv fuzz cross-check licenses issue-refs`, `cargo test --workspace --locked -- --skip within_the_bound`, `cargo test -p raf-source --release --locked`; hosted CI runs the rest. The default oracle seed list is small enough to run in debug and release.
* **Exported-items assertion.** The `exported_items` fixtures assert `Err(ExportError::Resolve(ResolveError::Ambiguous(_)))` where the surface query meets rule 2 and the item's absence where rule 1 hides it.
* **Fixtures permute across modules and name spaces** (enhancement 10): the module declarations at the root are permuted as well as the blocks inside each module, and one shape has a struct and a function of one name so a refusal in one name space leaves the other resolved. **Chains and routes** (11): a fixture where one item reaches a module by a `pub use` of a private-glob binding and by a `pub(crate)` glob, and a chain of two re-exports through two private globs, in every order.
* **Enhancement items taken:** 4 (move `Draw`, the xorshift generator, and the scratch/child helpers into `tests/support/` and share them; no second PRNG; the rustc child does not use `within_watchdog`), 5 (a bulk metamorphic check: for each generated program and a drawn permutation of its declarations, every query and `exported_items` agree), 12 (the oracle's program count, query count, divergences by class and the rustc version are recorded in the Verification section of the capability, without process wording). **Declined:** 7 (a second oracle for `exported_items`): cost M; the surface is covered by fixtures asserting the exact error and by the existing bulk surface test, and rule 1 and rule 2 reach it only through the same tables the oracle checks. Revisit if a surface divergence appears.
* The earlier figure of about 1000 every-order tests is corrected: 77 every-order test functions.

# Revision 3: the raf-source timing tests are converted before the capability is completed

Owner decision (verbatim, to the question of how to close the capability when `change complete` runs its bound elapsed-time tests): "Convert timing tests first". This is a material scope change, so the completeness, constraints, design and completion-adversary lenses rerun on this revision, then the final review; the enhancement lens does not rerun because the design below was produced by a dedicated design pass over the same ground and the change is a conversion toward a fixed target (the counts the crate already produces), not a new design space.

The design is `timing-conversion-design.md` beside this plan (inventory of the 14 clock sites, the counters, a conversion per test with the guarantee it keeps and the mutant it must fail on, the commit order, the risks). Its open decisions are taken here:

* Risk 1 (counters see only instrumented code): accepted, option (a); counters sit at the shared accessors.
* Risk 2 (blocking is not a count): the `cfg(test)` hook is taken: the Unix open paths record the flags they open with, and the containment tests assert `O_PATH` or `O_NONBLOCK` was set, so a regression that removes the non-blocking open fails instead of hanging.
* Risk 3 accepted. Risk 4: the negative control fails hard where fsnotify does not skip `O_PATH`. Risk 6 accepted (binding paths follow the moved tests).
* Risk 5 (the Contract reports wall-clock figures): Task 5 restates them as the step and read bounds the converted tests assert, and "refused before any pattern is expanded"; no figure from a timing run remains in the document.
* Risk 7 (lock-in): taken. A repository gate in `tests/repo_gates.rs` refuses `std::time`, `Instant`, `Duration`, `sleep` and `recv_timeout` in `crates/raf-source/src` and `crates/raf-source/tests` (examples excluded), with a fixture and its passing neighbour, so the rule holds in the crate being converted.
* No public API is added (`Usage::module_paths` is not added). The release-only 10,000-reader case of T4 is dropped, as the design argues.
* `examples/registry_scan.rs` (a per-crate timeout and a sleep poll in an example binary, not a test) is out of this scope and is flagged to the owner in the handoff.

## Tasks 4b.1 to 4b.10 (in the design's commit order, before Task 5)

Each task: the design's conversion, run through named test binaries only (never a whole-crate run while any clock test remains), with the mutant check performed in a scratch copy and recorded in validation.md; every renamed or moved bound test has its binding and Verification text updated in the same commit, followed by `make docs-sync` (twice, no diff); an independent task review; commit through the hooks; push; issue update.

1. `tests/support/counting.rs` (the counting `Tree` with an operation ceiling), replacing `Recording` and `CountingReads`.
2. `src/work.rs` (thread-local test-only tallies, `measure(ceiling, ..)` that fails fast, and its own unit test) with `mod work` in `lib.rs`.
3. T1 and the new wide-line unit test in `facts.rs`; T11 renamed in `symbol_index.rs`; bindings and Verification updated.
4. `glob.rs` and `members.rs` ticks and unit tests; T5 in `manifest_caps.rs` (three renames); T8 in `trait_keys.rs`; bindings and Verification updated.
5. T7 moved to `resolve.rs` tests; removed from `resolve_paths.rs`; binding and Verification updated.
6. `glob_scope.rs` and `exports.rs` ticks; T2, T3, T4 in a new `src/exports/tests.rs` with the exact linear-growth assertion over N, 2N, 3N; removed from `exported_items.rs`; bindings and Verification updated.
7. T9 and T10 in `module_tree.rs` through the counting tree; binding and Verification updated.
8. T6 in `prescan_budgets.rs` (renamed; the parser-ran flag); binding and Verification updated.
9. The watchdogs removed (`tests/support/mod.rs`, `disk_tree_containment.rs`, `workspace_containment.rs`, `src/tree/disk/tests.rs`) and the open-flags hook with its assertions added.
10. T14 with the inotify probe and both controls; the Verification text of the blocked writer rewritten; the repository gate of Risk 7 with its fixtures.

## Completeness contract additions
| Requirement | Task | Acceptance |
| --- | --- | --- |
| no test of raf-source measures or asserts wall-clock time | 4b.1 to 4b.10 | `grep -rn "Instant::now\|elapsed()\|Duration::from\|recv_timeout\|thread::sleep" crates/raf-source/src crates/raf-source/tests` finds nothing; the repository gate refuses a planted `Instant::now()` in a fixture and passes the clean neighbour |
| every guarantee the clock tests protected is kept | each | the converted test asserts the count with a lower bound or an exact value, and the recorded mutant of the design fails it (one mutant per test, run in a scratch copy, recorded) |
| capability text free of timing-run figures | 5 | the restated sentences name the counted bounds; `grep -n "ms\b\|seconds\| s)" docs/capabilities/source-index.md` shows no bound that came from a clock |
| `change complete` can run on this machine under the rule | 5 | the capability's bound tests include no clock test; `change complete` runs them and completes |

## Order
Task 4 (oracle, including the 1473 fix) finishes first; then 4b.1 to 4b.10; then Task 5 (capability: both rules, `resolve_from`, the two Constraints bullets, the cycle gap stated as a limit, the restated bounds, bindings for the new tests, reopen and complete with the tool), Task 6 (done: README), Task 7 (handoff).

## 2. paudley — 

Least confident about: (1) the rule-2 answer in glob-cycle shapes: the index gives one fixed answer where rustc's own depends on declaration order, and five further glob-cycle shapes diverge outside the documented run sizes (tracked); (2) the residual order dependence of a companion item that reaches a module before an explicit import supplies it (a debug build's consistency check can stop; the same at the base commit; tracked); (3) the timing conversions count work at shared accessors, so a regression that only allocates would pass; the counts were each checked against a recorded mutant.

What the owner may not know: the larger differential runs found a regression that the second rule had introduced and a pre-existing debug panic in the same area, both fixed; whole-crate test runs by early agents executed the old elapsed-time tests before conversion (no timings recorded); `examples/registry_scan.rs` still has a timeout and a sleep poll and is out of scope; `make check` as a whole is not run locally because timing tests remain in `raf-okf` and `raf-workplan`.

## 3. paudley — 

Correction. An earlier note here, on the tracking issue for the companion-item order dependence, and in the capability text and CHANGELOG entry said the order dependence is the same at the base commit. That is wrong. The comparison was made against the branch head after the second visibility rule, not against the base. Measured against the base: the base answers `m2::B` in all 120 declaration orders of the shape and never stops a debug build; the branch stops a debug build in 20 of the 120 orders and answers differently in a release build in those orders, starting at the commit that models the second rule. The order dependence is therefore a regression of this change, not a pre-existing limit. It is being fixed in this change (the limit sentence and the "holds in every declaration order" claims are corrected with the fix), and the tracking issue is closed when the fix lands.

## 4. paudley — 

Stage 2 summary for this change.

* Review: full path. A gap analysis and two independent completion audits (the exit audit passes at the final head with no required fix), plus independent reviews of every fix. They found, and the change fixed with fixtures that fail on the previous commit: an order dependence and a debug-build stop that the second rule had introduced (an earlier note of mine called it pre-existing; that was wrong and was corrected on the issues), the bare-first-segment case, the shared-crate-name case of `resolve_from`, the oracle's missing check that the index answers the same in every order, a cfg-twin over-refusal, two regressions on a second family of seeds, a quadratic search that a fix introduced, and wrong cost statements.
* Local validation (final code): the oracle runs of both seed families at 5,522 programs (release) and 1,500 (debug, no stop), 11,044 programs per family, the whole `raf-source` suite in debug and release (592 tests), `make fmt clippy doc deny reuse msrv fuzz licenses issue-refs`, the root tests, `workplan check` and every assessment current.
* Hosted CI: 12 of 12 jobs pass on the final head.
* Not run locally: `make check` as a whole and `cargo test --workspace` (elapsed-time tests remain in other crates, tracked; the repository forbids timing runs) and `cross-check` (hosted CI substitutes).
* Owner decisions, quoted in the PR: the settling cost of three legal shapes is its own change; macOS is minimal-effort before the first release.
* Tracked follow-ups: the settling cost, the glob-cycle divergences, the registry-scan example's clock use, platform-specific verification, the timing tests of two other crates.

