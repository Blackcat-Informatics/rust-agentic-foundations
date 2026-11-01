---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-rustdoc-observation
title: Explicit Rustdoc invocation observations
description: Selected compiler arguments, environment and native configuration queries joined to Cargo documentation artifacts.
state: delivered
bindings:
  code:
    - raf_workplan::diagram::cargo::observation
    - raf_workplan::exec::RustdocObservationOptions
    - raf_workplan::exec::RustdocObservationOptionsError
    - raf_workplan::exec::Invocation::cargo_doc_observed
    - raf_workplan::exec::Invocation::rustdoc_observation
    - raf_workplan::exec::Call::rustdoc_observation
    - raf_workplan::exec::Call::is_rustdoc_delegate
  tests:
    - raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing
    - cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context
  examples:
    - raf_workplan::example_crate::cargo_documentation
depends: [cap:workplan-cargo-documentation, cap:workplan-complete-gate, cap:rdf-report-profile]
---

## Purpose

Observe the documentation compiler used by an explicit Cargo operation. Its
arguments, environment, configuration query and successful execution stay
distinct from metadata features and the HTML artifact observations they accompany.

## Contract

**Explicit selection.** `RustdocObservationOptions` requires absolute driver,
delegate and new receipt-directory locators. The operation deliberately selects
that compiler; it does not infer ambient Cargo configuration or resolve a default
compiler. Executable entrypoint spellings remain intact, including shim behavior.
Physical identity checks reject recursive delegation and observed replacement.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

**Scoped execution.** `observation::document` instruments only its typed Cargo
documentation invocation. The driver dispatches before ordinary command parsing,
preserves Cargo's ordered arguments and complete environment, and delegates through
the existing bounded runner. Compiler calls restore selected `RUSTDOC`, remove
observer controls and use the production native PID namespace supervision.
Abandonment signals the admitted launcher identity, and parent death tears down
its namespace descendants through the same native runner contract.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

**Native configuration.** A separate information query uses the same compiler,
working directory and compiler environment. It preserves argument/value pairing
and adds `-Z unstable-options --print=cfg` before an actual option terminator.
Successful native output must contain `doc`; no predicate is synthesized.
Query output remains separate from the unchanged documentation invocation's
stdout, stderr and successful outcome.
<!-- raf:tested-by raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing,cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

**Closed observations.** Successful driver receipts use native canonical RDF 1.2
N-Triples, content digests and one new private directory. Scalar cardinalities,
ordered arguments, sorted unique environment names, successful outcomes and
complete assertion accounting are required. Unknown or orphaned facts, malformed
receipts and inconsistent query/environment reconstruction refuse.
<!-- raf:tested-by raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing -->

**Selected artifacts.** Every selected library requires exactly one matching new
compiler receipt and its ordinary admitted Cargo HTML artifact. Package name,
version, manifest, crate name and actual input operand must agree. An option value
that names the desired source cannot substitute for that operand. Missing,
duplicate or unmatched receipts refuse; a cached artifact alone is insufficient.
The existing metadata recapture and artifact validation apply unchanged.
<!-- raf:tested-by raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing,cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

## Usage

`raf_workplan::example_crate::cargo_documentation` demonstrates the native call
with an explicit Rustdoc selection and retained compiler observations.

Capture the package, feature and output selection with `cargo::capture`. Supply
absolute executable locators for `cargo-workplan` as the observing driver and the
chosen Rustdoc delegate, plus a new receipt directory whose parent exists. Pass
those options and the captured selection to `observation::document`.

The returned `ObservedDocumentation` retains the ordinary documentation result
and selected compiler observations in library order. Each observation exposes its
arguments, environments, separate outputs, native RDF graph and receipt file
identity. The operation publishes no assembled documentation and preserves
unrelated cumulative Cargo output.

## Examples

Two selected libraries share a native Cargo operation. One uses a selected feature
and a build-script configuration value; an owned Cargo configuration contributes
another Rustdoc predicate. The actual native query retains all three beside `doc`,
and the reported HTML entries identify their selected package versions.

The argument boundary includes a resource suffix whose value is `--`. That value
stays paired with its option while query controls precede an actual terminator.
An unrelated input accompanied by the selected source as an HTML option value
fails source association. A valid neighboring receipt decodes and matches.

Repeating the documentation request without a new compiler invocation cannot use
cached HTML as its compiler receipt. This refusal leaves the existing artifacts
and unrelated output intact.

Run the real consuming fixture with
`cargo test -p cargo-workplan --test rustdoc_observation`. Its driver is the
actual `cargo-workplan` binary; the fixture deliberately selects a native Rustdoc
and an owned alias of that executable.

## Constraints

- Argument and environment sequences admit at most 4096 entries each. Each
  original argument/environment payload and the complete receipt directory admit
  at most 16 MiB. Runner streams retain their existing bounds.
<!-- raf:tested-by raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing -->
- Native Unix private-directory permissions and a compiler supporting the stated
  Rustdoc information query are required. UTF-8 locators, arguments, environment
  names/values and receipt streams are required; unsupported observations refuse.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->
- The admitted HTML argument grammar requires one file operand. Unknown options,
  response-file expansion, standard input, partial/non-HTML output and unsupported
  unstable modes refuse. Only `unstable-options` and `threads=...` are admitted
  through `-Z`; the compiler validates their values.
<!-- raf:tested-by raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing -->
- Receipts are observations of this operation, not executable authentication,
  atomic source/input evidence, macro expansion, generated-definition admission
  or complete link validation. Earlier receipts do not establish a new invocation.
<!-- raf:tested-by cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context -->

## Explanation

The Cargo driver bridge uses closed invocation controls rather than changing the
process-wide environment. The same runner handles queries and documentation;
the outer Cargo watchdog supervises its admitted launcher through a kernel
process handle, and namespace teardown contains detached descendants. Native RDF construction,
canonical parsing, digests, contained file reads and existing artifact matching
provide the common storage and observation mechanisms.

The receipt namespace is `urn:raf:rustdoc-observation:`. The snapshot root has
the following closed fields; child rows use zero-based contiguous fragment names.

| Subject | Properties |
| --- | --- |
| Snapshot root | `form` (integer 1), `driver`, `delegate`, `directory` |
| Snapshot root | `argumentCount`, `queryArgumentCount`, `cargoEnvironmentCount`, `compilerEnvironmentCount` |
| Snapshot root | `configurationCode`, `configurationStdout`, `configurationStderr`, `configurationElapsedSeconds`, `configurationElapsedNanoseconds` |
| Snapshot root | `htmlCode`, `htmlStdout`, `htmlStderr`, `htmlElapsedSeconds`, `htmlElapsedNanoseconds` |
| Snapshot root | Optional `assembly`, a string containing the closed native assembly preparation graph |
| `#argument/<ordinal>`, `#queryArgument/<ordinal>` | `value` |
| `#cargoEnvironment/<ordinal>`, `#compilerEnvironment/<ordinal>` | `name`, `value` |

Counts, status codes and duration components are canonical nonnegative integers;
other values are strings. Successful status codes equal zero. Duration components
retain the runner's existing observation without another clock read; tests make
no claim about their measured values.

The optional preparation graph binds the exact plan, selected package, compiler
version output, effective HTML arguments, source observations and prepared
fragment identities described by [literal documentation includes](workplan-rustdoc-includes.md).

[Checked documentation assembly](workplan-rustdoc.md) separately combines compiler
observations with expanded/generated inputs, source and API links, package
membership, publication and host behavior. Artifact hashes or this separate cfg
query do not replace those contracts.

## Verification

The private receipt case checks actual input ownership, option-value/terminator
pairing and strict native RDF shape admission with valid neighbors. The native
Cargo consumer uses two libraries, a selected feature, build-script and Rustdoc
configuration, the real driver executable, admitted artifacts and a cached-result
refusal. Neither case asserts elapsed time.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::rustdoc_observation::selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::diagram::cargo::observation::driver::tests::receipts_bind_the_input_operand_and_preserve_option_value_pairing` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::example_crate::cargo_documentation` | example target | [Usage](#usage) |
<!-- raf:generated:end -->
