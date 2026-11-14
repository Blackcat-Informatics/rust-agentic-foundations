---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-rustdoc-includes
title: Static Rustdoc include fragments
description: Checked source references in returned Markdown without source mutation or compilation.
state: delivered
bindings:
  code:
    - raf_workplan::diagram::rustdoc::fragment
    - raf_workplan::diagram::rustdoc::Context
  tests:
    - raf_workplan::test_crate::rustdoc_fragments::fragments_refuse_private_or_incomplete_bindings_beside_a_public_reference
    - cargo_workplan::test_crate::rdf::static_fragments_preserve_aliases_and_all_bindings_without_compiler_authority
  examples:
    - raf_workplan::example_crate::cargo_documentation
depends: [cap:source-index, cap:workplan-generated-docs, cap:workplan-cargo-documentation]
---

## Purpose

Produce static presentation and public API references suitable for a caller-owned documentation include. Source and package identities remain explicit, while Markdown generation grants no authority to compile code or edit the receiving Rust module.

## Contract

The `docs fragment` command exposes the same returned Markdown for exactly one imported library, public receiving module, capability and shared named view. It derives every required code binding from that capability. Source/package identity mismatches and any invalid required reference refuse before fragment output. The complete Markdown goes to stdout and diagnostics remain on stderr; quiet and non-text output refuse before root loading.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::static_fragments_preserve_aliases_and_all_bindings_without_compiler_authority -->

**Static contract.** `rustdoc::fragment` returns Markdown for an explicit library target, public receiving module, capability and complete required binding selection. Each binding must resolve to one public, documented item in the selected source index. Hidden, private, ambiguous and unresolved references refuse. Imported package and manifest identities must match the selected source target.
<!-- raf:tested-by raf_workplan::test_crate::rustdoc_fragments::fragments_refuse_private_or_incomplete_bindings_beside_a_public_reference -->

## Usage

Invoke `cargo workplan docs fragment --package PACKAGE --metadata META --target-directory OUTPUT --view OWNER#NAME --module MODULE --capability CAPABILITY`. `META` is bounded imported format-v1 Cargo metadata, and `OUTPUT` is its explicitly authorized existing target directory. The module can name the crate root or a documented public nested module. The command uses the bounded default static Source load and publishes no file.

The documented data path is illustrated by `raf_workplan::example_crate::cargo_documentation`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

Supply a Workplan, source Workspace, diagram bundle and explicit `rustdoc::Context`. The result is a string. A caller may save documentation through its own explicit data-publication operation and author a literal `doc = include_str!` attribute. RAF never inserts that attribute or modifies Rust source.

## Examples

A fragment for `demo::nested` can refer to `demo::exported`, a public alias of an internal definition. A second required code binding remains mandatory even when the first resolves. A private or missing second reference refuses instead of producing a partial include.

A public re-export retains its exported spelling. An ambiguous definition with several aliases requires the intended public spelling. The fragment labels the imported metadata digest separately from the exact loaded source identity.

## Constraints

CLI metadata and static source selection do not establish effective compiler configuration, successful Rustdoc link resolution, archive membership or host assembly. Compiler executables are unnecessary for this data operation. A caller-owned include file and literal source attribute remain separate from the returned Markdown.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::static_fragments_preserve_aliases_and_all_bindings_without_compiler_authority -->

- Fragments describe the supplied static source index. They do not establish compiler cfg, generated declarations, macro expansion, successful link resolution or the contents of a later HTML build. At most 4096 API references are admitted.
<!-- raf:tested-by raf_workplan::test_crate::rustdoc_fragments::fragments_refuse_private_or_incomplete_bindings_beside_a_public_reference -->

## Explanation

The CLI delegates to `rustdoc::fragment` without a second binding resolver or presentation model. Imported library selection and the source target must agree, and the fragment retains the native metadata digest alongside its separately labelled loaded-source identities.

Ordinary disambiguated crate intra-doc destinations leave final filenames and anchors to Rustdoc. The shared static presentation remains script-free. Generating the string does not compile it, launch a driver or publish an assembly.

## Verification

The real CLI fixture checks the same static contract through direct and Cargo-prefixed argument forms, including adjacent valid requests after refusals. It preserves all input bytes and succeeds with unavailable compiler tools; it does not compile the returned fragment or claim package membership.

Artifact identity admission is covered by the named fixtures. Public fragment resolution is checked independently against the native source and export index.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::rdf::static_fragments_preserve_aliases_and_all_bindings_without_compiler_authority` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::example_crate::cargo_documentation` | example target | [Usage](#usage) |
| `raf_workplan::test_crate::rustdoc_fragments::fragments_refuse_private_or_incomplete_bindings_beside_a_public_reference` | test | [Contract](#contract), [Constraints](#constraints) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [source-index](./source-index.md) |
| Depends on | [workplan-cargo-documentation](./workplan-cargo-documentation.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depended on by | [workplan-rustdoc](./workplan-rustdoc.md) |
<!-- raf:generated:end -->
