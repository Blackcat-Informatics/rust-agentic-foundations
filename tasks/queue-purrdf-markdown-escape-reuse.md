<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native Markdown and terminal primitive reuse

Read-only source audit against current `/home/paudley/Active/purrdf`, after
reading its guide and relevant manifests. RAF scope: raf-okf body heading/link
helpers and raf-workplan escape.rs. No builds, mutations or performance claims.

## Concrete supplied predicate

`purrdf_text::unicode::is_bidi_control(char)` is public and reads the pinned
Unicode Bidi_Control property. This is the native semantic home for the exact
manual bidi ranges in RAF `escape::hidden`: ALM, LRM/RLM, embeddings/overrides
and isolates. Replace that local range list with the native predicate while
retaining Rust's `char::is_control()` for Cc and RAF's line/tab retention policy.
Native `is_removed_control` is NOT equivalent: it additionally removes WORD
JOINER, BOM and tag scalars and treats whitespace differently.

This is a predicate substitution, not a complete terminal escaping API.
The text manifest brings core, sparql-eval, hash, xsd and lex; it is wasm-clean
by its declared library boundary. That dependency cost is real even though
the predicate itself is allocation-free. No native dependency was added here.
Actual output/Unicode compatibility checks remain to be run by the owner.

## Heading recognition is unavailable as a compatible public primitive

RAF `body/heading.rs::atx` accepts up to three initial spaces, requires a
space/tab after one to six hashes, rejects an empty heading, and removes a
closing hash run only when whitespace separates it. Native
`markdown/src/dialect/markdown.rs::atx_heading` is private and intentionally
uses a different dialect: `# foo#` becomes `foo` rather than RAF's `foo#`.
Native heading trimming uses ASCII space/tab; RAF's trim removes Unicode
whitespace. Native also does not make RAF's empty-text rejection.

The public `purrdf_markdown::analyze` API accepts a source/profile and builds
the full structural slicer model. Its profile, sections, spans, verses and
citations are not a tiny heading decoder. It is neither a compatible grammar
replacement nor a coherent way to call that private helper. The package
depends on core and hash and owns its own full slicing identity/bounds.

## Slugs and links

No public GitHub heading slug/repeat-suffix API is exported by the inspected
native Markdown crate. Its anchors are citation/source identity concepts, not
RAF's lowercase/punctuation-filtered GitHub slugs. Native lex's public full
lowercase iterator includes context-sensitive Final_Sigma; applying it to a
whole heading would change RAF's per-character lowercase behavior (Greek
capital sigma at a word end is an immediate distinction). Reusing a parser
model or adding a per-character full-string adapter would not replace the
actual slug policy.

RAF link::unescape removes a backslash only before ASCII punctuation and
preserves unknown/trailing escapes. Native lex's public JSON unescape requires
JSON escapes, decodes newline/UCHAR and rejects unknown sequences; Turtle
ECHAR/UCHAR and HTML entity decoding are also different grammars. For example,
Markdown backslash-parenthesis must decode while backslash-n remains literal;
JSON has the opposite behavior. No public compatible Markdown punctuation
unescape primitive was found in the inspected markdown/lex exports.

## Terminal writer boundary

RAF shown/shown_lines emit lowercase Rust-style brace Unicode escapes,
preserve printable quotes/backslashes/non-ASCII text, escape Cc and bidi
formatting, and selectively retain newline/tab. Public native literal carriers
are Canonical, Xml and TurtleLong, with RDF quoting/escaping laws; JSON variants
escape quotes/backslashes, use four-digit Unicode escapes or short forms, and
do not preserve RAF's bidi policy. None is compatible. Native diagnostic
presentation types model messages rather than exposing this terminal writer.

The concrete available reuse is therefore the bidi predicate above. Keep the
remaining RAF grammar/presentation policy local unless PurRDF supplies an
explicit compatible primitive; do not duplicate a full parser, change the
declared output law, or replace one tiny scan with full-model parsing merely
because both implementations mention Markdown or escaping.
