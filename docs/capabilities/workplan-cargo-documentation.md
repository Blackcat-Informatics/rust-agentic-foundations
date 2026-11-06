---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-cargo-documentation
title: Explicit Cargo documentation data
description: Bounded imported metadata and invocation-scoped trusted library documentation.
state: delivered
bindings:
  code:
    - raf_workplan::diagram::cargo
    - raf_workplan::diagram::cargo::CargoDocOptions
    - cargo_workplan::bin_crate::cargo_workplan::trusted_docs::build
  tests:
    - cargo_workplan::test_crate::trusted_docs::documentation_trust_is_explicit_and_configuration_cannot_select_a_test_driver
    - cargo_workplan::bin_crate::cargo_workplan::trusted_docs::configuration::tests::compiler_flags_wrappers_and_includes_cannot_change_the_closed_doc_operation
    - raf_workplan::diagram::cargo::artifact::tests::imported_metadata_and_messages_are_bounded_data_with_explicit_roots
    - cargo_workplan::test_crate::verify::trusted_docs_is_refused_before_root_discovery_and_general_execution_is_absent
    - raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files
  examples:
    - raf_workplan::example_crate::cargo_documentation
depends: [cap:source-index, cap:workplan-cli]
---

## Purpose

Keep useful package selection and documentation artifacts while requiring explicit authority for compilation. Static callers can inspect existing carriers and contained HTML without trusting repository build scripts or procedural macros.

## Contract

**Static contract.** `cargo::read_metadata` imports bounded format-v1 metadata for explicit workspace packages and qualified features. Its workspace root and output directory must match the caller's explicit canonical roots. Imported metadata records declarations; it does not establish compiler execution.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::imported_metadata_and_messages_are_bounded_data_with_explicit_roots -->

Only the private CLI operation `docs build rustdoc` launches tools. Each invocation requires `--trust-docs`, explicit toolchain and output directories and package selection. Missing trust refuses before root discovery, process launch or output creation. Compilation uses fixed Cargo, Rust compiler and Rustdoc binaries with frozen, offline, library-only documentation arguments. Tests, examples and doctests are never selected.
<!-- raf:tested-by cargo_workplan::test_crate::verify::trusted_docs_is_refused_before_root_discovery_and_general_execution_is_absent -->

## Usage

The documented data path is illustrated by `raf_workplan::example_crate::cargo_documentation`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

Use `cargo workplan docs artifacts --metadata META --messages MESSAGES --target-directory OUTPUT --package PACKAGE` to inspect existing data. No build or publication occurs.

Use `cargo workplan docs build rustdoc --trust-docs --toolchain-directory TOOLS --target-directory OUTPUT --package PACKAGE` only when the repository and dependencies are trusted. Tool and output paths must be absolute; output must not overlap source, toolchain or Git administration.

## Examples

The data-only `cargo_documentation` example accepts ROOT, ARTIFACT_ROOT, PACKAGE, METADATA_JSON and MESSAGES_JSON. It reads existing files and prints imported artifact identities. It never builds those inputs.

A request without `--trust-docs` refuses even when the named workspace does not exist. The explicit refusal precedes discovery.

## Constraints

- Trust accepts arbitrary build-script and procedural-macro effects. The operation is not a sandbox, has no descendant teardown guarantee and does not guarantee build-time code leaves source or external systems unchanged. Compiler overrides, wrappers, flag injection and configuration includes refuse. Stream and carrier bounds are 16 MiB; version query streams admit 64 KiB.
<!-- raf:tested-by cargo_workplan::test_crate::trusted_docs::documentation_trust_is_explicit_and_configuration_cannot_select_a_test_driver -->

## Explanation

Package membership and imported feature declarations differ from actual compiler configuration. RAF does not infer generated declarations, execute a driver before CLI parsing, or retain complete compiler environments. Metadata is checked before and after a trusted build; equality does not imply atomic compiler inputs.

## Verification

The artifact reader fixtures cover explicit root mismatch, oversized carriers and malformed messages beside valid imports. The CLI refusal fixture covers absent trust before discovery and the absence of arbitrary-command evidence recording.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::bin_crate::cargo_workplan::trusted_docs::configuration::tests::compiler_flags_wrappers_and_includes_cannot_change_the_closed_doc_operation` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::trusted_docs::documentation_trust_is_explicit_and_configuration_cannot_select_a_test_driver` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::verify::trusted_docs_is_refused_before_root_discovery_and_general_execution_is_absent` | test | [Contract](#contract) |
| `raf_workplan::diagram::cargo::artifact::tests::imported_metadata_and_messages_are_bounded_data_with_explicit_roots` | test | [Contract](#contract) |
| `raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files` | test | [Verification](#verification) |
| `raf_workplan::example_crate::cargo_documentation` | example target | [Usage](#usage) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [source-index](./source-index.md) |
| Depends on | [workplan-cli](./workplan-cli.md) |
| Depended on by | [workplan-rustdoc](./workplan-rustdoc.md) |
| Depended on by | [workplan-rustdoc-includes](./workplan-rustdoc-includes.md) |
| Depended on by | [workplan-rustdoc-observation](./workplan-rustdoc-observation.md) |
<!-- raf:generated:end -->
