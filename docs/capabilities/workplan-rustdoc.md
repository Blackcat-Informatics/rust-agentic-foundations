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

Explicit Cargo assembly requires invocation-scoped `--trust-docs` through the
private command-line documentation operation. Selected packages, features and
artifact directories constrain its closed arguments. Build scripts and procedural
macros execute with the caller's permissions. Imported metadata and artifact
messages remain unauthenticated data; they cannot establish compiler success,
effective compiler configuration or generated-input provenance. Assembly refuses
required facts that the admitted data cannot establish.

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

The explicit documentation command carries package, feature and artifact-directory
selection through its private trusted operation. Analysis libraries consume data
only. Cargo failures retain bounded diagnostic streams. No document, saved result
or environment setting grants execution authority.

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

Source declarations, imported compiler facts, trusted-operation outputs and HTML
artifact observations are separate claims. Equality of before/after observations does not
prove an atomic read or unchanged intervening state. No approximate configuration,
guessed output root, invented public URL or missing-input fallback is admitted.

Required compiler capabilities and generated-input evidence must exist. Missing
tools or provenance produce actionable errors. Documentation assembly does not
infer a complete call graph or replace the native source and RDF models.

## Explanation

[Literal documentation includes](workplan-rustdoc-includes.md) provide static returned Markdown for a declared receiving path. Expanded definitions, explorer navigation and packaged host integration retain this capability's complete contract.

Selected Cargo metadata and reported HTML observations use [Cargo documentation artifacts](workplan-cargo-documentation.md). These imported observations remain distinct from authenticated compiler, generated-input, link and publication evidence required for assembly.

The private documentation operation selects fixed Cargo, rustc and rustdoc tools
and closed arguments after explicit trust. It installs no ambient compiler driver
and captures no complete environment. Configured source APIs consume admitted
facts; shared fragment and publication APIs supply presentation and owned output.
Required compiler provenance remains a separate design obligation; artifact
imports cannot silently satisfy it.

## Verification

Maintainer-controlled native Linux fixtures cover explicit trusted multi-crate
Cargo assembly, selected features
and configurations, generated definitions, module includes, compiler-resolved
aliases and required artifacts. Every refusal has a neighboring admitted input.
Static host fixtures cover long labels, nested pages, non-root URLs and source, API,
capability and evidence links. Packaging fixtures verify include membership and
the stated docs.rs asset/script constraints without release or service changes.
