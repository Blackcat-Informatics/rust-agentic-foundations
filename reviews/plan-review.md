# Plan final review (Stage 1)

Reviewed plan: `.stage/raf-workplan-change-lifecycle-cargo/plan.md`, Revision 3.

Inputs examined: the plan; `issue-analysis.md` (R1-R24, six NEEDS-JUDGMENT items);
`prior-art-assessment.md` (four NEEDS-JUDGMENT items); the five lens reports
`reviews/plan-completeness.md` (PASS, third recheck), `reviews/plan-constraints.md` (PASS, third
recheck, C8 by owner decision), `reviews/plan-design.md` (PASS, A1 advisory left open),
`reviews/plan-enhancement.md` (NEEDS-REVISION, E1-E15), `reviews/plan-adversary.md` (PASS, pass 3);
the three capability documents `workplan-reconcile.md`, `workplan-complete-gate.md`,
`workplan-cli.md`; the three change records under `work/changes/`; `rules.rs:118-175` (state rules).

## 1. Fixes are real

Every BLOCKING finding and every adopted enhancement was located in the revised plan text.

| Finding / item | Disposition claimed | Plan text found | Status |
| --- | --- | --- | --- |
| Completeness B1-B7, N1, A1-A8 | discharged | D2 L38-45, Task 11 L239-245, Task 9 L227, R1 L155, header L3 | real |
| Constraints C1, C2 (deferral; tracker reference) | discharged | D2 L38-45; L132 "Scenario A and Scenario B text stays as written; they are built" | real |
| Constraints C4 (bounds in the specs) | discharged | Contract-edit rows L125, L129 | real |
| Constraints C7, C10 (Gate contract; version-gate scope) | discharged | D4 L52-55: gate "NOT part of `Gate::open`/`assert_clean`", runs first in `Lifecycle::open` for `reconcile` and `complete` only | real |
| Constraints C8 (local `cross-check`) | owner decision | Task 12 L248 quotes the decision, narrows it to `cross-check`, runs every other target individually, requires the hosted job green before merge | real, recorded as a decision |
| Constraints C9, C11 (D9 relaxes Constraints; commit order) | discharged | rows L123, L128; Task 11 numbered order L239 | real |
| Design B1 (two spawners) | discharged | D3 L46-51, D4, Task 2 L190-193 | real |
| Design B2 (state check re-encoded) | discharged | D9 L91-92: "not re-encoded: it is 'no `check` finding'", plus the stated consequence that listed capabilities must end `delivered` | real |
| Design A8 (`gaps` only for `implementing`) | adopted | D6 L63-64: `roadmap::gaps` made `pub(crate)`, called for `implementing` and `delivered` | real |
| Design A9 (`Workplan::write` cannot carry the removal) | adopted | D9 L90-91: generated rewrite through `Workplan::write` first, `remove_file` last; R20/R21 L173 | real |
| Design A10 (Gate acquires a Git floor) | moot | D4 keeps the gate out of `Gate::open`, so no floor is added to the dev-dependency | real |
| E1 completion as transition | adopted | D9, Contract-edit rows L123-124, L128; R10/R15 | real |
| E2 typed `Evidence`/`Invocation` | adopted (trait declined) | D5 L56-60; D6 L61 | real |
| E3 touched = authored text | adopted | D7 L72-73, row L126, R7 "only-generated-block-changed reported" | real |
| E4 one load path, root discovery in the library | adopted | D4, Task 2 | real |
| E5 derived values, one `Impact` | adopted | D6 L63-66 | real |
| E6 `docs sync --copyright/--licence`, Makefile | adopted | D12 L107-108, row L130, Task 11 L239 | real |
| E8 `--affects`, template, guidance | adopted | D8 L76-84, row L127, R18 | real |
| E9 static-first | adopted | D9 L85-89, row L124 | real |
| E10 index-fact refusals | adopted | D11 L101-103, Task 3 | real |
| E11 fixture discipline | adopted | D11 L97-101 | real |
| E12 time bounds | adopted | D3 L49-50, row L125, Task 3 never-ending test | real |
| E13 in-process `Runner` fake | adopted | D3 L47, D11 L100-101 | real |
| E14 both argv shapes, JSON array | adopted | D12 L104-107, row L130, R13 | real |
| E15 pins (tiebreak, tense list, git cwd, TOCTOU, worktree) | adopted | D6 L62, L66-68; D7 L69-75; D9 L88-89 | real |
| Adversary F1-F6 | discharged | D2, R17 L172, R10/R14 L165-169, R1/R18 L155-156, L22-24, D8 L79-80 | real |

## 2. Declines are justified

- **E7 batching** (L112-115): technical reason (the Contract's per-binding argv rows and the
  "exactly one test ran" proof are kept) and a named cost (more `cargo` spawns on a large
  repository, mitigated by `complete` running only the change's own bindings). Justified.
- **Public obligation-source trait** (L116-117): the sources are internal functions; a public
  extension point is not added speculatively. This is the "lesser structure" paragraph the
  TRANSFORMATION item needs: `ObligationKind::ALL` stays closed per the Contract, the three
  sources are evaluated in a fixed order inside `reconcile`, and a later capability that adds a
  kind edits that function. Justified; the design reviewer concurred.
- **Design A1 (shared slug lookup)**: neither adopted nor declined in the plan. See ADVISORY 4.

## 3. Integration did not break the plan

Checked: D2 against R17/R22 and Task 11; D4 against Task 2 and the reconcile Contract's
"exits 2 before any other process"; D9 against R10/R15/R20/R21 and `rules.rs:143-174`; the
Contract-edit table against every Contract or Constraint the decisions change (writes, order,
time bounds, capture bound, "touched", `change new` options, JSON, argv shape, `docs sync`
options: all present); the three records' `affects` lists (each lists contract, usage, examples,
verification, constraints, so the PROPOSED blocks of Task 11 step 2 are valid without a record
edit). Tasks 10 and 11, the only tasks that grew substantially, have contract rows (R17, R22) and
inherit the per-task test, review, commit, push and issue-update steps stated at L179-180. No
contradiction between tasks was found beyond the Task 11 duplication judged below.

## 4. Coherence

Task order is implementable: exec (1) -> git and load path (2) -> executors (3) -> new/impact
(4) -> static obligations (5) -> git-backed and execution obligations, `verify` (6) ->
`complete` (7) -> CLI base (8) -> CLI lifecycle (9) -> scenarios (10) -> dogfood (11) -> PR (12).
No Task 0. Acceptance checks run on shipped entry points (`CARGO_BIN_EXE_cargo-workplan`,
`Gate`, the three records completed by the shipped binary). The items below are places an
implementer would otherwise have to guess; none changes the design.

## Findings

No BLOCKING findings.

### ADVISORY 1: Task 11 states its sequence twice, and the two differ by one step

The numbered order (L239) runs `reconcile` in step (3) while the capabilities are
`implementing` and in step (4) says only "set `state: delivered`, run `change complete` on each
record". The prose restatement (L241-243) and R17 (L172) run `reconcile` again *after*
`delivered` and record that output in `validation.md`. An implementer following the numbered
list records step-3 output, which does not satisfy R17's precondition "with the capabilities set
to `delivered`".

Judgment: a mild hazard, not a blocking one. Nothing fails silently, because `complete`
re-evaluates condition 1 itself and R17 is the binding acceptance row. The duplication is still a
maintenance trap (two texts that must be kept equal) and the numbered form is the one the
implementer will follow.

Quoted: "(4) in the working tree only, set `state: delivered`, run `change complete` on each
record (it deletes the record and rewrites the staled blocks), then commit the state edit, the
INVENTORY row, the record deletions and the rewrite together."

REWRITE TO: "(4) in the working tree only, set `state: delivered` on the three capabilities, run
`cargo-workplan change reconcile` on each record (empty list, exit 0) and then `change complete`
on each (exit 0), recording both outputs in `validation.md`; commit the state edits, the
INVENTORY row, the record deletions and the generated rewrite together." Then delete the prose
restatement from "Reconcile the three documents" through "together;" at L239-244, keeping only
the trailing sentence about stale statements elsewhere, `make docs-sync` idempotent, and
`docs_contract`/`Gate` clean.

### ADVISORY 2: Task 2 places the version gate in a type Task 4 creates

Quoted (L190): "version gate in `Lifecycle::open`, not in `Gate::open` (2.29 refused, 2.30 and
2.45.1 accepted) running first".

`lifecycle` is introduced in Task 4, and `new`/`impact` do not need Git, so Task 2 cannot host
the gate inside `Lifecycle::open`.

REWRITE TO: "`git::require_version(&dyn Runner) -> Result<(), GitError>` with the 2.29 refused /
2.30 and 2.45.1 accepted fixtures; Task 6 and Task 7 call it first in `Lifecycle::open` for
`reconcile` and `complete`, before any other process".

### ADVISORY 3: the "empty bindings in `designed`/`considered`" fixture needs a second open record

R10 (L165): "empty bindings satisfy condition 5 only in `designed`/`considered`". Under D9 the
post-deletion model hides the record, and `rules.rs:164-174` raises `no-open-record` for any
non-`delivered` capability with no listing record, so a `designed` capability listed only by the
completing change refuses on the post-deletion check even though condition 5 passes. The fixture
as written will refuse for a reason other than the one it tests.

REWRITE TO (append to the R10 row): "the `designed`/`considered` cases carry a second open
record listing the capability, so the post-deletion model is clean and the fixture isolates
condition 5".

### ADVISORY 4: design A1 (shared change-slug lookup) has no disposition

Five callers (`preview`, `impact`, `reconcile`, `complete`, `verify`) and the CLI rule "exit 2
when the record does not exist, except `new`" need one unknown-vs-not-loaded answer; today only
`preview.rs` holds it and `Workplan::unloaded` is `pub(crate)`. The plan neither adopts nor
declines the item.

REWRITE TO (add to Task 4): "`pub(crate) fn Workplan::change(&self, slug) -> Result<&Loaded<Change>,
ChangeLookup>` with `Unknown` and `NotLoaded` variants; `preview` converts to it; the CLI maps
`Unknown` to exit 2". Or add a line under Declined with the reason.

### ADVISORY 5: the `change new` Contract-edit row should name the default `affects`

Row L127 reads "`change new` options `--affects`; printed template and state guidance". R18
(L156) asserts the exact default value `cap:<slug>#contract` per capability, which the reconcile
Contract (L27-31) does not state.

REWRITE TO: "`change new`: default `affects` of `cap:<slug>#contract` per listed capability,
repeatable `--affects`; printed template and state guidance".

## Open user decisions

None block implementation. Two readings are recorded for the owner to confirm or overrule in PR
review, as the plan already states:

1. **U1**: the issue's "scaffolds ... a PROPOSED block" is read as *printing* the block template
   (the CLI Constraint fixes a one-file write). Alternative: `change new` also writes the block
   into each listed capability (a Contract edit to the one-file Constraint). The plan's reading
   stands unless the owner chooses the alternative.
2. **Hosted CI for `cross-check`**: recorded as the owner's decision ("Hosted CI counts
   (Recommended)"), quoted in the PR body; the constraints reviewer notes it reached the review
   through the coordinator, so the PR body must carry it verbatim.

## Summary

The five reviews were integrated: every BLOCKING finding maps to changed plan text that does
what its disposition says, both declines carry a reason and a cost, and the integration did not
introduce a contradiction. The Task 11 duplication is a mild hazard with a one-step omission in
the numbered list; the other four advisories are wording and fixture-setup gaps an implementer
would otherwise guess at. All five can be folded into the plan without re-review.

VERDICT: PASS
