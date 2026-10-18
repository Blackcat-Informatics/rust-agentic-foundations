# Plan completeness review

Reviewed plan: `.stage/raf-source-model-rustc-s-visibility-of/plan.md`
Inputs: `issue.md` (body, 0 comments), `issue-analysis.md` (R1 to R7, 3 NEEDS-JUDGMENT), the owner instruction on record ("In your next PR, make sure to update README.md to the current status."), `docs/capabilities/source-index.md`, `crates/raf-source/src`, `crates/raf-source/tests`, `README.md`.

Confirmed in source: `settle.rs`, `glob_scope.rs`, `tests/use_resolution.rs` and `tests/support/` exist. `Workspace::load` (workspace.rs:110) and `Workspace::resolve` / `resolve_in` (workspace.rs:373/382) are the shipped entry points. The Constraints bullet to remove is at source-index.md:1654-1661 and starts "Two visibility rules of rustc are not modelled", so the plan's grep string matches it. No rustc-oracle harness exists, and Task 1 creates one.

## Requirement -> task -> check

| Req | Source | Task | Check | Status |
|---|---|---|---|---|
| R1 model both rules in settling, order-independent | Scope 1 | 2, 3 | every-order fixtures plus oracle | Falsifiable only after F2 (the fixture expectations are not stated in the acceptance column) |
| R2 every-order fixture for each rule | Scope 2 | 2, 3 | fixtures named in Verification, with neighbours | Partial: no fixture names and no expected `Resolution`/`ResolveError` (F2) |
| R3 re-run the rustc-oracle fuzz with private globs, no new disagreements | Scope 3 | 1, 4 | `cargo test -p raf-source --test rustc_oracle` plus a full run | UNFALSIFIABLE: the oracle is never shown to detect either rule, and the run size is unbounded (F1) |
| R4 remove the two sentences from Constraints | Scope 4 | 5 | `grep -c` = 0, Contract states both, `change complete` | Mapped. Advisory F6 |
| R5 re-export no more visible than the binding it re-exports | Context 1 | 2 | rule-1 fixture | Expected observable missing (F2) |
| R6 refuse ambiguity through an inaccessible glob (E0659, sometimes E0573) | Context 2 | 3 | rule-2 fixture plus E0573 shape | Expected observable missing (F2). The neighbour is unclear (F5) |
| R7 both rules modelled | Context | 2, 3, 5 | as above | Mapped |
| NJ-1 no oracle harness | analysis | 1 | built as a committed test | Disposed |
| NJ-2 which error the fixture asserts | analysis | 3 | ambiguous vs not found | Disposed in prose. It must be carried into the acceptance (F2) |
| NJ-3 which Contract sentences | analysis | 5 | source-index.md Constraints bullet | Disposed, and verified at lines 1654-1661 |
| OWNER update README to current status | owner instruction | 6 | README matches `cargo workplan status` | Mapped. Advisory F7 |

## Findings

### F1 BLOCKING UNFALSIFIABLE (R3): the oracle is never shown to detect the two rules, and "larger" has no size

Plan line (contract row 1): "before Tasks 2 and 3 it reports the known disagreements (a fixed-seed failing case is captured as a named regression input)"

Plan line (contract row 5): "full oracle run at the final head with a larger seed list; the count of disagreements is zero; results recorded"

The issue's fuzz is gone, so "the known disagreements" names nothing. A generator that never produces the rule-1 or rule-2 shape reports zero disagreements before and after, and the "re-run" passes vacuously. "Larger" is also unbounded, while the issue's baseline is 5,522 programs.

REWRITE TO (row 1): "`cargo test -p raf-source --test rustc_oracle` generates programs with private globs from a fixed seed list, compiles one probe per query with `rustc --crate-type lib`, and compares each probe with `Workspace::load` + `Workspace::resolve` on the same tree. Before Task 2 the run at the pre-fix head reports at least one rule-1 disagreement (a module that globs a `pub use` of a privately-globbed binding resolves a name rustc rejects) and at least one rule-2 disagreement (the index answers an item where rustc reports E0659 or E0573). The seed, program and query for each are recorded in the stage notes and become the named regression inputs of Tasks 2 and 3."

REWRITE TO (row 5): "With `RAF_ORACLE_PROGRAMS` (or the chosen variable) set to at least 5,522 programs with private globs, the oracle at the final head reports 0 disagreements. The seed range, the program count, the query count and the per-rule count of shapes generated are recorded in the stage notes and in the PR body."

### F2 BLOCKING UNFALSIFIABLE (R2, R5, R6): the fixture acceptance has no expected result

Plan line (row 2): "the oracle has no rule-1 disagreement; the every-order fixture passes in all permutations of declaration order"

Plan line (row 3): "same for rule 2, including the E0573 shape"

"Passes" without a stated expectation can be met by a fixture that asserts the current behaviour. The oracle also has no defined way to attribute a disagreement to "rule 1" or "rule 2".

REWRITE TO (row 2): "`use_resolution.rs` gains `<rule-1 fixture name>`: in every declaration order, a module that globs the module whose `pub use` re-exports a binding it reaches only through a private glob gets `NotFound` (rustc E0432/E0425) for the name, and the re-exporting module itself still resolves the item. The neighbour `<name>` (the same re-export of a binding reached by a `pub` glob) resolves to the item in every order. The rule-1 regression input from Task 1 agrees with rustc."

REWRITE TO (row 3): "`<rule-2 fixture name>`: in every declaration order, a reader whose module's globs bring a name in from two items, one through a glob the reader cannot access, gets `Ambiguous` (rustc E0659). `<E0573 fixture name>` gets the not-found result in every order. The neighbour (see F5) resolves in every order. The rule-2 regression input from Task 1 agrees with rustc."

### F3 BLOCKING STRUCTURE: no per-task validation, review, commit, push or issue update

Tasks 1 to 6 name no focused validation command, no independent task review, no commit, no push and no issue update. Task 1 also plans a test that fails at its own head ("before Tasks 2 and 3 it reports the known disagreements"). Under the commit hooks and `make check`, that test cannot be committed unless the plan says how.

REWRITE TO (append to each of Tasks 1 to 6): "Validate: `<focused cargo test / workplan command>`. Then an independent task review, `make check`, commit, push, and an issue comment naming the commit and the check output."

REWRITE TO (Task 1 addition): "The committed default seed list passes at this head. The pre-fix disagreeing programs are recorded in the stage notes and committed as regression inputs in Task 2 (rule 1) and Task 3 (rule 2), in the same commit as their fix."

### F4 ADVISORY TEST-ONLY risk: "Skips" when rustc is missing

Plan line: "Skips with an explicit failure message when `rustc` is unavailable"

A skip makes the R3 guard pass with no oracle run.

REWRITE TO: "Fails with an explicit message when `rustc` cannot be run. It never skips."

### F5 ADVISORY: the rule-2 neighbour is not a definite program

Plan line: "a neighbour (ambiguity resolved because only one glob is accessible but the other item is not a candidate by rustc's rule stays resolved)"

REWRITE TO: "a neighbour where two globs bring in the same item (one of them inaccessible to the reader), and one where the inaccessible glob brings in no item of that name, each resolving to the item in every order. Also re-check, against rustc, each existing Verification fixture that answers an item beside an inaccessible glob (for example 'a cycle whose glob answer holds an item of a private glob ... resolves to the item it can access in all 24 orders'). Keep it or correct it, with the rustc outcome stated."

### F6 ADVISORY (R4): name the gate for the capability

Plan line: "capability reconciled and completed with the tool"

REWRITE TO: "`cargo workplan check` passes, the new fixtures appear in the generated verification matrix of `source-index` after `docs sync`, and `cargo workplan status` shows `source-index` delivered with no open change."

### F7 ADVISORY (owner instruction): make the README check concrete

Plan line: "README status text matches `cargo workplan status`"

REWRITE TO: "For each crate row of README.md, the Status column equals the state `cargo workplan status` reports for its capability. The `cargo-workplan` command list equals the subcommands in `cargo workplan --help`."

VERDICT: NEEDS-REVISION
