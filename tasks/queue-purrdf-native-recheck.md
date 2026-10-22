<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native cutover bounded recheck

Read-only current-source recheck during focused owner lint repairs. No builds,
tests or product edits by this reviewer. The owner reports passing native
profile/census/library/CLI/source checks and ongoing Clippy; those are owner
execution claims, distinct from the assertion bodies inspected here. No full
gate, wasm, lifecycle or completion acceptance is claimed.

## Remaining supplied duplicate reported immediately

`raf-okf/src/profile/typed.rs::Number::compare` still uses private
`Decimal::of`/`magnitude` to implement exact sign/zero/decimal ordering. Native
`purrdf_xsd::numeric::parse_decimal` (`xsd/src/numeric.rs:851`) returns
`exact::Decimal`, whose Ord (`exact/decimal.rs:2557`) supplies that operation.
This is a concrete additional reuse opportunity under the user's all-supplied-
cases instruction, not a failure of the RDF term cutover.

The owner and root were notified. Preserve Number's narrower grammar (no +1,
.5, 1., exponents or surrounding whitespace), verbatim text and text-based Eq;
native XSD decimal parsing accepts more spellings, so native value comparison
does not authorize broadening OKF ingress. Native parsed storage can be retained
at construction to avoid allocation on every comparison, but it then belongs
to raf-okf's actual type ownership and memory checks. Required neighbors include
long fractional precision, rounded-equal f64 values, leading/trailing zeros,
negative zero and negative magnitude order. No implementation or executed proof
of this additional replacement was observed at this checkpoint.

## Earlier graph finding closed

Graph::from_nodes now explicitly refuses both empty subject maps and empty
property value lists. The public test
`native_storage_refuses_empty_assertions_and_invalid_term_roles` actually
constructs both invalid maps, checks refusal, verifies nonempty Nodes adoption
roundtrips the example graph, refuses literal/over-depth quoted subject roles
and includes a valid quoted-statement neighbor. The owner reports it executed
passing. This closes the earlier invisible-container finding in source and the
reported execution; this reviewer did not rerun it.

## Production RDF and census paths

Current production src still uses native TermValue/TermBox throughout. Graph
stores the same native-term Nodes/Properties as census validation, orders by
native TermValue, rejects direct/adopted duplicates and validates owned roles,
root/references and depth. Census read transfers into_nodes; write validates
domain shape then transfers from_nodes. No generic census Term enum or full
ground conversion graph has returned.

The reader uses parse_dataset_with, rejects named_graphs before the native
flat_default_graph_quads projection, and moves direct TermValue rows including
reifier/annotation restoration. Ownership normalization retains exact selected-
root boundaries. Native parser success alone does not accept a document: graph
validation, complete BLAKE3 BASE comparison and exact canonical byte comparison
still follow. The output writer uses native term spelling/traversal with only
the report's BASE expansion callback and assertion/directive framing.

Relative IRI validation currently checks the actual absolute report namespace
through the native IRI parser after the report's fragment-alphabet restriction;
absolute spelling remains unnormalized. Literal language/direction and nested
triple refusal still apply in ordinary and quoted roles. Integer construction
uses the native exact integer parser and canonical lexical check. No lost role,
statement-layer or ownership contract was found in this bounded recheck.

## Hashing, arithmetic and SCC delegates

- Census fingerprints use native BLAKE3 while retaining exact u32 little-endian
  token lengths and full digest bytes. Native hex covers full/abbreviated
  display and lowercase decoding; no u64 native framing substitution occurred.
- Workplan digest uses sha2::Sha256 streaming, with original kind/NUL/name/NUL/
  decimal-length/NUL/text framing and CRLF normalization. Its copied compressor
  is removed. PurRDF itself selects sha2 for SHA-2; the supplied public native
  one-shot ContentDigest::of now owns constraint-unit hashing. Native hex owns
  canonical formatting/parsing without accepting uppercase record identities.
- Coverage times delegates to BigInt::from_decimal_digits/from_u128/mul. The
  surrounding exact comparison law remains intact; no float approximation or
  silent default output replaces the native result.
- Workplan strongly_connected delegates traversal to native tarjan_scc and
  explicitly sorts members/components, retaining its documented deterministic
  finding order instead of relying on incidental native emission order.

No additional handwritten supplied hash/hex/SCC/multiplication implementation
was found in those named paths. This is not a whole-repository helper census;
the separately reported Number comparison remains outstanding at this source
checkpoint.

## Source oracle JSON and dependency scope

`raf-source/tests/support/rustc.rs` now parses each compiler diagnostic through
`purrdf_lex::json::read` and accesses native Value directly. The handwritten JSON
tokenizer/model is gone. Diagnostic message/span/child traversal is domain
projection, not another parser. Required diagnostic level is now a hard error;
the source test checks absent/null/numeric levels fail beside valid level and
null code. Optional diagnostic code and primary-line extraction remain explicit
typed native Value accesses. No resident JSON roundtrip or rendered-message
substitute is used for source identity.

The six direct PurRDF workspace dependencies name version 3.0.1 and the exact
same git revision `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`; lockfile entries
observed use that source. Member manifests inherit the workspace pins. The new
production RDF parser has the larger native codec/GTS/SPARQL/regex/rayon closure
documented by its actual manifest; it is not just the smaller lexical leaf.
Both denial files show scoped lex/iri/core licensing exceptions and the approved
git source, plus the exact syn-major coexistence exception with its reason.
This inspection establishes explicit configuration, not a deny/MSRV/wasm gate
pass or an exhaustive transitive license audit. Core/rdf manifests describe
rlib/PyO3-free boundaries; no host portability execution was performed here.

Clippy may still make focused source repairs. Recheck any semantic edits and the
additional Number delegation before an all-reuse completion claim; existing
focused passing results do not replace the required final gates.

## Subsequent narrow repair disposition

Rechecked the owner's Number, line-ceiling, render-reuse and diagnostic changes
in current source without executing builds/tests. The previously outstanding
Number duplicate is now closed in source:

- Number retains one native `purrdf_xsd::exact::Decimal`, constructed only after
  the unchanged narrower OKF sign/digit/fraction grammar accepts the text.
  `compare` delegates to native Ord. Private Decimal splitting/magnitude code is
  removed, and textual PartialEq remains based on the authored spelling.
- `value` delegates numeric conversion to native `to_f64` and reapplies the
  authored sign with copysign. This retains negative-zero display without using
  the display approximation for exact comparison. The inspected records test
  asserts both zero display signs, distinct text equality, exact value order
  for long coefficients/fractions and reversed-order neighbors.
- raf-okf now inherits the same pinned purrdf-xsd workspace dependency. Native
  decimal storage is real retained field ownership rather than a dummy use.
  The owner reports actual records/writer/bounds and Number doctest runs pass;
  this reviewer has not independently rerun them or certified every memory bound.

Line ceilings now delegate the **whole** comparison to native exact Decimal:
line counts become exact native Integer/BigInt-backed values, and percentages
compare `orphan * 100` to `percent * counted` with native products. The old
times function, intermediate decimal-product strings and parse-failure-to-false
path are removed. Native parse is guarded by Number's accepted text invariant;
multiplying by an integer adds no fractional scale. The inspected unit assertion
includes long fraction above/below one-third, usize::MAX, counted zero, exact
12.5% neighbors, negative zero and exact line-ceiling neighbors. The owner
reports eight line-unit tests passed; no reviewer execution is claimed.

RDF `write::render` now returns its computed BASE and selected canonical text
together. Public graph output consumes the text; read compares that returned
digest and text instead of separately calling base then render. Normalized
identity is therefore produced once for the read's canonical check, without
changing digest precedence or carrier equality. Native term writers and BASE
expansion remain as previously reviewed.

The source oracle still reads native JSON Value directly. Required string
diagnostic level and missing/null/numeric refusal bodies remain present beside
valid null-code handling. The owner reports the actual 17 format/compiler cases
passed. No custom JSON parser or silent missing-level default has returned.

No new concrete defect found in these narrow repairs. The outstanding Number
reuse finding above is superseded by this verified source disposition; no
remaining duplicate is asserted for the named jobs reviewed here. Broader
qualification, Clippy and required gates remain the owner's work and are not
made complete by this review.
