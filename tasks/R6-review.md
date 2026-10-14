# R6 review: one additive parser recursion budget

## Scope

R6 alone: `raw/R6.patch` applied with `git apply -p1` to a `git archive e472918`
copy at `/tmp/r6rev/src` (17 files, +935/-129). `/tmp/raf-par/p1-r6r7` was read
only and not modified. Checked against "### R6", "Per-item gates", "Mutation
rule" and "Registry harness" in `remediation-plan.md`, plus AGENTS.md and
CLAUDE.md.

Coverage of the five requested parts:

| Part | Status |
| --- | --- |
| (1) Evasion: inputs under 1927 that overflow the parse thread | Not performed in the first pass. **Recheck:** covered by the committed 36-family boundary test, verified below |
| (2) False positives | Done (evidence audit) |
| (3) `::` x16 weight | Done |
| (4) Contract text accuracy and carve-outs | Done |
| (5) Falsifiability: 2 of 9 mutations re-run | Done |
| Per-item gates on R6 alone | Done |

## Evidence

All gates were run on the R6-alone copy with `CARGO_BUILD_JOBS=4`, `ulimit -v`, and
one job at a time. Logs are in `/tmp/r6rev/logs/`.

| Gate | Result |
| --- | --- |
| `cargo test -p raf-source --locked` (debug) | exit 0, 0 failed, 1 ignored (`corpus_replay::write_corpus`; it was there before R6, and R6 adds or removes no `#[ignore]`) |
| `cargo test -p raf-source --locked --release` | exit 0, 368 passed, 0 failed, same 1 ignored |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0, 0 warnings |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked` | exit 0, 0 warnings |
| `cargo test --locked --test docs_contract --test repo_gates` | 139 + 55 passed |
| `make cross-check` | exit 0 |
| `make msrv` | exit 0 |
| `make reuse` | exit 0, compliant |

Tests that passed in both debug and release:

- `every_committed_overflow_shape_is_refused_by_a_budget_and_none_aborts`
- `every_corpus_file_is_refused_or_indexed_with_its_recorded_symbols_and_none_aborts`
- `the_judge_rejects_a_wrong_index_a_misnamed_refusal_an_abort_and_a_stray_failure`
- `the_eighty_kilobyte_box_pattern_is_refused_in_under_a_second`
- `a_use_tree_at_the_recursion_limit_loads_resolves_and_drops_on_a_five_hundred_twelve_kib_thread`

**Carve-out grep.** `grep -rni 'no budget counts\|can end in such an abort\|can end in that abort' docs/capabilities/source-index.md SECURITY.md` prints nothing (exit 1).

**Contract figures** (`source-index.md`, lines 392-445) against
`raw/R6-stack-costs-*.txt`:

- **Debug figures** all match:
  - 16481: nested generic arguments
  - 15724: reference types
  - 14808: tuple, slice and array types
  - 12657: nested `fn` pointer arguments
  - 12257: inline modules (rerun figure)
  - 10414: nested `trait` bodies
  - 10258: closures with block bodies
  - 9982: `break` chains
  - 8831: `return` chains
  - 8277: struct literals
  - 7358: assignment chains
  - 7279: closure chains
  - 3657: unary operators
  - at most 1118: looped trailers and binary operators
- **Release maximum:** 4670 (`break` and `break_label`) matches.
- **Limit arithmetic:** floor(64 MiB / (2 x 17 KiB)) = 1927. At the limit the
  costliest unit uses 1927 x 16481 = 31,758,887 B, a factor of 2.11 under the
  64 MiB stack. The contract requires at least 2.
- **Counter definition:** the Contract text matches `prescan/recursion.rs`:
  - the 25 counted keywords;
  - `&`, `&&` and `<` doubled;
  - dotted numbers count 1;
  - labels count 1;
  - `::` counts 16 inside `use`;
  - opening delimiters count, except the attribute `[` and macro delimiters;
  - macro interiors count only their groups;
  - match-arm `|` counts nothing;
  - the reset and continuation rules, with `as`, `else` and `in`.
- The five existing budgets remain in `BUDGETS`.

**Fixtures.** Every counted keyword has an at-limit and over-limit fixture in
`prescan_budgets.rs`, including `mod m {}` (3 units) and the `where`-clause pair.
`raw/R6-falsification.txt` shows the over-limit texts and committed shapes are
PARSED or ABORT on e472918 (195 of 196 PARSED, none refused by the new kind).

**Falsifiability.** I re-ran two mutations and reverted each with `sed`:

- **M1** (`DOUBLED_UNITS = 1`): fails
  `every_operator_counts_one_for_each_of_its_characters_at_the_limit_and_one_over`
  and `a_construct_that_continues_past_a_comma_or_a_closing_brace_keeps_its_count`.
  This matches the record.
- **M8** (`USE_SEPARATOR_UNITS = 1`): fails
  `a_label_a_use_path_separator_and_each_delimiter_count_at_the_limit_and_one_over`.
  This matches the record.

After the reverts, `recursion.rs` is byte-identical to the implementer's file and
`prescan_budgets` passes 26/26.

**False positives (part 2).** From `raw/registry-scan-R6.txt` and
`raw/R6-prescan-all-registry-files.txt`:

- **Registry scan:** 1181 OK and 30 ERR, with 0 "parser recursion" refusals and
  no `Budget` ERR at all.
- **Same failures as before:** the 30 failing crates and their error kinds are
  identical to `registry-scan-R5b.txt`.
- **Every registry `.rs` file:** a prescan over all of them (1954 dirs, every
  version) peaks at 987 (`funty` 2.0.0 lib.rs:411), 51% of the limit, with p99 =
  215. The files covered include generated and table-heavy sources:
  - `winapi` d3d11
  - `typenum` gen/consts
  - `brotli` static_dict
  - `zerocopy-derive` expected output
  - `httparse` uri tests
  - `syn` expr.rs
  - `sqlparser`
- **This repository:** loads with no refusal in the `this_repository` tests.
- The limit is the derived value and was not raised.

**`::` x16 weight (part 3).**

- **Load test:** a `use` tree at the limit (1927/16 = 120 separators, or 107 nested
  groups) loads, resolves and drops on a 512 KiB thread in debug and release,
  and one more is refused.
- **Unweighted counterexample:** the implementer's base note records that an
  unweighted `::` at the limit overflowed a 2 MiB thread in `index::flatten`.
- **Real code:** no registry crate is refused because of the weight.

## Findings

### REQUIRED

1. **The evasion review (part 1) is not done.** This review did not build or
   run inputs that try to stay under 1927 while overflowing the 64 MiB parse
   thread. That covers the reset rules, macro interiors, match-arm
   alternatives, `where` clauses, `as` chains, attributes, generics, literal
   and lexer edge cases, and later stages that might parse macro bodies.

   The committed harness and corpus show that every *recorded* shape is refused
   and none aborts. That shows coverage of the recorded shapes, not that the
   counter holds against adversarial input, which the contract's "any abort
   under the limit is REQUIRED" needs.

   R6 cannot pass until a reviewer completes part 1 and records the result.
   No defect was found; this is an open review item.

### ADVISORY

1. **Stale first-run figures.** `raw/R6-stack-costs-debug.txt` still records
   `fn_where_nested` at 20086 B/unit and `mod_inline` at 18386 B/unit. Both are
   above the 16481 B unit the limit is derived from. They are superseded by
   `R6-stack-costs-debug-rerun.txt` (8037 and 12257), which was measured after
   `where` and `mod` became counted, and the Contract cites the rerun figures.
   A note in the first file pointing to the rerun would stop a later reader
   from taking the old figures as a breach.
2. **The committed harness child skips the fact visitor.** `tests/support/child.rs`
   parses with the `symbols` visitor, not the fact visitor (`extract`). The
   committed "none aborts" checks therefore cover parse plus drop on the parse
   thread, but not reading the facts, which the Contract counts in the
   per-unit cost. Only `walk_stack.rs` drives a full load. The stack-cost
   figures came from the implementer's uncommitted measurement tool. Running
   the overflow and corpus child through `Workspace::load` would bind the
   committed test to the same path the Contract describes.

First-pass verdict: BLOCKED (REQUIRED 1).

## Recheck

**Scope.** I checked the regenerated `raw/R6.patch` (17 files, +1414/-156)
applied to a fresh `git archive e472918` copy at `/tmp/r6rev2/src`. Logs are in
`/tmp/r6rev2/logs/`. After the mutations, `git apply -R --check` of the patch
succeeds, so the tree is exactly what the patch produces.

**The boundary test.** The test is
`overflow_harness::every_shape_family_loads_at_its_largest_admitted_size_and_one_size_more_is_refused`.

- **Families:** it asserts `families().len() == 36`.
- **Per family:** it runs `family_<name>_at` and `family_<name>_over` in a child
  process.
- **Assertions:** `at` must equal `Outcome::Index(symbols)` and `over` must equal
  `Outcome::Refused` with the family's budget. An abort is `Outcome::Aborted`,
  which never equals `Index`, so any family that aborts at its admitted size
  fails the test.
- **Load path:** the child (`tests/support/child.rs::run_child_if_requested`)
  builds a `MemoryTree` library and calls `Workspace::load`. That runs the parse,
  the fact visitor and the drop on the parse thread, then the index on the
  caller's thread. First-pass advisory 2 is resolved for the overflow harness
  and the corpus replay.
- **Boundary:** `at` is the largest admitted size and `at + 1` is refused, so
  every family is tested exactly at its boundary.
- **Results:** the test passes in debug (2.16 s) and release (0.47 s).

**Spot mutations** (debug, each reverted):

- **MA** (limit raised to `PARSE_STACK_BYTES / RECURSION_UNIT_BYTES`, the factor
  2 removed): the test fails at `overflow_harness.rs:154`. The left side is
  `Index(["f"])` and the right side is `Refused { file:
  "family_closures_with_parameters_over.rs", budget: "parser recursion" }`.
- **MB** (`return` removed from `COUNTED_KEYWORDS`, which the reset-exploiting
  families rely on through `PAD`): fails the same way.

So the test catches both a raised limit and a counted token that is dropped.

**Family list against the part-1 categories:**

| Category | Families |
| --- | --- |
| Reset-exploiting nesting split by `,`, `;` or `=>` | `closures_with_parameters`, `tuples`, `arrays`, `struct_literals`, `statement_blocks`, `match_arms`, `call_arguments`, `if_let_else_if`, `labelled_blocks`, `async_blocks` |
| Macro interiors and attributes | `macro_call_groups`, `macro_rules_bodies`, `attribute_arguments`. Because these now load through `Workspace::load`, they also show the fact visitor does not re-parse macro bodies into deeper recursion at the admitted size |
| Match-arm alternatives | `deep_pattern_alternatives` |
| `where` clauses and bounds | `where_bounds`, `impl_trait_arguments`, `nested_for_lifetimes` |
| Generics in types and in expressions | `generic_types`, `turbofish` |
| Other nested types | `reference_pointer_chains`, `fn_pointer_chains`, `dyn_fn_chains`, `tuple_and_array_types` |
| `as` chains | `cast_chains` |
| Other expression chains | `unary_chains`, `try_chains`, `range_chains`, `method_chains` |
| Lexer edge cases: strings, raw strings, char vs lifetime, `r#ident` | `raw_strings`, `chars_and_lifetimes`, `raw_identifiers`, `nested_block_comments`, `crlf_lines` |
| Nested items | `inline_modules`, `fns_in_blocks`, `impls_in_fns` |

Two categories from part 1 have no family:

- `///` and `//!` doc comments, which become `#[doc]` attributes;
- a leading shebang line.

Neither opens nesting by itself. I did not write or run new shapes for them.
They are listed for the implementer under ADVISORY R2.

**Headroom figures.** `raw/R6-shape-coverage.txt` gives the headroom with every
budget disabled:

- lowest debug: `generic_types` and `turbofish` at score 4258, 2.21x the limit;
- lowest release: 9.4x.

I did not reproduce these unbounded overflow runs. They are the implementer's
evidence. The committed test proves the part that matters for acceptance: each
family loads at its admitted boundary in both profiles without aborting.

**First-pass advisory 1.** The note is present on line 1 of
`raw/R6-stack-costs-debug.txt`. It names the rerun as authoritative for
`fn_where_nested` (8037) and `mod_inline` (12257).

**Gates rerun on this copy.**

| Gate | Result |
| --- | --- |
| `cargo test -p raf-source --locked` | 369 passed, 0 failed, 1 ignored (`write_corpus`, pre-existing) |
| `cargo test -p raf-source --locked --release` | 369 passed, 0 failed, 1 ignored |
| clippy `-D warnings` | exit 0, 0 warnings |
| rustdoc `-Dwarnings` | exit 0, 0 warnings |
| `docs_contract` | 139 passed |
| `repo_gates` | 55 passed |
| `make cross-check` | exit 0 |
| `make msrv` | exit 0 |
| `make reuse` | exit 0, compliant |
| carve-out grep | empty |

### Recheck findings

**REQUIRED:** none. REQUIRED 1 is resolved by the committed 36-family boundary
test, which I verified as described above.

**ADVISORY:**

- **R1.** The headroom figures in `R6-shape-coverage.txt` (2.21x debug, 9.4x
  release) come from the implementer's scratch probe with the budgets
  disabled. This review did not reproduce them.
- **R2.** No family covers `///` and `//!` doc comments or a shebang line.
  Adding both to `families()` would complete the lexical row.

## Not committed

Nothing was committed or stashed. The only file written to the stage is this
review.

VERDICT: PASS
