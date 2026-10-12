---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:okf-profile
title: "OKF profile reader and writer"
description: "The raf-okf crate: parse OKF Markdown, validate the RAF profile with closed key sets, and write documents back deterministically."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: []
---

# OKF profile reader and writer

## Purpose

`raf-okf` is the one reader and writer of RAF documents. Everything that touches a
capability, change or architecture page, the workplan gate and the repository
tooling alike, goes through it, so the profile is interpreted in a single place and
a document means the same thing to every tool.

## Contract

The facts of the profile, which are the document types, the closed key sets, the
value rules, the body rules and the PROPOSED and generated block syntax, are stated
once, in [the RAF profile of OKF](../architecture/okf-profile.md). `raf-okf`
implements that page and adds no rule of its own. This document states only what the
crate does with it.

**Parse.** `Document::parse` takes the text of one file and returns the frontmatter
and a body model. It fails with a positioned error when the file has no frontmatter
fence on line 1, when the YAML does not parse, or when the YAML is not a mapping.
SPDX lines are YAML comments inside the frontmatter and are preserved. Comments are
kept with the key they belong to: a full-line comment or blank line attaches to the
key that follows it, a trailing ` # comment` stays on its line, and comments after
the last key stay last. The frontmatter accepts the subset of YAML stated in
[the RAF profile of OKF](../architecture/okf-profile.md#frontmatter-subset) and
refuses the rest. Numbers are read as strings.

**Resource bounds.** Documents are read from sources the reader may not control, so
`Document::parse` bounds its input and fails with a positioned error and never a
panic, abort or unbounded allocation:

- the text is at most 1 MiB;
- YAML nesting is at most 32 levels, and the check runs before any recursive
  descent can overflow the stack;
- YAML anchors, aliases and merge keys are refused, because the closed key schema
  has no use for them and an alias expansion can grow without bound;
- Markdown block nesting (quotes and lists) is at most 64 levels;
- a link destination or title is at most 4096 bytes, not counting the delimiters of a
  title or of an angle-bracket destination. This bounds the link scanner, which
  would otherwise rescan a long paragraph for every `](`. A longer part is a
  positioned error, never a link that is silently not read.

**Validate.** `Profile::validate` checks a parsed document and its path against the
closed key set of its `type` and returns a typed result: the typed document
(`RafDocument`, with `State`, `Bindings`, `Affects` entries, `ChangeItem` entries and
their `Outcome`), or the list of every violation, not only the first. Each violation
names the file, the rule and, for a frontmatter rule, the key; a body rule names the
section or the line instead. The rules are:

- the key set comes from one table per type, in the order of the architecture page. A
  key outside the table, a missing required key, and the base key `resource` (a rule of
  its own, which names the key set of the type) are violations. The optional keys
  `tags` and `timestamp` may be absent;
- a missing `type`, or one other than `Capability`, `Change` or `Architecture`, is the
  only violation reported for that document, because no key set applies. A document otherwise sits directly in the directory
  of its type;
- the `id` carries the prefix of its type and a kebab-case slug equal to the file name
  without `.md`; `title` and `description` are non-empty strings;
- `state` is an enumeration, and `bindings` is a map with exactly `code`, `tests` and
  `examples`, each a list. A `designed` or `considered` capability binds nothing, and a
  `delivered` one binds at least one `code` and one `tests` entry;
- `depends` and `changes` hold capability ids (`changes` at least one), `affects`
  entries are parsed into a capability id and one of the seven sections, and name a
  capability listed in `changes`; `contract_effect` is `none` or `changes`; `items`
  entries are parsed into a non-empty claim and an optional outcome
  (`implemented`, `dropped` or `moved:change:<slug>`);
- `tags` is a list of non-empty strings with no duplicate, and `timestamp` is an RFC
  3339 date and time;
- a Capability body holds the seven sections in order, none empty; a Change body holds
  a non-empty `Summary`; an Architecture body is not empty. A section is empty when
  nothing is left after its HTML comments and generated blocks are removed;
- a PROPOSED block appears only in a Capability and only inside a section;
- no prose line, title, description or item claim holds a placeholder token, and
  no link is split across lines. `Body::placeholder_lines` returns the
  placeholder tokens of any body, with their lines, by the same rule, so a caller
  that reads Markdown without frontmatter applies the rule `Profile::validate`
  applies; fenced code, code spans and HTML comments are not read.

Checks that need other documents, such as whether an id exists or whether a graph is
acyclic, belong to [the document model](./workplan-document-model.md); `raf-okf`
validates one document at a time. Prose that talks about future work is not a
violation: the crate takes no position on tense.

**Body model.** The body is parsed into a structure that tools query instead of
re-scanning text. `Document::parse` builds the body of a document through
`Body::parse`, which also reads Markdown that has no frontmatter, such as a README: a
leading `---` fence closed by a later `---` line is skipped, and otherwise the whole
text is the body. Every span the model returns carries both line numbers and byte
offsets, in the coordinates of the whole text. The model holds:

- headings of levels 1 to 6 outside fenced code, each with its GitHub anchor (repeats
  numbered `-1`, `-2`), and a check whether an anchor exists;
- second-level sections, each with its heading, its slug and a span that starts on the
  line after the heading and ends before the next second-level heading;
- links, each with its target, its anchor, its line and its form: inline link, image,
  reference definition, `href` attribute or `src` attribute. Links inside fenced
  code, code spans of any backtick length and HTML comments are not links. A target
  resolves against the path of the linking document to an external target, a
  fragment, or a path inside the repository, and a target that climbs above the root
  or is an absolute path is an error, never clamped. A link inside a block quote reads
  the same on every quoted line;
- split links: an inline link or image whose text or parentheses span a line break is
  not read as a link and is recorded with its line, so that a split link is a
  violation;
- fenced code blocks, matched by character and length and recognised inside block
  quotes and lists, so markers inside a fence are never read as markers;
- generated blocks, each with its kind, its outer span (markers included) and its
  content span (markers excluded);
- PROPOSED blocks, each with the change slug it names, its span, its marker line, its
  unquoted content and the section that holds it;
- prose lines: the lines outside fenced code and HTML comments with code spans
  removed, each with its section and whether it lies in a PROPOSED or generated block.

A malformed PROPOSED marker in any spelling, an unclosed generated block, a nested
generated block and an end marker without a start are parse errors with line numbers.
So are block quotes and list items nested over 64 levels. `Body::replace_spans` applies
non-overlapping byte-range edits in one pass and reads the model again; it is all or
nothing, and it fails with a typed error for a range outside the body, a range inside a
character, two ranges that overlap, or a result that no longer parses. `Document` and
`Index` delegate to it.

**Write.** `Document::render` writes a document back. The writer is deterministic
and round-trips: parsing the output yields an equal document, and rendering a
document that was produced by `render` returns identical bytes. Key order follows
the order of the profile table: the keys of the document's type come first in table
order, then any other keys in input order, and a document of an unknown type keeps
input order. The top-level `tags` list is put in sorted order when a document is
parsed or built, so it is written sorted and a document equals the reading of its own
rendering; every other list keeps author order. Comments travel with the key they are attached to, and the body is written
unchanged apart from the replaced spans that a caller supplies, such as a generated
block. Two documents are equal when their keys, compared as a map, have equal values
and equal attached comments, and their bodies hold the same bytes.

**Index handling.** `docs/capabilities/index.md` is the one navigation file in the
capability directory. `is_capability_index` is true for that path and false for any
other, including `docs/index.md`. `Index::parse` accepts a text only when apart from an
SPDX header comment (one line, or several between `<!--` and `-->` lines, each tag with
a value) every non-blank line lies inside a generated block and each marker starts its
own line with no indentation. Prose, an indented or mid-line marker, an empty header
tag, a header after a block, an unclosed block and a text with no block are errors with
a line. The text of an index never parses as a Capability document: `Document::parse`
fails on it at line 1. `Index::render` returns the text, and `Index::replace_spans`
replaces a block and refuses an edit that leaves anything but an index.

## Usage

```rust
use raf_okf::{Document, Profile, RafDocument};

const TEXT: &str = "\
---
type: Architecture
id: arch:example
title: \"Example\"
description: \"One line.\"
---

# Example

What this page says.
";

let document = Document::parse(TEXT).expect("the text is a document");
assert_eq!(document.render(), TEXT);
match Profile.validate(&document, "docs/architecture/example.md") {
    Ok(RafDocument::Architecture(page)) => assert_eq!(page.header.title, "Example"),
    Ok(other) => panic!("not an architecture page: {other:?}"),
    Err(violations) => panic!("{violations:?}"),
}
```

The text comes from the caller, so the block needs no file access. A caller that rewrites
a span of the body reads the span from the body model, for example
`document.body().generated_blocks()[0].content()`, then calls
`document.replace_spans(&[(span.clone(), new_text)])` and `document.render()`.

## Examples

A frontmatter block the profile accepts, for a capability with nothing built:

```yaml
type: Capability
id: cap:example
title: "Example"
description: "One line."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: []
```

Between `---` fences, with the seven sections as its body, at the path
`docs/capabilities/example.md`, the block validates. The same block with
`resource: https://example.org/x` added is rejected with a violation at the key
`resource`, whose message names the `resource` key and the closed key set of
`Capability`.

## Constraints

- The crate reads and writes text only. It touches no file and runs no process, so
  a caller supplies the document text as strings.
- Input is bounded to 1 MiB, 32 YAML levels, 64 Markdown block levels and 4096 bytes
  per link destination or title, and YAML anchors and aliases are refused. A bound
  violation is a parse error.
- Only ATX headings (`#` marks) and fenced code blocks are modelled. A setext heading
  is not a heading, and indented code is read as ordinary text, so a marker or a link
  in it counts; write code that holds a marker or a link in a fence.
- Keys are exactly those of the profile. There is no pass-through for extra keys.
- A document is valid or invalid. There is no warning level.
- The writer preserves comments in the frontmatter but does not reflow the body.
- The crate takes no position on tense or content. Checks on prose belong to the
  reconcile operation.

## Explanation

The profile is deliberately narrower than its parent format. A base that requires one
key and leaves the rest open suits a wide ecosystem; a toolchain that must answer
"is this document complete" needs the opposite, so the key sets are closed and every
key is required, except the two base keys `tags` and `timestamp`, which carry no
fact the toolchain depends on and are optional. A missing required key then means a
missing fact rather than an optional one, and a misspelled key is an error rather
than a silent no-op.

The body model exists so that no other crate scans Markdown. A line that begins with
the PROPOSED marker inside a code fence is documentation of the syntax, not a
proposal, and only a parser that tracks fences can tell the difference. Putting that
rule in one crate keeps the gate, the generator and the reconcile operation agreeing.

The writer is deterministic because generated blocks are rewritten in files that
people also edit. If rendering could reorder keys or alter whitespace, every sync
would produce noise in the diff, and a clean check could not be told from a stale one.

## Verification

- A parse fixture per failure: no frontmatter, malformed YAML, a non-mapping
  frontmatter.
- A validation fixture per type for each missing key, each extra key, the key
  `resource`, and an unknown `type`; each asserts the named key and rule.
- A value fixture per rule: `state` outside its set, `bindings` missing a key,
  an `affects` entry with an unknown section, an `items` outcome outside
  `implemented`, `dropped` and `moved:change:<slug>`.
- Bound fixtures: a document of 1 MiB plus one byte, frontmatter nested 33 levels
  deep, a quote nested 65 levels deep, and a link destination or title of 4097 bytes
  each fail with a positioned error, and documents at the limit (a destination or
  title of 4096 bytes included) parse.
- An alias fixture: frontmatter with an anchor and an alias (a "billion laughs"
  document) fails fast with a positioned error naming the anchor, and the process
  allocates no more than the input size times a small constant.
- A body fixture where a PROPOSED marker and a generated marker appear only inside a
  fence and produce no blocks, and fixtures for each marker error.
- A round-trip property over the fixture documents: `render(parse(x))` equals `x`
  for canonical input, and `parse(render(d))` equals `d`.
- An index fixture: a generated-block index is accepted and a hand-written one is
  rejected.
