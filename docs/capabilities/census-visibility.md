---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-visibility
title: Workspace library visibility census
description: Inventory public declarations, root-reachable definitions and written cross-package uses.
state: delivered
bindings:
  code:
    - raf_census::visibility
    - raf_census::VisibilityError
    - raf_source::Workspace::load_members
    - raf_source::Workspace::library_prelude_dependencies
    - raf_source::CodeResolver::indexed_imports
    - raf_source::visit_grouped_import
    - cargo_census::command::visibility
  tests:
    - cargo_census::test_crate::delivery::visibility_counts_definitions_and_cross_member_paths_in_dependency_tiers
    - raf_census::test_crate::census_gate::mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches
  examples:
    - raf_census::visibility
depends: [cap:source-index, cap:census-patterns]
---

## Purpose

Use static source evidence to identify library declarations that are candidates
for narrower visibility. Review those candidates in dependency order from the
lowest tier upward.

## Contract

**Declaration and use evidence.** Report each workspace library's indexed
public declarations by kind, its distinct root-reachable definitions, the
written paths that other workspace packages resolve to its items, and its
public declarations with no such use.
Attribute a path through a re-export to the defining library, including uses
from test targets. Count each written path once per definition and consuming
package. Count the union of path spellings once per library.
The written paths omit generic arguments and raw-identifier prefixes.
<!-- raf:tested-by cargo_census::test_crate::delivery::visibility_counts_definitions_and_cross_member_paths_in_dependency_tiers -->

**Dependency tiers.** Tier zero contains libraries with no normal dependency
on a workspace library. Each other library has tier one above the highest tier
of its normal workspace library dependencies. A cycle in those edges refuses
the report.
<!-- raf:tested-by cargo_census::test_crate::delivery::visibility_counts_definitions_and_cross_member_paths_in_dependency_tiers -->

**Report fields.** The canonical ground RDF root has class
`WorkspaceVisibility`, integer `reportForm`, and one `library` link per library.
Libraries are numbered by tier and then by load-local package order.

| Node | Properties |
| --- | --- |
| `LibraryVisibility` | `package`, `tier`, `publicCount`, `exportedCount`, `usedExportedCount`, `writtenPathCount`, and links `declaration`, `unusedDeclaration`, `kindCount`, `use` |
| `PublicDeclaration` | `symbol`, `file`, `line`, `startByte`, `kind` |
| `VisibilityKindCount` | `kind`, `count` |
| `LibraryUse` | `symbol`, `file`, `line`, `startByte`, `consumer`, and one or more `writtenPath` strings |

Use `usedExportedCount` and `exportedCount` as the numerator and denominator
of export coverage. Count an exported definition once even when multiple root
paths reach it. The export count includes doc-hidden definitions.
<!-- raf:tested-by cargo_census::test_crate::delivery::visibility_counts_definitions_and_cross_member_paths_in_dependency_tiers -->

## Usage

Run `cargo census visibility` at a Cargo workspace or select it with
`--workspace DIRECTORY`. The command writes canonical N-Triples to stdout by
default. Use `--format turtle` for canonical Turtle and `--out PATH` for a report
destination inside the workspace. Regenerate and compare the bytes to check
the currency of a saved report.

Call `raf_census::visibility` with a Source tree and its workspace root to
obtain the same graph for direct inspection or rendering.

## Examples

If `bridge` re-exports `base::run` as `bridge::start` and `client` calls
`bridge::start()`, the report records `bridge::start` under `base` with
`client` as the consumer. A call inside `base` leaves the declaration
unused by other packages.

## Constraints

- The report uses the declared Cargo members and the structural Source index.
  Normal library edges include structural target alternatives. Dev and build
  edges do not set library tiers. The public declaration inventory uses written
  `pub` declarations in member library targets outside test code.
  <!-- raf:tested-by cargo_census::test_crate::delivery::visibility_counts_definitions_and_cross_member_paths_in_dependency_tiers -->
- Written path evidence covers indexed imports, lexical aliases, calls,
  constructors, patterns, types and macros named in source. It covers type or
  module prefixes that resolve in their written namespace, including prefixes
  of qualified macro calls. A block glob can resolve through another block glob
  visible from its declaration. If the glob prefix could name separate
  definitions through a glob and an outer scope, qualify the import. An identity
  that remains ambiguous across packages refuses the report when it can affect
  a public declaration. A block glob outside the index can likewise prevent
  attribution of an unqualified path. Macro expansion, implicit method
  selection, unresolved `Self` paths and associated selections through
  unsupported type-alias projections require evidence outside this report.
  An unused declaration has no resolved written use in the selected source
  index. The report builds and executes no workspace targets.
  <!-- raf:tested-by cargo_census::test_crate::delivery::visibility_counts_definitions_and_cross_member_paths_in_dependency_tiers,raf_census::test_crate::census_gate::mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches -->

## Explanation

A declaration can be public in a private module and reach consumers only
through a re-export. Declaration identity connects that use to the library
that owns the definition. Source spans distinguish declarations that share a
path, and synthetic crate roots are outside the declaration inventory.

## Verification

The CLI fixture checks the three library tiers, re-export attribution, the
import in an out-of-line module, the test-only dependency, trait and block
scope resolution, a constant pattern reached through a block glob, chained
and direct block glob prefixes, and a module named only through a qualified
macro call. It checks the exact union of written path spellings, compares
report bytes across entry directories and checks that a normal dependency
cycle and a potentially shadowing unindexed block glob refuse before output.
An excluded package and an inert build script exercise the static input
boundary.

A shared source file exercises a pattern identifier with conflicting meanings
across module reaches and a neighbour with consistent meanings.
<!-- raf:tested-by raf_census::test_crate::census_gate::mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches -->

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::delivery::visibility_counts_definitions_and_cross_member_paths_in_dependency_tiers` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches` | test | [Constraints](#constraints) |
| `raf_census::visibility` | doctest | [Usage](#usage) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-patterns](./census-patterns.md) |
| Depends on | [source-index](./source-index.md) |
<!-- raf:generated:end -->
