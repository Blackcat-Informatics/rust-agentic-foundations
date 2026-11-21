---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-cli
title: Census command line
description: Native static Census reports, ledger inspection, migration and explicit debt publication.
state: delivered
bindings:
  code:
    - cargo_census::bin_crate::cargo_census::main
    - cargo_census::run
    - cargo_census::cli
    - cargo_census::command
    - cargo_census::error
    - raf_census::Gate::init_ledger
    - raf_census::Gate::validate_tree_registrations
    - raf_census::registered
    - raf_census::RegisteredCheck
    - raf_census::RegisteredDebtWrite
    - raf_census::RegisteredOperation
    - raf_census::DEBT_WRITE_PROTOCOL_VERSION
    - raf_census::CHECK_PROTOCOL_VERSION
    - raf_census::MAX_CHECK_RESPONSE_BYTES
    - raf_census::selftest
    - raf_census::SelfTest
    - raf_census::SelfTestCase
    - raf_census::Gate::self_test
    - raf_census::MAX_INDEX_BYTES
    - raf_census::capnp_history
    - raf_census::capnp_history::persistence::check
    - raf_census::capnp_history::persistence::admission::check
    - raf_census::capnp_history::persistence::migration::check
    - raf_census::capnp_history::reader_evidence::verify
  tests:
    - cargo_census::test_crate::delivery::init_creates_a_clean_structural_policy_and_preserves_existing_records
    - cargo_census::test_crate::delivery::init_refuses_source_and_git_administration_destinations
    - cargo_census::test_crate::delivery::production_lints::default_checks_require_passing_production_evidence_for_the_current_sources_and_policy
    - cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs
    - cargo_census::test_crate::capnp_capture::capnp_capture_refuses_an_open_compiler_import_set
    - cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request
    - cargo_census::test_crate::capnp_capture::capnp_capture_compares_layouts_with_ordered_import_directories
    - cargo_census::test_crate::capnp_capture::capnp_capture_refuses_unrecorded_or_nonregular_earlier_imports
    - cargo_census::test_crate::capnp_capture::capnp_capture_refuses_invalid_import_directory_declarations
    - cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape
    - cargo_census::test_crate::capnp_capture::capnp_capture_refuses_oversized_inputs
    - cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_compares_retained_committed_layouts
    - cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_ignores_rpc_endpoint_layouts
    - cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_protects_declaration_continuity_after_first_adoption
    - cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_refuses_invalid_bindings_and_policy_rows
    - cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_validates_current_only_layouts
    - cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_authenticates_exact_scope_without_configuring_gate
    - cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_refuses_changed_payloads_and_noncanonical_framing
    - cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_preserves_strict_findings
    - cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_binds_paired_presence_annotation
    - cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_refuses_stale_or_different_claim_scope
    - cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_verifies_exact_specimens_and_preserves_removal
    - cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_admits_committed_pairs_and_preserves_strict_findings
    - cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_requires_exact_scope_and_output_validity
    - cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_refuses_changed_or_invalid_pair_blobs
    - cargo_census::test_crate::delivery::capnp_history_checks_committed_compiler_layouts
    - cargo_census::test_crate::delivery::capnp_history_compares_data_field_annotations
    - cargo_census::test_crate::delivery::capnp_history_compares_enumerant_annotations
    - cargo_census::test_crate::delivery::capnp_history_checks_committed_file_identities
    - cargo_census::test_crate::delivery::capnp_history_refuses_inconsistent_file_identities
    - cargo_census::test_crate::delivery::capnp_history_compares_file_declaration_identities
    - cargo_census::test_crate::delivery::capnp_history_refuses_malformed_constant_types
    - cargo_census::test_crate::delivery::capnp_history_refuses_malformed_annotation_types
    - cargo_census::test_crate::delivery::capnp_history_refuses_invalid_declaration_ownership
    - cargo_census::test_crate::delivery::capnp_history_bounds_file_declaration_nesting
    - cargo_census::test_crate::delivery::capnp_history_checks_inline_rpc_methods
    - cargo_census::test_crate::delivery::capnp_history_compares_named_rpc_endpoints
    - cargo_census::test_crate::delivery::capnp_history_compares_unbound_rpc_endpoints
    - cargo_census::test_crate::delivery::capnp_history_compares_nested_generic_rpc_owners
    - cargo_census::test_crate::delivery::capnp_history_refuses_invalid_nested_generic_rpc_owners
    - cargo_census::test_crate::delivery::capnp_history_compares_implicit_rpc_parameters
    - cargo_census::test_crate::delivery::capnp_history_refuses_invalid_implicit_rpc_parameters
    - cargo_census::test_crate::delivery::capnp_history_refuses_invalid_named_rpc_endpoints
    - cargo_census::test_crate::delivery::capnp_history_compares_rpc_inheritance
    - cargo_census::test_crate::delivery::capnp_history_refuses_invalid_rpc_inheritance
    - cargo_census::test_crate::delivery::capnp_history_compares_branded_rpc_inheritance
    - cargo_census::test_crate::delivery::capnp_history_refuses_invalid_branded_rpc_inheritance
    - cargo_census::test_crate::delivery::capnp_history_refuses_unsupported_rpc_endpoints
    - cargo_census::test_crate::delivery::capnp_history_compares_required_pointer_fields_in_committed_descriptors
    - cargo_census::test_crate::delivery::capnp_history_compares_declared_pointer_pairs
    - raf_census::capnp_history::tests::invalid_slot_layouts_and_missing_references_refuse
    - raf_census::capnp_history::tests::required_annotation_refuses_invalid_values_and_undeclared_structs
    - raf_census::capnp_history::tests::deep_group_findings_refuse_as_a_whole_above_the_payload_limit
    - cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets
    - cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration
    - cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate
    - cargo_census::test_crate::delivery::module_paths_in_package_are_required_by_the_flag_beside_a_clean_run_without_it
    - cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target
    - cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records
    - cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record
    - raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats
    - raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors
    - raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors
    - raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings
    - raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers
    - cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy
    - cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines
    - cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist
    - cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds
    - cargo_census::test_crate::delivery::registered_queries_share_cli_bytes_and_refuse_before_output
    - cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory
    - cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict
    - cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set
    - cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict
    - cargo_census::test_crate::delivery::a_sealed_registry_selector_is_a_reference_without_a_ledger_entry
    - cargo_census::test_crate::delivery::a_git_reference_is_refused_by_the_registry_options_beside_a_clean_registry_neighbour
    - raf_census::selftest::tests::the_documented_detector_list_and_case_count_equal_the_registry
    - raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text
    - raf_census::selftest::tests::a_detector_that_stops_firing_or_starts_over_firing_fails_its_side
    - raf_census::selftest::tests::a_normal_form_that_merges_or_splits_wrongly_fails_its_case
  examples:
    - raf_census::example_crate::registered
    - rust_agentic_foundations::example_crate::repository_census
depends: [cap:census-ledger, cap:census-debt-ratchet, cap:census-migration, cap:census-rule-api]
---

## Purpose

Expose native duplication and ledger data without building or executing repository programs. Library Gate evaluation, native reports and explicit debt publication preserve useful structural checks while leaving consumer test and example bodies inert.

## Contract

`init` and `Gate::init_ledger` create `helpers-ledger.ttl` in the discovered workspace root. The canonical Turtle ledger selects the whole workspace under the current normal and report forms with default options and no jobs. Publication requires native Linux directory locking and resolves the workspace sources before writing. A missing source input, a destination inside Git administration, an occupied ledger or temporary path, or a competing lock causes a refusal. Existing policy and debt are preserved. Structural duplicates remain findings until resolved or explicitly adopted.
<!-- raf:tested-by cargo_census::test_crate::delivery::init_creates_a_clean_structural_policy_and_preserves_existing_records,cargo_census::test_crate::delivery::init_refuses_source_and_git_administration_destinations -->

`check` requires the production Cargo lint policy and a current, successful
Clippy receipt from the calling build by default. Supply the build-owned
receipt through `--lint-evidence` or `RAF_CENSUS_LINT_EVIDENCE`. The
selection is `--workspace --lib --bins --locked`. Test-only callers can
therefore leave a production helper unused. `lint-inputs` identifies the
workspace before the build and `lint-receipt` seals the build's arguments,
compiler identity, status and diagnostics against the matching inputs.
The CLI reads these values as data. `--no-production-lints` selects only
the structural ledger and debt checks. See the [consumer guide](../../CENSUS-HOWTO.md).
<!-- raf:tested-by cargo_census::test_crate::delivery::production_lints::default_checks_require_passing_production_evidence_for_the_current_sources_and_policy -->

**Cap'n Proto history.** `capnp-history` compares unpacked compiler
descriptors read from explicit base and current Git commits. The caller
selects struct, enum and interface roots by type ID. From a data root,
Census follows data type references and groups. It reports an old root or member that disappears
and changes to existing names, ordinals, types, slots, discriminants and
defaults. The layout comparison permits a defaulted field or enum variant
that preserves the old layout. Written declaration order is ignored. A field
rename can violate history policy even when Cap'n Proto allows it on
the wire. List defaults use the native representation for their element
types. Struct-valued defaults are checked as native containers and
compared by their canonical pointer bytes. The caller verifies
acceptance by application readers.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_checks_committed_compiler_layouts -->

Strict history compares applications on retained data fields by annotation
declaration ID. The application list for one ID retains its authored order
and repeated values. The descriptor supplies a declared annotation with a
field target, and the application supplies a brand and a value of the
instantiated type. Text and data retain their bytes, and pointer values
compare in their bounded canonical representation. Each changed list reports
at the owning field with the annotation ID. Explicitly selected required
and paired-presence markers use their presence policies.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_data_field_annotations -->

Census validates applications on each enumerant in the selected strict-history
closure. The annotation declares an enumerant target. The application supplies
a value of the declared type as instantiated by its brand in the enum's lexical
scope. Census compares ordered applications per ID on retained names and
reports changes at the owning enum and enumerant. A trailing enumerant with
valid applications is compatible. Rename and ordinal findings still apply.
Field presence markers with an enumerant target count as ordinary applications
on enumerants. A malformed application identifies its owning member and retains
the native wire or tag cause when present.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_enumerant_annotations -->

The strict history check accepts explicitly selected interfaces. A generic
interface can declare its own type parameters or inherit them from its lexical
scopes. It requires a unique reciprocal declaration chain and generic flags
consistent with the declared parameters in that chain. The comparison reports
changes to its own or enclosing parameter counts on the interface ID and names
the enclosing scope in that finding.
A method can use named structs or inline lists for its parameters and results.
Census reports removed, renamed or moved methods and changes to an endpoint
ID, its named or inline form, or its canonical type arguments. It compares
parameters and results independently. A new trailing method with valid
endpoints is allowed.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_checks_inline_rpc_methods,cargo_census::test_crate::delivery::capnp_history_compares_named_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_compares_nested_generic_rpc_owners,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_nested_generic_rpc_owners -->

A method's implicit type parameters supplement the interface's lexical
parameters. Census compares each kind by position within its scope and
reports a changed implicit parameter count once on the method, including
when the changed position is unused. A method brand can carry an implicit
position only within the method's declared range.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_implicit_rpc_parameters,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_implicit_rpc_parameters -->

Census follows the direct superclasses of each selected interface and its
bases. Both ends of an inheritance edge require reciprocal declaration
ownership, unique names and consistent generic flags through a file node.
A nongeneric superclass takes an empty brand. A generic superclass binds
every parameter of its declaration and enclosing scopes. Bindings can
reference positions in the derived interface's proved lexical scopes.
A missing or wrong-kind target, incomplete brand, repeated canonical edge,
or cycle refuses.

Census compares the canonical set of brands per direct superclass ID.
Reordering and valid additions are compatible, including another
instantiation of a retained superclass. Removing a direct ID or losing
a brand under a retained ID reports a strict finding on the interface
that declared the edge, even if a transitive path still reaches that base.
Each defining interface owns its methods, ordinals and endpoint layouts.
A diamond projects each defining interface once.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_rpc_inheritance,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_rpc_inheritance,cargo_census::test_crate::delivery::capnp_history_compares_branded_rpc_inheritance,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_branded_rpc_inheritance -->

A named endpoint must be a declared nongroup struct with reciprocal lexical
ownership. A generic endpoint can bind any of its own or enclosing parameters.
An omitted argument, a descriptor's unbound entry and an unconstrained
`AnyPointer` compare alike, including in nested type arguments. Each bound
argument retains its position. Census refuses repeated or foreign scopes,
scope rows without declared parameters and non-pointer arguments.
The generic endpoint and its method interface each require a unique
reciprocal declaration chain. A method can bind its own implicit positions
and its interface's lexical parameter positions. A named endpoint can inherit
any lexical parameter scope that also belongs to its declaration chain.
The endpoint's generic flags must agree with its own and enclosing parameter
declarations.
Several methods and roles can share one endpoint declaration with different
brands. Census compares its layout once by type ID using ordinary struct
evolution, including implicit defaults for optional additions. Changed
enclosing parameter counts report on the named declaration even if every
method role leaves its arguments unbound.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_named_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_named_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_compares_unbound_rpc_endpoints -->

A compiler-generated inline list belongs to one method ordinal and endpoint
role. It must be a detached struct without a lexical declaration claim and
must declare the method's implicit parameters in the same order, even if the
role has no fields. Its generic state reflects those declarations and the
interface's parameter-bearing scopes. The inline brand inherits each lexical
parameter scope exactly once. Fields can use ordinary positions from their
own wrapper and lexical positions from the owner. The detached ID still
identifies the role layout. Fields use explicit ordinals in order from zero and
the same layout checks as ordinary data, including referenced types. Census checks each
retained inline input's explicit-default marker and requires an explicit
default for each appended inline input. It also compares the layout when a
renamed method retains its ordinal. Direct selection of a detached endpoint
and wrappers shared across method roles refuse.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_checks_inline_rpc_methods,cargo_census::test_crate::delivery::capnp_history_refuses_unsupported_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_named_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_compares_implicit_rpc_parameters,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_implicit_rpc_parameters -->

Persistence declarations bind declared data structs and enums. An
interface-typed pointer records its nominal identity as part of the data shape.
The interface's inheritance and method layouts belong to strict interface
history.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_refuses_invalid_bindings_and_policy_rows,cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_ignores_rpc_endpoint_layouts -->

Strict history can also select compiler file IDs. Each selected base ID
must have one requested-file row with a canonical repository-relative
filename and a file node with the same display name. A retained file
reports a changed filename. When both the requested row and its node
disappear, Census reports removal even if a different file ID uses the
old filename. Additional unrelated file requests can coexist with the
selected IDs.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_checks_committed_file_identities -->

A selected file with a missing row or node, a conflicting path owner,
a non-file node or an inconsistent name refuses the descriptor.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_refuses_inconsistent_file_identities -->

**Declaration identities.** File selection also follows written structs, enums,
interfaces, constants and annotations by compiler ID. Census reports each old
declaration that leaves the selected files or changes its local name, immediate
enclosing scope or kind. Additional declarations can coexist with the old
inventory. Renaming a file or moving a parent preserves the local name and
immediate scope of an otherwise unchanged descendant.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_file_declaration_identities,cargo_census::test_crate::delivery::capnp_history_checks_committed_file_identities -->

The compiler supplies each declaration's name and owner through `nestedNodes`
and `scopeId`. Census requires reciprocal ownership and unique names in each
selected scope, including when a file is removed. It refuses competing claims
from an unrelated namespace. Partial unrelated namespaces can otherwise coexist
with the selected inventory. Each file admits up to 64 declaration levels.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_refuses_invalid_declaration_ownership,cargo_census::test_crate::delivery::capnp_history_bounds_file_declaration_nesting -->

**Constant types.** Strict file history reports a declared type change
for a retained constant. A valid constant addition or a value change at
a fixed type is compatible. Census requires an explicit type in each
selected constant and checks its nominal IDs, kinds and brands through
the descriptor type reader. The type comparison uses only the needed
declaration metadata. Layout checks follow the independent `--type`
selection.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_file_declaration_identities,cargo_census::test_crate::delivery::capnp_history_refuses_malformed_constant_types -->

**Annotation declarations.** Strict file history compares each retained
annotation's declared type and reports each removed applicability
target. An added target or valid annotation is compatible. Census reads
the compiler's twelve target flags as a set. The wildcard and the
equivalent explicit list yield the same set, and the empty set is valid.
A selected annotation requires an explicit declared type with valid
nominal references and brands. Layout checks follow the independent
`--type` selection.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_file_declaration_identities,cargo_census::test_crate::delivery::capnp_history_refuses_malformed_annotation_types -->

A caller can select a compiler-declared Void field annotation for
required pointer presence. Census validates the marker in both
snapshots. It reports a newly marked ordinal in a struct reached from
the selected roots in both snapshots, including a marked append or a
field with an unchanged read default. It includes fields in
unconditional groups. An unmarked append, an unchanged or removed
marker, and a marker on a newly reached struct add no required-field
finding. The strict layout comparison also runs.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_required_pointer_fields_in_committed_descriptors -->

A caller can select a compiler-declared `UInt16` field annotation for
paired pointer presence. The pair declares that the fields are both
present or both absent. Apply the marker once to the lower ordinal
field and give the higher ordinal as its value. Both fields must be
direct unconditional pointer slots in the same declared struct.
Census validates the marker in both snapshots and reports a new or
changed pair with both field names in a struct reached from the
selected roots in both snapshots. An unchanged or removed pair adds
no finding. Required presence and strict layout findings remain
independent of the declared pairs.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_declared_pointer_pairs -->

Capture verification authenticates both compiler captures with one
caller-selected Ed25519 key. Each signed statement binds the compiler
identity and the exact bytes of the descriptor and each requested
schema file. Census reads those bytes from the same pinned Git tree,
checks the request's compiler version, and requires each signed
source path to agree with its requested filename and file node name.
Each reported import resolves to a requested file with the reported ID.
Each literal source import also resolves to a requested file, including
imports used only in scalar defaults. A refusal produces no findings.
With capture verification omitted, the caller selects the compiled
inputs directly.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_an_open_compiler_import_set,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape,cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request -->

An ordered capture signs the compiler's import directories. Census
resolves slash-rooted imports at the first regular candidate in the
declared order. The candidate must appear in the signed source set and
the compiler's requested files. Each rooted source import needs an
import row carrying its file ID. An earlier committed file takes
precedence even if the capture omits it. An earlier nonregular candidate
or refused lookup prevents a capture verdict.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_compares_layouts_with_ordered_import_directories,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_unrecorded_or_nonregular_earlier_imports -->

`capnp-persistence` selects protected layouts from a committed policy. The
same policy path must exist at both explicit revisions. A declaration
names one direct written Rust struct or enum in its designated Cargo
package and a declared compiled root in that revision. Census validates
the reachable layout of every declaration, including additions. An
explicitly empty base policy supports first adoption. Removing a
declaration or replacing its root
produces a finding even if a different declaration protects the old root.
A retained declaration can select a moved Rust symbol with the same
compiled root. Census compares only the layouts reachable from retained
roots with unchanged IDs.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_compares_retained_committed_layouts,cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_protects_declaration_continuity_after_first_adoption,cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_refuses_invalid_bindings_and_policy_rows,cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_validates_current_only_layouts -->

`capnp-reader-admission` compares one stable persistence declaration at
the two pinned revisions. A removed or rebound declaration remains a
policy finding. For a retained root, the command reports strict findings
and evaluates reader coverage independently. A trusted signed claim of
old-shape acceptance can admit the reader after Census verifies the exact
specimens named by the base declaration. Strict findings remain visible.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_preserves_strict_findings,cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_refuses_stale_or_different_claim_scope,cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_verifies_exact_specimens_and_preserves_removal -->

`capnp-migration-admission` compares one stable persistence declaration at
the two pinned revisions. A removed or rebound declaration is an uncovered
policy finding. A trusted signer affirms that the selected migrator accepts
the old shape, produced the listed outputs and satisfies the destination
application contract. Census verifies the exact signed pairs against input
blobs in the base and output blobs in the separate evidence revision. It
checks digests and canonical raw Cap’n Proto segments. Strict findings remain
visible independently of admission.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_admits_committed_pairs_and_preserves_strict_findings,cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_requires_exact_scope_and_output_validity,cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_refuses_changed_or_invalid_pair_blobs -->

`capnp-reader-evidence` verifies a signed affirmative claim for the
caller-selected base revision, reader revision, root type, reader
artifact and entry point, and exact payload paths. It reads the
evidence from a separate pinned commit and the payloads from the base
commit. After verifying every digest and raw segment, it prints the
verified paths in repository order and exits 0. A verification refusal
exits 1 with a diagnostic. The command runs independently of Gate
registration and of the strict schema-history verdict.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_authenticates_exact_scope_without_configuring_gate,cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_refuses_changed_payloads_and_noncanonical_framing -->

**Static contract.** `check` evaluates the native library Gate for the selected workspace. `index`, `job`, `count` and `fingerprint` derive complete native reports from that Gate. The CLI accepts no package/example executable selector and never invokes Cargo, tests, examples or custom repository binaries. Custom Rust registrations remain explicit library API operations performed by their caller; repository contents cannot instantiate them automatically.
<!-- raf:tested-by cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

A repository binary supplies a Gate configuration callback to `cargo_census::run`. The callback installs repository rules and ledger extension validators after the front end validates the arguments. `job`, `count` and `fingerprint` use the configured Gate and the same index queries and renderers as the stock binary.

`baseline` in the registered binary validates the ledger's rule registrations and extensions before rendering with the default Census options. The registered checks cover rule fixtures and required rules. Job rule references resolve against built-in detectors, declarative patterns and registered rules. Source matching, reference selection and debt evaluation belong to the other Gate operations. The stock baseline requires only the workspace source.

Both carriers preserve the stock report bytes when an accepted extension is the only ledger difference. A refused registration exits before the query writes a report to stdout or a file.
<!-- raf:tested-by cargo_census::test_crate::delivery::registered_queries_share_cli_bytes_and_refuse_before_output -->

`baseline` discovers structure through Census with default options without adopting debt. `dump-ledger` reads the selected native ledger. Reports use canonical RDF with explicit format selection and bounded complete index serialization. `--out` exclusively creates a new .ttl or .nt data file after serialization; existing files, source extensions and Git administration destinations refuse. Stdout mode writes no repository data.
<!-- raf:tested-by cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration -->

`debt --write` uses the native debt ratchet, permitting adoption and shrink while refusing growth. Migration reads explicit input carriers and writes the converted result to stdout. Malformed inputs and unsupported declarations fail actionably.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record -->

**Module paths in their package.** `check` and `debt --write` accept `--require-module-paths-in-package`, which selects `Gate::require_module_paths_in_package` for that run: each module `path` attribute that resolves to a file of another package, or of none, is a hard finding that `check` prints and that makes `debt --write` refuse with exit 1 and leave the debt record unchanged. Without the option no module path is judged, so one workspace is clean without it and refused with it. The option belongs to the command line alone: no ledger, report or debt record carries it, so every existing ledger, report and debt record keeps its exact bytes and the report form does not change. A persisted switch would first need the report-form rule applied, and a repository states the requirement where it runs the command. The other commands refuse the option by name.
<!-- raf:tested-by cargo_census::test_crate::delivery::module_paths_in_package_are_required_by_the_flag_beside_a_clean_run_without_it -->

**Declared reference tree.** `check`, `index`, `job`, `count`, `fingerprint` and `debt` accept one reference tree outside the lock graph. `--reference-repo LOCATION` with `--reference-commit COMMIT`, or with `--reference-pin-file FILE --reference-pin-key KEY`, compares with that repository's committed objects at the commit; the pin file is a workspace-relative TOML file and the key a dotted path to a string. `--reference-unpinned DIRECTORY` compares with a directory's current files, every index and query marks its receipts `unpinned-tree`, and every evaluating command prints one notice on stderr naming the directory while its findings and stdout stay identical to the pinned run; `debt --write` refuses it. A relative location is resolved from the main checkout of the workspace's Git repository. Any other combination of these options refuses with a message naming the accepted forms, and the commands that do not evaluate the Gate refuse every reference option rather than ignoring it. A ledger `reference` with no job homing it bans copies of that dependency: `check` exits 1 with one `REFERENCE-COPY` line per copy, `debt --write` refuses the same finding, and `index`, `job`, `count` and `fingerprint` evaluate the same inputs. A selector that matches nothing, a missing source or a malformed option refuses every one of these commands with no verdict on stdout.
<!-- raf:tested-by cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory,cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict,cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set,cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict,cargo_census::test_crate::delivery::a_sealed_registry_selector_is_a_reference_without_a_ledger_entry -->

**Registry dependencies sealed by the lock file.** The same six commands accept the options that run the reference-copy ban against registry dependencies, with no sibling checkout and no pinned commit. `--seal-registry SELECTOR` (repeatable; an exact package name or a `prefix-*` family) seals those registry packages and makes each a reference for the ban exactly as a ledger `reference` selector is (a ledger with no `reference` entry still bans a copy of a package named by `--seal-registry`, and without the option nothing is implied), `--registry-cache DIRECTORY` names Cargo's `registry/cache` directory (default `$CARGO_HOME/registry/cache`, then `$HOME/.cargo/registry/cache`), and `--registry-features PACKAGE=FEATURE,FEATURE` states the complete feature set the workspace enables on a sealed package, once per package (`PACKAGE=` for none). Giving any of them selects the locked check of the library (`Gate::check_registry` and `Gate::write_debt_registry`, the path of the library route in [census-reference-scan](census-reference-scan.md)) for `check`, `index`, `job`, `count`, `fingerprint` and `debt`, so all six read one set of sealed sources. The command reads archive bytes from the cache and authenticates each against the checksum in `Cargo.lock`; it fetches nothing, starts no process and runs no Cargo command. It runs from a checkout of this repository or an installed `cargo-census` built from a pinned commit, the consumer needs no sibling tree, `cargo fetch --locked` must already have populated the cache, and the command performs no network access. The caller states the complete feature set of every sealed package; the loader refuses a set it cannot verify. A copied body fails `check` with exit 1 and one `REFERENCE-COPY` line, and an absent, mismatched or oversized archive, a sealed package without a declared feature set, a declaration for a package that is not sealed, an incomplete or unknown feature, a selector or declaration the lock file and manifest disagree about, and a Git reference refuse every one of these commands with exit 1, the refusal on stderr and an empty stdout. The commands that do not evaluate the Gate refuse these options too.
<!-- raf:tested-by cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set,cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict,cargo_census::test_crate::delivery::a_sealed_registry_selector_is_a_reference_without_a_ledger_entry,cargo_census::test_crate::delivery::a_git_reference_is_refused_by_the_registry_options_beside_a_clean_registry_neighbour -->

**Self-test.** `self-test` and the library call `Gate::self_test` run a fixed set of seeded in-memory workspaces through the same evaluation as `check`, and report one `pass ID` or `fail ID: reason` line per case, then a `N cases, M failed` line. The command needs no workspace and `--workspace` is ignored. It reads and writes no file, creates nothing and prints no path or clock reading, so two runs of one build print identical text. The exit status is 0 only when every case holds. The cases check the evaluation engine, each built-in detector, the sanction and ratchet logic and the two grouping behaviours of the normal form, all over in-memory sources; a census that finds nothing where a case seeds a copy fails that named case instead of passing. No case reads the on-disk tree backend or compares the running build with any other. Rules registered on the Gate join the run as one `rule/ID/fixtures` case each, from their own positive and negative fixtures; required rules, extensions and reference roots do not apply.
<!-- raf:tested-by cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines,cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist,cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds,raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text -->

The stock self-test covers enum catch-all decisions through the required-rule
route, including qualified variants, imported variants, bare bindings, and
clean neighboring matches.

The cases, in the order they run:

- `normal-form/renamed-respelled-copy-groups`: two bodies of one algorithm that differ in parameter and local names, a dropped value adapter and an integer spelling form one body group.
- `normal-form/same-size-other-control-flow-separates`: two bodies of the same token count whose loop and branch differ stay ungrouped.
- `constant/respelled-forbidden-constant-is-a-copy`: a forbidden integer written in another spelling counts as a copy.
- `detector/ID/positive` and `detector/ID/negative` for each built-in detector in registry order: every fixture of its positive side is flagged and every fixture of its negative side leaves the workspace clean. The detectors are `length-prefix-frame`, `le-slice-int`, `align-up-mask`, `div-ceil-by-hand`, `xor-first-mismatch`, `hex-format-loop`, `hex-pair-radix`, `hex-digit-radix`, `hex-table`, `json-pointer-escape`, `home-literal`, `sole-name`, `vocabulary-literal`, `std-default-hasher`, `foreign-host-import`, `serde-json`, `rdf-text-writer`, `rdf-term-model`, `reifier-default-graph`, `rdf-list-walk` and `enum-catch-all`.
- `sanction/distinct-body/exact-set-holds`, `sanction/distinct-body/stale-member` and `sanction/distinct-body/stale-set`; `sanction/distinct-shim/exact-set-holds`; `sanction/distinct-multiplicity/exact-count-holds` and `sanction/distinct-multiplicity/stale-count`: an exact distinct sanction is clean, and a member that no longer resolves, a set equal to no current group or a changed source-unit count is a finding.
- `sanction/variant-detector/exact-site-holds`, `stale-path`, `stale-file` and `stale-detector`, and `sanction/variant-isomorphic/exact-site-holds` and `stale-detector`: an exact variant is classified as a variant and leaves the gate clean, and a path, file or detector that no longer matches is a finding.
- `debt/exact-record-holds`, `debt/rising-copies-refused`, `debt/falling-copies-require-lowering`, `debt/new-group-refused` and `debt/resolved-group-requires-lowering`: the ratchet accepts the exact record, refuses growth and requires a lowering write after shrinkage.
- `rule/ID/fixtures` for each rule registered on the Gate.
<!-- raf:tested-by raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text,raf_census::selftest::tests::a_detector_that_stops_firing_or_starts_over_firing_fails_its_side,raf_census::selftest::tests::a_normal_form_that_merges_or_splits_wrongly_fails_its_case,raf_census::selftest::tests::the_documented_detector_list_and_case_count_equal_the_registry -->

## Usage

Run `cargo census init` once from the new workspace or a member directory, then run `cargo census check --no-production-lints` to check the structural policy. For the default `cargo census check`, first capture the production compiler receipt. Inspect the ledger with `cargo census dump-ledger --format turtle`. Use `cargo census debt --write` to adopt existing duplication explicitly.
<!-- raf:tested-by cargo_census::test_crate::delivery::init_creates_a_clean_structural_policy_and_preserves_existing_records -->

Run `cargo census capnp-history --base BASE --current CURRENT` with
`--base-descriptor PATH --current-descriptor PATH`. Select at least one
type or file ID. Commit IDs use all 40 lowercase hexadecimal digits.
Type and file IDs use a `0x` prefix followed by 16 lowercase hexadecimal
digits with the high bit set. Descriptor paths are relative to the Git
repository root. Run from that root or name it with `--workspace`.
Capture the compiler request with `capnp compile -o- SCHEMA` and commit
it with the schema. Library consumers call `raf_census::capnp_history::check`.
The command exits 0 for a clean comparison or 1 with findings on stdout.
An input or resource refusal exits 1 with a diagnostic on stderr.

Use `--type 0xID` to select a struct, enum or supported interface.
For an interface, the comparison follows its superclasses and each reached
method's named and inline parameter and result structs.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_rpc_inheritance,cargo_census::test_crate::delivery::capnp_history_compares_named_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_checks_inline_rpc_methods -->

Use `--file-id 0xID` to compare a requested schema file and its written
declarations. Repeat `--file-id` for each selected file. Use `--type` to
compare compiled layouts, including generated group and inline method data
where their type or interface is selected. `--type` and `--file-id` may be
used together. Use each selected ID once.

Add `--required-annotation 0xID` to compare required pointer presence
or `--paired-presence-annotation 0xID` to compare paired pointer
presence. Both options can be used together. Library callers supply
the selections and annotation IDs through `HistoryOptions`.

To authenticate captures, supply `--capture-key HEX` and the four
repository-relative paths `--base-capture PATH`,
`--base-capture-signature PATH`, `--current-capture PATH` and
`--current-capture-signature PATH`. Use all 64 lowercase hexadecimal
digits of the Ed25519 public key. Library callers set
`HistoryOptions::capture` with the same key and both pairs of paths.

Produce the request with every schema input explicitly listed. For a
single-root capture, use
`capnp compile --no-standard-import --import-path=ROOT --src-prefix=ROOT -o-`
followed by those paths. Use the selected schema directory for both
options. For an ordered capture, use `--no-standard-import` and
`--src-prefix=ROOT` with one `--import-path` per declared directory
in signed order. Request the rooted import targets before the files
that import them, using their physical paths. Each requested filename
must be relative to `ROOT` and match its physical path. Sign the exact
canonical N-Triples bytes and store the raw 64-byte Ed25519 signature
as a separate committed blob.

Build the statement with `raf_rdf::Builder` and render it as
`Format::NTriples`. Its class and properties use the
`https://blackcatinformatics.ca/raf/report/1#` namespace. Each subject
has one `rdf:type` and the following fields.

| Class | Required properties |
| --- | --- |
| `CapnpCaptureV1` at the graph root | `compilerVersion` uses decimal major, minor and patch joined by dots. `compilerSha256` identifies the compiler artifact. `descriptorFile` is the descriptor path from the Git root and `descriptorSha256` covers its bytes. `schemaRoot` is the path from the Git root, with an empty string for the root itself. One or more `schemaSource` IRIs select owned source rows. |
| `CapnpCaptureV2` at the graph root | The same properties as `CapnpCaptureV1` plus one or more `importDirectory` IRIs selecting owned directory rows. |
| `CapnpSchemaSource` at an owned fragment | `file` is the requested filename relative to `schemaRoot`. `sha256` covers the source bytes. |
| `CapnpImportDirectory` at an owned fragment | `path` is the directory relative to `schemaRoot`, with an empty string for that root itself. `position` is a canonical nonnegative `xsd:integer` giving its zero-based search position. |

Scalar properties have exactly one value. `position` uses the integer
datatype and the other scalar properties use strings. Digests use 64
lowercase hexadecimal digits. The Git-backed capture fixtures build
and sign these records before the command reads them.

Run `cargo census capnp-persistence --base BASE --current CURRENT
--base-descriptor PATH --current-descriptor PATH --policy PATH`. Give the
repository root with `--workspace` when running elsewhere. The policy path
is canonical and relative to that root. Use the same optional
annotation and capture flags as for `capnp-history`. Library consumers
call `raf_census::capnp_history::persistence::check`. A clean comparison
exits 0. Findings go to stdout with exit 1. A refused comparison exits 1
with a diagnostic on stderr and no findings.

Render the policy as canonical N-Triples with the same RDF namespace as
compiler captures. Each subject has one `rdf:type` and the following fields.

| Class | Properties |
| --- | --- |
| `CapnpPersistencePolicyV1` at the graph root | Zero or more `persistedType` IRIs select owned declarations. |
| `CapnpPersistedType` at an owned fragment | `package` names the Cargo package, `symbol` gives the exact source-index definition path, and `rootType` gives the nominal compiled type ID. Each is one string. For admission, the current row supplies one `readerEntry` or `migrationEntry` string for the selected operation, and the base row supplies one or more canonical `specimenPath` strings. Entry points and specimens are optional for strict persistence checks. |

Run `cargo census capnp-reader-admission` with the revision, descriptor
and policy flags of `capnp-persistence` plus `--declaration SLUG`. To offer
reader coverage, give `--evidence-commit COMMIT --statement PATH
--signature PATH --key HEX --reader-sha256 HEX`. Select all five options
together. The current row selects the reader entry point and the base
row selects the exact specimen paths. The evidence commit is distinct
from the base and current revisions. Library consumers call
`raf_census::capnp_history::persistence::admission::check`.

A clean comparison or admitted reader exits 0. The command prints
strict findings even when a claim supports reader admission, then
names the admitted declaration and its verified specimens. Uncovered
strict findings exit 1. Invalid evidence for a retained declaration
refuses with a diagnostic. The command uses pinned inputs independently
of Gate registration.

Use canonical N-Triples in the capture vocabulary for admission claims.
The caller-selected Ed25519 key authenticates the statement. The
`CapnpReaderAdmissionV1` graph root requires these properties.

| Properties | Value |
| --- | --- |
| `policyFile`, `declaration`, `baseCommit`, `currentCommit`, `rootType` | Exact policy path, stable row slug, full Git pins and retained root ID as strings. |
| `baseDescriptorFile`, `baseDescriptorSha256`, `currentDescriptorFile`, `currentDescriptorSha256` | Exact committed descriptor paths and SHA-256 digests as strings. |
| `comparisonProfile`, `requiredAnnotation`, `captureVerified` | The string `strict-schema-history-v1`, the annotation ID string or `none`, and a canonical boolean matching whether compiler captures are selected. |
| `readerEntry`, `readerSha256` | The exact policy-selected entry point and expected artifact SHA-256 as strings. |
| `acceptsOldShape`, `acceptsSpecimens`, `payload` | Canonical boolean true for both claims and the exact base specimen rows of class `CapnpAcceptedPayload`. |

Include one `pairedPresenceAnnotation` string with the selected
annotation ID when the comparison uses `--paired-presence-annotation`.
Omit this property when the comparison omits that option.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_binds_paired_presence_annotation -->

Run `cargo census capnp-migration-admission` with the revision, descriptor
and policy flags of `capnp-persistence` plus `--declaration SLUG`. Supply
`--evidence-commit COMMIT --statement PATH --signature PATH --key HEX
--migration-sha256 HEX` together to offer a claim. The evidence commit is
distinct from both comparison revisions. Library callers use
`raf_census::capnp_history::persistence::migration::check`.

The signed `CapnpMigrationAdmissionV1` root uses the same policy,
declaration, revisions, root type, descriptors, comparison profile,
annotations and capture mode fields as reader admission. Use canonical
N-Triples and a detached Ed25519 signature over the exact statement bytes.

| Properties | Value |
| --- | --- |
| `migrationEntry`, `migrationSha256` | The current row's entry point and the expected artifact SHA-256 as strings. |
| `acceptsOldShape`, `producedOutputs`, `outputsValid` | Three canonical boolean true values. The signer attests to old-shape acceptance, production of the listed outputs and their application validity. |
| `specimenPair` | One owned `CapnpMigrationSpecimenPair` for each input path in the base row. Each pair supplies `inputFile`, `inputSha256`, `outputFile` and `outputSha256` as strings. |

Use canonical repository-relative paths and SHA-256 values of 64 lowercase
hexadecimal digits. Claims admit at most 128 pairs and each committed blob
has a two MiB limit. Several pairs may share an output path with the same
digest. A clean comparison or admitted migration exits 0. Census prints
strict findings, the admitted declaration and verified pairs sorted by input
path, with a tab separating the input and output paths. Uncovered findings
exit 1. An invalid claim for a retained declaration
refuses with a diagnostic.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_admits_committed_pairs_and_preserves_strict_findings,cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_requires_exact_scope_and_output_validity,cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_refuses_changed_or_invalid_pair_blobs -->

Run `cargo census capnp-reader-evidence --base BASE --current CURRENT
--evidence-commit COMMIT --statement PATH --signature PATH --key HEX
--type 0xID --reader-entry NAME --reader-sha256 HEX --payload PATH`.
Repeat `--payload` for each base path. The evidence commit contains the
canonical N-Triples statement and its detached 64-byte Ed25519
signature. Use full lowercase commit IDs, a lowercase hexadecimal
Ed25519 public key and SHA-256 digest, and canonical paths from the
Git root. Library callers use
`raf_census::capnp_history::reader_evidence::verify`.

Use the same RDF namespace as compiler captures.

| Class | Required properties |
| --- | --- |
| `CapnpReaderEvidenceV1` at the graph root | `baseCommit`, `currentCommit`, `rootType`, `readerEntry` and `readerSha256` are strings. `accepted` is canonical `xsd:boolean` true and affirms acceptance of every referenced payload. One or more `payload` IRIs select owned payload rows. |
| `CapnpAcceptedPayload` at an owned fragment | `file` gives a base payload path and `sha256` gives its digest. Both properties are strings. |

The documented data path is illustrated by `raf_census::example_crate::registered` and `rust_agentic_foundations::example_crate::repository_census`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

To host the full front end, depend on `cargo-census` and return its status from
your executable's `main`. Supply an `install_policy` function that accepts
the selected `raf_census::Gate` and returns it with the repository's rules and
validators.

```rust
fn main() -> std::process::ExitCode {
    cargo_census::run(
        std::env::args_os(),
        Some(install_policy),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    )
}
```

The executable accepts `job ID`, `count ID`, `fingerprint SYMBOL` and
`baseline` directly, with the standard workspace and output options.

Run `cargo census visibility` for the [workspace library visibility census](./census-visibility.md).

Before `cargo census check`, [capture a production Clippy receipt](../../CENSUS-HOWTO.md#capture-the-production-lint-result) and export its absolute path as `RAF_CENSUS_LINT_EVIDENCE`. Use `cargo census check --no-production-lints` to check structural policy alone.

Select a workspace with `--workspace DIRECTORY`. Use `cargo census check`, `cargo census index`, `cargo census baseline`, `cargo census job ID`, `cargo census count ID` or `cargo census fingerprint SYMBOL`. Use `--max-index-bytes` for an explicit bounded index allowance. Seal registry dependencies with `cargo census check --seal-registry acme-core --registry-features acme-core=default,std` after `cargo fetch --locked`. Require module paths to stay in their package with `cargo census check --require-module-paths-in-package`, and pass the same option to `debt --write`. Compare with a sibling repository at a reviewed commit with `cargo census check --reference-repo ../upstream --reference-commit COMMIT`, or name the commit in a pin file with `--reference-pin-file pins.toml --reference-pin-key upstream.rev`. Run `cargo census self-test` in a consumer gate before `check` to prove the installed census still detects seeded copies; it needs no workspace. A library caller uses `Gate::self_test` and reads `SelfTest::passed`, `SelfTest::cases` and its `Display` text. Help describes native migration and explicit debt options.

`cargo census embedded report|check` uses the separate source-region analysis.
`cargo census embedded debt create|shrink` explicitly publishes its independent
allowance record. See [embedded structural clones](census-embedded-clones.md)
for its syntax profile, resource-admission limits and shrink-only contract.

## Examples

A repository may contain a build script, example and test whose bodies write a marker and panic. `index` reads their source and returns the same native index as the direct library call. None of those bodies executes and no target directory is created. Exporting to src/lib.rs refuses beside successful creation of report.ttl. The stock `self-test` run in an empty directory prints `pass` lines for every case and `62 cases, 0 failed`, exits 0 and leaves the directory empty. A Gate with a registered rule whose positive fixture produces no hit reports a `fail rule/ID/fixtures` line naming the refused fixture. The registered front end includes the repository's rule cases and returns exit 1 when a case fails.

## Constraints

A capture uses UTF-8 schema source and double-quoted, unescaped import
paths. The profile refuses `embed` expressions. A relative import resolves
from its requesting file's directory. Parent segments stay within the
signed schema root and must resolve through the committed directory
entries. A slash-rooted import in a single-root capture resolves from
the signed schema root.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape,cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request -->

An ordered capture declares one to 16 distinct committed import
directories under the schema root, with consecutive positions starting
at zero. Its slash-rooted import suffixes are canonical paths without
parent segments. The pinned tree's work limits also apply to candidate
lookups. The profile refuses rooted source imports missing from the
compiler's import table, including a scalar default whose import the
compiler omits. Ambient and external directories are outside the profile.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_compares_layouts_with_ordered_import_directories,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_unrecorded_or_nonregular_earlier_imports,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_invalid_import_directory_declarations -->

A statement is limited to 128 KiB and an individual schema blob to
two MiB, subject to the pinned tree's existing read limits. The source
scan admits up to 8192 literal imports per capture.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_refuses_oversized_inputs,cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request -->

A persistence policy admits at most 128 KiB and 96 declarations. Each
owned fragment is a stable slug of at most 64 bytes. Start with an ASCII
lowercase letter, and separate nonempty lowercase alphanumeric words
with hyphens. Package and symbol names are nonempty and use at most
1024 bytes without control characters. Census indexes declared Cargo
members from the pinned repository root and admits only unique direct
definitions in first-party library or binary source outside test code.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_refuses_invalid_bindings_and_policy_rows -->

Reader evidence uses the same statement and pinned-tree read limits.
Select one to 128 distinct payloads of up to two MiB each. The entry
point has at most 1024 bytes and contains no control characters. Census
interprets each payload as one raw Cap'n Proto segment and checks its
canonicality with finite traversal and nesting limits.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_authenticates_exact_scope_without_configuring_gate,cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_refuses_changed_payloads_and_noncanonical_framing -->

Reader admission uses the same specimen confinement, framing and limits.
Census computes both descriptor digests over the bytes it compares.
It verifies compiler captures when selected. Specimen-only evidence
cannot establish a shape-wide claim.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_refuses_stale_or_different_claim_scope,cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_verifies_exact_specimens_and_preserves_removal -->

- Missing native registrations or unavailable implemented capabilities refuse rather than invoking a repository program. Imported protocol data is not an observed execution verdict. A report write failure may leave the newly created owned data file incomplete; it never truncates an existing destination.
<!-- raf:tested-by cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

Debt publication requires the native Linux retained-directory capability and
filesystem support for its locking and durability operations. No lock file is
created before loading inspected source. Unsupported publication fails without
a pathname-based writer or another locking protocol.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target,cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records -->

## Explanation

The native Gate supplies the policy verdict and complete configured job reports. Safe library reuse replaces the executable target transport. Reading Git-backed source uses native read-only object access without fetching or hooks.

The persistence declaration supplies the association between the Rust
definition and the schema root. The source index verifies the written
definition. The descriptor supplies the compiled layout. Application
requirements and reader or migration admission need separate evidence.
Signed reader evidence covers its exact selected specimens and clears
no declaration or strict layout finding.

Reader admission accepts a separate assertion from the trusted signer
that the identified reader accepts the declared old shape. The specimen
check corroborates the named bytes and framing. Census does not execute
the reader or generalize its behavior from the specimens.

The capture signer attests that the declared compiler and invocation
produced the descriptor from the complete input set. The compiler in the
capture recipe derives each requested filename and file node name from
the same source name. Census uses one bounded compiler request reader for
capture corroboration and layout projection. Its source scan checks
literal imports that the compiler can omit from its request, while the
request can report implicit imports such as the streaming schema.
The ordered profile checks the compiler's search order against the
whole pinned tree, because a source absent from the signed set can
take precedence over a signed file with the same relative name.

The reader evidence signer attests the executable identity, nominal
root type and acceptance of the selected bytes. Census independently
checks each pinned payload's digest and canonicality. The separate
evidence commit lets the signer bind both full revisions without
changing either one.

## Verification

The file history fixtures admit a value-only edit and a new constant,
report a retained type change, and refuse a missing type, a missing or
wrong-kind nominal target and a malformed brand. The foreign fixture
leaves unrelated declarations and the nominal layout outside the file
comparison. A separate selection checks that layout.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_file_declaration_identities,cargo_census::test_crate::delivery::capnp_history_refuses_malformed_constant_types -->

The annotation compiler fixtures admit widening, a valid addition and
equivalent target sets. They report a retained type change and each of
the twelve targets removed when a declaration narrows to the empty set.
A controlled descriptor compares a valid nominal type beside an absent
type and a malformed brand.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_file_declaration_identities,cargo_census::test_crate::delivery::capnp_history_refuses_malformed_annotation_types -->

The compiler-produced RPC diamond accepts reordered and added direct bases.
It reports one changed inherited method default and one removed direct base
when a transitive route still reaches the formerly direct base. Controlled requests pair a
valid inheritance graph with malformed edges, ownership failures and a cycle.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_rpc_inheritance,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_rpc_inheritance -->

Branded inheritance fixtures bind concrete and lexical types to a nested
superclass and accept distinct instantiations of one direct base. They
report a lost instantiation on the derived interface, and a mixed-brand
diamond reports one changed default on the defining method. An explicit
self-binding and an inherited scope normalize to the same brand.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_branded_rpc_inheritance,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_branded_rpc_inheritance -->

Migration fixtures compare strict and admitted verdicts for the same
committed declarations. They distinguish base inputs from current and
checkout copies and verify outputs from the signed evidence revision.
Signed validity, exact scope, pair membership, digests and segment framing
have refusal cases beside successful verification.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_admits_committed_pairs_and_preserves_strict_findings,cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_requires_exact_scope_and_output_validity,cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_refuses_changed_or_invalid_pair_blobs -->

The Cap'n Proto fixture compares real compiler output in committed Git
objects while the worktree contains different bytes. Selected layouts
refuse a null request root, missing nodes, invalid ordinals, incompatible
list storage and overlapping active slots. Union arms may share storage.
Each descriptor is limited to two
MiB, with a traversal grant of 1,048,576 words, a nesting limit of 64
and member names of at most 256 bytes. A result exceeding two MiB of
counted finding payload refuses the entire comparison.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_checks_committed_compiler_layouts,raf_census::capnp_history::tests::invalid_slot_layouts_and_missing_references_refuse,raf_census::capnp_history::tests::deep_group_findings_refuse_as_a_whole_above_the_payload_limit -->

The required-field check refuses a missing or invalid declaration, a
duplicate application, and an application with an absent or non-Void
value. A marker must apply to an ordinary pointer slot in a declared
struct. A group, a scalar, a synthesized RPC parameter or result, and
a field activated by any enclosing union arm are invalid placements.
The marker and marked structs must have consistent declaration paths
to a file within the nesting bound.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_required_pointer_fields_in_committed_descriptors,raf_census::capnp_history::tests::required_annotation_refuses_invalid_values_and_undeclared_structs -->

Signed compiler fixtures give a layout finding across two valid
captures. They refuse a wrong key, invalid signature, stale
descriptor and stale direct or transitive source. Separate cases
refuse a missing requested row, an unused import without a target,
a mismatched file node name, a known file ID on the wrong path, and
a path above the schema root. The closed fixture includes an in-root
parent path and a slash-rooted import. A scalar-default fixture accepts
both declared files and refuses the capture with the dependency
omitted. It also exercises escaped import paths and an embed expression.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_an_open_compiler_import_set,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape,cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request -->

Two compiler requests use opposite directory orders for the same
rooted import. Both captures admit comparison and report the differing
field type. The test also refuses an order inconsistent with the
descriptor, a requested filename alias, and a rooted scalar-default
import omitted from the compiler's import table. Other cases exercise
an absent earlier candidate, an unrecorded earlier file, a nonregular
candidate, and invalid directory declarations.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::capnp_capture_compares_layouts_with_ordered_import_directories,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_unrecorded_or_nonregular_earlier_imports,cargo_census::test_crate::capnp_capture::capnp_capture_refuses_invalid_import_directory_declarations -->

Reader evidence fixtures verify an exact old corpus despite different
bytes at the reader revision. A registered host's callback stays inert.
The fixtures exercise wrong keys, revisions, reader identities and
payload sets, a reused evidence revision, payload resource limits,
a stale digest, a negative verdict, and a payload with a signed digest
whose raw segment fails canonicality.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_authenticates_exact_scope_without_configuring_gate,cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_refuses_changed_payloads_and_noncanonical_framing -->

Persistence fixtures select a struct and an enum from committed Rust
source. A compatible append and unrelated layout change pass even
after a Rust symbol moves. A persisted field rename and enum member
removal produce findings. The fixtures also exercise first adoption,
declaration removal and rebinding, missing policy and schema roots,
invalid Rust bindings, and a malformed layout in a newly selected root.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_compares_retained_committed_layouts,cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_protects_declaration_continuity_after_first_adoption,cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_refuses_invalid_bindings_and_policy_rows,cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_validates_current_only_layouts -->

Reader admission fixtures retain the same strict incompatibility with
and without admitted coverage. They refuse stale revisions, another
declaration with the same root, mismatched descriptor paths or digests,
different comparison settings, and changed specimen scope or bytes.
They also preserve a policy removal finding with a supplied claim.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_preserves_strict_findings,cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_refuses_stale_or_different_claim_scope,cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_verifies_exact_specimens_and_preserves_removal -->

Controlled fixtures protect native CLI parity, the self-test case list, the absence of consumer execution, report destination restrictions, migration carrier bounds, monotone debt writes and the declared reference options.

The declaration history fixtures cover compatible additions, a partial import,
local identity changes and the independent type layout verdict. Malformed scope
claims and excessive nominal nesting refuse without a partial report.

The data-field fixtures select a named result. They cover an annotated appended
field, relative order across distinct annotation IDs, repeats within one ID,
changed Text and List values, and equal bytes under different brands. The same
method carries a valid annotation on an inline input. An invalid declaration target
or a value inconsistent with its applied brand refuses the descriptor.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_data_field_annotations -->

The named RPC fixtures check an optional field with an implicit default,
one finding per changed field in a shared layout, and a change of endpoint
form at the same node ID. A change from Text to Data in a nested type
argument or an enclosing lexical argument reports a type finding on the
affected method role. An endpoint with only enclosing bindings passes.
Foreign, repeated and parameterless scope rows refuse, as do non-pointer
arguments, ambiguous lexical ownership and inconsistent generic flags.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_named_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_named_rpc_endpoints -->

The unbound endpoint fixtures compare omitted and explicit arguments in
both endpoint and enclosing scopes. The descriptor fixture also checks
an unbound entry and a shortened argument list. Changing a bound position
in a nested argument reports a type change on its method role. Adding an
unused enclosing parameter reports one finding for the shared endpoint.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_unbound_rpc_endpoints -->

The file-scoped generic interface fixtures accept an appended method and
compare owner arity, an inherited inline slot and a bound named role.
They admit both named roles inheriting the same owner scope and report
one layout finding per changed named field. Competing ownership, missing
inline brands and out-of-range owner parameter positions refuse.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_named_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_named_rpc_endpoints,cargo_census::test_crate::delivery::capnp_history_refuses_unsupported_rpc_endpoints -->

The nested interface fixtures bind named and inline roles across multiple
lexical scopes. They accept an appended method and report changes to an
inline slot, a named role's binding and a named endpoint's field. An interface
that only inherits parameters reports the changed enclosing count even when
its detached role IDs and slot types stay stable. Missing, repeated, foreign
and bound inline scope rows refuse, as does an inconsistent owner flag.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_nested_generic_rpc_owners,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_nested_generic_rpc_owners -->

The implicit RPC fixtures accept an appended generic method. They distinguish
method and lexical positions with the same index, report a named role's
changed binding, and report one arity finding for an empty method's unused
parameter. Out-of-range positions, conflicting declarations and references
to another wrapper refuse. A raw implicit position also refuses in inline
slots and ordinary data types.
<!-- raf:tested-by cargo_census::test_crate::delivery::capnp_history_compares_implicit_rpc_parameters,cargo_census::test_crate::delivery::capnp_history_refuses_invalid_implicit_rpc_parameters -->

A persisted interface pointer keeps its data verdict across changes in
RPC inheritance and method layouts, including a stable branded pointer
to a generic owner with changed implicit method parameters.
<!-- raf:tested-by cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_ignores_rpc_endpoint_layouts -->

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::command::tests::self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_authenticates_the_exact_pinned_inputs` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_compares_layouts_with_ordered_import_directories` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_reads_literal_dependencies_missing_from_the_request` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_refuses_an_open_compiler_import_set` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_refuses_import_identity_mismatch_and_root_escape` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_refuses_invalid_import_directory_declarations` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_refuses_oversized_inputs` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::capnp_capture_refuses_unrecorded_or_nonregular_earlier_imports` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_admits_committed_pairs_and_preserves_strict_findings` | test | [Contract](#contract), [Usage](#usage) |
| `cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_refuses_changed_or_invalid_pair_blobs` | test | [Contract](#contract), [Usage](#usage) |
| `cargo_census::test_crate::capnp_capture::migration_admission::capnp_migration_requires_exact_scope_and_output_validity` | test | [Contract](#contract), [Usage](#usage) |
| `cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_compares_retained_committed_layouts` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_ignores_rpc_endpoint_layouts` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_protects_declaration_continuity_after_first_adoption` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_refuses_invalid_bindings_and_policy_rows` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::persistence::capnp_persistence_validates_current_only_layouts` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_binds_paired_presence_annotation` | test | [Usage](#usage) |
| `cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_preserves_strict_findings` | test | [Contract](#contract) |
| `cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_refuses_stale_or_different_claim_scope` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::reader_admission::capnp_reader_admission_verifies_exact_specimens_and_preserves_removal` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_authenticates_exact_scope_without_configuring_gate` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::capnp_capture::reader_evidence::capnp_reader_evidence_refuses_changed_payloads_and_noncanonical_framing` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::a_git_reference_is_refused_by_the_registry_options_beside_a_clean_registry_neighbour` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::a_sealed_registry_selector_is_a_reference_without_a_ledger_entry` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_bounds_file_declaration_nesting` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_checks_committed_compiler_layouts` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_checks_committed_file_identities` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_checks_inline_rpc_methods` | test | [Contract](#contract), [Usage](#usage) |
| `cargo_census::test_crate::delivery::capnp_history_compares_branded_rpc_inheritance` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_data_field_annotations` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_declared_pointer_pairs` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_enumerant_annotations` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_file_declaration_identities` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_implicit_rpc_parameters` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_named_rpc_endpoints` | test | [Contract](#contract), [Usage](#usage) |
| `cargo_census::test_crate::delivery::capnp_history_compares_nested_generic_rpc_owners` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_required_pointer_fields_in_committed_descriptors` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_compares_rpc_inheritance` | test | [Contract](#contract), [Usage](#usage) |
| `cargo_census::test_crate::delivery::capnp_history_compares_unbound_rpc_endpoints` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_inconsistent_file_identities` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_invalid_branded_rpc_inheritance` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_invalid_declaration_ownership` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_invalid_implicit_rpc_parameters` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_invalid_named_rpc_endpoints` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_invalid_nested_generic_rpc_owners` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_invalid_rpc_inheritance` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_malformed_annotation_types` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_malformed_constant_types` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::capnp_history_refuses_unsupported_rpc_endpoints` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records` | test | [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::init_creates_a_clean_structural_policy_and_preserves_existing_records` | test | [Contract](#contract), [Usage](#usage) |
| `cargo_census::test_crate::delivery::init_refuses_source_and_git_administration_destinations` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy` | test | [Verification](#verification) |
| `cargo_census::test_crate::delivery::module_paths_in_package_are_required_by_the_flag_beside_a_clean_run_without_it` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::production_lints::default_checks_require_passing_production_evidence_for_the_current_sources_and_policy` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::registered_queries_share_cli_bytes_and_refuse_before_output` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::registry_options_refuse_by_name_without_a_partial_verdict` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::report_exports_refuse_source_existing_files_and_git_administration` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines` | test | [Contract](#contract) |
| `raf_census::capnp_history::tests::deep_group_findings_refuse_as_a_whole_above_the_payload_limit` | test | [Verification](#verification) |
| `raf_census::capnp_history::tests::invalid_slot_layouts_and_missing_references_refuse` | test | [Verification](#verification) |
| `raf_census::capnp_history::tests::required_annotation_refuses_invalid_values_and_undeclared_structs` | test | [Verification](#verification) |
| `raf_census::example_crate::registered` | example target | [Usage](#usage) |
| `raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers` | test | [Verification](#verification) |
| `raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings` | test | [Verification](#verification) |
| `raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors` | test | [Verification](#verification) |
| `raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors` | test | [Verification](#verification) |
| `raf_census::selftest::tests::a_detector_that_stops_firing_or_starts_over_firing_fails_its_side` | test | [Contract](#contract) |
| `raf_census::selftest::tests::a_normal_form_that_merges_or_splits_wrongly_fails_its_case` | test | [Contract](#contract) |
| `raf_census::selftest::tests::every_case_holds_on_this_build_and_two_runs_print_the_same_text` | test | [Contract](#contract) |
| `raf_census::selftest::tests::the_documented_detector_list_and_case_count_equal_the_registry` | test | [Contract](#contract) |
| `raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats` | test | [Verification](#verification) |
| `rust_agentic_foundations::example_crate::repository_census` | example target | [Usage](#usage) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-debt-ratchet](./census-debt-ratchet.md) |
| Depends on | [census-ledger](./census-ledger.md) |
| Depends on | [census-migration](./census-migration.md) |
| Depends on | [census-rule-api](./census-rule-api.md) |
<!-- raf:generated:end -->
