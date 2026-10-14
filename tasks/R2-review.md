Current status: VERDICT: PASS after recheck — see Recheck

# R2 review: modules are placed under their own path

## Reviewed scope

Uncommitted changes on base c10abd9 (raf-source is byte-identical to 6c744af: `git diff --stat 6c744af HEAD -- crates/raf-source` is empty):
crates/raf-source/src/{index,modtree,workspace}.rs, crates/raf-source/tests/{module_tree,this_repository}.rs,
crates/raf-source/examples/registry_scan.rs (new), docs/capabilities/source-index.md, tests/docs_contract.rs.
Contract: remediation-plan.md "### R2", "Registry harness", "Mutation rule", "Per-item gates"; background CA F2.
The worktree was never modified by this review: `git diff | sha256sum` and the example's sha256 were recorded before and
after and are identical (1ae18f25..., 939b0105...); `git diff --stat` identical.

## Evidence

### Gates (all exit 0)
- `cargo test -p raf-source --locked`: every suite ok (module_tree 39 passed, this_repository 2, lib unit tests incl. the new
  `a_module_file_visited_before_the_declaration_that_names_it_is_an_internal_error`).
- `cargo test -p raf-source --locked --release`: every suite ok.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean (covers the example; no missing_docs or lint allow added).
- `cargo test --locked --test docs_contract --test repo_gates`: 138 + 55 passed.
- `make reuse`: compliant, 211/211 (the new example carries SPDX headers).

### Falsification on the old source (Mutation rule: every new test must fail on 6c744af)
Instead of checkout/apply in the shared worktree, I extracted `git archive HEAD` to /tmp/r2old (old src) and copied in the new
test files, docs and example; separate CARGO_TARGET_DIR. Equivalent outcome, no risk to the worktree.
- module_tree on old src: 6 FAILED, 33 passed. Failing: `a_child_file_that_sorts_before_its_mod_rs_parent...`,
  `a_flat_module_file_and_its_child_below_a_parent_that_sorts_first...`, `the_children_of_a_path_attribute_file...`
  (a and ga misplaced; z is the in-test control), `a_shared_test_module_reached_from_a_target_that_sorts_after_it...`,
  `super_super_from_a_child...` (panics with `src/a/b.rs:1: super::super::top climbs above the crate root`),
  `an_item_is_found_only_under_its_own_module_path`.
- this_repository on old src: `an_item_of_a_module_file_that_sorts_before_its_mod_rs...` FAILED
  (`raf_okf::body::blocks::FenceBlock` resolves to `raf_okf::body::FenceBlock`).
- docs_contract on old src: `a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found` FAILED.
  (`repository_bindings_resolve_in_the_source_index` also failed there, only because /tmp/r2old is not a git repository.)
- The index.rs unit test exercises the new `Visit` API, which does not exist on the old source; it is a direct test of the new
  Internal guard, not a regression fixture.
All fixtures listed under R2 "Check (B8)" and "Entry-point checks" are present and each fails on the old source.

### Root cause and fix
- The account is right. Old `Builder::ensure` read `idx = modules.len()` before recursively creating a missing parent, so the
  child was registered at the parent's slot; `Index::build` iterated files in path order, so any module file sorting before
  its declarer hit that path. I confirmed with a 5-module `use super::*` layout that only the first module file whose parent is
  not yet created collapses (subsequent siblings find the parent).
- The fix removes the class rather than the off-by-one: `Walker::load` pushes a `Visit` before `body()` follows the file's
  `mod` declarations, so a declarer always precedes its children; `Builder::declare(path, parent)` is only called from a `mod`
  declaration (file or inline) and takes the parent explicitly; crate roots are pre-created. `source_file` keeps `reached` in
  push order, so `Visit::reach` indexes stay valid. Item order is still canonicalised by `finish()` (`sort_key`), so outputs
  remain deterministic; only internal `ItemId` numbering changes.

### Adversarial shapes (scratch binary against new and old src, /tmp/r2adv)
All loaded on the new source with every item under the module path of its reach; resolutions as rustc would place them:
- deep `#[path]` chain (`x/y/deep.rs` -> `../../q/r.rs` -> `s` -> `s/a`), incl. `super^3` and `super^4` re-exports: correct
  (also correct on old: the declarers sort first there).
- one file reached by two module paths and two targets (`mod a;`, `#[path="a.rs"] mod a2;`, `tests/t.rs` `#[path]`):
  `a::child` -> `src/a/child.rs`, `a2::child` and `shared::child` -> `src/child.rs`; all items correct.
- inline module containing `#[path]` and file modules (`mod inl { #[path="aa.rs"] mod x; mod y; mod deeper { mod w; } }`):
  new correct; old misplaced `demo::inl::x::k::in_k` (NotFound) and `demo::inl::deeper::in_w`.
- cfg'd duplicate `mod a;` (unix/win `#[path]`s, each with child `c`): one module `demo::a`, both children under `demo::a::c`.
- `cfg_attr` path alternatives on a file module and on an inline module with a file child: correct; old misplaced the inline case.
- cfg twins pointing at the same file: deduplicated by `seen`, correct.
- a module file reached by a test target through `#[path]` and by the lib through `mod`: two reaches, both correct.
- bin target with `src/cli/mod.rs` + `src/cli/args.rs`: new `...::cli::args::parse`; old `...::cli::parse`.
- `#[cfg(test)] mod a_tests;` sorting before `mod.rs`: new correct, old misplaced.
- raw identifiers `mod r#type; mod r#fn;`: correct.
- `#[path] mod m;` inside a fn body / `const _` block: neither walker nor builder follows it, consistently (no Internal).

### Is the `Internal` error reachable?
No path found. Every module path the walker produces is `scope.module_path::name` for a `ModFact` in `body.mods` (recursing into
inline bodies), and the builder declares exactly `modules[module].path::decl.name` for the same facts, from the declarer's
visit, which is pushed before the child's. Dedup by `(file, target, module_path)` only skips a visit whose declarations were
already made by its first occurrence. Crate paths are checked unique (`check_crate_paths`) and roots are pre-created.
Index::build only runs after a successful walk. The guard therefore converts no fixable valid input into a refusal; it is a
true invariant check that fails hard, per the repository rule.

### Harness (`examples/registry_scan.rs`)
- Arguments: no args, `--jobs 0`, unknown flag, extra positional, and a non `<name>-<version>` directory each exit 1 with a
  message; newest-version selection picks `0.1.0` over `0.1.0-alpha`.
- Isolation: each crate runs as `--one <dir>` in a child; abort and timeout become their own `ERR abort|timeout` line.
- Oracle sanity: on a disk fixture (`a/mod.rs`+`a/b.rs`, inline `mod inl { mod x; }`, `#[macro_export]`, trait, inherent impl)
  the old library reports `misplaced=2` (`demo::a::in_b`, `demo::inl::in_x`), the new one `misplaced=0`.
- Widenings: inline-module paths are taken only from `Mod` items with a body in the same file and target, and each inline
  module item is itself checked against its parent, so the chain is grounded in the walk's reaches; trait members drop two
  segments (the trait is checked itself); impl members (`self_type` set) are skipped, which the plan's oracle cannot judge
  anyway. The R2 bug collapses a module into its nearest missing ancestor, never the pre-created crate root, so the
  `macro_rules` at-root widening cannot hide this bug class (see advisory 3).

### Registry results (implementer's raw files, cross-checked per crate)
- Before (harness on 6c744af): ok 1014, with misplaced 165 (2757 items), SuperAboveRoot 3. After: ok 1021, misplaced 0,
  SuperAboveRoot 0. brotli, structured-zstd, wasmi, asn1-rs, flate2, tungstenite and lalrpop now load.
- Crates whose failure changed: jni, rustls, wasm-bindgen moved UnresolvedTrait -> InherentImplSelf; web-sys moved
  InherentImplSelf -> timeout (in-scan). I checked the three: the self types are macro-generated (`bind_java_type!` in jni,
  `enum_builder!` in rustls `msgs/enums.rs`, `pub type Undefined;` in a `#[wasm_bindgen] extern` block): the existing A1 class,
  reached now that their modules are placed correctly.

### web-sys cost (question 6)
- Release `registry_scan --one`, old vs new library, on prefixes of web-sys (first N `gen_*` features, same lib.rs, mod.rs
  filtered): N=300: 3.9-4.0 s / 2107 MB vs 3.3-4.0 s / 2118 MB; N=600: 18.8 s / 7261 MB vs 18.7 s / 7279 MB;
  N=1000: 61.8 s / 20282 MB vs 61.4 s / 20318 MB.
- Synthetic `use super::*` + `pub use gen_i::*` workspace (MemoryTree): n=250 0.96 s vs 0.80 s; n=500 3.86 s vs 3.37 s;
  n=1000 13.6 s / 8966 MB vs 12.4 s / 8948 MB (old vs new).
- Full web-sys on the old library reached 47 GB RSS after 150 s and was stopped to protect the shared host; memory grows
  quadratically (20 GB at 1000 features extrapolates to about 60 GB at 1725), so both versions are far above the "~20 GB"
  quoted, and wall times on this host depend on swap and contention.
- Conclusion: R2 adds no cost of its own (the `visits` vector is one entry per arrival, bounded by the module-path budget);
  the cost is glob settling (R5c). But the stated reason, "1725 `use super::*` modules are no longer collapsed", is wrong:
  on the old code only the first `gen_*` file (gen_AbortController) collapsed into `features`.

### source-index.md
- The new Contract sentence ("The module path in an item's `path` is the chain of `mod` declarations that reached its file,
  whatever the order of the file names") matches the code, including inline modules. The Verification bullet matches the
  fixtures exactly (the flat `a.rs` fixture is nested under `d/` so that it falsifies, as described). The three new bindings
  resolve (docs_contract green). No issue/PR numbers or process wording in any added line of the docs, tests, src or example
  (grep for R-numbers, remediation, audit, issue, PR, stage, baseline, bug: no hit).

## Findings

1. REQUIRED (record only, no code change): the Registry-harness rule requires that "each crate that now fails for another
   reason is root-caused in `validation.md`". validation.md has no R2 entry. Record: jni@0.22.4, rustls@0.23.45 and
   wasm-bindgen@0.2.129 now fail with InherentImplSelf (macro-generated self types, A1 class, previously masked by
   UnresolvedTrait caused by misplaced modules); web-sys@0.3.106 timed out in the parallel scan and fails alone with the same
   InherentImplSelf as before; its cost is unchanged by R2 (measurements above), and the "1725 modules no longer collapsed"
   explanation must not be carried into the record, because it is incorrect.
2. ADVISORY: add a regression fixture for file modules declared inside an inline module (`mod inl { #[path="aa.rs"] mod x;
   mod deeper { mod w; } }` with the files sorting before `lib.rs`). The fix covers it and the old source misplaces it
   (`demo::inl::x::k::in_k` NotFound), but no committed test pins the shape.
3. ADVISORY: the oracle accepts any `macro_rules!` at the crate root, not only `#[macro_export]` ones (Item exposes no export
   flag). It cannot hide the R2 bug class, but a later item that misplaces a non-exported macro at the root would pass it.
4. ADVISORY: `raw/registry-scan-R2.txt` mixes a hand-written `# web-sys ... rerun alone` line into the harness output; keep
   annotations in validation.md so the raw file stays reproducible. The scan also has no per-child memory cap, so `--jobs N`
   with web-sys-sized crates can exhaust the host; consider documenting a lower `--jobs` for large registries.
5. ADVISORY, pre-existing, outside R2: a lib module named `bin_crate` (or `test_crate`, ...) shares module paths with the
   crate roots of bin/test targets. With `pub mod bin_crate { pub mod x { pub fn lib_fn() {} } }` beside `src/bin/x.rs`,
   `declare` returns the bin's crate root for `demo::bin_crate::x`, so lib items land in the bin's root module and
   `demo::bin_crate::x::bin_fn` resolves `Ambiguous`. Identical on the old source; belongs with the crate-path design
   (target.rs `crate_path`), not this item.

VERDICT: BLOCKED

## Recheck

Scope: REQUIRED finding 1 and the new fixture for advisory 2 only.

- Finding 1, resolved. validation.md now has "### R2: registry outcome changes, root-caused (harness rule)". I checked it
  against the two raw scan files, crate by crate:
  - The tallies are right: misplaced 165 crates / 2,757 items -> 0; SuperAboveRoot 3 -> 0; OK 1,014 -> 1,021. The seven
    crates that now load are the right ones.
  - jni, rustls, wasm-bindgen are recorded as the A1 macro-generated class.
  - The same-kind site changes are counted correctly: an awk join over both files gives exactly 15 InherentImplSelf crates
    (bindgen, bitfield, curve25519-dalek, derive_builder, futures-util, http-body-util, js-sys, portable-atomic, pyo3,
    ruma-events, tokio-stream, tower-http, tower, tss-esapi, write-fonts) and 2 UnresolvedTrait crates (petgraph, proptest).
  - petgraph's new site (`IntoNeighbors`, visit/filter.rs:95) names a trait defined inside a `trait_template!` invocation
    (visit/mod.rs:110-117), the same unexpanded-macro class as its old site (`IntoNodeIdentifiers`). That is not an R2 fault.
  - proptest's new site is the enum-variant glob `self::FileFailurePersistence::*` (CA-F6).
  - web-sys is recorded as unchanged by R2, citing the prefix measurements; the quadratic glob memory is assigned to R5c.
    The incorrect "1725 collapsed" explanation is absent.
  - The hand-written web-sys line is gone from raw/registry-scan-R2.txt, which now ends with the harness tally.
- Advisory 2, resolved. `module_tree.rs::a_file_module_declared_inside_an_inline_module_is_placed_under_the_inline_module`
  loads `src/lib.rs` (`pub mod a { pub mod b; }`) and `src/a/b.rs`, and asserts every item path.
  - It passes on the current source.
  - I copied the updated module_tree.rs into the /tmp/r2old tree (old src from `git archive` of c10abd9). There it fails at
    module_tree.rs:1039 with left `demo::a::in_b`, right `demo::a::b::in_b`.
- Gates rerun after the test change: `cargo test -p raf-source --locked --test module_tree` 40 passed;
  `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0; `cargo test --locked --test docs_contract` 138 passed.
- Advisories 3-5 are tracked elsewhere (3 open as advisory; 4 cap moved to R3; 5 moved to R4) and do not block R2.

VERDICT: PASS
