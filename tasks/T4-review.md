# T4 review (uncommitted change: settling fix, Cycle class, fixtures, corpus)

VERDICT: PASS

Nothing in the worktree was modified except this file. Scratch material is under /tmp/t4rev (scripts ord.py, f1149.py, f1023.py; a clone of HEAD with --no-hardlinks and its own target dir).

## Reviewed scope
crates/raf-source/src/settle.rs (blocked_for, pending_sources_for), tests/rustc_oracle.rs (Cycle class), tests/use_resolution.rs (two every-order fixtures), the four untracked corpus files.

## (a) Soundness and monotonicity of settle.rs
* `globbed_any(module, name, at)` reads `globs.views[read.view].names[name][at]` for the reads of the module's own globs. In glob_scope.rs a view's name lists are only ever inserted into (`entry().or_default()`, `lists[at].insert`, or overwritten at the same slot with a level); there is no remove/retain/clear of a name list except the wholesale `clear_globs` in the rebuild of `settle_agreed`, which restarts the settle from the definitions. So within a settle `globbed_any` goes false to true only, never true to false. The new term therefore cannot flip blocked_for from true to false by itself; the only true-to-false flips of `blocked_for` are the existing ones (the import becoming free/decided: `use_free` and `use_decided` only grow). The false-to-true flip (a glob later brings the name in while an unnameable import is undecided) is the same shape as the already accepted visible-import case, and the waiters are woken through the same `touch`/`wake` paths. Order independence: the condition depends only on the grown tables, not on the order of arrival.
* The new term makes `blocked_for` agree with `blocked_in`, which already holds the name back from every importer (export_ids, make_view, arrive all test `blocked_in` without a visibility filter); before, the glob-side test (visible import only) disagreed with the view-side test, which is the 1149 over-answer.
* `pending_sources_for` returns early when `defs[at]` holds an entry with `origin.link.is_none()` (a definition, not an import or glob binding). Bindings are only added, so that also never reverts. Rule: an item defined in a module shadows what its globs bring in for every reader (the reader's visibility filters the item afterwards, it does not reveal the glob binding); this is rustc's behaviour and the corpus entry `private_item_beside_a_private_import...` (private struct beside a private import and globs) is accepted by rustc identically in 24 random orders for all three probes. Sound.

## (b) 1149 fixture against rustc 1.101.0-nightly
Script f1149.py: crate with the fixture's declarations, every block shuffled at every level (root, m0, s0, s1, m2), 40 random orders per variant, probes appended in the reading module (`type _T = path;` plus per-candidate type-identity functions for TYPE; `fn _f() { let _ = path; }` plus `let _: fn() = path` for VALUE; no const eval), errors attributed to the probe line. Both variants: every one of the nine reads gave one answer in all 40 orders, equal to the asserted list (variant 1: m1::C, fn, m1::C, m1::C, fn, m1::C, refused, refused, fn; variant 2: struct m2::C everywhere, fn, refused at `demo::B`).

## (c) 1023 fixture
Script f1023.py, 80 random orders: `m2` T always refused, V always `s0::A`; `m0` and `m1` V always `s0::A`; `m0` T is `s0::A` in 39 orders and refused in 41; `m1` T the same split. So rustc's answer really varies for the two reads the comment calls order dependent, and the fixture's pins (Ambiguous, one answer in every order) cannot be derived from rustc there; the comment says so. This fixture also passes on HEAD's settle.rs (it pins behaviour, it is not a regression test for 1149).

## (d) Old source
Clone of HEAD with only the new use_resolution.rs: `an_import_of_a_value_a_reader_cannot_name_leaves_the_type_of_its_name_to_the_globs_in_every_order` FAILS (debug check in settle.rs:2031 "the imports of `B` bound {1,2} and reach {1} in the settled tables"); the 1023 test passes there. On the worktree both pass.

## (e) Cycle class and classify
`Class::Cycle` is reached only inside the `Kind::AcceptedButRefused` branch (rustc accepts, index refuses), after `Documented` and before `Order`, and only when an index answer is `Cyclic` and neither answer is `Ambiguous`. Rule 1 and rule 2 divergences are index-permissive ones (`RefusedButResolves`, `OtherItem`) handled after that branch, so they cannot be classified Cycle; a rustc-rejected probe never reaches it either. Not swallowed: an ambiguity in the same program (`!ambiguous`), documented lint case (checked first).
Residual (R1, non-blocking): `compare` reports AcceptedButRefused if either name space is, so a probe with `Cyclic` in one space and a wrong item (`OtherItem`) in the other is labelled `cycle` and the second divergence is hidden. Narrow. Suggested hardening: classify as Cycle only when the other space is not `OtherItem` (agrees with rustc's space).

## (f) Corpus headers
All four compile under rustc and give one answer in 24 random orders each (probe results as (type tag, value tag)): value_import_beside... (3,6) agree; private_item_beside... (None,7), (6,7), (2,7) agree; import_cycle_in_one_name_space... (3,None) agree; import_cycle_through_a_named_import... (None,7) cycle. The replay test asserts the recorded class for every entry. For the `cycle` entry I flipped its header to `agree` in the clone: the index answer is `ty: Cyclic, value: Item 7 (demo::m1::s1::B)`, rustc `ty: None, value: 7`; class Cycle confirmed.
R2 (non-blocking): that is the index answering Cyclic in a space where rustc finds nothing, while the value is found; the module doc and the `Cycle` doc say "rustc settles the cycle and accepts it / resolves the name", which is true of the probe but not of the type space. Consider wording "accepts the probe".

## (g) Repository rules
Added lines and the four corpus files grep clean for Instant, sleep, timeout, Duration, elapsed, issue/PR/task/revision/stage/review words, attribution. `make issue-refs` passes (tracked files only; untracked checked by hand). Comments checked against the code: blocked_for, pending_sources and pending_sources_for docs are accurate; fixture comments match the asserted lists (verified in b and c).
R3 (non-blocking, repository rule): the `Cycle` doc says the divergences stay "until the index resolves such cycles as rustc does". AGENTS.md section 7 forbids "later/until" style deferral without another tracked change with its own record; the class is a recorded gap, so a tracked change for it should exist before this one completes, and the wording could state the gap without the open-ended "until".

## Gates run
* `cargo fmt --all --check`: clean.
* `cargo clippy -p raf-source --locked --all-targets -- -D warnings`: clean.
* `cargo test -p raf-source --locked --test use_resolution --test rustc_oracle -- <skips>`: rustc_oracle 5 passed, 1 ignored; use_resolution 91 passed, 0 failed. No timing test and no RAF_ORACLE_PROGRAMS run.

## Report
No blocking finding. Non-blocking: R1 (Cycle can hide a second-space OtherItem in the same probe), R2 (doc precision), R3 (open-ended "until" and the need for a tracked change for the Cycle gap).
