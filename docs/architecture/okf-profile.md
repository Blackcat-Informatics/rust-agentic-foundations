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

## Relation to OKF

OKF v0.1 is a Markdown document with YAML frontmatter. Its base requires one key,
`type`, and leaves every other key open. RAF narrows that base into a profile:

- The RAF types are closed sets. A document carries exactly the keys listed for
  its type, no more and no fewer.
- `id` is RAF's identity key. The closed key sets do not include the OKF base key
  `resource`, so a document that carries it is rejected. This profile does not use
  an IRI identity.
- `type` takes one of three values: `Capability`, `Change` or `Architecture`.
  Any other value is an error.

This repository enforces the profile today with the document-contract test of the
root package. It checks keys, values, sections, markers and links, the exact
capability inventory, and the binding lists by state: `designed` and `considered`
capabilities bind nothing, and a `delivered` capability binds at least one code
path and one test. It does not check that a bound symbol exists: resolving a
binding waits for the source index. The `raf-okf` and `raf-workplan` crates will
enforce the whole profile once they are implemented.

## Document types and keys

| Type | Directory | Keys, all required |
| --- | --- | --- |
| Capability | `docs/capabilities` | `type`, `id`, `title`, `description`, `state`, `bindings`, `depends` |
| Change | `work/changes` | `type`, `id`, `title`, `description`, `changes`, `affects`, `contract_effect`, `items` |
| Architecture | `docs/architecture` | `type`, `id`, `title`, `description` |

`docs/capabilities` holds only Capability documents, with one exception: the
generated catalogue `docs/capabilities/index.md`. It has no frontmatter and carries
only generated blocks written by `docs sync`. A hand-written index is not allowed:
apart from its SPDX comment, every non-blank line lies inside a generated block,
and each block marker starts its own line.
Every document sits directly in its directory, and a document of a type lives only
in that type's directory. Each of these directories holds Markdown files only.

## Value rules

- **`id`** is `cap:<slug>`, `change:<slug>` or `arch:<slug>` for the matching
  type. The slug is lowercase kebab case and equals the file name without `.md`.
- **`title`** and **`description`** are non-empty strings.
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
- **`items`** is a list of maps with a non-empty `claim` and an optional
  `outcome`. The outcome is `implemented`, `dropped` or `moved:change:<slug>`.
  It is optional until the change is completed, and every item has one then. A
  `moved:change:<slug>` outcome names a change record that exists.
- **Scalars** are quoted or plain. A quoted value closes its quote. A plain value
  contains no `: `, does not start with one of `>`, `|`, `&`, `*`, `!`, `%`, `@` or
  a backtick, and is not a block scalar header. A trailing ` # comment` after a
  value is dropped.

## Body rules

A Capability body holds exactly seven second-level sections, in this order, none
empty: Purpose, Contract, Usage, Examples, Constraints, Explanation,
Verification. A section is empty when nothing is left after its HTML comments are
removed. No prose carries a placeholder token, that is, a marker for unfinished
text. Each section holds one kind of statement:

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
free-form and non-empty.

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
crate, which rewrites it deterministically. None exist yet.

```markdown
<!-- raf:generated:<kind> -->
...
<!-- raf:generated:end -->
```

Each block is opened once, closed once and never nested.

## Files and repository rules

- SPDX lines in an OKF document are YAML comments inside the frontmatter. In all
  other Markdown they are HTML comments.
- Every relative link resolves to an existing file, and a link anchor resolves
  to a heading in the target. The rule covers inline links and images, reference
  definitions (`[label]: target`) and the `href` and `src` attributes of inline
  HTML, in every tracked Markdown file. Links are single-line: the text and the
  target of a link sit on one line, and a link split across lines is not read.
  Anchors follow the GitHub rule: lowercase, spaces become hyphens, other
  punctuation is dropped.
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
