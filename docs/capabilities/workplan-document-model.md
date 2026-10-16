---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-document-model
title: "Workplan document model"
description: "Capability and change documents, the four states, PROPOSED blocks, the graph, binding resolution and the check rules."
state: implementing
bindings:
  code:
    - raf_workplan::Workplan
    - raf_workplan::Workplan::load
    - raf_workplan::Workplan::check
    - raf_workplan::Workplan::status
    - raf_workplan::Finding
    - raf_workplan::Graph
    - raf_workplan::Graph::code_of
    - raf_workplan::Graph::capabilities_of
    - raf_workplan::Graph::tests_for
    - raf_workplan::Graph::sections_of
    - raf_workplan::Graph::linking_sections
    - raf_workplan::Roadmap
    - raf_workplan::LinkContext
    - raf_workplan::Gate
    - raf_workplan::Gate::workspace
    - raf_workplan::Gate::assert_clean
    - raf_workplan::plain::check
    - raf_workplan::refs::banned_in_line
    - raf_workplan::Workplan::evidence
    - raf_workplan::Workplan::assessments
    - raf_workplan::Workplan::coverage
    - raf_workplan::Graph::evidence_of
    - raf_workplan::Graph::claims_on
    - raf_workplan::Graph::assessments_of
    - raf_workplan::Graph::inputs_of
    - raf_workplan::digest::Digest
    - raf_workplan::digest::Digest::section
    - raf_workplan::digest::Digest::symbol
    - raf_workplan::digest::Digest::record
    - raf_workplan::record::Record
    - raf_workplan::record::create
    - raf_workplan::record::replace
  tests:
    - raf_workplan::test_crate::documents::accepts_valid_fixture
    - raf_workplan::test_crate::documents::the_model_holds_typed_values_and_bodies
    - raf_workplan::test_crate::documents::a_broken_document_is_a_finding_and_not_in_the_model
    - raf_workplan::test_crate::documents::rejects_type_in_the_wrong_directory
    - raf_workplan::test_crate::documents::rejects_nested_documents
    - raf_workplan::test_crate::documents::rejects_capability_without_frontmatter
    - raf_workplan::test_crate::documents::rejects_proposed_marker_in_architecture
    - raf_workplan::test_crate::rules::every_rule_has_a_fixture_reported_with_file_and_location
    - raf_workplan::test_crate::parse_errors::every_reader_error_has_a_fixture_reported_with_file_and_line
    - raf_workplan::test_crate::kinds::every_rule_and_every_reader_error_has_its_own_kind
    - raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind
    - raf_workplan::test_crate::states::accepts_each_state_with_matching_content
    - raf_workplan::test_crate::states::rejects_considered_without_proposed_block
    - raf_workplan::test_crate::states::rejects_designed_with_proposed_block
    - raf_workplan::test_crate::states::rejects_delivered_with_proposed_block
    - raf_workplan::test_crate::states::rejects_delivered_with_open_change_record
    - raf_workplan::test_crate::states::rejects_each_non_delivered_state_without_an_open_record
    - raf_workplan::test_crate::states::rejects_orphan_proposed_marker
    - raf_workplan::test_crate::states::rejects_proposed_change_that_does_not_list_the_capability
    - raf_workplan::test_crate::states::rejects_proposed_section_missing_from_affects
    - raf_workplan::test_crate::states::a_marker_in_a_code_fence_is_not_read_as_a_block
    - raf_workplan::test_crate::states::surfaces_the_single_document_state_rules_from_the_profile
    - raf_workplan::test_crate::references::accepts_a_depends_on_an_existing_capability_and_an_acyclic_graph
    - raf_workplan::test_crate::references::rejects_depends_missing
    - raf_workplan::test_crate::references::rejects_changes_missing
    - raf_workplan::test_crate::references::rejects_moved_missing
    - raf_workplan::test_crate::references::rejects_depends_cycles_once_for_each_group_naming_its_members
    - raf_workplan::test_crate::references::a_slug_id_that_differs_from_the_file_name_is_refused
    - raf_workplan::test_crate::graph::the_graph_has_every_kind_of_node
    - raf_workplan::test_crate::graph::the_graph_has_every_kind_of_edge
    - raf_workplan::test_crate::graph::links_run_from_the_section_that_holds_them_to_the_section_they_name
    - raf_workplan::test_crate::graph::code_of_lists_the_items_that_realise_a_capability_in_path_order
    - raf_workplan::test_crate::graph::capabilities_of_names_every_capability_bound_to_a_symbol
    - raf_workplan::test_crate::graph::tests_for_a_binding_named_in_two_sections_substantiates_both
    - raf_workplan::test_crate::graph::sections_of_a_change_are_its_affected_sections_sorted
    - raf_workplan::test_crate::graph::linking_sections_are_section_granular
    - raf_workplan::test_crate::links::rejects_broken_relative_link
    - raf_workplan::test_crate::links::rejects_broken_anchor
    - raf_workplan::test_crate::links::rejects_link_to_a_tracked_symlink
    - raf_workplan::test_crate::links::rejects_link_leaving_the_repository
    - raf_workplan::test_crate::links::anchors_follow_github_rules
    - raf_workplan::test_crate::links::the_oracle_is_the_tracked_set_and_not_the_tree
    - raf_workplan::test_crate::refs::the_matcher_bans_references_and_process_wording
    - raf_workplan::test_crate::refs::the_matcher_spares_colours_prose_numbers_and_near_misses
    - raf_workplan::test_crate::refs::every_document_type_and_the_frontmatter_are_scanned
    - raf_workplan::test_crate::bindings::bindings_that_name_real_symbols_resolve
    - raf_workplan::test_crate::bindings::a_binding_through_a_re_export_resolves_to_its_definition
    - raf_workplan::test_crate::bindings::a_binding_to_a_missing_path_is_reported_with_its_capability
    - raf_workplan::test_crate::bindings::a_binding_to_a_macro_generated_function_is_not_found
    - raf_workplan::test_crate::bindings::a_path_two_globs_both_bring_in_is_ambiguous_and_names_both_definitions
    - raf_workplan::test_crate::bindings::a_tests_binding_must_name_a_function_marked_test
    - raf_workplan::test_crate::bindings::a_code_binding_must_name_a_shipping_item
    - raf_workplan::test_crate::bindings::an_examples_binding_is_an_example_target_or_a_documented_library_item
    - raf_workplan::test_crate::bindings::a_missing_example_target_is_reported_with_its_capability_list_and_binding
    - raf_workplan::test_crate::roadmap::the_groups_are_in_catalogue_order_and_sorted_by_slug
    - raf_workplan::test_crate::roadmap::an_implementing_capability_lists_its_gaps_exactly
    - raf_workplan::test_crate::roadmap::this_repository_lists_every_capability_once_in_its_state_group
    - raf_workplan::test_crate::gate::a_clean_repository_passes
    - raf_workplan::test_crate::gate::a_broken_repository_panics_listing_every_finding
    - raf_workplan::test_crate::gate::a_missing_capabilities_directory_is_an_error
    - raf_workplan::test_crate::gate::a_capabilities_directory_without_a_capability_document_is_an_error
    - raf_workplan::test_crate::gate::a_missing_changes_directory_is_the_empty_set_of_changes
    - raf_workplan::test_crate::gate::inside_a_git_checkout_the_tracked_paths_come_from_the_index
    - raf_workplan::test_crate::plain::reads_markdown_outside_the_document_directories_with_or_without_frontmatter
    - raf_workplan::test_crate::plain::a_broken_link_and_a_split_link_are_reported_together_for_a_plain_file
    - raf_workplan::test_crate::generated::a_body_that_mixes_line_endings_is_refused_by_sync_and_reported_by_check
    - raf_workplan::test_crate::records::each_record_type_loads_from_its_directory_into_the_model
    - raf_workplan::test_crate::records::a_repository_without_record_directories_has_no_records_and_no_new_findings
    - raf_workplan::test_crate::records::work_coverage_holds_only_the_coverage_record
    - raf_workplan::test_crate::records::a_record_that_names_what_does_not_exist_is_a_finding_and_its_neighbour_is_clean
    - raf_workplan::test_crate::records::a_claim_on_unknown_evidence_names_the_file_and_the_line
    - raf_workplan::test_crate::records::a_claim_on_evidence_for_another_capability_is_a_finding
    - raf_workplan::test_crate::records::claims_are_section_granular_and_one_edge_per_named_record
    - raf_workplan::test_crate::records::the_records_are_nodes_with_their_edges
    - raf_workplan::test_crate::records::each_input_of_a_record_that_resolves_is_an_edge_and_one_that_does_not_is_no_finding
    - raf_workplan::test_crate::records::replace_renames_a_new_file_over_the_target_and_never_writes_into_it
    - raf_workplan::test_crate::records::a_written_record_reloads_with_no_finding_as_the_same_value
    - raf_workplan::test_crate::records::create_never_replaces_a_file
    - raf_workplan::test_crate::records::replace_puts_new_text_in_place_of_an_existing_record_only
    - raf_workplan::test_crate::records::a_record_that_would_not_load_clean_is_not_written
    - raf_workplan::test_crate::records::a_hundred_thousand_claims_load_in_time_linear_in_their_number
    - raf_workplan::test_crate::digest::each_kind_hashes_its_documented_encoding
    - raf_workplan::test_crate::digest::a_digest_is_the_same_on_every_run
    - raf_workplan::test_crate::digest::crlf_and_lf_give_the_same_digest
    - raf_workplan::test_crate::digest::a_regenerated_block_leaves_the_section_digest_and_an_edit_moves_it
    - raf_workplan::test_crate::digest::every_definition_of_a_path_counts_once_in_file_order
    - raf_workplan::digest::tests::sha256_gives_the_standard_test_vectors
  examples:
    - raf_workplan::Gate
    - raf_workplan::LinkContext
    - raf_workplan::plain::check
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
[the RAF profile of OKF](../architecture/okf-profile.md) (its
[section table](../architecture/okf-profile.md#body-rules) says what each section holds) and read by
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

A change to a delivered capability puts the capability back under an open record,
which moves it to `implementing`: the current account stays as written, and the
proposed text sits beside it in PROPOSED blocks. Everything that moves a capability
between states is a change.

**PROPOSED blocks.** Text that a change proposes is written into the capability
document, marked as a block quote whose first line names the change:

```markdown
> [!PROPOSED change:incremental-parsing]
> The parser accepts input in arbitrary chunks and reports incomplete input when
> the caller finishes the stream.
```

The block continues while lines keep the quote marker, and ends at the first line
without it, at the next marker, or at a fenced code block. The branch diff therefore
shows the code and the documentation it produces together. Outside PROPOSED
blocks a document states the current contract only. A PROPOSED block is valid only in
a Capability document, only when `work/changes/<slug>.md` exists, only when that
record lists the capability, and only when the record's `affects` names the section
that holds the block. A `delivered` or `designed` capability has none, and a
`considered` capability has at least one.

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

**Loading.** `Workplan::load` reads `docs/capabilities`, `docs/architecture` and
`work/changes` through a source tree, validates each document with the OKF reader
and keeps the typed documents and their bodies. A document that does not validate is
a finding and is not in the model; nothing is skipped and nothing panics. Links are
judged against the tracked paths of a `LinkContext` (`Workplan::load_with`);
`Workplan::load` takes every file of the tree as tracked. `Workplan::check` returns
every finding. A `Finding` has a stable kebab-case kind
from `Finding::KINDS`, the file, a location (a key, a section or a line) and a fix;
each rule of the profile and each reader error has its own kind.

**The graph.** `raf-workplan` builds a graph (`Workplan::graph`) from the documents
and the source.

- Nodes: capabilities, sections, changes, architecture pages, proposals, and the
  code symbols, tests and examples that bindings name.
- Edges: bindings (capability to symbol, test or example), `depends`, `changes`,
  `affects` and Markdown links. A link runs from the section that holds it to the
  section its anchor names, or to the document when it has no anchor.

The graph must have no `depends` cycle. It answers questions in both directions:
which code realises this capability (`Graph::code_of`), which capabilities a symbol
is bound to (`Graph::capabilities_of`), which tests address a section
(`Graph::tests_for`), which sections a change must update (`Graph::sections_of`),
and which sections link to a section (`Graph::linking_sections`).

**Binding resolution.** Each entry in `bindings.code` is a Rust path to a shipping
item, each entry in `bindings.tests` a path to a function marked `#[test]`, and each
entry in `bindings.examples` an example target (`<package>::example_crate::<name>`)
or a documented item of a library whose documentation tests run. All resolve through
the [source index](./source-index.md), including re-exports, and the role is judged
on the definition the path leads to. A binding that does not resolve, that names more
than one definition, or whose item does not fit its list is an error naming the
capability, the list and the binding. A symbol that a macro generates is not in the
index, so it cannot be bound. A capability with nothing built binds nothing, so
`designed` capabilities carry empty lists. An `implementing` capability may have
empty or missing bindings; `status` and the catalogue report each one as a gap;
[reconcile](./workplan-reconcile.md) defines the obligation it becomes. A
`delivered` capability binds at least one code path and one test.

**Status.** `Workplan::status` returns the `Roadmap`: every capability that loaded,
grouped by state in the order `delivered`, `implementing`, `designed`, `considered`
and sorted by slug inside a group. Each entry carries its state, the open changes
that list it and, for an `implementing` capability, its gaps: the binding lists that
are empty and the bindings that do not resolve.

**Checks.** `check` reports every violation of these rules, each naming its file:

- the schemas and closed key sets of every document;
- flat document directories, each document in the directory of its type, and no
  Markdown file without frontmatter beside the documents;
- ids that equal their file names, and references that exist;
- an acyclic `depends` graph;
- links that resolve inside the repository against the tracked paths, including
  heading anchors, and no link to a tracked symlink;
- no issue or pull request reference and no development-process wording, by the
  one pattern set of `raf_workplan::refs` that every consumer's `check` applies;
- a document whose body mixes CRLF and LF line endings, which is an error;
- bindings that resolve through the source index in their roles;
- generated blocks that match what the sync operation (`Workplan::sync`) writes
  (see [generated documents](./workplan-generated-docs.md));
- PROPOSED blocks that name an existing change which lists their capability and
  affects their section;
- an open change record for every capability that is not `delivered`;
- a `delivered` capability with no PROPOSED block, no open change record and no
  empty required section;
- a `designed` capability with no PROPOSED block;
- a `considered` capability with its candidate contract in PROPOSED blocks.

Markdown outside the document directories, such as a README, is judged by the same
link, marker and reference rules through `raf_workplan::plain::check`.

> [!PROPOSED change:workplan-document-model]
> **Records.** `Workplan::load` also reads the records of the profile: the evidence
> records of `work/evidence`, the assessments of `work/assessments` and the coverage
> record `work/coverage/default.md`, each validated with the OKF reader and kept as a
> typed record (`Workplan::evidence`, `Workplan::assessments`, `Workplan::coverage`).
> A record directory holds its records directly, as a document directory does, and
> `work/coverage` holds only `default.md`: any other file there is
> `singleton-other-file` and is not read. A directory that does not exist holds
> nothing, so a repository without records loads and checks as it did before.
>
> The graph holds a node for each record, for each crate the coverage record
> scopes and for each symbol an input names, and five more edges: `for` from an
> evidence record to each capability it covers (`Graph::covers`,
> `Graph::evidence_of`); a claim from the section of a capability that holds a claim
> marker to each evidence record the marker names (`Graph::claims`,
> `Graph::claims_on`); the subject of an assessment, a section (`Graph::assesses`,
> `Graph::assessments_of`); `scope` from the coverage record to each crate
> (`Graph::scope`); and an input edge from a record to the section, the definitions
> of the symbol or the evidence record each of its input rows names
> (`Graph::inputs`, `Graph::inputs_of`). An input whose name resolves to nothing
> now makes no edge and is no finding of the load. A claim marker inside a PROPOSED block is not a
> claim until the block is resolved. `check` reports a claim on evidence that does
> not exist (`claim-evidence-missing`) or whose `for` does not list the capability
> (`claim-evidence-not-for`), each with the file and the line of the marker; an
> evidence record whose `for` names no capability document (`evidence-for-missing`);
> an assessment whose subject is a section of no capability document
> (`assessment-subject-missing`); and a `scope` entry that is no package of the
> source workspace (`scope-crate-missing`). A name whose document or record exists
> and is broken is explained by the findings of that file.
>
> **Digests.** `raf_workplan::digest::Digest` is the SHA-256 (FIPS 180-4, computed
> in the crate) of a canonical encoding of a record input, one or more fields
> `<kind> NUL <name> NUL <length> NUL <text>`: the kind (`section`, `symbol` or
> `record`) keeps the domains apart and the decimal length ends each field. `Digest::section` hashes
> the authored text of a section, with the content of its generated blocks left out;
> `Digest::symbol` hashes the source span of each definition a path resolves to, once
> per source item and in file order; `Digest::record` hashes the text of a record.
> Every text has CRLF read as LF, so a digest is the same on every run and platform.
> A digest is written and read as 64 lowercase hexadecimal digits.
>
> **Writing records.** `raf_workplan::record::create` writes a new `Record` file
> exclusively, never replacing a file, below the root, creating the directory of
> its type when it is missing and refusing a link or a file on the way;
> `record::replace` puts new text in place of an existing record file through a
> temporary file and a rename. Both write the text the OKF writer renders for the
> record, opened with the two SPDX comment lines of `RecordOptions` when both are
> given, and refuse, before anything is written, a record whose text would not load
> with no finding.

## Usage

A project runs the checks from a test with the dev-dependency API:

```rust
// tests/workplan.rs
#[test]
fn workplan() {
    raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR")).assert_clean();
}
```

`Gate::workspace` reads `docs/capabilities`, `docs/architecture` and `work/changes`
under the directory, loads the workspace sources from the roots `["."]`
(`Gate::source_roots` changes them), judges links against the paths `git ls-files`
lists, and runs the checks; `assert_clean` panics with every finding, each with its
location and fix, when there is one. A directory without a capability document is an
error, not a clean verdict, and a missing `work/changes` is the empty set of changes.
A command line is specified separately ([command line](./workplan-cli.md)) and is
not part of this capability.

> [!PROPOSED change:workplan-document-model]
> `Gate::workspace` also reads the records of `work/evidence`, `work/assessments`
> and `work/coverage`; a missing record directory holds no record, so a repository
> with none checks as before.

## Examples

A change to a delivered capability, shown as the capability document, in state
`implementing`, reads while the change is open. The note before the quote is plain
text; the quote is the proposal.

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
  rules between documents and the source, and the line-ending rule for the documents
  it rewrites, and restates the profile's single-document binding minimum for
  `delivered`.
- A change names at least one capability, and every `affects` slug is in `changes`.
- Outside PROPOSED blocks, text states the current contract. A statement about
  behaviour that is not built belongs in a `designed` or `considered` capability or
  inside a PROPOSED block, never in a delivered contract.
- Directories are flat: a document sits directly in its directory.
- A state does not advance on its own. The state of a capability is edited in the
  same change that earns it, and the check rules above reject a state its content
  does not support.
- Outside a git work tree, such as a copy of a commit's files, the gate takes every
  file under the directory as a tracked path, so a link to an untracked file passes
  there. `Gate::tracked_paths` supplies the list instead.

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
directions. It follows the discipline of the GMEOW
[Grounding Contract](https://github.com/Blackcat-Informatics/gmeow-ontology/blob/a631f73e6e39b4493d295bcd83b4deec80637f54/docs/GROUNDING.md):
a design promise needs an accounted-for realisation, and built functionality needs
documentation that follows it. Documentation maintenance becomes work the system can
identify: the graph shows which sections a broken binding or a failing example
touches, and the obligations those sections receive are defined by
[reconcile](./workplan-reconcile.md).

## Verification

- End-to-end fixtures for the Capability and Change documents: each rule of the OKF
  profile and each reader error surfaces through `check` with its file and location,
  and every kind in `Finding::KINDS` is produced by a named fixture.
- A state fixture per state: each accepted with matching content, and each rejected
  with a mismatch, such as a `delivered` capability holding a PROPOSED block or an
  open record, a `designed` capability holding a block, or a `considered` capability
  holding none.
- Marker fixtures: a block naming a missing change, a block whose change does not
  list the capability, a block whose section the change does not affect, a block in
  an Architecture document, and a marker inside a code fence that is not read as a
  block.
- Graph fixtures: every kind of node and edge, section-to-section links, each query,
  an acyclic graph that passes, cycles of one, two and three nodes that fail, and
  `depends`, `changes` and `moved` references to a missing document that fail.
- Link fixtures over every link form: a missing target, a missing anchor, a target
  outside the repository and a tracked symlink, judged against the tracked set and
  not the tree.
- Reference fixtures: issue and pull request numbers and process wording in every
  document type and in the frontmatter, beside hex colours and prose numbers that
  pass.
- Binding fixtures over a `MemoryTree` source: a symbol, a re-exported symbol, a test,
  an example target and a documented library item resolve; a missing path, an
  ambiguous path, a macro-generated symbol, a missing example target and an item of
  the wrong role fail with the capability, the list and the binding named.
- Roadmap fixtures: the group and slug order, the gaps of an `implementing`
  capability, and this repository with every capability once.
- The dev-dependency API: `Gate::workspace(...).assert_clean()` passes on a clean
  fixture repository on disk, panics with every finding on a broken one, and is an
  error for a directory without capability documents.

> [!PROPOSED change:workplan-document-model]
> - Record fixtures in their directories: each record type loads into the model; a
>   record in the directory of another type, under another file name or below its
>   directory is refused; a file beside the coverage record is
>   `singleton-other-file`; a repository without record directories reports nothing
>   new.
> - Reference fixtures for the records and the claims, each beside a neighbour that
>   passes: a capability in `for` with no document, a subject with no capability, a
>   `scope` crate that is no package, a claim on unknown evidence with its line, and
>   a claim on evidence for another capability; a claim inside a PROPOSED block is
>   not read. An input of each kind that resolves is an edge, a re-exported symbol
>   leads to its definition, and an input of each kind that does not resolve is no
>   edge and no finding.
> - Digest fixtures: the SHA-256 test vectors (the empty message, `abc`, the 448-bit
>   message and a million `a`), each encoding pinned against an independent SHA-256,
>   two runs equal, CRLF and LF equal, a regenerated block that leaves a section
>   digest, and each definition of a symbol counted once.
> - Writer fixtures: a written record of each type reloads with no finding as the
>   same value; `create` never replaces a file; `replace` needs a record file that
>   exists, renames a new file over it, and in a directory that refuses a new file
>   fails with the target unchanged and no temporary file left; a record that would
>   not load clean, a bad slug and half an SPDX header are refused with nothing
>   written.
> - Cost fixtures: a record at the size bound with the most measures, and a hundred
>   thousand claims, load in time linear in their size.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_workplan::Gate` | doctest | [Verification](#verification) |
| `raf_workplan::LinkContext` | doctest | [Verification](#verification) |
| `raf_workplan::digest::tests::sha256_gives_the_standard_test_vectors` | test | [Verification](#verification) |
| `raf_workplan::plain::check` | doctest | [Contract](#contract) |
| `raf_workplan::test_crate::bindings::a_binding_through_a_re_export_resolves_to_its_definition` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_binding_to_a_macro_generated_function_is_not_found` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_binding_to_a_missing_path_is_reported_with_its_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_code_binding_must_name_a_shipping_item` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_missing_example_target_is_reported_with_its_capability_list_and_binding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_path_two_globs_both_bring_in_is_ambiguous_and_names_both_definitions` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_tests_binding_must_name_a_function_marked_test` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::an_examples_binding_is_an_example_target_or_a_documented_library_item` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::bindings_that_name_real_symbols_resolve` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_digest_is_the_same_on_every_run` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_regenerated_block_leaves_the_section_digest_and_an_edit_moves_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::crlf_and_lf_give_the_same_digest` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::each_kind_hashes_its_documented_encoding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::every_definition_of_a_path_counts_once_in_file_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::a_broken_document_is_a_finding_and_not_in_the_model` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::accepts_valid_fixture` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_capability_without_frontmatter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_nested_documents` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_proposed_marker_in_architecture` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_type_in_the_wrong_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::the_model_holds_typed_values_and_bodies` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_broken_repository_panics_listing_every_finding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_capabilities_directory_without_a_capability_document_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_clean_repository_passes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_missing_capabilities_directory_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_missing_changes_directory_is_the_empty_set_of_changes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::inside_a_git_checkout_the_tracked_paths_come_from_the_index` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_body_that_mixes_line_endings_is_refused_by_sync_and_reported_by_check` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::capabilities_of_names_every_capability_bound_to_a_symbol` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::code_of_lists_the_items_that_realise_a_capability_in_path_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::linking_sections_are_section_granular` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::links_run_from_the_section_that_holds_them_to_the_section_they_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::sections_of_a_change_are_its_affected_sections_sorted` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::tests_for_a_binding_named_in_two_sections_substantiates_both` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::the_graph_has_every_kind_of_edge` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::the_graph_has_every_kind_of_node` | test | [Verification](#verification) |
| `raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` | test | [Verification](#verification) |
| `raf_workplan::test_crate::kinds::every_rule_and_every_reader_error_has_its_own_kind` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::anchors_follow_github_rules` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_anchor` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_relative_link` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_link_leaving_the_repository` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_link_to_a_tracked_symlink` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::the_oracle_is_the_tracked_set_and_not_the_tree` | test | [Verification](#verification) |
| `raf_workplan::test_crate::parse_errors::every_reader_error_has_a_fixture_reported_with_file_and_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::a_broken_link_and_a_split_link_are_reported_together_for_a_plain_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::reads_markdown_outside_the_document_directories_with_or_without_frontmatter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_on_evidence_for_another_capability_is_a_finding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_on_unknown_evidence_names_the_file_and_the_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_hundred_thousand_claims_load_in_time_linear_in_their_number` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_record_that_names_what_does_not_exist_is_a_finding_and_its_neighbour_is_clean` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_record_that_would_not_load_clean_is_not_written` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_repository_without_record_directories_has_no_records_and_no_new_findings` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_written_record_reloads_with_no_finding_as_the_same_value` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::claims_are_section_granular_and_one_edge_per_named_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::create_never_replaces_a_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::each_input_of_a_record_that_resolves_is_an_edge_and_one_that_does_not_is_no_finding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::each_record_type_loads_from_its_directory_into_the_model` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::replace_puts_new_text_in_place_of_an_existing_record_only` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::replace_renames_a_new_file_over_the_target_and_never_writes_into_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::the_records_are_nodes_with_their_edges` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::work_coverage_holds_only_the_coverage_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::a_slug_id_that_differs_from_the_file_name_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::accepts_a_depends_on_an_existing_capability_and_an_acyclic_graph` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::rejects_changes_missing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::rejects_depends_cycles_once_for_each_group_naming_its_members` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::rejects_depends_missing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::rejects_moved_missing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::every_document_type_and_the_frontmatter_are_scanned` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::the_matcher_bans_references_and_process_wording` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::the_matcher_spares_colours_prose_numbers_and_near_misses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::an_implementing_capability_lists_its_gaps_exactly` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::the_groups_are_in_catalogue_order_and_sorted_by_slug` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::this_repository_lists_every_capability_once_in_its_state_group` | test | [Verification](#verification) |
| `raf_workplan::test_crate::rules::every_rule_has_a_fixture_reported_with_file_and_location` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::a_marker_in_a_code_fence_is_not_read_as_a_block` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::accepts_each_state_with_matching_content` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_considered_without_proposed_block` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_delivered_with_open_change_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_delivered_with_proposed_block` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_designed_with_proposed_block` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_each_non_delivered_state_without_an_open_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_orphan_proposed_marker` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_proposed_change_that_does_not_list_the_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_proposed_section_missing_from_affects` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::surfaces_the_single_document_state_rules_from_the_profile` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [okf-profile](./okf-profile.md) |
| Depends on | [source-index](./source-index.md) |
| Depended on by | [workplan-current-rationale](./workplan-current-rationale.md) |
| Depended on by | [workplan-entity-history](./workplan-entity-history.md) |
| Depended on by | [workplan-evidence-claims](./workplan-evidence-claims.md) |
| Depended on by | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depended on by | [workplan-prose-assessment](./workplan-prose-assessment.md) |
| Depended on by | [workplan-reconcile](./workplan-reconcile.md) |
| Depended on by | [workplan-two-way-coverage](./workplan-two-way-coverage.md) |
| Changed by | [workplan-document-model](../../work/changes/workplan-document-model.md) |
<!-- raf:generated:end -->
