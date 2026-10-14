# Architecture review: raf-okf and raf-source (PR 13, issue 2)

## Scope

- Branch `paudley/2-shared-crates-raf-okf-raf-source`, HEAD `6c744af`.
- Diff `git diff origin/main...HEAD -- crates/raf-okf crates/raf-source tests/docs_contract.rs`.
- Read against AGENTS.md, `issue.md`, `plan.md` (user decisions 1-5), `historical-fit.md`, and the
  capability docs `okf-profile.md`, `source-index.md`, `census-rule-api.md`, `census-ledger.md`,
  `workplan-document-model.md` and `workplan-complete-gate.md`.
- I did not edit or commit any tracked file.
- Evidence comes from reading the code and from a scratch crate outside the repository
  (`/tmp/rafarch`). That crate depends on both crates by path and runs probes against the public
  API. Every behavioural claim below marked "probe" was observed there at HEAD 6c744af.

Severity scale:

- **CRITICAL**: unsafe to merge.
- **HIGH**: produces a wrong index or contract violation, or blocks a named consumer.
- **MEDIUM**: a contract or evolvability cost that becomes a breaking change once the consumers bind.
- **LOW**: hygiene.

Each finding is marked **Required** (fix before the two consumers bind) or **Advisory**.

## Summary

| # | Sev | Req/Adv | Finding |
|---|---|---|---|
| A1 | HIGH | Required (owner decision) | An inherent impl on a macro-generated type aborts the whole load. katamari and any crate that uses `bitflags!` cannot be indexed. |
| A2 | HIGH | Required (owner decision) | Two packages with the same lib name abort the whole load, so purrdf cannot be indexed. |
| A3 | HIGH | Required | Items of two different targets are merged as "cfg twins" (build script, explicit-path bin). This breaks the Contract and depends on file names. |
| A4 | HIGH | Required | An import cycle is settled by picking the first one in declaration order. The result depends on that order, which breaks the Contract, and it is not `Cyclic`. |
| A5 | MEDIUM | Required | The per-file extraction IR (`FileFacts` and 11 related types) is public with fully pub fields. `PathFact` is reachable but cannot be named. |
| A6 | MEDIUM | Required | raf-source data structs (`Target`, `Package`, `SourceFile`, `Reach`, `Entry`, `SpanFact`) are not `#[non_exhaustive]`. `Target` already lacks fields that workplan needs. |
| A7 | MEDIUM | Advisory | `settle` is quadratic in the number of imports that never resolve (every `use std::..`/`use serde::..`). |
| A8 | MEDIUM | Advisory | `#[non_exhaustive]` on closed-profile enums (`Outcome`, `LinkTarget`) forces fallback arms on consumers. |
| A9 | MEDIUM | Advisory | `Rule: Sync` is correct on the merits. The census would parse every file twice, and `LoadOptions: Copy` blocks a hook. |
| A10 | MEDIUM | Advisory | `Workspace::load` stops at the first error, while `Profile::validate` reports every violation. |
| A11 | MEDIUM | Advisory | The docs_contract binding check does not check a binding's role (tests binding versus test fn). |
| A12 | LOW | Advisory | `resolve` accepts `crate`/`self` in the middle of a path and swallows `SuperAboveRoot` into `NotFound`. |
| A13 | LOW | Advisory | A glob over a local path that does not resolve silently brings in nothing. |
| A14 | LOW | Advisory | `Tree` has no `Sync` bound, so a parallel load later is a breaking change. |
| A15 | LOW | Advisory | The two crates use two span conventions: half-open lines against an inclusive `end_line`. |
| A16 | LOW | Advisory | Small API redundancies and footguns in `Workspace` and `SourceFile`. |
| A17 | n/a | Advisory | `<pkg>::bin_crate::<bin>` naming is sound on the merits. It needs owner sign-off, and `build_script` needs the same reservation. |
| A18 | LOW | Advisory | `pub use syn` ties raf-source's semver to the syn major version. |

Module boundaries are good overall.

- raf-okf hides its modules and exposes accessor-based types.
- raf-source separates tree, parse, facts, modtree, index, settle, resolve and impls cleanly.
- Error types are typed throughout.
- Duplication between the crates is small and justified: raf-okf `Link::target_in` path folding
  repeats about 30 lines of `RelPath::join`, so that raf-okf stays dependency-free.
- docs_contract now delegates every per-document rule to raf-okf. What remains in it
  (cross-document checks, cycles, inventory, tracked-file links) is what `raf-workplan` `check`
  must own later. That code should move there and not be re-derived (advisory).

The findings that matter are A1-A4: the index refuses or misreports real code. A5-A6 are cheap now
and breaking later.

---

## A1. An inherent impl on a macro-generated type aborts the load (HIGH, Required, owner decision)

**Where.**

- `crates/raf-source/src/impls.rs:430-436` returns `InherentImplSelf` whenever the self type does
  not resolve to a local struct, enum or union.
- `docs/capabilities/source-index.md:344-346` makes that a load failure.
- `source-index.md:510` says macros are not expanded.

**Consequence (probe).**

- `MemoryTree` with `bitflags::bitflags! { pub struct F: u8 {..} }` followed by
  `impl F { .. }` gives `load error: src/lib.rs:2: the self type F of an inherent impl is not a local struct, enum or union`.
- `Workspace::load(DiskTree::new("/home/paudley/Active/katamari"), &["crates"])` fails after 1.4 s.
  The error is `crates/katamari-keys/src/material.rs:161: the self type DataKey ...`, where `DataKey`
  comes from a local `key_type!` macro.

Two contract rules combine here: "macros are not expanded" and "an inherent impl must name a local
type or loading fails". Together they mean any workspace that defines a type by macro and gives it
an inherent impl cannot be indexed at all. That pattern is ordinary Rust: bitflags, newtype macros
and generated IDs.

The census precedent ran on katamari; this port cannot load it. The rule is fail-hard (D10) at the
wrong granularity: one item the index cannot model refuses every other item in the workspace. That
is not "never a wrong index"; it is no index.

**Fix.** Keep the refusal typed, but make it per item, not per load.

- Record the members under the written self path, `<module>::<SelfType>::<method>` as
  `InherentMethod`.
- Add a typed marker on the item, for example `self_key: Option<SelfKey>` with
  `SelfKey::Unresolved { written }`, mirroring `TraitKey::Unresolved`.
- Keep the members out of `resolve`, or make them resolvable only as unresolved, as the owner
  decides.
- Expose the unresolved impls through `Workspace::unresolved_impls()`, so a consumer such as
  census-ledger can choose to fail.

This changes D10, a plan decision rather than a user decision. It needs the owner's call. The
Contract and the `InherentImplSelf` fixture change with it.

## A2. Two packages with one lib name abort the load (HIGH, Required, owner decision)

**Where.**

- `crates/raf-source/src/target.rs:448-472` (`check_crate_paths`).
- `source-index.md:262-263`: "two targets (in one package or two) with one crate path ... typed error".

**Consequence (probe).** `Workspace::load(DiskTree::new("/home/paudley/Active/purrdf"), &["crates"])`
fails in 44 ms with
`crates/rdf-capi/Cargo.toml: the crate path purrdf is already used by the target purrdf of crates/purrdf/Cargo.toml`.

`purrdf` and `purrdf-capi` are both listed in purrdf's `[workspace] members`, and both set
`[lib] name = "purrdf"`. Cargo builds that workspace. raf-source keys a library crate by its lib
name alone, so a valid Cargo workspace cannot be represented. The census, whose precedent lives in
purrdf, cannot run there. Narrowing `roots` to dodge it means listing every crate directory except
one, which census-ledger's `roots` setting was not designed for.

**Fix.** Separate the crate's identity from its lib name.

- Key `crate_roots` and `libs` by `(package, crate name)`.
- Let several packages share a crate name.
- Make `resolve("purrdf::X")` return `Ambiguous` only when both libraries define `X`, and add
  `resolve_in_package(pkg, path)` for callers that know which one they mean.

The alternative is to keep the refusal and document it as a known unsupported Cargo shape. Either
way it is an owner decision, because census-ledger `home` paths are written as crate paths.

## A3. Items of different targets merge as one symbol (HIGH, Required)

**Where.**

- `crates/raf-source/src/resolve.rs:435-445` groups sites by `(item.path, item.kind)` only.
- `resolve.rs:467-489` (`shadowed_namespace`) inspects only `sites[0]`'s target, and skips every
  prefix that is itself a crate root.

**Consequence (probe).**

| Fixture | `resolve(...)` | Contract says |
|---|---|---|
| lib `pub mod build_script { pub fn main() {} }`, plus `build.rs` `fn main() {}` | `demo::build_script::main` resolves to `Ok`, with 2 sites in targets `demo` and `demo::build_script` | `Ambiguous` (two canonical targets, `source-index.md:458-460`) |
| lib `pub mod bin_crate { pub mod x { pub fn main(){} } }`, plus `[[bin]] name="x" path="tools/x.rs"` | `demo::bin_crate::x::main` resolves to `Ok`, with 2 sites in targets `demo` and `demo::bin_crate::x` | `Ambiguous` (`source-index.md:461-464`) |
| the same, with the bin at the default `src/bin/x.rs` | `Ambiguous` | `Ambiguous` |

Whether the conflict is caught depends on whether the bin's file name sorts before `src/lib.rs`,
because `sites[0]` decides which target is inspected. For `build_script` it is never caught. That
path has no reserved `_crate` segment, and its only prefix (`demo`) is a crate root, which
`shadowed_namespace` skips.

The cfg-twin rule (D9) is meant for one module defining one name twice. Here two crates are
silently presented as one symbol. That is the "wrong index" the doctrine forbids. A census ledger
`home` at such a path would count the lib function and the build-script `main` as one
implementation.

**Fix.**

- Add the target to the twin key: `(item.target, item.path, item.kind)`. Twins are always in one
  module and so in one target, so the real case is unchanged.
- Any group that spans targets becomes `Ambiguous`.
- Check the shadowing for every site's target, not only `sites[0]`.
- Better: refuse the overlap once, at load. Add a typed `TargetProblem` when a lib module path
  equals or prefixes another target's crate path. That turns a query-time heuristic into an
  invariant.
- Add the three fixtures above.

## A4. Import-cycle settling depends on declaration order (HIGH, Required)

**Where.**

- `crates/raf-source/src/settle.rs:466-478`. When no stuck import is a leaf, it settles
  `vec![first]`, the lowest position, which means declaration order.
- `settle.rs:10-12` and `source-index.md:437-438` claim "the result does not depend on the order
  of the declarations".

**Consequence (probe).**

```rust
mod a { pub use crate::b::X; pub use crate::c::*; }
mod b { pub use crate::a::X; pub use crate::d::*; }
mod c { pub struct X; }
mod d { pub struct X; }
```

- With `a` written first, `demo::a::X` and `demo::b::X` both resolve to `demo::c::X`.
- With `b` written first, both resolve to `demo::d::X`.

The explicit imports form a cycle. rustc rejects this code; the Contract would make it `Cyclic`.
Instead the index returns a definite, order-dependent answer. The 1,440-permutation test
(`tests/use_resolution.rs:392-437`) never reaches this branch, because none of its imports is
mutually blocked. So the central correctness claim is unproven exactly where the code makes an
arbitrary choice. The choice is also the first-wins selection that plan section A bans.

**Fix.**

- When there are no leaves, compute the strongly connected components of the `blockers` graph.
- Settle every import of a cycle as failed at once, which is order-independent.
- Record those imports as cyclic, so that a lookup of the name they bind reports
  `ResolveError::Cyclic` and not the glob behind it.
- Add the fixture above in both orders, asserting `Cyclic`.

## A5. The extraction IR is public API (MEDIUM, Required)

**Where.**

- `crates/raf-source/src/lib.rs:130-133` re-exports `AttrFact, ConditionalPath, FieldShape,
  FileFacts, ImplFact, ItemFact, ItemKind, ModBody, ModFact, PathAttrs, SpanFact, TestRole, UseFact,
  UseTree, Vis`.
- `workspace.rs:40` has `pub facts: FileFacts`.
- `facts.rs:95` has `pub struct PathFact`, which is not re-exported. It is still reachable through
  `ItemFact::alias` (`facts.rs:161`) and `ImplFact::trait_segments` (`:265`), so it is reachable but
  cannot be named.

**Consequence.**

- The per-file IR that feeds the index is a second, parallel public model of an item, beside
  `Item`. It has about 80 pub fields and no `#[non_exhaustive]` on any struct.
- Any change to extraction is a breaking change for every consumer. Census is likely to need
  signatures and generics; settle may need more shapes.
- No capability requires this surface. Its only users are the integration tests
  (`tests/module_tree.rs:593-943`, `tests/symbol_index.rs:593`).
- A consumer cannot write a function that takes a `PathFact`.

**Fix (preferred).**

- Make `facts` `pub(crate)` and drop `SourceFile::facts` from the public API.
- Move the fact-level assertions into in-crate unit tests, or expose the few facts a consumer needs
  through `Item` or narrow accessors.

**Fix (minimum).**

- Mark every fact struct `#[non_exhaustive]`.
- Export `PathFact`.
- State in source-index.md that `FileFacts` is a supported contract.

## A6. raf-source data types are frozen by construction (MEDIUM, Required)

**Where.**

- `target.rs:43` (`Package`) and `target.rs:54` (`Target`).
- `workspace.rs:27` (`SourceFile`).
- `modtree.rs:52` (`Reach`).
- `tree/mod.rs:31` (`Entry`).
- `facts.rs:20` (`SpanFact`).

All have only pub fields and no `#[non_exhaustive]`. `Item`, `Resolution`, `TestFn` and
`LoadOptions` do have it, so the style within the crate is inconsistent. raf-okf avoids the problem
entirely with private fields and accessors.

**Consequence.** workplan's completion gate (`workplan-complete-gate.md:62-80`) builds
`cargo test --package --lib/--bin/--test` argv from the index. It needs per-target facts that
`Target` does not carry:

- `harness = false`: `#[test]` functions in such a target are not run by libtest, so `test_fns()`
  would hand out unrunnable bindings;
- `required-features`;
- `test`/`doctest = false`.

Adding any of these after workplan binds is a breaking change for every downstream struct literal
or exhaustive pattern.

**Fix.**

- Add `#[non_exhaustive]` to these six types now. They are only constructed in-crate, so it costs
  nothing.
- Decide whether `Target` gains `harness`, `required_features` and `doctest` in this PR or in the
  workplan change. Record that decision.

## A7. `settle` is quadratic in unresolved imports (MEDIUM, Advisory)

**Where.**

- `settle.rs:458-478`: each "stuck" round recomputes `stuck` and calls `blockers()` for every stuck
  import.
- `resolve.rs:274-288`: `unresolved_uses` scans every import linearly for each blocker source.

Every import of a dependency (`use std::..`, `use serde::..`) never resolves, so all of them pass
through this path.

**Measurement (probe, release build).** Synthetic workspace with N modules, each holding 20
dependency imports, one re-export and one trait impl:

| N | Approx. imports | Load time |
|---|---|---|
| 100 | 2.2k | 28 ms |
| 200 | 4.4k | 89 ms |
| 400 | 8.8k | 390 ms |
| 800 | 17.6k | 1.08 s |

The same workspace without the dependency imports loads in 88 ms at N=800 and grows linearly.

purrdf's `crates/` holds about 15.8k `use` lines (more once groups are flattened), so it should
load in seconds today. A 10x workspace would take minutes. RAF itself takes 0.49 s for 85 files.

**Fix.**

- Index the unsettled named imports by `(module, name)` once (a `BTreeMap<(usize, String),
  Vec<usize>>`, which `unsettled` already nearly is).
- Track reverse dependencies, so that leaves are found incrementally rather than by rescanning
  every round.
- Add a scaling test, for example asserting that the load-time ratio at 2N against N stays under 3,
  or one that counts `walk` calls.

## A8. `#[non_exhaustive]` on closed-profile enums forces fallbacks (MEDIUM, Advisory)

**Where.**

- `crates/raf-okf/src/profile/typed.rs:137` (`Outcome`).
- `body/link.rs:69` (`LinkTarget`).
- `State`, `DocType`, `ContractEffect`, `CapabilitySection` and `RafDocument` are exhaustive.

**Consequence.** The profile is closed and normative: `docs/architecture/okf-profile.md` defines
it, and raf-okf "adds no rule of its own". Yet the first consumer already needs runtime fallback
arms:

- `tests/docs_contract.rs:203` has `Some(other) => panic!("unmodelled outcome")`;
- `tests/docs_contract.rs:440` has `Ok(other) => ... "unmodelled link target"`.

A new outcome is a profile change that should break workplan at compile time. As written it would
reach a runtime panic or a generic message instead.

**Fix.**

- Drop `#[non_exhaustive]` from `Outcome` and `LinkTarget`, matching `State` and `DocType`.
- Keep it on error and violation enums, where growth is expected.
- Remove the two fallback arms.

## A9. `Rule: Sync` in census-rule-api.md (MEDIUM, Advisory)

**Merits.** The bound is correct and forced. `parse_source` (`parse.rs:37-41`) requires
`visit: impl FnOnce(&syn::File) -> R + Send`. A gate that runs a registered `Box<dyn Rule>` inside
that visitor captures `&dyn Rule`, and `&T: Send` requires `T: Sync`. The census `Rule::check`
takes `&syn::File`, and syn trees only exist on the parse thread. So no design that reuses
raf-source's bounded parse can avoid `Sync`.

It is also consistent with `source-index.md:293`. That line says facts leave the parse thread as
owned data, while rules run on it, so the two statements are complementary. historical-fit's R5
worry does not hold.

**What the edit leaves out.**

1. The same reasoning binds `FileCtx: Sync` and `RuleHit: Send`. The doc constrains only `Rule`.
2. `Workspace` keeps text and `FileFacts`, not syn trees. A census that runs rules must therefore
   call `parse_source` again for every file, after `Workspace::load` already parsed it. That
   doubles parse cost, which dominates load time.
3. A hook to avoid the second parse does not fit `LoadOptions`. It is `Copy`
   (`options.rs:10-12`), so adding a visitor field later is breaking.

**Fix.**

- Keep `Rule: Sync`. Extend the sentence to name `parse_source` as the reason, and add the
  `FileCtx`/`RuleHit` bounds.
- Decide now whether `Workspace::load_with` gets a per-file visitor run during the load. If it
  does, drop `Copy` from `LoadOptions` in this PR, while that is still free.
- Record this as an owner decision. It edits a capability that is not delivered.

## A10. Mixed error-aggregation policy (MEDIUM, Advisory)

**Where.**

- `workspace.rs:74-95` and `source-index.md:290`: "The walk stops at the first error".
- `Profile::validate` returns every violation (`profile/mod.rs`).

**Consequence.** A census or workplan run over a large workspace reports one load error per run.
A1 and A2 show how easily real repositories produce several. `Result<Self, LoadError>` is the
signature consumers will bind, and changing it to collect errors later is breaking.

**Fix.** Decide before binding.

- Either return `Result<Self, LoadErrors>` (non-empty, sorted) and keep walking unaffected targets,
- or keep fail-fast and state in the Contract that it is deliberate.

## A11. The binding check ignores each binding's role (MEDIUM, Advisory)

**Where.** `tests/docs_contract.rs:695-716` resolves every `code`, `tests` and `examples` binding
with `workspace.resolve`, regardless of which list it is in.

**Consequence.** `workplan-document-model.md:98-100` says `bindings.tests` entries are test
function paths. A `tests` binding that names a library function, or a `code` binding that names a
`#[cfg(test)]` helper, passes the gate. The workplan completion gate would later try to
`cargo test` it. A probe confirmed that every current binding happens to be well placed, so this
is not a present defect.

**Fix.**

- For `tests`, require `item.test == TestRole::TestFn`.
- For `code`, require `ws.is_shipping(item)`.
- Add one negative fixture each.

## A12. `resolve` input validation (LOW, Advisory)

**Where.** `resolve.rs:497-499` accepts any alphanumeric segment, and `resolve.rs:387` has
`Err(Fail::SuperAboveRoot) => {}`.

**Consequence (probe).**

- `demo::crate::a::x` and `demo::self::a::x` resolve to `Ok` with path `demo::a::x`, so a binding
  written that way passes docs_contract.
- `demo::super::x` is reported as `NotFound`, not `Invalid`. The swallowed arm is the
  `_ => {}`-over-a-fallible-case pattern that plan section A bans.

**Fix.** Return `ResolveError::Invalid` for `crate`, `self`, `super` and `Self` anywhere in a
canonical path, and remove the swallowed arm.

## A13. A local glob that does not resolve brings in nothing (LOW, Advisory)

**Where.** `settle.rs:377-380` has `let Ok(targets) = self.glob_targets(..) else { continue; }`.

**Consequence.** The silence is justified for dependency globs. `use crate::missing::*` is also
silently empty, though rustc rejects it. That cannot produce a wrong positive answer, only a
`NotFound`.

**Fix.** Return a typed error when the glob's first segment is `crate`, `self`, `super` or a local
module, and stay silent only for dependency heads, as `trait_key` already distinguishes.

## A14. `Tree` has no `Sync` bound (LOW, Advisory)

**Where.** `tree/mod.rs:39-43` and `Workspace::load(tree: &dyn Tree, ..)`.

**Consequence.** Parsing already runs on its own thread for each file, but the load is serial, and
`&dyn Tree` cannot be shared across threads. Both shipped backends are `Send + Sync` (probe).
Adding `+ Sync` later breaks every caller's signature, and the in-test recording tree's `RefCell`
(`tests/module_tree.rs:493`).

**Fix.** Either take `&(dyn Tree + Sync)` now, or record that a parallel load would be a breaking
change.

## A15. Two span conventions (LOW, Advisory)

**Where.**

- `raf_okf::Span { lines: Range<usize>, bytes: Range<usize> }` is half-open (`span.rs:8-21`).
- `raf_source::SpanFact { start_line, end_line, .. }` has an inclusive `end_line`
  (`facts.rs:20-29`).

**Consequence.** The plan declined a shared Span (enhancement 19) because "matching shapes give
the same consistency", but the shapes do not match. workplan joins document spans with code spans
(evidence, assessment) and will hit an off-by-one.

**Fix.** Make `SpanFact` half-open with `Range<usize>` fields, or document the difference in both
crates.

## A16. Small API redundancies and footguns (LOW, Advisory)

- `SourceFile::test` (`workspace.rs:36`) and `SourceFile::reached_only_through_tests()` (`:50`)
  compute the same thing. Keep one.
- `test_fns()` (`workspace.rs:320`) returns `Result<_, LoadError>` with an `Internal` kind for an
  invariant that `load` could establish. Check the invariant at load and return a `Vec`.
- `items_under(prefix)` (`workspace.rs:215`) matches by string prefix, so `items_under("demo::a")`
  also yields `demo::ab::..`. Offer a segment-aware form, or document it.
- `SourceFile::package` is "the package of the first target that reaches the file", which depends
  on walk order for a file that two packages share.

## A17. `<pkg>::bin_crate::<bin>` naming (Advisory, needs owner sign-off)

**Merits.** The scheme is sound.

- It removes the precedent's real collision: a lib and a bin with one name had one path
  (`tests/target_roots.rs:127-170`).
- It is uniform with `test_crate`, `bench_crate` and `example_crate`.
- It keeps every crate path unique without a special case.

**Costs.**

- Every binding and ledger `home` that names a binary item changes spelling from the katamari form
  (`<bin>::..`) to `<pkg>::bin_crate::<bin>::..`. Migrated ledgers need a stated rewrite rule
  (census-migration).
- The reserved segments are guarded only at query time and only partly (A3).
- `build_script` has no `_crate` reservation at all. `mod build_script` in a library is a
  plausible name, and it collides silently (A3).

**Recommendation.**

- Accept the scheme, and get the owner's explicit sign-off in the PR body, since it is not among
  user decisions 1-5.
- Either rename the build-script path to `<pkg>::build_script_crate`, or make the A3 load-time
  overlap refusal cover every reserved segment.

## A18. `pub use syn` (LOW, Advisory)

**Where.** `lib.rs:140`.

**Consequence.** The re-export, and the census `Rule::check(&syn::File)`, make the syn major
version part of the semver of raf-source and raf-census. A syn 4 bump is breaking for both. This is
inherent to the census design, not a defect.

**Fix.** Say it in source-index.md Constraints, so that the dependency policy owns the bump.

---

## Resolver checkability (overall judgement)

The resolver is split into stages: emit (`index.rs`), scope settle (`settle.rs`), walk/resolve
(`resolve.rs`) and impl attach (`impls.rs`). Every map is a `BTreeMap` and routes are ranked
(`Origin::rank`, `Hit::rank`), which makes the output deterministic given an order. Three
structural features make correctness hard to check:

1. **Two settling regimes.** A work queue settles imports against interim glob tables, and
   `rebuild_globs` (`settle.rs:510-532`) then recomputes the globs from scratch. Named imports
   resolved during the queue keep their interim answers. The invariant that the interim and final
   glob tables agree on every name a named import consumed is stated nowhere and asserted nowhere.
   **Advisory:** add a debug assertion that re-walks every resolved named import against the
   final tables.
2. **An arbitrary tie-break for cycles** (A4).
3. **Twin grouping by path string rather than by target** (A3).

A4 and A3 are the concrete defects behind points 2 and 3.

Point 1 needs an invariant check or a differential test, not more fixtures. A small property test
over generated module graphs, comparing every declaration order, would cover all three. A minimal
version permutes the order of 4-6 random `use`/glob/definition blocks, as the existing 1,440-case
test does by hand.
