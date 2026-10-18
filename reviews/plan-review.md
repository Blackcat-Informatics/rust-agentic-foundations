# Plan final review (Stage 1)

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/14-raf-source-model-rustc-s-visibility-of/.stage/raf-source-model-rustc-s-visibility-of/plan.md` (Revision 1, lines 53-80, supersedes lines 1-51; Revision 2, lines 82-92, supersedes Revision 1 where they differ).

Reports examined: `reviews/plan-completeness.md` (F1-F7), `plan-constraints.md` (C1-C7), `plan-design.md` (B1-B5, A1-A4), `plan-enhancement.md` (items 1-12), `plan-adversary.md` (F1-F4); `issue-analysis.md` (R1-R7, NJ-1..3). Dispositions are the Revision 1 and Revision 2 text (no separate disposition file exists). Claims were checked against `crates/raf-source/src/{settle,glob_scope,exports,workspace}.rs`, `crates/raf-source/tests/{use_resolution,corpus_replay}.rs`, `tests/support/`, `docs/capabilities/source-index.md`, `Makefile`, `README.md`, `crates/cargo-workplan/src/{cli,command}.rs`, `crates/raf-okf/tests/validate_cost.rs`, `fuzz/`.

## Pass 1 disposition check (Revision 1)

| Finding | Revision 1 text | Verified |
| --- | --- | --- |
| Compl F1, Adv F1, Adv F3 (oracle power, run size) | rows 1 and 4: at least 5,522 programs, one divergence per rule on the unfixed resolver, shape counts printed, corpus under `fuzz/corpus/rustc-oracle/` | yes; `fuzz/.gitignore` ignores only `target/`, `artifacts/`, `coverage/`, so the corpus commits; `make fuzz` iterates three named targets, so the new directory is inert there |
| Compl F2, Adv F2 (fixture observables) | rows 2 and 3: `NotFound` for the reader and the item for `m`; `Ambiguous` listing both; `one_outcome_in_every_order` | yes (helper exists, use_resolution.rs:459) |
| Compl F3 (per-task steps, test failing at its own head) | preamble line 55; Task 1 "asserts each still diverges" | yes; lifecycle wording settled in Revision 2 |
| Compl F4, Constr C1, Design B5 (skip) | row 1 and Task 1 last sentence: failure, names the tool | yes |
| Compl F5 (rule-2 neighbour; re-check existing fixtures against rustc) | row 3 neighbour adopted; re-check added in Revision 2 | yes after Revision 2 |
| Compl F6, Constr C3 (lifecycle gate) | row 5: `implementing`, PROPOSED blocks, INVENTORY same commit, `docs sync` twice, reconcile empty, complete | yes |
| Compl F7 (README check) | row 6 | yes; README.md:25-27 and `cli.rs` subcommands agree today |
| Constr C2, Enh 8, Adv F4 (second Constraints bullet, `exports.rs`) | Corrected evidence bullet 1; rows 2, 3, 5 | yes; bullet at source-index.md:1677-1682 and `exports.rs:463` as stated |
| Constr C4 (`make check` before handoff) | row 7, Task 7; gate list in Revision 2 | yes after Revision 2 |
| Constr C5 (comparator negative fixture) | row "the comparator is falsifiable" | yes |
| Constr C6 (timing) | preamble line 55; Revision 2 item 4 (no `within_watchdog`) | yes |
| Constr C7 (names, SPDX, support module) | Task 1 (revised) | yes |
| Design B1 (viewer-aware query) | Revision 2: `Workspace::resolve_from` decided | yes |
| Design B2 (rule-1 cap) | "Rule 1 cap" bullet | yes; matches `vis_of`/`allowed` at glob_scope.rs:232-318 |
| Design B3 (four homes) | "Rule 2 homes" bullet | yes; `seen_by` settle.rs:519, `export_ids` 426, `make_view` 533, `arrive` 879 |
| Design B4 (E0573 retraction) | "Rule 2 stance" bullet; cites use_resolution.rs:3068 | yes, citation correct |
| Design A1-A4 | Task 1 two-phase compile; `RAF_ORACLE_PROGRAMS`; helper named for rule 1; `hit.last` None cases | yes |
| Enh 1, 2 (TRANSFORMATION) | Declined with reason and cost | reason holds (settle.rs is 2940 lines); test count corrected in Revision 2 (77 functions) |
| Enh 3, 6 | adopted in part (corpus, classifier, tags, two-phase compile, `-A warnings`) | yes |
| Enh 9 (documented-divergence class) | Revision 2 bullet 2 | yes |
| Enh 4, 5, 10, 11, 12 taken; 7 declined (cost M, reason, revisit trigger) | Revision 2 bullets 7-8 | yes |

## Pass 1 findings (kept for the record)

### BLOCKING 1: the rule-2 rule contradicts a committed every-order fixture that claims rustc agreement

Plan line (Corrected evidence, "Rule 2 homes"): "`seen_by`, `export_ids`, the duplicate loop in `make_view`, and `arrive` all apply one rule: export the full candidate set when it has more than one item and the reading class sees at least one of them (monotone)."

Code: `use_resolution.rs:2837-2860`, `an_import_a_cycle_leaves_to_the_globs_binds_what_its_readers_could_access_in_every_order`: `pub mod m2 { use crate::m0::*; pub use crate::m3::Y; pub use crate::m1::*; }`, `pub mod m3 { pub use crate::m2::Y; }`, `m0::Y` and `m1::Y` distinct structs. It asserts `demo::m3::Y` resolves to `demo::m1::Y` in all 24 orders, and its comment says "rustc accepts the crate with both names `m1::Y`". Under the Revision 1 rule, `m3`'s named import reads `m2`'s globbed `Bound` `{m0::Y, m1::Y}` through `seen_by` with viewer `m3`, which sees `m1::Y`, so the whole `Bound` is returned and the answer becomes `Ambiguous`. `cycles_through_private_globs_resolve_as_each_reader_can_access_them_in_every_order` (2956-2997, "Two programs rustc accepts") rests on the same per-reader filtering.

### BLOCKING 2: the classifier has no class for a divergence the Contract already documents

`docs/capabilities/source-index.md:1208-1211` documents rustc warning (`ambiguous_glob_reexports`) and resolving where the index answers `Ambiguous`; `use_resolution.rs:3060-3113` is such a program built from the generator's vocabulary. With "unclassified fails the test", row 4's zero was unreachable without narrowing the generator.

### ADVISORY A1-A6

A1 corpus lifecycle stated two ways; A2 `resolve_from` written as a condition already decided (workspace.rs:373-401 has no viewer); A3 row 7 claimed a `make check` exception the Makefile does not provide (Makefile:121-133; `validate_cost.rs:71-75` asserts elapsed time); A4 row 3 `exported_items` "likewise"; A5 enhancement items 4, 5, 7, 10, 11, 12 undisposed and the "about 1000 tests" figure; A6 `support::within_watchdog` (support/mod.rs:22,107-114) is a 10 s wall-clock timeout.

## Recheck (Revision 2)

| Pass 1 finding | Status | Evidence in Revision 2 |
| --- | --- | --- |
| BLOCKING 1 (rule 2 vs fixture 2837) | DISCHARGED | Bullet 1: Task 3 compiles each existing fixture crate holding an inaccessible-glob collision with the Task 1 harness in every order before any rule is written; the rule is the exact condition under which rustc refuses; contradicted fixtures are corrected with the reason in their comment, confirmed ones stay as neighbours. The order-dependent case falls under the standing stance at use_resolution.rs:3068-3070, which Revision 1's "Rule 2 stance" already adopts. |
| BLOCKING 2 (documented class) | DISCHARGED | Bullet 2: `documented` class tied to the Contract sentence; `-A warnings` then `-W` for the two lints; `unclassified` fails, `documented` is counted and printed; the generator is not narrowed. |
| A1 (corpus lifecycle) | DISCHARGED | Bullet 3: the expected-divergence entry is deleted by the fixing task in the same commit; the program stays as a passing replay input; the test asserts each remaining entry still diverges and no unlisted entry diverges. Rows 2-4 are now satisfiable. |
| A2 (`resolve_from`) | DISCHARGED | Bullet 4: decided, on the `Heads::Use` walk, documented, fixtured with a neighbour, bound in Task 5; the oracle uses only public entry points. |
| A3 (gate claim) | DISCHARGED | Bullet 5: gates listed individually; `cargo test --workspace --locked -- --skip within_the_bound` matches exactly the three timing tests in `validate_cost.rs` (lines 88, 102, 154, all named `..._validates_within_the_bound`) and nothing else; `cargo test -p raf-source --release --locked` is `test-release` without the probe. |
| A4 (`exported_items` assertion) | DISCHARGED | Bullet 6: `Err(ExportError::Resolve(ResolveError::Ambiguous(_)))` for rule 2, absence for rule 1 (matches exports.rs:1394-1395). |
| A5 (enhancement dispositions, test count) | DISCHARGED | Bullets 7-9: 10, 11, 4, 5, 12 taken with concrete fixtures or records; 7 declined with cost M, a reason and a revisit trigger; 77 test functions. |
| A6 (`within_watchdog`) | DISCHARGED | Bullet 8, item 4: the rustc child does not use `within_watchdog`. |

No new contradictions introduced: Revision 2 bullet 1 supersedes row 3's fixed `Ambiguous` expectation only where rustc's recorded answer differs, which is the intended precedence; bullet 3 is consistent with rows 2-4; bullet 5 is consistent with row 7's second sentence (default seed list in debug and release). One note, not a finding: item 12 records the oracle counts "in the Verification section of the capability"; if that section is the generated matrix, the counts belong in the Contract text beside the existing measured counts so `docs sync` does not overwrite them. Task 5's "`docs sync` twice with no diff" will surface this if it is wrong.

## Open user decisions

None blocking. Two decisions were taken in Revision 2 that the user may wish to confirm: (1) the gate at handoff skips the three `validate_cost` timing tests on this machine and lists the remaining gates individually, leaving `make check` in full to hosted CI; (2) `Workspace::resolve_from(module, path)` joins the public surface of the delivered `source-index` capability.

VERDICT: PASS
