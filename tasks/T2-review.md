# Task 2 review: raf-source exported items

## Scope

The staged, uncommitted change on 6e75ac7 (16 files): `Workspace::exported_items`
and `exported_items_with` (`crates/raf-source/src/exports.rs` plus small changes in
facts, index, glob_scope, resolve, item, workspace, lib), the `exported_items`
and `this_repository` tests, the fuzz target, the five PROPOSED blocks and
bindings of `cap:source-index`, the `change:source-index` record, INVENTORY and
the inventory-shape counts, the capability index and CHANGELOG. Every experiment
ran in a scratch worktree (HEAD plus the staged diff); the real worktree was not
touched.

## Evidence

### Gates (scratch tree)

* `cargo test -p raf-source` debug and release: all green. `cargo test
  --workspace` (includes `docs_contract`, `repo_gates` and every pre-existing
  suite): all green. Pre-existing raf-source suites pass unchanged.
* `make fmt clippy doc deny reuse msrv issue-refs test-release`: all rc 0.
  `make docs-sync` twice: rc 0, no diff after either run.
* `make fuzz` (fmt, clippy, deny, bounded runs of the three targets over their
  corpora, so the seed replay is included): rc 0, 5000 runs of each of
  `parse-source`, `parse-document` and `load-workspace` (the last now lists
  every target's surface, hidden included), no crash.
* `make cross-check` was not run: the targets it needs are not installed on
  this machine (environment, not the change).
* `cargo workplan check`: clean. `change reconcile source-index`: exit 1 with
  the five unresolved PROPOSED blocks, six items without outcome,
  `bindings.examples is empty`, and a `tense-mismatch` on delivered line 1254
  ("though not yet for the globs of other modules"), all expected while the
  record is open (see Advisory 5).

### Real rustc comparison

A probe binary over raf-source listed each scratch crate (hidden included); a
generated downstream crate then named every listed path and a set of
hand-picked candidates through `use ::demo::<path> as _;` (methods and
inherent constants through an expression), compiled with nightly 2026-09-30,
lints capped. Each listed path that rustc refuses is a false positive, and each
candidate rustc accepts that is absent is a miss. Scenarios:

* `pub` item of a private module, `pub use` under rename, `pub mod` holding a
  private module and `pub(crate)` items: exact match.
* Glob shadowed by a private item, a `pub(crate)` item and a `pub(crate) use`
  (rustc warns "private item shadows public glob re-export"), and a shadow in
  the other name space only: exact match.
* `pub(super)`, `pub(crate)`, `pub(in path)` and `pub` named re-exports in a
  `pub` module, globbed again from the root; a `pub(crate) mod` globbed `pub`:
  exact match.
* Nested chains through private and `pub` modules, an explicit `pub use`
  beside a glob of the same name, a hidden glob beside a shown route: exact
  match.
* Glob cycles (`pub mod a { pub use super::*; }`, two private modules that glob
  each other): every item listed; only the documented "read once" paths
  (`demo::a::a::f`) are absent.
* `#[macro_export]` at the root, in a `pub` module and in a private module, a
  `pub use` of one under a new name and in a module, a local macro with its
  `pub(crate) use`: exact match.
* `pub extern crate alloc;`, `core as kore`, `self as me`: re-exports and the
  `me` item; the paths under `me` are the documented read-once case.
* `#[path]` modules (public and private) with a nested file: exact match.
* Inherent members (also from an impl in another module, under a renamed
  re-export), trait-impl members excluded, private and `pub(crate)` methods
  excluded, a type alias listed alone: exact match. Trait members are listed;
  rustc names them only through a type, which is the intended reading.
* `#[doc(hidden)]` on module, item, use, method; a hidden item re-exported
  under a new name: flagged as the Contract states.
* Workspace crate re-exported through a glob of a private module: correct
  `Reexport`s with their items.
* Not-test `cfg` (`feature`, `any()`, `not(any(test, unix))`): listed though
  rustc has no such item, and `cfg_attr(.., doc(hidden))` not read: both are
  the documented syntactic limits.
* Enum variants (glob and named): absent as items, as documented.
* Ambiguous globs: two `pub` globs, a private and a `pub` glob, a `pub` glob of
  a `pub(crate)` item beside a `pub` one (rustc accepts the crate with an
  `ambiguous_glob_reexports` warning and downstream can name `demo::x`), and
  four inaccessible-glob shapes: every one fails the listing loudly with
  `Resolve(Ambiguous)`, except the next case.
* The second unmodelled rule: `mod m { use self::a::*; pub use self::b::*; }`
  with `pub use m::*;` lists `demo::x = demo::m::b::x`, and rustc refuses
  `demo::x`. This is the case the PROPOSED Constraints describe. The first
  rule (`pub use self::x as y` of a private glob's binding) is refused by
  rustc (E0364) and the tool lists nothing, as the Constraints claim.
* External items carried by a glob: see Required 1.

### Costs (release, on a 2 MiB thread, two calls compared for equality)

| shape | items | one listing |
| --- | --- | --- |
| 20,000 chained renames (test) | 20,000 | passes on 2 MiB |
| 20,000 chained pub globs (test) | 1 | passes on 2 MiB |
| glob mesh 300 `pub mod mi { pub use super::*; }` | 180,600 | 0.49 s |
| glob mesh 1000 | over budget | `Budget(ItemBytes)` after 2.6 s, no panic |
| 2000 renames of a type with 2000 methods | over budget | `Budget(ItemBytes)`, 0.29 s |
| chain of 5000 globs, 200 refused and 200 public names | 200 | 1.6 s |
| 120 nested `pub mod a { .. pub use super::*; }` | 122 | < 1 ms |
| fan-in 300 globs of one 300-name module | 300 | 70 ms |
| one module with 1000 / 5000 / 20000 `pub` globs, one name each | n | 0.085 s / 2.1 s / 51 s |

Every run was deterministic. With a counting allocator, the peak bytes during a
listing were 0.50 to 0.94 of the amount charged (found by bisecting
`max_item_bytes` against `usage().item_bytes`) on seven shapes, so the charge is
an upper bound, temporaries included. Recursion: the BFS, the route search and
the member walk are all iterative.

The last row is quadratic (see Advisory 1); for comparison, the existing
`resolve` of each of the 20,000 names takes 27.7 s and the load 22.7 s.

### API

`ExportOptions`, `ExportedItem`, `Reexport`, `Exports`, `ExportError` are
`#[non_exhaustive]` with documented fields, as `LoadOptions`, `Item` and
`ResolveError` are; `ExportError` has `Display`, `Error::source`. The new
`UseFact`/`UseDecl`/`Item` booleans leave the sizes unchanged (measured at HEAD
and with the change: `UseDecl` 120, `UseFact` 120, `Item` 224 bytes).

### Mutation (each run against `exported_items` and `this_repository`)

| mutant | result |
| --- | --- |
| named-import visibility check dropped on one hop | killed (3 tests) |
| glob search limited to the first route | killed |
| `use` `doc(hidden)` ignored in the listing | killed |
| `cfg(test)` `use` declarations exporting | killed |
| exported path built from the defining name (rename lost) | killed (9 tests) |
| extra: glob-hop visibility dropped in `routes` | killed (4 tests) |
| extra: `doc(hidden)` dropped in the use facts | killed |
| extra: test check dropped in `declared_public` | survives: equivalent, as `module` and `members` check the item's test role first |

### Documents

* `source-index.md`: outside the five PROPOSED blocks the only changes are
  `state: implementing` and the added bindings (checked line by line over the
  diff). The bindings resolve (`check` clean).
* INVENTORY (`source-index` implementing) and the counts (designed 9,
  implementing 3, delivered 4, considered 5) match the tree.
* Usage/Examples PROPOSED blocks are prose-only: confirmed. A quoted
  `rust,no_run` block inside the Usage PROPOSED block makes
  `usage_and_examples_blocks_are_the_crate_doctests` fail ("block 2 is
  `rust,no_run` in the document and `rust` in the crate"), because the
  comparison is by position against the leading crate doctests. Task 11 can
  resolve them cleanly by moving the two new crate-doc blocks of `lib.rs`
  (now doctests 6 and 7) to the positions the document order gives them (the
  `no_run` listing second, after the existing `no_run` block, and the
  `MemoryTree` listing after the existing `MemoryTree` example), a prose
  reorder of crate docs with no code change.
* Hygiene: SPDX on the three new files (`reuse` passes); no tracker numbers,
  process wording or attribution in added lines; no new dependency; no
  `unsafe`.

## Findings

### REQUIRED

1. **External re-exports carried by a `pub` glob are silently missing.** A
   `pub` glob of a module that re-exports an item of a crate outside the
   workspace lists nothing for it:
   `mod a { pub use std::fmt::Debug; pub use std::collections::*; pub fn own() {} } pub use a::*;`
   gives the item `demo::own` and no `Reexport`, while rustc exports
   `demo::Debug` and `demo::HashMap` (the downstream `use` compiles). The
   same holds when `a` is a `pub` module: `demo::a::Debug` is a re-export but
   `demo::Debug` is not listed. The cause is that `unbound_imports` and
   `unseen_globs` run only for the module being visited, and the glob views
   hold only items the index sees. The Contract PROPOSED text ("A public
   re-export that names nothing the target defines is a `Reexport` ... of a
   crate outside the workspace or one of its items") and record item 4 state
   the opposite, with no error and no stated limit. Fix either way: carry the
   `pub`, not-test unbound named imports and unseen globs of each module that
   a public glob route reaches into the reading module's re-exports (under
   the reading path, unless an explicit name of the reading module shadows
   them, with the route's `doc(hidden)`), with a fixture beside the explicit
   `pub use a::Debug` neighbour that already works. Or state the limit in
   the Contract and Constraints PROPOSED blocks and narrow record item 4.

### ADVISORY

1. Listing time is quadratic in the number of `pub` globs of one module times
   its names: `routes()` clones the module's route vector on every call, and
   `public_route`, `reexport_hidden` and `binding_site` each scan every route
   for each name (51 s for one module with 20,000 globs, against 22.7 s for
   the load of the same tree and 27.7 s to `resolve` every name). The load and
   `resolve` are already of that order, so this is not a new class of cost,
   but returning the cached routes by reference and indexing the routes that
   hold a name (or reading the glob table's per-name entry next to
   `view_names`) would make it linear in what it lists.
2. A crate that rustc accepts with only an `ambiguous_glob_reexports`
   warning (two `pub` globs bringing in `x`, one `pub(crate)`, where downstream
   names `demo::x` without error) cannot be listed at all: the whole listing
   fails with `Ambiguous`. That is loud and documented as "two globs bring in
   two items", but Task 7's coverage then fails for the whole crate. Worth
   one sentence in the Constraints naming it, so a user knows why.
3. A named re-export of an enum variant (`pub use p::F::G;`) is listed as a
   `Reexport` with no item; the `Reexport` docs say it "names nothing the
   target itself defines". Say that a variant re-export is listed as a
   re-export (or exclude it, matching the glob-of-enum case).
4. Record item 1 says each item is listed "under its exported path"; a module
   reached by a second path is read only under the first, so
   `demo::b::f` (for `pub use a as b;`) and `demo::me::f` (for
   `pub extern crate self as me;`) are names rustc accepts and the list does
   not hold. The Contract states the read-once rule; qualify the record item
   to match.
5. Task 11 inherits two obligations from flipping the capability to
   `implementing`: `reconcile source-index` reports `bindings.examples is
   empty` (`crates/raf-source/examples/registry_scan.rs` exists and may be a
   candidate) and a `tense-mismatch` on the delivered sentence at line 1254
   ("though not yet for the globs of other modules"), which needs rewording
   without changing what it says. Add both to the hand-off list beside the
   `okf-profile` one.
6. `Item.macro_export` is `pub(crate)` while its sibling `doc_hidden` is
   `pub`; consider making it `pub` and documented, as a caller reading
   `Item` cannot otherwise tell why a macro is exported.
7. The test check in `declared_public` is redundant (equivalent mutant); keep
   it as defence or drop it, either is fine.

First-round verdict: BLOCKED (Required 1).

## Recheck after the fixes (R1, A1, A2, A3, A4, A6, A7)

Same method: the newly staged diff applied in a fresh scratch worktree, the
probe and downstream-crate harness rebuilt against it, and nightly rustc as
ground truth.

### R1: external re-exports carried by `pub` globs

| scenario | result against rustc |
| --- | --- |
| `mod a { pub use std::fmt::Debug; pub use std::collections::*; }` + `pub use a::*;` | `demo::Debug` is a `Reexport`, `demo::*` <- `std::collections::*` is a glob re-export: match |
| the same with `pub mod a` | `demo::Debug`, `demo::a::Debug`, `demo::*`, `demo::a::*`: match |
| renames (`Debug as Dbg`, `Cell as C`) | `demo::Dbg`, `demo::C` listed, the original names not listed: match |
| chain of three private modules of `pub` globs | `demo::Cell`, `demo::*` <- `std::rc::*`: match |
| reader shadows with `pub struct Cell` | the item, no re-export: match |
| mid-route module shadows (`pub struct Cell` in `a`; private `struct Cell` in `a`) | match (rustc refuses `demo::Cell` in the private case, and the tool lists nothing) |
| `pub(crate)` glob, private glob, `pub(crate) use` at the end, `pub(crate)` glob at the root | nothing listed, rustc refuses each: match; the `pub` sibling (`Write`) is listed |
| `#[doc(hidden)]` glob route, `doc(hidden)` on the carried `use`, a hidden root glob beside a `pub mod shown` glob | flagged hidden; `demo::shown::Write` shown: match |
| a module reached through a hidden and a shown route, in both orders | shown: match |
| two routes to one module, a glob cycle carrying from both sides | one entry each: match |
| `cfg(test)` on the carried `use` or on the root glob | not listed: match |
| explicit `pub use std::cell::Cell` beside a glob that brings an item `Cell` in the mid module | `Reexport` of std's `Cell`, no item: match (explicit shadows glob) |
| carried enum variants (`std::cmp::Ordering::Less`, `crate::E::A`) | `Reexport`s, as the Contract now says |

One shape is still missed, and it is documented. When the reader or a module
on the route binds the name in another name space (`fn Rc() {}` beside a
carried `pub use std::rc::Rc;`), rustc still exports the carried type, but the
listing drops it. The Contract PROPOSED block says so ("binds the name itself,
in any name space, as the index cannot tell the name space of an item it does
not see"). Per-name re-exports inside an outside crate's glob
(`demo::HashMap`) are represented by the single `demo::*` entry, as designed.

Regression run of the first-round scenarios (private modules, glob shadowing
in either name space, cycles, restricted imports, chains, macros, `extern
crate`, `#[path]`, members, `doc(hidden)`, ambiguity, the second unmodelled
rule, workspace crates through globs): results unchanged.

### Speed and budget

| shape (release) | load | one listing |
| --- | --- | --- |
| one module of 1000 / 5000 / 20000 `pub` globs of one item each | 0.07 s / 1.5 s / 29 s | 5 ms / 38 ms / 0.20 s (A1 fixed: was 51 s) |
| 20,000 chained renames | 0.12 s | 81 ms |
| 20,000 chained `pub` globs ending in a carried `pub use` | 0.52 s | 0.11 s |
| glob mesh 300 with a carried re-export in each module | 16 ms | 0.61 s |
| glob mesh 1000 with carried re-exports | 57 ms | `Budget(ItemBytes)` after 2.2 s |
| chain of 5000 globs ending in 2000 carried `pub use`s | 0.11 s | 3.6 s |
| **one module of 1000 / 3000 / 5000 / 10000 `pub` globs of a module that carries one outside re-export** | 0.06 / 0.53 / 1.1 / 4.2 s | **0.58 / 4.1 / 16.7 / 47 s** |

Every run was deterministic. Peak allocation during a listing divided by the
amount charged (found by bisecting `max_item_bytes`): 0.31 to 0.66 on six
shapes, including three carrying shapes. So the charge is still an upper bound.

### Mutation of the fix

| mutant | result |
| --- | --- |
| carried-import shadow check dropped | killed |
| no glob reach (carry nothing) | killed |
| the route's `hidden` dropped from a carried import | killed |
| the shown-routes-first pass of `glob_reach` dropped | **survives** |
| first-round M1, M4, M5 re-run | killed |

The surviving mutant is not equivalent. With
`mod c { pub use std::cell::Cell; } mod h { pub use crate::c::*; } mod s { pub use crate::c::*; } #[doc(hidden)] pub use h::*; pub use s::*;`
the real code lists `demo::Cell` shown. The mutant lists it as hidden, so a
default listing drops it.

### Other fixes

* A1: fixed as above. A2: the Constraints now name the
  `ambiguous_glob_reexports` case. A3: variants are documented as
  `Reexport`s. A4: record item 1 states the read-once rule. A6:
  `Item::macro_export` is `pub` and documented. A7: the redundant check is
  gone and the doc comment says why.
* `source-index.md`: outside the PROPOSED blocks, only `state` and the bindings
  changed. No tracker numbers, process wording, attribution or `unsafe` in
  added lines.
* Usage/Examples PROPOSED blocks are unchanged: still prose-only, still
  resolvable in Task 11 by reordering the crate-doc blocks (first round).
* A5 (`bindings.examples` empty, `tense-mismatch` now at line 1258) is still
  in `reconcile`. It is a Task 11 hand-off, not a code fix.

### Gates (fresh scratch tree)

`cargo test -p raf-source` debug and release, `cargo test --workspace`
(includes `docs_contract`, `repo_gates`), `make fmt clippy doc deny reuse msrv
issue-refs test-release`: all rc 0. `make docs-sync` twice: no diff.
`cargo workplan check`: clean. `make fuzz` was re-run on the restored tree
after the mutation runs: rc 0, 5000 runs of each target. `cross-check`
was not run, as the targets are not installed on this machine.

## Recheck findings

### REQUIRED

1. **Quadratic cost in carrying outside re-exports.** `unbound_imports` calls
   `glob_reach(visit.module, Some(name))` for each unbound `pub use` of each
   reached module. Each call is a fresh breadth-first walk over every module
   the reader's routes reach, so the cost is (carried imports) × (reached
   modules). One module of 10,000 `pub` globs, each reading a private module
   with one `pub use std::cell::Cell as Cn;`, takes 47 s to list against
   4.2 s to load. Growth is quadratic (0.58 s, 4.1 s, 16.7 s, 47 s for 1000,
   3000, 5000, 10000), while the same shape with items lists 10,000 globs in
   well under 0.2 s. This is the only listing shape found that costs far more
   than the load and `resolve`, and the fix introduced it.
   Fix: walk the reach once per visit, carrying the set of names that the
   modules on each route shadow. Or skip the per-name walk when no module on
   the reach binds the name. Or memoise the shadow answer per (reached
   module, name), so the cost is linear in the reach plus the carried
   imports. Add a scale test beside the existing 3000-glob one, for example
   3000 globs of modules that each carry one outside re-export, within the
   same release bound.

### ADVISORY

1. Add a fixture for a module reached by a hidden route before a shown one
   (the example above), which kills the surviving `glob_reach` mutant.
2. Consider stating in the Constraints block, beside the other syntactic
   limits, the documented other-name-space drop (`fn Rc` beside a carried
   `Rc`). Today it is only in the Contract.

Second-round verdict: BLOCKED (recheck Required 1).

## Third round: the carry walk

The implementer replaced the per-import walk with one reach walk per listed
module. That walk builds a binder index: the modules of the reach that bind
each name. A carried import whose name no other module of the reach binds is
carried at once. A name that another module does bind triggers one more walk,
once per name, and its result is cached in `Carry::walked` for the visit. The
change also adds a 6000-import scale test, the hidden-before-shown fixture and
the Constraints wording. Rechecked in a fresh scratch worktree with the same
probe and harness.

### Timings (release, one listing)

| shape | n = 1000 | 3000 | 5000 | 10000 |
| --- | --- | --- | --- | --- |
| second-round shape: n `pub` globs of modules each carrying `pub use std::cell::Cell as Cn;` | 12 ms | 40 ms | 65 ms | 141 ms (was 47 s) |
| the same, plus a sibling glob `pub use sN::*;` of `mod sN { fn Cn() {} }` per n | 0.65-0.93 s | 8.9-9.8 s | 20-35 s | not run |

Loads of the second shape take 0.07 s to 5.9 s. The other shapes are no
slower than in round two: 20,000 item globs list in 0.15 s, 20,000 renames in
0.10 s, a 5000-glob chain ending in 2000 carried imports in 36 ms, and
120-deep nesting in 41 ms. The 300-module mesh with carried imports takes
0.59 s, and the 1000-module mesh still stops with `Budget(ItemBytes)`. Every
run was deterministic.

Budget (peak allocation during the listing, divided by the bytes charged,
found by bisecting `max_item_bytes`): the second-round shape 0.33, the
5000-glob chain 0.33, 20,000 renames 0.66, nesting 0.60, mesh 0.63. **The
sibling-shadow shape: 5.53 at n = 1000 and 15.75 at n = 3000. At n = 5000 the
process peaks at 1.09 GiB resident.**

### Results against rustc

All 34 probe scenarios were re-run: the 17 carry scenarios of round two (now
including the hidden-before-shown pair in both orders), 16 first-round
scenarios, and one new scenario where a sibling glob shadows. Their outputs
are identical to round two. The only listed path rustc refuses is the
documented second-unmodelled-rule case (`rule2g`). The two ambiguity scenarios
still fail loudly. In the sibling-glob shape, `demo::Cell` is carried past a
private `fn Cell` of a sibling module: match.

### Mutants

All 10 killed: carried-name shadow check ignored, binder counted even when it
is the import's own module, hidden-first pass dropped (the surviving mutant of
round two, now killed by the new fixture), no reach, the route's `hidden`
dropped from a carried import, and the first-round M1-M5.

### Gates

All green, run on the unmutated tree after the mutation runs:
* `cargo test -p raf-source` in debug and release (no failures);
* `cargo test --workspace`, which includes `docs_contract` and `repo_gates`;
* `make fmt clippy doc deny reuse msrv issue-refs test-release`: rc 0;
* `make docs-sync` twice: no diff;
* `make fuzz`: rc 0, 5000 runs of each target;
* `cargo workplan check`: clean.

`source-index.md` still changes nothing outside the PROPOSED blocks but
`state` and the bindings. `cross-check` was not run, as its targets are not
installed on this machine.

## Third-round findings

### REQUIRED

1. **The per-name re-walk is still quadratic, and its cache is not charged.**
   `carried` walks the reader's whole reach again for every carried name that
   any other module of the reach binds. It keeps each walk's full module list
   in `Carry::walked` until the visit ends. So time and memory are
   (shadowed carried names) × (reach). Adversarial shape: for each n,
   `mod mN { pub use std::cell::Cell as CN; } mod sN { fn CN() {} } pub use mN::*; pub use sN::*;`.
   A private sibling definition is enough to count as a binder. This lists in
   9-10 s at n = 3000 and 20-35 s at n = 5000. Peak allocation is 15.75× the
   bytes charged at n = 3000, and 1.09 GiB resident at n = 5000. That breaks
   the module-level claim that uncharged temporaries are bounded by what the
   load charged and dropped before the next is made, and it lets a listing
   pass `max_item_bytes`.
   Fix, at minimum: keep only the answers actually asked (for each shadowed
   name, whether the carrying modules are reached), not the whole reach, and
   charge what is kept. Better: decide shadowing in the one walk, by passing
   along each route the binder names met on it. Or restrict the re-walk to
   the modules on routes between the reader and the carrying module. Add this
   shape at scale beside the 6000-import test.

### ADVISORY

1. One glob bringing an item `Rc` and another carrying an outside `pub use
   std::rc::Rc;` to the same reader both appear: an item `demo::Rc` and a
   `Reexport` `demo::Rc`. There is no `Ambiguous` failure, unlike two item
   globs. rustc accepts the downstream `use`, so this is not a wrong
   answer. But the Contract's ambiguity rule does not mention it, and one
   sentence would settle it.

Third-round verdict: BLOCKED (third-round Required 1).

## Fourth round: the carry graph and dominators

What changed. Each listed module now walks its reach once, into a graph. A
binder with no public routes of its own is ignored. Otherwise:
* the reader binding the name, or a binder that dominates the carrier in the
  graph (Cooper-Harvey-Kennedy dominator tree, built once per reader), keeps
  the name out;
* a single non-dominating binder lets it through;
* with several binders, none dominating, there is one search per name, which
  keeps only the answer for each carrier.

All of this work is charged to item-bytes at one binding per step. Rechecked
in a fresh scratch worktree with a rebuilt probe and harness.

### Timings (release, one listing; load in brackets)

| shape | n = 1000 | 3000 | 5000 | 10000 |
| --- | --- | --- | --- | --- |
| carried-import shape (round two) | 16 ms (0.08 s) | 49 ms (0.54 s) | 52 ms (0.97 s) | 110 ms (4.7 s) |
| sibling-shadow shape (round three; binder without routes) | 21 ms (0.26 s) | 83 ms (2.4 s) | 96 ms (6.5 s) | 261 ms (29 s) |
| the same, binder with its own `pub` glob (one binder, not dominating) | 31 ms | 82 ms | 122 ms | 0.55 s (93 s) |
| two such binders per name (several-binder search) | 0.20 s | `Budget(ItemBytes)` in 0.14 s | `Budget` in 0.12 s | `Budget` in 0.43 s |
| a dominating binder per name | 17 ms | 87 ms | 174 ms | |

Other shapes (release):
* 20,000 item globs: 0.33 s.
* 20,000 renames: 0.10 s.
* 5000-glob chain ending in 2000 carried names: 80 ms.
* 3000-glob chain with a binder beside each step (several-binder searches): 96 ms.
* 120-deep nesting: 44 ms.
* Mesh of 300 modules with carried names, with or without two private binders each: 0.5 s and 1.0 s.
* Mesh of 1000: `Budget` in 1.7 s.

Every run was deterministic.

Peak allocation during a listing divided by the bytes charged, on 12 shapes:
0.01 to 0.66. Never above 1. The several-binder shape at n = 1000 is 0.01,
because it charges 437 MB of work for 4.3 MB actually held. The largest peak
was 198 MiB (mesh 1000, refused). The round-three overrun (15.75× charged,
1.09 GiB resident) is gone.

### Results against rustc

* **The 34 earlier scenarios:** outputs identical to round three, line for
  line on the carry set. The only listed path rustc refuses is still the
  documented `rule2g`; the two ambiguity cases still fail loudly.
* **27 new binder scenarios**, using a private `struct Cell` as the binder so
  that it binds in the same name space as the carried `std::cell::Cell`:
  - 1, 2 and 3 binders on and off routes, a diamond with one and with both
    sides bound, a binder above a diamond, two bound routes plus a free
    third, three binders off every route;
  - a two-binder chain alone, with a free side route, and behind a free
    module;
  - a cycle where the binder dominates, a cycle entered twice, a cycle with
    two binders with and without a free route, an irreducible cycle;
  - the reader binding by a definition and by a private import, a binder that
    binds by import, a binder both on and off a route (with and without a
    free route);
  - a carrier that has `pub` routes of its own, two carriers of one name
    behind two binders in both orders, and a `pub` module reader.

  Every listed and unlisted `demo::Cell` matches rustc.
* **Except the `doc_hidden` flag; see Required 1.**

### Mutants (each against `exported_items` and `this_repository`)

| mutant | result |
| --- | --- |
| `dominates` always false | killed |
| `dominates` always true | killed |
| single non-dominating binder treated as keeping the name out | killed |
| search passes through binders | killed |
| hidden-first reach pass dropped | killed |
| first-round M1 to M5 | killed |
| binders without routes counted (the "off every route" rule dropped) | survives, equivalent: a sink can only be the end of a route, so it changes cost, not answers |
| reader check dropped | survives, equivalent: node 0 dominates every node and has routes |
| span comparison made strict | survives, equivalent: a binder is never the carrier |
| one pass of the dominator iteration | survives on the suite; the irreducible-cycle probes give the right answers with the real code |
| carrier counted among its own binders | **survives, not equivalent**: a carrier with `pub` globs of its own (`mod c2 { pub use std::cell::Cell; pub use crate::z::*; }`) would dominate itself and lose its re-export; the real code lists it (probe `binder_is_carrier_sibling_import`) |
| the first carrier's answer copied to every carrier of the name | **survives**: affects which declaration (file and line) a carried name is reported from when two carriers share a name |

### The bounded Budget refusal

Judged acceptable. It is:
* **Loud:** a typed `ExportError::Budget` of `ItemBytes` naming a file and
  line.
* **Fast:** at most 0.43 s in every shape I tried.
* **Charged:** every step of the tree and the searches.
* **Documented:** in the Contract, with its shape and numbers, and in the
  Constraints, as a refusal of crates rustc accepts.

It only arises when many distinct names re-exported from outside the
workspace are each bound by two or more modules that sit off the dominator
path and have public globs of their own. That needs adversarial or
generated input. The error text says only "over budget" at the reader's
line; naming the carry search would help a user (Advisory 2).

### Gates (fresh scratch tree, after the mutation runs)

All green:
* `cargo test -p raf-source` in debug and release, and `cargo test
  --workspace`, which includes `docs_contract` and `repo_gates`;
* `make fmt clippy doc deny reuse msrv issue-refs test-release`: rc 0;
* `make docs-sync` twice: no diff;
* `make fuzz`: rc 0, 5000 runs of each target;
* `cargo workplan check`: clean.

`source-index.md` still changes nothing outside the PROPOSED blocks but
`state` and the bindings. No tracker numbers, process wording, attribution or
`unsafe` in added lines. `cross-check` was not run, as its targets are not
installed on this machine.

## Fourth-round findings

### REQUIRED

1. **A carried re-export whose only free route is hidden is reported as
   shown.** Example:
   `mod c { pub use std::cell::Cell; } mod p { pub use crate::c::*; struct Cell; } mod q { pub use crate::c::*; } pub use p::*; #[doc(hidden)] pub use q::*;`
   The shown route through `p` is shadowed by `p`'s `Cell`, so the only route
   that carries `Cell` is the `#[doc(hidden)]` glob. The listing nevertheless
   gives `demo::Cell` with `doc_hidden: false`, and it appears in the default
   `exported_items` listing. The item analogue, with `mod c { pub struct
   Cell; }`, is correctly flagged hidden and left out by default. The same
   happens with two shadowed shown routes and a free hidden third. The cause:
   the hidden flag comes from the reach walk (a shown route reaches `c`),
   while carrying is decided separately over all routes. This is a silent
   wrong answer on a documented flag ("on the `use` declaration that exports
   it").
   Fix: decide carrying first over the shown routes. If a carrier is carried
   only once hidden routes are allowed, flag it hidden. Add a fixture beside
   the item analogue.

### ADVISORY

1. Two non-equivalent survivors are test gaps; the real code is right on my
   probes for both. Add fixtures for a carrier with `pub` globs of its own,
   and for two carriers of one name of which only one has a free route
   (checking which declaration is reported). An irreducible cycle fixture
   would also pin the multi-pass dominator iteration.
2. The several-binder `Budget` refusal reads "the public surface is over
   budget: src/lib.rs:1: exceeds the budget for bytes of index records".
   Naming the carry search, or the name being decided, would tell a user
   which re-export caused it.

Fourth-round verdict: BLOCKED (fourth-round Required 1).

## Fifth round: hidden routes in the carry

What changed:
* Carrying is decided first over the routes `#[doc(hidden)]` does not hide
  (`SHOWN`), then over every route (`EVERY`). Each has its own graph and
  dominator tree, and the `doc_hidden` flag follows from which pass carried
  the name.
* New fixtures: a hidden-only free route, a carrier with globs of its own,
  two carriers of one name, an irreducible cycle.
* The several-binder refusal is now `ExportError::CarryBudget { path, error }`,
  naming the re-export.

Rechecked in a fresh scratch worktree with a rebuilt probe and harness.

### Hidden and shown combinations

Each case was built twice: once with the carrier holding `pub use
std::cell::Cell;` (carried) and once with `pub struct Cell;` (the item
analogue). Each column gives the flag with `include_hidden`, presence in the
default listing, and rustc nameability. For the local-item variant I also
checked `cargo doc`.

| case | carried | item | `cargo doc` (item) |
| --- | --- | --- | --- |
| H1 shown route shadowed, hidden route free | hidden, absent, rustc ok | same | hidden |
| H2 two shadowed shown routes, hidden third free | hidden, absent | same | hidden |
| H3 shown and hidden both free | shown | same | shown |
| **H4 `mod q { #[doc(hidden)] pub use crate::c::*; } pub use q::*;`** | **hidden, absent** | **shown** | **shown** |
| **H4b H4 plus a shown route shadowed by a binder** | **hidden, absent** | **shown** | **shown** |
| H4c H4 plus a free shown route | shown | same | shown |
| H5 `#[doc(hidden)]` on the carried `use`, through a glob and directly | hidden, absent | same | |
| H6 `#[doc(hidden)] mod q` on the route | shown | same | shown |
| H7 `#![doc(hidden)]` in the route module's file | shown | same | |
| H8 `#[doc(hidden)] mod c` (the carrier) | shown | same | shown |
| H9 hidden `pub mod r` reader (`demo::r::Cell`) | hidden, absent | same | |
| H9b the same module re-exported shown (`pub use r as s;`) | read once, under `r` | same | |
| H10 every route blocked | absent, rustc refuses | same | |
| H11 hidden route through a dominating binder, shown route free | shown | same | |
| H12 short hidden route, longer shown one | shown | same | |
| H13 shown route through two binders, hidden route free | hidden, absent | same | hidden |
| H14 binder only on the hidden route | shown | same | |
| H15 cycle: shown route shadowed, hidden free | hidden, absent | same | hidden |

Every case agrees with its item analogue except H4 and H4b; see Required 1.
For outside items, `cargo doc` documents the carried `std::cell::Cell` even
in H1, H2, H13 and H15. So rustdoc is no reference for carried outside items,
and I judged against the item analogue, which the Contract names ("as an item
is") and which `cargo doc` confirms for local items.

### Results against rustc

The 34 earlier scenarios are identical to round four line for line, with the
same two loud ambiguity failures and the documented `rule2g`. The 25 binder
scenarios (plus the irreducible cycle, two carriers in both orders, and a
carrier with globs of its own) are identical to round four on whether
`demo::Cell` is listed, and each matches rustc.

### Timings and memory (release, one listing; load in brackets)

| shape | n = 1000 | 3000 | 5000 | 10000 |
| --- | --- | --- | --- | --- |
| carried-import shape | 11 ms | 35 ms | 125 ms | 240 ms (7.0 s) |
| sibling shadow, binder without routes | 21 ms | 46 ms | 110 ms | 160 ms (32 s) |
| one binder with routes | 17 ms | 93 ms | 149 ms | 0.51 s (71 s) |
| one binder with routes on the shown route, free hidden route | 39 ms | 85 ms | 118 ms | 0.31 s (94 s) |
| two binders with routes | 84 ms | `CarryBudget` in 0.13 s | `CarryBudget` in 0.12 s | `CarryBudget` in 0.18 s |
| dominating binder | 10 ms | 44 ms | 81 ms | 171 ms |

The `CarryBudget` text reads "deciding whether the public globs carry the
re-export `demo::C366` is over budget: src/lib.rs:2198: ...", naming the
re-export and its declaration. Advisory 2 of round four is done.

Other shapes:
* 20,000 item globs: 0.28 s.
* 20,000 renames: 0.10 s.
* 5000-glob chain carrying 2000 names: 74 ms.
* 3000-glob chain with binders beside it: 0.13 s.
* 120-deep nesting: 72 ms.
* Mesh of 300 modules: 0.61 s. Mesh of 1000: `Budget` in 1.6 s.

All deterministic. Peak allocation divided by the bytes charged, on 12
shapes: 0.01 to 0.66, never above 1. The largest peak was 198 MiB (mesh
1000, refused); maxrss across the run was 358 MiB.

### Mutants (each against `exported_items` and `this_repository`)

18 of 19 killed:
* the shown pass run over every route;
* the hidden pass removed;
* the shown target check dropped;
* hidden edges let into the shown graph;
* the carry's hidden flag ignored;
* `dominates` forced false or true;
* the carrier counted as its own binder (survived in round four, now
  killed);
* a single binder treated as blocking;
* one pass of the dominator iteration (survived in round four, now killed);
* the search passing through binders;
* answers copied between carriers (survived in round four, now killed);
* the hidden-first reach pass dropped;
* M1 to M5.

The survivor is equivalent: dropping the shown-reach filter on binders. A
binder that no shown route reaches is never on a shown path; it can only
turn the one-binder shortcut into a search with the same answer.

### Gates

All green on the unmutated tree:
* `cargo test -p raf-source` in debug and release, and `cargo test
  --workspace`, which includes `docs_contract` and `repo_gates`;
* `make fmt clippy doc deny reuse msrv issue-refs test-release`: rc 0;
* `make docs-sync` twice: no diff;
* `make fuzz`: rc 0, 5000 runs of each target;
* `cargo workplan check`: clean.

`source-index.md` still changes nothing outside the PROPOSED blocks but
`state` and the bindings. Hygiene grep of added lines: 0 hits.
`cross-check` was not run, as its targets are not installed on this machine.

## Fifth-round findings

### REQUIRED

1. **A `#[doc(hidden)]` glob in the middle of a route hides a carried
   re-export, but not an item.** Example:
   `mod c { pub use std::cell::Cell; } mod q { #[doc(hidden)] pub use crate::c::*; } pub use q::*;`
   `exported_items` leaves `demo::Cell` out, and with `include_hidden` it
   comes back with `doc_hidden: true`. With `mod c { pub struct Cell; }` and
   nothing else changed, `demo::Cell` is listed shown, and `cargo doc`
   documents `struct.Cell.html` at the crate root. The same holds with an
   extra shown route that a binder shadows (`mod p { pub use crate::c::*;
   struct Cell; } pub use p::*;` added): the carried re-export is dropped, the
   item is shown. rustc names `demo::Cell` in all four builds.
   The Contract PROPOSED text says such a re-export "is hidden when only
   routes through a `#[doc(hidden)]` glob carry it, as an item is". For an
   item, only the reader's own glob counts (`reexport_hidden` reads the
   reader's routes). So that sentence, and the carried result, are wrong for
   a hidden glob that is not the reader's own.
   The cause is in `reach_graph`: the `SHOWN` graph drops every hidden route
   (`.filter(|route| mode == EVERY || !route.hidden)`), not only the reader's.
   Fix: in the `SHOWN` graph, drop only the hidden routes out of node 0 (the
   reader), as items do. Add H4 and H4b as fixtures beside their item
   analogues.

No other wrong answer was found in this round.

Fifth-round verdict: BLOCKED (fifth-round Required 1).

## Sixth round: reader-only hidden routes, and an independent differential

What changed:
* The shown graph now drops only hidden routes that leave the reader. The
  reach walk does the same, so a hidden glob further along no longer hides.
* One fixture that encoded the old behaviour was changed.
* A differential test was added: 166 generated crates, each comparing a
  carried re-export against its item version, varying depth, binder, hidden
  position and reader location.

Rechecked in a fresh scratch worktree with a rebuilt probe and harness.

### Hidden/shown probes (carried vs item version, with rustc)

All 23 cases now agree with their item versions and with rustc:
* the 18 of round five, including H4 and H4b (the two repros), now shown in
  both;
* five new ones:
  - H4d, a hidden glob two hops in: shown;
  - H4e, a hidden reader glob plus a hidden mid glob: hidden;
  - H9c, a `pub mod` reader behind a mid hidden glob: shown;
  - H16, a mid hidden glob on one of two binder routes plus a hidden third:
    shown;
  - H9b, a hidden `pub mod` re-exported shown: absent in both, as the item
    rule says.

### Independent differential (my own generator)

Random crates of 1 to 6 private modules, each with 1 to 3 globs:
* globs to other modules or the carrier, a quarter `#[doc(hidden)]`, some
  `pub(crate)`, cycles allowed;
* binders by `struct Cell;` or by a private `use std::rc::Rc as Cell;`;
* `#[doc(hidden)]` on modules and on the carrier;
* readers at the root or in a (sometimes hidden) `pub mod r`, with 1 to 3
  globs, some hidden.

Each crate was listed as a carried re-export and as the item version, with
and without `include_hidden`, and every tenth to twenty-fifth crate was
checked with rustc. Results:
* 1400 crates, over three seeds, one of them with no `pub(crate)` globs;
* 9 differences, every one containing one of the two shapes below, from
  which the minimal repros were reduced;
* on 60 crates checked with rustc, presence agrees every time. One further
  mismatch came from a generated crate that rustc itself rejects (E0255);
  that generator bug was removed.

### Results against rustc (the standing probes)

The 34 earlier scenarios are identical to round five (the same digest of the
carry section; the same two loud ambiguity failures and the documented
`rule2g`). The 25 binder scenarios are identical, each matching rustc.

### Timings and memory (release, one listing; the machine was loaded)

| shape | n = 1000 | 3000 | 5000 | 10000 |
| --- | --- | --- | --- | --- |
| carried-import shape | 21 ms | 63 ms | 86 ms | 0.25 s |
| sibling shadow | 26 ms | 107 ms | 140 ms | 0.42 s |
| one binder with routes | 43 ms | 127 ms | 166 ms | 0.32 s |
| binder on the shown route, free hidden route | 17 ms | 148 ms | 0.30 s | 0.88 s |
| new: hidden mid glob per name | 39 ms | 104 ms | 185 ms | 0.29 s |
| two binders with routes | 0.15 s | `CarryBudget` 0.19 s | 0.24 s | 0.56 s |
| dominating binder | 29 ms | 95 ms | 116 ms | 0.27 s |

Other shapes:
* 20,000 item globs: 0.23 s.
* 20,000 renames: 0.18 s.
* Chains: 17-160 ms.
* 120-deep nesting: 59 ms.
* Mesh of 300: 1.0 s. Mesh of 1000: `Budget` in 3.0 s.

All deterministic. Peak allocation divided by the bytes charged, on 13
shapes: 0.01 to 0.66, never above 1. maxrss was 356 MiB.

### Mutants

20 of 20 killed:
* shown pass run over every route;
* hidden pass removed;
* shown target check dropped;
* the shown graph dropping every hidden route (the old behaviour);
* the shown graph keeping hidden routes;
* the reach walk hiding at mid globs;
* the carry's hidden flag ignored;
* `dominates` forced false or true;
* carrier counted as its own binder;
* single binder treated as blocking;
* one dominator pass;
* search passing through binders;
* answers copied between carriers;
* hidden-first reach pass dropped;
* M1 to M5.

### Gates

All green:
* `cargo test -p raf-source` in debug and release, and `cargo test
  --workspace`, which includes `docs_contract` and `repo_gates`;
* `make fmt clippy doc deny reuse msrv issue-refs test-release`: rc 0;
* `make docs-sync` twice: no diff;
* `make fuzz`: rc 0, 5000 runs of each target;
* `cargo workplan check`: clean.

`source-index.md` still changes nothing outside the PROPOSED blocks but
`state` and the bindings. Hygiene grep: 0. `cross-check` was not run, as its
targets are not installed on this machine.

## Sixth-round findings

Both come from my differential and lie in the item engine's `doc_hidden`
for glob-bound items (`reexport_hidden`, `By::Glob`). That code predates
the carry, and my earlier rounds did not reach it.

### REQUIRED

1. **An item is reported shown through a glob route that does not carry it
   publicly. `cargo doc` hides it.** Example:
   `mod c { pub struct Cell; } mod m0 { pub use crate::c::*; use std::rc::Rc as Cell; } pub use m0::*; #[doc(hidden)] pub use c::*;`
   * `m0`'s private import of `Cell` shadows its glob, so the only public
     route to `demo::Cell` is the hidden glob of `c`.
   * The listing nevertheless gives `demo::Cell` with `doc_hidden: false`,
     and it appears in the default `exported_items` listing.
   * `cargo doc` writes no `struct.Cell.html`. The carried version
     (`pub use std::cell::Cell;` in `c`) is correctly hidden and left out by
     default. rustc names `demo::Cell` in both.

   Cause: `reexport_hidden` (`By::Glob`) counts every `pub` glob of the
   reader whose view holds the item (`routes.holding(name).filter(view_holds)`),
   whether or not that glob is a public route for it. The index's view of
   `m0` still holds `c::Cell`, because the unbound private import is not seen
   to shadow it in the view. `binding_site` picks its declaration the same
   way.
   Fix: count only the routes whose place has a public route, using
   `public_route(route.target, at, name, id)`, which `decided` already
   caches. Add this repro beside its carried version.

2. **A `pub(crate)` glob on the shown route: the two engines disagree.**
   Example:
   `mod c { pub struct Cell; } mod m1 { pub(crate) use crate::c::*; } pub use m1::*; #[doc(hidden)] pub use c::*;`
   * The item version is listed shown.
   * The carried version (`pub use std::cell::Cell;` in `c`) is hidden and
     left out by default.
   * The same holds with the reader in a `pub mod r`.
   * `cargo doc` documents the item (`struct.Cell.html` is written). But
     `cargo doc` also documents carried outside items whose only route is
     hidden (round five), so it does not settle the question.

   By the Contract's rule, the declaration that exports the name is the
   hidden glob of `c`: `pub use m1::*` brings `Cell` in only at `pub(crate)`.
   So the carried answer is the one the Contract gives, and the fix for
   Required 1 makes the item engine agree. If rustdoc's answer is preferred
   instead, the carry must count `pub(crate)` routes as shown. Either way the
   two engines must give one answer, stated in the Contract, with this repro
   as a fixture. The differential test's claim of equality is currently
   false outside its family.

No other wrong answer was found in this round.

Sixth-round verdict: BLOCKED (sixth-round Required 1 and 2).

## Seventh round (final on `doc_hidden`): one export rule for both engines

What changed:
* `carrying_routes` now counts only the reader's `pub` globs whose target has
  a public route for the item. Both `reexport_hidden` and `binding_site` use
  it.
* The Contract states one export rule for items and carried re-exports: a
  glob whose target brings the name in only at `pub(crate)`, or keeps it
  behind a non-`pub` binding or a shadowing import, exports nothing.
* The Constraints record that, for repro 2, `doc_hidden` follows that rule
  where rustdoc documents the item.
* New fixtures for both repros (item and carried; reader at the root and in a
  `pub mod`), and a seeded randomized differential (450 crates in the suite).

Rechecked in a fresh scratch worktree with a rebuilt probe and harness.

### The two repros and the hidden/shown set (carried vs item, with rustc)

* Repro 1 (unbound private import shadows the mid glob): hidden and absent by
  default in both engines; rustc names `demo::Cell`. It now matches
  `cargo doc`.
* Repro 2 (`pub(crate)` mid glob beside a hidden glob): hidden and absent in
  both, with the reader at the root and in `pub mod r`; rustc names it. The
  control with a `pub` mid glob is shown in both.
* The 23 hidden/shown cases of rounds five and six, plus two new ones (a
  `pub(crate)` glob as the only route: absent, and rustc refuses; a
  `pub(super)` glob beside a free `pub` one: shown): all 27 agree between the
  engines and with rustc.

### Independent differential over fresh seeds

My generator ran over three new seeds (101, 202, 303), with 0.70 to 1.00 of
globs `pub` and the rest `pub(crate)`, private or `pub(super)`. It now also
uses `pub(crate) struct Cell;` binders, and `pub(crate)` reader globs.
Results:
* 1400 crates.
* 77 differences, every one of the same kind: the item version fails the
  listing with `Resolve(Ambiguous)`, because a `pub(crate) struct Cell`
  binder and the carrier's `pub struct Cell` are two items under one name.
  The carried version lists `demo::Cell`.
* I compiled all 77 carried crates with a downstream crate: the carried
  listing's presence matches rustc in 77 of 77.
* The item side's refusal is the documented loud `Ambiguous` rule ("also
  where rustc accepts the crate ... one of whose items is `pub(crate)`").
* No difference in flag or presence anywhere else.
* rustc spot checks of presence: 72 of 72 agree, and none of the sampled
  crates was invalid.

### Standing probes

The 34 earlier scenarios and the 25 binder scenarios give digests identical
to round six. The only listed path rustc refuses is still the documented
`rule2g`, and the two ambiguity cases still fail loudly.

### Timings and memory (release, one listing; load in brackets)

| shape | n = 1000 | 3000 | 5000 | 10000 |
| --- | --- | --- | --- | --- |
| carried-import shape | 14 ms | 39 ms | 49 ms | 105 ms (3.9 s) |
| sibling shadow | 14 ms | 44 ms | 90 ms | 127 ms (24 s) |
| binder on the shown route, free hidden route | 18 ms | 81 ms | 110 ms | 280 ms (71 s) |
| two binders with routes | 84 ms | `CarryBudget` 0.14 s | | |

New item fan-in shapes, which exercise `public_route` once per carrying
route:

| shape | listing | load |
| --- | --- | --- |
| 300 globs × 300 names | 0.18 s | 0.15 s |
| 1000 globs × 300 names | 0.73 s | 0.65 s |
| half `pub(crate)`, 300 globs × 300 names | 0.15 s | |
| half `pub(crate)`, 1000 globs × 300 names | 0.70 s | |

Other shapes:
* 20,000 item globs: 0.30 s.
* 20,000-glob chain: 0.23 s.
* 20,000 renames: 0.17 s.
* 120-deep nesting: 44 ms.
* Mesh of 300: 0.67 s. Mesh of 1000: `Budget` in 1.8 s.

All deterministic. Peak allocation divided by the bytes charged, on 13
shapes: 0.01 to 0.66, never above 1. maxrss was 364 MiB.

### Mutants

15 of 16 killed:
* the `carrying_routes` public-route check dropped;
* "every carrying route hidden" changed to "any";
* shown pass run over every route;
* hidden pass removed;
* the shown graph dropping every hidden route;
* the reach walk hiding at mid globs;
* `dominates` forced false or true;
* search passing through binders;
* hidden-first reach pass dropped;
* M1 to M5.

The survivor: `binding_site` choosing the last carrying glob instead of the
first. It changes only which declaration (`written`, file and line) a
re-export of another crate reached through two or more `pub` globs reports.
See Advisory 2.

### Gates

All green:
* `cargo test -p raf-source` in debug and release, and `cargo test
  --workspace`, which includes `docs_contract` and `repo_gates`;
* `make fmt clippy doc deny reuse msrv issue-refs test-release`: rc 0;
* `make docs-sync` twice: no diff;
* `make fuzz`: rc 0, 5000 runs of each target;
* `cargo workplan check`: clean.

`source-index.md` changes nothing outside the PROPOSED blocks but `state` and
the bindings. Hygiene grep of added lines: 0. `cross-check` was not run, as
its targets are not installed on this machine.

## Final findings

### REQUIRED

None. No wrong answer on whether a name is exported, and no `doc_hidden`
disagreement between the engines, was found in 1400 fresh crates, 27
hidden/shown cases, 25 binder and 34 standing scenarios.

### ADVISORY

1. **Documented loud differences, not wrong answers.**
   * Repro 2: `doc_hidden` follows the Contract's export rule where rustdoc
     documents the item (stated in the Constraints).
   * A name that a local `pub(crate)` item and a carried outside re-export
     share is listed as the re-export, while two local items under one name
     fail with the documented `Ambiguous`. All 77 differential differences
     are of this kind, and the carried listing matches rustc in each.
   * The several-binder `CarryBudget` refusal.
   * The documented second unmodelled rule (`rule2g`).
2. Add a fixture that pins `binding_site` to the first carrying glob, for
   example a re-export of a workspace crate's item through two `pub` globs,
   checking `written` and `line`. It kills the one surviving mutant.
3. The round-one Task 11 hand-offs still stand: `bindings.examples` is empty,
   `reconcile` reports a `tense-mismatch` on the delivered "not yet"
   sentence, and the crate doctests need reordering when the
   Usage/Examples PROPOSED blocks are resolved.

VERDICT: PASS
