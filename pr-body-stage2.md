Closes #2

## Resulting behaviour

- `raf-okf` reads, validates, models and deterministically re-renders the OKF Markdown documents that RAF tracks. It validates against the closed profile, returns a typed document or every violation with file, location and rule, and edits a body transactionally by spans.
- `raf-source` loads a Cargo workspace through a file-tree seam, discovers packages and targets, follows the module tree as rustc does (including `#[path]`), builds a symbol index with a `use`-alias table, and resolves paths with rustc's shadowing rules. Every read stays inside the canonical root, and every load is bounded by typed budgets.
- `cap:okf-profile` and `cap:source-index` are `state: delivered`. Their bindings resolve through `raf-source`, and `tests/docs_contract.rs` now reads every document through `raf-okf` and resolves every binding through `raf-source`.

## Issue requirements satisfied

`raf-okf`

- Parse Markdown with YAML frontmatter, `type` required. Built: a bounded hand-written reader for the profile's YAML subset. `title` and `description` are typed non-empty strings; `tags` (re-emitted sorted) and `timestamp` are optional keys on every type; `resource` stays rejected (see Decisions that differ).
- Profiles, closed key set per `type`, unknown keys a hard error. Built: one schema table drives both validator and writer. An unknown key, a key belonging to another type, and unknown keys nested under `bindings` and change items are each reported on every type.
- Deterministic writer. Built: keys in schema order, `tags` sorted, comments kept on their keys. Every tracked document round-trips byte for byte, and three committed fixtures round-trip independently of the repository.
- Body model. Built: H2 sections with line and byte spans, links of every form outside fences and code spans, fenced blocks (with `content()` and `is_closed()`), `<!-- raf:generated:<kind> -->` blocks and `> [!PROPOSED change:<slug>]` blocks with content spans, and `replace_spans`.
- `index.md` without frontmatter is navigation. Built: `Index::parse` applies the navigation rule and returns the generated blocks.

`raf-source`

- `Tree` with `read`, `list`, `Disk` and in-memory implementations. Built: `Tree` (`read`, `list`, `exists`) with `DiskTree` and `MemoryTree`, not-found distinct from refused and I/O failure.
- Package discovery under configurable roots. Built: explicit roots, no default (see Decisions that differ). Lib, bin and build.rs roots, and rustc's module rules including `#[path]`. `cfg(test)` and `#[test]` items are tagged, never dropped.
- Test, bench and example targets indexed as non-shipping. Built: `<pkg>::test_crate::<target>`, `bench_crate`, `example_crate`, `build_script`, and `<pkg>::bin_crate::<bin>`.
- Symbol index of every named item plus the `use`-alias table so re-exports resolve. Built: spans, namespaces, doc flags, targets, canonical trait keys, and a resolver returning `Resolution` or a typed `NotFound`, `Ambiguous` or `Cyclic`.
- Port from the precedent implementations. Built: ideas and behaviour taken from the precedent sources; the AGPL copies were used for ideas only, never text. Known defects of the precedents each have a named negative fixture.
- Path-traversal safety. Built: canonical containment checked again on the opened handle, per platform, and `#[path]` escapes refused (never clamped).

Acceptance criteria

- Fixture tests on `Memory` trees cover nested modules, `#[path]`, re-export chains, test/bench/example targets and a rejected `#[path]` escape. Done in `crates/raf-source/tests`.
- OKF round-trip golden tests and an unknown-key fixture that fails. Done in `crates/raf-okf/tests`.
- Public APIs documented with `missing_docs` denied. Done; the workspace lint is unchanged and `make doc` runs with `-Dwarnings`.
- Capability documents updated to match what was built. Both are `delivered`, and both change records are deleted after every item reached an `implemented` outcome.

## Design in brief

### raf-okf

- Frontmatter: a bounded reader (1 MiB input, YAML nesting 32, Markdown nesting 64) refuses flow mappings, duplicate keys, anchors, aliases, tags, merge keys, block scalars and unknown escapes with positioned errors. No `unsafe`. The billion-laughs input stays under 8 times its size in allocation.
- Body: `Body::parse` builds sections, anchors, links, fences, generated and PROPOSED blocks. Each section shares one slug allocation, so body memory is linear in input. Split links are the `SplitLink` violation.
- Profile: `Profile::validate` returns a typed `RafDocument` or every violation. Validation is linear (the change set is indexed, generated blocks found by binary search), with a 100 ms cost bound asserted in release.

### raf-source

- Tree seam and containment: every read goes through `Tree`. Files are keyed by identity (device and inode on Unix), so a file reached under several spellings is parsed once, from one verified open.
  - Linux: `O_PATH | O_NOFOLLOW`, a `/proc/self/fd` link and type check, then a reopen to read or list. No outside FIFO or device is opened for I/O.
  - macOS and the BSDs: an `openat` walk from a root descriptor with no symlink following (`rustix`); a `..` at or outside the root is refused.
  - Windows: a handle held without delete sharing, with a `GetFinalPathNameByHandleW` check before and after use (`windows-sys`). That call is the only library `unsafe`, in one module with a scoped allowance and a SAFETY argument; the workspace lint is unchanged.
  - Files are capped at 8 MiB and a FIFO is refused without blocking.
- Parsing: a token pre-scan enforces delimiter, generic, prefix-operator, closure-bar and `else if` budgets, one additive parser-recursion budget (limit 1927), and a per-file token budget. `syn` and the caller's visitor run on a named thread with a 64 MiB stack. A child-process harness classifies each hostile input as a typed refusal, a complete index or a genuine abort, and fails on anything else. All 35 committed overflow shapes are refused in debug and release; the earlier code aborted 39 times across them.
- Discovery and module walk: manifests are read with `toml`, with workspace inheritance and Cargo-accurate membership (root package, `members` minus `exclude`, path dependencies); `members` and `exclude` are capped at 4096 entries. The walk is iterative: 20,000 directory levels run on a 2 MiB thread, and nesting is bounded at 128 modules and 262144 walks per load.
- Symbol index and resolver: crates resolve through each target's extern prelude, keyed by package, so a shared crate name is `Ambiguous` and `resolve_in_package` disambiguates. Imports settle per namespace and independent of order: an explicit import shadows globs only in the namespaces it binds; a cycle that exactly one glob answers resolves through it. Trait-impl members are keyed by canonical trait path with an explicit worklist (20,000 chained aliases run on a 2 MiB thread). An inherent impl of a self type the index cannot see is indexed with an `Unresolved` marker, and resolution through it is a typed error.
- Budgets (each typed, naming the budget, limit and file): total bytes 256 MiB, item bytes 512 MiB, fact bytes 256 MiB, glob bytes 512 MiB, tokens per file 2,000,000, directory depth 128. Every charge is an upper bound derived from container layout, checked with a counting allocator.
  - Retained memory is at most 1.5 GiB at the defaults. A load near all limits at once keeps 1054 MiB and peaks at 1059 MiB.
  - One-file parse peak is at most `max_file_tokens` times 1664 bytes, about 3.1 GiB at the default. The costliest arrangement at exactly 2,000,000 tokens measured 2623 MiB allocated at peak, 1188 MiB resident. With `max_file_tokens = 320000` the peak stays under 512 MiB.
  - A load holds figure one plus one file's figure two at once, about 4.6 GiB at the defaults; every term is a `LoadOptions` limit.

## Decisions that differ from the issue text

Stage 1 decisions (recorded 2026-10-12):

1. "Extend the profile with `tags` and `timestamp`." Both are optional keys on all three document types; `resource` stays rejected. `tags` is a list of non-empty strings, a duplicate tag is a violation, and `render` emits `tags` sorted. `timestamp` is an RFC 3339 / `xsd:dateTime` string, and a malformed one is a violation.
2. "No default roots in `raf-source`." `Workspace::load` takes explicit roots, and each consumer owns its own default (census-ledger keeps the workspace root). The issue says "(default `crates/`)". An empty root list is the typed `NoRoots` error.
3. "Containment race: closed on Linux, with a portable fallback elsewhere." Linux verifies the handle's real path under the canonical root after opening with `O_NONBLOCK|O_NOFOLLOW` and `fstat`. The non-Linux part was later superseded by the owner's rustix and windows-sys decision below.
4. "Links split across lines are refused." `Rule::SplitLink` reports it at its line, and `docs_contract` applies it to every tracked Markdown file through `Body::parse`.
5. "Completion adversary." It ran once, on its own, on the merged plan before implementation (findings F-1 to F-8, all integrated).

Owner decisions (2026-10-12, verbatim answers):

- A1, an inherent impl whose self type the index cannot see: **"Per-item marker (Recommended)"**.
- A2, two packages with the same `[lib]` name: **"Key crates by package (Recommended)"**.
- G9, empty root list: **"Typed NoRoots error (Recommended)"**.
- Sign-offs: **"Unicode-3.0 for unicode-ident, NCSA for libfuzzer-sys, `<pkg>::bin_crate::<bin>` naming, `Rule: Sync` in census-rule-api"**. Built as crate-scoped licence exceptions in `deny.toml` and `fuzz/deny.toml`, pinned to the audited versions (`unicode-ident@=1.0.27`, `libfuzzer-sys@=0.4.13`).
- Per-file token budget (question: "R3's per-file token budget: syn's AST for the costliest admitted shape costs ~1,122 bytes/token, so a 512 MiB one-file parse peak allows ~475k tokens. Five real registry crates exceed it ... How should the bound and these crates be handled?"): **"(1) - default to 2M"**. `max_file_tokens` is configurable and defaults to 2,000,000.
- R8, non-Linux containment (question: "on macOS/BSD/Windows a swap made before the open and undone before the check can make a read return an outside file's contents ... How should non-Linux containment be handled?"): **"Add rustix + windows-sys"**.
- Merge scope and time cap (2026-10-14 22:25; question: "Which scope do you want for merging issue 2?"): **"Everythign - you have 6 hours - capped"**. The glob budget, resolve-path validation, API sealing and the licence-pin and contract-text work all stay in this PR, with the hard deadline 04:25 on 2026-10-15.

## Accepted known limits

All are stated in the `source-index.md` Contract and Constraints and in SECURITY.md.

- Two rustc visibility rules are not modelled. (1) A `pub use` of a binding reached only through a private glob binds the name as visible as the import, where rustc keeps it no more visible than the binding it re-exports. (2) An ambiguity through a glob the reading module cannot access yields the other item, where rustc refuses the path (E0659 or E0573). Evidence: 12 wrong queries in 10 of 5,522 rustc-accepted programs on fresh private-glob seeds, against 96 in 71 programs before; 11 of the 12 were wrong before too. Recorded as an orchestrator judgment; the owner may overrule.
- The import work queue and the graph of undecided imports built while scope tables settle are not budgeted. They are dropped once settled, and grow with the undecided imports times the places their paths read through globs. Time grows faster still and the crate sets no time bound: 100 modules that glob the crate root, each importing a name nothing defines, settle in 20 to 32 s at 87 MiB resident.
- Windows residual: outside names and kinds can leak through `list` and `entry`, never contents. Windows also refuses reserved names, trailing dot or space, and `:` segments.
- macOS and BSD link rule: a symlink whose target climbs back through `..` loads on Linux and Windows and is refused on macOS and the BSDs, because the `openat` walk refuses a `..` at or outside the root. FreeBSD is compiled in `make cross-check` and hosted CI but never run natively.
- First-error loads: `Workspace::load` returns the first error in the order discovery, targets, module walk, index, and does not collect errors. `Profile::validate` still reports every violation.
- Other stated limits: a residual parser stack overflow aborts the process (the run fails and is never a pass); macros are not expanded and `cfg` other than `test` is not evaluated; a glob that carries an alias of an outside crate's type item under a crate's name cannot be classed by namespace and is not refused; an unresolvable `use` is not a load error and shows as `NotFound` only when something resolves through it; loading has no time bound.

## Files and purpose

- `crates/raf-okf/src`: `frontmatter` (reader, writer, comment handling), `body` (sections, links, fences, blocks, splicing), `profile` (schema table, validation, typed documents), `index.rs` (navigation index), `document.rs`, `span.rs`, `limits.rs`, `error.rs`.
- `crates/raf-okf/tests`: refusal, render, schema, golden, bounds, cost and allocation suites, with fixtures.
- `crates/raf-source/src`: `tree` (seam, memory tree, per-platform disk strategies), `prescan` and `parse.rs` (bounded parsing), `discovery.rs`, `manifest.rs`, `members.rs`, `target.rs` (packages and targets), `modtree.rs` (module walk), `index.rs`, `item.rs`, `impls.rs`, `facts.rs` (symbol index), `glob.rs`, `glob_scope.rs`, `settle.rs`, `resolve.rs` (aliases and resolution), `budget.rs`, `meter.rs`, `cost.rs`, `options.rs` (budgets), `workspace.rs`, `error.rs`.
- `crates/raf-source/tests`, `examples/registry_scan.rs`: fixture suites on memory trees, containment and race suites per platform strategy, overflow-shape harness, load-budget and error-text suites; the registry scanner loads each crate in its own child process.
- `fuzz/`: targets `parse-source`, `parse-document` and `load-workspace`, committed corpora, and its own `deny.toml`.
- `tests/docs_contract.rs`, `tests/repo_gates.rs`, `tests/support`: documents read through `raf-okf`, bindings resolved through `raf-source`, doctests tied to the capability Usage and Examples blocks. The private scanners are gone.
- `Makefile`, `.github/workflows/ci.yaml`: new gates `test-release`, `fuzz`, `cross-check` (Apple, FreeBSD, Windows), and `deny` over the whole workspace (it used to check only the root package); CI jobs for each, plus `test-macos` and `test-windows`.
- `deny.toml`, `fuzz/deny.toml`, `Cargo.toml`, `Cargo.lock`: crate-scoped licence exceptions pinned to audited versions; new dependencies `syn`, `toml`, `rustix`, `windows-sys`.
- `docs/capabilities/okf-profile.md`, `source-index.md`, `docs/architecture/okf-profile.md`, the census-rule-api, workplan-document-model and census-reference-scan capabilities: delivered state and Contract text matching the build; `work/changes/*` deleted when complete.
- `AGENTS.md`, `README.md`, `SECURITY.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `REUSE.toml`, `.github/PULL_REQUEST_TEMPLATE.md`: describe what is built, each platform's containment guarantee and the hostile-input stance.

Branch size at drafting: 221 files, about 52.7k insertions, 1.9k deletions against `origin/main`.


## Validation (current head)

- **Local:** `make check` passes every gate except `cross-check`, which needs targets the Stage-pinned toolchain lacks: 1,304 tests, 0 failed. `licenses` and `issue-refs` were run directly and pass.
- **Hosted CI on the latest head:** all 12 jobs green, including cross-check and native macOS and Windows.
- **Registry scan:** 1,204 of 1,233 crates load, with 0 misplaced items and 0 budget refusals. web-sys loads.
- **Real workspaces:** katamari and purrdf `crates/` load.
- **Completion audit and final notes:** follow in the Stage 3 squash notes.
