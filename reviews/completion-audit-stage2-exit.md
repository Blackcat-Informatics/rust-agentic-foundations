# Completion audit, Stage 2 exit: issue 2 (raf-okf + raf-source), PR 13

Current status: VERDICT: PASS (Recheck on HEAD `5bcd741`, at the end of this file).

I am an independent auditor and did not implement any of this work. The audit was
time-boxed: it started at 00:29 and the report was written before 01:45 local time on
2026-10-14.

## Reviewed scope and authority

- **Code under review.** Repository Blackcat-Informatics/rust-agentic-foundations, worktree
  `.worktrees/2-shared-crates-raf-okf-raf-source`, branch `paudley/2-shared-crates-raf-okf-raf-source`.
  - HEAD is `bf08e45`, which is also the head of PR 13. The base is `origin/main` `2c41082`,
    which is also the merge base.
  - The diff is 37 commits, 223 files, +55599/-1889.
  - The tree was clean before and after the audit. I edited, committed and pushed nothing that
    is tracked.
- **Inputs read:**
  - `issue.md` (the body plus its acceptance list) and `plan.md` (the Stage 1 plan and its 5
    decisions);
  - `remediation-plan.md` (R1–R13, R-CI) and `validation.md`, which holds every Stage 2 user
    decision verbatim (A1, A2, G9, the sign-offs, the token budget, R8, the merge scope) and the
    orchestrator's R5b acceptance;
  - `tasks/R*-review.md` (all PASS), `gap-analysis.md` and the prior audit
    `reviews/completion-audit-stage2.md` (CA-F1..F7);
  - `docs/capabilities/okf-profile.md`, `docs/capabilities/source-index.md`, `SECURITY.md` and
    `AGENTS.md`.
- **`.deficiencies`.** The worktree root and the main checkout both hold only the notice and the
  marker. There are no entries.
- **Execution surface:**
  - local cargo and make in the worktree;
  - a scratch program `/tmp/raf-exit/probe`, which depends on both crates by path and is built
    into the worktree's target dir;
  - doc snapshots `/tmp/raf-exit/snap-*`, made with `git checkout-index`;
  - on-disk fixtures in `/tmp/raf-exit/esc`;
  - rustc control files in `/tmp/raf-exit/rustc`;
  - the committed registry harness.

  The shared host ran with `CARGO_BUILD_JOBS=6`.
- **Limits:**
  - The pinned toolchain has no Apple, FreeBSD or Windows targets, so local `make cross-check`
    cannot run. I did not change toolchains. Hosted CI's `cross-check` job on `bf08e45` is the
    evidence.
  - I did not run `make msrv` locally, because the 1.98.1 toolchain is not provisioned and I
    did not change toolchains. Hosted `msrv` on `bf08e45` is the evidence.
  - There is no macOS or Windows host. Native runtime evidence comes from the hosted
    `test-macos` and `test-windows` jobs. FreeBSD is compiled only, as the Contract states.

## Fresh runs (HEAD `bf08e45`)

| Command | Result |
|---|---|
| `cargo test --workspace --locked` | EXIT 0. 838 passed, 0 failed, 1 ignored (the corpus writer), across 54 suites. Includes `docs_contract` 141 and `repo_gates` 57 (the licences and issue-refs gates). |
| `make test-release` | EXIT 0. 466 passed. |
| `make clippy doc` | EXIT 0 (`-D warnings`, `RUSTDOCFLAGS=-Dwarnings`). |
| `make fmt deny reuse` | EXIT 0 each. deny: "advisories ok, bans ok, licenses ok, sources ok". reuse: compliant. |
| `make fuzz` | EXIT 0. Three targets (`parse-source`, `parse-document`, `load-workspace`), "Done 5000 runs" each, with `-max_len=400000 -rss_limit_mb=2048 -malloc_limit_mb=512`. The `parse-source` corpus is 7.4 MB in 56 files, the largest 390 KB, and none is truncated. The output has 0 `warning:` lines. |
| Registry harness: `cargo run --release -p raf-source --example registry_scan -- ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f` (auto jobs) | EXIT 0 in 50 s. 1233 crates, 1204 OK, **0 crates with misplaced items, 0 misplaced items**, 0 budget refusals. Failures: Syntax 2, Target 4, Tree 2, UnresolvedMod 7, UnresolvedTrait 5, UnsupportedItem 9. Largest values: item bytes 334.6 MB (`linux-raw-sys`, under 512 MiB), glob 33.2 MB (`web-sys`), tokens 1,505,567 (`chrono-tz`). Output: `/tmp/raf-exit/registry.txt`. |
| Hosted CI, run 38522139805, head `bf08e45` | success for all 12 jobs: fmt, clippy, test, test-release, doc, deny, msrv, reuse, fuzz, cross-check, test-macos and test-windows. |

## Criteria

Status key: **D** demonstrated, **F** failing, **U** unverified.

### Issue scope: raf-okf

| # | Requirement | Demonstration | Observation | Status |
|---|---|---|---|---|
| K1 | Markdown with YAML frontmatter. `type` is required. `title`, `description`, `tags` (sorted) and `timestamp` follow OKF, with `resource` refused (Stage 1 decision 1). | `probe okf <repo>` and `probe okfedge docs/capabilities/okf-profile.md`, through `Document::parse` and `Profile::validate`. | 42 tracked documents parse and validate: 21 Capability, 19 Change, 2 Architecture. Non-OKF Markdown (`docs/BRAND.md` and others) is refused at line 1. On the real capability: `tags: [zeta, alpha]` plus a timestamp validates and renders `tags: [alpha, zeta]`, which re-renders the same. `[a, a]` gives `DuplicateTag`, month 13 gives `BadTimestamp{OutOfRange month}`, `resource:` gives `ResourceKey`, removing `type` gives `MissingKey{type}`, and `state: 7` gives `BadState`. | D |
| K2 | A closed key set per type with typed values; an unknown key is a hard error. | `probe okf`: `extra: 1` injected into every tracked document. | All 42, across all three types, give exactly one `UnknownKey{extra}`. | D |
| K3 | A deterministic writer: round-trip stable, sorted where the spec says. | `probe okf`. | Every document gives `render() == text` and `parse(render()) == doc`, with 0 mismatches. | D |
| K4 | Body model: H2 sections with byte spans, links, fences, generated blocks and PROPOSED blocks. | `probe okfedge`. | The real capability has 7 sections, 3 links and 2 fences. With blocks injected, generated `catalogue` reports content span `83..84 / 6003..6015` and PROPOSED `demo-change` reports its span. `replace_spans` replaces the generated content. PROPOSED in an Architecture page gives `ProposedOutsideCapability` at line 18. A split link gives `SplitLink{Inline}` at line 81 (decision 4). | D |
| K5 | An `index.md` without frontmatter is navigation, not a document. | `probe okfedge`. | `Index::parse` accepts a generated-only index with an SPDX header. `Document::parse` refuses it at line 1. Prose outside a block is refused at line 6. `is_capability_index` is true only for `docs/capabilities/index.md`. | D |

### Issue scope: raf-source

| # | Requirement | Demonstration | Observation | Status |
|---|---|---|---|---|
| S1 | A `Tree` trait (`read`, `list`) with Disk and Memory implementations. | Every probe loads through `DiskTree` (this repository, purrdf, katamari, `/tmp/raf-exit/esc`) or `MemoryTree`. Suites `memory_tree`, `disk_tree_*` and `read_cap` pass. | Equal behaviour through both. | D |
| S2 | Package discovery under explicit roots (decisions 2 and G9), with lib, bin and build.rs roots. | `probe disc`, `probe repo`, `probe ext`. | `load(&[])` gives `NoRoots`. On Memory: roots `crates`, `other`, both, and `.` find the right packages. Lib, src/main.rs bin, src/bin bin, explicit-path bin, test, bench, example and build-script targets are found, with `harness`, `test` and `required_features` read. This repository: `crates` gives 2 packages, `.` gives 4. purrdf `crates/`: 44 packages, 83882 items, 18 s. katamari: 42 packages, 14513 items. | D |
| S3 | Mod-tree following as rustc does, including `#[path]` (CA-F2). | `probe f2`, `probe repo`, `probe filesitems`, the registry oracle. | `a/mod.rs` + `a/b.rs`: `demo::a::in_b` and `demo::a::b::in_a` are NotFound, beside `demo::a::b::in_b` Ok. `super::super::top` from `a/b.rs` loads. `#[path="gen/generated.rs"]` children `a`, `ga` and `z` sit under `generated`. `tests/common` reached from `tests/zz.rs` is placed under `test_crate::zz::common::a`. This repository: the 27 items of `body/blocks.rs` are under `raf_okf::body::blocks`, and `support/child.rs` is under `…::support::child` in every test crate. Registry: 0 misplaced; brotli, structured-zstd and wasmi load. | D |
| S4 | `#[cfg(test)]` and `#[test]` items are tagged, not dropped. | `probe disc`, `probe repo`. | `demo::tests` and `demo::only_test` (`cfg(all(test, unix))`) are `CfgTest`. `demo::tests::unit` and `demo::a::t_in_a` are `TestFn`, non-shipping. This repository: CfgTest 100, TestFn 628, and `test_fns()` 628. | D |
| S5 | Test, bench and example targets are non-shipping (`<pkg>::test_crate::<target>` and the like). | `probe disc`. | `demo_pkg::test_crate::it::case`, `demo_pkg::bench_crate::b::bench` and `demo_pkg::example_crate::e::helper` are `shipping=false`. Bins are `demo_pkg::bin_crate::<bin>` (sign-off). | D |
| S6 | Symbol index of every named item plus the `use`-alias table, so re-exported paths resolve. | `probe repo`, `f1`, `f5`, `f6`, `f7`, `misc`, `grp`, `ext`, and the registry. | Chains resolve (`raf_source::DiskTree` → `tree::DiskTree` → `tree::disk::DiskTree`; `demo::h::x` via a glob). A1 marker: `impl Made` from `gen!(Made)` and `impl dyn Tr` are listed by `unresolved_impls()`, and `demo::Made::m` gives `UnresolvedSelf`. A2: `purrdf` gives `Ambiguous(2)`, `resolve_in_package("purrdf"/"purrdf-capi", "purrdf")` gives Ok, and an unknown package gives `UnknownPackage`. Paths with `crate`, `super` or `self` in canonical position, and empty or trailing-`::` paths, give `Invalid`. A missing local glob loads, and lookups through it give `UnresolvedGlob`. **But** a rustc-valid `use` group with a leading `::` element fails the whole load (**X2**). | F (X2) |
| S7 | Port from the precedents, with no purrdf dependencies. | `crates/raf-source/Cargo.toml`. | Dependencies are `syn`, `quote`, `toml` and `proc-macro2`; `libc` on unix; `rustix` (non-Linux unix) and `windows-sys` (Windows), both per the R8 decision. There is no purrdf dependency. Provenance is replaced by decision 5. | D |
| S8 | Path-traversal safety: `#[path]` cannot escape the workspace root. | `probe disk` on real disk. The outside sentinel is mode 000, so any open would surface as `Io`. | `../../outside.rs`, an absolute path, an inline module, `cfg_attr(unix, path=..)`, a symlinked module file, `[[bin]] path="../outside.rs"`, and `vendored/../outside.rs` through a directory symlink beside an in-root decoy: each gives `Tree(OutsideRoot)` naming the module and attribute or the manifest key. The in-root redirect `#[path="other/x.rs"]` loads and resolves. | D |

### Acceptance criteria

| # | Criterion | Demonstration | Observation | Status |
|---|---|---|---|---|
| AC1 | Memory fixtures cover nested modules, `#[path]`, re-export chains, test/bench/example targets and a rejected escape. | Suites `module_tree` 39, `use_resolution` 74, `symbol_index` 26, `trait_keys` 46, `target_roots` 8, `workspace_containment` 16 and `discovery_targets` 28 all pass. My Memory fixtures reproduce each shape, including the first-before-parent layouts that the earlier fixtures missed. | Pass. | D |
| AC2 | OKF round-trip golden tests; an unknown-key fixture fails. | `render` 21, `profile` 36 (including `an_unknown_or_foreign_key_is_reported_on_every_type`), and my K2/K3 sweep. | Pass. | D |
| AC3 | Public APIs documented (`missing_docs = deny`). | Root `Cargo.toml` `missing_docs = "deny"`, every crate on `[lints] workspace = true`, and `make clippy doc` EXIT 0. | Pass. | D |
| AC4 | The `cap:okf-profile` and `cap:source-index` docs match what was built, with state `delivered`. | Both have `state: delivered`. The binding gate on doc snapshots: the real snapshot passes; `raf_okf::body::FenceBlock` fails ("names nothing"); a `tests` binding to a module fails ("not a function marked `#[test]`"); a `code` binding to a test fails ("test code or … not in a shipping target"). A Contract re-read against code and probes found **one mismatch (X2)**. | F (X2) |

### Prior audit findings (CA-F1..F7), re-demonstrated

| Finding | Re-demonstration | Observation | Status |
|---|---|---|---|
| CA-F1: `use <crate>;` resolved against itself | `probe f1`, `probe repo`, registry. | `pub use serde;` and `pub use serde::{self, Serialize};`: every path is NotFound, and `demo::Local` is Ok. `pub use std;`, `m::std` and `std::{self as s2}` are NotFound. `use std;` + `impl std::fmt::Display for &Foo` loads in editions 2015/2018/2021/2024, keyed `Canonical("std::fmt::Display")`. This repository: `raf_source::syn::File` is NotFound, not Cyclic. hmac, ripemd, sha3, p256, p384, p521, cx448 and rmp-serde all load. | Fixed |
| CA-F2: stale module index | S3 row above. The gate rejects the misplaced binding. | 0 misplaced in 1233 crates. | Fixed |
| CA-F3: listing guarantee overstated on non-Linux | `SECURITY.md` and `source-index.md`, Containment section; hosted test-macos and test-windows. | Per-platform text: Linux `O_PATH` and descriptor path; other Unix an `openat` walk (rustix); Windows `GetFinalPathNameByHandleW`, with the residual window stated (names or kinds, never contents). `on_handle` is gone. The one `unsafe` block is confined to `tree/disk/final_path.rs`, per the R8 decision. Native macOS and Windows jobs pass. | Fixed |
| CA-F4: promised fixtures missing | `profile.rs::an_unknown_or_foreign_key_is_reported_on_every_type`; `load_cap.rs` (exact cap and cap+1 for the lib root and a module file, a lowered cap, a raised cap). | Present and passing. | Fixed |
| CA-F5: a macro in a trait body aborts the load | `probe f5`, registry. | Loads. `demo::T::g` is Ok, and `demo::T::f` (macro-generated) is NotFound. rayon, rayon-core, tokio-util, time, wasmparser, crossterm, owo-colors, yansi, rmp, minijinja, bincode, matrix_indexed_db_futures and cranelift-assembler-x64 load. interprocess now fails only on the X2 `use`-group shape. | Fixed |
| CA-F6: an enum-variant glob refuses the trait-key lookup | `probe f6`, registry. | Loads, keyed `std::cmp::PartialEq`. http, itertools, ena, combine and wasm-bindgen-test load. | Fixed |
| CA-F7: trait-key recursion on the caller's stack | `probe f7 20000` on a 2 MiB thread, in debug and release. | 20000 `use std::collections::HashMap as H{i}` plus `use std;` loads (`Canonical("std::fmt::Display")`; 1.06 s debug, 0.17 s release). 20000 chained local aliases with `impl A19999 for &X` loads, keyed `Canonical("demo::m::T")`. rustls loads. | Fixed |

### Remediation items and plan or user decisions (spot checks beyond the issue)

| Item | Demonstration | Observation | Status |
|---|---|---|---|
| Token budget, user decision "(1) - default to 2M"; Contract figure (2) | Near-limit probe `probe cap` in release, with `VmHWM`. | The costliest shape (one body of N empty statements) at exactly 2,000,000 tokens loads in 0.53 s, VmHWM 1116 MB and VmPeak 1334 MB. The Contract says 0.80 s and 1188 MiB resident, with a bound of 3.1 GiB. One more token is refused typed (`Budget FileTokens 2000000`, naming the line) in 15 ms with 38 MB. `a<b;` at exactly 2M loads in 1.5 s at 815 MB, consistent with the stated ~430 B/token. | D |
| Size cap, 8 MiB | `probe cap`. | Exactly 8 MiB loads and resolves `demo::first`. 8 MiB + 1 is refused naming `src/lib.rs`. | D |
| Retained-memory budgets, figure (1) | Registry maxima. | All under their limits. I did not re-run the four-limits probe (time); it stays reviewer-recorded evidence. | D (scan) / U (time) for the combined probe |
| R6 parser recursion budget and R7 fuzz reach | `overflow_harness` 14 and `prescan_budgets` 26 pass in both profiles; `make fuzz`. | The carve-out phrases are gone. The only "no budget counts" text now concerns the settle graph (see the observations below). | D |
| R11 sealing, A15 line ranges, A16 `items_under` | Code read; the doc tests pass. | `#[non_exhaustive]` is on the six types. `SpanFact.lines: Range<usize>`. `compile_fail` doctests are present. | D |
| R13 / G12: PR body records the decisions | `gh pr view 13`. | **Stale (X1).** | F (X1) |
| Weakened tests | `git diff 6c744af HEAD` over the tests. | The 63 removed assertions belong to code that moved: fact assertions went to `facts.rs` unit tests (A5), the empty-roots test was inverted (G9), verifier tests were replaced with strategy tests (R8), and the inherent-impl refusal was replaced (A1). The `overflow_harness` judged count went from 11 to 35. No assertion was loosened to pass. | D |

## Open findings

### X1 (LOW, publication): the PR 13 body contradicts the delivered behaviour and lacks the Stage 2 decisions

- **Where:** the live PR 13 body. `headRefOid` is `bf08e45`, the PR is OPEN and MERGEABLE, and
  the body is 7879 characters, the same as the Stage 1 `pr-body.md` dated Oct 12.
- **What it says that is no longer true:**
  - "an empty root list loads an empty workspace without error". Delivered: `NoRoots`, user
    decision G9.
  - non-Linux containment paths are "never run on those platforms". Delivered: the hosted
    `test-macos` and `test-windows` jobs pass.
  - the fuzz gate runs "inputs of at most 4096 bytes". Delivered: 400000.
  - "please confirm" the Unicode-3.0 exception. It was confirmed in the sign-offs.
  - shapes "that no budget counts can still abort". R6 replaced this.
- **What it lacks:** A1, A2, G9, the four sign-offs, the token-budget decision, the R8 decision
  and the merge-scope decision.
- **Why it matters:** R13 requires the PR body to quote these decisions (G12, owner
  visibility). As it stands, the PR presents the owner with false statements about the merged
  behaviour.
- **Required fix:** replace the body before merge. The corrected text already exists in
  `$STAGE/pr-decisions.md` and the decisions section of `$STAGE/squash-notes-draft.md`. Bring
  the Validation and Residual-risk sections up to date as well.

### X2 (MEDIUM, AC4 and S6): rustc-valid `use` groups with a leading `::` element, and attribute-macro input items, fail the whole load, while the Contract says every `use` contributes

- **Where:**
  - `crates/raf-source/src/facts.rs:1068-1075`: the `other =>` arm turns syn's `Item::Verbatim`
    into `ExtractError::Unsupported`;
  - `modtree.rs:970`: that becomes `LoadErrorKind::UnsupportedItem`;
  - the Contract, `docs/capabilities/source-index.md:1032-1034`: "Every `use` declaration
    contributes to a table of the names of its module: renames, nested groups, `self` in a
    group, globs, a leading `::` …".

  Neither the Contract nor its Constraints or Verification states that any rustc-valid item
  fails the load with `UnsupportedItem`. The `Workspace::load_with` error list does not name
  that error either.
- **Evidence:**
  - `/tmp/raf-exit/rustc/grp.rs` holds `pub use { ::std::fmt::Debug as D, crate::a::f, };`.
    `rustc --edition 2021 --crate-type lib` exits 0.
  - The same file through `Workspace::load` on a `MemoryTree` (`probe grp`) gives
    "src/lib.rs:2: unsupported item: pub use { :: std :: fmt :: Debug as D , crate :: a :: f , }".
    Neighbours load and resolve: `pub use { crate::a::f };`, `pub use a::{f, g};` and
    `pub use {a::f as f2};`.
  - Registry: 9 crates of 1233 get no index at all.
    - 5 of them are this `use`-group shape: `macro_rules_attribute`,
      `macro_rules_attribute-proc_macro`, `ml-kem`, `zerocopy-derive` and `interprocess`.
    - 4 are attribute-macro input that syn returns verbatim: bodiless `#[ref_cast_custom] fn`
      in `fluent-uri`, `ref-cast` and `schemars`, and `typst-library`.
- **Context:**
  - Plan C7 says "`Item::Verbatim` is an error", and the earlier audit raised this as a scope
    point for the owner.
  - No user decision covers it.
  - The delivered Contract does not record the C7 refusal, and it states the opposite for
    `use`. A consumer reading the Contract cannot know which valid crates will be refused.
- **Required fix**, either of:
  - (a) model a verbatim `use` group: parse its tokens as a use tree whose elements may carry a
    leading `::`, with fixtures beside the neighbours above. Attribute-macro input stays as in
    (b).
  - (b) state the refusal in the Contract under Constraints, and in the `load_with` error docs:
    which rustc-valid shapes are `UnsupportedItem` (a `use` group with a leading-`::` element;
    an item syn cannot represent, such as a bodiless `fn` rewritten by an attribute macro). Add
    a Verification fixture: `pub use {::std::fmt::Debug};` is refused naming file and line,
    beside `pub use {crate::a::f};`, which loads. Correct the "Every `use` declaration
    contributes" sentence.

  Option (b) keeps plan C7. Option (a) removes 5 of the 9 registry refusals.

## Scope observations (not counted as findings)

- **The two unmodelled rustc visibility rules (R5b) were accepted by the orchestrator, not by
  the user.**
  - They are stated under Constraints (`source-index.md:1372-1379`):
    - a `pub use` reached only through a private glob is visible as the import;
    - an ambiguity through an inaccessible glob yields the other item.
  - They give extra answers only on code where rustc hides or refuses the path. The issue's
    "re-exported paths resolve" is met: I saw chains resolve on this repository, purrdf and
    Memory fixtures.
  - I did not independently re-probe the two rules (time). The R5b review's 5,522-program
    differential is the evidence.
  - The owner should confirm them explicitly, because no user decision covers them.
- **The settle work for undecided imports is unbudgeted, and its time grows superlinearly.**
  - Stated cost: 200 modules that glob the root, each with an import nothing defines, take
    613 s at 555 MiB resident (`source-index.md:683-692`, `SECURITY.md:120-125`).
  - The R5c review confirms it is not a regression. The issue sets no time bound, and
    SECURITY.md states that the parsers set none. It is disclosed, so I record it as residual
    risk, not a defect.
- **Old-edition syntax that syn 3 rejects is a typed `Syntax` refusal.**
  - `fxhash` uses a 2015 anonymous trait parameter; `signal-hook-registry` uses a bare trait
    object.
  - The Contract states that "a file that `syn` rejects is a `Syntax` error". This is
    consistent with "a `syn` workspace index".
- **Enum variants are not indexed items.** `demo::Inner::A` is NotFound. That matches
  `SymbolKind` and Rust's definition of items.
- **A lookup in all name spaces through a module with a broken glob fails.** `demo::g::y` gives
  `UnresolvedGlob` for a function defined in `g`, while the value name space resolves. The
  Contract states and tests this.
- **Remaining registry refusals.**
  - The other 20 registry refusals are correct under the Contract: test modules missing from
    published packages, target-file collisions or missing target files, `#[path]` outside the
    crate, and the five `UnresolvedTrait` traits that `include!` or `macro_rules!` generate.
  - Load-error origins for an explicit `[lib] path` now name `lib.path` correctly (G16).

## Verdict

Every issue scope bullet except S6, and AC1-AC3, has execution evidence through the real entry
points:

- `Workspace::load`/`resolve` on this repository, purrdf, katamari, Disk and Memory trees, and
  1233 registry crates;
- `Document::parse`/`Profile::validate` on all 42 tracked documents;
- `tests/docs_contract.rs` on mutated snapshots;
- the three fuzz targets.

All seven prior findings (CA-F1 to CA-F7) are re-demonstrated as fixed. Hosted CI is fully
green on HEAD `bf08e45`. There are no `.deficiencies` entries.

Two findings remain open:

- **X2 (MEDIUM).** The Contract says every `use` contributes, but rustc-valid `use` groups with
  a leading `::` element (and attribute-macro input) refuse the whole load, and the Contract
  states no such refusal. This fails AC4 and qualifies S6.
- **X1 (LOW).** The live PR 13 body is stale. It contradicts NoRoots and other delivered
  behaviour, and it lacks the Stage 2 decisions that R13 requires.

Remaining work:

- Fix X2 by modelling the group or by stating the refusal, with fixtures.
- Publish the corrected PR body.
- Re-run `cargo test --workspace --locked` and the binding gate. The registry harness is needed
  only if option (a) is chosen.

VERDICT: FINDINGS-OPEN (superseded by the Recheck below)

## Recheck (HEAD `5bcd741`, 2026-10-14 02:05-02:20)

This recheck covers X1 and X2 and the Contract text that X2 touched. Nothing else changed:
`git diff bf08e45 5bcd741` touches only `crates/raf-source/{src/facts.rs, src/verbatim.rs,
src/lib.rs, src/workspace.rs, tests/load_budgets.rs, tests/symbol_index.rs,
tests/use_resolution.rs}` and `docs/capabilities/source-index.md`. My earlier evidence for the
other criteria therefore still applies, and I re-ran the whole-suite gates anyway.

### Fresh runs on `5bcd741`

| Command | Result |
|---|---|
| `cargo test --workspace --locked` | EXIT 0. 851 passed, 0 failed, 1 ignored (13 more than on bf08e45). `docs_contract` 141 passed, including binding resolution of the six new test bindings, and `repo_gates` 57 passed. |
| `make test-release clippy doc fmt` | EXIT 0. 479 release tests passed. |
| `make fuzz` | EXIT 0. All three targets ran 5000 runs each. |
| Registry harness, release, auto jobs | EXIT 0. 1233 crates, **1213 OK**, 0 misplaced, 0 budget refusals. There are no `UnsupportedItem` failures (9 before). A line-by-line diff against my bf08e45 scan (`/tmp/raf-exit/registry.txt` against `/tmp/raf-exit/re-registry.txt`) changes exactly nine lines, the nine crates going from ERR to OK. Every other crate's outcome and item count is identical. |
| Hosted CI, run 38532696298, head `5bcd741` | `success`. All 12 jobs passed: fmt, clippy, test, test-release, doc, deny, msrv, reuse, fuzz, cross-check, test-macos and test-windows. |

### X2: re-demonstrated through `Workspace::load` on real registry crates

Command: `probe find <crate dir> <file> <names>`, which loads through `DiskTree` with root `.`
and resolves names in each module path of the file.

- **`interprocess-2.4.4`**:
  - **Load.** It loads (1606 items, 243 ms). Its
    `tests/tokio_local_socket/stream.rs:1` group mixes `crate::{…}`, `::tokio::{…}` and plain
    members.
  - **Local members resolve.** In `interprocess::tests::tokio_local_socket::stream`:
    - `ListenerOptions` resolves to `interprocess::local_socket::listener::options::ListenerOptions`;
    - `Name` resolves to `interprocess::local_socket::name::Name` (via glob);
    - `BoolExt` resolves to `interprocess::misc::BoolExt`;
    - `SubUsizeExt` resolves to `interprocess::misc::SubUsizeExt`.
  - **Dependency members are NotFound**, as they should be: `AsyncBufRead` and `task` (the
    `::tokio` member) and `Arc` (`std`).
  - **`Stream` is NotFound**, which is correct. It comes through the glob
    `stream::r#enum::*`, and `src/local_socket/tokio/stream/enum.rs:73` generates it with a
    macro, which is not expanded.
- **`ref-cast-1.0.27`:**
  - It loads (50 items).
  - `ref_cast::test_crate::test_custom::forbid_unsafe::Custom::new`, the
    `#[ref_cast_custom] pub fn new(s: &str) -> &Custom;` at line 13, is indexed as an
    `InherentMethod` with `body = None`, and it resolves Ok.
- **The other two shapes:**
  - `schemars-1.2.2` loads, with bodiless `Schema::ref_cast` and `Schema::ref_cast_mut` at
    lines 411 and 414.
  - `typst-library-0.13.1` loads, and `typst_library::foundations::bool::bool` is a
    `TypeAlias` with no target that resolves Ok.
- **`macro_rules_attribute-0.2.3`:** it loads, and its group member `ඞ::paste`
  (`::pastey::paste`) is NotFound, which is correct for a dependency.

The Memory checks of each Contract sentence X2 added (`probe x2`, `probe grp`, `probe x2cost`)
all hold:

| Contract claim (`source-index.md`) | Observation |
|---|---|
| A group with leading-`::` members binds as the separate `use` declarations do. | `pub use { ::std::fmt::Debug as D, crate::a::f, };` now loads, and `demo::f` resolves to `demo::a::f`. This file exits 0 under rustc 2021. Across two packages, with `#[cfg(test)] pub use { ::p_one::X as D, crate::a::f, ::p_one::inner::{self, go as run}, { ::std::fmt::Debug as Dbg } };`: `p_two::D` resolves to `p_one::X`, `p_two::f` to `p_two::a::f`, `p_two::inner` to `p_one::inner` and `p_two::run` to `p_one::inner::go`, while `p_two::Dbg` is NotFound. |
| In edition 2015 a `::inner::go` member reads the crate root. | `demo::m::run` resolves to `demo::inner::go`, and rustc 2015 exits 0 on the same file. |
| `a::{::b}` is a `Syntax` error naming file and line. | `Syntax` at `src/lib.rs:2`, column 13. rustc refuses the same shape (E0433, "the crate root in paths can only be used in start position"). |
| Bodiless `fn`, a method in an impl, and `type bool;` are indexed with no body. | `demo::f` is a `Fn`, `demo::S::g` an `InherentMethod` and `demo::bool` a `TypeAlias`, each with no body and the right name line. |
| `const C: u8;`, a bodiless `const` in an impl, and `type T: Bound;` are `UnsupportedItem` naming file and line. | Each gives `UnsupportedItem` at line 2. |
| A group of 6000 members under a 6000-name `cfg` loads within 500 ms in release. | 25 ms. |

The Constraints bullet ("indexed only in the three shapes the Contract names") and the
amended "Parsing is syntactic" bullet match the code (`facts.rs` and `verbatim.rs`; every other
verbatim form reaches the `UnsupportedItem` arm) and the probes above. The X2 review
(`tasks/X2-review.md`) ends in a passing verdict after its recheck.

**X2: fixed.** S6 and AC4 are now **D**.

### X1: the published PR body

`gh pr view 13 --json headRefOid,body` gives head `5bcd741` and a body of 15978 characters,
rebuilt from current content.

**Fixed:**
- It states that an empty root list is the typed `NoRoots` error.
- It quotes Stage 1 decisions 1-5 and the Stage 2 decisions A1, A2 and G9 verbatim, together
  with the four sign-offs, the token-budget decision, the R8 decision and the merge-scope
  decision. That meets R13 and G12.
- None of the stale claims remain: no "loads an empty workspace without error", no "never run"
  for macOS or Windows, no 4096-byte fuzz cap, no "please confirm" and no "no budget counts".
- The validation section reflects the current head.

**X1: fixed.**

### Advisories (non-blocking; fix in the Stage 3 publication)

- **A-1.** PR body line 82 still ends the visibility-rules bullet with "Recorded as an
  orchestrator judgment; the owner may overrule."
  - The owner has since decided, verbatim, **"make an issue for it and move on."** It is
    tracked as issue 14 (OPEN, created 2026-10-14T07:43Z) and recorded in `validation.md` and
    in `squash-notes-draft.md` line 97.
  - Because the user explicitly decided this cut, it is not a laundered follow-up. The rules
    remain stated under the Contract's Constraints.
  - Re-publish the PR body from the squash notes so that it names the owner's decision and
    issue 14.
- **A-2.** Two decision timestamps are impossible on 2026-10-14.
  - The merge-scope decision is dated "2026-10-14 22:25" in the PR body and the squash notes.
  - The visibility decision is headed "2026-10-15 01:43" in `validation.md`.
  - Correct both dates in the merge text.

### Recheck result

- X1 and X2 are fixed and re-demonstrated through the real entry points. The X2 demonstration
  covers 5 of the 9 previously refused registry crates, with group members and bodiless items
  resolved, and the 9-line registry diff covers all nine.
- Hosted CI is fully green on `5bcd741`.
- `.deficiencies` still has no entries, at the worktree root and in the main checkout.
- Every criterion in the table above is now demonstrated, apart from the items that the report
  marks as limited by platform or by time:
  - FreeBSD is compiled only, as the Contract states.
  - The combined four-limits memory probe is reviewer evidence.
  - The two visibility rules are now covered by the owner's decision.

VERDICT: PASS
