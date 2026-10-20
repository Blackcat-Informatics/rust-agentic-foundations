---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-normal-form
title: "Census normal form, fingerprints and grouping"
description: "How the RAF census normalises function bodies, fingerprints them with BLAKE3 and groups the duplicates."
state: delivered
bindings:
  code:
    - raf_census::Baseline
    - raf_census::Baseline::from_json
    - raf_census::Baseline::to_json
    - raf_census::BaselineGroup
    - raf_census::Census
    - raf_census::Census::baseline
    - raf_census::Census::body_groups
    - raf_census::Census::file
    - raf_census::Census::files
    - raf_census::Census::groups
    - raf_census::Census::hex_tables
    - raf_census::Census::index
    - raf_census::Census::load
    - raf_census::Census::match_jobs
    - raf_census::Census::options
    - raf_census::Census::shared_constants
    - raf_census::Census::shim_groups
    - raf_census::Census::unit
    - raf_census::Census::units
    - raf_census::Census::units_of
    - raf_census::CensusOptions
    - raf_census::CensusOptions::max_file_bytes
    - raf_census::CensusOptions::min_tokens
    - raf_census::ConstantRow
    - raf_census::DEFAULT_MAX_FILE_BYTES
    - raf_census::DEFAULT_MIN_TOKENS
    - raf_census::Divergence
    - raf_census::ENCODING
    - raf_census::Error
    - raf_census::FileCtx
    - raf_census::Fingerprint
    - raf_census::Fingerprint::body
    - raf_census::Fingerprint::kind
    - raf_census::Fingerprint::shim
    - raf_census::Form
    - raf_census::Form::is_empty
    - raf_census::Form::len
    - raf_census::Form::texts
    - raf_census::Form::tokens
    - raf_census::Group
    - raf_census::Group::member_symbols
    - raf_census::GroupKind
    - raf_census::Hashed
    - raf_census::HexTable
    - raf_census::Index
    - raf_census::Index::from_json
    - raf_census::Index::to_json
    - raf_census::IndexGroup
    - raf_census::IndexUnit
    - raf_census::Job
    - raf_census::Job::entry_points
    - raf_census::Job::forbidding
    - raf_census::Job::new
    - raf_census::Job::variants
    - raf_census::JobForbidden
    - raf_census::JobMatch
    - raf_census::JobReport
    - raf_census::JobRow
    - raf_census::LITERAL_TIER
    - raf_census::Level
    - raf_census::Level::ALL
    - raf_census::Level::id
    - raf_census::MAX_NESTING
    - raf_census::MIN_SHIM_TOKENS
    - raf_census::MatchClass
    - raf_census::MatchRow
    - raf_census::Member
    - raf_census::MemberRow
    - raf_census::NORMAL_FORM
    - raf_census::Normalizer
    - raf_census::Normalizer::explain
    - raf_census::Normalizer::new
    - raf_census::Normalizer::print
    - raf_census::Normalizer::tower
    - raf_census::PASSES
    - raf_census::ParseFingerprintError
    - raf_census::Print
    - raf_census::RuleId
    - raf_census::RuleId::as_str
    - raf_census::RuleId::new
    - raf_census::SHIM_MAX_TOKENS
    - raf_census::SharedConstant
    - raf_census::Slice
    - raf_census::Slice::new
    - raf_census::Slice::params
    - raf_census::StringKind
    - raf_census::StringLiteral
    - raf_census::TableKind
    - raf_census::TableKind::name
    - raf_census::TableRow
    - raf_census::Token
    - raf_census::TokenDiff
    - raf_census::Tower
    - raf_census::Tower::form
    - raf_census::Tower::new
    - raf_census::Tower::print
    - raf_census::Tower::shim_candidate
    - raf_census::Tower::structural_tokens
    - raf_census::Unit
    - raf_census::UnitKind
    - raf_census::VALUE_ADAPTERS
    - raf_census::WEAK_KEYWORDS
    - raf_census::check_normal_form
    - raf_census::manifest
  tests:
    - raf_census::error::tests::each_variant_renders_its_facts
    - raf_census::error::tests::only_a_load_error_has_a_source
    - raf_census::group::tests::a_collision_is_found_even_when_a_third_unit_stands_alone
    - raf_census::group::tests::members_order_by_symbol_then_file_then_line
    - raf_census::group::tests::one_fingerprint_with_two_digests_is_a_collision_beside_equal_digests
    - raf_census::normalize::literal::tests::an_integer_beyond_u128_is_still_an_int
    - raf_census::normalize::literal::tests::each_literal_kind_has_its_name
    - raf_census::normalize::literal::tests::floats_and_strings_are_not_integers
    - raf_census::normalize::literal::tests::spellings_of_one_value_respell_alike
    - raf_census::options::tests::defaults_are_twenty_tokens_and_eight_mebibytes
    - raf_census::options::tests::max_file_bytes_is_set
    - raf_census::options::tests::zero_min_tokens_is_refused_and_one_is_accepted
    - raf_census::report::tests::render_ends_with_one_newline_and_sorts_keys
    - raf_census::test_crate::forms::a_fingerprint_displays_and_parses_back
    - raf_census::test_crate::forms::a_fingerprint_refuses_every_malformed_spelling
    - raf_census::test_crate::forms::a_fixture_named_for_a_size_has_that_many_structural_tokens
    - raf_census::test_crate::forms::a_shim_candidate_keeps_its_body_print_and_the_shim_print_differs_by_callee
    - raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour
    - raf_census::test_crate::forms::below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them
    - raf_census::test_crate::forms::every_registered_pair_compares_as_registered
    - raf_census::test_crate::forms::explain_names_the_levels_and_the_first_differing_token_with_its_line
    - raf_census::test_crate::forms::fingerprints_sort_by_kind_then_value
    - raf_census::test_crate::forms::nesting_at_the_budget_lexes_and_one_deeper_is_refused
    - raf_census::test_crate::forms::parameters_are_numbered_in_signature_order_before_locals
    - raf_census::test_crate::forms::self_is_not_a_parameter_and_a_field_named_like_one_is_kept
    - raf_census::test_crate::forms::shim_forms_exist_for_six_to_twenty_nine_tokens
    - raf_census::test_crate::forms::the_body_print_needs_min_tokens
    - raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens
    - raf_census::test_crate::forms::the_literal_tier_boundary_is_thirty_structural_tokens
    - raf_census::test_crate::forms::the_tower_is_monotone_over_every_registry_pair
    - raf_census::test_crate::forms::the_value_adapters_are_the_eleven_of_the_page
    - raf_census::test_crate::forms::the_value_form_keeps_binary_operators_and_drops_borrows
    - raf_census::test_crate::golden::every_fixture_fingerprint_is_pinned
    - raf_census::test_crate::golden::the_manifest_is_pinned
    - raf_census::test_crate::golden::the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry
    - raf_census::test_crate::golden::the_pinned_version_is_the_implemented_one
    - raf_census::test_crate::groups::a_constant_in_one_package_is_not_shared
    - raf_census::test_crate::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group
    - raf_census::test_crate::groups::a_function_holding_a_table_is_reported_by_its_symbol
    - raf_census::test_crate::groups::a_job_classifies_home_entry_point_variant_and_copies
    - raf_census::test_crate::groups::a_job_that_matches_nothing_has_an_empty_list
    - raf_census::test_crate::groups::a_shim_id_matches_the_units_that_group_under_it_and_not_the_excluded_ones
    - raf_census::test_crate::groups::bodies_under_min_tokens_never_group
    - raf_census::test_crate::groups::constants_above_ffff_in_two_packages_are_shared
    - raf_census::test_crate::groups::every_form_of_a_hex_table_is_reported_in_both_cases
    - raf_census::test_crate::groups::groups_of_one_are_dropped_and_members_follow_symbol_order
    - raf_census::test_crate::groups::groups_order_by_count_then_tokens_then_fingerprint
    - raf_census::test_crate::groups::jobs_match_shim_and_table_fingerprints
    - raf_census::test_crate::groups::near_misses_of_a_hex_table_are_not_reported
    - raf_census::test_crate::groups::shared_constants_run_by_value_ascending
    - raf_census::test_crate::groups::shim_groups_use_the_shim_key_and_skip_trait_impls_and_self_forwarders
    - raf_census::test_crate::page::the_member_lists_are_the_closed_schemas
    - raf_census::test_crate::page::the_page_names_every_pass_level_and_the_encoding
    - raf_census::test_crate::page::the_page_names_every_report_member_of_both_reports
    - raf_census::test_crate::page::the_page_names_every_threshold_with_its_value
    - raf_census::test_crate::page::the_page_names_every_value_adapter_and_weak_keyword
    - raf_census::test_crate::reports::a_shared_constant_is_hexadecimal_text
    - raf_census::test_crate::reports::an_absolute_directory_reports_the_same_as_the_relative_spelling
    - raf_census::test_crate::reports::an_unknown_key_missing_key_or_wrong_type_is_not_a_report
    - raf_census::test_crate::reports::both_reports_read_back_what_they_write
    - raf_census::test_crate::reports::output_has_sorted_keys_and_one_trailing_newline
    - raf_census::test_crate::reports::the_capability_examples_hold_the_example_output
    - raf_census::test_crate::reports::the_example_output_is_the_pinned_golden
    - raf_census::test_crate::reports::the_example_output_names_the_expected_group_fingerprints
    - raf_census::test_crate::reports::the_index_renders_the_classified_matches_of_every_job
    - raf_census::test_crate::reports::two_fresh_loads_of_the_fixtures_give_identical_bytes
    - raf_census::test_crate::reports::two_fresh_loads_of_this_repository_give_identical_bytes
    - raf_census::test_crate::units::a_constant_whose_type_names_a_number_does_not_hold_it
    - raf_census::test_crate::units::a_module_declared_by_a_library_and_a_binary_is_one_unit
    - raf_census::test_crate::units::a_self_call_at_any_depth_removes_the_shim_key
    - raf_census::test_crate::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship
    - raf_census::test_crate::units::a_turbofish_call_is_a_forwarder_and_keeps_its_key_beside_a_twin
    - raf_census::test_crate::units::a_unit_is_found_by_symbol
    - raf_census::test_crate::units::a_workspace_without_roots_does_not_load
    - raf_census::test_crate::units::byte_strings_and_c_strings_are_strings
    - raf_census::test_crate::units::each_kind_yields_one_unit_with_every_field
    - raf_census::test_crate::units::every_spelling_of_an_integer_is_one_constant
    - raf_census::test_crate::units::file_context_names_package_path_and_shipping
    - raf_census::test_crate::units::hex_tables_are_read_from_strings_and_arrays
    - raf_census::test_crate::units::max_file_bytes_caps_a_file_and_the_cap_itself_loads
    - raf_census::test_crate::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten
    - raf_census::test_crate::units::parameters_of_every_pattern_are_renamed
    - raf_census::test_crate::units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders
    - raf_census::test_crate::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol
    - raf_census::test_crate::units::strings_carry_their_absolute_lines
    - raf_census::test_crate::units::test_code_of_a_shipping_target_does_not_ship
    - raf_census::test_crate::units::this_repositorys_own_crates_load_into_distinct_units
    - raf_census::units::sig::tests::a_default_method_of_an_implementation_is_a_signature
    - raf_census::units::sig::tests::an_unparsable_initializer_is_a_source_error_beside_a_valid_one
    - raf_census::units::sig::tests::an_unparsable_signature_is_a_source_error_beside_a_valid_one
    - raf_census::units::sig::tests::parameters_are_every_bound_name_in_signature_order
    - raf_census::units::sig::tests::the_initializer_is_between_the_equals_and_the_semicolon
    - raf_census::units::tests::text_outside_an_item_is_a_source_error_beside_text_inside_it
    - raf_census::version::tests::other_versions_are_refused_naming_both_and_the_migration
    - raf_census::version::tests::the_implemented_version_is_one
    - raf_census::version::tests::the_implemented_version_passes
  examples:
    - raf_census::example_crate::baseline
    - raf_census::example_crate::goldens
depends: []
---

# Census normal form, fingerprints and grouping

## Purpose

The census finds helpers that a workspace implements more than once. Textual
comparison misses most duplicates, because two copies of one algorithm differ in
local names, borrows and clones. The normal form rewrites a function body into a
canonical token sequence, and the fingerprint names that sequence, so equal
algorithms get equal fingerprints. This capability is the foundation every other
census capability stands on.

The census is a library: a caller loads a workspace through `raf-source`, and the
census reports which units share a fingerprint and which of them a ledger job
forbids outside its home. The fingerprint is only comparable inside one normal-form
version, so the version is a value the library exposes and a gate it applies to
every version a caller declares.

## Contract

The normative definition of the normal form is
[the architecture page](../architecture/normal-form.md). This capability
implements it as follows.

- The census implements `normal_form = 1`. A body is lexed with `proc_macro2`,
  including the arguments of macro invocations, and rewritten by four passes
  (`rename_params`, `rename_locals`, `kind_literals`, `erase_held`) into its shim
  form, its structural form and its value form. Integer literals are respelled to
  decimal at lex time, so every spelling of one value is one token.
- The value form drops borrows, derefs, `ref` bindings, the value adapters listed
  in `VALUE_ADAPTERS` (`as_str`, `as_ref`, `as_slice`, `as_mut`, `as_deref`,
  `borrow`, `borrow_mut`, `clone`, `cloned`, `copied`, `to_owned`), full-range
  reindexing, `let` type annotations and type-only turbofish.
- A body with fewer than 30 structural tokens keeps its literals verbatim (the
  literal tier). A longer body has its literals abstracted. Every threshold counts
  structural tokens, delimiters included, before the value form drops anything.
- A body is a shim candidate when it is one expression (no `let`, no `;` before
  its tail), contains a call, does not mention `self` and has 6 to 29 structural
  tokens; its shim form renames its own parameters. `MIN_SHIM_TOKENS` is 6 and
  `SHIM_MAX_TOKENS` is 30. A shim candidate keeps its `body` fingerprint when it
  reaches `min_tokens`, which the index lists, but it never joins a body group:
  only its `shim` fingerprint groups, and only when its `shim_key` is present. The
  `shim_key` is absent for a method of a trait implementation and for a forwarder
  that calls through `Self::`, a call at any depth whose callee path starts with
  `Self::`.
- A body with fewer than `min_tokens` structural tokens has no body fingerprint.
  `min_tokens` defaults to 20 and `[census] min_tokens` sets it.
- A fingerprint is `<kind>:` followed by an identifier. For `body` and `shim` the
  identifier is the lowercase hex of the first 8 bytes of the BLAKE3 hash of the
  form under the encoding `tokens-len-u32le-utf8/blake3/first8-be-hex16`. For
  `table` and `rule` it is a fixed name. The kinds are `body`, `shim`, `table` and
  `rule`.
- `raf_census::NORMAL_FORM` is 1. `raf_census::check_normal_form(declared)`
  accepts 1 and refuses every other version with `Error::Version`, whose message
  names the declared version, the implemented version and `cargo census migrate`.
  Reading a version out of a ledger file is the ledger's.
- `raf_census::CensusOptions` is `#[non_exhaustive]` and has `Default` and two
  setters. `min_tokens` defaults to 20, and the setter refuses 0 with
  `Error::Options`. `max_file_bytes` defaults to 8 MiB and is the per-file read cap
  of the workspace load; a file above it is a load error that names the file.
- `raf_census::Error` is the one error type: `Source` (file, line and reason of a
  slice that does not lex or a signature or initialiser that does not parse),
  `Options`, `Collision` (two different forms with one fingerprint, naming both
  symbols), `Version`, `Report` and a load error of `raf-source`. Nothing falls back
  to a partial result.
- Units are collected once per source item, whatever number of targets reach it,
  so a module declared by both a library and a binary yields one unit. A unit is
  a function, a method (inherent, trait implementation or trait default), a
  constant or a static; its identity is its file, line and symbol, its symbol is
  chosen deterministically (a shipping reach over a test reach, then the smallest
  path), and it is shipping when any target that reaches it ships. A unit holds its
  print, its integer constants by value, its string literals with their lines
  (string, byte string and C string) and the tables it holds.
- The census groups units by fingerprint in a `BTreeMap` from fingerprint to
  members. A group has two or more members. Groups sort by member count descending,
  then tokens descending, then fingerprint ascending, and their members by symbol,
  file and line, so the same source always produces the same output. Two units with
  one 8-byte fingerprint and different digests are an `Error::Collision`.
- Integer constants are compared by value in any spelling. The census recognises
  hex-digit tables (a string, a byte string or an array of 16 `char` or `u8`
  literals that is exactly the sixteen digits in one case) and reports them as
  `table:hex-lower` and `table:hex-upper`. A value above `0xffff` that units of two
  or more packages hold is a shared constant.
- A job (`raf_census::Job`) has an id, a home symbol, entry-point symbols, variant
  symbols and a forbidden set of fingerprints and integer constants.
  `Census::match_jobs` returns one report per job. A unit matches when its `body`
  fingerprint, its `shim_key` or its `table:` id is a forbidden fingerprint, or when
  one of its integer constants equals a forbidden constant by value. A match is
  `Home` when the unit's symbol is the home or an entry point, `Variant` when it is a
  listed variant, and `Copy` otherwise, and the job counts its copies. A `rule:`
  fingerprint matches no unit here.
- The baseline report has the members `normal_form`, `min_tokens`, `units` (the
  count), `body_groups`, `shim_groups`, `hex_tables` and `shared_constants`. The
  index report has `normal_form`, `units` (symbol, package, file, line, shipping,
  tokens, body and shim), `groups` (kind, fingerprint and members) and `jobs`
  (id, home, matches with symbol and class, and copies). The `shim` of an index unit
  is its `shim_key`. Both reports are closed schemas that read back what they write,
  and they render with sorted keys and a trailing newline. A constant value is a
  `0x` hexadecimal string.
- Any change that can alter a fingerprint bumps `normal_form`.

## Usage

A caller loads the workspace with `raf_census::Census::load(&tree, roots,
&options)`, normalises single sources with `raf_census::Normalizer`, and asks
`Census::match_jobs` which units a set of `Job` values forbids. `Census::baseline`
and `Census::index` build the two reports. `check_normal_form` is the gate a ledger
reader applies to the version it finds. `cargo census` ([census-cli](./census-cli.md))
and the rule API ([census-rule-api](./census-rule-api.md)) are the designed
consumers of this library.

A fingerprint is used in two places. A ledger job lists it under
`forbidden.fingerprints` so that a copy outside the job's home fails the gate (see
[census-ledger](./census-ledger.md)). A reader compares fingerprints in a baseline
to see which functions the census considers the same.

`Normalizer::explain` compares two sources level by level and names the coarsest
level at which they still differ and the first differing token with its line, which
is how a reader learns why two bodies that look alike are reported as distinct.

## Examples

These two functions are one algorithm in the value form, and the census puts them
in one `body` group:

```rust
fn total(items: &[u64]) -> u64 {
    let mut sum = 0;
    for item in items.iter() {
        sum += *item;
        if sum > 1_000_000 { sum = 1_000_000; }
    }
    sum
}

fn add_up(values: &[u64]) -> u64 {
    let mut acc: u64 = 0;
    for v in values.iter() {
        acc += *v;
        if acc > 1_000_000 { acc = 1_000_000; }
    }
    acc
}
```

The `baseline` example of the crate prints both reports of a built-in workspace
(`cargo run -p raf-census --example baseline`); given a directory it prints the
reports of that directory, with no jobs. The workspace has one body group (`alpha`
and `beta`), one shim group (`one` and `two`, forwarders to `helper`), a
hexadecimal digit table, the constant 65536 and one job `sum` whose home is
`alpha` and which forbids the body group's fingerprint and the constant 65536. The
baseline report:

```json
{
  "body_groups": [
    {
      "fingerprint": "body:6bb022e241a79a28",
      "members": [
        {
          "file": "src/lib.rs",
          "line": 1,
          "symbol": "demo::alpha"
        },
        {
          "file": "src/lib.rs",
          "line": 2,
          "symbol": "demo::beta"
        }
      ],
      "tokens": 21
    }
  ],
  "hex_tables": [
    {
      "file": "src/lib.rs",
      "line": 6,
      "symbol": "demo::digits",
      "table": "hex-lower"
    }
  ],
  "min_tokens": 20,
  "normal_form": 1,
  "shared_constants": [],
  "shim_groups": [
    {
      "fingerprint": "shim:203616cb54a3e53d",
      "members": [
        {
          "file": "src/lib.rs",
          "line": 4,
          "symbol": "demo::one"
        },
        {
          "file": "src/lib.rs",
          "line": 5,
          "symbol": "demo::two"
        }
      ],
      "tokens": 6
    }
  ],
  "units": 7
}
```

The index report of the same workspace lists each unit with its `body` and `shim`
fingerprints, each group with its members, and the job with its classified matches:

```json
{
  "groups": [
    {
      "fingerprint": "body:6bb022e241a79a28",
      "kind": "body",
      "members": [
        {
          "file": "src/lib.rs",
          "line": 1,
          "symbol": "demo::alpha"
        },
        {
          "file": "src/lib.rs",
          "line": 2,
          "symbol": "demo::beta"
        }
      ]
    },
    {
      "fingerprint": "shim:203616cb54a3e53d",
      "kind": "shim",
      "members": [
        {
          "file": "src/lib.rs",
          "line": 4,
          "symbol": "demo::one"
        },
        {
          "file": "src/lib.rs",
          "line": 5,
          "symbol": "demo::two"
        }
      ]
    }
  ],
  "jobs": [
    {
      "copies": 2,
      "home": "demo::alpha",
      "id": "sum",
      "matches": [
        {
          "class": "copy",
          "symbol": "demo::BIG"
        },
        {
          "class": "home",
          "symbol": "demo::alpha"
        },
        {
          "class": "copy",
          "symbol": "demo::beta"
        }
      ]
    }
  ],
  "normal_form": 1,
  "units": [
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 7,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::BIG",
      "tokens": 0
    },
    {
      "body": "body:6bb022e241a79a28",
      "file": "src/lib.rs",
      "line": 1,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::alpha",
      "tokens": 21
    },
    {
      "body": "body:6bb022e241a79a28",
      "file": "src/lib.rs",
      "line": 2,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::beta",
      "tokens": 21
    },
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 6,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::digits",
      "tokens": 1
    },
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 3,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::helper",
      "tokens": 3
    },
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 4,
      "package": "demo",
      "shim": "shim:203616cb54a3e53d",
      "shipping": true,
      "symbol": "demo::one",
      "tokens": 6
    },
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 5,
      "package": "demo",
      "shim": "shim:203616cb54a3e53d",
      "shipping": true,
      "symbol": "demo::two",
      "tokens": 6
    }
  ]
}
```

A directory is read as a root of the tree at the current directory when it is
relative; an absolute directory is read as its parent's tree, and the file paths of
the report then begin with the directory's name, as they do for `crates`.

## Constraints

- The census reads Rust source only. Generated code and code behind macros that
  expand to items are compared as written, not as expanded.
- Macro arguments are re-lexed as token streams. A macro whose arguments are not
  valid Rust tokens is compared by the tokens `proc_macro2` yields.
- A body shorter than `min_tokens` is never a `body` group, however many copies
  exist. Forwarders of 6 to 29 tokens are caught by the shim form instead.
- Fingerprints are 64 bits. The census keeps the full digest of every unit it
  groups and refuses two forms that share a fingerprint, naming both symbols, and a
  ledger that names a fingerprint still lists member symbols that a reader can
  check.
- The `table` and `rule` kinds carry names, not hashes.
- A unit that several targets reach is one unit; the census never counts a shared
  module once per target.
- A shim candidate never joins a body group, so a 20-token forwarder is judged by
  its shim form alone.
- A slice that does not lex, a signature that does not parse and an initialiser that
  does not parse are errors that name the file and line; the census does not skip
  them.
- Integer constants and the fingerprints of a constant or a static come from its
  initialiser expression only, never from its type or attributes.
- A string literal that is not valid UTF-8 is read lossily and never names a table.
- Nesting depth is bounded by `MAX_NESTING` (128) and by the delimiter budget of
  `raf-source`.

## Explanation

Seven copies of one helper census had diverged across the Blackcat repositories,
each with its own normal form. A fingerprint recorded in one repository meant
nothing in another, and a change to a normaliser silently moved every ledger row.
Version 1 unifies them from the strongest parts of each: the structural form,
value form and literal tier of the largest copy, and the `shim_key` refinement that
stops trait-implementation methods and `Self::` forwarders from forming false
groups.

Versioning the form is what makes the ledger trustworthy. A fingerprint is only
comparable inside one version, so the version is written in the ledger and the
census refuses a mismatch instead of reporting spurious drift. Golden tests turn
the version-bump rule from a convention into a check: a normaliser change that
moves a fingerprint fails until the version moves with it.

Literals are kept below 30 tokens because short bodies are mostly constants, and
abstracting them would merge unrelated tables. Longer bodies abstract literals
because there the algorithm, not the constants, is what two copies share.

## Verification

- Golden tests pin the fingerprint of every fixture under `normal_form = 1`, beside
  a manifest of the encoding, the passes, the levels, the thresholds and the word
  lists. They fail when a fixture has no pinned value, when a pinned value has no
  fixture and when a pinned value moves. Every form has fixtures: structural,
  value, literal tier, shim and macro arguments.
- Fixture pairs differing only in local names, parameter names, borrows, clones and
  a `let` type annotation fingerprint equal; pairs differing in a method name, a
  field name or a called function fingerprint different. A property test checks that
  equality at a finer level implies equality at every coarser one.
- A fixture of two 25-token bodies that differ only in a string literal
  fingerprints different, and the same pair at 35 tokens fingerprints equal.
- Shim fixtures cover the 5, 6, 29 and 30 token boundaries, a two-statement and a
  `let` body without a shim form, a trait-implementation method and a `Self::`
  forwarder each beside a neighbour that does group, and a module reached by two
  targets as one unit.
- A fixture with a `0x`-spelled and a decimal-spelled constant yields one constant
  value.
- `check_normal_form` is tested with 0 and 2 (refused, the message naming both
  versions and `cargo census migrate`) and with 1 (accepted). `CensusOptions` is
  tested with `min_tokens` 0 (refused) and 1 (accepted), with the defaults, and with
  a file above and a file at `max_file_bytes`.
- Grouping tests show groups of one are dropped, the order of groups and members,
  the hex tables of each form and case beside the near misses that are not tables,
  and the shared constants across packages.
- A job fixture over one workspace yields a `Home` for the home, a `Home` for an
  entry point, a `Variant`, a `Copy`, and a `Copy` for a unit holding the forbidden
  constant by value, with a copy count of 2, and a job that matches nothing has no
  matches.
- The reports are tested for byte-identical output over two fresh loads, including
  this repository's own crates, for a round trip through their JSON, and for
  refusal of an unknown key. The Examples above are the output of the `baseline`
  example, which a test compares byte for byte.
- A page test reads the architecture page and fails when it does not name every
  pass, level, threshold with its value, value adapter, report member and the
  encoding id.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_census::error::tests::each_variant_renders_its_facts` | test | [Verification](#verification) |
| `raf_census::error::tests::only_a_load_error_has_a_source` | test | [Verification](#verification) |
| `raf_census::example_crate::baseline` | example target | [Verification](#verification) |
| `raf_census::example_crate::goldens` | example target | [Verification](#verification) |
| `raf_census::group::tests::a_collision_is_found_even_when_a_third_unit_stands_alone` | test | [Verification](#verification) |
| `raf_census::group::tests::members_order_by_symbol_then_file_then_line` | test | [Verification](#verification) |
| `raf_census::group::tests::one_fingerprint_with_two_digests_is_a_collision_beside_equal_digests` | test | [Verification](#verification) |
| `raf_census::normalize::literal::tests::an_integer_beyond_u128_is_still_an_int` | test | [Verification](#verification) |
| `raf_census::normalize::literal::tests::each_literal_kind_has_its_name` | test | [Verification](#verification) |
| `raf_census::normalize::literal::tests::floats_and_strings_are_not_integers` | test | [Verification](#verification) |
| `raf_census::normalize::literal::tests::spellings_of_one_value_respell_alike` | test | [Verification](#verification) |
| `raf_census::options::tests::defaults_are_twenty_tokens_and_eight_mebibytes` | test | [Verification](#verification) |
| `raf_census::options::tests::max_file_bytes_is_set` | test | [Verification](#verification) |
| `raf_census::options::tests::zero_min_tokens_is_refused_and_one_is_accepted` | test | [Verification](#verification) |
| `raf_census::report::tests::render_ends_with_one_newline_and_sorts_keys` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::a_fingerprint_displays_and_parses_back` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::a_fingerprint_refuses_every_malformed_spelling` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::a_fixture_named_for_a_size_has_that_many_structural_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::a_shim_candidate_keeps_its_body_print_and_the_shim_print_differs_by_callee` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::every_registered_pair_compares_as_registered` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::explain_names_the_levels_and_the_first_differing_token_with_its_line` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::fingerprints_sort_by_kind_then_value` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::nesting_at_the_budget_lexes_and_one_deeper_is_refused` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::parameters_are_numbered_in_signature_order_before_locals` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::self_is_not_a_parameter_and_a_field_named_like_one_is_kept` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::shim_forms_exist_for_six_to_twenty_nine_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_body_print_needs_min_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_literal_tier_boundary_is_thirty_structural_tokens` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_tower_is_monotone_over_every_registry_pair` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_value_adapters_are_the_eleven_of_the_page` | test | [Verification](#verification) |
| `raf_census::test_crate::forms::the_value_form_keeps_binary_operators_and_drops_borrows` | test | [Verification](#verification) |
| `raf_census::test_crate::golden::every_fixture_fingerprint_is_pinned` | test | [Verification](#verification) |
| `raf_census::test_crate::golden::the_manifest_is_pinned` | test | [Verification](#verification) |
| `raf_census::test_crate::golden::the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry` | test | [Verification](#verification) |
| `raf_census::test_crate::golden::the_pinned_version_is_the_implemented_one` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_constant_in_one_package_is_not_shared` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_function_holding_a_table_is_reported_by_its_symbol` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_job_classifies_home_entry_point_variant_and_copies` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_job_that_matches_nothing_has_an_empty_list` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::a_shim_id_matches_the_units_that_group_under_it_and_not_the_excluded_ones` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::bodies_under_min_tokens_never_group` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::constants_above_ffff_in_two_packages_are_shared` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::every_form_of_a_hex_table_is_reported_in_both_cases` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::groups_of_one_are_dropped_and_members_follow_symbol_order` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::groups_order_by_count_then_tokens_then_fingerprint` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::jobs_match_shim_and_table_fingerprints` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::near_misses_of_a_hex_table_are_not_reported` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::shared_constants_run_by_value_ascending` | test | [Verification](#verification) |
| `raf_census::test_crate::groups::shim_groups_use_the_shim_key_and_skip_trait_impls_and_self_forwarders` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_member_lists_are_the_closed_schemas` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_names_every_pass_level_and_the_encoding` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_names_every_report_member_of_both_reports` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_names_every_threshold_with_its_value` | test | [Verification](#verification) |
| `raf_census::test_crate::page::the_page_names_every_value_adapter_and_weak_keyword` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::a_shared_constant_is_hexadecimal_text` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::an_absolute_directory_reports_the_same_as_the_relative_spelling` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::an_unknown_key_missing_key_or_wrong_type_is_not_a_report` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::both_reports_read_back_what_they_write` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::output_has_sorted_keys_and_one_trailing_newline` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_capability_examples_hold_the_example_output` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_example_output_is_the_pinned_golden` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_example_output_names_the_expected_group_fingerprints` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::the_index_renders_the_classified_matches_of_every_job` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::two_fresh_loads_of_the_fixtures_give_identical_bytes` | test | [Verification](#verification) |
| `raf_census::test_crate::reports::two_fresh_loads_of_this_repository_give_identical_bytes` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_constant_whose_type_names_a_number_does_not_hold_it` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_module_declared_by_a_library_and_a_binary_is_one_unit` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_self_call_at_any_depth_removes_the_shim_key` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_turbofish_call_is_a_forwarder_and_keeps_its_key_beside_a_twin` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_unit_is_found_by_symbol` | test | [Verification](#verification) |
| `raf_census::test_crate::units::a_workspace_without_roots_does_not_load` | test | [Verification](#verification) |
| `raf_census::test_crate::units::byte_strings_and_c_strings_are_strings` | test | [Verification](#verification) |
| `raf_census::test_crate::units::each_kind_yields_one_unit_with_every_field` | test | [Verification](#verification) |
| `raf_census::test_crate::units::every_spelling_of_an_integer_is_one_constant` | test | [Verification](#verification) |
| `raf_census::test_crate::units::file_context_names_package_path_and_shipping` | test | [Verification](#verification) |
| `raf_census::test_crate::units::hex_tables_are_read_from_strings_and_arrays` | test | [Verification](#verification) |
| `raf_census::test_crate::units::max_file_bytes_caps_a_file_and_the_cap_itself_loads` | test | [Verification](#verification) |
| `raf_census::test_crate::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten` | test | [Verification](#verification) |
| `raf_census::test_crate::units::parameters_of_every_pattern_are_renamed` | test | [Verification](#verification) |
| `raf_census::test_crate::units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders` | test | [Verification](#verification) |
| `raf_census::test_crate::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol` | test | [Verification](#verification) |
| `raf_census::test_crate::units::strings_carry_their_absolute_lines` | test | [Verification](#verification) |
| `raf_census::test_crate::units::test_code_of_a_shipping_target_does_not_ship` | test | [Verification](#verification) |
| `raf_census::test_crate::units::this_repositorys_own_crates_load_into_distinct_units` | test | [Verification](#verification) |
| `raf_census::units::sig::tests::a_default_method_of_an_implementation_is_a_signature` | test | [Verification](#verification) |
| `raf_census::units::sig::tests::an_unparsable_initializer_is_a_source_error_beside_a_valid_one` | test | [Verification](#verification) |
| `raf_census::units::sig::tests::an_unparsable_signature_is_a_source_error_beside_a_valid_one` | test | [Verification](#verification) |
| `raf_census::units::sig::tests::parameters_are_every_bound_name_in_signature_order` | test | [Verification](#verification) |
| `raf_census::units::sig::tests::the_initializer_is_between_the_equals_and_the_semicolon` | test | [Verification](#verification) |
| `raf_census::units::tests::text_outside_an_item_is_a_source_error_beside_text_inside_it` | test | [Verification](#verification) |
| `raf_census::version::tests::other_versions_are_refused_naming_both_and_the_migration` | test | [Verification](#verification) |
| `raf_census::version::tests::the_implemented_version_is_one` | test | [Verification](#verification) |
| `raf_census::version::tests::the_implemented_version_passes` | test | [Verification](#verification) |
<!-- raf:generated:end -->
