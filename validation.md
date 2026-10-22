<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Shared native primitive acceptance

Scope: plan.md, committed as 164904e with normal hooks, PR 54. This independently
complete delivery excludes strict schema/policy, resolver algorithms and the new
RDF report model. Parent adoption/provider requirements remain open.

| Criterion | Current result and execution scope |
| --- | --- |
| Actual callers | Full workspace all-target compilation passes: raw/native-shared-callers-current.log. Census uses one pinned native hash/lex identity; no external BLAKE3 remains |
| OKF behavior | Seven integration targets pass 87 tests: raw/native-okf-focused.log. Timestamp phase control covers 17 refusals and five valid neighbors; exact decimal spelling, signed zero and closed scalar escapes retain their policy |
| Allocation controls | Native calling-thread probes retain original allowances and 1 GiB process Cap. Both actual targets pass with added nonzero observation controls: raw/native-okf-probe-controls.log |
| Census behavior | All targets pass 139 actual tests, unchanged full fingerprint/report goldens and selection counts: raw/native-census-callers-goldens.log. Zero-test examples are compilation only |
| Real selection output | Actual Corpus/Input/Refusal IDs match independent b3sum pins. Mixed valid/missing-source run exits 1 with actionable refusal; accepted-only neighbor exits 0: raw/native-selection-identities.ttl, native-selection-valid.*, selection-id-reference.txt |
| Contracts/lifecycle | Six exact sections individually judged accepted; tasks/queue-okf-assessment-review.md and queue-census-purpose-examples-assessment-review.md. Actual reconciliation and completion exit 0: raw/native-shared-reconcile-current.log, native-shared-complete.log. The completed record was deleted |
| Required local checks | raw/native-shared-make-check-current.log passes fmt, warnings-denied Clippy, workspace tests/doctests, release tests, docs, deny, REUSE, MSRV and all five 5,000-run fuzz targets. It then exits 2 because required platform targets were absent. Installed all three into the exact same compiler; raw/native-shared-platform-and-final-gates.log exits 0 for resumed make cross-check licenses issue-refs. All required check coverage is complete on unchanged product bytes/compiler. The interrupted make check invocation itself did not exit 0 |
| Normal commit hooks | make hooks, normal commit and push exit 0. raw/native-primitives-commit.log includes staged-index fmt/REUSE/repository/document/self-workplan checks. No bypass or warning/limit change |
| Hosted checks | All twelve jobs pass for PR head 164904e; pr-checks.txt and run 39715743519. Linux executes the workspace; macOS/Windows execute their configured selections, not every workspace target on each OS |
| Independent review/feedback | Final source delta has no unresolved finding. Four missing assessment headers were repaired through actual CLI metadata, confirmed by reviewer and REUSE. Native reviews/inline comments/threads and stagectl top-level comments are empty; review-debt.md records dispositions |
| Integration | Fresh base 1b22912/head 164904e produces a clean merge tree equal to the qualified head; integration.md records actual commands and scope. Actual publication/archive/landing verification remains pending |

Historical failures remain intact: first caller compile caught an unrelated Debt
fixture; the next caught a remaining external Hash::to_hex expectation. Both were
repaired without changing expected bytes. First full invocation correctly refused
two stale Census assessments; fresh individual judgments and paired SPDX metadata
publication repaired them. Initial absolute-path toolchain component installation
refused; installing through the named installation underlying that same managed
symlink succeeded. No failed execution is represented as a successful invocation.
