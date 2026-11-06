---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-rustdoc-observation
title: Imported documentation artifacts
description: Contained HTML identities from explicitly supplied metadata and message carriers.
state: delivered
bindings:
  code:
    - raf_workplan::diagram::cargo::read_artifacts
    - raf_workplan::diagram::cargo::DocumentationArtifacts
    - raf_workplan::diagram::cargo::LibraryArtifact
  tests:
    - raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files
    - raf_workplan::diagram::cargo::artifact::tests::imported_metadata_and_messages_are_bounded_data_with_explicit_roots
  examples:
    - raf_workplan::example_crate::cargo_documentation
depends: [cap:workplan-cargo-documentation]
---

## Purpose

Inspect existing documentation output without granting build authority. Explicitly supplied metadata and message carriers identify contained files as imported data, keeping those assertions separate from an actual compiler execution observed by a caller.

## Contract

**Static contract.** `read_artifacts` consumes bounded supplied Cargo message data and admits one contained regular HTML entry per selected library. Opaque package ID, manifest, target name, target kinds, crate types and source locator must agree with the imported metadata. Repeated entries, missing selected entries and incompatible features refuse. Freshness is retained as supplied Boolean data.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files -->

Each admitted entry retains its relative path, physical file identity and native content SHA-256. Imported success messages never establish that RAF observed a compiler run. Reading never builds, launches a helper or publishes files.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::imported_metadata_and_messages_are_bounded_data_with_explicit_roots -->

## Usage

The documented data path is illustrated by `raf_workplan::example_crate::cargo_documentation`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

Call `cargo::read_metadata` with the explicit root authority, then `cargo::read_artifacts` with message bytes. The CLI `docs artifacts` exposes the same readers and labels its results as imported data.

## Examples

Cached and newly built entries both carry imported freshness. A matching neighboring entry succeeds after a mismatched target, duplicate receipt or outside path refuses.

## Constraints

- Each HTML file admits at most 16 MiB. Different selected libraries cannot share one physical file. Streams must include a complete successful build-finished record as a data-shape requirement, not an authenticated execution verdict.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files -->

## Explanation

Source declarations, message assertions and files read from disk are distinct facts. No compiler environment receipt, pre-CLI dispatch, namespace supervisor or delegated compiler is part of artifact inspection.

## Verification

Existing admission fixtures cover changed identities, malformed freshness, hard-linked aliases, outside files, explicit carrier size limits and successful neighboring artifacts without starting a compiler.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_workplan::diagram::cargo::artifact::tests::imported_metadata_and_messages_are_bounded_data_with_explicit_roots` | test | [Contract](#contract) |
| `raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::example_crate::cargo_documentation` | example target | [Usage](#usage) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-cargo-documentation](./workplan-cargo-documentation.md) |
<!-- raf:generated:end -->
