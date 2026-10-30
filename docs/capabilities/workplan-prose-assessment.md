---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-prose-assessment
title: "Prose assessment"
description: "Recorded assessments of explanatory prose by humans and agents, invalidated when their inputs change."
state: delivered
bindings:
  code:
    - raf_workplan::record
    - raf_workplan::Workplan::record_assessment_in
    - raf_workplan::assessment
    - raf_workplan::Workplan::assess_context
    - raf_workplan::Workplan::assessment_record
    - raf_workplan::Workplan::record_assessment
    - raf_workplan::Workplan::list_assessments
  tests:
    - raf_workplan::test_crate::assessments::a_crate_root_assessment_is_published_reloaded_and_invalidated_by_its_source
    - raf_workplan::test_crate::assessments::an_assessment_is_never_written_through_a_linked_directory_and_a_real_one_records
    - raf_workplan::test_crate::assessments::a_bound_symbol_named_by_its_last_segment_is_an_input_beside_one_named_in_full
    - raf_workplan::test_crate::assessments::a_capability_that_does_not_load_is_refused_and_a_section_of_generated_blocks_only_does_not_load
    - raf_workplan::test_crate::assessments::a_common_word_that_is_the_last_segment_of_a_binding_does_not_narrow_the_inputs
    - raf_workplan::test_crate::assessments::a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged
    - raf_workplan::test_crate::assessments::a_method_named_by_its_type_and_name_is_an_input_and_its_sibling_is_not
    - raf_workplan::test_crate::assessments::a_record_named_otherwise_than_its_subject_is_a_finding_and_not_the_assessment_of_it
    - raf_workplan::test_crate::assessments::a_regenerated_block_leaves_the_assessment_of_its_section_current
    - raf_workplan::test_crate::assessments::a_rejected_assessment_is_current_and_owes_an_acceptance_whoever_gave_it
    - raf_workplan::test_crate::assessments::a_suffix_two_bindings_end_in_names_both
    - raf_workplan::test_crate::assessments::a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not
    - raf_workplan::test_crate::assessments::an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not
    - raf_workplan::test_crate::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input
    - raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first
    - raf_workplan::test_crate::assessments::crlf_line_endings_in_the_document_and_the_source_leave_an_assessment_current
    - raf_workplan::test_crate::assessments::each_refusal_writes_nothing_and_its_neighbour_records
    - raf_workplan::test_crate::assessments::the_context_is_the_same_for_a_human_and_an_agent_and_holds_what_the_assessor_reads
    - raf_workplan::test_crate::assessments::with_no_bound_symbol_named_every_bound_symbol_is_an_input
    - raf_workplan::assessment::tests::a_long_contract_is_read_once_and_its_symbols_are_named_in_one_pass
    - raf_workplan::assessment::tests::a_moved_input_names_its_kind_its_name_and_how_it_moved
    - raf_workplan::assessment::tests::a_section_of_generated_blocks_and_comments_only_is_not_authored_and_one_word_is
    - raf_workplan::assessment::tests::a_subject_is_a_capability_and_one_of_the_seven_sections
    - raf_workplan::assessment::tests::a_weak_name_adds_an_input_and_never_ends_the_every_binding_rule
    - raf_workplan::assessment::tests::a_whole_path_or_a_suffix_of_two_segments_is_strong_and_the_last_segment_alone_is_weak
    - raf_workplan::assessment::tests::authored_text_is_what_the_profile_calls_a_section_that_is_not_empty
    - raf_workplan::assessment::tests::the_bindings_are_tested_against_the_names_once_to_find_a_strong_one_and_once_to_name_them
    - raf_workplan::assessment::tests::the_record_slug_is_the_capability_slug_and_the_section_and_no_two_sections_share_one
    - raf_workplan::assessment::tests::widening_a_section_with_weak_names_never_removes_an_input
    - raf_workplan::test_crate::assessments::an_edit_inside_a_bound_module_moves_the_assessment_and_a_sibling_edit_does_not
    - raf_workplan::test_crate::digest::a_crate_root_stands_for_its_own_file_and_an_edit_below_it_leaves_its_digest
    - raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does
    - raf_workplan::test_crate::assessments::prose_fenced_names_and_alternate_reexports_do_not_select_an_unnamed_bound_input
    - raf_workplan::test_crate::digest::a_type_digest_excludes_method_bodies_but_its_definition_and_method_digests_move
    - raf_workplan::files::tests::a_directory_swap_after_preflight_is_not_rechecked_by_exclusive_publication
    - raf_workplan::record::tests::a_directory_swap_after_preflight_is_not_rechecked_by_replacement_publication
  examples:
    - raf_workplan::assessment::record_slug
depends: [cap:workplan-document-model, cap:workplan-generated-docs, cap:workplan-complete-gate, cap:workplan-evidence-claims]
---

# Prose assessment

## Purpose

Reference sections, relationship tables, indices and verification matrices can be
generated. Explanations and instructional prose need an author, and they need a
reviewer. This capability records what was assessed, whether a human or an agent
assessed it and against which inputs, and treats the assessment as out of date when
any of those inputs change.

## Contract

A capability with a nonempty architecture model adds its normalized declaration
digest to every authored section's assessment inputs. Explicit model source,
public-interface, test and example bindings contribute source digests; bound
capabilities contribute Contract, native binding-declaration and direct bound-source
digests without recursively expanding model references. Associated verification records and
their recorded source inputs contribute independent record/source digests.
Header edits, changed bindings, moved source or verification inputs and removal
of the last declaration make the old assessment stale. Model-free assessments
retain their existing inputs. Currency never changes an authored scenario's
claim strength.
<!-- raf:tested-by raf_workplan::test_crate::assessments::an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not -->

`record_assessment_in` accepts an injected `RecordStore`, including a native
`MemoryTree`, and shares the ordinary disk workflow. It validates assessment
inputs and schema before inspection, selects exclusive creation for an absent
destination and replacement for an existing regular file, and refuses every
other destination. Rendered text is validated before publication, which rechecks
the destination. Reloaded memory and
disk records have identical bytes and input currency; replacement retains no
old verdict or assessor. The `record` boundary retains original store causes.
<!-- raf:tested-by raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first -->

### Authored sections
<!-- raf:tested-by raf_workplan::test_crate::assessments::a_capability_that_does_not_load_is_refused_and_a_section_of_generated_blocks_only_does_not_load,raf_workplan::test_crate::assessments::a_regenerated_block_leaves_the_assessment_of_its_section_current,raf_workplan::test_crate::assessments::each_refusal_writes_nothing_and_its_neighbour_records -->

Text inside a generated block is produced by
`docs sync` and is never assessed. A section whose text outside its generated
blocks and HTML comments is blank holds nothing an author wrote: it is not
assessed and the completion gate asks for no assessment of it. Such a section
cannot occur in a capability that loads: the profile refuses it
(`okf-section-empty`), so every section of a capability that loads is authored and
may be assessed. An assessment covers a whole section, `cap:<slug>#<section>`; a
section the capability does not have, or a name that is not one of the seven
sections, is an error. The verdict is `accepted` or `rejected`, and no other.

### Crate-root inputs
<!-- raf:tested-by raf_workplan::test_crate::assessments::a_crate_root_assessment_is_published_reloaded_and_invalidated_by_its_source -->

A bound crate root can be an assessment input. Its record can be published and
reloaded, and editing the root source invalidates the recorded digest.

### Canonical assessment records
<!-- raf:tested-by raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first,raf_workplan::test_crate::assessments::a_record_named_otherwise_than_its_subject_is_a_finding_and_not_the_assessment_of_it,raf_workplan::test_crate::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input,raf_workplan::test_crate::assessments::each_refusal_writes_nothing_and_its_neighbour_records,raf_workplan::test_crate::assessments::a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged -->

An assessment is the record
`work/assessments/<capability slug>-<section>.md`, with the id
`assessment:<capability slug>-<section>`: one id for each section, so a section
has at most one assessment, and a record named otherwise than its subject is the
`check` finding `assessment-id-mismatch` and is not the assessment of the
section. The record names the section (`subject`), the verdict (`accepted` or
`rejected`), who assessed by kind only (`by: human` or `by: agent`, never a
name), and one `inputs` row for each input with the digest of its text when the
assessment was made. The inputs are the section itself, its authored text with
the content of its generated blocks left out; the bound symbols; and each
evidence record a claim marker of the section names.

### Named symbol inputs
<!-- raf:tested-by raf_workplan::test_crate::assessments::a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not,raf_workplan::test_crate::assessments::a_bound_symbol_named_by_its_last_segment_is_an_input_beside_one_named_in_full,raf_workplan::test_crate::assessments::a_common_word_that_is_the_last_segment_of_a_binding_does_not_narrow_the_inputs,raf_workplan::test_crate::assessments::a_method_named_by_its_type_and_name_is_an_input_and_its_sibling_is_not,raf_workplan::test_crate::assessments::a_suffix_two_bindings_end_in_names_both,raf_workplan::test_crate::assessments::with_no_bound_symbol_named_every_bound_symbol_is_an_input -->

A code span outside fenced
code, PROPOSED blocks and generated blocks names a `code` binding strongly when it
holds, as a whole path, the binding's path or a `::`-suffix of it of two segments
or more (`raf_workplan::Workplan::check` or `Workplan::check`), and weakly when it
holds the last segment alone (`check`), a word that may stand for something
else; a suffix that two bindings end in names both. When the section names a
binding strongly, the bound symbols that are inputs are the bindings it names
strongly or weakly; when it names none strongly, they are every `code` binding of
the capability. A weak name therefore only ever adds an input and never narrows
the set. Recording a verdict for a section that has an assessment replaces that
record, and nothing of the earlier one is kept. A bound symbol that is an input
and does not resolve, or a claim on a record that does not load, is refused,
since the record would keep no digest of it, and so is a record over the
document bound. Recording writes that one file and never edits a document.

### Assessor context
<!-- raf:tested-by raf_workplan::test_crate::assessments::the_context_is_the_same_for_a_human_and_an_agent_and_holds_what_the_assessor_reads -->

`assess_context` gives the assessor the authored text of the
section, each bound symbol that is an input with the source text of its
definitions, each evidence record its claims name with its verdict, values and
limits, and the inputs with their digests now. Nothing in it depends on who
asks, so a human and an agent read the same context.

### Input currency and reconcile obligations
<!-- raf:tested-by raf_workplan::test_crate::assessments::an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not,raf_workplan::test_crate::assessments::a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not,raf_workplan::test_crate::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input,raf_workplan::test_crate::assessments::a_rejected_assessment_is_current_and_owes_an_acceptance_whoever_gave_it,raf_workplan::test_crate::assessments::a_regenerated_block_leaves_the_assessment_of_its_section_current,raf_workplan::assessment::tests::a_moved_input_names_its_kind_its_name_and_how_it_moved -->

An assessment is current while every input it keeps has the
digest it keeps and the section has no input the record lacks. An input moved
when its digest differs now, when it no longer resolves, when it is no longer an
input (the section no longer names the symbol, the capability no longer binds it,
the claim is gone) or when something is an input now that was not then. A
regenerated block, and an edit of another section or of a symbol that is not an
input, leave the assessment current. A section in a change's `affects` whose
assessment is missing, stale or rejected receives obligations in the
[reconcile](./workplan-reconcile.md) list: `assessment-missing`, one
`assessment-stale` for each input that moved, naming it, or
`assessment-rejected`.

### Completion acceptance
<!-- raf:tested-by raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does -->

The [completion gate](./workplan-complete-gate.md) requires a
current assessment with the verdict `accepted` for each authored section that a
change lists in `affects`. An assessment by an agent satisfies it by itself, as
one by a human does; the record says which. A `rejected` verdict never satisfies
it, whoever gave it.

## Usage

Assess model-bearing capability sections through the same assessment API. Inspect
the `declaration` input and independently listed symbol/record inputs. Removing
the last model yields `NoLongerInput`; it does not silently keep an old assessment
current or add empty declaration inputs to unrelated capabilities.
<!-- raf:tested-by raf_workplan::test_crate::assessments::an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not -->

Pass a mutable `MemoryTree` or `DiskRecordStore` to `record_assessment_in` for
explicit publication. Load the model from the same store's native `Tree` view
and reload it after a write to inspect current inputs. The ordinary
`record_assessment` disk API and `assess --verdict` use the same implementation.

The runnable API example `raf_workplan::assessment::record_slug` derives the stable record name used for a prose assessment.

- `cargo workplan assess cap:workplan-reconcile#explanation --context` prints the
  section, the bound symbols, the evidence and the inputs with their digests for
  the assessor.
- `cargo workplan assess cap:workplan-reconcile#explanation --verdict accepted --by agent`
  records the assessment, replacing the one the section had, if any; given
  together, `--copyright <text>` and `--licence <id>` open the record with its
  two SPDX lines.
- `cargo workplan assess --list` shows each assessment, whether it is current,
  and the inputs that moved.

The library gives the same through `Workplan::assess_context`,
`Workplan::record_assessment` (and `Workplan::assessment_record`, which builds
the record without writing it) and `Workplan::list_assessments`.

## Examples

Changing a component responsibility invalidates the declaration input. Editing
its associated test body invalidates that symbol input even if the evidence
record did not capture the test body. Changing an associated record invalidates
the record input. A passing test still supplies no observation of a message
sequence or parallel execution.
<!-- raf:tested-by raf_workplan::test_crate::assessments::an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not -->

An accepted assessment created in memory reloads as current, just as the disk
record does. Recording the same inputs replaces it with identical bytes; a
rejected human verdict replaces the verdict and assessor without retaining the
old values or changing another file.

After reconciliation the tool gives an assessor a concrete task: `change reconcile`
lists each authored section in `affects` that owes an assessment, with the input
that moved or the verdict that stands in the way:

```text
cap:streaming-parser  usage  assessment-stale  the section assessed again with assess (`assessment:streaming-parser-usage`: the symbol `raf_parser::Parser::push` has changed since)
cap:streaming-parser  constraints  assessment-rejected  the section revised and assessed again with assess, with the verdict accepted (`assessment:streaming-parser-constraints` is current and has the verdict rejected, by human)
cap:streaming-parser  explanation  assessment-missing  an assessment of the section recorded with assess (`cap:streaming-parser#explanation` has no assessment `assessment:streaming-parser-explanation`)
```

The lines come in the order of the sections. Each is resolved by recording an
assessment, after revising the section for the rejected one, and the record stays
current only until one of its inputs moves: the section, a bound symbol that is an
input or an evidence record a claim of the section names. `assess --list` shows an
assessment that moved with the input, for example:

```text
cap:streaming-parser#usage  assessment:streaming-parser-usage  accepted  agent  stale: the symbol `raf_parser::Parser::push` has changed since
```

## Constraints

- The store writes only the selected assessment. It preserves unrelated files
  and refuses a destination that cannot be created or replaced under the store's
  rules; no alternate store is selected on failure.
  <!-- raf:tested-by raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first -->

- An assessment never edits the section it assesses; recording writes only its
  own record.
  <!-- raf:tested-by raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first -->
- Assessment publication checks its directories before writing: `work` and
  `work/assessments` must each be a directory, or be missing and then be created,
  and a symbolic link there, inside the root or out of it, a file or anything else
  that is not a directory is refused at this preflight. Publication uses paths,
  without holding directory handles; a local writer that swaps a directory after
  preflight is outside the same local-writer trust boundary as
  [evidence publication](./workplan-evidence-claims.md#constraints).
  <!-- raf:tested-by raf_workplan::files::tests::a_directory_swap_after_preflight_is_not_rechecked_by_exclusive_publication,raf_workplan::record::tests::a_directory_swap_after_preflight_is_not_rechecked_by_replacement_publication,raf_workplan::test_crate::assessments::an_assessment_is_never_written_through_a_linked_directory_and_a_real_one_records,raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first -->
- Generated blocks are excluded from assessment, so a regenerated table does not
  invalidate the assessment of the prose around it.
  <!-- raf:tested-by raf_workplan::test_crate::assessments::a_regenerated_block_leaves_the_assessment_of_its_section_current -->
- The digest is deterministic: the same inputs always produce the same digest,
  and recording the same verdict on the same inputs writes the same bytes.
  <!-- raf:tested-by raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first,raf_workplan::test_crate::assessments::crlf_line_endings_in_the_document_and_the_source_leave_an_assessment_current -->
- Humans and agents receive the same context, and the record states which of them
  assessed, by kind and never by name.
  <!-- raf:tested-by raf_workplan::test_crate::assessments::the_context_is_the_same_for_a_human_and_an_agent_and_holds_what_the_assessor_reads -->
- A weak name, the last segment of a binding alone, never narrows the inputs: it
  adds its binding beside the others. Once a section names a bound symbol
  strongly, by its path or a suffix of two segments or more, a bound symbol it
  describes only in prose or in fenced code, or reaches through another path such
  as a re-export, is not an input; with no bound symbol named strongly, every one
  is.
  <!-- raf:tested-by raf_workplan::test_crate::assessments::prose_fenced_names_and_alternate_reexports_do_not_select_an_unnamed_bound_input,raf_workplan::test_crate::assessments::with_no_bound_symbol_named_every_bound_symbol_is_an_input,raf_workplan::assessment::tests::a_weak_name_adds_an_input_and_never_ends_the_every_binding_rule -->
- The digest of a bound symbol is the text of its own item: the binding of a type
  covers the type's definition and not the bodies of its methods, so a section
  that describes a method names the method, and the capability binds it. The
  binding of a module below a crate root stands for the module's whole content,
  its files and those of the modules below it, so any edit inside it moves the
  input. The binding of a crate root stands for its own span, the whole of its
  file, and for nothing below it, so an edit of a module file below the root
  leaves the input and an edit of the root's own file moves it.
  <!-- raf:tested-by raf_workplan::test_crate::digest::a_type_digest_excludes_method_bodies_but_its_definition_and_method_digests_move,raf_workplan::test_crate::assessments::an_edit_inside_a_bound_module_moves_the_assessment_and_a_sibling_edit_does_not,raf_workplan::test_crate::digest::a_crate_root_stands_for_its_own_file_and_an_edit_below_it_leaves_its_digest -->
- A record holds every input in one file under the document bound, so a section
  with more inputs than fit is refused rather than recorded in part.
  <!-- raf:tested-by raf_workplan::test_crate::assessments::a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged -->

## Explanation

Mechanical generation and explanatory writing have different jobs. A crate can produce
reference sections, tables, links and indices deterministically, and it can check them.
It cannot decide that an explanation is clear or that an example teaches the right
thing. Humans and agents do that work from the same structured context, and the crate
records what they assessed.

Recording the assessment alone is not enough, because prose and its inputs drift
apart. An assessment that survived a change to the code it describes would certify a
statement nobody has read since. Tying the assessment to a digest of its inputs makes
it expire exactly when it stops being true, and the expiry becomes an explicit
obligation. Documentation work becomes specific and reviewable: the tool names the
sections to rewrite and the evidence still missing, and an assessor resolves them.

## Verification

The existing repeated-assessment case compares disk and memory bytes after
creation and replacement, verifies reloaded currency and preserves all original
files. The existing command-line cases continue to publish through the disk store.
<!-- raf:tested-by raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first -->

- An assessment is current for an unchanged section and invalidated after an edit to
  the section text, naming the section; an edit of another section leaves it
  current.
- Changing a symbol the section names invalidates its assessment, naming the
  symbol, while changing a bound symbol it does not name does not; with no bound
  symbol named, a change of any bound symbol does, and a binding added or removed
  is an input that moved.
- A section that names one bound symbol by its whole path and another by its
  last segment goes stale when either changes and stays current when a bound
  symbol it does not name changes; a section whose only match is a common word,
  the last segment of a binding in a command line, keeps every bound symbol as an
  input and goes stale when another changes; a method named by its type and name
  is an input and its sibling method is not; a suffix of two segments that two
  bindings end in makes both inputs, and a last segment they share narrows
  nothing.
- Over sections and bindings that the test generates, adding code spans that hold
  only last segments never removes an input, and a weak name never ends the rule
  that, with no strong name, every bound symbol is an input.
- CRLF line endings in the document, the source and the evidence leave the
  assessment current, and an edit under them still makes it stale.
- Replacing evidence that a claim of the section names invalidates the
  assessment, naming the record, and a record that no longer loads is an input
  that does not resolve.
- A regenerated generated block leaves the neighbouring assessment current.
- The digest is identical across two runs on the same tree: a second record of
  the same verdict is the same bytes, and it replaces the first.
- The context is byte-identical for a human and an agent, and the two records
  differ only in `by`.
- A record named otherwise than its subject is a `check` finding and not the
  assessment of the section; a subject not written `cap:<slug>#<section>`, a
  section that is not one of the seven, an unknown capability, a bound symbol
  that does not resolve and a claim on a record that does not load are each
  refused with nothing written, beside a neighbour that records; one SPDX field
  alone is refused and the two together are written.
- A section of ten thousand named symbols and one near the document bound are
  recorded and judged. The work is counted, not timed: a Contract of twelve
  thousand lines is read once, line by line, and each bound symbol is tested
  against the names of the section once to find whether any is named strongly and
  once more to say how it is named. The record of ten thousand named symbols is
  over the document bound and is refused with nothing written, and the record of
  eight thousand is written, is current, and goes stale naming the one symbol
  edited.
- A subject is a capability and one of the seven sections, and its record slug is
  the capability slug and the section, which no two sections share; a capability
  that does not load is refused; a section of generated blocks and comments only
  is not authored, and one word outside them is.
- A moved input names its kind, its name and how it moved.
- The documentation examples of `Workplan::assess_context` (a section that is not
  one of the seven and a capability that does not exist are refused) and
  `assessment::record_slug` run.
- The completion gate rejects a change whose affected authored section has no
  current assessment, or a current one that is `rejected` by a human or an agent.
- A directory link inside the root at `work` or at `work/assessments`, with the
  record of the section behind it, and a file where `work/assessments` belongs, are
  each refused with the tree byte-identical, and the same assessment through the
  real directories replaces the record.
- The digest of a crate-root binding is unchanged by an edit in the file of a
  module below the root and by one in the file of a module below that, while both
  edits move the digest of the module below the root, whose content holds both
  files, and an edit of the root's own file moves the root's digest.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_workplan::assessment::record_slug` | doctest | [Usage](#usage) |
| `raf_workplan::assessment::tests::a_long_contract_is_read_once_and_its_symbols_are_named_in_one_pass` | test | [Verification](#verification) |
| `raf_workplan::assessment::tests::a_moved_input_names_its_kind_its_name_and_how_it_moved` | test | [Contract](#contract) |
| `raf_workplan::assessment::tests::a_section_of_generated_blocks_and_comments_only_is_not_authored_and_one_word_is` | test | [Verification](#verification) |
| `raf_workplan::assessment::tests::a_subject_is_a_capability_and_one_of_the_seven_sections` | test | [Verification](#verification) |
| `raf_workplan::assessment::tests::a_weak_name_adds_an_input_and_never_ends_the_every_binding_rule` | test | [Constraints](#constraints) |
| `raf_workplan::assessment::tests::a_whole_path_or_a_suffix_of_two_segments_is_strong_and_the_last_segment_alone_is_weak` | test | [Verification](#verification) |
| `raf_workplan::assessment::tests::authored_text_is_what_the_profile_calls_a_section_that_is_not_empty` | test | [Verification](#verification) |
| `raf_workplan::assessment::tests::the_bindings_are_tested_against_the_names_once_to_find_a_strong_one_and_once_to_name_them` | test | [Verification](#verification) |
| `raf_workplan::assessment::tests::the_record_slug_is_the_capability_slug_and_the_section_and_no_two_sections_share_one` | test | [Verification](#verification) |
| `raf_workplan::assessment::tests::widening_a_section_with_weak_names_never_removes_an_input` | test | [Verification](#verification) |
| `raf_workplan::files::tests::a_directory_swap_after_preflight_is_not_rechecked_by_exclusive_publication` | test | [Constraints](#constraints) |
| `raf_workplan::record::tests::a_directory_swap_after_preflight_is_not_rechecked_by_replacement_publication` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::a_bound_symbol_named_by_its_last_segment_is_an_input_beside_one_named_in_full` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::a_capability_that_does_not_load_is_refused_and_a_section_of_generated_blocks_only_does_not_load` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::a_common_word_that_is_the_last_segment_of_a_binding_does_not_narrow_the_inputs` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::a_crate_root_assessment_is_published_reloaded_and_invalidated_by_its_source` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::a_method_named_by_its_type_and_name_is_an_input_and_its_sibling_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::a_record_named_otherwise_than_its_subject_is_a_finding_and_not_the_assessment_of_it` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::a_regenerated_block_leaves_the_assessment_of_its_section_current` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::a_rejected_assessment_is_current_and_owes_an_acceptance_whoever_gave_it` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::a_suffix_two_bindings_end_in_names_both` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::an_assessment_is_never_written_through_a_linked_directory_and_a_real_one_records` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::an_edit_inside_a_bound_module_moves_the_assessment_and_a_sibling_edit_does_not` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
| `raf_workplan::test_crate::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::crlf_line_endings_in_the_document_and_the_source_leave_an_assessment_current` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::each_refusal_writes_nothing_and_its_neighbour_records` | test | [Contract](#contract) |
| `raf_workplan::test_crate::assessments::prose_fenced_names_and_alternate_reexports_do_not_select_an_unnamed_bound_input` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::the_context_is_the_same_for_a_human_and_an_agent_and_holds_what_the_assessor_reads` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::assessments::with_no_bound_symbol_named_every_bound_symbol_is_an_input` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does` | test | [Contract](#contract) |
| `raf_workplan::test_crate::digest::a_crate_root_stands_for_its_own_file_and_an_edit_below_it_leaves_its_digest` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::digest::a_type_digest_excludes_method_bodies_but_its_definition_and_method_digests_move` | test | [Constraints](#constraints) |
<!-- raf:generated:end -->
