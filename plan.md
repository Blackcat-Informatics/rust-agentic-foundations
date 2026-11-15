# Plan: strict review mode (tracker issue 268)

Base: origin/main 9824b56 (includes the measured-evidence change). Worktree: .worktrees/268-workplan-add-a-strict-review-mode-that. Build lock: /tmp/raf-build.lock.
Verbatim issue: `stagectl issue 268` (acceptance below is copied from it).

## Behavior
- A capability gains an optional key selecting strict review. Under it the review requirement is satisfied only by a review whose verdict is clean and whose input digests are current.
- The review status carries the verdict so the finding can say which condition failed: a verdict with findings, or a stale digest.
- A later review with findings shadows an earlier clean one for the same scope and kind; the design states this rule and tests it.
- Without the key, behaviour is unchanged.

## Explicitly not included
A review written after the change it covers. Nothing links a review to a change record; digest staleness already covers code that changes after a review.

## Acceptance (verbatim from the issue)
- Under strict review, a review with findings does not satisfy the requirement and a clean review at current digests does, side by side.
- A clean review whose digests are stale does not satisfy it.
- A capability without the key is accepted exactly as before.

## Facts from the code (verify before relying on them)
- review_status: crates/raf-workplan/src/review.rs (latest non-blocked review, digest comparison; verdict unread today).
- required review finding: crates/raf-workplan/src/strict.rs check(), beside required_reviews.
- ReviewStatus has public fields and is not non_exhaustive: a deliberate addition; update the contract text and its assessment.
- Closed key sets: crates/raf-okf/src/profile/ (the capability key goes there, as the measured-evidence key did).

## Compatibility
An optional key on a closed schema: existing documents unaffected; an older binary refuses a document that states it.

## Exclusions
No Git reads, no process runs, no other keys, no review-to-change linkage.

## Documents
Update the capability document(s) for review and workplan strict contracts; open a change record with cargo workplan, complete it with change complete; renew stale assessments only after reading each section against the change, restoring the two licence lines. No tracker numbers or process wording in committed files.

## User rulings
- Coordination (standing direction as Patrick's): build order 268 after 267 (merged), then 269; each its own reviewed PR; no second writer on the same schema files at the same time.
