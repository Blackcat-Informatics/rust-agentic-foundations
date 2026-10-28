---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-code-reviews
title: Code review records
description: Record reviews against actual source revisions and query item currency, coverage and required review policy.
state: implementing
bindings:
  code:
    - raf_workplan::review
    - raf_workplan::digest::Digest::review_symbol
    - raf_workplan::digest::Digest::review_item
    - raf_workplan::git::head
    - raf_workplan::lifecycle::touched::BaseTree::at_revision
    - raf_okf::profile::review
    - cargo_workplan::bin_crate::cargo_workplan::review
    - cargo_workplan::bin_crate::cargo_workplan::result_graph::review
  tests:
    - cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone
    - raf_workplan::digest::tests::review_currency_changes_only_the_edited_definition_and_keeps_container_contracts
    - raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind
    - raf_okf::test_crate::profile::valid_documents_of_each_type_validate_to_typed_values
    - raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates
  examples:
    - raf_workplan::Workplan::review_inputs
depends: [cap:okf-profile, cap:source-index, cap:workplan-document-model, cap:rdf-report-profile]
---

# Code review records

## Purpose

Code reviews state what source was examined, the question the review answered and
the conclusion it reached. Persisted item inputs make that scope inspectable as
the code changes. A repository can distinguish reviewed code from changed, added
or removed items and can require a current review for particular capabilities.
The same library model serves command callers, native RDF claims and coverage
queries. Review records retain reviewer categories and structured dispositions;
the source index supplies identities and definitions, and Git supplies the actual
tree examined when a review is recorded.

## Contract

### Recorded scope and trust

A Review is an empty-body record in `work/reviews/<slug>.md`, identified by
`review:<slug>`. It stores a required RFC 3339 timestamp, full reviewed commit,
scope, kind, verdict, reviewer category, source inputs and findings. Kinds are
security, correctness, design, performance and documentation; verdicts are
clean, findings and blocked. Reviewer categories are human and agent. A writer
derives the inputs from the actual committed source tree and accepts a draft
without input rows. Structured findings carry severity, source location and an
open, fixed, accepted or false-positive disposition. Fixed findings name a full
fixing commit; accepted and false-positive findings state their reason.
<!-- raf:tested-by cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone,raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates -->

### Item currency and coverage

Crate scopes select every indexed target of that Cargo package. Module scopes
select their indexed descendants; explicit item scopes select those definitions.
A capability scope selects its bound code and the descendants of bound modules.
Every canonical path retains a digest of its own source and definition context.
Container inputs exclude their indexed children, so editing a leaf changes that
leaf rather than every enclosing item. Conditional definitions contribute to
the same canonical path. Currentness compares these persisted inputs against
current source and reports changed, added and removed paths. A rename is removal
plus addition. A stored review remains queryable when its original commit is
absent. Unchanged items of a stale completed review still count toward coverage;
blocked reviews cover no items. The latest completed review of a scope and kind
uses its timezone-aware date, with record identity breaking equal-date ties.
<!-- raf:tested-by cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone,raf_workplan::digest::tests::review_currency_changes_only_the_edited_definition_and_keeps_container_contracts -->

### Required reviews and semantic claims

A capability's optional `review_required` list names distinct closed kinds.
Each kind requires a completed current review of that exact capability scope.
Missing and stale reviews raise their corresponding reconciliation obligations.
Complete checks retain these requirements independently of prose debt. An
unreadable record or scope produces an unavailable finding or a typed error.
A current review with findings satisfies the review requirement while retaining
its findings verdict. An absent or empty list imposes no review policy.
Semantic queries reuse document identities and the shared native RDF vocabulary:
a review reifies its verdict about its source set and retains each input and
finding. Text, RDF and native JSON-LD expose the same typed outcomes; serialized
statement layers retain their complete triple-term data.
<!-- raf:tested-by raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind,cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone -->

## Usage

Inspect a current source selection through
`raf_workplan::Workplan::review_inputs`; its runnable example selects one item
from a small consuming workspace. Record a completed review with
`cargo workplan review record --id audit --scope crate:demo --kind correctness
--verdict clean --by human --date 2030-01-01T00:00:00Z`, supplying the ordinary
SPDX options. Omit `--revision` to pin the checked current HEAD, or give the full
reviewed commit explicitly. Supply `--findings <path-or-url>` for an external
findings document. Library callers can supply structured findings through the
same typed record writer.

Use `review list`, `review show review:audit`, `review status crate:demo`,
`review changed-since crate:demo`, `review unreviewed --crate demo` and
`review coverage --crate demo`. Queries default to correctness; `--kind` selects
another question. Repeated explicit item selectors form one sorted scope.
`--format json` uses native JSON-LD with RDF 1.2 triple terms; Turtle and
N-Triples retain the same facts. List and show export their semantic review
selection and refuse quiet machine output. Status, coverage and unreviewed
queries use the shared typed report graph. Findings return exit 1, usage and
execution failures return exit 2, and clean results return exit 0. Recording a
findings or blocked verdict writes its valid record and returns exit 1.
<!-- raf:tested-by cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone -->

## Examples

The runnable `raf_workplan::Workplan::review_inputs` example leaves an unselected
function outside the inspected item scope. A consuming Git fixture records a
crate review and an independently bounded module review. All seven commands
read those claims. Its original reviewed object is pruned after a squash-shaped
snapshot, and status remains current from stored inputs. Editing one definition,
adding one and renaming or deleting others reports exactly those paths; stable
neighbors remain covered. Queries preserve repository files and record bytes.
<!-- raf:tested-by cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone -->

The same record passes native Turtle, N-Triples and JSON-LD parsing with equal
complete statement data. A missing findings document refuses before publication;
its readable neighbor records the findings verdict. Unknown ids, scopes, kinds,
malformed dates and unsupported JSON commands return usage failures with no
machine result. Closed profile examples reject repeated or unknown required
kinds. Policy fixtures distinguish current findings, blocked and wrong-kind
records, stale code and unavailable references.
<!-- raf:tested-by cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone,raf_okf::test_crate::profile::valid_documents_of_each_type_validate_to_typed_values,raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind -->

## Constraints

- Recording requires the actual first-party commit and a complete readable
  source selection. Historical reads reuse the bounded lazy Git tree, checked
  paths and source loader. Findings locations are checked against that reviewed
  tree; repository findings references are checked against current readable files.
  <!-- raf:tested-by cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone -->
- Scope and coverage concern indexed source definitions and their declared
  structural descendants. Input framing retains canonical paths, relative files
  and normalized line endings. It excludes load-local identifiers and offsets.
  Standard execution validation covers native Linux.
  <!-- raf:tested-by raf_workplan::digest::tests::review_currency_changes_only_the_edited_definition_and_keeps_container_contracts,cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone -->
- Required kinds, verdicts, reviewer categories and finding rows are closed data.
  External HTTP and HTTPS findings references are validated opaque IRIs. Source
  query, storage and malformed-record failures produce actionable refusals.
  <!-- raf:tested-by raf_okf::test_crate::profile::valid_documents_of_each_type_validate_to_typed_values,raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind -->
- An accepted review finding records its disposition and reason. Census continues
  to validate its own ledger and debt ratchet. Review requirements remain active
  independently of unrelated prose debt and retain visible findings verdicts.
  <!-- raf:tested-by raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind -->

## Explanation

Review currency is a comparison of source inputs, while the reviewed revision
records provenance. Separating them lets a genuine review survive squash merges
and Git object retirement. Item-owned residuals keep a module's imports,
visibility and surrounding contract distinct from the bodies of its children.
Canonical path sets make additions and removals visible without guessing that
two differently named items represent the same review subject.

Review verdicts are attested claims about that source set. Native projection
preserves their identity, input evidence and findings without interpreting the
reviewer's judgment. Explicit capability requirements state where review is a
gate, while ordinary repositories retain their existing behavior. The shared
record store, source index, obligation model and semantic serializer keep the
recording and query paths consistent.

## Verification

The command fixture exercises recording, every query, all exit classes, carrier
agreement, module boundaries, source mutation and loss of the reviewed object.
The digest case protects leaf ownership, container contracts and normalized
input framing. The closed profile corpus carries every finding disposition and
round-trips through native RDF. Required-review fixtures protect default absence,
the valid findings verdict and neighboring missing, stale or unreadable inputs.
These checks use operation and value assertions, with source identities and
repository bytes retained as observable behavior.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_okf::test_crate::profile::valid_documents_of_each_type_validate_to_typed_values` | test | [Examples](#examples), [Constraints](#constraints) |
| `raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates` | test | [Contract](#contract) |
| `raf_workplan::Workplan::review_inputs` | doctest | [Usage](#usage), [Examples](#examples) |
| `raf_workplan::digest::tests::review_currency_changes_only_the_edited_definition_and_keeps_container_contracts` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
<!-- raf:generated:end -->
