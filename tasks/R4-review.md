Current status: PASS after recheck 3. N-1 and N-2 are fixed and both fixtures fail without their fix. Every finding of the earlier rounds is resolved or advisory (A-6, the R5a self-alias note). See "## Recheck 3". The verdicts of the earlier rounds below are superseded.

# R4 review: index facts (CA-F5, CA-F6, A1, A2, A3, S-15, G16)

## Scope

The uncommitted R4 change on top of 51d84e5 (21 files, +2252/-492): `crates/raf-source/src/{error,facts,impls,index,item,lib,manifest,members,modtree,resolve,settle,target,workspace}.rs`, seven test files, and `docs/capabilities/source-index.md`.

It was judged against:

- "### R4" in `remediation-plan.md`, with its Mutation rule, Per-item gates and Registry harness;
- the user decisions A1 ("Per-item marker (Recommended)") and A2 ("Key crates by package (Recommended)");
- AGENTS.md and CLAUDE.md (fail hard, no fallbacks, no unsafe, no process wording);
- R3's rule that every charge is an upper bound (held ≤ charged).

The four declared deviations are judged under "Deviations".

## Evidence

Scratch work is in `/tmp/r4rev` (gate logs, probe crate, old-code copy). No repository file was edited except this one.

### Gates

| Gate | Command | Result |
|---|---|---|
| raf-source debug | `cargo test -p raf-source --locked` | First run: 1 failure, `module_tree::a_chain_of_files_that_each_declare_the_next_twice_is_refused_quickly_beside_a_small_diamond`, at 11.07 s against its 10 s wall-clock bound (load average 34–40 on 32 cores). Rerun with `--no-fail-fast`: **331 passed, 0 failed** (23 binaries). See A-3. |
| raf-source release | `cargo test -p raf-source --locked --release` | **331 passed, 0 failed** |
| clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | rc=0 |
| rustdoc | `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked` | rc=0 |
| docs_contract + repo_gates | `cargo test --locked --test docs_contract --test repo_gates` | 139 + 55 passed |
| cross-check | `make cross-check` | rc=0 |
| msrv | `make msrv` | rc=0 |
| reuse | `make reuse` | rc=0, 224/224 compliant |

Other checks:

- **Process wording:** grepping the diff's added lines for item ids, `remediation`, `reviewer`, `#NN`, `TODO` and `unsafe` finds nothing.
- **`.deficiencies`:** holds only the notice and the marker.

### Falsification on old code

Method: `git archive 51d84e5 | tar -x -C /tmp/r4rev/old`, with its own `CARGO_TARGET_DIR`. I copied in the new `target_roots.rs` and `trait_keys.rs`, plus the implementer's `R4-head-shims.rs.txt`, and ran each test with `--exact`. **All six fail on old code:**

- `target_roots::a_lib_module_named_bin_crate_stays_in_the_library`: `demo::bin_crate::x::bin_fn: the path names 2 different symbols`.
- `target_roots::an_explicit_path_bin_beside_a_lib_module_bin_crate_is_ambiguous_in_both_file_orders`: assertion failed for `a/x.rs`.
- `trait_keys::a_macro_call_in_a_trait_body_is_skipped`: `unsupported item: decl ! () ;`.
- `r4_head_shims::ur_two_libraries_with_one_name_load`: `the crate path shared is already used by ...`.
- `r4_head_shims::si_bitflags_style_loads`: `the self type Kind of an inherent impl is not a local struct, enum or union`.
- `r4_head_shims::si_enum_glob_loads`: `the trait self::Inner::* of an impl cannot be named`.

These agree with the implementer's `R4-falsification.txt`.

### Semantics probes

A probe crate (`/tmp/r4rev/probe`, `src/bin/sem.rs`) depends on the worktree's raf-source and was run in release. Results:

- **P3, build script in the other file order** (`package.build = "zz/build.rs"` beside `pub mod build_script { pub fn main() {} }`): `demo::build_script::main` is `Ambiguous [src/lib.rs, zz/build.rs]`. Correct.
- **P4, `[target.'cfg(unix)'.dependencies] ren = { workspace = true }`** (renamed in `[workspace.dependencies]`) and a target-specific build-dependency:
  - the library names `ren` but not `shared`;
  - the build script names `shared` but not `ren`.

  Correct.
- **P5, optional dependency and a leading `::`:** both resolve. This over-approximates features, which is acceptable.
- **P6, `extern crate` placement:** an `extern crate` in a non-root module names the crate only there, and a sibling module's path is `NotFound`. Correct (only root `extern crate` items join the prelude).
- **P7, A1 marker:**
  - `impl Kind` in `k.rs`, reached as `k` and `k2`, is listed once per module path (`demo::k::Kind`, `demo::k2::Kind`).
  - `impl super::Gen` gives `demo::Gen`, `impl dyn Shape + Send` and `impl (dyn Shape)` give `demo::Shape`, and `impl ::ext::Ty` gives `ext::Ty`.
  - The six items marked `Unresolved` are exactly the members of the listed impls.
  - The self path is `NotFound`. A path through it (`demo::k::Kind::is_a`, `::zzz`, `::is_a::deeper`) is `UnresolvedSelf`.
- **P8:**
  - `resolve("shared::Gen::f")` is `Ambiguous(2)`.
  - `resolve_in_package("p-one", ..)` is `UnresolvedSelf` and `resolve_in_package("p-two", ..)` is `NotFound`. So the marker is filtered by package.
- **P1, P1-cargo, P10:** see REQUIRED-1 and REQUIRED-2.
- **P2:** see A-1.
- **P9:** see A-2.

I checked Cargo/rustc directly (`/tmp/r4rev/cargows`). p-three depends on p-one (lib `shared`) and dev-depends on `shared = { path = "../two", package = "p-two" }`:

- `cargo build -p p-three` **succeeds**, with `shared` naming p-one in library code;
- `cargo test --no-run` fails with **E0464** (multiple candidates for `shared`) in the lib-test and test-target units.

### Held ≤ charged (counting global allocator)

`/tmp/r4rev/probe/src/main.rs` measures the live bytes a `Workspace` adds over the load, against `Usage`, in release:

| Shape | kept | item_bytes charged | kept ≤ item alone |
|---|---|---|---|
| A2: 150 packages, all libs named `shared`, each depending on all others in four tables (normal renamed path, dev `workspace = true` renamed, build unrenamed, `[target]` registry) | 30,075,942 | 57,932,169 | yes |
| same, deps → comments (Δ) | Δkept 28,019,490 | Δitem 53,928,690 | yes |
| A2 with distinct lib names | 32,983,172 | 62,966,740 | yes |
| A1: one file reached by 8 module paths, 2000 unresolved inherent impls × 10 members (16,000 `UnresolvedImpl`) | 90,356,796 | 151,489,292 | yes |
| A1: 16 module paths × 4000 member-less unresolved impls (64,000 `UnresolvedImpl`) | 15,803,262 | 32,516,228 | yes |

Every shape keeps less than its item-bytes charge alone, without counting the fact or total budgets.

### Contract figure "170 MiB"

`/tmp/r4rev/probe/src/bin/chain17.rs` builds the module_tree `chain(2, 17)` shape (967 bytes of source, 2^18 − 1 walks) and ran in release three times:

- every run is refused by the item-bytes budget (`src/m14.rs:2 ... (536870912)`);
- each took 1.04–1.18 s;
- VmHWM was 172,760–172,932 kB, which is **168.7 MiB**.

The Contract's "after 1.0 s at a peak of 170 MiB" is accurate.

### Entry points

`src/bin/disk.rs` loads with `DiskTree` and roots `["crates"]`:

- **katamari:** OK. 42 packages, 13,774 items, 2 unresolved impls (`katamari_keys::material::{DataKey,VaultKek}`).
- **purrdf:** OK. 44 packages, 83,653 items, 6 unresolved impls. Two of its packages have a lib named `purrdf` (purrdf, purrdf-capi): `resolve("purrdf")` is `Ambiguous(2)` and `resolve_in_package("purrdf", "purrdf")` is `Ok`. A real A2 case.
- **This repository:** OK. 2 packages, 2,310 items, 0 unresolved impls.

### Registry scan

`raw/registry-scan-R4.txt` covers 1,207 crates: 1,163 OK, 0 misplaced. Compared with `registry-scan-R3.txt` (1,177 crates, 1,021 OK):

- `InherentImplSelf` goes from 100 to 0;
- "inherent impl" messages: 0;
- no remaining `UnsupportedItem` is a trait-body macro;
- no `self::<Enum>::*` trait failure remains.

The remaining `UnsupportedItem`, `UnresolvedMod`, `Target`, `Syntax` and `Tree` failures are classes outside R4. One message change is R4's: macro_rules_attribute's origin is now "package discovery", from the lib-origin fix.

I root-caused three of the crates whose first failure changed, in `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f`:

- **bincode 1.3.3** (edition 2015). Before: a `serde_if_integer128!` macro in a trait body (CA-F5, fixed). Now: `src/de/mod.rs` has `use serde;` (2015 crate-relative, through the root `extern crate serde;`) and `impl serde::Deserializer<'de> for ...`. This is the R5a self-alias / 2015 class.
- **core-foundation 0.10.1** (2021). Before: `InherentImplSelf` for the macro-generated `CFAttributedString` (A1, fixed). Now: `src/base.rs` has `use std;` and `impl fmt::Debug for CFType`. This is the R5a `use std;` class.
- **ena 0.14.4** (edition 2015). Before: glob `self::UndoLog::*` (CA-F6, fixed). Now: `use undo_log::{Rollback, ..}` (2015 crate-relative) with `impl Rollback<..> for Vec<..>`. This is the R5a edition-2015 class.

I also read wide 1.7.1 (`pub use bytemuck;` and `use bytemuck::*;`), which is R5a's "an import never resolves through its own binding". In each case the R4 fix exposed a later error that R5a owns.

## Deviations

1. **The blanket "path through a lib module named `test_crate` and so on is Ambiguous" rule is removed; Ambiguous only where both crates define the path.** Sound. The canonical namespace is RAF's own construction. Grouping by `(target, path, kind)` makes overlap Ambiguous wherever both crates define the path, in either file order (fixtures, and probe P3 for the other build-script order). It also lets a path that only one crate defines resolve to it, which is what R2-review advisory 5 asks for. The Contract is updated to match.
2. **A glob of an unseen local path becomes an unconfirmed trait-key candidate.** Sound in principle. It is the existing dependency-glob rule ("a glob that cannot be looked into is a candidate under its path") applied to a local module or enum that a macro or `include!` generates. Several candidates still give `TraitKey::Unresolved`. The new `unseen_local` resolves an ambiguous module binding first-wins; see A-1.
3. **Root `extern crate` items, including `extern crate self as x`, join the extern prelude.** Correct per the rustc reference: `extern crate` at the crate root adds its name to the extern prelude, and `extern crate self as x` is the usual self-naming idiom. The fixture `extern_crate_self_names_the_crate_itself_from_every_module` covers it, and probe P6 covers the non-root case. See REQUIRED-2 for the multi-root case.
4. **The build script does not see its own library.** Correct Cargo semantics: a build script links only `[build-dependencies]`. Fixture: `p_one::build_script::only_one` is `NotFound`.

## Findings

### REQUIRED-1: dev-dependency libraries leak into non-test code when one crate name is both a normal and a dev dependency

`ExternName` keeps one `everywhere` flag per **name** (`index.rs`, `preludes` / `offer`: `entry.everywhere |= everywhere`), but a list of `roots`. `extern_name` filters on that flag only. So when a name stands for a normal dependency's library and a dev-dependency's library of different workspace packages, non-test code of a library or binary sees both roots.

Probe P1:

- p-three has `[dependencies] p-one` (lib `shared`) and `[dev-dependencies] shared = { path = "../two", package = "p-two" }`.
- Non-test `pub use shared::X as A;` gives `resolve("p_three::A")` = `Ambiguous [one/src/lib.rs, two/src/lib.rs]`.
- With real Cargo the same package's `cargo build` succeeds and `shared` is p-one. Only the test units are E0464-ambiguous.

The Contract says library and binary code names `[dependencies]` and names `[dev-dependencies]` only in test code. The code does not do this in this case. The same applies to `crate_known`.

**Fix:** keep test-only-ness per root (for example `roots: Vec<(usize, bool)>`, or separate everywhere and test-only lists). Have `extern_name` / `extern_roots` return only the roots the code's test flag allows.

**Fixture:** the layout above.

- `p_three::A` (non-test) is `Ok` to `one/src/lib.rs`.
- A `#[cfg(test)]` re-export of `shared::X` and the test target's `shared::X` are `Ambiguous`.
- The fixture fails on the current code.

### REQUIRED-2: an `extern crate` of a name that stands for several workspace libraries silently names no crate

`index.rs` (the `extern crate` arm of the builder) records `module_of` only for `Some([root])`. With two roots the item gets no module, so a path through it is `NotFound`, the answer for "the workspace defines nothing here". The same name used directly in a `use` is `Ambiguous`.

Probe evidence:

- **P1:** `#[cfg(test)] extern crate shared as sh; #[cfg(test)] mod t2 { pub use sh::X as C; }` gives `p_three::t2::C` = `NotFound`, beside `p_three::tests::B` = `Ambiguous`.
- **P10:** with a `[target]` table redeclaring `shared` for a second package, `use shared::X` is `Ambiguous` but the `extern crate shared as sh` route is `NotFound`.

rustc reports E0464 for this `extern crate`. The index drops the ambiguity on the floor instead of failing hard, which is what the brief's "Ambiguous vs first-wins anywhere" check targets.

**Fix:** when the prelude entry has more than one root, make a path through the `extern crate` item `Ambiguous`, naming each crate root. For example, record every root for the item and let the walk's existing multi-module merge report the conflict.

**Fixture:** the P1 `t2::C` case, `Ambiguous` beside the single-root `p_four::Again`, which stays `Ok`.

### ADVISORY

- **A-1: first-wins in `unseen_local`** (`impls.rs`, the new function: `bound.keys().find_map(|id| self.module_for(*id))`, twice).
  - Probe P2: in `mod user { use crate::a::*; use crate::b::*; use m::Gen::*; impl Tr for u8 { fn f() {} } }`, where `a::m` and `b::m` both exist, the trait is keyed `Canonical("demo::a::m::Gen::Tr")`. Before R4 this was a hard `UnresolvedTrait`.
  - The input is invalid Rust (E0659). The same first-wins pattern already exists in `absolute()` (`impls.rs` around lines 324 and 334).
  - Recommended: in R5a, which rewrites trait-key derivation as a worklist, collect every module a binding stands for at all three sites, so several give several candidates and `TraitKey::Unresolved`.
- **A-2: an inherited dependency (`workspace = true`) is silently treated as outside the index** when the package has no workspace, is not a member, or the workspace has no such `[workspace.dependencies]` entry (`workspace.rs` `dependency_info`). Probe P9: it loads, and `use ren::X` is `NotFound`. Cargo refuses that manifest. `members.rs` already behaves this way. Consider a manifest error naming the dependency key, as `package.edition` inheritance already does.
- **A-3: the module_tree timing test is flaky on a loaded host.** It failed once at 11.07 s against a 10 s bound in the full parallel debug run (load average 34–40) and passed on rerun. Run on its own, alternating three times: new code 5.29 / 5.94 / 6.25 s, old code 4.76 / 5.82 / 5.04 s. R4 adds about 0.5 s and does not cause the failure, but the bound has under 2× margin in debug under contention.
- **A-4: the build-script A3 fixture covers one file order.** `build.rs` sorts before `src/lib.rs`. Probe P3 confirms the other order (`package.build = "zz/build.rs"`) is also `Ambiguous`. Adding it would match the explicit-path bin fixture.
- **A-5: a Contract figure outside R4's list changed.** The three-limit figure ("716 → 746 MiB, peak 721 → 752 MiB, 11.4 → 19 s", with the doc-comment shape re-specified) is not in R4's listed scope, and I did not re-measure it.
- **A-6: `impl ::ext::Ty { fn e() }` is recorded at `ext::Ty`,** a path outside every crate root, so `resolve("ext::Ty::e")` is `UnresolvedSelf` and not `NotFound`. This is documented ("a path that starts with `::` from its first segment"), and the input is invalid Rust (E0116), so it is cosmetic.
- **A-7: the registry root causes need recording in `validation.md`,** as the Registry harness rule requires. Changed first failures: bincode, core-foundation, ena, js-sys, read-fonts, wide, write-fonts, interprocess. My spot-check of three (above) agrees they now stop at R5a-owned classes.

## Plan-item coverage

Each item below matches the plan.

- **CA-F5:** `trait_keys::a_macro_call_in_a_trait_body_is_skipped`.
- **CA-F6:** `symbol_index::a_glob_of_enum_variants_beside_an_impl_loads`.
- **A1:**
  - `SelfType::Unresolved` items under the written self path;
  - `items()` / `item_at()` carry the marker;
  - `unresolved_impls()`;
  - `UnresolvedSelf`;
  - `bitflags!` and `impl dyn LocalTrait` fixtures;
  - Contract and Verification updated;
  - fail-hard is kept: `super` above the root is still an error, and the old `(None, None) => continue` became `Internal`.
- **A2:**
  - `crate_roots` is keyed by crate path with a list of roots, and `module_by_path` by (crate root, path);
  - the extern prelude is built per target;
  - `resolve` is `Ambiguous` naming both crate roots;
  - `resolve_in_package` and `UnknownPackage`;
  - the third-package fixture, renames, `workspace = true` renames, and dev-dependencies in test code and test targets;
  - purrdf loads.
- **A3:** in both orders.
- **R2-advisory 5:** fixed.
- **S-15 / G16:** both fixtures.
- **Entry points:** met.

The two REQUIRED findings are gaps in the A2 extern-prelude semantics: per-root test scoping, and ambiguity through `extern crate`.

VERDICT: BLOCKED

## Recheck

This checks the revised uncommitted change on 51d84e5: 21 files, +2594/−526.

### REQUIRED-1: dev-dependencies leaking into non-test code

**Resolved.**

The fix: `ExternName.roots` is now `Vec<(usize, bool)>`, and `ExternName::roots(test)` filters per root. Both `extern_roots` and the `extern crate` arm use it.

I rebuilt the probe crate against the revised code (`/tmp/r4rev/probe`, `src/bin/sem.rs`, release) and ran these layouts. Each is p-one (lib `shared`) beside p-two (lib `shared`).

- **P1 (my original layout).**
  - Non-test `p_three::A` is now `Ok` → `one/src/lib.rs`. It was `Ambiguous`.
  - `p_three::tests::B` and the test target's `D` stay `Ambiguous`.
  - This matches real Cargo: `cargo build` succeeds, and the test units fail with E0464.
- **P11a, build-dependencies.** `[dependencies] p-one`, plus `[build-dependencies] shared = { path = "../two", package = "p-two" }`.
  - These are all `Ok` → one: library `A`, root `extern crate shared as sh` → `E`, the submodule route `m::F`, and `#[cfg(test)]` `tests::B`. Build-dependencies never reach the library, its test code included.
  - `build.rs` `G` → two.
- **P11b, target-specific dev-dependency.** `[target.'cfg(unix)'.dev-dependencies] shared = p-two`.
  - `A`, `E` and `m::F` → one.
  - `tests::B` and the test target are `Ambiguous`.
- **P11c, optional normal dependency.** `p-one` with `optional = true`, plus a renamed dev-dependency.
  - `A` and `E` → one.
  - `tests::B` is `Ambiguous`.
- **P11d, renamed normal dependency.** `shared = { path = "../two", package = "p-two" }`, plus an unrenamed dev-dependency on p-one.
  - `A`, `E`, `m::F` and the binary's non-test `H` → two.
  - Library test `tests::B`, binary test `t::I` and the example `J` are `Ambiguous`. Examples get dev-dependencies.
- **P11e, no collision.** Everything → one, test code included.
- **P4, P5, P6** (earlier probes, rerun): unchanged and correct.
  - A target-specific `workspace = true` rename resolves.
  - A build-only dependency stays out of the library.
  - A leading `::` resolves, and so does an optional dependency.
  - A non-root `extern crate` stays local.

Fixture `use_resolution::a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name` is my layout plus `extern crate`, binary and test-target routes. The Contract text (source-index "Use-alias index") states the per-library test scoping.

### REQUIRED-2: `extern crate` of a several-root name naming no crate

**Resolved.**

The fix: the `extern crate` arm records every root in `crate_of`, charged as `btree_entry + capacity × usize`. `modules_for` replaces `module_for` at every site (resolve, settle, impls), so a walk through the item visits each root.

Probes:

- **P1:** `#[cfg(test)] extern crate shared as sh; mod t2 { pub use sh::X as C; }` is now `Ambiguous [one, two]`. It was `NotFound`.
- **P10** (two `[target]` tables redeclaring `shared`): `extern crate shared as sh` → `Y` is `Ambiguous`, as `use shared::X` is.
- **Single root** (P11a/b/c/e `E`): stays `Ok`.

### A-1: first-wins over an ambiguously bound module

**Addressed.**

Probe P2''': the two globs bring in `a::m` and `b::m`, and the key is now `Unresolved { candidates: [demo::a::m::Gen::Tr, demo::b::m::Gen::Tr] }`. It was `Canonical(demo::a::m::...)`. Both `absolute()` and `unseen_local` now follow every bound module.

### A-2: inherited dependency the workspace cannot give

**Addressed.**

Probe P9 now gives `three/Cargo.toml: invalid manifest: `workspace = true` but there is no workspace manifest above the package (reached by manifest key `dependencies.ren`)`. The fixture covers all three cases (no workspace, not a member, missing entry) beside one that loads.

### A-3, A-4, A-5, A-7

- **A-3:** the debug wall-clock bound is removed. The budget refusal is still asserted in all builds, and the 30 s bound applies to release only.
- **A-4:** both build-script file orders are covered.
- **A-5:** the figure is re-stated as "746 kept / 752 peak MiB, in 10 to 15 s". I did not re-measure it.
- **A-7:** `validation.md` "### R4: registry outcome changes, root-caused" lists every changed crate with cause and owner. It agrees with my three spot-checks.

### Falsification on 51d84e5

I spot-ran these on the old-code copy:

- **trait_keys:** the new `trait_keys.rs` copied in as-is.
- **recheck_shims:** the two `use_resolution.rs` fixtures and the `SPLIT` constant extracted verbatim into `recheck_shims.rs`. They compile on the old API.

All four fail:

- `recheck_shims::a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name`: panics with "`p_three::d::D` names nothing in the workspace".
- `recheck_shims::a_name_bound_to_several_modules_gives_a_trait_key_through_each`: panics with "src/lib.rs:11: the trait `m::Gen::*` of an impl cannot be named by an absolute path".
- `trait_keys::an_inherited_dependency_the_workspace_cannot_give_is_an_error_naming_the_key`: panics with "no workspace manifest above the package", because old code loads it.
- `target_roots::a_build_script_main_beside_a_lib_module_build_script_is_ambiguous`: returns `Ok` with two sites.

On the first R4 revision, my P1 probe returned `Ambiguous` for `p_three::A` and `NotFound` for `p_three::t2::C`. The new fixture asserts `Ok` and `Ambiguous` for these, so it falsifies both REQUIRED defects.

### Held ≤ charged (rerun)

I reran the counting-allocator probe at 150 packages:

| Shape | kept | item_bytes charged |
|---|---|---|
| A2 same-name, all dependency tables | 34,683,942 | 65,456,545 |
| A2 Δ (deps vs comments) | 32,617,890 | 61,438,290 |
| A2 distinct names | 37,999,172 | 70,491,116 |
| A1, 16,000 unresolved impls | 90,356,796 | 151,489,668 |
| A1, 64,000 member-less unresolved impls | 15,803,262 | 32,516,604 |

Every shape keeps less than its item-bytes charge.

### Gates (rerun)

| Gate | Result |
|---|---|
| raf-source debug, `--no-fail-fast` | **334 passed, 0 failed** |
| raf-source release | **334 passed, 0 failed** |
| clippy `-D warnings` | rc=0 |
| rustdoc `-Dwarnings` | rc=0 |
| docs_contract + repo_gates | 139 + 55 passed |
| `make cross-check` | rc=0 |
| `make msrv` | rc=0 |
| `make reuse` | rc=0 |

The diff's added lines contain no process wording, and `.deficiencies` holds only its marker.

### ADVISORY (new)

- **A-8: a path that only one library defines resolves through an ambiguous crate name.**
  - In test code where `shared` stands for both libraries, `pub use shared::only_one as K` resolves `Ok` to the one library that defines `only_one` (probe `src/bin/k.rs`). rustc rejects any use of that name there (E0464).
  - The Contract says this explicitly ("`Ambiguous` where both define it"). It is deterministic, not first-wins, and only arises in test code that does not compile. So it is not blocking.
  - It differs from the crate-path rule ("two libraries named `shared`: `Ambiguous` whatever follows"). Making a several-root name `Ambiguous` whatever follows would make the two rules match.

A-6 stands as written.

VERDICT: PASS

## Recheck 2

This round covers the A-8 follow-up, uncommitted, in `resolve.rs`, `settle.rs`, `impls.rs`, `tests/use_resolution.rs` and `source-index.md`.

- **Walk check:** a walk step that would go on in the crate roots of several crates fails `SeveralCrates`. `resolve` maps it to `Ambiguous`, naming the crate roots.
- **Imports:** a named import that hits it binds the name in the type namespace to those roots.
- **Globs:** a glob of it brings in nothing.
- **Trait keys:** a trait path through such a name keys `Unresolved`, with one candidate in each library.

### No path that compiles changed

I compared resolve outputs before and after this round on katamari, purrdf and this repository.

**Method:**

- **"Before":** my probe's build of the previous revision. Its build directory was saved to `/tmp/r4rev/prev-build` before any rebuild.
- **`dump-prev`:** built with `rustc --extern raf_source=<prev rlib> --extern raf_source=<prev rmeta>`.
- **`dump-new`:** the same source built against the current tree.

`dump.rs` loads `crates/` and, for every distinct item path, prints three things:

- `resolve(path)`, with sorted sites or ambiguous ids as `file:line`;
- every successful `resolve_in_package(pkg, path)`;
- every item's trait key.

**Results:**

| Workspace | Paths | Output lines | Diff lines |
|---|---|---|---|
| katamari | 13,648 | 27,344 | **0** |
| purrdf | 82,514 | 165,238 | **0** |
| this repository | 2,294 | 4,594 | **0** |

purrdf includes 1,035 `Ambiguous` lines from its two libraries named `purrdf`. Those are unchanged too.

**Semantics probes** (`sem`, rerun):

- Every non-test answer is unchanged: P1 `A`, P4, P5, P6, and P11a–e (library, binary, submodule and `extern crate` routes, build script).
- P11f `t2::K`, a path through a `#[cfg(test)] extern crate` to an item only p-one defines, is now `Ambiguous`. It was `NotFound`.

Checked against the previous build with the same probe source (`k2`):

- The test-code trait key was `Canonical("p_two::Tr")`, the first library. It is now `Unresolved { candidates: ["p_two::Tr", "shared::Tr"] }`.
- The library-code key stays `Canonical("shared::Tr")`.

### Fixture falsifiability

Each mutation was applied to a copy, `/tmp/r4rev/mut`.

1. **`Index::one_crate` never refuses** (`if roots.len() > 1 && false`).
   - `use_resolution::a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name` fails at `p_three::tests::B`, because the ids are no longer the crate roots.
2. **The trait-key `SeveralCrates` arm is disabled,** so the key falls through to `absolute()`.
   - The same test fails: `test_side` is `Canonical("shared::Tr")`, not `Unresolved`. The fixture catches the case where the candidate texts collapse.

### Findings

#### REQUIRED N-1: an import through a several-library name is `NotFound` in the value namespace

The `SeveralCrates` arm of `settle_named` (`settle.rs`, about line 646) binds the import only `if ns == Namespace::Type`. So `use shared::only_one as K;` and `use shared::X as B;` in test code leave `K` and `B` unbound in the value namespace.

Probe `k2`:

- `resolve("p_three::t::K")` is `Ambiguous` (two crate roots).
- `resolve_in("p_three::t::K", Namespace::Value)` is **`NotFound`**.
- `resolve_in("p_three::t::B", Namespace::Value)` is **`NotFound`**, though `X` is a unit struct.
- In the previous round both were `Ok`/`Ambiguous` on the items themselves.

`NotFound` means "the workspace defines nothing at the path". The Contract says "a path ... through an import of it is `Ambiguous`". An unbound value name also leaves room for a glob of the module to supply `K` silently.

**Fix:**

- When the import path is longer than the crate name (`rest < segs.len()`), bind the name to the roots in every namespace that the walk tried.
- Keep type-namespace-only binding for `use shared;` / `use shared as S;`, where `rest == segs.len()`.

**Fixture:** `resolve_in(.., Namespace::Value)` of `p_three::tests::T` and `p_three::t2::E` is `Ambiguous`.

#### REQUIRED N-2: a glob through a local import of a several-library name brings in both crates

`pub use shared; pub use shared::*;` in `#[cfg(test)] mod t` makes `p_three::t::only_one` resolve **`Ok`** to p-one's `only_one` (probe `k2`). Without the glob line it is `NotFound` (probe `k3`).

The cause: the glob's head `shared` is found as the local import, which is bound to both crate roots. A walk with no further segment never reaches `one_crate`, so `glob_targets` gets both roots and the glob brings in the names of both crates.

The Contract says "a glob of it brings in nothing", and rustc rejects the code (E0464). The same leak existed in the previous round (`k2-prev`), but this round's Contract now states the opposite.

**Fix:** apply `one_crate` to the glob's targets, for example at the end of `walk` or in `glob_targets` / the glob settle path, so a glob whose path ends in several crate roots brings in nothing.

**Fixture:** the `k2` layout, with `p_three::t::only_one` `NotFound` (or `Ambiguous`, if routed through the binding), beside a single-root glob that brings in its names.

#### ADVISORY

- **R5a self-alias class, not this round:** `pub use shared;` in non-test code with one root makes `p_three::shared` and `n::Via` (`super::shared::X`) `NotFound`, identical on the previous build. R5a ("an import never resolves through its own binding") owns it.

### Contract text

The trait-key paragraph ("one candidate per library: its crate path followed by the rest of the path", "never the first") matches the code, the fixture and the probes.

In the Use-alias paragraph and the `Ambiguous` list:

- These clauses are accurate:
  - "ambiguous whatever follows it";
  - "through an `extern crate` item of it";
  - "naming the crate root of each library".
- "Through an import of it" holds for `resolve` but not for `resolve_in` in the value or macro namespace (N-1).
- "A glob of it brings in nothing" does not hold for a glob through a local import of it (N-2).

### Gates (rerun)

| Gate | Result |
|---|---|
| raf-source debug (`--no-fail-fast`) | **334 passed, 0 failed** |
| raf-source release (`--no-fail-fast`) | **334 passed, 0 failed** |
| clippy `-D warnings` | rc=0 |
| rustdoc `-Dwarnings` | rc=0 |
| docs_contract + repo_gates | 139 + 55 passed |
| `make cross-check` | rc=0 |
| `make msrv` | rc=0 |
| `make reuse` | rc=0 |

VERDICT: BLOCKED

## Recheck 3

This round covers the uncommitted N-1 and N-2 fixes:

- `settle_named` binds an import that goes past a several-library crate name in every namespace; an import of the name alone stays type-only.
- `glob_targets` runs `one_crate` on its targets.

### N-1: imports through a several-library name in the value namespace

**Fixed.**

Probe `k2`, rebuilt against the current tree:

- `resolve_in("p_three::t::K", Value)` (`use shared::only_one as K`) is `Ambiguous`, naming both crate roots. It was `NotFound`.
- `resolve_in("p_three::t::B", Value)` is `Ambiguous`.

Probe `k4`:

- `use sh::mac as M` through a test `extern crate` is `Ambiguous` in the macro namespace and in `resolve`.
- `use shared as S`, the name alone, is `Ambiguous` in the type namespace and `NotFound` in the value namespace. That is correct: a crate is only a type-namespace name, and the Contract says so.

### N-2: globs through a several-library name

**Fixed.**

- `k2`: `pub use shared; pub use shared::*;` gives `t::only_one` `NotFound`. It was `Ok` → p-one.
- `k4`: each of these globs brings in nothing in test code, so every name only they could bring is `NotFound`:
  - direct `use shared::*`;
  - `use ::shared::*`;
  - `use sh::*` through a test `extern crate`;
  - a deeper glob `use shared::sub::*`.
- `k4`, non-test code, same package: `use shared::*` and `use shared::sub::*` still bring in p-one's names (`g::only_one`, `g::X`, `g::deep` all `Ok`). There is no over-blocking.

### No regression

- **`sem`:** P1, P10, P11f and P2''' are unchanged.
- **Resolve dump** against the saved previous-round build:
  - purrdf: 0 diff lines.
  - This repository: 56 diff lines, all line-number shifts in raf-source's own `settle.rs`. After normalising line numbers there is no difference.
  - katamari changed upstream. Running `dump-prev` and the new dump on today's katamari source gives 0 diff lines.

### Falsifiability

Each fix was mutated in `/tmp/r4rev/mut` and `use_resolution` was run. Each mutation fails exactly `a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name` (30 passed, 1 failed).

- **N-1:** the condition becomes `ns == Namespace::Type || (rest < segs.len() && false)`. The test fails at `p_three::tests::T: Err(NotFound(..))`.
- **N-2:** `self.one_crate(&targets, segs.len())?` in `glob_targets` is removed. The test fails at `p_three::t::only_one`, which is `Ok(shared::only_one, via_glob)`.

### Contract text

The Use-alias paragraph now reads:

> "... is `Ambiguous`, naming the crate root of each library, even where only one of them defines the rest of the path, and in each name space the path could end in (an import of the name alone names crates, in the type name space); a glob of it, directly or through an import of it, brings in nothing."

It matches the probes above exactly. The trait-key paragraph and the `Ambiguous` list are unchanged and still accurate.

### Gates (rerun)

| Gate | Result |
|---|---|
| raf-source debug, `--no-fail-fast` | **334 passed, 0 failed** |
| raf-source release | **334 passed, 0 failed** |
| clippy `-D warnings` | rc=0 |
| rustdoc `-Dwarnings` | rc=0 |
| docs_contract + repo_gates | 139 + 55 passed |
| `make cross-check` | rc=0 |
| `make msrv` | rc=0 |
| `make reuse` | rc=0 |

VERDICT: PASS
