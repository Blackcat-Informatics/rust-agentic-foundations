<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Focused native RDF cutover review

Independent read-only source review while strict_inventory owns ongoing edits,
compilation and tests. No product mutations, builds or test executions by this
reviewer. Reviewed the current raf-rdf term/graph/read/write/lib/vocabulary,
actual census report shape/read/write, library report projections, CLI graph
helper ingress and pinned dependency declarations. No full adoption, registry,
wasm or gate acceptance is claimed.

## Finding reported and rechecked

Initial `Graph::from_nodes` validated keys only while visiting object values.
It retained empty property/subject containers, so an accepted Graph could hold
invisible data that vanished on serialization and canonical reparse. A valid
root plus `bad subject -> { bad predicate -> [] }` was a concrete source witness.
The owner was notified immediately.

Current `graph.rs::from_nodes` now refuses an empty subject property map and an
empty property value list before iteration/sorting. This closes the reported
defect in source while preserving the in-place native storage adoption. Public
refusal/valid-neighbor execution was not observed here and remains part of the
owner's qualification. No other concrete blocking defect found in this bounded
read after that repair.

## Native terms, roles and ordering

`raf-rdf` reexports native TermValue/TermBox. The old Atom/Object/Statement/Iri
algebra is removed from production src. The remaining Triple is a report-ingress
wrapper with closed subject/predicate/object roles, not another generic nested
RDF term representation. Graph stores native objects in the same Nodes/Properties
map representation now used by census validation. It exposes borrowed triples
without cloning subjects/predicates and transfers consumed Nodes to the domain
validator without rebuilding a second graph.

All native carrier variants are checked at insertion/adoption/output boundaries:
owned report subject; absolute IRI predicate; native IRI or no-language literal
object; no blanks; no language/direction datatypes without metadata; and at most
one quoted triple with named subject, absolute predicate and atomic object.
Native IRI component parsing is retained without normalization. Native integer
value parsing plus canonical_lexical replaces the handwritten decimal grammar.
The graph root and every owned ordinary/quoted reference are checked before
output/snapshot calculation. External IRI objects remain allowed.

Current subject/predicate order is BTree lexical order; objects use native
TermValue order via binary_search/sort_unstable. This is the **authorized native
order cutover**, not a defect for failing to preserve the earlier uncommitted
custom literal order. Initial model/writer audit preservation advice has been
explicitly superseded. Updated fixtures must pin the resulting complete graph
and snapshot consistently; no parallel generic comparator is warranted.

## Production parser and statement restoration

`read.rs` now calls pinned `purrdf_rdf::parse_dataset_with` for the selected
media type. It verifies the chosen BASE/root digest namespace and canonical
lowercase full hash, rejects any native named_graphs entry, then moves
`flat_default_graph_quads`' native TermValue s/p/o into the report wrapper.

The native flat-value method's inspected source chains base, reifier_quads and
annotation_quads. It restores `<reifier> rdf:reifies <<( s p o )>>` rather than
losing folded statements. It creates TermValue directly from native DatasetView
instead of owned RdfTerm followed by another borrowed conversion/clone. The
named-graph rejection is necessary because this API deliberately drops graph
slots, and native named_graphs includes declared empty graphs.

Local ownership normalization only strips the selected root's exact boundary:
root equality or a following `#`. A suffix without `#` and another snapshot's
fragment remain external. Quoted subject/object ownership is normalized too;
predicates/datatypes remain absolute and nested triple terms are subsequently
refused. Empty fragment refusal is explicit. This is a closed profile check
after a native parser, not a streaming parser or a new generic syntax walker.

Native deduplication does not authorize duplicate input spellings: final exact
canonical re-render comparison rejects alternate, repeated or additional bytes
even if the native IR collapsed an equivalent input row. Direct Graph insertion
and adopted Nodes also refuse actual duplicate assertions. Native parsing,
domain-role refusal, digest mismatch and noncanonical bytes remain separate
error paths; no fallback accepts a failed parse.

## Writer and identity law

Normalized objects use `write_term_value_with_memory`; expanded output uses
native `try_write_nested_with_memory` and only the application-specific local
IRI BASE callback. Literal spelling and triple-term syntax/traversal therefore
have native owners. No generic escape decoder, quoted-term walker or hash
implementation remains in raf-rdf/src. Its full-IRI assertion framing, report
ownership, VERSION/BASE directives and exact canonical check are domain policy.

Snapshot identity still hashes UTF-8 `VERSION "1.2"\n` plus the native-ordered
relative body, with native BLAKE3 and lowercase hex and the entire digest in the
configured BASE namespace. Turtle adds BASE after VERSION. N-Triples expands
ordinary and quoted owned positions using that same digest. There is no
TermId-derived identity, hash shortening, RDFC substitution or rebuild of an
expanded term graph. Generic native traversal may allocate working storage;
the writer uses Resident memory and does not claim application-budget admission
or zero allocation.

## Actual downstream ingress and remaining execution

CLI result_graph and raf-workplan report helpers now construct TermValue
directly at real Graph insertion. Native quoted provenance is retained through
TermBox rather than flattened to messages. Census aliases its domain Term to
TermValue and its shape Graph/Node to raf-rdf Nodes/Properties; its old generic
ground encode/decode module is removed. Census writing validates actual domain
shapes, then transfers that same storage through Graph::from_nodes; reading
transfers native parsed storage back through into_nodes before shape validation.
Domain numeric ranges, class/property restrictions, missing/repeated fields and
closed report semantics remain consumer responsibilities, rather than being
silently delegated to the permissive RDF carrier.

At the time of this read, some raf-rdf examples/profile tests and one CLI test
still referenced removed old term APIs; the owner is actively migrating and
compiling them. This is unfinished validation, not evidence of a passing cutover
or a reason to reinstate obsolete compatibility types. Do not claim completion
until all real callers and required gates pass.

Needed execution includes exact both-carrier/native-order and full digest pins,
direct Nodes empty/duplicate/role refusal neighbors, reifier/annotation
restoration, named/empty named graph refusal, boundary ownership, foreign
subjects, invalid IRI/literal positions and over-depth terms, plus actual census,
library and CLI reports. Earlier pre-cutover passing logs do not validate the
new public native ingress. No broader Stage review or unconditional acceptance
is inferred from this focused review.
