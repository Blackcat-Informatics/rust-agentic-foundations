<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Collection ownership review

Independent source, caller, failure-path, test and contract review is recorded in
`tasks/queue-collection-review.md`, with dependency/session analysis in
`tasks/queue-census-span-ownership-review.md`. The reviewed correction removes the
documented destructive caller-span restriction; it does not claim that restriction
was undisclosed. The ineffective file-relative-offset cleanup assertion is removed.

The real public path reads a non-Sync Tree on its caller thread and shares only
the completed Workspace with an owned collection worker. No whole workspace copy,
unsafe sharing, alternate normalizer, partial-success fallback or global span reset
was introduced. Spawn failures retain their typed I/O source and escaped diagnostic;
panics resume with their original payload after worker TLS teardown. Code/doc changes
preserve the existing normal-form token and fingerprint algorithms.

Focused results pass 139 census tests, all-target workspace compilation and denied-
warning census clippy. Actual fresh lifecycle completion removed its record, and
seven assessment contexts were reviewed/refreshed with SPDX retained. The required
full make check and the normal commit hook passed with exit 0. Commit 9fae762
is pushed in PR 52. The current merge candidate is clean and equals the tested
head's tree; no additional base delta is present. All 12 hosted CI jobs passed in run 39550508241.
Paginated review submissions, inline comments, review threads and discussion
comments are empty; the final refreshed merge brief confirms passing checks and no comments.

No open source finding remains in the inspected native correction. OS thread
exhaustion was not induced; the typed mapping is source-reviewed and error rendering/
source tested. File ordinals observe caller registrations rather than heap bytes.
Standalone normalization/session storage and thread-free WASM acceptance remain
explicit unfinished parent contracts, not acceptance of this correction or a waiver.

The selected stage contains source-linked evidence, raw logs and current issue
capture (about 1.3 MiB including full-gate output); it has no symlinks or copied checkout.
Final evidence/feedback writers and results must be settled before ghprsq capture.
