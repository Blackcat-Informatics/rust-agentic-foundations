const fs=require('fs');
const b=String.fromCharCode(96);
const log=fs.readFileSync('.stage/raf-workplan-strict-mode-force-coverage/raw/queue-sole-policy-repository-before-adoption.log','utf8');
const tests=[...log.matchAll(/the test `(rust_agentic_foundations[^`]+)`/g)].map(m=>m[1]);
tests.push(...['rdf_first_party_manifests_have_no_json_result_dependency','rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class','rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them'].map(n=>'rust_agentic_foundations::test_crate::repo_gates::dependencies::'+n));
const marker=names=>'<!-- raf:tested-by '+names.map(n=>'rust_agentic_foundations::test_crate::'+n).join(',')+' -->';
const text=`---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:repository-contract
title: "Repository contract"
description: "The tracked document inventory, repository policy gates and accepted assessment check of the unpublished root package."
state: implementing
bindings:
  code:
    - rust_agentic_foundations
  tests:
${[...new Set(tests)].sort().map(n=>'    - '+n).join('\n')}
  examples:
    - rust_agentic_foundations::example_crate::repository_gate
depends: [cap:workplan-document-model, cap:workplan-strict, cap:source-index]
---

# Repository contract

## Purpose

The unpublished root package checks the repository's own documented system and
shared engineering rules. Its tests connect tracked files, capability inventory,
bindings, licence fingerprints, dependency restrictions and accepted prose
assessments to the same complete workplan policy that the libraries provide.

## Contract

**Tracked inventory.** The inventory names every capability with its exact state,
every architecture page, open change, record and generated catalogue. Missing,
extra, repeated or differently identified documents are refusals, including an
open change listing capabilities other than its inventory entry.
${marker(['docs_contract::inventory_accepts_a_matching_tree','docs_contract::inventory_rejects_a_state_change','docs_contract::inventory_rejects_a_change_record_that_lists_other_capabilities','docs_contract::inventory_rejects_a_document_the_table_does_not_list'])}

**Document contract.** Tracked documents are read through the profile, source
bindings and workplan model. Links use the tracked path and symlink oracle.
Generated blocks must match their renderer, delivered capabilities hold their
verification matrix, and documented Rust examples match their bound doctests.
${marker(['docs_contract::repository_documents_satisfy_the_profile','docs_contract::repository_bindings_resolve_in_the_source_index','docs_contract::the_checker_judges_links_against_the_tracked_paths_and_symlinks','docs_contract::a_block_that_differs_by_one_line_is_reported','docs_contract::usage_and_examples_blocks_are_the_crate_doctests'])}

**Licence and version integrity.** Pinned licence text must retain its digest,
and the citation version must equal the workspace version. Altered text, missing
fingerprint records and a version changed on only one side are refused.
${marker(['repo_gates::licenses_accepts_bytes_that_hash_to_the_pin','repo_gates::licenses_rejects_an_altered_mulan_text','repo_gates::licenses_rejects_a_record_without_a_fingerprint_line','repo_gates::citation_accepts_matching_versions','repo_gates::citation_rejects_a_manifest_bump_without_a_matching_citation'])}

**Manifest policy.** Every first-party manifest inherits the workspace lint
policy. The fuzz workspace mirrors the root lint and deny configuration with its
declared scoped exception. First-party normal, build, development, workspace and
target dependency declarations refuse the removed JSON result dependency,
including renamed packages; malformed manifests are hard errors.
${marker(['repo_gates::lints_every_workspace_manifest_inherits_the_workspace_lints','repo_gates::lints_rejects_a_weakened_level_in_the_fuzz_tables','repo_gates::deny_rejects_a_missing_or_widened_or_repeated_exception','repo_gates::dependencies::rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class','repo_gates::dependencies::rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them'])}

**Source discipline.** Tracked text refuses tracker and development-history
references. The source clock rule checks every owned crate and its tests and
examples, with only its named clock implementation and rule-definition files
exempt. New crates are covered without another path inventory.
${marker(['repo_gates::issue_refs_reports_every_hit_as_file_and_line','repo_gates::issue_refs_spares_near_miss_prose','repo_gates::time_in_crates_rejects_each_source_spelling_in_the_library_and_the_tests','repo_gates::time_in_crates_passes_the_clean_neighbour','repo_gates::time_in_crates_judges_a_crate_added_under_the_crates_directory'])}

**Current assessments.** The self-workplan test refuses any complete-policy
finding, any stale assessment or any verdict other than accepted. It requires at
least one loaded capability, so an empty or unreadable documentation set cannot
masquerade as a clean repository.
${marker(['self_workplan::repository_workplan_is_clean_and_assessed'])}

## Usage

Run the ${b}rust_agentic_foundations::example_crate::repository_gate${b} example
from this checkout to apply the ordinary gate to the repository. The root tests
add inventory, licence, dependency and assessment checks that are specific to
this unpublished package; library consumers use their own repository policy.

## Examples

The repository-gate example opens the manifest directory through the ordinary
Gate API and reports failure through its result. A clean documented checkout
succeeds; editing a contract or breaking a binding makes the same gate refuse.
Its source is the runnable example rather than a second copied implementation.

## Constraints

- These gates judge tracked repository policy, not arbitrary documents outside
  the tracked-file oracle. Non-regular files and symlinks are refused before
  their bytes can stand for ordinary tracked text.
${marker(['docs_contract::refuses_non_regular_files_instead_of_reading_them','docs_contract::rejects_a_tracked_symlink_that_nothing_links_to'])}
- Text scanning treats a NUL byte as binary and decodes other bytes for the
  repository reference rules. A non-UTF-8 text file therefore cannot hide a
  forbidden reference merely by using a different character encoding.
${marker(['repo_gates::issue_refs_flags_a_token_in_a_latin_1_file','repo_gates::issue_refs_treats_only_a_nul_byte_as_binary','repo_gates::issue_refs_skips_a_nul_containing_file_holding_a_token'])}

## Explanation

Repository-only guarantees have a distinct owner because they concern the
checked-in RAF system rather than the public behavior of a single library.
Keeping their actual tests bound makes their source part of the complete
coverage account without assigning unrelated policy to a feature capability.

## Verification

The bound tests contain accepted neighbors and precise refusals for inventory,
file kinds, profile loading, links, generated blocks, licences, versions,
manifest inheritance, dependency aliases, source rules and current assessments.
The runnable example uses the same Gate API as the repository test; it performs
no separate interpretation of the documents or source index.

<!-- raf:generated:verification-matrix -->
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
<!-- raf:generated:end -->
`;
if(fs.existsSync('docs/capabilities/repository-contract.md'))throw Error('already exists');
fs.writeFileSync('docs/capabilities/repository-contract.md',text);
let record=fs.readFileSync('work/changes/repository-adoption.md','utf8');
record=record.replace('title: Name this change','title: "Repository adoption of the complete policy"').replace('description: Say in one line what this change does to the capabilities it lists.','description: "All built first-party packages substantiate their documented claims, bindings, runnable examples and counted source lines without a debt baseline."').replace(/^(changes: \[[^\n]+)\]$/m,'$1, cap:repository-contract]');
record=record.replace('contract_effect: changes','  - cap:repository-contract#purpose\n  - cap:repository-contract#contract\n  - cap:repository-contract#usage\n  - cap:repository-contract#examples\n  - cap:repository-contract#constraints\n  - cap:repository-contract#explanation\n  - cap:repository-contract#verification\ncontract_effect: changes');
record=record.replace('items: []','items:\n  - claim: "Each delivered capability has complete authored sections, runnable examples, a current verification matrix and claim-specific bound tests."\n  - claim: "Every public item and test of every built first-party package has a truthful capability owner."\n  - claim: "All counted first-party shipping and test lines are explained, with zero orphan lines and no debt baseline."\n  - claim: "The root repository contract owns its actual inventory, manifest, licence, source-policy and assessment gates."\n  - claim: "The generated documentation and accepted assessments describe the actual complete-policy tree."');
record=record.replace('Describe what this change does and why, in a short paragraph.','The built first-party system adopts the complete workplan policy through actual\ncapability ownership, executable examples and claim-level proof. The measured\ncoverage account includes the root package, libraries, command wrapper, shared\nRDF crate and fuzz workspace. Recording existing debt is not a successful\nzero-debt adoption.');
fs.writeFileSync('work/changes/repository-adoption.md',record);
