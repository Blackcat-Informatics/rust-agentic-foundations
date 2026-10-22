<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Two-way coverage assertion ownership

Read-only current source/assertion map, without builds, product edits or a panel.
This is preparation, not a claim that the capability or adoption is qualified.
Graphify has no available snapshot. Exact names below are inspected owners unless
explicitly labelled a proposal or missing proof.

Prefixes: C = `raf_workplan::test_crate::coverage::`,
S = `raf_workplan::test_crate::strict::`,
B = `raf_workplan::test_crate::bindings::`,
L = `raf_workplan::test_crate::lines::`,
CLI = `cargo_workplan::test_crate::coverage::`.

## Actual surface operation boundaries

`Workspace::exported_items_with` (raf-source/src/workspace.rs:661) creates a new
Meter, admits the existing item bytes, gathers hidden files and invokes
`Surface::new(...).list()` each call. There is no exported-surface result cache.
This fact alone does not establish a violation of the current performance law:

- `Workplan::finish` (model.rs:281) calls `coverage::check` during loading and
  stores its findings. That computation lists the union of explicit scope and
  packages named by exclusions once, using a set and one target loop.
- `Workplan::check` (model.rs:398) reads base findings and runs strict/debt.
  `strict::reaching_bindings` explicitly excludes packages already in scope.
  Thus ordinary scoped packages are not recomputed just because check runs.
- `strict::explained` lists binding-reached packages outside scope once per
  check, again with a package set and one target loop.
- `coverage_report` intentionally constructs a fresh computation over all
  packages once per report invocation.

An outside-scope package both named by an exclusion and reached by a binding
is listed during load and again during each check. The separate once-per-load
and once-per-check promises can permit that overlap; they must not be silently
reinterpreted as a single lifetime cache promise, nor as permission to remove
either bound. Multiple report calls likewise perform separate listings.

**Actual proof gap:** S `a_scoped_package_is_enforced_once_with_a_binding_present`
asserts one finding and its explicit-scope reason, not an exported-items call
count. The coverage tally's exclusion/entry/lookup tests do not count surface
listings either. Add an actual operation receipt at the production surface-call
boundary (test-scoped counter, no fake answer): load with scoped, excluded-only,
binding-only, exclusion-plus-binding and untouched packages; then two checks and
two reports. Assert exact target identities/counts at each phase, including a
multi-target package and refused surface. Preserve current semantics and the
existing performance promise. No new cache is required by this source finding.

## Direction table: meaningful clause owners

Split the six rows into named clauses, retaining each question/check:

1. **Resolved implementation bindings.** B
   `every_unresolved_binding_of_every_list_is_reported` requires four findings
   naming the exact code/test/example bindings; `bindings_that_name_real_symbols_resolve`
   is its positive owner. B `a_code_binding_must_name_a_shipping_item` asserts
   exact wrong-role messages for unit/test code. These prove structural binding
   resolution, not that prose is semantically true of the implementation.
2. **Public implementation ownership.** C
   `a_bound_item_is_covered_and_its_unbound_neighbour_is_a_gap` excludes the bound
   path, retains its neighbor and checks covered count. C
   `the_report_of_check_and_its_findings_follow_the_scope` checks exact finding
   kinds/files/lines, reports outside-scope gaps and has a fully accounted item
   neighbor. Attach automatic-package enforcement separately, below.
3. **Resolved test bindings.** The same unresolved-list owner plus B
   `a_tests_binding_must_name_a_function_marked_test` (exact wrong-role refusal
   for plain functions) own the structural test-list check. They do not prove a
   test's assertions establish arbitrary prose.
4. **Test ownership.** C
   `a_bound_test_is_covered_and_every_unbound_test_unit_tests_included_is_a_gap`
   asserts exact integration/unit paths, unit location, covered count and the
   other package's unit test. Use scope/exclusion owners separately.
5. **Change obligations.** Retain the actual reconcile obligation owners mapped
   in queue-reconcile-claim-owner-map.md, particularly exact raised/repaired
   cases. Coverage tests cannot own the obligation list merely by naming it.
6. **Evidence dependency view.**
   `raf_workplan::test_crate::evidence::the_views_list_the_claims_of_a_record_and_the_records_of_a_capability`
   asserts exact capability/path/section/line/statement rows for actual claim
   markers and the reverse dependency view. It proves recorded dependencies,
   not universal epistemic dependence of all prose.

Contract markers may follow meaningful named clause headings. Constraints
markers must immediately follow the actual paragraph/list item whose statement
they own; a Constraints heading marker does not cover all its list entries.

## Marked Contract clauses: current proof and narrow gaps

- **Exported public surface:** C
  `the_code_report_lists_every_public_item_and_no_private_or_hidden_one` checks
  the exact library surface, excludes private/crate-only/hidden and unreexported
  private-module members, and checks definition/location of a public reexport.
  The current binary-empty paragraph is false as a universal behavioral claim:
  Judge::surface lists Lib and Bin identically; the actual completion binary
  fixture proves explicit `pub` binary functions are enforced. Correct both the
  capability and coverage.rs module prose; add the proposed direct binary
  public/private/doc-hidden report/check neighbor. Do not narrow enforcement.
  This inspected fixture does not independently exercise trait members,
  multiple aliases' first-path selection, or cross-workspace reexport ownership.
  Those need exact assertions, not this blanket marker alone.
- **Unresolved reexports:** C
  `a_re_export_the_index_cannot_follow_is_reported_and_enforced_and_its_module_covers_it`
  checks exact kind/path/named path/file/line, excludes external Serialize,
  requires the actual finding, then module-binding and exclusion repairs.
  `a_re_export_written_through_super_names_the_path_super_reaches` owns the
  super-path case. The entire crate/self/bare/super grammar should be split
  from report/enforcement; not every grammar alternative is proved by these
  two examples. Preserve source-index grammar limits explicitly.
- **Definition coverage:** C `binding_an_item_covers_what_is_below_it_and_nothing_beside_it`
  checks module/type descendants, sibling exclusion and root binding covering
  nothing. `an_item_bound_through_a_re_export_or_its_definition_is_covered_under_its_exported_path`
  exercises both binding forms; `an_item_bound_only_in_examples_is_a_gap` proves
  code versus example role with a code-binding repair. The sentence asserting
  evidence/assessment module digests is a separate guarantee: attach actual
  digest mutation owners or split it; these coverage assertions do not own it.
- **Report contents/order/rendering:** C
  `the_table_and_rdf_preserve_the_same_entries_and_escape_their_carriers`
  asserts exact text entries, reasons/escaping, surface-error table/native graph
  rows and both native RDF round trips. C
  `the_report_and_its_two_forms_are_the_same_on_every_run` compares loaded
  report objects and both MIME renderings. CLI
  `gaps_in_either_direction_are_printed_and_exit_one` checks both exact tables,
  RDF fields, quiet finding retention and unchanged repository bytes; CLI
  `an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero` owns
  excluded reason, zero exit/quiet outcome and other-direction independence.
- **Scope/exclusions:** C scope-name, unmatched/empty-reason, exported/definition
  path, sibling-prefix and first-match tests assert the matching behavior their
  full names state, with actual refused/valid neighbors. Do not conflate first
  winning reason with only that exclusion being marked: the unit oracle tests
  require all matching rows marked, preventing false unmatched findings.
- **Invalid documents:** C
  `a_capability_that_does_not_load_gives_its_own_findings_and_no_cascade_of_gaps`
  checks clean, malformed capability, unmatched exclusion still reported,
  malformed change with gaps retained, and repaired capability with one missing
  binding becoming exactly one gap. Existing ambiguity and budget tests own
  surface failure independently; they do not prove every unreadable-surface
  subtype during capability suppression.
- **Automatic package enforcement:** S
  `a_binding_reached_package_is_enforced_and_an_unreached_one_is_not` proves
  code-binding reach, exact reason and repaired ownership; scoped duplication
  and exclusion repair have their own inspected S owners. This fixture does
  not prove test-only and example-only reach or automatic unreadable-surface
  location. Keep those as missing dedicated proof unless another actual owner
  asserts them. The source iterates graph bindings without role restriction.
- **Line independence:** C empty-scope test uses
  `item_findings_after_line_refusals`, which first checks exact retained default
  zero orphan counts, no exceptions/debt/ceilings and exact line refusals before
  separating item assertions. It does not silently waive line debt. L
  `an_excluded_item_that_nothing_covers_is_orphan_until_an_exception_names_it`
  directly checks extra's exact orphan range, then no orphan and exact exception
  after adding its independent line exception.

## Constraints: exact owners and remaining work

1. **Report-only:** CLI gaps test snapshots the full repository unchanged after
   both directions. Keep this separate from possible trusted build effects of
   other commands; coverage does not execute bound examples.
2. **Visibility comes from source:** C exact surface test plus production
   Judge::surface -> Workspace::exported_items is meaningful ownership; naming
   conventions do not choose visibility. Binary sentence correction above is
   required.
3. **Exclusion reasons:** C empty/unmatched tests plus
   `raf_okf::test_crate::records::an_exclusion_has_a_reason_with_no_placeholder`
   assert whitespace-empty, missing and placeholder reasons refused.
   `an_exclusion_names_an_item_or_a_module_prefix_below_a_crate_root` refuses
   whole-crate roots and malformed forms with shortest valid module neighbor.
   CLI exclusion table proves actual reasons are displayed.
4. **State boundary:**
   `raf_okf::test_crate::profile::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests`
   asserts both forbidden early-state bindings, required delivered lists and
   valid implementing forms. Keep role resolution and complete-policy content
   state rules separate; the profile test alone is not a coverage-execution
   proof for every state.
5. **Refused surface never empty:** C ambiguous-surface test asserts typed
   Unresolved, exact target/root, non-complete report even with empty item list,
   scoped finding and outside-scope report; budget test asserts typed Budget
   and retained test findings. CLI surface-error test owns code-versus-tests
   command boundary. Concrete missing subtype coverage here: import-cycle,
   glob-of-nothing, CarryBudget, and local crate-only versus external carried
   reexport collision. Existing generic mapping to Unresolved/CarryBudget is
   source-inspected; do not infer these particular producer cases from ambiguity.
6. **Listing frequency and deterministic order:** actual phase distinction
   above; add counted receipt. Existing repeated-render equality owns stable
   bytes, not listing frequency. Source sorts package/path outputs explicitly.
7. **Binding-only once per check:** same receipt gap, preserve exact original
   performance bound. S scoped finding deduplication is not a call-count test.
8. **Exclusion versus line exception:** bind the exact L orphan/exception test
   above, retaining the real distinction in judged Constraints.

No Constraints limitation is proposed for relocation to Explanation. Do not
replace any missing operation/subtype assertion with the entire coverage suite,
or treat all implementation names above as executed test results. No current
full gate, candidate count or adoption completion is claimed.
