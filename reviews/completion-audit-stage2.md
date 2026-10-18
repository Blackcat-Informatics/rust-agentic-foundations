# Completion audit, Stage 2 (adversary stance): issue 14, PR 26

## Scope and authority

- Reviewed source: branch `paudley/14-raf-source-model-rustc-s-visibility-of` at HEAD `fc21ada`
  (equal to the PR head and the pushed branch), base `origin/main` `16ffd2f`. All runs used
  `git clone --no-hardlinks` copies under `/tmp/raf-audit14-s2/` (`clone`, `clone2`: HEAD;
  `clone-old`: `5f5049b`, the harness commit whose settling equals `origin/main`'s, with only the
  additive `resolve_from` entry point; `clone-main`: `origin/main`). The worktree was not mutated.
- Not covered: the worktree has since gained uncommitted changes by another agent (`src/resolve.rs`,
  `src/workspace.rs`, `tests/rustc_oracle.rs`, `tests/support/{mod,program,orders}.rs`, two new corpus
  files). This audit judges `fc21ada` only.
- Context layers read: `CLAUDE.md`, `AGENTS.md`, `issue.md`, `plan.md` (Revisions 1 to 3),
  `validation.md`, `tasks/T4c-review.md` (for the companion claim), issue 14 comments, PR 26 body,
  issues 24 and 25. `gap-analysis.md` was not read.
- Not run, by instruction or rule: `make check`, `cargo test --workspace`, the `raf-okf` and
  `raf-workplan` timing tests. No timing assertions were written; scratch code reads no clock.
- `.deficiencies` (root and worktree): notice and marker only, no entries.

## Criteria

| # | Criterion | Demonstration run | Observation | Status |
|---|---|---|---|---|
| C1 | Rule 1 and rule 2 behave as rustc on the issue's shapes | 16 auditor-written programs (not fixtures; `/tmp/raf-audit14-s2/scripts/c1_programs.py`): 6 for rule 1 (renamed re-export, `pub(super)` glob, `pub(in crate::a)` glob, glob re-export chain, capped plus wider route, a cap that hides an ambiguity) and 10 for rule 2 (private vs `pub` glob, `pub(super)` vs `pub`, module vs unit struct, value space, glob chain, struct and fn of one name, no item accessible, three neighbours). 63 probes, 13 orders each (as written plus 12 shuffles of the blocks and of the declarations inside each module, fixed seeds). Per probe and order, 5 compiles with `rustc --edition 2024 --crate-type lib`: `use <p> as __p;`, `type __T = <p>;`, `fn __f() { let _ = <p>; }`, plus type and value identity probes. Index through `Workspace::resolve_from` and `resolve_from_in` (scratch example `audit_resolve`). Rerun with edition 2021. Control: the same programs on the pre-fix resolver. | Rule 1: 364 of 364 probe-orders agree. Rule 2: 443 of 455 agree. The 12 disagreements are one shape (`crate::hub::Mix` read from outside `hub` and from the root): rustc accepts the `use` in 6 of 13 orders and refuses it with E0659 in 7, and the index answers `Ambiguous` every time. The per-space answers agree in all 13 orders. This is the documented "rustc answer depends on order" limit. The index gives one answer in every order for all 63 probes. Edition 2021 gives the same result. The pre-fix resolver agrees on only 403 of 819, so these programs do tell the two resolvers apart. | PASS |
| C2 | Every-order fixtures pass | `cargo test -p raf-source --locked --test use_resolution --test exported_items` | 93 passed and 39 passed, exit 0. The one changed fixture expectation (`an_import_a_cycle_leaves_to_the_globs...`) gives rustc's order-dependent reading as its reason and adds an assertion. It is not weakened. | PASS |
| C3 | Oracle at its documented size, and with other seeds | `RAF_ORACLE_PROGRAMS=11044 cargo test -p raf-source --locked --release --test rustc_oracle -- --nocapture`; then a scratch copy with seeds `0xb0b0001..8` at 5,522; then a labelled copy that prints every tolerated divergence, run at HEAD and at `5f5049b` | 11,044 programs, 49,283 probes (23,635 accepted, 25,648 refused). Counts: rule1 0, rule2 0, documented 7, order 18, cycle 4, unclassified 0. 5 passed, 1 ignored, exit 0. Other seeds at 5,522 (24,347 probes): rule1 0, rule2 0, unclassified 0, documented 4, order 20, cycle 6. Labelled comparison: all 4 cycle divergences already occur on the pre-fix resolver, which shows 10. 9 tolerated divergences are new at HEAD. 8 of them are the deliberate fixed `Ambiguous` where rustc itself depends on order. The ninth (seed `0xa11ce08` program 1309, `self::B` in `demo::m2::s0`) is the index depending on order. HEAD answers `Ambiguous` or `m0::s0::B` depending on the order, agreeing with rustc in 8 of 25 orders and wrong in both directions. The pre-fix resolver gives one answer and agrees in 17 of 25. The classifier counts this as `order` (see F2). | PASS on the stated counts; see F2 |
| C4 | Capability document | `cargo run -q -p cargo-workplan -- workplan check`; `workplan assess --list`; `grep -n 'not modelled' docs/capabilities/source-index.md`; corpus replays across 13 orders with rustc and the index | `check` exit 0. 56 of 56 assessments accepted and current. The grep is empty (exit 1), and `visibility rules of rustc are not modelled` occurs 0 times. Cycle limit reproduced in both directions. Two corpus files give rustc "cannot determine resolution" while the index resolves (13 of 13 orders), and one gives index `Cyclic` in the type space while rustc accepts the `use` (13 of 13). Order limit reproduced. `glob_order_changes_what_rustc_accepts.rs`: rustc refuses the crate itself in 6 of 13 orders and resolves value `A` in 7, while the index answers `Ambiguous` in all 13. `glob_order_decides...`: rustc's type answer varies with order, and the index stays `Ambiguous`. The companion limit (line 1668) also reproduces, but this change introduced it (F1). Line 1318 says "the answer does not depend on the order of the declarations", which F1 contradicts. | FAIL (F1) |
| C5 | Clock-free claim; whole raf-source suite | `grep -rnE 'std::time\|Instant\|Duration\|SystemTime\|sleep\|recv_timeout\|elapsed\(\|timeout\|UNIX_EPOCH\|...' crates/raf-source/src crates/raf-source/tests`; `cargo test -p raf-source --locked` on a pristine clone | The grep is empty (exit 1). Clock words remain only in `examples/registry_scan.rs`, which is disclosed and tracked as issue 27. Suite: 33 result blocks, 564 passed, 0 failed, 2 ignored (the two hand-run `write_corpus` helpers), doc tests included, exit 0. | PASS |
| C6 | README matches `workplan status` | `workplan status`, `workplan --help` and the subcommand `--help` output, `ls crates` | Status shows 12 delivered (okf-profile, source-index, workplan-cli and 9 workplan-*), 9 designed (census-*), 0 implementing. The README's "twelve capabilities", its table rows, "nine capability documents", "no census code exists" (no census crate) and its command list all match. | PASS |
| C7 | PR 26 body is honest against the evidence | `gh pr view 26 --json body`, checked statement by statement against the runs above | Demonstrated: 0, 0 and 0 at 5,522 and 11,044; the tolerated classes; `resolve_from`; the every-order fixtures; the clock conversion; README; the gates listed as not run. Contradicted or inaccurate, see F3: (a) "rule 1 in 101 programs and rule 2 in 61": the original harness at `5f5049b` gives 101 and 61 *divergences*, not programs, plus 16 unclassified that the body does not mention; (b) the remaining "rare companion-item order dependence" is presented as a residual limit "tracked as separate changes", but rule 2 introduced it (F1), and the issue note "the same at the base commit" is false. | FAIL (F3) |
| C8 | Hosted CI | `gh pr checks 26`; `gh run view 39102290996` | 12 of 12 jobs pass (clippy, cross-check, deny, doc, fmt, fuzz, msrv, reuse, test, test-macos, test-release, test-windows) on head `fc21ada`. | PASS |

## Findings

### F1 (blocking): the rule-2 change makes resolution depend on declaration order, and a debug build panics on valid input; this is documented as a limit and moved to a new issue

- Requirement: issue 14 Scope, "Model both rules in the settling of `use` paths, keeping resolution
  independent of declaration order". The Contract (`docs/capabilities/source-index.md:1318`) also says
  the answer does not depend on the order of the declarations.
- Demonstration on issue 25's own program, all 120 block orders, with `use self::B as probe` in `mm`:

  ```rust
  pub mod m1 { pub mod s1 { pub struct B {} } }
  pub mod m2 { pub struct B {} }
  pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*; }
  pub mod w { pub use crate::m2::B; }
  pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B as probe; }
  ```

  HEAD in debug: `Workspace::load` panics in 20 of 120 orders at `settle.rs` `check_settled` ("the
  imports of `probe` bound {ItemId(1), ItemId(4)} and reach {ItemId(1)}"). HEAD in release: `Ambiguous`
  in those 20 orders and `m2::B` in 100. `origin/main` (debug, `Workspace::resolve`) and `5f5049b`
  (debug, `resolve_from`): `m2::B` in all 120 orders, with no panic. When the declarations inside each
  module are shuffled too (41 orders), HEAD panics in 7 orders, and 4 of those are orders rustc accepts
  (it resolves `m2::B`). Bisect: `e0a75e9` is clean, and `79fb598` (rule 2) introduces it, 7 of 41,
  which persists through `173a2cb`, `1ef2915`, `e5bc866` and HEAD.
- Breadth: a metamorphic sweep of 40,000 generated programs (the committed generator, 8 seeds × 5,000,
  9 orders each, release with debug assertions; scratch `tests/audit_metamorphic.rs`) was run, and each
  finding was rechecked on its rustc-judged (valid) program over 25 orders. At HEAD, 22 valid programs
  give order-dependent answers (19 change the verdict, for example `Ambiguous` against an item). 21 of
  those 22 give one answer in all 25 orders on the pre-fix resolver. Net fairness: the pre-fix resolver
  had 30 order-dependent valid programs, 29 valid-program debug panics, and 240 panics on the unjudged
  programs. The branch removed nearly all of those. The 21 are new.
- Failure shape: LAUNDERED-TO-FOLLOWUP. Constraints line 1668 states the behaviour as a limit, and issue
  25, opened 2026-10-18T04:42Z during this work, carries the fix. The issue requires this item. The
  tracking issue does not predate the session. No user scope decision covers it: the only quoted owner
  decisions are the README update and "Convert timing tests first".
- Required fix: make the glob views and the settled tables agree in every order for the companion shape
  and its variants. Add an every-order fixture for the program above that must pass in a debug build,
  with neighbours, plus a corpus entry. Remove the line-1668 Constraint, or keep only behaviour shown to
  exist at `origin/main`. Rerun the 40,000-program sweep (or the committed equivalent required by F2)
  with no order dependence that the base lacks.

### F2 (blocking): the oracle's `order` class hides index order dependence, and the planned bulk metamorphic check was never built

- `tests/rustc_oracle.rs` `classify` (around lines 354 and 366) returns `Class::Order` whenever rustc's
  answer varies across orders. It never asks whether the index's own answer varies with order, or
  whether the index matches rustc in any order. Seed `0xa11ce08` program 1309 (C3) is index order
  dependence introduced here, and it is counted as tolerated `order`.
- Plan Revision 2, "Enhancement items taken: ... 5 (a bulk metamorphic check: for each generated program
  and a drawn permutation of its declarations, every query and `exported_items` agree)". No committed test
  does this. `Program::shuffled` is used only inside `rustc_depends_on_order`, and no task record drops
  the item. This is the check that would have caught F1.
- Required fix: add the bulk metamorphic test, run in a debug build so `check_settled` runs. In the
  oracle, require the index to give one answer across the tried orders before a divergence is classed
  as `order`. Otherwise it is unclassified.

### F3 (blocking for the handoff text, not the code): statements in the PR and issue that the evidence contradicts

- PR body: "The unfixed resolver showed rule 1 in 101 programs and rule 2 in 61 of 5,522". Reproduced
  with the original harness at `5f5049b`: rule1 101 and rule2 61 *divergences*, documented 3,
  unclassified 16, over 5,522 programs. Required: say "divergences" and give the unclassified count.
- PR body Disclosures, "a rare companion-item order dependence; ... tracked as separate changes", and
  issue comment 2026-10-18T05:56:53Z, "(a debug build's consistency check can stop; the same at the base
  commit; tracked)". `origin/main` has no such stop and one answer for this shape (F1). The T4c review's
  "pre-existing and not worsened" compared against the branch before `e5bc866`, not against the base.
  Required: correct both texts once F1 is fixed, or, if F1 is not fixed, state that rule 2 introduced it.
- Issue 25's body says rustc refuses the program "as ambiguous (E0659) in every order". Measured: E0659
  in 60 of 120 block orders and accepted in 60. Correct it when that issue is next touched.

### F4 (minor)

- The Verification bullet (`source-index.md:2477`) cites "the rustc of the pinned toolchain
  (1.101.0-nightly, 21b707e3f)". `rust-toolchain.toml` names a floating `nightly`, and `AGENTS.md` §2
  says it floats. Say "the toolchain used for the run" instead.

## Scope findings and user decisions

- Explicit owner decisions found: "In your next PR, make sure to update README.md to the current status"
  (delivered, C6) and "Convert timing tests first" (delivered, C5). Neither covers F1 or F2.
- Cycle class: shown to exist before this change (all 4 HEAD cycle divergences occur on the pre-fix
  resolver, which has 10). It is not a new disagreement, and issue 24 tracks it.
- The rule-2 stance (one fixed `Ambiguous` where rustc's own answer depends on order) is a stated design
  choice in plan Revision 1. It is consistent across my 63 probes and the 8 new tolerated divergences it
  explains, and I do not count it as a defect.

## Limits of this evidence

- The sweep and the oracle use one generator family; F1's 21 programs are a lower bound found with
  9 discovery orders.
- `5f5049b` stands in for the base in the sweeps. Its settling is byte-identical to `origin/main`'s (the
  delta is the additive `resolve_from` and a small refactor in `resolve.rs`). The F1 program was also
  checked directly on `origin/main`.
- C1 covers the shapes I wrote (63 probes). It shows agreement there, not in general.
- Scratch artefacts: `/tmp/raf-audit14-s2/` (`c1-run.log`, `c1-run-2021.log`, `c1-run-old.log`,
  `c3-oracle-11044.log`, `c3-oracle-b0b-5522.log`, `head-oracle-labels-11044.log`,
  `old-oracle-11044.log`, `meta2.log`, `meta-old.log`, `only-clone.log`, `only-clone-old.log`,
  `c4-limits.log`, `c5-suite.log`, `newdivs.log`, `alldivs-{HEAD,OLD}.log`, `work/i25blocks/`).

VERDICT: BLOCKED

REQUIRED fixes: F1 (make resolution independent of declaration order again for the companion shape and
the sweep's 21 programs, with no debug-build stop, plus fixture and corpus entry, and the line-1668
Constraint removed or limited to base behaviour); F2 (the committed bulk metamorphic check in a debug
build, and an `order` class that requires an index answer that does not depend on order); F3 (correct
the PR body and the least-confident note); F4 (wording).
