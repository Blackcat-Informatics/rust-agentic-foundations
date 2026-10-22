<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Reuse PurRDF's production text parser

Source of truth: clean `/home/paudley/Active/purrdf`, branch `main`, head
`5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`, inspected read-only. Its workspace
version remains 3.0.1, but its source has changed since the installed registry
package; that version string does not establish source equivalence. No builds,
dependency changes, product edits or posting were performed for this audit.

## Concrete replacement

The complete public Turtle/N-Triples parser lives in **purrdf-rdf**, under
`crates/rdf/src/native_codecs/{parse,text_parse}.rs`. The smaller purrdf-gts
crate owns the binary GTS container, not a public RDF text parser. purrdf-core's
`turtle` module is an emitter. purrdf-lex supplies lexical primitives but not
the complete Turtle/N-Triples grammar. There is no existing parser-only Cargo
feature in the current purrdf-rdf manifest.

The facade re-exports the required core model, visitor, parser request and
backend traits. RAF can therefore use one direct parser dependency rather
than adding a direct purrdf-core edge solely for those names.

Replace RAF's `read.rs::Parser` cursor, quoted-string decoding, term grammar
and statement loop with `purrdf_rdf::parse_dataset_with(bytes, media, None,
&ParseOptions::default())`. Select exactly `text/turtle` or
`application/n-triples`. This returns `ParseOutcome` with `dataset` and the
parser's actual `document_base`; no caller BASE is needed. The smaller call
`parse_dataset(bytes, media, None)` returns `Arc<RdfDataset>` if directive
state is not needed. Retain RAF's profile conversion, validation, digest and
exact canonical byte comparison. Do not copy PurRDF's private text parser or
use a SPARQL token stream as a second RDF grammar.

First reject any `dataset.named_graphs()` entry, including an empty declared
graph. Then use current main's `dataset.flat_default_graph_quads()`, which
directly yields `QuadValues` with native `TermValue` s/p/o fields. It includes
base quads, reifiers restored as `rdf:reifies` triple-term rows, and annotation
rows without an intermediate quad vector or conversion through owned RdfTerm.
The named-graph refusal must precede this iterator because it intentionally
drops graph slots. Reading only `dataset.quads()` would lose the statement-layer
assertions that the parser folds out. Move the native terms into RAF's validated
model; preserve lexical forms without canonicalizing numeric values or language
metadata away. This supersedes the earlier owned `flat_rdf_quads` conversion
recommendation and avoids its second string clone.

## Events, visitors and actual streaming

`RdfDataset::emit(&mut impl RdfDatasetVisitor)` is allocation-free output of
the already-frozen dataset; its callbacks are infallible and term-before-use.
It is not a parser accepting a caller's incremental visitor.

`GtsCodecBackend` implements `RdfParserBackend::parse_into(RdfParseRequest,
&mut impl RdfEventSink)`. The event sink is fallible, but the implementation
first calls `parse_dataset`, then `FrozenDatasetSource::drive`. The source
explicitly documents **replay, not streaming emission**. A custom RAF sink
therefore does not avoid materializing the PurRDF dataset.

`parse_dataset_from_reader<R: Read>` streams physical source lines for
N-Triples/N-Quads/HexTuples; it buffers Turtle before parsing. Its documented
line-oriented peak is one line + graph + dataset, not constant memory. RAF's
existing input is already `&str`, so that seam does not remove its input buffer.

## RAF validation that remains necessary

PurRDF accepts general RDF 1.2. RAF must still enforce its narrower report
contract after parsing:

- Default graph only; no blank nodes, language/direction metadata, nested
  triple terms, triple-term subjects or unsupported term positions. A quoted
  object is depth one with IRI subject/predicate and atomic IRI/literal object.
- Exactly the snapshot-owned top-level subjects, valid ASCII fragment profile,
  required root and no dangling owned references; absolute predicates and
  datatypes. Continue refusing metadata-free `rdf:langString`/`dirLangString`.
- Recover the checked snapshot base. Turtle can use `document_base` with its
  actual directive provenance; N-Triples has no BASE, so select the unique
  report-root IRI matching RAF's namespace + 64 lowercase hex digits. Exact
  canonical rendering still enforces that root's first statement/order.
- PurRDF resolves Turtle relative IRIs. Normalize exact base and `base#fragment`
  back to RAF-owned nodes for both formats, only in the existing subject/object
  roles. A shared textual prefix is insufficient. Do not normalize predicates
  or datatype IRIs into owned nodes.
- Recompute RAF's BLAKE3 snapshot from its declared preimage, compare the full
  digest, render with RAF's canonical writer and require byte equality with
  the input, including directives, order, escaping, duplicates, CR and final LF.
  This rejects otherwise legal prefixes/comments/alternate syntax without
  retaining another RDF parser. If PurRDF deduplicates a repeated assertion,
  byte comparison still refuses the source, but preserving RAF's exact
  `Repeated` error category needs explicit verification rather than assumption.

The parser does **not** enforce RAF's version header. In current
`text_parse.rs::version_string`, a single-line string is accepted and skipped;
its value is not restricted to `1.2` or returned as version metadata. RAF's
exact canonical writer comparison must continue requiring `VERSION "1.2"`.
The parser's legitimate acceptance of RDF 1.1-compatible constructs does not
authorize accepting another RAF profile version.

## Resource and dependency consequences

The public `ParseOptions` currently contains only `track_source_spans`; neither
`RdfParseRequest` nor the public text parse signatures accept byte, term, work
or allocator admission limits. Internally the parser refuses syntactic nesting
past 128, and the frozen model/event contract bounds triple nesting at 16.
Those are real safety controls, but they do not impose RAF's depth-one profile
or a general peak-memory budget. Post-parse profile validation cannot claim a
pre-allocation bound. A stricter caller-required allocation envelope remains
an explicit upstream API requirement, not grounds for keeping a second grammar.

Current purrdf-rdf dependencies are unconditional: core, GTS, events, hash, IRI,
lex, XSD, Ed25519, SPARQL algebra/eval, regex, hashbrown, rayon and sha2. Thus
`default-features = false` does not create a light parser package. This is the
smallest existing *public complete parser* seam, not a claim about artifact
size, build cost or a minimal possible future crate extraction.

The closure also changes the licence audit: current purrdf-core declares
`(MIT OR Apache-2.0 OR MulanPSL-2.0) AND Unicode-3.0`. RAF's existing narrowly
scoped leaf exceptions do not automatically cover a new core dependency.
Review the resolved graph and notices under the normal root/fuzz deny gates;
no blanket licence exception is implied by parser reuse.

PurRDF explicitly supports wasm32-unknown-unknown: the current `Makefile` wasm
allowlist includes purrdf-rdf and these first-party dependencies; its manifest
documents rayon's sequential fallback on wasm. This is inspected source policy,
not a gate run in this audit. No RAF+PurRDF wasm compile/runtime acceptance or
peak-memory/performance result has been established. Current main also adds
allocation-aware leaf APIs absent from the inspected registry source; a
registry 3.0.1 result must not be presented as testing current main.

## Existing executed evidence

RAF's retained helper at `scratch/rdfprobe/oracle` inside this stage pins registry
`purrdf-rdf` and `purrdf-core` exactly 3.0.1. It actually calls `parse_dataset`
with no caller BASE, selects each real media type, then serializes the parsed
graph to N-Triples. `raw/queue-rdf-oracle-final.log` and the foundation handoff
record successful replay of eight graphs in both formats: 7, 41, 89, 11, 11,
45, 28 and 22 triples. The handoff reports terminal exit 0 and exact graph pins,
with the documented U+0085 serializer spelling adjustment. This is positive
native parsing evidence for the shipped package and existing fixtures. It is
not execution of the proposed production adapter, refusal coverage, a current
PurRDF main build, or wasm acceptance. Those checks remain to be performed
after choosing the intended dependency source.
