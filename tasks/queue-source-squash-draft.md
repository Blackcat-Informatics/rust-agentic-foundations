<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Draft squash notes

Draft only. Refreshed integration/merge gates remain with the parent owner.
Current Windows acceptance is explicitly waived rather than claimed passed.
The final sibling .squash_notes file has not been created or changed.

## Proposed commit text

feat(source): bind locked sources to declared compilation contexts

Retain complete Cargo dependency declarations and select their exact outgoing
locked package rows. Join supplied roots and sealed committed-Git bytes to actual
loaded PackageIds, preserving package-qualified alias/reexport destinations and
refusing missing, ambiguous, incompatible or inactive requested evidence.

Validate complete caller-declared per-package features and target/host cfg,
select the actual library or build-script target, and filter modeled declaration
headers through the existing source loader and resolver. Reuse native PurRDF
storage, traversal, hashing, source/URL and integer APIs under the original
resource grant. Installed Git owns bounded committed-object decoding; the unused
raw-object decoder path and its additional dependency closure are removed.

The exact PurRDF pin 5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43 resolves
purrdf_lex::allocation::Memory to crates/lex/src/allocation.rs:367 and the
Shared reexport to purrdf_lex::allocation::shared::Shared at
crates/lex/src/allocation/shared.rs:80, in actual purrdf-lex 3.0.1.
These routes retain actual locked package/source identities and prove declaration/entry-point
authority, not Census provider execution or a code-occurrence oracle. Git and
its local object store remain a trusted host boundary. Cfg is declared rather
than observed Cargo state; bodies, fields, variants and macros are not expanded.

Progresses issues 48 and 53 without closing either. Their full external/std,
unsupported/macro, differential and actual provider-enforcement requirements
remain open.

Native full-bound change completion and all seven current accepted Source
assessments passed. Mandatory local coverage passed across recorded executions:
fmt, workspace Clippy/tests, Source release tests, documentation, offline deny,
REUSE, workspace/fuzz MSRV, five 5,000-run fuzz targets, all three cross targets,
licences and issue-reference gates. The initial make check exited 2 at REUSE
after missing licence headers in assessment replacements; native comment-only
repair, affected metadata gates and the remaining required targets passed.
Normal signed commit hooks passed without bypass.

Hosted CI currently has eleven successful jobs. Linux accounts for all 2,875
bindings (2,873 passed / 2 compiler-proved not applicable); macOS accounts for
all 2,875 (2,861 passed / 14 not applicable), with no failure/error dispositions.
Windows binding verification remains running and unverified. Explicit user
direction: “Please disable CI windows tests on your next PR. For now, ignore
those CI results.” Current PR 58 proceeds without Windows acceptance under this
scoped waiver. No Windows green result is claimed; no source/CI configuration
changed and no GitHub job was cancelled/restarted. Feedback was refreshed after
the waiver. No CI rerun has been requested.

Defect-Class: none

## Finalization evidence

- Commit/PR: f601a80 / PR 58. Parent owns final base/head refresh and ghprsq.
- Current acceptance: ../validation.md and queue-pr58-acceptance-inventory.md.
- Local receipts: queue-delivered-base-qualification.md and linked raw logs.
- Independent reviews: queue-committed-lock-review.md,
  queue-configured-definition-review.md and reused incremental reviews linked
  from validation.md; no concrete in-scope finding remains open.
- Initial stagectl pr-create exit 1 after actual publication remains distinguished
  from the confirmed single open PR and subsequent successful supported lookup.
- Raw/task evidence and every recovery stash remain retained. No source change,
  merge or cleanup is performed by this draft.
