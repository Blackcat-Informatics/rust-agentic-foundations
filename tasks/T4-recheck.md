# T4 recheck of required findings R1-R8 at 973d7ab (contains df26e24)

## Scope
Committed code at 973d7ab, in the clone `scratch/t4-recheck` (detached, clean after every mutant). The worktree was not touched. No timing runs.
Code: `crates/raf-workplan/src/lifecycle/touched.rs` (compare loop at 355-405, `Key{path,kind,header}` at 208-330, `impl_headers` at 455). Tests: `crates/raf-workplan/tests/complete.rs`, `crates/cargo-workplan/tests/reconcile.rs`. Text: `docs/capabilities/workplan-strict.md`.

## Findings
| ID | Result | Evidence |
| --- | --- | --- |
| R1 | FIXED | A merge-base unit missing from the working-tree surface but still an item of its path and kind is reported `Touch::Hidden` (touched.rs ~385-390). Tests `under_strict_a_bound_item_made_private_or_visible_to_its_crate_alone_refuses_...` (P04/P03), `..._a_bound_module_made_private_...` (P05), `..._a_public_method_of_a_bound_type_made_private_...` (P29): ok. Mutant (Hidden branch replaced by no-op): those 3 tests FAIL. |
| R2 | FIXED | The innermost impl header's tokens are in `Key.header`. Test `..._a_changed_header_of_the_impl_block_of_a_bound_type_refuses_for_its_member_...` (P02/P30) ok. Mutant (header dropped from the key): that test FAILS. |
| R3 | FIXED | Units compare by key to their own counterpart only. Test `..._a_method_edited_to_the_tokens_of_a_sibling_method_refuses_...` (P13) ok. Mutant (equal to any merge-base stream is untouched): that test FAILS. |
| R4 | FIXED | Removed members come from a merge-base index (BaseTree), keyed by self-type path. Test `..._a_method_removed_from_an_impl_in_another_file_or_with_its_file_refuses_...` (P11c, P11d) ok. Mutant (Removed branch no-op): 4 tests FAIL including it. The wrong binding-missing sentence is gone (grep: no `binding-missing` in workplan-strict.md). |
| R5 | FIXED | Text now says whitespace/line breaks/plain comments are free and that a trailing comma or doc-comment text is a token (workplan-strict.md ~292, ~766). Test `..._a_reflow_that_adds_a_trailing_comma_or_a_space_in_a_doc_comment_...` ok; `under_strict_a_reformatted_bound_body_and_a_plain_comment_complete` ok. |
| R6 | FIXED | The reconcile Contract names the obligation/lifecycle/git bindings (workplan-reconcile.md:12-31). The assessment now has 13 symbol inputs including modules `raf_workplan::obligation`, `::lifecycle`, `::git`, `Needed`, `ObligationKind`, `Impact`. `assess --list`: no workplan-reconcile or workplan-strict assessment is not `current`. |
| R7 | FIXED | MA `..._the_bound_items_of_a_file_absent_at_the_merge_base_...`; MB `..._a_private_helper_removed_from_a_bound_module_...`; MC `..._a_trait_impl_member_of_a_bound_type_edited_or_removed_...` (decided: judged); MD `..._a_bound_package_whose_surface_cannot_be_listed_is_an_error_...` plus CLI `reconcile::under_strict_a_bound_package_whose_surface_cannot_be_listed_is_exit_two`. All ok. The R4 mutant also killed the MB and MC tests. |
| R8 | FIXED | `..._a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none` (recording runner) ok; the unit test on the one-listing BaseTree is in the 6 `touched` unit tests, ok. The Constraints state the linear cost (workplan-strict.md ~611-615), and the Contract states the one `ls-tree` and one `cat-file` per file. |

## Advisories
* Trait-impl members of bound types: judged, tested, stated (workplan-strict.md ~282, ~606-610). Foreign self types and empty trait impls are stated as unjudged.
* Retargeted `pub use`: judged (`Retargeted`), tested, stated at ~277-278.
* Macro-generated items: stated as a limit (~606-607) and pinned by `..._an_edit_of_a_macro_that_generates_an_item_...is_not_judged`. Truthful.

## Existing conditions
Full `complete` test binary: 64 passed, 0 failed (includes the existing obligation fixtures, the 13/15-kind tests, reasons, Usage/Examples, failing-test and tense cases). CLI reconcile: 11 passed, 0 failed.

## Commands (all in the clone, `--locked`)
* `cargo test -p raf-workplan --test complete`: exit 0, 64 passed.
* `... --test complete under_strict`: 27 passed (filter ran >0).
* `cargo test -p raf-workplan --lib touched`: 6 passed.
* `cargo test -p cargo-workplan --test reconcile`: 11 passed; filter `surface`: 1 passed.
* Mutants (each applied alone, restored with `git checkout -- .`, clean): R1 3 failed; R3 1 failed; R4 4 failed; R2 1 failed. Two first-try mutants did not compile (unreachable pattern, unused variable) and were redone.
* `cargo workplan assess --list`: no stale reconcile/strict assessment. `cargo workplan check`: exit 0.
* Not rerun here: full-workspace gates, `make` targets (validation.md claims them; outside this recheck).

VERDICT: PASS
