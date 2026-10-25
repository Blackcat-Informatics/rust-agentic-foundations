---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:rdf-report-profile
title: "Canonical ground RDF reports"
description: "The closed shared RDF 1.2 ground profile, snapshot identities, and typed domain projections."
state: delivered
bindings:
  code:
    - raf_rdf::graph
    - raf_rdf::Builder

    - raf_rdf::read
    - raf_rdf::write
    - raf_rdf::term
    - raf_rdf::Error
    - raf_rdf::Format
    - raf_rdf::Graph
    - raf_rdf::Builder
    - raf_rdf::Node
    - raf_rdf::Value
    - raf_rdf::Triple
    - raf_rdf::vocab
    - raf_workplan::report
    - raf_census::Baseline::to_ntriples
    - raf_census::Index::to_ntriples
    - raf_census::DebtRecord::to_ntriples
    - raf_census::RegisteredCheck::to_ntriples
    - raf_census::RegisteredDebtWrite::to_ntriples
    - cargo_workplan::bin_crate::cargo_workplan::result
    - cargo_workplan::bin_crate::cargo_workplan::result_graph
  tests:
    - raf_census::test_crate::reports::canonical_reports_preserve_golden_output_documentation_and_roundtrip
    - raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats
    - raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors
    - raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement
    - raf_rdf::test_crate::profile::duplicate_native_membership_and_real_subject_queries_have_valid_neighbors
    - raf_rdf::test_crate::profile::parsed_statement_duplicates_and_repeated_source_bytes_refuse_with_valid_neighbors
    - raf_rdf::test_crate::profile::malformed_escapes_and_predicates_refuse_beside_canonical_neighbors
    - raf_rdf::test_crate::profile::caller_supplied_large_literals_are_validated_without_a_profile_byte_budget
    - raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot

    - cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms
    - cargo_workplan::bin_crate::cargo_workplan::result_graph::tests::actual_reach_budget_refusal_projects_a_lines_error_without_partial_rows
    - rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_first_party_manifests_have_no_json_result_dependency
    - rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class
    - rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them
    - raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot
    - raf_rdf::test_crate::profile::roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs
    - raf_rdf::test_crate::profile::absolute_iris_use_component_validation_without_normalizing_identity
    - raf_rdf::test_crate::profile::absolute_iri_and_literal_profile_refusals_keep_valid_neighbors
    - raf_rdf::test_crate::profile::roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs
    - raf_rdf::test_crate::profile::exact_bytes_include_cr_final_lf_and_multibyte_diagnostic_neighbors
    - raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals
    - raf_rdf::test_crate::profile::an_external_iri_sharing_a_snapshot_prefix_is_not_local
    - raf_rdf::test_crate::profile::abbreviations_blank_nodes_and_quads_are_outside_the_closed_profile
    - raf_rdf::test_crate::profile::ground_validation_does_not_apply_domain_shapes_or_datatype_value_spaces
    - raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals
    - raf_rdf::test_crate::profile::pinned_registry_expansions_remain_distinct_from_native_canonical_bytes
    - raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary
    - raf_workplan::test_crate::rdf_reports::findings_preserve_all_location_variants_and_refuse_unknown_kinds
    - raf_workplan::test_crate::rdf_reports::real_coverage_keeps_packages_gaps_exclusions_and_named_source_provenance
    - raf_workplan::test_crate::rdf_reports::unreadable_public_surface_retains_its_structured_failure
    - raf_workplan::test_crate::rdf_reports::unresolved_reexports_keep_declaration_observations_without_claiming_a_definition
    - raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry
    - raf_workplan::coverage::report_projection_tests::surface_failure_projection_preserves_optional_paths_and_direction
    - raf_workplan::coverage::report_projection_tests::registry_collection_visits_quoted_atoms_and_both_datatype_positions
    - cargo_workplan::test_crate::evidence::recorded_binding_failures_keep_failed_and_unresolved_outcomes_distinct
    - cargo_workplan::test_crate::assess::moved_inputs_preserve_raw_unresolved_reasons_only_for_unresolved_changes
  examples:
    - raf_rdf::example_crate::report

depends: []
---

## Purpose

`raf-rdf` owns the closed report profile and format-independent snapshot identity
used by census and workplan reports. PurRDF owns RDF storage, mutable assertion
membership, indexed queries, term representation, parsing, term traversal,
lexical output, hashing and standard vocabularies.
Domain consumers own their closed classes, properties, cardinalities and semantic
invariants. A ground graph is not proof that a census or workplan result is valid.

## Contract

### Native storage and membership
<!-- raf:tested-by raf_rdf::test_crate::profile::duplicate_native_membership_and_real_subject_queries_have_valid_neighbors,raf_rdf::test_crate::profile::parsed_statement_duplicates_and_repeated_source_bytes_refuse_with_valid_neighbors,raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement -->

The parser's native `RdfDataset` is the authoritative immutable storage, retained
through `Arc`. Ordinary assertions, reifiers and annotation layers all contribute
to the ground report meaning. `Builder` constructs membership through the native
mutable dataset and `finish` freezes a validated `Graph`. `Graph::get` returns a
borrowed `Node` only for an asserted subject, and `Node::values` yields borrowed
native `Value` views. Interning an object or predicate does not create a subject.
RAF retains no parallel assertion map. A statement repeated across physical
native representations is refused rather than silently dropped.

`Node::objects` streams borrowed outgoing objects from the same native subject
query, including reifier and annotation assertions. It yields each whole object;
a triple term's components remain available through `Value::quoted`. Predicate
and datatype IRIs are not emitted as separate object values. Domain code can
traverse ownership without copying assertions into a second graph.
<!-- raf:tested-by raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement -->

### Ground graph invariants
<!-- raf:tested-by raf_rdf::test_crate::profile::roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs -->

`Graph` contains duplicate-free ground triples. Its top-level subjects
are the owned root or nonempty ASCII percent-encoded fragments. Predicates and
literal datatypes are absolute IRIs. Objects are named IRIs, typed literals or
depth-one RDF 1.2 triple terms whose object is atomic. Every owned reference,
including references inside triple terms, must name a subject, and the root must
exist. Quoted subjects can be owned or external named IRIs. Duplicate triples,
missing roots, dangling owned references and invalid term roles are errors.

### Term validation
<!-- raf:tested-by raf_rdf::test_crate::profile::absolute_iris_use_component_validation_without_normalizing_identity,raf_rdf::test_crate::profile::absolute_iri_and_literal_profile_refusals_keep_valid_neighbors,raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals -->

Absolute IRIs are validated with the shared RFC 3987 parser, retaining their exact
lexical spelling. No URI normalization rewrites identity. The profile excludes
blank nodes, named graphs, quads, comments, language-tagged literals and recursive
triple terms. The language-only datatypes `rdf:langString` and `rdf:dirLangString`
are refused in this profile. Numeric lexical and range semantics
belong to the domain consumer. Native computed-scalar constructors emit canonical
integer and boolean text. Census retains the upper half of the u128 range with
native typed literals because the computed integer constructor accepts i128.

### Vocabulary ownership
<!-- raf:tested-by raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry,cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms -->

The application vocabulary is `https://blackcatinformatics.ca/raf/report/1#`.
Standard structural terms are `rdf:type` and `rdf:reifies`; literal primitives use
the XML Schema namespace. Domain registries enumerate the terms they permit.

### Census vocabulary
<!-- raf:tested-by raf_census::test_crate::reports::canonical_reports_preserve_golden_output_documentation_and_roundtrip,raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats,raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors -->

Engine-owned Census report terms occupy the shared versioned namespace.
The inventory covers baseline, index, debt and registered response outcomes.

| Census class | Properties and object types |
|---|---|
| `Baseline` | `normalForm`, `reportForm` version integers; `minTokens` positive integer; `unitCount` nonnegative integer |
| `Index` | `normalForm`, `reportForm` integers |
| `BodyGroup`, `ShimGroup` | `fingerprint` normal-form fingerprint IRI; `tokens` integer; `member` links to units |
| `Unit` | `symbol`, `file` strings; `line` positive integer; `startByte` nonnegative integer; index units also carry `package` string, `tokens` integer and `shipping` boolean; optional `body`, `shim` fingerprint IRIs; `holds` links to hexadecimal-table IRIs |
| `Job` | `jobId`, `homeSymbol` strings; `copies` integer; `match` links to units |
| `Match` | `matchClass` match-role IRI; standard RDF reifies links the quoted job-to-unit match |
| `Home`, `Variant`, `Copy` | Enumerated match-role IRIs, with no owned properties |
| `SharedConstant` | `value` integer greater than 65535; `packageCount` integer at least two and no greater than the member count; `member` links to units |
| `DebtRecord` | `normalForm`, `reportForm` integers |
| `BodyDebt`, `ShimDebt` | Repeated `memberSymbol` strings; optional `multiplicity` links to counted members |
| `MemberMultiplicity` | `symbol` string; `copies` integer greater than one |
| `JobDebt` | `jobId` string; `copies` positive integer |

| Registered Census class | Properties and object types |
|---|---|
| `RegisteredCheck` | `protocolVersion` fixed protocol IRI; `status` string; `findingCount` integer; `finding` links to owned findings; `errorMessage` string only for evaluation error |
| `CensusFinding` | `ordinal` nonnegative integer; `message` string |
| `RegisteredDebtWrite` | `protocolVersion` fixed protocol IRI; `status` string; `groupCount`, `jobCount` integers only for a written response; `errorMessage` string only for publication error |

The inventories cover engine-emitted classes, properties and enumerated object
IRIs in the shared namespace, including references inside triple terms. Literal
datatypes in that namespace are checked too. Counts are nonnegative unless a row
states a stricter bound. Each inventory must equal the terms observed from actual
native serializers in both directions: an undocumented emission and a declared
term with no emission are failures. These tables inventory vocabulary and RDF
object types. Complete per-class cardinalities, scalar ranges and admitted
versions remain governed by the domain shape rules.

Consumer extension IRIs retain their consumer-owned meaning. Inspecting a
declared ledger preserves its entire graph; it does not register arbitrary
caller-provided IRIs as built-in report vocabulary. The registered check and
debt protocol IRIs, status values and outcome shapes are defined by the
[Census command contract](./census-cli.md).

### Snapshot identity
<!-- raf:tested-by raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot,raf_rdf::test_crate::profile::pinned_registry_expansions_remain_distinct_from_native_canonical_bytes -->

One fixed normalized spelling supplies identity: UTF-8 `VERSION "1.2"` followed
by LF, then one complete-predicate Turtle triple per line in the graph's derived
term order, each ending ` .` and LF. Owned nodes use `<>` or `<#fragment>`.
Terms use PurRDF's native canonical term output and `TermValue` ordering. Subject
and predicate keys use exact string ordering; object values use native `Ord`.
The snapshot BASE is `https://blackcatinformatics.ca/.well-known/genid/raf-report/`
followed by the full lowercase BLAKE3 digest of those normalized bytes. No BASE
line participates in the preimage. This is an unkeyed integrity check, not an
authentication claim, and relies on the cryptographic collision assumption.

### Equivalent formats and owned names
<!-- raf:tested-by raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot,raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary,raf_rdf::test_crate::profile::an_external_iri_sharing_a_snapshot_prefix_is_not_local -->

Canonical Turtle inserts exactly `BASE <snapshot>` and LF after the version
line. Canonical N-Triples has no directives and expands each owned root/fragment
with that same BASE. Both formats preserve the same absolute ground graph and
snapshot identity. Selection of output format never changes the preimage.
The root sorts before fragments; the N-Triples reader obtains its BASE from the
unique root subject. It normalizes only that exact absolute root and its nonempty
validated fragments, never an arbitrary textual prefix or another digest.
Construction maps local names through a private owner IRI and rejects caller
IRIs that alias that owner. Parsed names are normalized only against the selected
actual report owner; a longer external IRI sharing its prefix remains external.

### Exact canonical acceptance
<!-- raf:tested-by raf_rdf::test_crate::profile::exact_bytes_include_cr_final_lf_and_multibyte_diagnostic_neighbors,raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals -->

Both readers validate the graph, recompute identity and compare exact selected
canonical bytes. Unsupported syntax, digest edits, reordered triples, alternate
escapes, whitespace changes, CRLF and a missing or extra final LF are errors.
Canonical spelling errors identify the first differing line including newline
bytes. Error display escapes untrusted control characters.

### Typed observations
<!-- raf:tested-by raf_workplan::test_crate::rdf_reports::findings_preserve_all_location_variants_and_refuse_unknown_kinds,raf_workplan::test_crate::rdf_reports::real_coverage_keeps_packages_gaps_exclusions_and_named_source_provenance,raf_workplan::test_crate::rdf_reports::unresolved_reexports_keep_declaration_observations_without_claiming_a_definition,raf_workplan::test_crate::rdf_reports::unreadable_public_surface_retains_its_structured_failure -->

Workplan's typed adapters preserve ordered result membership with named result
nodes and explicit zero-based ordinal values. Finding locations retain their
line, key or section; document locations omit all three. Item coverage retains
package totals, scope, gaps, exclusions, reasons and public-surface failures.
SourceLocation nodes reify the exact reported item-name assertion and name its
file and line. An unresolved re-export points to its declaration and written
target, not a resolved definition. These observations do not assert a source
content digest or test execution.
Human tables and command exit policy remain stated by their respective
capabilities; this profile owns their shared carrier and vocabulary.

### Shared finding append
<!-- raf:tested-by cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms -->

Composite command results use the same finding projection through `finding_into`
on the shared `Builder`;
there is no second finding-field encoder. The caller supplies its owned subject,
membership and ordinal. A failed append requires discarding the partial builder.

### Workplan projection registry
<!-- raf:tested-by raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry,raf_workplan::coverage::report_projection_tests::surface_failure_projection_preserves_optional_paths_and_direction -->

The workplan projection registry below names application terms; every class has
one `rdf:type`. Required properties occur once unless stated otherwise. String
fields preserve their exact text, integer counts are nonnegative, and membership
ordinals preserve the order of the typed report. Source line values are positive.
`reportForm` is 1 for these
projections. Census's separate report profile and full closed shapes are stated
in [the normal-form architecture](../architecture/normal-form.md#reports).

| Class | Properties and object types |
|---|---|
| `Report` | `reportForm` integer, `command` string; `result` owned node zero or more; `direction` string for coverage only |
| `Finding` | `ordinal` integer, `kind`, `file`, `message`, `fix` strings; optional mutually exclusive `line` integer, `section` string or `key` string |
| `CoveragePackage` | `ordinal`, `items`, `coveredItems`, `tests`, `coveredTests` integers; `package` string, `inScope` boolean |
| `CoverageGap` | `ordinal`, `line` integers; `kind`, `package`, `path`, `item`, `file` strings; `inScope` boolean; `source` owned node |
| `CoverageExcluded` | All `CoverageGap` properties, then `exclusion`, `reason` strings |
| `CoverageError` | `ordinal` integer; `package`, `target`, `root`, `failure`, `message` strings; `inScope` boolean; optional `failurePath` string for carry-budget only |
| `SourceLocation` | `rdf:reifies` triple term; `file` string and `line` integer |

### Typed command registry
<!-- raf:tested-by cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms,cargo_workplan::bin_crate::cargo_workplan::result_graph::tests::actual_reach_budget_refusal_projects_a_lines_error_without_partial_rows -->

The command extension registry is `COMMAND_TERMS`, disjoint from `TERMS` above.
The following table uses shared properties from the first registry where their
class-specific meaning applies. Every result and ordered child has one `ordinal`
integer, including reference, range, invocation and argument rows. Child
properties link to owned named nodes; repeated child links preserve sequence
through those ordinals. Optional scalar properties are absent. All counts are
nonnegative integers; `exitCode` alone is signed. Decimal measurements remain
exact string data. The command root remains the `Report` above, with
`reportForm` 1. These typed results do not introduce a generic object/array value
or a string containing another machine serialization.

| Command class | Properties and object types |
|---|---|
| `Rewritten`, `Completed` | `path` string |
| `Preview` | `path`, `capability`, `text` strings |
| `State` | `state` string, `count` integer; `capability` ordered `Capability` nodes |
| `Capability` | `id`, `slug`, `title`, `path` strings; `gap` ordered `BindingGap` nodes; `openChange` ordered `ChangeReference` nodes |
| `BindingGap` | `list`, `message` strings; optional `binding` string |
| `ChangeReference` | `slug` string |
| `OpenChanges` | `count` integer; `change` ordered `OpenChange` nodes |
| `OpenChange` | `id`, `slug`, `path` strings; `capability` ordered `CapabilityReference` nodes |
| `CapabilityReference` | `id` string |
| `NewChange` | `slug`, `path`, `template` strings, `syncNeeded` boolean; `capability` ordered `CapabilityReference` nodes; `affects` ordered `SectionReference` nodes; `guidance` ordered `Guidance` nodes |
| `SectionReference` | `subject` string |
| `Guidance` | `capability`, `state`, `advice` strings |
| `Impact` | `change` string; `capability` ordered `CapabilityReference` nodes; `section` ordered `SectionReference` nodes; `binding` ordered `BindingResolution` nodes; `dependant` ordered `Dependant` nodes; `link` ordered `SectionLink` nodes |
| `BindingResolution` | `capability`, `list`, `path` strings, `resolves` boolean; optional `reason` string |
| `Dependant` | `capability`, `dependsOn` strings |
| `SectionLink` | `from`, `target` strings |
| `Obligation` | `kind`, `subject`, `section`, `path`, `evidence` strings; optional `line` positive integer |
| `Refused` | `failures` integer, `message` string |
| `Refusal` | `condition`, `source`, `message` strings, `repeated` boolean; optional `number` integer; optional `obligation` node of class `Obligation` |
| `Binding` | `outcome`, `capability`, `list`, `path`, `message` strings; optional `reason` string; optional `listed`, `selected`, `passed`, `failed`, `ignored`, `okLines` integer proof counts; `run` ordered `VerificationRun` nodes |
| `VerificationRun` | `programKind`, `ended` strings; optional `exitCode` signed integer; `argument` ordered `Argument` nodes |
| `Argument` | `text` string |
| `EvidenceRecorded` | `path`, `id`, `verdict`, `subject`, `by`, `message` strings; optional `command`, `revision` strings; `for` ordered `CapabilityReference` nodes; `value`, `limit` ordered `Measure` nodes; `input` ordered `Input` nodes; `failed` ordered `EvidenceFailure` nodes |
| `Measure` | `name`, `decimal` strings |
| `Input` | `kind`, `name`, `digest` strings |
| `EvidenceFailure` | `reason` string; optional `exitCode` signed integer or `message` string; binding failures carry `outcome` string distinguishing failed execution from unresolved planning; `measure` ordered `MeasureReference` nodes for over-limit failures |
| `MeasureReference` | `name` string |
| `EvidenceClaim` | `evidence`, `capability`, `path`, `statement` strings, `line` positive integer; optional `section` string |
| `EvidenceSupport` | `capability`, `evidence`, `path` strings, `listed` boolean; optional `verdict` string; `claim` ordered `ClaimReference` nodes |
| `ClaimReference` | `line` positive integer |
| `AssessmentContext` | `subject`, `record`, `recordPath`, `document`, `section` strings; `symbol` ordered `ContextSymbol` nodes; `evidence` ordered `ContextEvidence` nodes; `input` ordered `CurrentInput` nodes |
| `ContextSymbol` | `path` string, `named` boolean; optional `unresolved` string; `definition` ordered `Definition` nodes |
| `Definition` | `file`, `text` strings of an actual resolved definition |
| `ContextEvidence` | `id`, `path` strings; optional `verdict` string; `value`, `limit` ordered `Measure` nodes |
| `CurrentInput` | `kind`, `name` strings; exactly one of `digest` or `unresolved` string |
| `AssessmentRecorded` | `path`, `id`, `subject`, `verdict`, `by` strings, `replaced` boolean; `input` ordered `Input` nodes |
| `Assessment` | `id`, `path`, `subject`, `verdict`, `by` strings, `current` boolean; `moved` ordered `MovedInput` nodes |
| `MovedInput` | `kind`, `name`, `change`, `message` strings; `unresolved` string only when `change` is unresolved, retaining the underlying reason separately from the display message |
| `History` | `id`, `path` strings; `entry` ordered `HistoryEntry` nodes; `removed` ordered `RemovedBinding` nodes |
| `HistoryEntry` | `commit`, `date`, `subject` strings; `source` ordered `HistorySource` nodes; `path` ordered `FileReference` nodes |
| `HistorySource` | `kind` string; optional `binding` string for a symbol source |
| `FileReference` | `path` string |
| `RemovedBinding` | `binding`, `identifier`, `removal` strings; `directory` ordered `DirectoryReference` nodes; optional `commit`, `date`, `subject`, `lastResolved` strings for a found removal |
| `DirectoryReference` | `path` string |
| `ClaimCoverageReport` | `claim` ordered `ClaimCoverage` nodes |
| `ClaimCoverage` | `capability`, `section`, `claim` strings, `line` positive integer; `test` ordered `TestReference` nodes |
| `TestReference` | `path` string |
| `LinesReport` | `package` ordered `CrateLines` nodes; `module` ordered `ModuleLines` nodes; `orphan` ordered `UnitLines` nodes; `excepted` ordered `ExceptedLines` nodes; `error` ordered `LinesError` nodes |
| `CrateLines` | `package`, `ceiling` strings; `counted`, `covered`, `excepted`, `orphan`, `shippingCounted`, `testCounted`, `shippingOrphan`, `testOrphan`, `debt` integers; `ceilingRow` ordered `Ceiling` nodes |
| `Ceiling` | `of` string; exactly one of `lines` or `percent` exact decimal string |
| `ModuleLines` | `package`, `module` strings, `counted`, `orphan` integers |
| `UnitLines` | `package`, `module`, `item`, `unit`, `file` strings; `range` ordered `LineRange` nodes |
| `LineRange` | `first`, `last` inclusive positive line integers |
| `ExceptedLines` | All `UnitLines` properties, then `exception`, `reason` strings |
| `LinesError` | `message` string; a report with errors contains no partial package or line rows |
| `DebtWritten` | `path`, `change` strings, `findings`, `lines` integers |

Named verification invocation rows exist only for the executor's returned runs;
their program kind, arguments and process result do not invent an execution id
or timestamp. A planning refusal has no run. Stored evidence inputs retain their
actual digests; an assessment's unresolved input retains its refusal instead of
substituting a digest. Human rendering and command exits are stated by the
[command line](./workplan-cli.md).

Finding `kind` must be a member of `Finding::KINDS`. Gap `kind` is `public item`,
`unresolved re-export` or `test`. Surface `failure` is `budget`, `carry-budget`,
`unresolved` or `other`. The finding and item-coverage adapters use command values
`check` and `coverage`; typed command results preserve their actual command names. These enums
are domain text, not extensible ad-hoc application IRIs.

## Usage

Select the domain inventory for the report's root class. Classes, properties and
enumerated object IRIs share the versioned report namespace; source symbols,
messages and other literal values remain data. Validate consumer extensions with
their registered owner rather than treating their IRIs as built-in declarations.

Walk a native subject's outgoing objects with `Node::objects`. For a quoted
object, use `Value::quoted` to inspect its subject, predicate and object while
retaining their original native identities.

Run `raf_rdf::example_crate::report` to serialize the same typed ground graph in either supported canonical format.

Build native `TermValue` values and insert `Triple` assertions into `Builder`,
which validates their profile roles. Call `finish` to obtain the immutable
`Graph`, then `to_text(Format::Turtle)` or `to_text(Format::NTriples)`.
`from_text` reads only that exact canonical profile. `snapshot` returns the shared
content-addressed BASE. `dataset` borrows the authoritative native store;
`get`, `nodes`, `Node::values` and `Value` support borrowed domain validation.
`raf_workplan::report` projects actual typed findings and
item/test coverage. The bound `report` example builds typed values and provenance.

## Examples

A registered clean check emits a RegisteredCheck root with status clean and
findingCount zero. A report with findings adds owned CensusFinding nodes with
contiguous ordinals and messages; an evaluation error instead carries its cause.
A written debt response has both counts, while a failed publication has its
error message and no counts. These complete outcomes expose every protocol term.

The Census index example classifies a documented home, a configured variant and
an unsanctioned copy. Body and shim groups, a hexadecimal table, constants shared
by two packages, counted debt members and job debt expose the core inventory
without inventing another report representation.

The native statement-layer fixture exposes one reifier triple term and its
annotation value through `Node::objects`, alongside exact property queries.
Duplicate physical assertions still refuse before that graph is accepted.
<!-- raf:tested-by raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement -->

`cargo run -p raf-rdf --example report` emits canonical Turtle. Passing `nt` to
that example selects N-Triples over the identical graph, including a custom typed
literal, control and Unicode text, an integer, a boolean and a named reifier.
Census's baseline and index expose `to_turtle`, `from_turtle`, `to_ntriples` and
`from_ntriples` under the same ground profile and their own report shape.

## Constraints

- Physical native statement and term-ID order is not canonical report order.
  Canonical output selects exact normalized subject/predicate keys and native
  object ordering across the complete ground assertion meaning.
  <!-- raf:tested-by raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot,raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement -->

- This is a closed canonical profile, not a general RDF parser. It accepts no
  arbitrary Turtle abbreviations, prefix declarations, blank nodes or quads.
  <!-- raf:tested-by raf_rdf::test_crate::profile::abbreviations_blank_nodes_and_quads_are_outside_the_closed_profile -->
- Accepted triple-term nesting has depth one.
  <!-- raf:tested-by raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals -->
- Typed domain consumers must validate closed shapes, ranges and cross-field
  invariants; a caller-built generic graph has only ground-profile validation.
  <!-- raf:tested-by raf_rdf::test_crate::profile::ground_validation_does_not_apply_domain_shapes_or_datatype_value_spaces -->
- The public read/write API takes no resource-budget argument: it admits a
  64 KiB literal through native parsing and ground validation. Parsing precedes
  exact-profile rejection. Native parsing and resident output impose no RAF
  allocation or stack ceiling; consumers must admit their input before calling
  this layer. The semantic triple-term depth restriction is not a parser budget.
  <!-- raf:tested-by raf_rdf::test_crate::profile::caller_supplied_large_literals_are_validated_without_a_profile_byte_budget -->

## Explanation

### Native parsing and allocation boundary

PurRDF parses the input before closed profile validation; the profile does not
impose a parser allocation or stack budget. Native indexes and canonical sorting
also contribute to reader cost. Rendering retains normalized text for its hash
and uses native traversal with resident allocation, which does not impose an
allocation ceiling. This layer does not load files or admit a filesystem load
budget; consumers enforce source/report input admission limits before supplying
text. The accepted depth of ground graphs is a semantic profile restriction,
not a bound on the parser or its temporary allocation.

Census and workplan need the same lexical, ownership and identity laws without a
dependency from one domain to the other. A small owned graph permits complete
validation and deterministic rendering while domain-specific indices retain
their own invariants. A fixed normalized Turtle spelling gives both serializers
one concrete preimage, avoiding circular hashes or format-dependent identities.
The design revisits depth one only when a concrete consumer needs nested
provenance, and revisits the ordered representation only with counted evidence
that its storage or comparison cost exceeds admitted workloads.

## Verification

The existing Census report and registered-response cases compare actual native
emissions with both documented inventories using native ordered tables and term
visitation. The core cases include all match roles, both group kinds, shared
constants, table references and counted debt; the protocol cases include clean,
findings, evaluation error, written and publication error outcomes. Removing an
emitted term from the document fixture and adding a never-emitted declaration
each refuse beside the complete matching inventories. No report wire format or
normal-form identity changes for this vocabulary check.

The bound profile tests cover both formats, canonical bytes, term roles, Unicode
and control escapes, IRI components, language-datatype refusals, duplicate and
dangling references, exact ownership boundaries and valid refusal neighbors.
Workplan integration tests project actual check and coverage results and verify
typed fields, exclusions, surface errors, result ordinals and named provenance.
RDF 1.2 expansions from a separately pinned revision of the PurRDF parser family
pin the emitted absolute graph; census
retains its separate semantic and normal-form golden checks.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::bin_crate::cargo_workplan::result_graph::tests::actual_reach_budget_refusal_projects_a_lines_error_without_partial_rows` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::assess::moved_inputs_preserve_raw_unresolved_reasons_only_for_unresolved_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::recorded_binding_failures_keep_failed_and_unresolved_outcomes_distinct` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms` | test | [Contract](#contract) |
| `raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors` | test | [Contract](#contract) |
| `raf_census::test_crate::reports::canonical_reports_preserve_golden_output_documentation_and_roundtrip` | test | [Contract](#contract) |
| `raf_census::test_crate::reports::native_scalar_readers_preserve_canonical_report_refusals_in_both_formats` | test | [Contract](#contract) |
| `raf_rdf::example_crate::report` | example target | [Usage](#usage) |
| `raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::abbreviations_blank_nodes_and_quads_are_outside_the_closed_profile` | test | [Constraints](#constraints) |
| `raf_rdf::test_crate::profile::absolute_iri_and_literal_profile_refusals_keep_valid_neighbors` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::absolute_iris_use_component_validation_without_normalizing_identity` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::an_external_iri_sharing_a_snapshot_prefix_is_not_local` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_rdf::test_crate::profile::caller_supplied_large_literals_are_validated_without_a_profile_byte_budget` | test | [Constraints](#constraints) |
| `raf_rdf::test_crate::profile::duplicate_native_membership_and_real_subject_queries_have_valid_neighbors` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::exact_bytes_include_cr_final_lf_and_multibyte_diagnostic_neighbors` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::ground_validation_does_not_apply_domain_shapes_or_datatype_value_spaces` | test | [Constraints](#constraints) |
| `raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_rdf::test_crate::profile::malformed_escapes_and_predicates_refuse_beside_canonical_neighbors` | test | [Verification](#verification) |
| `raf_rdf::test_crate::profile::parsed_statement_duplicates_and_repeated_source_bytes_refuse_with_valid_neighbors` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::pinned_registry_expansions_remain_distinct_from_native_canonical_bytes` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs` | test | [Contract](#contract) |
| `raf_rdf::test_crate::profile::roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs` | test | [Contract](#contract) |
| `raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry` | test | [Contract](#contract) |
| `raf_workplan::coverage::report_projection_tests::registry_collection_visits_quoted_atoms_and_both_datatype_positions` | test | [Verification](#verification) |
| `raf_workplan::coverage::report_projection_tests::surface_failure_projection_preserves_optional_paths_and_direction` | test | [Contract](#contract) |
| `raf_workplan::test_crate::rdf_reports::findings_preserve_all_location_variants_and_refuse_unknown_kinds` | test | [Contract](#contract) |
| `raf_workplan::test_crate::rdf_reports::real_coverage_keeps_packages_gaps_exclusions_and_named_source_provenance` | test | [Contract](#contract) |
| `raf_workplan::test_crate::rdf_reports::unreadable_public_surface_retains_its_structured_failure` | test | [Contract](#contract) |
| `raf_workplan::test_crate::rdf_reports::unresolved_reexports_keep_declaration_observations_without_claiming_a_definition` | test | [Contract](#contract) |
| `rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them` | test | [Verification](#verification) |
| `rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_first_party_manifests_have_no_json_result_dependency` | test | [Verification](#verification) |
<!-- raf:generated:end -->
