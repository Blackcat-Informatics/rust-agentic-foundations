# Review of the Stage 2 fixes G1, G2, G3 and G6

Scope: `fc21ada..1c5a765`. The commits are ed436a9 (G2 bare first segment, G3 shared crate
names), 92cc2f4 (G6 `index-order`) and 1c5a765 (G1 companion order dependence). Note that
1c5a765's parent is 92cc2f4, not fc21ada, so the fixtures were run against both.

Method: scratch clones under `/tmp/greview-CHYO6y/` (`head` = 1c5a765, `base` = 92cc2f4 with
HEAD's tests, `fc` = fc21ada, `main16` = origin/main 16ffd2f), all made with
`git clone --no-hardlinks`. I did not mutate the worktree and did not run the oracle. The
probe is a scratch example (`scripts/greview_probe*.rs`): it loads a program through
`MemoryTree` and asks `resolve_from_in` (or `resolve_in` for canonical paths) in the type name
space. A panic counts as an answer. The driver (`scripts/drive.py`) shuffles the top-level blocks
and the bodies of the chosen modules, either in all orders or in seeded random ones. For each probe
and order it compiles `use <path> as __gp;` with `rustc 1.101.0-nightly (21b707e3f 2026-09-30)`,
`--crate-type lib --emit=metadata`. If rustc accepts, it adds an identity line
`const _: fn(__gp) -> crate::<index answer> = |x| x;`. Only errors on those lines count. The specs
are in `specs/*/spec.py`.

Gates run in the scratch clone at HEAD: `cargo fmt --all --check` exit 0, and
`cargo clippy -p raf-source --locked --all-targets -- -D warnings` exit 0. I ran
`cargo test -p raf-source --locked --test use_resolution --test exported_items --test resolve_paths`
in debug and release. Results: 96, 40 and 15 passed, 0 failed, in both builds.

## (b) The G1 fixtures fail before and pass after

| Fixture | 92cc2f4 debug | 92cc2f4 release | fc21ada debug | HEAD debug and release |
|---|---|---|---|---|
| `an_item_an_explicit_import_supplies_beside_a_glob_that_carries_it_with_a_companion_is_one_answer_in_every_order` (use_resolution.rs:4251) | panic settle.rs:2047 (`probe` bound {2,4} reach {4}) | answers differ (`ambiguous [m1::s1::B, m2::B]`) | panic settle.rs:2047 | pass |
| `a_crate_visible_import_or_glob_that_supplies_an_item_beside_its_companion_...` (:4288) | panic settle.rs:2047 | answers differ | panic | pass |
| `a_companion_a_glob_cycle_carries_beside_an_explicit_import_...` (:4332) | panic settle.rs:2047 | answers differ (also `rr`) | panic | pass |
| `a_surface_refused_for_a_module_name_a_glob_cycle_carries_lists_the_same_candidates_in_every_order` (exported_items.rs) | candidates `[m1::s1, m2::s1, m3::s1]` vs `[m1::s1, m3::s1]` | same | same | pass |

rustc results for the main shape:

- All 120 block orders: rustc accepts 60, and in every one of them the identity line confirms
  `m2::B`. It refuses the other 60 with E0659.
- 40 seeded orders with the bodies of `m0` and `mm` shuffled as well: 28 accept `m2::B` and 12 give
  E0659.
- The index answers `m2::B` in all 160.
- The exported_items fixture's crate compiles in 16 of 16 random orders, and every run warns
  `ambiguous_glob_reexports`, as the fixture comment says.

## (a) The G1 mechanism

What holds:

- **Termination and the bound.** Each pass through `settle_agreed` either adds at least one held
  import or records at least one new drop (`recorded` counts only new keys, glob_scope.rs:785-803).
  So settle k obeys k-1 <= held + drops <= `uses.len() + drop_count()`. The check at settle.rs:1067
  is therefore sound, and drops recorded in a second settle that need a third are covered. The drop
  keys (glob, target, name, space) are finite, so the loop terminates.
- **Monotone and deterministic for given tables.** The record only grows: `clear_globs` keeps it,
  glob_scope.rs:380. The iteration order is fixed (modules in order, then a `BTreeSet` of names).
- **Budget.** Each record entry is charged through `meter.items` before it is inserted
  (glob_scope.rs:775-797). Each extra settle is charged `settle_bytes` (settle.rs:1994). A hostile
  input therefore cannot multiply settles or drops past the index-bytes budget, on the same footing
  as held imports. The uncharged temporaries are documented and are bounded by the charged views.

The problems are that a drop and a hold are never re-validated against the final tables, and
`check_settled` skips held imports. Two consequences follow, and neither is checked in any build.

### F1 (HIGH, blocking): the module variant of the companion shape still depends on order, in debug and release

`specs/g1v1`: the companion shape with `B` a module, `pub mod m2 { pub mod B { pub struct X {} } }`,
and with `use self::B::X as probe;` in `mm`, over all 120 block orders:

| Build | `demo::mm::probe` | `self::B::X` from `mm` |
|---|---|---|
| origin/main 16ffd2f (`resolve_in`) | `m2::B::X` 120/120 | `m2::B::X` 120/120 |
| fc21ada, 92cc2f4 | `m2::B::X` 100, `not found` 20 | `m2::B::X` 120 |
| HEAD, debug and release, editions 2021 and 2024 | `m2::B::X` 100, `not found` 20 | `m2::B::X` 120 |

How the 20 orders arise:

- The import's walk is ambiguous at the middle segment while the companion is still delivered.
- `settle_named` then holds the import (settle.rs:2231-2236).
- The hold is sticky: `held`/`use_ambiguous` only grow (settle.rs:1979, :1998).
- `check_settled` skips held imports (settle.rs:2024). So no build notices that the final tables
  walk the same path unambiguously, to `m2::B::X`, in all 120 orders.

All 20 `not found` orders are orders in which rustc refuses the crate. Even so, this is index order
dependence, a regression against origin/main, and it contradicts glob_scope.rs:34-37 and Contract
line 1318.

Required fix: a held import must still be ambiguous in the final tables, and the tables must be
settled again otherwise. Alternatively, do not hold on an ambiguity made only of companions a
recorded drop removes. Either way, `check_settled` must also check held imports: a held import
walks ambiguous in the final tables. Add an every-order fixture for this shape.

### F2 (HIGH, blocking): a recorded drop can justify itself and undo rule 2 on a cycle

`specs/g1v3`: `pub mod w { pub use crate::mm::B; }`, where `mm` globs `m0` and `w`. The rest is the
main shape. All 120 orders:

| | `demo::mm::B`, `self::B` in `mm`, `use self::B as probe` | `w::B` | rustc |
|---|---|---|---|
| fc21ada, 92cc2f4 | `Ambiguous(m1::s1::B, m2::B)` 120/120 | Ambiguous | E0659 120/120 (also at w's import), editions 2021 and 2024 |
| HEAD, debug and release | `m2::B` 120/120 | `m2::B` | same |
| origin/main | `m2::B` (before rule 2) | `m2::B` | same |

The branch parent agreed with rustc. HEAD returns to the pre-rule-2 answer.

Likely mechanism: the drop is recorded (glob_scope.rs:729, called at settle.rs:1987) because `w`
binds `B` itself and delivers `m2::B` (`redelivered`, glob_scope.rs:669). But `w`'s import reads
`B` through `mm`, which is the very read being dropped. Replayed from the start, the drop justifies
itself. `record_drops` also runs when `first` is `Some`, that is, on tables that are about to be
discarded because an import is newly held.

Required fix:

- A recorded drop must be justified by the final settled tables, by a binding that does not reach
  `B` through the dropped read.
- Do not record drops from a settle that is being redone for a new held import.
- Add an every-order fixture for this shape that expects the ambiguity.

### F3 (blocking, maintainer evidence, not my run): the oracle fails at its documented sizes

`/tmp/final-oracle-11044.txt`:

- rule1 0, rule2 0, documented 10, order 28, cycle 5, **index-order 1**, **unclassified 3**,
  exit 101.
- The index-order entry is seed 0xa11ce08 program 1309 (`self::B` in `demo::m2::s0`). This is the
  audit's F2 example, and it is still order-dependent: `Ambiguous` as written, `m0::s0::B` in
  shuffle 3. The reduced program is in the log.
- The unclassified entries are 767 (`m3::B`), 858 (`s0::B`) and 1309.

`/tmp/final-oracle-22088.txt`: **rule2 1**, **index-order 3**, **unclassified 7**.

The G1 and G2 acceptance requires zeros. This review assumes those runs were made at 1c5a765.

## (c) G2: the bare first segment (resolve.rs:641)

Each shape was checked against rustc in edition 2024 over 10 seeded orders, and the starred shapes
also in 2021 over 8 orders:

| Shape | Result |
|---|---|
| g2s1\*: `pub` glob and private glob of two `s` | Agree in every order: `s::A`/`s::B` from `m`, `m::k`, `r`, and `self::s::A` from `r`, all Ambiguous; rustc E0659/E0432 |
| g2s3\*: definition `s` beside a glob bringing `s` | Agree: definition wins; `s::B` NotFound, rustc E0432 |
| g2s4: two globs, one module (via `pub use crate::p::s`) | Agree: resolves `p::s::A` |
| g2s5\*: explicit `pub use crate::q::s` shadows two globs | Agree |
| g2s8: settle-time `pub use s::A as X` with an ambiguous `s` | rustc refuses the import (E0659/E0432); the index binds nothing (`self::X` Ambiguous in `m`, NotFound from `r`) |
| g2s9\*: ambiguity two segments deep through a chain of globs | Agree |
| g2s2/g2twin\*: cfg twins carried by a glob | **Disagree, F4** |

### F4 (HIGH, blocking): cfg-twin modules carried by a glob are now refused at a bare first segment

`specs/g2twin` is the idiomatic shape:

```rust
#[cfg(unix)] mod sys { pub struct Fd {} }
#[cfg(not(unix))] mod sys { pub struct Fd {} }
mod tests { use super::*; use sys::Fd as Mine; }
```

| Build | `sys::Fd` from `tests` | `self::Mine` |
|---|---|---|
| fc21ada | `demo::sys::Fd` | `demo::sys::Fd` |
| HEAD | `Ambiguous(demo::sys, demo::sys)` | `Ambiguous` (the settled import binds nothing) |

rustc accepts in 2021 and 2024, every order.

- The same regression shows for `s::A` read from `r` (`use crate::m::*`) and from `m::k`
  (`use super::*`) in `specs/g2s2b`.
- The cause is `from_glob && segs.len() > 1 && bound.len() > 1` (resolve.rs:641), which counts the
  two alternatives as two glob candidates.
- The middle-segment check at resolve.rs:732 refuses `self::sys::Fd` from `tests` too. That part
  already happened at fc21ada.
- The commit message says "Definitions and imports of one name under cfg are alternatives, and no
  longer an ambiguity at any segment of a path". That is false for glob-carried twins.
- `definitions_and_imports_of_one_name_under_cfg_are_alternatives_at_any_segment_of_a_path`
  (resolve_paths.rs:675) covers only twins that are not carried by a glob.

Required fix: count the cfg alternatives of one module and name as one candidate in both glob
checks. Add an every-order fixture for `use super::*` with twins and `use sys::X`, at the start and
in the middle, in a `use` and through `resolve_from`.

## (d) G3: shared crate names

`module_named` (resolve.rs:1336-1362) answers `Ambiguous` with the crate roots whenever
`crate_roots[first]` has more than one root. This mirrors `resolve` (resolve.rs:1236). Only
libraries and build scripts get a bare crate path (target.rs:362-370); a bin gets
`pkg::bin_crate::name`. So a package with both a lib and a bin is not affected. The fixture checks
`resolve` and `resolve_from` agree, all name spaces, a missing module under the shared name, and the
neighbours. PASS.

The Contract text for this ("the Contract states which") belongs to the pending documentation
commit.

## (e) G6: `index-order`

- `index_instability` (rustc_oracle.rs) compares both name spaces plus the ambiguous candidates for
  every probe, and the exported surface. It uses the as-written order and the 12 shuffles of the same
  seed as `rustc_depends_on_order`. `only_probe` keeps `lines`, so the shuffles are the same.
- A panic, caught by `catch_unwind` per order, is its own answer. A surface that panics in every
  order is reported.
- `order` requires `index_stable && order_dependent` at both sites, and `IndexOrder` is not in
  `KNOWN`, so it fails the run.
- The two negative fixtures are meaningful. The first tampers order 3 and expects the report, the
  orders `[0, 3]` and that the class is unknown. The second makes an `order` corpus entry vary and
  expects no `Order` class.
- Limits (LOW): the check runs only on programs `rustc::judge` accepts, and no fixture exercises the
  panic path.

PASS. F3 is this check working.

## (f) Hygiene

- No clock or sleep in the added lines.
- No issue or pull request numbers or process words in the committed text or the commit messages.
- No attribution trailers.
- `tests/support/orders.rs` carries SPDX. The corpus files are covered by `REUSE.toml`
  (`fuzz/corpus/**`).
- LOW: `support::orders` is adopted only by resolve_paths.rs. use_resolution.rs keeps identical
  copies (`permutations` :440, `read_as` :3210, `reads_in_every_order` :3232,
  `ambiguous_between` :3530), the new G1 fixtures use them, and exported_items.rs has its own
  `orders` (:1449). Move them onto the shared helper.

### F5 (MEDIUM): the Contract's stance on order-dependent rustc answers conflicts with G1

`docs/capabilities/source-index.md:1318` says that where rustc's own answer depends on the order,
"the index gives one fixed answer, the ambiguity, in every order". The G1 fixtures fix `m2::B` for a
shape where rustc accepts in 60 of 120 orders, and the fixture comment gives the reason: the answer
of the orders that compile. The Contract and `CHANGELOG.md:20` ("Both rules hold in every
declaration order") must state both stances, or one, in the pending documentation commit. After F1,
the order-independence sentence is not yet true.

First-pass verdict at 1c5a765: BLOCKED. Required were F1, F2, F3 and F4, then F5 in the
documentation commit. The recheck below supersedes this verdict.

# Recheck at 4bae3a7 (90d89a2 and 4bae3a7)

Method:

- A fresh scratch clone, `/tmp/greview2-3IJi5h/head` at 4bae3a7.
- A second fresh clone of 5f5049b, `/tmp/greview2-3IJi5h/pre`, as the comparison base before the
  rules. `/tmp/baseline-pre-rule` was not touched.
- The same probe and driver (`/tmp/greview-CHYO6y/scripts`).
- No worktree edits and no oracle runs.

Gates at 4bae3a7:

- `cargo fmt --all --check`: exit 0.
- `cargo clippy -p raf-source --locked --all-targets -- -D warnings`: exit 0.
- `cargo test -p raf-source --locked --test use_resolution --test exported_items --test resolve_paths`
  in debug and release: 100, 40 and 17 passed, 0 failed.

## The four findings

| Finding | Shape | 4bae3a7 debug | 4bae3a7 release | 5f5049b | rustc 2024 | Status |
|---|---|---|---|---|---|---|
| F1 | module form, `use self::B::X as probe` in `mm`, all 120 orders | `probe`, `self::B::X`, `B::X` = `m2::B::X` 120/120, no panic | same | same | 60 accept (`m2::B::X`), 60 E0659 | FIXED; fixture use_resolution.rs:4365 |
| F2 | `w { pub use crate::mm::B; }`, all 120 orders | `mm::B`, `probe`, `w::B` all `Ambiguous(m1::s1::B, m2::B)` 120/120 | same | `m2::B` 120/120 | E0659 120/120 (at `w` and in `mm`) | FIXED; fixture :4398 |
| F3 | oracle | not rerun (coordinator's tables: 5,522 all zero; 11,044 unclassified 2 = 767 and 858; 22,088 rule2 1 = 2431, unclassified 6, all said to diverge at 5f5049b) | | | | ACCEPTED on that evidence, which I did not reproduce |
| F4 | cfg twins through `use super::*` (`sys::Fd`, `use sys::Fd as Mine`), and through `use crate::m::*` and `m::k` | resolve to `demo::sys::Fd` and `demo::m::s::A` in every order | | | accept | FIXED; `several_symbols` resolve.rs:1343 |
| F4 neighbour | `p::s` and `q::s` through two globs (g2s1, g2s9), twins beside a different `s` (g2s2 `r`) | Ambiguous in every order | | | E0659/E0432 | holds |

## The mechanism

- **Bound `(uses + 1) × (drops + 1)` (settle.rs:1068).** Sound.
  - Within one count of recorded drops, each repeat adds at least one held import, so a phase has at
    most `uses + 1` settles.
  - Drops are recorded only when no import is newly held (settle.rs:1996). Each new phase needs at
    least one new drop, so there are at most `drops + 1` phases.
  - Held imports are reset after a drop (settle.rs:2015), which is why the old additive bound no
    longer holds and the product is the correct one.
  - The loop terminates: the drop keys are finite.
- **Cost.** Each record entry is still charged through `meter.items` before insertion
  (glob_scope.rs:805). Each settle after the first is still charged `settle_bytes` (settle.rs:1994)
  to the item-bytes budget, which never releases. So the up-to-quadratic number of settles is
  bounded by the budget, whatever the input.
  - LOW: `arrive` now recomputes `export_ids` for each hidden arrival. This is an uncharged temporary
    of the size of the name's bindings in the module, and it is not listed among the documented
    uncharged temporaries in the module doc.
- **`check_settled` (settle.rs:2046-2058).** It now requires every held import to walk an ambiguous
  path in the final tables.
  - I found no input that trips it. There was no panic in debug over 3,000+ loads of the shapes
    below, including n1, where one import (`k`'s `x::t::s`) is held for a middle-segment ambiguity
    beside another import that starts with `s`.
  - Held imports are now re-derived after each new drop, so an import held for an ambiguity of
    companions that a drop removes is released. That is the intended case.
  - I found no case of an import that should stay held being released.
- **The F2 rule (glob_scope.rs:697-703).** It is not too strong. If the supporting read also
  delivers one of the companions, the name stays ambiguous through that read anyway.

## Eight new shapes: index at 4bae3a7 vs 5f5049b vs rustc 2024

All 120 or 720 orders where the shape has 5 or 6 blocks, otherwise 80 seeded orders with the module
bodies shuffled. Debug build.

| Shape | 4bae3a7 | 5f5049b | rustc | Note |
|---|---|---|---|---|
| n1: a hold beside another hold (`k { pub use crate::x::t::s; }` with `t` ambiguous, `m` starts with `s`), 720 | stable; `m::X` and `s::A` = `p::s::A`; `k::s` Ambiguous | stable, same | E0659 (crate refused) | pre-existing divergence on a refused crate; no panic |
| n2: private justifying import in `w`, 120 | Ambiguous 120 | `m2::B` | E0659 120 | improved |
| n3: two companions from two private globs, a reader `rr`, 80 | `m2::B` 80 | `m2::B` | 54 accept, 26 E0659 | stable, tolerated `order` |
| n4: `w { pub use crate::m0::B; }`, 120 | Ambiguous 120 | `m2::B` | E0659 120 | improved |
| n5: cycle `m0` and `mm`, with `w` a glob of `m2`, 120 | `mm::B` Ambiguous 120; `m0::B` Ambiguous 120 | `mm::B` = `m2::B`; `m0::B` Ambiguous | `mm`: 60 accept (`m2::B`), 60 E0659; `m0`: accept 120 | stable. `mm` changed from `m2::B` to the fixed ambiguity (tolerated `order`). `m0` diverges at 5f5049b too |
| n6: justifier a child of `mm` that `mm` globs, 80 | `m2::B` 80 | same | 36 accept, 44 E0659 | stable |
| n7: readers of two classes (`pub(in crate::m0)` glob, reader `m0::k`), 80 | `m2::B` 80 | same | 54 accept, 26 E0659 | stable |
| n8: chain of imports through the module form (`probe`, `P2`, `rr::P3`), 80 | `m2::B::X` 80 | same | 54 accept, 26 crate E0659 | stable |

No shape agrees at 5f5049b and varies now. None varies at 4bae3a7 at all, and none panics. The
earlier shapes (main, main with shuffled bodies, g1v2, g1v6, g2s1-g2s9, g2twin) were rerun at
4bae3a7 and are stable and as before.

Hygiene of the two new commits:

- No clock, no issue or pull request numbers, no process words, no attribution.
- New fixtures exist for each shape: use_resolution.rs:4365, :4398, :4429 and :4467, and
  resolve_paths.rs `the_cfg_alternatives_of_one_module_a_glob_brings_in_are_one_candidate_in_every_order`
  and `two_different_modules_of_one_name_stay_ambiguous_beside_the_cfg_alternatives_of_one`.

## Still open

- **F5 (MEDIUM, for the pending documentation commit).** Contract line 1318 states one fixed answer,
  "the ambiguity", where rustc's answer depends on the order. At 4bae3a7 the index gives the
  ambiguity for some such shapes (n5 `mm`) and `m2::B` for others (the main shape, n3, n6, n7). The
  Contract and `CHANGELOG.md:20` must describe this as it is.
- **Documentation.** The documentation must disclose that the oracle test fails at 11,044 and
  22,088 on divergences that predate the rules.
- **LOW.** The `support::orders` duplicates in use_resolution.rs and exported_items.rs remain. There
  is no fixture for the oracle's panic path. The `export_ids` temporary in `arrive` is undocumented.

Verdict at 4bae3a7: PASS for the code through that commit. The delta recheck below supersedes it.

# Delta recheck at 7ba465a (7ba465a `globs_supply`, 9fc8aa3 oracle seeds)

Method:

- Fresh scratch clones under `/tmp/greview3-WHm7Cm/`: `head` = 7ba465a, and `prev` = 9fc8aa3 with
  7ba465a's tests checked out over it.
- The 5f5049b probe from `/tmp/greview2-3IJi5h/pre` as the comparison before the rules.
  `/tmp/baseline-pre-rule` was not touched.
- A probe that asks either name space (`scripts/probe.rs`).
- A new driver for programs in the oracle's line format (`scripts/drive2.py`). It shuffles every
  sibling list, compiles `use <p> as __gp;` with rustc 1.101.0-nightly, and checks identity with the
  program's tags: `__gp::TAG == N` for types, `__gp() == N` for values. For a name space where the
  index finds nothing, a line that needs that name space (`Option<__gp>` or `let _ = __gp`) confirms
  rustc has nothing there either.
- No worktree edits and no oracle runs.

Gates at 7ba465a:

- `cargo fmt --all --check`: exit 0.
- `cargo clippy -p raf-source --locked --all-targets -- -D warnings`: exit 0.
- use_resolution 103, exported_items 40 and resolve_paths 17 pass, in debug and release.
- `rustc_oracle -- the_seed_base_is_read_in_hexadecimal_or_decimal_and_refuses_anything_else`: 1
  passed.

## (1) The three new fixtures

At 9fc8aa3, with 7ba465a's tests, all three fail:

- `an_item_a_glob_cycle_carries_back_...` gives `ambiguous [m1::A, m3::s0::A]`.
- `a_module_a_glob_cycle_carries_back_...` gives `ambiguous [m0::s0, m1::s0]` in both name spaces.
- `an_item_a_glob_brings_in_from_its_definition_...` gives `ambiguous [m1::A, m3::s0::A]`.

At 7ba465a all three pass.

Against rustc, with 25 orders in 2024 and 17 in 2021, every answer the fixtures assert is rustc's,
in every order, with identity confirmed by tag:

- fx1 and fx3a: value `m1::A`; nothing in the type name space.
- fx2: type `m1::s0::B`; nothing in the value name space.
- fx3b: value `m1::B`.

5f5049b gives the same answers.

Seed 0xc0ffee01 program 1344 (regenerated with the committed generator, then reduced to the lines
rustc accepts, 13 orders):

- 7ba465a and 5f5049b agree with rustc on all 10 probe and name-space pairs. `crate::B` from
  `m2::s1` is value `m1::B`.
- 9fc8aa3 answered `ambiguous [m1::B, m2::B]` there.

The nine audit programs in `c2min-red/`, 13 orders each in 2021:

- 7ba465a has the same number of disagreements as 5f5049b on seven of them.
- On the other two (c0ffee03 p0136 and p0437) it has none, where 9fc8aa3 had 2 and 1.
- No program has more disagreements at 7ba465a than at 5f5049b.

## (2) Soundness of `globs_supply`: shapes vs rustc, 9fc8aa3 and 5f5049b

13 or 25 orders each, 2024, debug, both name spaces; the 2021 and release checks of s11 agree.

| Shape | 7ba465a | 9fc8aa3 | 5f5049b | rustc |
|---|---|---|---|---|
| s1: supply only through a private glob of an intermediate `g`, with `g` globbing `m0` | not found | same | same | E0432 13/13 |
| s2: `s1` holds `m1::A` through its own private glob | not found | same | same | E0432 13/13 |
| s3: `pub(crate)` definition, supply two levels deep | `m1::A` | Ambiguous | `m1::A` | accept `m1::A` 13/13 |
| s5: an intermediate module shadows with an explicit `pub use z::A` (a different item) | `z::A` | Ambiguous | `z::A` | accept `z::A` 13/13 |
| s6: `pub(crate)` glob of the definer in `g` | `m1::A` | Ambiguous | `m1::A` | accept 13/13 |
| s7: no cycle, companion a different item rustc keeps (`m3`: private `s0::A` and `pub m1::A`), supply through `s1` | `m1::A` | Ambiguous | `m1::A` | 7 accept `m1::A`, 6 E0659 |
| s7b: s7 with `m0` globbing the definer directly | `m1::A` | `m1::A` | `m1::A` | 11 accept, 2 E0659 |
| s8: s7 with a cycle, a `pub(crate)` glob and a reader inside `m3` | `m1::A` | Ambiguous | `m1::A` | 11 accept, 2 E0659 |
| s9: `g { use crate::m1::*; pub use crate::m3::*; }`, `s1` globs `g` | `m1::A` | Ambiguous | `m1::A` | 7 E0432, 4 E0659, 2 accept |
| s9b: s9 without the private glob in `g` | Ambiguous | Ambiguous | `m1::A` | E0432/E0659 13/13 |
| s10: s9 with `g` globbing `m0` instead of `m3` | `m0::A` Ambiguous, others not found | same | `m0::A` = `m1::A` | E0432/E0659 13/13 |
| s11: supply only through a *private explicit import* in `d`, which `g` globs | Ambiguous | Ambiguous | `m1::A` | E0432/E0659 25/25 |

Results:

- No shape varies with order at 7ba465a.
- No shape panics in debug.
- No shape disagrees with rustc where rustc is stable and 5f5049b agrees.
- Every change from 9fc8aa3 is a move to rustc's stable answer (s3, s5, s6) or back to 5f5049b's
  answer where rustc's own answer varies (s7, s8, s9).

Observation (LOW, not a regression under the stated test): s9 against s9b shows that a private glob
in `g` (`use crate::m1::*`), which passes nothing on to `s1`, decides the drop.

- Mechanism: `g` drops its companion through the older own-binding rule, justified by that private
  read, and exports the result through its `pub` glob. `globs_supply` then finds the definer through
  the same private glob, because it follows every glob without asking whether the glob passes the
  item on.
- rustc refuses in 11 of 13 orders, so the oracle tolerates this as `order`.
- s11 shows that a private explicit import alone does not trigger it.

## (3) The earlier findings at 7ba465a

- F1 module form, all 120 orders, debug and release: `probe`, `self::B::X` and `B::X` =
  `m2::B::X`.
- F2: Ambiguous 120/120.
- F4: cfg twins resolve, and two different `s` stay Ambiguous.
- The main shape is `m2::B` 120/120.
- n1 to n8 are stable. n5 `mm` is now `m2::B`, which is 5f5049b's answer and rustc's in the 60
  orders it accepts. The F5 inconsistency is therefore smaller.

## (4) Cost

- **Unchanged:** the settle bound and the drops record. settle.rs is unchanged since 4bae3a7, and
  the record is still charged before insertion (glob_scope.rs:805 region).
- **F6 (blocking): `globs_supply` makes the work grow with the square of a chain of globs.**
  - `redelivered` (glob_scope.rs:702-724) is called from `delivered` on every read of what the globs
    of a module bring in. It runs the breadth-first search of `globs_supply` (glob_scope.rs:735-757)
    over the whole glob graph reachable from the other read's target. That happens for every pair of
    reads that share an item, where the other read's module does not bind the name itself.
  - This includes the common case where the read has no companions at all, so the drop cannot
    matter.
  - The search is not memoized and is not charged to any budget.

  Measured with `perf stat -e instructions:u` (instruction counts, not time) on the release probe.
  The input is a chain of k pass-through `pub use crate::c_i::*` modules from `h`, and k readers
  that each glob `h` and the end of the chain and import one name:

  | k (= readers) | 9fc8aa3 | 7ba465a | ratio |
  |---|---|---|---|
  | 25 | 14.0 M | 16.0 M | 1.14 |
  | 100 | 54.8 M | 90.0 M | 1.64 |
  | 200 | 110.5 M | 266.8 M | 2.41 |
  | 400 | 225.5 M | 892.1 M | 3.96 |
  | 1000 | 571.2 M | 5,094.3 M | 8.92 |

  - 9fc8aa3 grows linearly. The extra work at 7ba465a grows about four times for each doubling.
  - This contradicts glob_scope.rs:44 ("a chain of globs costs what it brings in, once for each
    link") and the Contract (`source-index.md:1326`, "Neither does the work grow with a chain").
  - Required fix:
    - Test whether the read holds a companion first: `delivered` keeps everything otherwise.
    - Memoize the supply per (target, module, name, name space, item) for the tables being read, or
      precompute it once per settle.
    - Add a work-tally test that a chain of n pass-through globs read by n modules stays linear.
  - The search should also follow only globs that pass the item on to the importer: the
    `pub`/visibility level of the glob and the class. The s9 observation is the visible symptom.

## (5) Seeds and environment (9fc8aa3)

- `parse_seed_base` takes `0x`/`0X` hex or decimal, ignores underscores and trims. It refuses empty
  input, `0x`, non-digits and negatives, and refuses a base that leaves no room for 8 seeds, with
  the variable named in the message. A unit test covers each case.
- `families()`:
  - With no variables set, it runs both bases at 64 programs each.
  - With `RAF_ORACLE_PROGRAMS` set, it runs only 0xa11ce00, so the documented 5,522, 11,044 and
    22,088 runs remain the first family.
  - With `RAF_ORACLE_SEED_BASE` set, it runs only that base.
- `RAF_ORACLE_LIST=1` prints one line per divergence, plus `index-order` lines.
- `write_corpus` is pinned to the first family.
- No clock; no issue or pull request numbers or process words in the added lines or the commit
  messages; no attribution.
- Note: the default run doubles to 128 programs, which costs CI time.

FINAL VERDICT: BLOCKED

Required: F6. Make the supply check run only when the read holds companions and memoize it, or
otherwise keep the work linear in a chain of globs, and add a work-tally test. Recommended in the
same change: limit the search to globs that pass the item on (the s9 observation).

Everything else passes:

- The three fixtures fail before and pass after, and rustc agrees with them in every order.
- The eight shapes and their neighbours (s1 to s11) are stable and show no regression against
  5f5049b.
- F1, F2 and F4 still hold.

F5 is a required item of the pending documentation commit.

# Delta recheck at c8e5187 (F6 fix and the s9 change)

Method:

- A fresh scratch clone, `/tmp/greview4-HhTuQW/head` at c8e5187, with no worktree edits.
  - The worktree has uncommitted documentation changes by another agent. I did not touch them.
- The comparison probes are 7ba465a (`/tmp/greview3-WHm7Cm/head`), 9fc8aa3
  (`/tmp/greview3-WHm7Cm/prev`) and 5f5049b (`/tmp/greview2-3IJi5h/pre`).
- Instruction counts come from `perf stat -e instructions:u`. These are counts, not time; valgrind
  cannot run on this CPU (SIGILL in ld.so).
- Exact search steps come from a scratch in-crate test using `work::measure`. I appended it to
  `glob_scope.rs` in the clone and restored the file with `git checkout` afterwards.

## (1) F6

**Chain without companions.** My generator from the previous pass: a chain of k pass-through
globs and k readers.

| k | 9fc8aa3 | 7ba465a | c8e5187 |
|---|---|---|---|
| 25 | 14.0 M | 16.0 M | 14.0 M |
| 200 | 110.5 M | 266.8 M | 110.7 M |
| 1000 | 571.2 M | 5,094.3 M | 572.4 M |

c8e5187 is linear: 1000/25 = 40.9 times, the same as 9fc8aa3.

**Chain where the readers hold a companion.** This is the fixer's test shape (`a` globs `d` publicly
and `p` privately; n readers glob `a` and the end of the chain).

- Instructions:
  - 7ba465a: 24.2 M, 54.1 M, 132.0 M, 369.4 M, 1,162.8 M for n = 50, 100, 200, 400, 800.
  - c8e5187: 22.1 M, 44.4 M, 88.7 M, 183.4 M, 371.2 M. Linear.
- Search steps (`Work::SupplySteps`): 202, 402, 802 and 1,602 for n = 25, 50, 100, 200. Linear.
- With 8 names: 1,616 to 12,816. Linear, and 8 times one name.
- The answer is `demo::d::X` everywhere.

**F7 (blocking): the worst case is not that input.** The reading module can lie on the supply
chain. My input is the fixer's chain plus a fan-in: the start of the chain also globs every reader,
`pub mod c0 { pub use crate::d::*; pub use crate::r0::*; … pub use crate::r{n-1}::*; }`. This is
the "cycle" input.

- Every reader is then among the modules that the supplier's reads lead to. So
  `holders(…, Some(reader))` (glob_scope.rs `supplied`, the `without` search) runs a fresh search
  of the whole holder graph once for each reader.
- Search steps, one name: 5,408, 20,808, 81,608 and 323,208 for n = 25, 50, 100, 200. That is
  8n^2.
- Search steps, 8 names: 43,264 to 2,585,664.
- Instructions:

| n | 5f5049b | 9fc8aa3 | 7ba465a | c8e5187 |
|---|---|---|---|---|
| 50 | 23.3 M | 41.1 M | 43.3 M | 52.6 M |
| 200 | 94.5 M | 245.1 M | 286.2 M | 720.1 M |
| 400 | 192.8 M | 706.0 M | 888.0 M | 4,170.2 M |
| 800 | 387.6 M | 2,238.8 M | 3,062.2 M | 28,522.9 M |

- On this input c8e5187 is 9 times worse than 7ba465a and 73 times worse than the base before the
  rules. Growth is about 6.8 times per doubling, because the cost per step rises with the size of
  the sets as well.
- This contradicts the claim in the `record_drops` doc and the commit message: "a chain of globs
  costs a step for each link and each item, name and name space asked".
- The unit test does not cover it.
- The answer is correct and stable: `d::X`, as at 5f5049b. rustc accepts `d::X` in 10 of 16 orders
  and refuses 6 with E0659, so the oracle tolerates it as `order`. 9fc8aa3 answered Ambiguous.

**Budget.**

- Steps are charged at `SUPPLY_STEP_BYTES` = 49 bytes each to the item budget (512 MiB by
  default), so the search is bounded at about 11 M steps per load. Hostile input is therefore
  bounded, but the bound allows tens of billions of instructions on the cycle input: n = 800 already
  costs 28.5 G and still loads.
- LOW: the charge is made after the search for each module and name, not before each step. The work
  past the budget is therefore at most the searches of one module and name.

Required fix: answer "does `from` hold the item apart from `reader`" for all readers at once. One
way is to compute dominators once per (item, name, name space) over the holder graph, from a
virtual root at the binders: `from` holds the item apart from `reader` exactly when `reader` does
not dominate `from`. Alternatively, bound the `without` searches some other way. Then add the fan-in
input to the work-tally test.

## (2) The s9 change: drop levels and class-filtered delivery

- **Monotone.** A recorded level only widens (`width(level) < width(known)`). Each widening counts
  as a recorded drop, so the `(imports + 1) × (drops + 1)` bound still holds. Levels lie on a finite
  ancestor chain, so the loop terminates.
- **Order independent.** `delivered(…, Some(class))` drops only for classes that can access the
  level. The module's own lookups (`None`) always drop. The level is decided from the settled
  tables.
- Every shape I reran at c8e5187 is stable in every order tried, in debug and release, editions
  2024 and 2021, with no panic:
  - s1 to s11, 17 orders each.
  - F1 (g1v1) and F2 (g1v3p), all 120 orders.
  - The main shape, g1v2 and g1v6, 120 or 720 orders.
  - F4 (g2twin, g2s2b, g2s1) and g2s3, g2s5, g2s9.
  - n1 to n8, 30 to 720 orders.
- Changes from 7ba465a:
  - s9, s9b and s11 now answer Ambiguous. rustc refuses s9 in 14 of 17 orders and s9b/s11 in 17 of
    17, so all three are now closer to rustc than at 7ba465a.
  - 5f5049b answers `m1::A` on all three, which is wrong where rustc is stable.
- There is no shape where rustc is stable, 5f5049b agrees and c8e5187 does not.

## (3) Fixtures and audit programs

- `use_resolution`, `exported_items` and `resolve_paths` pass in debug and release: 103, 40 and 17.
- The lib tests pass: 83.
- Against rustc over 13 orders each, with no disagreement: fx1, fx2, fx3a, fx3b, program 1344
  (judged), and audit programs c0ffee03 p0136 and p0437.

## (4) The unit test

`the_holders_of_an_item_apart_from_each_reader_of_a_chain_of_globs_are_found_in_steps_linear_in_the_chain`
passes at c8e5187. Against mutants:

- **Holders cache removed:** fails at once (`SupplySteps work passed the ceiling of 8000: 8001`,
  0.02 s).
- **Leads cache removed:** fails at once (same message).
- **Companion guard in `redelivered` removed:** still passes. Every reader in the test holds a
  companion, so the guard is not exercised there. My chain input without companions shows the
  guard working.

## (5) Gates

- `cargo fmt --all --check`: exit 0.
- `cargo clippy -p raf-source --locked --all-targets -- -D warnings`: exit 0.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p raf-source --no-deps --locked` (the Makefile `doc`
  form): clean.
- With `--document-private-items` there are unresolved links in disk.rs, work.rs, parse.rs and
  error.rs. That flag is not part of the gate, and none of those links is in this change.
- No clock, issue numbers, process words or attribution in the added lines.

FINAL VERDICT: BLOCKED

Required: F7. The supply search is quadratic when the readers lie on the supply chain (a fan-in of
globs onto the chain's start), and on that input it is 9 times worse than 7ba465a. Make the
"apart from the reader" question linear (for example with dominators) and add the fan-in input to
the work-tally test.

Everything else passes:

- F6 for the chain and companion-chain inputs.
- The s9 change.
- F1, F2 and F4 still hold.
- The fixtures and audit programs agree with rustc.
- The gates are clean.

F5 remains due in the documentation commit.

# Delta recheck at af0ce46 (F7 fix: dominators, charge before each step, `redelivered` per item)

Method:

- A fresh scratch clone, `/tmp/greview5-POfMZU/head` at af0ce46.
- A second clone, `mut1`, for one mutant.
- Comparison probes: c8e5187 (`/tmp/greview4-HhTuQW/head`) and 5f5049b (`/tmp/greview2-3IJi5h/pre`).
- Instruction counts come from `perf stat -e instructions:u`, release probe.
- Exact `Work::SupplySteps` counts come from a scratch in-crate test. It was appended in the clone
  and removed with `git checkout`.
- No worktree edits and no oracle runs.

## (1) Cost

The fixer's fan-in figures are confirmed (instructions):

| n | 5f5049b | c8e5187 | af0ce46 |
|---|---|---|---|
| 25 | 11.8 M | 21.0 M | 20.0 M |
| 200 | 94.6 M | 720.2 M | 239.7 M |
| 800 | 389.9 M | 28,525 M | 2,004 M |

Nine families of input. The figures are search steps for n = 25, 50, 100, 200, 400, 800, then
instructions at af0ce46 for n = 25, 200, 800:

| Input | Search steps | af0ce46 instructions |
|---|---|---|
| plain chain | 878 to 27,228 | 11.4 / 91 / 383 M |
| fan-in (start globs every reader) | 1,028 to 32,028 | 20.0 / 240 / 2,004 M |
| nested (readers chained) | 1,022 to 32,022 | 15.4 / 126 / 522 M |
| many items, each with its own fan-in (n/10 structures) | 856 to 34,240 | 15.6 / 164 / 686 M |
| fan-in with 8 names | 8,224 to 256,224 | 50.7 / 995 / 11,890 M |
| wide (random DAG chain, each reader globs 3 chain nodes and a random reader, plus fan-in) | 1,622 to 51,222 | 34.5 / 365 / 2,484 M |
| deep nesting of fan-ins (4 levels) | 1,620 to 52,060 | 20.6 / 177 / 740 M |
| readers that are also holders (every other reader `pub use crate::d::X`) | 980 to 30,428 | 20.7 / 204 / 1,273 M |
| one module with n globs onto the chain | 1,042 to 32,042 | 14.1 / 113 / 474 M |

- **The supply search is linear in every family.** Steps double exactly with n.
- **The remaining superlinear term is outside the search** (`redelivered` and `read_lists`, which
  are known and filed separately). Instruction growth per doubling from n = 400 to 1,600:
  - fan-in: 3.0 then 3.4 (658 M, 2,004 M, 6,768 M);
  - 8 names: 3.6 then 3.8 (3.3 G, 11.9 G, 45.0 G);
  - wide: 2.7 then 3.1.
- So the growth is at most quadratic and approaches 4 per doubling only with many names. I could not
  build an input that is worse than quadratic, or one that makes the supply search superlinear.
- This term exists already at 9fc8aa3, before the supply search (fan-in: 41 M, 95.6 M, 245 M,
  706 M, 2,239 M for n = 50 to 800). The base before the rules is linear.
- **Budget.**
  - `holding()` charges each step through `step(1)?` before it makes it: each binder, each reader
    edge looked at, each new holder, each dominator edge and each depth-first number, and each child
    of the span walk.
  - A refusal is kept in `supply.refused` and returned by `record_drops` right after `redelivered`,
    so answers computed after a refusal are thrown away.
  - With 49 bytes per step against 512 MiB, the cap is about 11 M steps. At linear growth, that is far
    above any input here.

## (2) The dominators and the equivalence

- **Reading the code.**
  - The depth-first walk marks a node when it is popped and pushes all unvisited successors in
    reverse. That is a valid depth-first tree, and a node's parent is the node that pushed the entry.
  - `eval` with iterative path compression matches the recursive `compress`, top of the path first.
  - Semidominators are computed over the predecessors, skipping unnumbered nodes. The bucket of the
    parent is drained after each link, and nodes in the root's bucket are drained when node 1 is
    processed last.
  - The final pass `if dom[w] != vertex[semi[w]] { dom[w] = dom[dom[w]] }` runs in preorder.
  - It is correct on cyclic graphs and self-loops; a self-loop's own `eval` returns the node itself.
    An unreachable node keeps `NONE`, but `holding()` only builds nodes reachable from the root.
  - `dominates` uses Euler-tour entry and exit stamps, which are reflexive.
  - `from == reader` answering false is right: a module cannot hold an item apart from itself, and
    `dominates(reader, reader)` would answer the same.
- **Brute force** (scratch test): 200,000 random graphs of 2 to 10 nodes, with cycles, self-loops,
  repeated edges, edges into the root and unreachable nodes.
  - `dominators()` matches a brute-force immediate dominator on all 416,396 reachable nodes, and
    leaves `NONE` on every unreachable one. Run time 0.3 s.
  - The same test catches a mutant that drops the final pass.
- **Equivalence with c8e5187.**
  - 4,200 programs from the committed generator (seeds 0xa11ce01, 0xa11ce05, 0xc0ffee01,
    0xc0ffee03, 0xbad5eed, 0xdec0de1, 700 each), as written plus 3 shuffles, both name spaces, debug
    builds: 199,480 answers, **0 differ**, no debug panic in either build.
  - 237 programs vary across orders in both builds identically. Every one of those 604 varying
    answers is only the text of an `UnresolvedGlob`/`Cyclic` error, which names a different glob per
    order; none is a resolution answer.

## (3) Shapes at af0ce46, debug and release

- s1 to s11: 17 orders in 2024 (debug) and 2021 (release). Every probe in both name spaces is stable
  and identical to c8e5187, which the previous section compared against rustc and 5f5049b.
- F1 (g1v1, 120 orders), F2 (g1v3p, 120), main (120), g1v2 and g1v6 (720), the F4 set (g2twin,
  g2s2b, g2s1) and g2s3, g2s5, g2s9: all stable, as before, with no disagreement with rustc where
  checked.
- n1 to n8 are stable, as before.
- fx1, fx2, fx3a, fx3b, program 1344 (judged), and audit programs c0ffee03 p0136 and p0437: no
  disagreement with rustc over 13 orders each.
- There is no new order dependence, no debug panic, and no regression.

## (4) Mutants against the new tests

- **Holdings cache removed (the dominator tree rebuilt for each question):** the plain-chain and
  nested-chain tests fail at once (`SupplySteps work passed the ceiling of 16000: 16001`).
- **Reads without companions grouped and asked:** survives. Such reads have no items in `asked`, so
  nothing is searched either way; this mutant does not change behaviour.
- **G-F8 (LOW, recommended): the reader is ignored in `supplied`** (`holding.holds(from)` alone, so
  no dominator check). It survives every committed test (lib 4/4 in `glob_scope::tests`,
  use_resolution 103, exported_items 40).
  - It also changes no answer on the 4,200 generated programs (199,480 answers) or on my 27 shapes and
    fixtures.
  - The same holds for removing the final dominator pass: only my brute-force test catches it.
  - So the "apart from the reading module" guard is correct, but nothing observed depends on it. Any
    route through the reading module also carries the reader's companion and is already left out as
    a carrier.
  - Recommended: commit a small brute-force or table test of `dominators()`. It costs 0.3 s and is
    the cheap guard against a future edit. Either add a fixture where the guard decides, or say in
    the `supplied` doc that the carrier rule normally decides first and the guard is defensive.

## (5) Gates

- `cargo fmt --all --check`: exit 0.
- `cargo clippy -p raf-source --locked --all-targets -- -D warnings`: exit 0.
- `RUSTDOCFLAGS=-Dwarnings cargo doc -p raf-source --no-deps --locked`: clean.
- lib 86, use_resolution 103, exported_items 40 and resolve_paths 17 pass, in debug and release.
- No clock, issue numbers, process words or attribution in the added lines or the commit message.

FINAL VERDICT: PASS

F7 is fixed. The supply search is linear on every input I could build, its steps are charged before
they are made, it is equivalent to c8e5187 on 199,480 answers, and the dominator code matches brute
force.

Remaining items, none blocking:

- The quadratic term in `redelivered`/`read_lists`, which predates the supply search and is filed
  separately.
- G-F8: the dominator guard is not pinned by any committed test.
- F5 (the Contract wording on rustc-order-dependent shapes), in the documentation work.
