---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-reference-scan
title: "Census reference scan"
description: "Compare first-party code with exact dependency identities selected through lock files and manifest declarations, or with caller-declared pinned commits and unpinned directories."
state: delivered
bindings:
  code:
    - raf_census::gate::reference
    - raf_census::report::reference
    - raf_census::Gate::reference_roots
    - raf_census::Gate::check_tree_with_references
    - raf_census::gate::Gate::evaluate_reference_tree
    - raf_source::ReferenceObservation
    - raf_source::ReferenceRequest
    - raf_source::workspace::locked::reference
    - raf_census::ReferenceSource
    - raf_census::ReferenceWitness
    - raf_census::ReferenceConstantRow
    - raf_census::ExternalReference
    - raf_census::Gate::external_references
    - raf_census::gate::reference::external
    - raf_source::PinnedGitTree
    - raf_source::PinnedGitTree::open
    - raf_source::Workspace::package_version
    - raf_source::git::Repository::main_work_dir
  tests:
    - raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count
    - raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts
    - raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator
    - raf_census::test_crate::reference_scan::family_selector_scans_every_matching_dependency_but_no_member
    - raf_census::test_crate::reference_scan::family_selector_follows_lock_growth_without_a_ledger_edit
    - raf_census::test_crate::reference_scan::family_selector_matching_nothing_refuses_naming_the_selector
    - raf_census::test_crate::reference_scan::exact_and_family_selectors_scan_a_package_once
    - raf_census::test_crate::reference_scan::family_selector_excludes_member_rows_not_every_row_with_a_member_name
    - raf_census::test_crate::reference_scan::family_selector_refuses_a_name_with_two_reachable_versions
    - raf_census::test_crate::reference_scan::unhomed_reference_copy_is_a_hard_finding
    - raf_census::test_crate::reference_scan::unhomed_reference_bans_shim_and_member_copies_beside_changed_neighbours
    - raf_census::test_crate::reference_scan::locked_git_reference_copy_is_a_hard_finding
    - raf_census::test_crate::reference_scan::first_party_shared_constants_stay_in_the_baseline_beside_the_cross_boundary_rows
    - raf_census::test_crate::reference_scan::large_constants_shared_with_a_reference_cross_the_boundary_by_value
    - raf_census::test_crate::reference_scan::an_index_without_a_reference_has_no_reference_constant_rows
    - raf_census::test_crate::reference_scan::a_constant_held_only_by_two_reference_packages_has_no_row
    - raf_census::test_crate::reference_scan::external_references_report_cross_boundary_constants_like_a_lock_selected_one
    - raf_census::test_crate::reference_scan::pinned_commit_reports_reference_copies_exactly_as_a_lock_selected_reference
    - raf_census::test_crate::reference_scan::pinned_reference_refusals_name_the_failed_input_beside_the_exact_commit
    - raf_census::test_crate::reference_scan::linked_worktree_and_main_checkout_compare_against_the_same_pinned_tree
    - raf_census::test_crate::reference_scan::unpinned_directory_finds_the_same_copies_and_labels_every_receipt
    - raf_source::test_crate::git_tree::pinned_tree_never_reads_or_size_checks_a_file_the_caller_already_holds
    - cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory
    - cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict

    - raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes
    - raf_source::test_crate::locked_sources::registry_reference_observations_match_selected_manifest_identity
    - raf_source::test_crate::locked_sources::local_reference_owner_selection_spends_its_original_work_allowance
    - cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets
  examples:
    - raf_census::example_crate::references
depends: [cap:census-ledger]
---

# Census reference scan

## Purpose
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count -->

A project can duplicate an implementation that one of its dependencies already
provides. Reference scanning compares first-party implementations with named
dependency packages, or with every package of a named family, and permits a job
to name its home in a dependency. It can also compare them with another
repository at an exact reviewed commit that the lock graph does not record.


## Contract

### Declaration selection
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes,raf_source::test_crate::locked_sources::registry_reference_observations_match_selected_manifest_identity,raf_census::test_crate::reference_scan::family_selector_scans_every_matching_dependency_but_no_member,raf_census::test_crate::reference_scan::family_selector_follows_lock_growth_without_a_ledger_edit,raf_census::test_crate::reference_scan::family_selector_matching_nothing_refuses_naming_the_selector,raf_census::test_crate::reference_scan::exact_and_family_selectors_scan_a_package_once,raf_census::test_crate::reference_scan::family_selector_excludes_member_rows_not_every_row_with_a_member_name,raf_census::test_crate::reference_scan::family_selector_refuses_a_name_with_two_reachable_versions -->

The native RDF ledger's `reference` values name dependency packages. The owning
package's actual manifest name and explicit or inherited version select its lock
row. Actual dependency declarations, including inherited declarations and renamed
packages, select exact outgoing rows. Missing, ambiguous, incompatible and
unavailable sources fail with context. Acquisition uses lock files and manifests;
it does not execute Cargo metadata, download packages or inspect shared caches.

A `reference` value is an exact package name or a family selector: a
package-name prefix followed by `-*`, such as `acme-*`, which names every
package whose name continues that prefix after its hyphen: `acme-a` and
`acme-a-extra` match `acme-*`, while `acme`, `acmeish` and `other-x` do not.
Selection resolves the selectors over the lock rows reachable from the
first-party packages, in the dependency planning step and again when the
reference sources are prepared; both passes read the same lock graph and
resolve the same names. A family selects every reachable locked row it names except the rows of
first-party packages, so an external package that shares a first-party
package's name at another version is still selected, and a package added to the lock graph under the prefix is
scanned with no ledger change. A family that selects no package refuses with a
message naming the selector. An exact name keeps selecting a first-party
package. A package selected by several values is scanned once.
A selected name with two reachable locked versions refuses as ambiguous, whether
an exact name selected it or a family selected two non-first-party versions.

### Declared reference trees
<!-- raf:tested-by raf_census::test_crate::reference_scan::pinned_commit_reports_reference_copies_exactly_as_a_lock_selected_reference,raf_census::test_crate::reference_scan::pinned_reference_refusals_name_the_failed_input_beside_the_exact_commit,raf_census::test_crate::reference_scan::linked_worktree_and_main_checkout_compare_against_the_same_pinned_tree,raf_census::test_crate::reference_scan::unpinned_directory_finds_the_same_copies_and_labels_every_receipt,cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory,raf_source::test_crate::git_tree::pinned_tree_never_reads_or_size_checks_a_file_the_caller_already_holds -->

A caller can declare a reference tree that the lock graph does not select,
through `Gate::external_references` and an `ExternalReference`. A pinned
reference names a repository location and a commit: a full 40-character
lowercase hexadecimal SHA-1 given directly, or the string value of a dotted
key in a workspace-relative TOML pin file. `PinnedGitTree` reads that commit's
committed objects through the native read-only Git library; it never reads the
repository's working tree, fetches an object or starts a process, and it
carries no lock identity, so it never supplies preferred-provider proof. A file the caller already holds is returned as seen without a size check or
blob read, so an accepted oversized file is never refused or read. An
unpinned reference names a directory and reads its current files through a
contained disk tree.

Every package that the ordinary source walk discovers from the tree's root is
a reference package, whatever the ledger's `reference` values select. Its
version is the manifest's explicit or workspace-inherited version, or `0.0.0`
when the manifest sets none. Declared packages are compared exactly as
lock-selected ones, so the same copy yields the same `REFERENCE-COPY` finding,
and a job's home can name one of their implementations. A relative location is
resolved from the main checkout of the workspace's Git repository, so a linked
worktree and the main checkout read the same tree and produce identical
reports. A relative location refuses where there is no main checkout to
resolve it from, including on an opaque Tree. Each receipt records the
location exactly as declared.
A pinned receipt has source kind `git`, the commit as its revision and the
`committed-git` observation. An unpinned receipt has source kind `path`, no
revision and the `unpinned-tree` observation, so every index and query from
that mode marks its source as unpinned. `cargo census` also prints one notice
on stderr naming the unpinned directory for every evaluating command, so a
`check` run is marked although its findings are identical to the pinned run.
Because a debt record cannot carry that mark, an unpinned comparison refuses to
record debt.

A malformed commit ID, a missing repository, a commit absent from the
repository, a location inside a repository rather than its root, a pin key that
names no string, a tree that yields no packages, and one tree declared twice
each refuse with a message naming the input to fix. An empty comparison is
never reported as clean.

### Contained acquisition
<!-- raf:tested-by raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes,raf_source::test_crate::locked_sources::registry_reference_observations_match_selected_manifest_identity -->

Path dependencies inside the supplied Tree use their actual declaring manifest.
Inherited paths use the workspace manifest that supplied the declaration.
Parent segments receive a source-relative image only after every original
directory prefix is a real directory and both manifest spellings have the same
Tree identity. Parent traversal through a directory symlink requires an explicit
manifest locator, including when the manifests are hardlinks.
Host paths outside that Tree use a separate contained disk root. An explicit
path locator must resolve to the manifest named by the actual declaration.
External registry and Git sources require explicit locally available locators.
Git source is read from exact committed objects through `CommittedGitTree`.
The Gate deducts its actual Git opening usage before source admission. Each
committed observation reopens the sealed source under the same Meter as
declaration selection and source loading; caller-owned Tree construction remains
outside that observation's own usage.
Registry source includes its explicit index mapping and recorded archive checksum;
a contained Tree observation does not authenticate that archive.

### Observation authority
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes,raf_source::test_crate::locked_sources::registry_reference_observations_match_selected_manifest_identity,raf_source::test_crate::locked_sources::local_reference_owner_selection_spends_its_original_work_allowance -->

A `ReferenceObservation` validates actual package name and version, including
workspace inheritance. Its source workspace exposes no locked provider binding or
configured external dependency edge. Reference observations never constitute
preferred-provider proof.

### Matching and isolation
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts,raf_census::test_crate::reference_scan::unhomed_reference_copy_is_a_hard_finding,raf_census::test_crate::reference_scan::unhomed_reference_bans_shim_and_member_copies_beside_changed_neighbours,raf_census::test_crate::reference_scan::locked_git_reference_copy_is_a_hard_finding -->

Reference units use the same normal form as first-party units. They remain
separate from first-party structural groups, ordinary rule execution and group
debt, including when the selected dependency is a workspace member. A job's home
or entry point can name a reference implementation or module. Full body and shim
fingerprints from reference homes classify first-party copies even when the
job's explicit forbidden selectors are empty. Each physical consumer counts once
per job; every matching reference witness is retained. A copy without a matching
reference home is a `REFERENCE-COPY` finding that names the consumer, the
reference unit and the fingerprint. It is never debt, and a baseline or debt record cannot hold it.

**The ban.** The declaration that bans copies is a ledger `reference` that no
job names as a home, with no other job needed: a ledger with a `reference` and
an empty job set evaluates and bans. Every first-party body or shim whose full
fingerprint equals a reference unit is then a `REFERENCE-COPY` finding, whether
the dependency is selected by an exact name, a family, a sealed Git or registry
source under `Gate::check_locked`, or is also a workspace member; the member's
own units are not copies of themselves. `check` fails, `debt --write` refuses
the finding and records nothing, and a body that differs stays clean. A
reference with a home job is a different declaration: its copies are classified
into the job, and an unenforced job records them as job debt that may only
shrink under the debt ratchet, while an enforced job reports them as findings of that job.
Choose the unhomed form to forbid every copy, and a homed job to adopt a
reference module with a recorded allowance. The index, `job`, `count` and
`fingerprint` evaluate the same inputs as `check`, so they cannot disagree.
<!-- raf:tested-by raf_census::test_crate::reference_scan::unhomed_reference_copy_is_a_hard_finding,raf_census::test_crate::reference_scan::unhomed_reference_bans_shim_and_member_copies_beside_changed_neighbours,raf_census::test_crate::reference_scan::locked_git_reference_copy_is_a_hard_finding,cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict -->

### Closed native evidence
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_census::test_crate::reference_scan::large_constants_shared_with_a_reference_cross_the_boundary_by_value,raf_census::test_crate::reference_scan::first_party_shared_constants_stay_in_the_baseline_beside_the_cross_boundary_rows,raf_census::test_crate::reference_scan::an_index_without_a_reference_has_no_reference_constant_rows,raf_census::test_crate::reference_scan::unpinned_directory_finds_the_same_copies_and_labels_every_receipt,raf_census::test_crate::reference_scan::a_constant_held_only_by_two_reference_packages_has_no_row,raf_census::test_crate::reference_scan::external_references_report_cross_boundary_constants_like_a_lock_selected_one -->

Report form 5 carries source receipts, qualified reference units and full
cross-boundary fingerprint witnesses. It also carries each
cross-boundary shared constant: an integer constant strictly above `0xffff` that
at least one first-party unit and at least one reference unit both hold, compared
by value in any spelling, listed with its first-party and reference members. A
value held only by first-party units stays in the baseline shared constants, and a
value held only by reference units, or at or below the threshold, is not reported.
An index without such a constant has no such row. Every job occurrence resolves to an indexed
unit. Native Index queries retain the selected occurrences and incident witness,
endpoint and source closure. Reference units cannot receive a COPY role or enter
a first-party structural group. Migrating form 4 preserves its ordinary
first-party rows without inventing historical source observations. Normal form 1
fingerprints remain unchanged.
Form 5 receipts also admit the `unpinned-tree` observation, only with the `path`
source kind and no checksum; every existing receipt keeps its bytes and
identity.

## Usage
<!-- raf:tested-by raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator -->

Run `raf_census::example_crate::references` for an owned path dependency and its
complete native Index and query round trips.

Name a dependency and its job home in the native ledger:

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-report/4a9ae8720de8c714365401e1d1b156942dcbdb3667864cc9cf861b136250bf85>
<> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#Ledger> .
<> <https://blackcatinformatics.ca/raf/report/1#job> <#job> .
<> <https://blackcatinformatics.ca/raf/report/1#minTokens> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
<> <https://blackcatinformatics.ca/raf/report/1#normalForm> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
<> <https://blackcatinformatics.ca/raf/report/1#reference> "acme-core" .
<> <https://blackcatinformatics.ca/raf/report/1#reportForm> "5"^^<http://www.w3.org/2001/XMLSchema#integer> .
<> <https://blackcatinformatics.ca/raf/report/1#root> "." .
<#job> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#LedgerJob> .
<#job> <https://blackcatinformatics.ca/raf/report/1#enforced> "false"^^<http://www.w3.org/2001/XMLSchema#boolean> .
<#job> <https://blackcatinformatics.ca/raf/report/1#homeSymbol> "acme_core::iri::parse" .
<#job> <https://blackcatinformatics.ca/raf/report/1#jobId> "iri-parse" .
<#job> <https://blackcatinformatics.ca/raf/report/1#summary> "Shared dependency implementations" .
```

`Gate::workspace(root).check()` acquires contained path dependencies.
`Gate::reference_roots` supplies locally available external roots and manifests,
including workspace context required for inherited package fields. A registry
locator also supplies the explicit registry-name/index mapping.
A family value such as `acme-*` in place of `acme-core` selects every reachable
locked dependency package named `acme-` and a suffix, with no ledger edit when
the family gains a package.
`Gate::check_tree_with_references` accepts opaque source observations with the
same identity checks; traversing their path declarations requires an owning
source locator. The native CLI reads the same Gate directly and exports its complete Index.
Repository callbacks are supplied through the library builder, never by selecting
an executable target on the shipped command.
`Gate::external_references` declares a reference tree outside the lock graph,
and the command line declares one with
`--reference-repo ../upstream --reference-commit COMMIT`, with
`--reference-repo ../upstream --reference-pin-file pins.toml --reference-pin-key upstream.rev`,
or with `--reference-unpinned DIRECTORY`. Such a tree needs no ledger
`reference` value; a job home may name its implementations.
To adopt the ban, give the ledger one `reference` line per package or family
(`<> report:reference "acme-*"`), no job homing it, and a `Cargo.lock` that
lists the packages; add a reference option only for a tree outside the lock
graph. A selector that matches nothing, a missing source, a malformed option
or an unavailable commit makes `check`, `index`, `job`, `count`, `fingerprint`
and `debt` refuse with a message and no verdict on stdout.
The pin key `upstream.rev` reads the string `rev` of the `[upstream]` table.
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets,cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict,raf_census::test_crate::reference_scan::family_selector_follows_lock_growth_without_a_ledger_edit,cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory,raf_census::test_crate::reference_scan::linked_worktree_and_main_checkout_compare_against_the_same_pinned_tree -->

## Examples
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts -->

A first-party function whose body equals two implementations in a reference
module contributes one physical COPY to that module's job and two full body
witnesses. A function with a different body remains a clean neighbor.

A reference package containing duplicate implementations contributes no
first-party structural group or group debt. Its HOME rows carry qualified source
locators and source receipts in the native Index.

For contained path dependencies, ordinary `Gate::check` and the native
`cargo census index` path use the production evaluator. External source
observations are supplied directly through the library's reference APIs.
The command never builds a repository target to obtain a reference index.
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

## Constraints

- Only Rust source is compared. Named references and family members must be
  reachable from the owning lock graph; a caller-declared tree is outside it and
  contributes every package found from its root.
  <!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_census::test_crate::reference_scan::family_selector_scans_every_matching_dependency_but_no_member -->
- Source reads retain the source loader's containment, regular-file, file-size
  and parse limits. Acquisition shares cumulative source, item, fact, resolver
  and work allowances across observations. Exhaustion fails rather than returning
  a partial index.
  <!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes,raf_source::test_crate::locked_sources::local_reference_owner_selection_spends_its_original_work_allowance -->
- Source roots retain workspace manifests needed for inherited package metadata.
  <!-- raf:tested-by raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->
- Witness pair work, retained states and native storage growth consume the same
  explicit matching grant as subsequent policy matching. Exhaustion refuses the
  entire scan; witness evidence is never truncated.
  <!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count -->
- Parent traversal through a directory symlink requires an explicit manifest
  locator; matching manifest file identity does not admit a different directory.
  <!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

## Explanation
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

Lock identities and actual dependency declarations provide the selection context.
A caller-declared commit supplies that context for a repository whose reviewed
revision is maintained apart from dependency resolution; reading only its
committed objects keeps the comparison reproducible, and resolving relative
locations from the main checkout keeps linked worktrees in agreement.
Source observations supply comparison evidence while preserving the stronger
sealed authority required for preferred-provider admission. Source-qualified
locators prevent independent workspaces' local package and item identifiers from
colliding in a report.

## Verification
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes,raf_census::test_crate::reference_scan::family_selector_scans_every_matching_dependency_but_no_member,raf_census::test_crate::reference_scan::unhomed_reference_copy_is_a_hard_finding -->

Owned fixtures cover inherited renamed path dependencies, workspace-member
selection, family selection with a matching member, lock growth, empty and
malformed families and overlapping selectors, unhomed reference copies, body and shim copies, multiple reference witnesses with one consumer
count, changed-body neighbors and reference isolation from group debt. Fixtures also cover a constant shared across spellings, the threshold itself, a constant held only by two reference packages, pinned and unpinned external references that report the same rows as a lock-selected one, and a member listed on the wrong side of the boundary. Native
round trips and queries preserve full receipts and witnesses. Closed readers
reject package/receipt disagreement, invalid source-relative manifests and
witnesses unrelated to the original selection. Source fixtures compare invalid
manifest identities and generic Git Trees with valid registry and exact committed
neighbors, and confirm that provider bindings do not escape observations.
Declared-tree fixtures build Git repositories with a later head and an edited
checkout, compare a pinned commit with the same package selected through the
lock graph, read the same pinned tree from a linked worktree and the main
checkout, compare an unpinned directory with its pinned commit, and pair each
refusal with a valid commit or pin key.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict` | test | [Contract](#contract), [Usage](#usage) |
| `cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets` | test | [Usage](#usage), [Examples](#examples) |
| `cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory` | test | [Contract](#contract), [Usage](#usage) |
| `raf_census::example_crate::references` | example target | [Usage](#usage) |
| `raf_census::test_crate::reference_scan::a_constant_held_only_by_two_reference_packages_has_no_row` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::an_index_without_a_reference_has_no_reference_constant_rows` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::exact_and_family_selectors_scan_a_package_once` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator` | test | [Contract](#contract), [Usage](#usage), [Constraints](#constraints) |
| `raf_census::test_crate::reference_scan::external_references_report_cross_boundary_constants_like_a_lock_selected_one` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::family_selector_excludes_member_rows_not_every_row_with_a_member_name` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::family_selector_follows_lock_growth_without_a_ledger_edit` | test | [Contract](#contract), [Usage](#usage) |
| `raf_census::test_crate::reference_scan::family_selector_matching_nothing_refuses_naming_the_selector` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::family_selector_refuses_a_name_with_two_reachable_versions` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::family_selector_scans_every_matching_dependency_but_no_member` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::reference_scan::first_party_shared_constants_stay_in_the_baseline_beside_the_cross_boundary_rows` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::large_constants_shared_with_a_reference_cross_the_boundary_by_value` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::linked_worktree_and_main_checkout_compare_against_the_same_pinned_tree` | test | [Contract](#contract), [Usage](#usage) |
| `raf_census::test_crate::reference_scan::locked_git_reference_copy_is_a_hard_finding` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count` | test | [Purpose](#purpose), [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints), [Explanation](#explanation) |
| `raf_census::test_crate::reference_scan::pinned_commit_reports_reference_copies_exactly_as_a_lock_selected_reference` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::pinned_reference_refusals_name_the_failed_input_beside_the_exact_commit` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts` | test | [Contract](#contract), [Examples](#examples) |
| `raf_census::test_crate::reference_scan::unhomed_reference_bans_shim_and_member_copies_beside_changed_neighbours` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::unhomed_reference_copy_is_a_hard_finding` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::unpinned_directory_finds_the_same_copies_and_labels_every_receipt` | test | [Contract](#contract) |
| `raf_source::test_crate::git_tree::pinned_tree_never_reads_or_size_checks_a_file_the_caller_already_holds` | test | [Contract](#contract) |
| `raf_source::test_crate::locked_sources::local_reference_owner_selection_spends_its_original_work_allowance` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes` | test | [Contract](#contract), [Constraints](#constraints), [Explanation](#explanation) |
| `raf_source::test_crate::locked_sources::registry_reference_observations_match_selected_manifest_identity` | test | [Contract](#contract) |
<!-- raf:generated:end -->
