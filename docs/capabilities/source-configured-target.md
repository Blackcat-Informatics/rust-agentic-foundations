---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:source-configured-target
title: Exact configured source target
description: Index one selected package target under declared configuration without admitting other targets or provider authority.
state: delivered
bindings:
  code:
    - raf_source::Workspace::load_configured_target
  tests:
    - raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths
  examples:
    - raf_source::Workspace::load_configured_target
depends: [cap:source-index]
---

## Purpose

Read one package target when no configuration is available for its workspace
siblings. Documentation compiler observations can supply this declaration
without inventing contexts for unrelated packages.

## Contract

**Exact package.** `Workspace::load_configured_target` takes a contained Tree, an exact Cargo.toml
manifest, one DeclaredConfiguration and the original LoadOptions. It discovers
that package only. Workspace ancestors still supply inherited manifest values.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths -->

**Selected target.**
The kind and name must identify exactly one derived target; required features
must be enabled. Only that target's module tree is parsed and indexed.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths -->

**Shared selection.** Declaration selection, indexing, public exports and configured syntax use the
existing source machinery and one resource allowance. Public re-export spellings
retain their actual definitions. Hidden and private items retain the existing
export policy. An absent target or invalid configuration refuses.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths -->

**Authority boundary.** The returned workspace retains no locked binding and admits no dependency edge.
Its declared selection does not authorize preferred-provider resolution.
Dependency declarations remain syntax; unobserved dependencies cannot supply
definitions.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths -->

## Usage

Supply the actual package manifest and one complete declared target context.
Keep the Tree root broad enough for workspace-inherited fields. Read the returned
target's public surface with exported_items and its selected declaration syntax
with CodeResolver::with_configured_syntax. The runnable Rustdoc example on
`raf_source::Workspace::load_configured_target` demonstrates a declared `doc`
predicate selecting a public item, without querying a compiler.
<!-- raf:tested-by raf_source::Workspace::load_configured_target -->

## Examples

A workspace root package selects its named library with `doc` and an enabled
feature. A public alias resolves to the active definition while a sibling package
and a separate binary with invalid Rust remain unparsed. Changing the declared
feature set selects the neighboring alias definition. A missing target refuses.
An explicitly selected binary requiring that feature refuses while it is inactive
and loads when it is enabled.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths -->

The method's runnable Rustdoc example declares `doc` for a small memory-backed
library and reads its public `Documented` item through exported_items.
<!-- raf:tested-by raf_source::Workspace::load_configured_target -->

## Constraints

- Configuration remains a caller declaration. This loader observes no compiler,
  expands no macro and establishes no generated definition.
<!-- raf:tested-by raf_source::Workspace::load_configured_target -->
- Selection covers the same modeled declaration headers as the configured
  workspace loader; conditional body expressions, fields and variants are not
  transformed.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths -->
- No unobserved sibling or dependency configuration is synthesized. Required
  target features must be present in the caller's declaration.
<!-- raf:tested-by raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths -->

## Explanation

Exact package discovery shares the ordinary manifest reader and ancestor
inheritance. Target selection shares configuration admission, module walking and
index construction. The selection is stored separately from locked bindings.
The compiler driver can therefore admit its observed target before generating
documentation includes, without upgrading declared context to source provenance.
Compiler input observations, required links and documentation publication remain
distinct contracts of [checked Rustdoc assembly](workplan-rustdoc.md).

## Verification

The bound configured-source case covers active public aliases, hidden/private
export exclusion, inherited edition, cfg-selected declarations, unparsed sibling
and binary sources, absent target/manifest refusals, unresolved dependency access
and absence of provider-admitted bindings. It also exercises configured syntax
through the exact target's retained declaration and pairs required-feature refusal
with enabled-feature admission. The method's Rustdoc example is independently
runnable through the ordinary doctest path.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_source::Workspace::load_configured_target` | doctest | [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_source::test_crate::configured_sources::configured_aliases_resolve_only_active_real_definitions_and_paths` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
<!-- raf:generated:end -->
