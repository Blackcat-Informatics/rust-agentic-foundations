---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-document-model
title: "Workplan document model"
description: "Capability and change documents, the four states, PROPOSED blocks, the graph, binding resolution and the check rules."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:okf-profile, cap:source-index]
---

# Workplan document model

## Purpose

Plan changes to the documented system. Implement those changes. Reconcile the
documentation with the result. Let Git retain the previous states. The workplan model
is built on that sentence. A capability has one permanent document that states its
current contract, and a change is a temporary record of how that document is about to
move. A reader never has to reconstruct chronology to learn what applies now.

## Contract

**Capability documents.** A capability lives at `docs/capabilities/<slug>.md`, with
the identity `cap:<slug>`, the keys and the seven sections defined in
[the RAF profile of OKF](../architecture/okf-profile.md) and read by
[the OKF reader](./okf-profile.md). The document is permanent: it keeps its identity
and its path from the first time the capability is considered until the capability is
removed.

**The four states.** `state` is one of the following, and each fixes what the
document and the change records may say.

| State | What the capability document holds | Change record |
| --- | --- | --- |
| `considered` | Purpose, open questions and the candidate contract, the candidate inside PROPOSED blocks | An open record lists it |
| `designed` | The designed contract, the selected approach, rationale and planned examples, stated plainly | An open record lists it |
| `implementing` | The same contract with real code and test bindings and the remaining gaps | An open record lists it |
| `delivered` | The supported contract, usage, examples, limitations, current rationale and verification links | No open record lists it |

A change to a delivered capability puts the capability back under an open record: the
current account stays as written, and the proposed text sits beside it in PROPOSED
blocks. Everything that moves a capability between states is a change.

**PROPOSED blocks.** Text that a change proposes is written into the capability
document, marked as a block quote whose first line names the change:

```markdown
> [!PROPOSED change:incremental-parsing]
> The parser accepts input in arbitrary chunks and reports incomplete input when
> the caller finishes the stream.
```

The block continues while lines keep the quote marker. The branch diff therefore
shows the code and the documentation it produces together. Outside PROPOSED
blocks a document states the current contract only. A PROPOSED block is valid only in
a Capability document, only when `work/changes/<slug>.md` exists, and only when that
record lists the capability. A `delivered` capability has none, and a `considered`
capability has at least one.

**Change documents.** A change lives at `work/changes/<slug>.md` with the identity
`change:<slug>`. Its keys are `type`, `id`, `title`, `description`, `changes`,
`affects`, `contract_effect` and `items`:

- `changes` lists the capabilities the change alters.
- `affects` lists `cap:<slug>#<section>` entries naming the sections that must be
  reconciled.
- `contract_effect` is `changes` when the documented contract moves and `none` when
  the change is internal.
- `items` lists the claims the change makes. Each has a `claim` and, before the
  change completes, an optional `outcome` of `implemented`, `dropped` or
  `moved:change:<slug>`. A `moved` outcome names an existing change that now carries
  the claim.

The body holds a non-empty `## Summary`. Completing a change deletes the file, and Git
holds its history.

**Slug identities.** A slug is lowercase kebab case and equals the file name without
`.md`. Every reference between documents uses the id form, so a rename is one edit to
the file name, the id and the references, which `check` verifies.

**The graph.** `raf-workplan` builds a graph from the documents and the source.

- Nodes: capabilities, sections, changes, proposals, code symbols and tests.
- Edges: bindings (capability to symbol, test or example), `depends`, `affects` and
  Markdown links.

The graph must have no `depends` cycle. It supports questions in both directions:
which code realises this capability, which capability explains this symbol, which
tests address this guarantee, and which sections this change must update.

**Binding resolution.** Each entry in `bindings.code` is a Rust path, each entry in
`bindings.tests` a test function path, and each entry in `bindings.examples` an
example target or a documentation test item. All resolve through the
[source index](./source-index.md), including re-exports. A binding that does not
resolve is an error. A capability with nothing built binds nothing, so `designed`
capabilities carry empty lists. An `implementing` capability may have empty or
missing bindings; `status`, the catalogue and the reconcile obligations report each
one as a gap. A `delivered` capability binds at least one code path and one test.

**Checks.** `check` reports every violation of these rules, each naming its file:

- the schemas and closed key sets of every document;
- ids that equal their file names, and references that exist;
- an acyclic `depends` graph;
- links that resolve, including heading anchors;
- bindings that resolve through the source index;
- generated blocks that match what `docs sync` writes (see
  [generated documents](./workplan-generated-docs.md));
- PROPOSED blocks that name an existing change which lists their capability;
- a `delivered` capability with no PROPOSED block and no empty required section;
- a `considered` capability with its candidate contract in PROPOSED blocks.

## Usage

A project runs the checks from a test with the dev-dependency API:

```rust
// tests/workplan.rs
#[test]
fn workplan() {
    raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR")).assert_clean();
}
```

`Gate::workspace` loads `docs/` and `work/` under the manifest directory and the
workspace sources, runs the checks, and `assert_clean` panics with every violation
when there is one. The `cargo workplan` binary exposes the same checks and the change
lifecycle ([command line](./workplan-cli.md)).

## Examples

A change to a delivered capability, shown as the capability document reads while the
change is open. The note before the quote is plain text; the quote is the proposal.

```markdown
## Contract

The parser accepts a complete input and returns its tree.

> [!PROPOSED change:incremental-parsing]
> The parser also accepts input in arbitrary chunks through `push` and `finish`.
```

and the matching change record frontmatter:

```yaml
type: Change
id: change:incremental-parsing
title: "Add incremental parsing"
description: "Chunked input for the streaming parser."
changes: [cap:streaming-parser]
affects:
  - cap:streaming-parser#contract
  - cap:streaming-parser#usage
  - cap:streaming-parser#verification
contract_effect: changes
items:
  - claim: "Input may arrive in arbitrary chunks."
```

## Constraints

- Types, keys and values are exactly those of the profile page; this model adds
  rules between documents and none inside one.
- A change names at least one capability, and every `affects` slug is in `changes`.
- Outside PROPOSED blocks, text states the current contract. A statement about
  behaviour that is not built belongs in a `designed` or `considered` capability or
  inside a PROPOSED block, never in a delivered contract.
- Directories are flat: a document sits directly in its directory.
- A state does not advance on its own. The state of a capability is edited in the
  same change that earns it, and the check rules above reject a state its content
  does not support.

## Explanation

The pattern this model removes is the reader who has to apply amendments in their
head: the parser behaves as one decision says, except for a second decision, with an
error-handling adjustment from a third. The current documentation instead states the
complete contract that applies, and the historical sequence stays in Git, where
reading it is a deliberate investigation.

Each capability gets a permanent home because the work items that create or change it
are execution concerns, and the capability has a continuing identity. The roadmap
follows from the states: a considered or designed capability is upcoming work, an
implementing capability with unresolved bindings shows its gap, and a delivered one
is in the catalogue. A completed requirements document does not remain beside a
second, partly updated description of what was built.

Design begins in the shape of the eventual documentation. If the first design were
written as a proposal for approval, turning it into documentation would be a second
writing project, and that project is easy to neglect. With seven fixed sections from
the first draft, uncertainty is marked and resolved in place. Promotion to
`delivered` requires reconciliation, not just a status edit: the implementation may
have changed the API, a guarantee may have proved impractical, and an attractive
option may have been dropped. Those outcomes are written into the current text before
the state moves. Design material maps to outcomes in the same way: a stated need
becomes a documented guarantee with linked tests, an open question becomes an
explicit contract with a short explanation of the choice, a sketched API becomes a
real reference with a compiling example, and a possibility that was not pursued
becomes either a stated limitation or a claim with the outcome `dropped` or
`moved:change:<slug>`. A claim is never left open as an outcome. Normal links between current topics
remain useful: the reader follows the structure of the system and not its
chronology.

The graph keeps intention, implementation and documentation aligned in both
directions. It follows the discipline of the GMEOW Grounding Contract: a design
promise needs an accounted-for realisation, and built functionality needs
documentation that follows it. Documentation maintenance becomes work the system can
identify. If a referenced API disappears, an example stops compiling or a criterion
changes, the affected section receives an explicit obligation
([reconcile](./workplan-reconcile.md)).

## Verification

- Schema fixtures for the Capability and Change documents, one per failing rule,
  reuse the document-level fixtures of the OKF reader.
- A state fixture per state: each accepted with matching content, and each rejected
  with a mismatch, such as a `delivered` capability holding a PROPOSED block or a
  `considered` capability holding none.
- Marker fixtures: a block naming a missing change, a block whose change does not
  list the capability, a block in an Architecture document, and a marker inside a
  code fence that is not read as a block.
- Graph fixtures: an acyclic graph passes, a two-node and a three-node `depends`
  cycle fail, and a `changes` entry for a missing capability fails.
- Binding fixtures over a `MemoryTree` source: a symbol, a re-exported symbol, a test
  and an example resolve; a renamed symbol, a missing test and an example target that
  does not exist fail with the binding named.
- The dev-dependency API: `Gate::workspace(...).assert_clean()` passes on a clean
  fixture repository and panics with every violation on a broken one.
