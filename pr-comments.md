# PR #13 — comments and review threads

13 comment(s). **Every one needs a disposition** in
`review-debt.md`: addressed (with the commit hash) or declined (with the
reason posted in reply). Bot comments count.

## 1. paudley — 

# Plan: Shared crates `raf-okf` + `raf-source` (issue #2)

Worktree `.worktrees/2-shared-crates-raf-okf-raf-source`, branch `paudley/2-shared-crates-raf-okf-raf-source`, base `origin/main`.
Evidence: `issue-analysis.md`, `prior-art-assessment.md`; reviews in `reviews/`.

## Issue summary

Create the first two workspace member crates. `raf-okf` reads, validates against the closed RAF
profile, models the body of, and deterministically re-renders OKF Markdown documents.
`raf-source` is a `syn` workspace index: a `Tree` file seam with disk and memory backends,
package and target discovery, rustc-faithful module following including `#[path]`, a symbol
index and a `use`-alias resolver, under canonical path containment and parse resource bounds.
The issue delivers `cap:okf-profile` and `cap:source-index`, and requires their documents to
match what was built with state `implementing` or `delivered`.

**Target state: `delivered` for both capabilities.** Every Contract, Constraints and
Verification statement in `docs/capabilities/okf-profile.md` and
`docs/capabilities/source-index.md` is built and executed by a gate, both change records get
every item an `implemented` outcome and are then deleted, and bindings name real symbols that
`raf_source::Workspace::resolve` resolves in a test.

## Baseline defaults, standing goals, governing decisions

- No `.baseline`, `.goals`, `CONSTITUTION.md` or ADRs exist for this repository (checked in the
  main checkout). The governing contract is `AGENTS.md` plus the capability and architecture
  documents, which `AGENTS.md` names as the specification.
- AGENTS.md rules that bind every task: fail hard (closed schemas, no fallbacks, no
  best-effort parsing); every rule has a negative fixture; a refusal is tested next to a
  neighbouring valid case; workspace lints `missing_docs`, `unreachable_pub`, `unsafe_code`
  denied and clippy pedantic with warnings denied; every `#[allow]` carries a reason; no
  `#![feature]`; MSRV 1.98.1; Rust-first gates; no tracker numbers or process wording in any
  committed file (enforced by `repo_gates`); SPDX headers on every file (REUSE); conventional
  commits; no attribution.
- `docs/architecture/okf-profile.md` is the normative profile; `raf-okf` implements it and adds
  no rule of its own.

## Prior art and recurrence

- Port source: the superset of every precedent implementation, taking the best part of each, with
  licensing ignored (user decision). See section D for the per-concern start-from / take-from map.
- Port source for `raf-okf`: this repository's `tests/docs_contract.rs` restricted YAML subset,
  fence tracking, marker scanning, anchors, per-document checks and index rule, plus its ~90
  negative fixtures; purrdf's OKF reader (permissive) for fence detection, duplicate-key refusal
  and the CommonMark link-scanner fixes.
- Recurrence: `GhpRsq-Defect-Class` trailers count `none` x1 in RAF; no precedent commit touching
  either `source.rs` or the purrdf OKF reader carries a trailer, so trailer counts for every class
  are **UNKNOWN**. Counted fix commits by reading history: raw-identifier module names x2 (the two
  copies now disagree on spelling); missing test/bench/example targets x1; double-counted shared
  files x1; hard-coded roots x1; `#[path]` escape unnoticed x1; OKF link scanner x3; general YAML
  reader 1 commit fixing 38 bugs; RAF frontmatter hardening 1 commit (6+ classes); link to tracked
  symlink x1.
- Prior attempts in RAF: none. The census walkers held only as census-internal tools; they drop
  test code instead of tagging it, clamp escaping paths instead of refusing them (L4), silently
  prefer `x.rs` over `x/mod.rs` (L3), resolve `#[path]` inside inline modules from the wrong
  directory (L1, L2), ignore manifest values with comments or single quotes (L6), hide ambiguity
  with first-wins inserts and an `Option` resolver with a silent 4096-node cap (L8), and parse
  on the caller's stack with no size cap (L5). Each of these is a named negative fixture below.


## Context

The Stage 1 plan for issue #2 (`.stage/shared-crates-raf-okf-raf-source/plan.md` in worktree
`.worktrees/2-shared-crates-raf-okf-raf-source`) passed only a single combined review. On the
user's instruction, four separate reviews then ran: completeness, constraints, SOLID/ETHOS design
and enhancement/utility. All four returned NEEDS-REVISION. The user also directed that the port
source is the **superset of every precedent implementation, taking the best part of each, with
licensing ignored**. The Stage 1 completion-adversary run was stopped by the user and has not been
re-run.

This file is the revision. After approval it is merged into the stage `plan.md`, which is then
re-posted to the issue. Partial Task 1 code (`crates/raf-okf/src/{error,value,scalar}.rs`, a stub
`lib.rs`, `Cargo.toml` with `cap`) remains uncommitted in the worktree. It is reworked under this
design, not discarded.

Target state is unchanged: `delivered` for `cap:okf-profile` and `cap:source-index`.

## User decisions (recorded 2026-10-12)

1. **Extend the profile with `tags` and `timestamp`.**
   - Both are optional keys on all three document types. `resource` stays rejected.
   - `tags` is a list of non-empty strings. A duplicate tag is a violation, never silently deduplicated.
   - `render` emits `tags` sorted, so canonical input carries sorted tags, and unsorted input re-renders sorted and then stays stable.
   - `timestamp` is an RFC 3339 / `xsd:dateTime` string, validated as in purrdf reader 383-400. A malformed timestamp is a violation.
   - The schema table marks each key required or optional, and both the writer and the validator read that table.
   - The architecture page "Document types and keys" table and its text ("all required") gain the optional keys and their value rules. The okf-profile Contract and Examples are updated to match.
   - Fixtures: sorted render; duplicate tag; malformed and valid timestamp; `resource` still rejected; a document without either key still valid.
2. **No default roots in `raf-source`.** `Workspace::load` takes explicit roots, and each consumer owns its own default (census-ledger keeps the workspace root). The PR quotes the issue's "(default `crates/`)" and this decision.
3. **Containment race: closed on Linux, with a portable fallback elsewhere.**
   - Linux: after opening the canonical path with `O_NONBLOCK|O_NOFOLLOW` and running `fstat`, the handle's real path (`std::fs::read_link("/proc/self/fd/<fd>")`) must still lie under the canonical root. No check is separated from the open it guards.
   - Other unix: the same open flags and `fstat`, then re-canonicalise the path after the open and require both the prefix and an equal `(dev, ino)` with the handle.
   - Non-unix: a std-only open, then a regular-file check on `File::metadata()` of the handle, then post-open re-canonicalisation with the prefix re-check.
   - The Contract and the SECURITY.md Containment bullet state the guarantee for each platform. A residual rename race exists only on non-Linux.
   - Each fallback is a plain std function compiled on every platform and unit-tested on Linux too, so the fallback code is exercised.
   - `libc` is a unix-only target dependency.
   - New gate `make cross-check`: `cargo check -p raf-source --locked` for `x86_64-apple-darwin` and `x86_64-pc-windows-gnu`. A missing target is a hard failure. It goes into `check` and gets a CI job.
   - The "unix-only" statements are removed from the plan.
4. **Links split across lines are refused.**
   - `Body` records every inline link or image whose text or destination spans a line break, outside fences and code spans. `Rule::SplitLink` reports it at its line.
   - Through `Body::parse`, `docs_contract` applies the rule to every tracked Markdown file.
   - The architecture sentence becomes "A link split across lines is a violation."
   - Any existing split link in the repository is rejoined in Task 2. None was found at review time.
   - Fixtures:
     - `[text\nmore](a.md)` is refused;
     - `[a](\nb.md)` is refused;
     - a link inside a fence is not refused;
     - a one-line link and two adjacent one-line links on separate lines are both accepted.
   - Multi-line HTML `href`/`src` tag state keeps the existing checker's behaviour.
5. **Completion adversary.** The user stopped the earlier run. It runs once, on its own, on the merged plan, unless the user declines at plan approval.

## A. Cross-cutting rules (bind every task and implementation prompt)

- **Superset port.** The start-from source per concern is listed in section D. Code is ported without text from any precedent's census payload: `Unit`, `BodyPrint`, `function_unit`, `constant_unit`, `keep`, `relex`, `normalize`, `rules`/`scan_rules`, `EXCLUDED_DIRS`/`is_excluded`/`is_outside_or_generated`, `PathEscapes`, `trait_impl` as census policy, `settle`'s fallback, and every `is_test_only(..) => continue`. Task reviews grep `crates/raf-source/src` for these names.
- **No silent paths.** No ported or new line turns an error into absence, a default or a skip:
  - no `.ok()`, `filter_map(Result::ok)`, `unwrap_or*`, `let … else { continue }` or `_ => {}` over a fallible or unmodelled case;
  - only `TreeError::NotFound` may mean "absent";
  - no first-wins map (`or_insert`, `entry().or_insert_with`) and no visited-set skip.

  Specific precedent sites removed:
  - katamari `:83`, `:87-94`, `:248`, `:470`, `:584-589` (flat-first lookup, L3), `:642-647`, `:794-795`/`:811-812` (`super` above the crate root), `:989-1002` (unparseable cfg treated as "not test"), `:1231-1249`;
  - gmeow `:212-213` (`seen` skip) and `:244-279` (cfg evaluation);
  - `tests/support/mod.rs:287-288` (`cfg(not(unix))` skip).
- **No process or plan labels in committed files.** This covers code, comments, test names, fixtures, the corpus, docs and commit subjects. Banned:
  - plan IDs (`I-*`, `O-*`, `S-*`, `A-*`, `C-*`, `R-*`, `L1`-`L14`, `F-*`), `task N`, `ACn`, tracker numbers;
  - precedent project names, and words like "ported" or "regression in".

  Fixtures are named for the behaviour they pin.
- **Every rule gets a refused case plus a neighbouring accepted case.** Every refusal is a typed error.
- **Per-task gates.**
  - All cargo commands run with `--locked`.
  - Each task runs: `cargo test -p <crate> --locked`; clippy `-D warnings`; doc with `RUSTDOCFLAGS=-Dwarnings`; `cargo test --locked --test docs_contract --test repo_gates`; `cargo fmt --all --check`; `make reuse`; `make msrv`.
  - `make deny` also runs in every task that changes a lockfile (1, 4, 5, 6, 8).
  - After staging, `git diff --quiet -- Cargo.lock fuzz/Cargo.lock` must hold.
- **Docs stay current per commit.** AGENTS.md members and layout change in Tasks 1 and 4. SECURITY.md Bounds changes in Task 5. Task 9 carries only state-dependent text.
- **Commit types.**
  - Tasks 1-7: `feat(raf-okf)` / `feat(raf-source)`.
  - Task 5 gate wiring: a separate `build:`/`ci:` commit.
  - Task 8: `test(docs-contract)`.
  - Task 9: `docs(capabilities)`.
- **No `unsafe`.** The allocation test uses `cap::Cap<System>` and `max_allocated()`; there is no `unsafe_code` allow and no `cap` `nightly` feature.

## B. raf-okf design (Tasks 1-3)

**Module layout.** Modules are private; the API is `pub use` at the root.
- `limits` (`MAX_TEXT_BYTES`, `MAX_YAML_DEPTH`, `MAX_BLOCK_NESTING`), `span`, `error` (`ParseError`, `ErrorKind`, `SpliceError`)
- `frontmatter/{reader, scalar, value, writer}`
- `body/{blocks, heading, link, marker, prose}`
- `document`, `index`
- `profile/{schema, typed, body_rules, violation}`

B1. **Frontmatter subset (Task 1).**
- Start from the partial `scalar.rs`/`value.rs` and DC 136-405 with its fixtures, plus purrdf's reader for fence/CRLF/unclosed fence, the byte cap before parse, and duplicate keys at every level.
- Accepted: block mappings, block or flow sequences of scalars, a list of block maps (`items`), and a nested block map (`bindings`).
- Refused, each with a positioned error naming the key or token: flow mappings, duplicates, anchors, aliases, tags, merge keys, block scalars, and unknown escapes.
- Numbers are read as strings. Depth over 32 is refused before descent; 100 000 levels returns `Err` on a 2 MiB thread. Text over 1 MiB is refused.
- The architecture page Value rules state this exact subset, edited in Task 1.

Fixes required in the partial code:
- Replace every `unwrap_or_default()` with a char-boundary byte cursor.
- Move `MAX_DEPTH` to `limits`.
- Rename the list element `ListItem` and the profile item `ChangeItem`.
- Add `Frontmatter { map, trailing_comments }`.
- Remove `InvalidSpan` from `ErrorKind`.

B2. **Writer (Task 1).**
- One schema table, `profile/schema.rs` (`DocType -> [KeySpec{key, ValueKind}]` in architecture-page order, plus body rules), drives both `render` key order and `validate`.
- Render order: the type's keys in table order (optional `tags`/`timestamp` in their table position, `tags` sorted), then any other keys in input order. An unknown type keeps input order.
- Parsed scalars re-emit their source style.
- Reader and writer share one escape set (`\\ \" \n \r \t \0`) and one `plain_ok` predicate.
- Constructors `Scalar::new`, `List::{block, flow}`, `Map::new`, `Entry::new` and `Document::new(frontmatter, body)` choose the style with gmeow's three-way selection, but only through `plain_ok`/`ESCAPES`. A value the reader cannot read is refused by the constructor.
- Equality: keys are compared as a map, with values and attached comments, plus body bytes.
- Comments: full-line comments attach to the next key, trailing comments stay on their line, and comments after the last key stay last.

B3. **Body (Task 2).** `Body::parse(text)` handles Markdown with or without frontmatter around it, and `Document::parse` builds its body through it.
- Fences: matched by character and length, indented up to 3 spaces, recognised inside quotes and lists.
- `headings()` covers levels 1-6 outside fences, with GitHub anchors deduplicated by `-N`, and `has_anchor`.
- `sections()` returns the H2 sections: heading, slug, and a content span starting after the heading.
- `links()` returns inline links, images, reference definitions and HTML `href`/`src`, tagged by form. It takes from purrdf's CommonMark scanner (escaped `\)`, CommonMark whitespace, `[a] (b)` not a link), DC's reference and HTML reader, and DC's multi-line HTML tag state. Links are excluded from fences and code spans of any backtick length.
- `Link::target_in(doc_path)` returns `External`, `Fragment` or `Relative{path, anchor}`, and refuses an escape. It is taken from purrdf's `resolve_link_path`.
- `GeneratedBlock { kind, outer, content }`.
- `ProposedBlock { slug, span, marker, content (unquoted), section }`.
- `prose_lines()` yields `(line, text with code spans removed, section, in_proposed, in_generated)`, skipping fences and HTML comments.
- The Markdown nesting bound is 64.
- `Span { lines, bytes }` is in whole-text coordinates.

B4. **Splicing (Task 2).**
- `Body::replace_spans(&[(Span, &str)]) -> Result<(), SpliceError>` applies non-overlapping edits in one pass and rebuilds the model.
- It is transactional: on `Err` the body is unchanged.
- `SpliceError::{OutOfBody, NotCharBoundary, Overlap, Reparse(ParseError)}`.
- `Document` and `Index` delegate to it. `Index::render()` returns the text.

B5. **Index (Task 2).**
- `Index::parse(text)` accepts an SPDX header (one line or several) followed by generated blocks only. An empty header tag is refused.
- `is_capability_index(path)`.

B6. **Profile (Task 3).**
- `Profile` is a unit struct.
- `Profile::validate(&doc, path) -> Result<RafDocument, Vec<Violation>>`, where `RafDocument` is `Capability | Change | Architecture` with typed fields: `State`, `Bindings`, `Affects{capability, section}`, `ChangeItem{claim, outcome}`, `Outcome::{Implemented, Dropped, Moved(slug)}`.
- `Violation { file, at: Location::{Key, Section, Line, Document}, rule: Rule }`. `Display` is derived from `Rule`.

Rules enforced, all reported, each with a fixture:
- **Keys:** the closed key set per type, with required and optional keys from the schema table; missing required key; `resource` rejected; unknown `type` (`Rule::UnknownType`).
- **Values:**
  - id prefix, and kebab-case slug equal to the file stem;
  - non-empty `title`/`description`;
  - `state` enum; `bindings` exact keys with each value a list;
  - `depends` as cap ids;
  - `changes` non-empty, as cap ids;
  - `affects` format, section set and slug present in `changes`;
  - `contract_effect`;
  - items with a non-empty `claim`, no unknown item key, and a valid outcome;
  - optional `tags`: non-empty strings, no duplicates;
  - optional `timestamp`: RFC 3339.
- **Bindings by state:** `designed`/`considered` must be empty; `delivered` needs at least one code and one test.
- **Type vs directory** of `path`.
- **Body rules:**
  - seven sections in order;
  - a section is empty when nothing remains after HTML comments and generated blocks are removed (the architecture sentence is updated);
  - Change bodies need `## Summary`; Architecture bodies must be non-empty;
  - PROPOSED appears only in a Capability, and only inside a section;
  - `Rule::Placeholder` applies over `prose_lines`;
  - `Rule::SplitLink` (decision 4).

Negative check: future-tense prose is not a violation. A test scans `src` (including doc comments, with no exemption) for `std::fs`/`std::process`, with a negative fixture.

B7. **Usage/Examples.**
- Usage blocks are rewritten to run without file access: text comes from `include_str!` or `#` hidden lines. Blocks that need a real tree are `no_run` in both the doc and the doctest, and the doc says so.
- The okf Examples block is in block form, wrapped in fences with a seven-section body at path `docs/capabilities/example.md`.
- A docs_contract test extracts each fenced `rust` block through `raf_okf::Body` and asserts that it equals its doctest. A one-line-difference negative fixture checks that this test fails.

## C. raf-source design (Tasks 4-7)

**Module layout:**
- `path` (`RelPath`), `tree/{mod, memory, disk}` (`std::fs` only in `disk`)
- `prescan`, `parse`, `facts`, `cfg`
- `manifest`, `discovery`, `target`, `modtree`
- `index`, `resolve`, `workspace`, `error`

C1. **Paths and Tree (Task 4).**
- `RelPath` is a validated root-relative `/`-separated UTF-8 path. `join` normalises `.` and refuses `..` above the root and absolute input.
- `trait Tree` is object-safe, with no `Send` or `Sync` bound:
  - `read(&RelPath, max_bytes)` — both backends enforce the cap;
  - `list(&RelPath) -> Vec<Entry{name, kind: File|Dir|Symlink|Other}>`, sorted;
  - `entry(&RelPath) -> Result<Option<EntryKind>>`.
- `TreeError::{NotFound, OutsideRoot, NotRegular, SymlinkLoop, TooLarge, NotUtf8, NonUtf8Name, DuplicatePath, Io}` (`DuplicatePath`: two `MemoryTree` keys normalising to one path, reported from every call and by `try_from_iter`).
- `MemoryTree` implements `From<[(&str,&str);N]>` and `FromIterator`.
- `DiskTree::new(root) -> Result` canonicalises the root once. For each candidate it then:
  1. canonicalises the path;
  2. checks the canonical-root prefix;
  3. opens the canonical path with `O_NONBLOCK|O_NOFOLLOW`;
  4. runs `fstat` on the handle and refuses non-regular files;
  5. verifies containment on the opened handle, per decision 3;
  6. reads through `take(max+1)`.
- `list` never descends into a directory symlink. The walker builds candidates with `RelPath::join` before touching the tree.
- Fixtures at the tree level, with a sentinel or FIFO outside the root that is never read:
  - a symlinked file and a symlink-to-symlink;
  - a directory symlink with `..`;
  - a directory symlink loop;
  - a FIFO, which returns within a 10 s watchdog;
  - a missing root and a symlinked root.
- Further fixtures: `NotFound` vs `OutsideRoot`; `0xFF` content and a non-UTF-8 name; `MemoryTree` refusing `../a` and `/a`; size cap at 8 MiB+1, a lowered cap, and exactly 8 MiB.
- A `std::fs` scanner confines `std::fs` to `tree/disk`, with a negative fixture.

C2. **Errors.** There are three public types, all `#[non_exhaustive]`:
- `TreeError` from the tree backends;
- `LoadError { file, line, origin, kind }`, where `Origin` is one of `ModDecl{module}`, `PathAttr{module, predicate}`, `ManifestKey{key}` or `Discovery`, and `kind` is one of `Tree`, `Budget{budget, limit}`, `Syntax`, `Spawn` (the parse thread could not start), `Manifest`, `UnresolvedMod`, `BothModFiles`, `DuplicatePathAttr`, `ModuleCycle`, `InherentImplSelf`, `Target`, `UnsupportedItem`, `SuperAboveRoot`, `CfgParse` or `DirSymlink`;
- `ResolveError` from `resolve`.

C3. **Bounded parse (Task 5).**
- `pub fn parse_source<R: Send>(file: &RelPath, text: &str, visit: impl FnOnce(&syn::File) -> R + Send) -> Result<R, LoadError>`. It runs the pre-scan on the caller's thread, then `syn::parse_file` and `visit` on a scoped thread named after the file with `PARSE_STACK_BYTES = 64 MiB`. A panic is re-raised with `resume_unwind`.
- `pub use syn;`.
- The pre-scan is one lexer feeding independent counters, with `const BUDGETS` as the single source. The lexer handles strings, raw strings with any number of hashes, byte and C strings, char vs lifetime, and nested comments. Each counter's definition is written into the Contract.
- In-process fixtures on the caller's 2 MiB thread, debug and release, each returning `Budget::*` and naming the file: 500 `A<`, 4000 `!`, 20000 `&`, 10000 `||`, and a 20000-link `else if`. At-budget variants parse.
- Neighbouring valid cases: 300 `!x;` statements, `a!()`, 300 closures, 300 `if`s, `'a` vs `'a'`, and 300 comparisons. Tokens inside strings and comments do not count.
- The child-process three-outcome harness runs the test binary itself, with `RUST_MIN_STACK` removed, and only for the composites and the six uncounted shapes. Exit 0 must come with the exact symbol list.
- A deterministic overflow fixture, sized from the measured per-level cost, asserts a non-zero exit and stderr containing `thread '<path>' has overflowed its stack`. A hook asserts `thread::current().name()` during a normal parse.

C4. **Fuzz and gates (Task 5; gate wiring is its own `build:`/`ci:` commit).**
- `fuzz/` is its own cargo-fuzz workspace, excluded from the root. It has `[workspace.lints]` identical to the root, and a `repo_gates` test (with a negative fixture) asserts they are equal. It also has `[lints] workspace = true`.
- `fuzz/.gitignore` ignores `target/`, `artifacts/` and `coverage/`.
- `fuzz/deny.toml` is the root `deny.toml` plus a crate-scoped NCSA exception, and a `repo_gates` test asserts the shared sections are equal.
- `fuzz_targets/parse_source.rs` calls `parse_source`. The corpus holds the shapes and the at/over-budget files.
- A crash input that trips `repo_gates` is rewritten to an equivalent reproducer, and the gate is never exempted. Every abort found is minimised, committed and covered by a budget.
- `make fuzz` runs fmt, clippy `--locked` and `deny --offline` on the fuzz manifest, then `cargo fuzz run parse_source <mktemp dir> fuzz/corpus/parse_source -- -runs=N -seed=S`. A missing `cargo-fuzz` is a hard failure.
- `make msrv` also checks the fuzz manifest, and `make fmt` covers it.
- `make test-release` runs `cargo test -p raf-source --release --locked`.
- Both targets go into `check`, and CI gets jobs with a pinned `cargo-fuzz`.
- Every gate list is updated: the Makefile header, AGENTS.md section 3 and its missing-tool sentence, CONTRIBUTING.md, the PR template and the CHANGELOG Gates entry.
- Root `deny.toml` gets a crate-scoped `unicode-ident` `Unicode-3.0` exception, with a system-terms reason.
- SECURITY.md Bounds is updated.
- Corpus REUSE annotations are added.
- A replay test runs every corpus file in a child process.

C5. **Discovery and targets (Task 6).**
- `toml` is used with `default-features = false`, an explicit feature list, and never `unbounded`. Manifests are read through `Tree` under the cap.
- Discovery recurses through non-package directories. It does not descend below a `[package]` manifest that lacks `[workspace]`.
- It skips `target` and dot-directories; the Contract names both.
- A directory symlink is a `DirSymlink` error. Two packages with one crate name are an error naming both manifests.
- `Target { package (Cargo name), name (Cargo target name), kind, crate_path, root, shipping }`:
  - the lib crate path is `[lib] name`, or the package name with `_`;
  - bins are `<pkg>::bin_crate::<bin>`; tests, benches and examples are `<pkg>::{test,bench,example}_crate::<target>`; the build script is `<pkg>::build_script`;
  - `shipping` is derived from `kind`.
- `auto*` flags are supported.
- These are typed errors naming the manifest key: a missing explicit target file (checked with `entry`), a target without a name, two targets sharing one root, and a target `path` that escapes.

C6. **Module tree and facts (Task 6).**
- Per file, the `parse_source` visitor extracts an owned `FileFacts`: items with name line, span, body span, attributes, doc presence and test role; `mod` declarations with raw `path`/`cfg_attr`; `use` trees; impl headers with their written self type; and `#[macro_export]`.
- `Tree` reads and recursion run on the caller's thread.
- rustc directory rules:
  - inline-module components and `#[path]` on inline modules;
  - a `#[path]` file is mod-rs for its children;
  - both `name.rs` and `name/mod.rs` existing is an error naming both;
  - raw identifiers.
- Unconditional duplicate `path` attributes are an error.
- `cfg_attr(pred, path=…)` alternatives are each resolved, contained and followed as further sites, carrying the predicate. `cfg_attr(test, path=…)` is tagged test. No cfg is evaluated other than test, and no item is dropped.
- A cycle is detected on the active-file stack, never with a visited set. A file reached by several module paths is parsed once and indexed under every path.
- Test roles: `None`, `CfgTest` or `TestFn`.
  - The test predicate matrix is: `all(test, x)` → test, `any(test, test)` → test, `any(test, unix)` → not test, `not(test)` → not test.
  - An unparseable cfg is an error.
- An unresolved `mod` is an error giving file and line.
- `Workspace::load` delegates to `load_with(&LoadOptions::default())`. The walk stops at the first `LoadError`.

C7. **Index and resolution (Task 7).**
- `Item { id, path, kind, namespace, file, name_line, span, body, visibility, documented, doc_hidden, test, target }`.
  - `documented`: non-blank `#[doc]` text; `doc(hidden)` alone does not count.
  - Kinds include fn, struct, enum, union, trait, type alias, const, static, mod, macro, `extern crate`, foreign items, inherent method, trait-impl method, trait default method and assoc const.
  - `Item::Verbatim` is an error.
- `(file, span.bytes.start)` is the identity.
- Accessors:
  - `Workspace::source(file) -> &str` returns the exact indexed text;
  - `files()` yields each `SourceFile { path, package, reached: [(TargetId, module path)], shipping, test }` once;
  - `items_in(file)`, `item_at(file, offset)`, `items()` (sorted by path, so a prefix is a range), `item(id)`, `packages()`, `targets()`, `is_shipping(&Item)`.
- Alias table:
  - covers renames, groups, `self`, globs, `use ::x` and `extern crate x as y`;
  - `crate`, `self` and `super` are handled; `super` above the root is an error.
- `resolve(path) -> Result<Resolution, ResolveError>`, with `Resolution { path, namespace, sites (non-empty, sorted), via_glob, chain }`.
  - Segments resolve with rustc precedence: definitions and explicit uses shadow globs.
  - The search runs to completion with no cap.
  - Same path, namespace and kind gives one symbol with several sites (cfg twins).
  - `Ambiguous(Vec<ItemId>)` only for real conflicts (glob vs glob, or two canonical targets). `Cyclic` and `NotFound` complete the set.
  - Raw identifiers are normalised per segment.
- Impl rules:
  - An inherent impl self type must resolve (through aliases) to a local type; otherwise `InherentImplSelf` naming file and line.
  - A trait impl on a local path records under that type.
  - A trait impl on any other self type records under `<trait>::<method>` as `TraitImplMethod`, with the written self type as text.

## D. Superset source map (start from → take from → fix)

| Concern | Start from | Take from | Fix |
|---|---|---|---|
| YAML reader | partial `scalar.rs`/`value.rs`, DC 136-405 + fixtures | PURO 284-345, 305-313, 885-938 | cursor, one escape set, refusals per B1 |
| YAML writer | partial `scalar.rs` writer | GMO style choice (constructors only) | shared `plain_ok`/`ESCAPES`; parsed style kept |
| Fences, headings, anchors | DC 411-485, `anchors` 448-467 | — | char/length match, ATX indent ≤ 3, all levels |
| Links | PURO scanner 567-760, `resolve_link_path` 767-800 | DC 1123-1291 incl. multi-line tag state | fence and code-span aware |
| Markers, index, per-doc checks, placeholders | DC 993-1069, 1572-1635, 593-990, 1094-1109 | — | spans, typed rules |
| Tree, Memory | KAT 67-132 (= PUR) | — | `Result`, `entry`, sorted `list`, cap in both |
| Disk handle check | `tests/support/mod.rs` `check_handle` | — | no stat before open, non-blocking, cap, no non-unix skip |
| Containment | GMB `checked_path` (refusal) | KAT `PathEscapes` messages | canonical, in-root symlinks allowed |
| Discovery, targets | KAT `walk_dirs` 237-262, targets 1157-1258, `lib_name` 1088/1165 | PUR raw-ident tests | `toml`; `bin_crate`; typed target errors |
| Mod tree | GMB 181-310, 618-641 (minus cfg evaluation and `seen`) | KAT 563-592 children of a `#[path]` file | E0761; `#[path]` mod-rs; `cfg_attr` alternatives |
| cfg(test) | KAT `is_test_only` 979-1013 + tests | — | tag, never drop; unparseable cfg is an error |
| Items, docs | KAT 388-473, 650-679, `documented` 663 | KAT 521 trait-impl kind | spans, namespaces, `doc_hidden`, foreign items |
| Uses, resolve | KAT 594-648, 785-859 | — | precedence, no cap, typed errors, `Resolution` |
| Fixtures | KAT 1278-1437, DC ~90, PURO link fixtures, GMB rustc-rule tests | — | positive beside every negative |

## Completeness contract

Where a row below and sections A-D differ, sections A-D and the user decisions govern.

Every requirement maps to a task and an executable check on the shipped entry point.

| Requirement (issue / capability doc) | Task | Acceptance check |
|---|---|---|
| Parse Markdown + YAML frontmatter, positioned errors: no fence on line 1, bad YAML, non-mapping (I-2, O-2) | 1 | `Document::parse` fixtures each `Err` with line; repo documents parse `Ok` |
| SPDX comments preserved; comments kept (O-3, O-16) | 1 | `render(parse(x)) == x` over every repo document, which carry SPDX comment lines |
| Bounds: 1 MiB text, YAML depth 32 checked before descent, anchors/aliases/merge keys refused, Markdown block nesting 64 (O-4..O-7) | 1 (YAML), 2 (Markdown) | 1 MiB+1 fails, 1 MiB parses; 33 levels fails, 32 parses, 100 000 levels returns `Err` on a 2 MiB thread; `&x`/`*x`/`<<` each fail naming the anchor; billion-laughs fails and the counting allocator shows peak <= 8 x input; 65 nested quotes / list levels fail, 64 parse |
| Deterministic writer, round trip, profile-table key order, `Body::replace_spans` (I-5, O-15, O-16, O-24) | 1 (render), 2 (splice) | goldens byte-identical; out-of-order input renders to profile order and re-renders identically; `parse(render(d)) == d` under the stated equality (keys compared as a map with their values and attached comments, plus body bytes); `Body::replace_spans` with two generated blocks changes only those byte ranges; a second call uses spans read from the rebuilt model; an overlapping pair returns `SpliceError::Overlap` and the body is unchanged |
| Subset refusals (A-4, Examples) | 1 | a flow mapping value `bindings: {code: []}` fails at its line naming the key, and the neighbouring block-form fixture parses |
| Body model: H2 sections (heading, slug, line + byte span), links (target, anchor, line, form), fences, generated blocks (kind, span), PROPOSED blocks (slug, span) (I-6..I-10, O-13, C-18) | 2 | accessor fixtures; markers only inside a fence produce no blocks; four-backtick fence holding three backticks stays open (L11); `v[0](x)` in a fence is no link (L12) |
| Body model for Markdown without frontmatter (okf-profile Purpose: the repository tooling goes through raf-okf) | 2, 8 | `Body::parse` on README-shaped text with `[a](b.md)` inside a fence and one outside yields exactly the outside link; a PROPOSED marker in such text is still a block; `Document::parse` builds its body through `Body::parse` |
| Malformed PROPOSED, unclosed / nested generated, orphan end -> parse errors with lines (O-14, A-7, A-8) | 2 | one fixture each, exact line asserted; each misspelling of the marker fails |
| `index.md` navigation (I-11, O-17, A-9) | 2 | `Index::parse` accepts a generated-only index, rejects prose, zero blocks, indented and mid-line markers |
| Profiles: closed key set per type, typed values, every violation reported with file/key/rule; `resource` and unknown `type` rejected (I-3, I-4, O-8..O-12, O-19, O-21, O-22, A-1..A-6) | 3 | per type: each missing key, an extra key, `resource` -> violation with that key and `Rule`; value fixtures (state, bindings keys, affects section, items outcome incl. `moved:change:<slug>`, id prefix and slug vs file name, kebab case, scalar grammar); a document with three faults returns three violations; the Examples block of `okf-profile.md`, rewritten to block-form `bindings`, validates clean as exact text and with a `resource:` line added names `resource` and the seven Capability keys; `depends` on a missing id is not a raf-okf violation |
| Base OKF optional keys `tags` and `timestamp` (issue scope, user decision 1) | 1 (render), 3 (validate, docs) | `Document::render` of a Capability with `tags: [b, a]` emits `tags: [a, b]` and re-renders byte-identically; `Profile::validate` returns a violation naming `tags` for `tags: [a, a]` and for `tags: [""]`, a violation naming `timestamp` for `timestamp: 2026-13-01T00:00:00Z`, and none for `timestamp: 2026-01-01T00:00:00Z`; a Capability with neither key validates clean; `resource:` still fails |
| Body rules per type: seven sections in order, none empty after comments; Change `## Summary`; Architecture non-empty; PROPOSED only in Capability (A-5..A-7) | 3 | fixtures per rule with neighbouring valid case |
| raf-okf touches no file and runs no process (O-20) | 3 | test scanning `crates/raf-okf/src` for `std::fs`/`std::process` fails on a hit; negative fixture of the scanner |
| `Tree` with disk and memory implementations; list, read, exists (I-12, S-1, S-2) | 4 | `MemoryTree` and `DiskTree` fixtures; a test scanning `crates/raf-source/src` finds `std::fs` only in the disk module |
| Canonical containment, `DiskTree::new` canonicalises once, symlinks leaving the root refused, no descent into directory symlinks, loop ends at first link, non-regular refused via non-blocking open + `fstat` (S-3..S-7, S-31) | 4 | `DiskTree` fixtures in a temp dir beside a sentinel: symlinked module file, `vendored/..` dir symlink, symlink to symlink, dir-symlink loop, FIFO (returns within a 10 s watchdog), each `Err` and sentinel unread |
| `max_file_bytes` default 8 MiB, `LoadOptions`, error names file; Cargo.toml same cap (S-8, S-13) | 4 (cap in `read`), 6 (manifest) | 8 MiB+1 fails naming the file, lowered cap honoured, 8 MiB parses; a `Cargo.toml` with inline tables nested 1000 levels (under the cap) makes `Workspace::load` return a typed manifest error naming the file on the caller's 2 MiB stack, and an 8-level manifest loads |
| Token pre-scan budgets (128 delimiters, 64 `<`, 256 prefix ops, 128 closure bars, 256 `else if`), strings/comments skipped, typed error naming file and budget (S-9) | 5 | at-limit parses, limit+1 fails with `Budget::*`; same tokens inside strings, raw strings and nested block comments do not count; 300 comparisons in one function parse |
| Dedicated 64 MiB parse thread, stated constant, named after the file (S-10, S-28) | 5 | `PARSE_STACK_BYTES == 64 MiB`; thread-name assertion; abort message of an overflow child contains the file name |
| Overflow fixtures in debug and release on the caller's default stack; composite and uncounted shapes in a child-process three-outcome harness (S-12, S-26, S-27) | 5 | `cargo test -p raf-source` and `cargo test -p raf-source --release` both run the listed shapes (500 `A<`, 4000 `!`, 20000 `&`, 10000 `||`, 20000 `else if`, the composites and the six 20000-repetition shapes); each ends in exactly one allowed outcome, exit 0 only with the exact expected symbol list |
| Committed fuzz target `parse_source` and corpus; corpus replay under `cargo test`; bounded fuzz run (S-11, S-29, S-30) | 5 | `fuzz/fuzz_targets/parse_source.rs` exists; `make fuzz` exits 0; the replay test asserts `fuzz/corpus/parse_source/` holds a named file for each of the 5 single shapes, 5 composites and 6 uncounted shapes and an at-budget and over-budget file for each of the 5 budgets (fails on a missing name), runs every corpus file in a child in debug and release, and asserts each ends in exactly one outcome: typed refusal naming file and budget, exit 0 with the index equal to the recorded expected symbol list, or non-zero abort; negative fixture: a recorded list differing from the real index fails the replay test |
| Package discovery under configurable roots: lib, bin (incl. `src/bin/*`), build script, test, bench, example with default and explicit paths; shipping flags; `autotests = false` etc. (I-13, I-14, I-17, S-14, S-32, C-7..C-9) | 6 | `MemoryTree` fixture per target kind x (default, explicit path) asserting kind, Cargo target name, `shipping` and `crate_path` (lib `demo`, binary `demo::bin_crate::x`, test `demo::test_crate::it`, bench `demo::bench_crate::b`, example `demo::example_crate::e`, build script `demo::build_script`), and an item resolves under each (e.g. `ws.resolve("demo::test_crate::it::case")` has a site in `tests/it.rs` whose target has `shipping == false`); virtual manifest contributes nothing; manifest with a trailing comment and single-quoted path honoured (L6); explicit target path outside root fails naming the key on both trees |
| Mod-tree following as rustc does, `#[path]`, inline modules, unresolved `mod` error with file and line, path escape rejected without reading (I-15, I-16, I-20, S-15..S-18, C-10) | 6 | fixtures for `name.rs`, `name/mod.rs`, both present (error naming both), inline, nested, `#[path]` in mod-rs, non-mod-rs, inside inline modules in each, on an inline module, via `cfg_attr`, two `path` attributes, raw identifier, `mod missing;` -> `src/lib.rs:3`; `#[path="../../x.rs"]` and absolute path fail naming module and attribute, with a recording `Tree` showing zero reads of the outside path |
| `cfg(test)`/`#[test]` tagged, not dropped, incl. modules reached only through them (I-16, S-19) | 6 | `#[cfg(test)] mod t;` -> every item in `src/t.rs` tagged; `#[test] fn` tagged; `any(test, unix)` and `not(test)` untagged; each also resolves positively |
| Symbol index of every named item with path, kind, file, line, visibility, test tag; methods under the impl type path (I-18, S-20) | 7 | one fixture per kind (fn, struct, enum, union, trait, type alias, const, static, mod, `macro_rules!` incl. `#[macro_export]` at crate root), method in another file than its type |
| Use-alias table: renames, nested groups, globs, re-export chains; typed NotFound / Ambiguous / Cyclic; glob reported; macro-generated and dependency items not found, and the consumer documents (census-ledger, workplan-document-model) say so (I-18, I-21, S-21, S-22, S-24, C-14) | 7, 9 | rename; nested group; glob with `via_glob`; three-hop chain; cycle -> `Cyclic`; two globs -> `Ambiguous`; `#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()` -> `Ok(Resolution)` with exactly two sorted `sites` and `via_glob == false`; `demo::nope` and `pub use serde::X` -> `NotFound`; `macro_rules!`-generated fn -> `NotFound`; Task 9: the docs-contract binding test refuses a binding to a `macro_rules!`-generated function with `NotFound`; impl self type that does not resolve is an error, not a guess |
| Determinism, sorted output, `DiskTree` equals `MemoryTree` for regular files (S-25, I-12) | 7 | two loads equal; same tree on disk and in memory equal |
| Usage and Examples compile (O-18, S-33, S-34) | 3, 7 | crate-level doctests equal to the documents' Usage and Examples blocks |
| Public APIs documented, `missing_docs` deny (I-23) | every crate task | `cargo clippy -p <crate> --all-targets -- -D warnings` and `RUSTDOCFLAGS=-Dwarnings cargo doc -p <crate> --no-deps`; a `repo_gates` test asserts every `[workspace] members` manifest carries `[lints] workspace = true`, with a negative fixture of a member manifest without it |
| raf-okf is the one reader of RAF documents (okf-profile Purpose) | 8 | `tests/docs_contract.rs` parses and validates through `raf_okf`; its private YAML, fence and marker parsers are deleted; its whole existing fixture suite passes unchanged in intent |
| Bindings never name a missing symbol (architecture value rule) | 8 | new docs-contract test loads this workspace with `DiskTree` and resolves every binding of every capability; a negative fixture with a bogus binding fails |
| Capability docs match the build; state `delivered`; INVENTORY; change records completed (I-1, I-24) | 9 | `cargo test --test docs_contract` green with both rows `delivered`, bindings non-empty, records deleted after a commit giving every item `outcome: implemented`; the Task 9 checklist runs both ways with no unmatched entry (every Contract, Constraints and Verification sentence maps to a test whose assertion matches its wording; every completeness row maps to the Contract sentence stating its behaviour); AGENTS.md 'Every crate below is **designed, not yet implemented**' and '(none exist yet; `members` is empty)', README.md 'designed — not yet published' for these crates, the architecture page 'binding waits for the source index' and 'will enforce the whole profile once they are implemented', and SECURITY.md 'Once implemented' for the two surfaces are each replaced, and the Task 9 review greps that none remain |
| Handle containment per platform (source-index Contract 'No check separates from the open it guards', user decision 3) | 4 (code, fixtures, Contract, SECURITY.md Containment bullet), 5 gate commit (`make cross-check`) | on Linux: a seam in `tree/disk` swaps an intermediate directory for a symlink to outside the root between the prefix check and the open, and `DiskTree::read` returns `OutsideRoot` with the sentinel unread; the Linux verifier given a handle on an outside file returns `OutsideRoot` and an in-root neighbour `Ok`; the other-unix verifier given a re-canonicalised `(dev, ino)` differing from the handle's returns `OutsideRoot` and an equal pair `Ok`; the non-unix verifier refuses a non-regular handle with `NotRegular` and an outside re-canonicalised path with `OutsideRoot`, each beside an accepted case; `make cross-check` exits 0 for `x86_64-apple-darwin` and `x86_64-pc-windows-gnu` and non-zero naming a missing target, is in `check` with a CI job, and is named in the Makefile header, AGENTS.md section 3, CONTRIBUTING.md, the PR template and the CHANGELOG Gates entry |
| Split links refused (user decision 4) | 2 (`Body` records, architecture sentence, rejoin), 3 (`Rule::SplitLink`), 8 (docs_contract) | the four decision-4 fixtures through `Body::parse` and `Profile::validate`; docs_contract fails on a frontmatter-less tracked-file fixture holding `[a](\nb.md)` naming its line and passes on the one-line neighbour; the architecture 'Files and repository rules' sentence reads 'A link split across lines is a violation.' |
| Full qualification | 10 | one `make check` (incl. `test-release`, `fuzz` and `cross-check`) |

## F. Review dispositions

- **Completeness (29):**
  - adopted: 1-3 (B6), 4 (B3), 5 (B3 `prose_lines`, B6), 6 (B3, Task 2), 7 (Task 1), 8 (B4), 9 (C1 tree level, Task 6 load level), 10 (C1), 11 (Task 6), 12 (C5), 13 (C6), 14 (C7), 15 (C7 impl rules), 16-17 (C3), 18 (B7), 19 (C6), 20 (Task 3 Contract wording, Task 9 checklist), 21 (B6), 22 (Task 1), 23 (B5), 24 (Task 7), 25 (C4), 26 (D, the L-fixtures in Tasks 1/2/6/7), 28 (Task 1), 29 (C6, C7);
  - 27 becomes decisions 1-2.
- **Constraints (21):**
  - adopted: 1-2 (A, C6), 3 (C5), 4-5 (C4), 6 (A), 7 (B7), 8 (A), 9 (Task 1 Value rules; placeholder rule moved into B6), 10 (fixtures listed in E), 11 (Task 8), 12 (A), 14-16 (C4), 17 (C4), 18 (B2), 19 (A), 20 (Task 1), 21 (C5);
  - 13 becomes decision 3.
- **SOLID/ETHOS (25):**
  - adopted: 1-2 (C3, C6), 3 (C5), 4 (C7), 5 (C6, cfg_attr alternatives; this replaces constraints-1's "two cfg_attr paths are an error", which would refuse legal code), 6 (C6, C7), 7 (C7), 8 (C7), 9 (C1), 10 (C2), 11 (B2), 12 (B2), 13 (B4), 14 (A), 15 (layouts in B and C), 16 (B1), 17 (C5), 18 (C3), 19 (B6), 20 (C5), 21 (C7), 22 (C6), 23 (A), 24 (B3), 25 (B6, C6).
- **Enhancement (23):**
  - adopted: 1 (C3), 2-4 (C7), 5 (C5, C7), 6 (C7), 7 (C7), 8 (C7), 9 (C5), 10-12 (B3), 13 (B4), 14 (B2), 15 (B6), 16 (B6), 17 (B3), 18 (C7);
  - declined, with reasons:
    - 19: a shared Span crate — no consumer mixes Markdown and Rust spans, and matching shapes give the same consistency;
    - 20: following `include!` — the Constraints forbid macro expansion;
    - 21: reverse references and effective visibility — that would pre-decide a `considered` capability's open question;
    - 23: `DiskTree` for docs_contract — the tracked-path and symlink rule differs from `DiskTree`'s;
  - 22 becomes decision 4.

## Completion adversary (pass 1)

`reviews/completion-adversary.md`: FINDINGS-OPEN, F-1..F-8, no user decision required. All eight integrated: F-1 tags/timestamp row and Task 3 docs; F-2 handle-containment row, Task 4/5 text; F-3 split-links row, Task 2 docs; F-4 rows for cfg twins and splice corrected; F-5 corpus replay row; F-6 crate_path row; F-7 doc ownership in Tasks 2, 6, 7 and the two-way checklist row; F-8 lints-inheritance gate.

## Verification

- Before implementation:
  - every review finding has the disposition above;
  - the stage `plan.md` is regenerated from this file, and the decisions are recorded;
  - per decision 5, the completion adversary runs on it;
  - the plan is re-posted to issue #2.
- Per task: the focused gates in section A, then an independent task review (`tasks/TN-review.md`). The review confirms the section D sources were used, that no census payload or silent path was ported (grep), and that the named fixtures exist and fail when their rule is removed.
- Final: one `make check`, which includes `test-release`, `fuzz`, `cross-check` and `msrv` on both manifests. Also a completeness review of the code against this plan and the issue, plus the Task 9 two-way checklist in `validation.md`.


## Task 1: raf-okf frontmatter and writer

(B1, B2).

Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
- Fixtures cover every subset refusal, plus duplicates (top-level, under `bindings`, inside an item), CRLF and a missing final newline.
- Goldens are every tracked `docs/capabilities/*.md` except `index.md`, plus `docs/architecture/*.md` and `work/changes/*.md`, taken from `git ls-files`, with a count assertion.
- Out-of-order keys, comment attachment, a constructed Change document, and the escape round trip.
- Billion laughs under `cap`, with peak memory ≤ 8× the input.
- Bound errors are positioned.
- AGENTS.md members and layout are updated, as are the architecture Value rules.
## Task 2: raf-okf body, splice, index

(B3-B5). Fixtures per accessor and form:

Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
- slug and span text;
- 3- vs 4-space headings;
- the purrdf link fixtures;
- the four multi-line HTML tag fixtures;
- code-span links;
- a link split across lines;
- `replace_spans` with two blocks, the index catalogue, overlap refused, and malformed output refused while the document stays unchanged;
- each marker error with its line;
- the index cases;
- nesting at 64 and 65.
Docs: okf-profile Contract "Body model" (line and byte spans, `Body::parse` for Markdown without frontmatter, link forms, split links refused) and "Index handling" (`Index::parse`, `is_capability_index`); the architecture sentence 'A link split across lines is a violation.'; any split link in the repository rejoined.

Parity decisions carried from the Task 2 review (each decided deliberately, with a fixture, in the task named):
- Task 3: `Rule::Placeholder` also scans frontmatter scalar values (title, description, claims), since `prose_lines` covers the body only.
- Task 8: links and placeholder tokens inside HTML comments are not read (the body model treats comments as non-prose); docs_contract follows the body model and the change is stated in the architecture page.
- Task 8: `[a]()` yields `LinkError::Empty` from `Link::target_in`; docs_contract reports it as a violation (the old checker skipped it).
- Task 8: marker errors stop at the first one per document; docs_contract fixtures that expect several marker errors in one document are split per document.

## Task 3: raf-okf profile

(B6, B7). One fixture per rule with its neighbouring valid case; the Usage/Examples doc edits; the Contract "Validate" wording.

Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
Docs (user decision 1): first add `cap:okf-profile#explanation` to `affects` of `work/changes/okf-profile.md`; then rewrite the architecture 'Document types and keys' table, its heading and the 'no more and no fewer' bullet for optional `tags`/`timestamp`; the okf-profile Contract 'Validate' and Examples; and the okf-profile Explanation sentences 'every key is required ... rather than an optional one'.

## Task 4: raf-source tree and containment

(C1, C2). The source-index Contract "Tree"/"Path containment" (per decision 3), Usage `DiskTree::new(".")?`, and AGENTS.md layout are updated.

Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
Includes the per-platform handle verifiers and their seam fixtures (completeness row 'Handle containment per platform') and the SECURITY.md Containment bullet stating the per-platform guarantee.

## Task 5: raf-source bounded parse, harness, fuzz, gates

(C3, C4).

Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
The gate commit also adds `make cross-check` (targets `x86_64-apple-darwin`, `x86_64-pc-windows-gnu`; missing target is a hard failure) to `check` and CI, and to every gate list.

## Task 6: raf-source discovery, targets, module tree

(C5, C6). These fixtures run through `Workspace::load` with `DiskTree`:

Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
- every Verification containment case, plus a symlinked `Cargo.toml`, with an error naming the module, attribute or key and the sentinel unread;
- a valid in-root symlink.

Also:
- manifests at 8 MiB+1 and exactly 8 MiB;
- 1000-level inline tables;
- a nested fixture package;
- auto flags;
- lib/bin same name;
- `test_crate` collision;
- a shared `tests/common/mod.rs`;
- L1, L2, L3, L14 module fixtures;
- cycle;
- `cfg_attr` alternatives;
- raw identifiers via both spellings.
Docs: source-index "Package discovery" (explicit roots; recursion and stop rule; skipped `target` and dot directories; build-script kind and shipping flag; `<pkg>::{bin,test,bench,example}_crate::<target>` and `<pkg>::build_script` paths) and "Mod-tree following" (rustc's rule replacing 'relative to the directory the module would otherwise use'; a `#[path]` file is mod-rs for its children; `name.rs` beside `name/mod.rs` is an error; `cfg_attr` path alternatives; a module cycle is an error).

## Task 7: raf-source index and resolution

(C7). Fixtures:

Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
- per kind, asserting kind, file, line and visibility;
- per impl shape;
- namespaces;
- cfg twins as a multi-site symbol;
- explicit-vs-glob shadowing, glob-vs-glob ambiguity, and a mutual `super::*`;
- a 5000-hop chain;
- `crate`/`self`/`super` paths, `extern crate as`, `use ::x`;
- a macro-generated symbol and a dependency item, each `NotFound`, beside a positive resolution;
- documented, `doc_hidden`, blank `///`;
- `files()`, `items_in()` and `item_at()`;
- the `--test`/`--bin` argv data;
- two loads equal, with strictly increasing order;
- `DiskTree` vs `MemoryTree` equality.
Docs: source-index "Use-alias index" (the `Resolution` fields; cfg twins as one multi-site symbol; `Ambiguous` only for glob against glob or two canonical targets) and "Symbol index" (item fields, namespaces, identity).

## Task 8: docs_contract through raf-okf and raf-source.


Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
- The root takes path dev-dependencies on both crates.
- docs_contract uses `Document::parse`, `Body::parse` (every frontmatter-less tracked Markdown file), `Profile::validate`, `Index::parse`, `headings`/`has_anchor`, `Link::target_in` and `prose_lines`.
- It keeps only these: cross-document checks, link resolution against tracked files, tracked-symlink refusal, directory layout, inventory, and the banned-wording scan.
- Every existing fixture is kept or moved, none dropped.
- Binding test: load `DiskTree::new(support::gate_root())?` over `["crates"]`, resolve every binding, and include a negative fixture (a bogus binding and a macro-generated function).
- The doctest-equality test from B7.
- The "standard library only" comments are updated.
## Task 9: Capabilities delivered.


Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
- Commit 1 sets `outcome: implemented` on every item.
- Commit 2 does the rest:
  - state `delivered` with real bindings;
  - INVENTORY and the shape-test counts;
  - delete both change records;
  - workplan-document-model gains "A symbol that a macro generates is not in the index, so it cannot be bound.";
  - census-rule-api gains "A rule runs on the parse thread, so `Rule: Sync`.";
  - present-tense README, architecture page, SECURITY.md and CHANGELOG;
  - the per-platform containment statement in AGENTS.md section 2.
- A two-way checklist is recorded in `validation.md`: every Contract, Constraints and Verification sentence maps to its executing test, and every completeness row maps to the Contract sentence stating its behaviour.
## Task 10: Qualification and PR.


Each task: implement -> focused gates (section A) -> independent task review -> commit -> push -> issue update.
 - One background run of `make check`.
 - A completeness review of the implementation, then the artifact inventory.
 - The PR quotes the governing lines for the `unicode-ident` exception and for decisions 1-4.
 - The plan is posted to the PR, along with the "least confident" note.

## 2. paudley — 

**Handoff: what I am least confident about, and what you should know**

**Least confident**
1. **Hostile Rust input can still abort.** The pre-scan bounds the five recursion shapes the spec names, and both evasion paths the review found are now refused. But composite shapes, and repetitions that no budget counts (`return return …`, `&mut &mut …`, `a = a = …`), still abort a run by overflowing the 64 MiB parse stack. That is allowed, and an abort always fails rather than passing. Still, any consumer that loads untrusted source will see process aborts, not typed errors, for those inputs.
2. **The non-Linux containment code is untested at runtime.** It compiles for macOS and Windows (`make cross-check`) and its checker functions are unit-tested on Linux, but nothing runs on those platforms. The rename race remains open there by design.
3. **The resolver was rewritten more than once under review.** Review found order-dependent glob shadowing, trait keys that were incomplete or unverified, and target crate roots that did not resolve. It now passes a test that resolves one fixture in 1,440 declaration orders. It is still the newest and most intricate code in the change, and its correctness rests on fixtures, not on a check against rustc.

**What you should know**
- **The licence exception needs your confirmation.** `deny.toml` now has a crate-scoped exception allowing Unicode-3.0 for `unicode-ident`. `syn` cannot be used without it. `deny.toml` says an addition needs "someone to look at the new crate and decide", and your merge is that look. `fuzz/deny.toml` likewise allows NCSA for `libfuzzer-sys`.
- **Your four decisions, and one consequence.** The PR quotes your four decisions (optional `tags`/`timestamp`, no default roots, per-platform containment, split links refused) against the issue text. Under the no-default-roots decision, an empty root list loads an empty workspace without error.
- **Work beyond the issue text.** Canonical trait keys, Cargo-accurate workspace membership (`members`/`exclude` globs, implied path-dependency members), and crate-root module items all go further than the issue asks. Without them the delivered bindings, and real workspaces, would not resolve.
- **`make check` needs more tools now.** It requires `cargo-fuzz` and the `x86_64-apple-darwin` and `x86_64-pc-windows-gnu` rustup targets. A missing one fails the gate.
- **`make deny` was too narrow before.** It ran without `--workspace`, so it checked only the root package. It now covers every member.
- **Plan reviews.** All five plan reviews (completeness, constraints, SOLID/ETHOS, enhancement, completion adversary) ran separately, and every finding was integrated. Only the completion adversary re-reviewed the merged plan, and it passed. The other four did not re-review it.
- **Two `stagectl` automation gaps, worked around:**
  - `tasks` reads the first `VERDICT:` line of a review file, so a review that blocked and later passed read as NEEDS-FIX. I added current-status headers to those review files.
  - `pr-create` failed to parse gh's JSON even though the PR (#13) was created.

**Not done here:** the Stage 2 (gap analysis) and Stage 3 (merge) gates have not run. CI on the PR has not been observed yet.

## 3. paudley — 

**CI status: not run (account billing).** GitHub did not start any job on this PR. Every check-run carries the annotation "The job was not started because recent account payments have failed or your spending limit needs to be increased. Please check the 'Billing & plans' section in your settings". The latest CI run on `main` also failed. This is an account-level block, unrelated to the code. Hosted CI has therefore produced no evidence for this PR.

The local qualification is the evidence: `make check` on 6c744af exits 0, with all twelve gates and 963 tests, which are the same gates CI runs. After billing is restored, re-run the workflow (`gh run rerun 38280511329 --repo Blackcat-Informatics/rust-agentic-foundations`) before merging.

## 4. paudley — 

# Stage 2 remediation plan (full review path)

The Stage 2 review found the delivered crates incomplete against their own contracts. Inputs were the gap analysis, an independent completion audit, and the security, architecture, quality and performance reviews. The audit ran `Workspace::load` over all 1,177 crates in the local registry.

**Headline defects**
- **Wrong module paths:** in 164 crates, including this repository, a module whose file sorts before its parent's file gets the wrong path. The binding gate then accepts the non-existent `raf_okf::body::FenceBlock`.
- **Self-resolving re-exports:** `pub use <dependency>;` resolves against itself. `raf_source::syn::File` comes back `Cyclic`, and a trait impl that names a type through the alias fails the whole load.
- **Process aborts on known inputs:** 10 of the 11 committed hostile shapes abort the process, and the contract was reworded to exempt them.
- **Listing containment:** listings go through the verified handle only on Linux.
- **Unbounded memory:** in raf-okf (a heading slug copied onto every line), in symlink-alias walks, and in glob tables.
- **Valid code refused:** loads fail on macro-generated impls, enum-variant globs, macros in trait bodies and shared `[lib]` names.

**Owner decisions recorded (verbatim in the stage validation index)**
- A1: per-item `Unresolved` marker for impls of unseen types.
- A2: crates keyed by package, plus a `resolve_in_package` query.
- G9: an empty root list is a typed `NoRoots` error.
- Sign-offs confirmed: Unicode-3.0 for `unicode-ident`, NCSA for `libfuzzer-sys`, `<pkg>::bin_crate::<bin>`, and `Rule: Sync`.

**Plan review:** NEEDS-REVISION, then revisions 2, 3, 3.1, 3.2 and 3.3, then PASS. The main thread of the review was making each resource bound true against worst-case input, not just typical input.

**Hosted CI:** blocked by account billing. It is reported separately from local validation as an external blocker.

---

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
  - Path segment validation is linear, and `shadowed_namespace` uses a per-target index (P-F9).
  - `resolve` returns `Invalid` for `crate`, `self`, `super` or `Self` in any position of a canonical path, and the swallowed `Err(Fail::SuperAboveRoot) => {}` arm (`resolve.rs:387`) is removed (A12).
  - A13 / A6-adv: a glob of a missing local module gives a typed `ResolveError` on any lookup that would consult it. The **load still succeeds**. Contract updated (case 3).
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
  - The `parse-source` `INITED` line shows the full corpus (about 1.8 MB).
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

## 5. paudley — 

**R1 fixed: raf-okf body memory is bounded** (commit c10abd9, pushed)

- **Gap:** S-1 / P-F1. Every line, prose line and PROPOSED block cloned its section's heading slug, so memory grew with lines × heading length. A 200 KB document exhausted the allocator, which breaks the okf-profile Contract's "no unbounded allocation".
- **Fix:** each section has one shared slug allocation (`Arc<str>`), used by the heading, the section, the anchor table and every line and block under it. Public accessors are unchanged and still return `&str`.
  - The plan named a section index. `Arc<str>` was chosen because it keeps the public signatures and the memory is equally linear. The reviewer accepted it.
- **Check:** `crates/raf-okf/tests/bounds.rs::a_long_heading_over_many_lines_stays_linear`.
  - A counting allocator with a 1 GiB ceiling compares two 1 MiB documents, one with a 60 KB heading and one with a 1-byte heading, for prose lines and for PROPOSED blocks.
  - After the fix, the peak difference is 307,181 bytes against an allowance of 311,296.
  - **On the old code it aborts:** `memory allocation of 61440 bytes failed`, SIGABRT. This was reproduced independently by the reviewer.
- **Also:** the new fuzz target `parse-document` (`Document::parse` + `Profile::validate`) is run by `make fuzz`.
- **Gates:**
  - `cargo test -p raf-okf --locked`: pass.
  - Workspace clippy with `-D warnings`: clean.
  - `make fuzz`: both targets ran 5000 times, no findings.
  - `make reuse`: 210/210.
  - Pre-commit hook: docs_contract 138 and repo_gates 55 passed.
- **Review:** `tasks/R1-review.md`, VERDICT: PASS. Three advisories: the margin is thin, the test has only one section, and there is no child-process harness for the old-code run.

## 6. paudley — 

**R2 fixed: modules are placed under their declared path** (commit 0617985, pushed)

- **Gap:** CA-F2 (HIGH). Whenever a module's file sorted before the file that declares it, its items were recorded under the parent's path. In this repository the binding gate accepted the non-existent `raf_okf::body::FenceBlock`, and valid crates failed with a false `SuperAboveRoot`.
- **Root cause:** `Index::build` read files in file-name order, and `Builder::ensure` let the child take a slot before it created the missing parent.
- **Fix:**
  - The walk records each file arrival in walk order, before following that file's own `mod` declarations, and the index processes arrivals in that order.
  - A module is created only by its `mod` declaration.
  - A reach whose module was not declared first is a fail-hard internal error.
- **Checks:** seven `module_tree.rs` placement fixtures:
  - a `mod.rs` child and a flat child that sort first;
  - the `#[path]` children `a`, `ga` and `z`;
  - a shared `tests/common` module reached from a later target;
  - `super::super`;
  - a file module inside an inline module;
  - paths that must not be found.

  Also: `this_repository.rs` (`raf_okf::body::blocks::FenceBlock` resolves, the misplaced path is `NotFound`), and a `docs_contract` negative fixture. **Every one fails on the old code**, which the implementer and the reviewer each checked separately.
- **Registry scan:** the new example `crates/raf-source/examples/registry_scan.rs` loads each crate in its own child process. Over 1,177 crates:
  - crates with misplaced items: 165 → **0** (2,757 items → 0);
  - `SuperAboveRoot`: 3 → **0**;
  - crates that load: 1,014 → 1,021.

  Every changed outcome is root-caused in the stage validation index:
  - jni, rustls and wasm-bindgen now stop at the macro-generated-impl class, which R4 owns.
  - web-sys's quadratic glob memory is unchanged by R2 (measured), and R5c owns it.
- **Gates:**
  - raf-source tests pass in debug and release (257+).
  - Workspace clippy with `-D warnings`: clean.
  - docs_contract 138, repo_gates 55.
  - `make reuse`: 211/211.
- **Review:** `tasks/R2-review.md`, VERDICT: PASS after recheck. One advisory is folded into R3 (a tighter macro oracle and a per-child memory cap in the harness). One pre-existing defect is folded into R4 (a lib module named `bin_crate`).

## 7. paudley — 

**R12o fixed: raf-okf tests, fenced-block accessors and validation cost** (commits 100bf2f, d89b78f, 671850b, f5ede44, pushed)

- **G4 / Q-3:** the unknown-key check runs on every document type. Each type gets an unknown key, a key that belongs only to another type, and unknown keys nested under `bindings` and Change items.
  - Mutation: adding a key to the Capability table makes the test fail.
- **Q-1:** the round-trip golden test asserts, per directory, that it rendered exactly as many documents as `git ls-files` tracks there.
  - It also asserts at least one document for capabilities and architecture. It does not for change records, which are legitimately deleted when they complete.
  - Two mutations, skipping a directory and dropping a file, each make it fail.
  - Three committed fixture documents, one per type, round-trip independently of the repository's own documents.
- **Q-2:** tests now pin a leading BOM, a NUL in the body and a `--- ` fence. The Contract is silent on all three, and each test has a recorded mutation that fails it.
- **G17:** `FencedBlock::content()`, which borrows from the shared body text with no copy per block, and `FencedBlock::is_closed()`.
  - `Debug` and equality cover only the block. The derived forms would have printed and compared the whole body; a test catches that.
  - `docs_contract` uses the accessors, and a `~~~rust` Usage block is now accepted. That test fails on the old code.
- **P-F2 / P-F3:** `Profile::validate` no longer scans quadratically. `changes` is held in a set, and generated blocks are found by binary search.
  - On the old code the release probes took 324 ms and 512 ms. They are now under the 100 ms bound, which `validate_cost.rs` asserts in `make test-release` and the release CI job.
  - The reviewer compared 47,564 violations over 15,102 mutated documents before and after: byte-identical.
- **Gates:** raf-okf tests 176 (debug and release), docs_contract 139, repo_gates 55, workspace clippy, rustdoc, `make reuse`, fmt, and the hook on each commit.
- **Review:** `tasks/R12o-review.md`, VERDICT: PASS after recheck. Two REQUIRED findings were fixed: the Debug and equality blow-up through the shared body, and a brittle floor on change records.

## 8. paudley — 

**R3 fixed: file identity, iterative traversal and work budgets** (commit cd7d4fb, pushed)

- **Gaps:**
  - **S-2 (High):** files were keyed by how the path is spelled, so in-root symlinks gave 2^k parses. A 1.3 KB tree took 24 s and 2 GB, and a larger one was OOM-killed.
  - **S-4:** discovery recursed on the caller's stack, and deep directories aborted.
  - **P-F4:** the module-path budget did not bound cost. 120 KB of source produced 3.1 GB.
  - **P-F8 / P-F10:** cubic discovery, and a recursive module walk.
- **Identity:** a file's identity is its device and inode on Unix, its canonical path elsewhere, or its normalised key in memory. `Tree::read_unseen` gets the identity and the text from **one** verified open; strace confirms one open per file. A file reached under several spellings is parsed once.
- **Budgets:** each is typed and refused naming the budget and the file or line.
  - Total bytes: 256 MiB.
  - Item bytes: 512 MiB.
  - Fact bytes: 256 MiB, charged on the parse thread.
  - Directory depth: 128.
  - Tokens per file: 2,000,000, the owner's decision "(1) - default to 2M". It is configurable, and the one-file parse peak is ≤ limit × 1,664 B, about 3.1 GiB at the default. The Contract recommends 320,000 tokens for hosts that need under 512 MiB.
- **How the charges hold:** every charge is an **upper bound by construction**, derived in `cost.rs` from how `Vec`, `BTreeMap` and hash tables allocate.
  - Three review rounds tried to beat the meters: per-arrival copies, one-name scopes, nested inline modules, many tiny files, targets, impls and spellings.
  - The last round's held/charged ratios are 0.30–0.92 across all of them, and a load with every budget at its limit keeps about 716 MiB, within the stated 1 GiB.
- **Honest limit:** the scope tables that `use` declarations and globs build have **no budget yet**. The Contract and SECURITY.md say so plainly, with the measured case of 166 KB of source taking 7.3 GiB. R5c adds that budget.
- **Iterative traversal:** discovery, member expansion, the module walk and index building no longer recurse.
  - 20,000 directory levels run on a 2 MiB thread, and 128 nested modules on a 512 KiB thread.
  - Line-position offsets are now computed in linear time.
- **Security probes re-run:**
  - Symlink amplification: 24 s / 2 GB before, `ModuleCycle` in 1–3 ms now.
  - The 2,100-level disk chain and the 20,000-level memory chain end typed in milliseconds.
  - The 8 MiB `a<b; ` file (2.6 GB before) is refused by the pre-scan in 4–25 ms.
- **Registry scan at defaults:**
  - 1,021 of 1,177 crates load, with 0 misplaced items and **0 budget refusals**.
  - The largest item charge is 64% of its limit (`linux-raw-sys`).
  - chrono-tz (1.5M tokens) loads.
  - web-sys stops at the per-child memory cap in the unbudgeted glob tables.
- **Behaviour preserved:** the reviewer found output identical to the old code on this repository and on 161 registry crates (items, file reaches, every resolution).
- **Gates:** raf-source tests 294+ in debug and release, workspace clippy, rustdoc, docs_contract 139, repo_gates 55, `make msrv`, `make cross-check`, `make reuse`, and the hook.
- **Review:** `tasks/R3-review.md`, VERDICT: PASS after three rechecks.

## 9. paudley — 

**R8 fixed: containment verified on the handle that is read, on every platform** (commit 51d84e5, pushed)

- **Gaps:**
  - **G3 (CRITICAL) / S-8:** listing "through the handle" held only on Linux.
  - **S-6 / S-7:** a raced read could open an outside FIFO, and `entry` revealed outside existence.
  - **G7:** non-Linux code never ran, and the Windows test build failed.
  - **Found in review:** on macOS, BSD and Windows, a swap made before the open and undone before the check **returned outside file contents**.
- **Owner decision** (verbatim in the stage validation index): "Add rustix + windows-sys".
- **Fix by platform:**
  - **Linux:** `O_PATH | O_NOFOLLOW`, then a `/proc/self/fd` link and type check, then a reopen to read or list. No outside FIFO or device is ever opened for I/O, and `read_unseen` keeps one open per file (confirmed by strace).
  - **macOS and BSD:** an `openat` walk from a root descriptor with no symlink following (`rustix`).
    - Links are resolved by the walk, and a `..` at or outside the root is refused, which removes the existence oracle.
    - It holds one descriptor at a time (tested at 300 levels under `ulimit -n 64`) and re-checks after use.
  - **Windows:** a held handle without delete sharing, plus a `GetFinalPathNameByHandleW` check before and after use.
    - That FFI call is the only library `unsafe`, in one module with a scoped allowance and a SAFETY argument. The workspace lint is unchanged.
    - Reserved names, trailing dot or space and `:` segments are refused.
- **Checks:**
  - Every strategy runs on Linux under test.
  - Swap, FIFO-race, out-and-back-link and descriptor-limit suites. The FIFO race fails on the old code 10/10.
  - Over 65,536 exhaustive swap schedules and 200k races per strategy: **0 outside contents** (old code: about 3k leaks), and Linux/macOS-model 0 outside names.
  - The Windows model keeps its documented residual: outside *names or kinds* can leak via listing and entry, never contents.
- **Docs:** the Contract, SECURITY.md and AGENTS.md state each platform's exact guarantee and residual. They now say which links load where: links that climb back via `..` load on Linux and Windows but not on macOS or BSD.
- **CI and gates:** new `test-macos` and `test-windows` jobs. `make cross-check` now covers all targets for Apple, **FreeBSD** and Windows. `make check` passed (1,112 tests), plus clippy for each target and `make deny` with the two new dependencies (no duplicate versions).
- **Not yet verified:** macOS, FreeBSD and Windows runtime. These strategies compile with `-Dwarnings` and are exercised on Linux, but only the hosted jobs will run them natively, and hosted CI is blocked by billing.
- **Open option for the owner:** running the same walk on Linux would remove Linux's remaining one-bit existence answer for planted links and the `/proc` dependency. It is outside the current authorization, so it was not done.
- **Review:** `tasks/R8-review.md`, VERDICT: PASS after three rechecks.

## 10. paudley — 

**R4 fixed: macro-generated impls, enum globs, macros in trait bodies, shared crate names** (commit d1c4a7f, pushed)

**Owner decisions applied:**
- **A1 (per-item marker):** an inherent impl of a type the index can't see no longer fails the load.
  - Its members are items marked `SelfType::Unresolved`, recorded under the written self path.
  - A path through it is `ResolveError::UnresolvedSelf`.
  - `Workspace::unresolved_impls()` lists them, so a consumer can refuse them on its own terms.
- **A2 (key crates by package):** each target resolves crate names through its own extern prelude.
  - The prelude holds the package's library, the dependencies its kind sees (with renames), and root `extern crate`.
  - Dev-dependencies are seen only by test code and targets.
  - Build-dependencies are seen only by `build.rs`, which does not see its own library, as in Cargo.
  - A crate name that stands for several libraries is `Ambiguous` whatever follows it, through every route: direct, `extern crate`, imports in each namespace, and globs. Trait keys through it are `Unresolved`, with a candidate per library.
  - `Workspace::resolve_in_package(package, path)` disambiguates.

**Other fixes in this item:**
- **CA-F5 / CA-F6:** macro calls in trait bodies are skipped, and globs of local enums bring in their variants.
- **A3 and the R2-review `bin_crate` collision:** items are grouped by target, so an overlap is `Ambiguous` in either file order, and a lib module named like a target segment keeps its items.
- **First-wins removed:** a name bound to several modules now gives a trait key with a candidate through each.
- **`workspace = true` without an entry** is a manifest error.
- **S-15 / G16:** error text fixed.

**Entry points:**
- **katamari `crates/`:** loads (13k+ items, 2 unresolved impls). It failed before.
- **purrdf `crates/`:** loads (83k items). Its two `purrdf` libraries resolve `Ambiguous(2)`, and `resolve_in_package` works for each. It failed before.
- **Registry:** 1,163 of 1,207 crates load, with 0 misplaced items and 0 budget refusals. No crate fails with the inherent-impl, macro-in-trait or enum-glob message; R3 had 100, 14 and 6 of them.
  - Every changed outcome is root-caused in the stage validation index.
  - The remaining self-alias failures (js-sys, wide, core-foundation, bincode, ena) are R5a's.

**Checks:**
- Every new fixture fails on the old code.
- A resolve dump of every public path in katamari, purrdf and this repository was **unchanged** by the final consistency rounds.
- Counting-allocator probes keep held memory ≤ charged for the new data.

**Gates:** raf-source 334 tests in debug and release, workspace clippy, rustdoc, docs_contract 139, repo_gates 55, cross-check, msrv, reuse, and the hook.

**Review:** `tasks/R4-review.md`, VERDICT: PASS after three rechecks.

## 11. paudley — 

**R5a fixed: extern-prelude aliases and stack-safe trait keys** (commit e472918, pushed)

- **Gaps:**
  - **G2 / CA-F1 (CRITICAL):** `pub use serde;`, `use std;` and `pub use dep::{self, ..}` resolved through their own binding, giving `Cyclic`. A trait impl through such an alias failed the whole load; this repository's own `raf_source::syn::File` was `Cyclic`.
  - **CA-F7:** trait-key recursion could abort the process on valid input.
- **Fix:**
  - An import never consults its own binding. A single-segment import of an outside crate binds nothing, so paths through it are `NotFound`, and trait keys follow it to the dependency path. An import of a workspace library binds its crate root.
  - Edition 2015 paths start at the crate root, and a head names a crate only through `extern crate` or the injected `std`/`core`. `#![no_std]` is honoured.
  - **rustc's rules, verified with the compiler:** an item or explicit import shadows a crate, and a glob-imported name collides with it (E0659). The plan's broader "local item vs crate → Ambiguous" was corrected to this.
  - **Order-independent:** a path waits for its module's globs and for unsettled explicit imports before reading a crate, and a final pass re-checks every import against the settled tables.
  - Trait keys are derived with an explicit worklist. 20,000 chained aliases or globs key on a 2 MiB thread, where the old code overflowed at about 2,000.
- **Checks:**
  - Fixtures from the plan plus every-order fixtures, over 24 orders and with or without globs. Each fails on the old code; the parts that are already correct have mutation checks.
  - The reviewer's random-program search (1,440 programs × 12 orders) found no remaining order dependence in R5a's scope. Two cycle shapes rustc rejects belong to R5b.
- **Registry:** 1,183 of 1,213 crates load. `UnresolvedTrait` went from 19 to 5, and the remaining 5 are correct refusals (traits from `include!` or macros, and a cfg'd-off invalid module).
  - Now load: bincode, ena, option-ext, core-foundation, rmp-serde, js-sys, wide, cx448, p256, p384, p521, hmac, ripemd, sha3.
- **Real workspaces:** item resolutions and trait keys are unchanged across 99k+ paths. Alias paths now resolve as rustc does; for example, 135 katamari test paths behind `use …testkit as support;` now resolve.
- **Gates:** 357 tests in debug and release, clippy, rustdoc, docs_contract 139, repo_gates 55, cross-check, msrv, reuse, `make check`, and the hook.
- **Known limit, stated in the Contract:** a glob carrying an alias of an outside crate's *type item* under a crate's name (`pub use serde::de as other;`) can't be classed by namespace, so it is not refused. rustc gives E0659 there.
- **Review:** `tasks/R5a-review.md`, VERDICT: PASS after two rechecks.

## 12. paudley — 

**Remediation batch: R5b, R-CI, R6, R7, R9, R10, R12s and a BOM span fix** (all pushed)

**R5b: import cycles settled by rustc's rules, independent of order** (5a5e499)
- **Namespaces:** names settle per namespace. An explicit import shadows globs only in the namespaces it binds.
- **Cycles:** a cycle that exactly one glob answers resolves through it. Cycles with no answering glob, or more than one, are `Cyclic`.
- **Visibility:** paths are read with rustc's accessibility rule, and an item is as visible as its most visible glob route.
- **Bounds:** both settle-loop bounds are checked in release.
- **Performance:** chains of 20,000 imports and import cycles settle in under a second, where they took minutes before.
- **Evidence against the rustc oracle:** 24,000+ generated programs, plus fresh-seed private-glob runs.
  - Programs with no order dependence and no panics.
  - Wrong `Cyclic` refusals: 0, down from 38.
  - Wrong `Ok` results on valid code: 12 queries in 10 of 5,522 programs, down from 96 in 71 on the old code.
- **Real code:** 117 imports in katamari and purrdf now bind the public item that rustc picks, not another crate's private module.
- **Stated limits:** two rustc visibility rules are not modelled, and the Contract lists them. Recording them as accepted limits was the orchestrator's call; the owner can overrule it.
- **Review:** `tasks/R5b-review.md`, PASS after four rechecks.

**R-CI: findings from the first hosted runs** (4d56b6f)
- Fixes for the floating nightly's new `map_or_default` lint. The local toolchain is updated to match CI.
- On macOS, the non-UTF-8 listing test now asserts that APFS refuses the file name.

**R6: one additive parser recursion budget** (b187300)
- **Gap:** G1 (CRITICAL). The Contract's carve-outs for known overflow shapes are removed.
- **Counter:** one per-level count of the tokens that start a nested syn parse, with reset rules and a limit of 1927. That limit is 64 MiB divided by twice the costliest measured debug unit.
- **Results:**
  - All 35 committed overflow shapes are refused in both debug and release, and none aborts. The old code aborted 39 times across them.
  - 39 shape families each load at their largest admitted size and are refused one size more. The tightest has 2.2× debug stack headroom.
  - The registry scan has 0 refusals; its highest score is 987.

**R7: fuzz reach** (a26e75d)
- **Targets:** three, `parse-source`, `parse-document` and the new `load-workspace`.
- **Limits:** a 400 KB input cap, plus resident-memory and allocation limits.
- **Fixed in review:** `--cfg fuzzing` made `proc-macro2` drop span locations, so `load-workspace` never reached the resolver. The targets now build with `--no-cfg-fuzzing`, a seed-load test guards against a repeat, and coverage rose from 4,501 to 11,772.

**R9: an empty root list is a typed `NoRoots` error** (bd6d3aa)
- This is the owner's decision "Typed NoRoots error".
- G15's impossible branch is removed.

**R10: manifest and error-text hardening** (655834b)
- **Manifests:** `members` and `exclude` are capped at 4096 entries each, and patterns are compiled once. The 880 KB probe took 2 minutes and now gets a typed refusal in 2 ms.
- **Error text:** every error `Display` escapes control characters, bidi controls included, and so does `PathError`.

**R12s: Verification tests on the real load path** (20b59c5)
- 8 MiB cap tests through a load.
- A real parse-thread `Spawn` failure under `ulimit`.
- A guard that the root manifest keeps `missing_docs` set to deny.
- CRLF, BOM and non-UTF-8 source loaded through `Workspace::load`.

**BOM span fix** (71bd2c7)
- **Bug:** found while writing the R12s tests. A leading BOM shifted every item span on line 1 by 3 bytes.
- **Fix and check:** line 1 now starts after the mark. On the old code, the test reports the first item starting at byte 0 instead of 3.

**For every item:** fixtures fail on the old code or carry a recorded mutation; there is an independent review file under `tasks/`; and clippy, both test profiles, docs_contract and repo_gates were green at each commit, with the hooks run.

**Still to do in Stage 2:**
- R5c: a budget for glob scope tables, which fixes web-sys's memory.
- R5d: resolve-path validation, plus `ResolveError` escaping.
- R11: API sealing.
- Then one `make check`, green hosted CI and a fresh completion audit.

## 13. paudley — 

**Final remediation batch: R13, R5c, R11a, R5d, R11b** (all pushed)

The owner's scope decision, recorded verbatim in the stage validation index: "Everythign - you have 6 hours - capped". Every remaining item stays in this PR.

**R13: licence exceptions pinned to the audited versions** (d145545)
- `unicode-ident@=1.0.27` and `libfuzzer-sys@=0.4.13`.
- A deliberately wrong pin fails `make deny`, which proves the pin binds.

**R5c: glob scope tables have a byte budget** (be71d40)
- **Design:** shared per-visibility tables, built once; only the watching globs are woken; a 512 MiB glob-bytes budget, charged before allocating, with a typed refusal over it.
- **web-sys** now **loads** in about 5 s at 144 MiB, where it used to run out of the 8 GiB cap.
- **166 KB hostile file:** this was 7.3 GiB before. It is now refused, typed, at about 414 MiB.
- **Found in review and fixed:** repeated identical globs allocated before charging. They are now deduplicated and charged first, so 40,000×1,500 loads in 0.5 s, held ≤ charged.
- **Checks:** registry outcomes unchanged (1,204 OK, 0 budget refusals). Resolve dumps of this repository, katamari and purrdf are byte-identical.

**R11a: `items_under` matches whole segments; doc-contract checks binding roles** (df71a42)
- `items_under("demo::a")` no longer returns `demo::ab` or `demo::a1`.
- A `tests` binding must name a `#[test]` function, and a `code` binding must name a shipping item.

**R5d: resolve-path validation** (d8aba1e)
- `crate`, `self`, `super` or `Self` mid-path is now `Invalid`.
- The swallowed `SuperAboveRoot` arm is removed.
- A glob of a missing local module is the typed `UnresolvedGlob`, and the load still succeeds.
- The error order is fixed and documented.
- A 50,000-segment path takes milliseconds in release, against seconds before.
- `ResolveError` text is escaped.
- **Pre-existing misreport, found and fixed:** two dependency globs in one module reported a missing name as `Cyclic`. It is now `NotFound`, and real glob cycles stay `Cyclic`.

**R11b: public API sealed** (bf08e45, `feat(raf-source)!`)
- `#[non_exhaustive]` on the types consumers bind to, and the per-file facts are crate-private.
- `Target` gains `harness`, `required_features`, `test` and `doctest`, with Cargo's defaults. A library `required-features` is refused, as Cargo refuses it.
- `SpanFact` line ranges are half-open.
- `LoadOptions` is no longer `Copy`.
- `Outcome` and `LinkTarget` are exhaustive.

**For every item:** an independent review is under `tasks/`, and every one is PASS. Fixtures fail on the old code or carry a recorded mutation, and each commit passed its gates and hooks.

**Hosted CI:** fully green on 44e18b7 (all 12 jobs, including native macOS and Windows). The run on bf08e45 is in progress.

**Running now:** the final `make check` and a fresh completion audit. Then Stage 3.

