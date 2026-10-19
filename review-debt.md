# Review debt: PR 30 and issue 21

Head: 3ab002d39a7bd224f82803e7c3232a84534db42d (matches `headRefOid` of PR 30; state OPEN).

## Commands run
* `gh pr view 30 --json reviews,comments,headRefOid,state` -> `reviews: []`, `comments: []`.
* `gh api repos/Blackcat-Informatics/rust-agentic-foundations/pulls/30/comments --paginate` -> `[]` (no inline comments).
* GraphQL `pullRequest(number:30){reviewThreads(first:100){nodes{...}}}` -> `nodes: []` (no threads, so none unresolved).
* `gh issue view 21 --comments` -> two comments, both by the issue author (paudley); dispositions below.
* `gh pr checks 30` -> 12 checks, all pass (below).

## PR 30
* Review submissions: none (including bots). Nothing to disposition.
* Inline review comments: none (including bots).
* Unresolved review threads: none.
* PR conversation comments: none (including bots).

## Issue 21 comments
1. Scope comment (raf-source has its own elapsed-time tests in resolve_paths, manifest_caps, trait_keys, module_tree, exported_items, prescan_budgets, symbol_index, facts, tree::disk; each needs conversion to an operation or allocation count).
   Disposition: addressed by the already merged source-index change, as the plan comment records (work tallies, counting tree, clock-free tests, gate `time_in_raf_source`). Evidence on this head: `grep -rnE 'Instant|SystemTime|sleep|recv_timeout' crates/raf-source/src crates/raf-source/tests` prints nothing. The clock gate now judges every crate from `crates/*` (commit e9aeae3).
2. Plan comment (plan, Revisions 1 to 3: tasks 1-10, completeness contract, declined items).
   Disposition: addressed. It is a plan, not a request for change; the branch has 20 commits over main implementing it. Items it explicitly declines stay declined for the reasons it gives (coverage exclusion index, counting newtype accessors, `perf stat` make target, shared cost-model crate, public `Clock` trait, event-driven wait). The plan's own acceptance (full `make check`, per-mutant results, listing of surviving comparisons) is recorded in `validation.md` in this directory, not restated here.

## Required checks on the current head (`gh pr checks 30`)
clippy pass, cross-check pass, deny pass, doc pass, fmt pass, fuzz pass, msrv pass, reuse pass, test pass, test-macos pass, test-release pass, test-windows pass. 12 of 12 pass; none pending or failing (run 39182413952).

## Outstanding debt
None.
