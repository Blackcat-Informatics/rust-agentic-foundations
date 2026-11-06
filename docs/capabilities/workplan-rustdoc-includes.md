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
  examples:
    - raf_workplan::example_crate::cargo_documentation
depends: [cap:source-index, cap:workplan-generated-docs, cap:workplan-cargo-documentation]
---

## Purpose

Produce static presentation and public API references suitable for a caller-owned documentation include. Source and package identities remain explicit, while Markdown generation grants no authority to compile code or edit the receiving Rust module.

## Contract

**Static contract.** `rustdoc::fragment` returns Markdown for an explicit library target, public receiving module, capability and complete required binding selection. Each binding must resolve to one public, documented item in the selected source index. Hidden, private, ambiguous and unresolved references refuse. Imported package and manifest identities must match the selected source target.
<!-- raf:tested-by raf_workplan::test_crate::rustdoc_fragments::fragments_refuse_private_or_incomplete_bindings_beside_a_public_reference -->

## Usage

The documented data path is illustrated by `raf_workplan::example_crate::cargo_documentation`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

Supply a Workplan, source Workspace, diagram bundle and explicit `rustdoc::Context`. The result is a string. A caller may save documentation through its own explicit data-publication operation and author a literal `doc = include_str!` attribute. RAF never inserts that attribute or modifies Rust source.

## Examples

A public re-export retains its exported spelling. An ambiguous definition with several aliases requires the intended public spelling. The fragment labels the imported metadata digest separately from the exact loaded source identity.

## Constraints

- Fragments describe the supplied static source index. They do not establish compiler cfg, generated declarations, macro expansion, successful link resolution or the contents of a later HTML build. At most 4096 API references are admitted.
<!-- raf:tested-by raf_workplan::test_crate::rustdoc_fragments::fragments_refuse_private_or_incomplete_bindings_beside_a_public_reference -->

## Explanation

Ordinary disambiguated crate intra-doc destinations leave final filenames and anchors to Rustdoc. The shared static presentation remains script-free. Generating the string does not compile it, launch a driver or publish an assembly.

## Verification

Artifact identity admission is covered by the named fixtures. Public fragment resolution is checked independently against the native source and export index.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
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
