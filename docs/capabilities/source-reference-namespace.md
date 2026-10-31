---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:source-reference-namespace
title: Structural reference namespace
description: One source index across exact first-party and reference contexts without configured provider authority.
state: delivered
bindings:
  code:
    - raf_source::Workspace::load_reference_sources_under
    - raf_source::Workspace::reference_binding
    - raf_source::ReferenceTree
    - raf_source::ReferencePackageRoots
    - raf_source::ReferenceBinding
  tests:
    - raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes
  examples:
    - raf_source::example_crate::reference_namespace
depends: [cap:source-index]
---

# Structural reference namespace

## Purpose

Consumers need to resolve aliases and public reexports into selected dependency
sources while preserving the meaning of each source root. Structural reference
navigation joins those sources in one identity domain and keeps its metadata
separate from configured provider evidence.

## Contract

### One structural identity domain

Structural reference loading uses `Workspace::load_reference_sources_under`
with actual `ReferenceTree` contexts and exact `LockedManifest` associations.
One index settles imports and reexports across those sources. A reference
binding records the selected row, actual name and version and declared source;
every configured `locked_binding` remains absent. No active target context or
provider admission is inferred from structural navigation.
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

### Verified source relationships

Cross-root path declarations require actual disk-directory relationships or
one exact committed Git context. Equal manifest-file identities cannot equate
different package directories. Opaque trees cannot assert a cross-authority
host relationship. Supplied configurations refuse in this structural mode.
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

## Usage

Borrow contained disk roots, opaque trees or sealed committed trees through
`ReferenceTree`. Group exact manifest associations in `ReferencePackageRoots`
and provide their retained declaration keys. Inspect `reference_binding` for
navigation metadata; configured provider consumers still need the separate
locked/configured entry point and its real context evidence.
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

Call `Workspace::load_reference_sources_under` with the catalog, first-party
context and roots, local associations, reference groups and load limits. Use
the returned workspace's ordinary resolution methods to inspect the selected
definitions. The runnable `reference_namespace` example supplies these inputs
from contained disk roots in an owned native temporary directory.
<!-- raf:tested-by raf_source::example_crate::reference_namespace -->

## Examples

A dependency renamed to `renamed` can reexport a function from a library whose
actual crate name is `native_helper`. The consumer's imported alias resolves
to that function's item identity. A same-named local module resolves to its
own definition. A different supplied directory with a hardlinked copy of the
expected manifest refuses as the path destination.
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

Run `cargo run -p raf-source --example reference_namespace`. The example resolves
`app::selected` through the renamed dependency and its public export to
`native_helper::implementation::provided`, then checks the actual item's
identity and the reference package's name, version and catalog row. Every
package lacks a configured locked binding. It starts no Cargo subprocess and
writes only inside its owned temporary directory.
<!-- raf:tested-by raf_source::example_crate::reference_namespace -->

## Constraints

- Structural optional and conditional declarations are observed syntax, not
  feature activation. Unrequested local paths retain ordinary structural lookup;
  unselected external dependencies remain unresolved.
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->
- All source reads, retained descriptors and index construction share the supplied
  load limits. No source is fetched and no registry cache is enumerated.
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

## Explanation

Keeping the actual backends through a unified load preserves path-directory
meaning and lets the existing index settle every admitted import together.
Merging independent settled indexes would preserve unresolved lookup caches;
joining observation rows alone would lose their actual directory context.
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

## Verification

The existing reference-observation case exercises renamed library exports,
local shadows and separate reference/configured identities. It pairs admitted
disk paths with opaque cross-authority, wrong-directory and configured-input
refusals. A hardlinked manifest does not substitute for its package directory.
<!-- raf:tested-by raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes -->

The public example resolves the renamed export to its actual item and consumes
reference metadata while requiring configured locked bindings to remain absent.
<!-- raf:tested-by raf_source::example_crate::reference_namespace -->

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_source::example_crate::reference_namespace` | example target | [Usage](#usage), [Examples](#examples) |
| `raf_source::test_crate::locked_sources::reference_observations_verify_manifests_without_admitting_provider_routes` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints), [Explanation](#explanation) |
<!-- raf:generated:end -->
