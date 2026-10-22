<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native APIs for the locked provider foundation

Read-only extension of queue-provider-prerequisite-review.md. All upstream API
observations below use git show/git grep against the exact commit
5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43 in the local PurRDF repository, not an
assumed registry equivalent or moving main. RAF observations use the current
adoption checkout's manifest/load code. No product edits, builds, upstream
changes, new checkout or Graphify rebuild were performed. The foundation remains
planned, not implemented or qualified.

## Decode and identify: preserve the actual missing seams

RAF manifest.rs already decodes with toml::{Table,Value}; raf-source/Cargo.toml
selects the existing toml parser with std/parse/serde. Extend that one source
manifest/lock boundary rather than add a separate Census TOML parser.

The pinned PurRDF helper-census/src/toml.rs is explicitly a strict ledger SUBSET.
Its parse and Value are pub(crate), helper-census/Cargo.toml provides an
unpublished binary and no library target, and the reader explicitly refuses
inline tables, dotted assignment keys, multiline strings, dates and floats.
Cargo manifests already use inline dependency tables and dotted inheritance.
This is neither accessible nor semantically compatible. Its source.rs manifest
walk is similarly a private census consumer, not a public Cargo dependency API.
Its policy.rs::no_features runs Cargo metadata and parses JSON, which also
conflicts with RAF's explicit source-only owner requirement. Do not transplant
either implementation or replace full TOML with JSON/YAML decoding.

Bounded pinned-source searches for toml, semver, VersionReq, lockfile,
Cargo.lock and package identity found no public Cargo manifest/lock/package-id,
SemVer requirement evaluator or dependency solver. RDF-CAPI's ABI version
constants and mapping-package identities are different contracts. Native XSD
numeric readers do not parse Cargo versions/requirements: signs, leading zeros,
prerelease ordering, build metadata and requirement operators need their actual
grammar. Do not write a dotted-integer comparator, prefix-match a requirement or
invent a version solver. Consume exact locked selections; any required version
grammar provider still needs a deliberate compatible dependency decision.

Current RAF manifest dependency records retain alias/kind/path/workspace
inheritance, but not locked versions, git/registry identity, optional features or
structured target predicates. package_info/dependency_info link only already
discovered workspace packages; discovery rejects duplicate package names. These
are domain-model and shared-resolver changes, not missing generic parsing
algorithms supplied by a differently named native crate.

## Compatible public primitives: use these for the actual new operation

| Foundation operation | Exact pinned API and ownership boundary |
| --- | --- |
| Admitted arrays, strings and growth | purrdf_lex::allocation::{Admission,Memory,StorageError,growth_capacity}; Memory::reserve, reserve_for_push, reserve_string, resume and release methods. reserve requests an exact extent; use its native geometric push/growth operation for repeated appends. Resume the same original live grant, include coexisting old/new buffers and separately admit owned payloads. Preserve RAF cumulative ItemBytes versus transient GlobBytes semantics and first typed failure. |
| Private ordered identity/edge indexes | allocation::OrderedTable<K,V>: get/get_mut, borrowed get_by/get_key_value_by, reserve_insert/insert, storage_bytes, ordered and values_mut. This is an admitted slot-arena AVL implementation, so do not implement another tree. Insert replaces an existing key and returns its old value: domain duplicate rules must explicitly distinguish deduplication from an invalid conflicting identity. Native try_clone requires shallow immutable K/V clones and does not supply general deep infallible Clone/Eq contracts. |
| Source/dependency DFS frontier | purrdf_lex::walk::WorkList<T,N>::try_push_admitted, heap_capacity, pop, iter, len/is_empty. The exact pin ALSO exposes top_mut at walk.rs:306, reverse_top and set. Thus a blanket claim that this pin lacks a mutable-top API is incorrect; assess a concrete continuation against these actual operations. Retain declared traversal order and admit spills before mutation. No public native FIFO was identified in this bounded inspection; a FIFO cannot silently become LIFO. |
| Fallible ordering/comparison | walk::{try_sort_unstable_by,try_equal_range_by,try_dedup_by,try_partition_unique_by}, re-exported from walk/sort.rs. Reuse when comparison itself must propagate work/resource refusal; keep the required identity ordering and duplicate semantics in the domain comparator. |
| Short pair lists | purrdf_lex::assoc::{get,get_mut,get_sorted,try_insert}. Its ordinary list semantics are FIRST-match/replace-or-append; that does not validate duplicate Cargo keys or prove unique package selection. try_insert delegates allocation to the caller's fallible reserve seam. |
| Dependency SCCs, if the actual domain operation requires them | purrdf_core::graph::tarjan_scc_with_memory(adjacency,&mut Memory) returns typed GraphError, validates dense edges before allocation and is iterative. Caller owns package-to-dense-id mapping, target/kind edges, result order and domain meaning of a cycle. A component alone is not a Cargo validity decision. Do not duplicate Tarjan or infer public topological/member order. |
| Ephemeral hash lookup | purrdf_hash::fixed::FixedState is a BuildHasher for standard HashMap/HashSet. Its documented contract is trusted/ordinary data with no chosen-collision resistance; do not remove an adversarial-input guarantee by applying it indiscriminately to package/source strings. It is build-dependent and must never become a persisted package/source identifier. Numeric/internal indexes are a direct compatible case. |
| Hex syntax and bytes | purrdf_hash::hex::{decode_32,decode_32_canonical,decode_to_slice,nibble,nibble_canonical,encode,encode_to_slice}. Use the needed exact case/width contract for checksum/revision text. A valid digest spelling does not verify an archive, prove a git object exists or authenticate a provider source. |
| Newly defined cache/evidence fingerprints | purrdf_hash::blake3::{hash,Hasher}, Domain and frame::{frame_le,try_frame_le_with,frame_le_into,frame_le_with}. Reuse actual digest and injective framing instead of hand-coded length-prefix or hex loops; define the source-domain schema explicitly. These cannot substitute BLAKE3 for a Cargo checksum or change existing identity preimages. |
| Raw SHA-256 of bounded bytes | purrdf_core::ContentDigest::of(bytes), as_bytes, from_raw and from_hex are PUBLIC, re-exported by rdf-core/lib.rs. content_store.rs:39 hashes bytes directly with SHA-256, with no domain prefix. This is compatible with computing an archive checksum from already-held bounded bytes; purrdf_hash alone has no SHA-256 API. ContentDigest::from_hex accepts either case, so separate canonical spelling policy when needed. No generic incremental SHA-256 interface is exposed by this type; do not require buffering an unbounded archive or claim an incremental API exists. |

Native Shared/SharedText/SharedOwned in allocation/shared.rs can supply fallible
immutable shared ownership when that is actually required. try_clone is fallible
and admitted constructors have owner/lifetime requirements. Reuse RAF's existing
interner/SharedStr for source names where applicable; do not create a competing
interner or silently alter public string Clone/Ord/equality semantics.

## Source and catalog boundaries that are not supplied by a similar name

RAF Tree/RelPath, verified DiskTree/MemoryTree, workspace_of/member caches and the
shared extern-root resolver are the concrete source-load seams to reuse. No
public pinned PurRDF Cargo cache locator, verified locked source catalog, git
checkout verifier or registry acquisition API was identified. Actual source
provenance and revision/checksum matching remain required domain behavior; native
hash/hex/path helpers are not evidence that this behavior already exists.

purrdf_iri::{parse,parse_uri,Iri,BaseIri} provides RFC IRI/URI validation and
reference operations. Use it only for a genuine URI subfield or report-IRI
operation. Cargo source IDs such as git+URL?query#revision carry Cargo-specific
identity semantics; Iri::normalize is not permission to canonicalize their
authoritative lock spelling, resolve filesystem paths or establish containment.

purrdf_slice::{SliceCatalog,dependency_closure,link_units,source_unit_key} takes
RDF slice catalogs, ontology ownership edges and its own content-address schema.
It does not accept arbitrary Cargo packages/lock edges. Its public catalog name
does not make it a compatible Cargo catalog, and its RDF identity framing must
not become Cargo package equality. The generic SCC operation it uses is already
public separately. artifact::Identity likewise hashes its own labelled canonical
component encoding; it is not Cargo PackageId or the raw archive checksum.

testkit::paths::workspace_root_from searches host ancestors for a literal
[workspace] line and collapses read errors into non-matches. It is a test-fixture
helper, not a substitute for RAF's containment-aware, full-TOML discovery/error
contract. Native testkit::text::nested and temp/allocation fixtures remain usable
for compatible new fixture construction, not production source authority.

## Next implementation boundary and qualification limits

Keep TOML decoding in the existing raf-source parser. Add only the required
Cargo-domain identity/source/edge fields there, reuse the shared resolver for
actual symbol paths, and use the native storage/traversal/graph/digest helpers
above wherever that operation is present. Do not introduce a second resolver,
hand-written TOML/SemVer engine, private upstream code copy or source-name guess.

The prior review's locked-identity, alias/shadow/version, real source integrity,
target-context, actual input/result delegation, dummy-call and exact-budget
negative/valid fixtures remain necessary. This API audit ran none of them and
does not establish Cargo semantics, complete provider enforcement, source
portability or complete native adoption. It records exact available operations
and missing seams before implementation; no dependency or upstream change is
requested by this artifact.

This is the final write to the issue 53 stage for this audit. No owned process
is active, and no further stage writes will occur after handback/archive freeze.
