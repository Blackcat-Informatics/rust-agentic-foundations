---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-embedded-clones
title: Embedded structural clones
description: Bounded candidate discovery and lexical correspondence for checked source regions under an independent matching policy.
state: implementing
bindings:
  code:
    - raf_census::embedded
    - raf_census::embedded::run::Run
    - raf_census::embedded::options::Options
    - raf_census::embedded::views
    - raf_census::embedded::error::Error
    - raf_census::embedded::error::InspectionError
    - raf_census::embedded::error::InspectionError::cause
    - raf_census::embedded::publication::write_debt
    - cargo_census::bin_crate::cargo_census::embedded
    - raf_census::embedded::analysis::Analysis
    - raf_census::embedded::collect::function
    - raf_census::embedded::context::for_file
    - raf_census::embedded::coordinates::Coordinates
    - raf_census::embedded::native_spans
    - raf_census::embedded::workspace::collect
    - raf_census::embedded::pairing::find
    - raf_census::embedded::policy::eligible
    - raf_census::embedded::proof::digest
    - raf_census::embedded::tokens::inventory
    - raf_census::embedded::tokens::join
    - raf_census::embedded::regions::admit
    - raf_census::embedded::report
    - raf_census::embedded::debt::Record
    - raf_census::embedded::identity::endpoint
  tests:
    - raf_census::embedded::proof::tests::renamed_bindings_preserve_repeated_uses_and_distinct_shadowing_declarations
    - raf_census::embedded::proof::tests::captures_cannot_be_reclassified_as_internal_or_merge_distinct_variables
    - raf_census::embedded::proof::tests::fixed_names_operators_shorthand_fields_and_control_opcodes_are_retained
    - raf_census::embedded::proof::tests::missing_declaration_and_untyped_control_opcode_refuse_beside_valid_markers
    - raf_census::embedded::proof::tests::exact_shared_work_and_storage_grants_succeed_and_one_unit_short_refuses
    - raf_census::embedded::resource::tests::simultaneous_proof_scratch_shares_the_original_grant_until_deallocation
    - raf_census::embedded::discovery::tests::rightmost_ties_and_persistent_minima_keep_original_positions_without_duplicate_seeds
    - raf_census::embedded::discovery::tests::an_embedded_renamed_sequence_keeps_a_shared_seed_after_unrelated_prefix_atoms
    - raf_census::embedded::discovery::tests::colliding_candidate_hashes_do_not_establish_a_binding_correspondence
    - raf_census::embedded::discovery::tests::discovery_refuses_exhaustion_and_preserves_the_first_input_cause
    - raf_census::embedded::analysis::tests::retained_functions_share_the_original_grant_across_parser_workers
    - raf_census::embedded::analysis::tests::source_preparation_grants_precede_parsing_and_survive_until_each_worker_joins
    - raf_census::embedded::tokens::tests::original_tokens_verify_an_embedded_renamed_region_and_reject_variable_operator_and_spacing_changes
    - raf_census::embedded::tokens::tests::syntax_admission_excludes_opaque_interiors_cross_block_sequences_and_partial_utf8
    - raf_census::embedded::tokens::tests::control_markers_keep_their_original_anchors_beside_adjacent_closure_and_return_regions
    - raf_census::embedded::tokens::tests::missing_original_tokens_and_overlapping_or_partial_binding_spans_never_become_witnesses
    - raf_census::embedded::context::tests::actual_inline_outline_and_literal_path_ancestors_preserve_disabled_prelude_bindings
    - raf_census::embedded::context::tests::shared_source_reaches_require_independent_agreement_instead_of_pooling_disabled_flags
    - raf_census::embedded::context::tests::ancestor_expansion_conditionals_and_custom_preludes_refuse_beside_inert_documentation
    - raf_census::embedded::workspace::tests::actual_workspace_collection_keeps_every_reach_and_verifies_checked_embedded_regions
    - raf_census::embedded::workspace::tests::actual_impl_and_trait_const_generics_remain_fixed_in_method_bodies
    - raf_census::embedded::workspace::tests::empty_body_attributes_and_pruned_original_functions_cannot_become_accepted_owners
    - raf_census::embedded::boundaries_tests::async_function_block_and_closure_transfers_keep_their_own_future_targets
    - raf_census::embedded::boundaries_tests::try_scopes_intercept_question_marks_without_intercepting_return_or_loop_labels
    - raf_census::embedded::boundaries_tests::const_scopes_keep_generic_items_inner_bindings_and_const_function_returns
    - raf_census::embedded::boundaries_tests::constant_capture_and_control_fences_refuse_beside_valid_lexical_neighbours
    - raf_census::embedded::boundaries_tests::public_collection_refusals_escape_untrusted_paths_and_preserve_their_identity
    - cargo_census::test_crate::delivery::embedded_report_uses_static_source_and_requires_existing_exact_debt
    - cargo_census::test_crate::delivery::embedded_debt_publication_never_creates_a_declared_missing_rust_target
    - cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records
    - raf_census::embedded::pairing::tests::an_embedded_helper_survives_unrelated_locals_and_a_conflicting_surrounding_binding_map
    - raf_census::embedded::pairing::tests::repetition_counts_distinct_disjoint_occurrences_while_short_input_and_boilerplate_stay_empty
    - raf_census::embedded::pairing::tests::selected_hash_collisions_remain_candidates_without_coarse_or_binding_proof
    - raf_census::embedded::pairing::tests::indexing_repetition_and_failed_candidates_share_exact_original_grants_without_partial_results
    - raf_census::embedded::report::tests::canonical_reports_keep_actual_spans_and_all_owners_under_source_and_collection_reordering
    - raf_census::embedded::report::tests::imported_claims_refuse_unknown_profiles_impossible_ranges_and_invalid_support_beside_valid_evidence
    - raf_census::embedded::report::tests::reminted_claims_cannot_alias_native_paths_exceed_the_source_or_bypass_the_match_floor
    - raf_census::embedded::debt::tests::adoption_preserves_line_shifts_counts_repetition_and_refuses_growth_after_exact_shrink
    - raf_census::embedded::debt::tests::complete_owner_descriptors_separate_traits_generic_self_types_and_physical_aliases
    - raf_census::embedded::debt::tests::checked_external_sources_survive_catalog_reordering_while_unbound_roots_refuse
    - raf_census::embedded::debt::tests::imported_allowances_refuse_unknown_policy_zero_counts_orphans_and_total_overflow
    - raf_census::embedded::debt::tests::descriptors_and_debt_tables_share_exact_grants_and_release_failed_projection_storage
    - raf_census::embedded::collect::tests::initializers_read_the_previous_binding_and_inner_shadowing_ends_with_its_block
    - raf_census::embedded::collect::tests::let_else_divergence_precedes_activation_and_closure_parameters_do_not_replace_captures
    - raf_census::embedded::collect::tests::or_patterns_share_declarations_with_their_guard_and_refuse_different_alternative_names
    - raf_census::embedded::collect::tests::return_targets_and_shadowed_block_labels_keep_their_actual_control_destinations
    - raf_census::embedded::collect::tests::shorthand_fields_keep_fixed_names_and_unadmitted_attributes_or_boundaries_refuse
    - raf_census::embedded::debt::tests::debt_requires_the_selected_run_and_import_restores_exact_comparison
    - raf_census::embedded::names::tests::canonical_equivalence_uses_native_ordering_with_exact_work_and_storage_refusals
    - raf_census::embedded::names::tests::native_unicode_seventeen_decompositions_fit_the_admitted_scalar_work_certificate
    - raf_census::embedded::prelude::tests::actual_default_inherited_and_unsupported_editions_control_the_declared_profile
    - raf_census::embedded::prelude::tests::declared_standard_names_stay_fixed_without_mistaking_type_or_macro_names_for_values
    - raf_census::embedded::prelude::tests::disabled_preludes_have_no_fixed_variants_and_custom_or_malformed_flags_refuse
    - raf_census::embedded::scopes::tests::closed_source_names_separate_constants_constructors_and_type_only_items
    - raf_census::embedded::scopes::tests::imports_conditionals_and_unexpanded_attributes_never_prove_binding_absence
    - raf_census::embedded::scopes::tests::local_activation_and_scope_exit_preserve_shadow_identity_and_raw_names
    - raf_census::embedded::scopes::tests::unmodeled_preludes_refuse_while_explicit_const_parameters_remain_fixed
  examples:
    - raf_census::example_crate::debt
depends: [cap:census-normal-form, cap:source-index, cap:rdf-report-profile, cap:census-debt-ratchet]
---

## Purpose

The fragment core separates coarse candidate discovery from ordered
lexical correspondence. It does not change the Census whole-body normal form.

Census identifies a meaningful helper embedded in a larger function, including
renamed locals and surrounding statements, and explains the exact source
regions whose structure corresponds.

## Contract

The private proof core compares fixed tokens in order and establishes a
bidirectional correspondence between already classified declarations. Repeated
uses and distinct shadowing declarations retain their relationships. Captures
remain distinct from declarations inside a fragment. One declaration table
supplies immutable binding metadata. Fixed external names, operators, shorthand
field names and control opcodes remain part of the comparison. A missing
declaration or untyped control marker refuses.

The private Source-backed collector records delayed local activation,
shadowing, captures, or-pattern identities, shorthand field roles and lexical
control destinations. Names use canonical Unicode equivalence through the
native Unicode 17 decomposition API while retaining original source spelling.
A declared standard-prelude profile uses actual target-edition metadata;
unmodeled prelude, import and expansion context cannot prove binding absence.

File context joins actual indexed module declarations, their exact original
syntax spans and the unique body files reached by the actual target. Inline
ancestry follows lexical modules; outline ancestry follows Source's loaded
reaches. Unconditional literal path attributes are admitted only on module
declarations. Every shared-file reach must independently agree on its edition,
disabled-prelude and crate `no_std` profile. Parent values are not inherited;
unknown ancestor expansion remains a refusal even with the standard prelude
disabled. Conditional module context and non-root `no_std` placement refuse.

An owning analysis retains function facts and lexical output together with the
original resource account. Each operation consumes that owner; failure discards
the complete analysis. The owner stays on the caller while the scoped parser
worker borrows it. Source explicitly joins that worker before returning a
native failure or visitor result, or resuming a panic, so the account outlives
worker cleanup and thread-local teardown. Facts borrow externally owned source,
and parser-owned syntax and spans never escape their callback. This custody
does not establish complete native allocation or work admission.

Both Source inspection paths first prepay the root native source-text copy and
line-offset vector. For the inspected native compiler profiles, original byte
length `b` and `L` equal to one plus the
newline count, requested payload is bounded by
`b + (3 * L + 4) * size_of::<usize>()`. Before vector growth, old capacity is
less than `L` and new capacity is at most `max(2 * L, 4)`, covering old/new
overlap. Initial line entries and cumulative
relocation fit `3 * L + 4` elements. The preceding byte scan prepays `6 * b`
work units for three predicates and three counter updates per byte: newlines,
minus bytes and ASCII digit bytes. Native logical UTF-8 extent,
source copying, per-character count and newline predicates, and line writes or
relocation prepay a further `4 * b + 3 * L + 4` work units. Padding preserves
byte length and newline count; native registration precedes BOM stripping.
The root text and line storage remain funded until worker and TLS teardown.
The registry ceilings below require ownership of an initially fresh native
registry. Current standard thread creation does not establish that invariant:
configured host spawn hooks can run child callbacks before RAF's worker body
and initialize the parser's thread-local state. Those callbacks originate in
host code, not inspected source, but their prior registrations and retained
capacity invalidate an unconditional dummy-plus-root premise. Worker-entry
ownership remains required for complete resource admission.

The pinned parser exposes no read-only registry ownership or occupancy query.
Its invalidation API destroys foreign registrations, invalidates existing spans
and retains vector capacity. Registering a sentinel first would already mutate
possibly foreign state. Neither operation establishes the required ownership
boundary. Both named toolchains expose hook suppression only through an unstable
API, while RAF uses stable language features and libraries. The current
implementation has not established or enforced the required entry invariant;
these component formulas do not discharge that remaining requirement.

Under that required entry invariant, the inspection registry includes the dummy,
root and numeric parser-created registrations. Here `F` bounds the pinned private
`FileInfo` from its actual field types:
`size_of::<String>() + size_of::<Vec<usize>>() +
size_of::<BTreeMap<usize, usize>>() + 2 * size_of::<u32>() + 6 * (A_record - 1)`, where
`A_record` is the maximum field alignment. Two padding boundaries cover the inner Span
and four cover the outer record for the inspected compiler profiles. No private
layout surrogate is constructed.

Let `a` and `M` count original minus and ASCII digit bytes. Additional
registrations number at most `A = 6 * a + 4 * M`. Successful negative literal
reconstruction can register once per original general-literal visit, at most
six per original minus. Tuple-index splitting has at most `J <= 2 * M` parts;
each part registers its input stream and at most one reconstructed negative
Literal. Failed Index entries retain these registrations too. Other public
string-parsing and literal-construction APIs are outside this file-grammar
inspection route.

The additional registered source-text lengths total at most
`T = 8 * (b + a)`, or zero when `A` is zero. Only numeric spellings enter
successful reconstruction. Their disjoint original spelling lengths sum to
`S_num <= b`; documentation-generated string spellings do not enter this
numeric projection. Original and generated negative helper inputs fit
`7 * (S_num + a)`, and the outer Index stream inputs add at most
`S_num + min(a, M)`. Unicode numeric suffixes contain no newline. Every extra
registration therefore needs exactly one line offset.

For `Q = A + 2` records, requested registry backing, additional text and the
dummy and additional line vectors reserve
`(3 * Q + 4) * F + T + (A + 1) * size_of::<usize>()` bytes. The registry growth
allowance includes old/new overlap. Logical record construction, insertion and
geometric relocation prepay `4 * Q * F` work units. The dummy-line write,
additional UTF-8 extent, text copying, character count/predicates and line
writes prepay another `1 + 4 * T + A`. Root text and root line offsets retain
their separate allowance above. Both Source paths keep the combined
reservation through worker and TLS teardown.

The checked prospective position bound `1 + b + T + A` must fit `u32::MAX`,
including each registration's separating position. An exhausted bound refuses
as the named `native span positions` count before Source starts; reduce the
input to fit this fixed native domain. The inspection worker and its internal
callbacks request no native byte-range or source-text lookup. Their lazy
byte-offset maps remain empty. Other Source operations have independent
contracts and may populate such maps. TLS machinery and other native
obligations remain separate.

The same reservation prepays original native lexer token-vector backing at
`10 * b * size_of::<TokenTree>()` requested bytes, using the public token type.
Source-credited nodes include nodes under construction before insertion. Their
count is at most `2 * b`: ordinary nodes have disjoint source anchors, written
groups have delimiter anchors, and documentation comments produce at most six
nodes from at least three bytes. This also bounds malformed source prefixes.
Under the inspected native Vec growth rules, all live builder and completed
stream capacities total at most four times these credits. The documentation
builder's exact three-slot allocation is credited before its first insertion.
One active growth can additionally retain old capacity no greater than the
credited node count. Building a stream moves the vector without copying its
backing. The reservation remains live through the worker join.

Original completed streams also reserve `(b + 1) * B` bytes for their Rc
blocks, including an empty root. Each written or documentation group accounts
for at most one other stream. The inspected Rc representation has C layout,
alignment at least two, two `Cell<usize>` counters and the `Vec<TokenTree>`
value. `B` is their aligned allocation size, composed with checked public
`Layout` operations. No private-layout surrogate or unsafe operation is needed.

Original identifier and literal spelling buffers reserve another `30 * b`
bytes. Ordinary exact copies occupy disjoint source ranges totalling `R`.
For `d` documentation comments with body-byte total `n`, their emitted string
lengths sum to at most `10 * n + 2 * d`. Each initial capacity is its body
length plus two; each later requested capacity is at most twice its final
length or eight, whichever is greater. Character-iterator lower size hints
never reserve beyond remaining output bytes. Including the one overlapping old
allocation, all documentation string capacities therefore fit
`30 * n + 14 * d`. Their generated `doc` identifiers add `3 * d` bytes.
The disjoint ordinary ranges, bodies and at least three prefix bytes per
comment give `R + n + 3 * d <= b`, so the combined spelling bound is `30 * b`.
Rejected literal alternatives and bare-carriage-return documentation refusal
allocate no spelling before acceptance.

Lexer frames and original-stream cleanup stacks have separate reservations
that may coexist on lexical failure. Let `D` be
`min(b, Budget::Delimiters.limit())`; Source checks that depth before starting
the worker. The native frame contains one u32, one public Delimiter and one
token-vector builder. Its two single-field wrappers and two nested tuples
have six field-padding boundaries. The frame-size ceiling `K` adds those
public field sizes and `6 * (A_f - 1)`, where `A_f` is their maximum alignment.
Live old/new frame backing fits `(3 * D + 4) * K` bytes under the inspected
growth rules. Cleanup's single-field iterator wrapper fits
`I = size_of::<std::vec::IntoIter<TokenTree>>() + align_of::<std::vec::IntoIter<TokenTree>>() - 1`.
Documentation lowering can add one group to the written depth, so cleanup
backing fits `(3 * (D + 1) + 4) * I`. Nested drops of emptied streams allocate
no additional stack backing. Later Syn streams and clones, their cleanup, AST,
errors, other thread-local state, allocator execution and native work
remain separate obligations; these payload reservations are not complete
native resource admission.

Syn's original input conversion also reserves its flattened token buffer and
native iteration copies. For `N` complete original token nodes and `G` groups,
the flattened buffer has `E_buffer = N + G + 1 <= 4 * b + 1` entries. Each
group adds one End entry and the root adds one terminal End. Replacing a group
placeholder adds no entry. Growing the entry vector and shrinking it to a
boxed slice reserve at most `(3 * E_buffer + 4) * B_entry` bytes, including
old/new allocation overlap.

The private Entry size is bounded from its actual public variant fields.
Let `P_entry` be the largest of `size_of::<Group>() + size_of::<usize>()`,
`size_of::<Ident>()`, `size_of::<Punct>()`, `size_of::<Literal>()` and
`2 * size_of::<isize>()`; let `A_entry` be their maximum field alignment.
Then `B_entry = P_entry + size_of::<usize>() + 4 * (A_entry - 1)` under the
inspected native compiler profiles. The four padding boundaries cover two
payload fields, the tag prefix and final alignment. No private enum surrogate
is constructed.

This enum ceiling depends on the named compiler layout rules. Rust
[1.98.1](https://github.com/rust-lang/rust/blob/1.98.1/compiler/rustc_abi/src/layout.rs)
and nightly
[`21b707e3f97e0b522ebd2f277a862339625ad83f`](https://github.com/rust-lang/rust/blob/21b707e3f97e0b522ebd2f277a862339625ad83f/compiler/rustc_abi/src/layout.rs)
build prefixed variant layouts and select no larger niche layout. The initial
tag is bounded by the default isize discriminant. Tag widening uses the minimum
first-field alignment; Entry's End variant has two isize fields, bounding that
alignment. Their integer-selection rules require integer size to equal the
selected alignment, as defined in the corresponding
[stable](https://github.com/rust-lang/rust/blob/1.98.1/compiler/rustc_abi/src/lib.rs)
and [nightly](https://github.com/rust-lang/rust/blob/21b707e3f97e0b522ebd2f277a862339625ad83f/compiler/rustc_abi/src/lib.rs)
layout primitives. On native x86_64 GNU the tag therefore fits usize, aggregate
base alignment is one byte, and enum alignment is bounded by `A_entry`.

Root conversion visits each original stream once. Shared vectors clone at
their exact lengths; copied native token arrays total at most
`2 * b * size_of::<TokenTree>()` bytes. Identifier and String clones copy exact
spelling lengths, bounded by the original lexer spelling allowance of
`30 * b`, including expanded documentation strings. Moving those tokens into
Entry values makes no additional spelling clone. Group streams share their
existing Rc allocation. The combined root conversion-copy reservation is
`(2 * size_of::<TokenTree>() + 30) * b` bytes.

Tuple-index parsing can keep one additional buffer live alongside the root.
Index parsing cannot recursively enter another tuple-index parser. Its
numeric-derived input has at most two leaf nodes and no groups, including
failed parses; leaf spelling can belong to an identifier or a literal. When
`M` is nonzero, the reservation adds `13 * B_entry` for entry growth/shrinking,
four native token-vector slots, one native Rc block of size `B`, and `b + 1`
leaf spelling bytes. Conversion owns that stream uniquely and makes no further
token-vector or spelling clone. Numeric registration storage retains its
separate allowance above. These parser-input reservations remain live through
the worker join. AST, errors, other parser storage and native work still need
their separate admission; this is not a complete native parser bound.

The same reservation includes the possible retained shebang and padded
lexical-input buffers from that same account. Five work units prepay at most
five prefix-byte inspections for the necessary-condition check for an optional
BOM and `#!`; this does not duplicate Source's header-trivia grammar. A possible header reserves
`4 * b + 8` requested payload bytes and the same ceiling of logical copy work,
where `b` is the original source byte length. The reservation remains live
through the visitor and is refunded only after the worker joins, including on
native failure or visitor refusal. Panic discards the complete caller-owned
analysis after worker cleanup. Plain source reserves no header payload.

The header copy requests at most `b` bytes. Native formatting produces exactly
`b` padded bytes; each growth has old capacity less than `b` and new capacity
at most `max(2 * b, 8)`, with both allocations live during replacement.
Including the header therefore fits `4 * b + 8`. Nonempty capacities at least
double, so cumulative relocation also fits the final capacity, giving the same
ceiling for header, normalized-output and relocation copies. Formatter control,
allocator execution, stack, lexer, parser, span and TLS obligations remain
separate; this reservation is not complete native resource admission.

The private workspace collector joins each physical function's exact item and
body spans, name and function kind to every compatible actual indexed owner.
Every eligible indexed body is accounted for exactly once. Each retained
function borrows its actual workspace, source file and every indexed owner,
and retains the effective declared edition and prelude flags of its lexical
module. Enclosing impl or trait generic frames do not hide that module policy.
Joined original tokens are mandatory; raw-text ownership exists only in
controlled fixtures. The collector carries enclosing impl and
trait const parameters into method value scope. A receiver has its own original
`self` declaration and retains that capture identity in the body. Owner and
generic-parameter attributes are validated even for empty bodies.

Async functions, closures and blocks have distinct control identities. Their
syntax requires an actual declared edition of 2018, 2021 or 2024. Await targets
the nearest async context and cannot cross an ordinary function, closure or
const boundary. Return targets the nearest function, closure or async block;
try blocks intercept `?` but not return or an enclosing loop destination.
Loop transfers cannot escape a function, closure, async or const boundary.

Const functions remain function-like. Inline const blocks and explicit const
expression positions fence older dynamic locals while retaining const
parameters, item names and their own local bindings. These positions include
array lengths, repeat lengths, const generic arguments and range-pattern
endpoints. A const block cannot transfer return or `?` to an outer context.
This is lexical classification, not compiler acceptance or const evaluation.
Language feature declarations admit only inner crate-root lists containing
`try_blocks`; unknown features, malformed lists and other placements refuse.
Function parameter patterns supply declarations, while their signature-owned
const and type syntax remains outside the body analysis. Closures and local
patterns inside the body retain their embedded const and type boundaries.

The private token join checks original native lexical spans, keeps delimiters
and punctuation spacing, and requires each binding annotation to cover one
original identifier. Stable marker ordering preserves traversal ties. Every
control marker retains its owning source anchor. Region admission accepts a
complete expression, body or consecutive statements within one block, excludes
opaque interiors, and checks UTF-8 containment and line ranges.

One flat lexical inventory per file is built under the original work and live
storage grants. Every function join borrows that inventory; it does not consume
a clone of the native file stream. Group envelopes retain exact subtree ends
so opaque source regions can skip a complete original subtree before lexical
validation. Native token and span objects remain inside the parser callback.
Native tree traversal reserves one extra group for documentation-comment
lowering beyond the admitted written delimiter depth.

One inverse coordinate table per parser callback converts native line and scalar
column positions into original byte offsets. Its vector capacities, construction
and endpoint queries spend the same original analysis grants. The table models
Source's retained BOM and byte-padded shebang without copying normalized source.
Requested endpoints must be valid original UTF-8 boundaries; invalid coordinates
refuse rather than clamp or fall back to the native byte-offset cache. Native
span construction, lexer and TLS storage remain incompletely admitted.
Block bounds use the original stored brace span without printing statements
into a temporary token stream to recover their endpoints.
Function and module ownership joins select their original first outer token
and final delimiter or semicolon directly. Outer attributes, visibility and
function modifiers retain their printed order; inner attributes stay inside
their owning delimiter. Attribute selection spends the original work grant,
and endpoint conversion does not introduce another native span-join search.
Diagnostic starts for typed items, generic parameters, local bindings and field
values also select stored tokens under the original work grant.

Expression regions, expression statements and closure or const entry anchors
use original source endpoints. First and last endpoint walks charge every
visited expression and inspected attribute to the same original work grant.
They follow stored tokens and child links without recursive calls or retained
buffers. Source extents include an attributed range's operands and punctuation;
synthetic groups produced by a syntax printer cannot shorten that extent to
its attribute alone. A statement includes its actual semicolon when present.
Closure and const entries reuse the checked original extent.

Typed type endpoints, including cast and path output tails, use stored tokens
and an iterative traversal of referenced types, paths, returns and bounds.
Each transition and inspected bound pair spends the original work grant.
Borrowed pair traversal and direct indexed lookup avoid the native boxed
value iterator. Collection uses the same borrowed traversal for native syntax
lists, retaining reverse traversal when stacking grouped imports.
Type attributes retain their separate validation; they do not
contribute to the native type printer's endpoints. Generic-argument diagnostics
select their first stored token through these same endpoint helpers.

Pattern extents and match-arm diagnostic starts use the same original source
fields and metered traversal. Range and guard patterns borrow expression
endpoints; type ascriptions borrow type endpoints without cloning alias nodes.
The binding model retains its existing refusals for unsupported patterns.
Local, item and macro statement extents end at their stored semicolon or real
delimiter. Tuple-struct extents include the trailing where clause and semicolon.
Opaque item and macro endpoints are cached before recording their regions.

Empty opaque type, bound or pattern components emit no endpoint. The preceding
qualifier, reference prefix, return arrow, path component, pattern punctuation
or bound separator remains the tail. Qualified paths retain their closing
punctuation and inserted angle delimiter in the native printing order.
Opaque type, bound, expression, pattern and item endpoints share a shallow
token-copy operation. The same source-coordinate context supplies its immutable
source length and original account. Before cloning, it admits clone, copy,
iteration and cleanup work and reserves temporary requested storage of at most
`(2 * size_of::<TokenTree>() + 50) * source_bytes`. These bounds apply to the
pinned raw-source lexer and monotone original Verbatim range producers, not
arbitrary constructed token streams or generated spellings.

Endpoint work uses source-level events. For at most `T = 2 * source_bytes`
shallow tokens and `D = 50 * source_bytes` copied spelling bytes, the allowance
is `10 * T + D + 18`, in addition to the initial empty-stream check. The
per-token events are clone dispatch, at most one spelling allocation, iterator
advance, stored-span access, token destruction, at most one spelling
deallocation, checked count increment, count-bound comparison, first-span test
and last-span assignment. Group reference updates belong to clone/drop dispatch;
spelling copies contribute the separate byte events. The fixed
18 counts eight native events (root reference clone, sharing decision, vector
allocation, cloned reference release, iterator construction, terminal advance,
iterator teardown and vector deallocation), seven reader events (three
initializations, first assignment and three final result operations), and
three grant transactions (creation, reservation and refund). Admission-envelope
arithmetic and control plumbing, scalar literal construction, allocator execution
and machine instructions are outside these event units.

The original stream stays borrowed until the cloned iterator and all copied
tokens have been destroyed. Nested groups therefore remain shared through
cleanup. The reservation is refunded only after that cleanup; only first and
last stored spans escape. No syntax printing or native span join is required.
Exhausted work or storage refuses before the native copy starts. Empty streams
retain their existing complete-root or child empty-emission behavior.

This reservation covers the additional endpoint copy. Original Verbatim
storage and construction, other parser and lexer storage, TLS ownership, native
parsing work, errors and cleanup still require separate admission. Endpoint
extraction does not establish a complete native resource bound.
<!-- raf:tested-by raf_census::embedded::workspace::tests::actual_workspace_collection_keeps_every_reach_and_verifies_checked_embedded_regions -->

Native parser dependency requirements pin Syn 3.0.8, proc-macro2 1.0.107 and
quote 1.0.47, including when RAF is consumed without its repository lockfile.
Resource audits apply to those implementations. Exact versions do not fix
features unified by other consumers, compiler configuration or target layout,
and do not establish complete native resource admission.

Candidate discovery uses full BLAKE3 digests of eight-atom grams and windows of
four consecutive grams, selecting the rightmost minimum. UTF-8 text has an
explicit little-endian u32 byte length, and token-role tags distinguish fixed
tokens, declarations and uses. Each atom also encodes its native punctuation
spacing bit. The encoding domain is
`raf:embedded:coarse:v1`, followed by the length-framed identifier profile
`rust-identifier-canonical-equivalence/unicode-17`. Discovery erases binding numbers and variable spellings;
it retains fixed spellings and occurrence roles. Neither a seed nor its digest
establishes a finding.

Pairing counts every unordered distinct matching-hash seed pair before any
coarse, overlap or proof rejection. Exact coarse comparison precedes expansion
to a maximal identical coarse alignment. Every complete expression, body and
consecutive same-block statement sequence inside that alignment is considered
independently. The admitted left atom range maps to the right; original lexical
tokens supply its byte endpoints, and right-side admission must reproduce that
exact atom range. A conflicting surrounding binding map cannot poison the
fresh ordered proof of a smaller region.

The matching policy is `embedded-lexical-v1/k8-w4/lex24-info4-fixed2`.
An eligible region has at least twenty-four original lexical tokens, excluding
control markers, at least four distinct informative features and at least two
distinct informative fixed spellings. Proven binding identities and fixed
spellings are distinct feature domains. Only ordinary value declarations and
captures contribute binding features; receiver, label and control identities
remain in proof without inflating diversity. Repeated uses of one value binding
count once. An annotated value token contributes its binding identity; its fixed
shorthand-field role remains proof metadata. Fixed features exclude punctuation
except `+`, `-`, `*`, `/`, `%`, `^`,
`&`, `|`, `!`, `<` and `>`, and exclude the words `let`, `mut`, `ref`, `return`,
`break`, `continue`, `move`, `self`, `Self`, `as`, `true`, `false`, `fn`, `pub`
and `unsafe`. Marker counts and punctuation spacing cannot inflate diversity.
These are eligibility heuristics, not proof of useful abstraction or universal
boilerplate exclusion. Every literal spelling, fixed name, operator and spacing
distinction remains in proof; there is no enclosing-body literal tier.

Witness endpoints are distinct, nonoverlapping physical source regions.
Different target reaches of one source function are not separate occurrences.
Repeated regions retain distinct occurrence pairs. A verified pair is suppressed
only by a policy-eligible verified pair containing both endpoints under the
same atom-offset correspondence. Crossing alignments remain distinct.

Verified identities use the separate `raf:embedded:verified:v1` domain, binding
the matching and identifier profiles. Binding IDs become first-occurrence ranks;
the encoding retains declaration family, event, internal or captured status,
fixed roles, native spacing and complete fixed token spelling. It never reuses
a candidate hash as a witness identity or changes body/shim fingerprints.

The private RDF evidence uses the independent namespace
`https://blackcatinformatics.ca/raf/census/embedded/1#` and report form one.
The existing whole-unit report form is unchanged. Native shared RDF storage,
canonical Turtle 1.2 and N-Triples output supply one serialization path.
The root binds the matching, identifier and declared-prelude model profiles.
Source records retain the load-local authority, exact relative path, full
original file digest, byte length and line bound. Function records retain body
bounds, atom count and effective declared edition and prelude flags. Every
indexed owner retains its actual package manifest and target identity, symbol
and item bounds. Target vector positions do not identify owners.

Candidate records contain original seed positions and hashes, with optional
complete coarse alignments. Witness records retain exact region byte, line and
atom ranges and the independent verified digest. Endpoint orientation uses
source identities. A witness selects the smallest semantic name among actual
supporting candidates, independently of traversal order. Region records may be
shared by different physical occurrence pairs. Operational work and traversal
counters are not part of canonical evidence identity.

Imported reports are separate claims. Closed shapes validate profiles,
cardinality, canonical names, range relationships, candidate support and
reachability. Reading them never creates source-owned matches or establishes
that their source, binding proof or digest is authentic. Load-local authority
numbers distinguish roots within that report's catalog; they do not prove
durable provider identity across catalogs. Saved claims grant no execution,
provider, delegation or debt authority.

Every evidence node has exactly one `rdf:type`. The closed record shapes are:

| Class | Required properties | Repeated or optional properties |
|---|---|---|
| `EmbeddedReport` | `reportForm` integer one; `matchingProfile`, `identifierProfile`, `preludeProfile` strings | Zero or more `function`, `candidate`, `witness` links |
| `Source` | `sourceAuthority`, `sourceBytes`, `sourceLines` integers; `file`, `sourceDigest` strings | None |
| `Function` | `source` link; `startByte`, `endByte`, `atomCount`, `edition` integers; `noImplicitPrelude`, `noStd` booleans | One or more `owner` links |
| `Owner` | `function` link; `manifest`, `package`, `targetRoot`, `targetKind`, `targetName`, `cratePath`, `symbol` strings; `manifestAuthority`, `targetRootAuthority`, `itemStartByte`, `itemEndByte` integers | None |
| `Region` | `function` link; `startByte`, `endByte`, `firstLine`, `lastLine`, `atomStart`, `atomEnd` integers | None |
| `Candidate` | `leftFunction`, `rightFunction` links; `leftAtom`, `rightAtom` integers; `seedDigest` string | Either all of `alignmentLeft`, `alignmentRight`, `alignmentLength` integers, or none |
| `Witness` | `left`, `right` region links; `digest` string; `candidate` link | None |

Integers and booleans have canonical typed spellings. Digests are complete
lowercase BLAKE3 values. Source, manifest and target-root paths retain canonical
Source spelling; redundant empty and dot segments cannot mint aliases. Item
and region bounds fit their source. A witness needs at least twenty-four total
atoms, equal endpoint lengths and disjoint physical regions within its linked
coarse alignment. This necessary atom floor does not establish the lexical or
diversity floors without actual source proof. Unreferenced nodes refuse.

Private debt adoption consumes source-owned verified matches. It counts retained
physical occurrence pairs by the independent verified digest and the sorted
stable descriptors of both endpoints. Each descriptor retains the file path,
effective declared edition and prelude flags, and the complete sorted owner
set. Owners retain actual package and semantic target identity, function kind,
symbol, self type, trait path, native trait key and any indexed trait-definition
target. Different trait implementations and generic self types cannot share an
allowance merely because their displayed method paths agree.

Debt descriptors omit byte and line positions and whole-file digests. Unrelated
line shifts and surrounding locals preserve the allowance. These are exact
counts under the declared descriptors, not persistent identities of individual
physical occurrences. Equivalent replacements under the same descriptors may
consume the same allowance. Reaches through multiple targets remain one physical
endpoint with its full owner set; they do not multiply occurrence counts.

The first-party root is the caller's explicit local scope. Other roots require
an actual checked native source binding. Descriptors retain its configured or
structural observation role, actual package name and version, complete source
kind, repository, reference variant and value, and revision presence and value.
Missing evidence refuses rather than persisting a catalog row or load-local
authority number. This identifies declared observed sources; it does not
authenticate an archive or grant provider authority.

Initial adoption is explicit. Comparison recomputes present pairs from actual
source matches; imported evidence claims cannot supply them. Every new key or
increased positive pair count is growth and refuses. Shrinkage is a distinct
result requiring an explicit record rewrite. Rewriting an existing record first
checks the entire present bag for growth, then retains exactly the present
counts, removing resolved allowances. Containment changes can expose different
fragment digests; allowances never transfer between those keys.

Normative debt has its own namespace
`https://blackcatinformatics.ca/raf/census/embedded-debt/1#` and form one.
`EmbeddedDebt` binds `debtForm` and the matching, identifier and prelude profiles,
with zero or more `pair` links. Each `PairDebt` has one verified `digest`, `left`
and `right` endpoint link, and positive `pairs` count. An `Endpoint` has one
versioned `identity` descriptor. Endpoint naming hashes are checked against
their complete descriptor contents; comparison uses the full descriptors.
Unknown shapes and profiles, duplicate keys, zero or overflowing counts and
orphan records refuse. Both canonical formats use the shared native RDF path.
Nested descriptors validate exact field counts, canonical native percent
encoding, source and target roles, canonical file paths, optional identity
framing, native absolute repository IRIs and stored query/fragment laws.
Unresolved trait candidates are nonempty, sorted and unique; one candidate can
represent distinct native target identities with the same canonical path.
The sixteen MiB serialized limit applies before import and before publication;
it is not a native heap bound. Reading this normative allowance never creates
source proof. Neither adoption nor comparison writes any file.

The complete source collector establishes lexical declaration identities,
activation, captures, shadowing and control destinations before candidate
verification. Syntactically ambiguous pattern names require positive binding
classification or closed absence of a constant or constructor. Missing
resolution alone never proves a new binding. Unsupported required context
refuses with its source location.

Verified regions are complete expressions or contiguous complete statements
within one block. Their source authority, owner, half-open byte range and line
range resolve to the actual loaded source. UTF-8 boundaries and containment
are checked before a witness is exposed. Fixed literal values and structural
operators are retained under an independently versioned fragment policy.

Minimum meaningful size, boilerplate, overlap and multiplicity rules form part
of that policy. Empty or short inputs yield no clone, and repeated discovery
windows never count as independent occurrences. Candidate collisions cannot
produce a verified witness without ordered lexical correspondence.

## Usage

Load an explicit Source `Workspace` with its selected load options, then call
`raf_census::embedded::Run::analyze` with the analysis `Options`. Candidates and
witnesses borrow their actual source files and indexed owners. `report` produces
closed RDF evidence; `import_evidence` reads inert claims and cannot create
witnesses or debt. `adopt_debt`, `import_debt`, `compare_debt` and `shrink_debt`
operate on exact allowances tied to the original run. Serialized output retains
its original grant and exposes borrowed text, without detaching its payload.
These report, import, comparison, shrink and serialization methods accept a
scoped observer. Successful evidence, debt and text retain their original
grants; an `OutputFailure` borrows the unchanged diagnostic only while the
observer runs. Its layer and optional exact resource reason are separate typed
data. `debt_refusal` retains machine-readable account mismatch, arithmetic
overflow and growth fields; growth borrows the original digest and descriptor
hashes and copies the exact recorded and present counts. Work already consumed
by one operation remains spent by the next.

`cargo census embedded report` exports fresh evidence independently of the
whole-body Census gate. It accepts `--format ntriples|turtle` and an explicit
`--out` destination using the existing guarded create-only report writer.
`cargo census embedded check` requires `census-embedded-debt.ttl` in the
workspace root. It refuses growth and any unpublished shrink. Both commands
inspect source as data without executing consumer targets.

On native Linux, `cargo census embedded debt create` explicitly adopts fresh
exact allowances only when the destination is absent. `debt shrink` requires
an existing regular record, refuses all growth, and publishes only exact
shrinkage. `raf_census::embedded::write_debt` exposes the same operation with
explicit Source and analysis options. The writer loads its own Workspace and
Run under the retained-directory lock; supplied claims or a detached Run cannot
authorize publication. Initial publication atomically refuses replacement.
Its observer receives the result before that internal Run is destroyed.
Failures before a Run exists use the outer static error result. The command
line writes scoped diagnostics directly while their original owners live.
Temporary output is exclusive, verified by identity and synchronized before
publication. The directory is synchronized after publication; failures after
the atomic operation retain a distinct typed already-published state.

Explicit static library and command-line operations share the production
collector, candidate index and verifier. Their report distinguishes candidate
evidence from verified source witnesses. A separate matching-policy identity
and closed RDF report shape keep existing body and shim fingerprints intact.

## Examples

A renamed helper inside a larger function retains a verified region despite
an unrelated earlier local. A neighbouring operator or variable-use change
refuses that correspondence. A closure capture, inner shadow, shorthand field
and control destination retain their own lexical roles. Ordinary boilerplate,
repetition and short or empty regions obey the stated meaningfulness rules.

## Constraints

Report, evidence and debt RDF construction, controlled import, closed-profile
and domain validation, queries, descriptor projection and table storage charge
the original analysis account. Import retains its native source account while
admitting the destination graph and validation scratch under the same aggregate
limit. Physical ordinary, reifier and annotation records retain their layers.
Queries use charged borrowed native views without resident query collections
or implicit native index construction. Failed construction and import retain
the original diagnostic and its original owner through scoped observation.
Raw metadata keeps its independent byte ceiling; that ceiling alone is not a
native heap bound.

Canonical rendering admits selection buffers, native term
copies, ordering and digest work, and output growth before construction.
Returned text retains its original grant until destruction; scratch and
output overlap in the same aggregate limit. A rendering refusal exposes no
partial text and preserves the original typed resource cause.
<!-- raf:tested-by raf_census::embedded::debt::tests::descriptors_and_debt_tables_share_exact_grants_and_release_failed_projection_storage -->
Publication requires native Linux directory locking, hard-link creation for
atomic initial adoption, and directory synchronization. Unsupported operations
refuse without another protocol. Older file-lock writers do not coordinate with
the directory lock. Advisory locking provides neither an immutable source
snapshot nor authentication of arbitrary concurrent directory mutation.
Imported text has a sixteen MiB input limit and RDF work and storage admission
under the original account. This does not establish a total native resource
guarantee: Source lexing, parsing and span storage remain incompletely admitted.
Source collection refuses
const closures, yield expressions, unmodeled syntax and simultaneous async and
const function signatures. Declared prelude profiles require
unambiguous actual source ancestry; unevaluated conditional or expanding
ancestor context remains unsupported.

The workspace collector refuses original syntax that disagrees with indexed
source selection. It never restores a configured load's pruned function from
the file's original text or returns a partial set of accepted bodies.
These private components do not establish classification of arbitrary Rust.

Original work and live-storage grants cover retained facts, lexical inventory,
join buffers and proof scratch, seed postings, candidates and witnesses. Explicit count limits
bound all retained atoms, postings, matching-hash seed pairs, discovery
candidates, unique syntax-region proposals and distinct verified pairs before
containment suppression. Every attempted comparison, including a rejected
candidate, spends the original work grant. Exhaustion returns no partial
analysis. Native parser and lexer storage still require a separately
admitted envelope in the complete pipeline. The current counters do not claim
a bound on the complete analysis heap.

The winnowing guarantee concerns identical contiguous coarse sequences of at
least eleven atoms. It does not cover arbitrary padding, reordered statements,
different binding relationships or semantic equivalence. Binding erasure makes
discovery permissive; verification must still reject incompatible relationships.

Indexed tokens, retained postings, candidate pairs, verification work and
resident storage have explicit bounds, including repetitive inputs. Exhaustion
is actionable failure, never truncation, sampling or an ordinary nonmatch for
the stages that charge the original account: discovery, proof, reporting and
adoption. Source lexing, parsing and span storage are not part of that
guarantee. The parser worker is started without a registry-ownership step, so
the worker-entry invariant is not enforced and bounded storage through the
whole pipeline is not delivered. That residual is a separate tracked change.

Similarity never authorizes a preferred provider, validates delegation or
exempts debt. Embedded findings have explicit versioned adoption and exact
shrink-only debt handling. Discovery cannot create an allowance or rewrite an
existing debt record. Existing normal form 1 fingerprints and debt meaning
remain unchanged; any change to those meanings requires explicit migration.

Analysis remains static and process-free. It runs no consumer program, test,
example, compiler or command, and changes no inspected source or Git state.

## Explanation

The fragment proof and discovery components do not use windows from an existing
whole-body value form. Its binding numbers and literal threshold depend on the
enclosing body. A fragment instead needs its own lexical identities and literal
policy. The shared native BLAKE3 and allocation primitives provide hashing and
fallible physical storage; RAF owns the matching policy and original work and
live-storage grants.

Native parser accounting includes speculative traversals. In the pinned
parser, each primitive peek invokes its selected token predicate at most once.
Skipping to the second or third token does not repeat the predicate; lookahead
failure records its display label without retrying. A general literal
predicate runs one general literal parse and allocates one unexpected-token
tracking cell. A typed literal predicate also retains an error-position fork,
so it runs one general literal parse but allocates two such cells, including on
rejected or empty input. Successful optional-literal parsing composes a peek
with a separate actual parse. Numeric classification within each general
literal parse can attempt both integer and float conversion in that order.
Primitive peek counts therefore do not equal retained literal counts.

In the pinned full-feature expression parser, let `E` count precedence-parser
entries from outside its right-hand-side helper, `P` count all right-hand-side
helper calls (including range ends), and `Q` count precedence-loop iterations
that advance the original cursor, including iterations that subsequently fail.
The two precedence fork sites together create at most `E + 2 * P + 3 * Q` forks.
Each precedence invocation has at most one nonadvancing terminal iteration.
Each continuing right-hand-side iteration observes cursor progress; a
nonadvancing recursive result or an error terminates that loop. Distinct
progressing recursive calls own distinct advancing iterations. Consequently,
precedence invocations number at most `E + Q + P`, their loop forks at most
`Q + E + Q + P`, and precedence peeks at most `Q + P`.

For one original raw-source parse, let `N` count the complete native token tree,
including nested groups and documentation lowering, and let `b` be the original
source byte length. Each external precedence entry has a distinct introducing
token for its left-hand side. Each right-hand-side helper call follows a
distinct consumed operator or range, and each advancing precedence iteration
consumes a distinct operator, range or cast token. Thus `E`, `P` and `Q` are
each at most `N`, and the precedence fork bound is `6 * N`.

Other fork families have the following ceilings in the same source scope:

| Fork family | Ceiling | Original-source anchor |
| --- | --- | --- |
| Restricted visibility | `2 * floor(b / 3)` | A written `pub` is parsed by dispatch and its selected production at most once each |
| Generic disambiguation | `2 * floor(b / 3)` | A written `for` reaches earlier-boundary and atomic expression selection at most once each |
| Break label selection | `floor(b / 5)` | The consumed written `break` keyword |
| Statement macro-path probe | `N` | The distinct statement entry token, including an attribute-only invalid tail |
| Attribute name-value probe | `b` | A consumed written `=` or one synthesized for a distinct documentation comment |
| Item dispatch, signature and receiver probes | `3 * N` | Distinct item and argument entry tokens before their outer attributes |
| Typed literal error-position heads | `10 * N + J` | Typed literal predicates and actual typed parses, with `J` nested integer parser entries |

The full-feature speculative paths do not parse and abandon declaration or
expression bodies. Consumed outer attributes are passed onward rather than
reparsed. Documentation comments occupy disjoint original ranges of at least
three bytes and each lower to one name-value anchor. Invisible-group branches
and non-full parsing alternatives are outside this raw-source path.

Each item dispatch creates one discriminator fork and at most one signature
probe. Each argument creates one receiver probe. Item and argument entry tokens
are separately injective into `N`; their anchor sets need not be disjoint.
Bare function-pointer arguments use their separate grammar, without the item
receiver probe.

Primitive literal predicate calls number at most `13 * N + 3`. Typed predicates
contribute at most `5 * N`: ABI parsing contributes `2 * N` for its speculative
and selected-production entries, while post-dot float selection, numeric member
selection and the explicit item-extern string branch contribute at most `N`
each. General literal predicates contribute at most `8 * N + 3`: expression
start checks after break, return or yield; atomic expressions; single patterns;
pattern range bounds; initial generic arguments; associated arguments after
`=`; const arguments; and attribute name-value selection. Each family has a
distinct introducing or already-consumed anchor. Atomic expressions, single
patterns and initial generic arguments each additionally allow one terminal
empty-input failure.

Actual typed literal parses outside predicates number at most `5 * N + J`:
optional ABI strings, post-dot floats, explicit item-extern strings, original
numeric members and nested integer parser entries. Adding typed predicates
gives the literal-head fork ceiling. Tuple-field float decomposition starts
these nested integer parses for its dot-separated parts, so one source-file
parse does not imply one native parser entry.

Together the fork families give a ceiling of
`20 * N + J + 4 * floor(b / 3) + floor(b / 5) + b` executions. Nested native
integer entries satisfy `J <= 2 * M`, where `M` counts original ASCII digit
bytes. Each tuple-field float consumes an original numeric literal once; its
spelling has at most one dot, and decomposition parses at most two parts.
Negative reconstruction adds no dot, and an invalid part terminates the loop.
The nested integer parser never enters expression parsing again.

Fresh unexpected-token tracking cells come from parser entries, forks, literal
predicates and allocating commits. If their respective counts are `R`, `F`,
`L` and `H`, the total is `R + F + L + H`. Here `R <= 1 + J` and `H <= F`.
Original token anchors give a tighter ceiling: each visibility, `for`, `break`
and name-value assignment family consumes an original token. Thus
`F <= 26 * N + J`. Each consumed original float creates at most two numeric
entries, so `J <= 2 * N`. Together with `L <= 13 * N + 3`, the total is at most
`65 * N + 3 * J + 4 <= 71 * N + 4` fresh cells. Total creations also bound
simultaneously live cells without relying on their individual drop times.

The private tracking value has three variants: empty, a public `Span` and
`Delimiter`, or a sized `Rc` chain handle. Let `P` be the larger of the public
span-plus-delimiter sizes and `size_of::<Rc<()>>()`, and let `A` be the maximum
alignment of those fields, `usize` and `isize`. Under the named compiler
enum-layout premise above, `P + size_of::<usize>() + 4 * (A - 1)` bounds the
value, including tag widening and padding. `Cell` is transparent. Checked
layout composition pads this value, appends it after the two public counter
layouts and applies `RcInner`'s minimum alignment of two to obtain block
ceiling `G`. Sized `Rc` handles use the same thin-pointer layout for this
profile; no private Rust-layout surrogate is constructed.

Actual Source inspection first prepays the byte-derived storage, then obtains
the immutable `PreparedSource` and prepays `(71 * native_token_bound + 4) * G`
before consuming it. Both reservations remain live through Source's explicit
worker join and cleanup; preparation refusal refunds the original reservation,
and tracking admission failure drops the owning analysis before any parse.
This funds tracking-cell storage only. Error, AST, remaining native work and
other storage obligations are separate.

Delimiter-child buffers number at most twice the original group count, hence
at most `2 * N`. Ordinary grammar enters a group once. Restricted visibility
can add one entry through dispatch replay or a rejected interpretation followed
by ordinary type parsing, such as a tuple field beginning with `pub (T)`.
A rejected interpretation cannot also select another item parser because its
discriminator remains at the unconsumed parenthesis. Numeric subparses contain
no delimiter groups. Delimiter buffers share an existing tracking cell.

Every reachable commit uses a fresh fork at most once, so total commits `C`
satisfy `H <= C <= F`. When both terminal cells are unset, a commit links the
fork's terminal cell to the receiver's terminal cell, then replaces the fork's
own root with a fresh cell. Ordinary
delimiter children share their stable primary root. The only surviving child
alias to a committed fork's old root belongs to restricted visibility; it drops
before that visibility returns and before an enclosing dispatch can commit.
Other reachable commits retain no such child alias. Failed visibility probes
do not link cells, and recording an unexpected token creates no link. Thus
this raw-source grammar follows at most one chain link per lookup, visiting
at most two cells. Arbitrary public parser callbacks do not share this bound.

Tracking-chain traversal, including buffer teardown, is consequently at most
`2 * (2 * C + F + L + B + 2 * R)` visits, where `B` counts delimiter-child
buffers. Substitution gives `2 * (3 * F + 15 * N + 2 * J + 5)`. This counts
chain visits, including cleanup, rather than allocator execution or all native
instructions. Cell layouts, other parser and lexer storage, other work and
cleanup retain separate obligations. These traversal bounds do not establish
complete native admission.

Lookahead comparison lists have their own bounds. The full file grammar reaches
37 constructor sites, each at most `N + 1` times, so the number of instances
`W` is at most `37 * (N + 1)`. The extra attempt covers terminal empty input.
Each instance invokes at most 23 predicates and appends at most 23 static label
references. Looping productions construct fresh instances; replaced and
shadowed instances have separate constructor sites. Binary-operator probes
and nested numeric parsers create no lookahead list.

On native x86_64 GNU, the Rust 1.98.1 and nightly
`21b707e3f97e0b522ebd2f277a862339625ad83f` standard-library growth rules give
comparison-vector capacities of zero, four, eight, sixteen or thirty-two.
Only one vector can grow at a time on the parser worker, with at most sixteen
old slots overlapping its new allocation. Total requested comparison payload
is therefore at most `size_of::<&str>() * (32 * W + 16)`. Label text remains
static and is not copied into those vectors. Growth relocates at most
`28 * W` references; predicate costs retain their separate bounds.

A lookahead label has at most fifteen bytes. Its expected-token message has
at most `17 + 23 * 15 + 22 * 2 = 406` bytes; the end-of-input prefix raises
the final ceiling to 431. At most three message strings coexist during
formatting, prefixing and error construction, with one additional old growth
buffer. Under the same standard-library profiles, each buffer requests at
most `max(8, 2 * 431) = 862` bytes, giving a 3,448-byte message-payload ceiling.
The first lookahead error propagates out of this full file grammar; recovery
paths cannot catch it and resume another lookahead-producing parse. This
payload is therefore a single-worker peak term, not multiplied by `W`.
Other error records, spans, thread-local state, predicate allocations and
general parser error formatting remain separate. These payload bounds are
not allocator overhead, resident-set or complete parser bounds, and do not
establish growth rules for other standard-library implementations.

General parser errors also have separate construction and payload bounds.
Every constructed error ends at a fork-associated recovery, a literal
predicate, an integer-index span replacement or a parser-root exit. A typed
literal failure may construct both a general error and a replacement; its
head-fork recovery and subsequent sink count them separately. Without error
cloning or combination, the coarse count is `F + L + R + J`. After recovered
sinks, a direct file failure contributes one error; alternatively, the first
failing nested integer parse contributes one error and its span mapper creates
one replacement before aborting the file. Thus the tighter construction count
is `Q <= F + L + 2`.

Each error allocates one `ErrorMessage` record. Its private representation is
two spans inside a span range, that range and a thread identity inside a
thread-bound value, and that value beside a string. For the same named native
compiler profiles, let `A` be the maximum of one and the alignments of the
public `proc_macro2::Span`, `std::thread::ThreadId` and `String` types. A record
requests at most
`2 * size_of::<Span>() + size_of::<ThreadId>() + size_of::<String>() + 6 * (A - 1)`
bytes: each of the three two-field structures adds at most two padding gaps.
The native constructors allocate a single-element vector; the file grammar
neither combines nor clones its errors. The named native standard libraries
construct the one-element `vec!` through a boxed array, convert it to a boxed
slice and use its length as both vector length and capacity, without growth.
The Rust 1.98.1 sources are
[`alloc/src/macros.rs`](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/macros.rs),
[`alloc/src/boxed.rs`](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/boxed.rs)
and [`alloc/src/slice.rs`](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/slice.rs);
the named nightly uses the same allocation route. Vector handles are inline
storage rather than additional heap allocations. Multiplying this record
ceiling by `Q` bounds aggregate requested record payload separately from strings.

Live errors have a smaller source-anchored bound. Let `E_error` count original
assignment `=` punctuation, `as` keywords and dot punctuation. The pinned Syn
crate uses Rust edition 2021. Its failed BinOp `if let` probe remains alive
through the following assignment, range and cast branches while they parse
recursively. A chain such as `a = a = a` therefore retains multiple recovered
errors. Each retaining frame consumes a distinct operator anchor, giving at
most `E_error <= N` such live errors.

Other targeted recovery lifetimes are local. Boolean probes drop failed
temporaries before returning their predicates or entering recursive branch
bodies; `Result::is_ok` borrows its result rather than consuming it. Earlier
successful operands in short-circuit chains contain no error. Statement-path
and receiver probes finish before fallback grammar, and precedence peeks do
not recursively produce errors in their branches. Typed-literal replacement
and Index span mapping can overlap an old and replacement error. Without
cloning or combining errors, at most `E_error + 2` ErrorMessage records are
live. Source error conversion and arbitrary public parser callbacks retain
their separate obligations.

All reachable error messages have at most 431 bytes. Aside from lookahead,
formatting inputs are fixed token spellings, cast kinds, rejected identifier
keywords, keyword lifetimes or fixed integer-conversion errors. Arbitrarily
long identifiers are accepted by classification rather than interpolated into
these errors. Nested integer span replacement copies the message without
adding another end-of-input prefix. Public helpers that format arbitrary paths,
parse string contents or print an expression into an error span are outside
this file-grammar route.

Recovered errors have a smaller seventy-byte ceiling: the largest conservative
case is a 36-byte identifier prefix, an eight-byte rejected keyword, one closing
quote and the twenty-five-byte end-of-input prefix. Their string payload is
at most `4 * max(8, 2 * 70) = 560` requested bytes per construction. The two
possible terminal constructions each retain the 3,448-byte ceiling. A
conservative aggregate string-payload bound is therefore
`560 * (F + L) + 2 * 3448`. Identifier-classification spelling copies, source
error conversion, thread-local state, allocator internals and other parser
storage remain separate; this is not complete native admission.

Each retained recovered message needs only its final string allocation, at
most 140 bytes. Two local construction allowances cover temporary formatting
and terminal replacement. Live message-string payload is consequently bounded
by `140 * E_error + 6896`, separately from the live record payload above.
Actual inspection prepays `(N + 2) * R_message + 140 * N + 6896`, where
`R_message` is the checked record ceiling and `N` is the immutable prepared
native-node bound. This charge joins the tracking reservation before parsing
starts and remains funded through the explicit worker join.

Source's conversion to its own error string occurs inside that worker. The
native Syn display writes its first message directly; the fallback lexical
error writes a fixed message. The named standard libraries' generic
`ToString` implementation starts with an empty string and calls that display
directly, as in
[`alloc/src/string.rs`](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/string.rs).
The single write has at most 431 bytes, so its buffer fits the conservative
862-byte native growth ceiling. Preparation reserves these bytes alongside
the tracking and Syn error envelopes, including their overlap. Source's path
clone shares its existing `Arc<str>` and adds no path-text allocation; this
route sets no boxed origin.

On native inspection failure, `InspectionError` retains the Source cause and
the original account. Before Source can fail, preparation fallibly allocates
one mandatory terminal-account slot through the original native admission API.
Its exact public layout size is charged once, and four times that size prepays
the terminal record's logical reads, construction and final write. The prepared
owner drops this slot before Analysis on unwind. On success or visitor refusal,
the slot is deallocated before its exact payment is refunded.

Joined transient envelopes remain conservatively
reserved. Retained analysis buffers are destroyed while their original owner
still holds the account, then the account is consumed into immutable terminal
state and safely written into the existing slot without allocation. All work,
live, peak, limits and captured-refusal values are preserved;
the terminal account offers no clone, reset or admission operation. The cause
precedes the terminal account in field drop order, so its converted message
remains funded until destruction. The terminal representation is naturally
sendable and shareable without a lock. Guard preparation failures have no
additional diagnostic string but use the same owned inspection transition.
Successful parsing and visitor refusals still refund after worker join.

Public `Error::Inspection` carries this owned failure and exposes its native
cause without changing display or the native source chain. `Error::Source`
represents workspace loading before an analysis account exists. Neither path
uses an optional account or an unowned inspection fallback. Conversion and
span lookup have a separately prepaid input-dependent work component. A Syn
error performs two registry searches, one for its span join and one for Source's
start position, and one line-offset search. A lexical error needs one registry
and one line search. No character-to-byte cache is populated. Every selected
line vector has at most `L = 1 + LF` entries; generated numeric and dummy
registrations have one line, and registry size is at most `Q = A + 2`.

The named standard libraries' search kernel shrinks the remaining size by its
floor half on each iteration and makes one final comparator call. For nonempty
size `n`, calls are `B(n) = ceil(log2(n)) + 1 <= n`. Including record visits,
callback relations, loop iterations and explicit search-control comparisons,
one registry search fits `8 * B(Q) + 2` events and a line search fits
`5 * B(L) + 2`. Four join/start span-bound comparisons and at most 862 message
byte reads/writes give a combined ceiling of `16 * Q + 5 * L + 872` events.
The existing registry calculation prepays the first term; the original line
count prepays the rest, without another source scan. The MSRV kernel is stated
in [`core/src/slice/mod.rs`](https://github.com/rust-lang/rust/blob/1.98.1/library/core/src/slice/mod.rs);
the inspected nightly uses the same iteration and comparator ceilings.

Current-thread and identity operations, Arc bookkeeping, remaining inline
scalar operations and implicit bounds checks, worker names, spawn-error storage,
thread initialization, TLS, AST, allocator internals and remaining native
work/storage remain separate. This is not total conversion work or complete
native admission.

Worker-name backing has a separate source-derived bound. For
`p = max(1, path.len())`, RelPath display writes the path once, or a dot for
the root. String-to-vector conversion moves its buffer. CString conversion
scans for NUL, reserves one terminator and shrinks to a boxed slice, with at
most one old/replacement overlap. For both named profiles, requested backing
fits `max(8, p) + p + 1 <= 2 * p + 8`; `4 * p + 8` is a coarser ceiling.
This does not include the thread's control structures or allocator internals.

Its cleanup premise remains separate: standard-library spawn hooks receive a
thread handle before spawning and may retain a clone of its Arc. Such a handle
can retain the name after join or spawn refusal. The actual inspection visitor
does not retain a thread handle, but join alone does not establish final name
deallocation in a host that installs an escaping spawn hook. No unconditional
worker-name refund is implemented from this bound. Runtime control structures,
scope, stack, TLS and native spawn failures retain their own obligations.

General literal parsing includes both predicates and actual parses. Typed
literal entries contribute at most `10 * N + J`, general literal predicates at
most `8 * N + 3`, and ordinary consuming literal entries at most `N + 1`.
Attribute speculation can add one actual parse per name-value fork, at most
`b`; its predicate is already counted separately. This partitions all general
literal entries into a ceiling of `19 * N + J + b + 4`. Numeric classification
within each entry attempts integer conversion at most once and float conversion
at most once, including negative reconstruction. Other literal kinds select one
conversion family from their leading bytes. These invocation counts do not
bound conversion payloads, spelling-copy multiplicity or total literal work.

Each original lexical literal participates in at most six general literal
visits. A foreign-module ABI name reaches that ceiling through the speculative
signature, explicit extern classifier and selected ABI parser, each with a
predicate and an actual parse. Function ABIs and attribute-expression fallback
need at most four visits; expression, generic-constant, pattern and integer
member contexts need at most three. Floating member tokens need at most two.
Negative helpers inspect the same original token through these contexts.
Generated integer-index parts retain their separate charge.

Let `S_lit` be the sum of original literal spelling lengths and `a_minus` the
number of original minus punctuation tokens. Let `K_lit` count original
literals, `M_lit` count original numeric literals, and `J_lit = 2 * M_lit`
bound generated index parts. Across original and generated index visits,
the sum of effective helper-input lengths is at most `7 * (S_lit + a_minus)`.
The original visits contribute at most `6 * (S_lit + a_minus)`; attempted
index parts contribute at most `S_lit + min(a_minus, M_lit)`. Splitting and removing
a trailing dot do not increase length. The number of data-conversion visits
is at most `6 * K_lit + J_lit`.

For effective input length `n >= 1`, converter scratch is at most `13 * n + 12`
requested bytes. Integer conversion uses at most
`3 * max(8, 4 * d + 4) + s`, where consumed radix digits `d` and suffix bytes
`s` are each at most `n`. Float scratch is at most `2 * n`; string, byte and
C-string scratch is at most `2 * max(8, 2 * (n + 1)) + s`. These are sequential
conversion peaks, including failed integer trials before float classification.

Let `B_lit` bound the private classified representation box. Under the same
native compiler profiles, it is at most the size of public
`proc_macro2::Literal`, twice the size of `Box<str>`, and four padding gaps,
each below the maximum alignment of those public types. This also bounds
ordinary representation boxes. The cursor's literal clone requests at most
`n` spelling bytes. Display insertion and negative-destination insertion each
need at most `3 * max(8, n)` bytes including old/new growth overlap. A negative
destination can retain excess capacity; its backing cannot be counted as just
its spelling length. Including scratch and the representation box gives a
per-visit ceiling of `20 * n + 60 + B_lit`.

Summing visits gives requested payload at most
`140 * (S_lit + a_minus) + (60 + B_lit) * (6 * K_lit + J_lit)`. The outer tuple-index
operation additionally retains its token clone and displayed spelling while
parsing borrowed parts; these add at most
`2 * (S_lit + min(a_minus, M_lit)) + 8 * M_lit` bytes across calls. The original boxed
float remains covered by its producing visit. Summing the visit ceilings keeps
parent results admitted while nested classification runs. Other native reparse
storage, thread-local state, span operations and AST literal clones remain
separate from this converter allowance. Inspection preparation funds the
source-map registrations described above.

Both production inspection entrypoints use Source's consuming `with_lexed`
ticket before syntax parsing. Measurement counts actual native tokens and
literal spellings; comments, identifiers and string contents do not become
numeric literals. A fresh lexer emits numeric literals with an ASCII digit
first, with a negative sign represented separately as punctuation. Display
writes to a counting formatter without retaining a spelling copy. Checked
overflow retains its typed storage refusal; an unexplained formatting failure
refuses as an invariant violation. Zero literals require no converter payload.

The original parser-input copy reservation also covers measurement's shallow
native stream copies. Recursive group traversal retains the original stream
and bounds depth by the written delimiter limit plus one documentation group.
Every measurement iterator and temporary token is destroyed before parsing
consumes the same ticket, so these phases reuse that reservation sequentially.
Measurement work is admitted before the worker starts. Converter storage and
integer arithmetic work are admitted after measurement and before syntax
parsing. The combined storage receipt is checked before charging and updated
immediately after successful admission; it remains with the original Analysis
through explicit join. Native lexical or syntax failures retain their owned
diagnostic transition, while measurement refusals use the ordinary located
refusal and cleanup path.

Measurement prepays `N * (4 * size_of::<TokenTree>() + 128) + 24 * b + 128`
logical work events. Each shallow native vector is copied once; group copies
share their child streams. Let `I` and `R` be ordinary identifier and literal
spelling bytes, `C` documentation body bytes, and `d` documentation comments.
These disjoint inputs satisfy `I + R + C + 3 * d <= b`. Documentation literal
spellings total at most `10 * C + 2 * d`, and their synthesized `doc`
identifiers add `3 * d`. Thus copied logical spelling lengths total
`D <= I + R + 10 * C + 5 * d <= 10 * b`; copy reads and writes fit `20 * b`
of the byte allowance. Cloning String and boxed string spellings copies length,
not spare capacity. The counting formatter inspects only chunk length and its
first byte; it does not scan or retain displayed bodies.

Metadata events count the selected data or resource operations below. Pure
forwarding calls do not add units; repeated element visits, backend selections
and reference-count checks are charged separately. Pointer arithmetic, scalar
control plumbing, allocator execution,
formatter internals and machine instructions are outside these units. For
`S` visited shallow streams, `S <= N + 1`:

| Metadata component | Ceiling | Included operations |
| --- | --- | --- |
| Stream setup and cleanup | `64 * S` | Stream acquisition and reference increment/overflow check; uniqueness checks; clone allocation and iterator setup; terminal probes; length publication; source reference release; owning iterator setup, remaining-slice cleanup and deallocation; wrapper dispatch and depth checks |
| Native token operations | `32 * N` | Clone-loop iterator visits and progress; variant dispatch and payload allocation; group reference operations; owning iterator advance; token destruction or remaining-slice destruction; shared group-stream destructor |
| Measurement updates | `32 * N` | Checked counters and numeric square; assignments; spelling-state initialization; node-bound check; formatting result; chunk length/first-byte capture; digit and punctuation classification |
| Root and result state | `64` | Counter initialization, root invocation and result construction |

The native-token row comprises four clone-loop enumerate/take/index events,
three token-kind/backend clone selections, at most six payload metadata,
reference or allocation-request events, six owning-advance routing and
pointer/end/read events, one reader handoff, one remaining-slice drop visit,
and ten destruction events: `4 + 3 + 6 + 6 + 1 + 1 + 10 = 31 <= 32`.
The destruction ceiling includes two token/backend selections, the stream
destructor entry, empty-stack initialization, the uniqueness query and two
reference-count comparisons, reference decrement and zero check, and empty-stack
teardown. No repeated element loop is hidden inside a single dispatch unit.

The stream row totals at most 39 events: acquisition, reference increment and
overflow check (three); conversion/backend/ownership routes (four); uniqueness
tests (two); vector clone setup and allocation request (four); clone adapters,
terminal probe and length publication (five); old reference release and zero
check (two); owning iterator field setup (seven); terminal advance routing and
end test (six); cleanup guard, remaining view and deallocation request (three);
and depth limit, addition and check (three). Their sum is
`3 + 4 + 2 + 4 + 5 + 2 + 7 + 6 + 3 + 3 = 39 <= 64`.
The fixed root/result allowance covers seven statistic initializations and
the root clone/invocation and result checks and moves.
A numeric literal uses seven
checked updates and six statistic assignments. Its three spelling-state
initializations, node-bound and formatting-result checks, at most six chunk
measurement operations, two digit-classification operations and variant
dispatch give at most 27 reader events, below its row ceiling. Other token
kinds use fewer reader events. The metadata sum is at most
`64 * (N + 1) + 32 * N + 32 * N + 64 = 128 * N + 128`.

The inspected native `RcVec::make_owned` clones each shallow vector once. The
generic slice clone visits each element once; the owning iterator moves each
yielded token once, and its destructor visits each remaining token once on
refusal. These are disjoint yielded and remaining sets. The borrowed original
keeps every child stream shared. Consequently the native group-stream
destructor performs its empty-stack initialization and failed uniqueness check,
then releases its reference; it cannot enter the recursive ownership walk or
grow that stack. The same vector clone, owning iterator and shared-reference
kernels were inspected in the named nightly and Rust 1.98.1 sources:
[slice cloning on stable](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/slice.rs)
and [nightly](https://github.com/rust-lang/rust/blob/21b707e3f97e0b522ebd2f277a862339625ad83f/library/alloc/src/slice.rs),
[owning iteration on stable](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/vec/into_iter.rs)
and [nightly](https://github.com/rust-lang/rust/blob/21b707e3f97e0b522ebd2f277a862339625ad83f/library/alloc/src/vec/into_iter.rs),
and [reference counting on stable](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/rc.rs)
and [nightly](https://github.com/rust-lang/rust/blob/21b707e3f97e0b522ebd2f277a862339625ad83f/library/alloc/src/rcs/rc.rs).
The default literal formatting operation forwards its single body chunk to the
counting writer. It does not repeat the reader's chunk-accounting operations
per spelling byte. These measurement event bounds do not close other native
work obligations.

Integer arithmetic uses radix digits, including hexadecimal letters in a
generated part such as `0xFFFF` from floating spelling `0.0xFFFF`. Original ASCII
digit count alone does not bound this work. Let `S_num` sum original numeric
spelling lengths and `Q_num` sum their squares. Six original visits and disjoint
generated parts give total consumed-digit counts at most `7 * S_num` and
squared counts at most `7 * Q_num`. With the local multiply/carry ceiling
`2 * d * d + 4 * d`, aggregate arithmetic visits are at most
`14 * Q_num + 28 * S_num`, coarsely `14 * b * b + 28 * b`. Scans, validation,
allocation, formatting, decoding and other literal work remain separate.

Literal conversion work also distinguishes explicit byte inspections, loop
iterations, UTF-8 decode lengths, Unicode predicates and logical byte writes or
copies. These units do not count scalar control plumbing, machine instructions
or allocator execution. For one decoder input of `n` spelling bytes:

| Decoder component | Ceiling |
| --- | --- |
| Explicit indexed-byte inspections | `2 * n + 8` |
| Combined decoder-loop iterations | `2 * n + 2` |
| UTF-8 bytes decoded by character access | `n` |
| Decoded content and copied suffix bytes | `n` |
| Unicode identifier predicates | zero |

Cooked decoding advances through content, escapes or continuation whitespace;
its repeated boundary inspections map to disjoint escape introducers. Raw
decoding checks opening and closing hashes, with a separate reverse-quote
search over at most `n` bytes. Character and byte decoders satisfy the same
ceilings. Unicode escape loops also consume underscores, so their work is not
bounded by the six accepted hexadecimal digits alone. For `U` successful
C-string Unicode escapes, add `4 * U` scratch-zero writes and at most `4 * U`
encoding writes; `U <= floor(n / 5)` because the shortest escape occupies five
bytes. Output copies are already counted above. Native NUL searches, buffer
relocations and other delegated kernels remain separate. Raw helpers require
lexer-produced spellings; arbitrary unterminated raw input can reach an
unchecked closing-quote assumption.

The admitted decoder bundle sums these inspection, loop, UTF-8 and logical
output events to at most `8 * n + 10` per invocation, using
`8 * U <= 2 * n`. Decoded content and copied suffix bytes together fit `n`:
their input ranges are disjoint, and decoding does not expand content beyond
its consumed spelling. This copy unit counts one logical copied byte, without
separate physical read/write or relocation events.

Only original nonnumeric literals reach these decoder families. Negative
helpers call integer and float converters, including on failed nonnumeric
trials; generated tuple-index parts contain numeric spelling and suffix data,
not quoted bodies. Each original decoder spelling therefore has at most six
visits. Both production inspection callers prepay
`48 * S_lit + 60 * K_lit` decoder events alongside the separate radix arithmetic
charge, before syntax parsing starts. Counting numeric literals too is a
conservative linear allowance; ordinary comments and identifier text are absent
from these actual literal statistics. Documentation comments lower to string
literals and correctly contribute linear decoder work. Zero literals contribute
zero decoder work.
NUL searches, raw reverse searches, validation spans and buffer relocation have
an additional logical-span charge below. Kind dispatch, reconstruction and
kernel execution retain their separate work obligations. The numeric linear
components below have their own admission charge.

Integer conversion has the following additional components, separately from
its multiply/carry visits. Here `d` counts consumed radix digits.

| Integer component | Ceiling |
| --- | --- |
| Header and main-loop byte inspections | `n + 4` |
| Exponent-disambiguation scan bytes | `n` |
| Suffix UTF-8 decode bytes | `2 * n` |
| Unicode identifier predicates | `2 * n` |
| Decimal-vector reserve calls | `2 * d` |
| Reserve suffix-element comparisons | `6 * d` |
| New zero digits | `4 * d` |
| Decimal-vector growth relocation | `max(8, 4 * d + 4)` |
| Decimal-output traversal and writes | `4 * d` each |
| Negative decimal-output shift and growth relocation | `4 * d` each |
| Conditional minus-sign write | one |
| Decimal boxed-string shrink copy | `4 * d + 1` |
| Suffix ownership copy | `n` |

The exponent branch leaves integer scanning. It can validate a candidate suffix
and subsequently validate the final suffix, hence the two-validation ceiling.
The vector-growth ceiling uses its separate capacity proof; the coarser output
traversal ceiling does not establish that capacity.

Float conversion copies `n` input bytes, visits at most `n` main-loop positions
and writes at most `n` compacted bytes. Direct main-loop buffer reads number
at most `2 * n + 1`: each visited position supplies the dispatch read; digit
and accepted-sign positions supply disjoint second reads, and the sole accepted
minus sign can supply one third read. The sign state is never reset, so another
sign fails before either extra read. Every continuing iteration advances the
read cursor; malformed spellings terminate without revisiting a position.
Exponent lookaheads cover disjoint ranges totaling at most `n`; two fixed
header inspections remain additional.
For suffix length `s` and compacted output length `w`, suffix splitting and
UTF-8 decoding each cover at most `s` bytes, Unicode predicates number at most
`s`, and boxed output shrinking copies at most `w` bytes, with `w + s <= n`.
Validation before truncation sees the original `n`-byte buffer. These are
logical scan lengths for delegated native validation, not a physical-read or
machine-instruction bound. Literal-kind dispatch, source-spelling copying and
reconstruction, allocation events and other native kernels retain their own
work obligations. Aggregate linear terms use the weighted spelling ceiling;
digit terms use the separate radix-digit ceiling, and fixed terms use the
invocation count.

The listed integer components sum to `7 * n + 36 * d + 14`, using
`max(8, 4 * d + 4) <= 4 * d + 8`. The listed float components sum to at most
`9 * n + 3`: suffix splitting, suffix decoding, identifier predicates and
boxed output shrinking together fit `3 * n`. The conversion may try integer
and then float, so their combined listed linear components fit
`16 * n + 36 * d + 17` per classification visit.

Both production inspection paths prepay
`112 * (S_lit + a_minus) + 252 * S_num + 17 * (6 * K_lit + J_lit)` for these
components before syntax parsing. The weighted input ceiling is
`7 * (S_lit + a_minus)`, while radix digits total at most `7 * S_num`.
The charge is zero when there are no literals. Failed negative probes before
nonnumeric literals can still copy the full spelling in the float converter
before header refusal. Their scan/copy allowance therefore uses all original
literal spellings, rather than `S_num` alone. Only radix-derived work uses
numeric spellings. Checked arithmetic combines this charge with the decoder
and quadratic arithmetic allowances before charging the original work grant.
This admits the listed logical components, without claiming delegated
validation, kind dispatch, reconstruction, allocator or other native kernels.

Residual decoder scan/copy spans and terminal writes fit `6 * n + 12` per
invocation. Cooked string content growth relocates at most `2 * n + 8` bytes,
and boxing may copy at most `n` more. Cooked byte strings retain the vector,
with no final string boxing. Cooked C strings additionally scan at most `n`
decoded bytes for NUL, may relocate `n` bytes to reserve the terminator, may
copy `n + 1` bytes on boxing, append one NUL byte, and write one byte on drop.
Even summing those alternatives gives at most `5 * n + 11`. The terminator
growth and final boxing shrink are mutually exclusive in the inspected path.
Raw C strings replace cooked growth with an at-most-`n` reverse quote search.
Their boxed content moves into String and then a vector without cloning.
Cooked or raw byte strings and ordinary strings fit the same ceiling.

The inspected CString generic constructor moves its owned vector input and
performs one NUL search. Its private unchecked constructor reserves exactly
one byte, appends NUL and boxes the vector. Both named toolchains use that
path: [stable CString](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/ffi/c_str.rs)
and [nightly CString](https://github.com/rust-lang/rust/blob/21b707e3f97e0b522ebd2f277a862339625ad83f/library/alloc/src/ffi/c_str.rs).
String character pushes write UTF-8 directly into reserved output without a
separate temporary encoding buffer. Boxed string conversion moves ownership;
float conversion validates the original buffer through `String::from_utf8`:
[stable String](https://github.com/rust-lang/rust/blob/1.98.1/library/alloc/src/string.rs)
and [nightly String](https://github.com/rust-lang/rust/blob/21b707e3f97e0b522ebd2f277a862339625ad83f/library/alloc/src/string.rs).

The production charge adds `36 * S_lit + 72 * K_lit` for six decoder visits
and `7 * (S_lit + a_minus)` for at most one float-validation span per
classification visit. These units count logical scan spans, possible relocated
or copied bytes, and the stated terminal writes. They do not bound physical
machine reads, allocator execution, all kernel instructions, constructor
dispatch or reconstruction. Checked arithmetic combines the spans with the
other conversion work before parser entry; zero literals contribute zero.

Spelling construction uses explicit source-read and destination-write events.
For an effective classification input of length `n`, the cursor's literal
clone costs at most `2 * n` such events, and native Display into a fresh String
costs another `2 * n`. Inserting the first negative sign can relocate and shift
at most `n` bytes each, for `4 * n + 1` events including its sign write.
Successful negative reconstruction copies at most `n` bytes into its native
literal representation and restores the sign, adding `6 * n + 1` events.
The total is at most `14 * n + 2`. These are source-spelling insertions; the
separate decimal-output insertion is already in the numeric linear charge.

Native literal Display writes its stored spelling in one chunk, so fresh
String growth has no existing payload to relocate. Sign insertion grows only
when capacity equals length; its possible relocation therefore copies at most
the old spelling length. Both named String implementations encode the inserted
ASCII sign directly into its destination without a temporary encoding buffer.
The weighted spelling and visit bounds give an admission charge of
`98 * (S_lit + a_minus) + 2 * (6 * K_lit + J_lit)`.

The outer tuple-member operation clones the native float spelling and displays
it before parsing its parts. That pair is additional to classification visits
and costs at most `4 * (S_num + min(a_minus, M_lit))` read/write events.
The minus allowance is necessary for negative member spellings, even when the
subsequent integer-index parse refuses. Production inspection checks and adds
both copy charges before syntax parsing, retaining the zero-literal guard.
Native source-map registration work keeps its existing separate prepayment.
Reconstruction numeric scans and cursor count spans have an additional
admission below. Split/subspan/index conversion, span joins and constructor
metadata remain separate obligations. In particular, native cursor advancement
counts characters under span locations; it must not be treated as constant work.

Successful negative conversion reparses its spelling through the native literal
constructor. For its input length `n`, the numeric lexer routes have these
named scan/visit/predicate bounds:

| Component | Float route | Integer route |
| --- | --- | --- |
| UTF-8 decoded byte extents | `2 * n + 8` | `n + 8` |
| Cursor-advance character-count spans | `3 * n` | `2 * n + 2` |
| Digit/exponent loop visits | `n + 2` | `n + 1` |
| Suffix identifier visits | `n + 1` | `n + 1` |
| Identifier predicates | `n + 3` | `n + 2` |
| Digit byte inspections | zero | `n` |
| Total | `8 * n + 14` | `7 * n + 14` |

Float exponent handling eagerly counts its pre-exponent prefix and may then
count the full numeric extent. Suffix parsing decodes its characters and
advances over that suffix again. Both float and integer routes are charged,
including failed float selection before integer lexing. Failed nonnumeric
family prefixes add at most 13 compared bytes and radix prefixes add six.
Outer negative handling adds at most ten events: two first-character probes
of at most four bytes each, one advance byte and one digit predicate. The sum
is at most `15 * n + 57 <= 16 * n + 64`.

Only successful numeric conversions enter this reconstruction constructor.
Nonnumeric negative probes therefore contribute no reconstruction inputs.
Across original and generated parts, input lengths total at most
`7 * (S_num + min(a_minus, M_lit))` and calls number at most `8 * M_lit`.
Six original visits contribute six weighted numeric spellings; generated
parts add one disjoint weighted spelling and at most two calls per original
numeric literal. At most one original minus can prefix each such literal.
Production inspection prepays
`112 * (S_num + min(a_minus, M_lit)) + 512 * M_lit` for these named events,
alongside the other conversion terms and before parsing. The weighted numeric
spelling sum and every product and addition are checked before charging.
This excludes source-map registrations and spelling copies already charged
separately, constructor metadata, predicate internals, pattern scratch,
allocator execution and general scalar/control machinery. Split/subspan/index
conversion, span joins and the other native obligations remain open.

The production pipeline retains every required distinction through collection,
discovery, proof, reporting and adoption. Independent policy and report
identities extend the exact Census contract without changing whole-unit
grouping or using a similarity score as an enforcement verdict.

## Verification

Private-core fixtures cover renaming, repeated uses, distinct shadowing
declarations, captures, fixed operators and names, shorthand fields, control
opcodes, missing declaration metadata, shared live storage, and exact and short
work and storage grants. Discovery fixtures cover rightmost ties, persistent
minima, an embedded renamed sequence, short input and deliberately colliding
candidate hashes.

Source fixtures read Rust as data and join its original native tokens. They
cover a renamed Unicode binding inside a larger body, an unrelated prefix,
changed variable use and operators, punctuation spacing, closure and return
anchors, opaque expression macros, cross-block cuts and invalid UTF-8 cuts.
Documented opaque local items preserve disjoint valid regions. Missing tokens,
overlapping binding annotations and partial identifier spans refuse.
Documentation lowering at the written delimiter-depth limit preserves an
adjacent valid expression beside an opaque local constant.
Context fixtures cover inline and outline ancestry, literal paths and raw
module identifiers, disabled-prelude inheritance, distinct shared-file reaches,
parent globs, root `no_std`, invalid attribute placement and original context
work and storage exhaustion.
Workspace fixtures exercise actual Source owners and matching regions, every
reach of a shared function, enclosing const parameters, receiver captures,
empty-body attributes, all generic-parameter kinds and configured pruning.
Exact and short collection grants cover retained owner records as well as
facts and lexical output.
Boundary fixtures cover async function, closure and block destinations, nested
try destinations, const functions, const-local shadowing and retained generic
parameters. Invalid dynamic captures, outward control transfers, unavailable
async edition profiles and unsupported feature declarations refuse beside
valid lexical neighbours. Explicit type and generic const positions retain
their own boundaries. Fixture acceptance makes no compiler-validity claim.
Public collection diagnostics escape untrusted path spellings, including
newlines, terminal escapes and bidi controls, while preserving typed source
identity. An ordinary filename supplies the neighbouring accepted spelling.
Parameter-pattern fixtures retain their binders without emitting body markers
for signature-owned const arguments, const blocks or range endpoints. Actual
workspace provenance fixtures retain root and inline-module prelude policy
through free functions, inherent methods and trait-default bodies.
Pairing fixtures protect the embedded helper beneath an invalid surrounding
map, different operators and variable uses, three distinct repeated occurrences,
crossing maximal alignments, short or empty input and repetitive boilerplate.
Repeated joined operators and receiver plumbing refuse beside valid neighbours.
Forced candidate hash collisions
fail coarse or binding proof. Exact and short count, work and storage grants
exercise the complete Source-collection and pairing path. Verified identity
bytes agree on both independently admitted endpoints.
The owning-analysis fixture retains multiple functions across bounded parser
callbacks and refuses an exhausted shared grant. No fixture program is executed.
The preparation fixture keeps a negated string containing 256 digits within a
work grant below the mistaken quadratic charge for those string digits alone.
This intentionally compiler-invalid syntax is only data. A grant one unit below
the measured work refuses the final conversion admission after lexing, before
the syntax visitor can run. Ordinary negative numeric syntax remains an
admitted neighbouring case.
Cooked and raw C strings, a Unicode C-string escape and a raw byte string in
that same fixture exercise the terminal and moved-buffer paths without running
the represented program.
Long negative integer and suffixed exponent spellings exercise sign-buffer
growth, failed float selection before integer reconstruction, repeated exponent
prefix counts and suffix advancement, preserving their original token spelling.
Private report fixtures preserve exact source spans and all indexed owners
under file, function and owner reordering. Both native canonical formats round
trip through the claims reader. Forced hash collisions remain candidate
evidence without witness records. Invalid profiles, unknown fields, impossible
ranges and incompatible candidate support refuse beside valid evidence.
Consistently reminted path aliases, oversized owner bounds and subthreshold
witness claims refuse without relying on stale node names. Empty source and
short bodies produce valid reports without candidates or witnesses. Oversized
imported text refuses before native parsing.
Debt fixtures adopt three repeated occurrence pairs, preserve unrelated source
position and local changes, record exact shrinkage and refuse restored or new
growth. Trait and generic self-type changes retain different owner descriptors.
Physical aliases preserve one occurrence count and canonical owner ordering.
Actual checked external MemoryTree source domains preserve allowances when
catalog mount order changes; neighbouring unbound roots refuse. Native RDF
round trips preserve exact counts, while wrong profiles, zero and overflowing
counts, orphan records and evidence-report roots refuse.

Native Linux source fixtures exercise the actual production library and CLI
with checked matching spans and neighbouring invalid inputs. File-order
changes produce identical canonical reports. Adversarial repetition refuses
actionably beside an admitted neighbour. Existing fingerprint goldens remain
unchanged. Explicit adoption and subsequent debt comparison reject growth and
retain exact occurrence multiplicity.
