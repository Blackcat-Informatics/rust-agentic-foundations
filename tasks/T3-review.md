# Task 3 review: raf-okf profile

Reviewer: independent, read-only. Staged diff on top of e94cd8d (10 files, +2997/-36): `profile/{mod,typed,violation,body_rules}.rs`, `lib.rs`, `tests/profile.rs`, `tests/usage_docs.rs`, the two docs, `work/changes/okf-profile.md`.

## Scope

Plan: user decisions 1 and 4, section A, B6, B7, the "Parity decisions carried from the Task 2 review" Task 3 item (placeholder scan also covers frontmatter scalar values), completeness rows including 3, and "Task 3" with its Docs line.

Read in full: `profile/{mod,typed,violation,body_rules}.rs`, the schema table keys, the `lib.rs` diff, `usage_docs.rs`, the Task 3 tests listed below, and the docs diffs. Compared line by line with `tests/docs_contract.rs` 535-1010 (`check_keys`, `check_scalars`, `check_id`, `check_section_layout`, `check_binding_contents`, `check_capability`, `check_change`, `check_affects`, `check_items`, `check_architecture`, `check_document`, `scan_markers` per-document part) and the placeholder scan (`normalise_prose`, `has_word`, `scan_placeholders`).

## Evidence

| Command | Result |
|---|---|
| `cargo test -p raf-okf --locked` | no failures (orchestrator: 160 passed) |
| `cargo test -p raf-okf --locked --doc` | 1 passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo fmt --all --check` | clean |
| `RUSTDOCFLAGS=-Dwarnings cargo doc -p raf-okf --no-deps --locked` | clean |
| grep of added src lines for `.ok()`, `unwrap_or`, `else { continue`, `_ => {}`, `.unwrap()`, `expect(`, `unsafe`, `allow(` | no hits (one `expect` inside a rustdoc example) |
| grep of added lines for plan IDs, `task N`, `decision`, `plan`, precedent names, tracker numbers | no hits |

## 1. B6 rules against tests (all in `tests/profile.rs`; each asserts `at` and `rule` via `assert_hit`, beside a `clean(...)` or neighbouring accepted case)

| Rule | Test |
|---|---|
| closed key set, unknown key | `an_unknown_key_is_reported_at_that_key` |
| missing required key, every key of every type | `every_required_key_of_every_type_is_reported_when_missing` |
| missing `type` (only violation) | `a_missing_type_key_is_the_only_violation` |
| `resource` own rule, every type, message names key set | `the_key_resource_has_its_own_rule_on_every_type`, and `usage_docs::the_examples_block_validates_between_fences_and_resource_is_rejected` |
| unknown `type` (only violation) | `an_unknown_type_is_one_violation_at_the_type_key` |
| type vs directory (nested, other dir, bare, wrong extension, empty stem; `./` prefix accepted) | `a_document_must_sit_directly_in_the_directory_of_its_type` |
| id prefix, kebab slug, equals file stem | `an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name` |
| non-empty title and description | `a_title_and_a_description_are_non_empty_strings` |
| state enum | `a_state_is_one_of_the_four` |
| bindings exact keys, each a list | `bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list` |
| bindings by state (designed and considered empty; delivered needs code and tests) | `bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests`, `a_delivered_capability_with_code_and_tests_is_typed_with_its_bindings` |
| depends as cap ids, missing target is not a violation | `depends_is_a_list_of_cap_ids_and_does_not_need_the_capability_to_exist` |
| changes non-empty cap ids | `changes_is_a_non_empty_list_of_cap_ids` |
| affects format, section set, slug in changes | `affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes` |
| contract_effect | `contract_effect_is_none_or_changes` |
| items: claim, unknown key, outcome, moved | `items_need_a_claim_known_keys_and_a_known_outcome`, `the_three_outcomes_are_typed` |
| optional tags (empty, duplicate, shape) and timestamp (range, malformed, shape); absent is valid; sorted tags typed | `tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339` |
| seven sections in order (missing, unexpected, out of order, duplicated) | `a_capability_holds_the_seven_sections_in_order` |
| section empty after comments and generated blocks | `a_section_is_empty_when_only_comments_and_generated_blocks_remain` (six empty forms, prose beside block/comment and a fenced block accepted) |
| Change `## Summary` | `a_change_needs_a_non_empty_summary` |
| Architecture body non-empty | `an_architecture_body_is_not_empty` |
| PROPOSED only in a Capability, only inside a section | `a_proposed_block_is_valid_only_inside_a_section_of_a_capability` |
| placeholder in prose at its line; spared in code, longer words, dotted prose; frontmatter values at the key | `placeholder_tokens_in_prose_are_reported_at_their_line`, `placeholder_scan_spares_code_longer_words_and_dotted_prose`, `placeholder_tokens_in_frontmatter_values_are_reported_at_the_key` |
| future tense is not a violation | `future_tense_prose_is_not_a_violation` |
| `Rule::SplitLink` at its line, one-line link accepted | `a_split_link_is_reported_at_its_line_and_a_one_line_link_is_not` |
| every violation reported | `a_document_with_three_faults_returns_three_violations` (frontmatter and body mix); `every_required_key_of_every_type_is_reported_when_missing` |
| Display from `Rule` | `a_violation_displays_the_file_the_place_and_the_rule` |
| typed fields | `valid_documents_of_each_type_validate_to_typed_values` (State, Bindings, Affects{capability, section}, ChangeItem{claim, outcome: Option<Outcome>}, Outcome::{Implemented, Dropped, Moved(slug)}, Header incl. tags/timestamp) |

All present. Typed fields match B6. Validation reads the one schema table: `check_key_set` and `check_values` iterate `doc_type.keys()` and dispatch on `ValueKind`; required/optional comes from `KeySpec.required`. The seven section names have a single list (`CapabilitySection::ALL`) used by affects and the body rule. The key-order dependence (`state` before `bindings`, `changes` before `affects`) is satisfied by the table order.

The `build` invariant (a clean check always yields a typed document) holds: every `?` in `build` corresponds to a required key whose absence or fault already added a violation, and optional keys use `Slot`.

## 2. Parity with docs_contract 593-990 and the placeholder scan

Reproduced: `check_keys` (unknown, missing, tags list/non-empty/duplicate, timestamp), `check_scalars`, `check_id` (prefix, kebab, stem), `check_section_layout` (missing, unexpected, duplicate, order, empty), `check_binding_contents`, state, bindings shape and keys, depends, changes, affects (format, section, in-changes), contract_effect (including non-scalar), items (unknown key, empty claim, outcome incl. `moved:change:<slug>`, shape), Summary, architecture emptiness, missing and unknown `type`, type-vs-directory in both directions (collapsed into `WrongDirectory`; nested paths covered), PROPOSED placement, `normalise_prose` and `has_word` (same algorithm), code-span-aware prose view.

Intended differences, all plan-mandated and documented: section emptiness also ignores generated blocks; one violation per frontmatter fault with typed `Rule`; frontmatter values (title, description, claims) are now scanned for placeholders, as the carried parity decision requires.

Cross-document checks (`check_cross_references`, `find_cycles`, state/PROPOSED correspondence) are out of scope and the Contract says so.

## 3. B7

- The `lib.rs` crate doctest is the Usage block: `usage_docs::the_usage_block_is_the_crate_doctest` asserts byte equality of the extracted `rust` block of the Usage section and the doctest, and that it has no `std::fs`. It uses a `const TEXT`, so no file access. The doctest passes.
- Examples: `bindings` is in block form (tested to have no `{`); the block validates as exact text inside `---` fences with a seven-section body at `docs/capabilities/example.md`; with `resource:` appended it gives exactly one violation at `Key("resource")`, `Rule::ResourceKey { Capability }`, and the message names every key of the Capability table.
- The "one-line-difference negative fixture" of the doctest-equality test is assigned in the plan to the docs_contract test (Task 8), not here.

## 4. Docs

- `cap:okf-profile#explanation` is added to `affects` in `work/changes/okf-profile.md`.
- Contract "Validate" is present tense, lists every rule and matches the code (including "a `type` ... is the only violation of that document"). The cross-document statement (ids existing, cycles belong to the document model; one document at a time) is consistent with the tests (`depends_is_a_list_of_cap_ids_and_does_not_need_the_capability_to_exist`).
- Examples (block form), Usage prose (span edit through `document.replace_spans`), and the Explanation sentence (`tags` and `timestamp` optional, everything else required) are accurate. The architecture page keeps the optional-key table and rules from the earlier task, and the emptiness sentence now reads "HTML comments and generated blocks".

## 5. Cross-cutting

No silent paths in `profile/`. The remaining `match ... None =>` and `map_or` uses either build a message (`"<not a scalar>"`) or are total path-string helpers (`file_stem`, `is_directly_in`). `lists[index]` and `chars[before]` are bounds-safe by construction. `body_rules::visible_content` slices stay inside the body (content spans lie in the body; generated blocks filtered to the section and ordered). No `unsafe`, no `allow`, no labels or precedent names. `no_io_in_source` still passes.

## Findings (all non-blocking)

- N1. The placeholder rule over title, description and item claims is stated in the Contract "Validate" but not on the architecture page, whose sentence still reads "No prose carries a placeholder token". The Contract says `raf-okf` adds no rule of its own; one clause on the architecture page ("... in a title, a description or an item claim") would make that exact. This preserves what docs_contract scanned in the whole file text, so it is not a new refusal.
- N2. Frontmatter placeholder scan does not remove code spans (`check_frontmatter_placeholders` passes the raw scalar). A title like ``Use `<token>` here`` is now flagged; the old whole-file scan spared it through `outside_code`. Rare, and arguably consistent with "values are text"; say so or strip spans.
- N3. `affects` is checked against `found.changes`, which is `None` when any `changes` entry is invalid, so every `affects` entry then also reports `AffectsNotInChanges` (over-reporting; the old checker kept the valid ids). Cosmetic.
- N4. Contract "Validate" lists the unknown-`type` early return but not that a missing `type` is also the only violation (tested).
- N5. Order of violations is rule order, not source order; fine, but consumers that print them should not assume sorting.

## Unchecked

- docs_contract (117) and repo_gates (42) over the staged files and `make reuse`, `make msrv`: taken from the orchestrator's report.
- Task 8 migration itself.

VERDICT: PASS

## Recheck

Evidence: `cargo test -p raf-okf --locked` no failures; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0; `cargo fmt --all --check` clean.

- N1 DISCHARGED. The architecture Body rules sentence now says the placeholder rule covers `title`, `description` and item claims, with code spans exempt.
- N2 DISCHARGED. The frontmatter scan strips code spans; `a_placeholder_token_in_a_code_span_of_a_frontmatter_value_is_spared` pins the bare value flagged beside the code-span value accepted, for a title and a claim (double-backtick form).
- N3 DISCHARGED. `an_invalid_changes_entry_does_not_cascade_into_the_affects_entries` shows one violation (the bad id) and, with an affects entry naming an unlisted capability, exactly two.
- N4 DISCHARGED. Contract Validate states that a missing or unknown `type` is the only violation for that document.
- N5 was an observation and needed no change.

VERDICT: PASS
