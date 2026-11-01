---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-rustdoc-includes
title: Compiler-observed literal documentation includes
description: Prepare shared static includes before Cargo and admit public API links through the actual selected Rustdoc compiler.
state: delivered
bindings:
  code:
    - raf_workplan::diagram::cargo::assembly
    - raf_workplan::exec::RustdocObservationOptions::with_assembly
    - raf_workplan::exec::Invocation::rustdoc_compiler_version
  tests:
    - cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context
    - raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing
  examples:
    - raf_workplan::example_crate::cargo_documentation
depends: [cap:source-configured-target, cap:workplan-generated-docs, cap:workplan-cargo-documentation, cap:workplan-rustdoc-observation]
---

## Purpose

Connect a shared named diagram to the exact library and public module consuming
a literal documentation include. Ordinary dependency compilation receives a
complete static fragment; the actual documentation compiler consumes the final
source-admitted fragment before immutable bundle publication.

## Contract

**Explicit inputs.** `assembly::document` takes the captured Cargo selection,
explicit observing driver and compiler, an AssemblyPlan, a new absolute plan
locator and a caller-owned Workplan seed model. Each IncludeRequest names its
selected package, receiving module, include, named view, capability and complete
required API spellings. Plans use a closed canonical native RDF carrier with
shared item and byte bounds.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

**Complete initial includes.** Every static fragment and destination ownership
decision is prepared before writing. Existing different contents require the
exact prior-content digest; identical contents can be reused. The exclusive plan
records the exact initial include digests before those includes are written and
Cargo starts. Include, plan and receipt locators remain distinct. The caller's
seed model supplies initial presentation and explicit document-model package
roots. Each driver reloads that complete declared-source model, preserving all
document and binding validation. This model does not establish another library's
compiler configuration. Final publication uses the driver's regenerated bundles
and successful compiler observations.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

**Observed source and links.** The actual driver queries native configuration and
verbose compiler identity, loads one exact configured target and audits the
receiving literal `doc = include_str!` attribute. That include supplies the
module's sole documentation content. Every declared capability code binding
must appear in the request, with distinct public aliases retained even when they
share a definition. Each spelling is resolved again in the exact configured
source index; document-model item identities are never transferred between
indexes. Raw identifiers retain valid Rust spelling in labels; intra-doc
destinations use Rustdoc's canonical names. Hidden, private,
ambiguous and unresolved required references refuse. Rustdoc receives ordinary
disambiguated intra-doc links with broken and private links forbidden; lint caps,
force-warn overrides and unsupported modes refuse.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context,raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing -->

**Identity and publication.** Fragments label Cargo-observed package/version and
metadata identity separately from native content SHA-256 of loaded root bytes
and the existing framed symbol SHA-256 identities. They confer no provider
authority. Final includes, source observations and actual Cargo HTML artifacts
are rechecked before shared immutable publication. Bundles occupy the shared
documentation root beside Cargo's crate directories, which Cargo can replace
before a compiler invocation. Public context contains
explicit identities and digests; complete compiler environments stay in private
instrumentation receipts. Cumulative unrelated output remains intact.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

## Usage

Capture the Cargo selection and declare literal receiving includes in source.
Construct the requests with their actual capability code-binding paths and named
views. Supply a loaded shared seed model to `assembly::document`, with a new plan
file and receipt directory. Use `IncludeRequest::replace_owned` when replacing
different previously owned include contents. Read private compiler evidence from
the returned observations and public bundle locations from publications.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

Run `raf_workplan::example_crate::cargo_documentation` to capture selected
libraries, preserve cumulative output and then assemble a literal include with
an explicit seed model, plan locator and compiler receipts.
<!-- raf:tested-by raf_workplan::example_crate::cargo_documentation -->

## Examples

The runnable `raf_workplan::example_crate::cargo_documentation` example
first checks compiler observations, generated and macro-expanded documentation,
and cumulative output. It then documents a simple `doc`-selected public struct
through a literal root include, checks the returned immutable publication and
confirms the final `struct@crate::Documented` reference.
<!-- raf:tested-by raf_workplan::example_crate::cargo_documentation -->

A two-library workspace documents a function selected by `doc`, build-script
configuration, Rustdoc flags and an enabled feature. Two public aliases of that
function remain separate required references, beside a raw-keyword module and
function. Root and nested-module includes use the same shared diagram, with
compiler-resolved links from their actual pages. Each selected library retains
its immutable publication while an unrelated output file remains unchanged.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

## Constraints

- Only literal receiving includes and indexed declaration headers are admitted.
  Opaque rewriting attributes, item expansion and possible builtin include
  shadowing require unavailable expanded-input evidence and refuse. Explicit
  diagram navigation needing Cargo-host href admission also refuses. The broader
  [Rustdoc assembly contract](workplan-rustdoc.md) covers generated definitions,
  materialized explorer navigation and packaged hosting.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context,raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing -->

- Source and output checks are observations before and after use. They do not
  provide atomic compiler inputs, a directory transaction or isolation from a
  concurrent publisher. A failed later operation can leave its owned plan and
  includes; those files alone are not successful compiler or publication evidence.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

## Explanation

Dependency compilation can read a documentation include before Cargo launches
that library's Rustdoc invocation. Complete initial fragments satisfy that read
without inventing a compiler configuration. The driver then replaces exactly
those owned bytes after observing its real context, and publication verifies the
final fragments actually consumed by Rustdoc. Presentation, configuration,
source identity, compiler link admission and publication retain separate roles.

## Verification

The bound native case exercises actual Cargo selection, build-script and Rustdoc
predicates, initial include creation, exact configured source, public aliases,
raw identifiers, native digest domains, required-link lint arguments, final HTML
and immutable public context without private environment leakage. The bound
argument case pairs admitted options with forbidden overrides and checks omitted
or canonically duplicated capability bindings beside two valid public aliases.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::example_crate::cargo_documentation` | example target | [Usage](#usage), [Examples](#examples) |
<!-- raf:generated:end -->
