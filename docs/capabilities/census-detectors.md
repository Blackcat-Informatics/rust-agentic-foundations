---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-detectors
title: "Built-in census detectors"
description: "Twenty-one built-in detectors for recurring helpers, enum match arms, and RDF representation and structure, each with positive and negative fixtures."
state: delivered
bindings:
  code:
    - raf_census::detectors
    - raf_census::gate::context
    - raf_census::gate::subjects
    - raf_census::Unit::string_initializer
  tests:
    - raf_census::detectors::tests::every_executable_detector_has_positive_and_negative_gate_fixtures
    - raf_census::test_crate::census_gate::enum_arms::enum_catch_alls_share_the_required_forbidden_and_ratcheted_routes
    - raf_census::test_crate::census_gate::enum_arms::enum_catch_alls_keep_only_arms_proved_across_shared_reaches
    - raf_census::test_crate::reference_scan::enum_catch_alls_prove_variants_from_a_selected_dependency
    - raf_census::test_crate::census_gate::ratchet::ratcheted_rules_record_counted_sites_and_hold_both_directions

    - raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases
    - raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries
    - raf_census::test_crate::census_gate::layout_findings_require_related_operands_and_rendered_prefix_order
    - raf_census::test_crate::census_gate::fact_detectors_reuse_collected_values_and_keep_their_target_scope
    - raf_census::test_crate::census_gate::hex_detectors_apply_to_their_declared_targets_and_actual_format_fields
    - raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence
    - raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope
    - raf_census::test_crate::census_gate::structure_detectors_judge_calls_against_the_jobs_declared_entry_points
    - raf_census::test_crate::census_gate::structure_variants_require_the_job_scoped_detector
    - raf_census::test_crate::census_gate::foreign_host_import_reports_only_repeats_of_the_homes_host_function
    - raf_census::test_crate::census_gate::foreign_host_import_refuses_method_constructor_and_property_shapes_beside_a_valid_neighbour
    - raf_census::test_crate::census_gate::foreign_host_import_refuses_arguments_it_cannot_place_beside_a_valid_neighbour
    - raf_census::test_crate::census_gate::foreign_host_import_names_a_property_by_the_property_it_reads
    - raf_census::test_crate::census_gate::foreign_host_import_names_a_class_by_its_js_class_or_constructed_type
    - raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package
    - raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis
    - raf_census::test_crate::census_gate::configured_provider_checks_every_context_of_a_shared_consumer_body
    - cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets
  examples:
    - raf_census::example_crate::providers
depends: [cap:census-normal-form]
---

# Built-in census detectors

## Purpose

Fingerprints find copies of a body that already exists elsewhere. Some duplicated
constructs are better recognised by what they do: a hex-digit table, a hand-rolled
ceiling division, a default-hasher map, a declaration that imports a host function.
Detectors are generic checks for these recurring constructs, and they report as
`rule` fingerprints that a ledger job can forbid.

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

### RDF structure policies
<!-- raf:tested-by raf_census::test_crate::census_gate::structure_detectors_judge_calls_against_the_jobs_declared_entry_points,raf_census::test_crate::census_gate::structure_variants_require_the_job_scoped_detector -->

`reifier-default-graph` and `rdf-list-walk` judge a function against the home and
entry points of the job that selects them. A job selects a detector to forbid
matching code or to validate a variant. Both detectors require a job and read
shipping code only.

`reifier-default-graph` reports a call, or a method call, that reaches the home or
an entry point of the job, in a function that handles named graphs, when it
builds a row for the default graph and the row is a reifier row.

A four-argument row is built for the default graph when the final argument
spells `None`, `DefaultGraph`, `DEFAULT_GRAPH`, or a call of `default_graph` or
`default` with no arguments. A three-argument row uses the default graph unless
its receiver selects a graph or a call directly stores it in a graph.

The storage call has a name containing the word `add`, `insert`, `push`, `store`
or `write`. It takes the row alongside a graph argument, in either order, or
takes the row as its only argument on a graph-selecting receiver. A graph argument
names a variable or field spelled `g`, `name` or with the word `graph`, a variable
declared with a `GraphName` or `NamedGraph` type spelling, a call such as
`GraphName::Named(g)` with the word `graph` in its path, or `Some(graph)`. A
default-graph spelling stays a default graph. The graph applies only to the
directly stored row. References, parentheses, `?`, `unwrap` and `expect`
can wrap the row or graph without losing that association.

A receiver selects a graph when it calls a method named for a `graph` with one
non-default argument, such as `dataset.graph_mut(name)`. A local initialized
from such a call or another selected local carries the selection in its lexical
scope until shadowed or reassigned, including in a destructuring assignment.
A possible reassignment of an incoming handle also applies to later loop
iterations. A terminal `break` limits the repeated region to one pass when
its body and condition have no possible `continue`. A branch that ends in an
unlabeled `break` to that loop and has no possible `continue` contributes its
writes to the loop exit. An assignment excluded by `cfg(any())` contributes
no write. A closure or async block used directly as the initializer of a
wildcard or terminal `let` drops its writes when the initializer is discarded.
These written-source rules derive the graph from recognizable storage syntax
and binding declarations.

A row is a reifier row when its predicate is `rdf:reifies`, spelled as `RDF_REIFIES`,
as `REIFIES` under a path segment named `rdf` (including an imported `REIFIES`,
resolved through a declaration-preserving `use` chain), as `vocab::REIFIES` or
as the literal IRI, or when its object is a triple term. The predicate can sit
inside a recognized IRI constructor such as `Term::iri` or `builder.intern_iri`.
A triple-term object is a call, method call or struct whose last name is
`Triple`, `TripleTerm`,
`triple`, `triple_term`, `new_triple`, `from_triple` or `intern_triple`.
The same value-preserving wrappers accepted for graph operands also work for
reifier predicates and triple terms.

A function handles named graphs when a type written in it or its enclosing impl
has the word `dataset`, `datasets`, `quad` or `quads`, or when an identifier
has the words `named` and `graph`, or `graph` and `name`, in that order. Words
are the parts of an identifier split at underscores and capital letters
(`Dataset`, `QuadStore`, `GraphName`, `named_graph`). Dataset code can name its
graph `name` or `g` and obtain its evidence through the written type. Other
words that happen to contain `graph`, such as `paragraph` and `graphql`, give
no named-graph evidence.

A row with another predicate and an object other than a triple term passes
this rule. Explicit graphs, calls outside the job's declared entry points and
functions without named-graph evidence also pass.

`rdf-list-walk` reports a function that names `rdf:first` and `rdf:rest` (as
`RDF_FIRST` and `RDF_REST`, as `FIRST` and `REST` under a path segment named
`rdf` after declaration-preserving `use` resolution, or as the literal IRIs),
contains a `for`, `while` or `loop` expression or calls itself, and calls
neither the home nor any entry point of the job. A recursive call resolves to
the function or method being examined. Calls and loops inside a closure or
async block supply no evidence of delegation or iteration for the enclosing
function. Delegating to a
declared entry point, naming only one term or naming both without iteration
leaves the function clean.

The native source index matches calls by definition identity on each shipping
reach. It reads direct function and associated paths, including indexed module
imports, reexports and direct block imports with established lookup context.
Local callable bindings shadow function names. A dot call can establish an
inherent method when the receiver is a direct `self` in an inherent method or
a binding with a written nominal type, and its first receiver candidate matches
the method's plain `self`, `&self` or `&mut self` exactly. The definition is an
inherent impl at the indexed crate root. Attributes on the binding and
definition must preserve their declarations. An uncertain declaration shadows
any earlier receiver evidence for its name. Calls, loops and RDF terms under
attributes that may alter their source supply no evidence. Imports whose
attributes may change their bindings supply no identity. RDF term lookup skips
imports and lexical shadows excluded by `cfg(any())`. Block-level call lookup
skips those absent bindings as well. When an absent module-level binding would
obscure a single live rooted import, call lookup reads the target of that
import. The lookup through a
written type uses its declaration scope. Inherent precedence over trait
candidates requires one nominal type and one exact inherent definition.
Receiver adjustments, type aliases, dynamic dispatch and macro expansion
supply no dot call identity. Reference-qualified self types and unsupported
associated alias projections supply no call identity. Use an explicit
associated path when the receiver would need an adjustment. Every definition
a call may resolve to in that reach must be declared for the call to count as
delegation. The job must name homes and entry points indexed in the workspace
that is being checked. Both detectors
name the home as the replacement.

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
<!-- raf:tested-by raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence,raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope,raf_census::test_crate::census_gate::foreign_host_import_reports_only_repeats_of_the_homes_host_function,raf_census::test_crate::census_gate::foreign_host_import_refuses_method_constructor_and_property_shapes_beside_a_valid_neighbour,raf_census::test_crate::census_gate::foreign_host_import_refuses_arguments_it_cannot_place_beside_a_valid_neighbour,raf_census::test_crate::census_gate::foreign_host_import_names_a_property_by_the_property_it_reads,raf_census::test_crate::census_gate::foreign_host_import_names_a_class_by_its_js_class_or_constructed_type -->

Job-dependent evidence is isolated to the job that names its detector.
Home-literal evidence includes empty ordinary strings and requires at least
one home literal. Vocabulary evidence uses constants named `NS` or ending in
`_NS` under the home or entry points. Each candidate must have one nonempty
ordinary string initializer, optionally parenthesized or inside a single-value
block; computed or unavailable values are errors. A literal that is an
embedded document is not a vocabulary hit: it contains a line feed and, as a
whole word in any ASCII letter case, one of the keywords `PREFIX`, `BASE`,
`SELECT`, `WHERE`, `CONSTRUCT`, `ASK`, `DESCRIBE`, `INSERT` or `DELETE`. A
word is a maximal run of alphanumeric characters and underscores, so a keyword
inside a longer word such as `PREFIXED` or `select_all` does not count.
A single-line literal remains a hit whatever words it contains, as does a
multi-line literal without a listed keyword. Sole-name includes test
targets in the home package and exempts the actual home and entry-point units.

### Enum catch-all arms
<!-- raf:tested-by raf_census::test_crate::census_gate::enum_arms::enum_catch_alls_share_the_required_forbidden_and_ratcheted_routes,raf_census::test_crate::census_gate::enum_arms::enum_catch_alls_keep_only_arms_proved_across_shared_reaches,raf_census::test_crate::reference_scan::enum_catch_alls_prove_variants_from_a_selected_dependency -->

The `enum-catch-all` detector reports the source location of each unguarded
top-level wildcard or bare binding in a match whose other arms establish an
indexed enum. It checks qualified variant paths and resolved imports against the
enum's declared member names and forms at the actual source reach. A physically
shared arm reports once when each reading context establishes the role.

A variant pattern supplies enum evidence through its own head. Wildcards inside
tuple or record fields leave that arm outside the catch-all inventory. An
or-pattern supplies enum evidence when all its alternatives resolve to variants
of the same enum. Parentheses preserve the role of the enclosed pattern.
A bare identifier denoting a constant or variant leaves the arm outside the
inventory. A guard limits its arm and prevents the catch-all role.

The detector requires an indexed variant witness. Opaque imports, unresolved
aliases, and arm or match attributes that can change configuration supply no
such witness. Selected reference roots and dependency sources can supply enum
identities. Only selected first-party targets contribute arm locations. The
matcher charges source and retained evidence to the shared matching grant.

Enable the detector as a required rule, forbid its rule fingerprint in an
enforced job, or count it with the job ratchet. The detector observes matching
arms within the home package too.

### Complete fixture coverage
<!-- raf:tested-by raf_census::detectors::tests::every_executable_detector_has_positive_and_negative_gate_fixtures -->

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
<!-- raf:tested-by raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases,raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries,raf_census::test_crate::census_gate::fact_detectors_reuse_collected_values_and_keep_their_target_scope,raf_census::test_crate::census_gate::hex_detectors_apply_to_their_declared_targets_and_actual_format_fields,raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence,raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope,raf_census::test_crate::census_gate::ratchet::ratcheted_rules_record_counted_sites_and_hold_both_directions -->

The census ships twenty-one built-in detectors. Each reports hits as `rule:<id>`, where
`<id>` is the detector id, and a ledger job names the hit in
`forbiddenFingerprint`. A job may also name a detector in `ratchetedRule`, so that
its hits are counted sites in the debt record instead of findings (see the
[debt ratchet](./census-debt-ratchet.md)). Every detector except `home-literal`,
`vocabulary-literal`, `sole-name`, `foreign-host-import`, `reifier-default-graph` and
`rdf-list-walk` may be ratcheted; those six judge one job's home and context and the
ledger refuses to ratchet them. A
syntax detector's site uses the byte of its hit; the fact detectors `hex-table` and
`json-pointer-escape` state a line, and their site uses the start of that line
inside the owning item.

| Id | Reads | Meaning |
| --- | --- | --- |
| `std-default-hasher` | all | A std `HashMap` or `HashSet` left on its random default hasher. |
| `foreign-host-import` | all | A function an `extern` block imports from the host that the home or an entry point also imports, declared outside the home and entry points. |
| `hex-format-loop` | all | A `{:02x}` or `{:02X}` format inside a loop or closure. |
| `hex-pair-radix` | all | `u8::from_str_radix(_, 16)` used to decode a hex pair. |
| `hex-table` | all | A hex-digit table written by hand, subject to the job's home policy. |
| `hex-digit-radix` | shipping | A single hex digit decoded through a radix conversion. |
| `home-literal` | shipping | A string literal equal to one the home item's own bodies spell, outside the home item. |
| `sole-name` | all | A function inside the home crate that a `forbiddenName` expression matches, other than the home or an entry point. |
| `vocabulary-literal` | shipping | A string literal equal to or starting with a namespace constant declared under the job's home and entry-point modules, outside those modules, unless it is an embedded multi-line query or data document. |
| `json-pointer-escape` | shipping | Hand-written escaping of `~` and `/` for a JSON Pointer. |
| `length-prefix-frame` | shipping | A hand-written length-prefixed frame read or write. |
| `le-slice-int` | shipping | An integer assembled from a little-endian byte slice by hand. |
| `align-up-mask` | shipping | Alignment rounding written as an add-and-mask. |
| `div-ceil-by-hand` | shipping | Ceiling division written as an add-and-divide instead of `div_ceil`. |
| `xor-first-mismatch` | shipping | A first-mismatch search written as an XOR and a bit scan. |
| `serde-json` | all | A written external JSON dependency path or alias. |
| `rdf-text-writer` | all | A formatting macro whose literal spells the recognized RDF row shape. |
| `rdf-term-model` | all | A local IRI, blank-node and literal enum without a triple-term alternative. |
| `reifier-default-graph` | shipping | A reifier row built for the default graph through the home or an entry point, in a function that handles named graphs. |
| `rdf-list-walk` | shipping | A function that names `rdf:first` and `rdf:rest`, iterates or recurses, and calls neither the home nor an entry point. |
| `enum-catch-all` | all | An unguarded top-level wildcard or bare binding in a match over an indexed enum. |

The `home-literal`, `sole-name`, `vocabulary-literal` and `foreign-host-import`
detectors apply to jobs that name them in `forbidden`. The census walks shipping
code and every test, bench and example target. The `Reads` column of
the table names what each detector reads: `shipping` for shipping code only, and `all`
for shipping code and every test, bench and example target. The eleven `shipping`
detectors are `hex-digit-radix`, `json-pointer-escape`, `length-prefix-frame`,
`le-slice-int`, `align-up-mask`, `div-ceil-by-hand`, `xor-first-mismatch`,
`home-literal`, `vocabulary-literal`, `reifier-default-graph` and `rdf-list-walk`.
Each marks itself as shipping-only in its report. The ten
`all` detectors also include the three representation policies alongside
`std-default-hasher`, `foreign-host-import`, `hex-format-loop`, `hex-pair-radix`,
`hex-table`, `sole-name` and `enum-catch-all`. Apart from `sole-name`, `foreign-host-import`,
`enum-catch-all` and the three representation policies,
a detector exempts the home package of an ordinary job.
A preferred-provider declaration removes this package exemption; only the actual
home and entry-point items retain their classification as homes.

A `foreign-host-import` job needs a home or entry point that declares a host
function import in an `extern` block; without one the Gate check fails. An import is
identified by its namespace, each entry carrying the kind of the argument that gave
it, and by its imported name. The kinds are `module`, `raw_module`, `inline_js`,
`js_class` and `js_namespace` (also written `static_method_of`) of `wasm_bindgen`,
the receiver type of a `method` import without a `js_class`, the returned class of a
`constructor` import without a `js_class`, `wasm_import_module` and `name` of `link`,
and `indexing_getter`, `indexing_setter` and `indexing_deleter`. The name is
`js_name` or `link_name`, else the declared name, except that a `constructor`
import with neither is named `constructor` whatever its Rust name. For a `getter` or
`setter` the property is the attribute's own value, then `js_name` or `link_name`,
then the declared name (without a `set_` prefix for a setter), and a read and a
write are different imports. The same value under two kinds is two namespaces.

Arguments of `wasm_bindgen` that do not select a host function, such as `catch` and
`variadic`, are ignored, as are `kind`, `modifiers`, `import_name_type` and the
parenthesized `cfg(..)` predicate of `link`. Attributes inside `cfg_attr` are read.
The `wasm_bindgen` attribute is read by its last path segment, so a re-exported
`wasm_bindgen::prelude::wasm_bindgen` counts; `link`, `link_name` and `cfg_attr` are
read by their bare names only.

The detector refuses, and the Gate check fails with the file and line, when:

- an argument is not one it can place, or is written with a path such as
  `wasm_bindgen::catch`;
- an argument that needs a value has none, or its value is not a string, a path or a
  list of paths;
- an argument other than the `cfg(..)` of `link` is followed by a parenthesized list;
- one import is both a `getter` and a `setter`, or repeats either;
- a `method` import has no typed first parameter, or its receiver is not a named type;
- a `constructor` import has no return type, or its return type is not a named type;
- `js_namespace` is written on an `extern` block and again on an item in it: current
  releases of the binding attribute refuse the combination, and code written for
  release 0.2.125 or earlier, where the item's value replaced the block's, is refused
  too; or
- a macro invocation stands inside an `extern` block, since it may declare imports
  the detector cannot read.

A getter with a setter, and a repeated getter or setter, are refused although the
binding attribute accepts them with a warning: which of them it keeps is its own
rule, and the detector would otherwise name the import by a choice it does not make.

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

Select built-in requirements directly with `Gate::require_rules` and install
custom rules through `Gate::rules`. The native CLI evaluates its built-in Gate
without starting repository targets; package/example selectors are refused.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries,cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->


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
A `vocabulary-literal` job whose home declares `NS` as `"urn:demo:"` reports
`"urn:demo:term"`, `"urn:demo:term WHERE"` and a two-line literal whose second
line is `PREFIXED reselect select_all ask1`; it leaves a two-line literal
starting with the namespace and continuing `select ?s where {?s ?p ?o}` clean.
A `foreign-host-import` job whose home module imports `now` from the `clock` module
through `#[wasm_bindgen(module = "clock")] extern "C"` reports another `extern`
block, in any package or target, that imports `now` from `clock`, however the
local function is called, including through `js_name` and `cfg_attr`. It leaves an
import of `later`, an import of `now` from another module, from `raw_module` or from
`js_namespace = clock`, a getter of `now` and the home itself clean. A home whose
`js_namespace = Date` import is named `now` matches
`#[wasm_bindgen(static_method_of = Date)] fn now()` and not a `method` import of
`now`. A home without a host import, and an argument such as `module = 1` or an
unknown name each fail the Gate check.
<!-- raf:tested-by raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence,raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope,raf_census::test_crate::census_gate::foreign_host_import_reports_only_repeats_of_the_homes_host_function,raf_census::test_crate::census_gate::foreign_host_import_refuses_method_constructor_and_property_shapes_beside_a_valid_neighbour,raf_census::test_crate::census_gate::foreign_host_import_refuses_arguments_it_cannot_place_beside_a_valid_neighbour,raf_census::test_crate::census_gate::foreign_host_import_names_a_property_by_the_property_it_reads,raf_census::test_crate::census_gate::foreign_host_import_names_a_class_by_its_js_class_or_constructed_type -->

A preferred-provider adapter can return the result of its admitted native
encoder while another helper in the same package spells a hex-digit table.
Selecting `hex-table` still reports that competing helper. A discarded native
call inside the helper cannot exempt it; a neighbouring nonmatching byte string
remains clean. Only proved native consumers and approved API definitions receive
home or entry-point exemptions.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package,raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis -->

A direct `Gate::workspace(root).require_rules(["serde-json"])` check applies the
required built-in without a repository target. The native CLI likewise reads
its own Gate and refuses package/example selectors instead of compiling that
repository's source to obtain detector implementations.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries,cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

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
- `foreign-host-import` reads the attributes written on the `extern` block and its
  functions, directly or inside `cfg_attr`, and `wasm_bindgen` through a path by its last
  segment. `link`, `link_name` and `cfg_attr` under a path are another crate's attributes
  and are not read. A `use` alias that renames an attribute is not read. An `extern`
  block produced by a macro expansion is not read, and a macro invocation written
  inside an `extern` block is refused.
  A `js_class` is its own namespace kind: the same import with and without it, or beside
  a `method` receiver of the same type, is two host functions.
  <!-- raf:tested-by raf_census::test_crate::census_gate::foreign_host_import_reports_only_repeats_of_the_homes_host_function,raf_census::test_crate::census_gate::foreign_host_import_refuses_method_constructor_and_property_shapes_beside_a_valid_neighbour,raf_census::test_crate::census_gate::foreign_host_import_refuses_arguments_it_cannot_place_beside_a_valid_neighbour,raf_census::test_crate::census_gate::foreign_host_import_names_a_property_by_the_property_it_reads,raf_census::test_crate::census_gate::foreign_host_import_names_a_class_by_its_js_class_or_constructed_type -->
- Detectors work on syntax. They do not resolve types, so `std-default-hasher`
  recognises the std paths and aliases it can see in the file.
  <!-- raf:tested-by raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases -->
- A detector reports hits; a required-rule selection makes eligible hits hard
  findings, while other hits become findings through the ledger job that forbids
  them.
  <!-- raf:tested-by raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence,raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope,raf_census::test_crate::census_gate::foreign_host_import_reports_only_repeats_of_the_homes_host_function,raf_census::test_crate::census_gate::foreign_host_import_refuses_method_constructor_and_property_shapes_beside_a_valid_neighbour,raf_census::test_crate::census_gate::foreign_host_import_refuses_arguments_it_cannot_place_beside_a_valid_neighbour,raf_census::test_crate::census_gate::foreign_host_import_names_a_property_by_the_property_it_reads,raf_census::test_crate::census_gate::foreign_host_import_names_a_class_by_its_js_class_or_constructed_type -->

## Explanation

These detectors recognise recurring byte-level encodings, integer layout
operations, literals, hash-collection defaults, host function imports, enum
catch-all arms, RDF representation syntax and hand-written RDF structure.
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

`vocabulary-literal` targets a single term spelled by hand outside its home.
A fixed query or a small data document embedded in code writes namespace IRIs
as part of a larger text, and reporting each one would push a consumer to
sanction every document or drop the detector. Requiring both a line feed and a
whole-word keyword keeps the exemption narrow: the keyword set is the minimal
Turtle, TriG and SPARQL vocabulary that opens or shapes such a document, and it
grows only for a stated need.

## Verification

The existing detector Gate case covers all three policy families with an
empty-job ledger, ordinary/test contexts, macro declarations and native extern
aliases. Positive and negative fixtures participate in the existing executable
registry completeness check. The real self-Census selects these same rule IDs.

The registry check compares executable built-in ids against the fixture
families consumed by the behavioral Gate tests. Missing or duplicate families,
unregistered ids and missing positive or negative fixture sets fail the check.
The behavioral Gate cases consume the registered built-ins directly and pair
required findings with neighbouring clean source.
<!-- raf:tested-by raf_census::detectors::tests::every_executable_detector_has_positive_and_negative_gate_fixtures,raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

- Each detector has a positive fixture that yields exactly one hit with the right
  `rule:<id>`, and a negative fixture that yields none.
- A fixture shows that a detector other than `sole-name` and `foreign-host-import`
  reports no hit inside the home package of an ordinary job that forbids it, except
  the three representation policies, which do not exempt home packages. A second
  module of the home package that repeats the home's host import is reported.
- A preferred-provider fixture reports unlisted competing functions and structs
  in the adapter package, including a helper containing a discarded native call,
  while accepting a native helper and a neighbouring nonmatching table.
- A fixture shows `sole-name` reports a matching function inside the home crate that
  is neither the home item nor an entry point, and reports neither of those two.
- A fixture per `shipping` detector shows it flags a positive source in shipping
  code and is silent on the same source in a test target, and a fixture per `all`
  detector shows it flags a positive source in both.
- `reifier-default-graph` fixtures flag a default-graph row with `rdf:reifies` as a
  constant, as the literal IRI and through an import, and a triple-term object.
  Default rows remain visible beside unrelated arguments, in nested calls and
  after graph handles are shadowed, go out of scope or are reassigned. Fixtures
  leave clean rows with explicit graphs and rows with reifier terms only in the
  subject, as well as undeclared callees and functions without named-graph
  evidence. `rdf-list-walk` fixtures
  flag a loop and a recursion over the two terms, as constants and as literal IRIs,
  and leave clean the same loop delegating to a declared entry point, one term alone
  and both terms without iteration. Neither is reported in a test target, and a
  job that does not declare the entry point sees the delegating function reported.
- A fixture shows `home-literal`, `sole-name`, `vocabulary-literal` and
  `foreign-host-import` are silent for a job that does not name them.
- `foreign-host-import` fixtures flag a repeated host function in another package,
  another module of the home package and a test target. They leave clean a different
  function, the same name in another namespace, the home itself and an import with no
  namespace, including the same value under another namespace kind. A static method
  matches the namespace form of the same function, and a method receiver does not.
  Two spellings of one property, by `js_name` and by the attribute's own value,
  match, while two `js_name` values on one declared name do not.
- A home without an import, and an unplaceable argument including a `js_name` with no
  value, each fail the Gate check beside a valid neighbour. Each refusal listed in the
  contract section fails beside a corrected neighbour that is accepted: a getter with
  a setter, a repeated getter, a malformed `method` receiver, a `constructor` without
  a named return type, a path-qualified argument, a namespace on both block and item,
  and a macro in a block. A `link` with a `cfg(..)` predicate is accepted.
- Two constructors of one class match under any Rust names, and constructors of
  different classes do not. Methods of one `js_class` match across Rust receiver types,
  and methods of different classes do not.
- `vocabulary-literal` fixtures flag a single-line literal with and without a
  keyword, a multi-line literal without a keyword and one whose keywords appear
  only inside longer words, and leave a multi-line literal with a lowercase
  whole-word keyword clean.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets` | test | [Usage](#usage), [Examples](#examples) |
| `raf_census::detectors::tests::every_executable_detector_has_positive_and_negative_gate_fixtures` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::example_crate::providers` | example target | [Usage](#usage) |
| `raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::configured_provider_checks_every_context_of_a_shared_consumer_body` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::configured_provider_detects_unlisted_competitors_in_the_adapter_package` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
| `raf_census::test_crate::census_gate::configured_provider_homes_exempt_only_proved_consumers_and_apis` | test | [Contract](#contract), [Examples](#examples) |
| `raf_census::test_crate::census_gate::contextual_literal_detectors_use_only_their_jobs_home_evidence` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::default_hasher_detector_uses_visible_std_paths_and_lexical_aliases` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::enum_arms::enum_catch_alls_keep_only_arms_proved_across_shared_reaches` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::enum_arms::enum_catch_alls_share_the_required_forbidden_and_ratcheted_routes` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::fact_detectors_reuse_collected_values_and_keep_their_target_scope` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::foreign_host_import_names_a_class_by_its_js_class_or_constructed_type` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::foreign_host_import_names_a_property_by_the_property_it_reads` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::foreign_host_import_refuses_arguments_it_cannot_place_beside_a_valid_neighbour` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::foreign_host_import_refuses_method_constructor_and_property_shapes_beside_a_valid_neighbour` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::foreign_host_import_reports_only_repeats_of_the_homes_host_function` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::hex_detectors_apply_to_their_declared_targets_and_actual_format_fields` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::layout_findings_require_related_operands_and_rendered_prefix_order` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::ratchet::ratcheted_rules_record_counted_sites_and_hold_both_directions` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::structure_detectors_judge_calls_against_the_jobs_declared_entry_points` | test | [Contract](#contract) |
| `raf_census::test_crate::census_gate::structure_variants_require_the_job_scoped_detector` | test | [Contract](#contract) |
| `raf_census::test_crate::reference_scan::enum_catch_alls_prove_variants_from_a_selected_dependency` | test | [Contract](#contract) |
<!-- raf:generated:end -->
