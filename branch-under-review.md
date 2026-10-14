# Branch under review — issue #2

Branch `paudley/2-shared-crates-raf-okf-raf-source` against `origin/main`. Gathered by `stagectl brief
--mode review`; every hash and path came from this repository.

## Commits (13)

- `6c744af63` docs(capabilities): deliver the OKF profile and source index  _2026-10-12_
- `e6f3e9bb8` fix(raf-source): resolve a target's crate root as a module item  _2026-10-12_
- `376baf93b` docs(changes): record the outcomes of the raf-okf and raf-source items  _2026-10-12_
- `4e9c2f397` test(docs-contract): read documents through raf-okf and resolve bindings  _2026-10-12_
- `52155fa59` fix(raf-source): inherit workspace keys from the member's workspace  _2026-10-12_
- `e4c6a6622` feat(raf-source): symbol index and use-alias resolution  _2026-10-12_
- `e15dd4b1a` feat(raf-source): package discovery and rustc module-tree walk  _2026-10-12_
- `1e965a1ab` build: release-mode, fuzz and cross-target gates  _2026-10-12_
- `0a9d24352` feat(raf-source): bounded parsing on a dedicated stack with a fuzz target  _2026-10-12_
- `57d9a0614` feat(raf-source): file tree seam with canonical containment  _2026-10-12_
- `1a67fa3e5` feat(raf-okf): profile validation with typed documents  _2026-10-12_
- `e94cd8df8` feat(raf-okf): body model, transactional splicing and capability index  _2026-10-12_
- `3f8697b35` feat(raf-okf): bounded frontmatter reader and deterministic writer  _2026-10-12_

## Files changed

```
.github/PULL_REQUEST_TEMPLATE.md                   |    2 +-
 .github/workflows/ci.yaml                          |   57 +-
 AGENTS.md                                          |   48 +-
 CHANGELOG.md                                       |   25 +-
 CONTRIBUTING.md                                    |   15 +-
 Cargo.lock                                         |  165 ++
 Cargo.toml                                         |    7 +-
 Makefile                                           |   52 +-
 README.md                                          |    9 +-
 REUSE.toml                                         |   15 +
 SECURITY.md                                        |   32 +-
 crates/raf-okf/Cargo.toml                          |   20 +
 crates/raf-okf/src/body/blocks.rs                  |  472 ++++
 crates/raf-okf/src/body/heading.rs                 |  218 ++
 crates/raf-okf/src/body/link.rs                    |  659 +++++
 crates/raf-okf/src/body/marker.rs                  |  233 ++
 crates/raf-okf/src/body/mod.rs                     |  363 +++
 crates/raf-okf/src/body/prose.rs                   |   49 +
 crates/raf-okf/src/document.rs                     |  213 ++
 crates/raf-okf/src/error.rs                        |  455 ++++
 crates/raf-okf/src/frontmatter/mod.rs              |    9 +
 crates/raf-okf/src/frontmatter/reader.rs           |  615 +++++
 crates/raf-okf/src/frontmatter/scalar.rs           |  477 ++++
 crates/raf-okf/src/frontmatter/value.rs            |  532 ++++
 crates/raf-okf/src/frontmatter/writer.rs           |  187 ++
 crates/raf-okf/src/index.rs                        |  171 ++
 crates/raf-okf/src/lib.rs                          |   83 +
 crates/raf-okf/src/limits.rs                       |   16 +
 crates/raf-okf/src/profile/body_rules.rs           |  239 ++
 crates/raf-okf/src/profile/mod.rs                  |  742 ++++++
 crates/raf-okf/src/profile/schema.rs               |  163 ++
 crates/raf-okf/src/profile/timestamp.rs            |  126 +
 crates/raf-okf/src/profile/typed.rs                |  231 ++
 crates/raf-okf/src/profile/violation.rs            |  331 +++
 crates/raf-okf/src/span.rs                         |   22 +
 crates/raf-okf/tests/alloc_bound.rs                |   91 +
 crates/raf-okf/tests/body_robustness.rs            |   96 +
 crates/raf-okf/tests/body_structure.rs             |  385 +++
 crates/raf-okf/tests/index.rs                      |  188 ++
 crates/raf-okf/tests/links.rs                      |  416 +++
 crates/raf-okf/tests/markers.rs                    |  187 ++
 crates/raf-okf/tests/no_io_in_source.rs            |  164 ++
 crates/raf-okf/tests/placeholder_lines.rs          |   24 +
 crates/raf-okf/tests/profile.rs                    | 1304 ++++++++++
 crates/raf-okf/tests/refusals.rs                   |  820 ++++++
 crates/raf-okf/tests/render.rs                     |  686 +++++
 crates/raf-okf/tests/repo_markdown.rs              |   59 +
 crates/raf-okf/tests/schema.rs                     |  147 ++
 crates/raf-okf/tests/splice.rs                     |  312 +++
 crates/raf-okf/tests/usage_docs.rs                 |   91 +
 crates/raf-source/Cargo.toml                       |   25 +
 crates/raf-source/src/budget.rs                    |  102 +
 crates/raf-source/src/cfg.rs                       |  137 +
 crates/raf-source/src/discovery.rs                 |  209 ++
 crates/raf-source/src/error.rs                     |  520 ++++
 crates/raf-source/src/facts.rs                     | 1102 ++++++++
 crates/raf-source/src/glob.rs                      |  187 ++
 crates/raf-source/src/impls.rs                     |  479 ++++
 crates/raf-source/src/index.rs                     |  746 ++++++
 crates/raf-source/src/item.rs                      |  192 ++
 crates/raf-source/src/lib.rs                       |  143 +
 crates/raf-source/src/manifest.rs                  |  401 +++
 crates/raf-source/src/members.rs                   |  296 +++
 crates/raf-source/src/modtree.rs                   |  584 +++++
 crates/raf-source/src/options.rs                   |   25 +
 crates/raf-source/src/parse.rs                     |   83 +
 crates/raf-source/src/path.rs                      |  224 ++
 crates/raf-source/src/prescan.rs                   |  545 ++++
 crates/raf-source/src/resolve.rs                   |  517 ++++
 crates/raf-source/src/settle.rs                    |  610 +++++
 crates/raf-source/src/target.rs                    |  473 ++++
 crates/raf-source/src/tree/disk.rs                 |  501 ++++
 crates/raf-source/src/tree/disk/tests.rs           |  300 +++
 crates/raf-source/src/tree/memory.rs               |  219 ++
 crates/raf-source/src/tree/mod.rs                  |   83 +
 crates/raf-source/src/workspace.rs                 |  598 +++++
 crates/raf-source/tests/corpus_replay.rs           |  212 ++
 crates/raf-source/tests/discovery_targets.rs       |  724 ++++++
 crates/raf-source/tests/disk_tree_containment.rs   |  310 +++
 crates/raf-source/tests/disk_tree_root.rs          |   60 +
 crates/raf-source/tests/fs_confinement.rs          |  180 ++
 crates/raf-source/tests/index_disk.rs              |   61 +
 crates/raf-source/tests/memory_tree.rs             |  189 ++
 crates/raf-source/tests/module_tree.rs             |  976 +++++++
 crates/raf-source/tests/overflow_harness.rs        |  261 ++
 crates/raf-source/tests/prescan_budgets.rs         |  242 ++
 crates/raf-source/tests/read_cap.rs                |   97 +
 crates/raf-source/tests/relpath.rs                 |  114 +
 crates/raf-source/tests/support/child.rs           |  224 ++
 crates/raf-source/tests/support/mod.rs             |  116 +
 crates/raf-source/tests/support/shapes.rs          |  252 ++
 crates/raf-source/tests/symbol_index.rs            |  673 +++++
 crates/raf-source/tests/target_roots.rs            |  174 ++
 crates/raf-source/tests/this_repository.rs         |   28 +
 crates/raf-source/tests/trait_keys.rs              |  844 ++++++
 crates/raf-source/tests/tree_error.rs              |   66 +
 crates/raf-source/tests/use_resolution.rs          |  557 ++++
 crates/raf-source/tests/workspace_containment.rs   |  339 +++
 deny.toml                                          |    8 +
 docs/architecture/okf-profile.md                   |   86 +-
 docs/capabilities/census-rule-api.md               |    3 +-
 docs/capabilities/okf-profile.md                   |  250 +-
 docs/capabilities/source-index.md                  |  601 ++++-
 docs/capabilities/workplan-document-model.md       |    3 +-
 fuzz/.gitignore                                    |    6 +
 fuzz/Cargo.lock                                    |  232 ++
 Cargo.toml => fuzz/Cargo.toml                      |   47 +-
 fuzz/corpus/parse_source/budget_angles_at.rs       |    1 +
 fuzz/corpus/parse_source/budget_angles_over.rs     |    1 +
 fuzz/corpus/parse_source/budget_closure_bars_at.rs |    1 +
 .../parse_source/budget_closure_bars_over.rs       |    1 +
 fuzz/corpus/parse_source/budget_delimiters_at.rs   |    1 +
 fuzz/corpus/parse_source/budget_delimiters_over.rs |    1 +
 fuzz/corpus/parse_source/budget_else_if_at.rs      |    1 +
 fuzz/corpus/parse_source/budget_else_if_over.rs    |    1 +
 fuzz/corpus/parse_source/budget_prefix_at.rs       |    1 +
 fuzz/corpus/parse_source/budget_prefix_over.rs     |    1 +
 fuzz/corpus/parse_source/composite_bang_paren.rs   |    1 +
 fuzz/corpus/parse_source/composite_bars_paren.rs   |    1 +
 .../parse_source/composite_double_ref_paren.rs     |    1 +
 fuzz/corpus/parse_source/composite_generic_ref.rs  |    1 +
 fuzz/corpus/parse_source/composite_ref_paren.rs    |    1 +
 .../evasion_else_if_block_condition_20000.rs       |    1 +
 .../evasion_else_if_block_condition_at.rs          |    1 +
 .../parse_source/evasion_ref_lifetime_20000.rs     |    1 +
 .../corpus/parse_source/evasion_ref_lifetime_at.rs |    1 +
 fuzz/corpus/parse_source/single_bang_4000.rs       |    1 +
 .../parse_source/single_closure_bars_10000.rs      |    1 +
 fuzz/corpus/parse_source/single_else_if_20000.rs   |    1 +
 fuzz/corpus/parse_source/single_generics_500.rs    |    1 +
 fuzz/corpus/parse_source/single_ref_20000.rs       |    1 +
 fuzz/corpus/parse_source/uncounted_assignment.rs   |    1 +
 fuzz/corpus/parse_source/uncounted_fn_pointer.rs   |    1 +
 fuzz/corpus/parse_source/uncounted_move_closure.rs |    1 +
 fuzz/corpus/parse_source/uncounted_ptr_const.rs    |    1 +
 fuzz/corpus/parse_source/uncounted_ref_mut.rs      |    1 +
 fuzz/corpus/parse_source/uncounted_return.rs       |    1 +
 deny.toml => fuzz/deny.toml                        |   15 +
 fuzz/fuzz_targets/parse_source.rs                  |   21 +
 tests/docs_contract.rs                             | 2738 +++++++++-----------
 tests/repo_gates.rs                                |  273 +-
 tests/support/mod.rs                               |    2 +-
 work/changes/okf-profile.md                        |   27 -
 work/changes/source-index.md                       |   29 -
 144 files changed, 30583 insertions(+), 1811 deletions(-)
```

## ADRs the changed files cite

None of the changed files cite an ADR.

These are the settled decisions this branch touches. The question
for stage2 is whether the change fits them, not merely whether it
compiles.

