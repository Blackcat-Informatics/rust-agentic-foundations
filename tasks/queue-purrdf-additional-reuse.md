<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Additional concrete PurRDF reuse

Read-only bounded audit of active RAF work and current PurRDF main
`5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`. No builds or product/PurRDF mutations.
Exact actionable findings were sent to root (resolver/SCC/admission owner) and
strict_inventory (issue 16 owner). Source inspected here does not constitute a
passing execution claim. Current main APIs must not be assumed present in the
published registry package merely because both report version 3.0.1.

## Concrete replacements for the strict owner

| Current RAF implementation | Existing native replacement | Preservation requirement |
| --- | --- | --- |
| `raf-census/src/normalize/fingerprint.rs::digest` external blake3 Hasher; fingerprint abbreviation/display/parsing and `report/write.rs` hex conversions | `purrdf_hash::blake3::Hasher::{new,update,finalize}`/`hash`; `hex::Lower`, `encode_to_slice`, `decode_32_canonical`, `Digest32` | Keep exact token framing: **u32** little-endian byte length followed by UTF-8. PurRDF `frame_le` uses **u64**, so it is not a drop-in and would change the normal form. Keep full 32 bytes/64 hex identity and abbreviation as presentation only. Preserve Width versus Hex domain errors when replacing parser internals. |
| `raf-workplan/src/digest.rs::Display`, handwritten nibble/parser and strict claim-key hex loop | `purrdf_hash::hex::{Lower,decode_32_canonical,encode_into}`; optionally wrap `Digest32` as the domain digest | `decode_32` and `ContentDigest::from_hex` accept either case, so they are wrong for the closed lowercase record language. Keep existing ParseDigestError shape/message rather than leaking a new permissive reader. `Lower` writes without allocating an intermediate string. |
| `raf-workplan/src/graph.rs::strongly_connected` | `purrdf_core::graph::tarjan_scc` or admitted `tarjan_scc_with_memory` | Native component/member order is not its stable API. Keep RAF's explicit ascending member/component sorting. Singleton SCCs need the existing self-edge check to distinguish self-cycle from an isolated node. Preserve findings' file/member order. |
| `raf-workplan/src/coverage/lines.rs::times` handwritten digit multiplication | `purrdf_xsd::bigint::BigInt::{from_decimal_digits,from_u128,mul}` with native Display; `compare_products` can avoid product strings if the surrounding comparison is changed coherently | The current policy is exact arbitrary decimal arithmetic. Keep long fractional tails, canonical comparisons, zero and usize::MAX neighbors; do not substitute floating point or bounded u128 products. Use a proven parser invariant or a typed failure, not default text on a failed conversion. This is a concrete supplied arithmetic kernel, not a proposal to replace the whole Number profile parser. |

## SHA-256: actual supplied API and its limit

RAF `digest.rs` contains a complete handwritten SHA-256 state/compressor;
`strict/claims.rs::constraint_unit` uses it for a one-shot folded paragraph.
PurRDF's actual public `purrdf_core::ContentDigest::of(&[u8])`
(`rdf-core/src/content_store.rs:35-44`) computes SHA-256 through `sha2`, with
`as_bytes`, native hex display and `to_hex`. It can replace the one-shot clause
hash immediately without a new cryptographic implementation. The clause-key
truncation length is a published key law and must stay unchanged.

PurRDF hash deliberately implements no SHA-2. Its core itself imports
`sha2::{Digest,Sha256}` for incremental content folds. There is no inspected
public streaming SHA-256 owner in PurRDF to replace RAF's multi-field/module
stream directly. Do not change those SHA-256 identities to BLAKE3, nor quietly
buffer entire module source preimages merely to call the one-shot wrapper.
The smallest stream-preserving removal of RAF's copied compressor uses the same
`sha2::Sha256` that PurRDF selects; a shared upstream streaming exposure is an
alternative if approved and implemented there. Exact RAF kind/NUL/name/decimal
length/NUL/text framing remains application code; it is not PurRDF's u64 frame.

## Concrete resolver replacements for root

Issue 28 `settle.rs::{read_components,components}` still contains iterative
Tarjan. Replace its algorithm with
`purrdf_core::graph::tarjan_scc_with_memory(adjacency, memory)`; preserve RAF's
separate Work charge and error site. If only a component id per node is needed,
the real `scc_component_index_with_memory` accepts generic adjacency rows and
guarantees reverse-topological component ids. `tarjan_scc` only documents a
partition and determinism, so do not accidentally promote its current emission
order to an API guarantee. Verify the resolver's dependency-first component
processing against actual native output/explicit ordering before removal.

`purrdf_lex::allocation::Memory` supplies the actual admitted vector/string/box
growth operations now duplicated by manual graph/scratch reservation helpers:
`reserve`, `reserve_for_push`, `push`, `collect`, `sort_by`, `boxed`, `boxed_slice`,
`reserve_string`, `push_str`, `release_vec`, `release_string`, `scope` and
`try_scope`. Use native construction under the original RAF account; do not
allocate with the resident path and measure/adopt it afterward.

The inspected allocation module supplies no generic admitted BTreeMap insertion
primitive. Therefore its vector/graph admission APIs do not by themselves
replace all existing RAF BTreeMap node-layout bounds. Do not advertise that
conversion as complete or delete necessary map admissions without an actual
native owner for that allocation.

## Admission integration contract and pitfalls

`Admission::resize(live_bytes)` (`lex/src/allocation.rs:272`) receives an
**absolute native live total**, not an increment. A RAF adapter needs its own
native-total field and must adjust only the delta in the shared GlobBytes meter;
unrelated tables/query/names/pending-row leases must remain live and additive.
Calling old `Transient::charge(total)` on every resize cumulatively overcharges;
setting the meter's entire total to the native total erases overlapping owners.

`StorageError::AdmissionFailed` intentionally contains no domain error. When
the RAF meter refuses growth, retain the **first** original LoadError with its
Budget/file/line in the adapter and return AdmissionFailed to native code. At the
operation boundary recover that exact error. A later refund callback must not
overwrite it. AllocationFailed, SizeOverflow, Capacity and InvalidEdge are
different failures; do not relabel all of them as the caller's byte budget.
They must produce actionable typed failures rather than a silent empty SCC.

Native `Memory::reserve` admits the new buffer while the old one is still live,
allocates fallibly, moves contents, drops the old allocation, then shrinks the
grant. Those overlapping capacities are real and must stay charged. It checks
the requested typed Layout and the resulting exact capacity before use.
`release_vec` destroys its buffer before refund. `release_vec_after` preserves
an existing operation failure; a refund failure only replaces success.
`scope`/`try_scope` release failed local buffers after destruction, preserve the
first physical/domain error and retain the grant if a sticky callback refuses
shrink.

Memory is an accounting view, not an RAII owner: it has no automatic drop refund.
Successful returned SCC/components retain their grant until the result is
destroyed. Keep the adapter/outer lease alive across all consumers, then drop
the result before refund/drop of the owning lease. Do not refund at function
return while a cached/result Vec still survives. `Memory::resume` assumes that
the supplied buffers already belong to that same original grant; it neither
admits them nor licenses adopting arbitrary resident buffers.

Native SCC validates adjacency before allocation and exposes GraphError's
InvalidEdge versus Storage variants. It does not expose a per-edge Work
callback; byte admission does not replace algorithmic Work accounting. Count
the actual bounded native traversal/validation/component processing separately
at RAF's agreed work units, with checked arithmetic. Do not call the resident
`tarjan_scc` then add a retrospective storage estimate.

## Deliberate non-replacements

RAF's terminal control/bidi display escaping is not JSON/RDF literal escaping:
its `\\u{...}` spelling and allowed newline/tab policy differ. The inspected
native JSON/literal escapers are not a byte-compatible substitute. Census Rust
string decoding already delegates to syn; RDF lexical escape decoding already
uses PurRDF. No duplicate replacement is asserted for those paths. This audit
does not claim an exhaustive repository helper census.

All proposed replacements still require actual caller compilation, exact
identity/order/refusal neighbors and mandatory gates. Source comments, retained
prior logs and this map do not substitute for those executions.
