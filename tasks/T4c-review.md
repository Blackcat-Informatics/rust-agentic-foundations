# T4c review

VERDICT: BLOCKED (superseded by the re-review below; first-round text kept)

Scope: crates/raf-source/src/glob_scope.rs, crates/raf-source/tests/use_resolution.rs
(new fixture), fuzz/corpus/rustc-oracle/hidden_module_a_glob_cycle_carries_back_beside_the_module_itself.rs.

## Blocking finding (e)

Program 1578 (/tmp/p1578-saved.rs) is not just an answer that differs from rustc once the fix is in.
Added to a scratch clone's corpus as `// oracle: agree`:

- HEAD without the fix: `every_corpus_program_shows_the_divergence_it_records_and_no_other` passes (agrees).
- HEAD plus the three-path change and the staged glob.rs/members.rs changes: the same test PANICS at
  crates/raf-source/src/settle.rs:2044 (`check_settled`, debug builds):
  "the imports of `probe0` bound {ItemId(6), ItemId(9)} and reach {ItemId(9)} in the settled tables".

So the queue-time tables bind the import to two items (the firm m0::C plus a companion) while the
rebuilt tables, which read through `delivered`, drop the companion. Every oracle run that draws this
shape panics in debug. The fixer's note ("resolves to m0::C", "known-open, import-cycle gap")
describes the settled answer but not the invariant panic.

Cause, from the diff: `arrive`/`gather` now recompute through `export_ids`, but `add_to_view` only
adds. A companion that reached the view before the firm item arrived through another glob stays in
the view; the recomputed set is smaller, and the union keeps the old companion. The settled tables,
built by `delivered`, do drop it. rustc itself finds no `B` at all here (E0425 at root and
m1::s0), and a companion-free variant (m0's private glob of m1::s1 and m0::s0's `B` removed) makes
rustc find `B` as m0::C. So the hole is partly pre-existing (the index and rustc already differ in
which items survive), but the debug panic is new and comes from this change. Confidence 80 percent
that the panic is attributable to the retained-companion mechanism and not to something else.

Needed: make the view drop a companion a later arrival makes redundant (or make settle-time binding
read through `delivered`), or classify and record 1578 in the corpus so the replay and generated
runs do not panic; then re-run 1578 and the generated oracle by whoever owns the full runs.

## (a) Monotone / order independent

`export_ids`/`delivered` read only the views, which only grow; `add_to_view` only adds, so the
fixture's 720 orders x 2 `s0` forms agree. The caveat is the non-monotone input: `delivered` drops
companions when another glob later delivers the firm item, so a recompute can shrink while the
view cannot. That is the 1578 failure above; it is not shown by the fixture's shape.

## (b) Fixture vs rustc

Compiled 80 random declaration orders (two seeds x 40 x both `s0` forms; shuffled top-level blocks
and the m2 body) with `rustc --edition 2024 --crate-type lib --emit=metadata`, one probe per
compile, in the reading module:
- m3::s0, root, m0::s0: type `crate::s0::C` / `s0`: resolves (E0573 expected-type-found-other, not
  E0659), value `crate::s0::C`/`self::s0::C` compiles. No type `C`, value `C` is m2's fn, `s0` is m0::s0.
- m2 `self::s0`: E0659 (ambiguous) in every order.
This matches the asserted answers. Fixture fails without the fix: scratch clone of HEAD
(--no-hardlinks) with only the new test applied fails; four reads answer ambiguous instead of the
asserted values.

## (c) Corpus header

`// oracle: agree` is right; the program body equals /tmp/t4o/five/p1473.rs; the corpus replay
test passes in the worktree.

## (d) Regressions

Worktree: `cargo test -p raf-source --locked --test use_resolution --test exported_items
--test resolve_paths --test rustc_oracle -- <skips>`: 92 + 39 + 10 + 5 passed, 0 failed.
`cargo fmt --check -p raf-source` clean. `cargo clippy -p raf-source --locked --all-targets -- -D warnings` clean.
(The default seeds pass; 1578 is outside them.)

## (f) Repo rules

No timing, issue/PR numbers or process words in the added text; SPDX untouched in the source file;
comments accurate. Nit: in `arrive` the two `export_ids` branches have identical bodies and could
be one condition. Fixture comment is accurate.

## Process note

I ran `rm -rf` once on a path that did not exist (no effect) before cloning the scratch; nothing
was removed. The worktree was not mutated; scratch is /tmp/rev/scratch.


---

# Re-review of the updated change (final)

VERDICT: BLOCKED

One inaccurate comment in the added fixture blocks; everything else passes. The fix is a one-sentence
change, then PASS.

## Blocking: the new fixture's comment misstates rustc

`an_item_a_glob_cycle_carries_back_through_the_module_itself_keeps_its_companion_in_every_order`
says rustc "finds no `B` in the root or in `m1::s0` (E0425) and refuses `B` in `m0` as ambiguous
(E0659)", "a known gap". That is true of program 1578, not of this fixture's program. I compiled the
fixture's program (with both `use self::B as probe;` imports, several orders) with rustc 2024: `B` in
the root, in `m1::s0` and in `m0` is E0659 ambiguous, and `crate::probe` is E0659 too. So the index's
asserted answers (everything ambiguous between `m1::s1::B` and `m2::B`) agree with rustc here, and the
comment must not claim a gap. Reword it (the fixture then documents an agreeing shape; it may also
be worth recording the program in the corpus as `// oracle: agree`).

## (a) Rule and doc comments

Coherent. `delivered` now lets a read justify dropping companions only when its target binds the name
itself (`scopes[target]` definition or explicit import: `own`), so an item carried back through the
module itself cannot drop the companions it was passed on with. The module header, the `delivered`
doc and the `arrive` comment agree with the code. Views still only grow (`add_to_view` untouched);
`delivered` is read-only. Focused binaries with the skips: use_resolution 93, exported_items 39,
resolve_paths 10, rustc_oracle 5, all pass (1473 fixture and the original fixtures hold). fmt clean;
clippy `-D warnings` clean (one transient error in workspace_containment came from another agent's
concurrent edit and was gone on the next run).

## (b) New fixture on HEAD

Scratch clone of HEAD with only the two test hunks applied: the new fixture FAILS with
`settle.rs:2044: the imports of `probe` bound {ItemId(1), ItemId(5)} and reach {ItemId(1)} in the
settled tables`. With the change it passes. Its asserted ambiguity answers match rustc (see above)
and the index's stated behaviour; only the comment is wrong.

## (c) Residual risk is real, but pre-existing and not worsened

Shape (no glob cycle at all), in `demo`:

    pub mod m1 { pub mod s1 { pub struct B {} } }
    pub mod m2 { pub struct B {} }
    pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*; }
    pub mod w  { pub use crate::m2::B; }
    pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B as probe; }

Over the 120 block orders (inner order fixed), with the full change:
- debug build: 20 of 120 orders panic at settle.rs:2044 (the same invariant);
- release build: two different answers across orders for `demo::mm`: `self::B` = m2::B always, and
  `self::probe` either `m2::B` or ambiguous between `m1::s1::B` and `m2::B`.
So a companion that arrives at `mm` before `w` supplies its item via an explicit import stays in the
view. HEAD without the change gives the identical 20/120. rustc says E0659 for `use self::B as probe`
here (it does not drop), so the shape is also an oracle divergence. Variants `w` = `pub use m2::*; pub use mm::*`
and `w` = `pub use mm::*` panic 20/120 at HEAD and 0/120 with the change (the analyst's rule fixes those).
Variants with `w` = `pub use m0::B` / `mm::B` are clean either way.
Needed: either record this shape and its owner (it is the same companion-retained-after-drop mechanism,
now narrowed to an explicit import in the delivering module) or fix it; it is not introduced here.
My scratch test (not committed) is /tmp/rev/ur_fixed.rs, `zz_residual_a`.

## (d)

Not rerun, as instructed.
