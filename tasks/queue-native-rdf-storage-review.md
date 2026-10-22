<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Bounded native RDF storage review

Disposition: no concrete correctness defect found in the reviewed kernel and Census extraction. This is a source review, not lifecycle, fuzz, full-gate or landing acceptance. No product edits or builds were performed by this reviewer.

## Inspected implementation

- `raf-rdf/src/{graph,read,term,write,error}.rs`: parsed Graph retains the original native `Arc<RdfDataset>`; Builder uses native MutableDataset membership and freeze. There is no second retained generic assertion graph. Operation-local subject/predicate sets and output key selectors do not replace the authoritative store.
- Graph subject lookup requires an actual ordinary, reifier or annotation subject row; a dictionary entry used only as an object/predicate does not create a subject. Validation combines those physical layers, rejects overlap, checks the actual root, closed term roles and dangling local references, including quoted endpoints. Exact root and root-fragment normalization avoids textual-prefix ownership collisions. Construction refuses external aliases of its private owner.
- Native RDF 1.2 reifier and annotation assertions participate in both borrowed subject views and flat output. Native `flat_default_graph_quads` is used only after named-graph rejection. The physical-overlap unit has both a refused overlapping store and a valid named-statement neighbor; parsed-statement duplicate tests exercise native base membership rather than a separate RAF membership table.
- Output uses one normalized Turtle body, exact normalized subject/predicate strings and native TermValue object ordering. The same body supplies the full digest for both formats. Native term output, IRI escaping, BLAKE3/hex, common-prefix and source-line primitives remain in the actual production paths. N-Triples expands owned names from that common identity. Ingestion IDs do not select emitted order.
- `raf-census/src/report/{read,shape,write}.rs`: application shape joins/sets remain domain checks, not a generic RDF model. Every actual node/property is checked; required/cardinality, canonical integer/boolean, mint identity, member/reference, orphan, job/reifier correspondence and copy-count checks remain. Writers use the native Builder and then run the same domain readers before returning text. Typed ground errors remain nested sources of Census report errors.
- The report serialization migration is explicit reportForm 2; reportForm 1 and 3 are refused with both declared and implemented versions. Normal-form implementation and full fingerprint inputs are unchanged. An independently declared normalForm is preserved by reading and separately checked by the normal-form policy.

## Final iterator delta

After the frozen execution below, Node::values changed from an allocated vector to a borrowed native iterator. Reviewed the settled body and Census consumers: singleton reads inspect first and second values, maximum/minimum cardinalities count fresh iterators, optional fields take the first value only after complete shape validation, and repeated-property validation still precedes typed output construction. Predicate lookup captures its native ID, so temporary property strings do not escape. No semantic defect found. The prior frozen execution does not qualify this later delta; owner reported final Clippy/execution pending at this review checkpoint.

## Assertion and execution evidence

Read `raw/native-rdf-frozen-tests.log`: Census library 20, forms 27, golden 4, groups 23, page 7, reports 26, units 30; kernel library 2 and public profile 8; Census doctest 1 all passed. Kernel's zero doctests are not an execution proof. The first failed/churn-invalidated run is preserved separately and is not counted as passing evidence.

Inspected actual public profile assertions for reversed ingestion equality in both formats, snapshot parity/change, native duplicate membership, object-only subject absence, depth/role/ownership/dangling refusal with valid neighbors, exact CRLF/final-LF/Unicode diagnostics, typed literal/IRI boundaries, and distinct registry expansion pins. Inspected Census shape-table required-property removal and unknown-property additions against valid graphs, report-form refusal, and cross-normal-form preservation assertions. Registry pins are a distinct revision of the same parser family; they are not an independent parser implementation.

## Limits retained

The public kernel deliberately has no input-byte, allocation or stack ceiling. Native parsing can allocate before semantic/profile rejection, and depth one is not parser admission. The capability states this as a Constraints boundary. Exact canonical acceptance can reject duplicate source spellings after native set normalization; physical-store duplicate overlaps are checked separately. Generic datatype lexical semantics remain a consumer responsibility.

No source review finding requires relaxing these limits or adding another generic parser/model/writer. Fuzz execution, lifecycle/assessment freshness, full mandatory gates and final edited-state execution are outstanding acceptance work outside this disposition. The Workplan/CLI donor is not reviewed or accepted by these results.
