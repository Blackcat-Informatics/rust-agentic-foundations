---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-detectors
title: "Built-in census detectors"
description: "Seventeen built-in detectors for recurring helpers and RDF representation, each with positive and negative fixtures."
state: delivered
bindings:
  code:
    - raf_census::detectors
    - raf_census::gate::context
    - raf_census::gate::subjects
    - raf_census::Unit::string_initializer
  tests:
    - raf_census::detectors::tests::every_executable_detector_has_positive_and_negative_gate_fixtures
    - cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity
    - raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases
    - raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries
    - raf_census::test_crate::census_gate::layout_findings_require_related_operands_and_rendered_prefix_order
    - raf_census::test_crate::census_gate::fact_detectors_reuse_collected_values_and_keep_their_target_scope
    - raf_census::test_crate::census_gate::hex_detectors_apply_to_their_declared_targets_and_actual_format_fields
    - raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence
    - raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope
    - raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package
    - raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis
    - raf_census::test_crate::census_gate::configured_provider_checks_every_context_of_a_shared_consumer_body
  examples:
    - raf_census::example_crate::providers
depends: [cap:census-normal-form]
---

# Built-in census detectors

## Purpose

Fingerprints find copies of a body that already exists elsewhere. Some duplicated
helpers are better recognised by what they do: a hex-digit table, a hand-rolled
ceiling division, a default-hasher map. Detectors are generic checks for these
recurring hand-written helpers, and they report as `rule` fingerprints that a
ledger job can forbid.

Representation policies also reject written JSON dependencies, recognized RDF
text formatters and local RDF term enums that omit triple terms. A repository
can require these checks even when its ledger has no helper jobs.

Complete fixture coverage makes detector verdicts dependable: the executable
registry and behavioral cases must agree before the suite can pass.

## Contract

### Required representation policies
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

The executable registry adds `serde-json`, `rdf-text-writer` and
`rdf-term-model`. All three inspect every selected first-party target kind,
including tests, examples, benches and build scripts. Their source subjects
include imports, types and macros without fingerprintable function bodies.
Required-rule selection turns their hits into hard findings before job home,
variant and debt policies apply.

`serde-json` recognizes written dependency paths and import/extern aliases,
respecting local type/module shadows and explicit external roots. Recognized
formatting macros emit a finding when their literal format spells an RDF row
with an IRI subject, predicate or typed literal and terminating dot. A local
enum with IRI, blank-node and literal alternatives but no triple-term
alternative produces `rdf-term-model`. Literal enum declarations and formatter
invocations inside macro definitions are included. These are stated syntax
recognizers, not proof of arbitrary semantic equivalence or RDF conformance.

All three representation policies read shipping and nonshipping targets and
have no ordinary-job home-package exemption.

### Source ownership and byte-layout recognition
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries,raf_census::test_crate::census_gate::layout_findings_require_related_operands_and_rendered_prefix_order -->

Byte-layout recognition runs on the bounded source parse and assigns each hit
to the innermost actual source item, deduplicated by physical Census ownership.
Its shipping eligibility comes from that owner's aggregate target reaches.
Alignment recognition requires the added bias to equal the complemented low
mask; clearing low bits alone is not rounding up. First-mismatch recognition
requires the zero count's receiver to retain an XOR of loaded words, with
reassignment invalidating tracked values. An unrelated bit scan is not a hit.
Formatting-based framing follows rendered positional or named argument order,
including implicit precision arguments, rather than source argument order.

### Decoded facts and replacement diagnostics
<!-- raf:tested-by raf_census::test_crate::census_gate::fact_detectors_reuse_collected_values_and_keep_their_target_scope,raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

Hex-table and JSON Pointer detectors reuse the collector's decoded facts.
Tables apply to every target; JSON Pointer hits are ordinary string literals
exactly equal to `~0` or `~1` in shipping units. The byte-layout, hex-table and
JSON Pointer detectors exempt the actual home package for an ordinary job.
Syntax and fact hits share the same production Gate matching path.
A rule finding names the job's configured home as its required replacement,
retaining the chosen public entry point when it is a reexport.
Syntax recognizers describe the detected construction; they do not select a
different hard-coded library or standard-library alternative.

### Job-scoped literals and names
<!-- raf:tested-by raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence,raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope -->

Job-dependent evidence is isolated to the job that names its detector.
Home-literal evidence includes empty ordinary strings and requires at least
one home literal. Vocabulary evidence uses constants named `NS` or ending in
`_NS` under the home or entry points. Each candidate must have one nonempty
ordinary string initializer, optionally parenthesized or inside a single-value
block; computed or unavailable values are errors. Sole-name includes test
targets in the home package and exempts the actual home and entry-point units.

### Complete fixture coverage
<!-- raf:tested-by raf_census::detectors::tests::every_executable_detector_has_positive_and_negative_gate_fixtures,cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

Every executable built-in has positive and negative Gate fixtures. The fixture
inventory is checked against the executable registry, rejecting missing sides,
duplicate coverage and unregistered fixture families. Behavioral cases consume
the same fixture inputs as this inventory check.

### Preferred-provider ownership
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package,raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis,raf_census::test_crate::census_gate::configured_provider_checks_every_context_of_a_shared_consumer_body -->

A preferred-provider job does not exempt the adapter's package. Its selected
detectors also report unlisted competing helpers and non-function items in
that package. A discarded provider call does not excuse a detector hit.
Home and entry-point exemptions contain only validated consumer and API
identities. A module home does not exempt unrelated descendants. Every source
identity of a shared physical consumer body is checked in its own context before
the body can be classified as a home.

### Detector identities and target eligibility
<!-- raf:tested-by raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases,raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries,raf_census::test_crate::census_gate::fact_detectors_reuse_collected_values_and_keep_their_target_scope,raf_census::test_crate::census_gate::hex_detectors_apply_to_their_declared_targets_and_actual_format_fields,raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence,raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope -->

The census ships seventeen built-in detectors. Each reports hits as `rule:<id>`, where
`<id>` is the detector id, and a ledger job names the hit in
`forbiddenFingerprint`.

| Id | Reads | Meaning |
| --- | --- | --- |
| `std-default-hasher` | all | A std `HashMap` or `HashSet` left on its random default hasher. |
| `hex-format-loop` | all | A `{:02x}` or `{:02X}` format inside a loop or closure. |
| `hex-pair-radix` | all | `u8::from_str_radix(_, 16)` used to decode a hex pair. |
| `hex-table` | all | A hex-digit table written by hand, subject to the job's home policy. |
| `hex-digit-radix` | shipping | A single hex digit decoded through a radix conversion. |
| `home-literal` | shipping | A string literal equal to one the home item's own bodies spell, outside the home item. |
| `sole-name` | all | A function inside the home crate that a `forbiddenName` expression matches, other than the home or an entry point. |
| `vocabulary-literal` | shipping | A string literal equal to or starting with a namespace constant declared under the job's home and entry-point modules, outside those modules. |
| `json-pointer-escape` | shipping | Hand-written escaping of `~` and `/` for a JSON Pointer. |
| `length-prefix-frame` | shipping | A hand-written length-prefixed frame read or write. |
| `le-slice-int` | shipping | An integer assembled from a little-endian byte slice by hand. |
| `align-up-mask` | shipping | Alignment rounding written as an add-and-mask. |
| `div-ceil-by-hand` | shipping | Ceiling division written as an add-and-divide instead of `div_ceil`. |
| `xor-first-mismatch` | shipping | A first-mismatch search written as an XOR and a bit scan. |
| `serde-json` | all | A written external JSON dependency path or alias. |
| `rdf-text-writer` | all | A formatting macro whose literal spells the recognized RDF row shape. |
| `rdf-term-model` | all | A local IRI, blank-node and literal enum without a triple-term alternative. |

The detectors that read a job's home (`home-literal`, `sole-name`,
`vocabulary-literal`) apply only to jobs that name them in `forbidden`. The census
walks shipping code and every test, bench and example target. The `Reads` column of
the table names what each detector reads: `shipping` for shipping code only, and `all`
for shipping code and every test, bench and example target. The nine `shipping`
detectors are `hex-digit-radix`, `json-pointer-escape`, `length-prefix-frame`,
`le-slice-int`, `align-up-mask`, `div-ceil-by-hand`, `xor-first-mismatch`,
`home-literal` and `vocabulary-literal`; each marks itself so in its report. The eight
`all` detectors also include the three representation policies alongside
`std-default-hasher`, `hex-format-loop`, `hex-pair-radix`, `hex-table` and
`sole-name`. Apart from `sole-name` and the three representation policies,
a detector exempts the home package of an ordinary job.
A preferred-provider declaration removes this package exemption; only the actual
home and entry-point items retain their classification as homes.

Every detector ships positive fixtures it flags and negative fixtures that look
similar and that it leaves alone.

Detectors that concern one repository's domain are not built in. Such a detector is
written as a `Rule` in the repository that needs it (see
[census-rule-api](./census-rule-api.md)). An RDF vocabulary's application-specific
structure rule belongs to its consuming workspace, for example.

## Usage

Select the representation policies with
`Gate::workspace(root).require_rules(["serde-json", "rdf-text-writer", "rdf-term-model"])`.
Selection does not require a ledger job or a source home. The repository's
self-Census makes this selection for the real snapshot and its mutation
fixtures. Other detectors retain their documented source eligibility.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

A job forbids a detector by listing its fingerprint. Serialize its graph with
the [native report writer](./rdf-report-profile.md), which computes the snapshot
BASE and canonical bytes. This complete ledger is native writer output:

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-report/30b0289c5affe30deea1d0581eed949178d1fc783e0d9ce69a3b19b6e1543140>
<> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#Ledger> .
<> <https://blackcatinformatics.ca/raf/report/1#job> <#ceil-div> .
<> <https://blackcatinformatics.ca/raf/report/1#normalForm> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
<> <https://blackcatinformatics.ca/raf/report/1#reportForm> "4"^^<http://www.w3.org/2001/XMLSchema#integer> .
<#ceil-div> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#LedgerJob> .
<#ceil-div> <https://blackcatinformatics.ca/raf/report/1#enforced> "true"^^<http://www.w3.org/2001/XMLSchema#boolean> .
<#ceil-div> <https://blackcatinformatics.ca/raf/report/1#forbiddenFingerprint> "rule:div-ceil-by-hand" .
<#ceil-div> <https://blackcatinformatics.ca/raf/report/1#homeSymbol> "demo_util::math::ceil_div" .
<#ceil-div> <https://blackcatinformatics.ca/raf/report/1#jobId> "ceil-div" .
<#ceil-div> <https://blackcatinformatics.ca/raf/report/1#summary> "Ceiling division." .
```

The native ledger is read by `Gate::workspace(directory).check()` or by
`check_tree` for a caller-owned tree. Each hit retains its file, line and
`rule:<id>`. The declared home must resolve in that workspace.

A repository-owned registered check reports selected built-ins through the same
command as custom rules: `cargo census --workspace DIR check --package PACKAGE
--example EXAMPLE`. Its registered target evaluates the ledger's chosen built-in
ids and returns their native findings to the command.
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

For a preferred-provider job, evaluate the configured locked Gate described in
[the ledger](./census-ledger.md) and retain the functionality detector on the job.
The approved package is not a package exemption: the Gate must prove each
declared native consumer and still report unlisted competing code.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package -->

The executable `raf_census::example_crate::providers` constructs the canonical
native ledger and exact configured source associations, accepts its approved
encoder adapter and detects an unlisted competing table in the same package.

## Examples

An empty-job ledger still refuses a function that calls
`serde_json::from_str`, a `format!("<{s}> <{p}> <{o}> .")` emitter, or a local
`enum Term { Iri(String), BlankNode(String), Literal(String) }`. A native
dependency renamed to `serde_json` is distinct from that package; ordinary
formatting without RDF syntax and a term enum retaining `TripleTerm` are
neighboring accepted syntax. Native RDF writers and term types remain the
required representation providers.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

A positive fixture for `div-ceil-by-hand`, which the detector flags:

```rust
fn pages(bytes: usize, page: usize) -> usize {
    (bytes + page - 1) / page
}
```

Its negative fixture, which the detector leaves alone because it already uses the
standard method:

```rust
fn pages(bytes: usize, page: usize) -> usize {
    bytes.div_ceil(page)
}
```

For a `home-literal` job, the home package's function returns `"token"` and another
package repeats `"token"` in a helper. Naming `home-literal` reports that helper;
removing the job leaves the same sources clean. A `sole-name` job likewise needs
explicit selection, including for matching functions in the home package.
<!-- raf:tested-by raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence,raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope -->

A preferred-provider adapter can return the result of its admitted native
encoder while another helper in the same package spells a hex-digit table.
Selecting `hex-table` still reports that competing helper. A discarded native
call inside the helper cannot exempt it; a neighbouring nonmatching byte string
remains clean. Only proved native consumers and approved API definitions receive
home or entry-point exemptions.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package,raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis -->

## Constraints

Written syntax does not authenticate dependency identities, expand arbitrary
macros or establish full RDF semantics. Locked cross-package resolution and
preferred-provider evidence retain their own admission and refusal rules.
Required findings cannot be adopted into structural debt or sanctioned by a
job-specific variant.
Recognized formatter literals begin with an IRI subject, contain a formatted
IRI and either another formatted IRI or literal syntax, and end in a dot.
Fixed subject and predicate IRIs with an IRI or quoted literal object are also
recognized when the term boundaries use spaces.
Other serialization constructions are outside this recognizer's syntax
contract. An accepted snippet does not establish RDF 1.2 conformance or
delegation to a native provider.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

- The detector set is closed; there is no configuration that adds a built-in id.
  New generic detectors arrive with a new release of the crate.
  <!-- raf:tested-by raf_census::detectors::tests::every_executable_detector_has_positive_and_negative_gate_fixtures -->
- Detectors work on syntax. They do not resolve types, so `std-default-hasher`
  recognises the std paths and aliases it can see in the file.
  <!-- raf:tested-by raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases -->
- A detector reports hits; a required-rule selection makes eligible hits hard
  findings, while other hits become findings through the ledger job that forbids
  them.
  <!-- raf:tested-by raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence,raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope -->

## Explanation

These seventeen detectors recognise recurring byte-level encodings, integer layout
operations, literals, hash-collection defaults and RDF representation syntax.
The shared crate gives each
consumer the same source ownership, target eligibility and ledger interpretation.
Domain-specific rules stay in the consuming repository as `Rule` implementations,
so its type names and policy do not become dependencies of every Census consumer.

The representation recognizers implement RAF's source policy. PurRDF's inspected
helper-census package is a binary with private recognizers rather than a public
detector library. Native provider identity and actual delegation remain separate
from these syntax checks.

Pairing every detector with a negative fixture is deliberate. A detector that
matches too much is worse than none, because its false positives teach people to
add exemptions.

## Verification

The existing detector Gate case covers all three policy families with an
empty-job ledger, ordinary/test contexts, macro declarations and native extern
aliases. Positive and negative fixtures participate in the existing executable
registry completeness check. The real self-Census selects these same rule IDs.

The registry check compares executable built-in ids against the fixture
families consumed by the behavioral Gate tests. Missing or duplicate families,
unregistered ids and missing positive or negative fixture sets fail the check.
The real registered command transports built-in and custom-rule findings
unchanged from the native Gate, and accepts the neighbouring clean source.

- Each detector has a positive fixture that yields exactly one hit with the right
  `rule:<id>`, and a negative fixture that yields none.
- A fixture shows that a detector other than `sole-name` reports no hit inside the
  home package of an ordinary job that forbids it, except the three representation
  policies, which do not exempt home packages.
- A preferred-provider fixture reports unlisted competing functions and structs
  in the adapter package, including a helper containing a discarded native call,
  while accepting a native helper and a neighbouring nonmatching table.
- A fixture shows `sole-name` reports a matching function inside the home crate that
  is neither the home item nor an entry point, and reports neither of those two.
- A fixture per `shipping` detector shows it flags a positive source in shipping
  code and is silent on the same source in a test target, and a fixture per `all`
  detector shows it flags a positive source in both.
- A fixture shows `home-literal`, `sole-name` and `vocabulary-literal` are silent
  for a job that does not name them.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity` | test | [Contract](#contract), [Usage](#usage) |
| `raf_census::detectors::tests::every_executable_detector_has_positive_and_negative_gate_fixtures` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::example_crate::providers` | example target | [Usage](#usage) |
| `raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::configured_provider_checks_every_context_of_a_shared_consumer_body` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
| `raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis` | test | [Contract](#contract), [Examples](#examples) |
| `raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::fact_detectors_reuse_collected_values_and_keep_their_target_scope` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::hex_detectors_apply_to_their_declared_targets_and_actual_format_fields` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::layout_findings_require_related_operands_and_rendered_prefix_order` | test | [Contract](#contract) |
<!-- raf:generated:end -->
