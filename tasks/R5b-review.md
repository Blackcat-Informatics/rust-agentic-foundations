# R5b review: settle import cycles as components

## Scope

I reviewed the uncommitted R5b diff on top of e472918 in the worktree `2-shared-crates-raf-okf-raf-source`:
- `settle.rs`, `resolve.rs`, `index.rs`, `impls.rs`, `workspace.rs`;
- `tests/use_resolution.rs`, `tests/trait_keys.rs`;
- `docs/capabilities/source-index.md`.

The diff hash was `6be296a1…` at the start and the end of the review. The worktree was not edited.

I checked it against:
- the plan's "### R5b" section, the Mutation rule, the Per-item gates and the Registry harness;
- the R5a review's open items for R5b;
- the orchestrator's decision at the end of `validation.md`: keep the conservative block, and report the hidden case as `ShadowedGlob`;
- AGENTS.md and CLAUDE.md.

**Tools:** everything below is in `/tmp/r5b-review`.
- `probe-new` and `probe-old` are debug and release probes built against the R5b tree and against `git archive e472918`. The debug builds carry R5b's debug assertions.
- `rc.sh` is a rustc 1.100-nightly harness.
- `fuzz_rc.py` is a rustc-oracle fuzzer. It compiles each random program together with one assertion per module, name and name space. Each assertion pins the tool's answer to an item identity.
- `order.py` is the order search.
- `perf-*` and `alloc` are the perf and retention harnesses.
- `repro/` holds the minimal programs behind each finding.

## Evidence

### Gates (R5b tree)

Every gate exits 0:
- `cargo test -p raf-source --locked`: 365 passed, 0 failed;
- the same with `--release`: 365 passed, 0 failed;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `RUSTDOCFLAGS=-Dwarnings cargo doc -p raf-source --no-deps`;
- `docs_contract` + `repo_gates`: 139 and 55 passed;
- `make fmt`;
- `make cross-check`;
- `make msrv` (1.98.1);
- `make reuse`: 224 of 224 files compliant;
- `make fuzz`: 5,000 runs per target. It was re-run without my `ulimit -v`, because ASan cannot reserve its shadow memory under that limit.

There is no process wording and no `unsafe` in the diff, and `.deficiencies` holds only its marker.

### (1) The cycle rule against rustc

| Shape | rustc | R5b | e472918 |
|---|---|---|---|
| Two-import cycle `a::X ↔ b::X` | E0432 | both `Cyclic` | `Cyclic` |
| A4 (each module globs a different `X`) | cannot determine, E0432 | both `Cyclic` | `Ok demo::c::X` (wrong) |
| A4, both globs of the same `c` (V3) | cannot determine | `Cyclic` | `Ok` (wrong) |
| `pub use self::a as b; pub use self::b as a;` | E0432 | `Cyclic` | `Cyclic` |
| 3-cycle across modules plus a waiter `w` | E0432 | all four `Cyclic` | `Cyclic` |
| 3-cycle, globs in two of its modules (V6) | cannot determine | `Cyclic` | `Ok` (wrong) |
| Self-cycle `A ↔ B` in one module with a glob (V8) | cannot determine | `Cyclic` | `Ok` (wrong) |
| **2-cycle `e::A ↔ d::A`, only `d` globs `b` (V1)** | **OK: both are `b::A`** | `Cyclic` | `Ok b::A` (right) |
| **3-cycle, glob in one module only (V5)** | **OK** | `Cyclic` | `Ok` (right) |
| **V1 with a waiter `w::W` (V7), V1 with the glob in the other module (V9)** | **OK** | `Cyclic` | `Ok` (right) |
| A cycle beside a definition in the cycle's module (E0255 shapes) | E0255 | unchanged from e472918 | |

Every rustc-refused cycle is now `Cyclic` for every name in it and for its waiters, and I found no wrong `Ok` from a cycle. The cycles that rustc *accepts*, those with exactly one glob fallback on the cycle, are now refused: see REQUIRED-2.

The plan's A4 fixture holds in all 24 orders, and so does its neighbour without the `b → a` import, which gives `demo::c::X`.

### rustc-oracle fuzz

`fuzz_rc.py` ran 12,000 random programs over modules `a`–`e` and names `A`/`B`, mixing structs, functions, imports, renames and globs. Of these, 5,483 compile cleanly ("clean" in the table), and their probes give rustc's exact answer.

| Finding | R5b | e472918 |
|---|---|---|
| Clean programs: a rustc answer that the tool refuses with `Cyclic` | 124 queries | 0 |
| Clean programs: a rustc answer that the tool refuses with `NotFound` | 93 queries (79 of them in 29 programs without a cycle) | 0 |
| Clean programs: `ShadowedGlob` where rustc answers (the documented limit) | 212 | n/a |
| Clean programs: `ShadowedGlob` where rustc also finds nothing | **0** | n/a |
| Programs rustc refuses: an `Ok` for a name only a rejected import binds | 12 (all E0255/E0252 conflict programs) | 696 |
| Programs rustc refuses: a definite wrong `Ok` (the probe line fails) | 11 | 14 |

R5b removes most of the wrong answers on cycles. Every refusal of clean code in the first two rows is new in R5b, and each one is a case of REQUIRED-1 or REQUIRED-2.

### (2) Order search (my generators)

Each program was loaded in 12 shuffled orders, of its modules and of the statements inside them, with the debug probe.

| Set | Programs | Order-dependent | Debug-assertion panics |
|---|---|---|---|
| local (structs, functions, imports, renames, globs) | 5,000 | 2 | 0 |
| ws (crates `shared` and `other`, a local `k::other`, value-only imports named like the crates, aliases, impls) | 5,000 | 12 | 11 |

- **The 2 local programs:** only `via_glob` differs, in rustc-invalid programs with duplicate explicit imports. See ADVISORY-1.
- **The 12 ws programs:**
  - 11 contain a `check_settled` panic (REQUIRED-3), and the panic appears in some of their orders only;
  - 1 differs only in which glob the `UnresolvedTrait` message names, the known message-only class.
- **e472918 on 4,000 of the same programs:** 30 order-dependent programs, every one with a real difference in the result.

R5b fixes the order dependence of the R5a-review cycles.

### (3) The "at most three rounds" claim

The claim is sound as far as I can establish:
- Walks read the scope tables and `crates_open`, and never `use_settled` or `use_ambiguous`.
- Holding or failing an import binds nothing, so the drains that follow re-walk against unchanged tables.
- So:
  - every ambiguity found when the queue is empty is held in one pass;
  - every non-cyclic component fails in the next round;
  - the third round finds nothing left.

No program among my roughly 22,000 debug-mode programs, each loaded in up to 12 orders, fired the `rounds <= 3` assertion. The same holds for the `settles <= named + 1` assertion and the "resolves once the queue is empty" assertion.

Both bounds are `debug_assert!` only. In release builds, a violation would go undetected, but the loop still ends, because each round settles or re-queues at least one import. The one untracked dependency I found is REQUIRED-3. It does not add rounds, but it leaves the tables inconsistent.

### (4) ShadowedGlob

**What holds:**
- The trigger needs an explicit import that resolved, the name space empty in its module, and a glob of the module, or of a module that globs into it, that is visible and holds the name.
- I found no case where rustc also finds nothing (0 in the fuzz).
- An import of an outside item, or one that failed, stays `NotFound`, as the Contract says.
- `resolve()` goes on to the next name space and returns `ShadowedGlob` only when none answers. I confirmed this with the purrdf shape (`observed::m` resolves to the function).

**What leaks:** the block is not confined to the module that holds the import. See REQUIRED-1.

### (5) purrdf `xsd::ops::tests::Decimal`, Ok → NotFound

`tests` has `use super::*;`, which brings in `numeric::Decimal`, and `use crate::XsdDatatype::{.., Decimal, ..}`, which imports an enum variant. A variant lives in the type and the value name space, so the explicit import shadows the glob in both.

rustc confirms this on a reduction: `pub type T = Decimal;` in that module gives E0573, "expected type, found variant". The old `Ok purrdf_xsd::numeric::Decimal` was wrong. The new `NotFound` is right, because variants are not indexed. The metaschemas change to `ShadowedGlob` is the documented limit.

### (6) Performance and memory

| Probe | Re-run (release) | Implementer's figure |
|---|---|---|
| `spread` 32k imports | 314–321 ms | 340 ms |
| 1,600 2-cycles | 50 ms | 49 ms |
| 20,000-link chain ending outside | 375 ms | 389 ms |
| E2 / E8 / E12 at n=8,000 | 142 / 77 / 75 ms | |
| E2 / E8 / E12 at n=20,000 | 390 / 199 / 193 ms | |
| E2 at n=8,000 on e472918 | 28.2 s | |

**Held ≤ charged:** I rebuilt the index-retention harness on the current tree with a probe-only `keep_index`. All eight shapes give `index <= item_bytes`, with byte counts identical to `/tmp/r5b2/alloc-idx-new.txt`.

### (7) Registry

`raw/registry-scan-R5a.txt` against `raw/registry-scan-R5b.txt` gives:
- 0 outcome changes;
- 0 message changes among the failures;
- 0 budget refusals;
- two crates missing from the R5b scan (`reed-solomon-erasure` and `sia_reed_solomon`), which were removed from the local registry.

The shapes behind the REQUIRED findings do not occur in the scanned registry.

### (8) Falsifiability

**Fixtures on e472918.** I extracted e472918 into `/tmp/r5b-review/oldfals` and copied in the R5b test files, removing the two `ShadowedGlob` tests, which cannot compile there. Each fixture below fails as recorded in `raw/R5b-falsification.txt`:

| Fixture | Result on e472918 |
|---|---|
| `a_cycle_of_explicit_imports_is_cyclic_whatever_its_globs_hold_in_every_order` | `Ok demo::c::X` where `Cyclic` was expected |
| `a_cycle_of_imports_through_a_crate_name_and_a_glob_is_cyclic_in_every_order` | `Ok("demo::k::inn")` where `Cyclic` was expected |
| `an_import_binds_what_a_name_it_read_gains_later_in_every_order` | `NotFound(demo::x)` |
| `trait_keys::an_explicit_import_of_a_dependency_trait_shadows_a_glob_of_a_local_one` | panics at `trait_keys.rs:26` |

**My mutations**, each on a copy of the R5b tree:
- **M-a:** `taken` ignores the import's own exemption. Two tests fail: `a_single_segment_import_of_a_workspace_library_binds_its_crate_root` and `a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name`.
- **M-b:** `shadowed_glob` drops its "an import resolved" condition. Three tests fail: `an_explicit_import_shadows_the_glob_whether_or_not_the_index_sees_what_it_binds`, `a_glob_kept_out_by_an_import_in_another_name_space_is_reported_as_shadowed` and `a_cycle_of_explicit_imports_is_cyclic_whatever_its_globs_hold_in_every_order`.

## Findings

### REQUIRED

**REQUIRED-1: the name-space-blind block leaks out of the module that holds the import, and gives wrong answers.**

`visible()` skips a taken name entirely for every module that globs the importer. So:
- the glob candidates that rustc sees there vanish;
- other routes then answer alone, or nothing answers.

e472918 got every case below right. The programs are in `repro/`.

| Case | Program | rustc | R5b | e472918 |
|---|---|---|---|---|
| (a) | `m { use x::*; use y::A; }` (struct `y::A`, function `x::A`), `n { use m::*; use z::*; }` (function `z::A`) | `n::A` value is E0659 | **`Ok demo::z::A`** | `Ambiguous` |
| (b) | The same with traits, and `impl n::T for String` | E0659 | **`Canonical("demo::z::T")`** | `Unresolved {x::T, z::T}` |
| (c) | `p { use vals::shared; use k::*; }` (a function `shared` and a module `k::shared`), `n { use p::*; pub use shared::x; }` | E0659 | **`Ok shared::x`** (the crate) | `Ambiguous` |
| (d) | `a { use b::B; use c::*; }` (function `b::B`, struct `c::B`), `e { pub use a::B; }`, `e::B` in the type name space | `c::B` | `NotFound` | `Ok c::B` |
| (e) | `m { use vals::shared; pub use shared::x; }`, a value-only import named like a crate | `shared::x` | `NotFound` | `Ok shared::x` |

- (a) to (c) are wrong answers on code rustc refuses. (b) is a wrong canonical trait key.
- (d) and (e) give `NotFound` where the orchestrator decision requires the typed error, "NotFound would falsely claim the workspace defines nothing there". The Contract's limit paragraph covers neither.

**Required:**
- a module that globs, or imports through, a name kept out in a name space must not let another glob or a crate answer for it there, and must report `ShadowedGlob` (or `Ambiguous`) rather than `NotFound`;
- a use head in a module whose explicit import binds the name only in other name spaces must not report `NotFound`;
- add fixtures for (a) to (e), with a resolve assertion and a trait-key assertion where one applies;
- state the remaining limit in the Contract.

**REQUIRED-2: cycles that rustc resolves through a single glob are now refused, and so is a valid crate that names a trait through one.**

rustc resolves a single import while ignoring the import itself (`ignore_import`). In a cycle of explicit imports where exactly one module has a glob that binds the name, the import that reads that module therefore resolves through the glob, and the rest of the cycle follows. With two or more such globs, every member stays undetermined, as in A4.

- V1, V5, V7 and V9 compile with rustc.
- e472918 resolved them correctly. R5b gives `Cyclic`.
- `repro/B1-one-glob-cycle.rs`, with `impl e::T for String`, compiles with rustc and loads on e472918 with key `demo::b::T`. **R5b refuses the whole load with `UnresolvedTrait e::T`.**
- The fuzz shows 124 such `Cyclic` refusals of clean programs, and 14 `NotFound` refusals of readers of those cycles.
- This contradicts:
  - the Contract's "as in rustc: the globs of a module never answer for a name that an explicit import of the module binds, whether the import … is part of a cycle";
  - the plan's wording, "a component whose members are bound only by imports inside it is cyclic": V1's component has an outside binding.

**Required:**
- decide the component the way rustc does: one fallback glob on the cycle resolves it, two or more do not;
- fixtures for V1, V5 and V7, with the trait-key case, beside A4 and V3/V6/V8, which stay `Cyclic`;
- correct the Contract sentence.

**REQUIRED-3: the R5b debug re-walk (`check_settled`) panics on a three-line program that rustc accepts, and in release the tables stay inconsistent.**

`repro/C1-check-settled-panic.rs`:

```
pub mod vals { pub fn other() {} }
pub mod a { pub use other as inn; pub use crate::e::*; }
pub mod e { pub use crate::vals::other; }
```

(a workspace with a library `other`; rustc accepts `crate::a::inn::x`)

- **Debug, all 6 orders:** panics at `settle.rs:894`: "the imports of `inn` bound {} and reach {ItemId(16)} in the settled tables".
- **Release:** `demo::a::inn::x` is `NotFound`, where rustc resolves `other::x`.
- **Order search:** the same panic appeared in 11 of 5,000 ws programs.

Cause:
- In the first phase (`crates_open` false, the module has globs), the import binds the glob's value `other` in the value name space and is settled.
- The type-name-space walk waited on the crate. When `crates_open` is set, only the *unsettled* imports and the globs are queued again, so this import is never re-walked, and its crate reading is never bound.

This breaks the plan's "Resolver checkability" bullet and the settle doc ("a resolved import binds what its path reaches once the tables are settled"). It also crashes any debug-build consumer. The release result is pre-existing (e472918 gives the same `NotFound`), but the invariant that R5b adds and documents does not hold.

**Required:** re-queue the named imports whose walks waited on the extern prelude, or every named import, once the crates open, and add this program as a fixture in debug and release.

### ADVISORY

- **ADVISORY-1:** `via_glob` alone differs between orders in 2 of 5,000 local programs. Both are rustc-invalid, with duplicate explicit imports of one name read as `cfg` alternatives. A `cfg`-alternatives variant that I built by hand was stable over 120 orders.
- **ADVISORY-2:** the `ShadowedGlob` message says that rustc would read the glob. Where the explicit import also reaches an item outside the index through a `cfg` twin, it may not. The error is still a refusal.
- **ADVISORY-3:** the round and settle bounds are debug-only assertions (see (3)). This is acceptable, because both loops end in release.
- **ADVISORY-4:** the 11 remaining definite wrong `Ok`s in fuzz programs that rustc refuses are all programs with E0255/E0252 conflicts, where rustc recovers with dummy bindings. e472918 had 14, so this is not R5b scope.

## Verdict

R5b fixes most of what it set out to fix:
- every rustc-refused cycle is `Cyclic` in every order;
- the R5a order-dependent cycle programs are stable;
- chains settle in linear time (0.2–0.4 s at 20,000);
- the 32k probe takes 0.32 s;
- held ≤ charged holds;
- the registry is unchanged;
- the falsification evidence reproduces.

Three findings block it:
- **REQUIRED-1:** the per-name-space block leaks into globbing and importing modules, giving wrong `Ok`s and a wrong `Canonical` key where e472918 was right;
- **REQUIRED-2:** single-glob cycles that rustc accepts are refused, including a whole crate load;
- **REQUIRED-3:** the new debug invariant check panics on valid three-line input.

First-round verdict: BLOCKED (REQUIRED-1, REQUIRED-2, REQUIRED-3). See the Recheck below for the current status.

## Recheck

**Scope:** the reworked R5b, uncommitted on e472918 (diff hash `2cfd168b…`, unchanged during the recheck). The rework has five parts:
- a per-name-space explicit-import rule;
- cycles that resolve through exactly one glob place per name space;
- every import walked again once the crates open;
- the stuck phase decided in dependency waves;
- `ShadowedGlob` removed, together with its Contract text.

The probes were rebuilt against the reworked tree. The "old" probe is still e472918.

### Gates (reworked tree)

Every gate exits 0:
- `cargo test -p raf-source --locked`: 377 passed, 0 failed;
- the same with `--release`: 377 passed, 0 failed;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- rustdoc with `-Dwarnings`;
- `docs_contract` + `repo_gates`: 139 and 55 passed;
- `make fmt`;
- `make cross-check`;
- `make msrv`;
- `make reuse`: compliant;
- `make fuzz`: 5,000 runs per target.

`ShadowedGlob` no longer appears in `crates/` or `docs/`. The diff holds no process wording, and `.deficiencies` holds only its marker.

### First-round findings

| Finding | Repro | Reworked R5b | rustc |
|---|---|---|---|
| REQUIRED-1 (a) | `A1-glob-chain-second-glob.rs` | `n::A` value is `Ambiguous` | E0659 |
| REQUIRED-1 (b) | `A2-trait-key.rs` | `Unresolved {demo::x::T, demo::z::T}` | E0659 |
| REQUIRED-1 (c) | `A3-glob-chain-crate.rs` | `n::x` is `Ambiguous` | E0659 |
| REQUIRED-1 (d) | `A4-import-chain.rs` | `e::B` type is `Ok demo::c::B` | `c::B` |
| REQUIRED-1 (e) | `A5-value-import-crate-head.rs` | `Ok shared::x` | `shared::x` |
| REQUIRED-2 | `B1-one-glob-cycle.rs` | loads, keyed `Canonical("demo::b::T")` | OK |
| REQUIRED-3 | `C1-check-settled-panic.rs` | no panic; `a::inn` type is `Ok other`, and `a::inn::x` is `Ok other::x` | OK |

**REQUIRED-1, REQUIRED-2 and REQUIRED-3 are fixed.**

The cycle table from (1) now matches rustc on every row:
- V1, V5, V7 and V9 resolve through their single glob;
- A4, V3, V6, V8 and the plain 2-cycle stay `Cyclic`.

The metaschemas shape again gives the module in the type name space and the function in the value name space.

### rustc-oracle fuzz, fresh seeds 41–52 (12,000 programs, 5,479 clean)

| Finding | Reworked R5b | e472918, same programs |
|---|---|---|
| Clean programs: a rustc answer the tool refuses | 7 queries in 6 programs | 8 queries |
| Programs rustc refuses: an `Ok` for a name only a rejected import binds | 34 | 673 |
| Programs rustc refuses: a definite wrong `Ok` | 5 (all E0252/E0255/E0428 conflict programs) | 21 |

Of the 7 refusals of clean code:
- **4 queries are pre-existing**, and e472918 refuses the same programs. They are `Ambiguous` answers in glob cycles where rustc, with an ambiguous glob re-exported through a second glob, picks the direct glob silently.
- **2 programs are new**, regressions from correct e472918 answers. See ADVISORY-R2.

Of the 34 `Ok` answers for rejected imports:
- 30 are in conflict programs;
- 4 are cycles where rustc rejects the whole import because one name space stays undetermined, while the tool answers in the name space that is determined. See ADVISORY-R3.

The implementer's "0 disagreements" holds for seeds 1–13. On fresh seeds it does not quite hold: 2 programs.

### Order search, fresh seeds 21–25 (10,000 programs × 12 orders)

| Set | Programs | Order-dependent | Panics |
|---|---|---|---|
| local | 5,000 | 0 | 0 |
| ws | 5,000 | 2 | 0 |

Both ws differences are message-only: which impl the `UnresolvedTrait` error names.

### Termination: adversarial glob networks (`term.py`)

The networks are random dense graphs of public and private globs, imports, renames, definitions and nested `use super::*` modules.

| Modules | Build | Programs | Hangs | Worst time | Panics |
|---|---|---|---|---|---|
| 8 / 20 / 60 | debug | 1,300 | 0 | 0.3 s | 179 (all `check_settled`, `settle.rs:1492`) |
| 200 | release | 100 | 0 | 0.84 s | 0 |
| 1,000 | release | 80 | 0 | 21.6 s | 0 |
| 8 / 20 / 60, private globs replaced by public | debug | 1,300 | 0 | 0.26 s | **0** |

I found no hang, rotation or toggle, and no round or settle-bound assertion fired. Every panic involves a private glob, and the minimal case is REQUIRED-4. The time at 1,000 modules grows quadratically with glob density. That cost is pre-existing and belongs to R5c, which is tracked.

### Performance and memory (release, loaded host)

| Probe | Reworked R5b | e472918 | First R5b |
|---|---|---|---|
| `spread` 32k imports | 574–585 ms | 238–292 ms | 314–321 ms |
| 1,600 2-cycles | 85 ms | | 50 ms |
| 20,000-link chain ending outside | 607 ms | | 375 ms |
| E2 / E8 / E12 at n=8,000 | 0.24 / 0.15 / 0.18 s | | |
| E2 / E8 / E12 at n=20,000 | 0.59 / 0.42 / 0.48 s | | |
| Glob chain L=4,000 | 4.0 s | 8.0 s | |
| Glob chain L=8,000 | 13.0 s | | |
| Glob chain L=20,000 | 115 s | 275 s | |

- **The plan's checks hold:** the 32k probe runs under 1 s, and the chains of imports that bind nothing are linear.
- **The glob chain** is quadratic but faster than e472918. It belongs to R5c (P-F5).
- **Held ≤ charged:** the index-retention harness, rebuilt on the reworked tree, gives `index <= item_bytes` for all eight shapes.

### Falsifiability (my mutations, on a copy of the reworked tree)

The new one-glob cycle fixture passes on e472918, because e472918 already resolved those cycles, so I covered it by mutation:
- **M-c:** a cycle never falls back to its glob. Two tests fail: `a_cycle_of_imports_with_one_glob_on_it_resolves_through_the_glob_in_every_order` and `imports_that_wait_on_one_another_through_globs_are_decided_as_rustc_does_in_every_order`.
- **M-d:** a name space with two fallback places takes the first. Three tests fail: `a_cycle_of_explicit_imports_is_cyclic_whatever_its_globs_hold_in_every_order`, the one-glob fixture, and `resolution_does_not_depend_on_the_order_of_the_declarations`.

### REQUIRED (recheck)

**REQUIRED-4: a private glob route decides visibility by declaration order, so a four-line program that rustc accepts gets a different answer in different orders and fails the debug re-walk.**

`repro/D1-check-settled-glob-loop.rs`:

```
pub mod m3 { pub use crate::m1::*; }
pub mod m4 { pub use crate::m3::X; }
pub mod m1 { pub use crate::m2::*; use crate::m4::*; }
pub mod m2 { pub fn X() {} }
```

- **rustc** accepts it, with `crate::m3::X()` and `crate::m4::X()`. `m3::X` is `m2::X`.
- **Release:** in 12 of 24 orders `demo::m3::X` is `NotFound`; in the other 12 it is `Ok demo::m2::X`.
- **Debug:** the same 12 orders panic at `settle.rs:1492`: "the imports of `X` bound {ItemId(4)} and reach {} in the settled tables".
- **With the glob of `m4` made `pub`:** stable and correct in all 24 orders.

Cause: `m1` reaches `m2::X` by two glob routes, a public one through `m2` and a private one through `m4`.
- `Bound` keeps one route per item.
- `Origin::rank` breaks the tie on `Link`, which compares module indices, and module indices follow declaration order.
- `visible(m1, m3)` filters by the kept route's visibility.
- So when the private route ranks first, the item is hidden from `m3`.

e472918 gives the same order-dependent release result, so the defect is pre-existing. R5b still owns it, because:
- R5b's Contract states that "the result does not depend on the order of the declarations";
- the debug check that R5b adds crashes on valid input;
- the random-network search hits it in 179 of 1,300 programs.

**Required:**
- visibility must not depend on which route is kept (for example, the most visible route wins, or a public route ranks below a private one);
- a fixture of this shape in every order, in debug and release.

### ADVISORY (recheck)

- **ADVISORY-R1:** the 32k `spread` probe went from about 0.3 s to 0.58 s, against e472918 and against the first R5b, on the same host minutes apart. It is within the plan's "under 1 s".
- **ADVISORY-R2: two convoluted rustc-valid programs regress from correct answers to refusals.** Each involves a cycle through a glob that includes a renamed import or a self-import in one name space.
  - `w276m.rs` (in `/tmp/r5b-review`): `a::B ↔ b::B` resolves in rustc to `b::A` through `a`'s glob of `d`. The tool gives `Cyclic` for both. `d`'s import is undecided in the value name space only, but its glob passes nothing in any name space.
  - `w742m.rs`: `e::B` in the type name space is `c::B` in rustc. The tool gives `NotFound`.
  - The Contract's "a glob of another module does not bring the name in from the import's module meanwhile, as rustc hides a glob binding of a name that a single import may still bind" describes a per-name-space rule in rustc, but the tool applies it across name spaces. "A renamed import, where rustc cannot determine the imports" is also too broad for these cases.
  - Fix the hold-back to be per name space, or narrow the Contract wording and add a fixture.
- **ADVISORY-R3:** in 4 cycles that rustc rejects as a whole (E0432, because one name space stays undetermined), the tool answers `Ok` in the name space that is determined, and `NotFound` in the other. This is far fewer than the 673 such answers on e472918, and it is not a wrong item.
- **ADVISORY-R4:** the round and settle bounds remain debug-only assertions; no input exceeded them.

### Recheck verdict

The rework fixes REQUIRED-1, REQUIRED-2 and REQUIRED-3:
- the rustc-oracle fuzz on 12,000 fresh programs leaves 2 new refusals of clean code, down from 217 queries;
- the order search on 10,000 fresh programs finds only message-only differences;
- the gates, perf, memory and mutations pass.

One new finding blocks it. **REQUIRED-4:** on a four-line program that rustc accepts, the result depends on declaration order through a private glob route, and the R5b debug re-walk panics in half the orders. The random-network search reproduces it in 179 of 1,300 programs.

Recheck verdict: BLOCKED (REQUIRED-4). See Recheck 2 for the current status.

## Recheck 2

**Scope:** the final R5b round, uncommitted on e472918 (diff hash `dc726ae6…`). It covers:
- the REQUIRED-4 fix: an item reached by several glob routes is as visible as its most visible route;
- the per-name-space hold-back (ADVISORY-R2);
- `Cyclic` through an import that a cycle leaves undecided in another name space (ADVISORY-R3);
- the round and settle bounds checked in release (ADVISORY-R4);
- four further rustc fixes.

The probes were rebuilt. The "old" probe is still e472918.

### Gates

Every gate exits 0:
- `cargo test -p raf-source --locked`: 388 passed, 0 failed;
- the same with `--release`: 388 passed, 0 failed;
- clippy (`-D warnings`);
- rustdoc (`-Dwarnings`);
- `docs_contract` + `repo_gates`: 139 and 55 passed;
- `make fmt`, `make cross-check`, `make msrv`, `make reuse`;
- `make fuzz`: 5,000 runs per target;
- `make check`.

The release bounds return `LoadErrorKind::Internal` (`within_settles`, and the round check), not a debug assertion.

### Repros

- **D1 (REQUIRED-4):** `demo::m3::X` is `Ok demo::m2::X` in all 24 orders, in debug and in release, with no panic.
- **REQUIRED-1 to REQUIRED-3:** every repro still matches rustc (A1–A5, B1, C1).
- **ADVISORY-R2:** both programs now match rustc. In `R2-renamed-cycle-cross-ns.rs`, `a::B` and `b::B` in the type name space are `b::A`. In `R2-self-import-cross-ns.rs`, `e::B` in the type name space is `c::B`.

**REQUIRED-4 is fixed, and ADVISORY-R1 to ADVISORY-R4 are addressed.** The `spread` probe takes 0.26–0.37 s, against 0.26–0.34 s for e472918 on the same host. The host load average was 37–52.

### rustc-oracle fuzz, fresh seeds

**Public globs, seeds 81–92 (12,000 programs, 5,445 clean):**
- **Clean programs:** 0 wrong `Ok`. 5 programs give a refusal, and every one is the `Ambiguous` class discussed below.
- **Programs rustc refuses:**
  - 2 `Ok` answers for rejected imports, both in conflict programs;
  - 4 programs with a definite wrong `Ok`. 3 are conflict programs. The fourth is error recovery: rustc's dummy binding for the failed import `crate::d::B` collides with a glob, while the tool keeps the failed import's name out of the glob.

**Private globs, seeds 101–112 (12,000 programs, 40% of globs private).** I extended the harness for this run:
- each probe sits in a `mod __p` inside its module and reads `super::N`, so rustc's privacy errors (E0603) do not mask the result;
- every query was compared with e472918 on the same program.

| Clean programs | Reworked R5b | e472918 |
|---|---|---|
| Wrong `Ok` (the probe fails) | 90 queries | 92 queries, the same ones |
| Different item | 2 | 2 |
| Refused, `Ambiguous` | 31 | 33 |
| Refused, `NotFound` | 0 | 36 (fixed) |
| **Refused, `Cyclic`** | **38 queries in 13 programs, all new** | 0 |

- **The 90 wrong `Ok`s** are the documented limit, "a path walk does not check visibility". The tool reads a glob-imported private binding of an intermediate module that rustc cannot see. They are the same queries on e472918.
- **The 38 `Cyclic` refusals** are REQUIRED-5.

### Order search, fresh seeds 41–45

| Set | Programs × orders | Order-dependent | Panics |
|---|---|---|---|
| local | 5,000 × 12 | 0 | 0 |
| ws | 5,000 × 12 | 0 | 0 |
| net (private globs, `cfg` twin imports and modules, unit structs, renames) | 5,000 × 12 | 0 | 0 |

### Termination: private-glob networks, fresh seeds 31–35

| Modules | Build | Programs | Hangs | Panics | Worst time |
|---|---|---|---|---|---|
| 8 / 20 / 60 | debug | 2,500 | 0 | 0 | 0.96 s |
| 200 | release | 100 | 0 | 0 | 1.4 s |
| 1,000 | release | 40 | 0 | 0 | 21.9 s |

The time at 1,000 modules is R5c's glob-density cost, which is unchanged from Recheck 1. No `Internal` error and no hang appeared.

### The remaining `Ambiguous` refusals, and rustc's lint

Shape (`/tmp/r5b-review/rc/amb.rs`):
- `a` globs `b` and `c`, and both bring in a different `B`;
- `b` globs `e` and `a`.

rustc reports `a::B` as E0659 on use. For `b::B` it compiles and gives `e::B`: `b`'s direct glob of `e` and its glob of `a`'s ambiguous binding agree on `e::B`.

On this program rustc 1.100-nightly emits only `warning: ambiguous glob re-exports` (`ambiguous_glob_reexports`, warn by default) at `a`. The future-compatibility lint `ambiguous_glob_imports` is deny by default, but it does not fire here, even under `#![deny(ambiguous_glob_imports)]`.

The tool gives `Ambiguous {e::B, c::B}` for `b::B`, as e472918 does.

**Judgment: acceptable under fail-hard.**
- It is a typed refusal, never a wrong item.
- It covers a name that rustc itself flags as an ambiguous re-export.
- It is not an R5b regression.

ADVISORY-R5 asks for one Contract sentence on it.

### Q13: is the guard truly unreachable?

I built a probe with the guard disabled (`if false && left_to_globs(..) && !covers_globbed(..)`). I compared it with the reworked tree on 9,000 cycle-heavy programs of 4 to 25 modules. These programs mix public and private globs, `cfg` twin imports and twin modules, unit structs and renames, and 3,645 of them give at least one `Cyclic` answer.

The two builds gave the same output on every program, and neither panicked in debug.

The implementer's argument holds as long as walks ignore visibility. A keeper that reached an item beyond the glob answer would have bound it in the queue, which leaves the place with a definition, so it would no longer be a fallback place.

A fix for REQUIRED-5 that makes import walks visibility-aware changes that premise, so the guard should stay. It already fails hard, with `Cyclic`.

### REQUIRED (Recheck 2)

**REQUIRED-5: a cycle that rustc breaks by visibility is refused, and so is a whole crate whose trait goes through it.**

`repro/E3-child-use-super-cycle.rs`, the idiomatic child `use super::*`:

```
pub mod c { pub struct B {} }
pub mod a { pub use self::b::B; pub use crate::c::*; mod b { pub fn B() {} use super::*; } }
```

- **rustc** accepts it. `a::B` is the function `b::B` in the value name space and `c::B` in the type name space.
- **e472918** gives exactly that.
- **The reworked R5b** gives `Cyclic` for `a::B` in every name space and every order.

`repro/E1-private-glob-cycle-trait.rs`, the same shape with a trait `c::B` and `impl a::B for String`:
- rustc accepts it;
- e472918 keys it `Canonical("demo::c::B")`;
- **R5b refuses the whole load with `UnresolvedTrait a::B`.**

**Cause:** rustc resolves `a`'s import of `b::B` while skipping any binding that `a` cannot access. `b`'s glob of `a` is private to `b`, so in the type name space `b::B` is empty from `a`, the import binds the function only, and `a`'s glob of `c` answers the type name space. The tool walks without visibility, so it sees `a::B (type) → b's glob → a::B`, which is a cycle with an undecided name space. The new rule then makes it `Cyclic` everywhere.

**Control:** with `b`'s glob made `pub`, rustc gives E0432, and the tool's `Cyclic` is right there. Only the private case is wrong.

**Scale:** 13 of about 5,450 clean programs in the private-glob fuzz.

**Contract:** this contradicts "rustc refuses every such crate, failing an import of the cycle".

**Required:** when an import path is walked for the cycle and undecided decision, a glob-imported binding that the importing module cannot access must not count. That is rustc's `is_accessible_from` on single-import resolution. Add fixtures:
- E3 and the E1 trait form, in every order;
- beside them, the `pub` glob form, which stays `Cyclic`.

Then re-run the Q13 differential, because this changes its premise.

### ADVISORY (Recheck 2)

- **ADVISORY-R5:** the Contract says that "two items make it ambiguous, as in rustc". For an ambiguous binding re-exported through a glob that another glob agrees with, rustc only warns (`ambiguous_glob_reexports`) and resolves. The index refuses with `Ambiguous`. One sentence would state this conservative difference.
- **ADVISORY-R6:** the 90 wrong `Ok`s through intermediate private glob bindings are pre-existing and fall under the documented "a path walk does not check visibility". A REQUIRED-5 fix along rustc's accessibility rule would also remove most of them.

### Recheck 2 verdict

**Fixed:**
- REQUIRED-4;
- ADVISORY-R1 to ADVISORY-R4;
- the two ADVISORY-R2 programs.

**Clean:**
- 12,000 public-glob oracle programs: no wrong `Ok` on clean code;
- the order search, 15,000 programs × 12 orders, including private-glob networks;
- 2,640 termination programs: no hangs and no panics;
- gates, `make check`, perf, and held ≤ charged.

**Acceptable:** the `Ambiguous` versus `ambiguous_glob_reexports` class, under fail-hard.

**Q13:** unreachable in 9,000 differential programs.

**Blocking: REQUIRED-5.** A rustc-valid crate that uses the idiomatic child `use super::*` beside a parent glob that binds the same name in another name space is now refused (`Cyclic`, or `UnresolvedTrait` for the whole load). e472918 resolved it correctly, and the Contract claims that rustc refuses every such crate.

Recheck 2 verdict: BLOCKED (REQUIRED-5). See Recheck 3 for the current status.

## Recheck 3

**Scope:** the REQUIRED-5 fix, uncommitted on e472918 (diff hash `14fea968…`, unchanged during the recheck):
- `use` paths are read with rustc's accessibility rule;
- the cycle machinery reads places the same way;
- the Q13 guard is narrowed to what each reader can access;
- the private-import re-walk panic is fixed;
- the Contract states two rustc visibility rules that are not modelled, under Constraints.

### Gates

Every gate exits 0:
- `cargo test -p raf-source --locked`: 395 passed, 0 failed;
- the same with `--release`: 395 passed, 0 failed;
- clippy (`-D warnings`);
- rustdoc (`-Dwarnings`);
- `docs_contract` + `repo_gates`: 139 and 55 passed;
- `make fmt`, `make cross-check`, `make msrv`, `make reuse`;
- `make fuzz`: 5,000 runs per target;
- `make check`.

There is no process wording in the diff.

**Held ≤ charged:** the index-retention harness, rebuilt on this tree, gives `index <= item_bytes` for all eight shapes.

**Perf:** the 32k `spread` probe takes 262 ms, against 324 ms for e472918 on the same host (load average 19–34).

### Repros

| Repro | rustc | Reworked R5b |
|---|---|---|
| `E3-child-use-super-cycle.rs` | OK | `a::B` is `c::B` in the type name space and `a::b::B` in the value name space, matching e472918 |
| `E1-private-glob-cycle.rs` | OK | `a::B` and `b::B` are `c::B` in the type name space; `a::B` is `b::B` in the value name space; the same in all 6 orders |
| The E1 trait form | OK | keyed `Canonical("demo::c::B")` |
| The E1 trait form with a `pub` glob | E0432 | `UnresolvedTrait`, which is right |
| D1 | OK | correct in all 24 orders |

A1–A5, B1, C1 and both ADVISORY-R2 programs are unchanged and still match rustc.

**REQUIRED-5 is fixed.**

### rustc-oracle fuzz, fresh seeds

**Private globs, seeds 141–152 (12,000 programs, 5,522 clean),** compared per query with e472918 on the same programs:

| Clean programs | Reworked R5b | e472918 |
|---|---|---|
| Wrong `Ok` | 12 queries in 10 programs | 96 queries in 71 programs |
| Different item | 0 | 4 |
| Refused, `Cyclic` | **0** | 0 |
| Refused, `NotFound` | 0 | 13 |
| Refused, `Ambiguous` (the accepted `ambiguous_glob_reexports` class) | 4 | 30 |
| Programs rustc refuses: an `Ok` for a rejected import | 95 queries | 788 queries |

All 12 remaining wrong `Ok`s fall into the two rules that the Contract now lists under Constraints:
- **Unnarrowed re-exports**, for example seed 143 #106: `c { use crate::e::*; pub use crate::c::B as A; }` re-exports a privately globbed type, which rustc keeps private, so `d::A` is empty for rustc.
- **Ambiguity through an inaccessible glob**, for example seed 144 #918: `b { pub use crate::c::*; use crate::a::*; }` gives E0659 for `e::B` when `e` globs `b`.

11 of the 12 are wrong on e472918 too. One query (seed 148 #404) was `NotFound` on e472918, which is right, and is a class-2 `Ok` now.

**Public globs, seeds 171–176 (6,000 programs):**
- 0 disagreements on clean programs;
- 2 wrong `Ok`s, both in programs where rustc reports E0252 (a name defined twice).

### Order search and termination, fresh seeds

**Order search, seeds 51–55:** 0 order-dependent programs and 0 panics, for each of:
- local: 5,000 programs × 12 orders;
- ws: 5,000 programs × 12 orders;
- net, with private globs, `cfg` twins, unit structs and renames: 5,000 programs × 12 orders.

**Termination, seeds 41–44:** private-glob networks.

| Modules | Build | Programs | Hangs | Panics | Worst time |
|---|---|---|---|---|---|
| 8 / 20 / 60 | debug | 2,500 | 0 | 0 | 0.27 s |
| 1,000 | release | 40 | 0 | 0 | 5.6 s |

### The 117 dump changes

`/tmp/r5b4/dumps` holds 18 changed import lines in katamari and 99 in purrdf, with purrdf's `Decimal` line on top of those. Every change has one shape:
- the target module has a private `mod x;` and a `pub use` of an item named `x`;
- an outside importer that names `x` used to bind the inaccessible module in the type name space;
- it now binds only the public item.

Spot-checks:

| Import | Source | Old binding | New binding |
|---|---|---|---|
| `purrdf_core::parse_iri` | `pub use purrdf_iri::{parse as parse_iri}`; `purrdf_iri` has private `mod parse;` and `pub use parse::{parse, ..}` | the module `purrdf_iri::parse` | the function |
| `purrdf_geo_kernel::determinism::relate` | `use crate::topology::relate;`; topology has private `mod relate;` and `pub use relate::{relate, ..}` | the module | the function |
| katamari `…::log_values::value` | `use katamari_log::{.., value}`; private `mod value;` and the exported `macro_rules! value` | the module | the macro |

A reduction compiled with rustc confirms the pattern:
- `crate::core::parse_iri()` and the function `relate()` compile;
- `crate::core::parse_iri::other()` and `relate::x()` through the import fail with E0433, so the type-name-space module binding is gone, as the tool now has it (`NotFound` in the type name space);
- e472918 answered `Ok demo::iri::parse` and `Ok demo::topo::relate`, the modules, which is wrong.

The remaining purrdf retrieval and iri lines (`plan`, `resolve`) have the same private-module shape.

**The 117 changes are rustc-correct.**

### Judgment: the two unmodelled visibility rules

**What they are:**
- They give wrong `Ok` answers on valid but unusual code. Each needs private globs combined with an explicit re-export, or with a glob ambiguity, under a name that collides in a name space.
- They are pre-existing: e472918 is wrong on 11 of the 12 queries and on 84 more in the same programs.
- R5b cuts them from 96 to 12 queries, and from 71 to 10 programs.
- They are outside R5b's mandate: A4, P-F6 and checkability.
- They are now stated, not hidden: a Constraint names both rules.
- Class 2 in rustc depends on which of the ambiguous bindings rustc holds as primary. In seed 148 #404, rustc gives E0573 ("cannot find type") in the reading module, not E0659. Matching that exactly means modelling rustc's internal choice.

**Judgment:** acceptable as documented known limits for R5b, with two conditions:
- **ADVISORY-R7:** correct the class-2 Constraint sentence. For the reading module rustc reports either the ambiguity (E0659) or that nothing is there (E0573/E0425), depending on which binding it keeps.
- **ADVISORY-R8:** accepting a documented wrong-`Ok` limit is a scope decision, not mine to authorize. Record it in `validation.md` as the orchestrator's decision, or move it to a tracked item. Either satisfies "no deferrals".

If the orchestrator does not accept the limit, it becomes REQUIRED: model rustc's narrowing of re-export visibility and the propagation of an ambiguity through inaccessible globs.

### ADVISORY (Recheck 3)

- **ADVISORY-R7** and **ADVISORY-R8:** above.
- **ADVISORY-R9:** the one class-2 query that e472918 got right and R5b now gets wrong (seed 148 #404) is not a separate defect. Note it in the Constraint's evidence, or add it as a fixture that pins the stated behaviour.

### Recheck 3 verdict

**Fixed:** REQUIRED-1 to REQUIRED-5 stay fixed. In particular, REQUIRED-5's `Cyclic` refusals went from 38 to 0 on fresh seeds, and the E1 trait crate loads.

**Clean:**
- the public-glob oracle;
- the order search, 15,000 programs × 12 orders;
- the termination runs, 2,540 programs;
- the gates, `make check`, perf, and held ≤ charged.

**Dumps:** the 117 changes are rustc-correct.

**Remaining:** the only wrong answers left on clean code are the two documented visibility constraints. They are pre-existing, reduced eightfold, and acceptable for R5b, provided the orchestrator records the known-limit decision (ADVISORY-R8).

VERDICT: PASS
