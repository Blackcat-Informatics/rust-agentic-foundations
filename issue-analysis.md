## Issue Analysis: #2

**Title:** Shared crates: raf-okf + raf-source
**State:** OPEN
**Author:** (not recorded in staged issue.md)
**Created:** (not recorded in staged issue.md)
**Labels:** enhancement, crate:raf-okf, crate:raf-source
**Comments:** 0
**Depends on:** #1, "Scaffold workspace, project docs and branding, plus spec-first capability docs" (CLOSED; landed as commit 2c41082)
**Linked PRs:** none found in the staged evidence (brief.json / prior-art.md)

Evidence read: `.stage/.../issue.md`, `brief.json`, `prior-art.md`, `open.json`; `AGENTS.md`;
`docs/capabilities/okf-profile.md` (OKF), `docs/capabilities/source-index.md` (SRC);
`docs/architecture/okf-profile.md` (ARCH); `work/changes/okf-profile.md`, `work/changes/source-index.md`;
`tests/docs_contract.rs` (DC), `tests/repo_gates.rs`, `tests/support/mod.rs`; `Cargo.toml`, `deny.toml`,
`Makefile`, `.github/workflows/ci.yaml`, `.githooks/pre-commit`, `REUSE.toml`, `README.md`, `SECURITY.md`;
port sources `katamari/.../source.rs` (KAT) and `purrdf/.../source.rs` (PUR); `gmeow-ontology/docs/okf.md`,
`gmeow-gts/docs/GTS-OKF.md`; the local cargo registry for dependency licences.
Line numbers below are 1-based against those files at base 2c41082.

### Summary

Create the first two workspace member crates. `raf-okf` parses, validates (against the closed
RAF profile), models the body of, and deterministically re-renders OKF Markdown+YAML documents.
`raf-source` is a `syn`-based workspace index: a `Tree` file abstraction (disk and memory),
Cargo package/target discovery, rustc-faithful module following (including `#[path]`), a
symbol index and a `use`-alias resolver, under strict path containment and resource bounds.
The issue is a short scope sketch; the two capability documents are far more detailed and are
the binding specification (AGENTS.md:49-51 "its capability documents ... are the
specification"). Several issue bullets contradict the capability docs and must be resolved in
the docs' favour, with doc edits wherever the built behaviour adds to them. Completion moves
`cap:okf-profile` and `cap:source-index` to `implementing` or `delivered`, which has concrete
consequences in `tests/docs_contract.rs`, the change records, AGENTS.md, README.md and ARCH.

---

## 1. Traceability table

Legend: entry points are the production names the capability docs commit to
(`raf_okf::Document::parse`, `Document::render`, `Document::replace_span`, `Document::sections`,
`Profile::default().validate(&doc, path)`; `raf_source::{DiskTree, MemoryTree, Tree, Workspace,
LoadOptions}`, `Workspace::load`, `Workspace::resolve`, `Workspace::targets`). Where a doc does
not name an entry point, a proposed name is marked (proposed) and must be written into the doc.

### 1a. Issue requirements (title, scope checkboxes, acceptance criteria)

| ID | Source | Requirement | Executable acceptance check (input -> entry point -> observed) |
|---|---|---|---|
| I-1 | issue:15 | Delivers `cap:okf-profile` and `cap:source-index` | `cargo test --test docs_contract` with INVENTORY rows `okf-profile`/`source-index` set to the new state passes; `repository_documents_satisfy_the_profile` green (DC:1853) |
| I-2 | issue:19 | Parse Markdown with YAML frontmatter; `type` required | (a) `"---\ntype: Capability\n...---\n# x\n"` -> `Document::parse` -> `Ok`, frontmatter map has `type`; (b) same without `type` -> `Profile::validate` -> one violation naming key `type`, rule "missing key" (parse itself succeeds: OKF:33-35 limits parse failures to fence/YAML/non-mapping) |
| I-3 | issue:19 | `title`, `description`, `resource`, `tags` (sorted), `timestamp` follow base OKF v0.1 | Resolved by conflict C-1: `title`/`description` checked as non-empty strings (empty `title: ""` -> violation naming `title`); `resource: https://example.org/x` -> violation naming `resource` and the Capability key set (OKF:114-115); `tags: [b, a]` and `timestamp: 2026-01-01T00:00:00Z` -> each an extra-key violation |
| I-4 | issue:20 | Closed key set per `type`, typed values; unknown key is a hard error | Capability fixture + `extra: 1` -> `Profile::validate` -> exactly one violation `{file, key: "extra", rule: closed key set}`; no warning level exists (OKF:124) |
| I-5 | issue:21 | Deterministic writer, round-trip stable, sorted where the spec says | For every fixture `x` in the golden directory: `Document::parse(x)?.render() == x` byte-for-byte; for a non-canonical input `y` (keys out of profile order): `r = parse(y).render()`, then `parse(r).render() == r` and `parse(r) == parse(y)`; key order equals the ARCH:42-44 table order (C-3) |
| I-6 | issue:22 | Body: H2 sections with byte spans | Doc with `## Purpose\n\ntext\n## Contract\n...` -> `doc.sections()` -> `[Section{heading:"Purpose", slug:"purpose", lines: 2..=3, bytes: a..b}, ...]`; `&text[bytes]` equals the section content (C-2) |
| I-7 | issue:22 | Markdown links `[text](path)` | `see [x](./a.md#anchor-b)` on line 7 -> `doc.links()` (proposed accessor name) -> `Link{target:"./a.md", anchor:Some("anchor-b"), line:7}`; same text inside a fence -> no link |
| I-8 | issue:22 | Fenced code blocks | A ```` ``` ```` fence holding `> [!PROPOSED change:x]` and `<!-- raf:generated:k -->` -> `doc.proposed_blocks()`/`doc.generated_blocks()` empty (OKF:161-162) |
| I-9 | issue:22 | Generated blocks `<!-- raf:generated:<kind> -->` | Open+`<!-- raf:generated:end -->` -> one block `{kind, span}`; unclosed, nested, and orphan-end each -> `Document::parse` Err with line number (OKF:71-72) |
| I-10 | issue:22 | PROPOSED blocks `> [!PROPOSED change:<slug>]` | Well-formed marker -> block `{slug, span}` spanning all following `>` lines; `> [!proposed change:x]`, `>[!PROPOSED change:x]`, `> > [!PROPOSED change:x]`, `>  [!PROPOSED  change:x]` -> each Err "malformed PROPOSED marker" with line (ARCH:117-121) |
| I-11 | issue:23 | `index.md` without frontmatter is navigation | Resolved by C-6: generated-only index text -> index entry point Ok; index with prose outside a block -> violation; index text passed to `Document::parse` -> Err (no frontmatter) |
| I-12 | issue:26 | `Tree` trait with disk and memory impls | Same `MemoryTree` content written to a temp dir -> `Workspace::load(&DiskTree::new(tmp)?, &["."])` and `Workspace::load(&MemoryTree::from(..), &["."])` -> equal indexes (SRC:216-219) |
| I-13 | issue:27 | Package discovery under configurable roots | Tree with `a/Cargo.toml` ([package]) and `b/Cargo.toml` (virtual, [workspace] only) -> `Workspace::load(&t, &["a","b"])` -> targets only for `a`; roots `["a"]` -> `b` absent (C-9 on defaults) |
| I-14 | issue:27 | lib/bin/build.rs roots | `src/lib.rs`, `src/main.rs`, `src/bin/x.rs`, `build.rs` -> `ws.targets()` holds lib, two bins, build script with their shipping flags (C-8: build.rs is absent from SRC) |
| I-15 | issue:27 | Mod-tree following as rustc does, incl. `#[path]` | Fixtures in S-15..S-18 plus the rustc base-directory cases in C-10 |
| I-16 | issue:27 | `#[cfg(test)]`/`#[test]` items tagged, not dropped | `#[cfg(test)] mod tests { fn helper() {} }` and `#[test] fn t() {}` -> `ws.resolve("demo::tests::helper")?.test == true`, `ws.resolve("demo::t")?.test == true`; both present in the index (PUR:371 drops them; must not) |
| I-17 | issue:28 | Test/bench/example targets non-shipping, `<pkg>::test_crate::<target>` | `tests/it.rs` with `fn case() {}` -> `ws.resolve("demo::test_crate::it::case")` Ok and its target `shipping == false`; same for `benches/b.rs` -> `bench_crate::b`, `examples/e.rs` -> `example_crate::e` (C-7) |
| I-18 | issue:29 | Symbol index of every named item + use-alias table | One fixture per kind (fn, struct, enum, union, trait, type alias, const, static, mod, `macro_rules!`) -> each resolves with the right `kind`; `pub use inner::go as run;` -> `resolve("demo::run")?.path == "demo::inner::go"` (SRC:126-132) |
| I-19 | issue:30 | Port from katamari/purrdf | Not directly testable; conflict C-11 (licence) and C-12 (fail-hard deltas) govern what may be ported |
| I-20 | issue:31 | `#[path]` cannot escape workspace root | `#[path = "../../outside.rs"] mod x;` in a `MemoryTree` -> `Workspace::load` Err naming module `x` and the `path` attribute; the outside file's content is never requested (instrumented `Tree` counting `read` calls on that path == 0) |
| I-21 | issue:34 | Memory fixtures: nested mods, `#[path]`, re-export chains, targets, rejected escape | Covered by S-15..S-18, S-24, S-25, I-17, I-20 |
| I-22 | issue:35 | OKF round-trip golden tests; unknown-key fixture fails | I-5 over committed goldens (include every current repo document under docs/ and work/ as goldens: they are canonical by construction) + I-4 |
| I-23 | issue:36 | Public APIs documented (`missing_docs = deny`) | `cargo clippy --workspace --all-targets --locked -- -D warnings` and `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked` pass with both crates as members inheriting `[lints] workspace = true` (Cargo.toml:30-50) |
| I-24 | issue:37 | Capability docs updated to what was built; state -> implementing or delivered | Section 3 below; DC green; every doc API name exists (spot-check by a doctest that compiles the Usage snippet) |

### 1b. `docs/capabilities/okf-profile.md` (Contract, Constraints, Verification)

| ID | Source | Statement | Executable acceptance check |
|---|---|---|---|
| O-1 | OKF:29-31 | implements ARCH, adds no rule of its own | Review check plus: every rule in ARCH "Value rules" (ARCH:56-79) has a positive and a negative fixture in raf-okf tests |
| O-2 | OKF:33-35 | parse fails positioned on: no fence on line 1, bad YAML, non-mapping | `"# no frontmatter\n"` -> Err at line 1; `"---\ntype: [\n---\n"` -> Err with line 2; `"---\n- a\n- b\n---\n"` -> Err "frontmatter is not a mapping" with line; `"\n---\ntype: X\n---\n"` (fence on line 2) -> Err line 1 |
| O-3 | OKF:36 | SPDX lines are YAML comments inside frontmatter and are preserved | Any repo doc (e.g. docs/capabilities/okf-profile.md) -> `parse` -> `render` keeps lines 2-3 byte-identical |
| O-4 | OKF:42 | text at most 1 MiB | 1_048_577-byte doc -> Err positioned (offset/line) naming the bound; 1_048_576-byte valid doc -> Ok (OKF:155-157) |
| O-5 | OKF:43-44 | YAML nesting at most 32, checked before recursion | 33-level nested mapping -> Err positioned; 32 levels -> Ok; a 100_000-level input on a 2 MiB thread returns Err (no stack overflow), proving the check precedes descent |
| O-6 | OKF:45-46 | anchors, aliases, merge keys refused | `a: &x [1]\nb: *x` -> Err naming anchor `x`; `<<: *x` -> Err; billion-laughs doc -> Err fast; peak allocation <= k * input size (needs a measuring harness; see R-9) |
| O-7 | OKF:47 | Markdown block nesting at most 64 | 65 nested `>` -> Err positioned; 64 -> Ok; same for nested list items |
| O-8 | OKF:49-51 | validate reports every violation, each naming file, key, rule | Capability with two extra keys and one missing key -> exactly 3 violations, each with `file == path arg`, distinct `key`, rule id |
| O-9 | OKF:51-54 | typed values: state enum, bindings exact keys, affects -> (cap id, section), items -> (claim, optional outcome) | `state: done` -> violation; `bindings: {code: [], tests: []}` -> violation naming `examples`; `affects: [cap:x#usage]` -> parsed `Affects{cap:"cap:x", section: Usage}`; `affects: [cap:x#intro]` -> violation; `items: [{claim: c, outcome: shipped}]` -> violation; `outcome: moved:change:y` -> parsed `Moved("y")` |
| O-10 | OKF:54-55 | extra key, missing key, or `resource` rejected | One fixture per type x (each missing key, an extra key, `resource`) -> violation naming key and rule (OKF:150-151) |
| O-11 | OKF:55-56 | `type` other than the three is an error | `type: Concept` -> violation naming `type` |
| O-12 | OKF:56-59 | cross-document checks are not raf-okf's job | `depends: [cap:does-not-exist]` -> `validate` returns no violation for existence (negative of the negative: confirms no hidden graph check) |
| O-13 | OKF:61-69 | body model: sections(heading, slug, line span), links(target, anchor, line), fences, generated(kind, span), PROPOSED(slug, span) | Accessor fixtures as I-6..I-10; slug of `## Usage and limits` -> `usage-and-limits` (GitHub rule, ARCH:155-156) |
| O-14 | OKF:71-72 | malformed PROPOSED, unclosed/nested generated, orphan end -> parse errors with line numbers | Four fixtures, each Err with exact line |
| O-15 | OKF:74-76 | render deterministic and round-trips | I-5 |
| O-16 | OKF:76-79 | key order = profile table order; comments kept; body unchanged except caller-supplied spans | `doc.replace_span(gen_block.span, "new")` then `render()` -> output differs from input only inside that span; frontmatter comments retained (C-4) |
| O-17 | OKF:81-83 | `docs/capabilities/index.md` accepted only with a generated block; never parsed as Capability; hand-written is a violation | Generated-only index -> Ok; index with one prose line -> violation; index with zero blocks -> violation (mirror DC:3579-3726 cases) |
| O-18 | OKF:87-98 | Usage API compiles | Crate-level doctest equal to the Usage block (`Document::parse`, `Profile::default().validate(&doc, path)`, `doc.sections()`, `doc.replace_span`, `doc.render()`) compiles and runs |
| O-19 | OKF:104-115 | Example accepted; with `resource` rejected naming `resource` and the key set | Exact YAML from OKF:105-111 -> zero violations; plus `resource:` line -> violation message contains `resource` and lists the 7 Capability keys |
| O-20 | OKF:119-120 | touches no file, runs no process | `raf-okf` has no `std::fs`/`std::process` use: a `repo_gates`-style test (or `clippy::disallowed_methods` in a crate-local clippy config) failing on `std::fs::`/`std::process::` in `crates/raf-okf/src` |
| O-21 | OKF:123 | no pass-through for extra keys | Same as I-4; also `Document` exposes no "extra keys" accessor |
| O-22 | OKF:124 | valid or invalid, no warnings | Violation type has no severity field (API review + compile-time) |
| O-23 | OKF:126-127 | no position on tense/content | Placeholder token in prose -> `validate` returns no violation (DC keeps that check) |
| O-24 | OKF:163-164 | round-trip property over fixtures | Property test over all goldens + generated variants (no proptest dep needed; deterministic enumeration) |

### 1c. `docs/capabilities/source-index.md` (Contract, Constraints, Verification)

| ID | Source | Statement | Executable acceptance check |
|---|---|---|---|
| S-1 | SRC:27-30 | all file access through `Tree`: list, read, test existence | Trait has three methods (C-5); `crates/raf-source/src` contains `std::fs` only inside the `DiskTree` module (grep-style gate test) |
| S-2 | SRC:28-30 | `DiskTree` reads disk; `MemoryTree` is a path->text map | I-12 |
| S-3 | SRC:32-39 | every opened path checked against canonical root; error names module/attribute/manifest key; not opened | DiskTree fixtures S-27..S-32 each: Err mentions the module or `path`/manifest key, and a sentinel file outside root keeps its atime unchanged / is a FIFO that would block if opened (stronger than atime) |
| S-4 | SRC:35-36 | `DiskTree::new` canonicalises root once | `DiskTree::new("missing-dir")` -> Err (C-13: requires `Result`) ; `DiskTree::new(symlink_to_dir)` -> root equals the target's canonical path |
| S-5 | SRC:40-41 | lexical `..` check insufficient (`link/..`) | S-28 |
| S-6 | SRC:41-43 | refuses non-regular files; does not descend into dir symlinks; loop ends at first link | S-31, S-32 |
| S-7 | SRC:43-47 | file type checked on the opened handle (O_NONBLOCK open, then fstat) | FIFO fixture returns Err within a 10 s watchdog (pattern of DC:3500-3538); code review: no `symlink_metadata`-before-open in DiskTree (contrast with tests/support/mod.rs:303-324 which stats first) |
| S-8 | SRC:49-52, 141 | per-read cap `max_file_bytes`, default 8 MiB, `LoadOptions::max_file_bytes`; larger file errors naming it | 8 MiB+1 file -> Err naming path; `LoadOptions{max_file_bytes: 1024}` + 2 KiB file -> Err; exactly 8 MiB valid file -> Ok (SRC:177-179) |
| S-9 | SRC:52-58, 142-143 | pre-scan budgets: delim 128, `<` 64, prefix ops 256, closure bars 128, else-if 256, skipping strings/comments | Per budget: at-limit file parses; limit+1 -> typed Err naming file and budget; budget-exceeding token sequences inside a string literal, raw string `r##"..."##`, and nested block comment `/* /* */ */` do NOT count |
| S-10 | SRC:58-59, 144 | parse on dedicated thread with 64 MiB stack (stated constant) | `raf_source::PARSE_STACK_BYTES == 64 * 1024 * 1024` (proposed public const, documented); overflow fixtures on the caller's default 2 MiB test thread end in one of the three outcomes |
| S-11 | SRC:59-64 | committed fuzz target `parse_source` at `fuzz/fuzz_targets/parse_source.rs`, corpus `fuzz/corpus/parse_source/` with listed fixtures | File exists; corpus contains each S-20/S-21 shape and at/over-budget files; `cargo fuzz build parse_source` succeeds (R-6) |
| S-12 | SRC:68-70 | residual overflow aborts; never a pass | Child-process harness: abort -> non-zero status, test asserts it is classified as outcome 3 and not as success |
| S-13 | SRC:70-72 | `Cargo.toml` read with same cap; `toml` recursion limit -> error; every refusal typed | 1000-level nested inline table in Cargo.toml -> typed Err, no crash; 8 MiB+1 Cargo.toml -> Err |
| S-14 | SRC:75-80 | packages = dirs with Cargo.toml holding `[package]`; lib/bin/test/bench/example with explicit paths and default layout; test/bench/example non-shipping | One `MemoryTree` per rule (lib, bin, test, bench, example) x (default path, explicit `path =`) asserting `shipping` (SRC:170-172) |
| S-15 | SRC:82-83 | `mod name;` -> `name.rs` or `name/mod.rs` | Two fixtures; plus both present -> Err (C-10b) |
| S-16 | SRC:83-85 | `#[path]` redirects relative to "the directory the module would otherwise use" | See C-10a: fixtures for mod-rs file, non-mod-rs file, and inside an inline module, expected paths per rustc reference |
| S-17 | SRC:85-87 | escape by `..`, absolute path, or symlink -> error naming module + attribute; file not read | `#[path="../x.rs"]` at root, `#[path="/etc/hostname"]` -> Err; symlink case on DiskTree (S-28) |
| S-18 | SRC:87-88 | inline mods followed in place; unresolved `mod` -> error naming file and line | `mod a { pub fn f() {} }` -> `demo::a::f`; `mod missing;` at line 3 of `src/lib.rs` -> Err containing `src/lib.rs:3` (PUR:343 omits the line) |
| S-19 | SRC:90-92 | `#[cfg(test)]` items and modules reached only through them tagged | I-16 plus `#[cfg(test)] mod t;` -> every item in `src/t.rs` tagged (SRC:211-212) |
| S-20 | SRC:94-97 | symbol record = full path, kind, file, line, visibility, test tag; methods under impl type path | `impl Thing { pub fn go(&self) {} }` in `ops.rs`, `Thing` defined in `base.rs` -> `resolve("demo::base::Thing::go")` with file `src/ops.rs`, line, `Visibility::Public` |
| S-21 | SRC:99-104 | use-alias: renames, nested groups, globs; `resolve` follows re-exports with cycle guard; typed NotFound / Ambiguous / Cyclic; glob resolution reported | Fixtures (SRC:213-214): rename; `use a::{b, c::{d}}`; `pub use inner::*` -> resolved item has `via_glob == true`; 3-hop chain; `pub use b::x; (in b) pub use a::x;` cycle -> `Err(Cyclic)`; two globs exporting `x` -> `Err(Ambiguous)`; `demo::nope` -> `Err(NotFound)` |
| S-22 | SRC:103-104 | macro-generated symbols not indexed; consumer docs say so | `macro_rules! m { () => { pub fn gen() {} } } m!();` -> `resolve("demo::gen")` = `NotFound` |
| S-23 | SRC:137-138 | syntactic only: no macro expansion, no cfg eval except test | `#[cfg(unix)] fn a() {}` + `#[cfg(windows)] fn a() {}` -> see C-14 (both indexed => Ambiguous or merged) |
| S-24 | SRC:146-147 | dependency items -> not found | `pub use serde::Serialize;` -> `resolve("demo::Serialize")` = `NotFound` (not a panic, not a guess) |
| S-25 | SRC:148-149, 215 | built once per load; sorted by path; two loads equal | `load` twice on same tree -> `==` on the whole index (derive `PartialEq`) |
| S-26 | SRC:174-191 | bound fixtures, debug AND release, on 2 MiB caller stack | 500 nested `A<`, 4000 `!`, 20000 `&`, 10000 `||`, 20000-link `else if` -> each typed Err naming file; at-budget variants parse; run under `cargo test` and `cargo test --release` (R-5) |
| S-27 | SRC:192-202 | composite/uncounted shapes in child process; exactly one of three outcomes; expected symbol list compared | Harness spawns `current_exe()` with a filter selecting a hidden child test (env var selects fixture), asserts outcome in {typed refusal naming file+budget, exit 0 + exact symbol list, abort non-zero}; exit 0 with wrong list fails |
| S-28 | SRC:203-205 | parse thread named after file; abort message contains name | Child aborts on overflow fixture; stderr contains `thread '<file path>' has overflowed its stack`; a non-overflowing parse observes `thread::current().name() == Some(path)` via a test hook |
| S-29 | SRC:206-208 | corpus replay under `cargo test` | Test iterates `fuzz/corpus/parse_source/*`, each in a child, each in one of three outcomes |
| S-30 | SRC:209-210 | fuzz target builds and runs bounded iterations | `cargo +nightly fuzz run parse_source -- -runs=N` exits 0 (needs new Makefile/CI target, R-6) |
| S-31 | SRC:220-231 | DiskTree containment fixtures (6 cases) with sentinel never read | (1) `src/inner.rs` symlink to outside file via `mod inner;`; (2) `#[path="vendored/../x.rs"]`, `vendored` -> outside dir; (3) `[[bin]] path="../outside.rs"` and absolute path; (4) symlink -> symlink -> outside; (5) dir-symlink loop -> Err, terminates; (6) FIFO as `src/inner.rs` -> Err without blocking (10 s watchdog). Each asserts Err text names the module/attribute/key |
| S-32 | SRC:139-140 | explicit Cargo.toml target paths bounded by the same rule | (3) above, on both `MemoryTree` (lexical) and `DiskTree` (canonical) |
| S-33 | SRC:108-117 | Usage compiles | Doctest of the Usage block (with `?` on `DiskTree::new`, see C-13) |
| S-34 | SRC:121-133 | Example: `MemoryTree::from([...])`, `resolve("demo::run")?.path == "demo::inner::go"` | Exact snippet as a doctest |

### 1d. ARCH rules raf-okf must implement (OKF:29-31 delegates to ARCH)

| ID | Source | Rule | Check |
|---|---|---|---|
| A-1 | ARCH:22-28, 42-44 | three types, closed sets, all keys required | O-10/O-11 per type |
| A-2 | ARCH:56-57 | `id` prefix per type; slug kebab and equals file stem | `validate(&doc, "docs/capabilities/x.md")` with `id: cap:y` -> violation (needs the path argument, which the API already takes) |
| A-3 | ARCH:58-73 | title/description non-empty; state set; bindings exact; depends list of cap ids; changes non-empty; affects format + section set + slug in changes; contract_effect enum; items claim/outcome | One negative + one neighbouring positive fixture each (AGENTS.md:156-157) |
| A-4 | ARCH:76-79 | scalar grammar: quoted closes; plain has no `: `, no leading `> | & * ! % @` or backtick, no block-scalar header; trailing ` # comment` dropped | Mirror DC:2688-2843 cases against `Document::parse` |
| A-5 | ARCH:83-87 | Capability body: exactly seven H2 sections in order, none empty after HTML-comment removal | Missing, reordered, duplicated, comment-only section -> violations |
| A-6 | ARCH:99-100 | Change body non-empty `## Summary`; Architecture body non-empty | Fixtures |
| A-7 | ARCH:117-131 | PROPOSED syntax exact; only in Capability documents | Malformed spellings -> parse Err; marker in Change/Architecture -> validate violation |
| A-8 | ARCH:139-144 | generated block once-opened, once-closed, never nested | O-14 |
| A-9 | ARCH:46-50 | index: no frontmatter, SPDX comment allowed, every non-blank line inside a block, markers at line start | O-17 + indented-marker and mid-line-marker negatives (DC:3602-3726) |

---

## 2. Conflicts and ambiguities, with recommended resolutions

Governing rule: the capability docs are the specification (AGENTS.md:49-51), and issue:37
requires the docs to match what was built. Where the build must go beyond a doc, edit the doc
in the same change (the change records list `#contract`, `#usage`, `#examples`,
`#constraints`, `#verification` as affected, so such edits are in scope).

**C-1. Base OKF keys `resource`, `tags`, `timestamp` (issue:19) vs closed RAF profile.**
ARCH:24-26 and OKF:54-55, 114-115 reject `resource`; the closed sets (ARCH:42-44) contain no
`tags` or `timestamp`; OKF:123 forbids pass-through; OKF:29-31 forbids extra rules. GTS-OKF:99-101
and GTS-OKF:45 describe the lenient base reader the issue cites.
Resolution: implement only the RAF profile. `type` required (as a missing-key violation),
`title`/`description` typed per ARCH:58; `resource` rejected with a named rule; `tags` and
`timestamp` rejected as keys outside the closed set. Do not add a lenient "base OKF" mode (it
would be a pass-through, contradicting OKF:123). The issue's "follow the base OKF v0.1 shape"
is satisfied in the sense that RAF documents are valid base OKF (Markdown + YAML frontmatter
with string `type`). Record this in the PR description, not in committed files.

**C-2. Section spans: "byte spans" (issue:22) vs "line span of its content" (OKF:64-65).**
Both are needed: `replace_span` (OKF:97, 77-78) must splice bytes, and errors/consumers use
lines. Resolution: a `Span` carrying both a 1-based line range and a byte range; update
OKF:64-65 to "the line span and byte range of its content" and OKF:68-69 likewise for
generated and PROPOSED spans. Define whether a section span includes its heading line (DC
`sections()` at DC:470 returns content only; keep "content" = after the heading line, up to the
next H2 or EOF, and state it).

**C-3. Writer ordering: "sorted where the spec says so" (issue:21) vs profile-table order (OKF:76).**
GTS-OKF:154-155 sorts keys and tags; RAF does not. No RAF value is sorted by rule.
Resolution: emit keys in ARCH:42-44 table order; do not sort list values (`depends`,
`changes`, `affects`, `items` keep author order). Consequence: `render(parse(x)) == x` holds
only for canonical input (OKF:163-164 already says so); a document with keys out of order is
still valid (ARCH does not require order) but is re-ordered by `render`. State in OKF Contract
whether out-of-order keys are a violation; recommendation: not a violation, since ARCH has no
order rule and OKF:29-31 forbids inventing one.

**C-4. Comments: "comments are kept" (OKF:77, 125) vs "a trailing ` # comment` after a value is dropped" (ARCH:78-79).**
ARCH means the comment is not part of the value. Resolution: the value excludes the trailing
comment (validation sees the bare scalar), and the writer re-emits the comment on the same
line. Define attachment: full-line comments attach to the key that follows them (so the SPDX
lines stay above `type`); trailing comments stay on their key's line; comments after the last
key stay last. Add a round-trip golden containing each. Clarify ARCH:78-79 wording to "is not
part of the value" if the implementer agrees, otherwise the two texts read as contradictory.

**C-5. `Tree` shape: issue:26 "`Tree` trait (`read`, `list`) with `Disk` and `Memory`" vs SRC:27-30 `DiskTree`/`MemoryTree`, "list a directory, read a file, test existence".**
Resolution: doc names (`DiskTree`, `MemoryTree`, `Tree`) and three methods. PUR:57-62 returns
`Option<String>` and `Vec<(String,bool)>`, swallowing I/O errors (PUR:72-90): that is a silent
fallback forbidden by AGENTS.md:149-151. Make every method return `Result<_, TreeError>`
(not-found distinct from refused/IO), and pass the byte cap into `read` (or have the tree
enforce `LoadOptions`) so `DiskTree` reads at most cap+1 bytes via `Read::take` and never
allocates the whole oversize file. Non-UTF-8 source is a typed error, not lossy.

**C-6. `index.md` (issue:23, generic navigation, GTS-OKF:52, 104) vs OKF:81-83 / ARCH:46-50 (only `docs/capabilities/index.md`, only generated blocks, hand-written rejected).**
Also `Document::parse` must fail without a frontmatter fence (OKF:34), so the index cannot go
through it, and raf-okf "touches no file" (OKF:119) so it cannot know paths except via the
`validate` path argument. Resolution: add an explicit entry point, e.g. `raf_okf::Index::parse(text)`
(name to be fixed in OKF Usage), applying ARCH:46-50 and returning its generated blocks; plus a
path predicate (`Profile::is_capability_index(path)`) so callers route the file. Other
`index.md` files elsewhere are not RAF documents at all; raf-okf takes no position (callers
decide what to scan). Update OKF "Index handling" and Usage to name the API.

**C-7. Non-shipping target naming `<pkg>::test_crate::<target>` (issue:28; PUR:47-53, PUR:1184) is absent from SRC.**
It matters because workplan bindings name test functions and example targets
(workplan-document-model.md:98-101). Resolution: adopt PUR naming (`test_crate`,
`bench_crate`, `example_crate`, hyphens -> underscores) and write it into SRC Contract
"Package discovery" and Examples. Also define: a target root resolves as a module-kind item
(so `bindings.examples: [demo::example_crate::e]` resolves); a first-party module literally
named `test_crate` at crate root collides -> `Ambiguous`, never first-wins (PUR:658-660 uses
`or_insert`, i.e. first wins: must change).

**C-8. Build script: issue:27 "lib/bin/build.rs roots" vs SRC:76-80 (lib, bin, test, bench, example only).**
PUR:1152-1157 indexes `build.rs` as `<pkg>::build_script`, shipping. Resolution: include the
build script as a target kind (`TargetKind::BuildScript`), state its naming and its shipping
flag in SRC, and add a discovery fixture. Recommendation: `shipping = true` (it is first-party
code the census must see) but distinguished by kind so workplan consumers can filter.

**C-9. Default roots: issue:27 "default `crates/`" vs census-ledger.md:43-44 "`roots` ... defaults to the workspace root" vs SRC (no default; Usage passes `&["crates"]`, Example `&["."]`).**
Resolution: `Workspace::load` takes roots explicitly with no default inside raf-source; the
census default lives in census-ledger. Also clarify discovery depth: SRC:75-76 "finds every
package: each directory holding a `Cargo.toml`". PUR:223-229 only checks immediate children
of each root; the SRC Example (`&["."]` with `Cargo.toml` at `.`) requires the root itself to
count. Recommend: a root that holds a `[package]` manifest is a package; otherwise recurse
(skipping `target/`, hidden dirs, and directory symlinks) and state it in SRC. Decide what
happens with nested packages (the repo root is both `[package]` and `[workspace]`, Cargo.toml:4-20,
and `crates/*` are below it): recommend recursing below a package too, since this repo's own
root package would otherwise hide `crates/`.

**C-10. Module path rules ("as rustc does it", issue:27) vs SRC:83-85.**
(a) SRC:84-85 says `#[path]` is "relative to the directory the module would otherwise use".
rustc's rule (Rust Reference, "Module source filenames / the path attribute"): for a `mod`
not inside an inline module, the path is relative to the directory of the file containing the
declaration, for both `mod.rs` and non-`mod.rs` files; so in `src/a.rs`, `#[path="b.rs"] mod x;`
is `src/b.rs`, whereas "the directory the module would otherwise use" is `src/a/`. Inside an
inline module block the rule changes (mod-rs: dir + inline components; non-mod-rs: dir + file
stem + inline components). PUR:561-564 uses `parent_dir(place.file)` regardless of inline
nesting, which is wrong for the inline case. Resolution: implement rustc's rule, rewrite
SRC:83-85 to state it, and add four fixtures (mod-rs file, non-mod-rs file, inline in each).
A file reached through `#[path]` is treated as a mod-rs file for its own children.
(b) Both `name.rs` and `name/mod.rs` present: rustc error E0761; PUR:566-571 silently prefers
`name.rs`. Resolution: typed error naming both files; add to SRC and a fixture.
(c) Raw identifiers: `mod r#match;` -> `match.rs` (KAT:1334-1342 test); keep.

**C-11. "Port from katamari" (issue:30) vs licensing.**
KAT is `SPDX-License-Identifier: AGPL-3.0-only` (KAT:2; katamari Cargo.toml line 2), while RAF
code is `MIT OR Apache-2.0 OR MulanPSL-2.0` (AGENTS.md:135-137). PUR is already under the RAF
licence (PUR:1-2). Copying AGPL text into RAF needs an explicit relicensing decision by the
copyright holder (the katamari header names "Blackcat Informatics Inc." without the registered
mark, otherwise the same holder). Resolution: take the code text from PUR; take only ideas from
KAT (configurable parents `walk_dirs`, KAT:243; `#[path]` escape detection, KAT:863-890) and
re-implement them; or obtain and record the maintainer's relicensing decision before copying.
Flag for the maintainer.

**C-12. Fail-hard deltas in both port sources.** Each is a forbidden fallback under AGENTS.md:149-151
and must not be ported as is:
- Tree methods return `Option`/empty vec on I/O error (PUR:72-90).
- Escaping or out-of-package files are silently excluded, not errors (PUR:326-333, PUR:1007-1033);
  `join_normalised` is lexical (PUR:991-1003) and pops past the root.
- Test-only items and `#[test]` fns are dropped (PUR:371-373), contrary to issue:27 and SRC:90-92.
- Hand-rolled `Cargo.toml` line parser (PUR:1050-1104) ignores single quotes, inline tables,
  `[lib] path` in dotted keys, `autotests = false`, etc.; SRC:70-72 requires the `toml` parser.
- `resolve` returns `Option` with BFS first-match (PUR:804-839), so ambiguity is invisible and a
  4096-node cap silently gives up; SRC:101-103 requires typed NotFound/Ambiguous/Cyclic and a glob flag.
- `settle` falls back to `module::LastSegment` when an impl self type does not resolve (PUR:740-748).
- `define` keeps the first of duplicate paths (PUR:658-660).
- No line in the unresolved-mod error (PUR:343).
- No size cap, no pre-scan, parse on the caller's stack.
- Census-only material (Unit, fingerprints, rule hits, relex) stays out of raf-source.

**C-13. `DiskTree::new` signature.** SRC:35 says it canonicalises the root (fallible), but
Usage SRC:111 writes `&DiskTree::new(".")` without `?`. Resolution: `DiskTree::new(root) -> Result<DiskTree, TreeError>` and fix Usage to `&DiskTree::new(".")?`.

**C-14. Duplicate definitions under mutually exclusive `cfg`.** SRC:137-138 says no cfg
evaluation except `test`, so `#[cfg(unix)] fn a()` and `#[cfg(windows)] fn a()` both exist
syntactically. Resolution: record both; `resolve` returns `Ambiguous` with both locations
(fail-hard), and SRC states it. Methods from two trait impls with the same name on one type
(SRC:96-97 "under their impl type path") collide the same way; state it.

**C-15. `LoadOptions` vs Usage.** SRC:50 names `LoadOptions::max_file_bytes` but Usage/Example
call `Workspace::load(&tree, roots)` with no options. Resolution: keep `load(tree, roots)`
with defaults and add `Workspace::load_with(tree, roots, &LoadOptions)` (name to be written
into SRC Usage).

**C-16. Pre-scan definitions are underspecified (SRC:52-58).** "nested `<`" cannot be told from
comparison lexically (`a < b && c < d` would accumulate); `&&` is one token or two; `!` after an
identifier is a macro bang; `||` is both an empty closure head and logical-or; `'a` lifetime vs
`'a'` char; nested block comments and raw strings with N hashes must be skipped exactly or the
pre-scan is bypassable. Resolution: define each counter in SRC (e.g. "`<` depth resets at
`;`, `{`, `}` and at any closing delimiter of the enclosing group; `<` is counted only after
an identifier, `::` or `<`") and add a neighbouring-valid fixture per budget so ordinary code
(e.g. 300 comparisons in one function) is not refused (AGENTS.md:156-157).

**C-17. `Profile::validate` signature and violation type.** OKF:92 passes a path string, OKF:50-51
requires file/key/rule per violation. Specify a `Violation { file, key: Option<String>, rule: Rule, line: Option<usize> }`
(body violations have no key) and make `Rule` an enum so tests assert the rule, not message text.

**C-18. Which link forms.** Issue:22 says `[text](path)`; OKF:66 says "links, each with its
target, its anchor and its line"; ARCH:150-154 covers inline links, images, reference
definitions and inline-HTML `href`/`src`. OKF:137-140 says no other crate scans Markdown.
Resolution: extract all forms ARCH names, tagged by form, single-line only (ARCH:153-154).

---

## 3. What `delivered` vs `implementing` requires

### Common to both states (state changes from `designed`)
1. `tests/docs_contract.rs` INVENTORY (DC:102-103): change the two rows. AGENTS.md:173-176
   requires this in the same commit as the state change.
2. `inventory_table_has_the_documented_shape` (DC:2549-2563) hard-codes `count("designed") == 16`
   and `count("considered") == 5`: change to 14 designed and add an assertion for the new
   state count (2 implementing or 2 delivered). Not mentioned in the issue; will fail otherwise.
3. Root `Cargo.toml:20` `members = []` -> `["crates/raf-okf", "crates/raf-source"]`; each crate
   uses `version/edition/rust-version/license/... .workspace = true` and `[lints] workspace = true`.
   Add `exclude = ["fuzz"]` (or give fuzz its own `[workspace]`, see R-6). `Cargo.lock` regenerates
   (all gates run `--locked`).
4. Prose that becomes false: AGENTS.md:41-45 (gates "join them once those crates are implemented"
   still true for census/workplan, keep), AGENTS.md:49-51 ("Every crate below is designed, not
   yet implemented. The workspace members list is empty"), AGENTS.md:55-56 (status column
   `designed`), AGENTS.md:70 ("none exist yet; `members` is empty"); README.md:21 and :26-27
   ("designed — not yet published"); ARCH:30-36 ("The `raf-okf` and `raf-workplan` crates will
   enforce the whole profile once they are implemented"; "resolving a binding waits for the
   source index"). SECURITY.md:80 "Once implemented" stays true for the unimplemented surfaces
   but should say the source/OKF surfaces now apply. CHANGELOG.md [Unreleased] gets entries
   for both crates. Rewrite in present tense; no process wording (support/mod.rs:30, :44-87).
5. Capability docs: replace any Contract/Usage/Examples/Constraints/Verification statement the
   build diverges from (C-2..C-18). No PROPOSED blocks are needed (the change records already
   list those sections in `affects`; text is edited in place since both caps are `designed`).
   Keep all seven sections non-empty, no placeholder tokens (DC:68-74).
6. New files need SPDX headers (REUSE); `make reuse` runs on the staged snapshot in the hook.

### `implementing`
- Frontmatter `state: implementing`; bindings may be empty or partial (DC:722-748 imposes
  nothing for `implementing`; workplan-document-model.md:102-104 says gaps are reported).
- Change records `work/changes/okf-profile.md` and `work/changes/source-index.md` stay; DC:1506-1514
  requires an open record listing the cap, and check_inventory (DC:1749-1755) requires
  `work/changes/<slug>.md` to exist.
- ARCH:110 "The designed contract, stated plainly" + an open record.

### `delivered`
- Frontmatter `state: delivered`; `bindings.code` and `bindings.tests` each non-empty
  (DC:733-745). DC checks only counts, not that symbols exist (ARCH:33-35); nonetheless ARCH:60-62
  says a list never names a missing symbol, so every binding must be a real path resolvable by
  the new `raf_source::Workspace::resolve`. Recommended: add a test in the root package (or in
  raf-source's own tests) that loads this repository with `DiskTree` and resolves every binding
  of every capability; this is also the strongest dogfood check of raf-source. Note this makes
  the root package take a dev-dependency, contradicting tests/support/mod.rs:9 ("the root
  package takes no dependencies") and DC:10 ("Only the standard library is used"); if chosen,
  edit those comments. Alternatively put the test in `crates/raf-source/tests/`.
- Binding path format: code e.g. `raf_okf::Document::parse`, `raf_source::Workspace::load`; tests
  in the C-7 naming, e.g. `raf_okf::test_crate::<target>::<fn>` for `crates/raf-okf/tests/<target>.rs`;
  unit tests inside `#[cfg(test)] mod tests` are `raf_okf::<module>::tests::<fn>`.
- No PROPOSED blocks (DC:1487-1493).
- No open change record lists the cap (DC:1494-1502; ARCH:111): delete both
  `work/changes/okf-profile.md` and `work/changes/source-index.md` (ARCH:167-168; AGENTS.md:183-186).
  If not deleted, check_inventory also flags them as "not in the inventory table" (DC:1764-1771).
- Item outcomes: AGENTS.md:179-180, 184 and ARCH:72-75 require every item to carry an outcome
  when the change completes. DC does not enforce it (outcome optional, DC:919-928), and the record
  is deleted on completion, so the outcomes exist only in history. Recommended sequence: one commit
  sets `outcome: implemented` on all 6 (okf) and 8 (source) items with the cap at `implementing`;
  the completing commit flips state to `delivered`, adds bindings, edits INVENTORY, and deletes the
  records. Any item that is not fully built must be `moved:change:<slug>` to a new record that
  exists (DC:1559-1567); but that new record would list the same cap, which then cannot be
  `delivered`. So `delivered` requires every item fully implemented, including the fuzz target,
  the release-mode runs and the child-process harness (source item 7 and Verification SRC:192-210).
- Items to satisfy (`work/changes/okf-profile.md:17-22`): positioned parse errors; closed keys +
  typed values + all violations; body model (sections, links, fences, generated, PROPOSED);
  deterministic round-trip writer; index only with generated block; bounds + anchors refused.
  (`work/changes/source-index.md:17-24`): two Tree impls; discovery of 5 target kinds with
  shipping flag; `#[path]` + root containment; cfg(test) tag; symbol + alias with typed failures;
  canonical containment, non-regular refusal, symlink loop; size cap + pre-scan + dedicated stack
  + fuzz target; fstat on opened handle.

---

## 4. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| R-1 `unicode-ident` licence `(MIT OR Apache-2.0) AND Unicode-3.0` (registry: unicode-ident-1.0.26). `syn` and `proc-macro2` always depend on it, and `Unicode-3.0` is not in `deny.toml:12` allow list -> `make deny` fails | HIGH (certain) | HIGH (gate blocks) | Narrow addition: `[[licenses.exceptions]] crate = "unicode-ident"`, `allow = ["Unicode-3.0"]` rather than a global allow; deny.toml:10-11 says an addition needs a deliberate decision, so get maintainer sign-off and note it in the PR |
| R-2 Other licences: `syn`, `proc-macro2`, `quote`, `toml` 1.1.6 and `toml_parser`/`toml_datetime`/`toml_writer`/`serde_spanned`, `serde_core`, `libc`, `indexmap`, `equivalent`, `tempfile`, `fastrand`, `getrandom`, `once_cell`, `windows-sys`, `bitflags`, `cc` are MIT/Apache; `winnow` and `pulldown-cmark` MIT; `rustix`/`linux-raw-sys` `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` (passes via a plain disjunct); `memchr` `Unlicense OR MIT` (passes). Failing: `foldhash` Zlib (pulled by `hashbrown` default features, e.g. via `hashlink`/`yaml-rust2`, or `toml`'s optional foldhash), `encoding_rs` `AND BSD-3-Clause`, `libfuzzer-sys` `(MIT OR Apache-2.0) AND NCSA` | MED | HIGH | Avoid a YAML crate entirely (see R-3); keep `toml` default features (check `cargo tree -e features` shows no foldhash); keep `libfuzzer-sys` outside the main workspace graph (R-6) or add an NCSA exception scoped to it |
| R-3 YAML library choice. `serde_yaml` is deprecated (0.9.34+deprecated), drops comments (breaks OKF:76-78 comment keeping) and expands aliases before you can refuse them; `yaml-rust2`/`saphyr` pull hashlink/hashbrown/foldhash (Zlib). | HIGH if a crate is used | HIGH | Hand-write a bounded parser for exactly the ARCH:76-79 subset (block maps, block/flow sequences of scalars, list of maps for `items`, nested map for `bindings`, comments). The subset already forbids `&`, `*`, `!`, `|`, `>` as plain-scalar starts, so anchors/aliases/tags are refused by grammar; nesting is counted iteratively. DC already contains such a parser (DC:137-411); note the census would flag a second copy once both exist, so plan to have DC call raf-okf later (or now, with the dependency caveat in section 3) |
| R-4 `multiple-versions = "deny"` (deny.toml:21). syn 3 is current (registry has syn-3.0.7; `serde_derive` 1.0.229 and `thiserror-impl` 2.0.21 depend on syn 3), while both port sources use syn 2 (purrdf Cargo.toml:131). Mixing syn 2 and 3, or two `windows-sys`/`hashbrown` majors via dev-deps, fails `cargo deny` | MED | HIGH | Use syn 3 (`features = ["full", "visit", "parsing", "printing"]`) and proc-macro2 with `span-locations` (needed for lines); port syn 2 API calls carefully; avoid `thiserror` unless its syn major matches; avoid `tempfile` (DC:3467-3496 already shows a std-only scratch dir) to keep the target-agnostic graph small; run `cargo deny --offline check` after each dependency addition |
| R-5 Release-mode requirement (SRC:181-182, 197). `make test` and CI run only `cargo test --workspace --locked` (Makefile:28-29, ci.yaml test job) | HIGH | HIGH (Verification unexecuted -> cannot claim delivered) | Add `cargo test --workspace --locked --release` to `make test` (or a `test-release` target wired into `check`) and a CI job; child processes must `env_remove("RUST_MIN_STACK")` so the caller-stack premise holds |
| R-6 Fuzz target placement. cargo-fuzz expects `fuzz/Cargo.toml` with `cargo-fuzz = true` metadata and its own `[workspace]`; it needs nightly + `cargo-fuzz` installed; `libfuzzer-sys` builds C++ with `cc` and carries NCSA. If `fuzz` joins the root workspace, `make msrv` (`cargo check --workspace --all-targets` on 1.98.1, Makefile:50) and `cargo deny` see it. If it is excluded, clippy/fmt/deny do not cover it, and nothing currently builds it | HIGH | MED | Keep `fuzz/` as its own workspace, add `exclude = ["fuzz"]` to root `[workspace]`; add a Makefile `fuzz` target (bounded `-runs=`) that fails hard if `cargo-fuzz` is missing (AGENTS.md:112-113) and a nightly CI job; run `cargo fmt`/`clippy` on `fuzz/` explicitly in that job; give `fuzz/Cargo.lock` a REUSE annotation |
| R-7 Corpus files vs repository gates. `issue_refs_tracked_files_are_clean` scans every tracked text file (repo_gates.rs:314-326, support/mod.rs:199-220): a corpus input with `#12`, `PR 3`, `issue 4`, `task 5`, `AC1`, `GH-7` fails `make issue-refs`; only NUL-containing files are skipped (support/mod.rs:348-353). Corpus files cannot carry SPDX headers -> `reuse lint` fails without a REUSE.toml annotation. Also the docs_contract scans only Markdown under docs/work/top level, so `.rs` fixtures are safe there | HIGH | HIGH | Annotate `fuzz/corpus/**` in REUSE.toml; curate committed corpus inputs so none matches the banned patterns (do not weaken the gate); if a crashing input contains such a token, minimise it or add a NUL byte that keeps it equivalent; record this constraint in SRC Constraints. Same care for test fixture strings in `.rs` files (e.g. avoid `"#1"` literals) |
| R-8 Platform. O_NONBLOCK and FIFOs are Unix; `std::os::unix::fs::OpenOptionsExt::custom_flags(libc::O_NONBLOCK)` needs `libc` (or `rustix`) for the constant; Windows has neither. CI runs only ubuntu | MED | MED | `#[cfg(unix)]` implementation; on non-unix either a documented equivalent or `compile_error!`/typed refusal — fail hard, never a silent weaker check; state the platform in SRC Constraints. `libc` is MIT/Apache and needs no `unsafe` for the constant; `File::metadata()` is the fstat on the handle (no unsafe) |
| R-9 Canonicalise-then-open race. SRC:46-47 says "No check separates from the open it guards", but canonicalisation is a separate step from `open`; an intermediate directory swapped to a symlink between the two escapes. Measuring allocation for O-6 needs a counting `#[global_allocator]` (requires `unsafe impl GlobalAlloc`, denied by `unsafe_code = "deny"`, Cargo.toml:37) | MED | MED | Open the canonical path with `O_NOFOLLOW` (final component) and compare the handle's dev/ino with a fresh `symlink_metadata` of the canonical path, or use `openat2(RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS)` via `rustix` on Linux (safe API); state the residual race in SRC if any. For allocation: run the alias fixture in a child process under an address-space limit, or a tightly scoped `#[allow(unsafe_code, reason = "...")]` counting allocator in a test file only (AGENTS.md:124-125 permits reasoned scoped allows; it is not a lint weakening) |
| R-10 MSRV 1.98.1 + edition 2024 + no `#![feature]` | LOW | MED | All candidate deps declare rust-version <= 1.88 (toml 1.85, syn 1.71); `make msrv` proves it. cargo-fuzz needs nightly, so keep the fuzz crate out of `make msrv` (R-6) |
| R-11 Debug-build stack cost: syn uses ~16 KiB/level in debug (SRC:66-67); 128 delimiter levels x composite prefix runs may still overflow 64 MiB in debug; outcome 3 (abort) is then legal but a single overflow in a non-child test kills the whole test binary | MED | HIGH | Run every overflow-capable fixture only in a child process; keep in-process tests to shapes that are refused by the pre-scan |
| R-12 Test runtime: 8 MiB parse in debug, 20000-element shapes, child-process spawns x2 profiles | MED | LOW | Generate fixtures at test time (not committed as giant files, except corpus copies); share one parse per fixture |
| R-13 Scope size: two crates, a pre-scan lexer, a YAML subset parser, a Markdown block scanner, a fuzz crate, a child-process harness, doc rewrites. "No deferrals" (AGENTS.md:199-201) means `delivered` needs all of it | HIGH | HIGH | Plan for `delivered` only if every Verification bullet is executable in `make check` (+ fuzz target); otherwise land at `implementing` with records open and items without outcomes, which the issue explicitly permits (issue:37) |
| R-14 Licence of ported code (C-11) | MED | HIGH | Port from purrdf only, or get explicit relicensing for katamari text |
| R-15 Process-wording gate on new prose: words like "task 3", "this issue", "the plan's" in docs, comments or test names fail `make issue-refs` (support/mod.rs:30, 81-86, 146-167) | MED | MED | Review new text; never reference the tracker |

### Technical notes

**Affected areas:** new `crates/raf-okf/`, `crates/raf-source/`, `fuzz/`; root `Cargo.toml`
(members, exclude), `Cargo.lock`, `deny.toml` (exception), `REUSE.toml` (corpus, fuzz lockfile),
`Makefile` and `.github/workflows/ci.yaml` (release tests, fuzz), `tests/docs_contract.rs`
(INVENTORY + shape test), `docs/capabilities/okf-profile.md`, `docs/capabilities/source-index.md`,
`docs/architecture/okf-profile.md`, `work/changes/{okf-profile,source-index}.md`, `AGENTS.md`,
`README.md`, `SECURITY.md`, `CHANGELOG.md`.

**Open questions for the maintainer:**
1. Approve the `unicode-ident` Unicode-3.0 exception (R-1) and, if the fuzz crate is graphed, NCSA for libfuzzer-sys.
2. Relicense katamari `source.rs` text, or port from purrdf only (C-11)?
3. Target state: `delivered` (all Verification incl. fuzz/release/child harness) or `implementing`?
4. Should the root package take raf-source/raf-okf as dev-dependencies so docs_contract resolves bindings and stops duplicating the OKF parser (section 3, R-3)?
5. Build script shipping flag (C-8) and nested-package discovery (C-9).

**Rejected approaches implied by the evidence:** lenient base-OKF mode (C-1); key sorting (C-3);
lexical-only containment (SRC:40-41); stat-before-open (SRC:43-47); dropping test items (issue:27);
hand-rolled Cargo.toml parsing (SRC:70-72).

---

## Stage 2: requirement-to-branch status

Branch `paudley/2-shared-crates-raf-okf-raf-source` at 6c744af against `origin/main` (2c41082).
Status comes from the code on the branch and from tests run in this stage. The Stage 1 task
reports were not taken as evidence.

**Evidence run in this stage (all exit 0):**
- `cargo test -p raf-okf --locked`: 17 targets, 162 integration tests plus 1 doctest.
- `cargo test -p raf-source --locked`: 14 unit tests, 225 integration tests (1 ignored: the corpus
  writer at `crates/raf-source/tests/corpus_replay.rs:206`), 5 doctests.
- `cargo test -p raf-source --locked --release`: the same counts, all passing.
- `cargo test -p rust-agentic-foundations --locked --test docs_contract --test repo_gates`: 137
  and 55 passing.
- Every test name in the two capability `bindings.tests` lists and in `delivery-checklist.md` was
  checked against the test sources. None is missing.
- No `#[ignore]` other than the corpus writer. `disk_tree_containment.rs:9` and
  `workspace_containment.rs:9` are `#![cfg(unix)]`, which matches the ubuntu-only CI.

**Evidence not re-run here:**
- `make fuzz` and `make cross-check` need cargo-fuzz and rustup targets. `raw/make-check.log`
  shows both passing on 6c744af: lines 1306-2021 end with "Done 5000 runs", and lines 2022-2028
  show the macOS and Windows checks.
- Hosted CI never started (account billing; last owner comment).

**Legend:**
- `okf/` = `crates/raf-okf/src/`; `src/` = `crates/raf-source/src/`.
- `O:` = `crates/raf-okf/tests/`; `S:` = `crates/raf-source/tests/`; `DC` = `tests/docs_contract.rs`.
- **MET**: code at the cited lines, and a passing test exercises it.
- **PARTIAL**: part of the statement has no code or no test.
- **UNMET**: not built.
- **UNVERIFIED**: code exists, but nothing run here or in a gate exercises it, or the claim cannot
  be checked from code.

### 2a. Issue scope and acceptance criteria

| ID | Requirement | Implemented at | Demonstrated by | Status | Reason |
|---|---|---|---|---|---|
| IS-1 | Parse Markdown with YAML frontmatter; `type` required | okf/document.rs:50-60; okf/frontmatter/reader.rs:112-140; okf/profile/mod.rs:173-207 | O:refusals::a_document_without_a_fence_on_line_one_is_refused; O:profile::a_missing_type_key_is_the_only_violation | MET | Parse is positioned, and a missing `type` is the only violation reported |
| IS-2 | `title`, `description`, `resource`, `tags` (sorted), `timestamp` follow base OKF v0.1 | okf/profile/schema.rs:66-98 (optional `tags`/`timestamp`); okf/profile/mod.rs:233 (`ResourceKey`), 341-397; okf/document.rs:51,73,190-205 (`sort_tags`); okf/frontmatter/writer.rs:53 | O:profile::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339; O:render::tags_render_sorted_and_stay_sorted; O:profile::the_key_resource_has_its_own_rule_on_every_type | MET | As redefined by user decision 1: `tags`/`timestamp` optional and checked, `resource` rejected by its own rule. It does not "follow base OKF" for `resource` |
| IS-3 | Closed key set per `type`, typed values; unknown key is a hard error | okf/profile/mod.rs:225-250, 253-306 | O:profile::an_unknown_key_is_reported_at_that_key; DC::rejects_unknown_key | MET | Unknown key gives `Rule::UnknownKey`, and there is no warning level |
| IS-4 | Deterministic writer, round-trip stable, sorted where specified | okf/frontmatter/writer.rs:25-56, 151-160; okf/document.rs:167-187 | O:render::every_tracked_document_renders_to_its_own_bytes; O:render::keys_out_of_order_render_in_table_order_and_then_stay_put | MET | Byte-identical over every tracked document |
| IS-5 | Body model: H2 sections with byte spans, links, fenced code, generated and PROPOSED blocks | okf/body/mod.rs:88-300; okf/body/blocks.rs:77-215; okf/body/link.rs:242-400; okf/body/marker.rs:140; okf/body/heading.rs:156-190 | O:body_structure::sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading; O:links::every_link_form_is_read_with_its_target_anchor_and_line; O:markers::generated_blocks_report_kind_outer_and_content_spans | MET | Every span carries both lines and bytes (okf/span.rs:17) |
| IS-6 | `index.md` without frontmatter is navigation, not a document | okf/index.rs:17-19, 49-87, 90-160 | O:index::* (9 tests) | MET | Limited to `docs/capabilities/index.md` (plan C-6). `docs/index.md` is not an index (O:index::only_the_capability_index_path_is_the_index) |
| IS-7 | `Tree` trait (`read`, `list`) with Disk and Memory implementations | src/tree/mod.rs:43-82; src/tree/disk.rs:108-188; src/tree/memory.rs:28,72 | S:memory_tree::*; S:disk_tree_containment::*; S:index_disk::disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions | MET | Also adds `entry`; the types are named `DiskTree`/`MemoryTree` (plan C-5) |
| IS-8 | Discovery under configurable roots (default `crates/`); lib/bin/build.rs roots; rustc mod-tree incl. `#[path]`; `cfg(test)`/`#[test]` tagged, not dropped | src/workspace.rs:74-108; src/discovery.rs:36-209; src/target.rs:209-400; src/modtree.rs:82-535; src/cfg.rs:35-40 | S:discovery_targets::there_are_no_default_roots; S:discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag; S:module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone | MET | "(default `crates/`)" was removed by user decision 2. Per the handoff, an empty root list loads an empty workspace without error |
| IS-9 | Test/bench/example targets non-shipping, `<pkg>::test_crate::<target>` etc. | src/target.rs:36 (`is_shipping`), 287-297 (`crate_path`), 367-397 (build script); src/index.rs:533-590 (root items) | S:discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag; S:target_roots::every_target_kind_resolves_at_its_crate_path_as_a_module_item_of_its_root_file | MET | `build.rs` is a shipping `<pkg>::build_script` (plan C-8) |
| IS-10 | Symbol index of every named item plus the `use`-alias table, so re-exports resolve | src/index.rs:114-420, 591-640; src/settle.rs:52-610; src/resolve.rs:65-110, 367-500 | S:symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility; S:use_resolution::a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap | MET | Handoff item 3: correctness rests on fixtures and has not been compared with rustc |
| IS-11 | Port from katamari `source.rs` (no purrdf deps), with the `Tree` trait from purrdf | crates/raf-source/Cargo.toml:14-22 (deps: `syn`, `quote`, `toml`, `proc-macro2`, unix-only `libc`) | none | UNVERIFIED | "No purrdf deps" holds. Provenance cannot be checked from code, and user decision 5 replaced the clause with "superset of precedents, licensing ignored" |
| IS-12 | `#[path]` cannot escape the workspace root | src/path.rs:127-160 (`RelPath::join` refuses a climb); src/modtree.rs:394-470; src/tree/disk.rs:96-105 | S:module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it (recording `MemoryTree`, the outside file is never read); S:workspace_containment::a_path_attribute_through_an_outside_directory_symlink_with_a_parent_segment_is_refused | MET | Refused lexically in memory and canonically on disk |
| AC-1 | `Memory` fixtures: nested mods, `#[path]`, re-export chains, test/bench/example targets, rejected `#[path]` escape | (as IS-8 to IS-12) | S:module_tree::inline_modules_add_directory_components_for_their_file_children; S:module_tree::a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory; S:use_resolution::a_three_hop_chain_resolves_and_lists_each_hop; S:discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag; S:module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it | MET | All are built on `MemoryTree` |
| AC-2 | OKF round-trip golden tests; an unknown-key fixture fails | okf/document.rs:50,167 | O:render::every_tracked_document_renders_to_its_own_bytes (more than 20 tracked documents read from disk act as the goldens); O:profile::an_unknown_key_is_reported_at_that_key; DC::rejects_unknown_key | MET | There is no separate golden fixture directory; the tracked documents serve as goldens |
| AC-3 | Public APIs documented (`missing_docs` = deny) | Cargo.toml `[workspace.lints.rust]` `missing_docs = "deny"`; crates/raf-okf/Cargo.toml:19-20 and crates/raf-source/Cargo.toml:24-25 `[lints] workspace = true` | Both crates compile under the deny lint in this stage's test builds; `make doc` passes in raw/make-check.log:1262-1270 | MET | The lint is a compile error, so a successful build is the proof |
| AC-4 | `cap:okf-profile` and `cap:source-index` docs match what was built, with state `implementing` or `delivered` | docs/capabilities/okf-profile.md:8 and docs/capabilities/source-index.md:8 (`state: delivered`); both change records deleted | DC (137 pass), incl. DC:908 `repository_bindings_resolve_in_the_source_index` (resolves every binding through `raf_source::Workspace`, with a non-vacuity guard at DC:914-916) | MET | Some doc sentences are only partly true; see SC-13, SC-15, SC-16, OV-2 and SV-6 |

### 2b. `docs/capabilities/okf-profile.md` (as on the branch)

**Contract**

| ID | Statement (doc line) | Implemented at | Demonstrated by | Status | Reason |
|---|---|---|---|---|---|
| OC-1 | Implements the architecture page and adds no rule of its own (80-84) | okf/profile/schema.rs:66-98; okf/profile/mod.rs | O:schema::each_type_lists_its_keys_in_the_order_of_the_architecture_page | MET | The split-link and optional-key rules were also written into docs/architecture/okf-profile.md:23,46-48,66-68,195 |
| OC-2 | Parse fails positioned on: no fence on line 1, YAML that does not parse, a non-mapping (86-88) | okf/frontmatter/reader.rs:126-140, 296-312 | O:refusals::a_document_without_a_fence_on_line_one_is_refused; O:refusals::a_frontmatter_that_is_not_a_block_mapping_is_refused; O:refusals::layout_errors_are_refused_with_their_position | MET | |
| OC-3 | SPDX comments are preserved; comment attachment rules (89-92) | okf/frontmatter/writer.rs:72-81, 145-150 | O:render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line; O:render::every_tracked_document_renders_to_its_own_bytes | MET | |
| OC-4 | The stated YAML subset is accepted and the rest refused; numbers are read as strings (92-94) | okf/frontmatter/scalar.rs:160-229, 323-370; okf/frontmatter/reader.rs:513-517 | O:refusals::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token; O:refusals::numbers_booleans_and_null_are_read_as_strings; O:refusals::block_scalars_are_refused_naming_the_header | MET | |
| OC-5 | Resource bounds: 1 MiB; 32 YAML levels checked before descent; anchors, aliases and merge keys refused; 64 block levels; 4096-byte link part; never a panic, abort or unbounded allocation (96-109) | okf/limits.rs:8-16; okf/frontmatter/reader.rs:112-122, 405, 458; okf/frontmatter/scalar.rs:220-229, 323-360; okf/body/blocks.rs:281-310; okf/body/link.rs:19, 440-487; okf/body/mod.rs:106-154 | O:refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused; O:refusals::a_hundred_thousand_levels_return_an_error_on_a_small_stack; O:body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused (quotes, lists, indentation); O:links::a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused; O:alloc_bound; O:body_robustness | MET | |
| OC-6 | `Profile::validate` returns a typed `RafDocument` or every violation; a violation names the file, the rule, and the key, section or line (111-116) | okf/profile/mod.rs:173-222; okf/profile/violation.rs:16-31, 294-320; okf/profile/typed.rs:224 | O:profile::valid_documents_of_each_type_validate_to_typed_values; O:profile::a_document_with_three_faults_returns_three_violations | MET | |
| OC-7 | One key table per type in architecture order; unknown, missing and `resource` (own rule naming the key set) are violations; `tags`/`timestamp` optional (118-121) | okf/profile/schema.rs:51-98, 156; okf/profile/mod.rs:225-250 | O:schema::tags_and_timestamp_are_optional_and_every_other_key_is_required; O:profile::the_key_resource_has_its_own_rule_on_every_type; O:profile::every_required_key_of_every_type_is_reported_when_missing | MET | |
| OC-8 | A missing or unknown `type` is the only violation (122-123) | okf/profile/mod.rs:176-205 | O:profile::a_missing_type_key_is_the_only_violation; O:profile::an_unknown_type_is_one_violation_at_the_type_key | MET | |
| OC-9 | The document sits directly in the directory of its type (124) | okf/profile/mod.rs:74-93, 207 | O:profile::a_document_must_sit_directly_in_the_directory_of_its_type | MET | |
| OC-10 | `id` prefix and kebab slug equal to the file stem; non-empty `title`/`description` (125-126) | okf/profile/mod.rs:308-348 | O:profile::an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name; O:profile::a_title_and_a_description_are_non_empty_strings | MET | |
| OC-11 | `state` enumeration; `bindings` with exactly code/tests/examples lists; designed/considered bind nothing; delivered binds code and tests (127-129) | okf/profile/mod.rs:399-492 | O:profile::a_state_is_one_of_the_four; O:profile::bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list; O:profile::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests | MET | |
| OC-12 | `depends`/`changes` capability ids (`changes` non-empty); `affects` parsed and in `changes`; `contract_effect`; `items` with claim and outcome (130-134) | okf/profile/mod.rs:494-662 | O:profile::changes_is_a_non_empty_list_of_cap_ids; O:profile::affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes; O:profile::contract_effect_is_none_or_changes; O:profile::items_need_a_claim_known_keys_and_a_known_outcome; O:profile::the_three_outcomes_are_typed | MET | |
| OC-13 | `tags` are unique non-empty strings; `timestamp` is RFC 3339 (135-136) | okf/profile/mod.rs:351-397; okf/profile/timestamp.rs:83 | O:profile::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339; O:schema::malformed_and_out_of_range_timestamps_are_refused | MET | |
| OC-14 | Body rules: seven sections in order and none empty; Change `Summary`; non-empty Architecture body; emptiness measured after comments and generated blocks (137-139) | okf/profile/body_rules.rs:25-147 | O:profile::a_capability_holds_the_seven_sections_in_order; O:profile::a_section_is_empty_when_only_comments_and_generated_blocks_remain; O:profile::a_change_needs_a_non_empty_summary; O:profile::an_architecture_body_is_not_empty | MET | |
| OC-15 | PROPOSED only in a Capability, inside a section (140) | okf/profile/body_rules.rs:149-160 | O:profile::a_proposed_block_is_valid_only_inside_a_section_of_a_capability | MET | |
| OC-16 | No placeholder token in prose, title, description or claim; no split link; `Body::placeholder_lines` uses the same rule (141-145) | okf/profile/body_rules.rs:15, 167-178, 219-240; okf/profile/mod.rs:665-697; okf/body/mod.rs:298 | O:profile::placeholder_tokens_in_prose_are_reported_at_their_line; O:profile::a_split_link_is_reported_at_its_line_and_a_one_line_link_is_not; O:placeholder_lines::* | MET | Implements user decision 4 |
| OC-17 | One document at a time; no position on tense (147-150) | absence of cross-document checks in okf/profile | O:profile::depends_is_a_list_of_cap_ids_and_does_not_need_the_capability_to_exist; O:profile::future_tense_prose_is_not_a_violation | MET | |
| OC-18 | `Body::parse` reads frontmatter-less Markdown; spans in whole-text coordinates with lines and bytes (152-157) | okf/body/mod.rs:88-120; okf/span.rs:17 | O:body_structure::markdown_without_frontmatter_is_the_whole_body; O:body_structure::spans_are_in_whole_text_coordinates_when_frontmatter_precedes_the_body | MET | |
| OC-19 | Headings at levels 1-6 with GitHub anchors and numbered repeats; anchor check (159-160) | okf/body/heading.rs:89-190; okf/body/mod.rs:225-235 | O:body_structure::headings_of_every_level_carry_github_anchors_with_repeats_numbered | MET | |
| OC-20 | H2 sections with heading, slug and a span starting after the heading line (161-162) | okf/body/heading.rs:156-172; okf/body/mod.rs:237-250 | O:body_structure::sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading | MET | |
| OC-21 | Links with target, anchor, line and one of five forms; none in fences, code spans or comments; resolution, with escape and absolute targets as errors; block quotes (163-169) | okf/body/link.rs:24-226, 242-400, 615-659 | O:links::every_link_form_is_read_with_its_target_anchor_and_line; O:links::links_inside_html_comments_are_not_links; O:links::targets_that_leave_the_root_or_are_empty_or_absolute_are_refused | MET | |
| OC-22 | Split links recorded and not read (170-172) | okf/body/link.rs:344-381 | O:links::a_link_split_across_lines_is_recorded_and_not_read; O:links::split_links_are_not_recorded_in_fences_code_spans_or_for_one_line_links | MET | |
| OC-23 | Fences matched by character and length, inside quotes and lists (173-174) | okf/body/blocks.rs:172-215 | O:body_structure::a_fence_closes_only_on_the_same_character_with_at_least_the_same_length; O:body_structure::fences_are_recognised_indented_up_to_three_spaces_and_inside_quotes_and_lists | MET | |
| OC-24 | Generated blocks (kind, outer and content spans); PROPOSED blocks (slug, span, marker, content, section); prose lines (175-180) | okf/body/marker.rs:19-95, 140; okf/body/prose.rs:9-49 | O:markers::generated_blocks_report_kind_outer_and_content_spans; O:markers::a_proposed_block_reports_slug_spans_content_and_section; O:body_structure::prose_lines_skip_fences_and_comments_and_remove_code_spans | MET | |
| OC-25 | Marker errors and nesting over 64 are parse errors with lines (182-184) | okf/body/marker.rs:140-233; okf/body/blocks.rs:281-310 | O:markers::every_misspelling_of_the_proposed_marker_is_refused_with_its_line; O:markers::each_generated_marker_error_names_its_line | MET | |
| OC-26 | `Body::replace_spans` is all or nothing with typed errors; `Document` and `Index` delegate (184-188) | okf/body/mod.rs:320-350; okf/document.rs:104-121; okf/index.rs:81-87; okf/error.rs:409 | O:splice::* (12 tests, incl. the refused non-index edit) | MET | |
| OC-27 | Write: deterministic round trip; key order; `tags` sorted on parse and build; other lists keep order; body unchanged except supplied spans; equality as a map with comments and body bytes (190-200) | okf/frontmatter/writer.rs:25-56, 151-160; okf/document.rs:50-52, 71-73, 167, 190-211; okf/frontmatter/value.rs:404 | O:render::* (20 tests) | MET | |
| OC-28 | Index handling: exact path predicate, header and block rules, `Document::parse` fails at line 1, `render`, `replace_spans` (202-211) | okf/index.rs:17-19, 49-87, 90-160 | O:index::* (9); O:splice::the_catalogue_of_an_index_is_replaced_through_the_index | MET | |

**Usage, Examples, Constraints, Verification**

| ID | Statement | Implemented at | Demonstrated by | Status | Reason |
|---|---|---|---|---|---|
| OU-1 | Usage block runs with no file access (213-243) | okf/lib.rs:31-54 (identical doctest) | raf-okf doctest (1 pass); DC check that the doc blocks equal the crate doctests | MET | |
| OE-1 | Examples block validates; adding `resource` is rejected naming the key and the Capability key set (245-266) | okf/profile/mod.rs:225-240 | O:usage_docs::the_examples_block_validates_between_fences_and_resource_is_rejected | MET | |
| OK-1 | Text only: no file, no process (270-271) | crates/raf-okf/Cargo.toml:14 (no runtime deps) | O:no_io_in_source::* (3) | MET | |
| OK-2 | Input bounds; a bound violation is a parse error (272-274) | as OC-5 | as OC-5 | MET | |
| OK-3 | Only ATX headings and fences; setext is not a heading; indented code is text (275-277) | okf/body/heading.rs:89 | O:body_structure::only_atx_headings_are_headings_and_setext_headings_are_not; O:body_structure::indented_code_is_ordinary_text_so_links_and_markers_in_it_count | MET | |
| OK-4 | Exact keys, no pass-through (278) | okf/profile/mod.rs:225-250 | O:profile::an_unknown_key_is_reported_at_that_key | MET | |
| OK-5 | Valid or invalid, no warning level (279) | okf/profile/violation.rs:294-320 (no severity field) | type-level | MET | |
| OK-6 | Comments preserved; the body is not reflowed (280) | okf/frontmatter/writer.rs; okf/document.rs:167 | O:render::every_tracked_document_renders_to_its_own_bytes | MET | |
| OK-7 | No position on tense or content (281-282) | | O:profile::future_tense_prose_is_not_a_violation | MET | |
| OV-1 | A parse fixture per failure (305-306) | | as OC-2 | MET | |
| OV-2 | A validation fixture **per type** for each missing key, **each extra key**, `resource`, and an unknown `type`, asserting key and rule (307-308) | okf/profile/mod.rs:225-250 (one shared path for all types) | Missing key and `resource` loop over `DocType::ALL` (O:profile.rs:244-272, 296-311). The extra-key fixture O:profile::an_unknown_key_is_reported_at_that_key (O:profile.rs:170-182) and DC::rejects_unknown_key cover **Architecture only** | PARTIAL | No Capability or Change document is tested with an extra key. The shared code path makes a defect unlikely, but the fixture the sentence promises does not exist |
| OV-3 | A value fixture per rule (309-311) | | as OC-11 and OC-12 | MET | |
| OV-4 | Bound fixtures at and over each limit (312-315) | | as OC-5 | MET | |
| OV-5 | Billion-laughs fails fast, naming the anchor, and allocation stays at most input times a small constant (316-318) | | O:alloc_bound::refused_documents_stay_within_eight_times_their_size (`cap` global allocator; at most 8x input + 16 KiB; anchor `lol0` at line 3) | MET | |
| OV-6 | Markers inside a fence make no blocks; one fixture per marker error (319-320) | | O:markers::markers_inside_a_fence_produce_no_blocks; O:markers::each_generated_marker_error_names_its_line | MET | |
| OV-7 | Round trip: `render(parse(x)) == x` and `parse(render(d)) == d` (321-322) | | O:render::every_tracked_document_renders_to_its_own_bytes; O:render::a_parsed_document_equals_its_rendering_read_back | MET | Runs over the tracked documents and inline fixtures, not a generated property |
| OV-8 | A generated index is accepted and a hand-written one rejected (323-324) | | O:index::a_generated_only_index_is_accepted_with_either_header_form; O:index::prose_outside_the_blocks_is_refused_with_its_line | MET | |

### 2c. `docs/capabilities/source-index.md` (as on the branch)

**Contract**

| ID | Statement (doc line) | Implemented at | Demonstrated by | Status | Reason |
|---|---|---|---|---|---|
| SC-1 | `RelPath` validation; `join` drops `.` and refuses absolute, backslash, NUL, drive letter and a climb above the root; keeps `..` below the root (91-96) | src/path.rs:101-200 | S:relpath::* (8) | MET | |
| SC-2 | Three `Result` calls: `read(path, max_bytes)`, `list` sorted with kinds, `entry` giving `None` when absent (96-100) | src/tree/mod.rs:43-82; src/tree/disk.rs:108-188; src/tree/memory.rs | S:memory_tree::entry_reports_file_dir_and_absence_without_an_error; S:disk_tree_containment::listings_are_sorted_by_name; S:disk_tree_containment::entry_kinds_cover_file_directory_symlink_other_and_absent | MET | |
| SC-3 | Object-safe trait with no `Send`/`Sync` bound (100) | src/tree/mod.rs:43; src/workspace.rs:74 takes `&dyn Tree` | compile | MET | |
| SC-4 | The `TreeError` variants name the path; only `NotFound` means absence; a non-UTF-8 name fails the listing (100-104) | src/error.rs:20-75; src/tree/disk.rs:135-142 | S:tree_error::*; S:disk_tree_containment::a_non_utf8_entry_name_fails_the_listing_beside_a_listing_without_one | MET | |
| SC-5 | `DiskTree::new` returns `Result` and canonicalises once; a missing root is `NotFound`; a symlinked root is followed (104-106) | src/tree/disk.rs:71-84 | S:disk_tree_containment::a_missing_root_is_an_error_beside_an_existing_one; S:disk_tree_containment::a_symlinked_root_is_followed_to_its_target_and_containment_is_judged_there; S:disk_tree_root::* | MET | |
| SC-6 | `MemoryTree`: inferred directories, `max_bytes`, `OutsideRoot`, `DuplicatePath`, and infallible conversions that report the error from every call (106-111) | src/tree/memory.rs:28, 72 | S:memory_tree::*; S:read_cap::memory_tree_enforces_the_cap_at_eight_mib | MET | |
| SC-7 | A missing `/proc` is `Io`, never `NotFound` (111-112) | src/tree/disk.rs:397-404 | src/tree/disk/tests.rs::a_missing_descriptor_link_is_an_io_failure_and_never_absence | MET | |
| SC-8 | `entry` gives `OutsideRoot` through an outside directory (112-113) | src/tree/disk.rs:158-187 | S:disk_tree_containment::a_parent_segment_after_an_outside_directory_symlink_is_resolved_on_disk_and_refused | MET | |
| SC-9 | Nothing else in the crate opens a file (113-114) | | S:fs_confinement::* (4) | MET | |
| SC-10 | Canonical-prefix containment for mod files, `#[path]`, target `path`s and manifests; the error names the module, attribute or key; the file is not opened; no lexical clamp (116-126) | src/tree/disk.rs:96-118; src/modtree.rs:394-470; src/target.rs:102-125 | S:workspace_containment::* (12); S:module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it | MET | |
| SC-11 | In-root symlinks are followed; non-regular files refused; no descent into a directory symlink; a loop ends in `SymlinkLoop` (125-129) | src/tree/disk.rs:121-128, 368-377; src/discovery.rs:162-188 | S:disk_tree_containment::a_directory_symlink_loop_ends_with_an_error_and_is_listed_but_never_entered; S:disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted | MET | The directory-symlink refusal in `list` is an `lstat` of the candidate (disk.rs:125) taken before the open. Containment is still enforced on the handle |
| SC-12 | The file type is read from the opened handle; `O_NONBLOCK`/`O_NOFOLLOW` open, so a FIFO cannot hang (129-133) | src/tree/disk.rs:216-223, 476-501 | S:disk_tree_containment::a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file; S:workspace_containment::a_fifo_named_as_a_module_file_is_refused_without_blocking | MET | On Unix. Non-unix uses a plain std open (disk.rs:227-230), per user decision 3 |
| SC-13 | "A listing opens the canonical directory the same way ..., verifies that handle as reads do, and reads the entries through the handle, so a swap between the check and the listing cannot list a directory outside the root" (133-136) | src/tree/disk.rs:121-131, 257-291 | src/tree/disk/tests.rs::swap::a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused (Linux only) | PARTIAL | True only on Linux, where it lists `/proc/self/fd/<fd>`. On other Unix, `listing = canonical` (disk.rs:281) and `fs::read_dir` (disk.rs:137) opens the path again after the identity check, so a swap in between can list outside. On non-Unix no directory handle is opened at all: disk.rs:286-289 uses path `fs::metadata`, and the parameter is misleadingly named `on_handle` at disk.rs:437. The sentence is unconditional, and the per-platform bullets only cover reads |
| SC-14 | Linux: the kernel's path for the handle lies under the root; a directory swap after the check is refused (139-142) | src/tree/disk.rs:338-350, 406-416, 483-485 | src/tree/disk/tests.rs::linux_verifier_refuses_a_handle_link_that_leads_outside_and_accepts_one_inside; src/tree/disk/tests.rs::swap::a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused | MET | |
| SC-15 | Other Unix: canonicalise again plus the same device and inode (143-147) | src/tree/disk.rs:317-336, 419-431, 486-496 | src/tree/disk/tests.rs::unix_verifier_refuses_differing_identities_and_accepts_equal_ones; `make cross-check` (x86_64-apple-darwin) in raw/make-check.log:2022-2028 | UNVERIFIED | The verifier is unit-tested on Linux and compiles for macOS, but the dispatch branch never runs (handoff item 2) |
| SC-16 | Other platforms: a regular-file handle plus a second canonicalisation inside the root; a residual rename window (148-150) | src/tree/disk.rs:451-466, 497-500 | src/tree/disk/tests.rs::portable_verifier_refuses_a_non_regular_handle_and_an_outside_path_beside_an_accepted_one; cross-check x86_64-pc-windows-gnu | UNVERIFIED | Same reason as SC-15 |
| SC-17 | Each read capped at `max_file_bytes` (default 8 MiB, `LoadOptions`); a larger file is an error naming it (152-155) | src/options.rs:7-25; src/modtree.rs:189; src/discovery.rs:87; src/members.rs:166; src/workspace.rs:382 | S:read_cap::* (Tree level); S:discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads; S:trait_keys::an_ancestor_manifest_over_the_size_cap_is_a_typed_error (`load_with`) | MET | Code applies the cap on every read path. For the gap in source-file fixtures, see SV-6 |
| SC-18 | The pre-scan lexer skips comments (nested), strings, raw/byte/C strings and char literals, and tells lifetimes from chars (156-160) | src/prescan.rs:255-405 | S:prescan_budgets::tokens_inside_strings_raw_strings_and_comments_count_for_nothing; S:prescan_budgets::a_lifetime_is_not_the_start_of_a_character_literal | MET | |
| SC-19a | A `Budget` error names the file, budget, limit and line; `Syntax` gives the parser's line and column, read on the parse thread (160-164) | src/prescan.rs:405-470; src/parse.rs:38-80 | S:prescan_budgets::the_refusal_names_the_line_where_the_count_went_over; S:prescan_budgets::a_syntax_error_reports_its_own_line_and_column_beside_a_parsing_neighbour | MET | |
| SC-19b | A thread that cannot start is a `Spawn` error (164) | src/parse.rs:63; src/error.rs:242 | src/error.rs::tests::a_spawn_error_displays_the_file_and_carries_its_cause (builds the variant by hand) | UNVERIFIED | The spawn-failure path is never provoked. The test checks only Display and source |
| SC-20 | One budget table and five counter definitions with their limits and reset rules (164-185) | src/budget.rs:34-75; src/prescan.rs:110-253 | S:prescan_budgets::* (16); src/budget.rs unit test | MET | Each budget is tested at its limit and one over |
| SC-21 | Module nesting 128 and module walks 262144 per load; a `Budget` error names the file and line (187-200) | src/modtree.rs:214-232, 287-300; src/members.rs:38-50 | S:module_tree::module_depth_of_exactly_the_budget_loads_and_one_more_is_refused; S:module_tree::a_chain_of_files_that_each_declare_the_next_twice_is_refused_quickly_beside_a_small_diamond; S:module_tree::inline_modules_with_path_alternatives_are_bounded_by_the_module_path_budget | MET | |
| SC-22 | A dedicated thread named after the file, with a 64 MiB stack (`PARSE_STACK_BYTES`); `parse_source` runs the visitor (202-205) | src/parse.rs:17, 38-80 | S:overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack; S:overflow_harness::deep_but_admitted_source_parses_on_the_dedicated_stack_not_the_callers | MET | |
| SC-23 | A committed fuzz target and corpus holding the composites, single shapes, at- and over-budget files, and every input that ever aborted (205-210) | fuzz/fuzz_targets/parse_source.rs:14-21; fuzz/corpus/parse_source/ (30 files) | S:corpus_replay::every_named_shape_has_a_corpus_file_and_no_file_is_unrecorded; S:corpus_replay::corpus_files_hold_exactly_the_text_of_their_shapes | MET | "Every input that ever aborted" is a historical claim and cannot be checked |
| SC-24 | A residual overflow aborts the process and is never a pass; no budget counts the composite and uncounted shapes (210-221) | crates/raf-source/tests/support/child.rs:111-125 (classifier) | S:overflow_harness::composites_and_uncounted_shapes_end_in_exactly_one_allowed_outcome; S:overflow_harness::the_classifier_accepts_only_a_signal_a_stack_overflow_status_or_the_runtime_message_as_an_abort | MET | Handoff item 1: most composite and uncounted shapes still abort. The Contract allows that, but a consumer loading untrusted source sees process aborts, not typed errors |
| SC-25 | Evasion shapes are counted and refused (218-220) | src/prescan.rs:117-166 | S:prescan_budgets::a_block_as_the_condition_of_each_else_if_still_counts_the_chain; S:prescan_budgets::a_lifetime_between_reference_marks_does_not_end_the_run; S:corpus_replay::evasion_shapes_are_refused_naming_their_budget_beside_their_at_budget_neighbours | MET | |
| SC-26 | `Cargo.toml` has the same cap; the toml recursion limit gives an error; every pre-scan refusal is typed (221-223) | src/discovery.rs:87; src/manifest.rs:310 | S:discovery_targets::deeply_nested_inline_tables_are_a_typed_error_on_the_callers_stack_and_a_shallow_one_loads | MET | |
| SC-27 | Explicit roots and no default; `"."` is the tree root (226-227) | src/workspace.rs:74-108 | S:discovery_targets::there_are_no_default_roots; S:discovery_targets::only_the_named_roots_are_searched | MET | User decision 2 |
| SC-28 | Discovery walk: packages, `[workspace]` recursion, no descent below a plain package, skips `target` and dot-directories (228-234) | src/discovery.rs:36-160 | S:discovery_targets::a_package_nested_below_a_package_without_a_workspace_is_not_discovered; S:discovery_targets::target_and_hidden_directories_are_skipped_beside_a_discovered_sibling | MET | |
| SC-29 | Symlinks classified by what they resolve to, reading at most one byte (234-237) | src/discovery.rs:162-188 (`read(path, 0)`) | S:workspace_containment::discovery_classifies_a_symlink_by_what_it_resolves_to | MET | |
| SC-30 | A virtual manifest adds nothing; duplicate names name both manifests; one manifest from two roots is one package; `toml` parsing; `Manifest` errors; sorted by manifest path (237-244) | src/discovery.rs:73-130, 193-209; src/manifest.rs:22-310 | S:discovery_targets::a_virtual_manifest_contributes_nothing_but_its_children_are_found; S:discovery_targets::two_packages_with_one_name_are_an_error_naming_both_manifests; S:discovery_targets.rs:343 (overlapping roots); S:discovery_targets::a_manifest_with_a_trailing_comment_and_single_quoted_values_is_honoured | MET | |
| SC-31 | Target derivation (order, kinds, defaults, `auto*`, explicit tables, crate paths, shipping) and typed target errors naming the key (246-264) | src/target.rs:159-470 | S:discovery_targets::* (25) | MET | |
| SC-32 | Each crate path resolves to the target root as a module item (259) | src/index.rs:533-590, 692-710 | S:target_roots::* (4) | MET | Added in e6f3e9b |
| SC-33 | Mod-tree: `name.rs`/`name/mod.rs`, both present is an error; module-directory rules; inline components (266-271) | src/modtree.rs:82-110, 471-535 | S:module_tree::a_module_with_both_name_rs_and_name_mod_rs_is_an_error_naming_both_files; S:module_tree::inline_modules_add_directory_components_for_their_file_children | MET | |
| SC-34 | `#[path]` base rules (file, inline, on an inline module); a path file counts as mod-rs; two unconditional `path` attributes are an error (271-276) | src/modtree.rs:304-470 (316 `DuplicatePathAttr`) | S:module_tree::a_path_attribute_inside_an_inline_module_is_relative_to_the_module_directory; S:module_tree::a_file_named_by_a_path_attribute_is_a_mod_rs_file_for_its_children; S:module_tree::two_unconditional_path_attributes_are_an_error_beside_one | MET | |
| SC-35 | `cfg_attr` path alternatives followed with their predicates, nested ones flattened, the default followed too; no `cfg` evaluated (276-280) | src/modtree.rs:413-470 | S:module_tree::every_cfg_attr_path_alternative_is_followed_carrying_its_predicate; S:module_tree::a_nested_cfg_attr_path_is_flattened_with_the_combined_predicate | MET | |
| SC-36 | Raw-identifier module; escape error naming module and attribute; unresolved `mod` names file and line (280-284) | src/modtree.rs:551-557 | S:module_tree::a_raw_identifier_module_reads_the_plain_file_name; S:module_tree::an_unresolved_mod_names_the_declaring_file_and_line | MET | |
| SC-37 | Cycle detected on the active stack; nesting 128; a file reached several ways is parsed once; one spelling per file; stop at first error (284-291) | src/modtree.rs:133, 176-270, 536-550 | S:module_tree::a_module_cycle_is_an_error_naming_both_files; S:module_tree::a_file_reached_twice_without_a_cycle_is_parsed_once_and_recorded_under_each_path; S:module_tree::one_file_spelled_two_ways_is_one_file_recorded_under_both_module_paths | MET | |
| SC-38 | `targets`, `files` and owned facts, including the documentation rule (291-299) | src/workspace.rs:172-205; src/facts.rs | S:module_tree::facts_carry_names_lines_documentation_visibility_and_macro_export; S:module_tree::facts_record_struct_shapes_unnamed_consts_leading_colons_and_documentation_forms | MET | |
| SC-39 | `cfg(test)` roles and predicate matrix; `CfgParse` error; modules reached through tests flagged; nothing dropped (301-311) | src/cfg.rs:35-45; src/modtree.rs:572-584; src/index.rs:158-166 | S:module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone; S:module_tree::an_unparseable_cfg_is_an_error_naming_file_and_line_beside_a_parseable_one; S:symbol_index::test_roles_follow_the_module_tree_and_the_item | MET | |
| SC-40 | Every named item kind, its fields and name spaces; `macro_export`; `const _` not indexed; no macro expansion; documentation (313-330) | src/index.rs:114-160, 311-420; src/item.rs:15-160 | S:symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility; S:symbol_index::a_module_a_function_and_a_macro_of_one_name_coexist_and_each_selects_by_namespace; S:symbol_index::documentation_counts_text_and_include_str_but_not_hidden_or_blank_comments | MET | |
| SC-41 | Crate-root items; namespace segments are not items; roots left out of `items_by_source`; collision is `Ambiguous` (332-342) | src/index.rs:533-590, 692-710 | S:target_roots::*; src/index.rs unit test | MET | |
| SC-42 | Methods under the impl type; an inherent impl must resolve locally; trait impls on foreign self types keyed by `TraitKey` (344-351) | src/impls.rs:68-120, 329-440 | S:symbol_index::impl_members_are_recorded_under_the_type_or_under_the_trait_by_the_shape_of_the_impl; S:symbol_index::an_inherent_impl_on_a_type_that_does_not_resolve_is_an_error_naming_file_and_line | MET | |
| SC-43 | Trait-key canonicalisation (dependency heads by target kind and rename, `UnresolvedTrait`, `core`/`alloc` written as `std`, edition preludes); workspace-manifest discovery and membership; glob patterns; dependency globs; `Unresolved` (351-411) | src/impls.rs:17-440; src/workspace.rs:363-460; src/members.rs:22-296; src/glob.rs:24-187; src/manifest.rs:161-256 | S:trait_keys::* (29) | MET | Beyond the issue text (handoff) |
| SC-44 | Item identity, ordering and the lookup API, including `test_fns` with a typed error (413-427) | src/workspace.rs:166-330; src/index.rs:591-640 | S:symbol_index::items_that_share_a_source_item_are_grouped_by_its_identity; S:symbol_index::files_items_in_and_item_at_answer_by_position; S:symbol_index::test_functions_carry_the_package_target_and_crate_relative_name; src/workspace.rs unit test | MET | |
| SC-45 | Determinism, and disk equals memory (427-429) | | S:symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order; S:index_disk::*; S:disk_tree_root::disk_and_memory_trees_agree_on_regular_files | MET | |
| SC-46 | The `use` forms; `super` above the root is an error naming the file and line (431-434) | src/index.rs:168-240; src/resolve.rs:59, 147, 206 | S:use_resolution::a_rename_a_nested_group_and_a_plain_use_each_resolve_to_the_definition; S:use_resolution::super_above_the_crate_root_is_an_error_naming_file_and_line; S:use_resolution::extern_crate_renames_and_leading_colon_paths_reach_the_library_from_other_targets | MET | |
| SC-47 | Explicit imports shadow globs; order independence; glob visibility; fixed point with no cap; `via_glob` (434-445) | src/settle.rs:52-610 | S:use_resolution::resolution_does_not_depend_on_the_order_of_the_declarations (1440 orders); S:use_resolution::globs_import_only_the_names_the_importer_can_name; S:use_resolution::a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap | MET | |
| SC-48 | `resolve`/`resolve_in`, raw identifiers, method paths, `Resolution` fields, cfg twins, typed `NotFound`/`Ambiguous`/`Cyclic`/`Invalid`, the `test_crate` collision, and consumer docs on macro-generated symbols (447-466) | src/resolve.rs:65-110, 367-500; src/workspace.rs:278-292 | S:use_resolution::raw_identifiers_resolve_under_both_spellings; S:use_resolution::a_cycle_of_imports_is_cyclic_beside_a_chain_that_ends; S:use_resolution::an_invalid_path_is_an_invalid_error_beside_a_valid_one; S:symbol_index::cfg_twins_are_one_symbol_with_two_sorted_sites; DC::a_binding_to_a_macro_generated_function_is_not_found | MET | The consumer sentence is in docs/capabilities/workplan-document-model.md |

**Usage, Examples, Constraints, Verification**

| ID | Statement | Implemented at | Demonstrated by | Status | Reason |
|---|---|---|---|---|---|
| SU-1 | Usage `no_run` block (470-489) | src/lib.rs doctest | doctests (5 pass); S:this_repository::this_repository_loads_from_its_root_with_the_crates_directory_as_the_root | MET | |
| SE-1 | Examples block (491-506) | src/lib.rs doctest | doctests | MET | |
| SK-1 | Syntactic only (510-511) | | S:use_resolution::missing_names_dependency_items_and_macro_generated_items_are_not_found | MET | |
| SK-2 | Every opened path is canonical-inside and a regular file (512-513) | as SC-10 and SC-12 | as SC-10 and SC-12 | MET | Proven on Linux. For other platforms, see SC-13, SC-15 and SC-16 |
| SK-3 | Size cap, budgets and module bounds are typed errors; 64 MiB stack; a residual abort fails the gate (514-520) | as SC-17 to SC-24 | as SC-17 to SC-24 | MET | |
| SK-4 | Dependency items are not found (521-522) | | S:use_resolution::missing_names_dependency_items_and_macro_generated_items_are_not_found | MET | |
| SK-5 | Built once per load (523) | no reload API | | MET | |
| SK-6 | Output sorted; loads equal (524) | | as SC-45 | MET | |
| SV-1 | `MemoryTree` fixture per target kind with default and explicit paths, root module recorded (545-548) | | S:discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag; S:discovery_targets::each_target_kind_with_explicit_paths_resolves_beside_its_default_neighbour; S:discovery_targets::the_root_module_of_every_target_is_recorded_under_its_crate_path | MET | |
| SV-2 | Discovery fixtures (549-559) | | S:discovery_targets (25) | MET | |
| SV-3 | Mod-tree fixtures (560-568) | | S:module_tree (33) | MET | |
| SV-4 | `#[path]` fixtures with a recording tree (569-571) | | S:module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it | MET | |
| SV-5 | `cfg(test)` fixtures (572-574) | | as SC-39 | MET | |
| SV-6 | Bound fixtures: "a file over `max_file_bytes` fails with an error that names the file, a lowered `max_file_bytes` is honoured, and **a file of 8 MiB parses at the default**"; delimiters 128/129; shapes over budget fail typed in debug **and** release on the caller's 2 MiB stack; at-budget files parse (575-589) | src/modtree.rs:189 (cap on source reads) | S:read_cap.rs:20-61 calls only `Tree::read` and never loads or parses anything. Manifests are capped through a load (discovery_targets.rs:529-550, trait_keys.rs:433-447). The budget part (S:prescan_budgets.rs:40-95) passed here in debug and in release | PARTIAL | No fixture parses an 8 MiB Rust source file at the default. No fixture refuses an over-cap `.rs` file through `Workspace::load` or `load_with`. The `max_file_bytes` sentences are proven for `Tree::read` and for manifests only. The budget sentences are MET |
| SV-7 | Composite and uncounted shapes in a child-process harness, debug and release (590-600) | crates/raf-source/tests/support/child.rs | S:overflow_harness::composites_and_uncounted_shapes_end_in_exactly_one_allowed_outcome (debug and release, run here) | MET | |
| SV-8 | Evasion fixtures, in process and in the harness (601-604) | | as SC-25 | MET | |
| SV-9 | Syntax-error positions at lines 3 and 5 (605-607) | | S:prescan_budgets::a_syntax_error_reports_its_own_line_and_column_beside_a_parsing_neighbour | MET | |
| SV-10 | Strict harness judge (608-611) | | S:overflow_harness::the_classifier_accepts_only_a_signal_a_stack_overflow_status_or_the_runtime_message_as_an_abort, plus the panic, unknown-selector and exit-3 fixtures | MET | |
| SV-11 | Thread name, and the overflow message carries it (612-614) | | S:overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack; S:overflow_harness::a_deliberate_overflow_aborts_and_the_runtime_names_the_parse_thread | MET | |
| SV-12 | Corpus replay under `cargo test` (615-617) | | S:corpus_replay::every_corpus_file_ends_in_exactly_one_allowed_outcome_with_its_recorded_symbols | MET | |
| SV-13 | The fuzz target builds and runs bounded iterations with inputs cut to 4096 bytes (618-621) | Makefile:28-33, 76-84 (`-runs=5000 -seed=1 -max_len=4096`); .github/workflows/ci.yaml:101-104 | raw/make-check.log:1306-2021 ("Done 5000 runs") | UNVERIFIED | Not re-run in this stage, and hosted CI never started. The only evidence is the owner's local log on 6c744af |
| SV-14 | `cfg(test)` tag on an item and on a module reached through it (622-623) | | S:module_tree::a_cfg_test_module_tags_every_item_reached_only_through_it | MET | |
| SV-15 | Symbol fixtures (624-635) | | S:symbol_index (18), incl. the impl shapes at symbol_index.rs:301-312 | MET | |
| SV-16 | Alias fixtures (636-644) | | S:use_resolution (26) | MET | |
| SV-17 | Order and trait-key fixtures (645-652) | | S:use_resolution::resolution_does_not_depend_on_the_order_of_the_declarations; S:trait_keys (29) | MET | |
| SV-18 | Determinism (653-654) | | as SC-45 | MET | |
| SV-19 | `DiskTree` equals `MemoryTree` on regular files (655-658) | | S:index_disk::disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions | MET | |
| SV-20 | The `DiskTree` containment fixtures, with sentinel unread; the swaps; per-platform verifiers compiled everywhere and run on Linux (659-684) | | S:disk_tree_containment (13); S:workspace_containment (12); src/tree/disk/tests.rs (incl. `swap::*`) | MET | As a fixture list. Runtime coverage of the non-Linux dispatch is SC-15 and SC-16 |

### 2d. Owner comments that change or qualify requirements

- **User decisions** (plan revision comment, recorded 2026-10-12). Each one overrides issue text, and the branch follows all of them:
  1. `tags`/`timestamp` are optional, and `resource` stays rejected. This qualifies IS-2.
  2. No default roots; each consumer owns its default. This replaces "(default `crates/`)" in IS-8.
  3. Containment is closed on Linux, with portable fallbacks elsewhere, `libc` unix-only, and a `make cross-check` gate. This reverses the Stage 1 plan's "unix-only, `compile_error!` elsewhere"; no `compile_error!` exists in `src/`.
  4. Split links are refused. This adds a requirement beyond the issue (OC-16, OC-22).
  5. The port is a superset of the precedents, with licensing ignored. This replaces IS-11's provenance clause.
- **Handoff, least confident:**
  - Hostile Rust input can still abort the process. This is allowed by SC-24, but untrusted-source consumers get aborts, not typed errors.
  - Non-Linux containment is untested at runtime (SC-15, SC-16, and the SC-13 gap).
  - The resolver has been rewritten several times, and its correctness rests on fixtures, not on a check against rustc.
- **Handoff, maintainer actions:**
  - Confirm the crate-scoped `unicode-ident` Unicode-3.0 exception (deny.toml:20-26) and the `libfuzzer-sys` NCSA exception (fuzz/deny.toml:28-33).
  - `make check` now hard-requires cargo-fuzz and the `x86_64-apple-darwin`/`x86_64-pc-windows-gnu` targets.
  - `make deny` previously covered only the root package; it now runs with `--workspace`.
- **Handoff, work beyond the issue:** canonical trait keys, Cargo-accurate workspace membership, and crate-root module items (SC-32, SC-41, SC-43).
- **Handoff, plan reviews:** only the completion adversary re-reviewed the merged plan. The other four reviews saw the pre-merge revision.
- **Task 2 comment correction:** the Task 1 clippy evidence was package-scoped. Workspace clippy applies from Task 2 on. `make clippy` in raw/make-check.log:6-12 covers the final head.
- **CI comment:** hosted CI produced no evidence because of account billing. `gh run rerun 38280511329` is required before merge. Every "CI job" claim (test-release, fuzz, cross-check, deny, msrv) is locally evidenced only.

### 2e. Minor observations (no status change)

- DC:1569: the comment in `rejects_resource_key` says the unknown-key path rejects `resource`. It is now the separate `ResourceKey` rule (okf/profile/mod.rs:233), as the asserted message shows.
- src/tree/disk.rs:435-447: `verify_portable_directory` takes a parameter named `on_handle`, but its only caller (disk.rs:286-288) passes path metadata. Its doc comment says "the handle must be a directory". Both are misleading, and they belong to the SC-13 gap.
- raw/make-check.log:1309-1320: `make fuzz` clippy prints cargo's `non_kebab_case_bins` warning for the `parse_source` binary. It is a cargo lint, so `-D warnings` does not deny it, but the gate output is not clean.
- `.deficiencies` holds no entries below its marker.

### 2f. Summary of items not MET

| ID | Status | One line |
|---|---|---|
| IS-11 | UNVERIFIED | Port provenance cannot be checked from code; user decision 5 superseded it. "No purrdf deps" holds |
| OV-2 | PARTIAL | The extra-key fixture exists for Architecture only, not per type as stated |
| SC-13 | PARTIAL | "Reads the entries through the handle" is true only on Linux; other Unix re-opens by path, and non-Unix opens no handle |
| SC-15 | UNVERIFIED | The other-Unix identity check is unit-tested on Linux but never runs through dispatch |
| SC-16 | UNVERIFIED | The same applies to the non-Unix fallback |
| SC-19b | UNVERIFIED | The `Spawn` error path is never provoked |
| SV-6 | PARTIAL | No 8 MiB Rust source is parsed at the default, and no over-cap source file is refused through a load; the cap is proven only for `Tree::read` and manifests |
| SV-13 | UNVERIFIED | The bounded fuzz run was not re-run here and CI never started; only the owner's local log shows it |
