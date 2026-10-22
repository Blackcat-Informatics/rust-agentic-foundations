<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Integration and edition-context repair

Current scope: preserve donor, integrate the delivered census/source-root contracts and authored local-main policy, reproduce/repair the shared-file edition cache, qualify focused production callers. This is not completion of the RDF/sole-default/adoption delivery. No full make check, product-fix commit or push is authorized before independent review.

## Preserved donor

Exact stash: `a5754d22fc7c53ac0542bc4a216df4bfddade4dd`; created with include-untracked from clean HEAD dc1115b plus dirty T8. Refer to this OID, not the globally moving stash ordinal. `raw/donor-t8/` holds the binary tracked patch, NUL-separated tracked/untracked path lists, raw tar copies and SHA-256 sums. Unique pre-existing ignored evidence remains in place. The subsequent checkout was verified clean before merge. Premature delivered states, old completion outcomes and inventory removals were not restored as delivered behavior.

## Origin integration

Merged exact origin/main `e1e52f60930494089aa8035d54608b2d87a5cb97` normally; resulting merge `2518cc6`.

Substantive resolution retains strict in-flight contracts alongside delivered census, both additive symbol-index test sets, source discovery without metadata processes, and the actual remaining nine open change records. The invocation table has 23 rows: the original strict donor's extra invocation survives while the removed metadata invocation is gone. Census normal form is delivered; the nine strict-affected capabilities remain implementing. The old normal-form change record is removed as on landed main. Corrected inherited CHANGELOG first-eight-byte wording to the delivered full 32-byte identity.

Early `cargo check --workspace --all-targets --locked`: exit 0, `raw/queue-integration-callers.log`. Docs sync: exit 0, `raw/queue-integration-docs-sync.log`. Twelve stale assessments were captured from actual merged contexts and refreshed through the production assessment API with explicit SPDX options. Contexts and exact temporary Rust helper are retained under raw; helper removed from source before staging.

First normal commit hook refused a rustfmt-only inventory assertion layout; 31 documentation tests, 68 repository tests and one self-workplan test and REUSE passed. That failed attempt remains historical in `raw/queue-origin-merge-hook.log`. Formatting was fixed and the complete normal hook reran successfully, exit 0, in `raw/queue-origin-merge-hook-fixed.log`; no bypass.

## Remaining terminal results

Local-main synchronization completed normally as `4e5e187bb9d20fd8c978b95bc69ee6e27438c3b3`, full merge hook exit 0 (`raw/queue-policy-main-merge.log`). It carries the authored CLAUDE policy commits 7976c1c and 6c788ac without rewriting them and preserves the donor mutation ban. No direct-main edit/push occurred.

The mixed-edition shared physical source regression reproduced the defect before repair: exit 101 in `raw/queue-r5-before.log`, where combined 2024/2021 roots lost the 2021 generator reached by its separate walk. Unexpanded macro bodies provide valid source in both editions; raw-name neighbors, both input root orders and swapped package editions are exercised. The cache now keys retained runs by `(RelPath, edition)` and charges the enlarged key; RelPath clones share Arc text. No per-reference path-body allocation is introduced.

Terminal focused results, all exit 0:

| Check | Result | Raw log |
|---|---|---|
| raf-source reachability integration binary | 24 passed | queue-r5-reachability.log |
| raf-source reach unit group | 15 passed, including counted work and exact byte-limit/cache refusal neighbors | queue-r5-count-budget-final.log |
| raf-workplan lines production callers | 19 passed, including mixed-edition source-copy union and restored donor exclusions/impl neighbors | queue-r5-lines-callers.log |
| raf-workplan touched unit group | 5 passed; Frame's restored Eq fixture now derives PartialEq | queue-r5-touched.log |
| source/workplan clippy, all targets, denied warnings | passed | queue-r5-clippy.log |
| workspace all-target locked callers check | passed, including census and CLI callers | queue-r5-callers.log |
| docs sync | passed | queue-r5-docs-sync.log |
| changed assessment context capture and production refresh | 8 contexts reviewed/refreshed with explicit SPDX | queue-r5-contexts.log; queue-r5-assessment-refresh.log |

The early budget-test development compile failures remain in queue-r5-count-budget.log, -fixed.log and -current.log; they are historical failures, superseded by the successful final log. No current result relies on them. The context helper is retained at raw/queue-assess-refresh.rs and removed from product source. No full make check, product-fix commit or push was performed. The ordinary product commit hook remains to be run by the parent on the staged final snapshot.

The production source/docs are frozen for parent review: CHANGELOG; raf-source reach.rs, workspace.rs and tests/reachability.rs; raf-workplan lifecycle/touched/tests.rs and tests/lines.rs; capability source-index and workplan-strict; five source-index assessments and three workplan-reconcile assessments. Existing donor stash/archive remains preserved. Only useful test deltas were restored; no premature delivered-state outcomes or inventory removals were applied. Final formatting status is recorded at raw/queue-r5-fmt.log; handoff supplies its terminal result.

Independent review completed in `tasks/queue-r5-independent-review.md`. The normal
commit hook completed exit 0, including formatting, REUSE, 31 documentation tests, 68 repository tests and
one self-workplan test, creating `07169f5`; normal push completed exit 0. Raw logs:
`queue-r5-commit-hook.log` and `queue-r5-push.log`. This supersedes the pending
commit/push state above. Full delivery qualification remains unfinished; source
ownership is released for the RDF foundation.
