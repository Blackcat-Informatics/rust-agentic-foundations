<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# History and assessment claim preparation

Read-only mapping in the issue 16 checkout. No product edits, builds, suites,
process probes or measurements ran. Source assertion inspection is evidence of
what a test checks, not evidence that it passed now. All markers below name
actual current bound paths unless explicitly identified as cross-capability
candidates. `T::` abbreviates `raf_workplan::test_crate::`; `W::` abbreviates
`raf_workplan::`. Expand prefixes in inserted markers.

Use meaningful headings and `<!-- raf:tested-by path[,path...] -->` immediately
after each heading. Preserve all Contract/Constraints semantics. Quoted anchors
are stable insertion points; current history Contract begins at 69 and
Constraints at 216, assessment Contract at 63 and Constraints at 164.

## Entity history: inspected assertion mapping

| Proposed named clause / insertion anchor | Specific marker paths | Actual source assertions and limits |
|---|---|---|
| History entries and stable order; keep The command opening, split date filter | `T::history::a_capability_history_holds_its_document_through_a_move_and_its_bound_code`, `T::history::every_question_is_asked_in_order_as_exactly_its_argv` | `history.rs:245` uses actual Git fixture: exact document commits through move, old/new paths, symbol-only commit subset, one entry/source union and newest author instants. Line 1003 uses scripted responses to check equal instants with differently written offsets sort by commit id, exact source/path union and no duplicate entry. |
| Author-date filtering; before `--since <date> keeps` | `T::history::since_reaches_no_process_and_keeps_each_entry_by_its_author_date`, `W::git::log::tests::a_since_day_starts_at_midnight_utc_on_the_scale_of_an_author_date`, `W::git::log::tests::a_strict_author_date_is_read_and_ordered_by_its_instant` | `history.rs:1142` passes whole logs with east/west offsets, requires no --since in any process, keeps only exact UTC-day-boundary entry and checks previous-day neighbor includes all three. `git/log.rs:523,559` checks exact instants, leap/invalid date/time/offset spelling and one-second-before boundary with epoch/2000/2099 examples. |
| Document history moves and merges; retain Document history | `T::history::a_capability_history_holds_its_document_through_a_move_and_its_bound_code`, `T::history::a_merge_whose_resolution_changed_the_document_is_in_its_history_and_a_clean_one_is_not` | Real Git at line 245 checks --follow move paths. Line 557 creates an actual conflicting merge, resolves to third content, then a clean non-fast-forward merge; expects resolved/main/side/root history, excludes clean merge and checks resolved path. Exact argv test at 1003 verifies --cc/--root/--no-show-signature flags. No claim that this reruns arbitrary user Git configurations. |
| Symbol span history and refusal; retain Binding history opening, split error paragraph | `T::history::a_capability_history_holds_its_document_through_a_move_and_its_bound_code`, `T::history::a_symbol_log_that_git_refuses_names_the_binding_and_the_file_to_commit` | Actual Git line 245 excludes sibling-function/deletion commits from second's line history. Line 1073 scripts exit 128 for exact -L, checks typed SymbolLog with binding/file/source error and exact actionable HEAD/commit guidance; document-log failure remains ordinary Git failure; unchanged scripted neighbor succeeds. |
| Whole-module history; before `A binding that resolves to a module below its crate root` | `T::history::a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling`, `T::history::a_module_binding_asks_its_span_and_then_the_log_of_each_of_its_files_in_order` | Actual Git at line 716 checks exact own/child commit list, excludes sibling/root/inline changes, records actual paths, root has only root-file history and inline module only span history, deduplicates entries. Line 1216 pins document→span→own-file→child-file command sequence, no sibling question, exact path union. |
| Uncommitted module file hard failure; before `A file of the module that no commit holds` | `T::history::a_module_file_that_is_not_committed_names_the_binding_and_the_file_until_it_is`, `T::history::a_module_binding_asks_its_span_and_then_the_log_of_each_of_its_files_in_order` | Real Git line 809 leaves child uncommitted, requires typed Uncommitted/binding/file and message saying commit file; committing the neighbor then produces history. Scripted line 1216 independently turns empty child log into same typed failure. |
| Removed binding lookup; before `A binding that no longer resolves` | `T::history::a_capability_history_holds_its_document_through_a_move_and_its_bound_code`, `W::git::log::tests::the_pickaxe_answers_one_record_with_its_parent_or_none`, `T::history::every_question_is_asked_in_order_as_exactly_its_argv` | Actual Git line 245 verifies removed symbol/identifier, whole-root directory, observed removal commit/subject and previous parent. `git/log.rs:675` checks exact record parsing, first parent of merge, root with no parent, empty answer, malformed multiple records/parents/date. Line 1003 checks unavailable crate returns NoCrate and asks no pickaxe for it. This proves the documented lookup algorithm, not true semantic removal in every repository. |
| Validated queries and exact invocation; retain Invocation opening, split below command block | `T::history::every_question_is_asked_in_order_as_exactly_its_argv`, `T::history::an_id_or_a_date_that_cannot_be_asked_about_starts_no_process`, `T::history::stubs::a_path_named_like_an_option_reaches_git_after_the_separator_or_the_colon` | Line 1003 pins every actual argument form/order and 16 MiB/C-locale settings. Line 1306 rejects malformed/unknown/not-loaded ids and invalid dates with zero Runner calls; valid neighbor starts queries. Real stub process at 1545 records exact argv for option-looking files in single -L argument or after --; verifies no unintended x.rs. Shell absence and working directory are implementation/Runner properties beyond serialized command-string equality. |
| Strict log grammar and subject as data; before `Each record of the first two forms opens` | `W::git::log::tests::a_subject_keeps_every_byte_but_a_newline`, `W::git::log::tests::output_that_is_not_a_log_of_the_form_is_an_error`, `W::git::log::tests::a_symbol_log_lists_no_path_and_refuses_any_other_line` | `git/log.rs:598` embeds unit/record separators, commit-like text, pipe and escape sequence in a subject and requires one intact subject; invalid UTF8 is explicitly lossy U+FFFD. Line 613 rejects preamble, bad id/date, malformed quoted path and invalid UTF8 path. Line 635 accepts no-path symbol record and rejects unexpected patch lines. |
| Whole-log output bound; before `The command reads at most 16 MiB` | `T::history::a_log_past_sixteen_mebibytes_is_an_error_and_the_bound_itself_is_read`, `T::history::stubs::a_real_process_that_prints_past_the_bound_is_an_error` | Fully inspected line 1456 scripts bound+1, requires typed LogTooLarge/16777216 message, removes one byte and accepts exact bound. Real-process test at 1627 is a candidate assertion reread for production pipe draining, not inspected in full here. |
| Text rendering and terminal safety; retain Output, separate RDF paragraph | `T::history::the_table_preserves_typed_entries_sources_and_removed_bindings_in_order`, `W::history::render::tests::a_table_cell_cannot_break_the_table_or_reach_the_terminal` | `history.rs:441` checks exact table header, rows in typed entry order, seven-character ids, date/day/subject/sources and removed-binding table/parent. `history/render.rs:107` checks pipe escaping, ESC/unit separator, newline flattening, code-backtick handling, bidi escapes and non-ASCII preserved neighbor. |

## History clauses still needing specific assertion reread or ownership

- **Supported ids**: `T::history::each_kind_of_id_has_the_history_of_its_file`
  at line 360 is the exact bound positive matrix for cap/change/arch/evidence/
  assessment; malformed-id negatives were inspected above.
- **Git version/shallow gate**: bound
  `T::history::an_old_git_or_a_shallow_answer_stops_before_any_history_is_read`,
  `T::history::a_shallow_clone_is_refused_with_the_missing_history_named_and_a_full_clone_is_not`,
  `T::history::stubs::a_git_reporting_2_29_stops_the_history_before_any_log`, and
  `W::git::log::tests::the_shallow_answer_is_true_or_false` are exact candidates.
  The library tests cannot independently establish CLI exit 2; use the actual
  command tests owned by workplan-cli for that claim.
- **UTC versus committer date and no removal narrowing**: bound
  `T::history::since_selects_by_the_author_date_and_not_by_the_committer_date`
  at 843, `T::history::since_counts_from_the_start_of_the_day_in_utc_in_every_time_zone`
  at 639 and `T::history::since_limits_the_logs_and_not_the_removal_lookup`
  at 398 are specific final rereads, not substitutes for one another.
- **Ambiguous/indeterminate binding refuses before processes**: bound
  `T::history::a_binding_that_names_no_single_symbol_is_refused_before_any_process`
  at 1373 must be reread for exact cases; do not infer it from invalid-id tests.
- **RDF carrier**: the current history capability binds no cargo-workplan test.
  The actual `cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings`
  is bound by workplan-cli. Its body at `crates/cargo-workplan/tests/history.rs:58`
  was inspected: compares RDF entry count with table rows, full 40-char commits
  and abbreviated table correspondence, source kind/binding/path, actual removed
  commit/identifier/directory and optional lastResolved. Consider adding that
  actual behavior test as an explicit binding here if needed, or keep renderer
  ownership in the CLI capability. It is not an already bound marker candidate
  for this document. This inspected body does not itself check every ordinal,
  N-Triples output or exact author-date lexical values: inspect shared Rdf helper
  and dedicated renderer tests before claiming those independently.

## History Constraints disposition

1. **Read-only**: the actual Git test at 245 snapshots everything including .git
   byte-for-byte before/after. It proves the fixture's no-write behavior. Keep
   the explicit partial-clone-fetch exception: no current fixture establishes
   every Git partial-clone behavior and Git configuration remains trusted input.
2. **Every entry has commit**: typed-entry/parser/table assertions above cover
   commit identity presence/validation. `checkable with plain Git` is operational
   guidance; a fixture cannot prove retained objects in every external clone.
3. **Two path bases in nested workspace/diff.relative**: no inspected bound
   fixture establishes both nested Git worktree root and diff.relative=true.
   This is a genuine concrete test-search gap, not a reason to assign a normal
   root-relative-path test. Inspect command implementation/user-config fixture
   coverage before adding a targeted neighbor.
4. **Shallow hard failure**: map the dedicated shallow/version tests, not a
   successful history snapshot; preserve no-partial-list behavior.
5. **Validated process data**: exact argv, invalid-id/date no-call and option-file
   real-stub tests above provide concrete boundaries. Identifier generic/raw
   stripping and commit-parent validation need their actual parser assertions;
   pickaxe parent validation was inspected.
6. **Since still reads whole bounded log**: scripted no-since argv and bound
   tests establish those two parts separately. No combined over-bound log with
   a narrow since was inspected; this may need a targeted assertion if claiming
   production caller composition, rather than reusing only each primitive.
7. **Git -L limits**: source/typed error test proves hard failure propagation;
   real move/rename-between-files and dirty working-span offset semantics are
   delegated Git behavior. No inspected test covers every detailed limitation.
8. **Whole-module files/uncommitted**: inspected actual own/child/sibling/root/
   inline and uncommitted-valid-neighbor fixtures directly cover these claims.
9. **Removed-binding heuristic**: keep its limits, not a semantic truth claim.
   The inspected test proves the query and one literal removal. No fixture here
   proves the stated same-substring false-positive/rename/count-preserving move/
   merge-removal examples. Add exact regression examples or retain explanatory
   algorithm limits without pretending the happy path proves them.
10. **Terminal escapes**: direct bound render unit test covers control and bidi
    threats with ordinary Unicode neighbor; it does not claim subject bytes are
    physically preserved in terminal rendering (they intentionally are escaped).
11. **No document embedding/edit**: actual whole-repository snapshot and table
    renderer tests support current command behavior. Cross-command tracked-path
    discovery and no automatic invocation belong to command-dispatch evidence.

There are no timing or footprint measurements in this Contract to move as a
claim-coverage shortcut. Partial-clone behavior, Git configuration/rename
semantics and removal heuristic rationale may be elaborated in Explanation,
while retaining their operative limits in Constraints.

## Prose assessment: inspected assertion mapping

| Proposed named clause / insertion anchor | Specific marker paths | Actual source assertions and limits |
|---|---|---|
| Authored versus mechanical text; retain Mechanical and authored text | `T::assessments::a_capability_that_does_not_load_is_refused_and_a_section_of_generated_blocks_only_does_not_load`, `T::assessments::a_regenerated_block_leaves_the_assessment_of_its_section_current` | `assessments.rs:800` removes Usage authored text leaving a comment, requires CapabilityNotLoaded and okf-section-empty, then restores valid neighbor. Line 596 adds test binding and executes actual sync, verifies bytes changed and existing Verification assessment remains Current. The generated-only exact unit rule has a separate bound test below. |
| Subject and allowed record; split before `An assessment covers a whole section` | `T::assessments::each_refusal_writes_nothing_and_its_neighbour_records`, `T::assessments::a_record_named_otherwise_than_its_subject_is_a_finding_and_not_the_assessment_of_it` | Line 717 refuses malformed subject, unknown section/capability, checks unchanged snapshot then successful neighbor. Line 694 renames record/id away from subject, requires exactly assessment-id-mismatch and missing canonical section assessment. |
| Deterministic single-file replacement; retain The record opening, split input selection | `T::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first` | Line 183 snapshots whole fixture, checks exactly one assessment path changed, exact canonical id/subject/actor/input names, no findings, current/no obligations; second identical request writes identical bytes and reports replaced. Opposite actor/verdict changes only those two fields, still one file. |
| Strong/weak binding input selection; before `A code span outside fenced code` | `T::assessments::a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not`, `T::assessments::a_bound_symbol_named_by_its_last_segment_is_an_input_beside_one_named_in_full`, `T::assessments::a_common_word_that_is_the_last_segment_of_a_binding_does_not_narrow_the_inputs`, `T::assessments::a_method_named_by_its_type_and_name_is_an_input_and_its_sibling_is_not`, `T::assessments::a_suffix_two_bindings_end_in_names_both` | Lines 262/371 check exact selected symbols, absent unnamed/spare inputs, unrelated edit stays Current, selected edit yields exact Symbol stale input. Line 402 weak common-word command name leaves all bindings; 424 type+method suffix narrows and sibling-method edit stays current; 448 two-segment suffix selects both and weak single suffix leaves spare too. |
| Evidence claim inputs; split before `and each evidence record a claim marker` | `T::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input` | Line 553 records actual passing evidence, checks Record input, edits value and requires exact stale input/reconcile obligation; removes record and requires Unresolved reason. |
| Recording refusals; before `A bound symbol that is an input and does not resolve` | `T::assessments::each_refusal_writes_nothing_and_its_neighbour_records`, `T::assessments::a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged` | Line 717 requires typed missing-symbol and missing-evidence errors, no-write snapshot for unresolved symbol and valid neighbors; missing evidence context still exposes missing input. Line 826 actual 10,000-symbol context succeeds but record exceeds bound, requires AssessError::Record and unchanged entire tree; 8,000-symbol neighbor records all 8,001 inputs, Current, then exact single-symbol staleness. |
| Assessor context parity; retain The context | `T::assessments::the_context_is_the_same_for_a_human_and_an_agent_and_holds_what_the_assessor_reads` | Line 613 checks section text, selected source definitions/docs, evidence verdict/value/limit and all three input digest headers; excludes unselected symbol; recording by either actor leaves context identical, inputs equal and record differs only by actor kind. |
| Currency and reconcile; retain Invalidation, split unchanged unrelated/generated effects | `T::assessments::an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not`, `T::assessments::a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not`, `T::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input`, `T::assessments::a_rejected_assessment_is_current_and_owes_an_acceptance_whoever_gave_it` | Lines 232/262/553 check precise Section/Symbol/Record moved-input descriptors and exact assessment-stale obligations. Line 669 records rejected for every Actor, checks Current plus exact assessment-rejected obligation, then accepts and clears it. Missing assessment is asserted by mismatched canonical record test. New/removed input-set categories need bound helper tests below. |
| Module and crate-root digests; split long Constraints binding paragraph | `T::assessments::an_edit_inside_a_bound_module_moves_the_assessment_and_a_sibling_edit_does_not`, `T::digest::a_crate_root_stands_for_its_own_file_and_an_edit_below_it_leaves_its_digest` | Line 299 checks sibling edit Current, child and own module file edits stale module input after separate assessments. `digest.rs:328` checks below-root own/child module edits leave root digest, move module digest; root-file edit moves root digest. |
| Safe assessment directories; give Constraints its own named clause | `T::assessments::an_assessment_is_never_written_through_a_linked_directory_and_a_real_one_records` | Unix-only line 896 replaces each of work/work/assessments by in-root symlink after plan loads, asserts exact Write path and unchanged snapshot/preserved sentinel; restores real directory and replaces canonical file. Also refuses file at work/assessments with unchanged snapshot. Does not itself exercise outside-root links or every native Windows case. |

## Assessment remaining bound candidates and real gaps

- Bound `W::assessment::tests::a_section_of_generated_blocks_and_comments_only_is_not_authored_and_one_word_is`
  and `W::assessment::tests::authored_text_is_what_the_profile_calls_a_section_that_is_not_empty`
  are exact candidates for authored/profile parity; their bodies were not reread.
- Bound `W::assessment::tests::a_subject_is_a_capability_and_one_of_the_seven_sections`
  and `W::assessment::tests::the_record_slug_is_the_capability_slug_and_the_section_and_no_two_sections_share_one`
  cover complete subject and collision-free canonical-id mapping; keep distinct
  from one fixture's successful record path.
- Bound naming-unit tests for strong/weak whole path, monotonically widening weak
  names and two-pass lookup counts complement the inspected concrete assessment
  integrations. They are not a runtime performance measurement. The no-strong
  fallback has the exact bound integration
  `T::assessments::with_no_bound_symbol_named_every_bound_symbol_is_an_input`.
- Bound `W::assessment::tests::a_moved_input_names_its_kind_its_name_and_how_it_moved`
  is the precise next read for added/removed/unresolved input descriptors.
  Inspected integrations prove changed/unresolved examples but not every way the
  section's names/bindings/claims can stop or start being inputs.
- **Actual completion gate**: current bound assessment tests check reconcile
  obligations, not actual complete refusal/acceptance. The exact existing
  `raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does`
  is bound by workplan-complete-gate, not this capability. Use explicit ownership
  or add that real binding if this document must directly marker-map The gate;
  do not represent `fixture.owed()` as execution of `complete`.
- **Type versus method digest limit**: method-selection integration checks
  sibling-method edits do not stale a method assessment. That is not the same
  as a type binding excluding its methods. Inspect the existing Digest symbol
  definition tests for an explicit type/method neighbor before assigning it.
- **Cross-path reexport/fenced/prose limitations**: the exact selected-code-span
  behavior and whole-body naming helper source support the algorithm. No
  inspected integration specifically proves the reexport-alternate-path
  exclusion, prose-only name, fenced name, or PROPOSED/generated exclusion when
  another binding is strong. These are targeted assertion-search gaps.
- **Linked-directory universal wording**: inspected test is Unix-only, in-root
  link plus file and real-directory neighbors. Outside link, missing directory
  creation, linked races and platform-specific guarantees must be supported by
  shared record-write tests/source rather than silently inferred.
- **Large record refusal** is directly tested without partial write; the name
  of the 10,000-symbol test can mislead because that record is refused, while
  8,000 is the successful neighbor. Do not describe 10,000 as recorded.
- **Evidence input failure snapshot**: the missing-evidence error is checked,
  but that subcase does not take a fresh before/after snapshot in the inspected
  test. The code can fail before write by construction; distinguish source
  evidence from the test's explicit no-write assertions.

All eight assessment Constraints have corresponding categories above: one-file
write, safe directories, generated-content exclusion, deterministic bytes,
actor parity, weak-name selection limits, symbol/module/root digest scope and
over-bound whole-record refusal. Preserve these runtime requirements. None is
a historical timing/footprint observation that should be moved merely to avoid
claim coverage. Explanatory reasons for input-selection approximation and
whole-item/module digest scope can live in Explanation, with the actual limits
remaining visible in Constraints.
