# T2 review: the graph and its queries

## Scope
Uncommitted Task 2 on top of 79cc6fb: `crates/raf-workplan/src/graph.rs` (new), `load.rs`, `model.rs`
(`note_present`, `relate`, `graph()`), `finding.rs` (4 kinds), `lib.rs` exports, tests `graph.rs`,
`references.rs`, `kinds.rs`/`support/cases.rs`, CHANGELOG. Checked against plan Task 2, G1, G2, R12, R24,
"Generated content" section rule, the document-model contract (graph).

## Requirement check
- G1 nodes (capability, section, change, architecture, proposal, symbol, test, example) and edges (binding,
  depends, affects, changes, link) built once at load; `NodeKind::ALL`/`EdgeKind::ALL`; fixtures per kind: met.
  Link edges are section-to-section (from the holding section, to the anchored section, else the document): met.
- R12 four questions on the shipped API: `code_of` (code bindings only, path-sorted), `capabilities_of(ItemId)`,
  `tests_for(cap, section)` (G10 rule, typed `&Binding`), `sections_of(change)` (sorted, deduplicated), plus
  `linking_sections`: met, each with a fixture and a determinism test.
- Rules: `depends-missing`, `changes-missing`, `moved-missing`, `depends-cycle` (one finding per strongly
  connected group naming all members, Tarjan without recursion; self, 2-, 3-node, two cycles, tail): met.
  `affects` section errors stay raf-okf's (G3). `outcome: deferred` reaches the okf kind; R24 reuses
  okf-id-file-name/okf-invalid-id: met.
- Binding resolve errors are kept on the edge, no binding findings (Task 4): as planned.

## Evidence
- `cargo test --locked -p raf-workplan`: all pass (26 + 15 + 4 + 1 + 13 + 1 + 1 ...), including the G2 both-direction coverage.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: clean. `docs_contract` 141 + `repo_gates` 57: pass.
  `make reuse`: compliant (294/294). rustfmt --check on all raf-workplan sources and tests: clean.
- No issue numbers or process wording in the new files; SPDX headers present; `missing_docs` lint passes (public API documented).
- Mutations (graph.rs restored byte-identical, sha256 verified; run with --no-fail-fast):
  - ignore 3-node cycles: KILLED (references::rejects_depends_cycles_once_for_each_group_naming_its_members, every_reference_case_is_reported_exactly, kinds coverage).
  - ignore self cycles: KILLED (same plus two_separate_cycles_...).
  - `tests_for` rule inverted: KILLED (4 tests in graph.rs).
  - link source not section-granular / link target not section-granular: both KILLED (links_run_from_the_section..., linking_sections_are_section_granular).
  - drop one SCC group size (>=3 not reported): KILLED (kinds coverage).
  - code-span boundary: dropping the "ident before / ':' before" and "ident after" checks: KILLED (tests_for_a_binding_named_in_two_sections..., ..._does_not_resolve...).
  - code-span boundary: dropping the `starts_with("::")` after-check: SURVIVED (see A1).
- Hostile probe (scratch test, deleted afterwards): two files claiming one id (second is rejected by okf-id-file-name, graph keeps a single node set, no panic);
  `depends` on itself (one depends-cycle finding); a fragment to a heading that exists only in another document, or to a duplicated heading `#usage-1`
  (resolved by raf-okf anchors: edge to the right section, nonexistent anchors give no edge); links inside a fence and inside a code span are not edges;
  unterminated backticks and `demo::dup` resolving to two sites (cfg variants) neither panic nor change ordering. `Ambiguous` itself could not be provoked with
  a fixture here, but `bind` stores whatever `Result` the index returns and `items()` is empty on Err, so no panic path exists.
  Output order is from sorted vectors and BTree sets only, so it is deterministic.

## note_present soundness
Sound. Every `note_present` call sits in a branch that also pushes a finding for that file (unreadable-file, a non-MissingFence parse error, or the
document's own validation/wrong-directory findings or a loaded document), so a reference to a present-but-broken document is never silently clean:
the document's own finding blocks the gate. A file with no frontmatter fence and a nested file are not marked present, so references to them are
still reported missing (conservative). An unreadable file named like the id hides only the "missing" report, not the failure. Present ids use the file
stem, so a file whose declared id differs (okf-id-file-name) hides only the stem id. Acceptable.

## Whole-path code-span rule vs plan
The plan says the section "contains the bound path in a code span". The code requires the path to stand as a whole path inside the span (no identifier
character or `::` before, no identifier character or `::` after). This is stricter than the literal text but is the intended meaning (`{CASE}_extra` and
`other::{CASE}` are fixture cases) and is documented on `Binding::sections`. Task 5's renderer must read `Binding::sections` rather than re-matching.

## Findings

### REQUIRED
None.

### ADVISORY
1. (A1) The after-boundary `starts_with("::")` is not pinned by any test (mutant survives). Add a section case such as `` `<CASE>::inner` `` that must not substantiate.
2. (A2) Indented code blocks are treated as prose: a link inside a 4-space indented block yields a link edge, and a path inside an indented block counts as a
   code span. Links come from raf-okf `Body::links` (G3, so the fix belongs there); the span scan in `prose_of` could blank indented blocks too. CommonMark
   treats both as code. Low risk for repository documents, but it contradicts "links inside code are not links" for that one construct.
3. (A3) Missing explicit fixtures for the hostile cases verified above (duplicate id across files, anchor present only in another document, duplicated
   heading `-1` anchors, unterminated backtick run). They pass today; adding them would pin them.
4. (A4) The plan sentence for the section rule could be updated to say "as a whole path" so the specification and G10 renderer text match the code.
5. (A5) `Graph::nodes()`/`edges()` clone on every call; fine for now, a borrowing iterator would suit large repositories later.

VERDICT: PASS
