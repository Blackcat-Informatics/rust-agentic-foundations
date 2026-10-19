# Plan final review (Stage 1), pass 2 of 3

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/27-raf-source-the-registry-scan-example/.stage/raf-source-the-registry-scan-example/plan.md` (Rev 2)

Reports examined: `reviews/plan-completeness.md` (F1-F8), `reviews/plan-constraints.md` (BLOCKING 1,
ADVISORY 1-2), `reviews/plan-design.md` (1-5), `reviews/plan-enhancement.md` (1-8),
`reviews/plan-adversary.md` (A1-A7), `issue-analysis.md`, `prior-art-assessment.md`, and this
review's pass-1 findings. Dispositions: the caller's note that the user took the counted-work
budget as the answer to A1 / F1 / enhancement 1; every other disposition is read from the plan.

Code read: `crates/raf-source/src/{budget,options,meter,work,parse,settle,resolve,glob,
glob_scope,exports,workspace,error}.rs`, `crates/raf-source/examples/registry_scan.rs`,
`docs/capabilities/source-index.md` L1700.

## 1. Pass-1 findings, rechecked against Rev 2

### BLOCKING 1 (charge-site set and meter reach): DISCHARGED
- Design L29 is the pass-1 rewrite: the unit is the glob-table work charged beside
  `charge_work`/`meter.globs` in the view gather, `pass_on`, `arrive` and `holding`'s `step`,
  all of which hold `&Meter` and return `Result<_, LoadError>` (`glob_scope.rs` L1472-1483,
  L1531-1552, L1784, L1862, L1131-1134); `Meter::work` refuses at the charge that passes
  `max_work`, so "the metered count at the refusal equals the limit plus the last charge" (L66)
  is now true of the design. `Budget::Work.name()` is "glob table work steps" (no clock word).
- The five `cfg(test)` kinds and the parse-thread kinds are stated as unbudgeted, with the reasons
  that match the code (`&Meter` not `Sync`; `Lookups`/`SegmentReads` also run under
  `Workspace::resolve`; `WalkSteps` belongs to `exported_items` on its own meter).
- Task 1 Files L62 names `glob_scope.rs` and the sites, states that `work.rs` and its 21 sites are
  unchanged, and names the `source-index.md` sentence "No budget bounds the time of these loads"
  for rewriting.
- Task 1 Tests L67 adds the count-only growth test (hub doubling multiplies `Usage::work` by more
  than 4; prelude and private fan-in by more than 2; a plain chain at most doubles) and the
  `pass_on`-only mutant. This is the check that the site set sees the superlinear cost.

### BLOCKING 2 (refusal strings): DISCHARGED
Design L41, R1 row L53 and Task 2 L74 now expect `ERR Budget :: <file>:<line>: exceeds the budget
for glob table work steps (<limit>)` and `# err Budget=1`. Verified against the code:
`load_one` prints `ERR {Debug variant} :: {error}` (`registry_scan.rs:373-377`), `variant()` of
`Budget { .. }` is `Budget`, `LoadError`'s Display is `{file}[:{line}]: {kind}`
(`error.rs:525-536`) and the kind prints `exceeds the budget for {budget} ({limit})`
(`error.rs:427-428`) with `{budget}` = `Budget::name()`. Exact match.

### ADVISORY 3 (time word in R1): DISCHARGED (L53 uses the count criterion).
### ADVISORY 4 (enhancement items 4, 5, 7 undispositioned): DISCHARGED (L94, each with a reason and a cost; the decline of 7 rests on R6's three planted-word fixtures, which is a sound reason for the raf-source directories this issue is about).
### ADVISORY 5 (`--max-work 0`): DISCHARGED in Task 2 L74 (refused with a message naming the flag; `--max-work 1` over the tiny crate prints `OK ... work=0`). Residual, advisory only: Design L39 still says `--max-work N (default the budget)` without the minimum the check relies on.
REWRITE TO (L39): `takes \`--max-work N\` (a whole number of at least 1; default the limit of \`Budget::Work\`)`.
### Coherence note (Task 4 files): DISCHARGED (L78).

## 2. Lens findings (unchanged from pass 1)

All BLOCKING findings of the five lenses remain discharged or moot in Rev 2: completeness F1-F5,
constraints B1, adversary A1-A7; design 1-3 applied to the removed CPU limit and nothing in Rev 2
reintroduces a signal reading, a timer or a CPU rlimit. Advisory items F6-F8, constraints ADV1-2,
design 4-5, enhancement 2-8 are addressed, moot or now dispositioned (L94).

## 3. New in Rev 2

### ADVISORY 6: a leftover sentence in Design bullet 3 reads against bullet 1
L33-34: "The tally moves from a `cfg(test)` thread-local to the `Meter` the load already threads
(a `Cell<usize>` charged at the sites; ...)". With bullet 1 and Task 1 L62 ("The `work.rs` tally
and its 21 sites are unchanged") the budgeted counter is new and separate; nothing "moves". An
implementer reading bullet 3 first could take "the sites" to be the `work.rs` sites.
REWRITE TO (L33-35): `* The budgeted counter is a \`Cell<usize>\` on the \`Meter\` the load already threads, charged only at the glob-table sites of bullet 1; the \`cfg(test)\` per-kind tally of \`work.rs\` stays as it is, so the existing exact counts keep their meaning. The default limit ...`

No contradictions found between the contract rows and the tasks, or between the changed Task 1/2
text and the unchanged Tasks 3-5. Task order and dependencies hold (index, example, gate,
calibration, PR); no Task 0; the final task makes the PR.

## 4. Open user decisions

None.

VERDICT: PASS
