---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-evidence-claims
title: Authored evidence and claims
description: Persisted evidence metadata, static claim relationships and input invalidation.
state: delivered
bindings:
  code:
    - raf_workplan::evidence
    - raf_workplan::record
    - raf_workplan::Workplan::evidence_claims
    - raf_workplan::Workplan::evidence_claims_dataset
    - raf_workplan::Workplan::evidence_for
    - raf_workplan::diagram
    - raf_workplan::Workplan::diagram
    - raf_workplan::evidence::claims_dataset
  tests:
    - raf_workplan::test_crate::evidence::the_views_list_the_claims_of_a_record_and_the_records_of_a_capability
    - raf_workplan::test_crate::evidence::a_view_of_what_does_not_exist_is_an_error_and_its_neighbour_answers
    - raf_workplan::test_crate::records::a_written_record_reloads_with_no_finding_as_the_same_value
    - cargo_workplan::test_crate::evidence::stored_command_text_is_never_executed_by_checks_or_dependency_views
    - raf_workplan::test_crate::records::create_never_replaces_a_file
    - raf_workplan::test_crate::records::replace_puts_new_text_in_place_of_an_existing_record_only
    - cargo_workplan::test_crate::process::a_usage_error_starts_no_process
    - cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output
    - raf_okf::test_crate::records::by_is_human_or_agent_and_never_a_name
    - raf_okf::test_crate::claims::a_claim_marker_names_its_evidence_and_supports_the_paragraph_above_it
    - raf_okf::test_crate::claims::a_claim_may_rest_on_several_records
    - raf_okf::test_crate::claims::a_claim_after_a_list_item_supports_that_item_only
    - raf_okf::test_crate::claims::a_malformed_claim_marker_is_refused_with_its_line
    - raf_okf::test_crate::claims::a_claim_marker_with_no_statement_directly_above_is_refused
    - raf_workplan::test_crate::records::a_claim_on_evidence_that_covers_its_capability_is_clean_and_an_edge
    - raf_workplan::test_crate::records::a_claim_on_unknown_evidence_names_the_file_and_the_line
    - raf_workplan::test_crate::records::a_claim_on_evidence_for_another_capability_is_a_finding
    - raf_workplan::test_crate::records::a_record_that_names_what_does_not_exist_is_a_finding_and_its_neighbour_is_clean
    - raf_okf::test_crate::records::a_record_has_no_title_description_or_body
    - raf_workplan::test_crate::records::a_claim_inside_a_proposed_block_is_not_a_claim_until_it_is_resolved
    - raf_workplan::test_crate::evidence::a_handwritten_record_can_exceed_the_writer_value_limit_but_not_the_document_bound
    - raf_workplan::files::tests::a_directory_swap_after_preflight_is_not_rechecked_by_exclusive_publication
    - raf_workplan::record::tests::a_directory_swap_after_preflight_refuses_replacement_publication
    - raf_workplan::test_crate::evidence::a_record_supports_a_capability_by_its_for_list_without_a_claim
    - raf_workplan::test_crate::evidence::the_claims_of_a_record_are_refused_while_a_capability_document_does_not_load
  examples:
    - raf_workplan::Workplan::evidence_claims
depends: [cap:workplan-document-model, cap:workplan-generated-docs]
---

## Purpose

Keep authored evidence relationships inspectable without executing the command text or test bindings they describe. Persisted input identities and claim references support traceability while remaining assertions recorded by their authors.

## Contract

**Static contract.** Evidence records are authored metadata. Their verdict, values, limits, revisions and input digests describe assertions recorded by an author. Reading a stored command never runs it, and stored runtime verdicts never become RAF-observed execution success. The generic record API permits explicit metadata creation and replacement within the evidence directory; the CLI has no command-executing `evidence record` operation.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::stored_command_text_is_never_executed_by_checks_or_dependency_views,raf_workplan::test_crate::records::a_written_record_reloads_with_no_finding_as_the_same_value -->

Claims refer to evidence by slug and retain capability, section, line and statement text. `evidence_claims` and `evidence_for` report complete static relationships, including explicitly missing evidence. Unloaded capability documents refuse a complete dependency answer. Native RDF projections preserve the same authored claims and support edges.
<!-- raf:tested-by raf_workplan::test_crate::evidence::the_views_list_the_claims_of_a_record_and_the_records_of_a_capability,raf_workplan::test_crate::evidence::a_view_of_what_does_not_exist_is_an_error_and_its_neighbour_answers -->

## Usage

Use one explicit record store for inspection and publication. The native disk
adapter retains admitted directories and destination snapshots; a changed binding
requires inspection rather than writing through the new name.

The documented data path is illustrated by `raf_workplan::Workplan::evidence_claims`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

Use `cargo workplan evidence claims ID` for statements resting on a record and `cargo workplan evidence for cap:NAME` for supporting records. Use `record::Record::Evidence` and explicit record publication APIs for authored metadata. These operations never execute the described subject.

## Examples

An evidence destination created by another writer after absent-name admission
cannot be overwritten by exclusive publication. A changed parent directory
refuses replacement, while an unchanged real directory permits it.

A record may contain shell-looking command text. Checking the repository and querying that record leave the text inert and write no marker file. A malformed evidence slug refuses beside a valid dependency query.

## Constraints

- Authored verdicts do not prove tests ran or compilers accepted source. Exact numeric comparisons and declared source digests remain data validation. Record writes are explicit, confined to owned destinations and subject to the documented filesystem publication limits.
  The native disk store retains one root authority, existing parent directory
  identities and original destination snapshots through publication. Originally
  absent destinations use exclusive publication; replacement refuses changed
  identity, content, link count or mode. Temporary files receive bytes privately
  and restore the intended publication permissions. Cleanup checks ownership and
  reports failures. Checks and renames remain separate operations, with no atomic
  multi-file transaction, directory durability guarantee or isolation from
  adversarial concurrent publishers. The native disk adapter requires the primary
  Linux target and refuses other platforms before mutation. Other caller-supplied
  stores retain their explicitly supplied semantics.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::stored_command_text_is_never_executed_by_checks_or_dependency_views -->

## Explanation

The claim graph connects statements to persisted assertions. It supplies traceability and stale-input diagnostics without turning a document or saved result into execution authority.

## Verification

The native record directory-swap fixture checks retained admission, unchanged
foreign bytes and a valid replacement after restoring the original directory.
<!-- raf:tested-by raf_workplan::record::tests::a_directory_swap_after_preflight_refuses_replacement_publication -->

Native record round trips, claim lookup refusals and the stored-command fixture cover the data-only boundary. Static completion never imports these records as substitutes for its binding and document checks.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::evidence::stored_command_text_is_never_executed_by_checks_or_dependency_views` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::process::a_usage_error_starts_no_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_claim_after_a_list_item_supports_that_item_only` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_claim_marker_names_its_evidence_and_supports_the_paragraph_above_it` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_claim_marker_with_no_statement_directly_above_is_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_claim_may_rest_on_several_records` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_malformed_claim_marker_is_refused_with_its_line` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_record_has_no_title_description_or_body` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::by_is_human_or_agent_and_never_a_name` | test | [Verification](#verification) |
| `raf_workplan::Workplan::evidence_claims` | doctest | [Usage](#usage) |
| `raf_workplan::files::tests::a_directory_swap_after_preflight_is_not_rechecked_by_exclusive_publication` | test | [Verification](#verification) |
| `raf_workplan::record::tests::a_directory_swap_after_preflight_refuses_replacement_publication` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_handwritten_record_can_exceed_the_writer_value_limit_but_not_the_document_bound` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_record_supports_a_capability_by_its_for_list_without_a_claim` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::a_view_of_what_does_not_exist_is_an_error_and_its_neighbour_answers` | test | [Contract](#contract) |
| `raf_workplan::test_crate::evidence::the_claims_of_a_record_are_refused_while_a_capability_document_does_not_load` | test | [Verification](#verification) |
| `raf_workplan::test_crate::evidence::the_views_list_the_claims_of_a_record_and_the_records_of_a_capability` | test | [Contract](#contract) |
| `raf_workplan::test_crate::records::a_claim_inside_a_proposed_block_is_not_a_claim_until_it_is_resolved` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_on_evidence_for_another_capability_is_a_finding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_on_evidence_that_covers_its_capability_is_clean_and_an_edge` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_on_unknown_evidence_names_the_file_and_the_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_record_that_names_what_does_not_exist_is_a_finding_and_its_neighbour_is_clean` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_written_record_reloads_with_no_finding_as_the_same_value` | test | [Contract](#contract) |
| `raf_workplan::test_crate::records::create_never_replaces_a_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::replace_puts_new_text_in_place_of_an_existing_record_only` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depended on by | [workplan-cli](./workplan-cli.md) |
| Depended on by | [workplan-prose-assessment](./workplan-prose-assessment.md) |
| Depended on by | [workplan-strict](./workplan-strict.md) |
| Depended on by | [workplan-two-way-coverage](./workplan-two-way-coverage.md) |
<!-- raf:generated:end -->
