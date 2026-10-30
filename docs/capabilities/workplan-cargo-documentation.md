---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-cargo-documentation
title: Cargo documentation artifact capture
description: Typed selected Cargo documentation with bounded metadata, complete output and contained HTML observations.
state: delivered
bindings:
  code:
    - raf_workplan::exec::CargoDocOptions
    - raf_workplan::exec::CargoDocOptionsError
    - raf_workplan::exec::Invocation::cargo_doc_metadata
    - raf_workplan::exec::Invocation::cargo_doc
    - raf_workplan::diagram::cargo
  tests:
    - raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files
    - raf_workplan::diagram::cargo::artifact::tests::documentation_refuses_dependency_metadata_drift_with_unchanged_libraries
    - raf_workplan::diagram::cargo::artifact::tests::typed_requests_and_bounded_protocol_refusals_keep_a_valid_neighbor
  examples:
    - raf_workplan::example_crate::cargo_documentation
depends: [cap:workplan-complete-gate, cap:source-index]
---

## Purpose

Capture a selected Cargo documentation operation and the HTML entries Cargo
actually reports. Package selection, metadata, diagnostics and output identities
remain available together without reconstructing documentation filenames.

## Contract

**Validated requests.** `CargoDocOptions` validates one to 256 unique packages, at most 256 unique
qualified feature requests and an optional bounded output locator. Features name
selected `package/feature` pairs. Empty, malformed, repeated or control-bearing
requests refuse before invocation; arbitrary command flags are not accepted.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::typed_requests_and_bounded_protocol_refusals_keep_a_valid_neighbor -->

**Metadata selection.** `cargo::capture` runs format-v1 metadata through the supplied runner. Each
selected workspace package has exactly one documented library, an opaque Cargo
package identity, absolute manifest and source locators and workspace-resolved features.
The result retains the complete bounded metadata output and its native digest.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::documentation_refuses_dependency_metadata_drift_with_unchanged_libraries,raf_workplan::diagram::cargo::artifact::tests::typed_requests_and_bounded_protocol_refusals_keep_a_valid_neighbor -->

**Complete execution.** `cargo::document` uses the same validated feature and output request with
`--locked --no-deps --lib --message-format=json`. It retains complete bounded
stdout, stderr and native JSON messages. One successful final `build-finished`
message and successful process status are required. Reported broken intra-doc
link diagnostics refuse. A second metadata capture must have identical bytes,
selected identities and output root, including dependency-resolution data.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::typed_requests_and_bounded_protocol_refusals_keep_a_valid_neighbor,raf_workplan::diagram::cargo::artifact::tests::documentation_refuses_dependency_metadata_drift_with_unchanged_libraries -->

**Contained artifacts.** Each selected library requires exactly one reported HTML entry matching its
package, manifest, target and crate types. The native disk reader
admits a contained regular file at the reported path, with a 16 MiB byte ceiling.
Duplicate, missing, aliased, outside-root, malformed and unreadable entries
refuse the whole result. Each observation retains its actual entry directory,
physical file identity and native content digest.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files -->

**Reported features.** Each artifact retains its own canonical Cargo-reported feature set. Every explicitly requested feature for that selected package must be present. Workspace metadata features remain a separate observation; differences between those scopes do not reject valid artifacts.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files,raf_workplan::example_crate::cargo_documentation -->

**Reported freshness.** Cargo's `fresh` field is a required Boolean. False reports
newly built output; true reports cached output. Both values are retained.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files -->

## Usage

`raf_workplan::example_crate::cargo_documentation` demonstrates the complete call.

Construct `CargoDocOptions` for explicit packages and qualified features. Pass
the same trusted `Runner` and workspace root to `cargo::capture`, then pass the
selection to `cargo::document`. Inspect `DocumentationRun::artifacts` for admitted
HTML entries and `DocumentationRun::output` for the complete diagnostic streams.
`Selection::metadata_output` retains the metadata response independently.
`DocumentationRun::recaptured_metadata_output` retains the second metadata
response, including its independently observed diagnostic stderr.
`LibraryArtifact::features` reports the selected documentation feature set;
`Library::features` retains the workspace metadata feature union.

The runner starts Cargo in the selected workspace. Cargo owns its cumulative
build and documentation output; capture does not clean or publish an assembly.
An explicit output locator becomes a shared Cargo configuration argument for
metadata and documentation, while metadata supplies the effective absolute root.

## Examples

Run `cargo run -p raf-workplan --example cargo_documentation` without arguments.
It documents two local libraries with an explicit feature request, admits their
reported HTML and checks actual output containing a documented alias, a macro
item, generated source and a documentation-only item. It then documents just one
library under the reported output directory and checks that the other entry's
bytes remain unchanged.

An unselected third member enables `extra` for the first library in workspace
metadata. Its selected documentation artifact retains the requested `extended`
feature and omits `extra`, preserving the distinction between operation scopes.

A cached `fresh: true` receipt is admitted beside a newly built `fresh: false`
receipt. A missing or string-valued freshness field refuses. Matching package
names do not admit a different opaque package identity or source path. A missing
explicitly requested feature refuses; additional observed features are retained.
A complete matching neighboring receipt succeeds.

## Constraints

- The request admits at most 256 packages and 256 qualified features, and each
  runner stream admits at most 16 MiB. Larger observations refuse.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::typed_requests_and_bounded_protocol_refusals_keep_a_valid_neighbor -->
- Each reported HTML entry must be a contained regular file of at most 16 MiB.
  Aliased files cannot satisfy different selected libraries.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files -->
- Complete metadata stdout must remain byte-identical. Dependency drift refuses
  even when the selected library identity is unchanged.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::documentation_refuses_dependency_metadata_drift_with_unchanged_libraries -->
- An artifact must report every explicitly requested feature for its selected
  package. A receipt that omits one refuses.
<!-- raf:tested-by raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files -->

## Explanation

This interface observes metadata, runner output and reported artifact bytes.
Source paths are locators. Metadata features are not effective compiler cfg;
generated or expanded definitions are not inferred from source declarations.
Matching before/after metadata does not establish atomic subprocess inputs.
Workspace metadata can include features enabled by an unselected member, so its
feature union does not identify the selected documentation artifact's feature set.

Artifact observations do not authenticate HTML provenance or all embedded links,
and do not authorize assembly publication or cached compiler-receipt reuse.
The checked assembly contract separately requires those inputs. The local
adapter supplies Cargo selection and protocol policy; native PurRDF supplies
bounded JSON, sorting, allocation and byte identities, while `raf-source`
supplies contained file reads.

Neither Cargo freshness value authenticates compiler inputs, effective cfg or
source expansion.
`no_default_features` controls the Cargo request flag; dependencies can still
activate features, including defaults.

Cargo determines package identities, feature resolution and documentation output
locations. Retaining its complete responses makes these observations inspectable.
Strict matching prevents one library's reported HTML from satisfying another's
selection. Native handle-based reads preserve the admitted file's byte identity.
Full metadata equality catches dependency drift even when selected libraries are
unchanged, with deliberate refusal of formatting-only metadata changes too.

## Verification

The artifact fixtures exercise identity, containment, uniqueness, byte limits,
freshness and missing-output refusals beside valid observations. A metadata
fixture changes a dependency while preserving the selected library and requires
the documentation operation to refuse. The native example uses the real Cargo
protocol and verifies selected features, reported entries and cumulative output.
It also admits documentation whose reported features differ from the workspace
metadata union because an unselected member enables an additional feature.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_workplan::diagram::cargo::artifact::tests::documentation_refuses_dependency_metadata_drift_with_unchanged_libraries` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::diagram::cargo::artifact::tests::reported_html_requires_exact_selected_receipts_and_contained_files` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::diagram::cargo::artifact::tests::typed_requests_and_bounded_protocol_refusals_keep_a_valid_neighbor` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::example_crate::cargo_documentation` | example target | [Contract](#contract), [Usage](#usage) |
<!-- raf:generated:end -->
