# T5 review: static obligations

Reviewed: the staged, uncommitted diff on top of 4c97695 (12 files, `git diff --cached`).
Experiments ran in a throwaway worktree built from HEAD plus the staged files; it is removed.
The staged index and the worktree were not touched.

## Scope

Task 5 of the plan (D6, D9, rows R3-R6, R8, R9, R14, carried items): `ObligationKind`,
`Obligation`, `Reconciled`, `Workplan::reconcile`, the derived sources, the tense scan,
`contract_effect: none` suppression, and the change to the delivered loading path
(`Workplan.unbound` in `load.rs` / `model.rs`).

## Evidence

Gates (all in the worktree, `CARGO_BUILD_JOBS=6`):

* `cargo test -p raf-workplan`: all 25 binaries pass (unit 135, `obligations` 27, `lifecycle` 25,
  `roadmap` 12, `states` 16, `gate` 18, `generated` 39, and the rest), 0 failed. No existing test
  file is modified by the diff; the pre-existing suite passes unchanged.
* `cargo test -p raf-workplan --release --locked`: pass.
* `make fmt`, `make clippy` (workspace, `-D warnings`), `make doc` (`-Dwarnings`): clean.
* `docs_contract` (26) and `repo_gates` (57): pass, so `Gate`/`check` on the repo itself reports
  the same as before. `make issue-refs`: pass.
* `make reuse`: compliant (348/348). `make deny`: advisories, bans, licences, sources ok.

(1) Kinds and evidence: `ALL` lists the seven kinds in the Contract table order, with a const
assertion on rank/length and a both-directions test. The evidence test parses the real table of
`docs/capabilities/workplan-reconcile.md`, requires the kinds in table order, and compares each
`Needed::contract_text`. It can fail: mutant I (`"a docs sync run"` -> `"a docs-sync run"`)
failed three tests, including this one.

(2) Derivation:

* `unresolved-proposal`: only this change's blocks (`proposal.change() == slug`); the line is
  `block.marker().lines.start`. Mutants A and K were killed.
* `item-without-outcome`: missing outcome, `moved:` to an unknown record, and `moved:` to the
  record itself. A `moved:` target that exists but did not load counts as existing (the
  record's own findings explain it). Mutant B killed.
* `binding-missing`: roadmap `gaps` for `implementing` and `delivered`; a delivered capability
  does not owe `examples` (mutant C killed); wrong-role bindings use the same `wrong_role` as
  `check`.
* Delivered loading change: `note_unbound` only fills a side list (`Workplan.unbound`). The
  violations are still pushed into `findings`, the document still does not enter
  `capabilities`, graph, roadmap, rules or generated analysis. `check`, `Gate` and the repo
  gates are unchanged (suite and `docs_contract` unchanged and green). It records only when
  every violation is `BindingRequired` (mutant H, which drops that condition, is killed), and
  the match is by the path `docs/capabilities/<slug>.md` for the listed id; id/stem
  disagreement is itself a profile violation, so a mismatched file cannot be matched. The
  capability is then judged by nothing else until it loads; the empty list keeps the output
  non-empty, so this is not a silent pass for that capability. A document is never treated as
  loaded. See REQUIRED R1 for the one silent path I found nearby.
* `generated-stale`: listed capabilities' files plus the index, from `generated::analyze`
  repairs; the line is the opening marker (`outer().lines.start`), the same expression
  `generated.rs` uses, so the two `unreachable!` branches are consistent. Mutant D (index
  dropped) killed. Stale-block reconcile cost is linear (4000 stale blocks: 20 ms in release).

(3) Tense scan: phrase list is sensible (eight phrases). Word-boundary matching verified
(`goodwill`, `William`, `willing`, `autowill`, `will_go`, `will-not` behave as documented:
`will-not` hits on `will`, `will_go` does not). Probed on the real scanner:

* hits: `Will it work.` (sentence start), `Is Planned`, `in the Future`, `NOT yet`, list item,
  table cell, heading text (`### It will`), `**will**`, curly apostrophe prose
  (`It’s going to rain.`), `will` + NBSP, link text `[will](x)`, a phrase wrapped over one line
  break, also inside a list continuation.
* no hit: `It WILL work.` (all-caps skip), `[a](http://x/will)` (destination skipped), phrase
  with two spaces or a tab, phrase across a blank line, `won’t`, fences/spans/comments, PROPOSED
  and generated lines. Mutants G (all-caps skip removed) and J (any whitespace joins) killed.
* Real repo docs, `docs/capabilities/*.md`, delivered states:
  `okf-profile` 0, `workplan-generated-docs` 0, `workplan-document-model` 1 (line 135,
  `proposed text sits beside it in PROPOSED blocks`, section contract), `source-index` 1
  (line 1225, `not yet for the globs of other modules`, section contract). Both are correct
  descriptive uses, so they are false positives (A2). No capability is `implementing`. Designed
  documents (not scanned by reconcile): `workplan-reconcile` has 4 hits, `workplan-cli` 1.

(4) `suppressed` is true only for `SectionUntouched` + Contract/Usage/Examples + effect `none`.
Unit test covers all kinds x sections; mutant E (Usage dropped) killed. `section-untouched` is
not raised yet, so integration coverage is `contract_effect_none_keeps_every_obligation...`.

(5) Ordering: key is (subject, part, kind, line, evidence text, path); `sort_by_cached_key` +
`dedup`. The unit test feeds a shuffled and a reversed list and compares the results; mutant F
(line dropped from the key) killed. The integration test "whatever the load order" is vacuous
for ordering: `MemoryTree` stores a `BTreeMap`, so rotating the input changes nothing (A4).
Parts sort document, sections in capability order, items. Subjects sort capability, change,
index.

(6) Hostile input, release build, reconcile call only (the load, which is not part of this
change, is a separate cost): 5000 PROPOSED blocks 1.4 ms; 4000 stale blocks 20 ms; 100k lines
and a 900 KB line are covered by tests that pass (0.7 s for the whole binary in release).
One super-linear path: many items `moved:change:<unknown>` with many change records. 2000x500:
64 ms, 4000x1000: 274 ms, 8000x2000: 1.33 s, 8000x10: 4.5 ms (`check` takes 1.4 ms). Cause:
`plan.change(target)` per item builds and sorts the full list of known slugs for the
`Unknown` error (A1).

(7) Mutants (11, all killed, each reverted): A proposals not filtered by change; B
moved-to-itself removed; C delivered owes examples; D index stale dropped; E Usage not
suppressed; F line out of sort key; G all-caps skip removed; H unbound read with other
violations; I evidence text changed; J any whitespace joins a phrase; K proposal line from
ordinal 0.

(8) Hygiene: SPDX headers on every new file (`make reuse`), no issue/PR numbers, no process
wording, no attribution text in the added lines (grep over the staged diff and `issue-refs`
gate). CHANGELOG entry added and matches the code. Module and `lib.rs` docs match the behaviour.

(9) Gates: see Evidence above.

## Findings

### REQUIRED

* R1. Silent pass: a PROPOSED block that names the change and stands in a capability document
  that does not load is never raised. `proposals()` reads `Graph::proposals()`, which holds
  loaded documents only; `Workplan.unvalidated_blocks` (path, slug, line), kept for exactly this
  case, is not read. Probe: base fixture plus an unlisted `docs/capabilities/zeta.md` missing
  `title` with one `> [!PROPOSED change:shape]` block gives `check` = one `okf-missing-key`
  finding on `zeta.md`, and `reconcile("shape")` = empty list (exit 0). The Contract says
  `unresolved-proposal` is raised "when a PROPOSED block names the change" and
  complete-gate condition 3 says "No PROPOSED block for the change remains in any capability
  document". The same gap hides the block of a listed delivered capability that is held in
  `unbound` (only its binding is owed until it loads). Fix: also iterate
  `plan.unvalidated_blocks` with `slug == change`, subject `cap:<file stem>` (section
  `Part::Document`), line from the tuple, and add a fixture (invalid unlisted document with a
  block; fixing the document or removing the block removes the line). Task 7 should rely on
  the same source for condition 3, since its `check`-finding condition covers listed
  capabilities only.

### ADVISORY

* A1. Quadratic cost in `items()` for `moved:` outcomes to unknown records (items x change
  records, measured above, 1.3 s at 8000x2000 in release, about 5x that in debug). Build the
  set of loaded slugs once (or ask only for existence) instead of calling `plan.change` and
  discarding the `Unknown` payload per item. Add a hostile fixture next to the tense ones.
* A2. Two delivered documents the repository already has produce a `tense-mismatch` for correct
  text if a future change lists them: `workplan-document-model.md` line 135 and
  `source-index.md` line 1225 (quoted above). The phrases come from a fixed list with no
  per-line allowance, so the only way to settle them is rewording. Reword when a change first
  lists those capabilities (or in the Task 11 pass over stale statements). For the same
  reason the `workplan-reconcile` Constraints sentence that names the phrases in double quotes
  (lines 129-130) and the Examples line 110 (`has not yet updated`) will be owed in Task 11
  step 4 once that capability is `delivered`; put the quoted phrases in code spans or reword.
* A3. The all-caps skip applies to every phrase, but its stated reason is the `PROPOSED`
  marker name. `It WILL work.` is therefore not found. Restrict the skip to `proposed`, or say
  in the module docs and the reconcile Constraints that all-caps text is never wording.
* A4. `equal_inputs_print_equal_output_whatever_the_load_order` cannot vary the order: the
  `MemoryTree` it collects into is a sorted map. The determinism proof is the unit test with a
  shuffled and a reversed input; rename or reword the integration test so it does not claim
  the load order is varied, or build the plan from a tree that lists in the given order.
* A5. Smaller scan edges, all false negatives or rare false positives, none worth a rework now:
  a heading line ending in `not`/`going`/`is` followed by a paragraph starting with the next
  word of a phrase joins across the block boundary (`### Not` / `yet fine` gives `not yet`);
  a phrase wrapped across `> ` blockquote lines is not joined (the continuation line does not
  lead with whitespace); a `](` that is not a link skips to the next `)` on the line.
* A6. "Sorted by capability, section and kind" in the reconcile Contract does not say which
  section order; the code uses the capability's own order (purpose ... verification). State
  it when the Contract text is reconciled in Task 11.
* A7. A `moved:` target that exists but did not load is treated as existing, so no obligation
  is raised for it; `check` reports the record. Matches condition 2 ("names a change that
  exists"); worth one sentence in the doc comment of `items`.


## Recheck after the fixes (second pass)

Staged again (12 files). Re-run on the new staged tree:

* R1 fixed. `Workplan::unresolved_proposals(slug)` reads the graph and `unvalidated_blocks`.
  My original probe (unlisted capability without `title`, one PROPOSED block for the change)
  now gives `cap:zeta  document  unresolved-proposal  line 25: ...` instead of an empty list.
  Mutant (the unvalidated loop disabled) is killed by the implementer's fixture
  `each_fault_is_owed_...`. Fixtures cover the untitled unlisted capability, the unbound
  delivered capability and a neighbour.
* A1 fixed: slugs are listed once. Release reconcile with moved items to unknown records:
  4000x1000 2.4 ms, 8000x2000 4.9 ms, 16000x4000 10.4 ms (was 1.33 s at 8000x2000); linear.
* A3 fixed: `It WILL work.` is now found. A4 fixed: the test is renamed
  `equal_inputs_print_equal_output` and says the load order cannot be varied there. A6 and A7
  addressed per the coordinator; A2 and A5 remain advisory as stated.
* Gates: raf-workplan tests debug (25 binaries, 0 failed) and release (25 ok, 0 failed),
  `docs_contract` 26 and `repo_gates` 57 pass; fmt, clippy `-D warnings`, doc `-Dwarnings`,
  `make reuse`, `make deny`, `make issue-refs` all clean.

No REQUIRED findings remain.

VERDICT: PASS
