# Review debt: PR 31 and issue 27

Head checked: 56ffc9fdcd2e48426bf7b453936224e8c5599287 (PR state OPEN).

## Commands run

* `gh pr view 31 --json reviews,comments,headRefOid,state`
* `gh api repos/Blackcat-Informatics/rust-agentic-foundations/pulls/31/comments --paginate`
* `gh api graphql` with `reviewThreads(first:100)` on pull request 31
* `gh issue view 27 --comments`
* `gh pr checks 31`

## Review submissions

None. `reviews` is `[]` (no human or bot review submitted).

## Inline review comments

None. The pulls/31/comments call returned `[]`.

## Review threads

None. `reviewThreads.nodes` is empty, so there are no unresolved threads.

## PR comments

One comment (IC_kwDOVFffUs8AAAABdRa24w, author paudley): the "Plan (Rev 2)" post. It is a plan post, not a request, so no disposition is owed. No bot comments.

## Issue 27 comments

One comment (author paudley): the same "Plan (Rev 2)" text. It is a plan post, not a request, so no disposition is owed. No bot comments and no other comments.

## Required checks on the current head (`gh pr checks 31`)

Checks are still PENDING on head 56ffc9f. They have not finished, so this is not a pass.

* pass: fmt (16s), reuse (9s)
* pending: test-macos, clippy, cross-check, deny, doc, fuzz, msrv, test, test-release, test-windows

Re-run `gh pr checks 31` until none are pending before merge.

## Open dispositions

Addressed: 0. Declined: 0. Unaddressed: 0. There is no review debt, apart from the pending checks above.
