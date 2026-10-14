# Completeness review: implementation against plan and issue (raf-okf + raf-source)

## Scope

- Branch `paudley/2-shared-crates-raf-okf-raf-source`, commits `2c41082..6c744af` (13 commits,
  3f8697b .. 6c744af), clean working tree.
- Inputs read: `issue.md` (every scope checkbox and acceptance criterion), `plan.md` (user
  decisions 1-5, sections A-D, completeness contract, Tasks 1-10), `validation.md`,
  `delivery-checklist.md`, `tasks/T1..T9-review.md`, `reviews/*.md`, `raw/make-check.log`,
  `docs/capabilities/{okf-profile,source-index}.md`, `docs/architecture/okf-profile.md`,
  `AGENTS.md`, `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, PR template, `Cargo.toml`,
  `crates/*/Cargo.toml`, `crates/*/src`, `crates/*/tests`, `tests/docs_contract.rs`,
  `tests/repo_gates.rs`, `tests/support/mod.rs`, `fuzz/`, `Makefile`, `.github/workflows/ci.yaml`,
  `deny.toml`, `.deficiencies`.
- Method: reading, grep and `git` only. No cargo or make was run (a concurrent `make check` was
  in progress). Gate results are taken from `raw/make-check.log`.

## Evidence

### Full qualification

`raw/make-check.log` (mtime 10:47, written after HEAD 6c744af was committed at 10:45:36, clean
tree) runs every `check` target in order: fmt (both manifests), clippy, test, test-release, doc,
deny (`advisories ok, bans ok, licenses ok, sources ok`), reuse (201/201), msrv (workspace and
fuzz manifest on `rust-validate 1.98.1`), fuzz (fmt, clippy, deny on the fuzz manifest, then
`Done 5000 runs`), cross-check (both targets), licenses, issue-refs; final line `EXIT=0`.
`validation.md` still lists `make check` as "pending (Task 10)" (finding 1).

### 1. Issue checkboxes and acceptance criteria

| Issue item | Status | Evidence |
|---|---|---|
| Parse Markdown + YAML frontmatter; `type` required | delivered | `Document::parse`; `refusals::a_document_without_a_fence_on_line_one_is_refused`, `a_frontmatter_that_is_not_a_block_mapping_is_refused`; `profile::a_missing_type_key_is_the_only_violation` |
| `title`, `description`, `tags` (sorted), `timestamp` per base OKF | delivered, changed by decision 1 (tags/timestamp optional) | `schema::tags_and_timestamp_are_optional_and_every_other_key_is_required`; `render::tags_render_sorted_and_stay_sorted`, `unsorted_tags_render_sorted_and_the_document_equals_its_rendering_read_back`; `profile::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339`; `schema::malformed_and_out_of_range_timestamps_are_refused`; architecture table rows 46-48, 66-68 |
| `resource` | changed by decision 1: rejected | `profile::the_key_resource_has_its_own_rule_on_every_type`; `usage_docs::the_examples_block_validates_between_fences_and_resource_is_rejected`; architecture page line 26 |
| Profiles: closed key set per type, typed values, unknown key a hard error | delivered | `profile::an_unknown_key_is_reported_at_that_key`, `every_required_key_of_every_type_is_reported_when_missing`, `valid_documents_of_each_type_validate_to_typed_values` (`RafDocument`) |
| Deterministic writer, round-trip stable, sorted where the spec says | delivered | `render::every_tracked_document_renders_to_its_own_bytes`, `a_parsed_document_equals_its_rendering_read_back`, `keys_out_of_order_render_in_table_order_and_then_stay_put` |
| Body model: H2 sections with byte spans, links, fences, generated and PROPOSED blocks | delivered | `body_structure::sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading`; `links::every_link_form_is_read_with_its_target_anchor_and_line`; `body_structure::a_fence_closes_only_on_the_same_character_with_at_least_the_same_length`; `markers::generated_blocks_report_kind_outer_and_content_spans`, `a_proposed_block_reports_slug_spans_content_and_section` |
| `index.md` without frontmatter is navigation | delivered (as `docs/capabilities/index.md`, the only index the profile defines) | `index::only_the_capability_index_path_is_the_index`, `a_generated_only_index_is_accepted_with_either_header_form`, `document_parse_of_index_text_fails_at_line_one`; `tests/docs_contract.rs:275` routes it through `Index::parse` |
| `Tree` trait (`read`, `list`), `Disk` and in-memory `Memory` | delivered (plus `entry`) | `crates/raf-source/src/tree/{mod,disk,memory}.rs`; `memory_tree.rs`, `disk_tree_containment.rs`, `disk_tree_root::disk_and_memory_trees_agree_on_regular_files` |
| Package discovery under configurable roots "(default `crates/`)" | changed by decision 2: explicit roots, no default | `Workspace::load(tree, roots)`; `discovery_targets::there_are_no_default_roots`, `only_the_named_roots_are_searched`; Contract line 226-227 |
| lib/bin/build.rs roots | delivered | `discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag`, `each_target_kind_with_explicit_paths_resolves_beside_its_default_neighbour` |
| mod-tree as rustc, including `#[path]` | delivered | 40 tests in `module_tree.rs`, e.g. `a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory`, `a_path_attribute_inside_an_inline_module_is_relative_to_the_module_directory`, `a_module_with_both_name_rs_and_name_mod_rs_is_an_error_naming_both_files` |
| `#[cfg(test)]`/`#[test]` tagged, not dropped | delivered | `module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone`, `a_cfg_test_module_tags_every_item_reached_only_through_it`; `symbol_index::test_roles_follow_the_module_tree_and_the_item` |
| Test/bench/example targets non-shipping at `<pkg>::test_crate::<target>` etc. | delivered | `discovery_targets::each_target_kind_with_default_paths_...`; `target_roots::every_target_kind_resolves_at_its_crate_path_as_a_module_item_of_its_root_file` |
| Symbol index of every named item plus `use`-alias table, re-exports resolve | delivered | `symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility`; `use_resolution::a_three_hop_chain_resolves_and_lists_each_hop`, `a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap` |
| Port (no purrdf deps) with the purrdf `Tree` trait | delivered as the superset port | raf-source deps are `syn`, `quote`, `toml`, `proc-macro2`, unix-only `libc`; see section 3 |
| `#[path]` cannot escape the workspace root | delivered, per-platform handle check by decision 3 | `module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it` (recording tree, zero outside reads); `workspace_containment.rs`; `src/tree/disk/tests.rs` verifier and swap tests |
| AC: Memory-tree fixtures: nested mods, `#[path]`, re-export chains, test/bench/example targets, rejected `#[path]` escape | delivered | `module_tree.rs`, `use_resolution.rs`, `discovery_targets.rs`, `target_roots.rs` all build `MemoryTree`; escape test above |
| AC: OKF round-trip goldens; unknown-key fixture fails | delivered | `render::every_tracked_document_renders_to_its_own_bytes` (42 tracked documents; count assertion `> 20`); `profile::an_unknown_key_is_reported_at_that_key`; docs_contract `rejects_unknown_key` |
| AC: public APIs documented, `missing_docs` deny | delivered | `Cargo.toml` `[workspace.lints.rust] missing_docs = "deny"`; both members `[lints] workspace = true`; `repo_gates::lints_every_workspace_manifest_inherits_the_workspace_lints` with negative `lints_rejects_a_member_manifest_that_does_not_inherit`; `make doc` with `-Dwarnings` passed |
| AC: `cap:okf-profile`, `cap:source-index` match the build, state set | delivered (`delivered`) | both frontmatters `state: delivered` with non-empty code and tests bindings; `docs_contract::repository_bindings_resolve_in_the_source_index`; change records `work/changes/{okf-profile,source-index}.md` deleted after 376baf9 |
| Decision 4: split links refused | delivered | `links::a_link_split_across_lines_is_recorded_and_not_read`; `profile::a_split_link_is_reported_at_its_line_and_a_one_line_link_is_not`; `docs_contract::rejects_a_link_split_across_lines_in_any_tracked_markdown_file`, `accepts_one_line_links_and_links_in_fences_beside_the_split_rule`; architecture page line 195 |

### 2. Completeness-contract rows -> executed test or gate

Every test name in `delivery-checklist.md` (322 snake-case identifiers extracted mechanically)
was grepped as `fn <name>` across `crates/` and `tests/`: all exist except
`no_io_in_source`, which is the file name of the suite, not a function. No test is
`#[ignore]`d other than `corpus_replay::write_corpus`, a generator, not a check. Per row:

| Row | Test or gate |
|---|---|
| Parse with positioned errors | `refusals::a_document_without_a_fence_on_line_one_is_refused`, `layout_errors_are_refused_with_their_position` |
| SPDX/comments kept | `render::every_tracked_document_renders_to_its_own_bytes`, `comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line` |
| Bounds | `refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused`, `nesting_over_thirty_two_levels_is_refused_before_descent`, `a_hundred_thousand_levels_return_an_error_on_a_small_stack`, `anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token`, `a_billion_laughs_document_is_refused_at_the_first_anchor`; `alloc_bound::refused_documents_stay_within_eight_times_their_size`; `body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused` |
| Writer, key order, `replace_spans` | `render::keys_out_of_order_render_in_table_order_and_then_stay_put`; `splice::two_generated_blocks_change_only_their_byte_ranges`, `a_second_call_uses_spans_read_from_the_rebuilt_model`, `overlapping_spans_are_refused_and_the_body_is_unchanged` |
| Subset refusals | `refusals::a_flow_mapping_value_is_refused_naming_the_key` |
| Body model | `body_structure`, `links`, `markers` suites (names in section 1) |
| Body model without frontmatter | `body_structure::markdown_without_frontmatter_is_the_whole_body`; `markers::document_parse_builds_its_body_through_the_body_model` |
| Marker errors with lines | `markers::every_misspelling_of_the_proposed_marker_is_refused_with_its_line`, `each_generated_marker_error_names_its_line` |
| `index.md` navigation | `index::*` (8 tests) |
| Profiles | `profile::*` (value, key and three-fault tests) |
| `tags`/`timestamp` | `render::tags_render_sorted_and_stay_sorted`; `profile::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339` |
| Body rules per type | `profile::a_capability_holds_the_seven_sections_in_order`, `a_change_needs_a_non_empty_summary`, `an_architecture_body_is_not_empty`, `a_proposed_block_is_valid_only_inside_a_section_of_a_capability` |
| raf-okf no file/process | `no_io_in_source::the_crate_sources_name_neither_the_file_system_nor_the_process_module` (+ negative fixture in the same file) |
| `Tree` disk/memory | `memory_tree::*`, `disk_tree_containment::*`, `fs_confinement::only_the_disk_backend_names_the_file_system_module` |
| Canonical containment | `disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted`, `a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file`, `a_directory_symlink_loop_ends_with_an_error_and_is_listed_but_never_entered` |
| 8 MiB cap, manifest cap | `read_cap::*`; `discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads`, `deeply_nested_inline_tables_are_a_typed_error_on_the_callers_stack_and_a_shallow_one_loads` |
| Pre-scan budgets | `prescan_budgets::*` (budget values match `src/budget.rs` table: 128/64/256/128/256, module depth 128, module walks 262144) |
| 64 MiB named parse thread | `overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack`; `PARSE_STACK_BYTES = 64 * 1024 * 1024` |
| Overflow fixtures, harness | `overflow_harness::composites_and_uncounted_shapes_end_in_exactly_one_allowed_outcome`, `a_deliberate_overflow_aborts_and_the_runtime_names_the_parse_thread`; `make test-release` |
| Fuzz target, corpus, replay | `fuzz/fuzz_targets/parse_source.rs`; `corpus_replay::*` (3 tests + judge negative); `make fuzz` (`Done 5000 runs`) |
| Discovery and targets | `discovery_targets::*` |
| Mod tree | `module_tree::*` |
| cfg(test) | `module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone`, `an_unparseable_cfg_is_an_error_naming_file_and_line_beside_a_parseable_one` |
| Symbol index | `symbol_index::*` |
| Use-alias table and typed failures | `use_resolution::*`; `docs_contract::a_binding_to_a_macro_generated_function_is_not_found` |
| Determinism, disk = memory | `symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order`; `index_disk::disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions` |
| Usage/Examples compile | `docs_contract::usage_and_examples_blocks_are_the_crate_doctests` + negative `a_block_that_differs_by_one_line_is_reported`; crate doctests |
| `missing_docs` deny | workspace lint + `repo_gates` lint-inheritance tests; `make clippy`/`make doc` |
| raf-okf the one reader | `tests/docs_contract.rs` imports `raf_okf::{Document, Body, Profile, Index, is_capability_index, ...}`; no private YAML, fence or marker parser remains (grep of its `fn` names); old 114 tests all present except two renamed (`comment_stripping_keeps_visible_text` -> `comments_hide_text_but_not_the_text_beside_them`, `parses_quoted_and_block_forms` -> `accepts_quoted_and_block_forms`), 137 now |
| Bindings never name a missing symbol | `docs_contract::repository_bindings_resolve_in_the_source_index` (with a non-empty guard), `a_binding_to_a_missing_path_is_reported_with_its_capability`, `every_unresolved_binding_of_every_list_is_reported` |
| Docs delivered, records deleted, stale text gone | grep for "not yet implemented", "Once implemented", "waits for the source index", "will enforce": only AGENTS.md:54 and README rows for the four still-designed crates remain, which is correct |
| Handle containment per platform | `src/tree/disk/tests.rs` (`linux_verifier_*`, `unix_verifier_*`, `portable_verifier_*`, `swap::*`); `make cross-check` in `check`, CI job `cross-check`, named in Makefile header, AGENTS.md section 3, CONTRIBUTING.md, PR template, CHANGELOG Gates entry |
| Split links refused | see section 1 |
| Full qualification | `raw/make-check.log` `EXIT=0` at HEAD |

### 3. Superset port, census payload excluded

`grep -nwE 'Unit|BodyPrint|function_unit|constant_unit|relex|normalize|scan_rules|EXCLUDED_DIRS|is_excluded|is_outside_or_generated|PathEscapes|is_test_only' crates/raf-source/src`:
- `facts.rs:108` / `item.rs:140,183`: `FieldShape::Unit` (unit struct shape), not the census `Unit`.
- `members.rs:219,245,287`: `is_excluded` is the closure applying Cargo's `[workspace] exclude`
  patterns, not the census directory exclusion.
- No `BodyPrint`, `relex`, `normalize`, `scan_rules`, `EXCLUDED_DIRS`, `PathEscapes`, `keep`.

Best-of parts present: purrdf link scanner fixes (`links::an_escaped_closing_parenthesis_belongs_to_the_destination`, `a_space_between_the_text_and_the_parenthesis_is_not_a_link`, `only_ascii_space_and_controls_end_a_destination_not_unicode_space`); docs_contract multi-line HTML tag state (`links::an_html_tag_split_across_lines_yields_its_attributes_with_their_lines`, `tag_state_does_not_survive_a_blank_line_or_a_fence`); raw identifiers (`module_tree.rs:371-384`, `use_resolution::raw_identifiers_resolve_under_both_spellings`); handle check without stat-before-open and with a portable non-unix verifier (`disk.rs` `verify_linux_link`, `verify_same_file`, `verify_portable`); E0761 both-files error; `cfg_attr` path alternatives; cfg tagged never dropped.

### 4. Fail-hard sweep (`crates/*/src`)

| Hit | Judgement |
|---|---|
| `.ok()`, `filter_map(Result::ok)`, `or_insert`, `unwrap_or_default` | none |
| `target.rs:214` `unwrap_or("src/lib.rs")`, `:219` `unwrap_or_else(package name)` | modelled: Cargo's default lib path and name; a missing explicit lib file is an error (`target.rs:224`) |
| `index.rs:329` `renamed_from.unwrap_or(&fact.name)` | modelled: `extern crate x` without rename |
| `workspace.rs:542` `Rc::unwrap_or_clone` | not a fallback |
| `impls.rs:283`, `prescan.rs:249`, `facts.rs:547` `_ => {}` | modelled: plain first segment; tokens that no counter tracks; attributes that change nothing in the index (kept in facts) |
| `settle.rs:374`, `impls.rs:204` `let UseKind::Glob .. else { continue }` | modelled: filter to glob declarations |
| `settle.rs:378` `let Ok(targets) = glob_targets(..) else { continue }`, `settle.rs:606`, `resolve.rs:387` `Err(SuperAboveRoot) => {}` | modelled: `super` above the root is refused for every import and impl path before settlement (`settle.rs:299 super_check`, `impls.rs:392 super_check_impl`), and a glob of an unindexed crate brings in nothing, as the Contract states (dependencies are not indexed) |
| `impls.rs:83` `Err(Missing | SuperAboveRoot) => continue` | modelled: a self-type path that names nothing yields no local type; the inherent impl then fails `InherentImplSelf`; SuperAboveRoot is pre-checked |
| `impls.rs:79` visited set over alias walks | modelled: ends an alias cycle; the empty result becomes a typed error, not a silent pass |
| `index.rs:504` `seen` skip | modelled: identical facts for one (file, target, module path) reached twice by a `cfg_attr` alternative; nothing is lost |
| `members.rs:266` (`workspace = true` dependency without a path), `:273` (path dependency outside the tree) | modelled: neither is a workspace member; other `PathError`s are typed errors |
| `manifest.rs:201,205,340` | modelled: absent table; a dependency in string form is not a path dependency |
| `workspace.rs:115` | modelled: a virtual manifest has no package (documented) |
| `error.rs:121` `let _ = error; false` | modelled: platforms with neither unix nor windows report a loop as `Io` |
| `#[allow]` in src | two `clippy::too_many_arguments`, each with `reason` |

No silent path found.

### 5. Process wording in committed files

Added lines of `git diff 2c41082..HEAD` (lockfiles excluded, 30 186 lines) and all 13 commit
messages grepped for `#[0-9]+`, `issue`, `task N`, `plan`, `katamari`, `purrdf`, `gmeow`,
`ported`, `L1-L14`, `C-/I-/O-/S-/A-/F-/R-N`, `ACn`, `regression in`, `TODO`, `FIXME`,
`follow-up`, `deferr`: the only hits are the gate name `issue-refs` (Makefile, PR template,
CHANGELOG, CONTRIBUTING) and the word "reported" in a commit body. No attribution trailers; all
commits authored by the user.

### 6. Docs vs build

Spot-checked numbers (1 MiB, depth 32, nesting 64, link part 4096, 8 MiB default, budget table,
64 MiB stack), the `Workspace::load(tree, roots)` signature and no-default-roots wording, the
per-platform containment bullets against `disk.rs`, crate-root items (Contract 332-341, including
the `items_in`/`source_key` sentence the T7 follow-up asked for), Usage/Examples blocks (equality
gated by docs_contract), and the trait-key, prelude and workspace-membership text (lines
355-410) against `impls.rs`, `members.rs` and `trait_keys.rs`. Delivered public items not in the
issue (trait keys, `items_under`, `items_by_source`, `same_source`, `test_fns`,
`LoadErrorKind::UnresolvedTrait`) are described in the Contract. Nothing described is missing.

### 7. `.deficiencies`

The file holds the notice and `--- ENTRIES BELOW THIS LINE ---` as its last line; no entries;
not touched by the branch.

## Findings

1. **non-blocking** — `validation.md` "Full qualification" still reads `make check | pending
   (Task 10)`, while `raw/make-check.log` shows a full run at HEAD 6c744af ending `EXIT=0`. Record
   the result (or the result of the concurrent run, if it differs) before the PR is created, so
   the PR body cites recorded evidence.
2. **non-blocking** — Plan section A lists `tests/support/mod.rs:287-288` (`cfg(not(unix))`
   skip) among "precedent sites removed". The port honours this (the raf-source `DiskTree` has a
   portable verifier and no non-unix skip), but the root test helper still carries the skip at
   `tests/support/mod.rs:287-288`: on a non-unix host `check_handle` does not compare the inspected
   file with the opened handle. All gates run on Linux, so no gate is weakened today; the plan
   wording is satisfied under the "not ported" reading, which section D's row ("Disk handle
   check ... no non-unix skip") supports. Either remove the skip on this branch or state that
   reading in the PR body; it is not a deferral candidate.
3. **non-blocking** — `render::every_tracked_document_renders_to_its_own_bytes` asserts
   `paths.len() > 20` while 42 documents are tracked; the plan asked for "a count assertion". A
   lower bound still catches an empty list, which is its purpose (already recorded in
   `validation.md` open findings).
4. **non-blocking** — `LoadErrorKind::Spawn` is exercised only through its `Display` and
   `source()` (`error.rs` unit test); the failure to start the parse thread is not provoked. The
   checklist states this; the PR should too.

No blocking finding: every issue item is delivered or changed by a recorded user decision, every
completeness row has an executed test or gate, and no requirement is stubbed or deferred.

## Residual risks for the PR body

- Issue deviations governed by user decisions: `resource` is rejected and `tags`/`timestamp`
  are optional (decision 1); `raf-source` has no default roots, and an empty roots list loads an
  empty workspace without error, so each consumer must name its roots and guard against an empty
  index (decision 2; docs_contract does so); containment is closed on Linux only, with a residual
  rename race on other platforms (decision 3); links split across lines are violations
  (decision 4). Quote the issue lines and decisions.
- The non-Linux verifiers are compiled by `make cross-check` and unit-tested on Linux, but no
  test executes on macOS or Windows.
- No pre-scan budget counts the composite shapes or most uncounted shapes; such a file can end
  in a stack-overflow abort of the process (documented in the Contract, Constraints and
  SECURITY.md). Consumers that load untrusted source should run the load in a child process.
- The crate sets no time bound on a load; budgets bound recursion and module walks, not time.
- Imports and globs that never resolve are not load errors (dependencies are not indexed), so a
  misspelled `use` path surfaces only as `NotFound` when a consumer resolves through it.
- Licence exceptions: `unicode-ident` `Unicode-3.0` in `deny.toml` (crate-scoped) and the NCSA
  exception in `fuzz/deny.toml`.
- The bounded fuzz run is 5000 runs at `max_len 4096`; larger inputs are judged only by the
  corpus replay in child processes.
- Scope beyond the issue text: canonical trait keys (preludes by edition, dependency tables,
  Cargo workspace membership with `members`/`exclude` patterns) were added to key trait-impl
  methods; they are documented and tested (`trait_keys.rs`) but widen the surface that manifests
  can reach.
- Spawn failure of the parse thread is covered by its report only (finding 4).

VERDICT: PASS
