# Branch under review — issue #14

Branch `paudley/14-raf-source-model-rustc-s-visibility-of` against `origin/main`. Gathered by `stagectl brief
--mode review`; every hash and path came from this repository.

## Commits (21)

- `fc21ada21` test: compare the clean neighbour of the clock gate to an empty list  _2026-10-17_
- `19f3701f7` test: the clock gate also refuses the core and async timers and the timeout waits, and two counts get a lower bound  _2026-10-17_
- `d3a779c6b` docs(source-index): deliver the visibility rules, the reading-module entry points and the limits that remain  _2026-10-17_
- `c60fa0dc5` docs(source-index): reopen the capability to state the visibility rules, the entry points and the limits  _2026-10-17_
- `f1603cd70` test(raf-source): the listing tests count glob view reads and walk steps instead of reading the clock, and a gate keeps the clock out of the crate  _2026-10-17_
- `d495a433d` test(raf-source): an outside FIFO reached by a swap is shown unopened by an inotify watch, not by a blocked writer  _2026-10-17_
- `937fd1a9c` test(raf-source): the disk tests call the tree directly, and every open is checked to be a reference or non-blocking  _2026-10-17_
- `e5bc866bf` fix(raf-source): a companion item no longer rides around a glob cycle past the rule that drops it  _2026-10-17_
- `760b80c15` test(raf-source): the box-pattern refusal shows the parser never ran, not how fast it was  _2026-10-17_
- `36fc3e017` test(raf-source): the module-path budget tests count tree calls instead of reading the clock  _2026-10-17_
- `5b96f786c` test(raf-source): a path of fifty thousand segments is bounded by counted reads, not by the clock  _2026-10-17_
- `c299b91d8` test(raf-source): member and exclude patterns are bounded by counted work, not by the clock  _2026-10-17_
- `72bedc497` test(raf-source): facts and long-line positions are bounded by counted work, not by the clock  _2026-10-17_
- `3d39a9996` test(raf-source): test-only work tallies with a ceiling that fails fast  _2026-10-17_
- `5aced3c9c` test(raf-source): one counting tree with an operation ceiling replaces the two ad hoc read counters  _2026-10-17_
- `1ef2915da` test(raf-source): the oracle asks rustc one name space at a time before calling a divergence order-dependent  _2026-10-17_
- `d692c8244` docs(readme): state the current status of the crates and lead with the built workplan gate  _2026-10-17_
- `173a2cbac` fix(raf-source): an import a reader cannot name holds a glob binding back only while the globs bring the name in  _2026-10-17_
- `79fb59834` fix(raf-source): refuse a name that globs bring in as different items when the reader can access one  _2026-10-17_
- `e0a75e9e1` fix(raf-source): a re-export is no more visible than the binding it re-exports  _2026-10-17_
- `5f5049baa` test(raf-source): compare use-path resolution with rustc on generated programs  _2026-10-17_

## Files changed

```
CHANGELOG.md                                       |   22 +
 README.md                                          |   44 +-
 crates/raf-source/src/exports.rs                   |    9 +-
 crates/raf-source/src/exports/tests.rs             |  168 ++++
 crates/raf-source/src/facts.rs                     |   88 +-
 crates/raf-source/src/glob.rs                      |   36 +
 crates/raf-source/src/glob_scope.rs                |  928 ++++++++++++-----
 crates/raf-source/src/lib.rs                       |    1 +
 crates/raf-source/src/members.rs                   |   90 +-
 crates/raf-source/src/resolve.rs                   |  142 ++-
 crates/raf-source/src/settle.rs                    |  128 ++-
 crates/raf-source/src/tree/disk.rs                 |   36 +
 crates/raf-source/src/tree/disk/handle_path.rs     |   11 +-
 crates/raf-source/src/tree/disk/linux.rs           |    5 +-
 crates/raf-source/src/tree/disk/other_unix.rs      |    2 +
 crates/raf-source/src/tree/disk/tests.rs           |  206 ++--
 crates/raf-source/src/work.rs                      |  233 +++++
 crates/raf-source/src/workspace.rs                 |   50 +
 crates/raf-source/tests/disk_tree_containment.rs   |   35 +-
 crates/raf-source/tests/exported_items.rs          |  378 +++----
 crates/raf-source/tests/manifest_caps.rs           |  140 ++-
 crates/raf-source/tests/module_tree.rs             |  151 +--
 crates/raf-source/tests/prescan_budgets.rs         |   34 +-
 crates/raf-source/tests/resolve_paths.rs           |  101 +-
 crates/raf-source/tests/rustc_oracle.rs            | 1001 ++++++++++++++++++
 crates/raf-source/tests/support/counting.rs        |  170 ++++
 crates/raf-source/tests/support/json.rs            |  246 +++++
 crates/raf-source/tests/support/mod.rs             |   28 +-
 crates/raf-source/tests/support/program.rs         |  889 ++++++++++++++++
 crates/raf-source/tests/support/random.rs          |  127 +++
 crates/raf-source/tests/support/rustc.rs           |  427 ++++++++
 crates/raf-source/tests/symbol_index.rs            |   12 +-
 crates/raf-source/tests/trait_keys.rs              |   13 +-
 crates/raf-source/tests/use_resolution.rs          | 1063 +++++++++++++++++++-
 crates/raf-source/tests/workspace_containment.rs   |   83 +-
 docs/capabilities/source-index.md                  |  297 +++---
 .../rustc-oracle/accessible_glob_reexported.rs     |   13 +
 .../ambiguity_a_glob_cycle_delivers_back.rs        |   22 +
 ...mbiguity_hidden_by_a_private_glob_in_a_chain.rs |   15 +
 .../ambiguity_through_inaccessible_glob.rs         |   19 +
 .../ambiguous_module_in_the_middle_of_a_path.rs    |   14 +
 .../ambiguous_module_through_inaccessible_glob.rs  |   28 +
 .../glob_order_changes_what_rustc_accepts.rs       |   15 +
 ...whether_a_type_a_glob_cycle_carries_is_found.rs |   24 +
 .../rustc-oracle/glob_reexport_ambiguity_warned.rs |   14 +
 ...item_beside_an_item_the_reader_already_holds.rs |   19 +
 ..._cycle_carries_back_beside_the_module_itself.rs |   26 +
 ...n_one_name_space_beside_a_glob_that_holds_it.rs |   12 +
 ...not_determine_beside_a_glob_that_resolves_it.rs |   19 +
 ...termine_through_a_named_import_and_two_globs.rs |   17 +
 ...ugh_a_named_import_and_two_globs_of_a_module.rs |   17 +
 .../named_reexport_of_a_private_glob_binding.rs    |   16 +
 .../private_glob_beside_a_different_name.rs        |   14 +
 ...rivate_import_shadows_the_glob_for_outsiders.rs |   14 +
 ...a_private_import_of_its_name_in_a_glob_cycle.rs |   22 +
 .../private_item_reexported_by_a_child_glob.rs     |   12 +
 .../struct_and_function_of_one_name.rs             |   10 +
 ...ide_a_glob_that_brings_in_a_type_of_its_name.rs |   23 +
 tests/repo_gates.rs                                |  157 ++-
 work/assessments/source-index-constraints.md       |   13 +-
 work/assessments/source-index-contract.md          |   10 +-
 work/assessments/source-index-examples.md          |   11 +-
 work/assessments/source-index-usage.md             |   13 +-
 work/assessments/source-index-verification.md      |   13 +-
 64 files changed, 7000 insertions(+), 996 deletions(-)
```

## ADRs the changed files cite

None of the changed files cite an ADR.

These are the settled decisions this branch touches. The question
for stage2 is whether the change fits them, not merely whether it
compiles.

