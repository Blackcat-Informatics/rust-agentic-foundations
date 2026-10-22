<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Selectable source owners patch scope

Unapplied patch: `recovery-source-selectable-owners.patch`. `git apply --check` returned exit 0 against the current strict source. No product files were edited and no builds/tests were run by the patch author. Root remains the only product writer.

The patch follows the actual Census selection integration pattern: a reasoned path-module import of the real example, with no copied implementation/test bodies. It adds tests/registry_scan.rs and makes registry_scan test=false, so its eight exact existing assertions are selected under the integration target, not duplicated in an example harness. Source bindings become `raf_source::test_crate::registry_scan::registry_scan::tests::<existing name>`. The runnable original example binding remains.

Both existing integration `write_corpus` names become nonignored real temporary writer assertions. The shape test checks the complete written filename set and every exact shape byte. The oracle writer test selects a real recorded class program, checks exact filename/bytes/parsed Program (including expectation/probes), and verifies overwrite of stale output in an owned directory. It does not launch generation, minimization or the compiler. Existing ordinary replay, generator and minimizer assertions retain ownership of those separate computation phases.

A single new corpus_generate example imports the existing implementations, again without a parallel generator. Its no-argument path executes only those safe temporary writer checks. Full maintenance remains explicit:

- `cargo run -p raf-source --example corpus_generate -- shapes --write fuzz/corpus/parse-source`
- `cargo run -p raf-source --example corpus_generate -- oracle --write fuzz/corpus/rustc-oracle`

The full oracle evaluation/classification/minimization body is retained, changing only destination and delegation to the same write helper tested normally. It is not an ignored test or ordinary completion action. No environment flag can accidentally turn a selected ordinary writer test into full search or committed output. The two maintenance functions have a reasoned dead_code allowance for integration contexts where only the writer phase runs; imported modules use the established subset-helper allowance. Conditional test-only imports avoid non-test example unused imports.

Source Usage/Verification and the new example binding are included. The existing two writer test bindings remain valid names with genuine new assertions. The tested-by statement owns exact temporary writing, not a claim that a full maintenance search ran. Generated verification-matrix rows are deliberately not hand-edited: root must run production docs sync after applying to update the eight selectable paths/new example, then recapture affected Source sections.

Qualification remains required: formatting, all-target caller/lint checks, actual selectable list/run receipts for the eight tests and both writer tests, the no-argument original and maintenance examples, and ordinary malformed maintenance argument refusal beside the safe default. Verify physical source groups are owned through actual integration bindings/shared helpers, not duplicate test-body copies. This patch does not address the other completion failures, oracle classification limits or full repository acceptance.
