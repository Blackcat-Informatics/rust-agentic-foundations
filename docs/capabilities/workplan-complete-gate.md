---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-complete-gate
title: Static completion gate
description: Prospective document completion and static verification without consumer execution.
state: delivered
bindings:
  code:
    - raf_workplan::Gate::open
    - raf_workplan::Opened::workplan_after
    - raf_workplan::lifecycle::Lifecycle
    - raf_workplan::lifecycle::Lifecycle::complete
    - raf_workplan::lifecycle::verify
    - raf_workplan::lifecycle::VerifyReport
    - raf_workplan::lifecycle::Completion
    - raf_workplan::lifecycle::Condition
    - raf_workplan::lifecycle::Failure
    - raf_workplan::Gate::tracked_list
    - raf_workplan::lifecycle::Lifecycle::open_with
    - raf_workplan::staged
    - raf_workplan::git::head
  tests:
    - raf_workplan::sync::tests::a_late_rename_failure_preserves_published_files_and_discards_the_tail
    - raf_workplan::lifecycle::tests::verification_returns_static_bindings_without_running_their_bodies
    - raf_workplan::test_crate::complete::prospective_completion_refuses_an_implementing_capability_and_an_undone_item
    - raf_workplan::test_crate::complete::completion_rewrites_owned_documents_and_leaves_source_and_git_untouched
    - raf_workplan::test_crate::complete::forged_sync_reports_cannot_overwrite_source_or_manifests
    - cargo_workplan::test_crate::verify::verify_prints_static_results_without_git_or_consumer_execution
    - cargo_workplan::test_crate::complete::failing_consumer_bodies_do_not_prevent_structurally_valid_completion
    - raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line
    - raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present
    - raf_workplan::test_crate::scenarios::planning_produces_the_documentation
    - raf_workplan::test_crate::scenarios::a_document_that_merely_exists_does_not_satisfy_the_scenario
    - raf_workplan::test_crate::scenarios::a_thin_document_is_not_the_account_the_scenario_builds
    - raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does
    - raf_workplan::test_crate::sync::write_publishes_the_computed_snapshot_without_comparing_later_authored_edits
    - raf_workplan::test_crate::scenarios::each_fault_of_the_account_has_a_document_that_breaks_only_it
    - raf_workplan::staged::tests::a_replaced_file_reads_as_its_new_text_within_the_bound
    - raf_workplan::staged::tests::every_other_path_is_answered_as_the_inner_tree_does
    - raf_workplan::staged::tests::the_hidden_file_is_not_read_listed_or_found
  examples:
    - raf_workplan::Gate
depends: [cap:workplan-reconcile, cap:workplan-generated-docs]
---

## Purpose

Complete a documented change only when its static contract and prospective repository state are coherent. Declared bindings, current assessments and owned publication provide document assurance without executing potentially destructive consumer programs.

## Contract

**Static contract.** Completion requires no remaining obligations, valid item outcomes, no `PROPOSED` blocks, current generated blocks, structurally valid source bindings and current accepted affected-section assessments. The repository is checked in an in-memory view with the completed record removed and generated replacements applied. A remaining implementing capability or undone item refuses without publication.
<!-- raf:tested-by raf_workplan::test_crate::complete::prospective_completion_refuses_an_implementing_capability_and_an_undone_item -->

Verification reports declared bindings and canonical static findings. It executes no tests, examples, doctests, compilers or arbitrary commands. Stored verdicts cannot substitute for these checks or become observed test success. A panicking consumer body is data and does not prevent structurally valid completion.
<!-- raf:tested-by cargo_workplan::test_crate::verify::verify_prints_static_results_without_git_or_consumer_execution,cargo_workplan::test_crate::complete::failing_consumer_bodies_do_not_prevent_structurally_valid_completion -->

Before publishing, completion reopens the static inputs and compares source, tracked paths, loaded documentation, binding selection and generated replacements. Only owned documentation is rewritten and the change record removed. Source, manifests and Git administration are never publication targets.
<!-- raf:tested-by raf_workplan::test_crate::complete::completion_rewrites_owned_documents_and_leaves_source_and_git_untouched,raf_workplan::test_crate::complete::forged_sync_reports_cannot_overwrite_source_or_manifests -->

## Usage

Inspect a reported filesystem binding change before retrying completion. The
generated-document publisher retains its admitted destinations and reports
temporary cleanup failures alongside the original error.

The documented data path is illustrated by `raf_workplan::Gate`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

Run `cargo workplan verify` to inspect the complete static repository contract. An optional change slug selects the displayed bindings. Run `cargo workplan change complete SLUG` to evaluate prospective completion and publish its owned document edits.

## Examples

If another writer replaces a generated document's staged temporary, publication
refuses and preserves the foreign file. An unchanged neighboring publication
continues to use the same owned-document path.
<!-- raf:tested-by raf_workplan::sync::tests::a_late_rename_failure_preserves_published_files_and_discards_the_tail -->

A consumer test containing a panic remains a declared test binding. Static verification prints `declared`, never a passing execution verdict. A prospective check that leaves a capability implementing refuses and retains every input byte.

## Constraints

- Completion is static assurance only. It does not prove tests pass, examples are safe, code compiles or runtime behavior matches the documentation. Generated replacements retain admitted filesystem identities; checks and renames remain separate operations. Reopening and publication do not form an atomic snapshot transaction.
<!-- raf:tested-by cargo_workplan::test_crate::complete::failing_consumer_bodies_do_not_prevent_structurally_valid_completion -->

## Explanation

Native Git reads support section and source comparisons at the merge base. The prospective overlay supports document consistency without a temporary execution checkout, cloned index, process namespace or imported runtime result.

## Verification

The shared generated-document publication fixture refuses changed destinations
and replaced temporaries, preserves foreign bytes, reports cleanup ownership
failures and accepts an unchanged neighboring batch.
<!-- raf:tested-by raf_workplan::sync::tests::a_late_rename_failure_preserves_published_files_and_discards_the_tail -->

Controlled RAF fixtures exercise prospective refusal, record removal, generated rewrites, malicious source bodies treated as data and forbidden publication destinations. Maintainer verification of RAF does not grant RAF authority to execute inspected consumer programs.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::complete::failing_consumer_bodies_do_not_prevent_structurally_valid_completion` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::verify::verify_prints_static_results_without_git_or_consumer_execution` | test | [Contract](#contract) |
| `raf_workplan::Gate` | doctest | [Usage](#usage) |
| `raf_workplan::lifecycle::tests::verification_returns_static_bindings_without_running_their_bodies` | test | [Verification](#verification) |
| `raf_workplan::staged::tests::a_replaced_file_reads_as_its_new_text_within_the_bound` | test | [Verification](#verification) |
| `raf_workplan::staged::tests::every_other_path_is_answered_as_the_inner_tree_does` | test | [Verification](#verification) |
| `raf_workplan::staged::tests::the_hidden_file_is_not_read_listed_or_found` | test | [Verification](#verification) |
| `raf_workplan::sync::tests::a_late_rename_failure_preserves_published_files_and_discards_the_tail` | test | [Examples](#examples) |
| `raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::completion_rewrites_owned_documents_and_leaves_source_and_git_untouched` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::forged_sync_reports_cannot_overwrite_source_or_manifests` | test | [Contract](#contract) |
| `raf_workplan::test_crate::complete::prospective_completion_refuses_an_implementing_capability_and_an_undone_item` | test | [Contract](#contract) |
| `raf_workplan::test_crate::scenarios::a_document_that_merely_exists_does_not_satisfy_the_scenario` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::a_thin_document_is_not_the_account_the_scenario_builds` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::each_fault_of_the_account_has_a_document_that_breaks_only_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::planning_produces_the_documentation` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::write_publishes_the_computed_snapshot_without_comparing_later_authored_edits` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depends on | [workplan-reconcile](./workplan-reconcile.md) |
| Depended on by | [workplan-cli](./workplan-cli.md) |
| Depended on by | [workplan-prose-assessment](./workplan-prose-assessment.md) |
| Depended on by | [workplan-rustdoc](./workplan-rustdoc.md) |
| Depended on by | [workplan-strict](./workplan-strict.md) |
<!-- raf:generated:end -->
