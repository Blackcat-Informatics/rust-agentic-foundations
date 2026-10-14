# T4 ported map: docs_contract binding tests and the raf-workplan fixtures that replace them

raf-workplan paths are under `crates/raf-workplan/`. All rules live in `src/bindings.rs`; the graph keeps each resolution
(`Graph::bindings_of`, `Binding::resolution`, `Binding::example_kind`). Task 7 removes the docs_contract originals once the thin caller exists.

| docs_contract (tests/docs_contract.rs) | raf-workplan (tests/bindings.rs unless noted) |
|---|---|
| `wrong_role` / `check_bindings` | `src/bindings.rs` `wrong_role`, `check`, `unresolved` |
| `bindings_that_name_real_symbols_resolve` | same name (plus `examples`) |
| `a_binding_to_a_missing_path_is_reported_with_its_capability` | same name; case `binding-renamed-symbol` |
| `a_binding_to_a_macro_generated_function_is_not_found` | `a_binding_to_a_macro_generated_function_is_not_found_and_says_why`; case `binding-macro-generated-symbol` (kind `binding-macro-generated`) |
| `a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found` | same name; case `binding-item-under-the-declaring-module` |
| `every_unresolved_binding_of_every_list_is_reported` | same name |
| `a_tests_binding_must_name_a_function_marked_test` | same name; cases `binding-test-is-not-a-test-function`, `binding-test-is-shipping-code` |
| `a_code_binding_must_name_a_shipping_item` | same name; cases `binding-code-in-test-module`, `binding-code-in-test-target` |
| `demo_workspace` / `bound` / `binding_violations` | tests/support/mod.rs `demo_workspace`, `DEMO_LIB`, `fixtures::bound`, `fixtures::bound_findings` |

Messages keep the precedent text: `<cap> <list> binding `<path>` does not resolve: <error>` and `... names an item that ...`.

## New beside them (no docs_contract original)
- Kinds: `binding-unresolved`, `binding-ambiguous`, `binding-wrong-role`, `binding-macro-generated`; named cases are `binding_cases()` in tests/support/cases.rs
  (15 rows, run by tests/kinds.rs in both directions).
- Re-exports: `a_binding_through_a_re_export_resolves_to_its_definition`, `the_role_is_judged_on_the_definition_a_re_export_leads_to`.
- Ambiguity: `a_path_two_globs_both_bring_in_is_ambiguous_and_names_both_definitions`.
- Examples role (G9): `an_examples_binding_is_an_example_target_or_a_documented_library_item`,
  `a_library_that_runs_no_documentation_tests_has_no_doctest_examples`.
- Macro hint scope: `the_macro_hint_is_given_only_for_a_macro_at_the_item_level_of_the_parent_module`,
  `a_macro_in_the_file_of_a_module_is_read_for_the_path_of_that_module`.
- Fix text per list: `every_finding_of_a_wrong_role_names_the_fix_for_its_list`.
- Graph view: `the_graph_keeps_each_resolution_for_the_rules_and_the_views_that_read_it`.
- List-level rules: `a_capability_that_binds_nothing_has_no_binding_findings` (designed/considered empty lists, implementing may bind nothing);
  designed-with-bindings and delivered-without-code-or-test stay raf-okf rules, surfaced in states.rs.

## Decisions
- `binding-macro-generated`: raf-source reports `NotFound` for a generated name, as for a misspelling. The rule lexes the parent module's
  text (`proc-macro2`, no syn) and reports the kind when an item-level macro invocation of that module holds the missing name as an
  identifier; otherwise the finding is `binding-unresolved`. Invocations inside function bodies and nested inline modules do not count.
- Doctest examples are stricter than G9's wording in two places, both grounded in what cargo and rustdoc do: the target must be the library
  (cargo ignores `doctest` on other targets; fixture `example-doctest-key-on-a-non-library`) and the item must not be test code (rustdoc
  skips it; fixture in `an_examples_binding_is_an_example_target_or_a_documented_library_item`).
- Existing fixtures bound the file path `src/lib.rs`; they now bind `demo::real` and `demo::test_crate::it::case`, and graph.rs
  `the_fixture_loads_with_only_its_two_unresolved_bindings` pins the two unresolved bindings that fixture holds on purpose.
- Task 7 must keep the anti-vacuity guard of `repository_bindings_resolve_in_the_source_index` (known symbols such as `raf_okf::Document` and
  `raf_source::Workspace` resolve before the verdict is trusted) in its disposition table.
- Review round: the invocation list of a module is built once per check (unit test
  `the_invocations_of_a_module_are_listed_once_however_many_bindings_miss` asserts one lex for 300 misses); a binding with several sites fails when any
  site fails (`one_failing_site_fails_a_binding_that_resolves_to_several`); the macro message is hedged ("may generate it").
