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
SPDX lines are YAML comments inside the frontmatter and are preserved.

**Validate.** `Profile::validate` checks a parsed document against the closed key
set of its `type`. It reports every violation, not only the first, and each names
the file, the key and the rule. Values are typed: `state` is an enumeration,
`bindings` is a map with exactly `code`, `tests` and `examples`, `affects` entries
are parsed into a capability id and a section, and `items` entries into a claim and
an optional outcome. A document with a key outside its set, with a key missing, or
with the base key `resource` is rejected. A `type` other than `Capability`,
`Change` or `Architecture` is an error. Checks that need other documents, such as
whether an id exists or whether a graph is acyclic, belong to
[the document model](./workplan-document-model.md); `raf-okf` validates one document
at a time.

**Body model.** The body is parsed into a structure that tools query instead of
re-scanning text:

- second-level sections, each with its heading, its slug and the line span of its
  content;
- links, each with its target, its anchor and its line;
- fenced code blocks, so markers inside a fence are never read as markers;
- generated blocks, each with its kind and span;
- PROPOSED blocks, each with the change slug it names and its span.

A malformed PROPOSED marker, an unclosed generated block, a nested generated block
and an end marker without a start are parse errors with line numbers.

**Write.** `Document::render` writes a document back. The writer is deterministic
and round-trips: parsing the output yields an equal document, and rendering a
document that was produced by `render` returns identical bytes. Key order follows
the order of the profile table, comments are kept, and the body is written
unchanged apart from the replaced spans that a caller supplies, such as a generated
block.

**Index handling.** `docs/capabilities/index.md` is the one navigation file in the
capability directory. It is accepted only when it holds a generated block, and it is
never parsed as a Capability document. A hand-written index is a violation.

## Usage

```rust
use raf_okf::{Document, Profile};

let text = std::fs::read_to_string("docs/capabilities/example.md")?;
let doc = Document::parse(&text)?;
let violations = Profile::default().validate(&doc, "docs/capabilities/example.md");
assert!(violations.is_empty());
let sections = doc.sections();
```

Callers that rewrite a span use `doc.replace_span(span, new_text)` and then
`doc.render()`.

## Examples

A frontmatter block the profile accepts, for a capability with nothing built:

```yaml
type: Capability
id: cap:example
title: "Example"
description: "One line."
state: designed
bindings: {code: [], tests: [], examples: []}
depends: []
```

The same block with `resource: https://example.org/x` added is rejected with a
message that names the `resource` key and the closed key set of `Capability`.

## Constraints

- The crate reads and writes text only. It touches no file and runs no process, so
  a caller supplies the document text as strings.
- Keys are exactly those of the profile. There is no pass-through for extra keys.
- A document is valid or invalid. There is no warning level.
- The writer preserves comments in the frontmatter but does not reflow the body.
- The crate takes no position on tense or content. Checks on prose belong to the
  reconcile operation.

## Explanation

The profile is deliberately narrower than its parent format. A base that requires one
key and leaves the rest open suits a wide ecosystem; a toolchain that must answer
"is this document complete" needs the opposite, so the key sets are closed and every
key is required. A missing key then means a missing fact rather than an optional
one, and a misspelled key is an error rather than a silent no-op.

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
- A body fixture where a PROPOSED marker and a generated marker appear only inside a
  fence and produce no blocks, and fixtures for each marker error.
- A round-trip property over the fixture documents: `render(parse(x))` equals `x`
  for canonical input, and `parse(render(d))` equals `d`.
- An index fixture: a generated-block index is accepted and a hand-written one is
  rejected.
