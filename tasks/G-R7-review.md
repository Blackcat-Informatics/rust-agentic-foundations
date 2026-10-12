# G-R7 review: document-contract hardening

## Scope

Unstaged diff in tests/docs_contract.rs, tests/support/mod.rs, docs/architecture/okf-profile.md and AGENTS.md. Spec: remediation-plan.md R7 and its Check list. Finding source: reviews/quality-review.md (HIGH-1, MEDIUM-1..4, LOW-2/3/5/6/7). Carried items A2 (G-R1) and A-7 (G-R3). No file in the repo was edited, staged or committed; probes ran in a scratch copy (/tmp/rv/scratch, fresh `git init` + `git add -A`, separate target dir).

## Evidence

- `make clippy fmt test msrv` rc 0 (clippy `-D warnings` clean; msrv `cargo check` on 1.98.1 passes). `uvx reuse@6.2.0 lint`: compliant, 79/79. `git hook run pre-commit` rc 0. Worktree status unchanged afterwards.
- docs_contract: 95 passed (46 original + 49 new). Count is exactly as R7 requires.
- WEAKENED TESTS: all 46 original test names still exist (name diff `comm`: none removed or renamed). Removed diff lines are the old `unquote`, `check_items`, state-to-body block and `>= 1` guards, each replaced by a stricter version. Only `accepts_moved_outcome` and the `capability()` fixture were edited among tests, and both edits are sound: the moved target must now exist (new Q-L2 rule), so the test adds `other-work`, and its expectation (no violations) is unchanged; `capability()` now injects `body` into the Purpose section (the PROPOSED-in-a-section rule) and gives a `delivered` fixture one code and one test binding (G12). The real-tree `>= 1` guards are now unconditional and `check_inventory` is run on the real tree.
- Probe results in scratch (rc, first violation):
  - 7 (remove cap + change, links repointed): rc 101, "the document is missing".
  - 9 (remove docs/capabilities and work/changes): rc 101, inventory missing.
  - 18 (PROPOSED in designed doc): rc 101, "state designed must have no PROPOSED blocks".
  - 19 (`[!proposed`, `>[!PROPOSED`): rc 101, "malformed PROPOSED marker".
  - 22 (`TBD`): rc 101 placeholder; (`<!-- later -->`): rc 101 "section `Usage` is empty"; (`Usage will be documented later.`): rc 0, see ADV-2.
  - 24 (`moved:change:ghost`): rc 101. 25 (.txt, .MD, symlink in docs/capabilities): rc 101 each. 26 (Capability in docs/other): rc 101.
  - 29 (unterminated `"abc`, `Built-in: census`, `description: >`, `title: >`): rc 101 each, with named error.
  - 31 (designed `code: [foo::Bar]`, also considered `examples: [x]`): rc 101 "must have empty bindings".
  - 32 (reference definition, `<a href="">`, `<img src="">`, `<a href=''>`, indented/titled reference definitions): rc 101 each. Two-line link rc 0 (documented as unsupported). 34 (.github template broken link): rc 101.
  - Inventory: slug renamed consistently everywhere: rc 101 ("document is missing", "not in the inventory table"). Removed change record, state flip, delivered flip, dropped `affects` line (G6), placeholder in frontmatter: all rc 101.
  - New-rule evasions that are correctly ignored: PROPOSED marker, link and placeholder inside a fenced block (rc 0); link and href inside inline code (rc 0). `TBD` in a list item, in bold, in parens, in a link text: caught.
  - Scalar parser: `"a" junk`, `*alias`, `@x`, `%x`, backtick, `!tag`, `&anc`, `|`, `|-`, `>-`, `'a' 'b'`, `"a""b"`, `[a`, `{a: b}` rejected; `a #c`, `a#b`, `"a # b"`, `"a" # c`, `'it''s'` accepted. Correct.
- FIFO fixture: `refuses_non_regular_files_instead_of_blocking` uses `mkfifo` with `.expect`/`assert!`, so a missing mkfifo is a hard failure, not a skip. A mutation removing the pre-open `symlink_metadata` regular-file check makes it fail after the 10 s timeout (observed rc 101, 10.9 s). Symlink and plain-file arms assert too. Only `#[cfg(unix)]` gating (acceptable; the project targets Linux).
- Carried items: A2 closed (okf-profile "enforces today" now names empty-bindings and the delivered rule). A-7 half closed: FIFO and symlink refused before `open`, type rechecked on the handle (see ADV-4).
- okf-profile "enforces today" text vs code: accurate for keys, values, sections, markers, links, inventory, binding lists by state and the delivered code+test rule; it says binding resolution waits for the source index (true). Slight overclaim: "No prose carries a placeholder token" (ADV-1). AGENTS.md section 6 sentence names `INVENTORY` in tests/docs_contract.rs, the same-commit edit rule and `raf_workplan::Gate`; matches the code and the doc comment.

## New tests as observed (`cargo test --test docs_contract -- --list`, 95 total; the 49 not in HEAD)

- accepts_a_generated_capability_index
- accepts_comments_after_scalars
- accepts_resolving_reference_and_attribute_links
- comment_stripping_keeps_visible_text
- inventory_accepts_a_matching_tree
- inventory_rejects_a_document_the_table_does_not_list
- inventory_rejects_a_missing_architecture_document
- inventory_rejects_a_missing_capability
- inventory_rejects_a_missing_change_record
- inventory_rejects_a_state_change
- inventory_rejects_an_id_change
- inventory_table_has_the_documented_shape
- placeholder_scan_spares_code_and_longer_words
- refuses_non_regular_files_instead_of_blocking
- rejects_bad_affects_shapes
- rejects_bad_changes_list
- rejects_bad_depends_shapes
- rejects_bad_id_prefix_or_slug
- rejects_bad_scalars_in_every_value_position
- rejects_bindings_before_delivery
- rejects_block_scalar_indicator_value
- rejects_broken_href_and_src_attributes
- rejects_broken_link_outside_the_document_directories
- rejects_broken_reference_definition
- rejects_delivered_without_code_or_test_binding
- rejects_designed_with_proposed_block
- rejects_duplicate_section
- rejects_empty_or_non_scalar_title
- rejects_frontmatter_shape_errors
- rejects_generated_marker_errors
- rejects_item_shape_errors
- rejects_leading_indicator_character
- rejects_lowercase_proposed_marker
- rejects_missing_type_key
- rejects_moved_outcome_naming_a_missing_change
- rejects_non_markdown_file_in_a_document_directory
- rejects_non_scalar_contract_effect
- rejects_non_scalar_state_and_scalar_bindings
- rejects_placeholder_tokens
- rejects_proposed_block_outside_a_section
- rejects_proposed_change_that_does_not_list_the_capability
- rejects_proposed_marker_with_extra_spacing_or_nesting
- rejects_proposed_marker_without_space
- rejects_proposed_section_missing_from_affects
- rejects_sections_left_empty_by_comments
- rejects_self_and_three_node_dependency_cycles
- rejects_typed_document_outside_its_directory
- rejects_unquoted_colon_space
- rejects_unterminated_quote

## Findings

### Required

- **REQ-1. A hand-written capability index can forge the "generated" marker.** `check_directories` (tests/docs_contract.rs:1432) treats `docs/capabilities/index.md` as generated when its text merely `contains("<!-- raf:generated:")`. The implementer also made `check_document` skip the "missing frontmatter" error for this path (:946) and exempted it from the inventory (:1528). Before R7 the missing-frontmatter error rejected any frontmatter-less index, so a forged marker was incidentally refused; now it is accepted. Verified in scratch (all rc 0, check clean):
  - `# Hand written\n\nSome prose.\n<!-- raf:generated:x -->\n<!-- raf:generated:end -->\n`
  - the marker only inside inline code (`` `<!-- raf:generated:x -->` ``), which is not even a marker
  - `hand prose only <!-- raf:generated:x --> foo` (mid-line, unclosed text accepted)
  The profile and A10 say the index "carries only generated blocks written by `docs sync`" and a hand-written index is not allowed. The test does not enforce "only". No test covers a forged index (`rejects_hand_written_capability_index` uses `# Index\n` with no marker, and still passes; it is not weakened). Accepting a generated index is consistent with A10 and okf-profile.md, and with the 49-test `accepts_a_generated_capability_index` fixture; the defect is that the marker is a substring test. Fix: for this path, require every non-blank line to lie inside a well-formed generated block (a line that is exactly an open marker, content, a line that is exactly the end marker), reject any other line, and add a negative fixture for prose outside blocks and for an in-line or code-span marker. `scan_markers` already catches unbalanced markers; only the "prose outside blocks" part is missing.

### Advisory

- **ADV-1. Placeholder scan is case-sensitive and spelling-exact.** `tbd`, `Tbd`, `xxx`, `T.B.D.`, `to  be  written` (two spaces) and a `to be`/`written` split across lines all pass (scratch rc 0); lowercase `todo:` would too. The real tree has no lowercase hit (grep of docs, work and the root Markdown), so making TBD/TODO/FIXME/XXX case-insensitive with word boundaries and collapsing whitespace in "to be written" costs nothing. The profile sentence "No prose carries a placeholder token" is slightly broader than the code. The required R7 forms (uppercase tokens, "to be written", empty-after-comments) are met.
- **ADV-2. `Usage will be documented later.` still passes** (quality-review mutation 22c). R7 only asks for the token list and empty sections, so this is not a regression; a short phrase list ("will be documented", "to be documented") would close it.
- **ADV-3. `<a href=./x.md>` (unquoted attribute) is not read** (rc 0), and a link split over two lines is not read (documented as single-line). The profile says `href` and `src` of inline HTML are covered; add "quoted" to the profile sentence or handle the unquoted form.
- **ADV-4. A-7 residual TOCTOU.** `read_regular` does `symlink_metadata`, then a blocking `open`, then rechecks the handle. A FIFO swapped in after the first check would still block in `open`. Under the stated trusted-input model this is low priority; the fix needs `O_NONBLOCK`, which is not available std-only without a raw constant, so recording the residual in the doc comment is enough. Side effect worth knowing: `read_tracked` is also used by tests/repo_gates.rs, so a tracked symlink anywhere would now panic those gates (none today: 0 mode-120000 entries).
- **ADV-5. Cosmetics in okf-profile.md.** The new sentences were inserted into already-wrapped paragraphs without re-wrapping ("The marker is written exactly as" and "Anchors follow the GitHub rule: lowercase, spaces" are over-long lines). Rewrap.
- **ADV-6. FIFO fixture hygiene.** The scratch directory is named by pid and not removed on assertion failure; a reused pid after a failed run makes `mkfifo` fail with "File exists" (a loud failure, not a silent pass). Use a unique per-run dir or remove first.

## Verdict

Everything in the R7 Check list is met and every probe fails with the named rule, but the hand-written index guard (REQ-1) can be bypassed with a one-line marker, in the one place the implementer changed behaviour unasked. Fix REQ-1 and re-run the suite.

VERDICT: BLOCKED

## Recheck

Scope: the coordinator's fixes for REQ-1 and ADV-1, 3, 4, 5, 6 (ADV-2 accepted as out of scope). Repo still unstaged and unedited by me; probes ran in a scratch copy with a fresh `git init` and `git add -A`.

### Evidence

- `make clippy fmt test msrv` rc 0 (clippy `-D warnings` clean, msrv check on 1.98.1 passes). `uvx reuse@6.2.0 lint` compliant. `git hook run pre-commit` rc 0.
- docs_contract: 103 tests (95 + 8). All 46 original names are still present (none removed or renamed). The 8 added since my first review: `accepts_a_generated_index_with_an_spdx_header`, `placeholder_scan_spares_longer_words_and_dotted_prose`, `refuses_a_file_swapped_in_after_inspection`, `rejects_an_index_whose_marker_is_only_in_a_code_span`, `rejects_an_index_with_a_mid_line_marker`, `rejects_an_index_with_prose_outside_the_blocks`, `rejects_broken_unquoted_attribute_values`, `rejects_placeholder_spellings`.
- No newly weakened behaviour found. The earlier probes (7, 9, 18, 19, 22, 24, 25, 26, 29, 31, 32, 34) were not re-run in full; the real-tree and fixture suite still pass, and the changed code is the index guard, placeholder scan, attribute scan and `read_regular`.

### REQ-1 (hand-written index forgery): closed

`index_problem` accepts only an optional leading SPDX comment plus well-formed generated blocks, each marker alone at the start of its line, at least one block. Forge attempts against the real-tree test (rc 101 unless noted):
- prose before, after the header, after a block, or between blank lines: rejected.
- prose with a marker pair: rejected. Marker only in a code span or mid-line: rejected (fixtures).
- indented open marker, indented end marker: rejected. Trailing space on the marker: rejected. Uppercase kind: rejected.
- unclosed block, nested open, stray end, `end` used as an opener, no block at all, header only, empty file: rejected.
- two SPDX headers, SPDX header after a block, a header comment that contains non-SPDX prose, YAML frontmatter: rejected.
- Genuine index (header plus block, or block only), and CRLF line endings: accepted (rc 0), as intended.
- Mutations of the guard: dropping "no block", or "text outside a block", fails named fixtures.

Residual, advisory:
- ADV-7. A single-line `<!-- SPDX-anything you like -->` between blocks is skipped, so it is a one-line prose channel (rc 0). Require `SPDX-` followed by a known tag (`FileCopyrightText:` or `License-Identifier:`).
- ADV-8. Prose inside an open block is accepted. This is inherent (block content cannot be verified until `docs sync` exists); note it in the doc comment.
- ADV-9. No fixture pins the "marker must start its own line" rule for an indented marker: changing `line.starts_with(GENERATED_OPEN)` to `line.trim().starts_with(...)` leaves all 103 tests green (mutation rc 0), although the real code does reject the indented case. Add an indented-marker negative fixture.

### Advisories

- ADV-1 (placeholders): closed for the listed forms. `tbd`, `Tbd`, `T.B.D.`, `t.b.d`, `xxx`, `todo:`, `FixMe`, `to  be  written`, tab-separated, `TO BE WRITTEN`, `- todo`, `tbd-free`, `the todo.` are rejected; `a todolist`, `metadata`, `xxxx`, `x.tbd.y` are spared. Still passing: `T B D` and `to be`/`written` split across lines (acceptable, not claimed).
- ADV-3 (unquoted attributes): closed. `href=./x.md`, `src=./x.png`, `HREF=`, single-quoted and a line-split tag are checked; `href=https://...` is spared. Residual: `href = "./nope.md"` with spaces around `=` is not read (rc 0), and `data-href=` is read (a harmless false positive). Advisory ADV-10.
- ADV-4 (TOCTOU): partly closed. Symlinks are now refused with the path named (mutation fails the FIFO/symlink fixture) and the handle is compared by device and inode, which catches a swap to another regular file or a different object. A FIFO swapped in between `symlink_metadata` and `open` would still block in `open`; this is not closable std-only and is acceptable under the trusted-input model.
- ADV-5 (rewrap), ADV-6 (drop guard, unique name): closed as far as the diff shows; clippy, fmt and the suite pass.
- ADV-2: accepted as out of scope.

### Verdict

REQ-1 is fixed and held against every forgery attempt I could construct. The remaining items (ADV-7 to ADV-10) are advisory and do not affect the R7 contract.

VERDICT: PASS

## Recheck 2

Scope: ADV-7, ADV-9, ADV-10 changes in tests/docs_contract.rs. Evidence: `cargo test --test docs_contract` 107 passed; `make clippy` rc 0. Probes in a scratch copy (fresh `git init`, `git add -A`), README or docs/capabilities/index.md mutated, real-tree test run.

### ADV-7 and ADV-9: closed

- Index header: single-line `SPDX-FileCopyrightText` and `SPDX-License-Identifier` comments and the multi-line form are accepted before the first block. Prose comment (`<!-- SPDX-hand written -->`), unknown tag (`SPDX-Foo:`), an SPDX line after a block, and a multi-line header with an unknown tag are rejected. An indented marker is rejected and now has a fixture.
- Residual (advisory, ADV-11): the tag value is not checked. `<!-- SPDX-License-Identifier: -->` and `<!-- SPDX-FileCopyrightText: <a sentence of prose> -->` pass, and an SPDX line inside an open block passes (inherent). The prose channel is narrowed to the value of two tag lines.

### ADV-10: whitespace around `=` and the name-character check work, but the open-tag scoping loses links (REQUIRED)

`attribute_targets` decides "inside a tag" per line: `head.rfind('<') > head.rfind('>')`, where `head` is the text before the attribute on the same line. Probes (a broken target in each; rc 0 means unread):
- `<img` newline `src="./nope.png">`: **rc 0, unread**. The `src=` line has no `<`, so it is not "inside a tag".
- `<img` / `  src="./nope.png"` / `  alt="x">` (the common multi-line form): **rc 0, unread**.
- `<a` newline `class="x" href="./nope.md">`, and `<div>` / `<a` / `href="./nope.md">`: **rc 0, unread**.
- `<a href=` newline `"./nope.md">`: rc 0, unread (value on the next line; arguably out of scope).
- `<a title="x>y" href="./nope.md">x</a>`: **rc 0, unread**. A `>` inside a quoted attribute earlier on the same line ends the "tag" for the scan. Same for `title="a > b" href=./nope.md`, and `title='>' href='./nope.md'`. These are single-line, so they are unread even under the profile's single-line rule.
- Correctly read: `<a href="./nope.md">`, `<a href = "./nope.md">`, two attributes on one line (`<a href=...><img src=...>`), unquoted values. Correctly spared: `Use a > b and href="./nope.md" in prose` (the prose false positive that was the aim), `data-href=`, `xhref=`. Residual false positive: `1 < 2 and src="./nope.md" > 3` is still read (rc 101); harmless.

Regression note: in the previous round the line-split form `<a` / `href="./n.md">` was read (rc 101); it now passes. Broken links that the gate caught before are now missed, so this is a loosened check, not only a missed new case. The two-line markdown link is the documented single-line exception; HTML tags are not.

Fix: track tag state across lines rather than per line. Scan the whole text for `<` followed by a letter, then walk to the closing `>` while honouring quotes (`"` and `'`), collecting `href`/`src` from inside that span (it may cross lines), and report the line of the attribute. Alternatively drop the in-tag requirement and rely on the name-character check plus a preceding whitespace requirement, accepting the prose false positive in exchange for no missed links. Add negative fixtures: split `<img` / `src=`, and `title="x>y" href=`. If a split tag is declared out of scope instead, state it in okf-profile.md ("the tag sits on one line") and still fix the quoted `>` case.

### Verdict

The index guard changes are sound. The attribute scan change reintroduces unread broken HTML links (split tags, and `>` inside a quoted attribute), so ADV-10 is not safely closed.

VERDICT: BLOCKED

## Recheck 3

Scope: the cross-line `TagState` in `check_links` / `html_attribute_targets`, and the ADV-11 SPDX value requirement. Evidence: `cargo test --test docs_contract` 110 passed; `make clippy msrv` rc 0. Probes in a scratch copy (fresh `git init`, `git add -A`), README or docs/capabilities/index.md mutated, real-tree test run; rc 101 means the broken target was reported.

### Closed

- Recheck 2 findings: `<img` / `src=` on the next line, the 3-line `<img` form, `<a` / `class=… href=…`, `<div>` / `<a` / `href=`, `title="x>y" href=`, `title="a > b" href=./x`, `title='>' href=`, a multi-line quoted title, spacing around `=`, `<A HREF=`, tab-separated, single-quoted, unquoted with trailing `/`, `<br/><img src=…/>`: all rc 101. The previously regressed split-tag case is read again.
- Correctly spared (rc 0): `Use a > b and href="…" in prose`, `data-href=`, `xhref=`, tags inside code spans (`` `<a href="./nope.md">` `` and `` `<img src=./nope.png>` ``), and autolinks (`<https://…>`, `<mailto:…>` followed by a broken `<img>`: the img is still read).
- Your three questions:
  - Unclosed `<a` running to end of file: markdown links are NOT swallowed. Inline links and reference definitions are taken from `link_targets`, independent of tag state (`<a` / blank / `[x](./nope.md)` is reported, also with a following broken `<img>`).
  - Tag in a code span: ignored, as above. A lone `` `<a` `` followed by a markdown link and a real broken `<img>`: both reported.
  - `a<b and [x](./nope.md)`: the markdown link is reported; a broken `<img>` two lines later is also reported. `if a<b then` / `<img src=broken>` is reported.
- ADV-11: `SPDX-License-Identifier:` and `SPDX-FileCopyrightText:` with an empty value are rejected, in single-line and multi-line headers; valid values pass.
- Mutating the previous probes (7, 9, 18, 19, 22, 24, 25, 26, 29, 31, 32, 34 classes) was not repeated; this round touches only link scanning and the index header. No existing test was removed.

### Required

- **REQ-2. Prose `<letter` plus a stray quote hides later HTML links.** `a<b it's fine` followed by a blank line and `<img src="./nope.png">`: rc 0, the broken src is unread. The prose `<b` opens tag state, the apostrophe opens a quote, and both persist across blank lines and paragraphs until another `'` or `>` appears, which can be the rest of the file. Without a quote, `a<b` stays "in tag" until the next `>`, so later prose such as `src=./x` would be read as an attribute (false positive) and attributes after it are mis-scoped. Inline HTML cannot contain a blank line in CommonMark (a blank line ends the paragraph or HTML block), so the sound fix is to reset `TagState` (in_tag and quote) on every blank line, and at the start of each fence. Add a fixture: `a<b it's fine`, blank line, broken `<img src>`, expecting the link to be reported.

### Advisory

- ADV-12. `<a href=` with the value on the next line is not read (rc 0). It is a rare form; say so in okf-profile.md ("an attribute name and its value sit on one line") or handle it.
- ADV-13. A genuinely unterminated quote inside a real tag (`title="it's` with no closing quote) hides later tags in that paragraph. After REQ-2's blank-line reset this is bounded to the paragraph.

### Verdict

Everything asked in Recheck 2 and ADV-11 is fixed, markdown links cannot be hidden by tag state, and code spans and autolinks are handled. One silent-pass remains: tag/quote state leaking past a blank line from prose such as `a<b it's`.

VERDICT: BLOCKED

## Recheck 4

Scope: REQ-2 (TagState reset on blank and fence lines) and ADV-12 (`read_value`, value on the next line). Evidence: `cargo test --test docs_contract` 112 passed; `make clippy` rc 0. Probes in a scratch copy (fresh `git init`, `git add -A`); rc 101 means the broken target was reported.

### REQ-2: closed

- `a<b it's fine` / blank line / broken `<img src>`: rc 101 (was rc 0). Also reported after a fence line in between, after `a<b "q` (double quote) and after bare `a<b`.
- Tag state still carries within a paragraph: `<img` / `src=`, the 3-line `<img`, `<a title="x>y" href=`: rc 101.
- A broken `<img>` after a fenced block that follows prose `a<b it's` is reported; a tag inside a fence is not scanned (rc 0), as intended.
- A tag split by a blank line is not a tag in CommonMark, so `<a` / blank / `href="./nope.md">` and a quoted title spanning a blank line pass (rc 0). This follows the reset rule and is acceptable.
- Markdown link `[x](./nope.md)` still reported.

### ADV-12: closed

- `<a href=` / `"./nope.md"` and `<a href=` / `./nope.md>`: rc 101. Value on the next line to an https target or `data-href`: spared. `href=` / `"#nohead"`: reported as a broken anchor. Prose `Use a > b and href=…`: spared.
- Residual, advisory (ADV-14): the name and `=` split over lines (`href` / `=` / `"./nope.md"`) is not read (rc 0). Very rare; no action needed.

### Verdict

Both fixes hold against the neighbouring probes, with no new silent pass found in the link scan. All R7 findings (REQ-1, REQ-2) are closed. The tree is clean on clippy and the contract suite.

VERDICT: PASS
