# Task 8 review: current rationale checks

Scope: staged diff on 6b317fc (15 files). Probed in /tmp/t8rev with a throwaway probe test.

## Evidence so far
* Rules read against D10: all three rules, exact/level matching, closing `#`s and extra spaces handled
  (`##  Decision`, `## Decision ##`, tab) by the okf heading parser; `## Decision##` is text `Decision##` (CommonMark-correct).
* Unlinked pause: `present` (file stems) vs loaded ids. A present-but-unloaded doc always carries its own error finding
  (parse error, okf-id-file-name, wrong-directory), so check cannot be clean while paused; when everything loads, the sets are equal
  and the rule is live (stray no-fence file does not pause: probed).
* List item probes: `- `, `-`, `* [ ]`, bullets of only a code span / emoji / punctuation / empty comment => empty (reported);
  link-only, non-ASCII letters (e, CJK), nested, ordered, `* [ ] todo` => count.
* Real repo: `check` exit 0; both pages linked (census-normal-form -> normal-form, okf-profile -> okf-profile); no `## Decision` in either.
* docs preview: exit 2 for workplan-two-way-coverage (l.157), workplan-entity-history (l.112), workplan-prose-assessment (l.151),
  workplan-complete-gate (l.463): each a fenced block inside a PROPOSED quote; all other 17 change records exit 0 (incl. evidence-claims).
  `change complete` does not call preview (it counts unresolved blocks), so hand-resolution works; check accepts them today.

## Gates (all green)
raf-workplan tests debug (inside `cargo test --workspace`) and release; `cargo test --workspace`; docs_contract + repo_gates
(57 + 28); clippy `--workspace --all-targets -D warnings`; `cargo doc -Dwarnings`; `fmt --check`; `make reuse` (431/431),
`deny`, `msrv` (1.98.1), `issue-refs` (57 passed). `make docs-sync` run twice on a copy with the diff applied: tree hash unchanged
after both runs and equal to the staged tree (index regenerated, "Considered" table dropped because empty).
Release timing of the 100k-heading test: 0.84 s (debug whole file 9.8 s). Cost is linear: each conditions heading scans to the next
heading of its own level, ranges are disjoint, and prose lines are found with partition_point.

## Evidence by item
1. Rules vs D10: all three match. Probes (page): `- `, `-`, `* [ ]`, `- \`x\``, `- ...`, emoji, empty comment, quoted `> - x`, trailing
   whitespace: reported. `- 7`, `- e`, CJK, link-only, nested, ordered, `* [ ] todo`, lazy `Text\n- item`: counted. `##  Decision`,
   `## Decision ##`, tab, 3-space indent: heading matches; `    ## Decision`, `## **Decision**`, `> ## Decision`, comment: not
   headings. Duplicate `## Decision`: one finding at the first; second empty conditions heading: one finding at it. Conditions after
   the next h2 or h1: not part of the section. Explanation: conditions before the decision, at h2/h4, or in Verification: reported.
   Pause cannot hide a page forever: it holds only while a cap/change/arch file is present and not loaded, and every such file has
   its own error finding (parse error, `okf-id-file-name`, wrong-directory), so `check` is never clean while paused; with all loaded the
   sets are equal and the rule is live (kinds.rs case, probe with a stray no-fence file: still reported).
2. Fixtures: diffs only add links/pages (`valid`, `synced::unsynced`, generated.rs no-capability test, lib.rs doctest, docs_contract
   checkout with a second page, `move_document` repoints gamma for the wrong-directory cases). No assertion weakened; one test
   (`the_tree_is_walked_only_when_a_relative_link_needs_it`) removes the page and link so its premise (no relative link) still holds.
   The unlinked rule fires on the pre-change-style fixture (`unlinked()` = valid minus the link; kinds case + 2 more).
3. Repo: `cargo workplan check` exit 0; normal-form linked from census-normal-form, okf-profile linked from okf-profile and two others;
   neither page has `## Decision`/`## Conditions`.
4. Documents: current-rationale.md diff touches only front matter (state, bindings) and PROPOSED blocks. Counts verified:
   9 designed, 12 implementing, 0 delivered, 0 considered, 21 capabilities (verified per file; the extra `state: designed` my grep
   found at okf-profile.md:366 is quoted text, not front matter). Change record `change:workplan-current-rationale` open, `changes: [cap:workplan-current-rationale]`.
   Bindings: module + 14 integration tests + 2 unit tests resolve; `check` (which enforces the Coverage record) is clean, so every new
   public item and test is bound.
5. Fence-in-quote documents (all pre-existing, none touched by this diff; `check` accepts all of them; `docs preview` exit 2):
   * docs/capabilities/workplan-two-way-coverage.md line 157-162 (yaml fence, change:workplan-two-way-coverage)
   * docs/capabilities/workplan-entity-history.md line 112-116 (text fence, change:workplan-entity-history)
   * docs/capabilities/workplan-prose-assessment.md line 151-153 (text fence, change:workplan-prose-assessment)
   * docs/capabilities/workplan-complete-gate.md line 463-467 (text fence in the PROPOSED block at 459, change:workplan-complete-gate)
   All other 17 change records preview with exit 0 (current-rationale, evidence-claims, cli, generated-docs, document-model, reconcile,
   and the 11 census/okf/source records). `change complete` does not use preview (it counts unresolved blocks), so Task 11 resolves the
   four blocks by hand (plain text, fence kept); the preview limit then no longer applies. Not a Task 8 defect.
6. Mutants (in a copy): self-link counted KILLED (kinds, unlinked self-link case); level check dropped KILLED (rationale);
   fence skipping removed (raw-text list scan) KILLED (rationale); empty list accepted KILLED (kinds + rationale); order not required
   KILLED; unlinked pause removed KILLED (`no_page_is_reported_unlinked_while_...`). 6/6.
7. Hygiene: SPDX on both new files; no tracker numbers, process wording, attribution, `unsafe` or dependency changes in the diff.

## Findings

### REQUIRED
None.

### ADVISORY
1. Setext headings are invisible to the rules (the okf body model reads ATX headings only, so `Decision` over `---` is no decision,
   and a setext conditions or later heading ends no section). `check` gives no finding either. Pre-existing okf limitation; state it
   in the Contract ("ATX headings only") or add a plain rule against setext headings in architecture pages.
2. Small list-item misreads: an item that is only a code span (`- \`x\``) counts as empty, though the Contract says so; `* [x]` alone
   counts as text (the `x`); a 4-space-indented `    - code` line (indented code) and a line inside an HTML block count as items.
   Harmless for authored pages; the first is worth one sentence in the page that Task 10 writes.
3. Task 11 must hand-resolve the four fenced PROPOSED blocks above before it can use `docs preview` on those changes; add the list to
   the plan.
4. `has_unloaded_documents` compares file stems to header ids, correct only because a mismatch is itself an error finding
   (`okf-id-file-name`); a one-line comment saying so would keep it from being "fixed" later.

VERDICT: PASS
