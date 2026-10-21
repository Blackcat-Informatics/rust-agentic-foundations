<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Census caller-span correction

This run repairs a subsequently confirmed defect in the delivered native census
foundation. It is separate from the completed PR 39 archive. The parent remains
open for full WebAssembly acceptance; no native-worker result proves that scope.
The capability Constraints already disclose caller-span invalidation and forbid
holding those spans across a load; the public method's rustdoc omits that
restriction. This correction removes that destructive composability limitation
through a fresh contract change, rather than claiming it was undisclosed.

Observed public-API failure: a caller creates a live proc_macro2 span, successfully
loads a census, then querying the original span panics because collection cleanup
invalidated the entire caller-thread source map. The invalid-workspace neighbor
preserves the span because collection is never entered. The actual failing log
is retained in `raw/queue-census-caller-span-before.log` (exit 101: one valid-load
regression failed and the invalid-workspace neighbor passed). The correction was
transferred exactly into this checkout; paused strict/RDF files remain separate.

| Acceptance | Current state |
|---|---|
| Preserve caller spans on successful repeated loads and refusal neighbors | Focused public API tests pass; held standalone forms also retain bytes/prints |
| Owned worker teardown, actionable spawn error, original panic propagation | Worker success/real-normalizer refusal/panic tests pass with TLS teardown witness and exact payload; spawn-error mapping reviewed, diagnostic/source tested |
| No additional Tree Sync requirement; plain owned collection results | Real scoped Workspace borrow and all-target callers compile; Cell-backed Tree public API test passes |
| Replace misleading relative-offset cleanup test with meaningful behavior evidence | Old assertion removed; locked lexer file ordinals show only caller registrations; no retained-byte/RSS claim |
| Preserve normal-form hash bytes and existing corpus/report behavior | Census all-target suite passes 139 tests, including existing normalization/report goldens; no normal-form or report profile bump |
| Current capability contract, actual fresh lifecycle and assessments | Contract/Usage/Constraints/Explanation/Verification revised; seven current contexts captured/reviewed/refreshed with SPDX; production change complete passed exit 0 and removed its record |
| Settled full make check | Passed with exit 0; raw/queue-span-make-check.log includes all five fuzz targets at 5,000 runs and cross-target checks |
| Normal commit hooks | Passed with exit 0; raw/queue-span-commit.log includes documentation, repository and self-workplan gates |
| Hosted CI and native feedback | All 12 jobs passed in run 39550508241; refreshed discussion, review submissions, inline comments and threads are empty |
| ghprsq landing, archive verification and cleanup | Pending |

Focused terminal logs: `raw/queue-span-callers.log` (workspace/all targets, exit 0),
`raw/queue-span-census-tests.log` (139 passed, exit 0),
`raw/queue-span-clippy-fixed.log` (all census targets, warnings denied, exit 0),
`raw/queue-span-docs-sync.log` and `-final.log` (exit 0), and
`raw/queue-span-contexts.log` / `queue-span-assessment-refresh.log` (seven contexts,
exit 0). Initial clippy's one missing Markdown code span is historical in
`queue-span-clippy.log`; the fixed log supersedes it. The invalid-escape collection
probe in `queue-census-collection-span-before.log` was actually refused by the
workspace loader and is not evidence of a public collection-error path.

The collection-worker adapter's refusal test calls the real normalizer at 128
groups and 129 groups, separately from source loading, whose stronger file budget
can refuse before collection. OS thread exhaustion was not induced; the typed
spawn mapping is source-reviewed, and its I/O source/escaped rendering are unit
tested. Worker-local TLS destructor observations and proc-macro2's owned TLS source
map establish the lifetime argument; file ordinals observe registrations, not heap
bytes. Native worker ownership does not satisfy thread-free portable-core scope or
bound repeated standalone Normalizer/Tower storage.

`raw/queue-span-change-complete.log` records the terminal production lifecycle
success: `completed: removed work/changes/census-span-ownership.md`. Its exact
pre-completion record with four implemented outcomes and five affected sections is
retained at `raw/queue-span-change-before-complete.md`. The index was updated for
that removal; no staged added/deleted change record remains. Temporary context
helper source is retained as `raw/queue-contexts.rs` and removed from the product.
Source and contract metadata are frozen for parent qualification. No full suite,
product commit or push was performed in this correction lane by the implementer;
the parent owns required full qualification and ordinary hooks.

The full make check passed with exit 0 in primary execution session 78449. Normal commit hooks passed with exit 0 for commit 9fae762. All 12 hosted CI jobs passed. Refreshed native feedback is empty and the clean merge candidate equals the tested head tree. Landing and archive verification remain pending.
Reuse the canonical portfolio for backlog scope and ownership; no fresh intake or
Stage review panel is needed for this correction.
