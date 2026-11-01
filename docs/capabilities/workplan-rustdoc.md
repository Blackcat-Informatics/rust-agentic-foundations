---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-rustdoc
title: Checked Cargo and Rustdoc assembly
description: Compiler-observed configuration, generated inputs and public API links for shared documentation bundles.
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-generated-docs, cap:workplan-cli, cap:workplan-complete-gate, cap:rdf-report-profile, cap:source-index, cap:workplan-cargo-documentation, cap:workplan-rustdoc-includes]
---

## Purpose

Cargo and Rustdoc assembly connects a shared named diagram to the exact library
and module receiving it. Compiler configuration, generated source, public API
links and output observations remain distinct evidence, admitted together before
the assembled documentation is published.

## Contract

Crate and module include fragments retain checked source, package and version
identities and compiler-resolved public API links. Every required binding selects
one public, documented definition in its selected library. Private, hidden,
unresolved and conditionally ambiguous references refuse. Public aliases retain
their actual exported spelling; Rustdoc resolves final HTML paths and anchors.

Explicit Cargo assembly respects selected packages, features, effective Rustdoc
configuration, target directories and cumulative output. Actual Cargo-launched
Rustdoc arguments and environment identify compiler inputs, including build-script
configuration and Rustdoc flags. Requested metadata features and caller-declared
configuration cannot replace an observed compiler receipt. A reused Cargo result
requires a matching successful owned receipt; an absent receipt refuses.

Native configured source loading admits generated inputs with their expansion
site and file provenance. Item-level includes require indexed compiler-grounded
definitions; reading a generated file alone does not establish its declarations.
Required unsupported configuration or expansion refuses without partial success.

Each selected library has an admitted actual HTML artifact and required links.
Changed metadata, loaded source observations, generated inputs, missing assets,
unresolved links and changed owned output refuse actionably. Owned versioned
publication preserves unrelated files, cumulative output and shared semantic
identities, native geometry and provenance.

## Usage

Generate checked include fragments explicitly before the delegated Rustdoc HTML
operation reads them. Supply the selected library and its receiving public module
with the required capability bindings. Consuming a packaged static include invokes
no generator during ordinary library builds.

The explicit documentation command carries package, feature and target-directory
selection through the existing typed process runner. Generic command reporting,
source loading and generated-document contracts describe the same admitted
operation. Cargo failures retain both bounded diagnostic streams.

## Examples

A multi-crate library uses one declaration for root and nested module includes,
a nested book chapter and an explorer under a non-root URL prefix. Source, API,
capability and evidence links retain the selected package/version identities.
Compiler-resolved public aliases work beside explicit refusals for private,
hidden and missing targets.

Packaged static includes and versioned explorer links retain their stated
behavior under actual docs.rs asset and script constraints. Package membership
and host examples establish the supported path without publishing a release or
changing hosting. Unsupported hosting assumptions are explicit refusals.

## Constraints

Source declarations, observed compiler facts, subprocess inputs and HTML artifact
observations are separate claims. Equality of before/after observations does not
prove an atomic read or unchanged intervening state. No approximate configuration,
guessed output root, invented public URL or missing-input fallback is admitted.

Required compiler capabilities and generated-input evidence must exist. Missing
tools or receipts produce actionable errors. Documentation assembly does not
infer a complete call graph or replace the native source and RDF models.

## Explanation

[Literal documentation includes](workplan-rustdoc-includes.md) provide the compiler-observed literal receiving path. Expanded definitions, explorer navigation and packaged host integration retain this capability's complete contract.

Selected Cargo metadata and reported HTML observations use [Cargo documentation capture](workplan-cargo-documentation.md). These observations remain distinct from the compiler, generated-input, link and publication receipts required for assembly.

Cargo assembles the effective Rustdoc invocation. Its native driver preserves
that invocation while obtaining complete compiler configuration and input
observations. Existing configured locked-source APIs consume the admitted facts;
the shared fragment and publication APIs supply presentation and owned output.
This keeps compiler evidence independent from requested options and avoids a
second semantic renderer.

## Verification

Native Linux examples cover actual multi-crate Cargo assembly, selected features
and configurations, generated definitions, module includes, compiler-resolved
aliases and required artifacts. Every refusal has a neighboring admitted input.
Host examples cover long labels, nested pages, non-root URLs and source, API,
capability and evidence links. Packaging examples verify include membership and
the stated docs.rs asset/script constraints without release or service changes.
