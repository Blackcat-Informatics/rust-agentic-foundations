<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Corpus maintenance ownership repair

The unapplied `recovery-corpus-maintenance-structure.patch` passes `git apply --check`. It is metadata only; no product files, compiler, tests, generator, corpus or gate were executed or modified in preparing it.

The source diagnostic reports twenty-one corpus-example test definitions: nine replay tests, nine oracle tests and three copies of the shared diagnostic-level test. Importing whole integration roots into the example made these physical definitions part of the example source surface even though the example has `test = false`. That flag does not erase their source identity.

The repair separates assertion owners from reusable implementation:

- `tests/support/oracle.rs` owns the original non-test generator, evaluation, classification, order checks, minimizers and writer. Their computation and explicit full-search behavior remain unchanged. Only the crate-private test-facing interface visibility changes. The integration root retains the nine original test bodies and its comparator fixture; the example imports the shared implementation directly.
- `tests/support/corpus_writer.rs` owns the original shape writer and temporary writer assertion. The replay target retains its nine original tests and imports this small writer module directly.
- `tests/support/rustc.rs` contains no test definition after the move. The existing level helper becomes crate-visible. Its exact missing/null/numeric-level refusal and valid null-code assertions move to one real `rustc_oracle` integration test. No assertion or malformed neighbor is dropped.
- The example has one support-module import plus direct imports of the two maintenance modules. Neither maintenance module, nor the shared rustc support, defines a test. Ordinary invocation still calls only the two actual scratch writer checks; explicit `shapes --write` and `oracle --write` still invoke the single original maintenance implementations. No ordinary test triggers a full oracle search or writes committed corpus.

Source frontmatter replaces the sixteen duplicated `test_crate::<target>::support::rustc::diagnostic_levels_are_required_beside_a_valid_null_code` bindings with the sole selectable `raf_source::test_crate::rustc_oracle::diagnostic_levels_are_required_beside_a_valid_null_code` owner. Other selectable test paths remain unchanged. Generated bindings rows are deliberately left for production docs synchronization after application. The registry repair belongs to root and is not in this patch.

No change touches `tests/support/mod.rs`, its Scratch owner/imports/Drop, manifests, native dependency selection, registry files, Gate, Executor, exclusions or quantitative limits. The root writer's concurrent native TempDir repair therefore remains independent.

Required checks after application: format and compile actual source targets and example; run the relocated diagnostic test, both ordinary writer tests and the safe no-argument example; run actual replay/oracle targets under their existing settings; synchronize generated bindings and repeat delivered-state test/source coverage. Full oracle classification/minimization acceptance retains its existing cost and meaning. This metadata does not claim any of those checks passed.
