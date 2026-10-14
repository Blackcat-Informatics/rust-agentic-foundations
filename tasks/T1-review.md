# Current status

The first review below blocked; the findings were fixed and the later recheck sections (at the end of this file) passed. Earlier sections are kept as written.

VERDICT: PASS

---

# Task 1 review: raf-okf frontmatter and writer

## Reviewed scope

Staged diff (`git diff --cached`, 25 files) against plan.md: User decisions 1, section A, B1, B2, the section D YAML reader/writer rows, the completeness rows whose Task column includes 1, and "## Task 1". Read in full: every file under `crates/raf-okf/src`, `tests/{refusals,render,schema,alloc_bound,no_io_in_source}.rs`, the AGENTS.md, Cargo.toml, architecture and capability doc edits, and the DC YAML fixtures in `tests/docs_contract.rs` (lines 1997-2012, 2685-2842, 3311-3420).

## Evidence

- `cargo test -p raf-okf --locked -- --list`: 49 tests plus 1 doctest (orchestrator runs all green).
- Silent-path grep over `crates/raf-okf/src` for `.ok()`, `unwrap_or*`, `filter_map`, `else { continue }`, `_ => {}`, `#[allow`, `unsafe`, `or_insert`, `unwrap()`, `expect(`: one hit, `scalar.rs:359` `find([',', ']']).map_or(rest, ..)`. That is the "no delimiter, take the rest of the line" case of a search, not an error turned into a default; the following branches still reject. No `#[allow]`, no `unsafe`, no `unwrap_or_default`. `Cursor::new`/`advance` use `assert!` on a character boundary, an internal invariant (see non-blocking 5).
- Process-label / precedent-name / tracker grep over all staged files: clean. SPDX header present on all 19 `.rs` files and Cargo.toml.
- Throwaway probe crate in /tmp/probe (path dependency, outside the worktree):
  - 29 hand-written edge inputs (unterminated quotes in flow/list/nested positions, colon-space in a flow list, `>` indicator in an item value, stray/over-indented lines after a flow value, every trailing-comment form, multibyte columns, odd spacing): all behave correctly and carry positions.
  - 400 000 random token-soup frontmatters in release: no panic, `render` idempotent, `parse(render(d))` parses; `parse(render(d)) == d` for every case without `tags`.
  - `tags: [b, a]` parsed, rendered, re-parsed: `render` gives `tags: [a, b]` but `parse(render(d)) == d` is **false** (finding 1).
  - Unknown type with `tags: [b, a]` renders `tags: [a, b]` (tags sorted whatever the type; acceptable).
- DC fixture comparison against `refusals.rs`/`render.rs` (finding 2).

Covered with a refused case beside an accepted neighbour and an asserted position: flow mapping (`bindings: {code: []}` line 3 col 11, key named), duplicate keys top level / `bindings` / inside an `items` entry / on the `- ` line, anchors, aliases, `!x`/`!!str`, `<<`, block scalar headers, `\q`, `\x41`, `\u`, non-mapping, no fence line 1, unclosed fence, exact 1 MiB accepted and +1 refused (also multibyte), depth 32 accepted/33 refused at line 34 col 65 (and the flow-list level), 100 000 `[` / `- ` / `[ ` levels return `Err` on a 2 MiB thread, CRLF and missing final newline byte-identical, goldens from `git ls-files`, table key order (optional tags/timestamp), unknown type input order, tags sorted, comment attachment, constructors refusing, `plain_ok` implies reader accepts and reader refusals are never `plain_ok`, escape round trip, `PartialEq` semantics, billion laughs under `cap` with no unsafe, scanner for `std::fs`/`std::process` with positive and negative fixtures (comments, strings, grouped imports, multi-line). B1 "fixes required" all done: byte cursor, `MAX_YAML_DEPTH` in `limits`, `ListItem`, no `InvalidSpan`, `Frontmatter { map, trailing_comments }`.

## Findings

### Blocking

1. **Round-trip law fails for unsorted `tags`, and the capability document says otherwise.** `crates/raf-okf/src/frontmatter/value.rs:287-293` (`Entry` eq) and `:166` (list items compared in order) compare `tags` in author order, while `writer.rs:51-54,155-160` sorts on render. So `Document::parse("...tags: [b, a]...")` != `Document::parse(&d.render())` (probe: `eq=false`). The plan row (Deterministic writer, round trip) states `parse(render(d)) == d`. `docs/capabilities/okf-profile.md:80-81` states "an unsorted `tags` list is compared in the sorted order the writer gives it", which the code does not do. The only test (`render.rs:411-423`, `a_built_document_with_sorted_tags_...`) uses already-sorted tags, so nothing pins the claim. Fix: compare the top-level `tags` list in sorted order in the equality (or sort at construction and parse), add a test with `tags: [b, a]` asserting `parse(&d.render()) == d`, and keep the doc sentence; or drop the doc sentence and state the exception in `Document`'s doc and the plan law. Whichever, doc and code must agree and a test must pin it.
2. **The DC YAML-subset negative and accepted fixtures were not all brought across.** Present: unterminated `"`/`'` at top level, text after closing quote, `: ` and trailing `:`, `%` `@` `` ` `` indicators, block headers (top level and `- |`), duplicate key, `[a,,b]`, unterminated flow, `Bad Key`, `a:  b`. Missing from `crates/raf-okf/tests/refusals.rs` (all work per probe, none pinned): (a) unterminated quote inside a flow list (`[x, "y]`), as a list item (`- "cap`), and as a nested block-list item (`- 'src`); (b) `: ` inside a flow list (`[x, a: b]`); (c) block header `- >` and indicator `outcome: > folded` inside an `items` entry; (d) a `- stray` line and an over-indented line after a flow value (`code: []`); (e) the whole `accepts_comments_after_scalars` set: `"a # b" # c`, `'It''s # x'  # c`, `hash: a#b` (only `a#b` as a `plain_ok` candidate), `list: [one, "two # 2"] # c`, `empty: # nothing`, `- claim: x # c` with the value text unaffected by the comment. Add these with positions; each is a one-line case.

### Non-blocking

3. **Architecture page contradicts the built schema at this commit (medium).** `docs/architecture/okf-profile.md:22-23` ("exactly the keys listed ... no more and no fewer") and `:40` (column "Keys, all required") remain, while `crates/raf-okf/src/profile/schema.rs:63-93` marks `tags` and `timestamp` optional on all three types and `docs/capabilities/okf-profile.md:84-86` already documents tags sorting. The page never mentions `tags`/`timestamp` at all. The existing `docs_contract` validator still enforces the old closed set, so no gate fails, but the public `DocType::keys()` and the Write contract describe a profile the architecture page denies. Assigned to Task 3; acceptable only if Task 3 lands in the same PR before merge, which the plan guarantees. A one-line interim note in the page would remove the contradiction at this commit.
4. **Golden count assertion is a lower bound.** `render.rs:78-81` asserts `paths.len() > 20`; the plan says "with a count assertion". An exact number (or derive-and-compare to a second source) would catch a silently dropped directory. The gate root for contents (`RAF_GATE_ROOT`) and the file list (`git ls-files` of the real repository) also come from different places in the hook case; they agree in practice.
5. **`Cursor` uses `assert!` (panics) on a character boundary** (`scalar.rs:41,70`). The invariant is maintained by construction and the 400 000-case probe found no panic, so the "never panics" doc claim on `Document::parse` holds in practice. A returned `Fail` would match the no-panic stance more strictly.
6. **Span coordinates are ambiguous for Task 2.** `Document::parse` places the body at offsets of the source text (`document.rs:41`), `Document::new` places it at offsets of the canonical rendering (`:60`), and `render()` of a non-canonical parsed document (reordered keys) has a different head length from the source. Task 2's `replace_spans` and `docs sync` need one stated meaning (source text of the parse is the natural one); say so on `Body::span`.
7. **`Body::new(text)` is public, infallible, and unvalidated**, and `Document::new` accepts it. Task 2 adds fallible `Body::parse`; keep `Document::new` taking only a parsed body (or make `Body::new` crate-private) so a constructed document cannot hold text that does not re-parse, in keeping with "constructors refuse unreadable values".
8. **Alloc test strength.** The billion-laughs case is refused at the first anchor, so the 8x bound is checked on a 9-level 600-byte input and on a 900 KiB many-keys input; there is no case where an implementation expanding aliases would be caught beyond the first anchor refusal. The refusal-position assertion carries the real guarantee. Fine as is.

### Task 2 / 3 readiness

The API supports both without rework: `Document::frontmatter()`, `Map::get`, `Entry::line()`, `ListItem::line()`, `Scalar::text/style`, `ValueKind`/`KeySpec{required}` with `DocType::keys()` as the one table the writer already reads, `DocType::{directory,id_prefix,from_name}`, `validate_timestamp`, `ParseError{kind,line,column,offset}` with `#[non_exhaustive]` kinds. Module layout matches B (body/ and profile/ stubs only hold Task 1 needs). Only items 6 and 7 touch Task 2.

### Docs accuracy

AGENTS.md members/layout/crate-status edits and the architecture Value rules/"Frontmatter subset" section match the code (escape set, bounds, line-ending rule, refused list) and are in present tense; the trailing-comment wording is accurate. The one false statement is finding 1.

VERDICT: BLOCKED

## Recheck

Evidence: read the restaged diff of `document.rs`, `body/mod.rs`, `scalar.rs`, both docs, `tests/docs_contract.rs`, and the new tests in `refusals.rs` and `render.rs`; reran `cargo test -p raf-okf --locked` (all green: refusals 26, render 20, schema 5, alloc 1, no_io 3, doctest 1); reran the /tmp probe on unsorted tags.

Adjudication
1. Blocking 1 (unsorted `tags` round trip): DISCHARGED. `Document::parse` and `Document::new` both call `sort_tags` (`document.rs`, stable sort by scalar text, item comments travel with the item). Probe: `tags: [b, a]` and a block list with leading and trailing comments both give `eq=true` after render and re-parse. Test `unsorted_tags_render_sorted_and_the_document_equals_its_rendering_read_back` pins it, and the Contract sentence in `docs/capabilities/okf-profile.md` now says the list is sorted at parse and build, which matches the code.
2. Blocking 2 (DC fixtures): DISCHARGED. `refusals.rs` now holds the unterminated quote as a list item and nested item (`:692`, `:695`), `a: b` in a flow list (`:706`), the `> folded` item value with position and a quoted neighbour (`:735-741`), stray and over-indented lines after a flow value (`:746`), the full trailing-comment set (`:761-803`) and an unterminated quote in a flow list beside it (`:817`).
3. Non-blocking 3 (architecture page contradicts the schema): DISCHARGED. The page now lists optional `tags` and `timestamp` in table order, the Value rules for both, and "every key is required except". Interim gate in `tests/docs_contract.rs` (`OPTIONAL_KEYS`, list-of-unique-non-empty-strings for `tags`, non-empty scalar for `timestamp`, three fixtures): ACCEPTABLE as an interim. It is looser than raf-okf's RFC 3339 check (a malformed timestamp passes the old gate), so the page states a rule the gate does not yet enforce; that gap closes when Task 3's `Profile::validate` and Task 8's switch land in this change, and the old gate never claimed more than key and shape checks. It does not weaken any existing check (closed key set, `resource` rejection, required keys are unchanged). Non-blocking; Task 8 must delete the interim code rather than keep both.
4. Non-blocking 4 (golden count is a lower bound): STILL-OPEN, non-blocking. `render.rs` still asserts `paths.len() > 20`; harmless, an exact count is still preferred.
5. Non-blocking 5 (`Cursor` asserts): DISCHARGED. No `assert!` or `panic!` remains in `scalar.rs`; the invariant is stated in a comment and the cursor advances only by matched lengths.
6. Non-blocking 6 (span coordinates): DISCHARGED. `Body::span` documents parsed (source-text) versus built (render) coordinates, and `the_body_span_uses_the_coordinates_of_the_text_the_document_came_from` plus the built-document span test pin both.
7. Non-blocking 7 (`Body::new` public): DISCHARGED. It is `pub(crate)` and `Document::new` takes `impl Into<String>` for the body; Task 2 can route this through `Body::parse` without a public API break.
8. Non-blocking 8 (alloc test strength): unchanged, accepted.

New text: `sort_tags` orders non-scalar items before scalars by `Option` ordering, unreachable for constructed lists of scalars and for a parsed block list of maps under `tags`, where the profile will reject the value anyway; no issue. The sentence added to the capability document is one long line (`okf-profile.md` Write paragraph) that breaks the surrounding wrap; cosmetic.

No new blocking findings.

VERDICT: PASS
