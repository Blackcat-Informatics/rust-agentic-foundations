<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Source takeover: documentation and caller checkpoint

Subsequent implemented delta: `queue-relative-package-resolution.md` closes the
package-ID-qualified reading-module API gap identified below, with actual caller,
lint and affected-target receipts. Other provider/source and parent acceptance
limits remain. The following is the earlier inspected checkpoint.

Only product edit in this task: four doc-markdown fixes in
`crates/raf-source/src/locked/mod.rs`, enclosing SemVer/PurRDF names in backticks.
No waiver, behavior change, staging, commit, push or branch operation.

Before editing, inspected tracked/staged/untracked status and current locked
source. No staged changes existed. Preserved all 24 tracked modified files,
untracked `locked/{mod,read,source}.rs`, `tree/sources.rs` and `locked_catalog.rs`;
no stash was dropped or reapplied. Applicable AGENTS, current queue skill,
repository memory instructions, original issue capture, selected foundation
plan and validation index were read. `.baseline` and `.goals` are absent here.
The explicit repository bank recall reinforced the existing exact-source/parser
hazards. Graphify was attempted first for interface navigation, but its graph
file is absent; current source supplied the interface evidence.

## Actual focused command receipts

Both commands used the ordinary Stage cargo wrapper and this checkout's existing
separate managed target, `/opt/.cargo/target/48-raf-source-evaluate-cfg-against-a-d0922b41`.
No cache override, shared-target mutation, clean or cold build was requested.

- `cargo clippy -p raf-source --all-targets --locked --offline -- -D warnings`:
  exit 0, session 29656. Exact log/status:
  `raw/queue-takeover-locked-clippy.{log,status}`.
- `cargo check --workspace --all-targets --locked --offline`: exit 0, session
  5278. Compiled real Source, Workplan, Census, CLI and root callers. Exact
  log/status: `raw/queue-takeover-source-callers.{log,status}`.

These are compile/lint receipts, not runtime tests or full qualification. No
optional tests, broad suites or benchmarks ran.

## Interface Census must consume

- `Workspace::load_from` (`workspace.rs:173`) takes typed RelPath roots through
  the existing loader; `SourceTrees` dispatches independently rooted borrowed
  Trees. RelPath authority survives joins and returned source locations. This
  layer explicitly supplies physical separation, not package/revision proof.
- `Workspace::resolve_in_package_id` (`workspace.rs:468`) selects exact
  load-local PackageId roots for the ordinary resolver. Name-based
  `resolve_in_package` rejects ambiguous package names. IDs are local to their
  load and are not persistent source identities.
- `Resolution` (`resolve.rs:174`) retains canonical path, namespace, nonempty
  `sites`, glob status and import chain. Follow each ItemId through
  `Workspace::item`, `Item.target`, `Workspace::target(...).package_id` and
  `Workspace::package`. Retain the typed Item.file/source authority and
  `Item::source_key`/`items_by_source` physical identity. Displayed path or crate
  name equality is insufficient for ownership or preferred-provider proof.
- `resolve_from`/`resolve_from_in` share normal visibility-aware import traversal,
  but their reading module remains text-selected. The current public surface
  has no package-ID-qualified reading-module counterpart; duplicate crate-root
  names therefore remain ambiguous in these relative queries.
- `LockedCatalog::load` separately reads complete locked rows/outgoing IDs,
  names, versions, source kind/repository/reference/full revision and checksum.
  LockedPackageId and PackageId are different local domains. Catalog parsing
  does not authenticate supplied source contents or establish active features.

The missing production join is concrete: Workspace/discovery/index do not consume
LockedCatalog. No verified mapping currently binds a lock row to its supplied
authority/manifest and actual loaded PackageId. Manifest Dependency still retains
only alias/package/path/kind, not complete requirements/source/optional/features/
target conditions. Census preferred-provider enforcement must require that
grounded mapping and complete eligible declaration-to-outgoing-lock-edge
selection before accepting a resolved definition. A fake native package name,
source authority tag or parsed lock revision alone cannot replace it.

## Acceptance still open

The selected foundation still needs exact locked declaration/source binding,
ordinary checkout revision/relevant-state verification, configuration-completeness
refusals, a real pinned PurRDF definition route and migrated production consumer,
runtime semantic/admission controls and required delivery gates/lifecycle.
The existing real-lock test checks parsed package edges and source precision;
it is not a verified source-definition route, and was not executed here.

All original issue 48 requirements remain: declared cfg input and recorded result,
different cfg outcomes, path-dependency and external/std namespace resolution,
visible Unexpanded macro markers, safe per-item unsupported markers versus hard
resolution errors, tracked divergence ownership including external reexports and
CarryBudget, and expanded rustc differential acceptance. Full portability remains
with its existing owners. This narrow checkpoint completes none of those missing
contracts and does not authorize publication or closure.
