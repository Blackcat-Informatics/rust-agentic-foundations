# Referenced artifacts — issue #21

Everything the issue names, checked against disk. Facts only: whether a
file is a viable port source, or may be relicensed, is decided in the plan.
Paths resolve against the main checkout, then against a sibling checkout
only when that repo is named (first path component, or in the prose just
before the reference). "not here; exists in sibling(s)" is NOT resolved.

## Paths

| Reference | Exists | Resolved in | Kind | Lines | Licence header |
|---|---|---|---|---|---|
| `crates/raf-source/tests/exported_items.rs` | yes | repo | file | 1669 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 |
| `crates/raf-okf/tests/validate_cost.rs` | yes | repo | file | 165 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 |
| `crates/raf-workplan/tests/assessments.rs` | yes | repo | file | 965 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 |
| `crates/raf-workplan/tests/rationale.rs` | yes | repo | file | 608 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 |
| `crates/raf-workplan/src/evidence/tests.rs` | yes | repo | file | 966 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 |
| `exported_items.rs` | NO | repo | — | — | — |

### History: `crates/raf-source/tests/exported_items.rs`

- `1c5a765db fix(raf-source): a companion the settled tables drop is dropped from the start of the next settle, so its answer no longer depends on the order`
- `f1603cd70 test(raf-source): the listing tests count glob view reads and walk steps instead of reading the clock, and a gate keeps the clock out of the crate`
- `79fb59834 fix(raf-source): refuse a name that globs bring in as different items when the reader can access one`
- `e0a75e9e1 fix(raf-source): a re-export is no more visible than the binding it re-exports`
- `5f5049baa test(raf-source): compare use-path resolution with rustc on generated programs`
- `0738a45e3 test(raf-source): pin the first carrying glob as the site of a re-export two globs bring in`
- `8844239ea feat(raf-source): list the exported items of a target`

### History: `crates/raf-okf/tests/validate_cost.rs`

- `de0b842d2 feat(raf-workplan): load evidence, assessment and coverage records with a shared digest`
- `671850b2b perf(raf-okf): validate affects and generated blocks without quadratic scans`

### History: `crates/raf-workplan/tests/assessments.rs`

- `16d3648f8 fix(raf-workplan): never write a record or a generated block through a linked directory`
- `6b317fc6b feat(raf-workplan): two-way coverage of public items and tests`
- `bb415d37c feat(raf-workplan): prose assessments, their obligations and a sixth completion condition`

### History: `crates/raf-workplan/tests/rationale.rs`

- `c54db19d2 feat(raf-workplan): check current rationale pages and decisions`

### History: `crates/raf-workplan/src/evidence/tests.rs`

- `16d3648f8 fix(raf-workplan): never write a record or a generated block through a linked directory`
- `9bacdb380 feat(raf-workplan): evidence records, claims and their obligations`

## Identifiers (files containing each at HEAD)

- `perf`: 16
- `tree::disk::tests`: 3
- `resolve_paths`: 1
- `manifest_caps`: 1
- `trait_keys`: 3
- `module_tree`: 4
- `exported_items`: 17
- `prescan_budgets`: 2
- `symbol_index`: 3
- `facts`: 36
- `tree::disk`: 4

## Workspace packages named

- `raf-okf`
- `raf-source`
- `raf-workplan`

## URLs

None.
