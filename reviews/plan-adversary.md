# Plan completion adversary review

- Reviewed plan: `.stage/raf-workplan-change-lifecycle-cargo/plan.md` (revision 2 header, final
  amendments in place)
- Pass: 3 of 3 (final). It rechecks F5 and F6, the pass-2 observation, and the text their fixes
  added. F1-F4 were discharged on pass 2. That text is unchanged and is not reopened.
- Inputs re-read: plan.md, `issue.md`, the three capability documents,
  `crates/raf-workplan/src/rules.rs`.
- Scope decisions: none needed. The plan delivers the full issue scope (pass-1 choice 1: Scenario
  A/B built here, all three capabilities `delivered`, records completed with the shipped binary).
  Nothing in it is narrowed from the issue.

## Findings

### F1-F4: DISCHARGED (pass 2, text unchanged)

F1 is the delivery of the three capabilities (D2 L38-L45, R22 L174). F2 is the falsifiable R17.
F3 is the `complete` refusal per `ObligationKind::ALL` (R10 L165, R14 L169). F4 is the template
check plus the exact `affects` value (R1 L155, R18 L156).

### F5 (pre-emptive descope, unlimited contract rewording): DISCHARGED

- Baseline (L22-L24): "no Contract, Constraints or Verification statement outside the table is
  removed or weakened. A statement found unbuilt is either built in Task 11 or put to the owner
  before the PR is created; the implementer cannot waive it by recording a reason."
- Task 11 (L240): "(no Contract, Constraints or Verification statement outside the Contract-edit
  table is removed or weakened; an unbuilt statement is built or put to the owner)".

The route for dropping contract text on the implementer's own reason is closed in both places.
The two Contract-edit rows added since pass 2 remove no requested behaviour:

- L128: `complete` "never edits prose" and rewrites only the generated blocks the removal stales.
- L129: a per-stream capture bound.

The issue's "It then deletes the change file" is kept: the record removal stays the last step,
per D9 L91.

### F6 (R1 template check named no capability state): DISCHARGED

- D8 (L79-L80): "`delivered` or `designed` -> 'set `state: implementing` in the same commit' (a
  PROPOSED block is a `check` finding in both states); `considered` and `implementing` need none.
  The R1 template check runs on a capability in each of the four states."
- R1 row (L155): "the printed template placed in a capability (checked in each of the four states,
  after following the printed state guidance) gives exactly one `Graph::proposals()` block, `check`
  exit 0 and one `unresolved-proposal` line from `reconcile` (CLI test)".

The guidance now matches the merged rules (`rules.rs` L138-L142 for `designed`, L143-L160 for
`delivered`). The criterion names its four inputs and three observations. D8's closing "Check:"
sentence (L82-L84) names the `implementing` placement. After the guidance is followed, that is the
state reached from `designed` and `delivered`. The four-state R1 row is the binding acceptance, and
Task 4 tests "per R1".

### Pass-2 observation (states set before `complete`): resolved

- Task 11 (L242): "with the states set to `delivered` by the author BEFORE `change complete`".
- R17 (L172): "with the capabilities set to `delivered` by the author ... `change reconcile` on
  each of the repository's own three records prints an empty list and exits 0; then
  `change complete` is run for each and exits 0".
- D9 (L92) states the pre-deletion and post-deletion judgement that makes this order necessary.

## New text checked on the four grounds

Revision text added since pass 2 introduces no unfalsifiable criterion, descope, unmapped
requirement or acceptance-by-refusal:

- D4 (L54) places the Git version gate in `Lifecycle::open`. The 2.29 refusal is still observed
  on `reconcile`, as the reconcile Contract requires.
- D6 (L64) derives `binding-missing` for `implementing` and `delivered` capabilities.
- D9 (L91-L92) fixes the write order, with fixtures for left-`implementing`, `delivered`, and
  `delivered` with an unresolved binding.
- D11 (L100) runs fixtures on every CI platform.
- Task 10 (L233): Scenario B still ends `delivered`, with the refusals the Verification text names.
- Task 11 (L242) writes the Contract edits as PROPOSED blocks resolved by the self-run reconcile.
- R16 (L171) reads the ignored cargo target directory as outside "the repository". That matches
  the reconcile Constraint, which already has `reconcile` build examples with `cargo`.

Not adjudicated by this lens: Task 12 (L246) pre-declares that `make check` will fail locally on
`cross-check` and cites hosted CI instead. That is a repository-gate question for the constraint
compliance reviewer, not one of the four completion grounds.

## Verdict

All pass-1 and pass-2 findings are discharged with evidence, and no finding remains open. No user
decision is required.

VERDICT: PASS
