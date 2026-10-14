# T8 review: docs_contract through raf-okf and raf-source

Reviewed: staged diff over 52155fa (AGENTS.md, Cargo.toml/lock, raf-okf usage_docs, deleted raf-source usage_docs,
okf-profile.md, tests/docs_contract.rs, tests/support/mod.rs). Read-only.

## Evidence
- `cargo test --test docs_contract`: 137 passed (re-run here). `cargo test -p raf-okf --test usage_docs` passes.
- Fixture comparison: a script extracted every `#[test]` body from HEAD and the staged file and diffed them. No test is
  missing except two renamed ones (`comment_stripping_keeps_visible_text` -> `comments_hide_text_but_not_the_text_beside_them`,
  `parses_quoted_and_block_forms` -> `accepts_quoted_and_block_forms`). 22 new tests. Every changed assertion was read.
- Scan of tests/docs_contract.rs for `.ok()`, `unwrap_or`, `filter_map(Result::ok)`, `else { continue }`, `_ => {}`: none
  swallow an error. The `=> {}` arms are exhaustive-match no-ops with named variants (L263, L462, L486, L616); `filter_map`
  at L886 and L3172 filter by prefix, not by error. No plan labels, task numbers or process wording found; the `.github/ISSUE_TEMPLATE`
  strings are a path fixture.
- Cargo.toml dev-dependencies carry path + version for both crates. `tests/repo_gates.rs` header "Only the standard library is used"
  is still true (it imports no raf crate); support/mod.rs header reworded and accurate.

## Judgments
1. Private scanning. No private YAML, fence, heading/anchor, marker or link extractor remains. Surviving non-trivial scanners:
   `scan_placeholders` + `normalise_prose` + `has_word` + `PLACEHOLDER_TOKENS` (L49-57, L139-196), applied to `Body::prose_lines`
   of frontmatter-less Markdown only (`read_plain`, L335); `scan_banned` (allowed by plan); `identity()` (inventory id/state via
   `Document::frontmatter().map()`, through raf_okf, fine). The placeholder scan is a second interpreter of a raf-okf rule
   (`body_rules::placeholders` is `pub(crate)`). The plan's Task 8 line names `prose_lines` as a docs_contract input, so it is
   plan-conformant; ADVISORY (finding 3): drift between the two token matchers is checked by no test.
2. Gate strength. All fixtures still assert a refusal of the same input. Needles changed only to raf-okf's wording; each
   remains specific. Weaker spots (non-blocking): `rejects_unclosed_generated_block` needle "is never closed" (L1488) is shorter than
   before but only that violation produces it; in `rejects_frontmatter_shape_errors` (L2578-2587) two cases share the needle
   "indentation must be a multiple of two spaces" so one message could satisfy either (the inputs differ but one needle cannot
   tell which rule fired); the `rejects_leading_indicator_character` needles got stricter (per character).
3. Parity decisions. HTML-comment links/placeholders: implemented and tested (L3409) and stated on the architecture page.
   `[a]()` violation: tested (L3390, three forms plus a code-span neighbour) and stated on the page. Split links: refused for
   README.md, docs/notes.md, .github/PULL_REQUEST_TEMPLATE.md, crates/tool/README.md and a profile document, reported once (L3334-3380).
   Marker errors per document: fixtures are per document (`assert_cases` runs each case alone; plain-Markdown fixture asserts exactly
   one violation, L3442). The page does not say "one marker error per document"; the plan required that only for fixtures.
4. Bindings. `repository_bindings_resolve_in_the_source_index` (L960) uses `Workspace::load(&DiskTree::new(&gate_root())?, &["crates"])`,
   no wrapper, and resolves code/tests/examples of every capability; the same `gate_root()` the hook points at the snapshot.
   Negative fixtures: missing path, macro-generated fn, all-lists-reported (L3073-3120) on a MemoryTree demo workspace; positive
   `demo::real`. Gap (finding 4): today every capability is `designed`, so the real-tree test resolves zero bindings and would
   also pass if `crates` loaded nothing; no real-tree positive symbol is asserted.
5. Doctest equality. One definition (L3123-3320); crate-level `usage_docs` doctest tests removed from raf-okf and the raf-source file
   deleted; raf-okf keeps the Examples-validates test, header says so. Compares Usage+Examples `rust*` blocks of both capability
   documents with the leading crate doctests positionally, checks info strings (`rust,no_run`), `std::fs` in run blocks, and the
   "compiled and not run" sentence. Negative fixtures: real-file one-line change for both pairs and synthetic cases. Crate blocks
   after the documented ones are not checked (documents say "leading"; acceptable).
6. Docs. Architecture paragraph is present tense and accurate (cross-document list matches the code: cycles, change references,
   open change record, inventory, links/anchors, layout, bindings). AGENTS.md accurate. docs_contract header accurate.
7. Cross-documents run only on documents that pass `Profile::validate` (`read_document` L318). Markers are recorded before
   validation so orphan-marker checks still run. A document failing validation hides its own cycle/dangling-depends/state checks
   and causes extra false "dangling depends" / "does not list" / "requires an open change record" noise elsewhere; the file is red
   either way, never silently green. Non-blocking (finding 5); not required.
8. See Evidence: clean.

## Findings
1. NON-BLOCKING, fix in this commit (trivial): tests/docs_contract.rs:585 (`check_capability_state`, `None =>` arm) emits the
   literal "PROPOSED block `add-alpha` outside a section", a fixture slug hard-coded in the checker. The arm is unreachable (a block
   outside a section fails `Profile::validate`, so the capability is never recorded) and `rejects_proposed_block_outside_a_section`
   (L2087) passes on raf-okf's identical message. Use `marker.slug`/`unreachable!`, or drop the `Option` handling.
2. NON-BLOCKING: shared needle in `rejects_frontmatter_shape_errors` L2578-2587 (see Judgment 2); give each case a distinct needle
   or a line number.
3. ADVISORY: placeholder scan outside the profile duplicates raf-okf's token rule. Preferred: a public raf_okf placeholder check
   (e.g. on `Body`) so docs_contract holds no matcher; at minimum a parity test that runs the same text through both paths.
   Not required for Task 8 because the plan names `prose_lines` for this use.
4. NON-BLOCKING: add one assertion that the real-tree workspace resolves a known symbol (e.g. a `raf_okf` item), so the binding test
   cannot pass vacuously before Task 9 delivers capabilities.
5. NON-BLOCKING: silent-skip note in Judgment 7; no action required.
6. NON-BLOCKING: architecture page does not state the one-marker-error-per-document behavior; consider one sentence.

No blocking finding.

VERDICT: PASS

## Recheck

Run: `cargo test --locked --test docs_contract` 137 passed; `cargo test --locked -p raf-okf` all targets pass (includes the new `placeholder_lines.rs`).

1. Placeholder rule from raf-okf. DISCHARGED. `Body::placeholder_lines()` (crates/raf-okf/src/body/mod.rs) is the single path:
   `check_placeholders` in Profile::validate now calls it, and docs_contract's `scan_placeholders` (tests/docs_contract.rs:304) calls it.
   `normalise_prose`, `has_word` and the private matcher are gone; `PLACEHOLDER_TOKENS` (L47) is used only by fixtures (L1742). The
   frontmatter scan shares `placeholders` through a `pub(crate)` re-export. New tests cover token+line, `T.B.D.`, longer words, code,
   spans and comments; the Contract in docs/capabilities/okf-profile.md documents the method.
2. Unreachable arm as `panic!`. DISCHARGED. L531 names the cap path and the block's slug (no fixture literal) and a comment states the
   invariant (validate yields no typed capability for a block outside a section). A panic is acceptable here: the gate is a test, the
   arm is an invariant violation of the checker itself, and the same file already panics on unmodelled outcomes and unreadable input
   rather than fall back. The user-visible violation still comes from raf-okf's rule.
3. Distinct needles in rejects_frontmatter_shape_errors. STILL-OPEN (non-blocking). The staged text still gives the
   `"  code: []\n    - stray\n"` case (L2527) and the `"  code: []\n      deep\n"` case (L2532) the same needle,
   "indentation must be a multiple of two spaces", so one message satisfies both. Give each its own needle or line number.
4. Real-tree binding test. DISCHARGED. L914-916 resolves `raf_okf::Document` and `raf_source::Workspace` (with `?`) before
   `check_bindings`, so a load that finds nothing fails.
5. Architecture page. DISCHARGED. New paragraph under the markers section states a document reports its first marker error and
   that a second fault shows after the first is fixed.

No new findings; finding 2 of the first review stays open as above (non-blocking). Note: the staged diff now also touches crates/raf-okf (src/body/mod.rs, src/profile/*.rs) and docs/capabilities/okf-profile.md,
all within the stated change.

VERDICT: PASS

## Orchestrator note (item 3)

The two `rejects_frontmatter_shape_errors` cases genuinely produce the same raf-okf error (a block opened under a key that has a value), so distinct needles are not possible; both needles now pin the full positioned message `line 11, column 3: indentation must be a multiple of two spaces`, so neither passes on an indentation error elsewhere. Verified by probing each case's actual output. docs_contract 137 passes.

VERDICT: PASS
