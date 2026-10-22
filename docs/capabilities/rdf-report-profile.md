---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:rdf-report-profile
title: "Canonical native RDF reports"
description: "Closed ground RDF 1.2 reports over native storage with format-independent snapshot identities."
state: delivered
bindings:
  code:
    - raf_rdf::Builder
    - raf_rdf::Graph
    - raf_rdf::Node
    - raf_rdf::Value
    - raf_rdf::Triple
    - raf_rdf::Format
    - raf_rdf::Error
    - raf_rdf::vocab
    - raf_rdf::graph
    - raf_rdf::read
    - raf_rdf::term
    - raf_rdf::write
  tests:
    - raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot
    - raf_rdf::test_crate::profile::duplicate_native_membership_and_real_subject_queries_have_valid_neighbors
    - raf_rdf::test_crate::profile::parsed_statement_duplicates_and_repeated_source_bytes_refuse_with_valid_neighbors
    - raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement
    - raf_rdf::test_crate::profile::roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs
    - raf_rdf::test_crate::profile::absolute_iri_and_literal_profile_refusals_keep_valid_neighbors
    - raf_rdf::test_crate::profile::exact_bytes_include_cr_final_lf_and_multibyte_diagnostic_neighbors
    - raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals
    - raf_rdf::test_crate::profile::pinned_registry_expansions_remain_distinct_from_native_canonical_bytes
    - raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary
  examples:
    - raf_rdf::example_crate::report
depends: []
---

## Purpose

`raf-rdf` owns the closed ground report profile shared with Census. PurRDF owns
RDF storage, mutable membership, terms, parsing, indexed queries, term traversal,
lexical emission, hashing and standard vocabularies. Consumers own their closed
classes, properties, cardinalities and semantic invariants. A valid ground graph
is not proof of a valid Census report.

## Contract

### Native storage and membership

`Graph` retains an immutable native dataset. Reading retains the parser-returned
storage, including its named RDF 1.2 reifier and annotation layers. `Builder`
constructs or edits through native mutable set membership; repeated assertions
are refused. Finishing validates native storage and the ground profile. Native
physical row identities are not the report's flat semantic graph identity.
`Node` and `Value` query borrowed native subjects and terms. A subject must own an
assertion: an unused or object-only interned term is not a subject. No parallel
assertion map or second converted graph is retained for rendering.

### Closed term roles

Subjects are the owned root or nonempty validated ASCII fragments. Predicates
and literal datatypes are absolute IRIs. Objects are named IRIs, non-language
typed literals or depth-one RDF 1.2 triple terms with an atomic object. Every
owned reference, including quoted subject and object references, names an
asserted subject, and the root exists. Blank nodes, graph names, language tags,
directional language metadata and nested triple terms are refused. The
language-only datatypes `rdf:langString` and `rdf:dirLangString` are refused.

Native IRI validation retains exact lexical identity; percent escapes are not
rewritten. Local API names are presented as `""` and `"#fragment"`. Native storage
requires absolute IRIs, so construction expands those names under the private
`urn:raf:report:construction` owner. External aliases of that exact owner and its
fragments are refused during construction. Parsing instead retains the selected
snapshot owner. Only that exact owner and its validated fragments are normalized;
a textual prefix or another digest is not owned.

### One content identity and explicit order

The application vocabulary is `https://blackcatinformatics.ca/raf/report/1#`.
Standard predicates and primitive datatypes come from native RDF and XSD
vocabularies. Domain registries specify the classes and properties they allow.

One normalized spelling supplies the identity of both formats: UTF-8
`VERSION "1.2"` and LF, followed by one full-predicate Turtle assertion per line,
ending with ` .` and LF. Owned names are `<>` or `<#fragment>`. Subjects and
predicates sort by exact normalized string; objects sort by native `TermValue`
ordering. Native dataset IDs never select report order. An operation-local
selector retains native assertion IDs and caches normalized object keys once;
it is discarded after output.

The snapshot BASE is
`https://blackcatinformatics.ca/.well-known/genid/raf-report/` followed by the
full lowercase BLAKE3 digest of those normalized bytes. BASE is excluded from the
preimage. Canonical Turtle inserts `BASE <snapshot>` and LF immediately after the
version line. Canonical N-Triples emits no directives and expands the owned names
with that same snapshot. Both formats denote the same absolute RDF 1.2 graph;
format choice never changes the digest. The root precedes fragments.

### Exact acceptance and domain boundary

Readers use the native parser for the selected media type, reject graph/term
profile violations and compare both the declared identity and exact canonical
bytes. Whitespace, comments, alternative escapes, CRLF and a missing final LF
are not alternative canonical encodings. Refusals carry typed errors with
escaped diagnostic text and the first differing line where applicable. Native
common-prefix and source-line primitives locate that byte difference.

The kernel does not apply domain classes, cardinality or numeric value-space
rules. Census validates all its report shapes and cross-row invariants through
the native-backed views before accepting or emitting a domain result.

## Usage

Construct assertions with `Builder::insert`, then consume the builder with
`finish`. Use `Graph::to_text(Format::Turtle)` or `Format::NTriples` and read with
`Graph::from_text` using the matching format. `Graph::snapshot` returns the common
identity. `Builder::from_graph` edits a native base; `contains` and `remove` use
native membership. `Graph::dataset` exposes the immutable native storage while
`get`, `nodes`, `Node::values` and `Value` expose the report-local presentation.

## Examples

The bound `report` example constructs a root and named source statement, renders
both formats, reads them back and checks their common snapshot. The pinned
profile fixtures retain typed literals, Unicode/control text, a literal
backslash escape and named triple-term provenance.

## Constraints

- This is a closed canonical report profile, not a general RDF parser. Native
  parsing precedes closed-profile rejection.
- The public API has no input-byte, allocation or stack budget. Native parsing
  and resident term output impose no RAF resource ceiling. The depth-one term
  restriction is semantic, not a parser admission bound. Callers admit their
  input before this layer where their own contracts require it.
- The unkeyed content digest is an integrity identity, not authentication. Its
  uniqueness relies on the cryptographic collision assumption.
- A native frozen dataset uses local ID order internally. Flat report meaning
  includes ordinary, reifier and annotation assertions; neither that physical
  order nor physical-table equality replaces the specified report order or
  semantic graph equality.
- Ground acceptance does not validate arbitrary datatype lexical values or
  consumer-specific semantics. Those checks remain the consumer's obligation.

## Explanation

A native store serves parsing, construction, views and output without a second
RAF graph implementation. The report adapters only impose the closed roles,
owned-name law and exact byte profile. A fixed normalized Turtle body avoids a
circular hash and gives both formats one concrete identity. Native semantic
ordering is used through cached output keys rather than inferred from term IDs.

The registry parser pins were produced by a distinct revision of the same
PurRDF implementation family. They are retained expansions, not a claim of an
independently implemented parser. The fixture comparison normalizes the actual
U+0085 scalar to its escaped spelling; a literal backslash-u0085 neighbor remains
separate. Production canonical acceptance is unchanged by that comparison.

## Verification

The bound tests exercise real native construction and parsed storage, reordered
ingestion, native duplicate membership, actual subject queries, named statement
views, role/reference refusals and valid neighbors, exact canonical bytes and
common snapshot identity. The Census report suite supplies actual consumer shape
and value-space checks. The report-read fuzz target exercises both domain readers
and their canonical spelling using committed valid and refused seeds.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_rdf::example_crate::report` | example target | [Verification](#verification) |
| `raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement` | test | [Verification](#verification) |
| `raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::absolute_iri_and_literal_profile_refusals_keep_valid_neighbors` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::duplicate_native_membership_and_real_subject_queries_have_valid_neighbors` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::exact_bytes_include_cr_final_lf_and_multibyte_diagnostic_neighbors` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::parsed_statement_duplicates_and_repeated_source_bytes_refuse_with_valid_neighbors` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::pinned_registry_expansions_remain_distinct_from_native_canonical_bytes` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs` | test | [Verification](#verification) |
<!-- raf:generated:end -->
