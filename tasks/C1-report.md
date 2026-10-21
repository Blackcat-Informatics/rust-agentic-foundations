# C1 report: normaliser corrections

Commit: see `git log -1` on branch paudley/3-raf-census-core-normal-form-v1 (not pushed).
NORMAL_FORM stays 1; the goldens were regenerated once.

## Gap, change, test

| Gap | Change | Tests |
| --- | --- | --- |
| GAP-01 rename only bound names | `normalize/binders.rs` collects the bound names (let, if let, while let, for, match arms, closure and local fn parameters, local fn and type names, labels, lifetimes, macro metavariables, named macro arguments). `Namer` in `passes.rs` renames a name only when it is bound and the position is a use (not a field, method, lifetime of another namespace, path tail or macro name); every other identifier is kept. | Registry: `binder_*` (one pair per binder form, Equal(LiteralTier)), `binder_call_position`, `binder_shadows_an_item`, `item_function_passed_by_name`, `item_predicate_passed_by_name`, `item_lowercase_static` (Distinct); `groups.rs`: `bodies_that_apply_different_functions_share_no_group_but_renamed_locals_do`, `bodies_that_read_different_lowercase_statics_share_no_group` |
| GAP-02 head block is not a struct literal | `block_mask` marks the first brace group after an `if`/`while`/`match`/`for` head; `braces_hold_fields` never opens field position there. Used by `rename_params` and `rename_locals`. | Registry: `head_block_*` (Equal(LiteralTier)), `item_in_struct_shorthand_field`, `item_field_name_kept_beside_a_local` (Distinct); `groups.rs`: `a_block_after_a_constant_in_a_head_is_a_block_and_its_locals_are_renamed`, `a_real_struct_literal_keeps_its_field_names` |
| GAP-03 first-appearance numbering | `rename_locals` numbers shim-level `$n` parameters and bound names on one counter by first appearance (format placeholders included, `{$n}` handled); shim level keeps signature order. | Registry: `numbering_used_trailing_parameter`, `numbering_parameter_order`, `numbering_placeholder_first_appearance`, `shim_parameter_order`; `forms.rs`: `parameters_and_locals_are_numbered_together_by_first_appearance`, `copies_with_reordered_parameters_share_a_body_print_and_forwarders_keep_their_order`; `groups.rs`: `copies_that_reorder_their_parameters_share_a_body_group`, `forwarders_keep_their_argument_order_in_the_shim_key` |
| GAP-04 thresholds count the value form | `passes::held_len` counts the value form of the literal-tier form without building it; `Tower` decides the literal tier on it before `kind_literals`; `Tower::structural_tokens` is `Tower::value_tokens`; docs of the three threshold constants, `Print`, `Group::tokens`, `Baseline*`/`IndexUnit` tokens, `CensusOptions::min_tokens` corrected. | Registry: `threshold_literal_tier_straddle` (Equal(Value), 21/21), `threshold_min_tokens_straddle` (Equal(Value), 15/15); `forms.rs`: `every_threshold_counts_the_value_form` (tier, min_tokens at 10/15/16/20, shim bounds), `a_fixture_named_for_a_size_has_that_many_value_tokens`; `units.rs`: `min_tokens_counts_the_value_form_so_erased_noise_never_straddles_it`; `groups.rs`: `copies_that_differ_by_erased_noise_group_across_the_literal_tier`; `page.rs`: `the_page_states_that_every_threshold_counts_the_value_form` |
| GAP-07 rule text | CONTRIBUTING.md and AGENTS.md carry one rule: a version is fixed by the content of its first merge to the default branch; after that any change that can move a fingerprint is a new version with a migration path. Page Version-bump and Golden-test rules and the golden.rs header say the same. CLAUDE.md untouched. | `repo_gates` |

Registry expectations changed by the rules (not loosened): `shim_only_own_params_renamed` is now Distinct (a
name no one binds is an item); `shim_parameter_order` is Equal(LiteralTier) and the shim prints differ (asserted in
`forms.rs`); `numbering_used_trailing_parameter` is Equal(LiteralTier); `numbering_placeholder_parameter_above_tier`
is Equal(Structural) (the literal is abstracted); `lexical_nesting_128` uses parameters; `shim_method_turbofish`
is `x.convert::<u8>(1, 2)` (8 value tokens).

Note on a byte-identical pair: `count_a(xs, k)` / `count_b(k, xs)` with the same body text is equal at Verbatim and
differs at Shim (signature order), so it cannot sit in the monotone registry. It is proven at the print level in
`forms.rs` and through `Census::load` in `groups.rs`.

## Documents

`docs/architecture/normal-form.md` (Decision restored to "by order of first appearance", rename rule, numbering,
field names kept, head block, threshold count, version rule) and `docs/capabilities/census-normal-form.md`
(bindings, contract, constraints, verification, generated matrix via `docs sync`). The baseline example output did
not change (its bodies and shims hash the same), so the Examples already equal the real output. Assessments
purpose, contract, examples, constraints and verification re-read and accepted (agent).

## Checks (real toolchain cargo, exit status)

* `cargo test -p raf-census --locked`: 0
* `cargo run --locked -q -p raf-census --example goldens`: 0
* `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`: 0
* `make fmt clippy doc reuse issue-refs`: 0
* `cargo run --locked -q -p cargo-workplan -- --root . check`: 0; `docs sync` second run: no change

Environment note: the Stage cargo wrapper returns 1 while a build directory exists under
`.stage/.../scratch/rdfprobe/target` (written by a concurrent RDF probe), so the checks above were run with the
toolchain's own cargo first on PATH.

## Review fixes (separate commit)

| Finding | Change | Test |
| --- | --- | --- |
| F1 monotone law | Law qualified to slices with one parameter list in the page, `normalize/mod.rs` and the capability; `Divergence::equal_from` is the finest level from which every coarser level is equal. | `forms.rs` `explain_reports_reordered_parameters_as_equal_from_the_literal_tier` (equal_from LiteralTier, differs_through Shim) |
| F2 `name @ pattern` | Registry pairs `binder_at_binding_match_arm` and `binder_at_binding_if_let`, Equal(LiteralTier); goldens regenerated; capability lists the form. | `every_registered_pair_compares_as_registered`, golden test |
| F3 wrapping | CHANGELOG and the shim-candidate bullet rewrapped; `docs sync` run twice, no change on the second. | docs gates |

Assessments purpose, contract, usage, examples, constraints, verification re-read and accepted.

Checks (toolchain cargo, exit status): `make fmt clippy doc reuse issue-refs` 0; `cargo test -p raf-census --locked` 0;
`cargo run --locked -q -p raf-census --example goldens` 0; `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` 0;
`cargo run --locked -q -p cargo-workplan -- --root . check` 0.
