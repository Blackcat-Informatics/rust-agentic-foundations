# Native RDF storage and text audit

Read-only inspection of RAF donor source and exact PurRDF revision `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43` using `git show`/`git grep`. The local upstream checkout has advanced to `6f1199b1dfb2ca3e800db7fbc45fd9aefe372c6c`; conclusions below use the pinned tree, not that head. No product changes, new checkout or builds. The architectural graph was not rebuilt; direct source is the evidence.

## Concrete finding

`raf-rdf/src/graph.rs` still owns generic graph infrastructure: nested BTreeMaps, object-vector insertion/binary search, duplicate detection, flattened triple iteration and storage-derived subject membership. Native terms alone do not discharge the user's preference. The unlanded public `Nodes`, `Properties`, `from_nodes`, `into_nodes` and `TripleRef` APIs expose that competing storage and should change rather than become a compatibility constraint.

## Selected authoritative storage

Use the native `RdfDataset` storage family. Construction/mutation belongs to `purrdf_core::MutableDataset`, with a native frozen `Arc<RdfDataset>` base. A parsed report retains its parser-returned native dataset; it must not drain `flat_default_graph_quads()` into another RAF graph. Public report wrappers retain only that native handle and closed-profile metadata (selected owned root/namespace), not a parallel assertion map. Builders/frozen snapshots are native phase transitions, not independently maintained RAF storage.

Pinned source evidence:

- `crates/rdf-core/src/ir/mutable.rs:348`: `MutableDataset::new` takes a native DatasetHandle base. At 506, `insert_record(&RecordValues)` runs native record validation, inserts into the requested physical table and returns `bool` membership change. At 1373, `freeze` returns `Arc<RdfDataset>`; at 1441, `snapshot_view` exposes a native delta view. Native error propagation must stay typed/actionable.
- `dataset_view.rs:1763`: `DatasetMut::insert` returns false for an existing assertion; translate false to the report's duplicate refusal. Its error contract checks absolute IRIs, so retain profile role validation (or the stronger native `insert_record` route), not an assumption that this method alone enforces every profile role. Do not use `push_quad`'s silent deduplication as a report duplicate detector.
- `ir/builder.rs:945`: native `intern_value_with_memory` ingests TermValue through its existing interner. `push_quad_with_handle` at 2610 returns the existing handle on a duplicate; ordinary builder push is set insertion, not refusal. At 2877, freeze validates and seals native tables. The source explicitly states frozen sorting is by local IDs, not lexical order.
- `ir/dataset.rs:1706`: `resolve` returns borrowed TermRef; `term_id_by_iri`, `term_id_by_value`, native pattern cursors and DatasetView queries support domain reads without rebuilding a map. `reifier_quads` at 1907 and `annotation_quads` at 2079 restore the named statement layer as ID-native assertions. Iterate all three layers when implementing a flat report view; ordinary quads alone lose provenance.

Keep RDF physical roles explicit. `insert_record` preserves a chosen Ordinary/Reifier/Annotation role; `DatasetMut::insert` may infer statement roles. A report graph is the flat RDF assertion meaning, while a native dataset also distinguishes these physical records. Do not replace the current graph equality assertion with raw ID or physical-table equality. Compare the specified flat graph, and preserve complete named reifier assertions.

## Native-backed Census domain view

Replace Census shape.rs's `type Graph = raf_rdf::Nodes` and BTreeMap mutation with a domain read view over native IDs/TermRefs and the selected owned-root spelling. Closed shape lookup asks native pattern/query APIs for subject/predicate matches; a tiny profile adapter must include virtual reifier and annotation rows too. Check cardinality by actual returned values and retain all existing class/reference/name/job/copy/fingerprint/start-byte invariants. The view's presentation may expose root as empty and fragments as `#...` without rewriting stored strings. Owned-reference membership must query asserted subjects, not merely find an interned term: native dictionaries can contain unused/object-only IRIs.

The native IR refuses relative IRIs (`builder.rs:2536`, mutable absolute checks). Therefore the report's local `<>`/`<#fragment>` values become a profile boundary, not native storage. Construct using one explicit absolute internal owner; parse using the selected actual snapshot owner. Normalize only that exact owner and validated fragments in the domain/output view. A fixed construction owner must be disjoint from allowed external terms or refused explicitly; otherwise two distinct external and owned IRIs could alias. Do not hide this identity design decision in a generic converter or use textual replacement over serialized RDF/literal strings.

## Ordering and output: native ID order is not the report order

Retain the single normalized Turtle-body preimage and explicit profile ordering unless deliberately revising the unlanded report layout. Native `TermValue::Ord` remains the object comparator (`ir/term_walk.rs:1204`); do not reproduce its comparator. Native table order is not a substitute: IDs depend on ingestion history. An operation-local ordered selector of native assertion IDs, with normalized native term keys cached once where needed, is appropriate profile output scratch. It is not another authoritative graph. Release it after use; no per-comparison lexical cloning or second nested-map materialization. Subject/predicate keys use the documented normalized spelling, with the root before fragments.

Continue native `write_term_value_with_memory` / native nested term emission through the profile's exact owner expansion. The profile wrapper still owns VERSION/BASE placement, delimiters between assertions, exact canonical acceptance and hashing of the one preimage. These are the actual RAF serialization contract, not a reason to retain generic storage/insertion.

Two additional native whole-output paths exist, but neither is a byte-compatible drop-in:

- `purrdf_core::turtle_render::render` (`src/turtle_render.rs:56`) accepts a native RdfDataset, sorts by semantic lexical/content keys and renders native canonical Turtle. It emits `a`, abbreviations/native scalar spellings and grouped Turtle; it takes no report-owned root/base mapping callback. It folds graph slots, so named graph refusal remains mandatory. Replacing the report writer wholesale requires a stated canonical profile/identity migration, not pretending existing bytes match.
- `try_canonicalize_flat_view` (`ir/canon.rs:773`) directly canonicalizes native views without re-freezing; `Canonicalized.nquads` is bytewise-sorted, deduplicated, LF-terminated output. FlatAssertion presentation preserves real reifier/annotation assertions instead of overlay sentinels. It is useful native graph-equivalence/canonical graph infrastructure, but its sorted absolute N-Quads bytes are not the normalized relative Turtle preimage or root-first report order. The convenience rdf wrapper can panic on native refusal; use the typed fallible core API when refusal belongs in the public error path. Do not silently replace the agreed identity law with RDFC hashing or DatasetStateDigest.

## Exact text replacement found

The manual `split_inclusive` first-differing-line loop in `raf-rdf/src/read.rs` has an actual compatible native seam:

1. `purrdf_deflate::common_prefix_len(canonical.as_bytes(), text.as_bytes())` returns the exact first mismatching byte, or the shorter length. It is public/reexported at pinned `crates/deflate/src/lib.rs:64`; implementation `kernels.rs:98` explicitly handles prefixes/unequal lengths without scanning beyond either slice.
2. `purrdf_iri::LineIndex::new(text).try_locate(text, offset)` supplies the line. It counts LF with native scan helpers, retains CR in bytes and maps EOF/multibyte offsets safely. Use checked conversion for public usize line; propagate its real bounded-column refusal rather than introduce fallback. This adds the pinned purrdf-deflate dependency (runtime hash-only closure), not testkit golden machinery.

Retain meaningful CRLF, missing-final-LF, extra-line and non-ASCII mismatch controls. Identical bytes need no error lookup. The native testkit golden mismatch implementation is private and performs file/regeneration policy; it is not a compatible production replacement.

## Tests/API migration

Replace unlanded map-shape tests (empty subject/property rows and into_nodes equality) with native-backed public construction/role/duplicate/refusal tests. Empty graph/root absence remains meaningful; an empty map row is no longer representable input. Real native view tests must cover an unused interned root/object-only reference, parsed statement layer, duplicate insertion false and missing asserted subject, plus unchanged ordinary neighbor. Test ingest-order permutations explicitly; no TermId ordering assumption.

Keep byte/profile tests explicit: both formats share one digest, canonical CR/final-LF handling, exact own-root/prefix boundaries, unknown profile/IRI roles/language metadata/depth refusals and Census full fingerprints/start-byte uniqueness. If profile bytes change, update report layout/version, both goldens and parser pins coherently; NF1 hash input bytes must remain unchanged. The native storage cutover itself must not introduce a second converted graph or a general RDF parser claim. Actual focused callers, fresh lifecycle and full gates remain future execution in the parent's selected issue 40 checkout.
