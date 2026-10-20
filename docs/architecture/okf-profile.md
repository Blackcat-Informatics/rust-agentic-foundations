---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Architecture
id: arch:okf-profile
title: "The RAF profile of OKF"
description: "The normative statement of the document types, keys, values, sections and rules that RAF documents follow."
---

# The RAF profile of OKF

RAF keeps one current account of the system. Its design is written in the shape
of the documents themselves, so that a reader finds the contract, and a tool can
check it, in the same place. This page is the normative statement of that shape.
It is itself an OKF document of type `Architecture`.

## Decision

RAF documents follow a closed profile of OKF v0.1. Each of the six types has a
closed key set written in one order, `id` is the identity in place of the OKF base
key `resource`, and the frontmatter is read in a strict subset of YAML that
refuses every construct outside it with its line and column. A document that
breaks a rule is an error; nothing is read loosely or skipped. The documents a
person writes state the current contract, with proposed text inside PROPOSED
blocks named after their change, and a record states a result in its frontmatter
alone, with no prose, so a tool can judge it.

## Relation to OKF

OKF v0.1 is a Markdown document with YAML frontmatter. Its base requires one key,
`type`, and leaves every other key open. RAF narrows that base into a profile:

- The RAF types are closed sets. A document carries every required key listed for
  its type and no key outside the list. The optional keys `tags` and `timestamp`
  may be present on any type, and the Evidence and Coverage records have
  optional keys of their own.
- `id` is RAF's identity key. The closed key sets do not include the OKF base key
  `resource`, so a document that carries it is rejected. This profile does not use
  an IRI identity.
- `type` takes one of six values: the document types `Capability`, `Change` and
  `Architecture`, which people write, and the record types `Evidence`,
  `Assessment` and `Coverage`, which state a result. Any other value is an
  error.

`raf-okf` enforces the per-document profile: it reads each document and reports
every violation of the keys, values, sections, markers and links. `raf-workplan`
loads the documents of a repository through it and adds the rules that span
documents ([workplan document model](../capabilities/workplan-document-model.md)):
the directory layout, the dependency graph and its cycles, the capabilities and
sections that change records name, the PROPOSED blocks against their records, the
open change record each undelivered capability needs, link targets and anchors
against the tracked files, and the generated blocks. It also resolves every binding
of every capability through `raf-source`, so a bound symbol must exist. This
repository runs those checks through the document-contract test of the root
package, over every tracked Markdown file, which adds its own exact capability
inventory.

## Document types and keys

| Type | Directory | Keys, in writing order |
| --- | --- | --- |
| Capability | `docs/capabilities` | `type`, `id`, `title`, `description`, optional `tags`, optional `timestamp`, `state`, `bindings`, `depends` |
| Change | `work/changes` | `type`, `id`, `title`, `description`, optional `tags`, optional `timestamp`, `changes`, `affects`, `contract_effect`, `items` |
| Architecture | `docs/architecture` | `type`, `id`, `title`, `description`, optional `tags`, optional `timestamp` |
| Evidence | `work/evidence` | `type`, `id`, optional `tags`, optional `timestamp`, `subject`, `inputs`, `verdict`, `by`, `for`, optional `command`, optional `revision`, optional `values`, optional `limits` |
| Assessment | `work/assessments` | `type`, `id`, optional `tags`, optional `timestamp`, `subject`, `inputs`, `verdict`, `by` |
| Coverage | `work/coverage` | `type`, `id`, optional `tags`, optional `timestamp`, optional `strictness`, `scope`, `exclusions`, optional `ceilings`, optional `exceptions` |

Every key is required except those the table marks optional. The writer emits the
keys of a type in the order of this table.

The three record types carry no `title` and no `description`: a record states a
result, not prose. Evidence and an assessment share the trace keys `subject`,
`inputs`, `verdict` and `by`, in that order; the inputs are what the verdict was
reached from, each with the digest of its text at the time. A repository holds one
Coverage record, `coverage:default` at `work/coverage/default.md`.

`docs/capabilities` holds only Capability documents, with one exception: the
generated catalogue `docs/capabilities/index.md`. It has no frontmatter and carries
only generated blocks written by the sync operation (`Workplan::sync`). A
hand-written index is not allowed:
apart from its SPDX comment, every non-blank line lies inside a generated block,
and each block marker starts its own line.
Every document sits directly in its directory, and a document of a type lives only
in that type's directory. Each of these directories holds Markdown files only.

## Value rules

- **`id`** is `cap:<slug>`, `change:<slug>`, `arch:<slug>`, `evidence:<slug>`,
  `assessment:<slug>` or `coverage:<slug>` for the matching type. The slug is
  lowercase kebab case and equals the file name without `.md`. The slug of the
  Coverage record is `default`.
- **`title`** and **`description`** are non-empty strings.
- **`tags`** (optional) is a list of non-empty strings with no duplicate. The
  writer emits it sorted.
- **`timestamp`** (optional) is an RFC 3339 date and time with a capital `T`, a
  `Z` or numeric offset, and seconds from 00 to 59, such as `2026-01-01T00:00:00Z`.
- **`state`** is one of `considered`, `designed`, `implementing`, `delivered`.
- **`bindings`** is a map with exactly the keys `code`, `tests` and `examples`,
  each a list. A list never names a symbol that does not exist, so a capability
  that has nothing built binds nothing: the lists of a `designed` or `considered`
  capability are empty.
- **`depends`** is a list of capability ids. `depends` lists the capabilities whose
  contract this one relies on. Each id names an existing capability, and the graph
  they form is acyclic.
- **`changes`** is a non-empty list of capability ids that exist.
- **`affects`** is a list of `cap:<slug>#<section>` entries. The section is one
  of `purpose`, `contract`, `usage`, `examples`, `constraints`, `explanation`,
  `verification`, and the slug appears in `changes`.
- **`contract_effect`** is `none` or `changes`.
- **`items`** is a list of maps with a non-empty `claim`, an optional
  `outcome` and an optional `reason`. The outcome is `implemented`, `dropped` or
  `moved:change:<slug>`. It is optional until the change is completed, and every
  item has one then. A `moved:change:<slug>` outcome names a change record that
  exists. The reason is a non-empty line of text that says why the item has its
  outcome; it is accepted beside any outcome or none, and strict mode asks for it
  on a `dropped` or `moved` item.
- **`subject`** of an Evidence record is `command`, when the verdict comes from the
  command the record states, or `binding:<path>`, when it comes from a bound test
  or example; a path is two or more identifiers joined by `::`. The `command` key
  is present exactly when the subject is `command`. The subject of an Assessment
  is the `cap:<slug>#<section>` it assesses, with one of the seven sections.
- **`inputs`** is a non-empty list of maps with exactly the keys `kind`, `name` and
  `digest`, each row once. The kind is `section` (a name of the form
  `cap:<slug>#<section>`), `symbol` (a path) or `record` (an `evidence:<slug>`
  id). Evidence takes `symbol` rows only. The inputs of an Assessment hold the
  `section` row of its subject. The digest is a SHA-256 digest written as 64
  lowercase hexadecimal digits.
- **`verdict`** is `pass` or `fail` for Evidence and `accepted` or `rejected` for
  an Assessment.
- **`by`** is `human` or `agent`. A record names the kind of its author, never a
  person.
- **`for`** is a non-empty list of capability ids, each once.
- **`command`** (optional) is a non-empty string. **`revision`** (optional) is a
  commit id: 40 or 64 lowercase hexadecimal digits.
- **`values`** and **`limits`** (optional) are maps of names to numbers, where a
  number is an optional `-`, digits, and an optional `.` and digits. A limit of a
  record whose verdict is `pass` names a value (a failing execution, such as one
  killed at its time limit, may have printed none), and a record whose verdict is
  `pass` holds no value above its limit. Values and limits are compared exactly on
  their decimal text (`Number::compare`), never as floating-point numbers. An
  absent map holds nothing.
- **`strictness`** (optional) is `default` or `strict`, and a record without it
  says `default`. It sets how hard the repository is judged
  ([strict mode](../capabilities/workplan-strict.md)). The writer leaves out
  `default`.
- **`scope`** is a list of crate package names, each once; the empty list `[]`
  names no crate.
- **`exclusions`** is a list of maps with exactly the keys `item` and `reason`,
  each item once; the empty list `[]` holds none. The item is a path below a
  crate root, such as `raf_okf::Body`, or a prefix `<module path>::*` that names a
  module below a crate root. A crate root, alone or as a prefix, is refused: a
  crate that is not covered is left out of `scope`. The reason is non-empty.
- **`ceilings`** (optional) is a list of maps with `crate`, exactly one of
  `lines` and `percent`, and an optional `of`; the reverse coverage of
  [strict mode](../capabilities/workplan-strict.md) holds each package to them.
  `crate` is a crate package name; `lines` is a whole number and `percent` a
  number from 0 to 100, compared exactly on its decimal text; `of` is
  `shipping`, `test` or `all`, and `all` without the key. A crate has at most
  one row for each `of`. A row of another shape is refused, and a value that is
  not a number is refused as a measure. An absent key holds none, and the
  writer leaves out an empty list and an `of` of `all`.
- **`exceptions`** (optional) is a list of maps with exactly one of `item` and
  `file`, and `reason`, each `item` or `file` once; an absent key holds none.
  The item is a path below a crate root or a prefix `<module path>::*`, as an
  exclusion's is, and a crate root is refused. The file is a path relative to
  the repository root, of `/`-separated segments none of which is empty, `.`
  or `..`, or such a path with a trailing `/` for a directory; an absolute path
  is refused. The reason is non-empty and holds no placeholder token.
- **Scalars** are quoted or plain. A quoted value closes its quote. A plain value
  contains no `: `, does not start with one of `>`, `|`, `&`, `*`, `!`, `%`, `@` or
  a backtick, and is not a block scalar header. A trailing ` # comment` after a
  value is not part of the value. Numbers, booleans and null are read as the
  strings that were written, never converted. A double-quoted scalar has exactly
  these escapes: `\\`, `\"`, `\n`, `\r`, `\t` and `\0`. Any other escape, such as
  `\q` or `\x41`, is an error.

### Frontmatter subset

The frontmatter is a block mapping in the following subset of YAML. `raf-okf`
accepts exactly this and refuses everything else with an error that gives the
line and column and names the key or token:

- **Accepted.** Block mappings nested by two spaces, with keys of letters, digits,
  `_` and `-`. Block sequences (`- value`) and flow sequences (`[a, b]`) of
  scalars. Block sequences of block mappings (`- key: value`, with the other keys
  of the item indented under the first). Plain, single-quoted and double-quoted
  scalars. Full-line comments, blank lines, and a trailing ` # comment`.
- **Refused.** Flow mappings (`{code: []}`), a flow sequence inside a flow
  sequence or an item, anchors (`&x`), aliases (`*x`), tags (`!x`), merge keys
  (`<<`), and block scalars (`|`, `>`). A duplicate key at any level: at the top
  level, inside `bindings`, and inside an entry of `items`. A frontmatter that is
  not a block mapping, a document with no `---` fence on line 1, and a frontmatter
  that no `---` line closes.
- **Bounds.** The text is at most 1 MiB, and nesting is at most 32 levels, checked
  before the reader descends. The top-level mapping is level 1 and each mapping or
  list inside it adds one.
- **Line endings.** The frontmatter uses the line ending of the opening fence, `LF`
  or `CRLF`, throughout. The body is kept byte for byte.

## Body rules

A Capability body holds exactly seven second-level sections, in this order, none
empty: Purpose, Contract, Usage, Examples, Constraints, Explanation,
Verification. A section is empty when nothing is left after its HTML comments and
generated blocks are removed. No prose carries a placeholder token, that is, a marker
for unfinished text. The rule also covers the frontmatter values `title`,
`description`, item claims and item reasons; code spans are exempt in both. Each section holds one kind of statement:

| Section | Holds |
| --- | --- |
| Purpose | What problem the capability solves and where it fits |
| Contract | What behaviour callers can rely on |
| Usage | How to use it |
| Examples | Representative inputs, outputs and workflows |
| Constraints | Limits, prerequisites and unsupported cases |
| Explanation | Why it works this way and which tradeoffs matter |
| Verification | Which checks substantiate which claims |

A Change body holds a non-empty `## Summary` section. An Architecture body is
free-form and non-empty. A record body is empty: nothing is left after its HTML
comments are removed, because the record is its frontmatter. No placeholder token
appears in the reason of a coverage exclusion either.

## State and body

The state of a capability decides what its document and the change records say.

| State | Contract in the document | Change record |
| --- | --- | --- |
| `considered` | Purpose and open questions in plain text; the candidate contract inside PROPOSED blocks | An open record lists the capability |
| `designed` | The designed contract, stated plainly, with no PROPOSED blocks | An open record lists the capability |
| `implementing` | The designed contract, stated plainly | An open record lists the capability |
| `delivered` | The delivered contract, with no PROPOSED blocks; the capability binds at least one code path and one test | No open record lists the capability |

Outside PROPOSED blocks a document states the current contract only.

### PROPOSED blocks

A PROPOSED block is a block quote whose first line names the change that
proposes it. It continues while the lines keep the quote marker. The marker is
written exactly as shown below. A line that reads as a marker in any other
spelling, such as other letter case, no space after `>`, extra spacing or a nested
quote, is malformed.

```markdown
> [!PROPOSED change:example-slug]
> The text that change proposes to add to the contract.
```

The named change must exist as `work/changes/<slug>.md`, and that record must
list the capability that holds the block, and the block sits inside one of the
seven sections, which appears as `cap:<slug>#<section>` in the `affects` of that
record. A PROPOSED block appears only in a Capability document.

### Generated blocks

A generated block is delimited as follows and is reserved for the `raf-workplan`
crate, which rewrites it deterministically
([generated documents](../capabilities/workplan-generated-docs.md)).

```markdown
<!-- raf:generated:<kind> -->
...
<!-- raf:generated:end -->
```

Each block is opened once, closed once and never nested. A generated marker starts
its line; a `raf:generated:` comment elsewhere in a line is text.

### Claim markers

A statement in a capability document that rests on evidence names it with a claim
marker on its own line, directly after the paragraph or list item it supports:

```markdown
Parsing a 10 MiB input stays under 64 MiB resident.
<!-- raf:claim evidence=memory-bound -->
```

The marker is written exactly as `<!-- raf:claim evidence=<slug>[,<slug>...] -->`,
with leading spaces allowed. Its attributes are `key=value` pairs separated by
single spaces, from a closed set that holds one key, `evidence`, given once; its
value is a comma-separated list of distinct kebab-case slugs, each naming the
record `evidence:<slug>`. The statement is the run of lines directly above the
marker, up to a blank line, a heading, a fence, a PROPOSED block or another marker,
and no further up than the line that opens the nearest list item. A marker with no
such line directly above it is not attached and is an error. A claim marker appears
only in a Capability document.

Every other `<!-- raf:` comment outside a `raf:generated:` comment, in any letter
case and spacing after `<!--`, is an error, so a misspelt claim or tested-by marker,
or a comment with an unknown `raf:` word, is never read as a plain comment. A
`raf:generated:` comment that is not a generated marker, in another letter case or
spacing or not at the start of its line, is text. Markers inside fenced code and
code spans are not read, and neither
are claim and tested-by markers inside a PROPOSED block: a proposed claim is a claim
once the block is resolved.

A marker is one line. A comment that `<!--` opens at the end of a line, outside its
code spans, and whose first non-blank text on the lines below is `raf:` in any
letter case, other than `raf:generated:`, is the error of that word on the line of
`<!--`: a malformed claim marker for `raf:claim`, a malformed tested-by marker for
`raf:tested-by` and an unknown marker for any other word, so a marker split over
lines is never read as a plain comment either. A multi-line `raf:generated:`
comment, an SPDX header written over several lines and every other multi-line
comment are plain comments.

A document reports its first marker error: reading stops at the first malformed
PROPOSED marker, malformed, nested, unclosed or orphaned generated marker,
malformed or unattached claim or tested-by marker, or unknown `raf:` comment, so a
second fault in the same document shows once the first is fixed.

### Tested-by markers

A statement of a capability names the bound tests or examples that show it with a
tested-by marker on its own line, directly after the statement, or directly after
the statement's claim marker:

```markdown
- An exclusion with an empty reason is an error of the record.
<!-- raf:tested-by raf_workplan::test_crate::coverage::an_exclusion_without_a_reason_fails -->
```

The marker is written exactly as `<!-- raf:tested-by <path>[,<path>...] -->`, with
leading spaces allowed: one space after the word, the entries separated by `,` with
no space, each matching the `<item-path>` pattern of the
[completion gate](../capabilities/workplan-complete-gate.md) (a `,` inside a generic
argument list does not separate entries), and no entry twice. Its statement is the
statement of the claim marker directly above it, the heading line directly above it,
or the run of lines a claim marker on its line would support. A marker with none of
these directly above it, after a blank line or another comment, is not attached and
is an error, and so is a marker directly before a claim marker: the claim marker goes
between the statement and the tested-by marker. It is a family of its own and not an
attribute of the claim marker, whose forms stay as above, and it appears only in a
Capability document. Which statements a marker covers, and which paths it may name,
are rules of [strict mode](../capabilities/workplan-strict.md).

## Files and repository rules

- SPDX lines in an OKF document are YAML comments inside the frontmatter. In all
  other Markdown they are HTML comments.
- Every relative link resolves to an existing file, and a link anchor resolves
  to a heading in the target. The rule covers inline links and images, reference
  definitions (`[label]: target`) and the `href` and `src` attributes of inline
  HTML, in every tracked Markdown file. A link split across lines is a violation,
  and so is a link with an empty target. Links inside fenced code, code spans and
  HTML comments are not links, and placeholder tokens inside HTML comments are not
  read. Block quotes and list items nest at most 64 levels, and a link destination
  or title is at most 4096 bytes. Anchors follow the GitHub rule: lowercase, spaces
  become hyphens, other punctuation is dropped.
- A relative link resolves inside the repository root. The checker joins the link to
  the linking file's directory and normalises it against the canonical repository
  root. A target that leaves the root, by `..` segments or an absolute path, is a
  violation, and the target is neither opened nor stat'ed, so a link cannot probe or
  read a file outside the repository. A target inside the root is looked up in the
  list of tracked paths (the Git index), and a link whose target is a tracked symlink
  is a violation, so a symlink cannot lead a link outside the root.
- A committed file holds no issue or pull request number and no wording that
  refers to how the work was planned or organised. The account describes the
  system as it is.
- A change record lives under `work/changes` while its work is open. Completing
  the change deletes the record, and Git keeps its history.

## Conditions to revisit

- An OKF version after v0.1 requires a base key or an identity form, such as an IRI in
  `resource`, that other OKF tools need in order to read RAF documents.
- A document needs a YAML construct the subset refuses, such as a block scalar or
  a flow mapping, to state its contract, and no list or quoted scalar can hold it.
- A record cannot be understood from its result alone, so that a reader needs
  prose beside the verdict, such as the reason it was given.
- A repository needs a document type outside the six, so the closed set of types
  refuses documents its tools must read.
