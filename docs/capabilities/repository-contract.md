---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:repository-contract
title: "Repository contract"
description: "The tracked document inventory, repository policy gates and accepted assessment check of the unpublished root package."
state: delivered
bindings:
  code:
    - rust_agentic_foundations
    - rust_agentic_foundations::gate_root
    - rust_agentic_foundations::bin_crate::raf_ci::main
    - rust_agentic_foundations::bin_crate::raf_ci::run_examples
  tests:
    - rust_agentic_foundations::test_crate::repo_gates::ci::every_workflow_job_has_an_explicit_ten_minute_ceiling
    - rust_agentic_foundations::tests::the_absent_gate_root_uses_the_manifest_directory
    - rust_agentic_foundations::tests::an_explicit_gate_root_is_used_without_falling_back
    - raf_workplan::test_crate::history::a_fixture_commit_obeys_an_installed_hook_and_the_successful_neighbor
    - rust_agentic_foundations::test_crate::docs_contract::a_block_that_differs_by_one_line_is_reported
    - rust_agentic_foundations::test_crate::docs_contract::a_delivered_capability_holds_a_verification_matrix
    - rust_agentic_foundations::test_crate::docs_contract::a_tilde_fenced_usage_block_is_accepted_and_an_unclosed_block_is_refused
    - rust_agentic_foundations::test_crate::docs_contract::an_open_capability_may_be_listed_by_a_record_named_after_another
    - rust_agentic_foundations::test_crate::docs_contract::doctest_comparison_fixtures
    - rust_agentic_foundations::test_crate::docs_contract::inventory_accepts_a_matching_tree
    - rust_agentic_foundations::test_crate::docs_contract::inventory_refuses_a_record_id_of_no_record_type
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_capability_listed_twice_even_when_the_record_lists_it_twice
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_change_record_that_lists_other_capabilities
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_document_the_table_does_not_list
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_missing_architecture_and_index_documents
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_missing_capability_and_change_documents
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_missing_or_unlisted_record
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_state_change
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_an_id_change
    - rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_an_index_the_table_does_not_list
    - rust_agentic_foundations::test_crate::docs_contract::inventory_table_has_the_documented_shape
    - rust_agentic_foundations::test_crate::docs_contract::refuses_a_file_swapped_in_after_inspection
    - rust_agentic_foundations::test_crate::docs_contract::refuses_non_regular_files_instead_of_reading_them
    - rust_agentic_foundations::test_crate::docs_contract::rejects_a_tracked_symlink_that_nothing_links_to
    - rust_agentic_foundations::test_crate::docs_contract::rejects_a_typed_document_outside_the_document_directories
    - rust_agentic_foundations::test_crate::docs_contract::rejects_non_markdown_file_in_a_document_directory
    - rust_agentic_foundations::test_crate::docs_contract::repository_bindings_resolve_in_the_source_index
    - rust_agentic_foundations::test_crate::docs_contract::repository_documents_satisfy_the_profile
    - rust_agentic_foundations::test_crate::docs_contract::the_checker_accepts_a_valid_checkout
    - rust_agentic_foundations::test_crate::docs_contract::the_checker_judges_links_against_the_tracked_paths_and_symlinks
    - rust_agentic_foundations::test_crate::docs_contract::the_checker_judges_markdown_outside_the_document_directories
    - rust_agentic_foundations::test_crate::docs_contract::the_checker_judges_the_documents_through_the_model
    - rust_agentic_foundations::test_crate::docs_contract::usage_and_examples_blocks_are_the_crate_doctests
    - rust_agentic_foundations::test_crate::repo_gates::citation_accepts_matching_versions
    - rust_agentic_foundations::test_crate::repo_gates::citation_rejects_a_manifest_bump_without_a_matching_citation
    - rust_agentic_foundations::test_crate::repo_gates::citation_rejects_a_missing_version
    - rust_agentic_foundations::test_crate::repo_gates::citation_rejects_a_version_bump_without_a_matching_manifest
    - rust_agentic_foundations::test_crate::repo_gates::citation_version_matches_the_workspace
    - rust_agentic_foundations::test_crate::repo_gates::deny_accepts_the_root_configuration_plus_the_one_exception
    - rust_agentic_foundations::test_crate::repo_gates::deny_fuzz_configuration_is_the_root_configuration_plus_one_exception
    - rust_agentic_foundations::test_crate::repo_gates::deny_rejects_a_dropped_root_exception
    - rust_agentic_foundations::test_crate::repo_gates::deny_rejects_a_missing_or_widened_or_repeated_exception
    - rust_agentic_foundations::test_crate::repo_gates::deny_rejects_a_shared_section_that_differs
    - rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class
    - rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them
    - rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_first_party_manifests_have_no_json_result_dependency
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_flags_a_token_in_a_latin_1_file
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_change_number
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_colon_prefixed_number
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_github_style_number
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_hash_number_after_a_space
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_hash_number_after_a_word_character
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_parenthesised_hash_number
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_plan_possessive
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_pull_request_label
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_pull_url
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_task_number
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_an_acceptance_label
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_an_all_digit_css_colour
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_an_issue_url
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_per_the_plan
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_the_word_issue_with_a_number
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_the_words_pull_request_with_a_number
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_this_branch
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_this_issue
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_reports_every_hit_as_file_and_line
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_scans_text_files_and_skips_binary_ones
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_skips_a_nul_containing_file_holding_a_token
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_a_hex_colour_with_letters
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_a_quoted_three_digit_colour
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_a_six_digit_colour
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_an_xml_character_reference
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_near_miss_prose
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_tracked_files_are_clean
    - rust_agentic_foundations::test_crate::repo_gates::issue_refs_treats_only_a_nul_byte_as_binary
    - rust_agentic_foundations::test_crate::repo_gates::licenses_accepts_a_matching_fingerprint
    - rust_agentic_foundations::test_crate::repo_gates::licenses_require_a_canonical_fingerprint_with_non_hex_boundaries
    - rust_agentic_foundations::test_crate::repo_gates::licenses_accepts_bytes_that_hash_to_the_pin
    - rust_agentic_foundations::test_crate::repo_gates::licenses_mulan_texts_match_the_pinned_digest
    - rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_a_fingerprint_mismatch
    - rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_a_record_without_a_fingerprint_line
    - rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_an_altered_mulan_text
    - rust_agentic_foundations::test_crate::repo_gates::licenses_sha256_matches_known_vectors
    - rust_agentic_foundations::test_crate::repo_gates::licenses_zh_hans_matches_the_recorded_fingerprint
    - rust_agentic_foundations::test_crate::repo_gates::lints_accepts_equal_tables_whatever_the_comments
    - rust_agentic_foundations::test_crate::repo_gates::lints_every_workspace_manifest_inherits_the_workspace_lints
    - rust_agentic_foundations::test_crate::repo_gates::lints_fuzz_workspace_tables_equal_the_root_tables
    - rust_agentic_foundations::test_crate::repo_gates::lints_reads_single_and_multi_line_member_lists
    - rust_agentic_foundations::test_crate::repo_gates::lints_reads_the_level_of_a_lint_and_sees_a_weakened_one
    - rust_agentic_foundations::test_crate::repo_gates::lints_rejects_a_member_manifest_that_does_not_inherit
    - rust_agentic_foundations::test_crate::repo_gates::lints_rejects_a_root_without_lint_tables
    - rust_agentic_foundations::test_crate::repo_gates::lints_rejects_a_weakened_level_in_the_fuzz_tables
    - rust_agentic_foundations::test_crate::repo_gates::lints_rejects_an_added_or_removed_lint_in_the_fuzz_tables
    - rust_agentic_foundations::test_crate::repo_gates::lints_the_root_denies_missing_docs_unsafe_code_and_unreachable_pub
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_exempts_the_clock_file_but_not_its_neighbour
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_exempts_the_gate_file_but_not_its_neighbours
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_exempts_the_paths_outside_every_rule
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_judges_a_crate_added_under_the_crates_directory
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_lets_the_runner_name_durations_and_limits
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_passes_the_clean_neighbour
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_reading_in_every_ruled_directory
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_each_crate_spelling_in_every_other_directory
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_each_source_spelling_in_the_library_and_the_tests
    - rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock
    - rust_agentic_foundations::test_crate::repo_gates::python::python_tracked_files_are_forbidden
    - rust_agentic_foundations::test_crate::repo_gates::python::python_extensions_are_refused_even_for_empty_or_binary_files
    - rust_agentic_foundations::test_crate::repo_gates::python::python_shebangs_are_refused_without_a_python_extension
    - rust_agentic_foundations::test_crate::repo_gates::python::python_mentions_and_similar_names_are_allowed
    - rust_agentic_foundations::test_crate::self_workplan::repository_workplan_is_clean_and_assessed
    - rust_agentic_foundations::test_crate::self_census::repository_census_is_clean_and_duplicate_growth_is_refused
  examples:
    - rust_agentic_foundations::example_crate::repository_gate
depends: [cap:workplan-document-model, cap:workplan-strict, cap:source-index, cap:census-debt-ratchet]
---

# Repository contract

## Purpose

The unpublished root package checks the repository's own documented system and
shared engineering rules. Its tests connect tracked files, capability inventory,
bindings, licence fingerprints, dependency restrictions and accepted prose
assessments to the same complete workplan policy that the libraries provide.

## Contract

**Bounded CI.** Every workflow job declares an integer `timeout-minutes` from
one through ten. Missing, dynamic, zero and excessive values refuse the native
YAML policy gate. The package matrix comes from Cargo's complete workspace
inventory. Source, fixture, capability-document, change-record and build inputs
select every package. Other text changes select the root package's repository
gates. Compiler, release, MSRV and fuzz execution checks apply to source and
asset changes. File licensing and dependency policy run for every change, with
both the root and independent fuzz dependency graphs checked. Aggregate test
and fuzz results require their applicable jobs to succeed.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::ci::every_workflow_job_has_an_explicit_ten_minute_ceiling -->

**Binding validation.** The self-workplan gate resolves a native execution plan
for every test and example binding without starting a process. Package tests
execute unit, integration and documentation tests. The `raf-ci` binary uses the
same native executor to run each documented standalone example target of its
selected package. Only a passing execution satisfies a repeated plan; a
host-inapplicable binding cannot suppress a later applicable binding. Unknown
packages, invalid plans, model findings and failing examples are hard errors.
Full `cargo workplan verify` remains available for explicit per-binding runtime
auditing and affected capability completion retains its native execution gate.
<!-- raf:tested-by rust_agentic_foundations::test_crate::self_workplan::repository_workplan_is_clean_and_assessed -->

**Self-Census.** The root package runs the production Census gate over the same
selected read root. The canonical `helpers-ledger.ttl` declares both the first-party
workspace and `fuzz` roots, with the standard token threshold and source limits.
`census-debt.ttl` holds the exact adopted duplication; growth and stale slack both
fail. The census must contain actual units from the root package and fuzz harness.
The read-only test never writes or increases debt. Ordinary checks, Linux CI and
the staged commit and merge snapshots execute the same test.
<!-- raf:tested-by rust_agentic_foundations::test_crate::self_census::repository_census_is_clean_and_duplicate_growth_is_refused -->

**Shared read root.** The gates and repository-gate example use `gate_root`:
`RAF_GATE_ROOT` selects an explicit checkout snapshot, and its absence selects
the package manifest directory. Both roots receive the same ordinary gate rules.
<!-- raf:tested-by rust_agentic_foundations::tests::the_absent_gate_root_uses_the_manifest_directory,rust_agentic_foundations::tests::an_explicit_gate_root_is_used_without_falling_back -->

**Tracked inventory.** The inventory names every capability with its exact state,
every architecture page, open change, record and generated catalogue. Missing,
extra, repeated or differently identified documents are refusals, including an
open change listing capabilities other than its inventory entry.
<!-- raf:tested-by rust_agentic_foundations::test_crate::docs_contract::inventory_accepts_a_matching_tree,rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_state_change,rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_change_record_that_lists_other_capabilities,rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_document_the_table_does_not_list -->

**Document contract.** Tracked documents are read through the profile, source
bindings and workplan model. Links use the tracked path and symlink oracle.
Generated blocks must match their renderer, delivered capabilities hold their
verification matrix, and documented Rust examples match their bound doctests.
<!-- raf:tested-by rust_agentic_foundations::test_crate::docs_contract::repository_documents_satisfy_the_profile,rust_agentic_foundations::test_crate::docs_contract::repository_bindings_resolve_in_the_source_index,rust_agentic_foundations::test_crate::docs_contract::the_checker_judges_links_against_the_tracked_paths_and_symlinks,rust_agentic_foundations::test_crate::docs_contract::a_block_that_differs_by_one_line_is_reported,rust_agentic_foundations::test_crate::docs_contract::usage_and_examples_blocks_are_the_crate_doctests -->

**Licence and version integrity.** Pinned licence text must retain its digest,
and the citation version must equal the workspace version. Altered text, missing
fingerprint records and a version changed on only one side are refused.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::licenses_accepts_bytes_that_hash_to_the_pin,rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_an_altered_mulan_text,rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_a_record_without_a_fingerprint_line,rust_agentic_foundations::test_crate::repo_gates::citation_accepts_matching_versions,rust_agentic_foundations::test_crate::repo_gates::citation_rejects_a_manifest_bump_without_a_matching_citation -->

**Manifest policy.** Every first-party manifest inherits the workspace lint
policy. The fuzz workspace mirrors the root lint and deny configuration with its
declared scoped exception. First-party normal, build, development, workspace and
target dependency declarations refuse the removed JSON result dependency,
including renamed packages; malformed manifests are hard errors.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::lints_every_workspace_manifest_inherits_the_workspace_lints,rust_agentic_foundations::test_crate::repo_gates::lints_rejects_a_weakened_level_in_the_fuzz_tables,rust_agentic_foundations::test_crate::repo_gates::deny_rejects_a_missing_or_widened_or_repeated_exception,rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class,rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them -->

**Source discipline.** Tracked text refuses tracker and development-history
references. The source clock rule checks every owned crate and its tests and
examples, with only its named clock implementation and rule-definition files
exempt. New crates are covered without another path inventory.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::issue_refs_reports_every_hit_as_file_and_line,rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_near_miss_prose,rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_each_source_spelling_in_the_library_and_the_tests,rust_agentic_foundations::test_crate::repo_gates::time_in_crates_passes_the_clean_neighbour,rust_agentic_foundations::test_crate::repo_gates::time_in_crates_judges_a_crate_added_under_the_crates_directory -->

**Current assessments.** The self-workplan test refuses any complete-policy
finding, any stale assessment or any verdict other than accepted. It requires at
least one loaded capability, so an empty or unreadable documentation set cannot
masquerade as a clean repository.
<!-- raf:tested-by rust_agentic_foundations::test_crate::self_workplan::repository_workplan_is_clean_and_assessed -->

**Python refusal.** Tracked paths refuse `.py`, `.pyw`, `.pyi`, `.pyc`, `.pyo`,
`.pyz` and `.pyzw` extensions case-insensitively, including empty and binary
files. A Python or PyPy interpreter shebang is refused regardless of the file
name, including direct and `env` interpreter forms. Documentation mentions and
shell commands that merely name Python remain valid. The commit and merge hooks
apply this Rust gate to a checkout of the staged index; CI runs the same gate.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::python::python_tracked_files_are_forbidden,rust_agentic_foundations::test_crate::repo_gates::python::python_extensions_are_refused_even_for_empty_or_binary_files,rust_agentic_foundations::test_crate::repo_gates::python::python_shebangs_are_refused_without_a_python_extension,rust_agentic_foundations::test_crate::repo_gates::python::python_mentions_and_similar_names_are_allowed -->

## Usage

Use `cargo test --locked --package <package>` for one package's ordinary suite.
Build the example runner with `cargo build --locked --bin raf-ci`, then run
`target/debug/raf-ci <package>` to execute its documented standalone examples
through native verification. The command reports each
example before execution and its actual outcome afterwards. Run `make fuzz-check`
for harness Clippy and dependency policy, or `make fuzz-run` for the instrumented
corpus and bounded mutations; `make fuzz` retains the complete sequence.

Run `cargo test --locked --test self_census` to judge the repository ledger and
debt. Adopt missing debt or lower resolved debt explicitly with
`cargo run --locked -p cargo-census -- debt --write`, then review the complete
`census-debt.ttl` diff. The writer refuses growth into a current record; deleting
that record and adopting again does not prove that debt did not grow.

Run the `rust_agentic_foundations::example_crate::repository_gate` example
from this checkout to apply the ordinary gate to the repository. The root tests
add inventory, licence, dependency and assessment checks that are specific to
this unpublished package; library consumers use their own repository policy.

Run `cargo test --locked --test repo_gates` to apply the repository policy
checks, including the Python refusal. Install the commit and merge hooks with
`make hooks`; `make check` and CI also run the repository gates.

## Examples

A workflow job with `timeout-minutes: 10` passes the budget rule. Omitting the
property, spelling it as an expression, or giving eleven refuses and names the
job. Adding a workspace member automatically adds it to the source-change test
matrix. A documented example shared by several capabilities has one successful
execution for its package, while every binding still receives native planning.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::ci::every_workflow_job_has_an_explicit_ten_minute_ceiling,rust_agentic_foundations::test_crate::self_workplan::repository_workplan_is_clean_and_assessed -->

A clean self-Census accepts the currently adopted groups. Adding another
normal-form-equivalent implementation creates growth and makes the same gate
refuse. Removing duplication also requires explicitly lowering the debt record,
so its allowance never carries unused slack.

The repository-gate example opens the selected read root through the ordinary
Gate API and reports failure through its result. A clean documented checkout
succeeds; editing a contract or breaking a binding makes the same gate refuse.
Its source is the runnable example rather than a second copied implementation.

An empty `tools/check.py` or an extensionless script beginning with
`#!/usr/bin/env python3` fails the repository gates. A Rust tool in
`tools/check.rs` and prose discussing Python pass the Python policy check.

## Constraints

Workflow budgets must be literal integers from one through ten; an absent,
dynamic, zero or excessive value refuses the repository gate.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::ci::every_workflow_job_has_an_explicit_ten_minute_ceiling -->

- Self-Census applies the ordinary native Census normal form and exact debt
  ratchet. It does not claim preferred-provider execution or external consumer
  acceptance. Canonical ledger and debt bytes cannot carry inline comments, so
  their licensing metadata is recorded in `REUSE.toml`.
<!-- raf:tested-by rust_agentic_foundations::test_crate::self_census::repository_census_is_clean_and_duplicate_growth_is_refused -->

- These gates judge tracked repository policy, not arbitrary documents outside
  the tracked-file oracle. Non-regular files and symlinks are refused before
  their bytes can stand for ordinary tracked text.
<!-- raf:tested-by rust_agentic_foundations::test_crate::docs_contract::refuses_non_regular_files_instead_of_reading_them,rust_agentic_foundations::test_crate::docs_contract::rejects_a_tracked_symlink_that_nothing_links_to -->
- Text scanning treats a NUL byte as binary and decodes other bytes for the
  repository reference rules. A non-UTF-8 text file therefore cannot hide a
  forbidden reference merely by using a different character encoding.
<!-- raf:tested-by rust_agentic_foundations::test_crate::repo_gates::issue_refs_flags_a_token_in_a_latin_1_file,rust_agentic_foundations::test_crate::repo_gates::issue_refs_treats_only_a_nul_byte_as_binary,rust_agentic_foundations::test_crate::repo_gates::issue_refs_skips_a_nul_containing_file_holding_a_token -->

## Explanation

The job ceiling bounds execution on a runner; runner queue delay is external.
Process-free planning establishes source-to-invocation validity. Exact Cargo
listing, applicability and per-binding runtime receipts require native execution.
Capability documents are test inputs, so their changes retain full test coverage.
Fuzzing keeps its sanitizer, optimization, assertion policy, five committed
corpora and all bounded mutations when using sixteen code generation units.

Repository-only guarantees have a distinct owner because they concern the
checked-in RAF system rather than the public behavior of a single library.
Keeping their actual tests bound makes their source part of the complete
coverage account without assigning unrelated policy to a feature capability.

## Verification

The self-Census test requires a nonempty actual repository census and checks
both declared root families. A controlled valid fixture passes, while a second
equivalent function creates duplicate growth through the same production gate
and debt rules. Neither fixture writes debt. The missing-document inventory
case preserves distinct capability and change-record refusals.
<!-- raf:tested-by rust_agentic_foundations::test_crate::self_census::repository_census_is_clean_and_duplicate_growth_is_refused,rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_missing_capability_and_change_documents -->

The bound tests contain accepted neighbors and precise refusals for inventory,
file kinds, profile loading, links, generated blocks, licences, versions,
manifest inheritance, dependency aliases, source rules and current assessments.
The runnable example uses the same Gate API as the repository test; it performs
no separate interpretation of the documents or source index.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_workplan::test_crate::history::a_fixture_commit_obeys_an_installed_hook_and_the_successful_neighbor` | test | [Verification](#verification) |
| `rust_agentic_foundations::example_crate::repository_gate` | example target | [Usage](#usage) |
| `rust_agentic_foundations::test_crate::docs_contract::a_block_that_differs_by_one_line_is_reported` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::docs_contract::a_delivered_capability_holds_a_verification_matrix` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::a_tilde_fenced_usage_block_is_accepted_and_an_unclosed_block_is_refused` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::an_open_capability_may_be_listed_by_a_record_named_after_another` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::doctest_comparison_fixtures` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_accepts_a_matching_tree` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_refuses_a_record_id_of_no_record_type` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_capability_listed_twice_even_when_the_record_lists_it_twice` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_change_record_that_lists_other_capabilities` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_document_the_table_does_not_list` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_missing_or_unlisted_record` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_state_change` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_an_id_change` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_an_index_the_table_does_not_list` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_missing_architecture_and_index_documents` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_missing_capability_and_change_documents` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::inventory_table_has_the_documented_shape` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::refuses_a_file_swapped_in_after_inspection` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::refuses_non_regular_files_instead_of_reading_them` | test | [Constraints](#constraints) |
| `rust_agentic_foundations::test_crate::docs_contract::rejects_a_tracked_symlink_that_nothing_links_to` | test | [Constraints](#constraints) |
| `rust_agentic_foundations::test_crate::docs_contract::rejects_a_typed_document_outside_the_document_directories` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::rejects_non_markdown_file_in_a_document_directory` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::repository_bindings_resolve_in_the_source_index` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::docs_contract::repository_documents_satisfy_the_profile` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::docs_contract::the_checker_accepts_a_valid_checkout` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::the_checker_judges_links_against_the_tracked_paths_and_symlinks` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::docs_contract::the_checker_judges_markdown_outside_the_document_directories` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::the_checker_judges_the_documents_through_the_model` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::docs_contract::usage_and_examples_blocks_are_the_crate_doctests` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::ci::every_workflow_job_has_an_explicit_ten_minute_ceiling` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `rust_agentic_foundations::test_crate::repo_gates::citation_accepts_matching_versions` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::citation_rejects_a_manifest_bump_without_a_matching_citation` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::citation_rejects_a_missing_version` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::citation_rejects_a_version_bump_without_a_matching_manifest` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::citation_version_matches_the_workspace` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::deny_accepts_the_root_configuration_plus_the_one_exception` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::deny_fuzz_configuration_is_the_root_configuration_plus_one_exception` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::deny_rejects_a_dropped_root_exception` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::deny_rejects_a_missing_or_widened_or_repeated_exception` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::deny_rejects_a_shared_section_that_differs` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_first_party_manifests_have_no_json_result_dependency` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_flags_a_token_in_a_latin_1_file` | test | [Constraints](#constraints) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_change_number` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_colon_prefixed_number` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_github_style_number` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_hash_number_after_a_space` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_hash_number_after_a_word_character` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_parenthesised_hash_number` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_plan_possessive` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_pull_request_label` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_pull_url` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_a_task_number` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_an_acceptance_label` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_an_all_digit_css_colour` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_an_issue_url` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_per_the_plan` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_the_word_issue_with_a_number` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_the_words_pull_request_with_a_number` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_this_branch` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_rejects_this_issue` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_reports_every_hit_as_file_and_line` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_scans_text_files_and_skips_binary_ones` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_skips_a_nul_containing_file_holding_a_token` | test | [Constraints](#constraints) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_a_hex_colour_with_letters` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_a_quoted_three_digit_colour` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_a_six_digit_colour` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_an_xml_character_reference` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_near_miss_prose` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_tracked_files_are_clean` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::issue_refs_treats_only_a_nul_byte_as_binary` | test | [Constraints](#constraints) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_accepts_a_matching_fingerprint` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_accepts_bytes_that_hash_to_the_pin` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_mulan_texts_match_the_pinned_digest` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_a_fingerprint_mismatch` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_a_record_without_a_fingerprint_line` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_an_altered_mulan_text` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_require_a_canonical_fingerprint_with_non_hex_boundaries` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_sha256_matches_known_vectors` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::licenses_zh_hans_matches_the_recorded_fingerprint` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_accepts_equal_tables_whatever_the_comments` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_every_workspace_manifest_inherits_the_workspace_lints` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_fuzz_workspace_tables_equal_the_root_tables` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_reads_single_and_multi_line_member_lists` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_reads_the_level_of_a_lint_and_sees_a_weakened_one` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_rejects_a_member_manifest_that_does_not_inherit` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_rejects_a_root_without_lint_tables` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_rejects_a_weakened_level_in_the_fuzz_tables` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_rejects_an_added_or_removed_lint_in_the_fuzz_tables` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::lints_the_root_denies_missing_docs_unsafe_code_and_unreachable_pub` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::python::python_extensions_are_refused_even_for_empty_or_binary_files` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::python::python_mentions_and_similar_names_are_allowed` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::python::python_shebangs_are_refused_without_a_python_extension` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::python::python_tracked_files_are_forbidden` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_exempts_the_clock_file_but_not_its_neighbour` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_exempts_the_gate_file_but_not_its_neighbours` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_exempts_the_paths_outside_every_rule` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_judges_a_crate_added_under_the_crates_directory` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_lets_the_runner_name_durations_and_limits` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_passes_the_clean_neighbour` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_clock_reading_with_file_and_line` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_a_planted_reading_in_every_ruled_directory` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_each_crate_spelling_in_every_other_directory` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_each_source_spelling_in_the_library_and_the_tests` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_tracked_files_name_no_clock` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::self_census::repository_census_is_clean_and_duplicate_growth_is_refused` | test | [Contract](#contract), [Constraints](#constraints) |
| `rust_agentic_foundations::test_crate::self_workplan::repository_workplan_is_clean_and_assessed` | test | [Contract](#contract), [Examples](#examples) |
| `rust_agentic_foundations::tests::an_explicit_gate_root_is_used_without_falling_back` | test | [Contract](#contract) |
| `rust_agentic_foundations::tests::the_absent_gate_root_uses_the_manifest_directory` | test | [Contract](#contract) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-debt-ratchet](./census-debt-ratchet.md) |
| Depends on | [source-index](./source-index.md) |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depends on | [workplan-strict](./workplan-strict.md) |
<!-- raf:generated:end -->
