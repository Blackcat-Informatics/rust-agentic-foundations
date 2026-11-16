# Plan: failing measured records and unnamed units (tracker issue 272)

Base: origin/main 187f4d0 (includes measured evidence and strict review). Worktree: .worktrees/272-workplan-a-failing-measured-record-does. Build lock: /tmp/raf-build.lock.
Acceptance is copied from tracker issue 272.

## Behavior
1. A measured claim is satisfied only by a record whose verdict passes at the current digests. A record whose verdict fails is reported as its own finding, distinct from "no measurement", and fails the strict check.
2. A unit that names no value is refused at load, with a message that names the claim and the missing value, beside a valid neighbour.

## Acceptance
- A capability requiring measured evidence is not satisfied by a record with a failing verdict, and the finding names the failing record; a passing record beside it satisfies the requirement.
- A record whose verdict passes at stale digests does not satisfy the requirement.
- A unit that names no value is refused, beside a complete record that is accepted.
- A capability without the requirement is judged as before.

## Facts (verify before relying on them)
- The measured-evidence requirement lives in crates/raf-workplan/src/strict.rs (check()) and the evidence record type in crates/raf-okf/src/profile/ (units and conditions added by the measured-evidence change).
- The verdict is the evidence field recorded per record; the existing review verdict path (review_status) is a different path and is not changed here.

## Exclusions
No review-verdict change (strict review is done). No Git reads, no process runs.

## Compatibility
A record that fails now stops satisfying a capability that requires measured evidence; that is the intended effect. Passing records and records without the requirement are unaffected.

## Documents
Update the evidence and measured-claim capability documents; open a change record, complete it with change complete; renew stale assessments only after reading each section against the change, restoring the two licence lines. No tracker numbers or process wording in committed files.

## User rulings
- Coordination (standing direction as Patrick's): 272 follows 268 in lane 1; its own reviewed PR; both findings in one change.
