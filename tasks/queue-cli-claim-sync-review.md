<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Bounded CLI claim and sync review

Read-only source review covered the current CLI clauses beginning with Record
argument values, their assertion owners, the actual verify process fixture and
the four repaired sync fixtures. The broader historical dirty implementation
was not treated as this review's diff. No product edits or builds were run by
the reviewer.

Two concrete findings were repaired by the implementation owner and reread:

- Assessment replacement asserted supplied SPDX headers, but the final
  headerless replacement checked only its verdict. A stale-header mutant could
  survive. The fixture now asserts empty quiet sinks, an exact one-file changed
  set, the headerless Assessment prefix, absence of SPDX text and the new verdict.
- The production verify comment said Git was never asked anything, while
  Gate::open actually asks the worktree check and tracked-path listing. The
  comment now states that exact boundary.

Quiet command outcomes now form a meaningful clause separate from argument
and policy options. Its markers name the existing actual status, sync, new,
impact, reconcile, complete, preview and verify quiet fixtures. The shared
renderer proof alone would not establish that each handler classifies its
own result correctly; those caller fixtures supply that additional ownership.
This does not invent new quiet behavior or replace its refusal neighbors.

The new verify fixture starts from Repo::real, initializes Git and adds files
without making a commit. Recording wrappers forward to actual Git and Cargo.
It requires the actual passing binding, both complete Git vectors (worktree
check and tracked listing), both complete frozen Cargo vectors (list and run),
empty stderr and an unchanged full fixture snapshot. The inspected production
route is command::run to change::verify to Gate::open/lifecycle::verify and the
library SystemRunner. No independent spawn implementation was found in the CLI
source. The existing no-shell unit compares NUL-separated arguments and absence
of an injected file; its actual nested symbol path is used by the marker.

The repaired sync setup first uses the library to make the capability's blocks
current, then removes only the index or empties the actual verification matrix.
The generated-block fixture requires a stale finding, the exact alpha-only
changed path, presence of the resolved test binding, byte preservation outside
the matrix and a clean subsequent check. Both invocation forms start from the
same snapshot through both_writing and must produce equal reports and trees.
The refused-index fixture contains real pending matrix repair work, so its
unchanged snapshot proves that a refusal does not publish that repair; restoring
the valid index gives the accepted neighboring write. Index creation/rooted
fixtures now isolate index creation rather than accidentally asking to repair
other stale blocks.

Production sync still computes and validates its complete report before calling
Workplan::write. The writer still validates every target, stages every new text,
then publishes; the documented partial-rename failure boundary is unchanged.
The test changes neither bypass these callers nor claim broader transactional
publication guarantees.

No unresolved concrete defect was found in this bounded final source read.
The reviewer inspected the actual passing one-test verify log. The owner
separately reported repaired sync (11 tests), bound owners (70 tests), the real
no-shell test and all-target Clippy passing, followed by the strengthened
assessment test and fresh Clippy. Those are owner execution results, not tests
run by this reviewer. The final generated-document sync was also reported
passing. The active delivered-state candidate is not a result of this review.

Limits remain explicit: the recording fixture is Unix-only; no independent
qualification of other platforms, write races, all I/O publication failures,
full workspace gates, claim inventory currency or repository adoption was
performed here. Marker resolution and passing tests do not by themselves prove
every sentence of a broader capability. Existing separate refusal and valid
neighbors, actual caller routing and source evidence remain necessary.
