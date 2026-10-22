<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native YAML compatibility boundary

Read-only compatibility audit; no builds, product edits, timing or mutations.
Read actual raf-okf reader/scalar/value/document code and native public YAML
reader, resolver and lexical APIs. RAF pins PurRDF commit
5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43; local main is
6f1199b1dfb2ca3e800db7fbc45fd9aefe372c6c. Inspected the pinned YAML API with
git show and compared its YAML files to local main: no differences in those
files. This is source inspection, not a compatibility execution result.

## Whole-parser substitution is currently incompatible

Native `purrdf_lex::yaml::read_with(text, Limits)` returns JSON Value. Its public
knobs are collection depth, node count, aliases and scalar-key acceptance.
It has typed first-error byte offset/line/column and explicit nonrecursive
collection stacks. Setting aliases=false and max_depth=32 is useful but does
not express RAF's entire current reader contract:

- RAF Scalars retain every plain scalar as its original decoded text/style.
  The actual `numbers_booleans_and_null_are_read_as_strings` assertions include
  `0x1F`, `true`, `null`, `~` and `.inf`. Native YAML resolves core booleans/null,
  normalizes non-JSON number spellings (`0x1F` to 31), and refuses nonfinite
  `.inf`. A Value adapter cannot recover lost spellings or accept that valid
  RAF input. There is no public disable-core-scalar-resolution option.
- RAF retains single/double/plain quote style, Flow/Block list form, verbatim
  attached comments/blank lines, trailing comment whitespace and key/item
  lines. Document additionally retains fence EOL/closing-newline and raw body.
  Native Value keeps object order and semantic data but no successful-node
  spans, comments, scalar style or collection style. Its error location is
  not a successful-node provenance stream. Native write normalizes YAML and
  cannot implement byte-preserving RAF round trips.
- RAF refuses flow maps, nested flow/list shapes, multiline and block scalars,
  anchors/aliases/tags/merge keys, quoted or non-profile keys, unknown escapes,
  trailing commas and layout forms with precise RAF ErrorKind/payload/location.
  Native accepts many such YAML forms. aliases=false only covers one part;
  collection shape can sometimes be rejected from Value but original syntax,
  token payload, style and refusal location have already been lost.
- RAF's whole-text byte refusal happens before reading frontmatter, and
  nesting refusal precedes descent at the configured 32 collections. Native
  depth semantics count its own block/flow collections, while max_nodes also
  imposes an independent acceptance boundary. No public allocation Admission
  parameter exists on read_with. A post-parse filter cannot claim RAF's early
  refusal/allocation behavior. Reusing native parsing would require remeasuring
  actual resource bounds, not relabeling native depth/node limits as identical.

Fences, whole-document byte bound, EOL/body ownership and domain profile
validation can coherently remain RAF adapters. However a local full syntax pass
that reconstructs all successful-node metadata and scalar spelling before/after
native parsing would retain essentially the duplicate parser and add another
allocated model. Calling native merely to validate already-parsed RAF data is
not substantive functionality ownership. A native-first/local-fallback parser
would also violate the one-path requirement and obscure real refusals.

## Concrete small compatible primitive

Native public `purrdf_lex::terminals::echar_value` supplies exactly the decoded
values for RAF's five overlapping escapes: backslash, double quote, n, r, t.
RAF must still apply its closed six-member escape policy before accepting a
native result: native also accepts b, f and single quote, which RAF refuses.
RAF's sixth escape, 0 to NUL, has no echar_value mapping and remains an explicit
profile extension, not a fallback after an arbitrary native parse failure.
If adopted, the actual returned native character must feed Scalar.text, retaining
RAF's current UnknownEscape byte location and writer policy. This is a real
small decoding seam, not a compatible whole YAML parser or writer replacement.
The existing every-known/unknown-escape fixture is its direct refused/valid
owner. It adds only the already-pinned leaf lex dependency to raf-okf; native lex
has purrdf-hash as its sole runtime dependency and no RDF/wasm host boundary.

Native JSON escape decoding is not a better exact replacement: its grammar
accepts extra escapes and Unicode forms and lacks RAF's 0 extension. Native
YAML scalar decoding and quote-choice helpers are private; the public API
offers whole-document Value parsing/writing, not a bounded scalar-token decoder
with original lexeme/style and consumed span. Do not parse a synthetic YAML
document per scalar merely to claim native reuse.

## Exact native API gap for a substantive parser cutover

A coherent upstream seam would expose native lexical/parse events with raw
scalar lexeme plus decoded text/style, mapping/sequence style, key/item spans
and comment spans, with policy controls for string-only scalars and RAF's
closed forms/escape set plus early bounded admission. RAF could then own fences
and its public preservation model while native code actually owns tokenization
and collection parsing in one pass. This is a concrete missing public capability,
not permission to implement a new generic YAML walker in RAF or to alter
PurRDF main under the current read-only task.

Required preservation/refusal neighbors already exist in render.rs and
refusals.rs: exact render bytes/comments/EOL, original scalar text/styles,
duplicate/shape/anchor/escape locations, exact byte/depth limits and first-anchor
bomb refusal. None was executed in this audit. Keep them and the narrower
profile unchanged; do not infer compatibility from general YAML superset
acceptance. Current section assessment/candidate remains frozen and unaccepted
by this source review.

## Adopted ECHAR seam recheck

Current ESCAPES obtains all five overlapping decoded values directly from the
native const echar_value result, with explicit invariant expectations. The
reader pushes those actual returned characters; inverse writing and
representability also consume the same table. This is substantive shared
decoding ownership rather than an unused dependency/call. The sixth 0/NUL
extension remains explicit. The closed table still excludes native b, f and
apostrophe escapes, so no larger native grammar was accidentally accepted.

The refusal fixture independently asserts all six expected pairs, rather than
deriving its expected character solely from the production table. It also
requires exact UnknownEscape and line/column (2,5) for each extra native escape,
retaining a literal apostrophe valid neighbor. Actual render/reread exercises
all six values through constructors and the inverse writer. The direct lex
dependency is declared; root and fuzz lock ownership is updated. No whole-YAML
adapter or secondary parser path was added.

Explanation accurately records the API metadata/string-spelling gap and the
small adopted seam. It does not relocate a runtime resource/refusal limitation
outside Constraints. No concrete defect found in this narrow source recheck;
the read is released. Owner reports session 84740 exit 0 for all-target check,
29 refusal tests, 22 render tests, one bounds test and all-target Clippy. No
reviewer execution or broader section/adoption acceptance is claimed.
