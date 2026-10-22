<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Mixed-edition reachability repair

Independent source review of the focused repair after the ordinary census/root
and maintainer-policy integrations. This does not qualify the unfinished RDF,
sole-policy, adoption, or lifecycle work.

Observed defect: keyword-filtered `FileRuns` were cached by physical path even
when different package editions reached that path. The preserved failing
`raw/queue-r5-before.log` reproduces different combined/separate reachability.
The repair keys runs by `(RelPath, edition)`; subsequent references use exactly
the key returned by loading their module context. Context-specific name-resolution
caches already include the module. `RelPath` shares its text through `Arc`, so
the key clone does not allocate another path body. Retained entry accounting now
uses the actual larger key type and separately charges both edition caches.

The public regression swaps package editions and root order, compares the union
of separate walks to the combined walk, and retains raw-identifier neighbors.
The internal regression checks one lex per file/edition and exact/one-byte-below
retained-byte acceptance. The real workplan line-coverage caller executes the
same boundary; `coverage::lines::cover` calls `Workspace::reachable`, propagates
its errors, and unions physical-source copies when marking covered lines.

Inspected executed results: reachability integration 24 passed; reachability unit
tests 15 passed, including work-growth and exact-byte cases; production line
coverage 19 passed; focused clippy passed. Corresponding raw logs are
`queue-r5-reachability.log`, `queue-r5-count-budget-final.log`,
`queue-r5-lines-callers.log`, and `queue-r5-clippy.log`.

Final source/documentation check: the obsolete single-lex-per-file and
first-reached-edition statements are replaced with the file/edition law in the
module, public API, Contract, Constraints and changelog. The restored empty `Eq`
implementation fixture derives its required `PartialEq`; the focused touched
tests execute successfully. New tests are formatted and bound to their capability
documents. The deficiency ledger has no entries. Eight assessment contexts were
refreshed through the production API with SPDX preserved. The normal commit hook
completed successfully (31 documentation tests, 68 repository tests and one self-workplan test, formatting and REUSE),
creating `07169f5`. No full `make check`, release suite, or revised policy/adoption
acceptance is claimed here. Commit log: `raw/queue-r5-commit-hook.log`.
