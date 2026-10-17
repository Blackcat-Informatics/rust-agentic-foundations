# Plan completion adversary review

* Reviewed plan: `.stage/m2-rest-of-the-planning-essay-entity/plan.md`, Revision 3 with its later edits
* Pass: 3 of 3. This is the last Stage 1 pass; there is no fourth.
* Scope of this pass: the six findings still open after pass 2 (F2, F4, F6, F7, N1, N2), and the text that was
  written to fix them. Pass 2 discharged F1, F3 and F5. Their unchanged text is not reopened.

## User decisions on the record

* **Coverage scope.** Plan lines 28-34 quote "Workplan crates only (Recommended)". This was cleared on pass 1
  and is unchanged.
* **N1.** Plan D4 quotes the owner: "Run the command (Recommended)". The original ask was a recorder that runs
  the command, as the PROPOSED Usage line says. The plan now delivers that.

## Rulings

### F2. Evidence record content: DISCHARGED

R2 now reads: "the written record holds command, revision (when given), verdict, values, limits and input
digests, and recording changes no file under `docs/`". Both residual gaps from pass 2 are now checked: the
record's contents, and the PROPOSED constraint "Recording evidence never edits a capability document".

### F4. Blanket coverage exclusion: DISCHARGED

* D9: "an item path or a `<module>::*` prefix below a crate root ...; a crate-root prefix is refused: take the
  crate out of `scope` instead". A single `raf_workplan::*` entry can no longer make R4's real-use row
  ("`check` is clean with scope = workplan crates") trivially true.
* Taking a crate out of `scope` is visible in the record. The reports still name every gap in the workspace
  (plan lines 30-31; R4 "reports name raf-okf and raf-source gaps").
* The new refusal is covered by the standing commitment on plan line 20: "every rule has a negative fixture
  and every refusal a passing neighbour".
* Pass 2 also asked about bindings that name a crate root. That concern was conditional ("If a binding may name
  a crate root"). The delivered binding rule (document-model: "Each entry in `bindings.code` is a Rust path to
  a shipping item") gives no basis for that reading, so it is not carried forward as speculation.

### F6. Lifecycle rows: DISCHARGED

* R8 now reads: "none `dropped`; reworded claims are listed in the PR body".
* D14: "A claim found untrue as written is reworded to what was built ...; any part of the claim that is not
  built is moved to a tracked record, so a rewording never hides a cut ... nothing is `dropped`". This matches
  issue L21: "any scope that leaves an item is `moved:` to a tracked change".
* R6 now covers the delivered capabilities: "(also the delivered capabilities of D16: states `delivered` at
  HEAD, tables regenerated, text states the new contract)". D16 names the records through which reconcile,
  complete-gate, cli, document-model, okf-profile and source-index are changed. Task 11 step (1) puts them in
  `implementing`.

### F7. PROPOSED edit list: DISCHARGED

The edit list now has these rows:

* "optional command and revision | the record states the command and, when given, the revision".
* "`cfg(test)` tests count", under two-way-coverage.
* "all five | 'Open questions:' lists in Purpose | removed when the questions are resolved by the text;
  checked by a grep fixture in Task 11".

`doc(hidden)` has no row of its own. It is part of the public-surface definition, which is the
two-way-coverage open question "Which items count as public surface". The Open-questions row commits that
question's resolution to the document text, and the Task 11 grep check verifies the list is gone. On that basis
F7 is discharged.

### N1. The evidence command runs: DISCHARGED

* The owner decision is quoted in D4.
* D4 reads: "RUNS the argv after `--` ... the verdict (exit status 0 = pass, otherwise fail, or fail when a
  printed value exceeds its limit)".
* R2 checks the semantics: "`evidence record -- <argv>` runs the argv without a shell (stub proves argv, stdin
  closed, bounded output, timeout kill), writes exactly one file, derives the verdict from the exit status and
  printed `name=number` values against the limits (an over-limit value is `fail`), persists no output text,
  and a failing command still records a `fail`".

### N2. `cfg(test)` unit tests: STILL-OPEN (the acceptance contract contradicts the fix)

Two of the three places were fixed:

* D9: "`cfg(test)` unit tests count like other tests and are excluded by prefix with a reason where they only
  guard internals".
* Edit list: "`cfg(test)` tests count".

The executable acceptance check was not fixed. The R4 row (plan line 186) still reads:

> "bound vs unbound test; `doc(hidden)` and `cfg(test)` out;"

R4 is what Task 7 builds against ("Acceptance: the R4 fixtures"). Read literally, it calls for a fixture that
certifies unbound `cfg(test)` unit tests as absent from `coverage tests`. That is the cut N2 identified, and it
contradicts both D9 and the edit list. This repository binds such tests (for example
`raf_source::tree::disk::tests::...` in `docs/capabilities/source-index.md`), and the scope crates hold
`cfg(test)` modules. The criterion therefore matters on real input.

REWRITE TO (R4 cell): replace "`doc(hidden)` and `cfg(test)` out" with "a `doc(hidden)` item is absent from
`coverage code`. An unbound `cfg(test)` unit test appears in `coverage tests`, and is absent once a
`<module>::tests::*` exclusion with a reason lists it."

## Observation outside the four grounds (not a finding)

D12 still says "The tool starts no process other than `git` (and the already-built `cargo` rows) for these
capabilities". That contradicts D4's `Program::Trusted` run of the caller's argv. D4 and R2 are explicit, so
no criterion is weakened. The integration reviewer should still reconcile the sentence.

## ESCALATE-TO-HUMAN: pass 3 of 3, no further pass

One criterion is still open (N2, the R4 cell), and no further pass exists to review a fix. The owner chooses
one of:

* **(a), recommended.** Apply the REWRITE TO text above to R4, word for word. This matches the plan's own D9
  and edit-list row and changes no scope. With R4 reading exactly that text, nothing from this lens remains
  open.
* **(b)** The owner explicitly decides that `cfg(test)` unit tests are out of the tests-to-claims report by
  default. D9 and the edit-list row are then reverted to match R4, and the decision is quoted in "Baseline,
  goals, governing decisions" next to the other owner decisions.

Summary: F2, F4, F6, F7 and N1 are discharged. N2 is still open, and only on the R4 acceptance cell.
BLOCKED here means one finding is open after the final pass and has been escalated to the owner; in the Stage 1
vocabulary this is ESCALATE-TO-HUMAN.

VERDICT: BLOCKED
