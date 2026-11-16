# Validation: issue 207 (foreign host import detector)

VERDICT: PASS

Head 815dfd94 was independently reviewed (review-815dfd94.md, PASS, no required findings). Hosted CI on 815dfd94: all 25 checks passed. The merge preflight reported no blockers.

## Scope

Seven commits on paudley/207-census-one-home-per-foreign-host-import, 1604c1a5 through 815dfd94, merge base f7b87db8. The diff touches the census detector, its gate wiring, self-test fixtures, one gate test file, the census capability documents and the renewed assessments. The base and the branch change disjoint files, `git merge-tree` is clean, and CI ran on a merge ref that already contained the latest main.

## Evidence

- Commit hook for 815dfd94 (fix9-commit3.log in the worktree run directory), read by me: repo_gates 29 passed, docs_contract 81 passed, self_census 4 passed, self_workplan 1 passed, exit 0.
- Independent review runs at 815dfd94, all exit 0: raf-census census_gate 87 passed, raf-census library 102 passed, cargo-census self-test delivery 2 passed, the shipped self-test binary 56 cases and 0 failed, clippy with warnings denied on raf-census, prose-check on the changed documents and commit messages.
- Hosted CI: 25 of 25 checks passed on the PR head.
- The four open review threads were each addressed in a pushed commit; two were resolved on the forge and all dispositions are recorded.
- Source reading by me of the merging diff: the detector wiring (`detectors/mod.rs`, `gate/context.rs`, `gate/rules.rs`, `gate/evaluate.rs`, `rule.rs`, `selftest/fixtures.rs`) and the detector header and entry points. The detector contains no `unwrap`, `expect`, `panic!`, `unsafe` or printing. Not read by me: the remaining detector body, the gate test file and the documents, which the independent review covered.

## Findings left open

Seven optional review findings are tracked in issue 276: namespace merging of block and item attributes, a macro inside an extern block, undocumented and untested refusals, the `cfg` form for `link`, the job-versus-Gate wording of a refusal, the Purpose framing and line wrapping.
