---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-reconcile
title: "Change reconciliation"
description: "change new, change impact and change reconcile: opening a change, listing what it touches, and producing the documentation obligations."
state: implementing
bindings:
  code:
    - raf_workplan::Workplan::new_change
    - raf_workplan::lifecycle::NewChangeRequest
    - raf_workplan::lifecycle::NewChange
    - raf_workplan::lifecycle::NewChangeError
    - raf_workplan::Workplan::impact
    - raf_workplan::lifecycle::Impact
    - raf_workplan::Workplan::reconcile
    - raf_workplan::obligation::Reconciled
    - raf_workplan::obligation::Obligation
    - raf_workplan::obligation::ObligationKind
    - raf_workplan::obligation::Needed
    - raf_workplan::obligation::suppressed
    - raf_workplan::obligation::tense::PHRASES
    - raf_workplan::lifecycle::Lifecycle::open
    - raf_workplan::lifecycle::Lifecycle::reconcile
    - raf_workplan::lifecycle::section_digest
    - raf_workplan::git::require_version
    - raf_workplan::git::show_prefix
    - raf_workplan::git::default_merge_base
    - raf_workplan::git::blob_at
    - raf_workplan::git::MergeBase
  tests:
    - raf_workplan::test_crate::lifecycle::new_writes_one_file_with_the_keys_the_id_and_a_summary_and_it_loads_clean
    - raf_workplan::test_crate::lifecycle::a_bad_slug_is_refused_before_anything_is_written_and_a_good_one_next_to_it_is_not
    - raf_workplan::test_crate::lifecycle::an_existing_file_is_refused_and_left_byte_identical
    - raf_workplan::test_crate::lifecycle::a_capability_that_does_not_exist_is_refused
    - raf_workplan::test_crate::lifecycle::a_capability_whose_document_did_not_load_is_named_and_never_skipped
    - raf_workplan::test_crate::lifecycle::an_affects_entry_must_name_a_listed_capability_and_one_of_the_seven_sections
    - raf_workplan::test_crate::lifecycle::several_capabilities_are_listed_in_the_order_given_without_repeats
    - raf_workplan::test_crate::lifecycle::the_default_affects_is_the_contract_of_each_listed_capability
    - raf_workplan::test_crate::lifecycle::explicit_affects_are_written_as_given_and_impact_reports_them_sorted
    - raf_workplan::test_crate::lifecycle::the_guidance_follows_the_state_of_each_listed_capability
    - raf_workplan::test_crate::lifecycle::the_printed_text_holds_the_template_and_the_guidance
    - raf_workplan::test_crate::lifecycle::the_template_in_a_capability_of_each_state_gives_one_block_and_a_clean_check
    - raf_workplan::test_crate::lifecycle::both_spdx_fields_open_the_record_with_the_two_comment_lines_of_the_records_of_the_repository
    - raf_workplan::test_crate::lifecycle::one_spdx_field_without_the_other_is_an_error_that_names_the_missing_one
    - raf_workplan::test_crate::lifecycle::impact_gives_the_capabilities_sections_bindings_dependants_and_linking_sections
    - raf_workplan::test_crate::lifecycle::impact_prints_each_part_in_a_fixed_order
    - raf_workplan::test_crate::obligations::each_fault_is_owed_with_its_kind_location_and_evidence_and_fixing_it_removes_the_line
    - raf_workplan::test_crate::obligations::every_static_kind_is_raised_by_a_fixture_and_the_two_that_need_a_process_by_none
    - raf_workplan::test_crate::obligations::the_binding_obligations_are_the_gaps_of_the_roadmap
    - raf_workplan::test_crate::obligations::contract_effect_none_keeps_every_obligation_the_documents_can_show
    - raf_workplan::test_crate::obligations::equal_inputs_print_equal_output
    - raf_workplan::test_crate::obligations::wording_that_only_looks_like_a_phrase_owes_nothing
    - raf_workplan::test_crate::obligations::text_of_a_proposed_block_is_not_read_for_wording
    - raf_workplan::test_crate::obligations::reconcile_writes_nothing
    - raf_workplan::test_crate::complete::each_kind_of_obligation_is_a_reconcile_line_in_a_git_repository_and_fixing_it_removes_the_line
    - raf_workplan::test_crate::untouched::an_unchanged_affected_section_is_owed_with_the_line_the_contract_shows
    - raf_workplan::test_crate::untouched::a_section_edited_on_the_branch_is_not_reported_and_its_neighbour_is
    - raf_workplan::test_crate::untouched::an_edit_that_is_not_committed_counts_because_the_working_tree_is_compared
    - raf_workplan::test_crate::untouched::only_a_generated_block_changed_is_still_reported
    - raf_workplan::test_crate::untouched::a_file_absent_at_the_merge_base_counts_as_touched
    - raf_workplan::test_crate::untouched::on_the_default_branch_itself_every_affected_section_is_owed_and_it_is_not_an_error
    - raf_workplan::test_crate::untouched::a_linked_work_tree_compares_its_own_head
    - raf_workplan::test_crate::untouched::a_workspace_below_the_top_of_the_repository_is_compared_at_its_prefix
    - raf_workplan::test_crate::untouched::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one
    - raf_workplan::test_crate::untouched::a_default_branch_named_like_an_option_is_resolved_and_never_reaches_git_as_one
    - raf_workplan::test_crate::untouched::no_default_branch_is_an_error_that_names_the_fix_and_the_fix_clears_it
    - raf_workplan::test_crate::untouched::a_file_over_one_mebibyte_at_the_merge_base_is_an_error_and_exactly_one_is_read
    - raf_workplan::test_crate::untouched::a_file_at_the_merge_base_that_does_not_parse_is_an_error_and_not_a_pass
    - raf_workplan::test_crate::untouched::contract_effect_none_leaves_out_contract_usage_and_examples_and_asks_git_for_nothing_then
    - raf_workplan::test_crate::untouched::git_2_29_stops_the_open_before_any_other_process_and_2_30_goes_on
    - raf_workplan::test_crate::untouched::the_digest_leaves_out_generated_blocks_on_both_sides_and_nothing_else
    - raf_workplan::test_crate::untouched::reconcile_writes_nothing_and_prints_the_same_list_twice
    - raf_workplan::test_crate::untouched::stubs::reconcile_asks_git_exactly_the_commands_of_the_contract_in_order
    - raf_workplan::test_crate::untouched::stubs::a_root_below_the_top_reads_each_file_at_the_prefix
    - raf_workplan::test_crate::untouched::stubs::a_prefix_the_source_path_pattern_cannot_carry_is_an_error_and_not_a_guess
    - raf_workplan::test_crate::untouched::stubs::a_stub_git_2_29_runs_one_process_and_exits_with_the_error
    - raf_workplan::test_crate::git::stubs::every_question_reaches_git_as_exactly_its_argv_in_the_work_directory
    - raf_workplan::test_crate::verify::an_example_that_does_not_build_is_a_line_and_is_gone_once_it_builds_and_runs
    - raf_workplan::test_crate::verify::a_binding_to_a_function_with_no_test_reports_zero_tests_as_failing
    - raf_workplan::test_crate::verify::test_bindings_are_not_run_by_reconcile_and_unresolved_examples_are_not_started
    - raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present
    - raf_workplan::test_crate::scenarios::planning_produces_the_documentation
    - cargo_workplan::test_crate::reconcile::a_list_with_obligations_is_printed_one_line_each_and_exits_one
    - cargo_workplan::test_crate::reconcile::an_empty_list_prints_nothing_and_exits_zero
    - cargo_workplan::test_crate::reconcile::fixing_an_obligation_removes_its_line_and_only_its_line
    - cargo_workplan::test_crate::new::the_printed_steps_end_in_a_clean_check_and_skipping_the_stage_step_does_not
    - cargo_workplan::test_crate::process::a_git_older_than_2_30_is_exit_two_before_any_other_process
  examples:
    - raf_workplan::Workplan::new_change
    - raf_workplan::Workplan::impact
    - raf_workplan::Workplan::reconcile
    - raf_workplan::lifecycle::section_digest
depends: [cap:workplan-document-model, cap:workplan-generated-docs]
---

# Change reconciliation

## Purpose

Reconciling a change means folding it into the current account of the system. The
operation takes an open change and produces a precise list of what the documentation
still owes, each with the evidence that would settle it. A person or an agent then has
a concrete writing task rather than a vague instruction to update the docs.

## Contract

**`change new <slug> --capability cap:<name>`** creates `work/changes/<slug>.md` with
the identity `change:<slug>`, the named capability in `changes`, `contract_effect:
changes`, an empty `items` list, a title and a description for the author to replace,
and a `## Summary` stub. It fails when the slug is not kebab case, the file exists, or
the capability does not exist or its document did not load. Repeating `--capability`
lists several capabilities, and one given twice is listed once. Two further options
shape the record:

- `--affects cap:<name>#<section>`, repeatable, names a section the change affects.
  Each entry names a listed capability and one of the seven sections, or the command
  fails. Without the option the record affects `cap:<name>#contract` for each listed
  capability.
- `--copyright <text>` and `--licence <id>`, given together or not at all, open the
  frontmatter with the two SPDX comment lines, so a repository that checks the
  licence of every file accepts the record as written. A value that is empty or not
  one line of text is refused.

`change new` writes exactly that one file, created exclusively and never over an
existing file or link, and reads it back through the loader; a record that has a
finding is removed again and reported. It then prints the PROPOSED block template,
`> [!PROPOSED change:<slug>]` and one placeholder line, for the author to place in a
section named in `affects`, and the state guidance for each listed capability: a
`delivered` or `designed` capability is set to `state: implementing` in the same
commit as the block, because a PROPOSED block is a `check` finding in both states (and
beside a `delivered` capability the open record is one too); a `considered` or
`implementing` capability needs no change of state. When the new record makes a
generated block stale (the relationship tables name the open changes that list a
capability), it also prints the two steps that bring the blocks up to date:
`git add work/changes/<slug>.md`, then `docs sync`. Until the record is tracked,
`check` reports the link to it as `link-broken`.

**`change impact <slug>`** reads the record and the graph and prints, without
modifying anything: the capabilities in `changes`; the sections in `affects`; the
bindings of those capabilities; the capabilities that depend on them and the sections
that link to the affected sections, because those readers are affected too.

**`change reconcile <slug>`** prints the obligation list. An obligation is a
statement of work the documentation owes, with a kind, a location and the evidence
needed. The kinds are:

| Kind | Raised when | Evidence that resolves it |
| --- | --- | --- |
| `unresolved-proposal` | A PROPOSED block names the change | The block is replaced by plain contract text, or removed because the claim was dropped |
| `item-without-outcome` | An item has no `outcome` | An outcome of `implemented`, `dropped` or `moved:change:<slug>` |
| `binding-missing` | A capability the change lists binds nothing where it should, or a binding does not resolve | A binding that resolves through the source index |
| `section-untouched` | A section in `affects` has the same text as at the merge base | A diff of that section against the merge base, or an edit that records why none is needed |
| `example-failing` | A bound example target or documentation test of a capability the change lists fails to build or run | A passing `verify` run for the change, or a binding edit that removes the broken example |
| `generated-stale` | A generated block differs from what `docs sync` writes | A `docs sync` run |
| `tense-mismatch` | Delivered or implementing contract text uses proposal or forward-looking wording, or a delivered capability has a PROPOSED block | Text rewritten as the supported behaviour |

Each obligation line gives its subject (a capability, the change record or the
capability index), the part it is about (the document, a section or an item of the
record), the kind and the evidence, and for text findings `line <n>:` before the
evidence. The list is sorted by subject (the capabilities, then the change record,
then the index, each by id), part (the document, then the seven sections in the order
a capability holds them, then the items), kind (in the order of the table), line and
evidence text, so equal inputs print equal output. The exit code is 0 when the list
is empty and 1 when it is not.

**Merge base.** `section-untouched` compares each affected section in the working
tree against the version of the file at the merge base of `HEAD` and the default
branch, found by running `git`. A section is touched when its authored text differs:
its heading span, with the content of every generated block left out on both sides
and line endings read as LF, so a regenerated table is not an edit, while marker
lines, fenced code and PROPOSED blocks are authored text. A file that does not exist
at the merge base, or that has no such section there, counts as touched, and a file
there that does not parse is an error. When `HEAD` is the default branch's commit,
the merge base is `HEAD` itself and every affected section is reported.

**Invocation.** `reconcile` starts `git` directly with no shell, with arguments as an
argv vector, and takes no document text onto a command line. It first runs
`git --version` and exits 2 when Git is older than 2.30, the first release
whose `rev-parse` handles `--end-of-options`. It then asks once where the workspace
root is below the top of its work tree, and opens the repository, which lists the
tracked paths as the gate of the [document model](./workplan-document-model.md)
does. The commands are, in order:

```text
git --version
git rev-parse --show-prefix
git rev-parse --is-inside-work-tree
git ls-files -z
git symbolic-ref refs/remotes/origin/HEAD
git rev-parse --verify --end-of-options <default-ref>^{commit}
git merge-base HEAD <default-commit>
git cat-file blob <merge-base>:<prefix><path>
```

`<default-ref>` is the full ref name that the first command prints, such as
`refs/remotes/origin/main`, so a local branch named `origin/main` cannot be mistaken
for it. The remote is assumed to be named `origin`. A repository where it fails has no
default branch, which is an error and not a silent pass. The ref is resolved to a
commit id by the second command and every later command receives that id, so a branch
named like an option cannot reach `git` as an option. In the last command the path
follows `<merge-base>:` inside one argument, so it cannot be read as an option.
`<path>` is a repository-relative document path from the loaded document set, and
`<prefix>` is what `git rev-parse --show-prefix` printed: empty at the top of the
work tree, or the directory of the workspace root below it, ending in `/`, so a
workspace below the top of its repository reads the file it means. A prefix that does
not match the relative-source-path pattern of the
[completion gate](./workplan-complete-gate.md) is an error, and a linked work tree is
its own top. The output of `cat-file blob` is read up to the 1 MiB document bound, and
a larger blob is an error. The standard output and the standard error of every other
`git` command are each read up to 16 MiB, and a command that writes more or runs past
300 seconds is killed and is an error.

`example-failing` builds and runs bound examples with the `--frozen` argv forms, the
patterns, the stdin, timeout and output rules, and the proof-that-it-ran rule of the
[completion gate](./workplan-complete-gate.md), for bindings that resolve in the
[source index](./source-index.md). The program is `$CARGO` when set and `cargo` from
`PATH` otherwise. The processes run the workspace's cargo and Git configuration,
which is trusted build input (see [the security policy](../../SECURITY.md)).

**`contract_effect: none`.** For a change with this effect, the obligation list is
lighter. `section-untouched` is not raised for `contract`, `usage` and `examples`,
since the change declares they are unaffected. All other kinds apply, because stale
bindings and stale generated blocks are errors regardless of the contract.

## Usage

```sh
cargo workplan change new incremental-parsing --capability cap:streaming-parser \
  --affects cap:streaming-parser#contract --affects cap:streaming-parser#usage
cargo workplan change impact incremental-parsing
cargo workplan change reconcile incremental-parsing
```

`change new` prints the PROPOSED block to place and the state each listed capability
needs. An agent receives the output of `reconcile`, writes the prose each line asks
for, runs `docs sync`, and runs `reconcile` again until the list is empty.

## Examples

Output of `reconcile` for a change that renamed an API, left the Usage section as it
was at the merge base, and still has an open proposal:

```text
cap:streaming-parser  contract  unresolved-proposal  line 31: the block is replaced by plain contract text, or removed because the claim was dropped
cap:streaming-parser  usage  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed
cap:streaming-parser  verification  binding-missing  a binding that resolves through the source index (bindings.code `raf_parser::Parser::push` does not resolve: `raf_parser::Parser::push` names nothing in the workspace)
change:incremental-parsing  item 2  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
```

The `binding-missing` line shows the shape of every obligation: the capability, the
section and the evidence that settles it. A bound example that stops compiling adds a
line of kind `example-failing` for the capability in the same form.

## Constraints

- `reconcile` and `impact` never write a file in the repository. Only `change new`
  writes, and it writes one file.
- The merge base comes from `git`; a repository with no default branch reachable from
  `HEAD` makes `section-untouched` an error and not a silent pass.
- Tense findings come from a fixed list of phrases (for example `will`, `is planned`
  and `proposed`) and are reported as findings, not as proof. The list is defined in
  the tool and applies outside code fences, code spans, HTML comments, link
  destinations, PROPOSED blocks and generated blocks; a phrase matches whole words,
  and `PROPOSED` in capitals is the name of the marker and not wording.
- `example-failing` builds and runs bound examples with `cargo`, which is the one
  process `reconcile` starts besides `git`; it still writes no file in the
  repository.
- No `git` or `cargo` process receives document text except a validated path inside
  a `<commit>:<path>` argument, or a binding that resolved in the source index.
- An obligation never says "update the docs". It names the section and the evidence.

## Explanation

The failure this prevents is the reader who must apply amendments mentally. The
current text must state the whole contract that applies, so accepting a change cannot
end at appending "supersedes" to an old statement. An amendment can be an input to
reconciliation, but it must not become a standing instruction that every subsequent
reader has to remember.

A command that accepts a change therefore identifies and updates the affected current
statements. Reconciliation may revise a contract, remove an obsolete constraint,
change an example, adjust a code binding and invalidate some evidence. Those are
consequences of one change, and the tool treats them together.

Mechanical generation and explanatory writing have different jobs. The crate can find
what is wrong, and it can regenerate tables. Explanations and instructional prose need
authorship and assessment. The obligation list is the interface between the two: it is
specific enough to be a writing task, such as "the final API differs from the proposal
in these two places, so update the usage example and the lifecycle explanation".

## Verification

- A `change new` fixture asserts the file's keys, id and Summary, and the failures for
  a bad slug, an existing file and a missing capability.
- A `change new` fixture for the options: without `--affects` the record affects the
  contract of each listed capability, the entries given are written in order, and an
  entry that names an unlisted capability or an unknown section is refused; both SPDX
  options write the two comment lines, and one without the other is refused.
- The printed template placed in a capability of each of the four states, after the
  printed state guidance, gives exactly one PROPOSED block, a clean `check` and one
  `unresolved-proposal` line; and after a record that stales a generated block, the
  printed steps end in a clean `check`.
- A `change impact` fixture asserts the printed capabilities, sections, bindings and
  dependants, and that no file changed.
- One `reconcile` fixture per obligation kind in a temporary Git repository, each
  asserting the kind, location and evidence text, and that fixing it removes the line.
  The `example-failing` fixture binds an example that does not compile, asserts the
  line, and asserts that the line is gone once the example builds and runs.
- A merge-base fixture where an affected section is edited on a branch and is not
  reported, and one where it is unchanged and is reported; one where only the content
  of a generated block changed is reported too, and a workspace below the top of its
  repository is compared at its prefix.
- An invocation fixture with stub `git` and `cargo` on `PATH` records each argv and
  asserts the forms above, and a default branch named `--output=x` is resolved to a
  commit id and never reaches `git` as an option.
- A Git version fixture: a stub `git --version` reporting 2.29 makes `reconcile` exit
  2 before any other process starts, and a repository with no
  `refs/remotes/origin/HEAD` is an error.
- A default-branch fixture with a local branch literally named `origin/main` beside
  `refs/remotes/origin/main` resolves the remote-tracking commit.
- A blob over 1 MiB at the merge base is an error and not a truncated read.
- An `example-failing` fixture binding a function that is not a test shows that zero
  tests run is reported as a failing binding.
- A `contract_effect: none` fixture asserting the lighter list.
- A determinism check: two runs print identical output, and the exit code is 0 only
  for an empty list.
- Scenario A and Scenario B are described in
  [the completion gate](./workplan-complete-gate.md) and use `reconcile` at each step.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::new::the_printed_steps_end_in_a_clean_check_and_skipping_the_stage_step_does_not` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_git_older_than_2_30_is_exit_two_before_any_other_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::a_list_with_obligations_is_printed_one_line_each_and_exits_one` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::an_empty_list_prints_nothing_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::fixing_an_obligation_removes_its_line_and_only_its_line` | test | [Verification](#verification) |
| `raf_workplan::Workplan::impact` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::new_change` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::reconcile` | doctest | [Verification](#verification) |
| `raf_workplan::lifecycle::section_digest` | doctest | [Verification](#verification) |
| `raf_workplan::test_crate::complete::each_kind_of_obligation_is_a_reconcile_line_in_a_git_repository_and_fixing_it_removes_the_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::stubs::every_question_reaches_git_as_exactly_its_argv_in_the_work_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_bad_slug_is_refused_before_anything_is_written_and_a_good_one_next_to_it_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_capability_that_does_not_exist_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_capability_whose_document_did_not_load_is_named_and_never_skipped` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::an_affects_entry_must_name_a_listed_capability_and_one_of_the_seven_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::an_existing_file_is_refused_and_left_byte_identical` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::both_spdx_fields_open_the_record_with_the_two_comment_lines_of_the_records_of_the_repository` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::explicit_affects_are_written_as_given_and_impact_reports_them_sorted` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_gives_the_capabilities_sections_bindings_dependants_and_linking_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_prints_each_part_in_a_fixed_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::new_writes_one_file_with_the_keys_the_id_and_a_summary_and_it_loads_clean` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::one_spdx_field_without_the_other_is_an_error_that_names_the_missing_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::several_capabilities_are_listed_in_the_order_given_without_repeats` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_default_affects_is_the_contract_of_each_listed_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_guidance_follows_the_state_of_each_listed_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_printed_text_holds_the_template_and_the_guidance` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_template_in_a_capability_of_each_state_gives_one_block_and_a_clean_check` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::contract_effect_none_keeps_every_obligation_the_documents_can_show` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::each_fault_is_owed_with_its_kind_location_and_evidence_and_fixing_it_removes_the_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::equal_inputs_print_equal_output` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::every_static_kind_is_raised_by_a_fixture_and_the_two_that_need_a_process_by_none` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::reconcile_writes_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::text_of_a_proposed_block_is_not_read_for_wording` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_binding_obligations_are_the_gaps_of_the_roadmap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::wording_that_only_looks_like_a_phrase_owes_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::planning_produces_the_documentation` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_default_branch_named_like_an_option_is_resolved_and_never_reaches_git_as_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_absent_at_the_merge_base_counts_as_touched` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_at_the_merge_base_that_does_not_parse_is_an_error_and_not_a_pass` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_over_one_mebibyte_at_the_merge_base_is_an_error_and_exactly_one_is_read` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_linked_work_tree_compares_its_own_head` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_section_edited_on_the_branch_is_not_reported_and_its_neighbour_is` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_workspace_below_the_top_of_the_repository_is_compared_at_its_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_edit_that_is_not_committed_counts_because_the_working_tree_is_compared` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_unchanged_affected_section_is_owed_with_the_line_the_contract_shows` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::contract_effect_none_leaves_out_contract_usage_and_examples_and_asks_git_for_nothing_then` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::git_2_29_stops_the_open_before_any_other_process_and_2_30_goes_on` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::no_default_branch_is_an_error_that_names_the_fix_and_the_fix_clears_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::on_the_default_branch_itself_every_affected_section_is_owed_and_it_is_not_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::only_a_generated_block_changed_is_still_reported` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::reconcile_writes_nothing_and_prints_the_same_list_twice` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::a_prefix_the_source_path_pattern_cannot_carry_is_an_error_and_not_a_guess` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::a_root_below_the_top_reads_each_file_at_the_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::a_stub_git_2_29_runs_one_process_and_exits_with_the_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::stubs::reconcile_asks_git_exactly_the_commands_of_the_contract_in_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::the_digest_leaves_out_generated_blocks_on_both_sides_and_nothing_else` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::a_binding_to_a_function_with_no_test_reports_zero_tests_as_failing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::an_example_that_does_not_build_is_a_line_and_is_gone_once_it_builds_and_runs` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::test_bindings_are_not_run_by_reconcile_and_unresolved_examples_are_not_started` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depended on by | [workplan-cli](./workplan-cli.md) |
| Depended on by | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depended on by | [workplan-evidence-claims](./workplan-evidence-claims.md) |
| Changed by | [workplan-reconcile](../../work/changes/workplan-reconcile.md) |
<!-- raf:generated:end -->
