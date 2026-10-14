# Completion audit, Stage 2: issue 2 (raf-okf + raf-source), PR 13

Independent auditor. I did not implement any of this work.

## Reviewed scope and authority

- Repository Blackcat-Informatics/rust-agentic-foundations, worktree
  `.worktrees/2-shared-crates-raf-okf-raf-source`, branch `paudley/2-shared-crates-raf-okf-raf-source`.
- HEAD `6c744af`, comparison base `origin/main` `2c41082` (equal to the merge base). The working
  tree was clean before and after the audit, and I edited, committed and pushed nothing.
- Reviewed diff: `git diff origin/main...HEAD` (13 commits, 144 files).
- Plan (authoritative): `$STAGE/plan.md`, including its five user decisions.
- Issue and discussion: `$STAGE/issue.md` (body plus 13 comments), `raw/issue-2-comments.json`,
  `raw/pr-13*.json`.
- Context layers: AGENTS.md, CLAUDE.md, SECURITY.md, `docs/capabilities/okf-profile.md`,
  `docs/capabilities/source-index.md` and `docs/architecture/okf-profile.md`. The repository
  has no `.goals`, `.baseline`, `CONSTITUTION.md` or ADRs (checked).
- `.deficiencies`, at the worktree root and in the main checkout, holds only its notice and
  marker. It has no entries.
- Prior reports read: `delivery-checklist.md`, `validation.md`, `tasks/T1-T9-review.md`,
  `issue-analysis.md` (Stage 2 section) and `historical-fit.md`. Two Stage 2 reviews written
  in parallel, `reviews/architecture-review.md` and `reviews/quality-review.md`, appeared during
  this audit. I used them only as a cross-reference.
- Execution surface: local cargo and make in the worktree, plus scratch programs outside the
  repository that depend on both crates by path:
  - `/tmp/raf-audit/src/main.rs` and `/tmp/raf-audit/src/bin/*.rs`;
  - rustc control fixtures in `/tmp/raf-audit-rustc/`;
  - a cargo-buildable control crate in `/tmp/raf-audit-modrs/`;
  - docs snapshots in `/tmp/raf-snap-{fence,nope,real}`, created with `git checkout-index`.
- Limits:
  - Hosted CI is **unavailable**: account billing blocked every job. It is not counted as
    passed.
  - No macOS or Windows host was available, so the non-Linux containment branches were checked
    only to the evidence bar that user decision 3 sets (compiled for both targets, verifiers
    unit-tested on Linux).
  - I ran no live service and no forge mutation.

## User scope decisions applied (quoted from `plan.md`)

1. "**Extend the profile with `tags` and `timestamp`.** Both are optional keys on all three
   document types. `resource` stays rejected." This revises the issue's line that "`title`,
   `description`, `resource`, `tags` ... and `timestamp` follow the base OKF v0.1 shape". The
   deliverable is therefore optional `tags` and `timestamp`, with `resource` refused.
2. "**No default roots in `raf-source`.** `Workspace::load` takes explicit roots, and each
   consumer owns its own default." This replaces the issue's "(default `crates/`)".
3. "**Containment race: closed on Linux, with a portable fallback elsewhere.** ... The Contract
   and the SECURITY.md Containment bullet state the guarantee for each platform. A residual
   rename race exists only on non-Linux."
4. "**Links split across lines are refused.**" This adds a requirement.
5. The port source is "the superset of every precedent implementation, taking the best part of
   each, with licensing ignored". This replaces the issue's provenance clause, so no
   provenance check is required.

## Evidence reused and evidence run fresh

- **Reused.** `raw/make-check.log` is `make check` on `6c744af` (EXIT 0, 963 tests). It still
  applies: HEAD is unchanged, the tree is clean, and no gate input changed.
- **Fresh runs in this audit** (HEAD `6c744af`; each exited 0):
  - `cargo test --workspace --locked` passed 604 tests with 0 failures and 1 ignored (the corpus
    writer). That includes `docs_contract` 137 and `repo_gates` 55.
  - `make fuzz` reported "Done 5000 runs in 4 second(s)".
  - `make cross-check` built `x86_64-apple-darwin` and `x86_64-pc-windows-gnu`.
  - `make clippy` passed, as did `make doc` (`RUSTDOCFLAGS=-Dwarnings`).

## Criteria: demonstrations through the public entry points

Status key:

- **D**: demonstrated.
- **F**: failing.
- **U**: unverified.

### Issue scope, raf-okf

| # | Requirement | Demonstration (command and input) | Observation | Status |
|---|---|---|---|---|
| K1 | Parse Markdown with YAML frontmatter, with `type` required. Base keys follow decision 1. | `cargo run --bin raf-audit -- repo-okf` parses every tracked `docs/capabilities`, `docs/architecture` and `work/changes` document. The suite test `profile::a_missing_type_key_is_the_only_violation` covers the missing `type`. | 42 documents parsed and validated to typed values; 2 capabilities are `Delivered`. On real documents of all three types, `resource:` gives `ResourceKey` and names the closed key set. `tags: [b, a]` with a valid timestamp validates. `tags: [a, a]` gives `DuplicateTag`. Month 13 gives `BadTimestamp{OutOfRange month}`. | D |
| K2 | Closed key set per `type` with typed values; an unknown key is a hard error. | Same run: `extra: 1` injected into the real okf-profile capability, the census-cli change record and the okf-profile architecture page. | Each gives exactly `UnknownKey{extra}` at key `extra`. | D |
| K3 | Deterministic writer: round-trip stable, sorted where the spec says so. | Same run, plus `mdcorpus` over 4003 Markdown files in the local cargo registry. | All 42 repository documents satisfy `render == text` and `parse(render) == doc`. Unsorted tags render as `[a, b]` and re-render byte-identically. The 24 registry files with frontmatter round-trip with 0 mismatches. | D |
| K4 | Body model: H2 sections with byte spans, links, fenced code, generated blocks and PROPOSED blocks. | `repo-okf` and `okf-edge`. | The okf-profile capability gives 7 sections, 3 links and 2 fences. A generated block is spliced through `replace_spans`. PROPOSED in an Architecture page is refused at line 14. Markers inside a fence give 0 blocks. A split link is recorded at line 3. 4003 registry Markdown files gave 0 panics and 0 errors. | D |
| K5 | An `index.md` without frontmatter is navigation, not a document. | `okf-edge`. | `Index::parse` accepts a generated-only index with an SPDX header (1 block). `Document::parse` of the same text fails at line 1. Added prose is refused at line 5. `is_capability_index` is true only for `docs/capabilities/index.md`. | D |

### Issue scope, raf-source

| # | Requirement | Demonstration | Observation | Status |
|---|---|---|---|---|
| S1 | `Tree` trait (`read`, `list`) with Disk and Memory implementations. | `cargo run --bin tree` on this repository. | `list(crates)` is sorted. `read` respects the cap. A missing file is `NotFound`. The `target` directory symlink is listed as `Symlink`, and listing it is refused (`OutsideRoot`). `MemoryTree` lists inferred directories. | D |
| S2 | Package discovery under explicit roots (decision 2); lib, bin and build.rs roots. | `repo-source`, `repo-source-dot`, `memory`; `Workspace::load(&t, &[])`. | Root `crates` finds 2 packages and 36 targets. Root `.` finds 4 packages, including the root and fuzz packages. Empty roots find 0. The Memory fixture gives lib, bin, `src/bin/tool.rs` and `build.rs` with the documented crate paths and shipping flags. | D |
| S3 | Mod-tree following as rustc does, including `#[path]`. | `memory`, `pathchild*`, `modrs` and `order*`. | The file reach (which file belongs to which module) is correct. **The item paths built from it are wrong whenever a module file is indexed before its parent module exists**, which covers the common `a/mod.rs` + `a/b.rs` layout. A crate that `cargo build` accepts fails to load with `SuperAboveRoot`. See **F2**. | F |
| S4 | `#[cfg(test)]` and `#[test]` items are tagged, not dropped. | `repo-source`. | This repository: 51 items `CfgTest` and 407 `TestFn`. `raf_source::tree::disk::tests::swap::...` resolves with `test=TestFn`, `shipping=false`. | D |
| S5 | Test, bench and example targets are non-shipping (`<pkg>::test_crate::<target>` and the like). | `memory` and `repo-source`. | `demo::test_crate::it::case`, `demo::bench_crate::b::bench`, `demo::example_crate::e::helper` and `demo::example_crate::e` resolve as non-shipping. `raf_okf::test_crate::profile::an_unknown_key_is_reported_at_that_key` resolves to `tests/profile.rs:170` in a non-shipping target. | D |
| S6 | Symbol index of every named item, plus the `use`-alias table, so re-exported paths resolve. | `repo-source`, `dep*`, `modrs`, `enumglob`, `valid`, `realcrate`. | Re-export chains do resolve (`raf_source::DiskTree` follows the chain `[raf_source::DiskTree, raf_source::tree::DiskTree, raf_source::tree::disk::DiskTree]`, and a three-hop Memory chain gives `via_glob=true`). **But:** wrong item paths and false-positive resolutions (**F2**); `use <crate>;` resolved against itself (**F1**); load refusals of rustc-valid code (**F5**, **F6**); a process abort on the caller's stack (**F7**). | F |
| S7 | Port from the precedents, with no purrdf dependencies. | `crates/raf-source/Cargo.toml` dependencies are `syn`, `quote`, `toml`, `proc-macro2` and unix-only `libc`. | No purrdf dependency. Provenance was replaced by decision 5. | D (per decision 5) |
| S8 | Path-traversal safety: `#[path]` cannot escape the workspace root. | `disk-escape` on a real `DiskTree`. The sentinel `../outside.rs` is mode 000, so any open of it would surface as an `Io` error rather than `OutsideRoot`. A decoy `outside.rs` sits inside the root. | Each case is refused with `Tree(OutsideRoot)` before any open, naming the module and attribute: `../../outside.rs`, an absolute path, `vendored/../outside.rs` through a directory symlink, the same inside an inline module, and `cfg_attr(unix, path=..)`. A symlinked module file and `[[bin]] path = "../outside.rs"` are refused naming `mod inner;` and `bin.path`. The in-root redirect `#[path="other/x.rs"]` loads and resolves. | D |

### Acceptance criteria

| # | Criterion | Demonstration | Observation | Status |
|---|---|---|---|---|
| AC1 | Fixture tests on `Memory` trees cover nested mods, `#[path]`, re-export chains, test/bench/example targets and a rejected `#[path]` escape. | `cargo test -p raf-source` (in the 604), plus my own Memory fixture (`memory`). | The fixtures exist and pass. My Memory fixture reproduces each covered shape. The nested-module fixtures never index a module file before its parent, so they miss **F2**. | D (fixtures); the gap is filed under F2 |
| AC2 | OKF round-trip golden tests; an unknown-key fixture fails. | `render::every_tracked_document_renders_to_its_own_bytes`, `profile::an_unknown_key_is_reported_at_that_key`, and my K2/K3 runs. | Passes. Unknown keys are refused on all three types in my run. | D |
| AC3 | Public APIs documented (`missing_docs = deny`). | Fresh `make clippy` and `make doc`. Both crates have `[lints] workspace = true`, and `repo_gates::lints_every_workspace_manifest_inherits_the_workspace_lints` passes. No `allow(missing_docs)` exists. | Exit 0. | D |
| AC4 | `cap:okf-profile` and `cap:source-index` docs match what was built, with state `implementing` or `delivered`. | `docs_contract` (137 passing, including `repository_bindings_resolve_in_the_source_index`), my binding replay (`repo-source`: 117 bindings resolve), and the docs snapshots run through the shipped gate. | Both capabilities are `delivered`, the change records are deleted, and every binding resolves. **But** several Contract sentences describe behaviour the build does not have (**F1**, **F2**, **F5a**, **F6**). The listing guarantee is overstated for non-Linux (**F3**). Verification promises fixtures that do not exist (**F4**). The shipped binding gate accepts the non-existent binding `raf_okf::body::FenceBlock` (**F2**). | F |

### Plan, completeness contract and decisions (beyond the issue text)

| # | Requirement | Demonstration | Observation | Status |
|---|---|---|---|---|
| P1 | Decision 4: split links refused, and `docs_contract` applies the rule to every tracked file. | `okf-edge`, `repo-okf` (12 non-profile Markdown files read by `Body::parse`, none with a split link), and suite tests. | Refused at the right line. | D |
| P2 | Size cap: 8 MiB default, `LoadOptions`, error names the file. | `cap8` through `DiskTree` and `Workspace::load`. | A source of exactly 8 MiB loads (0.42 s) and resolves `demo::first`. 8 MiB + 1 gives `TooLarge` naming `src/lib.rs`. A lowered cap is honoured. | D, but the suite lacks this fixture (F4) |
| P3 | Pre-scan budgets, dedicated 64 MiB parse thread, `Spawn` error. | Suite tests (debug, and release in the reused log). For `Spawn`: `bash -c 'ulimit -v 60000; spawn'`, calling `parse_source` through the public API. | `LoadErrorKind::Spawn`: "src/lib.rs: cannot start the parse thread: Resource temporarily unavailable". It parses at 100000 KiB. Parser bounds hold. Index building is unbounded on the caller's stack (F7). | D (parse) / F (index stack, F7) |
| P4 | Fuzz target and corpus replay. | Fresh `make fuzz`; `corpus_replay` in the 604. | 5000 runs, exit 0. Replay passes. | D |
| P5 | Decision 3: containment per platform and `make cross-check`. | Fresh `make cross-check`; the swap and verifier tests in the suite. My `disk-escape` run covers Linux. | Linux: demonstrated. Other platforms: compiled, with verifiers unit-tested on Linux, which is the bar decision 3 sets. Runtime on those platforms is not available here. The docs overstate the listing guarantee (F3). | D (Linux) / U (non-Linux runtime, by decision) |
| P6 | `docs_contract` reads through raf-okf and resolves bindings through raf-source. | Code read of `tests/docs_contract.rs:26-30, 695-718, 908-922`, plus the snapshot runs below. | The gate is live and not vacuous: an injected `raf_okf::Nope` fails it naming the binding. But it passes `raf_okf::body::FenceBlock`, which does not exist (F2). | D (wiring) / F (accuracy, F2) |
| P7 | Whole qualification: `make check`. | Reused log plus fresh subsets. | EXIT 0. Hosted CI is unavailable (billing), not passed. | D (local) |

## Open findings

### F1 (HIGH). `use <crate>;` is resolved against itself: a false `Cyclic` result and false load refusals

- **Where:**
  - `crates/raf-source/src/settle.rs:536-560` (`settle_named` walks the import's own path in its
    own module, so the head is looked up in a scope that holds the import being settled);
  - `crates/raf-source/src/impls.rs:159-185` (`external_prefixes` recurses on the module's own
    `use <name>` declaration until the depth cap, then fails);
  - the result surfaces as `ResolveError::Cyclic` at `crates/raf-source/src/resolve.rs:429`.
- **Evidence:**
  - **This repository:** `ws.resolve("raf_source::syn::File")` returns `Err(Cyclic(..))`
    (`raf-source/src/lib.rs` has `pub use syn;`). `serde::Serialize` gives `NotFound`, as it should.
  - **Memory, edition 2021 (`dep2`):**
    - `pub use serde;` makes `demo::serde` and `demo::serde::Serialize` resolve to `Cyclic`.
    - `pub use serde::{self, Serialize};` makes even `demo::Serialize` `Cyclic`.
    - `pub use ::serde;` and `pub use serde as s;` correctly give `NotFound`.
  - **DiskTree, editions 2018, 2021 and 2024 (`dep4`):**
    - `use std; pub struct Foo; impl std::fmt::Display for &Foo {..}` fails `Workspace::load`
      with `UnresolvedTrait { path: "std::fmt::Display" }`. Exit 0 under `rustc --edition
      {2018,2021,2024}`, with no warnings.
    - `pub use std; pub mod m { pub use std; } pub use std::{self as s2};` loads, but every path
      through those names is `Cyclic`.
  - **Real crates:** each of these fails to load with `UnresolvedTrait`. Each uses the pattern
    `pub use dep::{self, ..}`, `pub use dep;` or `use serde;`:
    - `hmac-0.13.0`, `ripemd-0.1.3` and `sha3-0.11.0` (`pub use digest::{self, ..}`);
    - `p256-0.13.2`, `p384-0.13.1` and `p521-0.13.3` (`pub use elliptic_curve::{self, ..}`);
    - `cx448-0.1.1` (`pub use elliptic_curve;`);
    - `rmp-serde-1.3.1` (`use serde;`).

    `rustls-0.23.45` aborts the process instead (see F7).
- **Consequence:** `source-index.md:456-460` says `NotFound` covers "an item of a dependency",
  and that `Cyclic` applies only "when the imports on the way depend on each other". Lines
  351-363 say a dependency trait key is found "through the module's own `use` declarations",
  with the head a sysroot or declared crate. The index breaks both promises on this
  repository's own code, and refuses valid crates outright.
- **Required fix:**
  - When an import's first segment is settled or canonicalised, the import itself must not
    count as a binding of that name. If nothing else binds it, the name falls back to a crate
    the package can name (the extern prelude, or the sysroot).
  - Add fixtures, each beside a neighbouring case that resolves:
    - `pub use serde;` and `pub use serde::{self, Serialize};` give `NotFound`;
    - `use std;` plus a trait impl on `&Foo` loads, with key `std::fmt::Display`;
    - this repository's `raf_source::syn::File` gives `NotFound`.

### F2 (HIGH, blocking). A stale module index puts items under the wrong module path, so non-existent paths resolve and valid crates fail to load

- **Where:** `crates/raf-source/src/index.rs:249-271`, `Builder::ensure`.
  - `let idx = self.index.modules.len();` (line 253) is taken **before** the recursive
    `self.ensure(prefix, None)` (line 260) pushes the missing parent.
  - The child's path is then registered under that stale `idx` (line 269), which is the
    parent's slot.
  - The bug triggers in `Index::build` (`index.rs:498-507`) whenever, in sorted file order, a
    module file comes before the file that declares it: `src/a/b.rs` before `src/a/mod.rs`
    (any child sorting before `mod`), `#[path]` children, and `tests/support/child.rs` before
    `tests/workspace_containment.rs`.
- **Evidence:**
  - **This repository (`order2`, `order3`):**
    - All 27 items of `crates/raf-okf/src/body/blocks.rs` are recorded as `raf_okf::body::*`, for
      example `raf_okf::body::FenceBlock`. rustc's path is `raf_okf::body::blocks::FenceBlock`.
    - 15 items of `crates/raf-source/tests/support/child.rs` reached from `workspace_containment`
      are recorded as `...::workspace_containment::support::*` instead of `...::support::child::*`.
  - **Shipped gate (docs snapshot run):** with `raf_okf::body::FenceBlock` added to the
    okf-profile bindings, `RAF_GATE_ROOT=/tmp/raf-snap-fence cargo test --locked --test
    docs_contract repository_bindings_resolve_in_the_source_index` **passes**. With
    `raf_okf::Nope` it fails, naming the binding. So the "bindings never name a missing symbol"
    rule admits a symbol that does not exist.
  - **Minimal Memory tree (`modrs`):** `src/a/mod.rs` (`pub mod b; pub fn in_a(){}`) plus
    `src/a/b.rs` (`pub fn in_b(){}`) records `demo::a::in_b`. Both `demo::a::in_b` and
    `demo::a::b::in_a` resolve `Ok`, and neither exists.
  - **Control crate `/tmp/raf-audit-modrs`:** `cargo build` exits 0. `Workspace::load(DiskTree)`
    fails with `SuperAboveRoot { path: "super::super::top" }` at `src/a/b.rs:1`, a false refusal
    of a compiling crate.
  - **Isolation (`pathchild3`):** with `#[path = "gen/generated.rs"]`, children `a`, `b`, `c`,
    `ch`, `child`, `d`, `f`, `ga`, `gen` and `gem` are misplaced, while `h`, `z` and
    `generated2` are correct. That is, the outcome depends on file-name order.
  - **Real crates (registry survey):**
    - 164 of the 1014 crates that load carry misplaced items: 3057 items in all, for example in
      `aes`, `askama`, `arc-swap` and `anndists`.
    - Three fail to load with a false `SuperAboveRoot`, each a `mod.rs` child that sorts before
      `mod.rs`: `brotli-8.0.4`, `src/enc/backward_references/mod.rs:8`;
      `structured-zstd-0.0.49`, `src/decoding/block_decoder.rs:1`; and `wasmi-0.40.0`,
      `src/func/caller.rs:1`.
- **Consequence:** the headline promises fail, all of them in the Contract (`source-index.md`):
  - "full `path`" (line 317);
  - "mod-tree following ... as rustc resolves them" (lines 266-291);
  - "`NotFound` when the workspace defines nothing at the path" (line 456).

  Item paths, scopes, `super`, glob visibility and binding checks are all built on the
  corrupted module table. The determinism tests pass only because the corruption is
  deterministic.
- **Required fix:**
  - Allocate the child index after the parent exists, by moving `let idx = ...len()` below the
    parent resolution.
  - Add fixtures that load first-before-parent layouts and assert item paths:
    - `a/mod.rs` + `a/b.rs`;
    - a `#[path]` file with a child sorting first;
    - a shared `tests/common/mod.rs` + `tests/common/a.rs` reached from a target sorting after
      `tests/common`;
    - `super::super` from such a child.
  - Add a negative assertion that `demo::a::in_b` is `NotFound`.
  - Add a repository assertion that `raf_okf::body::blocks::FenceBlock` resolves to itself.

### F3 (MEDIUM). Containment docs overstate the listing guarantee on non-Linux

- **Where:**
  - `docs/capabilities/source-index.md:133-136` says a listing "reads the entries through the
    handle, so a swap between the check and the listing cannot list a directory outside the
    root", with no platform qualification;
  - `SECURITY.md:57-58` says "The opened handle, for a file read and for a directory listing
    alike, is verified".
- **Code:** `crates/raf-source/src/tree/disk.rs:267-289`:
  - **Linux** lists `/proc/self/fd/<fd>` (true to the claim).
  - **Other Unix** verifies the handle, then lists `canonical` by path through `fs::read_dir`
    (lines 281 and 137). A swap after verification is listed.
  - **Non-Unix** opens no directory handle at all (lines 286-289). `verify_portable_directory`
    is given path metadata through a parameter named `on_handle`.
- **Consequence:** decision 3 accepts a residual race on non-Linux, but it requires the Contract
  and SECURITY.md to state the guarantee per platform. As written, both state a guarantee for
  listings that macOS and Windows do not have (AC4).
- **Required fix:** either list through the handle on other Unix too, or restate the listing
  guarantee per platform in both files, naming the residual window. Rename or retype the
  `on_handle` parameter.

### F4 (LOW). Verification promises fixtures that do not exist

- **`okf-profile.md:307-308`** promises "A validation fixture per type for ... each extra key".
  Only Architecture has one: `crates/raf-okf/tests/profile.rs:170` and
  `tests/docs_contract.rs:1078`.
- **`source-index.md:575-577`** promises that "a file over `max_file_bytes` fails with an error
  that names the file ... and a file of 8 MiB parses at the default".
  - For Rust source, no fixture loads or parses an 8 MiB `.rs` file, and none refuses an
    over-cap `.rs` file through `Workspace::load`.
  - `read_cap.rs` exercises `Tree::read` only. The manifest fixtures cover `Cargo.toml` only.
- **Behaviour:** correct in both cases. My `repo-okf` run refused an extra key on all three
  types, and my `cap8` run loaded 8 MiB, refused 8 MiB + 1 naming `src/lib.rs`, and honoured a
  lowered cap. The two-way checklist in `delivery-checklist.md` nonetheless records these
  sentences as mapped.
- **Required fix:** add the three fixtures, or reword the sentences.

### F5 (HIGH). A macro invocation in a trait body aborts the load

- **Where:** `crates/raf-source/src/facts.rs:888-929`. In `trait_member`, the `other =>` arm
  turns `syn::TraitItem::Macro` into `ExtractError::Unsupported`. The `impl` arm (`facts.rs:967`)
  and the `extern` arm (`facts.rs:874`) skip macro invocations.
- **Evidence:**
  - `macro_rules! decl { () => { fn f(&self); } } pub trait T { decl!(); }`: exit 0 under
    rustc 2021. `Workspace::load` fails with `unsupported item: decl ! () ;`.
  - Real crates fail the same way. For each I checked that the refused line sits inside a
    `trait` body:
    - `rayon-1.12.0` (`trait Try`) and `rayon-core-1.13.0` (`trait ThreadSpawn`);
    - `tokio-util-0.7.20` (`trait FutureExt`) and `time-0.3.55` (`trait ComponentProvider`);
    - `wasmparser-0.259.0` (`trait VisitOperator`) and `crossterm-0.29.0` (`trait Stylize`);
    - `owo-colors-4.4.0`, `yansi-1.0.1`, `rmp-0.8.15` and `interprocess-2.4.4`;
    - `minijinja-2.21.0`;
    - `matrix_indexed_db_futures-0.7.0` and `bincode-1.3.3`;
    - `cranelift-assembler-x64-0.133.1`.
- **Consequence:** `source-index.md:325-326` says "A macro invocation that defines items is not
  expanded, so what it generates is not indexed". In a trait body it is instead a hard error,
  inconsistent with the impl-body rule.
- **Required fix:** skip `TraitItem::Macro` as `ImplItem::Macro` is skipped, with a fixture
  beside an impl-body macro.

### F6 (HIGH). The trait-key lookup refuses any module that globs a non-module local item

- **Where:** `crates/raf-source/src/impls.rs:199-231`. A glob whose target is not a local
  module, such as an enum's variants, falls into the "dependency glob" branch.
  `absolute(..)` then returns `Err(None)` and the load fails with `UnresolvedTrait`.
- **Evidence:**
  - `/tmp/raf-audit-rustc/e.rs`, with `pub enum Inner { A } use self::Inner::*;` and
    `impl PartialEq<Method> for str {..}`: exit 0 under rustc 2021. `Workspace::load` fails with
    "the trait `self::Inner::*` of an impl cannot be named by an absolute path".
  - Real crates that fail to load the same way, each with a glob of an enum or other
    non-module item:
    - `http-1.5.0` (`src/method.rs:19`, `use self::Inner::*`);
    - `itertools-0.14.0` (`crate::EitherOrBoth::*`);
    - `ena-0.14.4` (`self::UndoLog::*`);
    - `combine-4.6.8` (`crate::error::ParseResult::*`);
    - `wasm-bindgen-test-0.3.75` (`self::Label::*`).
  - Ten more `UnresolvedTrait` refusals in the survey I did not root-cause: `asn1-rs`, `jni`,
    `proptest`, `flate2`, `option-ext`, `cranelift-codegen`, `io-lifetimes`, `petgraph`,
    `tungstenite` and `wasm-bindgen`. Some of them glob local modules and may be F2 fallout.
    They should be re-run after F1, F2 and F6 are fixed.

### F7 (HIGH). Trait-key canonicalisation recurses on the caller's stack to a depth that grows with the input, so valid input aborts the process

- **Where:** `crates/raf-source/src/impls.rs:159-170` and `248-260`. `external_prefixes` and
  `absolute` call each other with the depth cap `depth > self.uses.len() + 8`, so the allowed
  recursion grows with the number of `use` declarations. It runs on the caller's thread
  (`Index::build` -> `attach_impls` -> `trait_key`), not on the 64 MiB parse thread.
- **Evidence:**
  - `/tmp/raf-audit-rustc/many.rs` is `use std;`, a `Display` impl on `&Foo` and 20000
    `use std::collections::HashMap as H{i};` lines (809 KB). It is under every budget and
    `rustc --edition 2021` exits 0.
    - At n=10 and n=2000, `Workspace::load` returns `UnresolvedTrait`. That is wrong (F1) but
      typed.
    - At n=20000 it prints "thread 'main' has overflowed its stack / fatal runtime error: stack
      overflow, aborting", in **both** debug and release (`overflow` bin).
  - **Real crate:** `realcrate rustls-0.23.45` aborts the same way in a debug build. gdb shows
    4826 frames alternating `external_prefixes`/`absolute` under
    `trait_key <- attach_impls <- Index::build <- Workspace::load`. The release build returns a
    typed `UnresolvedTrait` (`src/crypto/aws_lc_rs/hpke.rs:785`), which is itself an F1/F6-class
    false refusal.
- **Consequence:**
  - The Contract and SECURITY.md bound recursion on the caller's stack: module nesting is
    capped at 128 "because the walk recurses on the caller's stack". They describe the only
    residual abort as a parser overflow, for shapes no budget counts.
  - Here an ordinary, budget-compliant file aborts the process deterministically outside the
    parse thread. A consumer such as `cargo test` (2 MiB test threads) or a census run over a
    large crate dies rather than receiving a typed error.
- **Required fix:**
  - Make the canonicalisation iterative, or bound its recursion by a fixed, stack-safe constant
    that returns a typed error.
  - Remove the self-reference that drives it (F1).
  - Add a fixture: the 20000-`use` file loads or fails typed on a 2 MiB thread, in debug and in
    release.
- **Consequence:** `source-index.md:403-405` limits this error to "a glob whose first segment is
  neither a local module nor a crate the package can name". Here the first segment is `self`.
  Lines 365-367 also send an unbound single-segment trait name such as `PartialEq` to the
  prelude. An enum-variant glob cannot provide a trait, yet it aborts the whole load.
- **Required fix:**
  - A glob whose path resolves to a local non-module item (an enum's variants) contributes no
    trait candidate; the name falls through to the prelude.
  - Add a fixture with `use self::Inner::*;` beside a module without the glob.

## Scope findings for the owner (documented design, not counted as built-but-broken)

- **Load refusals of valid Rust that the Contract states deliberately.** Two shapes:
  - An inherent impl whose self type the index cannot see aborts the whole load
    (`source-index.md:344-346`). That covers a type generated by `pin_project!` or `bitflags!`,
    and `impl dyn LocalTrait { .. }` (`/tmp/raf-audit-rustc/d.rs`, which rustc accepts).
  - "Symbol index of every named item" therefore delivers no index for such workspaces.
- **Scale.** In the registry survey below this is the largest refusal class, at 98 of 1177
  crates. The parallel architecture review raises the same point as A1 and asks for an owner
  decision. I independently corroborate it, and add the `impl dyn LocalTrait` shape, which
  needs no macro (`ascii-canvas-4.0.0`, `chromiumoxide_cdp-0.9.1`).
- **Refused by plan design.** `use {::a::b}` (syn `Item::Verbatim`) is refused under plan C7.
  rustc accepts it (`macro_rules_attribute`, `ml-kem`, `zerocopy-derive`). The owner may want
  to revisit this together with the inherent-impl refusal.
- **Minor.** For a default library root, the load-error origin reads "reached by manifest key
  `lib.path`" although the manifest has no such key (`valid`, `cap8` runs). The fuzz harness
  build prints cargo's `non_kebab_case_bins` warning.

## Registry survey (representative real input)

`realcrate` loaded the newest version of every crate in the local cargo registry, one process
each, with `Workspace::load(&DiskTree::new(dir)?, &["."])`. The results:

The survey covered 1177 crates, using the debug build with a 60 s timeout per crate. Raw
results are in `/tmp/raf-audit-real.txt`.

| Outcome | Crates | Notes |
|---|---|---|
| Loaded | 1014 | 164 of them carry items under a wrong module path (3057 items in all). This is F2. |
| Inherent impl on a self type the index cannot see | 98 | Macro-generated types and `impl dyn Trait`. This is the documented design and needs an owner decision. |
| `UnresolvedTrait` | 23 | 8 are F1 (`{self}` and `use dep;` re-exports) and 5 are F6 (enum-variant globs). 10 were not root-caused. |
| `UnsupportedItem` | 22 | 14 are F5 (a macro invocation in a trait body). 4 are `use {::a::b}` groups (syn `Verbatim`, refused by plan design). 4 are attribute-macro input (`#[ref_cast_custom]` signatures without a body, and `typst-library`). |
| `mod` with no file | 7 | Test-only modules that the published package omits. This is a correct refusal under the Contract, since no `cfg` is evaluated. |
| `SuperAboveRoot` | 3 | False refusals, all F2 (`brotli`, `structured-zstd`, `wasmi`). |
| Other refusals | 8 | 2 target-root collisions, 2 missing target files, 2 `#[path]` outside the crate directory (correct), and 2 syntax errors in old-edition code (`fxhash`, `signal-hook-registry`). |
| Process abort | 1 | `rustls-0.23.45`: a stack overflow on the caller's thread (F7). |
| Timeout over 60 s (debug) | 1 | `web-sys-0.3.106` (1727 `.rs` files): still running at 300 s in debug. In release, after 6 min 30 s, it ends in the inherent-impl refusal, at `src/features/gen_AddEventListenerOptions.rs:56`, a `wasm_bindgen`-generated type. The Contract sets no time bound. |

The 161 refusals plus the abort mean that 13.8% of these real crates cannot be indexed at
all. Of the crates that load, 16% have wrong symbol paths. The capability exists so that two
tools can index real workspaces. Of the refusals, at least 33 are rustc-valid code refused by
the defects above, rather than by design: 8 from F1, 5 from F6, 14 from F5, 3 from F2, plus
the F7 abort.

## Verdict

The raf-okf scope is complete and demonstrated on this repository's documents and on 4003
real Markdown files. raf-source discovery, targets, containment, bounds, fuzzing and gates are
demonstrated.

The symbol index and alias resolver, which carry the issue's headline "so re-exported paths
resolve", are not correct:

- on this repository, item paths are wrong, and the shipped binding gate accepts a
  non-existent symbol (F2);
- a dependency re-export resolves to `Cyclic` (F1);
- rustc-valid crates are refused at load (F1, F5, F6);
- a valid file aborts the process on the caller's stack (F7).

AC4 also fails, because the Contract describes behaviour that was not built (F1-F7).

Remaining work:

- fix F1, F2, F5, F6 and F7 with the named fixtures;
- correct the listing guarantee in source-index.md and SECURITY.md (F3);
- add or reword the three Verification fixtures (F4);
- then re-run `make check`, the docs-contract binding gate and this registry survey.

The owner decision on load refusals of macro-generated and `dyn` self types is open alongside
this.

VERDICT: FINDINGS-OPEN
