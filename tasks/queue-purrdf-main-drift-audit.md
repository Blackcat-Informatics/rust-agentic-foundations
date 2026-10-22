# Pinned native closure versus current upstream main

Read-only comparison on 2026-10-21. Local HEAD and origin/main both resolve to `6f1199b1dfb2ca3e800db7fbc45fd9aefe372c6c`. Compared against the actual RAF pin `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`; no fetch, pin update, build or product mutation.

## Exact unchanged surfaces

`git diff --name-status 5db00aa..6f1199b1 --` over each of `crates/alloc-probe`, `crates/hash`, `crates/lex`, `crates/xsd`, `crates/rdf`, `crates/rdf-core`, `crates/iri` and `crates/deflate` returns no files. Their source, manifests, tests and public APIs are byte-identical between these revisions. Root Cargo.toml is unchanged too.

Thus current main adds no new APIs or semantic corrections in the shared-native packet's alloc-probe/hash/lex/xsd imports, or the next RDF packet's rdf/core/iri/deflate direct surfaces. All storage, native ordering, absolute-IRI, parser/profile and first-mismatch conclusions in queue-native-rdf-storage-audit.md apply equally to these direct surfaces. No missing native seam was repaired by this advance.

## Important transitive qualification

The statement 'the entire next RDF dependency closure is unchanged' would be false. Pinned `crates/rdf/Cargo.toml` directly depends on `purrdf-sparql-eval`, and current main changes three files there:

- modifier.rs: grouping now consumes native `OrderedRows` through `par_loop_try_map_owned` instead of immediately concatenating a full output Vec.
- parallel.rs: new **crate-private** OrderedRows/OrderedRowIter/WorkerRows ownership adapters preserve worker buffers, source order and checked total lengths; common error selection remains before output reservation. These are not public reusable RAF APIs.
- row_checkpoint.rs: internal admission/commit functions now accept exact-size IntoIterator inputs instead of requiring Vec, enabling that ownership change.

These three files also differ from the parent's previously inspected `68d5549936e83608eda517fde81b2640f318b69c` checkpoint. This changes transitive evaluator implementation/allocation, not any inspected native RDF parse, graph-storage, canonical serializer or scalar public API. RAF's proposed RDF paths do not execute SPARQL grouping, but a repin would still change compiled dependency source and must not inherit a claim of identical closure or qualification without an appropriate settled-tree check.

Root Cargo.lock changes only by adding alloc-probe to columnar's dependency list. The current packet does not need columnar; that lock change is not an RDF parser/API change. Other changed Datalog/shapes/columnar implementations were outside this targeted API review and supply no demonstrated replacement for the reported RAF seams.

## Decision supported by this inspection

Keep the settled RAF pin for current qualification. Record this exact inspected upstream head/limit; do not claim latest-head adoption. No direct imported API benefit justifies interrupting the running full gate. A future coordinated refresh must account for the actual transitive sparql-eval changes, rather than reuse the earlier narrower unchanged-surface statement as a closure-wide guarantee.
