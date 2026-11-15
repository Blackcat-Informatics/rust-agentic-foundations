# Plan: measured evidence requirement (tracker issue 267)

Base: origin/main f7b87db8. Worktree: .worktrees/267-workplan-let-a-capability-require. Build lock: /tmp/raf-build.lock.
Verbatim issue: `stagectl issue 267` (captured in this stage directory).

## Behavior
- Evidence records may carry optional `units` (one unit per measured value name) and `conditions` (how and where the values were taken).
- A measured record (declares values) with a value lacking its unit, or lacking conditions, is refused at load, beside a complete neighbour.
- A capability may declare an optional key requiring at least one evidence record carrying measured values with units and conditions. A finding raised beside `required_reviews` in `crates/raf-workplan/src/obligation/strict.rs` makes `check` and `change complete` refuse without it. `change complete` judges `check()` with the record removed, so no separate completion code.
- A capability that does not opt in behaves exactly as before.

## Exclusions
- Decisions get no declaration surface (the decision-chain work is a separate, unscheduled issue).
- No Git reads, no process runs. No other new keys.

## Acceptance
1. A capability requiring measured evidence is refused by `check` and `change complete` when no evidence record carries values, units and conditions; a neighbour with them is accepted.
2. A measured record with a value missing its unit, or with no conditions, is refused beside a complete one.
3. A capability without the key is accepted exactly as before.

## Compatibility
Optional keys on closed schemas: existing documents unaffected; an older RAF rejects a document stating the new keys.

## Tests
Extend existing behavioural cases where possible (workplan scenarios and the evidence/strict tests); each refusal beside a valid neighbour. No clock, no mutation testing, fixtures under CARGO_TARGET_TMPDIR.

## Documents
Capability document(s) for the evidence contract and the capability schema updated; assessments renewed only after reading each section against the change, with licence lines restored. `change complete` for the record.

## User rulings
- Coordination (standing direction as Patrick's): build order 267 (measured claim), then 268 (strict review), then 269 (artifact evidence), each its own reviewed PR after 209; state the tracker issue in plain terms only.
