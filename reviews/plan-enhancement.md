# Enhancement audit: plan.md

Reviewed plan: `.stage/m2-rest-of-the-planning-essay-entity/plan.md` (revision 1 audited; revisions 2
and 3 rechecked; final recheck against the revision-3 text that carries the owner's evidence decision).
Inputs read: issue.md, issue-analysis.md, prior-art-assessment.md, requirements.json, AGENTS.md
(no `.goals` file exists in the worktree), the five M2 capability documents and their change
records, `docs/architecture/okf-profile.md`, the delivered reconcile and complete-gate contracts,
and the code the plan says it reuses (`obligation`, `obligation/sources`, `lifecycle/{session,
complete,completion,untouched,verification}`, `bound/evidence`, `git`, `exec/invocation`, `graph`,
`model`, `gate`, `finding`, `raf-okf` `profile/schema`, `profile/typed`, `body/marker`,
`body/prose`, `raf-source` `item`, `workspace`).

## The concept the five features are shadows of

The revision-1 plan delivered five parallel features with three bespoke record shapes (D1) and
three bespoke staleness rules: evidence went stale by `git diff --name-only <revision> HEAD` over
bound files (D6), assessments by one combined SHA-256 over three input classes (D9), and coverage
exclusions never went stale (D10). Every one of these is an instance of the same thing: a
**verifying trace** in the sense of incremental build theory (a record that says "verdict V was
reached from inputs I1..In whose digests were H1..Hn"), and the five capabilities together form
an **assurance case** (claims, supporting evidence, judgements over the argument, and the
assumptions whose defeat would reopen a decision), laid over the graph `raf-workplan` already
builds. Items 1 to 3 are that design.

## Items (revision 1)

### TRANSFORMATION

1. **One verifying-trace record model for evidence, assessments and coverage exclusions.**
   One record shape in `raf-okf` shared by the new types: `subject`, `inputs` (rows
   `{kind, name, digest}`), a type-specific `verdict`, `by`. Staleness for every record is one
   function: recompute each input's digest now and compare; the obligation names the input that
   changed. Evidence records carry the digests of the bound code of the capabilities whose claims
   cite them, which is what D6 approximated with a file-level `git diff` against a recorded
   revision. This removes the recorded-revision dependency, which matters because this repository
   merges by squash (`(#18)` on `main`): after a squash merge or a rebase the revision in every
   evidence record is unreachable, `git diff <revision> HEAD` fails, and under fail-hard every
   later `reconcile` errors. Four of the five new obligation kinds become static kinds raised by
   one `trace` source in `Workplan::reconcile`; only the merge-base comparison stays in
   `Lifecycle::reconcile`. Coverage exclusions become traces too (input = the excluded item's
   span digest). Cost: L (net smaller than D6 + D7 + D9 + D10 written separately). Payoff:
   decisive. Where: D1, D3, D6, D8, D9, D10, Tasks 1, 2, 4, 5, 6.

2. **Claims as first-class support edges, with several support kinds, and conditions to
   revisit as negated claims.** The `raf:claim` marker should be able to name any support
   (`evidence=`, `tests=`, `examples=`), giving a statement-level "tests -> claims" edge; the
   graph already infers section-level substantiation from code spans (`graph::substantiated`).
   A condition to revisit is a defeater; where the author can state it as a measurable claim,
   the same marker lets `check` raise `condition-met`. Cost: M on top of item 1. Where: D5, D10,
   D11, Declined bullet 3, Task 7; also the vocabulary clash between `items[].claim` and the
   document claim.

3. **One shared `Digest` with per-input digests, canonical encoding, and `SectionDigest`
   underneath.** SHA-256 over a domain-separated, length-prefixed encoding; `section` from
   `SectionDigest::text()`; `symbol` from the source-index span with CRLF normalised; `record`
   from the record text; one digest per input persisted so the obligation names the input that
   moved. Cost: S/M. Where: D9, Task 5.

### LEVERAGE

4. Parse the claim marker in `raf-okf` `body/marker.rs`, not in `raf-workplan`. S. Task 2.
5. Per-section assessment inputs from the code spans the section names, falling back to all
   bound code (`cap:workplan-complete-gate` binds 13 code items; any refactor of any one would
   otherwise stale all seven assessments). M. D9, Task 5.
6. Let `evidence record` take a binding through `bound::Executor` and the gate's typed rows, and
   resolve the clash between `raf_workplan::bound::Evidence` and the new record type. M. D13.
7. Coverage under `work/coverage/` (a directory of the type), not a file in `work/`. S. D1, D10.
8. Decision pages by a typed structure; the unlinked-page rule as a filter over `graph.links()`.
   S. D11, Task 7.
9. `history <id>` over any loaded document id, the new record types included. S. D12, Task 3.

### UTILITY

10. A coverage closure rule (a binding covers its descendants; `doc(hidden)` out; "reachable
    from the crate root" needs an export walk `raf-source` does not expose). M. D10, Tasks 6, 9.
11. A generated `claims` block per capability and coverage counts in the catalogue. M.
12. Complete the `evidence record` command line (`--limit`, re-record keeps limits, verdict
    definition, value grammar). S. D3, D13, Task 8.
13. The reverse-direction findings `evidence-unused`, `exclusion-redundant`,
    `assessment-orphan`. S. Tasks 2, 6.

### ROBUSTNESS

14. D6 fails closed after a squash merge, a rebase or in a shallow clone (subsumed by item 1).
15. The history argv rows do not produce the "touched paths" D12 promises: no `--name-only` on
    the `--follow` row, and Git documents `--name-only` and the other non-patch formats as not
    implemented with `-L` (`--no-patch` is honoured); the series routing `-L` through the
    standard diff pipeline is 2026 work, far newer than the 2.30 minimum. S.
    Sources: [git-log documentation](https://git-scm.com/docs/git-log),
    [git-log(1) Linux manual page](https://www.man7.org/linux/man-pages/man1/git-log.1.html),
    [Git Rev News Edition 134](https://git.github.io/rev_news/2026/04/30/edition-134/),
    [line-log: integrate -L with the standard log output pipeline](https://ratatoskr.run/git/2026/04/8035335/t).
16. "A binding that no longer resolves is listed with the last commit in which it did" has no
    mechanism. M. D12, Task 3.
17. Re-recording must never execute stored command text. S. D13.
18. A command that cannot be observed to its end yields no verdict. S. D13.
19. Define the rejected-assessment obligation and the sixth gate condition. S. D8, D9, Task 5.
20. `DocType` is an exhaustive public enum with `ALL: [Self; 3]`; adding variants breaks. S. D1.
21. Symbol history: dedupe `-L` queries by source key; `-L` ends at a file rename. S.
22. Claim markers inside PROPOSED and generated blocks are not current claims. S. D5.
23. `assess` is the last authored step before `complete`; record directories load in
    `Opened::workplan_after`. S. Task 10.

## Recheck: revision 2

| Item | Disposition in revision 2 | Mark |
| --- | --- | --- |
| 1 | Adopted (D1 trace shape, D4 inputs, D5 digest staleness, D7 one static `trace` source, D9 exclusions with a digest). | DISCHARGED |
| 2 | Declined on a recorded design reason; residual: closed `key=value` marker grammar. | DISCHARGED |
| 3, 4, 7, 8, 10, 14, 15, 16, 17, 18, 19, 20, 22, 23 | Adopted or resolved by construction. | DISCHARGED |
| 5, 6, 9, 11, 12, 13, 21 | No disposition. | STILL-OPEN |

New in revision 2: N1 (no-record default enforced the whole workspace: breaks every fixture and every
downstream gate on upgrade; proposed one `coverage-record-missing` finding, `scope: []` as the
visible opt-out, a fixture helper) and N2 (`--for` duplicates the citation the claim marker carries).

## Recheck: revision 3 (first pass)

Items 5, 6, 9, 21, N1, N2 and the item-2 residual adopted in the text; 11, 12, 13 declined with a
reason. One blocking point: D1/D9 dropped the exclusion digest that revision 2 had adopted as part of
item 1 without a Declined entry. Residuals: no R2 row for `--from-binding`; no R1 row for the record
ids; a limit removed at the merge base not in D6; the `-L` rename limit not stated.

## Recheck: revision 3 (final, against the text carrying the owner's evidence decision)

| Item | Disposition now | Mark |
| --- | --- | --- |
| 1 | Evidence and assessment traces, the static `trace` source and digest staleness intact (D1, D5, D7). The coverage-exclusion digest is now declined with a technical reason ("an item digest changes on any edit, so exclusions would reopen more often than they rot; an exclusion that matches nothing is already a finding"). That is a sound trade: the reopening would be noise, and the rot case the digest guarded (an item that silently grew) is partly covered by the unmatched-exclusion finding. | **DISCHARGED** |
| 2 | Declined on the recorded design reason; the closed `key=value` grammar is in D2. | DISCHARGED |
| 3 | D3 unchanged. | DISCHARGED |
| 5 | D8: the symbols the section names in code spans, else all code-bound symbols. | DISCHARGED |
| 6 | D4: `--from-binding` through `bound::Executor`; `EvidenceRecord`; R2 now has the rows (a path matching no binding exits 2 with nothing started; the stub proves the gate's rows). | DISCHARGED |
| 9 | D11 ids include `evidence:` and `assessment:`; R1 lists them. | DISCHARGED |
| 11, 13 | Declined ("none is in the Contract"); for a generated block kind and reverse-direction findings beyond the candidate Contract, a sound boundary. | DISCHARGED |
| 12 | `--limit` adopted; inheritance declined ("a record is a result"); D6 now raises `evidence-changed` for "a limit present at the merge base and absent now", which closes the only way a bound could leave the gate's view silently. | DISCHARGED |
| 17 | Owner decision: `evidence record -- <argv>` runs the command (D4). D5: the stored command text and revision "neither reach a process"; R2: "the shared `exec` trusted variant is never reachable from document text". The stored text is informational only. | DISCHARGED |
| 18 | D4: verdict is exit status 0 = pass, otherwise fail; R2 tests "timeout kill" and "a failing command still records a `fail`". This follows the delivered gate's own precedent (a run killed at its time or output bound is a failing verdict; a process that cannot be started is an error, not a verdict), which is the right precedent to follow; my revision-1 wording asked for the stricter reading and the precedent wins. Residual, non-blocking: say in D4 which of the two a kill is (a `fail` record) and that a process that cannot start writes no record and exits 2. | DISCHARGED |
| 21 | D11 de-duplicates commits across sources; the `-L` rename limit is in the edit list ("the `-L` log ends where the file's history is renamed; stated in Constraints"). | DISCHARGED |
| N1 | Owner-decision paragraph (`coverage-record-missing`, `scope: []`), D9 helper, D19 ordering, R4 row. | DISCHARGED |
| N2 | D4: `--for` fixes what the evidence covers, the marker is the citation, both required, mismatch a finding. | DISCHARGED |
| all others | Unchanged from revision 2. | DISCHARGED |

New design reviewed in this pass: D4's `Program::Trusted` variant (the caller's own argv, no shell,
stdin null, the 1800 s bound, 16 MiB per stream, the workspace as working directory, a fifth stated
form named in SECURITY.md; values are `name=number` stdout lines and nothing else is persisted) is
consistent with the trust model and with the Executor's limits, and it is what the capability's
Purpose asks for ("records executions as evidence"). D9's change that `cfg(test)` unit tests count
like other tests and are excluded by prefix with a reason follows the Contract's own wording
("excluded with a reason") and is tractable for this repository (a handful of `::tests::*`
prefixes). No new enhancement item arises.

Residuals for the author, none blocking and none an enhancement: D12 still says "The tool starts
no process other than `git` (and the already-built `cargo` rows)", which D4's trusted variant now
contradicts (one clause); R4 still says "`cfg(test)` out", which D9 no longer says (one clause); and
the D4 kill-versus-cannot-start wording under item 18 above.

### Recheck summary

Every item of this audit is now adopted in the plan text or declined with a technical reason, the
three TRANSFORMATION items included; the revision-3 reversal is now a recorded decline. The plan is
maximal for its scope at the altitudes this audit covers.

VERDICT: PASS
