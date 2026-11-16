# Validation for issue 232 (PR 283)

Merging head: 20ff8da4 (merge base origin/main 4369c9cc).

## Hosted CI
- Run 43963860143 at 20ff8da4: every required check passed, and the PR merge state was CLEAN (checked after the run completed).
- Earlier run 43947704759 at 49124831: all required checks passed, including test shards for raf-source, raf-workplan, raf-census, cargo-census, test-release, fuzz, msrv, doc and clippy.

## Local hook
- The pre-commit hook passed for each commit on the branch. For 20ff8da4 the log ends EXIT=0 (docs_contract 29, repo_gates 81, self_census 4, self_workplan 1).

## Focused cargo runs
- a802766d: module_tree 51-52, reachability 30, configured_sources 17, all exit 0 under the build lock (review-a802766d.md records each).
- 49124831: cargo test -p raf-source --locked --test reachability exited 0 (29 passed).

## Independent review
- review/review-a802766d.md: BLOCKED on one documentation sentence, and no behaviour finding. That sentence was corrected in 20ff8da4. Under the repository rule a wording-only change needs no further independent read.

## Limits
- Not run locally: make check, make test-release, and the raf-census and raf-workplan suites beyond what the hook and CI run.
- Not run: rustc on stable or the 1.98.1 minimum; other platforms.
