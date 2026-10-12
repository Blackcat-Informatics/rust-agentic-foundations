---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:source-index
title: "Source index"
description: "The raf-source crate: a Tree abstraction, package discovery, mod-tree following with #[path], and a symbol and use-alias index."
state: delivered
bindings:
  code:
    - raf_source::Tree
    - raf_source::DiskTree
    - raf_source::MemoryTree
    - raf_source::RelPath
    - raf_source::Workspace
    - raf_source::parse_source
    - raf_source::Resolution
    - raf_source::LoadOptions
    - raf_source::Item
    - raf_source::Target
    - raf_source::FileId
    - raf_source::Usage
    - raf_source::Budget
    - raf_source::Opened
    - raf_source::PARSE_BYTES_PER_TOKEN
  tests:
    - raf_source::test_crate::relpath::a_parent_segment_above_the_root_is_refused_beside_the_one_that_stays_inside
    - raf_source::test_crate::relpath::an_absolute_input_is_refused_beside_its_relative_form
    - raf_source::test_crate::memory_tree::duplicate_keys_are_refused_literally_and_after_normalisation_beside_distinct_keys
    - raf_source::test_crate::memory_tree::keys_that_leave_the_root_are_refused_with_outside_root_beside_a_valid_key
    - raf_source::test_crate::disk_tree_containment::a_symlink_to_a_file_outside_is_refused_beside_an_in_root_symlink_that_is_accepted
    - raf_source::test_crate::disk_tree_containment::a_parent_segment_after_an_outside_directory_symlink_is_resolved_on_disk_and_refused
    - raf_source::test_crate::disk_tree_containment::a_directory_symlink_loop_ends_with_an_error_and_is_listed_but_never_entered
    - raf_source::test_crate::disk_tree_containment::a_fifo_named_as_a_file_is_refused_without_blocking_beside_a_regular_file
    - raf_source::test_crate::workspace_containment::a_symlinked_module_file_to_outside_is_refused_naming_the_module_beside_an_in_root_link
    - raf_source::test_crate::workspace_containment::a_manifest_target_path_that_leaves_the_root_is_refused_naming_the_key
    - raf_source::test_crate::workspace_containment::discovery_classifies_a_symlink_by_what_it_resolves_to
    - raf_source::test_crate::discovery_targets::each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag
    - raf_source::test_crate::discovery_targets::two_packages_with_one_name_are_an_error_naming_both_manifests
    - raf_source::test_crate::discovery_targets::a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads
    - raf_source::test_crate::module_tree::a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory
    - raf_source::test_crate::module_tree::every_cfg_attr_path_alternative_is_followed_carrying_its_predicate
    - raf_source::test_crate::module_tree::a_module_cycle_is_an_error_naming_both_files
    - raf_source::test_crate::module_tree::a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it
    - raf_source::test_crate::module_tree::the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone
    - raf_source::test_crate::module_tree::a_chain_of_files_that_each_declare_the_next_twice_is_refused_quickly_beside_a_small_diamond
    - raf_source::test_crate::module_tree::a_child_file_that_sorts_before_its_mod_rs_parent_is_placed_under_its_own_path
    - raf_source::test_crate::module_tree::an_item_is_found_only_under_its_own_module_path
    - raf_source::test_crate::symbol_index::every_kind_is_indexed_with_its_kind_line_file_and_visibility
    - raf_source::test_crate::symbol_index::cfg_twins_are_one_symbol_with_two_sorted_sites
    - raf_source::test_crate::symbol_index::test_functions_carry_the_package_target_and_crate_relative_name
    - raf_source::test_crate::symbol_index::two_loads_are_equal_and_items_and_targets_come_in_strictly_increasing_order
    - raf_source::test_crate::use_resolution::a_three_hop_chain_resolves_and_lists_each_hop
    - raf_source::test_crate::use_resolution::a_cycle_of_imports_is_cyclic_beside_a_chain_that_ends
    - raf_source::test_crate::use_resolution::missing_names_dependency_items_and_macro_generated_items_are_not_found
    - raf_source::test_crate::use_resolution::resolution_does_not_depend_on_the_order_of_the_declarations
    - raf_source::test_crate::use_resolution::a_long_chain_of_re_exports_resolves_in_either_order_without_a_cap
    - raf_source::test_crate::trait_keys::each_prelude_trait_loads_in_the_edition_that_has_it_and_is_refused_in_the_one_before
    - raf_source::test_crate::trait_keys::inheriting_requires_membership_by_the_members_and_exclude_lists
    - raf_source::test_crate::target_roots::every_target_kind_resolves_at_its_crate_path_as_a_module_item_of_its_root_file
    - raf_source::test_crate::target_roots::a_first_party_module_at_a_binary_crate_path_is_ambiguous_beside_a_binary_that_is_not
    - raf_source::test_crate::prescan_budgets::delimiter_nesting_is_refused_at_one_hundred_twenty_nine_beside_one_hundred_twenty_eight
    - raf_source::test_crate::prescan_budgets::five_hundred_nested_generics_are_refused_beside_sixty_four
    - raf_source::test_crate::prescan_budgets::twenty_thousand_else_if_links_are_refused_beside_two_hundred_fifty_six
    - raf_source::test_crate::prescan_budgets::a_syntax_error_reports_its_own_line_and_column_beside_a_parsing_neighbour
    - raf_source::test_crate::overflow_harness::composites_and_uncounted_shapes_end_in_exactly_one_allowed_outcome
    - raf_source::test_crate::overflow_harness::the_parse_thread_is_named_after_the_file_and_has_the_stated_stack
    - raf_source::test_crate::overflow_harness::the_classifier_accepts_only_a_signal_a_stack_overflow_status_or_the_runtime_message_as_an_abort
    - raf_source::test_crate::corpus_replay::every_corpus_file_ends_in_exactly_one_allowed_outcome_with_its_recorded_symbols
    - raf_source::test_crate::read_cap::disk_tree_enforces_the_cap_at_eight_mib
    - raf_source::test_crate::read_cap::the_default_cap_is_eight_mib
    - raf_source::test_crate::index_disk::disk_and_memory_trees_give_equal_workspaces_and_equal_resolutions
    - raf_source::test_crate::tree_error::every_variant_names_its_path_in_display_and_in_path
    - raf_source::test_crate::fs_confinement::only_the_disk_backend_names_the_file_system_module
    - raf_source::test_crate::this_repository::this_repository_loads_from_its_root_with_the_crates_directory_as_the_root
    - raf_source::test_crate::this_repository::an_item_of_a_module_file_that_sorts_before_its_mod_rs_is_found_under_its_own_module
    - raf_source::tree::disk::tests::linux_verifier_refuses_a_handle_link_that_leads_outside_and_accepts_one_inside
    - raf_source::tree::disk::tests::unix_verifier_refuses_differing_identities_and_accepts_equal_ones
    - raf_source::tree::disk::tests::portable_verifier_refuses_a_non_regular_handle_and_an_outside_path_beside_an_accepted_one
    - raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused
    - raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused
    - raf_source::tree::disk::tests::a_missing_descriptor_link_is_an_io_failure_and_never_absence
    - raf_source::error::tests::a_spawn_error_displays_the_file_and_carries_its_cause
    - raf_source::item::tests::an_item_record_is_at_most_two_hundred_fifty_six_bytes
    - raf_source::facts::tests::a_column_is_turned_into_its_byte_offset_on_short_long_and_wide_lines
    - raf_source::settle::tests::a_binding_list_stays_sorted_keeps_the_better_route_and_takes_only_its_room
    - raf_source::test_crate::load_budgets::a_file_at_the_token_limit_parses_and_one_more_token_is_refused_naming_its_line
    - raf_source::test_crate::load_budgets::each_costly_shape_at_the_token_limit_loads_and_one_more_token_is_refused
    - raf_source::test_crate::load_budgets::a_doc_comment_counts_as_the_tokens_of_its_attribute_and_other_comments_count_nothing
    - raf_source::test_crate::load_budgets::each_large_fact_payload_is_charged_and_refused_at_its_line_once_over_the_fact_budget
    - raf_source::test_crate::load_budgets::the_fact_budget_counts_over_every_file_of_the_load
    - raf_source::test_crate::load_budgets::index_records_at_the_item_budget_load_and_one_byte_less_is_refused
    - raf_source::test_crate::load_budgets::a_long_module_name_is_charged_in_every_item_path_and_refused_over_the_item_budget
    - raf_source::test_crate::load_budgets::a_file_reached_by_many_module_paths_is_charged_once_per_path_and_refused_over_the_item_budget
    - raf_source::test_crate::load_budgets::the_bytes_read_at_the_total_budget_load_and_one_byte_less_is_refused_naming_the_file
    - raf_source::test_crate::load_budgets::the_default_budgets_are_the_stated_limits
    - raf_source::test_crate::load_budgets::usage_reports_what_a_load_spent
    - raf_source::test_crate::load_budgets::a_long_alias_path_in_a_file_reached_many_times_is_held_and_charged_once
    - raf_source::test_crate::load_budgets::a_long_extern_crate_name_in_a_file_reached_many_times_is_held_and_charged_once
    - raf_source::test_crate::load_budgets::a_chain_of_files_that_each_declare_the_next_twice_is_charged_what_it_keeps
    - raf_source::test_crate::load_budgets::nested_inline_modules_whose_scopes_hold_one_name_are_charged_what_they_keep
    - raf_source::cost::tests::a_map_of_one_entry_is_covered_by_what_a_map_costs_once
    - raf_source::cost::tests::a_list_is_charged_at_least_its_room
    - raf_source::cost::tests::a_hash_table_is_charged_at_least_its_buckets
    - raf_source::test_crate::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen
    - raf_source::test_crate::disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen
    - raf_source::test_crate::walk_stack::a_memory_chain_of_twenty_thousand_directories_is_refused_typed_on_a_two_mib_thread
    - raf_source::test_crate::walk_stack::a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused
    - raf_source::test_crate::walk_stack::a_member_pattern_at_the_depth_limit_expands_and_one_deeper_is_refused_typed
    - raf_source::test_crate::walk_stack::one_hundred_twenty_eight_nested_modules_load_on_a_five_hundred_twelve_kib_thread
    - raf_source::test_crate::walk_stack::a_disk_chain_of_directories_is_refused_typed_on_a_two_mib_thread
    - raf_source::test_crate::workspace_containment::symlink_aliases_of_one_directory_parse_it_once
    - raf_source::test_crate::workspace_containment::a_file_reached_through_a_directory_symlink_is_one_file_under_the_spelling_that_sorts_first
    - raf_source::test_crate::workspace_containment::a_module_that_reaches_its_own_file_through_a_directory_symlink_is_a_cycle
    - raf_source::test_crate::module_tree::a_file_reached_under_two_spellings_is_read_once_and_keeps_the_spelling_that_sorts_first
    - raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses
    - raf_source::test_crate::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle
    - raf_source::test_crate::symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets_in_linear_time
  examples: []
depends: []
---

# Source index

## Purpose

`raf-source` answers one question for the other crates: does this Rust path name a
real item, and where is it defined. The census resolves ledger homes with it and the
workplan resolves code and test bindings with it, so one walker and one index serve
both and cannot disagree.

## Contract

**Tree.** All file access goes through the `Tree` trait, which takes `RelPath` values:
validated root-relative `/`-separated UTF-8 paths. `RelPath::join` drops `.` segments
and refuses an absolute input, a backslash, a NUL, a drive-letter segment and a `..`
that climbs above the root with a typed `PathError`. A `..` below the root is kept,
because on a disk it means the parent of the directory the earlier segments really
reach. The trait has four calls, each returning a `Result`: `read(path, max_bytes)`
returns the text of a file; `read_unseen(path, max_bytes, seen)` opens a file once,
takes its `FileId` from that open and reads its text from the same open unless `seen`
accepts the identity (`identity(path)` is the same call with every identity seen);
`list(path)` returns the entries of a directory sorted by name, each with a kind of
file, directory, symlink or other; and `entry(path)` returns the kind of one path or
`None` when nothing is there. Two paths have equal identities when they name one file:
`DiskTree` takes the identity from the handle it opened and verified, its device and
inode numbers on Unix and its canonical path below the root elsewhere, and
`MemoryTree` uses the normalised key. A device and inode pair names one file where the
file system keeps inode numbers unique and stable while the tree is read, as local
file systems do; a network or user-space mount that reuses or invents them can make
two files look like one. Because the identity and the text come from one open, no
change to the tree can pair one file's identity with another file's text.
`read_unseen` refuses what `read` refuses, and a file it does not read is not refused
for its size or encoding. The trait is object-safe and has no `Send` or `Sync` bound.
A failure is a `TreeError` that names the path: `NotFound`, `OutsideRoot`,
`NotRegular`, `SymlinkLoop`, `TooLarge`, `NotUtf8`, `NonUtf8Name`, `DuplicatePath` or
`Io`. Only `NotFound` means that nothing is there; an entry whose name is not UTF-8
fails the listing and is never dropped from it. `DiskTree::new(root)` returns a
`Result`: it canonicalises the root once, a missing root is `NotFound`, and a
symlinked root is followed to its target. `MemoryTree` holds a map of path to text,
infers directories from the keys, and exists so that fixtures build repositories
without touching the disk. It enforces `max_bytes` like the disk backend and refuses a
key such as `../a` or `/a` with `OutsideRoot`, and a key that names a path another key
already holds, whether literally or after normalisation, with `DuplicatePath`; a tree
built by the infallible conversions from such keys returns the same error from every
call. A missing `/proc` on Linux is an `Io` failure and never `NotFound`. `entry`
returns `OutsideRoot` when the directory that holds the path leads outside the root.
Nothing else in the crate opens a file.

**Path containment.** Every path the crate opens is checked against the workspace
root, so a hostile tree cannot make the index read a file outside it. This covers
files reached by `mod name;` with or without `#[path]`, explicit `path` values of
`Cargo.toml` targets, and every discovered `Cargo.toml`. For each candidate
`DiskTree` canonicalises the resolved path, which follows every symlink and applies
every `..` against the real directories, and requires the canonical root as a prefix
of the result. A candidate outside the root is an error that names the module,
attribute or manifest key, and it is neither opened nor read. A lexical check of
`..` segments is not enough, because `link/..` leaves the root when `link` is a
directory symlink, so `DiskTree` never clamps a `..` in text. A symlink whose target
is inside the root is followed. `DiskTree` refuses any file that is not a regular
file, such as a FIFO or a device, and it does not descend into a directory symlink:
a listing reports it with the symlink kind, listing it is refused, and a symlink
loop ends with `SymlinkLoop` at the first link. The file type is checked on the
opened handle and not by a stat before the open: the crate opens the canonical path
non-blocking and without following a final symlink, so opening a FIFO cannot hang,
and then reads the type from the handle and refuses anything that is not a regular
file. A listing opens the canonical directory the same way (as a directory, without
following a final symlink), verifies that handle as reads do, and reads the entries
through the handle, so a swap between the check and the listing cannot list a
directory outside the root. The opened handle is checked against the root, and the
guarantee depends on the platform:

- On Linux the kernel's own path for the handle (`/proc/self/fd/<fd>`) must lie
  under the canonical root. The check reads what the handle really is, so no check
  is separated from the open it guards: a directory swapped for a symlink to
  outside the root after the prefix check is refused.
- On other Unix systems the path is canonicalised again after the open and must lie
  under the root and name the file the handle is (same device and inode). A rename
  that wins the race against the second canonicalisation can still redirect the
  path, but not the handle, and the identity comparison refuses a handle that is not
  the file the path names.
- On other platforms the handle must be a regular file and the path canonicalised
  again after the open must still lie under the root. A rename that wins the race
  against the second canonicalisation is a residual window there.

**Resource bounds.** Source is parsed in-process. Each read is capped at
`max_file_bytes`, which defaults to 8 MiB and is set by `LoadOptions::max_file_bytes`
or, for the census, by `max_file_bytes` in `[census]` (see
[census-ledger](./census-ledger.md)). A larger file is an error that names it. `syn`
recurses on more than delimiters, so before any file is parsed the crate runs a token
pre-scan, one lexer pass feeding independent counters. The lexer skips comments
(block comments nest) and every string, raw string (with any number of `#`), byte
string, C string and character literal, and tells a lifetime from a character
literal, so tokens inside them count for nothing. A file that exceeds a budget is
refused with a `LoadError` of kind `Budget`, which names the file, the budget, its
limit and the line where the count went over. A file that `syn` rejects is a `Syntax`
error with the line and column of the parser's error, read on the parse thread, and a
thread that cannot start is a `Spawn` error. The budgets are one table in the source
(`Budget::limit`), and the counters are defined as follows:

- Delimiters: the number of `(`, `[` and `{` open at once; the limit is 128, so 128
  levels parse and 129 are refused.
- Nested `<`: each delimiter level has its own count; `<` adds one unless it is part
  of `<=`, and `>` subtracts one down to zero (`->` and `=>` are single tokens and
  subtract nothing). The count of a level returns to zero at `;`, `&&` and `||`, and
  the count of the enclosing level returns to zero when a `{ ... }` block closes, so
  comparisons in separate statements do not add up. The limit is 64.
- Prefix operators: the length of a run of consecutive tokens that are `!`, `-`, `*`,
  `&` or `&&` (a `&&` is one token), with only whitespace and comments between
  them. A lifetime between two operators neither ends the run nor adds to it. `!=`,
  `-=`, `*=`, `&=` and `->` are not prefix operators. The limit is 256.
- Closure bars: a closure head is `||`, or `|` up to the next `|` at the same
  delimiter level. The count is the length of a chain of heads in which each head
  follows the previous one directly, or with `move` between them. The limit is 128.
- `else if`: within one delimiter level, the number of links, where a link is an
  `else` directly after a `}` and followed by `if`. The count returns to zero at `;`,
  at an `else` that is not followed by `if`, and at an identifier other than `else`
  directly after a `}`. A block used as the condition of a link (`else if {x} {}`)
  does not end the chain. The limit is 256.
- Tokens: the number of tokens of the file as the parser's lexer reads them. An
  identifier, a keyword or a literal is one, a lifetime two (the quote and the
  name), each delimiter one and each character of punctuation one (`::` and `->`
  are two). A doc comment counts as the attribute it stands for: six tokens for
  `///` and `/**`, seven for `//!` and `/*!`; other comments count for nothing. The
  count feeds no other counter. The limit is `LoadOptions::max_file_tokens`, by
  default 2000000 (`Budget::FileTokens`), and `parse_source` applies the default.
  It bounds the memory that parsing one file takes at its peak at
  `PARSE_BYTES_PER_TOKEN`, 1664 bytes, per token. The costliest shape the other
  budgets admit is one function body of 2^k + 1 empty statements, whose list of
  statements doubles its storage for the last one: it holds 1602 bytes per token at
  its peak at every k from 14 to 20, counting a grown block as held beside the
  block it replaces until that is freed, and the constant adds about 4%. The same
  measurement gives about 430 bytes per token for `a<b; ` repeated, 290 for
  statements that name a type nested 64 generic arguments deep and 230 for a run of
  attributes. The default admits the largest generated files of real crates: over
  the registry scan below the most tokens of one file was 1505567 (a time-zone
  table).

Two more budgets are counted by the module walk and not by the pre-scan. Module
nesting is bounded at 128 levels along one path, files and inline modules together.
The walk, and the index that reads its result, keep a stack of their own, so their
depth costs the caller's stack nothing: 128 nested modules load on a thread of 512 KiB
in a debug build. Module walks are bounded at 262144 for one load: every arrival of
the walk at a file, over all targets, and every walk of an inline module body (the
default one and one per `cfg_attr` path alternative) is one module path, and a file
reached again is parsed once but walked again, so a chain of files that each declare
the next twice would otherwise record 2^n paths from a few kilobytes of input. The
limit is 2^18: a workspace of ten thousand files reached by five targets each records
fifty thousand, and a shared test-support diamond records one path per target and
module that reaches it, so the limit admits every realistic tree. What the walks
record is charged to the item-bytes budget below, which bounds what the limit costs: a
chain of seventeen files that each declare the next twice (2^18 - 1 walks from 1 KiB
of source) is refused by the item-bytes budget after 1.0 s at a peak of 214 MiB
resident, measured in a release build. A load that passes the limit is a `Budget`
error that names the file and the line of the declaration.

**Work and memory bounds.** Discovery and the expansion of `workspace.members`
patterns enter at most 128 directories below their root (`DirectoryDepth`), on a stack
of their own; a deeper directory is a `Budget` error that names it, which a chain of
20000 directories in memory meets on a thread of 2 MiB in a debug build, in under a
millisecond in a release build, and a chain of 2100 on disk in 22 ms with 4 MiB
resident. Three byte budgets bound one load. Each is a default that `LoadOptions` can
change:

- `max_total_bytes` (`TotalBytes`) bounds the bytes the load reads, source files
  and manifests together. Its default is min(1 GiB, max(256 MiB, 4 × the largest
  crate)) over the newest version of each of 1177 crates of a local registry, whose
  largest load read 16.9 MB (`linux-raw-sys`), so it is 256 MiB.
- `max_item_bytes` (`ItemBytes`, 512 MiB) bounds what the index and the module walk
  allocate. Every charge is an upper bound of the allocations it covers, derived from
  the layout of the containers that hold them and not measured on typical input: a
  list is charged twice its elements and its first room of four, a `BTreeMap` a fifth
  of a leaf and an eighth of an internal node for each entry and one leaf for the
  map, a hash table three buckets for each entry and eight for the table, and a
  shared string its text and two counters. An item charges its record, its places in
  the order, in its file's list and in its source group, each string it holds the
  first time the load stores it (paths, names, module paths and written self types
  are stored once and shared), and its scope entry (the name as key, a boxed entry
  and a binding in each name space) or its place among the members of its type. A
  module charges its record, its entry among the module paths, its scope table and
  the first room of its definitions; an arrival of the module walk its reach, its
  places in the orders of arrivals and its entries in the tables that mark it
  indexed; a file read its entries among the files and identities; a spelling its
  entry and text; a flattened import its entry, its strings and its place among the
  unsettled imports, and each binding it makes its scope entry and route; each impl
  block once per module path, each trait key and each type's table of members their
  allocations; and the aliased path of a type alias and the crate an `extern crate`
  item names are stored once for the file and charged once, however many module paths
  reach it. Measured with a counting allocator, no shape keeps more than it is
  charged: a chain of files that each declare the next twice keeps 1624 bytes of index
  for each arrival and is charged 3614, nested inline modules whose scopes hold one
  name keep 1474 bytes for each module and are charged 2375. An item record is at most
  256 bytes.
- `max_fact_bytes` (`FactBytes`, 256 MiB) bounds the facts of the files. Each fact
  charges its size and the room of every string and list it owns (each is trimmed
  to its length as it is built), on the parse thread, so an oversized file is
  refused at the line of the fact that passes the budget.

A load over a budget is a `Budget` error that names the budget, the limit that
applied and the file, with the line where there is one. `Workspace::usage` reports
what a load spent against each. Two figures follow, measured in a release build:

1. Retained memory, apart from the scope tables of imports. What a load keeps of
   its index, its source text and its facts is bounded by the item bytes (512 MiB),
   the source text (`max_total_bytes`, 256 MiB by default, which a caller may raise)
   and the fact bytes (256 MiB): 1 GiB at the defaults. 300000 unit structs of their
   own names in three files load in 0.70 s at a peak of 380 MiB on the heap, and
   400000 are refused by the item-bytes budget in 1.1 s; thirty files of 4 MiB of doc
   comments made of quotes, which double when escaped, hold 253 MB of facts and load
   in 1.8 s at a peak of 382 MiB, and forty are refused by the fact-bytes budget in
   1.8 s; 31 files of 8 MiB load in 0.30 s, and 33 are refused by the total-bytes
   budget in 0.30 s. A load near all three limits at once (2000 copies of a chain of
   100 nested inline modules, 20 files of 8.3 MB doc comments and 7 files of 8 MB of
   comments; 214, 459 and 217 MiB charged) keeps 716 MiB and peaks at 721 MiB, in
   11.4 s.
2. One-file parse peak. What parsing one file holds at its peak is at most
   `max_file_tokens` × `PARSE_BYTES_PER_TOKEN` (1664 bytes): about 3.1 GiB at the
   default of 2000000 tokens. The costliest arrangement of statement lists at
   exactly 2000000 tokens loads in 0.80 s with 2623 MiB allocated at the peak, 1188
   MiB of it resident; an 8 MiB file of `a<b; ` repeated (6.7 million tokens) is
   refused by the pre-scan in 40 ms with no more allocated than its text. A caller
   on a host with less memory lowers the limit to its memory over 1664 bytes:
   `max_file_tokens = 320000` keeps the peak below 512 MiB (the costliest
   arrangement at exactly that limit loads in 0.17 s with 462 MiB allocated, 190 MiB
   resident), and refuses files above it with the same typed error.

Apart from the scope tables of imports, a load holds at most figure (1) and one
file's figure (2) at once: about 4.1 GiB at the defaults, each term a limit of
`LoadOptions` that a caller sets to fit its host. The scope tables that `use`
declarations and globs build are bounded by no budget: their size grows with the
modules that glob a module times the names it holds, so a glob-heavy input can
exhaust a machine whatever the limits. One module of 3000 functions that 3000 other
modules glob, 166 KB of source and 66005 tokens, loads in 21 s at a peak of 7.3 GiB
on the heap and 9.2 GiB resident, while every budget stays below 14 MiB.

Parsing then runs on a dedicated thread, named after the file, with a stack of 64
MiB (`PARSE_STACK_BYTES`), which is a stated constant and not the stack of the
caller. `parse_source` runs a visitor on the syntax tree on that thread and
returns what the visitor returns. A committed fuzz
target, `parse_source` in `fuzz/fuzz_targets/parse_source.rs`, feeds arbitrary bytes
through the pre-scan and the parse. Its corpus, `fuzz/corpus/parse_source/`, is
committed and holds the composite and single-shape fixtures of Verification below,
files at and just over each budget, and every input that ever aborted a run. These
measures make a stack overflow unlikely. They do not prove its absence: the budgets
multiply across nesting (a run of prefix operators is legal inside every delimiter
level), some recursive shapes are counted by no budget, and a level of a nested
`return return ...` chain costs about 8.8 KiB of stack in a debug build and about
3.7 KiB in a release build, measured. The crate does not claim that an admitted file cannot
overflow. A residual overflow aborts the process, so the run fails and is never
reported as a pass. No budget counts the committed composite shapes or the uncounted
shapes that repeat `return `, `&mut `, `*const `, `fn() -> ` or `a = ` 20000 times, so a
file of one of them can end in that abort. Shapes that try to slip past a counter (a block as the condition of each
`else if`, a lifetime between reference marks) are counted by the definitions above
and refused. A parser overflow is a robustness bug, and an input that causes
one is added to the fuzz corpus and to the budgets. `Cargo.toml` is read with the same
size cap, and the `toml` parser's own recursion limit turns deep nesting into an error
and not a crash. Every refusal the pre-scan makes is a typed error and not a panic.
The crate sets no time bound.

**Package discovery.** `Workspace::load(tree, roots)` takes explicit roots; the
crate has no default roots, and each caller names its own (`"."` is the tree root).
Discovery reads each root as a directory. A directory with a `Cargo.toml` that has a
`[package]` table is a package. The walk descends, on a stack of its own and at most
128 directories below the root, through every directory that is not
a package, and through a package that also has a `[workspace]` table, so a root `.`
package with a `[workspace]` table and `crates/a` below it are both found. It does not
descend below a `[package]` manifest that lacks `[workspace]`, so a fixture package
under a tests directory is not found. It skips directories named `target` and
dot-directories. A symlink is classified by what it resolves to: a regular file inside the root
is left alone, a directory is a `DirSymlink` error, a special file such as a FIFO is
`NotRegular`, a target outside the root is `OutsideRoot`, a dangling link is
`NotFound` and a loop is `SymlinkLoop`. None of these reads more than one byte. A manifest that is a
virtual `[workspace]` contributes nothing but its children. Two packages with one name
are an error that names both manifests, and one manifest reached from two roots is one
package. Manifests are read through `Tree` under `max_file_bytes` and parsed with the
`toml` crate, so comments, single-quoted strings and inline tables are honoured; a
manifest that is not TOML, is nested deeper than the parser allows, or has a value of
the wrong type is a `Manifest` error that names the file and, when one is at fault, the
key. Packages are sorted by manifest path.

For each package the crate derives the targets, in this order: the library, the
binaries, the tests, the benches, the examples and the build script. A target has the
package name, its Cargo name, its kind, its root file, a shipping flag and a crate
path. The library is `[lib]` or `src/lib.rs`, named by `[lib] name` or the package
name. Binaries are the `[[bin]]` tables, `src/main.rs`, `src/bin/*.rs` and
`src/bin/*/main.rs`; tests, benches and examples are their tables plus `tests/*.rs`,
`tests/*/main.rs` and the same under `benches/` and `examples/`. `autobins`,
`autotests`, `autobenches` and `autoexamples` set to false turn the inferred ones off,
and an explicit table is found either way. An explicit table without `path` takes the
default location for its name. The crate path is the crate name for the library
(`[lib] name`, or the package name, with `-` as `_`), `<pkg>::bin_crate::<bin>` for a
binary, `<pkg>::test_crate::<target>`, `<pkg>::bench_crate::<target>` and
`<pkg>::example_crate::<target>` for the others, and `<pkg>::build_script` for the
build script. Each crate path resolves to the target's root as a module item. Library,
binary and build-script targets are shipping, so the census sees
`build.rs`; tests, benches and examples are not. These are typed errors that name the manifest key: an explicit
target file that does not exist (checked with `entry`), a table without a `name`, two
targets with one root file, two targets (in one package or two) with one crate path,
which would merge their items, and a `path` that leaves the tree.

**Mod-tree following.** From each target root the crate follows module declarations
and parses each file with `syn`, as rustc resolves them. Without a `path` attribute,
`mod name;` is `name.rs` or `name/mod.rs` in the module directory; both existing is an
error that names both files. The module directory of a root file or a `mod.rs` file is
its own directory, and that of any other file `a.rs` is `a/` beside it. An inline
module `mod m { .. }` adds `m` to the module directory. `#[path = "p"]` on `mod name;`
names a file relative to the directory of the file that holds the declaration, or,
inside an inline module, relative to the module directory including the inline
components; on an inline module it replaces that module's directory component. A file
named by a `path` attribute is treated like a `mod.rs` for its own children. Two
`path` attributes that apply unconditionally are an error. A path in
`#[cfg_attr(predicate, path = "p")]` is another site that is followed, carrying its
predicate, and a nested `cfg_attr` is flattened with the combined predicate
(`all(outer, inner)`); the default file is followed too when it exists, and may be missing when
every alternative names a file. No `cfg` is evaluated and no item is dropped. A raw
identifier module `r#match` reads `match.rs`. A path that leaves the workspace root,
whether by `..` segments, an absolute path or a symlink, is an error that names the
module and the attribute and does not read the file. An unresolved `mod` is an error
that names the declaring file and its line. Files are told apart by their identity
in the tree (`Tree::identity`), not by their spelling. The walk detects a cycle on the
stack of files being followed, compared by identity, never with a visited set, and a
cycle is an error that names the file that would be entered twice, also when it is
reached again under another spelling (`l/x.rs` from `x.rs` where `l` links to `.`).
Module nesting along one path is bounded at 128 levels, files and inline modules
together. A file reached by several module paths, by two targets such as a shared
`tests/common/mod.rs`, or under several spellings (`src/b.rs` and `src/z/b.rs` where
`z` links to `src`), is read and parsed once, listed once under the spelling that
sorts first, and recorded under every path it was reached by; a `dir/..` pair is
collapsed only when `dir` is a real directory, so a spelling is one the disk can
follow, and one file has one spelling in the index. The walk stops at the first
error. `Workspace::targets` yields the targets, and `Workspace::files` yields each
source file once with its text, its facts and the module paths it was reached by. The
facts of a file are read on the parse thread and owned: items with their name line,
span, body span, attributes, visibility, documentation, test role and, for a struct, its
field shape; `mod` declarations with their raw `path` and `cfg_attr` attributes; `use`
trees; impl headers with the written self type and its leading `::`; `macro_export`;
an unnamed `const _` as an explicit kind; and the file's own inner documentation.
Documentation counts a non-blank `#[doc]` text or `#[doc = include_str!(..)]`, and
`#[doc(hidden)]` alone does not count.

**`cfg(test)` tagging.** An item has the test role `None`, `CfgTest` (guarded by a
`cfg` that implies `test`, directly or through an enclosing module) or `TestFn` (marked
`#[test]`). A predicate implies `test` by this matrix: `test` does; `all(..)` does when
any operand does; `any(..)` does when every operand does; `not(..)` and every other
predicate do not. A `cfg`, `cfg_attr`, `path` or `test` attribute of the wrong shape, and a predicate
that cannot be read, are a `CfgParse` error that names the file and the attribute's
line. A module reached through `#[cfg(test)] mod t;`, or
through a `path` that a `test` predicate sets, is recorded with the test flag on that
way of reaching it, so every item in the file is test code when all its ways in are
tests. Test items stay in the index with that tag so that a caller decides whether they
count.

**Symbol index.** Every named item of every module tree is an `Item`: a function,
struct, enum, union, trait, type alias, constant, static, module, `macro_rules!`
macro, `extern crate` item, item of an `extern` block, method of an inherent or trait
`impl` block, default and required method of a trait, and associated constant or type.
An item has an `id`, its full `path`, its `kind`, its `namespace`, its `file`, the line
of its name, its `span` and `body` in lines and bytes, its `visibility`, `documented`,
`doc_hidden`, `test` role, `target` and, for an impl member, the written `self_type`
and `trait_path`. The module path in an item's `path` is the chain of `mod`
declarations that reached its file, whatever the order of the file names. The name
spaces are type, value and macro. A function, constant or static is a value; a
module, trait, enum, union, alias and `extern crate` item is a type; a unit or tuple
struct is a type and also a value, through its field shape; a `macro_rules!` macro is
a macro, and a `#[macro_export]` macro is indexed at the crate root wherever it is
written. A name can be in all three at once. `const _` defines no
name and is not indexed; it stays in the facts of its file. A macro invocation that
defines items is not expanded, so what it generates is not indexed. `documented` is
true for a non-blank `#[doc]` or `///` text and for `#[doc = include_str!(..)]`, and
`doc(hidden)` alone does not count; a module that has its own file is documented by the
file's inner `//!` text as well. The `test` role is `CfgTest` also for an item in a file
that is reached only through test code.

Every target's crate root is also an item of the module kind, at the target's crate
path in the type name space: its file is the target's root file, its line is 1, its
span is the whole file, it is documented when the file starts with `//!` text, and it
is test code for a test or bench target. `resolve` finds it by the crate path (`demo`,
`demo::test_crate::it`, `demo::example_crate::e`, `demo::bin_crate::x`,
`demo::build_script`). The segments `test_crate`, `bench_crate`, `example_crate` and
`bin_crate` are not items: `demo::test_crate` alone names nothing unless a first-party
module has that name. A root item spans the whole file, so its source key is not
unique: it can equal the key of an item that starts at byte 0. Root items are left out
of `items_by_source`, and `items()` and `items_in(file)` include them. If a crate path
is also the path of a first-party item, `resolve` returns `Ambiguous` naming both.

A method is recorded under the path of its impl type: an inherent impl must name, after
resolving type aliases, a struct, enum or union of the workspace, or loading fails with
a typed error that names the file and line; an impl of a trait on such a type records its
members under that type, as trait impl methods; an impl of a trait on any other self
type (a path the workspace does not define, a reference, a slice, a tuple, a generic
parameter) records its members under the key of the trait followed by the method, with the
written self type and the written trait path kept on the item, and the key itself
(`TraitKey`) on the item as well. The key of a local trait is the path it is defined at.
The key of a dependency trait is its absolute path, found by following the written path
through the module's own `use` declarations and `extern crate` items, through the `use`
declarations of a local module that the path passes through (`pub mod traits { pub use
serde::Serialize; }` and a direct `serde::Serialize` are one key), and through the globs
of local modules (`use super::*` reaches what the parent imported). The head of the
resulting path must be a crate the package can name: the sysroot crates `std`, `core`,
`alloc`, `proc_macro` and `test`, or a crate declared in `[dependencies]` (for a library
or binary), in `[dependencies]` and `[dev-dependencies]` (for a test, bench or example
target, and for test code of a library or binary) or in `[build-dependencies]` (for the
build script), under the name it has in code (a renamed dependency, `package = ".."`, keys under the new
name, not the name of the package it renames) and with `-` as `_`. A path whose head is none of these is a typed
`UnresolvedTrait` error that names the file and line. `core` and `alloc` are written as
`std`, which re-exports their traits at the same paths, so the three spellings of one
trait are one key. A single-segment name that nothing in the module binds is looked up
in the prelude of the package's edition, taken from its manifest (or from
`[workspace.package]` when it inherits one). The workspace manifest is found the way
cargo finds it, by walking up from the package directory through its ancestors within
the tree: each ancestor `Cargo.toml` is read through `Tree` under `max_file_bytes` (an
ancestor without one is skipped, an unreadable or malformed one is an error), the
nearest one with a `[workspace]` table decides, as long as the package is a member of it,
as cargo requires. The members are the root package of the workspace manifest, if it has
one; the directories that match an entry of `workspace.members` and no entry of
`workspace.exclude` (an exclude entry also covers what is below it); and, transitively,
the `path` dependencies of the members, in `[dependencies]`, `[dev-dependencies]`,
`[build-dependencies]` and the `[target.*]` tables, renamed ones and `workspace = true`
ones that `[workspace.dependencies]` gives a path included, that lie inside the
workspace directory and are not excluded. A pattern may use `*`, `?` and `[...]` (with
ranges and a leading `!` or `^`) in each path segment, and is matched in time
proportional to the pattern times the name; `**`, a backslash, an unclosed bracket and an
empty segment (`a//b`, `a//`) are a typed manifest error, never ignored; one trailing `/` is
allowed (`crates/*/` is `crates/*`). An entry
of `workspace.members` that has no wildcard names one directory, which must exist and have
a `Cargo.toml`, or it is a manifest error naming `workspace.members`, unless it is
excluded, in which case it is ignored and need not exist; a glob entry skips
what it matches that is not a directory or has no `Cargo.toml`. The member manifests are read through
`Tree` under the size cap, once for each workspace manifest, and the walk is bounded by
the module-path budget; a path dependency inside the workspace whose manifest is missing
or malformed, or whose `path` cannot be a path here (a backslash, a drive letter), is a
manifest error that names the dependency key; a path that leaves the tree or is absolute is
not a member. A `[workspace.dependencies]` entry whose `path` is not a string is a manifest
error naming it. A package that inherits
from a workspace it is not a member of is a manifest error that names `package.edition`
and the workspace manifest. Reading the workspace manifest
adds no package to what discovery found, because the roots alone decide that. An inherited edition that the
nearest workspace manifest does not set, or with no workspace manifest above the
package, is a manifest error that names `package.edition`, as it is for cargo, and a
package with no `edition` key is 2015: the traits of the first prelude, plus `FromIterator`,
`TryFrom` and `TryInto` from 2021, plus `Future` and `IntoFuture` from 2024. A glob of a
dependency (`use std::fmt::*;`) cannot be looked into. If exactly one such glob in scope
could provide the name, the trait is keyed under that glob's path followed by the name, so
`use serde::*` and `use serde::ser::Serialize` can give two keys for one trait: the glob
cannot be looked into, and its path is all the key has. A glob whose first segment is
neither a local module nor a crate the package can name is a typed `UnresolvedTrait`
error that names the glob's file and line when a trait key consults it. If
several could, or the name is also in the prelude, the key is `TraitKey::Unresolved`: it
holds the written path and every absolute path the trait could have, the members are
recorded under the written path, and `resolve` finds them by that text. Members of
different impls that share a key are one symbol with several sites, and where a local
trait declares the method the path resolves to the declaration. A trait's methods are recorded
under the trait.

The identity of an item is its file and the byte offset where its span starts. A file
that several module paths reach is indexed once for each path, so the same source item
can appear as several `Item` values that share a file and offset and differ in `path`
and `target`. `items` yields every item sorted by path, then name space, file and
position, strictly increasing, so the items under a path prefix are a contiguous run
(`items_under`); `items_by_source()` and `same_source(&item)` group the items that share
one identity, so a census can count a source item once however many paths reach it;
`item(id)` is a lookup; `items_in(file)` yields a file's items in order
of position; `item_at(file, offset)` yields the innermost items that hold the offset;
`is_shipping(&item)` is true for an item of a shipping target that is not test code;
`files()` yields each source file once, with its package, the targets and module paths
that reach it, and whether it ships; `source(file)` is the exact text that was indexed;
and `test_fns()` returns, for each `#[test]` function (or a typed error if one's path does not start with its target's crate path), the package, the kind and Cargo
name of its target (`--lib`, `--bin <name>` or `--test <name>`) and its path inside the
crate, which is what `cargo test -p <package> --test <target> <name>` needs;
`usage()` gives what the load spent against its budgets: the bytes read, charged to
the index and to the facts, the most tokens of one file and the deepest directory
entered. The index
has no order that depends on the tree implementation: two loads of one tree are equal,
and the same regular files on disk and in memory give equal indexes.

**Use-alias index.** Every `use` declaration contributes to a table of the names of its
module: renames, nested groups, `self` in a group, globs, a leading `::` (a library
crate by name), `extern crate x as y`, and the keywords `crate`, `self` and `super`; a
`super` above the crate root is an error that names the file and line. A name that a
module defines or imports explicitly shadows a name that one of its globs brings in, as
in rustc. A lookup answers from the globs of a module only when every explicit import of
that module that could bind the same name is settled, resolved or definitively failed;
until then the answer is indeterminate and the asker waits, so the result does not depend
on the order of the declarations. A glob brings in only the names the importing module
can name: `pub` items, `pub(crate)`, `pub(super)` and `pub(in path)` items when the
importer is inside the restriction, and private items only when the importer is the
defining module or lies inside it; a path walk does not check visibility otherwise. The
tables are settled to a fixed point, so a chain of re-exports of any length resolves and
nothing caps the search, and a cycle of globs adds nothing. Of two routes to one item the
one that uses no glob decides `via_glob`.

`resolve(path)` takes a canonical path such as `demo::inner::go` (a segment may be
written as a raw identifier: `r#match` and `match` are the same name) and tries the type,
value and macro name spaces in that order; `resolve_in(path, namespace)` takes one. A
path may end at a method: `demo::Thing::new`. The result is a `Resolution`: the
canonical `path` of the definition, the `namespace`, the `sites` (non-empty, sorted by
path, file and position), `via_glob` (true when a glob import carried the path to the
symbol), and the `chain` of locations the lookup passed through from the requested path to the
definition. The same path, name space and kind defined more than once is one symbol with
several sites, as `#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()`. The failures are
typed: `NotFound` when the workspace defines nothing at the path (an item of a
dependency, an item that a macro generates, and a misspelling all end here),
`Ambiguous(ids)` only for a real conflict, which is two globs that bring in different
items, two imports of one name that reach different items, or two canonical targets,
`Cyclic` when the imports on the way depend on each other, and `Invalid` when the text
is not a path. The crate path of a test, bench, example or binary target begins with the
package name and a name such as `test_crate`, so a first-party module with that name
(`mod test_crate;` in the library) and a test target share a prefix: a path through
that name is `Ambiguous`, naming the module and the test target's item. A symbol that a
macro generates is not indexed, and the documentation of the capabilities that rely on
the index says so.

## Usage

Load a workspace from a real directory and ask where a path is defined. The block
needs a real tree, so it is compiled and not run:

```rust,no_run
use raf_source::{DiskTree, Workspace};

let ws = Workspace::load(&DiskTree::new(".")?, &["crates"])?;
let resolution = ws.resolve("raf_okf::Document")?;
for site in &resolution.sites {
    let item = ws.item(*site).expect("a site is an item of this workspace");
    println!("{} at {}:{}", item.path, item.file, item.name_line);
}
for target in ws.targets().filter(|t| !t.shipping) {
    println!("{} is a non-shipping target", target.name);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

A line that starts with `# ` is hidden by rustdoc and is there so that the block
compiles.

## Examples

With `MemoryTree` a fixture needs no disk:

```rust
use raf_source::{MemoryTree, Workspace};

let tree = MemoryTree::from([
    ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ("src/lib.rs", "mod inner;\npub use inner::go as run;\n"),
    ("src/inner.rs", "pub fn go() {}\n"),
]);
let ws = Workspace::load(&tree, &["."])?;
assert_eq!(ws.resolve("demo::run")?.path, "demo::inner::go");
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Constraints

- Parsing is syntactic. The crate does not expand macros, evaluate `cfg` other than
  the `test` tag, or run type inference.
- Every opened path must be, after canonicalisation, inside the canonical workspace
  root and a regular file; the same rule bounds explicit `Cargo.toml` target paths.
- A file larger than `max_file_bytes` (default 8 MiB), or one that exceeds a
  budget of the pre-scan (128 delimiter levels, 64 nested `<`, 256 prefix
  operators, 128 closure bars, 256 `else if` links, 2000000 tokens by default), or a
  module walk that passes 128 nested modules or 262144 module walks in one load, or a
  directory
  more than 128 levels below its root, or a load that passes its total, item or fact
  bytes (256 MiB, 512 MiB and 256 MiB by default), is refused with a typed error.
  Parsing runs on a dedicated 64 MiB stack. A residual overflow aborts the run and
  fails the gate; it is never a pass. No budget counts the composite shapes or most of
  the uncounted shapes of Verification, so a file of one of them can end in such an abort.
- The scope tables that `use` declarations and globs build are bounded by no
  budget, so a glob-heavy input can take more memory than the figures of the
  Contract state.
- Resolution covers items the source defines. Items from dependencies are outside the
  index and resolve to a not-found failure.
- The index is built once per `Workspace::load`; callers reload to see changed files.
- Output order is sorted by path, so two loads of the same tree give equal results.

## Explanation

A census that compares function bodies, and a workplan that checks that a documented
symbol exists, both fail in the same way when they trust text: a re-export makes the
public path differ from the defining path, and a module declared with `#[path]` is
invisible to a directory walk. Both tools need the compiler's view of the module
tree, and `syn` gives a close enough one without the cost of invoking the compiler.

The `Tree` trait is the seam that keeps the crate testable. Acceptance fixtures
for both consumers are repositories built in memory, and a disk-backed run uses the
same code path, so a fixture that passes says something about the real workspace.

Keeping test and example targets in the index, tagged non-shipping, lets one
consumer ignore them and another bind to them. Workplan bindings name tests and
examples, so those must resolve; the census must not count test helpers as shipped
duplicates.

## Verification

- A `MemoryTree` fixture per target kind (library, binary, test, bench, example and
  build script) with default and with explicit paths, asserting the kind, the Cargo
  name, the shipping flag, the crate path and that each target's root module is
  recorded under its crate path.
- Discovery fixtures: a virtual manifest contributes nothing; a manifest with a
  trailing comment and single-quoted values is honoured; no roots find nothing; a
  package nested below a package without `[workspace]` is not found beside a root
  package with `[workspace]` and a member below it; `target` and dot-directories are
  skipped beside a sibling; roots exclude what they do not name; the `auto*` flags and
  explicit tables; a library and a binary with one name; two packages with one name,
  two targets with one root and two targets with one crate path, each beside a
  distinct-name neighbour; a missing target file, a table without a name and a
  target path that leaves the tree, each naming the manifest key; a manifest of
  exactly 8 MiB loads and one byte more is refused naming the file; inline tables
  nested 1000 levels are a typed error on the caller's stack and 8 levels load.
- Mod-tree fixtures: `name.rs`, `name/mod.rs`, both present (an error naming both),
  inline and nested modules, `#[path]` in a `mod.rs` file, in a non-`mod.rs` file, inside
  inline modules in each, on an inline module in each, a `#[path]` file's children,
  `cfg_attr` alternatives with their predicates (nested ones flattened), two
  unconditional `path` attributes, raw identifiers, a cycle naming both files, a file
  reached by several paths and by two targets listed once, an unresolved `mod` naming
  the file and line, the module-depth budget at 128 and 129, a chain of files that each
  declare the next twice refused quickly beside a small diamond, and a first-party
  `mod test_crate;` beside a test target.
- Placement fixtures, each asserting every item path: a child file that sorts before
  the file that declares it (`a/mod.rs` with `a/b.rs`, and `a.rs` with `a/b.rs` below a
  directory module that sorts before the crate root); the children `a`, `ga` and `z` of
  a `#[path = "gen/generated.rs"]` file; and `tests/common/mod.rs` with
  `tests/common/a.rs` reached from `tests/zz.rs`. `super::super` from such a child
  reaches the crate root, and `demo::a::in_b` and `demo::a::b::in_a` are `NotFound`
  beside `demo::a::b::in_b`. In this repository `raf_okf::body::blocks::FenceBlock`
  resolves and `raf_okf::body::FenceBlock` is `NotFound`, and the document-contract
  test rejects a binding to the same misplaced shape.
- `#[path]` fixtures: a redirect inside the root resolves; `..` out of the root and
  an absolute path outside it both fail naming the module and the attribute, and a
  recording tree shows that the outside path is never read.
- `cfg(test)` fixtures: the predicate matrix, `#[cfg(test)] mod t;` tagging every item
  reached only through it, `#[test]` functions, a `test` predicate on a `path`, and
  unreadable or misshapen attributes.
- Bound fixtures: a file over `max_file_bytes` fails with an error that names the
  file, a lowered `max_file_bytes` is honoured, and a file of 8 MiB parses at the
  default. A file at the delimiter limit (128 levels) parses and one at 129 levels
  fails. Each of these shapes just over its budget, and the shape sizes below, fails
  with a typed error that names the file, and runs in debug and in release on the
  default 2 MiB test-thread stack of the caller, so the dedicated stack and the
  pre-scan are what keep the process alive:
  - 500 nested `A<A<...>>` generics (delimiter depth 0);
  - `!` repeated 4000 times before an expression;
  - `&` repeated 20000 times before a type;
  - `||` repeated 10000 times as a closure chain;
  - an `else if` chain of 20000 links.

  A file at each budget (64 nested `<`, 256 `!`, 128 closure bars, 256 `else if`
  links) parses on the same stack.
- Token fixtures, at a limit lowered to 100000 for speed: a file of exactly the
  limit loads and one more token is refused naming its line; `a<b; ` repeated,
  statements naming a type nested 64 generic arguments deep, a run of attributes and
  the costliest shape, one body of a power of two and one more empty statements,
  each load at the limit and are refused one token over; a doc comment counts as six
  or seven tokens and other comments as none. The default limit is 2000000,
  `parse_source` refuses one token more than it before parsing, and 320000 tokens
  at 1664 bytes each stay below 512 MiB.
- Work and memory fixtures, each at the limit, where the load succeeds, and one byte
  over, where it is a `Budget` error that names the budget and the file: the bytes
  read (a manifest counts), the index records (a long module name charged in every
  item path, a file reached by a hundred module paths charged once per path, and a
  long aliased path and a long `extern crate` name in a file reached by 256 module
  paths, each charged once and not once per path, and a chain of files that each
  declare the next twice and nested inline modules each charged at least the bytes of
  index they keep), and
  the facts, over several files and for each large payload: a 1 MiB doc comment, a 1
  MiB written self type and a `use` group of long names, each refused at its own line
  below its size. `usage()` reports the spent bytes, the most tokens and the deepest
  directory, and an item record is at most 256 bytes.
- Unit fixtures of the structures the bounds rely on: a column of a short line, a
  long line and a long line of two-byte characters, before, on and after an index
  mark, turns into its byte offset, and a column or line past the text is refused;
  the bindings of one name stay sorted, keep the better route of a repeated item and
  take only the room the first binding needs.
- Stack fixtures, each on a small thread in debug and in release: a `MemoryTree` chain
  of 20000 directories and a `DiskTree` chain of 2100, on 2 MiB, are refused with the
  directory-depth budget naming the first directory past it, beside chains at the limit
  that load; a `workspace.members` pattern at 128 levels expands and one at 129 is
  refused; 128 nested modules, inline and in files, load on 512 KiB, and 129 are
  refused typed.
- Identity fixtures: `identity` gives one value for a file, a symlink to it, a hard
  link and a path through a directory symlink, and refuses what `read` refuses;
  `read_unseen` gives the identity and the text of one open, leaves a seen file
  unread and unrefused for its size, and refuses an unseen one over the cap; a file
  reached as `z/../b.rs` and as `b.rs` is read once and listed as `src/b.rs`; on disk,
  a file reached through a directory symlink is one file under the spelling that
  sorts first, a module that reaches its own file through such a link is a cycle, and
  a tree of two links back to `src` and eighteen levels of files, which spells its
  last file 2^17 ways, reads every file once and is refused by the item-bytes budget.
- Positions: twenty thousand items on one line of two-byte characters get exact byte
  offsets within the time a linear walk takes.
- Composite and uncounted-shape fixtures: `!` repeated 256 times before `(` repeated
  across 127 levels; `&` repeated 256 times before `(` across 127 levels, in a type;
  `&&` repeated 256 times before `(` across 127 levels; `A<` followed by 256 `&`,
  repeated across 64 levels; `||` repeated 128 times before `(` across 127 levels;
  and 20000 repetitions each of `return `, `&mut `, `*const `, `fn() -> `, `a = ` and
  `move || `. A harness runs each in a child process, in debug and in release, and
  asserts that it ends in exactly one of three outcomes: the pre-scan refuses it with
  a typed error that names the file and the budget; the process exits 0 with a
  correct, complete index (the symbols of the fixture, compared with the expected
  list); or the process aborts with a non-zero status. Exit 0 with a wrong or partial
  index, or with an index produced after an abort, fails the fixture.
- Evasion fixtures: an `if` followed by 20000 `else if {true} {}` links, and
  `type T<'a> = &'a &'a ... u8;` with 20000 reference marks, each fail with the typed
  budget error that names the file, in process and in the child-process harness; the
  same shapes at 256 links and 256 marks parse.
- A syntax error carries the line and column of the parser's error: a fixture with the
  error on line 3 and another on line 5 assert the exact position, beside a file that
  parses.
- The harness judges strictly: a child that panics, one given an unknown selector, and
  one that exits 3 without a result line each fail the fixture. An abort is accepted
  only as death by a signal, the Windows stack-overflow status, or the runtime's own
  stack-overflow message.
- The parse thread is named after the file it parses, so a stack-overflow message
  from the runtime names the culprit. A fixture asserts the thread name for a file
  and that the abort message of the overflow fixtures contains that name.
- Corpus replay under `cargo test`: a test parses every file of
  `fuzz/corpus/parse_source/` in a child process and asserts that each ends in one of
  the three outcomes above, so a regression is caught without running the fuzzer.
- The fuzz target `parse_source` builds and runs for a bounded number of iterations
  over its corpus without a crash. That run cuts inputs to 4096 bytes, because an
  abort would end the fuzzer; the larger corpus files are judged by the replay test
  in child processes.
- A `cfg(test)` fixture asserting the tag on an item and on a module reached only
  through it.
- Symbol fixtures: one item of every kind, asserting kind, file, line and visibility
  (`pub`, `pub(crate)` and private); the impl shapes (inherent on a local type, inherent
  through an alias, inherent on a type that does not resolve, which is an error, trait
  impls on a local type, `String`, a blanket `T`, `&Foo`, `[u8]` and a tuple); a module,
  a function and a macro of one name selected by name space, and unit and tuple structs
  in the value name space; `#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()` as one
  symbol with two sorted sites; items that share a source item grouped through a shared
  `tests/common/mod.rs`; documentation (text, hidden alone, a blank `///`,
  `include_str!`, a module's inner docs); `files()`, `items_in()`, `item_at()` and
  `is_shipping()`; the `test_fns()` data for a library, a binary and a test target; a
  first-party `mod test_crate;` beside a test target, which makes the shared path
  ambiguous beside the same layout without it, which resolves.
- Alias fixtures: a rename, a nested group, `self` in a group, a glob with `via_glob`
  beside the direct path without it, a chain of three hops with its `chain`, a cycle
  that yields `Cyclic` beside a chain that ends, definitions and explicit imports
  shadowing globs (also for the modules that glob the importer), two globs with
  different items that are `Ambiguous` beside two that reach one, a mutual `super::*`,
  `crate`, `self` and `super` paths, `super` above the root, `extern crate demo as d`
  and `use ::demo::..`, raw identifiers in both spellings, a chain of 5000 re-exports
  declared in either order, `demo::nope`, `pub use serde::Serialize` and a function that
  a macro generates, each `NotFound` beside a positive resolution.
- Order fixtures: an explicit import that must shadow a glob for a reader declared before
  it, and the same modules declared in every order of 1440, giving one answer for each
  path, with glob against glob staying ambiguous; glob visibility (private beside public,
  a private item into a descendant, `pub(crate)`, `pub(super)`, `pub(in path)`); the
  canonical keys of trait impls in each spelling and the error for a trait path that
  cannot be made absolute; the prelude per edition, an inherited edition, one and several
  dependency globs, a head that is no known crate, dependency tables by target kind with
  renames, and a path through a local re-export of a dependency.
- A determinism check: two loads over the same tree give equal indexes, and `items()`
  and `targets()` come in strictly increasing order.
- A `DiskTree` run over a temporary directory gives the same index as the
  equivalent `MemoryTree` for trees of regular files. `MemoryTree` has no symlinks or
  special files, so this equivalence does not cover the containment cases, which have
  their own `DiskTree` fixtures below.
- `DiskTree` containment fixtures, each built in a temporary directory beside a
  sentinel file outside the root, asserting an error and that the sentinel is not
  read:
  - `src/inner.rs` is a symlink to a file outside the root, reached by a plain
    `mod inner;`;
  - `#[path = "vendored/../x.rs"]` where `vendored` is a symlink to a directory
    outside the root, so the `..` leaves the root only on disk;
  - a `Cargo.toml` target with `path = "../outside.rs"` and one with an absolute
    path outside the root;
  - a link to a symlink that itself points outside the root;
  - a directory-symlink loop, which ends with an error and does not recurse;
  - a FIFO named as a module file, which is refused without blocking on a read;
  - a missing root, which is `NotFound`, and a symlinked root, which is followed to
    its target and judged there;
  - a directory swapped for a symlink to outside the root between the prefix check
    and the open, or between the check and a listing, which is refused and leaves the
    outside file unread and the outside names unreturned;
  - per-platform handle verifiers, each compiled on every platform and run on Linux:
    a handle on a file outside the root is refused beside an in-root handle; a
    differing device and inode pair is refused beside an equal pair; and a
    non-regular handle and an outside re-canonicalised path are each refused beside
    an accepted case;
  - a symlink to a file inside the root, and a chain of such symlinks, is accepted;
  - a chain that ends outside the root, a symlinked `Cargo.toml` and a symlinked
    target file that lead outside, and every discovery classification of a symlink
    (file, directory, FIFO, outside file, outside directory, dangling, loop).
