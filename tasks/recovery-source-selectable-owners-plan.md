<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Selectable source assertion owners

Read-only diagnosis of ten failures in `raw/recovery-strict-completion.log`. No builds/product edits. Keep Executor's refusal of example-local tests and ignored selected tests. This plan does not resolve the other fourteen completion failures.

## Eight registry-scan tests

Add `crates/raf-source/tests/registry_scan.rs` with a path module importing `../examples/registry_scan.rs` under the name `registry_scan`. The existing eight real test bodies can remain in that implementation's `#[cfg(test)] mod tests`; they become selectable integration tests without copied parser/loader/formatter code. Use a narrowly scoped, reasoned dead_code allowance on that imported module if required: its real production entry/orchestration are compiled but not called by each unit fixture. Do not add dummy references. Existing tests/support/mod.rs already documents this pattern for subset-used fixture helpers.

Change the example declaration from `test = true` to `test = false` so these same bodies do not execute again as an example test target. Keep its normal executable example and its existing examples binding, which runs the safe no-argument in-memory demonstration. This is not a registry-wide orchestration proof; the example's own comments already retain that limit.

Replace each `raf_source::example_crate::registry_scan::tests::<name>` binding with `raf_source::test_crate::registry_scan::registry_scan::tests::<name>` for these exact suffixes:

- child_arguments_carry_the_directory_and_the_limit
- count_callers_preserve_padding_width_suffixes_and_misplaced_totals
- ok_line_prints_work_last_among_the_usage_figures
- tally_keeps_the_largest_work_and_counts_budget_refusals
- the_child_is_given_the_limit_it_was_asked_for
- the_work_limit_reaches_the_load
- version_components_and_prerelease_precedence_keep_their_width_and_order
- work_limit_refuses_zero_and_names_the_flag

Do not broaden the version helper's permissive grammar into a Cargo/SemVer identity claim. These assertions pin the actual scanner's numeric/ordering behavior. Preserve the real default-work versus small-work loading neighbor and actual child-argument assertions.

## Two corpus maintenance writers

Both current `write_corpus` entries are ignored because they mutate committed files, not because they are ordinary refused/accepted fixtures. Removing ignore without changing the destinations would make routine completion rewrite repository inputs. Dropping their bindings would not supply the missing writer assertion.

Smallest option retaining the existing selectable names: convert each to an ordinary nonignored filesystem writer test with an explicit destination and a separate opt-in maintenance mode. Extract the actual filesystem write phase into a destination-parameterized helper; the normal test uses existing `support::Scratch`. A narrowly named explicit output-path environment variable can retain manual generation through the same integration target. Ordinary execution must not consult a default committed output directory or silently accept an empty test. Alternatively a separate explicit maintenance executable can call the helper, but is more target/API work and is unnecessary if the safe ordinary test plus opt-in mode is accepted.

For `corpus_replay::write_corpus`:

1. Helper writes all `shapes::all()` entries to its supplied directory using the existing names and exact text.
2. Nonignored test supplies Scratch, asserts the complete sorted file set equals the shape names, and compares every written byte to its actual shape text. Missing/extra/changed text must fail. Existing committed-corpus replay remains unchanged.
3. Explicit maintenance invocation supplies the committed directory, for example through `RAF_SHAPE_CORPUS_OUTPUT`; document this as a writing command. Do not use the oracle output variable for the same directory.

For `rustc_oracle::write_corpus`:

1. Keep existing full generated/evaluate/minimize behavior as the explicit maintenance operation. Factor destination and writing so normal verification cannot write the committed directory.
2. The nonignored default writer proof uses a bounded actual known-class corpus program (the existing minimizer test already requires one), runs the real classification/minimizer/writer path, then checks actual written filename, exact serialized bytes, parsed program/expectation and retained divergence/replay identity. Require a real output; no `None => return` or no-class success. Pass only the selected known class into this bounded proof if the full routine iterates all known classes.
3. An explicit `RAF_RUSTC_CORPUS_OUTPUT` maintenance mode can retain the original all-family generator and diagnostic printing. Its default-free full search is not thereby an executed ordinary-test claim. Existing generated-family, class partition, minimizer and replay assertions continue owning their separate computation guarantees.

This keeps both currently bound `test_crate::{corpus_replay,rustc_oracle}::write_corpus` names only if they become these actual nonignored writer assertions. A cleaner name such as `writing_a_corpus_preserves_its_exact_programs` is also valid, with explicit binding replacement. Do not keep an ignored entry and claim a different helper proves that ignored test ran. The parser-oracle comparison/classification tolerances and corpus contents must remain unchanged.

## Documentation and qualification

Source frontmatter needs the eight actual integration-path replacements. Keep or deliberately replace the two writer names according to the implemented nonignored tests. Current writers have no standalone tested-by marker outside frontmatter/generated matrix, so do not invent a universal runtime-generation guarantee. Add a precise Verification sentence distinguishing safe scratch writer assertions, manual committed-corpus generation and the real replay/generated-family owners. Update actual manual invocation comments. Regenerate the verification matrix from surviving bindings.

Run actual list/run selection through the production verifier for the new integration paths, ordinary writer tests and unchanged no-argument example. Inspect one selected test rather than a zero-test target. Then replay the affected complete target families and required current contexts as invalidated. No existing default/claim/executor gate should be weakened, and no full generator search is inferred from a bounded scratch writer proof.
