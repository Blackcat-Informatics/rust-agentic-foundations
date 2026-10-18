# Plan completion adversary review

- Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/14-raf-source-model-rustc-s-visibility-of/.stage/raf-source-model-rustc-s-visibility-of/plan.md`
- Pass: 1 of 3 (no prior review of this plan exists)
- Inputs read: `issue.md` (body, 0 comments), `issue-analysis.md`, `prior-art-assessment.md`, `requirements-candidates.md`, `CLAUDE.md`, `AGENTS.md`, `README.md`, `docs/capabilities/source-index.md` (Constraints, Verification), `crates/raf-source/src/exports.rs` header, `crates/raf-source/src/workspace.rs` entry points, `crates/raf-source/src/resolve.rs` (`ResolveError`). `.baseline`, `.goals`, `CONSTITUTION.md` and ADRs do not exist in the worktree.
- User scope decisions: none in the issue (0 comments). The plan quotes one owner instruction ("In your next PR, make sure to update README.md to the current status."). It adds scope and narrows nothing.

## Findings

### F1. UNFALSIFIABLE CRITERION: the oracle's power and pass condition (row 1, Task 1)

Quoted plan text:

> `cargo test -p raf-source --test rustc_oracle` generates a fixed seed list of programs with private globs, asks real rustc and the index the same questions, and reports every disagreement; before Tasks 2 and 3 it reports the known disagreements (a fixed-seed failing case is captured as a named regression input)

> asks `rustc --crate-type lib` whether `use` probes of each name compile (one probe item per query, in one compile with error codes per query where possible), then asks the index what the name resolves to for the probing module.

> Skips with an explicit failure message when `rustc` is unavailable (it is always available under cargo).

Why this fails the ground: the issue's fuzz no longer exists (prior-art-assessment.md: "Rustc-oracle harness: none found"). That means R3 ("no new disagreements") rests entirely on a new generator, and the plan never requires that generator to be able to produce either rule's shape. The issue measured the two rules in 10 of 5,522 programs. A generator that never builds the shapes reports zero disagreements and still satisfies rows 1, 2, 3 and 5. The quoted text also leaves these gaps:

- "the known disagreements" has no count and no rule.
- "a fixed-seed failing case" is singular, though there are two rules.
- "reports" does not say that the test fails.
- "where possible" leaves room to merge probe verdicts that cannot be attributed to one query.
- "asks the index" names no shipped entry point.
- "resolves to a different item" gives no way to observe which item rustc bound.
- "Skips" lets a run that compiled nothing pass. AGENTS.md says "A missing tool is a hard failure, not a skip."

REWRITE TO (row 1 acceptance and the matching Task 1 text):
"`cargo test -p raf-source --test rustc_oracle` builds library crates from a fixed seed list. For each query it compiles a probe with `rustc --crate-type lib` and records accept or reject with the error code. Each candidate item carries a distinct compile-time marker (for example a distinct `const ID` that the probe checks), so an accepted probe also records which item rustc bound. A verdict that cannot be tied to its own probe by span fails the test; it is never counted as agreement. The test asks `Workspace::resolve("<crate>::<probe module>::<name>")` for the same query. It fails, listing seed, module, name, rustc's answer and the index's answer, when any query disagrees. When `rustc` cannot be run, the test fails and names the command; it never skips. On the resolver as of main 16ffd2f (before Tasks 2 and 3), a run of at least 5,522 programs reports at least one disagreement of the rule-1 shape and at least one of the rule-2 shape. Each is recorded with its seed, module and name. Those seeds join the default seed list in the same commit as the fix that makes them agree."

### F2. UNFALSIFIABLE CRITERION: the every-order fixtures state no expected observation (rows 2, 3, 4; Tasks 2 and 3; NEEDS-JUDGMENT)

Quoted plan text:

> the oracle has no rule-1 disagreement; the every-order fixture passes in all permutations of declaration order

> same for rule 2, including the E0573 shape

> fixtures named in the Verification list of `source-index`; neighbours (accessible glob, unambiguous glob) still resolve

> Fixture in every order and a neighbour (ambiguity resolved because only one glob is accessible but the other item is not a candidate by rustc's rule stays resolved).

> the rare E0573 case gets its own fixture whose expectation is "refused"

Why this fails the ground:

- "passes" and "still resolve" name no query, no entry point and no expected value, so any assertion counts as passing.
- "the oracle has no rule-1 disagreement" needs disagreements to be sorted by rule, and the plan never defines how.
- The Task 3 neighbour sentence cannot be parsed into a program.
- "the E0573 shape" is never described.
- The NEEDS-JUDGMENT disposition ("refused", which any `Err` satisfies) contradicts Task 3 ("the E0573 shape refused as not found").
- The issue measured these rules "on code rustc accepts", but nothing ties the fixture texts to crates rustc accepts.

REWRITE TO (rows 2 to 4 and the fixture sentences of Tasks 2 and 3):
"Rule 1: a fixture in `crates/raf-source/tests/use_resolution.rs` on a crate that `rustc --crate-type lib` accepts (the Task 1 harness compiles the fixture text). In it, `b` reaches `x` only through a private glob and re-exports it with an explicit `pub use`, and `c` globs `b`. In the name space where the re-export is weaker, `Workspace::resolve("demo::c::x")` returns `Err(ResolveError::NotFound(_))` in all n! declaration orders. Neighbour: the same crate with `b`'s glob `pub` returns `Ok` binding `demo::a::x` in all orders.
Rule 2 (E0659): a module `n` brings `X` in through one glob a reader `r` cannot access and through one it can, as two different items. `Workspace::resolve("demo::r::X")` returns `Err(ResolveError::Ambiguous(_))` holding both items, in all orders.
Rule 2 (E0573): a named program for which the harness records E0573 on the probe. `resolve` returns `Err(ResolveError::NotFound(_))` in all orders.
Rule 2 neighbour: the same modules, with the inaccessible glob bringing in no `X`, resolve to the accessible glob's item in all orders.
The rule-1 and rule-2 seeds captured in Task 1 agree. Each fixture is listed by name in the Verification section of `source-index`.
Replace the NEEDS-JUDGMENT line with: the E0573 fixture expects `Err(ResolveError::NotFound(_))`."

### F3. UNFALSIFIABLE CRITERION: the size of the final re-run (row 5, Task 4)

Quoted plan text:

> full oracle run at the final head with a larger seed list; the count of disagreements is zero; results recorded

> Run the oracle with a large seed list at the final head (outside CI, results in the stage notes and the PR) and require zero disagreements

Why this fails the ground: "larger" and "large" set no size. A run of a few hundred programs reports zero disagreements while being about 5% of the issue's 5,522. At the 10-in-5,522 rate, such a run is unlikely to contain a single rule shape, so zero disagreements would say nothing about R3. "results recorded" names no content.

REWRITE TO:
"At the final head, the oracle runs with the program-count variable set to N ≥ 5,522, over a seed range that includes the rule-1 and rule-2 seeds from Task 1, and reports zero disagreements. The PR body records the seed range, program count, query count, `rustc --version` and the disagreement count (0). Each disagreement found is fixed in the settling with an every-order fixture, and the full run is then repeated until it reports 0."

### F4. UNMAPPED REQUIREMENT: rule 2 on `Workspace::exported_items`, and the second Constraints sentence about these rules (R1, R4; row 6, Task 5)

Quoted plan text:

> `grep -c "visibility rules of rustc are not modelled" docs/capabilities/source-index.md` is 0; the Contract states both rules; capability reconciled and completed with the tool

> Reopen `source-index` (`change new`), remove the Constraints bullet, state both rules in the Contract, bind the new tests, `docs sync`, re-assess the changed sections, `change complete`.

Evidence: the Constraints of `docs/capabilities/source-index.md` speak about these rules in a second bullet too (lines 1677 to 1682): "Of the two visibility rules above that the index does not model, the first does not reach the surface, ...; the second does: a name that the globs of a module bring in ambiguously, one of the items through a glob that a module which globs it cannot access, is listed under that module with the other item, where rustc refuses the path." The header of `crates/raf-source/src/exports.rs` says the surface "is read from the settled scope tables" and fails a name "with the error [`Index::resolve`] gives for its path". Modelling rule 2 in the settling therefore changes what the shipped `Workspace::exported_items` returns. The plan has no task, fixture or contract row for that entry point. It also does not edit this sentence, and the row-6 grep passes with the sentence still claiming the rules are unmodelled and that the surface lists the other item.

REWRITE TO (add a row and replace row 6):
"| rule 2 on the public surface | 3 | A fixture in `crates/raf-source/tests/exported_items.rs` on a library where a listed module `m` does `pub use n::*;` and `n` brings `X` in, as two different items, through a glob `m` cannot access and through a `pub` glob. `Workspace::exported_items` no longer lists `demo::m::X` as the other item. It returns `Err(ExportError::Resolve(ResolveError::Ambiguous(_)))`, as the Contract already does for a name two globs of a listed module bring in as two items, in every declaration order. (If the plan chooses another result, it states that result here.) Neighbour: without the inaccessible glob's `X`, the listing holds `demo::m::X`. |"
"| remove the two sentences from the Constraints | 5 | `grep -cE 'not modelled|does not model' docs/capabilities/source-index.md` prints 0. The public-surface Constraints bullet no longer refers to 'the two visibility rules above' and no longer says such a name 'is listed under that module with the other item'. The Contract states both rules and their effect on `exported_items`. `cargo workplan change complete` exits 0 for the change. |"

## Cleared on this pass

- Row 7 / Task 6 (README): falsifiable against `cargo workplan status` and the command list. The README is current at base, so its acceptance is a check, not a vacuous pass.
- Task 7 (handoff): no completion claim.
- Declined items: "A CI job running the oracle at full size" is not requested by the issue, whose Scope asks for a re-run, which rows 1 and 5 keep. "Modelling the rest of rustc's name resolution" is outside the issue. Neither is a pre-emptive descope.
- No PLANNED-BY-REFUSAL: refusal (E0659, E0573) is rustc's answer for rule 2 and the requested behaviour, and the oracle checks both directions (rustc accepts while the index refuses, and the reverse).
- R7 ("not modelled yet") and R5/R6 map to Tasks 2 and 3. Order independence (R1) is covered by every-order fixtures, once F2 states their observations.

## Required user decision

None on pass 1. All four findings can be fixed by rewriting the plan without changing scope.

VERDICT: FINDINGS-OPEN
