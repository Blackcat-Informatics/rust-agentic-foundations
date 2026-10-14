# R5a review: extern-prelude aliases and stack-safe trait keys

## Scope

I reviewed the uncommitted R5a diff on top of d1c4a7f in the worktree `2-shared-crates-raf-okf-raf-source`, covering 9 files: `impls.rs`, `index.rs`, `resolve.rs`, `settle.rs`, three test files, `docs/capabilities/source-index.md` and `SECURITY.md`.

I checked it against:
- the plan's "### R5a" section, including its "Revised at implementation (rustc-verified)" bullet;
- the Mutation rule, the Per-item gates and the Registry harness;
- AGENTS.md and CLAUDE.md;
- gap-analysis G2 and the completion audit's CA-F1 and CA-F7.

Old code was extracted with `git archive d1c4a7f` into `/tmp/r5a-review/old`. The mutations ran on a copy of the new tree at `/tmp/r5a-review/mut`. The worktree was not edited.

## Evidence

### Gates

All gates ran on the R5a tree under `ulimit -v 40000000`, and every one exits 0:
- `make fmt`;
- `cargo test -p raf-source --locked` in debug, and `--release`: every binary is ok;
- `make clippy`;
- `make doc`;
- `cargo test --locked --test docs_contract --test repo_gates`: 139 and 55 passed;
- `make cross-check`;
- `make msrv` on 1.98.1;
- `make reuse`: 224 of 224 files compliant.

### rustc probes

The probes used rustc 1.100 nightly, with throwaway rlibs `a` and `serde` passed through `--extern`.

| Shape | 2015 | 2018 / 2021 |
|---|---|---|
| `extern crate a as d; use d::f;` (at the root, and in a submodule) | OK | OK |
| `extern crate a as d;` with `d::f()` in a submodule's code | OK | OK |
| `mod m { extern crate a as d; } use d::f;` | E0432 | E0432 |
| `use ::a::f;` without `extern crate` | E0432 | OK |
| `extern crate a; use ::a::f;` | OK | OK |
| `mod m{..} use ::m::g;` | OK | E0432 |
| `::m::g()` in code | OK | E0433 |
| `use a::f;` without `extern crate` | E0432 | OK |
| `mod a{..} use a::f;` (local item and crate) | OK | OK: the item shadows the crate |
| `mod a{..} use self::a::f;` | OK | OK |
| `use m::a; use a::f;` (import and crate) | OK | OK: the import shadows the crate |
| `use m::*; use a::f;` (glob `a` and crate) | OK | **E0659** |
| `use m::*; pub use a;` | E0365 | **E0659** |
| `use m::*; a::f()` (glob `a`, in code) | OK | OK: the glob wins in code |
| a glob of `m` (which holds `pub use a;`) then `use a::f;` | E0432 | OK: same crate |
| a glob of a value `fn a` then `use a::f;` | E0432 | OK |
| `macro_rules! a` locally or `#[macro_export]` in a module, then `use a::f;` | E0432 | OK: the macro name space does not collide |
| `mod m { pub use std as serde; } use m::*; pub use serde::Serialize;` | | **E0659** |
| `#![no_std]` with a glob that brings in a module `std`, then `use std::fmt;` | | OK: no ambiguity |
| a glob of `inner::x::x` then `pub use x::x;` (an import that shadows the glob it used) | | OK |
| `mod inner { use super::*; pub use serde::*; } pub use inner::Serialize; impl Serialize for &Foo {}` | | OK |

The implementer's revised claims hold:
- a local item or an explicit import shadows the crate;
- only glob-against-crate is E0659;
- a glob of the same crate, or of a value only, does not collide;
- edition 2015 `use` paths start at the crate root;
- a leading `::` in 2015 starts at the crate root.

### Behaviour probes

I ran a throwaway `probe` binary over a `MemoryTree` and a `disk` binary over a `DiskTree`, built against the new and the old raf-source.

- Agrees with rustc:
  - `pub use serde;`, `pub use std as s2;` and `pub use self::x;` give NotFound; the old code gave Cyclic for the first and third.
  - `use a::b; mod a { pub use super::b; }` and the renamed cycle give Cyclic, which is correct.
  - `mod imp { pub mod parse {..} } pub use imp::parse; pub use parse::parse;` resolves in both name spaces.
  - In 2018 and later, a glob module `std` with `use std::fmt` gives Ambiguous; so do `pub use serde;` and `crate::serde::..` with a glob module `serde`.
  - A glob of a value, an explicit import that shadows the crate, and a code path through a glob module are each not ambiguous.
  - Edition 2015 `use std;` in a module with Display loads. The old code refused it.
  - `use serde::Serialize` with a glob module `serde` gives `Unresolved { candidates: [demo::m::serde::Serialize, serde::Serialize] }`.
- Order dependence: see REQUIRED-1.

### Stack safety

Each fixture ran on a 2 MiB thread.

| Fixture shape | n | New code | Old code (d1c4a7f) |
|---|---|---|---|
| CA-F7's own shape: `use std;` + Display + `use std::collections::HashMap as H{i}` | 20,000 | loads in debug (2.6 s) and release | stack overflow from n=2,000 in release, and at 20,000 in debug |
| E2: chained named imports `m{i}: pub use super::m{i+1}::T`, ending at `serde::Serialize as T`, with `impl m0::T` | 8,000 | loads in release | stack overflow at n=2,000 in release |
| E8: a glob of a chain of single-segment aliases ending at `use serde as aN` (Frame::Glob → Named) | 4,000 debug, 8,000 release | loads | stack overflow at n=2,000 |
| E12: an impl through a single-segment alias chain | 8,000 | loads in release | stack overflow at n=2,000 |
| E9: 20,000 chained `super::*` globs | 20,000 | loads in debug and release | not run |
| E10: a resolve query over 20,000 imports that lead to nothing | 2,000 | NotFound; the `unbound` search uses an explicit stack | not run |
| E11: a resolve query over a 20,000-import cycle | 2,000 | Cyclic; the `unbound` search uses an explicit stack | not run |

The trait-key search (`absolute` with `Frame` and `Step`) and `unbound` have no recursion: no function calls itself, and the only mutual call, `next_glob` ↔ `next_target`, has depth at most 2.

CA-F7 asked to "make the canonicalisation iterative, or bound its recursion by a fixed, stack-safe constant". The search is now iterative, so that requirement is met.
- The plan's literal 20,000-alias fixture passes on d1c4a7f. Its value as a guard comes from the drain mutation.
- The chained-globs fixture falsifies the old recursion directly: it aborts on d1c4a7f.
- So does the CA-F7 shape, which I re-ran here.

### Held ≤ charged (spot-check)

| New data | Charge |
|---|---|
| `target_editions` | `vec_slot::<u16>()` is charged per target before each push |
| `named_imports` and `glob_imports` | each holds exactly one `usize` per `UseDecl`, built with exact capacity; `UseDecl::bytes` adds one `vec_slot::<usize>()` |
| `use_ambiguous` | covered by `3 * vec_slot::<bool>()` |
| `LOAD_BASE_BYTES` | adds the first allocation of each new Vec |

`hold` allocates nothing. The search's `Frame` stack and visited sets are transient and are not held in the index. The charges are consistent with what is held.

### Falsifiability

Spot-run on d1c4a7f, with the R5a test files copied into the old tree:

| Fixture | Result on d1c4a7f |
|---|---|
| `a_use_head_from_a_glob_that_is_also_an_extern_prelude_crate_is_ambiguous` | FAILED at `use_resolution.rs:1112` |
| `a_trait_through_an_ambiguous_use_head_keys_every_reading` | FAILED at `trait_keys.rs:1123` |
| `a_single_segment_import_of_a_workspace_library_binds_its_crate_root` | FAILED |
| `a_trait_found_through_twenty_thousand_chained_globs_is_keyed_on_a_two_mib_thread` | stack overflow abort |

All four match `raw/R5a-falsification.txt`.

Mutations:
- **The recorded mutation**, which drains the settle queue by recursion, makes `twenty_thousand_chained_local_aliases_load_on_a_two_mib_thread` overflow its stack in debug and in release. This reproduces the recorded result.
- **Extra mutation 1:** `is_unsettled` ignores `own` (`count > 0`). `a_use_head_from_a_glob_that_is_also_an_extern_prelude_crate_is_ambiguous` fails with `demo::alone::serde: NotFound`.
- **Extra mutation 2:** `glob_against_prelude` never reports ambiguity. The trait_keys suite fails.

### Registry

The harness's `--one` mode ran on the new build:

| Crate | Old error on d1c4a7f | Cause | New result |
|---|---|---|---|
| bincode 1.3.3 | `UnresolvedTrait serde::Deserializer` | edition 2015 `use serde;` in modules, through the root's `extern crate serde` | loads |
| core-foundation 0.10.1 | `UnresolvedTrait fmt::Debug` | `use std;` | loads |
| p256 0.13.2 | `UnresolvedTrait FieldBytesEncoding` | `pub use elliptic_curve::{self, ..}` | loads |
| js-sys 0.3.106 | (old error not recorded) | `pub use wasm_bindgen;` | loads |
| rmp-serde 1.3.1 | (old error not recorded) | `use serde;` resolved through its own binding | loads |

The remaining refusals are correct:
- **io-lifetimes:** the trait `FilelikeViewType` is not imported in `impls_async_std.rs` behind `cfg(feature="async-std")`, so rustc would reject that module.
- **petgraph:** the trait comes from `trait_template!`, and macros are not expanded.

### Entry points

| Workspace | Claim | Re-run result |
|---|---|---|
| this repository | `raf_source::syn` is NotFound | covered by the passing `this_repository` test |
| katamari | paths behind `use katamari_replication_testkit as support;` now resolve | `…::streaming_transfer::support::vault` resolves to `katamari_replication_testkit::vault`, and `…::barrier::support::Primary` resolves to `katamari_replication_testkit::Primary`; both were NotFound on the old code. `support::nope` is still NotFound. |
| purrdf | the `wasm_bindgen` paths are NotFound | `purrdf_sparql_eval::wasm_host::wasm_bindgen` and `purrdf_testkit::host::wasm_bindgen` give NotFound on the new code and Cyclic on the old |

## Findings

### REQUIRED

**REQUIRED-1: the new E0659 rule depends on declaration order for a workspace-library crate name.**

This is a first-wins in time, and it leaves a silent wrong `Ok` and a wrong `Canonical` trait key.

Cause: in `Index::start` (`resolve.rs`), a 2018+ `use` head that the module does not bind *yet* falls straight through to `extern_roots`.
- For a workspace library, or a root `extern crate` of one, that succeeds at once.
- The import then binds through the crate and is settled before a glob of the same module has brought the name in.
- `glob_against_prelude` is therefore never consulted.
- An outside dependency is not affected: `extern_roots` fails there, so the import waits and is woken.

Reproduced with the workspace `shared` (lib: `pub fn x(){}` and `pub trait T`) and `demo` (2021, depending on `shared`):

| Declaration order | Result |
|---|---|
| `pub mod k { pub mod shared { pub fn x(){} } } use k::*; pub use shared::x;` | `demo::x` is `Ambiguous([..])`, which is correct |
| `pub use shared::x; pub mod k { pub mod shared {..} } use k::*;` | `demo::x` is `Ok shared::x`. Wrong: rustc gives E0659. |
| Trait key: `use k::*; use shared::T; impl T for String` | `Unresolved { candidates: [demo::k::shared::T, shared::T] }` |
| Trait key: `use shared::T; impl T for String; …; use k::*;` | `Canonical("shared::T")`. Wrong. |

This contradicts:
- the R5a contract bullet that glob-against-crate is `Ambiguous`;
- the new Contract text in `source-index.md` ("a first segment of a `use` path that a glob of the module brings into the type name space and that is also a crate of the extern prelude … is ambiguous");
- the existing order-independence property (`resolution_does_not_depend_on_the_order_of_the_declarations`).

The old code was already order-dependent for this shape: order A resolved through the glob and order B through the crate. R5a introduces the rule, so it has to hold in every order.

Required fix and checks:
- A 2018+ `use` head must not fall back to the extern prelude while a glob of its module could still bind the name. It must wait on the module's globs, so that the decision is taken against final glob tables.
- Add fixtures for both declaration orders: a workspace library, and a root `extern crate` of a workspace library. Each needs a resolve assertion and a trait-key assertion.

### ADVISORY

**ADVISORY-1: the trait-key cycle rule refuses valid code.**

The rule ("an explicit import met again while its own path is being made absolute is a cycle, which is an error") refuses rustc-valid code in which a glob back-edge reaches the import in progress.

Example: `mod inner { use super::*; pub use serde::*; } pub use inner::Serialize; impl Serialize for &Foo {}` compiles with rustc. Both the old and the new code refuse it with `UnresolvedTrait Serialize`.

A re-met named import could instead give nothing to that branch, as a re-met glob already does (`search.globs`). A pure cycle would still end in 0 candidates and `UnresolvedTrait`. The behaviour is pre-existing, not seen in the registry, and the refusal is typed.

**ADVISORY-2: settling chains of non-binding imports is quadratic.**

Chains of imports that bind nothing settle in quadratic time, one stuck "leaf" per round:

| Fixture | n | New time (release) | Old time (release) |
|---|---|---|---|
| E2 | 4,000 | 7.8–19 s | 109 s |
| E2 | 8,000 | 35 s | |
| E8 and E12 | 4,000 | | over 120 s |

At 20,000 they exceed 5 minutes. The new code is faster than the old, and the cost is pre-existing. R5b's blocker index (P-F6) and its 32k-import probe are where this belongs; R5b should include an E2/E8-shaped probe.

**ADVISORY-3: three edge cases disagree with rustc.**

| Case | rustc | New code |
|---|---|---|
| A glob brings in `pub use std as serde`, then `use serde::X` | E0659 | NotFound, because the re-export binds nothing in the index |
| `#![no_std]` with a glob module `std`, then `use std::fmt` | OK | Ambiguous, because `SYSROOT_PRELUDE` always holds `std` |
| Edition 2015 `use serde::Serialize;` without `extern crate`, or with `extern crate` only in a submodule | E0432 | the trait key is still keyed `serde::Serialize` (pre-existing) |

These are rare. The Contract wording "`std` and `core`, which every crate names without an `extern crate` item" is inaccurate for `no_std` crates.

**ADVISORY-4: first-wins patterns remain, but neither is new behaviour.**
- `enter_prefixes` keeps the pre-existing `.find()` over `extern crate` defs. It is practically unreachable, because bound `extern crate` items are handled for every candidate in `enter_absolute`.
- `trait_key` keeps the pre-existing `sorted(..).first()` over walked items.

**ADVISORY-5: one validation.md claim is worded wrongly.**

validation.md describes the purrdf paths as "`use wasm_bindgen;`". The source shape is `use wasm_bindgen::prelude::wasm_bindgen;`, which binds the name of its own first segment. The behaviour claim, Cyclic → NotFound, is confirmed.

**ADVISORY-6: the CA-F7 evidence shape is not a fixture.**

The completion audit's own CA-F7 fixture (`use std;` + 20,000 `use std::collections::HashMap as H{i}`) aborts on d1c4a7f and loads on R5a, but it is not committed as a fixture. Adding it would tie the CA-F7 closure to the auditor's own evidence.

## Verdict

Gates, stack safety, the charges, falsifiability, registry root causes and the entry-point claims all pass. REQUIRED-1 blocks: the E0659 rule that R5a adds, and documents in the Contract, is decided by declaration order for workspace-library crate names. In one order the code gives a wrong `Ok` resolution and a wrong `Canonical` trait key where rustc refuses the code.

First-round verdict: BLOCKED (REQUIRED-1). See the Recheck below for the current status.

## Recheck

**Scope:** the R5a fix rounds, uncommitted on d1c4a7f. They cover:
- two-phase settling (`crates_open`, `unbound_use_head`);
- the verification pass (`Index::settle_agreed`, `walks_ambiguous`);
- a glob-carried whole-crate alias collides (`globbed_outside`);
- `std` leaves the use prelude under an unconditional `#![no_std]`;
- the 2015 crate-root reading through an `extern crate` item;
- a named import met again gives nothing to that way;
- several trait paths give `Unresolved`;
- CA-F7's own shape as a fixture;
- the Contract and SECURITY text.

**Tools:**
- the `probe` and `disk` binaries were rebuilt against the revised tree;
- `probe-cnt` is the same probe over a copy of the revised tree (`/tmp/r5a-review/mut2`) that prints a line per `settle_agreed` iteration;
- `perm.py` loads every declaration order, normalises item ids, line numbers and error text, and counts the distinct outcomes;
- `fuzz.py` generates random programs over workspace libraries `shared` and `other` and a module `k`. It keeps only programs whose explicit imports bind distinct names, runs up to 12 orders of each, and reports the iteration count.

### Gates (revised tree)

Every gate exits 0:
- `make fmt`;
- `cargo test -p raf-source --locked` in debug and `--release`: 708 tests pass and 0 fail across the two (354 per profile);
- `make clippy`;
- `make doc`;
- `docs_contract` + `repo_gates`;
- `make cross-check`, all three targets;
- `make msrv`, on 1.98.1 with the fuzz crate;
- `make reuse`: 224 of 224 files compliant.

### REQUIRED-1 (glob against crate, by order): fixed

All the shapes below give the same outcome in every declaration order, and that outcome matches rustc:

| Shape | Orders | Outcome in every order |
|---|---|---|
| The original pair: `pub mod k { pub mod shared {..} }`, `use k::*;`, `pub use shared::x;`, `use shared::T;` and the impl | 120 | `demo::x` is `Ambiguous`; the key is `Unresolved {demo::k::shared::T, shared::T}` |
| Root `extern crate shared as sh;` with the glob and the import in submodule `m` (rustc E0659) | 6 | `demo::m::x` is Ambiguous; the key is `Unresolved {demo::k::sh::T, shared::T}` |
| Root `extern crate shared as sh;` with the glob and the import at the root | | the `extern crate` item shadows the glob: `Ok shared::x`, which matches rustc (probed) |
| A chain of aliases into a glob: `use shared as s1; use s1::inner as s2; use s2::*; pub use other::x; use other::T` with the impl | 120 | Ambiguous; the key is `Unresolved {other::T, shared::inner::other::T}` |
| A glob carried across modules: `a: pub use shared::inner::*`, `b: use super::a::*; pub use other::x` | | Ambiguous in both orders |
| The same, but through `self::other` and a code path | | `Ok`, `Canonical` (in code the glob wins) |

The verification pass mixed with trait keys is covered by the chain-of-aliases shape and the fixture shape.

### NEW REQUIRED-2: an explicit import that shadows a workspace-library crate is decided by declaration order

This is a first-wins in time, and it gives a silent wrong `Ok` and a wrong `Canonical` on rustc-valid code.

Cause: `Index::lookup_in` (`settle.rs:352`) returns `None` in two different cases:
- the module has no entry for the name;
- an explicit import that binds the name is still unsettled.

`Index::start` (`resolve.rs`, the `_` arm) cannot tell these apart. Unless `unbound_use_head` makes it wait, it goes straight to `extern_roots`:
- In a module **without globs**, `unbound_use_head` never waits. In any phase, a head that names a library of the workspace (or a root `extern crate` of one) starts at that crate even though an explicit import of that name is still unsettled.
- In a module with globs, the same happens once `crates_open` is set.

An outside crate is unaffected: `extern_roots` fails there, so the path waits.

The verification pass does not catch this, because it only looks for ambiguity.

rustc probe: `pub use a::Deserialize as D; use a::Deserialize; pub use serde::de as a; impl Deserialize for S {}` compiles. The import shadows the crate whatever the order. Tool fixture (workspace `shared` and `other`, demo in 2021, **no globs**):

```
pub use other::x;
pub use shared::inner::other;
use other::T;
impl T for String { fn m(&self) {} }
```

Over its 24 orders there are 4 distinct outcomes:

| Orders | `demo::x` | Key |
|---|---|---|
| 4 | `Ok other::x` (wrong) | `Canonical("shared::inner::other::T")` |
| 8 | `Ok other::x` (wrong) | `Canonical("other::T")` (wrong) |
| 8 | `Ok shared::inner::other::x` | `Canonical("shared::inner::other::T")` |
| 4 | `Ok shared::inner::other::x` | `Canonical("other::T")` (wrong) |

The correct outcome in every order is `shared::inner::other::x` and `shared::inner::other::T`.

The old code (d1c4a7f) gives the same order-dependent result, so the defect predates R5a. It still blocks R5a, for three reasons:
- the plan's revised bullet and the new Contract text say "a local item or explicit import shadows an extern-prelude crate";
- the new Contract text also says "the decision does not depend on the order of the declarations";
- the round touched exactly this decision.

The fuzzer found the same defect in 11 of about 900 rustc-plausible programs, for example `['pub use shared::inner::other;', 'pub use other::x;', 'use shared::T;']`.

Required fix and checks:
- When an explicit import binding the head is unsettled, `start` must wait (`Fail::Missing` on that name) and not fall back to the crate. That means telling "unsettled" from "unbound" in `lookup_in`.
- Add a fixture over every order of the shape above, asserting the resolve result and the trait key. Use it in a module without globs and in a module with globs.

### settle_agreed: termination and cost

- **Termination:** the held set only grows, and each loop either returns or adds at least one held import. So it ends within `uses.len() + 1` settles.
- **Iterations observed:**
  - about 900 fuzz programs × up to 12 orders: at most 2 iterations;
  - 3,000 independent copies of the fixture shape (`pub mod m{i} { pub use other::x; use shared::inner as inn; use inn::*; }`): 2 iterations, 140 ms in release;
  - an adversarial chain built to cascade, 200 modules `m{i}: use super::m{i-1}::y as inn; use inn::*; pub use other::inner as y;` (both libraries hold `inner::other` re-exporting `inner`): 2 iterations, 17 ms. The holds that cascade are found inside one settle, because an import whose name is held waits until the stuck loop, after the globs are settled.
- I found no input that needs more than 2 settles.
- No budget charges the number of settles. In the worst case the work is `uses.len()` times one settle, and one settle is still quadratic on chains of imports that bind nothing (ADVISORY-2, R5b). This stays ADVISORY: a fixture or an argument that bounds the number of iterations would close it.

### Advisories (first round) rechecked against rustc

| # | Probe | Result |
|---|---|---|
| 1 | `mod inner { use super::*; pub use serde::*; } pub use inner::Serialize; impl Serialize for &Foo` | **Resolved.** Now loads, keyed `Canonical("serde::Serialize")`, which matches rustc. A pure cycle (`mod inner { pub use super::Serialize; } pub use inner::Serialize;`) is still a typed `UnresolvedTrait`. |
| 3a | A glob-carried whole-crate alias: `mod p { pub use serde as other; } use p::*; pub use other::x;` (rustc E0659) | **Resolved.** Ambiguous in all 6 orders. |
| 3a | The same with `pub use other;`, the same crate (rustc OK) | **Resolved.** `Ok other::x` in all 6 orders. |
| 3b | `#![no_std]` with a glob module `std` and `use std::fmt` (rustc OK) | **Resolved.** `Ok demo::m::std::fmt::f`. |
| 3b | `#![cfg_attr(not(feature="std"), no_std)]` | Ambiguous, as the Contract says: `std` stays. |
| 3c | 2015 `use serde::Serialize;` without `extern crate`, and 2015 `extern crate serde;` only in a submodule (rustc E0432 for both) | **Resolved.** Both now refused with `UnresolvedTrait`. |
| 3c | 2015 `use std::fmt::Display` in a submodule (rustc OK, through the injected `std`) | **Resolved.** Loads. |
| 4 | | **Resolved.** Several trait paths give `Unresolved`. The `cfg` alternatives at one path still key a single path, which is correct. |
| 6 | | **Resolved.** CA-F7's shape is committed (`trait_keys.rs:1314`). In this recheck it loads at n=20,000 in debug on a 2 MiB thread in 1.1 s. |
| 5 | | Not rechecked: the coordinator's message did not cover the validation.md wording. |

### Macro false-positive fix: it misses a real E0659, but the Contract documents that

The new rule is that only a glob-carried alias of a *whole other crate* collides.

- **Valid code no longer refused:** `use quote::quote; mod m { use super::*; pub use quote::ToTokens; }` loads, and rustc accepts it (probed with a stand-in `quote` rlib).
- **What it misses:** a glob that carries a re-export of a *type-namespace item* of an outside crate, under the name of another crate, is E0659 in rustc but `Ok` in the tool.

| Glob carries | Then | rustc | Tool |
|---|---|---|---|
| `pub use serde::de as a` | `use a::f` | E0659 | |
| `pub use serde::Serialize as a` | `use a::f` | E0659 | |
| `pub use a::mac as a` (a macro) | `use a::f` | OK | |
| `pub use a::f as a` (a fn) | `use a::f` | OK | |
| `mod p { pub use serde::de as other; }` | `use p::*; pub use other::x;` | E0659 | `Ok other::x` in all 6 orders |

The index cannot know the name space of an item outside the workspace. The Contract states this limit in words: "A glob that brings in an item of a crate outside the workspace under that name does not [collide]: the index cannot tell the item's name space". That makes it a documented trade-off: the case is rustc-invalid, and flagging it would refuse 8 valid registry crates. **ADVISORY-7:** list this missed E0659 under the Contract's known limits next to the `quote` example, so a reader does not take the omission for a guarantee.

### Other rechecks

**Stack safety (revised tree, 2 MiB thread):** CA-F7's shape at n=20,000 loads in debug (1.1 s). The rest load in release:

| Shape | n | Time |
|---|---|---|
| chained `super::*` globs | 20,000 | 0.3 s |
| E8 | 4,000 | 8.1 s |
| E2 | 4,000 | 8.1 s |

**Registry spot-check (`registry_scan --one`, revised):** bincode, core-foundation, p256 and js-sys all load. Their item byte counts rose by 6 to 20 bytes, which is consistent with the new charges.

**Resolve dumps against d1c4a7f:** every item path was resolved in its own name space, with its trait key, using the old and the new `disk` binaries over `crates/`. Item ids were normalised.

| Workspace | Lines | Differences |
|---|---|---|
| this repository | 2,390 | 0 |
| purrdf | 83,653 | 0 |
| katamari | 13,995 | 0 |

So no item resolution and no trait key changed. Alias paths, targeted:

| Path | Result |
|---|---|
| katamari `…::streaming_transfer::support::vault` | `Ok katamari_replication_testkit::vault` |
| katamari `…::barrier::support::Primary` | `Ok katamari_replication_testkit::Primary` (both were NotFound on d1c4a7f) |
| purrdf `wasm_host::wasm_bindgen`, `host::wasm_bindgen` | NotFound (Cyclic on d1c4a7f) |
| `purrdf` | `Ambiguous(2)` |
| `raf_source::syn`, `raf_source::syn::File` | NotFound |
| `raf_source::DiskTree` | Ok |

**Minor: order-dependent error text (ADVISORY-8).** When several unresolvable globs could provide a trait, the glob that the `UnresolvedTrait` error names depends on declaration order. For example, the trait `sh::*` versus `inn::inner::*` in `['use sh::T;', 'use shared;', 'pub use inn::inner::*;', 'use other;', 'use sh::*;']`. The error is typed and the kind is stable; only the message varies.

### Recheck verdict

REQUIRED-1 is fixed and stays fixed under 120-order permutations, the extern-crate variants and fuzzing. A new defect blocks R5a: REQUIRED-2, an explicit import that shadows a workspace-library crate is decided by declaration order. That breaks the shadowing rule and the order-independence the revised Contract states, and on rustc-valid code it gives a wrong `Ok` and a wrong `Canonical` trait key. ADVISORY-1, 3a, 3b, 3c, 4 and 6 are resolved. ADVISORY-2 and the settle-count bound go to R5b. ADVISORY-7 and 8 are documentation and message nits.

Recheck verdict: BLOCKED (REQUIRED-2). See Recheck 2 for the current status.

## Recheck 2

**Scope:** the REQUIRED-2 fix (uncommitted). It has two parts:
- a head that is unbound only because an explicit import of that name in its module is unsettled now waits on that import, for `use` and code paths, with and without globs;
- the verification pass records the `use` paths that read a head as a crate, and rebuilds the tables when an explicit import binds that name once the tables are settled.

It also adds two new fixtures, the ADVISORY-7 limit in the Contract, and the ADVISORY-8 change: unresolvable globs no longer stop the trait-key search.

### Gates (revised tree)

Every gate exits 0:
- `make fmt`;
- `cargo test -p raf-source --locked` in debug and `--release`: 714 tests pass and 0 fail across the two (357 per profile);
- `make clippy`;
- `make doc`;
- `docs_contract` + `repo_gates`: 139 and 55 passed;
- `make cross-check`;
- `make msrv`;
- `make reuse`: 224 of 224 files compliant.

### Order-dependence search

**Search set-up (`fuzz2.py`):**
- 1,440 programs: 8 seeds × 120, then 8 seeds × 60;
- up to 12 shuffled declaration orders per program, run with a probe that counts `settle_agreed` iterations;
- workspace libraries `shared` and `other`, plus a local `mod k` holding modules `other` and `inn::inner::other`.

**Statement templates:**
- explicit shadowing imports (`pub use X::inner::other;`, `pub use crate::k::other as N;`, `pub use self::k::{other,inn};`);
- aliases (`use X::inner as N;`, `pub use X as N;`, `use shared;`);
- globs (`use X::*;`, `use self::k::*;`, `pub use X::inner::*;`);
- `extern crate shared|other as N;`;
- trait imports and code-path trait impls (`impl N::T for u8`);
- a submodule that globs its parent and imports through a head.

**Filter and measures:** programs whose explicit imports bind the same name twice are dropped (rustc E0252). Item ids, `chain=` and line numbers are normalised, so line numbers alone do not count as a difference.

**Results:** 5 programs gave more than one outcome. Every one of them is classified below:

| Programs | Shape | Class |
|---|---|---|
| 3 | **Several impls whose trait cannot be keyed.** The load fails with `UnresolvedTrait`, which names the impl that comes first in the file (`sh::T` versus `inn::T`). | Neither (a) nor (b): an error that names the first failing impl by its position in the file. In every order, every impl is refused and the error kind is the same. |
| 2 | **A cycle of explicit imports:** `use shared::inner as inn; use inn::inner as shared;` with `use self::k::*`, and `pub use inn::inner::other; pub use other as inn;`. Which import of the cycle settles as failed first decides whether the glob answers (`demo::inn` is `demo::k::inn` in some orders and `shared::inner` in others). | **(a) R5b's cycle scope.** rustc refuses both programs ("cannot determine resolution for the import", E0432). Dropping one edge of the cycle makes each program agree over 200 orders. R5b's check ("both declaration orders, a fixture that reaches the cycle branch") covers this. |

**No (b) case, that is no R5a order dependence, was found.**
- Explicit import against crate, with and without a glob, in a module and at the root: stable in every program.
- Glob against crate: stable.
- A root `extern crate` alias: stable.
- The verification-pass shapes mixed with trait impls: stable.

**settle_agreed iterations:** at most 3 over all 1,440 programs, with 3 seen once and 2 the usual count. The held set grows monotonically, so the loop terminates.

### ADVISORY-8 residual

**Acceptable.** When the search stops first on a path that fails for another reason, the set of globs it collected can depend on order. The result is still a load error: the same impl and the same error kind (`UnresolvedTrait`) in every order. Only the glob the message names changes.

- The 6-order fixture pins the common case.
- No program in the search succeeded in one order and failed in another.
- No program changed the error kind or the impl named.

This is message content only and does not block.

### Resolve dumps against d1c4a7f

Every item path was resolved in its own name space, with its trait key, using the d1c4a7f and the revised `disk` binaries over `crates/`. Item ids were normalised.

| Workspace | Lines | Differences |
|---|---|---|
| this repository | 2,396 | 0 |
| purrdf | 83,653 | 0 |
| katamari | 13,995 | 0 |

### Recheck 2 verdict

REQUIRED-2 is fixed. With 1,440 random programs, 12 declaration orders each, and permutation probes, the only order dependence left is:
- inside cycles of explicit imports, which rustc refuses and which belong to R5b's scope;
- in which failing impl or glob an error message names.

REQUIRED-1 stays fixed, the gates are green and the dumps are unchanged. Advisories 2 (settle cost, R5b), 7 (documented) and 8 (message only) do not block.

VERDICT: PASS
