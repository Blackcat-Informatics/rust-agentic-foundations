# Plan constraints review

Reviewed plan: `.stage/raf-source-model-rustc-s-visibility-of/plan.md`

## Governing documents

- `CLAUDE.md` (worktree): read.
- `AGENTS.md`: read (sections 2 to 7).
- `.deficiencies`: read. Only the notice and marker, with no entries.
- `.github/PULL_REQUEST_TEMPLATE.md`: read.
- `docs/capabilities/source-index.md` (Constraints, Verification): read.
- `docs/capabilities/workplan-reconcile.md:266-278` (`change new` state guidance): read.
- `tests/docs_contract.rs:77-109` (INVENTORY): read.
- `.gitignore:5` (`/.stage/` is ignored, so the stage notes are not committed files): read.
- Absent: `.baseline`, `.goals`, `CONSTITUTION.md`, `PRECOMMIT_GUIDANCE.md`, ADR directories
  (`docs/adr/`, `docs/decisions/`), and nested `AGENTS.md`/`CLAUDE.md` under `crates/raf-source/`.
  With no standing-constraint documents, the repository rules and session decisions are the
  rules that apply.

User decisions checked: no timing runs (the oracle must not assert or measure time); no
issue or PR numbers and no process wording in committed files; the owner's README instruction,
"In your next PR, make sure to update README.md to the current status."

## Findings

### C1 BLOCKING: the oracle skips when rustc is missing, but a missing tool must fail hard

Plan, Task 1: "Skips with an explicit failure message when `rustc` is unavailable (it is always
available under cargo)."

Governing text:
- `AGENTS.md:153`: "A missing tool is a hard failure, not a skip."
- `AGENTS.md:212`: "No silent fallbacks, no best-effort parsing, no skipped checks."
- `CLAUDE.md`: "Never: ... add a fallback where the design says fail hard".

"Skips with an explicit failure message" can be read either way, and an implementer could build
an early `return` that passes. That would be a skipped check.

REWRITE TO: "When `rustc` cannot be started (the `RUSTC` environment variable or `rustc` on
`PATH`), the test fails with a message that names the missing tool. It never skips or returns
early with a pass."

### C2 BLOCKING: removing only the one bullet leaves the Constraints describing the unmodelled rules

Plan, Task 5 and the completeness contract: "remove the Constraints bullet, state both rules in
the Contract"; acceptance is "`grep -c "visibility rules of rustc are not modelled"
docs/capabilities/source-index.md` is 0".

Governing text, `AGENTS.md:235-236`: "Outside a proposal, the text states the current contract
only."

A second Constraints bullet, the public-surface one, refers back to the bullet being removed and
describes the rule-2 gap as current behaviour of `exported_items`:
`docs/capabilities/source-index.md:1677-1682`: "Of the two visibility rules above that the index
does not model, the first does not reach the surface ...; the second does: a name that the globs
of a module bring in ambiguously, one of the items through a glob that a module which globs it
cannot access, is listed under that module with the other item, where rustc refuses the path."

The plan's grep acceptance passes while this text stays. The text would then point at a bullet
that no longer exists and state a surface behaviour that either no longer holds (if the settling
change reaches `exported_items`) or still holds without being declared anywhere. Either way the
document would not state the current contract.

REWRITE TO (Task 5): "Remove the Constraints bullet 'Two visibility rules of rustc are not
modelled' and rewrite the public-surface bullet (`source-index.md`, 'Of the two visibility rules
above ...') to the delivered behaviour of `exported_items` for both rules. If the listing now
refuses the rule-2 name, say so and bind a surface fixture with its neighbour. If not, state the
remaining difference in its own words, without referring to the removed bullet."
Add to the acceptance: "`grep -c "visibility rules above that the index does not model"
docs/capabilities/source-index.md` is 0."

### C3 ADVISORY: the state change and INVENTORY edit of the lifecycle are not planned

Plan, Task 5: "Reopen `source-index` (`change new`), remove the Constraints bullet, ... `change
complete`."

Governing text:
- `docs/capabilities/workplan-reconcile.md:270-272`: "a `delivered` or `designed` capability is
  set to `state: implementing` in the same commit as the block".
- `AGENTS.md:236-238`: "changing a state or adding a capability edits that table in the same
  commit".
- `AGENTS.md:244-246`: proposed text goes inside a `> [!PROPOSED change:<slug>]` block.
- `AGENTS.md:254`: "Identifiers are slugs, never numbers."

REWRITE TO: "`change new <descriptive-slug>` (a slug that names the behaviour, never a number).
In the same commit, put the new Contract and Constraints text inside `> [!PROPOSED
change:<slug>]` blocks, set `source-index` to `state: implementing`, and change its row in
`INVENTORY` (`tests/docs_contract.rs:89`). At completion, `change complete` deletes the record,
the state goes back to `delivered`, and the INVENTORY row is restored in the same commit.
`self_workplan` must pass with every source-index assessment current and accepted."

### C4 ADVISORY: no `make check` gate before handoff

Plan, Task 7: "Open the PR (`Closes #14`), post the plan and the least-confident note." No task or
acceptance row runs the gate.

Governing text: `AGENTS.md:140`: "Run `make check` before claiming work is done."
`CLAUDE.md`: "Gate before you claim done: `make check`." The PR template (`.github/PULL_REQUEST_TEMPLATE.md:9-11`)
also checks `make check`, the hooks and SPDX headers.

REWRITE TO (Task 7, first step): "Run `make check` at the final head with the hooks installed
(`make hooks`), with no `--no-verify`. Then open the PR." `make check` includes `test-release`,
which runs the raf-source tests a second time, and `cross-check`, which builds the new test for
macOS, FreeBSD and Windows. The default oracle seed list has to compile and pass in both.

### C5 ADVISORY: the oracle needs its own negative fixture

Plan, Task 1: "before Tasks 2 and 3 it reports the known disagreements (a fixed-seed failing case
is captured as a named regression input)".

Governing text: `AGENTS.md:215-216`: "Every rule has a negative fixture. A rule or check with no
input that makes it fail is not enforced."

Once Tasks 2 and 3 land, no committed input makes the oracle report a disagreement. That leaves
its comparator unproven.

REWRITE TO: "Commit a test of the comparator itself: a fixed rustc verdict paired with an index
answer that disagrees must be reported as a disagreement, and the agreeing neighbour must not be.
Keep the captured regression programs as permanent agreeing cases."

### C6 ADVISORY: make the no-timing decision explicit for measuring as well as asserting

Plan, Task 1: "No timing assertions or sleeps." Completeness row: "kept small enough for CI".

Governing text: `CLAUDE.md`: "No timing runs: time-based testing is forbidden. Write no timing
runs, wall-clock benchmarks, sleep-based waits or timing assertions." Session decision: the oracle
must not assert or measure time.

The plan does not say that it measures time. It also does not rule out a wall-clock timeout on
the `rustc` child or a duration in the recorded Task 4 results.

REWRITE TO: "No timing assertions, sleeps, wall-clock timeouts on the `rustc` child or measured
durations. The program count bounds the CI cost. Task 4 records seeds, program count, query
count and disagreement count, never elapsed time."

### C7 ADVISORY: process wording and headers in the new files

Plan: "a fixed-seed failing case is captured as a named regression input"; new files
`crates/raf-source/tests/rustc_oracle.rs` and a generator in `tests/support/`.

Governing text:
- `AGENTS.md:260-263`: "Do not write issue or pull request numbers, tracker links, or wording
  that refers to the development process that produced a file (plans, tasks, acceptance
  criteria, branches)."
- `AGENTS.md:196-198`: code files carry `MIT OR Apache-2.0 OR MulanPSL-2.0` SPDX lines.

REWRITE TO: "Name the regression inputs and fixtures after the program shape (for example
`a_pub_use_of_a_private_glob_binding_is_no_more_visible_than_it`), never after the issue, the
task or 'the known disagreements'. Each new `.rs` file opens with the two SPDX lines.
`crates/raf-source/tests/support/` already exists (`mod.rs`, `child.rs`, `shapes.rs`): add the
generator there as a module."

## Compliant items (no finding)

- README task (Task 6): it follows the owner's instruction. `README.md` already has its SPDX
  header.
- `Closes #14` and the plan/notes posted to the PR: these are not committed files
  (`AGENTS.md:260` covers committed files only), and `.stage/` is git-ignored.
- Declined full-size CI job and the rest of rustc name resolution: neither is part of the issue
  scope, so no deferral is created (`AGENTS.md:266-268`). Keep "out of scope" wording out of
  committed documents.
- No new dependency is planned (the PRNG is in the test, with no clock). If a temp-directory
  crate is added, `make deny` judges it. `CARGO_TARGET_TMPDIR` needs no new dependency.
- No `#![feature]`, no lint change, no census normal-form impact.

VERDICT: NEEDS-REVISION
