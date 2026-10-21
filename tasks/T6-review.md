VERDICT: BLOCKED

# T6 review and pre-PR completeness review

## Scope

Commit 7a49256 (T6) and the whole branch `origin/main...HEAD` of the raf-census normal-form change. Every sentence of `docs/architecture/normal-form.md` (P) and `docs/capabilities/census-normal-form.md` (C) was checked against `crates/raf-census/src`, not against the task reports. No file in the worktree was edited. A scratch clone is at `.stage/raf-census-core-normal-form-v1/scratch/t6review`. I ran one throwaway probe test there, `tests/zz_scratch.rs`, which is not part of the branch. No timing runs and no mutation passes.

## Commands run

| Command | Exit | Result |
|---|---|---|
| `cargo test -p raf-census --locked` | 0 | 22 unit, 19 forms, 4 golden, 15 groups, 5 page, 11 reports, 20 units, 1 doctest; all pass |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | 0 | 28, 68 and 1 passed |
| `cargo run --locked -q -p raf-census --example baseline` | 0 | 182 lines, empty stderr. Byte-equal to the two JSON blocks in C Examples (the C test also asserts this) |
| `cargo run --locked -q -p cargo-workplan -- --root . check` | 0 | clean |
| Probe in the scratch clone (`Normalizer`) | 0 | confirmed F2 and F4 below, and that the C Examples pair (`total`/`add_up`) gets one body fingerprint (`body:72820dab3c80098a`, 33 and 35 structural tokens) |

## Part 1: T6 checks that hold

- `work/changes/census-normal-form.md` is deleted, and the INVENTORY row is `delivered` (13 delivered, 8 designed).
- `work/changes/census-ledger.md` no longer holds the job-matching item and still holds "A normal_form mismatch fails before any job is read".
- `.deficiencies` holds only the notice and the marker.
- The added lines of the whole diff contain no tracker numbers, process words, agent names or attribution.
- P has no history narration. The "Seven copies" paragraph in C Explanation is pre-existing rationale.
- C bindings: every public item and method of the crate root is bound. 96 tests (the 96 test functions in the crate) and 2 examples are bound. The verification matrix is present. `workplan check` is clean.
- C Examples equal the real example output.
- The CHANGELOG, AGENTS and README status lines are true. The README table follows the `raf-workplan` "delivered for the capabilities listed" pattern.
- The version gate, options, report members, closed schemas, ordering, hex-table forms, shared constants, job classification, thresholds, `VALUE_ADAPTERS` and `WEAK_KEYWORDS`, the encoding, the `shim_key` rule and the `Self::` rule were each checked against code and are true as written.
- `tests/page.rs` drift reasoning:
  - A code constant, pass id, level id, adapter, weak keyword, or the encoding id changed without the page fails, because the page is searched for the exact `| \`NAME\` | value |` row, the ordered pass sentence, the level rows and the quoted names.
  - A report member added or removed in code fails `the_member_lists_are_the_closed_schemas`: the required-field deserialise fails, or `deny_unknown_fields` rejects the extra key.
  - Limits, none blocking: the check is one-directional (a page-only extra adapter passes), `BASELINE`/`INDEX` are hand lists that the closed-schema JSON does not force to follow a code change, and the unit kinds, ordering and shim predicate are not asserted. That matches the Rev 2 R11 wording, so it is a note and not a finding.

## Findings

1. **P:235-237 and C:583 say a constant or static is "fingerprinted" or has "fingerprints"; it has none.** In `units/mod.rs` `build`, `print` is `None` for `Const` and `Static`, so there is no body or shim fingerprint. The initialiser only feeds the integer constants, strings and tables. The index shows `body: null, shim: null, tokens: 0` for `demo::BIG`, and P:301 says `tokens` is 0 for a non-function, so P contradicts itself. A reader would expect const bodies to group.
   Required fix: P Units last bullet and C Constraints bullet at line 583. Say that a constant or static has no print and no fingerprint, and that its integer constants, strings and tables are read from its initialiser expression only.

2. **P:38-39 "Comments and whitespace are not tokens" is false for doc comments.** The probe `/// doc\nlet x = f(a); x` yields `# [ $1 = " doc" ] let ...` in the value form. A doc comment inside a body lexes to `#[doc = "..."]` tokens, so it changes the fingerprint, and the word `doc` is renamed as a local (`$1`).
   Required fix: state in P Token source that plain `//` and `/* */` comments are not tokens, and that a doc comment inside a body is the attribute tokens `# [ doc = "text" ]`, with `doc` renamed like a local. This describes version 1. It is not a code change, which would bump `normal_form`. Add a fixture or pair that pins the behaviour.

3. **P:234 and C:287 define "shipping" as "any target that reaches it ships".** The code uses `Workspace::is_shipping`: the item must be in a shipping target and not be test code (`item.test == TestRole::None`). The `Unit.shipping` doc states this, and the test `test_code_of_a_shipping_target_does_not_ship` pins it, but the page and the Contract omit it. A `#[cfg(test)]` item of the lib target is not shipping.
   Required fix: reword both to "any reach of it is in a shipping target and is not test code".

4. **P:143-144 and C:253 describe the shim candidate as "no `let`, no `;` before its tail".** `normalize/shim.rs::candidate` inspects the top level only. The probe `g(a, |x| { let y = x; y })` is a shim candidate and gets a shim fingerprint, as does a body with an inner `;`.
   Required fix: add "at the top level of the body" to both, or "one expression whose outermost level has no `let` and no `;`".

5. **R1 has no load-level demonstration.** The plan's R1 check is "`Normalizer` called from `units::collect` on a `MemoryTree` workspace". The macro-argument collapse (`macro_arguments`, `macro_inline_capture*`) is pinned only through `Normalizer` registry pairs. Nothing in `units.rs`, `groups.rs` or `reports.rs` loads a workspace holding two `format!("{x:02x}", x = b)` bodies of at least 20 tokens and shows them in one body group. The tower is shared, so the risk is low, but the contract names that route.
   Required fix: add one `groups.rs` test over a `MemoryTree` with two renamed-local `format!` bodies at or above `min_tokens` that group, plus a neighbour with a different macro argument that does not.

6. **The Contract and Constraints assessments are not sound as recorded, and none of the six has its section digest refreshed.** `assessment:census-normal-form-contract` and `-constraints` are `accepted` (by agent), but their sections contain the falsehoods in F1, F3 and F4. Purpose, Usage, Examples and Verification are sound against their sections: every named test exists, and the Examples are byte-equal to the output.
   Required fix: after F1 to F4 and F7, re-run `make docs-sync`, re-read the Contract and Constraints sections, and record fresh acceptances, which changes their digests. Keep `tests/docs_contract.rs` counts at 62 assessments and 64 records.

7. **Advisory, to fix with F1 to F4.**
   - C:263 and P:177 say `[census] min_tokens` sets the threshold. No code in this crate reads a ledger, and the setter is `CensusOptions::min_tokens`. Say "the ledger's `[census] min_tokens`, which `CensusOptions::min_tokens` carries".
   - C:567-568 says a macro whose arguments are not valid Rust tokens "is compared by the tokens proc_macro2 yields". Tokens that do not lex are an `Error::Source` (P:54-56). Reword to say that macro arguments which lex are compared as tokens and ones that do not lex are an error.
   - P:114-115 and C:246 say the value form drops "`ref` bindings". Code also drops the `mut` of `ref mut` (probe: `let ref mut z = a; z` gives `let $1 = $0 ; $1`). Say "`ref` and `ref mut`".

## R1-R18 demonstration through the public API

| R | Demonstrated by |
|---|---|
| R1 | `forms`: registry pairs `macro_arguments`, `macro_inline_capture`, `macro_inline_capture_local`, `macro_unknown_name_kept`, `macro_positional_kept`, `macro_escaped_brace_kept` and the above-tier pairs, via `Normalizer`. Through `Census::load` it is only indirect (own-crates load). **Gap: F5.** |
| R2 | pairs `structural_renamed_locals`, `_params`, `_lifetimes`, `_different_function`, `_different_method`, `_different_field`, `_different_type`, `_local_type_declared`, `_tuple_field_kept` (`every_registered_pair_compares_as_registered`); `units::parameters_of_every_pattern_are_renamed` |
| R3 | `value_*` pairs (borrow, `&mut`, `&&`, deref, `ref`, `ref mut`, the 11 adapters, adapter with argument kept, full vs partial range, `let` annotations, type vs const and bool turbofish, `m[&k]`, binary `&`, `*`, `&&` and compound assignment kept); `the_value_adapters_are_the_eleven_of_the_page`; `the_value_form_keeps_binary_operators_and_drops_borrows` |
| R4 | `tier_25`, `tier_29`, `tier_30`, `tier_35` and the kind, float, big-int, suffix, tuple-index and range-end variants; `the_literal_tier_boundary_is_thirty_structural_tokens`; `below_the_tier_...`; `a_fixture_named_for_a_size_has_that_many_structural_tokens` |
| R5 | `shim_5`, `shim_6`, `shim_29`, `shim_30` and the `shim_*` pairs; `shim_forms_exist_for_six_to_twenty_nine_tokens`; `units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders`; `a_self_call_at_any_depth_removes_the_shim_key`; `a_turbofish_call_is_a_forwarder_...`; `groups::shim_groups_use_the_shim_key_and_skip_trait_impls_and_self_forwarders`; `a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group` |
| R6 | `the_digest_is_blake3_of_length_prefixed_tokens`; `a_fingerprint_displays_and_parses_back`; `a_fingerprint_refuses_every_malformed_spelling`; `fingerprints_sort_by_kind_then_value` |
| R7 | `units::each_kind_yields_one_unit_with_every_field`; `every_spelling_of_an_integer_is_one_constant`; `strings_carry_their_absolute_lines`; `byte_strings_and_c_strings_are_strings`; `hex_tables_are_read_from_strings_and_arrays`; `shipping_is_true_when_any_reach_ships_...`; `test_code_of_a_shipping_target_does_not_ship`; `a_module_declared_by_a_library_and_a_binary_is_one_unit`; `a_support_module_shared_by_two_test_binaries_...`; `file_context_names_package_path_and_shipping` |
| R8 | `groups::groups_of_one_are_dropped_and_members_follow_symbol_order`; `groups_order_by_count_then_tokens_then_fingerprint` (count, tokens, and a pair tied on both ordered by fingerprint); `bodies_under_min_tokens_never_group`; collision tests are crate-internal in `group::tests`, with forged digests as planned |
| R9 | `every_form_of_a_hex_table_is_reported_in_both_cases`; `near_misses_of_a_hex_table_are_not_reported`; `constants_above_ffff_in_two_packages_are_shared`; `a_constant_in_one_package_is_not_shared`; `shared_constants_run_by_value_ascending`; `units::a_constant_whose_type_names_a_number_does_not_hold_it` |
| R10 | `reports::the_example_output_is_the_pinned_golden` (exact JSON of both reports); `both_reports_read_back_what_they_write`; `an_unknown_key_missing_key_or_wrong_type_is_not_a_report`; `the_index_renders_the_classified_matches_of_every_job`; `a_shared_constant_is_hexadecimal_text`; the `baseline` example run |
| R11 | `page.rs` (5 tests) plus `docs_contract`; the page text is checked above and F1 to F4 and F7 are page falsehoods or imprecisions |
| R12 | `golden.rs`: header comment present, `every_fixture_fingerprint_is_pinned`, `the_manifest_is_pinned`, `the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry`, `the_pinned_version_is_the_implemented_one`; the `goldens` example |
| R13 | the named registry pairs listed under R2 to R4, checked by `every_registered_pair_compares_as_registered` and `the_tower_is_monotone_over_every_registry_pair` |
| R14 | `reports::two_fresh_loads_of_the_fixtures_give_identical_bytes` and `two_fresh_loads_of_this_repository_give_identical_bytes` (a real `DiskTree` of `crates`); `an_absolute_directory_reports_the_same_as_the_relative_spelling`; the example output is deterministic (the example was run once, and the pinned golden is compared byte for byte) |
| R15 | `cargo workplan check` exit 0; record deleted; INVENTORY `delivered`; docs-contract tests pass |
| R16 | `version::tests::*` (0 and 2 refused naming both versions and `cargo census migrate`, 1 accepted); `golden::the_pinned_version_is_the_implemented_one` |
| R17 | as R15, and the bindings name 96 tests, 2 examples and the full public surface |
| R18 | `crates/raf-census/Cargo.toml`: `raf-source = { path = "../raf-source", ... }` |

Plan decisions checked as delivered with no stub or silent narrowing:

- Job matching: `Job`, `JobForbidden`, `match_jobs`; the home, entry point, variant, copy and constant-by-value fixture and the empty-match job.
- Version gate (`check_normal_form`).
- Shim candidate, and the shim print kept beside the body print.
- Tower and `explain`; the manifest golden.
- Hashed encoding.
- Units by source identity.
- `syn` token sources for parameters and initialisers.
- Fail-hard source errors, each with a valid neighbour (`sig::tests`, `forms::an_unlexable_slice_...`).
- `Census::load` with `CensusOptions` (`min_tokens` 0 refused, `max_file_bytes` at and over the cap).
- Inline format captures.
- Lexical edges, with `lexical_nesting_128`.
- `Fingerprint` as an enum, with accept and refuse cases.
- Collisions.
- Ordering.
- Neighbours (near-miss hex tables, `::<3>`, `x[1..]`).
- Reports with closed schemas.
- Golden registry.
- Example without an argument.
- `FileCtx`.
- `Group`, `member_symbols` and `Census::unit`.

The built-in example shows no `Variant` or entry-point class and no shared constant, because it has one package. The `groups.rs` job tests do cover Variant, entry point and shared constants.

## Why BLOCKED

F1 to F4 are factual errors in the normative page and the Contract and Constraints text, and the task requires every sentence to be true of the code. F6 follows from them, because the accepted assessments sit on unfixed text. F5 is a missing demonstration of a named requirement on its named route. All are small documentation and test fixes. No code defect and no failed gate was found.
