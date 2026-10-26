---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-detectors
title: "Built-in census detectors"
description: "The fourteen generic detectors the RAF census ships, each with a positive and a negative fixture."
state: implementing
bindings:
  code:
    - raf_census::detectors
    - raf_census::gate::context
    - raf_census::gate::subjects
    - raf_census::Unit::string_initializer
  tests:
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

## Contract

> [!PROPOSED change:census-detectors]
> Byte-layout recognition runs on the bounded source parse and assigns each hit
> to the innermost actual source item, deduplicated by physical Census ownership.
> Its shipping eligibility comes from that owner's aggregate target reaches.
> Alignment recognition requires the added bias to equal the complemented low
> mask; clearing low bits alone is not rounding up. First-mismatch recognition
> requires the zero count's receiver to retain an XOR of loaded words, with
> reassignment invalidating tracked values. An unrelated bit scan is not a hit.
> Formatting-based framing follows rendered positional or named argument order,
> including implicit precision arguments, rather than source argument order.
> Hex-table and JSON Pointer detectors reuse the collector's decoded facts.
> Tables apply to every target; JSON Pointer hits are ordinary string literals
> exactly equal to `~0` or `~1` in shipping units. The byte-layout, hex-table and
> JSON Pointer detectors exempt the actual home package for an ordinary job.
> Syntax and fact hits share the same production Gate matching path.
> A rule finding names the job's configured home as its required replacement,
> retaining the chosen public entry point when it is a reexport.
> Syntax recognizers describe the detected construction; they do not select a
> different hard-coded library or standard-library alternative.
> Job-dependent evidence is isolated to the job that names its detector.
> Home-literal evidence includes empty ordinary strings and requires at least
> one home literal. Vocabulary evidence uses constants named `NS` or ending in
> `_NS` under the home or entry points. Each candidate must have one nonempty
> ordinary string initializer, optionally parenthesized or inside a single-value
> block; computed or unavailable values are errors. Sole-name includes test
> targets in the home package and exempts the actual home and entry-point units.

A preferred-provider job does not exempt the adapter's package. Its selected
detectors also report unlisted competing helpers and non-function items in
that package. A discarded provider call does not excuse a detector hit.
Home and entry-point exemptions contain only validated consumer and API
identities. A module home does not exempt unrelated descendants. Every source
identity of a shared physical consumer body is checked in its own context before
the body can be classified as a home.

The census ships 14 built-in detectors. Each reports hits as `rule:<id>`, where
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

The detectors that read a job's home (`home-literal`, `sole-name`,
`vocabulary-literal`) apply only to jobs that name them in `forbidden`. The census
walks shipping code and every test, bench and example target. The `Reads` column of
the table names what each detector reads: `shipping` for shipping code only, and `all`
for shipping code and every test, bench and example target. The nine `shipping`
detectors are `hex-digit-radix`, `json-pointer-escape`, `length-prefix-frame`,
`le-slice-int`, `align-up-mask`, `div-ceil-by-hand`, `xor-first-mismatch`,
`home-literal` and `vocabulary-literal`; each marks itself so in its report. The five
`all` detectors are `std-default-hasher`, `hex-format-loop`, `hex-pair-radix`,
`hex-table` and `sole-name`. Apart from `sole-name`, which exists to report a function
inside the home crate, a detector exempts the home package of an ordinary job.
A preferred-provider declaration removes this package exemption; only the actual
home and entry-point items retain their classification as homes.

Every detector ships two fixtures: a positive fixture it flags, and a negative
fixture that looks similar and that it leaves alone.

Detectors that concern one repository's domain are not built in. Such a detector is
written as a `Rule` in the repository that needs it (see
[census-rule-api](./census-rule-api.md)). An RDF structure rule belongs to an RDF
workspace, for example.

## Usage

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

For a preferred-provider job, evaluate the configured locked Gate described in
[the ledger](./census-ledger.md) and retain the functionality detector on the job.
The approved package is not a package exemption: the Gate must prove each
declared native consumer and still report unlisted competing code.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package -->

The executable `raf_census::example_crate::providers` constructs the canonical
native ledger and exact configured source associations, accepts its approved
encoder adapter and detects an unlisted competing table in the same package.

## Examples

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

A preferred-provider adapter can return the result of its admitted native
encoder while another helper in the same package spells a hex-digit table.
Selecting `hex-table` still reports that competing helper. A discarded native
call inside the helper cannot exempt it; a neighbouring nonmatching byte string
remains clean. Only proved native consumers and approved API definitions receive
home or entry-point exemptions.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package,raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis -->

## Constraints

- The detector set is closed; there is no configuration that adds a built-in id.
  New generic detectors arrive with a new release of the crate.
- Detectors work on syntax. They do not resolve types, so `std-default-hasher`
  recognises the std paths and aliases it can see in the file.
- A detector reports hits; whether a hit is a finding is decided by the ledger job
  that forbids it.

## Explanation

These fourteen detectors are the recurring hand-written helpers that the diverged
census copies each grew separately: byte-level encodings, bit tricks, and
hash-collection defaults. They are generic, so they live in the shared crate.
Detectors that name one project's types would drag that project into every
consumer, so those stay in the consuming repository as `Rule` implementations.

Pairing every detector with a negative fixture is deliberate. A detector that
matches too much is worse than none, because its false positives teach people to
add exemptions.

## Verification

> [!PROPOSED change:census-detectors]
> A test lists the 14 ids and fails if the set of registered built-in ids differs
> from this table. A test fails for any registered detector that lacks either
> its positive or its negative fixture.

- Each detector has a positive fixture that yields exactly one hit with the right
  `rule:<id>`, and a negative fixture that yields none.
- A fixture shows that a detector other than `sole-name` reports no hit inside the
  home package of an ordinary job that forbids it.
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
