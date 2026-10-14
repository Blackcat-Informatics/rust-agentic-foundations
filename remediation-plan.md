# Remediation plan: issue 2 / PR 13 (Stage 2), revision 3

Revision 2 integrates `reviews/remediation-plan-review.md`: all of B1–B16, plus advisories A1–A18 as noted per item. Revision 3 integrates the recheck: N1 (R3 record sizing) and the six recheck advisories. Revision 3.3 integrates N4 (fact budget counted in owned bytes). Revision 3.2 integrates N3. Revision 3.1 integrates N2 (per-file data budget, parse-peak token budget, two Contract figures) and R3-1 (commit split).

**Inputs**
- `gap-analysis.md` (G1–G17)
- `reviews/completion-audit-stage2.md` (cited as **CA-F1..F7**)
- `reviews/security-review.md` (S-1–S-15)
- `reviews/architecture-review.md` (A1–A18, plus "Resolver checkability")
- `reviews/quality-review.md` (Q-1–Q-7)
- `reviews/performance-review.md` (cited as **P-F1..F11**)
- `issue-analysis.md` (Stage 2 section)
- `historical-fit.md` (cited as **HF-**)

**User decisions** (verbatim in `validation.md`, Stage 2 user decisions):
- A1: per-item marker.
- A2: key crates by package.
- G9: typed `NoRoots`.
- Sign-offs confirmed: Unicode-3.0, NCSA, `bin_crate`, `Rule: Sync`.

The Stage 1 decisions 1–5 and the design in `plan.md` (including C7: "`Item::Verbatim` is an error") still bind.

**Rule for every item:** the code is changed to meet the written contract. A contract sentence changes only in these cases, and the item labels which one applies:
1. A user decision changes the behaviour (A1, A2, G9).
2. The sentence is factually wrong and the correction does not weaken a guarantee.
3. The sentence adds a statement where the contract was silent (A10, A18).

The carve-outs added in Task 9 for G1 and G8 are deleted.

**Publication:** after each commit, post the gap, fix, checks and commit to issue 2 and PR 13. The final summary goes to both once the exit audit passes.

## Execution model

**Ordering (B1).**
- **raf-source chain** (serial): R2 → R3 → R8 → R4 → R5a → R5b → R5c → R5d → R6 → R7 → R9 → R10 → R11 → R12s.
- **raf-okf chain** (serial, in parallel with the raf-source chain up to R7): R1 → R12o.
- **Joins:**
  - R7 starts only after R1 and R12o land, because they edit `fuzz/`, the `Makefile` and `tests/docs_contract.rs`.
  - Every edit to `source-index.md`, `okf-profile.md`, `SECURITY.md`, `AGENTS.md` and `tests/docs_contract.rs` is made on top of the previous item's commit, never in parallel.

**Tiers (A1).**
- Implementers: R2, R3, R4, R5a–d, R6 and R8 at opus/high; all others at sonnet/medium.
- Reviewers: R2–R6 and R8 at opus/high; all others at sonnet/medium.
- Reviews are saved as `tasks/RN-review.md`, with `VERDICT: PASS | BLOCKED`.
- Implementers do not commit. The orchestrator commits each item, signed and with hooks, after its review passes.

**Per-item gates (A14).** Every item's acceptance includes, in addition to its own checks:
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test -p <crate> --locked`
- `--release` for raf-source

Every signed commit stays green.

**Mutation rule (B14).** Some tests cover behaviour that is already correct: G4, G5, Q-6, Q-7, and the new at-limit fixtures. For each, the reviewer runs the named mutation once, records that the test fails under it, and reverts. Every other test must fail on 6c744af, and the reviewer records that.

**Registry harness (A12).** Committed as `crates/raf-source/examples/registry_scan.rs`, which takes the registry directory as an argument. It is not part of `make check`, because it needs a local registry.
- It loads the newest version of each crate and records `name@version`, the outcome and the error message.
- **Misplaced-item oracle:** every item's path prefix equals a module path recorded in its file's `Reach`.
- The **baseline tally** on 6c744af is recorded in `validation.md` before R2.
- Item checks are written as "no crate fails with message M; each crate that now fails for another reason is root-caused in `validation.md`".
- **Budget false-refusal check:** across the scan and this repository, 0 crates are refused by any budget kind introduced in Stage 2.

## Items

### R1: raf-okf body memory is bounded (S-1 High, P-F1 required, S-12 part)

- **Gap:** each H2 heading's slug is cloned into every line, prose line and PROPOSED block (`body/mod.rs:131,173`, `marker.rs:191`). A 200 KB document runs out of memory, which breaks `okf-profile.md:96-98`.
- **Fix:**
  - Lines and blocks carry a section index (`usize`) into `sections`, and accessors return `&str` through the body.
  - Add the fuzz target `parse-document` (file `fuzz_targets/parse_document.rs`), driving `Document::parse` and `Profile::validate`.
- **Check (B2):** `crates/raf-okf/tests/bounds.rs::a_long_heading_over_many_lines_stays_linear`.
  - **Inputs:** two 1 MiB documents of the same line count and shape, one with a 60 KB H2 heading and one with a 1-byte heading.
  - **Assertion:** `peak(long) <= peak(short) + 4 × heading_bytes × sections + 64 KiB`, measured with a counting global allocator.
  - **Ceiling:** the allocator returns null above 1 GiB, so the run on HEAD ends in `handle_alloc_error` (a falsifying abort) rather than exhausting the host. That run is made in a child process and recorded.
  - The same pair is run for many PROPOSED blocks under one long heading.
- **Commit:** `fix(raf-okf): bound body memory by indexing sections`.

### R2: modules are placed under their own path (CA-F2 HIGH)

- **Gap:** `index.rs:249-271` (`Builder::ensure`) records a module under its parent's slot when the module file sorts before its declaring file.
  - The binding gate accepts the non-existent `raf_okf::body::FenceBlock`.
  - Valid crates fail with `SuperAboveRoot`.
- **Fix:** the module path comes from the declaration chain recorded by the walk. Index building visits a module before its children.
- **Check (B8):** `module_tree.rs` fixtures:
  - `a/mod.rs` + `a/b.rs` and `a.rs` + `a/b.rs`;
  - a `#[path = "gen/generated.rs"]` file whose children `a`, `ga` and `z` are each placed under `generated`;
  - `tests/common/mod.rs` + `tests/common/a.rs` reached from `tests/zz.rs`, with item paths asserted;
  - `super::super::top` from `src/a/b.rs` loads;
  - `demo::a::in_b` and `demo::a::b::in_a` are `NotFound`, beside `demo::a::b::in_b` as `Ok`.
- **Entry-point checks:**
  - `this_repository.rs`: `raf_okf::body::FenceBlock` is `NotFound` and `raf_okf::body::blocks::FenceBlock` resolves.
  - A `docs_contract` negative fixture binding the misplaced path is rejected.
  - Registry scan: 0 crates with misplaced items (before: 164), and no `SuperAboveRoot` in `brotli`, `structured-zstd` or `wasmi`.
- **Commit:** `fix(raf-source): place a module under its declared path`.

### R3: file identity, traversal and work budgets (S-2 High, S-4, P-F4 required, P-F8, P-F10)

- **Fix (B3, B4, P-F10):**
  - **`FileId`:**
    - on Unix `DiskTree`, (dev, ino) from the verified handle;
    - on other `DiskTree` platforms, the canonical root-relative path that `contain` already computes (std-only; no `#![feature]`, no `unsafe`);
    - on `MemoryTree`, the normalised key.
  - A file reached under several spellings is parsed once and keeps the first spelling in sorted order. The Contract states this next to "one file has one spelling".
  - **Item records (recheck N1):** the API keeps one `Item` per path (`source-index.md:413-416`; `items()`/`item_at()` return `&Item`), so no API or Contract change is needed.
    - Each record shrinks to ≤ 256 bytes: per-file data (file id, text spans, attributes, docs) is shared through an index or `Arc`, and module paths are interned.
    - A unit test asserts `size_of::<Item>()` ≤ 256 bytes (the heap part is charged by the item-bytes budget).
  - **Item-bytes budget (N4 applied to items):** counted in bytes with a 512 MiB limit. Each `Item` charges its `size_of` plus every owned or interned string the first time it is interned (path segments, names, written self types), plus a fixed per-entry charge for the intern table slot and `Arc` header.
    - Fixtures: an adversarial long-path or long-name fixture, plus at-limit and one-byte-over fixtures.
    - Registry scan: 0 refusals.
  - **Total-bytes budget** (`max_total_bytes`) per load: `min(1 GiB, max(256 MiB, 4 × the largest crate in the registry scan))`.
  - **Fact-bytes budget (N2, N3, N4):** a typed per-load budget, counted in **bytes** with a 256 MiB limit, covers the per-file facts (attributes including doc comments, use-tree leaves, impl headers, mod declarations, cfg predicates, `path` values, spans).
    - Each fact record charges its fixed `size_of` plus the length of every `String` and `Vec` it owns.
    - Charging happens as facts are built on the parse thread, so an oversized file is refused typed, naming the file and line.
    - Fixtures:
      - one adversarial large-payload fixture per kind (a 1 MiB doc comment, a 1 MiB written self type, a `use` group of long names);
      - an at-limit fixture that loads, and one byte over that is refused with `Budget`.
    - Registry scan: 0 refusals.
    - Figure (1) then holds by construction.
  - **Parse peak (N2):** the transient peak while one file parses is bounded by `max_file_bytes` × the measured syn AST factor. performance-review §2 measured 2.6 GB for an 8 MiB `a<b; ` file. That is reduced by a pre-scan **token budget** per file (default chosen so that token budget × bytes per token ≤ 512 MiB, with bytes per token measured on the costliest admitted shape, e.g. `a<b; `, deep generic args and attribute runs, each with a fixture), refused typed with at-limit and over-limit fixtures. The registry scan shows 0 refusals.
  - **Contract (N2):** states two measured figures. (1) Retained memory: item bytes (512 MiB) + source text (≤ 1 GiB) + fact bytes (256 MiB) + glob-edge bytes (512 MiB). (2) One-file parse peak. Both come with the measured time at each limit. The 8 MiB `a<b; ` probe is re-run and its peak is recorded against figure (2).
  - **Discovery and member-glob expansion:** iterative, with a depth limit of 128 below a root (matching module nesting).
  - **Mod-tree walk** (`Walker::enter`/`body`/`module`): uses an explicit stack (P-F10).
  - **P-F8 tail:** the package scan at `discovery.rs:104` becomes a `BTreeMap` by name.
  - **R2-review advisory 4:** `examples/registry_scan.rs` applies a per-child address-space limit (`--max-memory`, default 8 GiB, applied by starting the child through `sh -c 'ulimit -v <kib>; exec "$0" "$@"'`, std-only, with no `unsafe`). An over-limit crate prints `ERR memory`, so `--jobs` cannot exhaust the host.
  - **R2-review advisory 3:** the harness oracle accepts a crate-root position only for `#[macro_export]` `macro_rules!` macros.
  - **Contract:** states each budget's measured worst-case time and memory at its limit, and keeps "cannot exhaust a machine" true at those figures.
- **Check (B5):**
  - For each new budget (total bytes, directory depth, item bytes, fact bytes), a fixture at the limit loads, and one over is refused with `LoadErrorKind::Budget`, naming the budget and the file.
  - `workspace_containment.rs::symlink_aliases_of_one_directory_parse_it_once`: a k=18 symlink tree gives exactly one read per file (a read-counting `Tree` wrapper), and the load ends typed or loads within the stated figures.
  - The deep-directory test runs on a `stack_size(2 << 20)` thread in debug: a Memory chain at 20,000 levels and a `DiskTree` chain (unix), both with the typed error.
  - The 128-level module fixture runs on a 512 KiB thread in debug.
  - The security probes `src_load.rs` and `mem_deep.rs` are re-run, end typed within the stated figures, and are recorded.
  - Registry scan: 0 crates refused by the new budgets, with the largest observed values recorded.
- **Commit:** `fix(raf-source): key files by identity and bound total work`.

### R8: containment on every platform (G3 CRITICAL, S-6, S-7, S-8, G7, S-14)

- **Code (B10, A11):** `Strategy` is a `#[cfg]`-gated enum (Linux / OtherUnix / Portable), with a `#[cfg(test)]` override.
  - **Linux:**
    - Open with `custom_flags(libc::O_PATH | libc::O_NOFOLLOW)`.
    - `fstat` the handle and verify it via `read_link("/proc/self/fd/N")`.
    - Only once it is shown to be a regular file or directory, reopen `/proc/self/fd/N` (no `O_NOFOLLOW`) for reading or listing.
    - `entry` uses `symlink_metadata("/proc/self/fd/<dirfd>/<name>")` with a single-segment name.
  - **OtherUnix and Portable:** re-verify `(dev, ino)` / canonical identity after `read_dir`, after each read and after `entry`'s lstat, and refuse on a mismatch. macOS lists through `/dev/fd/<fd>`.
  - **Windows:**
    - Hold the handle with `share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)` (no delete sharing) and the local const `FILE_FLAG_BACKUP_SEMANTICS = 0x0200_0000`.
    - Rename the misnamed `on_handle` parameter.
    - **S-14 (B11):** under `cfg(windows)` only, `DiskTree` refuses a segment that is a reserved device name (case-insensitive, with any extension) or ends in a dot or a space, before canonicalising, as `OutsideRoot`. `RelPath` is unchanged on every platform.
  - Gate `within_watchdog` on `cfg(unix)`.
  - `make cross-check` runs `cargo check -p raf-source --locked --all-targets` with `-D warnings`.
- **Docs (B10):** per-platform rewrites of `source-index.md:112-113`, `:122-123` and `:129-150`, `SECURITY.md:52-65`, and `AGENTS.md:96-103`.
  - Linux: "an outside candidate is never opened for reading or listing; only an `O_PATH` reference is taken, which performs no I/O, device open or FIFO rendezvous".
  - Other Unix and Windows: "an outside file can be opened non-blocking in a race and is then refused before any read", with the `entry` and listing residual windows stated.
  - The Windows reserved-name refusal and the `..` lexical-join note are added as factual platform semantics.
  - User decision 3 covers this per-platform scoping.
- **CI (A10, G7):** add `macos-latest` and `windows-latest` jobs to `.github/workflows/ci.yaml`, running `cargo test -p raf-source --locked`.
- **Check:**
  - The swap test `a_directory_swapped_after_verification_before_the_entries_are_read_is_refused` and a new `entry` swap test pass under Linux, OtherUnix and Portable on Linux.
  - A committed Linux FIFO-race test uses the `#[cfg(test)]` swap hook between the prefix check and the open to place an outside FIFO with a blocked writer thread. It asserts the writer is never released (bounded wait) and the read is `OutsideRoot`. It fails on 6c744af.
  - A Linux test confirms that a tree containing `aux/` and `nul.txt` loads.
  - A `cfg(windows)` reserved-name unit test is compiled by `make cross-check`.
  - `RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target x86_64-pc-windows-gnu` exits 0.
  - The `/tmp/raf-sec` race probe is re-run: 0 outside FIFO opens and 0 outside existence answers in 200k tries.
  - Runtime on macOS and Windows becomes evidence only when hosted CI runs. Until then it is reported as unverified.
- **Commit:** `fix(raf-source): verify on the handle that is read, on every platform`.

### R4: index facts (CA-F5, CA-F6, A1, A2, A3, S-15, G16)

- **CA-F5:** a macro call in a trait body is skipped, as in an impl body.
  - Fixture: `trait_keys.rs::a_macro_call_in_a_trait_body_is_skipped`.
- **CA-F6:** `use self::Inner::*;` of an enum yields its variants in both namespaces.
  - Fixture: `symbol_index.rs::a_glob_of_enum_variants_beside_an_impl_loads`.
- **A1 (user decision, A4-adv):** an inherent impl whose self type cannot be named becomes an item with `SelfType::Unresolved`.
  - Its member `path` is the written self path in the impl's module, and `items()`/`item_at()` carry the marker.
  - `Workspace::unresolved_impls()` lists them, so a consumer can fail on its own terms.
  - Resolving through it gives `ResolveError::UnresolvedSelf`.
  - Contract and Verification are updated (case 1).
  - Fixtures: a `bitflags!`-style macro type with an `impl`, and an `impl dyn LocalTrait`.
- **A2 (user decision, B6):** `libs` and `crate_roots` are keyed by (package, crate name).
  - An import's crate head resolves through the importing target's extern prelude:
    - its own package's library (for non-library targets);
    - its declared dependencies for the target kind, mapped to workspace packages through `path`/`workspace = true` entries and renames;
    - the sysroot crates.
  - `Workspace::resolve(path)` returns `Ambiguous`, naming both targets, when the head is the crate name of more than one package.
  - The new `Workspace::resolve_in_package(package, path)` takes the Cargo package name separately.
  - Contract updated (case 1).
  - Fixtures:
    - two packages with `[lib] name = "shared"`;
    - a third package depending on one of them, whose `use shared::X` resolves to that one;
    - `resolve("shared::X")` is `Ambiguous`;
    - `resolve_in_package("p-two", "shared::X")` is `Ok`;
    - purrdf `crates/` loads.
- **A3:** the grouping key includes the target, and overlap is `Ambiguous` whatever the sort order.
  - **R2-review advisory 5:** a library module named `bin_crate` (or `test_crate`, `bench_crate`, `example_crate`, `build_script`) shares module paths with those crate roots, so lib items land in the bin root. Fix it with the same target-keyed grouping. Fixture: `target_roots.rs::a_lib_module_named_bin_crate_stays_in_the_library` (`demo::bin_crate::x::bin_fn` resolves to the bin; the lib module item resolves to the lib).
  - Fixtures: `target_roots.rs::a_build_script_main_beside_a_lib_module_build_script_is_ambiguous`, and the same for an explicit-path bin, in both file-name orders.
- **S-15 / G16 fixtures:**
  - `module_tree.rs::a_non_literal_path_attribute_is_reported_as_such_from_the_attribute`;
  - `target_roots.rs::an_over_cap_inferred_lib_names_discovery_not_lib_path`.
- **Entry-point check:**
  - Registry scan: no crate fails with "inherent impl names an unseen type", the macro-in-trait message, or the enum-glob message.
  - Katamari `crates/` and purrdf `crates/` load.
- **Commit:** `fix(raf-source): index macro-generated impls, enum globs and shared crate names`.

### R5a: extern-prelude aliases and stack-safe trait keys (G2/CA-F1 CRITICAL, CA-F7)

- **Fix:**
  - An import never resolves through its own binding. In edition 2018 and later, a single-segment `use x;` (or `{self}` / `self as y`) whose head is in the target's extern prelude (the R4 table) is an external alias. Resolving through it is `NotFound`, and trait keys follow it to the dependency path.
  - In edition 2015, `use x;` is crate-relative. In 2018+, a name that is both a local item in the type namespace and an extern-prelude crate is `Ambiguous` (A5-adv; value-namespace items do not collide).
  - **Revised at implementation (rustc-verified):** a local item or explicit import shadows an extern-prelude crate of the same name. Only a name brought in by a glob conflicts with an extern-prelude crate, which is `Ambiguous` (E0659). The rustc probes are recorded in `raw/R5a-falsification.txt`.
  - Trait-key derivation (`impls.rs:159-260`) becomes an explicit worklist, with no recursion on the caller's stack.
- **Check (B8):** fixtures in `use_resolution.rs` and `trait_keys.rs`:
  - `pub use serde;`, then `demo::serde::Serialize` is `NotFound`;
  - `pub use std;` is `NotFound`;
  - `pub use serde::{self, Serialize};` gives `NotFound` for `demo::serde::Serialize` and `demo::Serialize`;
  - `use std;` + `impl std::fmt::Display for &Foo` loads, keyed `std::fmt::Display`;
  - `pub use std::{self as s2};` is `NotFound`;
  - `pub use serde;` + `mod a { use crate::serde::Serialize; impl Serialize for String {..} }` loads;
  - the 2015 and ambiguity fixtures.
- **CA-F7 check:** a file of 20,000 chained local aliases (`use crate::m::T as A0; use A0 as A1; ...`) with `impl A19999 for X` loads or fails typed on a 2 MiB thread, in debug and release, and contains no self-referential import.
- **Entry-point checks:**
  - `this_repository.rs`: `raf_source::syn::File` is `NotFound`.
  - Registry scan: no crate fails with the self-alias message (before: 8).
- **Commit:** `fix(raf-source): resolve extern-prelude aliases and key traits without recursion`.

### R5b: settle import cycles as components (A4, P-F6, checkability)

- **Fix (B7):**
  - When no stuck import is a leaf, compute the strongly connected components of the explicit-import blocker graph.
  - A component whose members are bound only by imports inside it is cyclic, and every name it binds resolves `Cyclic`.
  - A glob never settles a name that an explicit import in that module binds.
  - The settle loop indexes blockers by name (P-F6).
  - A debug assertion re-walks every settled named import against the final glob tables (architecture "Resolver checkability").
- **Check:**
  - The architecture A4 fixture asserts `demo::a::X` and `demo::b::X` are both `Err(Cyclic)`, in both declaration orders.
  - A neighbour without the `b → a` import resolves `demo::a::X` to `demo::c::X`.
  - The permutation test is extended to a fixture that reaches the cycle branch.
  - The `/tmp/perf/src` 32k-import probe is re-run and finishes in under 1 s.
  - The R5a-review shape is also re-run: chains of imports that bind nothing, at 8,000 and 20,000 (about 35 s and over 5 min on R5a). It must settle in linear time.
- **Commit:** `fix(raf-source): settle import cycles as components`.

### R5c: shared glob scope tables (P-F5, S-3, P-F7)

- **Fix (A7):**
  - Glob scope tables are built from shared name sets (`Arc`).
  - When a module's scope changes, only the globs watching that module are woken, which makes a chain linear.
  - A glob-edge budget counted in bytes (512 MiB): each glob binding charges its `size_of` plus its owned strings, giving a typed `Budget` error.
- **Check:**
  - At-limit and over-limit glob-edge fixtures.
  - Re-run probes: S-3 `resolve_cost globs 2000`, the P-F5 4000-glob chain and the P-F7 1500×150 prelude, each recorded within the stated figures.
  - Registry: 0 glob-budget refusals.
- **Commit:** `fix(raf-source): share glob scope tables under a glob budget`.

### R5d: resolve-path validation and missing local globs (P-F9, A12, A13)

- **Fix:**
  - Path segment validation is linear (P-F9). The `shadowed_namespace` scan no longer exists: R4 removed the shadow rule when it grouped resolution by target, so P-F9's second half is closed by R4.
  - `resolve` returns `Invalid` for `crate`, `self`, `super` or `Self` in any position of a canonical path, and the swallowed `Err(Fail::SuperAboveRoot) => {}` arm (`resolve.rs:387`) is removed (A12).
  - A13 / A6-adv: a glob of a missing local module gives a typed `ResolveError` on any lookup that would consult it. The **load still succeeds**. Contract updated (case 3).
  - **S-10, the `ResolveError` part (moved from R10, which may not edit `resolve.rs`):** `ResolveError`'s `Display` (`resolve.rs:160-176`) wraps every user-derived field in `crate::error::Esc`. Test: a control character in a resolved path is escaped in every variant's message, like `tests/error_text.rs`.
- **Check:**
  - `demo::crate::a::x` and `demo::super::x` are `Invalid`, beside `demo::a::x` as `Ok`.
  - A missing-glob fixture: the load succeeds, the lookup is a typed error, and a sibling lookup is `Ok`.
  - The P-F9 50k-segment probe runs in under 100 ms.
- **Commit:** `fix(raf-source): validate resolve paths and refuse unresolvable local globs`.

### R6: one additive parser recursion budget (G1 CRITICAL, P-F11, S-5, S-11)

- **Fix (B9):**
  - **Token set:** derived from an audit of syn 3.0.8's recursive entry points, recorded in the Contract as the counter definition.
    - Expression: unary, trailer (`?`, `.f`, `()`, `[]`, `.await`), binary, cast `as`, range `..`, closure (including `async`/`const`/`static`/`move` heads), `if`, `match`, `while`, `let`, `box`, `return`/`break`/`yield`/`become`, `async`, `const`, `unsafe`, labelled block.
    - Type: reference, pointer, `fn` with return type (including `extern "C"`, `unsafe`), `impl`/`dyn` bound with return type, `for<'a>`.
    - Pattern: `box`, `&`, `ref`, `mut`, range.
  - **Reset rule:** a level's count returns to zero at `;`, `,`, `=>`, and at a `}` closing a block-like expression statement at that level.
  - **Accumulation:** the budget value is the sum, over all currently open delimiter levels, of each level's current count, plus one per open delimiter. A delimiter that closes discards its level's count.
  - **Limit:** taken from the measured debug stack cost per unit (parse plus fact visitor plus drop on the parse thread), for the costliest counted shape, with a factor of at least 2. The figures are written into the Contract.
  - The five existing budgets stay.
  - The carve-outs are deleted.
- **Check:**
  - The overflow harness, in both debug and `--release`: all committed shapes plus new corpus files for P-F11, S-5 and the S-11 `box` shape are `Refused`. The test asserts no `Aborted`.
  - Every counted token has an over-limit fixture that is refused and an at-limit fixture that parses, in `prescan_budgets.rs`.
  - The S-11 80 KB `box` probe runs in under 1 s.
  - This repository and the registry scan show 0 refusals of the new budget kind, and the limit is not raised above the derived value to reach that.
  - `grep -rni 'no budget counts\|can end in such an abort\|can end in that abort' docs/capabilities/source-index.md SECURITY.md` returns nothing. On 6c744af it returns `source-index.md` lines 216 and 519 and `SECURITY.md:77`.
- **Commit:** `fix(raf-source): bound parser recursion with one additive budget`.

### R7: fuzz reach (G8, G14, S-12)

- **Fix:**
  - `parse-source` gets `FUZZ_MAX_LEN=400000`.
  - Every target gets `-rss_limit_mb` and `-malloc_limit_mb`.
  - Bins are renamed to kebab-case (`parse-source`; `parse-document` is already named so from R1).
  - Add a `load-workspace` target that builds a structured `MemoryTree` (manifest plus a few files) from the input and calls `Workspace::load` and `resolve` (S-12).
  - Restore the Verification sentence and remove the 4096-byte text.
- **Check:**
  - `make fuzz` exits 0, running three targets.
  - The `parse-source` `INITED` line shows the full seed corpus (7.5 MB after R6 added its overflow shapes).
  - The output contains no `warning:` line.
- **Commit:** `build(fuzz): reach overflow-sized inputs and fuzz the OKF reader and workspace load`.

### R9: empty roots fail hard (G9, user decision)

- **Fix:** `Workspace::load(&tree, &[])` returns `LoadErrorKind::NoRoots`. The Verification line becomes "no roots is an error" (case 1), and `discovery_targets.rs:271-276` is inverted.
- **Check:** `cargo test -p raf-source --locked --test discovery_targets -- --exact an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads` reports `1 passed`.
- **Commit:** `fix(raf-source): refuse an empty root list`.

### R10: manifest and error-text hardening (S-9, S-10)

- **Fix:**
  - **S-9:** compile the `exclude` patterns once. The `exclude` length cap is 4096 and the `members` cap is 4096, each with a typed `Manifest` error (A8-adv).
  - **S-10:** `LoadError`'s `Display` escapes control characters (`char::escape_debug`) in every user-derived field.
- **Check:**
  - At-limit and over-limit fixtures for both caps.
  - The 880 KB duplicate-pattern probe and a distinct-pattern probe each load or refuse typed in under 1 s.
  - `error.rs::control_characters_in_a_path_are_escaped`.
- **Commit:** `fix(raf-source): harden manifest matching and error text`.

### R11: public API surface (A5, A6, A8, A9, A11, A14, A15, A16, A18)

- **A5 (A18-adv):** the per-file fact types become `pub(crate)`. Integration-test assertions on them move to unit tests, and consumers use `Workspace` accessors.
- **A6:** `Target`, `Package`, `SourceFile`, `Reach`, `Entry` and `SpanFact` become `#[non_exhaustive]`. `Target` gains `harness`, `required_features`, `test` and `doctest`, read from the manifest.
  - Fixture: `discovery_targets.rs::harness_required_features_test_and_doctest_are_read_from_each_target_table`, covering values beside defaults, including `test = false`.
- **A8:** `Outcome` and `LinkTarget` become exhaustive, and the fallback arms at `docs_contract.rs:203, 440` are removed.
- **A9 remainder:** `LoadOptions` drops `Copy` now, so a future per-file hook is not a breaking change. `FileCtx` and `RuleHit` bounds belong to the census-rule-api change (that capability is `designed`).
- **A11:**
  - A `docs_contract` `tests` binding must name a `#[test]` function, and a `code` binding must name a shipping item.
  - Each rule gets a negative fixture.
- **A14 (B12):** the `Tree` trait keeps no `Send`/`Sync` bound. Constraints gain (case 3): "Loading is serial; requiring `Tree: Sync` for a parallel load would be a breaking change to `Workspace::load`."
- **A15 (B13):** `SpanFact` replaces `start_line`/`end_line` with `lines: Range<usize>` (1-based, half-open), matching `raf_okf::Span`. Both Contracts say "line ranges are 1-based and half-open".
  - Test: a one-line item has `lines == n..n+1`.
- **A16 (B15):**
  - `items_under` matches whole segments. Fixture: `demo::a` beside `demo::ab`.
  - The `SourceFile::package` rule is stated: the package of the first target in target order.
- **A18:** add to Constraints: "`raf_source::syn` is re-exported so consumers use the syn version that produced the facts; a syn major bump is a breaking change of raf-source."
- **Sealing check (B14):** for each sealed type, a `compile_fail` doctest destructures it with every current field and no `..`. It compiles on 6c744af and fails once the type is `#[non_exhaustive]`; the reviewer records the 6c744af run.
- **Gates:**
  - `cargo test --workspace --locked`;
  - `cargo doc --workspace --no-deps` with `-Dwarnings`;
  - the A11 negative fixtures.
- **Commits:**
  - `feat(raf-source)!: seal public types, add target test settings and use half-open line ranges`: A5, A6, A9, A14 text, A15, A18 text.
  - `feat(raf-okf)!: make the outcome and link-target enums exhaustive`: A8, including the `docs_contract` arm removal.
  - `fix(raf-source): match items_under by whole segment`: A16.
  - `test(docs-contract): bind tests to test functions and code to shipping items`: A11.

### R12o / R12s: tests, accessors and cleanups

**R12o (raf-okf):**
- **G4 / Q-3:** `an_unknown_or_foreign_key_is_reported_on_every_type`, looping over `DocType::ALL`, plus nested unknown keys (`bindings.extra`, a Change item field).
  - Mutation: widen the Capability key set in `schema.rs`.
- **Q-1:** per-type golden floors, plus three committed independent fixture documents.
- **Q-2:** BOM, NUL and trailing-space fence tests.
- **G17:** `feat(raf-okf)` adds `FencedBlock::content()` and `is_closed()`. `docs_contract` uses them, and a `~~~rust` Usage fixture is accepted.
- **P-F2 / P-F3:** `perf(raf-okf)` indexes `changes` and blocks in `Profile::validate`. A cap probe runs in under 100 ms.

**R12s (raf-source and the repository):**
- **G5:** load-level cap tests in both profiles (exact cap, cap+1, a lowered `max_file_bytes` on a child file), using a comment-padded 8 MiB file (A9-adv).
  - Mutation: pass `u64::MAX` at `modtree.rs:189`.
- **G11:** a real `Spawn` child test under an address-space limit.
- **Q-6:** assert the root has `missing_docs = "deny"`.
  - Mutation: set `"warn"`.
- **Q-7:** non-UTF-8 module file, CRLF and BOM line-number tests.
  - Mutation: drop the BOM strip or CR handling in line counting.
- **G13:** fix the stale comment.
- **G15:** `Found` carries `PackageDecl`.

**Commits:** `test(raf-okf): ...`, `feat(raf-okf): ...`, `perf(raf-okf): ...`, `test(raf-source): ...`, `refactor(raf-source): ...`.

### R13: contract text, licence pins, PR body (G10, G12, S-13, A10, A16-adv)

- Each item's commit carries its own capability-doc edits.
- **A10 (case 3):** the `Workspace::load` Contract gains "`Workspace::load` returns the first error, in the order discovery, targets, module walk, index; it does not collect errors."
- **S-13:** pin the exceptions by version in both `deny.toml` files and `libfuzzer-sys` in `fuzz/deny.toml`. Use the version-spec syntax accepted by the installed cargo-deny (0.20.2; confirmed by `make deny` failing on a deliberately wrong pin, then passing). Keep the `repo_gates` equality gate (updated in step).
- **G10:** closed by the Stage 2 sign-off. `make deny` and `make fuzz` still pass.
- **PR body:** "Decisions that differ" quotes Stage 1 decisions 1–5, the Stage 2 decisions A1, A2 and G9, and the four sign-offs (G12).
- **State rule (A16-adv):**
  - Both capabilities stay `delivered` only if the exit audit passes.
  - Otherwise they move to `implementing`, with the `INVENTORY` edit in the same commit.
- **Commit:** `build(deny): pin the crate-scoped licence exceptions` for S-13. The A10 sentence rides with R11.


### R-CI: findings from the first hosted runs

- **Gap:** run 38361953775 on e472918 failed two jobs.
  - **clippy:** the floating nightly (2026-10-12) flags `map(_).unwrap_or_default()` at `impls.rs:1343` and `index.rs:670`.
  - **test-macos:** APFS refuses to create the non-UTF-8 file name that `disk_tree_containment::a_non_utf8_entry_name_fails_the_listing_beside_a_listing_without_one` writes.
- **Fix:**
  - Update the local nightly to the toolchain CI resolves, and rewrite both expressions the way the lint asks.
  - The test asserts the listing refusal where the filesystem stores the name. On macOS it asserts that creating the name fails, which is why no such entry can be listed there.
- **Check:** on the next push, the hosted `clippy` and `test-macos` jobs pass, with every other job still green.
- **Commit:** `fix(raf-source): pass the current nightly lints and the macOS test job`, made after R5b.
### External: hosted CI (G6)

This depends on account billing. When it is restored: `gh run rerun 38280511329 --repo Blackcat-Informatics/rust-agentic-foundations`, then the final head's run, including the new macOS and Windows jobs. Until then it is an **external blocker**, reported apart from local validation. It is never reported as passed.

### Declined, with reasons

- **A7 / P-F6:** fixed in R5b. **A12:** fixed in R5d. **P-F10:** fixed in R3. **A16:** partly fixed in R11. Kept from A16, with reason: `SourceFile::test` versus `reached_only_through_tests()`, and `test_fns()` returning `Result`. Removing either changes a documented signature with no correctness gain.
- **A9:** `Rule: Sync` is confirmed by the owner. The remainder is handled in R11.
- **A10:** kept as fail-fast, now stated in the Contract (R13).
- **A17:** confirmed by the owner. R4/A3 covers the `build_script` collision.
- **CA scope, `Item::Verbatim` refusals** (`use {::a::b}`, attribute macros; 8 of 1,177 crates): kept as a typed refusal under the approved plan C7 ("`Item::Verbatim` is an error"). Indexing them means guessing a token stream.
- **CA-F6 tail:** the 10 `UnresolvedTrait` crates are not declined. After R5a, the scan re-runs them, and each remaining refusal is root-caused in `validation.md` and either fixed or shown to be correct.
- **HF-R4 (docs_contract rewrite without a differential run):** covered at exit. The exit auditor runs main's `docs_contract` and the branch's over the same negative corpus and records any difference in verdict.
- **HF-D14** (`syn` `printing` + `quote`): kept. `quote::ToTokens` renders written self types (`facts.rs:12`), and a hand-written renderer would duplicate syn's printer. `cargo deny` covers the extra graph surface.

## Exit

- One `make check` runs on the final HEAD.
- The registry scan is re-run, with the before/after tally in `validation.md`.
- The rustdoc-JSON differential (A13-adv) runs: `crate_id == 0` items, with external re-exports expected `NotFound`, as `make rustdoc-diff` on the nightly gate toolchain. It is recorded in evidence, not added to `check`.
- A fresh completion auditor (opus/xhigh) re-runs the CA-F1..F7 demonstrations and every criterion, plus HF-R4.
- The final summary goes to issue and PR, keeping these apart: local validation, hosted CI (external blocker), macOS and Windows runtime (unverified until CI runs), and the user decisions.
