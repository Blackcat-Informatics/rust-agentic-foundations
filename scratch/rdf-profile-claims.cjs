const fs = require('fs');
const path = 'docs/capabilities/rdf-report-profile.md';
let text = fs.readFileSync(path, 'utf8');
const kernel = 'raf_rdf::test_crate::profile::';
const workplan = 'raf_workplan::test_crate::rdf_reports::';
function before(anchor, title, tests) {
  if (!text.includes(anchor)) throw new Error(anchor);
  text = text.replace(anchor, `### ${title}\n\n<!-- raf:tested-by ${tests.join(',')} -->\n\n${anchor}`);
}
before('`Graph` contains ordered', 'Ground graph invariants', [kernel+'missing_root_dangling_references_duplicates_and_relative_predicates_refuse']);
before('Absolute IRIs are validated', 'Term validation', [kernel+'absolute_iris_use_component_validation_without_normalizing_identity',kernel+'literal_language_metadata_and_relative_datatypes_are_refused_in_both_roles',kernel+'both_readers_refuse_language_datatypes_literal_subjects_and_nested_terms']);
before('The application vocabulary is', 'Vocabulary ownership', ['raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry','cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms']);
before('One fixed normalized spelling', 'Snapshot identity', [kernel+'formats_round_trip_the_same_graph_and_snapshot_in_canonical_order',kernel+'independent_parser_pins_preserve_typed_literals_controls_and_triple_terms']);
before('Canonical Turtle inserts', 'Equivalent formats and owned names', [kernel+'formats_round_trip_the_same_graph_and_snapshot_in_canonical_order','raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary',kernel+'an_external_iri_sharing_a_snapshot_prefix_is_not_local']);
before('Both readers validate the graph', 'Exact canonical acceptance', [kernel+'canonical_bytes_include_cr_and_final_lf_and_reject_alternate_escapes',kernel+'malformed_syntax_digest_and_ownership_have_valid_neighbors']);
before("Workplan's typed adapters preserve", 'Typed observations', [workplan+'findings_preserve_all_location_variants_and_refuse_unknown_kinds',workplan+'real_coverage_keeps_packages_gaps_exclusions_and_named_source_provenance',workplan+'unresolved_reexports_keep_declaration_observations_without_claiming_a_definition',workplan+'unreadable_public_surface_retains_its_structured_failure']);
before('Composite command results use', 'Shared finding append', ['cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms']);
before('The workplan projection registry below', 'Workplan projection registry', ['raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry','raf_workplan::coverage::report_projection_tests::surface_failure_projection_preserves_optional_paths_and_direction']);
before('The command extension registry is', 'Typed command registry', ['cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms','cargo_workplan::bin_crate::cargo_workplan::result_graph::tests::actual_reach_budget_refusal_projects_a_lines_error_without_partial_rows']);
fs.writeFileSync(path,text);
