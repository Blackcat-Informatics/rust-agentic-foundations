# Delivery review: 097fc864

## Scope

- Range: `git diff 4369c9cc..097fc864`, one commit,
  `docs(agents): a delta that changes no behaviour needs no further independent read`.
- Files: `AGENTS.md` only, +5 -3, the review bullet in section 9, under "Changes"
  (lines 366-371).
- Requirement: issue 281, read verbatim from `.stage/delta-that-changes-no-behaviour-needs/issue.md`.
  The issue has no comments. Acceptance: "section 9 states both rules; no other rule changes."

## Evidence

- `git diff 4369c9cc..097fc864 --stat`: one file, `AGENTS.md`.
- `grep -n -i -E "exact head|merging diff|final check|independent|review|head that"` over
  `AGENTS.md` and `CLAUDE.md`. Besides the amended bullet, this matches lines 361
  ("reviewed pull request"), 232 (SOLID "in review"), 417 (read-only review runs
  in parallel) and 418-419 ("The final check of a change that is about to merge is a reading of
  the merging diff itself, after the independent review and not in place of it").
- `prose-check` on the amended bullet: exit 0, no findings.
- Line widths of the amended bullet: 77 to 83 columns. The surrounding section 9 text
  runs to 85 columns.
- A grep of the diff for tracker numbers, `issue`, `pull request`, attribution and
  generated-by text found nothing.
- Cargo was not run, as the brief instructs. The PR body says the pre-commit hook
  passed. I did not re-verify that claim.

## Check results

1. **The two rules, stated exactly.** The old clause "and reads the exact head that
   will merge" is gone. The new text says: a delta after the review that changes
   behaviour "gets one further independent read of the exact head that will merge";
   a delta that changes no behaviour "(wording, comments or documentation) needs no
   further independent read." The parenthetical matches the issue's own wording.
   The rest of the bullet (no author's conclusion; findings fixed or disproved; gaps
   tracked) is unchanged except for re-wrapping. Nothing is broader or narrower than
   the issue asks.
2. **No conflicts.** Lines 418-419 (Delegation) require a final reading of the
   merging diff. That reading is the author's own check, explicitly "after the independent
   review and not in place of it". It is not an independent read, so it does not
   conflict with the amended bullet. Keeping it unchanged is what the issue's "no
   other rule changes" requires. Line 361 only requires a reviewed pull request. No
   other text in AGENTS.md or CLAUDE.md requires an independent read of every final
   delta.
3. **Clarity and wrap.** The rule is clear, and the wrap matches the file's existing
   width.
4. **Tracker numbers, process references and attribution.** The change contains no
   tracker number or attribution. Its only process wording is the review rule itself,
   which section 9 already contains.

## Findings

- **OPTIONAL** "A delta made after that review that changes behaviour" puts two
  relative clauses in a row. "A delta after that review that changes behaviour"
  would read a little more smoothly. The current wording is not ambiguous.
- **OPTIONAL** In RAF, capability documents are the specification and are checked by
  `docs_contract`. A reader could therefore treat the "documentation" example as also
  exempting a post-review edit to a capability Contract section. The issue uses the
  same parenthetical and "changes behaviour" remains the deciding test, so this is
  within the contract. Spelling it out is not required.

No REQUIRED findings.

VERDICT: PASS
