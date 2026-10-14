DRAFT NOTES. Delete this block before publishing.

Sections that still need final values (each marked `PENDING:` below):

- Validation: the final `make check`, the final hosted CI run on the merge head, and the post-glob-budget registry scan.
- Review dispositions: reviews for the glob-budget, resolve-path and API-sealing items, and the exit completion audit.
- Defect-Class: Stage 3 decides.
- Descriptions of the glob-budget, resolve-path validation and API-sealing work are written from the plan. The glob-budget work is uncommitted in the worktree at drafting time, so confirm each against the final commit list.
- The registry scan counts differ between two stage records (1,177 of 1,207 in the validation index, 1,183 of 1,213 in the fix record). Re-confirm on the final scan.
- Memory and parse-peak figures are quoted from the source-index Contract as it stands in the worktree (it includes the uncommitted glob-budget text). Re-confirm against the final text.

No `GhpRsq-` trailers or other trailer lines are written here; ghprsq computes them.

---

feat(raf-okf, raf-source): add the OKF profile reader and the workspace source index

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

- Two rustc visibility rules are not modelled. (1) A `pub use` of a binding reached only through a private glob binds the name as visible as the import, where rustc keeps it no more visible than the binding it re-exports. (2) An ambiguity through a glob the reading module cannot access yields the other item, where rustc refuses the path (E0659 or E0573). Evidence: 12 wrong queries in 10 of 5,522 rustc-accepted programs on fresh private-glob seeds, against 96 in 71 programs before; 11 of the 12 were wrong before too. By the owner's decision, "make an issue for it and move on.", both rules are tracked in a separate issue and stay stated as Contract limits for this merge.
- The import work queue and the graph of undecided imports built while scope tables settle are not budgeted. They are dropped once settled, and grow with the undecided imports times the places their paths read through globs. Time grows faster still and the crate sets no time bound: 100 modules that glob the crate root, each importing a name nothing defines, settle in 20 to 32 s at 87 MiB resident.
- Windows residual: outside names and kinds can leak through `list` and `entry`, never contents. Windows also refuses reserved names, trailing dot or space, and `:` segments.
- macOS and BSD link rule: a symlink whose target climbs back through `..` loads on Linux and Windows and is refused on macOS and the BSDs, because the `openat` walk refuses a `..` at or outside the root. FreeBSD is compiled in `make cross-check` and hosted CI but never run natively.
- First-error loads: `Workspace::load` returns the first error in the order discovery, targets, module walk, index, and does not collect errors. `Profile::validate` still reports every violation.
- Unparsed items: a `use` group whose members start with `::` and a bodiless `fn` or `type` (attribute-macro input) are indexed as written; any other item that syn leaves unparsed fails the load with a typed error.
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

## Commits on the branch (oldest first)

1. `3f8697b` feat(raf-okf): bounded frontmatter reader and deterministic writer
2. `e94cd8d` feat(raf-okf): body model, transactional splicing and capability index
3. `1a67fa3` feat(raf-okf): profile validation with typed documents
4. `57d9a06` feat(raf-source): file tree seam with canonical containment
5. `0a9d243` feat(raf-source): bounded parsing on a dedicated stack with a fuzz target
6. `1e965a1` build: release-mode, fuzz and cross-target gates
7. `e15dd4b` feat(raf-source): package discovery and rustc module-tree walk
8. `e4c6a66` feat(raf-source): symbol index and use-alias resolution
9. `52155fa` fix(raf-source): inherit workspace keys from the member's workspace
10. `4e9c2f3` test(docs-contract): read documents through raf-okf and resolve bindings
11. `376baf9` docs(changes): record the outcomes of the raf-okf and raf-source items
12. `e6f3e9b` fix(raf-source): resolve a target's crate root as a module item
13. `6c744af` docs(capabilities): deliver the OKF profile and source index
14. `c10abd9` fix(raf-okf): bound body memory by sharing section slugs
15. `0617985` fix(raf-source): place a module under its declared path
16. `100bf2f`, `d89b78f`, `671850b`, `f5ede44` raf-okf tests, fenced-block accessors, validation cost, profile-doc bindings
17. `cd7d4fb` fix(raf-source): key files by identity and bound total work
18. `51d84e5` fix(raf-source): verify on the handle that is read, on every platform
19. `d1c4a7f` fix(raf-source): index macro-generated impls, enum globs and shared crate names
20. `e472918` fix(raf-source): resolve extern-prelude aliases and key traits without recursion
21. `5a5e499` fix(raf-source): settle import cycles by rustc's rules, independent of order
22. `4d56b6f` fix(raf-source): pass the current nightly lints and the macOS test job
23. `b187300` fix(raf-source): bound parser recursion with one additive budget
24. `a26e75d` build(fuzz): reach overflow-sized inputs and fuzz the OKF reader and workspace load
25. `bd6d3aa` fix(raf-source): refuse an empty root list
26. `655834b` fix(raf-source): harden manifest matching and error text
27. `20b59c5` test(raf-source): exercise caps, spawn failure and source text through a load
28. `71bd2c7` fix(raf-source): start first-line spans after a byte-order mark
29. `44e18b7` test(raf-source): build the deep directory chain within macOS path limits
30. `d145545` build(deny): pin the crate-scoped licence exceptions to audited versions

31. `be71d40` fix(raf-source): bound glob scope tables with a byte budget
32. `df71a42` fix(raf-source): match items_under by whole segment and check binding roles
33. `d8aba1e` fix(raf-source): validate resolve paths and refuse globs that name nothing
34. `bf08e45` feat(raf-source)!: seal the public types consumers bind to
35. `5bcd741` fix(raf-source): index verbatim use groups and bodiless items

## Validation

Local gates

- `make check` on 6c744af: exit 0 in 1m27s, 963 tests passed, 0 failed; gates fmt (root and fuzz workspaces), clippy `-D warnings`, test, test-release, doc `-Dwarnings`, deny (whole workspace), reuse, msrv 1.98.1 (root and fuzz), fuzz (5000 seeded runs), cross-check, licenses, issue-refs. Later full `make check` on 51d84e5 passed with 1,112 tests.
- The pre-commit hook (fmt, REUSE, `repo_gates` and `docs_contract` on the staged snapshot) ran in full on every commit.
- The local toolchain is the pinned dated nightly, which has no Apple, FreeBSD or Windows targets, so the local `make cross-check` cannot run here. Hosted `cross-check` is its evidence.
- `make -k check` on bf08e45 (log `raw/make-check-final.log`, 3m34s): every gate passes except `cross-check`, which cannot run on the pinned toolchain. 86 suites, 1,304 tests, 0 failed. `make licenses` and `make issue-refs` were run directly and pass. On the merge head 5bcd741, which adds only the verbatim-item fix, the per-item gates pass: raf-source 477/477 in debug and release, clippy, rustdoc, docs_contract 141, repo_gates 57, fmt.

Hosted CI

- Run 38280511329: no job started (account billing block, not a code failure).
- Run 38308785911: success.
- Run 38324353179 on cd7d4fb: success (fmt, clippy, test, test-release, fuzz, cross-check, doc, deny, msrv, reuse).
- Run 38361953775 on e472918: clippy failed on the floating nightly's `map_or_default` lint and `test-macos` failed because APFS refuses a non-UTF-8 file name. `test-windows` passed, the first native evidence for the Windows containment strategy. Both fixed in 4d56b6f.
- Runs 38499493700 (4d56b6f) and 38500276876 (a26e75d): every job passed except `test-macos`, whose deep-chain fixture exceeded the 1,024-byte macOS path limit. Fixed in 44e18b7.
- Run 38501515159 on 44e18b7: every job succeeded, including native `test-macos` and `test-windows`.
- Run 38532696298 on the merge head 5bcd741: every job succeeded (clippy success, msrv success, doc success, test-macos success, cross-check success, fmt success, test success, reuse success, test-windows success, fuzz success, test-release success, deny success).

Registry scan (the committed `registry_scan` example over the newest version of each crate in a local cargo registry, one child process per crate)

- Baseline on 6c744af: 1,014 of 1,177 load, 164 with misplaced items.
- After module placement: 0 misplaced items (was 165 crates and 2,757 items); `SuperAboveRoot` 3 to 0.
- After budgets (defaults): 1,021 of 1,177 load with 0 budget refusals; the largest item charge is 64% of its limit (`linux-raw-sys`); chrono-tz (1.5M tokens) loads.
- After the macro-impl, enum-glob and shared-name work: 1,163 of 1,207 load, 0 misplaced, 0 budget refusals. Real workspaces load: katamari `crates/` (2 unresolved impls), purrdf `crates/` (83k items, 6 unresolved impls; the two `purrdf` libraries resolve `Ambiguous` and `resolve_in_package` disambiguates), and this repository.
- After the alias fix: 1,177 of 1,207 load (the fix record states 1,183 of 1,213); `UnresolvedTrait` 19 to 5, all five correct refusals (traits from `include!` or macros, one invalid cfg'd-off module). After the parser-recursion budget the highest recursion score on the registry is 987 of 1927, with 0 refusals.
- After the glob budget: 1,204 of 1,233 load, with 0 budget refusals. web-sys loads in about 5 s at 144 MiB.
- Final, with the verbatim-item fix (5bcd741): 1,213 of 1,233 load, with 0 misplaced items and 0 budget refusals. All nine crates that failed on unparsed items now load, and no other crate's outcome or item count changed.

Fuzz

- Three cargo-fuzz targets (`parse-source`, `parse-document`, `load-workspace`) with committed corpora, replayed under `cargo test` in debug and release and run by `make fuzz` (5000 seeded runs, inputs up to 400 KB with resident-memory and allocation limits). Coverage of the workspace-load target rose from 4,501 to 11,772 after fixing `--cfg fuzzing` stripping span locations.
- Overflow shapes: all 35 committed shapes are refused in debug and release (0 aborts, 39 before); 39 shape families each load at their largest admitted size and are refused one size more, with at least 2.2 times debug stack headroom.

rustc-oracle fuzz (generated programs compared against the compiler for import resolution)

- 24,000+ generated programs, and fresh-seed private-glob runs: no order dependence and no panics; wrong `Cyclic` refusals 0 (38 before); wrong `Ok` results on rustc-accepted code 12 queries in 10 of 5,522 programs (96 in 71 before).
- On real code, 117 imports in katamari and purrdf now bind the public item rustc picks, and resolve dumps of this repository, katamari and purrdf are unchanged except for alias fixes and the two now-correct `NotFound` cases.

Containment race evidence

- Over 65,536 exhaustive swap schedules and 200k races per strategy: 0 outside contents (about 3k before). The FIFO race fails on the old code 10 of 10. The Windows model keeps its documented residual of outside names and kinds.

Integration

- `origin/main` is still at 2c41082, the branch base, so `git merge-tree --write-tree origin/main HEAD` exits 0 (no conflicts) at drafting. PENDING: recheck at merge time.

PENDING: final completion audit (verdict, report path, and the exit demonstrations).

## Review dispositions

Every task review under `tasks/` ends in `VERDICT: PASS`; where a review first blocked, the finding was fixed and a recheck passed.

- T1 to T9 (reader and writer, body model, profile, tree seam, parsing and fuzz, discovery and module walk, symbol index, docs_contract, delivery): PASS after the rechecks noted in each file (T1, T2, T4, T5, T6, T7 blocked once or more before PASS).
- R1 (raf-okf body memory): PASS.
- R2 (module placement): PASS after recheck.
- R3 (file identity, iterative traversal, work budgets): PASS after three rechecks.
- R4 (macro impls, enum globs, shared crate names): PASS after three rechecks.
- R5a (extern-prelude aliases, stack-safe trait keys): PASS after two rechecks.
- R5b (import settling by rustc's rules): PASS after four rechecks; its first-round position (a namespace-blind block) was withdrawn after review and replaced by rustc's per-namespace rule; the two unmodelled visibility rules are accepted limits.
- R6 (additive parser recursion budget): PASS.
- R7 (fuzz reach): PASS.
- R8 (containment on every platform): PASS after three rechecks.
- R9 (typed `NoRoots` error): PASS.
- R10 (manifest and error-text hardening): PASS.
- R11a (whole-segment `items_under`, package rule, role checks in docs_contract): PASS.
- R12o (raf-okf tests, accessors, validation cost): PASS after recheck.
- R12s (Verification tests on the real load path): PASS.
- R13 (licence pins, contract text): PASS (orchestrator review of a small configuration change).
- Stage 2 specialist reports: security (NEEDS-FIX, 2 High, 3 Medium), architecture (4 High and 2 Medium required), performance (2 required) and quality (3 Medium) were turned into the remediation items above; the completion audit's open findings F1 to F7 likewise.
- R5c (glob budget): PASS after recheck (allocate-before-charge fixed).
- R5d (resolve paths): PASS after recheck (two-dependency-glob `Cyclic` misreport and the error order fixed).
- R11a and R11b: PASS.
- R13: PASS.
- X2 (verbatim items): PASS after recheck (quadratic attribute reading fixed).

## Defect-Class

Defect-Class: none

## Plan and stage directory

The authoritative plan is `.stage/shared-crates-raf-okf-raf-source/plan.md` in the issue worktree, with the Stage 2 remediation plan beside it as `remediation-plan.md`. The selected stage directory is `.stage/shared-crates-raf-okf-raf-source`; validation evidence is in `validation.md` and the per-item reviews are under `tasks/`.
