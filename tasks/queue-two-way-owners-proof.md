<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Counted two-way coverage proof

## Completed focused results

Latest settled candidate: session 92753 returned terminal exit 0 after current
warnings-denied all-target Clippy and production sync. Exact logs are
`raw/queue-two-way-producers-clippy-current.log`,
`raw/queue-two-way-producers-docs-sync.log` and
`raw/queue-adoption-two-way-producers-candidate.log`; output lives in
`raw/purrdf-adoption-two-way-producers/`. There are 60 findings, 29 uncovered
claims (source-index 28, coverage 1), zero code/test gaps and zero orphan,
excepted or debt lines across all eight packages. The other 31 findings are the
same 30 delivered/open-record and one catalogue mismatches of the diagnostic
overlay. Product states remain implementing and 59 assessment refusals remain
open. No root-gate, lifecycle or full-gate completion is claimed.

- Session 41690 exited 0: exact target receipt at the actual exported-items call
  and inverse-name binary visibility/report repair, one selected test each.
  Logs: `raw/queue-two-way-phase-receipt-current.log` and
  `raw/queue-two-way-binary-visibility.log`.
- Session 67400's role-only test passed: test binding and documented-library
  example binding each reach the actual outside-scope package, with exact gap
  kinds/attribution and the fully covered neighbor. Log:
  `raw/queue-two-way-binding-roles-current.log`.
- The same session's Clippy stopped on the receipt guard declaration order.
  The guard declaration moved before statements; session 46307 exited 0 for
  warnings-denied raf-workplan all-target Clippy. Current log:
  `raw/queue-two-way-owners-clippy-current.log`.
- Session 36778 exited 0: actual cyclic-import and empty-public-glob producers
  remain unresolved report errors, never complete empty surfaces, beside actual
  listable neighbors. Log: `raw/queue-two-way-refused-producers.log`.
- Production docs sync passed in `raw/queue-two-way-counted-docs-sync.log`.

Independent read-only review:
`tasks/queue-two-way-counted-owner-review.md`. It distinguishes owner execution
from reviewer source inspection and corrects its earlier premature example-role
description. No full gate or completed adoption is claimed.

## Preserved diagnostic failures

Session 64950 refused the receipt fixture's expected two unreadable findings:
load lists exclusion-named packages but enforces that finding only for scope.
The repaired receipt asserts the actual one binding-phase finding at the exact
capability bindings key. The original log is `queue-two-way-phase-receipt.log`.

Session 50690 correctly refused the example binding because its helper had no
documentation. A real same-line doc comment was added only to this test's owned
source; the existing wrong-role refusal was not weakened. Original log:
`queue-two-way-binding-roles.log`.

The first candidate invocation refused before running because the retained
helper manifest has several binaries. Session 19164 explicitly selects
`--bin adoption-probe` returned exit 0 with 62 findings and 29 uncovered claims,
correctly exposing the new unbound producer test and its 43 orphan test lines.
Its actual binding was added and generated blocks synchronized before the
settled replay above. The first final Clippy attempt (86424) refused the new
empty-array assertion style; the corrected diagnostic assertion then passed in
92753. Original failure logs are preserved, not cited as current passes.

## Remaining proof

The refused-surface Constraints item remains deliberately unmarked pending real
CarryBudget and local/external collision producer proof. Existing ambiguity and
ordinary budget tests do not substitute for those specific producers. The cycle
and empty-glob additions prove report behavior; source limits remain unchanged.
No raf-source or source-index edits are included in this slice.
