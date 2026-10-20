---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-two-way-coverage
title: "Two-way coverage"
description: "Checks from code to documentation and from tests to claims, complementing the documentation-to-code and documentation-to-tests checks."
state: implementing
bindings:
  code:
    - raf_workplan::coverage
    - raf_workplan::Workplan::coverage_report
  tests:
    - raf_workplan::test_crate::coverage::a_bound_item_is_covered_and_its_unbound_neighbour_is_a_gap
    - raf_workplan::test_crate::coverage::a_bound_test_is_covered_and_every_unbound_test_unit_tests_included_is_a_gap
    - raf_workplan::test_crate::coverage::a_crate_with_thousands_of_items_and_tests_is_judged
    - raf_workplan::test_crate::coverage::a_scope_entry_is_a_package_name_and_a_crate_name_says_which_package_it_means
    - raf_workplan::test_crate::coverage::a_surface_over_the_item_budget_is_a_budget_error
    - raf_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_an_error_of_the_report_and_a_finding_in_scope
    - raf_workplan::test_crate::coverage::a_unit_test_module_is_excluded_by_its_prefix_with_the_reason_shown
    - raf_workplan::test_crate::coverage::an_empty_scope_enforces_nothing_and_the_report_still_names_every_gap
    - raf_workplan::test_crate::coverage::an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file
    - raf_workplan::test_crate::coverage::an_item_bound_through_a_re_export_or_its_definition_is_covered_under_its_exported_path
    - raf_workplan::test_crate::coverage::an_item_is_excluded_by_its_exported_or_its_defining_path
    - raf_workplan::test_crate::coverage::binding_an_item_covers_what_is_below_it_and_nothing_beside_it
    - raf_workplan::test_crate::coverage::the_code_report_lists_every_public_item_and_no_private_or_hidden_one
    - raf_workplan::test_crate::coverage::the_report_and_its_two_forms_are_the_same_on_every_run
    - raf_workplan::test_crate::coverage::the_report_of_check_and_its_findings_follow_the_scope
    - raf_workplan::test_crate::coverage::the_table_and_the_json_carry_the_same_entries_escaped
    - raf_workplan::test_crate::coverage::this_repository_enforces_the_workplan_crates_and_they_are_covered
    - raf_workplan::test_crate::coverage::without_a_record_check_has_exactly_one_finding_that_says_to_add_it
    - raf_workplan::test_crate::coverage::a_prefix_matches_below_its_module_and_never_a_sibling_that_shares_its_text
    - raf_workplan::test_crate::coverage::a_re_export_the_index_cannot_follow_is_reported_and_enforced_and_its_module_covers_it
    - raf_workplan::test_crate::coverage::an_item_bound_only_in_examples_is_a_gap
    - raf_workplan::test_crate::coverage::the_first_matching_exclusion_gives_the_reason
    - raf_workplan::test_crate::coverage::a_capability_that_does_not_load_gives_its_own_findings_and_no_cascade_of_gaps
    - raf_workplan::test_crate::coverage::a_re_export_written_through_super_names_the_path_super_reaches
    - raf_workplan::coverage::tests::the_exclusions_are_indexed_once_and_each_entry_looks_up_its_paths_and_their_modules
    - raf_workplan::coverage::tests::without_exclusions_the_entries_are_judged_and_nothing_is_looked_up
    - raf_workplan::coverage::tests::the_work_grows_with_the_entries_and_no_faster
    - raf_workplan::coverage::tests::the_work_grows_with_the_exclusions_and_no_faster
    - raf_workplan::coverage::tests::the_work_grows_with_the_entries_and_the_exclusions_together_and_no_faster
    - raf_workplan::coverage::tests::every_kind_of_work_is_held_to_a_ceiling_linear_in_the_entries_and_the_exclusions
    - raf_workplan::coverage::tests::an_exclusion_that_another_matches_first_is_marked_and_only_one_that_matches_nothing_is_reported
    - raf_workplan::coverage::tests::the_index_finds_the_first_match_and_marks_what_testing_every_exclusion_finds
    - raf_workplan::coverage::tests::equal_exclusions_on_a_hand_built_list_are_both_marked_and_the_first_is_the_match
    - raf_workplan::test_crate::strict::a_bound_package_outside_scope_is_enforced_under_strict_and_an_unbound_one_is_not
    - raf_workplan::test_crate::strict::a_package_of_scope_is_judged_by_the_default_rule_once_under_strict
    - raf_workplan::test_crate::strict::an_exclusion_of_the_record_takes_a_gap_of_a_bound_package_out
  examples:
    - raf_workplan::Workplan::coverage_report
depends: [cap:workplan-document-model, cap:source-index, cap:workplan-evidence-claims]
---

# Two-way coverage

## Purpose

The document model checks that what is documented exists: every code binding
resolves to an item and every test binding to a test. The opposite directions ask
whether what exists is documented. This capability adds code to documentation
(public items that no capability explains) and tests to claims (tests that support
no documented guarantee), so that built functionality and its documentation stay
aligned in both directions.

## Contract

**The direction table.**

| Direction | Question | Check |
| --- | --- | --- |
| Documentation -> code | Where is this documented capability implemented? | Every code binding resolves (`workplan check`) |
| Code -> documentation | Where is this supported behaviour explained? | Every public item of a crate in the coverage `scope` is bound by a capability, or excluded with a reason |
| Documentation -> tests | Which tests address this guarantee? | Every test binding resolves (`workplan check`) |
| Tests -> claims | Which documented guarantees does this test support? | Every test of a crate in the coverage `scope` is bound by a capability, or excluded with a reason |
| Change -> documentation | Which explanations, examples and contracts must change? | The reconcile obligation list |
| Evidence -> claims | Which current statements depend on this result? | The [evidence](./workplan-evidence-claims.md) dependency view |

**The public surface.** The public items of a crate are the exported items of each
of its library and binary targets, as the [source index](./source-index.md) lists
them: what another crate can name through the crate root, a `pub` item of a private
module only through a `pub` re-export, and the members of an exported type or trait
under its path. An item that `#[doc(hidden)]` hides is not public surface. A
re-export of another crate is not an item of the re-exporting crate: an item of
another crate of the workspace counts in that crate, and a crate outside the
workspace is not counted. An item exported under several paths is one item and is
reported under the first of them in path order. A binary's items are `pub(crate)` or private in
practice, since no other crate can name them, so the public surface of a binary
target is empty in practice and the code direction asks nothing of it; the tests
of a binary package count in the tests direction like those of a library.

**Unresolved re-exports.** A public re-export of something of the workspace that the
source index cannot follow to an item, such as a type a macro generates, is reported
as an `unresolved re-export` with its exported path, the path it names and its
declaration's file and line. It cannot be bound itself; it is covered by a `code`
binding of a module that its exported path or the path it names is below, or it is
excluded with a reason. The path it names is read from the module that declares it,
the innermost inline module whose braces hold the declaration or else the module of
its file, as `rustc` reads a `use` path: `crate::` from the crate root, a path that
begins with the name of a crate of the workspace from that crate, `self::` and any
other bare path from that module, and each `super::` one module up. So
`pub use super::gen::Made;` in `src/api.rs` names `made::gen::Made`: a binding of
`made::gen` covers it, and a binding of `made::api`, which the path leaves, does
not.

**Covered.** A public item is covered when a `code` binding of a capability resolves
to it or to an item it is below: binding a type covers its methods and associated
items, binding a trait its members, and binding a module every item below it. A
binding is followed to the definition, so an item bound through a re-export is
covered under every path it is exported by. A crate root covers nothing below it.
A module binding means the same for the digests of evidence and assessment inputs:
it stands for the module's whole text, so an edit inside it moves them. Only `code`
bindings cover items: an item bound in `examples` alone is a gap. A
test is a function marked `#[test]` in any target, unit tests in the `#[cfg(test)]`
modules of a library or a binary included, which count like any other test; a
documentation test is bound in `examples` and is not a test of this check. A test
is covered when a `tests` binding resolves to it. A unit test that only guards
internals is a gap like any other until it is bound or its module is excluded with
the reason.

**Code to documentation.** `cargo workplan coverage code` lists, for every package
of the workspace, the public items that no capability binds and no exclusion takes
out, each with its crate, its path and its file and line, then the items the
exclusions take out with their reasons, then any surface that cannot be listed.

**Tests to claims.** `cargo workplan coverage tests` lists the tests that no
capability binds and no exclusion takes out, in the same form.

**The coverage record.** `work/coverage/default.md` names in `scope` the packages
whose coverage `check` enforces, by package name (`raf-okf`, not the crate name
`raf_okf`), and holds the exclusions. A `scope` entry that is no package of the
workspace is the finding `scope-crate-missing`, which names the package when the
entry is its crate name. An exclusion is an `item` and a non-empty `reason`, and
the item has one of two forms: an item path of two segments or more
(`raf_okf::Body`), which takes out the item exported or defined there, or a prefix
`<module>::*` below a crate root (`raf_okf::body::*`), which takes out every item
and test below the module and never one of a sibling module whose name starts with
the same text. When several exclusions match, the first in the record gives the
reason. A whole crate is never excluded: an exclusion of a crate root (`raf_okf` or
`raf_okf::*`) is refused when the record loads, and the crate is left out of
`scope` instead. An exclusion with an empty reason is an error of the record, and an
exclusion that matches no public item and no test of the workspace is an error
(`coverage-exclusion-unmatched`); each names the record.

**What `check` enforces.** For each crate of `scope`, an unbound public item is
`coverage-item-unbound` and an unbound test `coverage-test-unbound`, at the file and
line of the item, an unbound unresolved re-export is `coverage-reexport-unbound` at
its declaration, and a surface that cannot be listed is
`coverage-surface-unreadable` at the record's `scope`. A crate outside `scope` is not
enforced, and its gaps are in the reports. While a capability document exists and
does not load, `check` reports none of the three unbound kinds: what that document
binds is not in the graph, so every item and test it binds would read as bound by no
capability, and its own findings already say what is wrong and fail the check; the
gaps are judged once it loads. A change record that does not load binds nothing and
leaves the gaps reported, and a surface that cannot be listed and an exclusion that
matches nothing are reported either way. A repository with capability documents and
no coverage record has one coverage finding and no other, `coverage-record-missing`,
whose fix is to add the record; a record that does not load is reported by its own
findings and not as missing. `scope: []` enforces no crate and says so in the
repository. These kinds are in the closed list of finding kinds.

**Strict mode.** Under [strict mode](./workplan-strict.md) the same two directions
are also enforced for every package that holds an item a `code`, `tests` or
`examples` binding of a loaded capability resolves to, with the same four kinds and
the same exclusions; the message of a gap of a package that only strict mode enforces
says which capability binds which path of it, in place of the coverage record, and a
surface of such a package that cannot be listed is reported at the `bindings` of
that capability. The strict rules compute that coverage themselves, so `check` under
`default` and the reports are the same whatever the strictness. The coverage record
also takes the keys `strictness`, `ceilings` and `exceptions`, which strict mode and
its line report read and this capability does not; the direction table gains no
row.

**The account.** For a repository revision the coverage reports name every gap in
either direction for the whole workspace, so the current account is complete or
states what is missing.

## Usage

- `cargo workplan coverage code` prints the public items without a capability.
- `cargo workplan coverage tests` prints the tests without a bound guarantee.
- `cargo workplan coverage code --crate raf-okf` prints the report of one package,
  named by its package name or its crate name (`raf_okf`).
- `cargo workplan check` enforces both directions for the crates of the coverage
  record's `scope`, and a gap there fails the check.
- `scope: []` in `work/coverage/default.md` enforces no crate, and the reports name
  every gap all the same.

## Examples

Adding to `raf-okf` a public function `parse_partial`, and a `#[cfg(test)]` module
`body::tests` of unit tests of the private functions of `body`, gives the code report
a row for the function among its gaps:

| Gap | Crate | Path | Location |
| --- | --- | --- | --- |
| public item | raf-okf | `raf_okf::parse_partial` | crates/raf-okf/src/lib.rs:120 |

The gap closes when a capability lists the path, or a module or type it is below,
in `bindings.code`, or when the coverage record lists it with a reason. The record
takes the new unit tests out of the tests report by the prefix of their module:

```yaml
scope: [raf-okf]
exclusions:
  - item: "raf_okf::parse_partial"
    reason: "A wrapper of Document::parse that the OKF profile explains with it."
  - item: "raf_okf::body::tests::*"
    reason: "Unit tests of the private functions of the module."
```

## Constraints

- Coverage only reports. It never edits a capability document or a binding.
- Visibility is read from the source index, not guessed from naming.
- Every exclusion carries a reason, and the reasons are readable in the repository.
- A capability in `considered` or `designed` binds nothing, so the checks that need
  bindings apply from `implementing` onward.
- The surface is the one the source index lists, with its stated limits. A surface
  the index refuses to list (a public name that is ambiguous, an import cycle, a
  glob of nothing, or the item-bytes budget passed, as by several binders of one
  carried name) is an error of the report and, for a crate of `scope`, a finding;
  it is never read as an empty surface. A name that a `pub(crate)` item and a
  carried re-export of another crate share is listed as the re-export, so that item
  is not counted.
- `check` lists the surface of each library and binary target of a crate of `scope`
  or of a crate an exclusion names, once per load; the reports list every library
  and binary target of the workspace once. The order is by crate and path, the same
  on every run.

- Under strict mode `check` also lists, once for each check, the surface of each
  library and binary target of a package outside `scope` that a binding reaches.
- An exclusion takes an item or a test out of the two directions only. The line
  report of strict mode reads its own `exceptions`, so an excluded item whose lines
  nothing covers is orphan there until an exception names it with its reason.

## Explanation

A design promise needs an accounted-for realisation, and functionality that has been
built needs documentation that follows it. This is the two-way duty of the GMEOW
[Grounding Contract](https://github.com/Blackcat-Informatics/gmeow-ontology/blob/a631f73e6e39b4493d295bcd83b4deec80637f54/docs/GROUNDING.md). Checking only from documentation to code catches promises that were
not kept and misses behaviour nobody wrote down. Checking from code to documentation
and from tests to claims closes the other half.

The direction table also separates facts that become true at different times: a design
is accepted, code exists, tests address claims, executions provide evidence, and the
documentation describes the supported result. A planned feature stays visibly planned
until its implementation and documentation justify a stronger description. The
coverage reports keep the unfinished part visible, and they turn documentation
maintenance into work the system can identify: when a referenced API disappears,
the affected section receives a `binding-missing` obligation, and when a bound example
stops compiling or running it receives an `example-failing` obligation.

## Verification

- A bound public item is absent from the code report, and its unbound neighbour is
  listed.
- A fixture with a private item, a `pub(crate)` item and a `#[doc(hidden)]` item:
  none appears in the code report; a `pub` item re-exported through a private
  module does.
- Binding a module or a type covers the items below it and nothing beside it;
  binding an item by its exported or its defining path covers it, and what is below
  it, under its exported path; a crate root covers nothing below it; an item bound
  only in `examples` is a gap.
- A type a macro generates, re-exported, is an `unresolved re-export` in the report
  and `coverage-reexport-unbound` in `scope`, closed by a binding of its module or an
  exclusion; a re-export of a crate outside the workspace is not counted.
- A `<module>::*` exclusion takes out what is below the module and not a sibling
  whose name shares its text; of two matching exclusions the first gives the reason.
- A test bound by a capability is absent from the tests report, and an unbound test
  appears, a unit test included; a `<module>::tests::*` exclusion takes the unit tests
  out with its reason shown.
- An exclusion with an empty reason, and an exclusion that matches nothing, each
  fail with the file named, beside a passing neighbour.
- `check` reports the gaps of the crates of `scope` only; `scope: []` passes while the
  reports list gaps; no record gives exactly one finding, `coverage-record-missing`,
  while a record that does not load gives only its own findings and a repository
  without capability documents owes no record; a surface that cannot be listed is an error of
  the report and a finding in `scope`.
- Reports are identical across two runs on the same tree, and a crate with thousands
  of items and tests is judged with the counts, the gaps and the excluded entries of
  its report right. The work of judging grows with the entries plus the exclusions,
  counted and not timed: each entry is judged once, each exclusion is put into an
  index once, and each entry looks up each of its paths and each module above them,
  the same lookups for ten times the exclusions. The work grows with the entries at
  fixed exclusions, with the exclusions at fixed entries, and, the two grown
  together, no faster than their sum; every kind of it stays under a ceiling linear
  in their sum, which testing each exclusion against each entry passes. With no
  exclusion nothing is looked up. The packages a record needs are found by testing
  the first segment of each exclusion against each target of the workspace, so that
  work is exclusions times targets and does not grow with the entries.
- The index finds every exclusion that matches an entry, not only the first, and
  marks each: an exclusion whose entries another exclusion matches first, a prefix
  before an item or an item before a prefix, is not reported as matching nothing,
  one that matches nothing is, and the first in the record gives the reason. Over
  every item and prefix exclusion of the paths of one to four segments of `a`, `b`
  and `bc`, against every entry of one or two of those paths, the index gives the
  first match and marks the exclusions that testing each exclusion against each path
  gives, in the order of the record and in the reverse order, and the same on an
  index built again; two equal exclusions of a hand-built list are both marked and
  the first is the match.
- A `scope` entry that is a crate name and not a package name is
  `scope-crate-missing`, naming the package it means, and the package name is in
  scope.
- An item is excluded by its exported or its defining path; a surface over the item
  budget is a budget error.
- The table and the JSON of a report carry the same entries, escaped.
- This repository's coverage record has `scope: [raf-workplan, cargo-workplan]`, its
  `check` has no coverage finding, both directions of those crates are covered, and
  `raf-okf` and `raf-source`, outside the scope, are reported with their gaps.
- The documentation example of `Workplan::coverage_report` runs: two unbound public
  functions of a crate outside `scope` are the two gaps of the code direction.
- While a second capability that binds part of a crate of `scope` does not load,
  `check` reports its load finding and an exclusion that matches nothing, and no
  `coverage-item-unbound`, `coverage-test-unbound` or `coverage-reexport-unbound`;
  beside it a change record that does not load leaves the gaps reported, and the
  capability loaded with one binding dropped gives that one gap.
- A re-export the index cannot follow, written with `super::`: in a module file
  one `super::` names the item of the parent module, covered by a binding of that
  module and not by one of the declaring module; in inline modules two levels below
  the root, two `super::` reach the root, and a glob of an inline module is read
  from the module that holds it and counted; `self::` in a module file is read from
  that module.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_workplan::Workplan::coverage_report` | doctest | [Verification](#verification) |
| `raf_workplan::coverage::tests::an_exclusion_that_another_matches_first_is_marked_and_only_one_that_matches_nothing_is_reported` | test | [Verification](#verification) |
| `raf_workplan::coverage::tests::equal_exclusions_on_a_hand_built_list_are_both_marked_and_the_first_is_the_match` | test | [Verification](#verification) |
| `raf_workplan::coverage::tests::every_kind_of_work_is_held_to_a_ceiling_linear_in_the_entries_and_the_exclusions` | test | [Verification](#verification) |
| `raf_workplan::coverage::tests::the_exclusions_are_indexed_once_and_each_entry_looks_up_its_paths_and_their_modules` | test | [Verification](#verification) |
| `raf_workplan::coverage::tests::the_index_finds_the_first_match_and_marks_what_testing_every_exclusion_finds` | test | [Verification](#verification) |
| `raf_workplan::coverage::tests::the_work_grows_with_the_entries_and_no_faster` | test | [Verification](#verification) |
| `raf_workplan::coverage::tests::the_work_grows_with_the_entries_and_the_exclusions_together_and_no_faster` | test | [Verification](#verification) |
| `raf_workplan::coverage::tests::the_work_grows_with_the_exclusions_and_no_faster` | test | [Verification](#verification) |
| `raf_workplan::coverage::tests::without_exclusions_the_entries_are_judged_and_nothing_is_looked_up` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_bound_item_is_covered_and_its_unbound_neighbour_is_a_gap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_bound_test_is_covered_and_every_unbound_test_unit_tests_included_is_a_gap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_capability_that_does_not_load_gives_its_own_findings_and_no_cascade_of_gaps` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_crate_with_thousands_of_items_and_tests_is_judged` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_prefix_matches_below_its_module_and_never_a_sibling_that_shares_its_text` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_re_export_the_index_cannot_follow_is_reported_and_enforced_and_its_module_covers_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_re_export_written_through_super_names_the_path_super_reaches` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_scope_entry_is_a_package_name_and_a_crate_name_says_which_package_it_means` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_surface_over_the_item_budget_is_a_budget_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_an_error_of_the_report_and_a_finding_in_scope` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::a_unit_test_module_is_excluded_by_its_prefix_with_the_reason_shown` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::an_empty_scope_enforces_nothing_and_the_report_still_names_every_gap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::an_item_bound_only_in_examples_is_a_gap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::an_item_bound_through_a_re_export_or_its_definition_is_covered_under_its_exported_path` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::an_item_is_excluded_by_its_exported_or_its_defining_path` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::binding_an_item_covers_what_is_below_it_and_nothing_beside_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::the_code_report_lists_every_public_item_and_no_private_or_hidden_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::the_first_matching_exclusion_gives_the_reason` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::the_report_and_its_two_forms_are_the_same_on_every_run` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::the_report_of_check_and_its_findings_follow_the_scope` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::the_table_and_the_json_carry_the_same_entries_escaped` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::this_repository_enforces_the_workplan_crates_and_they_are_covered` | test | [Verification](#verification) |
| `raf_workplan::test_crate::coverage::without_a_record_check_has_exactly_one_finding_that_says_to_add_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::strict::a_bound_package_outside_scope_is_enforced_under_strict_and_an_unbound_one_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::strict::a_package_of_scope_is_judged_by_the_default_rule_once_under_strict` | test | [Verification](#verification) |
| `raf_workplan::test_crate::strict::an_exclusion_of_the_record_takes_a_gap_of_a_bound_package_out` | test | [Verification](#verification) |
<!-- raf:generated:end -->
