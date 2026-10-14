# R11a review

## Scope
Uncommitted change in /tmp/raf-par2/b-r11a (base 44e18b7): `Workspace::items_under` whole-segment matching, `SourceFile::package` rule fixture, `tests/docs_contract.rs` role checks, and the matching `source-index.md` text.

## Evidence
- `cargo test -p raf-source --locked`: all suites pass (51 unit, 26/25/46/72 etc. integration).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean.
- `cargo test --locked --test docs_contract --test repo_gates`: 141 and 57 pass; the repository's real docs satisfy both role rules.
- Falsifiability on `git archive 44e18b7` copies (/tmp/raf-r11a-rev):
  - `items_under_matches_whole_path_segments` FAILS on 44e18b7 (returns demo::a1, demo::ab, demo::a_fn alongside demo::a).
  - `a_file_reached_by_targets_of_two_packages_...` PASSES on 44e18b7 (correct, as the plan allows) and FAILS under the mutation `reached.first()` -> `reached.last()` (left "alpha", right "zeta"). Mutation rule met.
  - docs_contract: with both role predicates neutralised, `a_tests_binding_must_name_a_function_marked_test` and `a_code_binding_must_name_a_shipping_item` both FAIL. (`repository_bindings_resolve_in_the_source_index` also fails in that archive copy, which lacks the real-docs setup; not investigated, not a finding about the change.)

## Correctness
- items_under: order is sorted by path string, so `take_while(starts_with)` over the partition point still sees every candidate; siblings such as `demo::a1` sort between `demo::a` and `demo::a::A` and are skipped by the filter, not by ending the run. Exact item `demo::a` is kept (rest empty); `demo::a::` yields nothing (tested); empty prefix yields all; a crate-root prefix `demo` matches `demo` and `demo::*` but not `demox`. Correct.
- Role check uses raf-source's real facts: `TestRole::TestFn` and `Workspace::is_shipping` (test == None and shipping target). A `#[cfg(test)]` module's children inherit the test role via reach_test, so the `demo::unit::helper` fixture shows a cfg(test) item cannot pass as shipping. Items in tests/ targets are non-shipping.
- Package rule: `source_file` takes `visited.reached.first()`; the fixture proves the target-order (manifest path: zeta before alpha) reading.
- Docs: the false "contiguous run" claim is gone; Contract text matches behaviour. No process wording found.

## Findings
REQUIRED: none.

ADVISORY:
1. `items_under` still walks the whole `starts_with(prefix)` run, so a prefix with many same-stem siblings (`demo::a` with thousands of `demo::aN`) does linear extra work; harmless at current scale.
2. The `code` check uses `any`, so a binding that resolves to several sites flags if any is non-shipping; matches the stated rule, just worth knowing.

VERDICT: PASS
