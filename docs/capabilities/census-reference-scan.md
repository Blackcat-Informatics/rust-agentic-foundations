---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-reference-scan
title: "Census reference scan"
description: "Compare first-party code with exact dependency identities selected through lock files and manifest declarations."
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
  tests:
    - raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count
    - raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts
    - raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator
    - cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity
    - raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes
    - raf_source::test_crate::locked_sources::registry_reference_observations_match_selected_manifest_identity
    - raf_source::test_crate::locked_sources::local_reference_owner_selection_spends_its_original_work_allowance
  examples:
    - raf_census::example_crate::references
depends: [cap:census-ledger]
---

# Census reference scan

## Purpose
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count -->

A project can duplicate an implementation that one of its dependencies already
provides. Reference scanning compares first-party implementations with named
dependency packages and permits a job to name its home in a dependency.

## Contract

### Declaration selection
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes,raf_source::test_crate::locked_sources::registry_reference_observations_match_selected_manifest_identity -->

The native RDF ledger's `reference` values name dependency packages. The owning
package's actual manifest name and explicit or inherited version select its lock
row. Actual dependency declarations, including inherited declarations and renamed
packages, select exact outgoing rows. Missing, ambiguous, incompatible and
unavailable sources fail with context. Acquisition uses lock files and manifests;
it does not execute Cargo metadata, download packages or inspect shared caches.

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
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts -->

Reference units use the same normal form as first-party units. They remain
separate from first-party structural groups, ordinary rule execution and group
debt, including when the selected dependency is a workspace member. A job's home
or entry point can name a reference implementation or module. Full body and shim
fingerprints from reference homes classify first-party copies even when the
job's explicit forbidden selectors are empty. Each physical consumer counts once
per job; every matching reference witness is retained. A copy without a matching
reference home produces an unclassified finding.

### Closed native evidence
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

Report form 5 carries source receipts, qualified reference units and full
cross-boundary fingerprint witnesses. Every job occurrence resolves to an indexed
unit. Native Index queries retain the selected occurrences and incident witness,
endpoint and source closure. Reference units cannot receive a COPY role or enter
a first-party structural group. Migrating form 4 preserves its ordinary
first-party rows without inventing historical source observations. Normal form 1
fingerprints remain unchanged.

## Usage
<!-- raf:tested-by raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator,cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

Run `raf_census::example_crate::references` for an owned path dependency and its
complete native Index and query round trips.

Name a dependency and its job home in the native ledger:

```turtle
@prefix c: <https://blackcatinformatics.ca/raf/report/1#> .
<> a c:Ledger ; c:normalForm 1 ; c:reportForm 5 ;
   c:root "." ; c:reference "acme-core" ; c:job <#parse> .
<#parse> a c:LedgerJob ; c:jobId "iri-parse" ;
   c:summary "IRI parsing" ; c:homeSymbol "acme_core::iri::parse" ;
   c:enforced true .
```

`Gate::workspace(root).check()` acquires contained path dependencies.
`Gate::reference_roots` supplies locally available external roots and manifests,
including workspace context required for inherited package fields. A registry
locator also supplies the explicit registry-name/index mapping.
`Gate::check_tree_with_references` accepts opaque source observations with the
same identity checks; traversing their path declarations requires an owning
source locator. Registered commands transport the Gate's complete native Index.

## Examples
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts -->

A first-party function whose body equals two implementations in a reference
module contributes one physical COPY to that module's job and two full body
witnesses. A function with a different body remains a clean neighbor.

A reference package containing duplicate implementations contributes no
first-party structural group or group debt. Its HOME rows carry qualified source
locators and source receipts in the native Index.

## Constraints

- Only Rust source is compared, and named references must be reachable from the
  owning lock graph.
  <!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count -->
- Source reads retain the source loader's containment, regular-file, file-size
  and parse limits. Acquisition shares cumulative source, item, fact, resolver
  and work allowances across observations. Exhaustion fails rather than returning
  a partial index.
  <!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes,raf_source::test_crate::locked_sources::local_reference_owner_selection_spends_its_original_work_allowance -->
- Source roots retain workspace manifests needed for inherited package metadata.
  <!-- raf:tested-by raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->
- Parent traversal through a directory symlink requires an explicit manifest
  locator; matching manifest file identity does not admit a different directory.
  <!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

## Explanation
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

Lock identities and actual dependency declarations provide the selection context.
Source observations supply comparison evidence while preserving the stronger
sealed authority required for preferred-provider admission. Source-qualified
locators prevent independent workspaces' local package and item identifiers from
colliding in a report.

## Verification
<!-- raf:tested-by raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count,raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts,raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

Owned fixtures cover inherited renamed path dependencies, workspace-member
selection, body and shim copies, multiple reference witnesses with one consumer
count, changed-body neighbors and reference isolation from group debt. Native
round trips and queries preserve full receipts and witnesses. Closed readers
reject package/receipt disagreement, invalid source-relative manifests and
witnesses unrelated to the original selection. Source fixtures compare invalid
manifest identities and generic Git Trees with valid registry and exact committed
neighbors, and confirm that provider bindings do not escape observations.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity` | test | [Contract](#contract), [Usage](#usage) |
| `raf_census::example_crate::references` | example target | [Usage](#usage) |
| `raf_census::test_crate::reference_scan::explicit_path_roots_require_the_actual_owning_manifest_locator` | test | [Contract](#contract), [Usage](#usage), [Constraints](#constraints) |
| `raf_census::test_crate::reference_scan::path_references_preserve_all_witnesses_with_one_physical_consumer_count` | test | [Purpose](#purpose), [Contract](#contract), [Examples](#examples), [Constraints](#constraints), [Explanation](#explanation) |
| `raf_census::test_crate::reference_scan::reference_shim_matches_keep_every_witness_without_duplicate_copy_counts` | test | [Contract](#contract), [Examples](#examples) |
| `raf_source::test_crate::locked_sources::local_reference_owner_selection_spends_its_original_work_allowance` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes` | test | [Contract](#contract), [Constraints](#constraints), [Explanation](#explanation) |
| `raf_source::test_crate::locked_sources::registry_reference_observations_match_selected_manifest_identity` | test | [Contract](#contract) |
<!-- raf:generated:end -->
