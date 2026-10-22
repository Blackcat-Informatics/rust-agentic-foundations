# Plan final review (Stage 1, pass 2 of 3)

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/16-raf-workplan-strict-mode-force-coverage/.stage/raf-workplan-strict-mode-force-coverage/plan.md` (Rev 2; `## Rev 2` is authoritative over `## Rev 1`, which is authoritative over the Rev 0 text above it), with its annex `design.md` (D§n) and the new "Superseded annex text" note.

Reports examined: `reviews/plan-completeness.md` (F1-F6, A1-A4), `reviews/plan-constraints.md` (B1-B2, A1-A7), `reviews/plan-design.md` (B1-B3, A1-A5), `reviews/plan-enhancement.md` (1-15), `reviews/plan-adversary.md` (F1-F11, E1-E5). Dispositions: the plan's Rev 1 and Rev 2 sections. Code read in pass 1 (still current, worktree at `95c384c`): `Cargo.toml:25-26`, `fuzz/Cargo.toml:1-20`, `crates/raf-workplan/src/coverage.rs` (545-615, fn list; `compute` at 1092), `crates/raf-workplan/tests/support/cases.rs:14-123`, `crates/raf-okf/tests/schema.rs:118-212`, `crates/cargo-workplan/src/cli.rs:288-304`, `tests/docs_contract.rs` (`matrix_violations` at 337, test at 852), `tests/self_workplan.rs` (no strictness reference today).

## 1. Pass-1 findings: recheck

| Pass-1 finding | Rev 2 text | Status |
|---|---|---|
| B1 R18 does not observe the record | line 155: no-flag `cargo workplan check` exits 0 at the PR head; T2 CLI fixture: no-flag `check` exits 1 on a `strictness: strict` record with one thin section; `self_workplan` asserts `strictness() == Strictness::Strict` and checks under it; base-checkout debt check; `.deficiencies`; PR paragraph | **DISCHARGED** |
| B2 R15 count vs the collapse kind | line 156: thirteen own kinds, trigger defined ("no clause start and more than `MIN_SECTION_WORDS` words"), `OWN_KINDS` 48 to 61 | **DISCHARGED** |
| B3 fuzz exception matches nothing | line 157: not judged; stated as a Constraint with a `raf:tested-by` fixture (a workspace excluding a directory of Rust files reports no row); no exception row | **DISCHARGED** |
| B4 undispositioned lens items | line 158: design A2 adopted (integer comparison, empty-crate fixture, T6); A1 moot; A4 adopted at T8 with cost; R8 Examples neighbour named; enhancement 7 adopted (line 163); items 3-9, 13-15 by category | **DISCHARGED** for every item with a correctness consequence; the category disposition leaves four items unassigned (A10 below, advisory) |
| A1 stale Rev 0 text | lines 29-31 ("single dated nightly"), 64 ("four existing tests") fixed; "Superseded annex text" added (line 7-8) | **PARTLY** (A1 below; nothing left unresolved by a later authoritative layer) |
| A2 E3 whitespace | line 159: `proc_macro2` token-stream comparison, doc comments count, fixtures named | **DISCHARGED** |
| A3 change-record item 11 | line 160: reworded to a product guarantee | **DISCHARGED** |
| A4 line-debt row shape | line 161: `item` + `orphans`, ranges never recorded, growth/resolved/over defined | **DISCHARGED** |
| A5 scope crates' lines | line 162: bound, reached or excepted in T8; no `lines` row names a `scope` crate | **DISCHARGED** |
| A6 per-class ceiling | line 163: `of: shipping|test|all`, default `all`, fixtures, `okf-bad-ceiling` | **DISCHARGED** (A11 below on the default's interaction with a class row) |
| A7 item 2 paragraph | line 164 | **DISCHARGED** |
| A8 heading | line 151 and 165 | **DISCHARGED** |
| A9 R21 lightweight | line 166: counted-work tallies pinned | **DISCHARGED** |

Integration check of the Rev 2 text itself: no new contradiction with Rev 1 tasks or rows beyond the stale lines listed in A1; T6 absorbs design A2 and enhancement 7; T8 absorbs design A4 and the `scope`-crate lines; nothing moved a task out of order; T9 still ends with the handoff. The design A4 removal of `matrix_violations` should land in the same commit that sets `strictness: strict` (T8 step (b)), so the matrix rule is never unenforced in between; the plan says "at T8", which admits that reading.

## 2. Findings (none BLOCKING)

### A1 (ADVISORY, STILL-OPEN in part): Rev 0/Rev 1 lines that Rev 2 overrides still read as current
Each is resolved by a later authoritative layer, so an implementer who honours the two "authoritative" headers is not misled, but each is a trap for one who reads top to bottom:
* line 52 "bodies and doc comments excluded" (E3 and Rev 2 A2 say they count);
* line 75 R5 "body/doc edit completes" (amended R5 and E3 say it refuses under strict);
* line 84 R14 "the one schema pin" (E2: four);
* line 116 "(crate, module path, item, range)" (Rev 2 A4: `item` + `orphans`, no range);
* line 119 "the separate fuzz workspace is listed as an exception with a reason" (Rev 2 B3: no exception row);
* line 130 R15 "twelve ... 48 to 60" (Rev 2: thirteen, 61);
* line 132 R18 "`--strict check` exits 0" (Rev 2 B1: no-flag check);
* lines 93-103, the Rev 0 Tasks paragraph with its unnumbered PR task (T1-T9 at lines 139-149 replace it; its commit-type list is the only part still needed).

REWRITE TO: prefix each of those lines with "(superseded: see Rev 2 <bullet>)", or delete them and move the commit-type sentence of line 100-101 into the numbered task list.

### A10 (ADVISORY): the category disposition of enhancement items 3-9, 13-15 leaves four items unassigned
Line 158: "items that extend beyond the issue (new commands, provenance, selection, extra presets) are declined with the cost that those consumers stay unbuilt, items on determinism and sorting are adopted in T6". Items 8 (provenance), 9 (`debt` report command), 6 (a new generated block) and 14's `status` line fall under the first clause; item 3 is declared moot; item 7 is adopted (line 163). Items 4, 5, 13 and 15 are neither new commands nor determinism/sorting, so their disposition is a guess. Item 15's first half matters to the plan's own claim unit: a `###` clause's statement is its heading line, and the unattached-marker rules of `claims.rs:190-212` must be shown to accept a marker directly after a heading, or heading clauses cannot be marked at all.

REWRITE TO (append to line 158): "item 4 (Scenario B detector): adopted in T2/T3 checks, the end state judged with `with_strictness(Strict)` and the thin document with the strict kinds, or declined with the cost that detector and gate can drift; item 5 (exclusions imply exceptions): adopted in T6 with the fixture 'an excluded item is not an orphan line', or declined with the cost of writing one reason twice; item 13 (fold case and punctuation in Constraints digests): declined, a re-punctuated item is a new claim and the capability says so, cost: a comma fix is growth until marked; item 15: adopted in T3 (fixture: a `raf:tested-by` marker directly after a `###` clause heading is attached) and in T2 (the `strict-section-thin` fix text says a generated block does not count as prose)."

### A11 (ADVISORY): the default ceiling with a class row
Line 163 adds `of: shipping|test|all` (default `all`) to a ceiling row; D§3.9 says a package without a row has `lines: 0`. A package whose only row is `of: shipping` would then have no `all` row: state whether the `all: 0` default still applies (recommended: it does unless an `all` row exists; class rows only add constraints), and pin it with the fixture "a `shipping` row alone leaves test lines under the `all: 0` default".

## 3. The five escalations (unchanged from pass 1)
* **E1 (R18):** acceptable as a discharge: no deliverable is dropped, the record is bounded to the base tree (E1(1)), the content debt of the edited capabilities and the `scope` crates' lines are paid, the maintainer is asked in the PR's first paragraph and merge waits (T9 stops before Stage 3), alternatives A-D stay visible, and a different choice later changes T8's content only. It remains a user decision, listed below.
* **E2 (R14):** acceptable: reading B is unsatisfiable by the issue's own R1/R11 (any repository-level key changes the pinned key list at `schema.rs:124-127`); the four expectations are named, updated in T7 and listed in the PR.
* **E3 (R5):** acceptable: the literal reading is the stricter one; the comparison rule is now stated (Rev 2 A2).
* **E4 (R9):** acceptable and disclosed; listed below for the user's word.
* **E5 (R10):** both spellings ship with a byte-identity test; nothing declined.

## 4. Open user decisions
1. **E1, R18 reading.** A: enable strict with a recorded baseline of the base tree's findings and a tracked issue for the rest (the plan's recommendation, gated at merge). B: empty debt, pay everything in this change. C: pay the edited capabilities and the `scope` crates (the plan already does this part) and record the rest (equals A as now written). D: deliver the mode and ratchet here and move R18 to its own tracked change. Deciding now rather than at merge changes nothing in T1-T7 and settles T8's content.
2. **E4, ratchet semantics.** A: census semantics, a resolved entry fails until `debt --write` (the plan). B: only growth fails; the next write drops resolved rows.
3. **E2 confirmation.** R14 read as unchanged behaviour; the four named test expectations may change.

VERDICT: PASS
