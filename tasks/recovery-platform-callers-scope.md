<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Platform outcome caller patch

`recovery-platform-callers.patch` is unapplied metadata prepared from the current strict checkout. `git apply --check` passed. No compiler, tests, lifecycle, publication or build ran during preparation. Root owns applying, formatting, qualification and all production writes.

The patch expects the root-owned `BoundOutcome::NotApplicable { reason: String }`, stable name `not-applicable`, and unchanged false `Evidence::passed()`. It does not change the executor, platform query, bound plan, execution invocation or evidence profile schema.

- Verification accepts either an actual pass or compiler-proven host non-applicability. Its `passed_count()` counts only execution-proof passes; `not_applicable_count()` counts the distinct host result. Failure iteration excludes only these two satisfied cases.
- Completion retains actual binding outcomes in both Completed and Refused results. A pre-execution refusal has empty receipts. Successful ordinary output stays unchanged; non-applicable receipts are printed and projected as actual Binding records. Example obligations use the same explicit distinction.
- Evidence publication refuses host nonexecution with typed `EvidenceError::BindingNotApplicable` before record creation or replacement. Neither a pass nor a fail Evidence record is fabricated.
- Existing machine Binding projection already emits the outcome name, reason, optional proof counts and actual run receipts; no parallel graph builder is added. CLI treats non-applicability as an outcome, so exit status is 0 and quiet mode omits it.

Four new caller tests are proposed: verification (zero passing execution proofs, one non-applicable result); no-publication evidence (exact compiler-query receipt and unchanged complete snapshot); completion (one actual pass plus one retained nonexecution receipt); actual CLI (text, exit 0, machine outcome/reason/no passed count, query receipt, quiet filtering and unchanged snapshot). Non-applicability still requires a real compiler configuration observation; it does not imply zero process launches. The tests inspect that observation, not an invented bound-test execution.

Required focused execution after application includes library verify, complete and evidence targets and CLI verify; ordinary valid/failed/ignored/missing-feature neighbors already in those targets must remain. New tests require genuine opposite-host cfg resolution and the actual Cargo query. Required documentation and bound ownership must be updated by the root writer before lifecycle completion.

The public Completion enum gains receipts, so two existing integration expected-value constructions and its private constructors are mechanically updated. Their prior removal/rewrite assertions remain intact. Full cross-platform hosted execution, full gates, lifecycle completion and acceptance remain unclaimed.

Small API prose adjustment for the root writer: EvidenceError's leading documentation currently says most errors start nothing; `BindingNotApplicable` must be named among exceptions because its compiler configuration query has actually run. VerifyReport and completion Condition 5 prose should explicitly distinguish host non-applicability from an execution proof rather than describe every satisfied result as a passing run.
