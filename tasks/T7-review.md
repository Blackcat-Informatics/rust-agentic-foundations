# Current status

The first review below blocked; the findings were fixed and the later recheck sections (at the end of this file) passed. Earlier sections are kept as written.

VERDICT: PASS

---

# Task 7 review: raf-source index and resolution

## Scope

Staged diff on top of e15dd4b (13 files). Checked against plan sections A and C7,
the D rows Items/docs and Uses/resolve, every completeness row whose Task column
includes 7, "Task 7", and the validation.md open finding on `mod test_crate;`.
Read in full: `src/item.rs`, `src/index.rs`, `src/resolve.rs`, the `workspace.rs`,
`lib.rs`, `error.rs`, `facts.rs` diffs, the source-index.md diff, and the test names
of the four new test files.

## Evidence

- `cargo test -p raf-source --locked --test use_resolution --test symbol_index`:
  17 + 18 passed (orchestrator reports the full suites, docs_contract, repo_gates,
  clippy and fmt green; not re-run).
- A throw-away probe crate in /tmp (path dependency on the staged crate, no change
  to the worktree) loaded MemoryTree fixtures and called `resolve`. Results are
  quoted in the findings.
- Greps of the new src for the section A banned constructs (output summarised in
  finding 7).

Not checked: `make reuse`, `msrv`, `deny`, `cross-check`, `fuzz` (taken from the
implementer's report); the release-mode run; the doctest bodies beyond reading them;
census consumers (none exist yet).

## Findings

1. BLOCKING. Explicit imports are bound against stale glob bindings, so the answer
   depends on declaration order and can be a false `Ambiguous`.
   `settle_named` (`resolve.rs:571-615`) walks through `lookup`, which falls back to
   `globbed` while the module's explicit import that should shadow it is still
   queued. The importer's binding is never revisited: the later "settle the globs
   against the final definitions" pass (`resolve.rs:~500-520`) rebuilds only glob
   tables, and an importer is re-queued only if it waited on a missing name.
   Probe (fixture):
   `mod x{pub struct A;} mod y{pub struct A;}`
   `mod m{ pub use crate::x::*; pub use crate::p::A; }`
   `mod n{ pub use crate::m::A; }`
   `mod p{ pub use crate::y::A; }`
   `resolve("demo::n::A")` returns `Err(Ambiguous([ItemId(1), ItemId(3)]))`.
   rustc resolves `n::A` to `y::A`, and `demo::m::A` in the same workspace does
   resolve to `y::A`. When the chain is declared in the other order the result is
   correct, which is why the existing tests (`definitions_and_explicit_imports_shadow_globs`,
   `explicit_imports_shadow_globs_for_the_modules_that_glob_the_importer`) pass: both
   have the explicit import settled before its readers. This breaks C7 ("rustc
   precedence", "Ambiguous only for real conflicts") and the Contract sentence in
   source-index.md "an explicit import shadows a glob ... as well". Fix direction:
   after the queue drains, re-derive every named import from the final tables
   (clear the `defs` that came from `use`, re-walk, repeat until no change), or
   refuse to read a `globbed` binding for a name that a still-pending explicit import
   of the same module and name space could bind. Add a test pair with the importer
   between the glob and the explicit import (the fixture above and its reverse).

2. BLOCKING (section A, literal). `workspace.rs:138-158` turns an absent reach into
   a default: `package: package.unwrap_or_default()` yields `""`,
   `reached.iter().all(..)` yields `test == true` for an empty list, and
   `targets.get(..).is_some_and(..)` yields `false`. By construction a loaded file
   has at least one reach, so use a typed error (or build the field from the first
   reach directly) instead of a default. Section A names `unwrap_or*` explicitly.

3. BLOCKING (section A, literal). `Workspace::test_fns` (`workspace.rs:~305-325`) uses
   `filter_map` with `?` on `targets.get(..)` and on the two `strip_prefix` calls, so a
   test function whose path does not start with its target's crate path is silently
   omitted. The invariant holds today (item paths start with the target crate path),
   but the data feeds `cargo test` selection, where a dropped test is a silent miss.
   Return `Result<Vec<TestFn>, _>` or assert the invariant as a typed error.

4. NON-BLOCKING. Glob imports bring in private items, so a private name can create
   an `Ambiguous` that rustc does not report. Probe:
   `mod a{struct Foo;} mod b{pub struct Foo;} mod m{ pub(crate) use crate::a::*; pub use crate::b::*; }`
   `resolve("demo::m::Foo")` is `Ambiguous`, rustc picks `b::Foo`. The Contract states
   "Visibility is not checked", which is adequate for "does this path name a real item"
   and no consumer asks for a path whose glob conflict is private. It does stretch
   "Ambiguous only for real conflicts"; the Contract should say that a glob brings in
   private names too, or `visible()` (`resolve.rs:384`) should drop names that are not
   visible from the importing module. Advisory.

5. NON-BLOCKING. Trait impls on foreign or non-path self types are keyed by the
   trait path text as written (whitespace removed). Probe: `impl Display for Vec<X>`
   and `impl fmt::Display for Option<X>` give `Display::fmt` and `fmt::Display::fmt`;
   `std::fmt::Display::fmt` is `NotFound`. C7 says only "records under
   `<trait>::<method>`" and names the written form for the self type; the Contract in
   source-index.md states "the written trait path", so code, doc and fixtures agree
   (`impl_members_are_recorded_under_the_type_or_under_the_trait...`). Consumers
   (census, bindings) must therefore know the spelling. Advisory improvement: when the
   trait path resolves to a local trait, record under its canonical path (exact text
   stays available in `Item::trait_path`); foreign traits cannot be canonicalised, so
   written text is the only option there. Also note two impls with the same spelling
   (and on a local type, `Display::fmt` and `Debug::fmt` both under `demo::X::fmt`)
   merge into one `Resolution` with several sites rather than `Ambiguous`; this is
   the cfg-twin rule applied to genuinely different impls. Acceptable, worth a
   Contract sentence.

6. NON-BLOCKING. Identity (question 1). One `Item` per module path, `ItemId` a dense
   counter, `Item::source_key()` returns `(file, span.start_byte)`, `item_at` returns
   all innermost items (a `Vec`) and `items_in` is ordered by `(start, path, id)`.
   That satisfies C7: the identity is exposed and a census can group by it (a
   `BTreeMap` keyed by `source_key()`), and `item_at(file, offset)` serves the
   enhancement need. No grouping helper is provided and `source_key` has one test use
   (`symbol_index.rs:430`); a test showing two paths for one source item sharing a
   key (the `#[path]` or `cfg_attr` multi-reach fixture) would pin the contract.
   Advisory.

7. NON-BLOCKING. Section A greps of new src:
   - `resolve.rs:261,320,341,842`, `index.rs:556`, `resolve.rs:416` use
     `entry().or_insert`: first-wins on `Hit.via_glob`/`Origin` when one item is
     reached by two routes. Deterministic (BTreeMap order) and the item set is not
     affected; only `via_glob` and `chain` take the first route. Rule A bans the
     pattern; prefer an explicit merge (`via_glob` true only if every route is a glob).
     `insert_def`/`insert_globbed` return `false` on an occupied key and keep the first
     origin (same remark). `index.rs:556` is a placeholder overwritten on the next line.
   - `resolve.rs:633-636` visited-set skip in `nominal_types`, `resolve.rs:639`
     `let Ok(..) else { continue }` and `resolve.rs:653` `_ => {}`: an alias cycle
     (`type A = B; type B = A;`) ends with no local type and so with
     `InherentImplSelf`, which is typed, so the skip does not hide a result. The
     `walk` error here is `Missing` only (`SuperAboveRoot` is refused earlier). List
     the non-type kinds instead of `_ => {}`; add a comment saying why the guard is
     sound.
   - `resolve.rs:378` `glob_targets` `Err(_) => empty` and `index.rs:444` `seen.insert`
     skip (an identical file/target/module-path reach): both benign for the same
     reason; same remark.
   - `index.rs:170` `unwrap_or_default` and the dropped `use {self}` with no prefix:
     rustc rejects that source, the index accepts it silently. Make it a typed error.
   - `allow(clippy::too_many_arguments)` at `index.rs:233` has a reason. No `unsafe`,
     no `.ok()`, no panicking index or `unwrap`/`expect` on input outside doc/test code
     (internal `self.items[id.0]` indexing rests on ids minted by the builder).
   - No plan labels, precedent names, "ported", or tracker numbers in new src, tests
     or docs.

8. NON-BLOCKING. A cycle that goes through a glob is reported `NotFound`
   (`mod a{pub use crate::b::*;} mod b{pub use crate::a::X;}`: `demo::a::X` is
   `NotFound`). `depends_on_a_cycle` follows named imports only. rustc reports an
   unresolved import here, so `NotFound` is defensible. For named-import cycles the
   detection is sound in both directions: `Cyclic` is returned only when `found` is
   empty and an unresolved import of a missing name reaches a cycle in the blocker
   graph, so a resolvable chain is never reported `Cyclic` (verified by
   `a_cycle_of_imports_is_cyclic_beside_a_chain_that_ends`, including the
   self-referencing pair).

## Adjudication of the remaining questions

- Q3 resolver: precedence, definitions-over-globs, re-queue fixed point with no cap
  (5000 hops, both orders, linear), real-conflict Ambiguous for glob/glob, `super`
  above root as a typed error with file and line, `::x`, `extern crate as`, raw
  identifier normalisation per segment and the `test_crate` Ambiguous case are all
  implemented and tested. The exception is finding 1.
- Q4 visibility: acceptable for the capability ("does this path name a real item")
  given the Contract states it; see finding 4 for the one place it leaks into a
  verdict.
- Q5 fixtures: every row in the Task 7 list has a named test. Weak spots: the
  `test_crate` assertion is `ids.len() >= 2` (could assert the exact two ids), and
  the shadowing tests lack the importer-between-glob-and-explicit order (finding 1).
- Q6 `module_tree.rs`: the added `struct Holder<T>(T);` is legitimate. The test
  asserts recorded use trees and impl headers as written; an inherent impl on an
  undeclared type is now a typed load error by design (the new
  `an_inherent_impl_on_a_type_that_does_not_resolve...` test pins that), and no
  assertion in the test was weakened.
- Q7 docs: the Symbol index, Use-alias index, Usage (`no_run` stated, hidden-line
  note) and Examples text match the code and the `usage_docs.rs` equality tests,
  except the sentence "an explicit import shadows a glob ... for the modules that
  glob the importer as well" and "two imports of one name that reach different
  items" which finding 1 shows to be order-dependent. Docs say the trait path is as
  written, matching finding 5.

VERDICT: BLOCKED

## Recheck

Reviewed the restaged diff (new `settle.rs`, `impls.rs`, `resolve.rs` and workspace
changes). Method: read the new text, and ran a throw-away probe crate in /tmp against
the staged crate (removed afterwards; the worktree is untouched). Not re-run: the
full suites and gates (orchestrator reports them green).

### Blocking findings from the first pass

1. Order-dependent explicit-vs-glob binding: DISCHARGED. My original fixture now gives
   `demo::n::A = demo::y::A` and `demo::m::A = demo::y::A`. Further probes, all correct:
   a glob carrying a name that an import binds late (both declaration orders), a
   glob cycle with a real definition behind it, a cycle beside a real definition,
   `use a::{self as b}`, and the private-vs-public glob case (`m::Foo` is `b::Foo`).
   The 1440-order permutation test (`use_resolution.rs:375-432`) pins the invariance.
2. Defaults in workspace file construction: DISCHARGED. `LoadErrorKind::Internal`
   (`error.rs:301`) replaces the defaults; the unwrap_or in `workspace.rs` is now only
   `Rc::unwrap_or_clone`, which is not a fallback.
3. `test_fns` silent drop: DISCHARGED. It returns a `Result` with a typed `Internal`
   error and a unit test (`workspace.rs:300-312`, `:434`).

### Adopted advisory items

- Glob visibility (4): DISCHARGED. `is_visible`/`visible` filter by importer; probe above.
- Rank-based merges (7): DISCHARGED. `Origin::rank` (non-glob beats glob, then smallest
  link) makes the recorded origin independent of discovery order. No `or_insert` remains
  in settle/impls/resolve/index.
- `nominal_types` `_ => {}` (7): DISCHARGED. Kinds enumerated with a reason; the
  `visited` guard has a comment and ends in a typed `InherentImplSelf`.
- `use {self}` (7): DISCHARGED. Typed `UnsupportedItem` error ("has no path before
  `self`"); `use a::{self}` and `{self as b}` bind the module (probe).
- Identity grouping (6): DISCHARGED. `items_by_source()` and `same_source()` exist with
  tests at `symbol_index.rs:643-670`.
- Canonical trait keys (5): STILL-OPEN, in part; see findings A and B below.

### Scrutiny points

Drain-and-fail (can an import that would resolve be settled as failed?): not shown to
happen. At queue drain every glob has been processed to a fixed point, so an import
whose blockers (`blockers(p, false)`) are all settled can only gain a binding through a
new name, and a new name always wakes it: a "failed" import is still re-walked when
`pending.wake` fires, because `drain` does not skip settled positions and `settle_use`
only decrements the unsettled count once. So a premature failure can at worst let
another reader see a glob binding for a window, and the late-binding probes
(`globlate`, `globlate2`, `vallate`) gave correct results in both orders. A residual
corner remains and is non-blocking: an import counts as settled once it resolves in
any one name space (`use_resolved` set in the first matching space), so a reader may
see the glob binding in a second name space of the same name until it is bound.
Cycle detection still reports `Cyclic` for named cycles and finds the real definition
when one exists (`cycle+real` probe).

Std prelude table (`impls.rs:16`): every entry I checked names the right defining
path, so it is not a guess where it answers, but it is incomplete and it fails hard on
valid code (finding A).

### New findings

A. BLOCKING. Valid sources fail to load with `UnresolvedTrait`, which aborts the whole
   workspace. Probes:
   - `pub struct L; impl TryFrom<L> for u8 { .. }` (edition 2021 prelude, `TryFrom` and
     `TryInto` are missing from `PRELUDE_TRAITS`): `src/lib.rs:1: the trait TryFrom of an
     impl cannot be named by an absolute path`. `impl TryFrom<Local> for <foreign>` is
     a common shape. Add `TryFrom`/`TryInto` (`std::convert::`) and review the table
     against the current std prelude (the table has 25 entries; `Sized`/`Unpin` and the
     async closure traits cannot head a foreign-self impl that matters, but state the
     choice).
   - A trait brought in by a glob of a dependency (`use std::fmt::*; impl Display for
     Vec<L>`) and `use super::*` where the parent imported the dependency trait
     (`mod t { use super::*; impl Display for Vec<L> {..} }`) both fail. The index has
     no view into dependency modules, so a glob cannot be resolved, but `use super::*`
     can: follow the glob targets' own named imports when looking for the first
     segment (`canonical_trait` consults only the module's own `Named` uses, then the
     table). If a glob of a dependency really cannot be resolved, say so in the Contract
     next to the UnresolvedTrait sentence (source-index.md:~278), which currently
     mentions only the prelude.

B. BLOCKING (fail-hard rule). `canonical_trait` guesses when the path has more than one
   segment: `if segs.len() > 1 { return Ok(segs.join("::")) }` ("the first segment is
   the name of a crate, as written", `impls.rs:~150`). Nothing checks that the head is a
   crate. Probe:
   `pub mod traits { pub use serde::Serialize; } impl traits::Serialize for Vec<L> { fn m(&self){} }`
   `impl serde::Serialize for Option<L> { fn m(&self){} }` `impl bogus::Tr for u8 { fn m(&self){} }`
   record `traits::Serialize::m`, `serde::Serialize::m` and `bogus::Tr::m`: the same
   trait under two keys (the problem canonical keys were meant to remove), and a trait
   from a crate that does not exist accepted. The head must be a known dependency or
   `std`/`core`/`alloc` name (the package manifests are already parsed), a path through
   a local module must follow that module's `use` (here to `serde::Serialize`), and
   anything else must be `UnresolvedTrait`.

C. NON-BLOCKING. Contract text (source-index.md, trait-key paragraph) should state the
   two limits above once fixed, and the edition dependence of the prelude (2021 names).

Final status: three first-pass blockers and the adopted advisories discharged; two new
blocking findings (A, B) in canonical trait keys, each a small change.

VERDICT: BLOCKED

## Recheck 2

Scope: only the new text for findings A and B (`impls.rs`, `item.rs::TraitKey`, manifest
edition and dependency reading, the Contract paragraph, `tests/trait_keys.rs`). Method:
read the code, ran `cargo test -p raf-source --locked --test trait_keys` (9 passed), and
ran a throw-away probe crate in /tmp (removed afterwards; worktree untouched) against
MemoryTree fixtures. Not re-run: the full suites and gates.

### Finding A (valid sources fail to load): DISCHARGED

- `TryFrom` loads in edition 2021 (`std::convert::TryFrom`) and is refused in 2018.
- `use super::*` finds the parent's dependency import (`serde::Serialize`).
- One dependency glob keys the trait `serde::Serialize`; two globs give
  `TraitKey::Unresolved { written: "Serialize", candidates: [rand::Serialize,
  serde::Serialize] }`; a glob plus a prelude name (`Clone` with `use serde::*`) gives
  `Unresolved` with `serde::Clone` and `std::clone::Clone`.
- `core::fmt::Display`, `std::fmt::Display` and the imported spelling all key as
  `std::fmt::Display`.
- A missing inherited edition is a typed manifest error naming `package.edition`
  (probe: "`edition.workspace = true` but no `[workspace.package] edition` ... reached
  by manifest key `package.edition`"). No `edition` key reads as 2015, matching cargo.

### Finding B (written text used as a key): DISCHARGED

- `bogus::Tr` is `UnresolvedTrait`; a declared dependency loads; the original package
  name of a renamed dependency is refused while the key name in code works.
- `target.'cfg(unix)'.dependencies` and hyphen-to-underscore are honoured.
- A local re-export (`traits::Serialize`) and `serde::Serialize` give one key; an
  `extern crate serde as s` alias is followed.
- Dev-dependencies are applied only to test code, as asked: for a lib or bin, an
  `use serde::Serialize; impl Serialize for ..` in non-test code with `serde` only in
  `[dev-dependencies]` is refused; the same impl under `#[cfg(test)] mod` or with
  `#[cfg(test)]` on the impl loads (condition is `decl.reach_test || impl test role`,
  `impls.rs:~107`). A plain non-test module in the same package is refused.
- Remaining uses of written text: only `TraitKey::Unresolved`, which is an explicit,
  typed variant on `Item::trait_key` carrying `written` and every candidate. It is not
  a silent guess: the caller can see the key is unresolved and can enumerate the
  alternatives, and members stay reachable by the written path.

### Non-blocking observations

1. A single dependency glob is keyed `<glob path>::<Name>` as canonical. That is a
   deduction (rustc requires the name to come from somewhere, the prelude and other
   globs are excluded), sound for sources that compile, but the path is the glob's, not
   necessarily the defining path: `use serde::*` gives `serde::Serialize` while
   `use serde::ser::Serialize` gives `serde::ser::Serialize` for the same trait. The
   Contract (source-index.md ~294) states the rule; add a sentence that two spellings of
   a dependency trait are two keys because dependencies are not looked into.
2. A renamed dependency (`foo = { package = "bar" }`) is keyed with the name in code
   (`foo::Q`), so one dependency imported under different names by two packages gets two
   keys. Same remark.
3. `prelude_trait`'s doc comment says "the newest prelude if the edition is not known";
   the code refuses (no match). The behaviour is the right one; correct the comment.
4. `external_prefixes` ignores a glob whose crate head is unknown (`if let Ok(bases)`,
   `impls.rs:~210`). The result is still an error when nothing else supplies the name,
   so no wrong key is produced; an explicit error there would be clearer.

Residual risk accepted: a dependency's re-exports are not visible, so Unresolved and
the two-spelling effect are inherent and now stated.

VERDICT: PASS

## Orchestrator note (post Recheck 2)

The three non-blocking Recheck 2 leftovers were fixed before commit and verified by the orchestrator (not re-reviewed): `prelude_trait` doc comment matches the code; a consulted glob with an unknown crate head is `UnresolvedTrait` at the glob's line (fixture beside a declared-head glob); Contract states single-glob keys use the glob path and renamed dependencies key under the code name (fixture pins both keys). `cargo test -p raf-source --locked` and workspace clippy pass. The PASS verdict above stands.

VERDICT: PASS

## Follow-up fix: inherited keys from ancestor manifests

Scope: `discovery.rs` (editions map removed), `workspace.rs` (`workspace_edition`,
`package_info`), `tests/trait_keys.rs` (new fixtures), `tests/this_repository.rs`, and
the Contract paragraph in source-index.md. Ran
`cargo test -p raf-source --locked --test trait_keys --test this_repository`: 15 and 1
passed. Other staged files ignored as instructed.

Checks:
- Containment: the climb uses `RelPath::parent`, which returns `None` at the root
  (`path.rs:192`), so the loop ends at the tree root and cannot name a path above it;
  every read goes through `Tree::entry` and `Tree::read(path, max_file_bytes)`, so
  `DiskTree` containment, the handle checks and the size cap apply to ancestor manifests
  as to any other file.
- No silent paths: only `entry(..) == None` means "ancestor has no manifest" and is
  skipped; every other `Tree` error and every parse error is returned as a typed
  `LoadError` (manifest errors carry `Origin::ManifestKey` when the parser names one).
  No `.ok()`, `unwrap_or` or `_ => {}` was added. The inherited-edition error still
  names `package.edition`, and its message now says "nearest workspace manifest".
- Rule: the nearest manifest with a `[workspace]` table decides, and a nearer workspace
  with no edition returns `None` and so errors rather than deferring to a farther one.
  A package manifest that has no `[workspace]` table is skipped. The package's own
  manifest is the first one examined, as cargo treats a root package. The cache is keyed
  by the starting directory and holds the final answer, so a hit on an ancestor
  directory returns that directory's own answer, which is also the right one for a
  descendant. No package is added to discovery (asserted by
  `the_workspace_manifest_is_found_above_the_roots_and_adds_no_package`).
- Fixtures: above-roots workspace with no package added and the inherited edition
  deciding the prelude; no workspace table / no edition / nothing above (typed manifest
  error on the member naming `package.edition`); package manifest between is skipped;
  nearer workspace without an edition errors; nearer workspace with an edition wins;
  malformed ancestor errors with the ancestor's file; missing ancestor is fine.
  `this_repository.rs` loads this repository with root `crates` and resolves
  `raf_source::Workspace` to `crates/raf-source/src/workspace.rs`.
- Contract text matches the code (walk, read under cap, skip missing, error on
  unreadable or malformed, roots alone decide the package set).

Non-blocking:
1. No fixture for an ancestor manifest that is present but unreadable (over
   `max_file_bytes`); the code path is the same `?` as the malformed case, so a one-line
   fixture would close it.
2. `the_workspace_manifest_is_found_above_the_roots...` builds a `tree` it then drops
   unused (`drop(tree)`) and positions its Future refusal under a name that does not
   mention it; tidy.
3. Cargo's `package.workspace` key and `workspace.members`/`exclude` membership are not
   consulted, so a package that is not a member of the nearest workspace still inherits
   from it (cargo rejects that). Acceptable for a source index; the Contract says
   "the way cargo finds it" and could say "nearest, without a membership check".

VERDICT: PASS

## Follow-up 2: workspace membership

Scope: `members.rs`, `glob.rs`, the `manifest.rs` path-dependency and `members`/`exclude`
reading, the membership use in `workspace.rs::workspace_edition`, `tests/trait_keys.rs`,
`tests/this_repository.rs`, and the Contract text. Ran
`cargo test -p raf-source --locked --test trait_keys --test this_repository`: 23 and 1
passed. A throw-away probe crate in /tmp (removed afterwards) timed a hostile pattern.

Checked and fine:
- Rule vs cargo: member set = root package + `members` matches, minus `exclude` (a
  pattern matching the directory or an ancestor), plus the transitive closure of
  in-workspace, non-excluded `path` dependencies from every table including
  `[target.*]`, renames (by key) and `{ workspace = true }` through
  `[workspace.dependencies]`. A member that is not in the set inherits nothing and gets a
  typed manifest error naming `package.edition` and the workspace manifest.
- Containment: every read is `Tree::entry` then `Tree::read(.., max_file_bytes)`, listing
  goes through `Tree::list`; a path dependency is followed only when its tidied path
  starts with the workspace directory (`members.rs` `inside`, which also rejects `..`
  segments), so nothing outside the workspace directory is read, and the ancestor climb
  still ends at the tree root.
- Fail-hard: `**`, backslash, unclosed `[`, empty and absolute patterns are typed
  errors; a missing or malformed in-workspace path-dependency manifest is an error naming
  the dependency key; unreadable manifests propagate the `Tree` error.
- Budget: one shared 262144 counter (`Meter`) is spent on every `entry`, `list`, read
  and path-dependency step, and exhaustion is a typed `Budget` error.
- Fixtures: member lists with `?`, ranges, negation, exclude, root package, path
  dependency closure in all tables, outside/excluded path dependencies, missing manifest
  naming the key, unsupported patterns, over-cap ancestor.

### Blocking

1. Exponential glob matching on a hostile manifest. `glob.rs::matches_segment`
   (`Piece::Run` arm, `(0..=text.len()).any(|skip| matches_segment(rest, ..))`) is
   unmemoised backtracking, O(n^k) for k stars. The Meter counts directory visits, not
   matching work, so it does not bound this. Probe: `members = ["crates/" + "*a"x24 + "b"]`
   against a member directory named `a` x40, release build, `Workspace::load` had not
   returned after 40 s (killed). A manifest is untrusted input for this crate (it has
   parse budgets, a fuzz target and a cap on every other input), and a member pattern
   with a few stars and a long directory name is not exotic enough to leave unbounded.
   Fix: match a segment with the standard two-pointer wildcard algorithm
   (remember the last `*` and retry, O(n*m)) or memoise on (piece, offset); add a fixture
   with the pattern above that finishes at once.

### Non-blocking

2. `members.rs` (`let Ok(joined) = base.join(relative) else { continue }`): the `continue`
   is right for `AboveRoot` and `Absolute` (outside the tree, not followed), but it also
   swallows `Backslash`, `Nul` and `DriveLetter` errors. Match on the error kind and
   return the others as typed errors.
3. A literal `members` entry (no wildcard) whose directory has no `Cargo.toml` is
   silently not a member (`expand` -> `has_manifest` false). Cargo reports an error for
   an explicit member without a manifest; glob matches without one are skipped. Treat a
   literal pattern differently, or state the difference in the Contract.
4. `manifest.rs` (`workspace_paths`, `filter_map`) drops a `[workspace.dependencies]`
   entry whose `path` is not a string, which cargo rejects; return a manifest error.
5. `Pattern::parse` trims only leading `./` and trailing `/`; `a//b` yields an empty
   segment that matches only an empty name, so it never matches. Refuse empty segments.

VERDICT: BLOCKED

## Follow-up 2 recheck

Ran `cargo test -p raf-source --locked --test trait_keys`: 28 passed. Read the new
`glob.rs` matcher, the `members.rs` changes and the new fixtures.

1. Matcher (was blocking): DISCHARGED. `matches_segment` is the two-pointer wildcard
   algorithm: every non-`*` piece (`Literal`, `One`, `Set`, including negated sets and
   ranges) consumes exactly one character, so on a mismatch it is enough to resume after
   the last `*` one character further. Each step either advances the text pointer or
   moves the retry point forward, so the work is O(pattern x name) for any mix of `*`,
   `?` and bracket classes; there is no recursion. The trailing check
   (`pieces[p..]` all `*`) is right and `p` never passes the end. The fixture
   `matching_a_pattern_takes_time_in_proportion_to_pattern_and_name` covers the
   24-star hostile pattern.
2. Path-dependency join errors: DISCHARGED. Only `AboveRoot` and `Absolute` continue;
   every other `PathError` returns a manifest error naming the dependency key
   (`members.rs:~260`), fixture `a_path_dependency_with_an_unusable_path_is_an_error...`
   beside the outside-the-tree case.
3. Literal member without a directory or manifest: DISCHARGED. `is_literal` selects
   strict mode: a missing directory or missing `Cargo.toml` is a manifest error naming
   `workspace.members`; a glob skips non-package matches. Fixture
   `a_literal_member_without_a_manifest_is_an_error_and_a_glob_skips_such_a_directory`.
   The changed fixture (`crates/memberx*`) keeps its intent, since it needs a pattern
   that matches nothing without being an error.
4. Non-string `[workspace.dependencies]` path: DISCHARGED (fixture
   `a_workspace_dependency_whose_path_is_not_a_string_is_an_error_naming_the_key`).
5. Empty pattern segment: DISCHARGED (typed error; fixture
   `an_empty_pattern_segment_is_a_pattern_error_beside_a_plain_pattern`).

Non-blocking:
- `Pattern::parse` no longer accepts a trailing `/` (`crates/*/` is now a pattern error).
  Cargo accepts it (its glob crate matches directories for a trailing separator), so a
  valid workspace could be refused with a typed, fixable error. Prefer accepting one
  trailing `/` and refusing only `//` and a leading `/`; state it in the Contract.
- A literal member that is also excluded still errors if its manifest is missing, since
  `exclude` is applied after expansion; cargo ignores an excluded member. Rare; apply
  the exclude test before the strict check.

VERDICT: PASS

## Orchestrator note (follow-up 2 leftovers)

The recheck's two non-blocking false refusals were fixed before commit and verified by the orchestrator: one trailing `/` accepted in members/exclude patterns (`//` and a leading `/` still refused); an excluded literal member is ignored before the manifest existence check. Fixtures present; `cargo test -p raf-source --locked` and workspace clippy pass.

VERDICT: PASS

## Follow-up 3: target crate roots

Scope: `index.rs::add_roots` and the `items_by_source` exclusion, the crate-path branch in
`resolve.rs`, the adjusted count in `symbol_index.rs`, and `tests/target_roots.rs`. Ran
`cargo test -p raf-source --locked --test target_roots --test symbol_index`: 3 and 18
passed. A throw-away probe in /tmp (removed) covered the collision below.

Fine:
- Correctness: each target gets one `Mod` item in the type namespace at its crate path,
  file = root file, `name_line` 1, span the whole file, `pub`, docs from the file's `//!`,
  `module_of` set so the path continues into the root's module. The fixtures cover every
  target kind (lib, main, `src/bin`, test, bench, example, build script) with file, span,
  shipping flag and documentation, the value namespace staying empty, the intermediate
  `test_crate`/`bench_crate`/`example_crate`/`bin_crate` segments staying `NotFound`,
  the first-party `mod test_crate;` case staying `Ambiguous` for the test target's
  items, and this repository's own `raf_okf` and `raf_okf::test_crate::refusals`.
  Resolving a path through a root still works (`demo::test_crate::it::case`).
- Test roles are coherent: test and bench roots are test code (`CfgTest`, so
  `is_shipping` is false), an example root is `None` and non-shipping through its target,
  lib, bin and build script are shipping. This is the same rule `is_shipping` already
  applies to items.
- The `symbol_index` count change is legitimate: roots are whole files, not source
  items, so `covered + targets == items().count()` states exactly what changed.
- Excluding roots from `items_by_source` is justified, and more than cosmetic: a root's
  `(file, 0)` source key collides with any item that starts at byte 0 (probe: `src/main.rs`
  = `fn main(){}` gives the root and `main` both at offset 0), which would merge two
  different things into one identity group. With the exclusion, `same_source(main)`
  stays 1.

Blocking:
1. A first-party item at a path equal to a crate path changes resolution. Probe:
   `src/lib.rs` = `pub mod bin_crate { pub mod demo { pub fn q(){} } }` beside
   `src/main.rs`: `resolve("demo::bin_crate::demo")` returns one `Resolution` with two
   sites (the first-party module and the binary's root), because the crate-root hit is
   inserted into `found` (`resolve.rs:~390`) and `resolve_in` groups by (path, kind),
   both `Mod`. Before this change the same path resolved to the module alone, and the
   project's rule for colliding names (the open finding on `test_crate`) is
   `Ambiguous`, not a merged cfg-twin. Fix: when the crate-root hit is added and `found`
   already holds something, return `Ambiguous` with both (or key the group on the item's
   target as well for roots); add a fixture beside the existing collision test.
2. `add_roots` (`index.rs:~527-532`) has two `let Some(..) else { continue }` for a
   target whose root file is not in `files` or whose crate path is not a root module.
   Both hold by construction, but section A bans turning the absent case into a skip;
   return a typed `Internal` load error as `workspace.rs` now does.

Non-blocking:
3. `items_in(root file)` and `items()` now also yield the root `Mod` (probe above), so a
   consumer that walks items per file sees a whole-file item at offset 0. `item_at`
   still returns the innermost item. State this in the Contract's `items_in`/identity
   sentence (a root's `source_key` is the file's offset 0 and is not unique).

VERDICT: BLOCKED

## Follow-up 3 recheck

Ran `cargo test -p raf-source --locked --test target_roots`: 4 passed.

1. Merged sites on a crate-path collision: DISCHARGED. `resolve.rs:390-411` returns
   `Ambiguous` carrying the first-party items plus the root when `found` is non-empty
   at the point the crate-root hit is added, and inserts the root only into an empty
   result, so a path with no first-party item still resolves to the single root site.
   The fixture (lib `pub mod bin_crate { pub mod demo {} }` beside `src/main.rs`) and its
   neighbour without the module pass.
2. Silent skips in `add_roots`: DISCHARGED. It returns `Result<(), LoadError>`; a missing
   root file or missing crate-root module is `LoadErrorKind::Internal` naming the target's
   root file and crate path (`index.rs:533-545`), covered by a unit test
   (`index.rs:~741`).

The earlier non-blocking note on `items_in`/`source_key` of a root stands (Contract
wording only).

VERDICT: PASS
