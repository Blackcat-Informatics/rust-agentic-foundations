<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Concrete PurRDF model and writer reuse

Source of truth: clean `/home/paudley/Active/purrdf` main at
`5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`, workspace version 3.0.1. Read its
AGENTS policy and relevant manifests/source. Registry 3.0.1 is comparison only:
it is not API-identical to this current main. No product edits, builds, tests,
posting or wasm execution were performed. This is source compatibility analysis,
not execution proof or delivery acceptance.

## Replacements available now

| RAF surface | Actual PurRDF owner and replacement | What RAF must still own |
| --- | --- | --- |
| `raf-rdf/src/term.rs` Atom/Object/Statement tree | `purrdf_core::TermValue` (`rdf-core/src/ir/term.rs:408`) and `TermBox` replace the parallel IRI/literal/triple-term algebra. `TermValue::iri`, `simple_literal`, `typed_literal`, `Triple { s, p, o }` carry exact lexical data and RDF 1.2 triple terms. Owned `RdfTerm`/`RdfTriple` also exist, but TermValue has the reusable traversal/writing API needed below. | Validate the closed report subset: owned relative subjects, absolute predicates/datatypes, no blanks/language/direction, depth-one quoted statements, primitive canonical lexical policy. Native term constructors are carriers, not RAF profile validators. |
| RAF quoted-term writer | `purrdf_core::write_term_value` (`term_writer.rs:39`) writes canonical `<<( s p o )>>`, IRIs and literals to generic `TextOut`, without materializing a rendered string per term. It already delegates leaf spelling to `purrdf_lex::term_syntax`. | Snapshot BASE expansion of owned relative IRIs is a report law. For normalized relative terms the native writer is directly usable. |
| RAF traversal with BASE expansion | `TermValue::try_write_nested` (`ir/term_walk.rs:438`) supplies the existing iterative triple walk plus `leaf(out, &TermValue)` and text callbacks; `try_write_nested_with_memory` (`:467`) retains typed before-growth admission. Use delimiters `"<<( "`, `" "`, `" )>>"`. In the leaf callback delegate literals to native `write_term_value` or lexical `write_literal`; append BASE and local IRI through existing `iri_escape::push_escaped`. | Keep only the application-specific local-IRI remapping callback. This avoids cloning the term tree, rebuilding the graph, constructing expanded IRI Strings, or inventing another recursive term walker. |
| External BLAKE3 dependency in RAF writer | `purrdf_hash::blake3::hash(bytes).as_bytes()` plus `purrdf_hash::hex::Lower`/`encode` supplies existing native BLAKE3-256 and lowercase hex. `Hasher::update/finalize` is available for streaming. | Hash exactly the current preimage; replacing the algorithm implementation must not alter VERSION/header/body framing or use PurRDF's other content-id schemes. Exact vectors need execution before acceptance. |
| Copied W3C constants | `purrdf_iri::vocab::rdf::{NS,TYPE,REIFIES}` and `purrdf_xsd::datatype::{XSD_NS,XSD_STRING,XSD_INTEGER,XSD_BOOLEAN}` are the existing vocabulary homes. RAF REPORT and BASE namespaces remain caller-owned. | Do not put RAF's application namespace into PurRDF. |

The smallest coherent term change is to expose native TermValue to the real
builders and remove RAF's second term enum/quoted-statement representation,
rather than convert a complete RAF graph to PurRDF immediately before rendering.
Use a thin profile triple/graph wrapper over native terms with validated
insertion, owned-reference checks and the existing report ordering law. A plain
BTreeSet remains standard-library storage; the wrapper's useful responsibility
is the report's stricter invariant, not a general RDF model.

## Native graph APIs and concrete incompatibilities

Current main provides graph-free `TripleDatasetBuilder` and `TripleDataset`
(`ir/builder/triple.rs:82`, `ir/triple.rs:285`), smaller than using named-graph
slots for this profile. Actual ingress methods include `intern_iri`,
`intern_literal`, `intern_triple`, `push_triple`, their admitted counterparts,
and `freeze`; frozen `resolve` and `triples` borrow terms/compact IDs. The
general `RdfDatasetBuilder::{intern_iri,intern_triple,push_quad_with_handle,freeze}`
and `turtle::{write_dataset_term,write_dataset_quad}` are also real APIs.

These are useful native graph replacements for **absolute RDF graphs**, but
cannot be substituted blindly for RAF's normalized identity representation:

1. Native freeze requires every IRI absolute before other validation
   (`ir/validate.rs:290-293`). RAF's identity preimage intentionally contains
   `<>` and `<#fragment>` before the digest supplies BASE. Resolving them first
   changes the hash law; disabling native validation is not acceptable.
2. Builder duplicate insertion collapses rows. RAF `Graph::insert` must refuse
   a duplicate. General `push_quad_with_handle` returns the existing row handle
   for duplicates (`builder.rs:2600-2625`); preserve explicit duplicate detection
   at RAF's boundary instead of inheriting silent deduplication. The compact
   builder's public `push_triple` returns no inserted/existing flag.
3. Frozen row order is local-ID order, not RAF's lexical term order. TermIds
   are dataset-local (`ir/term.rs:36`), so neither their numeric order nor their
   bytes belong in a persistent identity.
4. `TermValue::Ord` is also **not** RAF's ordering: its literals compare
   datatype, language, lexical form, direction (`ir/term_walk.rs:1195-1265`),
   whereas RAF Atom literals currently compare lexical form then datatype.
   A literal `"2"^^xsd:integer` and `"true"^^xsd:boolean` demonstrate the different
   precedence. Keep an explicit borrowed profile comparator over native terms;
   do not replace it with native Ord or rendered-string comparison.

Therefore use native **terms/traversal now**, while retaining a narrow report
graph/profile adapter. A later direct frozen-payload representation would need
an explicit ownership/remapping design, not a fake absolute placeholder that
can collide with caller-provided external IRIs. No new RDF algebra is needed
to retain the current profile, and these compatibility facts do not justify
keeping duplicate term constructors/walkers indefinitely.

`turtle_render::render(dataset, prefixes)` is not the exact writer replacement:
its inspected source rebuilds per-subject/property indexes and content keys,
abbreviates prefixes, groups predicates, uses native literal forms and formats
collections/blanks. Those are legitimate canonical-Turtle choices, but their
bytes are different from RAF's full-IRI one-triple-per-line identity body. Use
the native canonical **term** writer, not this different graph presentation.

## Exact identity and domain builders

Current `raf-rdf/src/write.rs` hashes UTF-8 `VERSION "1.2"\n` followed by its
ordered relative-IRI body with BLAKE3, then forms the fixed BASE plus all 64
lowercase hex digits. Turtle adds BASE after VERSION; N-Triples expands owned
IRIs using the same digest, including owned positions inside triple terms.
All of this can remain unchanged while term storage/traversal is delegated.
Do not use RDFC canonicalization, PurRDF TermId/content IDs, shorter hashes or
`turtle_render` output as the preimage.

The real command builders (`cargo-workplan/src/result_graph.rs:18-47`) and
library projections (`raf-workplan/src/report.rs:214-250`) currently construct
RAF Atom/Iri/Triple directly. Change these production helpers to construct
native terms at insertion, preserving source field order/absence/provenance.
Do not add a second complete converted graph at render time. Numeric/boolean
canonical validation remains a report-construction responsibility.

Census has another full intermediate representation:
`report/shape.rs:12-20` Term plus nested Graph, then
`report/ground.rs::encode` walks it into raf-rdf, cloning each subject/predicate
per outgoing statement. Its decoder already consumes the ground graph and
moves lexical strings into the schema index. Replace generic shape terms with
native terms and share the profile insertion path; preserve the domain shape
index where it actually validates report classes/properties. Do not erase the
closed census validation or confuse that index with a competing RDF kernel.

## Allocation, dependency and portability limits

Native borrowed writers avoid graph/term clones. `TermValue::Clone` still copies
strings/boxes and uses workspace for triple terms (`term_walk.rs:896-922`), so
“native” does not make rebuilding free. Native `Ord` walks nested pairs and can
allocate workspace; the explicit depth-one profile comparator avoids both the
ordering mismatch and unnecessary whole-term materialization. Generic native
traversal has its own work-list storage; use its admitted API if accounting is
required, rather than claiming zero allocation from a borrowed signature.

The smaller crates are concrete choices already partly in use: lex depends on
hash only; iri depends on lex/hash; hash has zero runtime dependencies. Core
adds cdt, xsd, events, deflate, hashbrown, sha2, regex and regex-syntax in its
current manifest. Core is an rlib kernel without PyO3 and the inspected writer
and model paths use no native filesystem/process service. This source supports
a wasm-compatible design; no wasm build or full resolved dependency audit was
executed here. Core/iri carry Unicode licensing, so the current RAF dependency
policy must be checked for the newly selected core closure rather than assuming
the existing iri exception covers it.

Current main and packaged 3.0.1 differ despite the same version: the registry's
builder `intern_iri` returns TermId where main returns Result; the new compact
TripleDatasetBuilder/admitted writer APIs are not all present there. Pin a
concrete available source/revision or compatible published surface when making
the real dependency change. Do not cite current-main APIs as proof that a plain
registry `3.0.1` dependency compiles.

Before acceptance, run real production callers, exact current Turtle/NT/golden
snapshot pins, duplicate/invalid/depth/owned-reference refusal neighbors, literal
ordering neighbors and the required native/wasm gates. Supplied prior logs and
source comments are evidence of prior work only; this audit executed none of
those checks.

## Verified direct native-value parser projection

Further source inspection found the already supplied direct-value route:
`RdfDataset::flat_default_graph_quads()` (`ir/dataset.rs:2097`). It yields
`QuadValues { s: TermValue, p: TermValue, o: TermValue, g: None }`, chaining base
quads, `reifier_quads()` and `annotation_quads()`. It restores the same flat
assertion layers as `purrdf_rdf::flat_rdf_quads`, including
`<reifier> rdf:reifies <<( s p o )>>`, while constructing native TermValue
directly through the dataset's own `term_value` API. Move those native values
into the report graph; do not first construct owned RdfTerm and then borrow it
through `TermValue::from_rdf_term`, which copies its strings again.

This API intentionally drops graph slots. The closed default-graph report must
first reject `dataset.named_graphs().next().is_some()`; that native inventory
includes explicitly declared empty graphs. This is a projection of an already
parsed/frozen dataset, not a streaming parser claim. The ID-native `emit`
visitor exists but separates quad/reifier/annotation callbacks and is
infallible; it offers no smaller route than this existing direct flat-value API.
Any earlier recommendation of the owned flat stream plus borrowed conversion
is superseded by this source-verified path. No execution was run for this note.

## Subsequent authorized native-order cutover

The owner has since confirmed that native TermValue ordering intentionally
replaces the earlier uncommitted custom ordering. The compatibility observations
above remain factual, but the recommendation to retain a custom comparator is
superseded: use native ordering, update the uncommitted fixtures and their full
snapshot identities coherently, and verify both carriers against the same law.
Do not preserve a second generic comparator merely to retain a self-authored
uncommitted profile. The BASE namespace, complete BLAKE3 digest, exact VERSION
preimage framing, relative ownership and RDF 1.2 statement semantics still need
their actual current contract tests.
