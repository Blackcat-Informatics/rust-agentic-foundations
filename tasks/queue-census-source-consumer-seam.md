<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Census production seam

Prepare one Source Workspace and use it for unit collection, ledger resolution
and configured declaration/entry-point identity queries. Current Census collect_units already iterates
Workspace::items_by_source; supplying that same configured index preserves its
selected declaration units. Do not reload an unfiltered workspace or mix IDs.
Raw bodies remain untransformed, and this does not change NF 1 fingerprints.

1. A caller-owned MemoryTree fixture contains Cargo.lock plus actual versioned
   consumer/provider manifests and source, with a declared path dependency such
   as renamed={package='provider',path='../provider'}. Disk uses DiskTree::new.
2. LockedCatalog::load(tree, Cargo.lock RelPath, options) supplies actual rows.
   LockedManifest::new(row, manifest RelPath, requested_edges) supplies locators;
   LockedEdge::new("dependencies.renamed", None) requests the actual declaration.
   The loader checks the actual destination directory and manifest identity.
3. Every admitted manifest gets with_configuration(DeclaredConfiguration::new(
   DeclaredCfg::complete(triple, &[CfgAtom]), complete_features, TargetKind::Lib,
   actual_target_name)). BuildScript instead requires independent with_host cfg.
   Complete features are already unified caller input, validated against actual
   local implications and requested destinations. No Cargo-state inference.
4. Git rows use CommittedGitTree::open(catalog, row, existing_repository, options)
   and GitPackageRoots::new(snapshot, configured_manifests). Only committed bytes
   of the exact full locked revision qualify; labels cannot substitute for this
   sealed host Tree. Git/local object storage is trusted; no checkout mutation.
5. Workspace::load_locked_sources(catalog, first_tree, local_manifests,
   git_package_roots, options) returns the owned common index. Keep one original
   LoadOptions; public accounting/error types are Usage, Budget and LoadError.
   Workspace::usage reports the combined load. No private Meter is required.
6. Resolve with resolve_active_from_package_id_in(actual_reader_PackageId,
   module, path, Namespace). For every Resolution.sites ItemId follow Item.target
   to Target.package_id, then Workspace::locked_binding. Check row/name/version/
   source and configuration(), not just Resolution.path or a crate-name prefix.
   PackageId, TargetId, ItemId and LockedPackageId are load-local handles.

Actual pinned route input is tasks/queue-configured-purrdf-route.rs: local
raf-source requests dependencies.purrdf-lex; Git crates/lex/Cargo.toml requests
dependencies.purrdf-hash; Git crates/hash/Cargo.toml has no requested edge.
Actual library target names are raf-source, purrdf_lex and purrdf_hash. The source
pin is 5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43; explicit captured target cfg and
empty complete features select Memory and Shared definitions. Receipt reused.

Limits: one actual Lib/BuildScript target per package; all admitted packages
configured or all structural; distinct configured packages sharing one physical
file refuse. Configuration selects modeled declaration headers, not bodies,
fields or enum variants. Registry source snapshots, std oracle, macro/type
expansion and Census provider-policy execution are not supplied by this unit.
Foreign member trait_definition_target is evidence only when Some and associated
with an actual defining Trait; unresolved textual keys are not provider proof.
The public active resolver wraps resolve_from_package_id_in and uses import
lookup semantics. It is not a code-occurrence/call oracle. Heads::Code lookup
and lexical local-shadow evidence require a Source-owned production seam before
consumer-call proof; this is not part of the currently qualified extraction.
