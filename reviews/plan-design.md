# Plan design review (SOLID / ETHOS)

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/14-raf-source-model-rustc-s-visibility-of/.stage/raf-source-model-rustc-s-visibility-of/plan.md`

## Code read

- `crates/raf-source/src/settle.rs`: `Origin`, `vis_key`, `Bound::offer`/`widened`, `is_visible`, `lookup_seen`, `seen_by`, `lookup_in`, `blocked`/`blocked_for`, `glob_origin`, `settle_named`, `covers_globbed`, `settle_glob`, module doc (the "answers only grow" invariant, lines 4-17).
- `crates/raf-source/src/glob_scope.rs`: `View`, `GlobTables` (`vis_of`, `in_path`, `view_of`), `glob_visibilities`, `allowed`, `class_for`, `globbed`, `blocked_class`, `export_ids`, `make_view`, `read_glob`, `refresh`, `pass_on`, `arrive`.
- `crates/raf-source/src/resolve.rs`: `Hit`, `Fail`, `Heads`, `Resolution`, `ResolveError`, `start`, `walk_noting`, `origin_of`, `chain_for`, `resolve_in`, `resolve`.
- `crates/raf-source/src/workspace.rs`: the public query surface (`resolve`, `resolve_in`, `resolve_in_package`).
- `crates/raf-source/src/facts.rs`: `Vis`.
- `crates/raf-source/src/glob.rs`: workspace-member patterns; unrelated to `use` globs, nothing here touches it.
- `crates/raf-source/tests/use_resolution.rs`: `lib`, `path_of`, `outcome`, `permutations`, `one_outcome_in_every_order`, `every_order_in`, `named`, `answer`, `rotations`, and the two fuzz-derived fixtures at lines 3000-3113.

The plan's design content for the two rules is two sentences (Task 2: "the new binding carries the visibility of the weaker of the import and the binding it re-exports. Order independence follows from computing visibility as a meet in the existing fixpoint"; Task 3: "the reader gets rustc's answer (refused as ambiguity; the E0573 shape refused as not found)"). Each of the findings below is a place where those sentences meet a concrete structure in the code that does not carry them.

## BLOCKING findings

### B1. The oracle has no index question to ask: `resolve` has no viewer

Plan line (Task 1): "then asks the index what the name resolves to for the probing module."

Code: `Workspace` exposes `resolve`, `resolve_in`, `resolve_in_package` (`workspace.rs:373-401`), all of which call `Index::resolve` with a canonical path from a crate root (`resolve.rs:1316-1353`). The walk sets a viewer only for a `use` path: `let viewer = matches!(heads, Heads::Use { .. }).then_some(from);` (`resolve.rs:685`), and a canonical walk uses `Heads::Local` (`resolve.rs:1152`). So the index answers `demo::m::c::A` whether or not any module may name it; privacy is never consulted on a canonical walk.

Consequence: rustc's answer to a `use` probe depends on the probing module's access; the index has no query that depends on a module. Rule 1 makes a binding *less visible* (c::A becomes visible within m only), so a probe of `crate::m::c::A` placed at the crate root fails under rustc while the index resolves it correctly to `x::A` (the binding exists; the root may not name it). The oracle as written reports that as a disagreement, and there is no classification that separates it from a real rule-1 miss. Conversely a probe placed inside `m` passes under rustc and the index also resolves it, so that placement cannot detect the rule at all. Task 1's acceptance ("reports every disagreement", "no rule-1 disagreement") is therefore unverifiable with the designed seam.

REWRITE TO: decide the index question before building the generator, and make it one of these two:

1. Add one viewer-aware query to the public surface, `Workspace::resolve_from(module_path: &str, path: &str) -> Result<Resolution, ResolveError>`, implemented as the existing `use`-path walk (`walk_noting(from, segs, leading_colon, ns, Heads::Use { test: false, own: None }, ...)`, which already applies `lookup_seen` with `viewer = from`), returning `NotFound` when the viewer cannot name the binding. The rustc probe is then `use <path> as _probe_N;` written into module `module_path`, and both sides answer the same question. This is the only option under which the oracle checks the reader-side effect of rule 1 through a named path as well as through a glob.
2. Keep `resolve` as the index question and constrain the generator so that the probe's own access never differs from the index's: every probe is written at the crate root and probes only names that a *glob of a `pub mod` at the root* brings in (`pub mod g { pub use crate::...::*; }`, probe `crate::g::A`). Then rustc's acceptance is exactly "does g's glob bring in A", which is exactly what `resolve("demo::g::A")` answers, and rule 1 is observed through the glob table (a capped c::A must not arrive in g). Record in the plan that the oracle then never probes a capped binding by its own path.

Either is sound; the plan must name one. Option 1 is the stronger oracle and is what "asks the index the same question" needs.

### B2. Rule 1's "meet" produces a visibility the class model cannot carry

Plan line (Task 2): "the new binding carries the visibility of the weaker of the import and the binding it re-exports. Order independence follows from computing visibility as a meet in the existing fixpoint."

Code:

- A named import's binding is built with `vis: decl.vis.clone()` (`settle.rs:2232`); the walk's `Hit` carries `via_glob` and `last: Option<Source>` only (`resolve.rs:31-34`), so the visibilities of the glob route are not in hand at `settle_named`; they are in the `Origin` at `hit.last` (`origin_of`, `resolve.rs:1098-1110`, reads `defs` then `globbed`).
- Every visibility a binding of module `n` carries must be one of `vis_of[n]`, or `allowed` panics: `.expect("every visibility a binding of a module carries is one of the module's")` (`glob_scope.rs:316-319`). `vis_of` is computed once per load from the items and `use` declarations of each module (`glob_visibilities`, `glob_scope.rs:239-252`), before any settling; a view's class is a `Box<[bool]>` of that fixed length (`class_for`, `glob_scope.rs:324-329`; `view_of` keyed on it, `glob_scope.rs:102`).
- `Vis` is interpreted relative to the binding's home module: `Vis::Inherited` in `n` means "within n" (`is_visible`, `settle.rs:369`).

Consequence: the real case (the fuzz shapes in `use_resolution.rs:3038,3088` are `pub mod X { use super::*; }`) is a child `c` of `m` re-exporting a name it reads through `m`'s private glob: `mod m { use crate::x::*; pub mod c { pub use super::A; } }`. rustc's cap is "within m". The meet of `pub` and "within m" is not a visibility of `c`: `Vis::Inherited` in `c` means "within c", narrower than rustc (a sibling of `c` inside `m` may name `c::A` under rustc; the index would refuse it, a new disagreement), and any `Vis` naming `m` is absent from `vis_of[c]`, so the first glob that reads `c` hits the `expect` at `glob_scope.rs:318`. "In the existing fixpoint" is not a design: the fixpoint's `Bound::offer`/`widened` can hold the meet once it is a `Vis` of `c`, but nothing in the plan says what that `Vis` is or how the class vectors learn it.

REWRITE TO:

1. Normalise the cap to an ancestor of the importing module. A reader can see a glob binding of `m` with vis `v` only if it lies within the module `v` names (`m` itself for `Inherited`/`self`, `m`'s parent for `super`, the walked module for `in path`, the crate for `crate`, nowhere restricted for `pub`). So the cap for an import in `n` is always "within `p`" for some ancestor-or-self `p` of `n`, or the crate, or nothing. Represent it as the existing `Vis` shapes: `Vis::Inherited` when `p == n`, `Vis::Restricted("crate")` for the crate, else `Vis::Restricted("in <crate-relative path of p>")` (`is_visible` already walks such text with `crate` as a first segment, `resolve.rs:593-596`). The meet of two visibilities of `n` is then the deeper of two ancestors on one chain, a total order, so it is the same whichever route is found first.
2. Pre-seed `vis_of[n]` in `glob_visibilities` with that bounded set: `Inherited`, `Restricted("crate")`, and `Restricted("in <path>")` for each proper ancestor of `n` (cost: depth of `n` entries per module, fixed before settling, so class vectors keep their length). Record that `in_path[n]` then becomes true for every nested module (`glob_scope.rs:269-272`), so `touch` re-tries its globs on changes (`settle.rs:890-892`); if that cost matters, split the flag so only visibilities that need a walk set it.
3. At `settle_named`, compute the cap from the route, not the walk: for the item's `hit.last`, take `origin_of(last, id)` and, when that origin is a glob binding (`via_glob`, or the `defs` origin of an already-capped re-export), meet each of its `visibilities()` with `decl.vis` and offer the result as the new binding's `vis`/`also`. `Bound::offer` already unions visibilities monotonically (`settle.rs:156-184`), which keeps "answers only grow". State that the cap applies to the last segment's binding only; intermediate segments are already checked by `lookup_seen`.

### B3. Rule 2 has four places to live, and the plan names none

Plan line (Task 3): "When the globs of a module bring a name in ambiguously and one of the items arrives through a glob the reading module cannot access, the reader gets rustc's answer ... not the other item."

Code: the "only what the reader can access" filter is implemented independently at four sites, and a reader meets the name through different ones depending on how it reads and on the order the tables settle in:

- `seen_by` (`settle.rs:519-539`): filters `defs` and the globbed `Bound` for a named `use` path reading `module` as `viewer` (`lookup_seen`, `settle.rs:513`); also used by `covers_globbed` (`settle.rs:2333`).
- `export_ids` (`glob_scope.rs:426-456`): what a target exports to a class, used by `refresh` when a name of the target changes.
- the gathering loop in `make_view` (`glob_scope.rs:596-628`): the same computation written out again for a view made after the names arrived.
- `arrive` (`glob_scope.rs:898-903`): the incremental path; a glob the class cannot access is skipped per view, so a view never learns that a second item exists under the name.

Consequence: today two globs of `m`, one private carrying `a::Y`, one `pub` carrying `b::Y`, export `{b::Y}` to an outside class by every one of these paths, and rustc refuses. If the fix lands in `export_ids` and `make_view` but not in `arrive`, a view made before `a::Y` arrives keeps `{b::Y}` while a view made after holds the refusal, and the answer depends on declaration order, which is the very property the every-order fixtures guard. If it lands in the glob tables but not in `seen_by`, `use crate::m::Y;` from outside `m` resolves `b::Y` while `use crate::m::*;` from the same module is refused.

REWRITE TO:

1. Make `export_ids` the one computation of what a target exports under a name to a class, and have `make_view` call it per `(name, at)` instead of carrying its own loop (the plan already touches both files; the duplication at `glob_scope.rs:596-628` is where rule 2 would silently diverge).
2. Define rule 2 as a property of the candidate set, computed once: for a target `t`, name, name space and class, let `all` be the distinct items *every* glob of `t` brings in under the name (`globbed(t, name, at)`, unfiltered) and `seen` those the class may access. Export `seen` when `all` has one item; export `all` when `all` has more than one and `seen` is non-empty; export nothing when `seen` is empty. This is monotone (the set only grows), so it fits "answers only grow" and needs no new state in `View`.
3. Apply the same rule in `arrive`: when ids arrive for a name through a glob the class cannot access, add them to the view only if the view already holds a different item under that name from an accessible glob; when ids arrive through an accessible glob, also add the items the module's inaccessible globs already hold under that name. Both are "add", never "remove".
4. Apply the same rule in `seen_by` for the globbed branch only (`settle.rs:513`): when the globbed `Bound` has more than one item and the viewer sees at least one, return the whole `Bound`. Leave the `defs` branch as it is.

### B4. "The E0573 shape refused as not found" needs a retraction the tables forbid

Plan line (Task 3): "(refused as ambiguity; the E0573 shape refused as not found)". NEEDS-JUDGMENT 2: "the rare E0573 case gets its own fixture whose expectation is 'refused'".

Code: the settling invariant is "every answer only grows as the tables settle ... a decision never takes back an answer" (`settle.rs:13-17`); views hold item ids only (`Lists`, `glob_scope.rs:62`) and `add_to_view` only inserts (`glob_scope.rs:460-513`). `ResolveError::NotFound` is produced only when a walk finds nothing (`resolve_in`, `resolve.rs:1259-1260`).

Consequence: for the index to answer `NotFound` for a name whose view already holds an item from an accessible glob, something must remove that item from the view or from the walk's result after the second item arrives, which is a retraction; the alternative is a new "refused" marker in `View`, a second representation of ambiguity beside "more than one id". Either way the E0573 case takes a different mechanism from the E0659 case for what is the same rustc rule, and the plan's own oracle definition ("a probe rustc rejects that the index resolves" is the only disagreement on that side) does not distinguish the two errors. The repository already took this stance for a fuzz-derived case: "`Ambiguous` in every order is deliberate: it agrees with rustc wherever rustc reports an error and does not depend on the order" (`use_resolution.rs:3066-3070`).

REWRITE TO: the index answers `Ambiguous(all)` for both shapes (B3's rule), the E0573 fixture asserts `Ambiguous` with both items, and NEEDS-JUDGMENT 2's disposition is rewritten to say so. Drop "refused as not found" from Task 3.

### B5. The oracle test "skips" when `rustc` is missing

Plan line (Task 1): "Skips with an explicit failure message when `rustc` is unavailable (it is always available under cargo)."

Consequence: a test that skips passes; a CI box or sandbox without `rustc` on `PATH` then reports the oracle green with zero programs compiled, and Task 2/3's acceptance ("the oracle has no rule-1 disagreement") is satisfied by an oracle that ran nothing. That is a capability check that degrades silently (§H).

REWRITE TO: fail. Resolve the compiler once, from the `RUSTC` environment variable when set (cargo sets it for build scripts; the test runner may not) and otherwise `rustc` on `PATH`, and `panic!` with the resolved command and the spawn error when it cannot be run. Also assert that the generated program count is non-zero before comparing.

## ADVISORY findings

### A1. Two compile strategies in the oracle

Plan line (Task 1): "one probe item per query, in one compile with error codes per query where possible".

"Where possible" leaves a second path (one compile per probe) that the plan does not define and that would make the oracle's failure attribution differ between programs. REWRITE TO: one strategy: one compile per program with `--error-format=json`, every probe a `#[allow(unused_imports)] use <path> as _probe_<n>;` on its own line, and the JSON spans' lines mapped back to probe numbers; a diagnostic the map cannot place fails the test.

### A2. Environment-variable scaling of the same test

Plan line (Task 1): "with the count raised by an environment variable for the full re-run".

This is a modal knob on a committed test (§O), though a small one. If kept, name the variable in the plan and have the test print the seed list and count it used so the Task 4 record is reproducible from the output alone; an alternative with one path is a `#[ignore]`d test with the large list that `cargo test -- --ignored` runs.

### A3. Reuse of the every-order helpers

Task 2 and Task 3 say "fixture in every order". `use_resolution.rs` already has `permutations` (line 440), `one_outcome_in_every_order` (459), `every_order_in` (1408), `rotations` (3000), `answer` (1786) and `named` (1372). The plan should say which helper each fixture uses so the new fixtures do not add a seventh permutation loop. For the rule-2 fixture with several blocks, `rotations` is the one that keeps the load count linear.

### A4. `Hit` and the route's visibility

If B2 is resolved by reading `origin_of(hit.last, id)` at `settle_named`, note that `hit.last` is `None` for an associated member and for a path that ends at a module (`resolve.rs:740`, `settle.rs:2356`); the cap applies only when `last` is `Some` and the origin there is reached through a glob. Say so, or `is_visible` will be asked about a route that does not exist.

## Sound as planned

- Placing the fixtures in `use_resolution.rs` beside the existing fuzz-derived cases.
- Capability-document workflow and README tasks: no design content to review.
- Rule 1's reader side needs no new reader code once B2 is done: `seen_by` and `allowed` already consult every visibility a binding carries.

VERDICT: NEEDS-REVISION
