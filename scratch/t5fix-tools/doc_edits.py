"""Apply the raf-source reachability edits of this fix, and only them, to a
source-index.md and a CHANGELOG.md: each edited paragraph is taken from the
edited worktree copy (argv[1]) by its first words and put in place of the
same paragraph of the target (argv[2]); bindings are added by name.

usage: doc_edits.py <edited-root> <target-root>
"""

import sys

SRC, DST = sys.argv[1], sys.argv[2]
SI = "docs/capabilities/source-index.md"
CL = "CHANGELOG.md"


def lines(path):
    return open(path, encoding="utf-8").read().split("\n")


def find(rows, prefix):
    hits = [i for i, row in enumerate(rows) if row.startswith(prefix)]
    assert len(hits) == 1, (prefix, hits)
    return hits[0]


def block(rows, start, stop_prefixes):
    """Rows from `start` up to the first row after it that starts with one of
    `stop_prefixes` (exclusive)."""
    end = start + 1
    while end < len(rows) and not any(
        rows[end] == "" if p == "" else rows[end].startswith(p) for p in stop_prefixes
    ):
        end += 1
    return start, end


src = lines(f"{SRC}/{SI}")
dst = lines(f"{DST}/{SI}")

# Single-line paragraphs replaced whole.
for prefix in [
    "**Impl blocks.**",
    "**Reachability.**",
    "The answer is a `Reachability`",
    "- `impls()` (`symbol_index`)",
    "- `reachable` (`reachability`)",
    "- The work of reachability is counted",
]:
    dst[find(dst, prefix)] = src[find(src, prefix)]

# The new paragraph on `Self`, after the Reachability paragraph.
self_par = src[find(src, "A run that `Self` starts, and the name of a method")]
at = find(dst, "**Reachability.**")
if not any(row.startswith("A run that `Self` starts, and the name") for row in dst):
    dst[at + 1 : at + 1] = ["", self_par]

# The Constraints items on reachability: from the first to the item after the
# last one this fix writes.
s0 = find(src, "- Reachability reads paths from tokens")
s1 = find(src, "- An impl block inside `const _: () = { .. };`")
s1 = block(src, s1, ["- "])[1]
d0 = find(dst, "- Reachability reads paths from tokens")
d1 = block(dst, d0, ["- "])[1]
dst[d0:d1] = src[s0:s1]

# The item-bytes budget item of the work and memory bounds.
s0 = find(src, "- `max_item_bytes` (`ItemBytes`, 512 MiB)")
s1 = block(src, s0, ["- "])[1]
d0 = find(dst, "- `max_item_bytes` (`ItemBytes`, 512 MiB)")
d1 = block(dst, d0, ["- "])[1]
dst[d0:d1] = src[s0:s1]

# Bindings.
P = "    - raf_source::test_crate::reachability::"
U = "    - raf_source::reach::tests::"
S = "    - raf_source::test_crate::symbol_index::"


def rename(old, new):
    dst[find(dst, old)] = new


def after(anchor, new_rows):
    at = find(dst, anchor)
    dst[at + 1 : at + 1] = [row for row in new_rows if row not in dst]


rename(
    P + "a_root_of_another_workspace_is_refused_beside_one_of_this_workspace",
    P + "a_root_past_the_items_or_the_blocks_of_the_workspace_is_refused_beside_one_in_range",
)
after(
    P + "test_code_names_the_dev_dependencies_and_other_code_does_not",
    [
        P + n
        for n in [
            "gen_is_a_name_before_edition_2024_and_a_keyword_from_it",
            "a_pointer_to_a_type_and_a_raw_borrow_of_a_static_reach_them_and_a_constant_its_type",
            "self_paths_and_methods_called_on_self_reach_the_members_of_that_name",
            "self_in_a_trait_and_in_its_impl_blocks_reaches_the_members_of_the_trait_and_of_the_block",
            "a_variant_through_self_names_its_enum_as_a_path_through_the_enum_does",
            "the_type_a_where_clause_bounds_is_reached_and_a_bound_name_is_not",
            "a_walk_past_the_item_bytes_budget_is_refused_and_one_within_it_is_given",
        ]
    ],
)
after(
    S + "an_impl_block_of_a_file_reached_under_two_module_paths_is_listed_once_for_each",
    [S + "a_negative_impl_keeps_its_bang_in_the_trait_path_beside_a_positive_one"],
)
after(
    U + "a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix",
    [
        U + n
        for n in [
            "the_keywords_are_those_of_the_edition",
            "const_declares_nothing_so_a_pointer_and_a_raw_borrow_name_their_path",
            "a_where_clause_keeps_the_type_its_bound_is_put_on",
            "a_name_called_on_self_is_a_method_run_and_a_field_of_self_is_not",
            "the_work_over_types_impl_blocks_traits_and_members_grows_with_each_and_not_their_product",
            "what_the_walk_holds_is_charged_to_the_item_bytes_budget_on_from_the_load",
        ]
    ],
)
open(f"{DST}/{SI}", "w", encoding="utf-8").write("\n".join(dst))

# CHANGELOG: the two raf-source items, each a bullet with its continuation
# lines.
src = lines(f"{SRC}/{CL}")
dst = lines(f"{DST}/{CL}")
for prefix in [
    "- **`raf-source`:** `Workspace::impls` lists every impl block",
    "- **`raf-source`:** `Workspace::reachable` gives the items",
]:
    s0, s1 = block(src, find(src, prefix), ["- ", "#", ""])
    d0, d1 = block(dst, find(dst, prefix), ["- ", "#", ""])
    dst[d0:d1] = src[s0:s1]
open(f"{DST}/{CL}", "w", encoding="utf-8").write("\n".join(dst))
print("applied")
