# Current status

The first review below blocked; the findings were fixed and the later recheck sections (at the end of this file) passed. Earlier sections are kept as written.

VERDICT: PASS

---

# Task 2 review: raf-okf body, splice, index

Reviewer: independent, read-only. Staged diff on top of 3f8697b (21 files, +3804/-78).

## Scope

Plan: user decision 4, section A, B3, B4, B5, completeness rows whose Task column includes 2, "Task 2" with its Docs line. Task 3 and Task 8 are out of scope except where this API would block them.

Read in full: `body/{mod,blocks,heading,link,marker,prose}.rs`, `index.rs`, the `document.rs`/`error.rs`/`reader.rs`/`lib.rs`/`limits.rs` diffs, all new tests, both doc diffs, and the docs_contract scanners (`fenced_lines`, `heading_text`, `github_anchor`, `anchors`, `sections`, `scan_markers`, `normalise_prose`, `scan_placeholders`, `definition_target`, `html_attribute_targets`, `link_targets`, `normalize`, `check_links`, `index_problem`) with their fixtures.

## Evidence

| Command | Result |
|---|---|
| `cargo test -p raf-okf --locked` | exit 0. body_structure 14, markers 8, links 19, index 8, splice 10, refusals 26, render 20, schema 5, no_io 3, body_robustness 1, repo_markdown 1, alloc_bound 1, doctest 1 |
| `cargo clippy -p raf-okf --all-targets --locked -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |
| `RUSTDOCFLAGS=-Dwarnings cargo doc -p raf-okf --no-deps --locked` | clean |
| `git diff --cached -U0 -- crates/raf-okf/src \| grep -E '\.ok\(\)\|unwrap_or\|else \{ *continue\|_ => \{\}\|\.unwrap\(\)\|expect\(\|unsafe\|allow\('` | no hits in src; no `allow`, `#[expect]` or `unsafe` anywhere in the staged diff |
| grep of added lines for plan IDs, `task N`, `decision`, `plan`, precedent names, tracker numbers, agent credit trailers and vendor addresses | no hits |
| docs_contract 117 / repo_gates 42 / `make reuse` / `make msrv` | taken from the orchestrator or not run by me (see Unchecked) |

## 1. Task 2 requirements against tests

Present, each with a neighbouring accepted case unless noted:

- Sections: slug, heading, span text, no-final-newline end: `sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading`, `a_section_that_ends_the_body_without_a_final_newline_reaches_the_end`.
- 3- vs 4-space headings: `a_three_space_heading_is_a_section_and_a_four_space_heading_is_not`.
- Anchors, repeats, `has_anchor`: `headings_of_every_level_carry_github_anchors_with_repeats_numbered`.
- Whole-text coordinates and frontmatter-less Markdown: `spans_are_in_whole_text_coordinates_when_frontmatter_precedes_the_body`, `markdown_without_frontmatter_is_the_whole_body`, `document_parse_builds_its_body_through_the_body_model`.
- Fences (4-backtick holding 3, tilde vs backtick, shorter run, quotes, lists, 3-space indent): `a_fence_closes_only_on_the_same_character_with_at_least_the_same_length`, `fences_are_recognised_indented_up_to_three_spaces_and_inside_quotes_and_lists`.
- Markers only in a fence give no blocks: `markers_inside_a_fence_produce_no_blocks`. `v[0](x)` in a fence: `links_inside_code_spans_of_any_length_and_fences_are_not_links`.
- Marker errors with exact lines: `every_misspelling_of_the_proposed_marker_is_refused_with_its_line` (13 spellings, line and column, exact marker accepted beside), `each_generated_marker_error_names_its_line` (orphan end, nested, unclosed, six malformed forms), `marker_errors_in_a_document_carry_whole_text_lines`.
- PROPOSED and generated accessors: `a_proposed_block_reports_slug_spans_content_and_section`, `two_markers_in_a_row_make_two_blocks`, `generated_blocks_report_kind_outer_and_content_spans`.
- Link forms, reference definitions, code spans of any length, comments, escapes, `[a] (b)`, Unicode space, nesting: the `links.rs` tests of those names. HTML: `html_attributes_are_read_with_spacing_case_and_unquoted_values`, `an_html_tag_split_across_lines_yields_its_attributes_with_their_lines`, `an_attribute_value_that_starts_on_the_next_line_is_read`, `tag_state_does_not_survive_a_blank_line_or_a_fence`, `the_tag_scan_spares_prose_autolinks_and_closed_tags`.
- Split links, decision 4: all four fixtures are present. `a_link_split_across_lines_is_recorded_and_not_read` (text split, destination split), `split_links_are_not_recorded_in_fences_code_spans_or_for_one_line_links` (fence, one-line, two adjacent one-line links), `a_split_link_does_not_hide_a_link_after_it`. The repository sweep is `repo_markdown`.
- `target_in`: `targets_resolve_against_the_document_path`, `targets_that_leave_the_root_or_are_empty_or_absolute_are_refused`. All three `LinkError` variants are tested, each beside an accepted neighbour.
- Nesting 64/65 for quotes, list markers, indentation and mixed, plus 100 000 markers on a 2 MiB stack: `block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused`, `a_hundred_thousand_quote_markers_are_refused_on_a_small_stack`, `a_thematic_break_is_not_a_run_of_list_markers`.
- Prose view: `prose_lines_skip_fences_and_comments_and_remove_code_spans`, `code_spans_of_any_backtick_length_hide_their_content`.
- Splice: all of section 4 below.
- Index: `a_generated_only_index_is_accepted_with_either_header_form`, `prose_outside_the_blocks_is_refused_with_its_line`, `an_index_with_no_block_or_an_unclosed_block_is_refused`, `indented_and_mid_line_markers_are_refused`, `a_header_with_unknown_tags_or_in_the_wrong_place_is_refused`, `an_empty_header_tag_is_refused`, `document_parse_of_index_text_fails_at_line_one`, `only_the_capability_index_path_is_the_index`.
- Robustness: `random_token_soup_never_panics_and_parsed_spans_lie_in_the_text` (40 000 inputs through Body, Document and Index; section-content identity edit).

Absent or thin (all non-blocking; see findings):

- `ConstructError::Body` has no test.
- `Document::replace_spans` is tested only for an overlap error. No Document-level test for a malformed result, although the plan line reads "malformed output refused while the document stays unchanged". The Body and Index versions exist.
- A `Document` edit that fails in `place_body` (size bound) has no test.
- `Body::parse`/`Index::parse` over 1 MiB has no Body-level test. Only the Task 1 Document one exists.
- `LinkPartTooLong` is pinned at 4000 accepted and 5000 refused, not at 4096/4097.

## 2. Parity with docs_contract scanners

Reproduced, with a test of the same shape as the old fixture: fence handling (stricter, CommonMark: char and length), ATX headings and GitHub anchors with `-N` repeats, H2 sections, reference definitions (angle target, `[^1]:` skipped, 3-space indent), inline links and images, all HTML fixtures (spacing around `=`, `data-href` not read, split tags, quote state, value on next line, autolinks, state reset at blank line or fence), PROPOSED spellings (lowercase, no space, extra spacing, `> >`, `- >`, bare), generated marker errors (orphan, nested, malformed, unclosed), index header forms and empty tag, indented and mid-line markers, code-span-aware placeholder view, `normalize`/escape semantics via `target_in`.

Differences from docs_contract. None is pinned by an existing docs_contract fixture, but Task 8 must decide each deliberately:

- P1. HTML comments: links, `href`/`src` and prose placeholders inside `<!-- -->` are no longer seen. The old scanners saw them: `check_links` and `scan_placeholders` never stripped comments. The plan says `prose_lines` skips comments, so this is mandated for placeholders. For links it is an implementer addition. It is stated on the capability page but not on the architecture page.
- P2. Frontmatter is outside the body. Old `check_links` and `scan_placeholders` ran over the whole file text, so `title: TBD` or a link in a frontmatter scalar was flagged. `Rule::Placeholder` over `prose_lines` (B6) will not see frontmatter values. Task 3 or 8 must scan frontmatter scalars, or this is a lost check.
- P3. `Link::target_in` returns `LinkError::Empty` for `[a]()`. The old checker skipped empty targets silently. Task 8 must map `Empty` to a skip to keep parity, or define it as a violation.
- P4. Marker and index errors are first-error `ParseError`. Old `scan_markers` reported every marker problem in a file alongside other violations. Plan-mandated (parse errors), but "three faults, three violations" cannot include a marker fault.
- P5. No section-text accessor. `Section::content()` is a whole-text `Span`, so a consumer slices `body.text()[start - body.span().bytes.start ..]`. Workable, awkward.
- P6. Old fence toggle treated a ```` ``` ```` line at any indentation as a fence. New code uses 0-3 spaces plus container indent, as the plan requires. Pinned by `fences_are_recognised_indented_up_to_three_spaces...`.
- P7. Index fences. `check_index` is fence-blind (as the old `index_problem` was), while the body scan is fence-aware. A fenced end marker inside a block therefore fails hard (`UnclosedGenerated`), never silently. Parity.

None of P1-P7 is a missing required behaviour of Task 2.

## 3. Implementer decisions

### (a) `LinkPartTooLong` (more than 4096 bytes of destination or title): REQUIRED, document it

- It is a new refusal of input CommonMark accepts. The Contract says the crate "adds no rule of its own", and the architecture page does not mention it.
- Its purpose is legitimate. Without a cap each `](` can scan to the end of the chunk, which is quadratic. The alternative of treating an over-long part as "not a link" would be the silent fallback the plan forbids. The refusal is the correct shape (typed, positioned, hard).
- It is documented only in the `Body::parse` rustdoc (`body/mod.rs:85`) and `ErrorKind` (`error.rs:135`). It is absent from:
  - the Contract "Resource bounds" list (`docs/capabilities/okf-profile.md:43-52`);
  - Constraints, which now reads "bounded to 1 MiB, 32 YAML levels and 64 Markdown block levels" (`:162`), so the sentence is incomplete;
  - Verification bound fixtures (`:196`);
  - the architecture page.
- The 64-level block bound has the same gap on the architecture page, but the plan mandates it and the Contract states it. The 4096 cap has neither.
- Required fix: add it as a stated bound to Resource bounds, Constraints and Verification, with a fixture at exactly 4096 (accepted) and 4097 (refused). Say that it is a resource bound on the scanner. Alternatively state it on the architecture page if the owner considers every refusal a profile fact. Do not remove the cap.
- Smaller points: the title scan can run across lines to the end of a paragraph chunk, so an unclosed `"` followed by more than 4096 bytes without another `"` in the same paragraph is refused. It is an extreme case, but the doc should say "destination or title, counted from its opening delimiter".

### (b) Setext headings and indented code blocks not modelled: ADVISORY

- Spec: the Contract and the plan model fenced code only ("fenced code blocks ... so markers inside a fence are never read"). The old checker modelled neither. No required behaviour is missing.
- Can a marker or link inside an indented code block be misread? Yes.
  - A well-formed `    > [!PROPOSED change:x]` is read as a `ProposedBlock`, and a malformed look-alike is a hard `MalformedProposed`.
  - `    [a](missing.md)` is read as a link and can be reported broken. Marker detection trims the line (`marker.rs` `raw.trim()`), links see the code-span-stripped line.
  - Both fail closed (false refusal or false block, never a silent accept). That is the same behaviour as docs_contract today, and the workaround is a fence.
- Indented-code handling is partly there. A 4+ space fence opener is not a fence (`opens_fence`), a 4+ space heading is not a heading, and a 4+ space reference definition is not a definition (`link.rs:481`). Only markers and inline links ignore the indent, so the model is inconsistent.
- Setext: `Title\n===` and `Title\n---` are never headings, so no section or anchor. A link to a setext anchor is reported broken (fail closed). `---` under a paragraph is not a setext H2 here.
- Recommendation: add one sentence to Contract "Body model" saying that headings are ATX and that code is fenced code only, so indented code and setext headings are not recognised. That makes the limitation a stated contract rather than a surprise. A test pinning `    [a](b.md)` as a link would make it falsifiable.

## 4. Splice transactionality and coordinates

- `Body::replace_spans` (`body/mod.rs:291`) builds the new text and the rebuilt model in locals and assigns `*self` only after `parse_at` succeeds. No error path mutates.
  - Checks, in order: reversed span, before the body, after the body, `is_char_boundary` for both ends, overlap against the previous end.
  - Sorting by `(start, end)` makes out-of-order input work, and two insertions at one point are allowed. A zero-width insertion inside a replaced range is `Overlap`. I found no panicking index: `offset - base` is guarded by `start >= base`, and `self.text[copied..start - base]` by `start >= previous_end >= copied + base`.
- `Document::replace_spans` clones the body, edits the clone, re-places it at the render origin, and assigns both body and `closing_newline` only at the end. Transactional.
- `Index::replace_spans` clones the body, edits it, runs `check_index` on the result, and assigns only on success. Transactional.
- Coordinates. A parsed `Document` and `Index` report spans in the coordinates of the text that was parsed, which differs from `render()` when the writer reorders keys. After `Document::replace_spans`, and for `Document::new`, they are in `render()` coordinates (`body/mod.rs` `Body::span` docs, tests `a_document_delegates_and_renders_the_edit...`, `a_second_call_uses_spans_read_from_the_rebuilt_model`). An index renders its own text, so parse, render and spans always agree. Consistent and documented. A parsed-but-reordered document keeping input coordinates is a documented trap, not a defect.
- Each `SpliceError` variant is tested:

| Variant | Test | Unchanged asserted | Accepted neighbour |
|---|---|---|---|
| `OutOfBody` (before, after, reversed) | `spans_outside_the_body_or_inside_a_character_are_refused` | yes | edit of a whole character |
| `NotCharBoundary` | same | yes | same |
| `Overlap` | `overlapping_spans_are_refused_and_the_body_is_unchanged` (Body); `a_document_delegates...` (Document) | yes, both | adjacent and same-point insertions |
| `Reparse` | `a_malformed_result_is_refused_and_the_body_is_unchanged` (Body, `OrphanGeneratedEnd`); `the_catalogue_of_an_index_is_replaced_through_the_index` (Index, `IndexText`) | yes | the valid edits in the same tests |

  `Reparse` for `Document` is untested (finding N3).

## 5. Cross-cutting

- No `.ok()`, `unwrap_or*`, `.unwrap()`, `expect(` or `let ... else { continue }` in `body/` or `index.rs`.
- `unwrap_or`-like constructs: `heading.rs:163 byte_at` and `blocks.rs:304 depth_overflow` use `match ... None =>` fallbacks (`body_len`, offset `0`). Both are total by construction: the first covers the end-of-body index, and the second is reached only with `offsets.len() >= 1` because `depth > MAX` implies a marker on the line. Judged reasoned, not error-swallowing. A tighter form would make the invariant explicit but it is not a defect.
- `_ => {}`: none. `marker.rs:218 (Some(_) | None, _)` is an exhaustive error arm.
- `Some(_)`-style skips: `blocks.rs` `scan`, `if let Some(block) = fences.last_mut()` cannot fail because a fence is pushed before `open` is set. Fine.
- Slice indexing, read line by line. Every `[a..b]` is bounded by construction (ASCII-delimited offsets, lengths checked, `get` for lookahead). I found no input-reachable panic. The soup test exercises 40 000 inputs. Backslash skipping (`j += 2`) can land mid-character, but the following comparisons are against ASCII bytes, so slicing always happens at ASCII boundaries.
- No `unsafe`, no `allow`. `no_io_in_source` passes.
- Duplication (non-blocking): `is_kebab` and the `GENERATED_OPEN`/`GENERATED_END` constants appear in both `index.rs` and `body/marker.rs`. `marker::is_kebab` is already `pub(crate)`. If the two copies drift, the index and body would disagree on what a kind is.
- No process or plan labels or precedent names in staged files. No attribution text.
- Docs accuracy: two defects, F2 below, and F1 (the missing bound). Everything else checked in the capability diff matches the code: split links "not read as a link and recorded", `Index::replace_spans`, `is_capability_index`, "`Document::parse` fails on it at line 1", comments excluded. The architecture sentence reads exactly "A link split across lines is a violation." and the repository has no split link (`repo_markdown`).

## Findings

Blocking:

- F1 (blocking). `LinkPartTooLong` is an unstated rule. Fix: `docs/capabilities/okf-profile.md:43-52`, `:162`, `:196` (see 3a). Code: `body/link.rs:18`, checks at `:418`, `:445`, `:465`.
- F2 (blocking, small). `ConstructError::Body` documents "The position is in the text the document renders to" (`error.rs:359`), but `Document::new` produces the error from `Body::parse_at(&body_text, Origin::START)` (`document.rs:77`) before it knows the head length. Marker and nesting errors are therefore positioned relative to the body, line 1 being the first body line. Only a size failure in `place_body` is in render coordinates. Either parse after computing the head (as `place_body` does), or reword the docs. Add a test, which is also N1.

Non-blocking:

- N1. `ConstructError::Body` is untested (`Document::new` with a malformed marker, plus the accepted neighbour).
- N2. Quote-prefixed continuation lines are not unwrapped for link scanning. `> [a](\n> b.md)` is neither a link nor a recorded split link, so decision 4 does not catch it there. `> [text\n> more](a.md)` is caught (tested). Conversely, a stray `[` in one list item and `](` in the next item without a blank line joins as a split link (false positive). Rare; the paragraph chunk does not break at list items or quote boundaries (`link.rs:252-281`).
- N3. Test gaps: Document-level malformed-result refusal, `Document::replace_spans` failing in `place_body` (size), Body/Index over 1 MiB, the 4096/4097 boundary.
- N4. P1-P5 above. Task 8 must: skip `LinkError::Empty` or call it a violation, scan frontmatter scalars for placeholders itself, accept that comment contents are no longer link-checked, and slice section text via the body base offset. Optionally add `Body::section_text(&Section) -> &str`.
- N5. Architecture page does not state that links inside code spans, fences and comments are not links, nor the 64-level bound.
- N6. `Index::parse` also builds the full body model, so a >4096-byte link inside a generated block refuses the index. Consistent, but undocumented on the Index paragraph.
- N7. Duplicate `is_kebab` and marker constants (see section 5).
- N8. Worst-case cost of `parse_inline` (up to 4096 bytes per `](`, for example many `[](` before a space) is bounded but I did not measure it. The soup test and `alloc_bound` do not target it.

## Unchecked

- docs_contract (117) and repo_gates (42) over the staged files, `make reuse`, `make msrv`: not run by me, relied on the orchestrator's report.
- Task 8 migration itself, and performance of adversarial link inputs (N8): not exercised, per the time box.
- Behaviour of the scanners on every tracked Markdown file beyond what `repo_markdown` asserts (no split link, no marker error).

VERDICT: BLOCKED

## Recheck

Evidence: `cargo test -p raf-okf --locked` exit 0 (body_structure 20, links 19, index 9, splice 12, all others unchanged, 0 failed). `cargo fmt --all --check` clean. `cargo clippy -p raf-okf --all-targets --locked -- -D warnings` FAILS (below). Read the new text in `link.rs` (`strip_quotes`, flush), `document.rs` (`body_origin`, `new`), `Body::section_text`, `index.rs` imports, both doc diffs, and the new tests.

| Item | Status | Evidence |
|---|---|---|
| F1 `LinkPartTooLong` documented | DISCHARGED | Capability Resource bounds (new bullet, says it bounds the scanner and that a longer part is a positioned error, never a silent non-link), Constraints sentence now lists 4096 bytes, Verification lists 4097 refused and 4096 accepted, architecture link rule states it. Test `a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused` pins bare, angle and the three title forms at 4096 accepted and 4097 refused, with lines. |
| F2 `Document::new` coordinates | DISCHARGED | `Document::new` now computes `body_origin()` from the rendered head and parses the body at it (`document.rs`), after the size check. Test `a_document_built_with_a_malformed_body_reports_the_line_in_the_rendered_text`: marker error at line 5 with byte offset, nesting at line 6, 64-level and valid neighbours accepted. The `ConstructError::Body` doc is now true. |
| N2 split links in quotes | DISCHARGED | `strip_quotes` removes up to 3-space-indented leading `>` markers per line before chunking. Test `a_link_split_across_lines_inside_a_block_quote_is_recorded` covers `> [a](\n> b.md)`, nested quotes, images, with one-line quoted links accepted. The list-item false-join case (stray `[` in one item, `](` in the next) is unchanged; it stays a rare non-blocking edge. |
| N7 duplicate `is_kebab` and markers | DISCHARGED | `index.rs` imports `GENERATED_OPEN`, `GENERATED_END`, `is_kebab` from `body`. |
| 3(b) setext and indented code | DISCHARGED | Constraints sentence added (only ATX and fenced code modelled; indented code is ordinary text). Tests `only_atx_headings_are_headings_and_setext_headings_are_not` and `indented_code_is_ordinary_text_so_links_and_markers_in_it_count` pin it beside the fenced neighbour. |
| Section text | DISCHARGED | `Body::section_text(&Section) -> Option<&str>` uses checked arithmetic and `get`; test covers a document body and a foreign section (`None`). |
| Missing-test gaps | DISCHARGED | `ConstructError::Body` (above); Document `Reparse` on the size bound (`splice.rs`, fits-exactly neighbour at exactly 1 MiB); Body exact and over 1 MiB (`a_body_of_exactly_one_mebibyte...`); Index exact and over 1 MiB (`index.rs`). |
| P1-P5 for Tasks 3 and 8 | Recorded in the plan per the coordinator; not re-verified here. |

New finding:

- R1 (blocking, trivial). The clippy gate fails: `clippy::cloned_ref_to_slice_refs` at `crates/raf-okf/tests/links.rs:180` and `:185` (`[at.clone()]` should be `std::slice::from_ref(&at)`). `-D warnings` over `--all-targets` is a per-task gate and the first failure stops the other targets, so run clippy again after the fix and confirm nothing else surfaces.

All earlier findings are discharged. The one remaining defect is the clippy gate failure.

VERDICT: BLOCKED

## Recheck 2

R1 DISCHARGED. `crates/raf-okf/tests/links.rs:180` and `:185` now use `std::slice::from_ref(&at)`. I re-ran `cargo clippy --workspace --all-targets --locked -- -D warnings` (exit 0) and `cargo test -p raf-okf --locked` (no failures). The worktree equals the staged content. No findings remain open.

VERDICT: PASS
