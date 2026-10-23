<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Execution acceptance

Current PR: https://github.com/Blackcat-Informatics/rust-agentic-foundations/pull/59
Published signed head: af8b27a428e63dda80eb44e41de72d833d0ff06b, base d061457.
Assessed tree: dbb6535321a4a97c7e7a973f766ad35393b0d278; clean working tree.
Comment 4364372990's reproduced 4,000-site failure is corrected through pinned
native OrderedTable indexing. All 24 library and 60 Gate cases, workspace lint,
native completion, normal signed hooks, affected package tests/doctests, MSRV
and warnings-denied docs passed. Independent cache review is closed. Push, PR
update and metadata refresh exited 0. No known local feedback blocker remains.
Earlier Source count/cross checks on 003de42 and unchanged parser/release/fuzz
evidence remain explicitly reused. Native API and precise admission boundaries:
tasks/indexed-owner-cache-review.md and tasks/native-ordered-table-api.md.
Affected package/MSRV/docs qualification, push, PR update and metadata refresh
all exited 0. Hosted CI remains the delivery owner's merge prerequisite, with
Windows explicitly skipped. [validation.md](./validation.md) is
the current evidence index. Historical raw failures remain unchanged.

| Behavior | Evidence | Result and limits |
| --- | --- | --- |
| All six native matcher kinds and Source code context | raw/pattern-native-completion-final.log; raw/namespace-native-complete.log | Native lifecycle completed for the published feature and namespace commits, with actual bound tests/examples passing. |
| Published gate qualification | raw/pattern-make-check.log; namespace affected-check logs | Full baseline make check passed on 1841a757; affected Census/CLI, lint, MSRV, docs and normal hooks passed on 4666996. Unchanged Source/release/fuzz/cross results reused explicitly. |
| Confirmed transitive miss | raw/transitive-namespace-rustc.log; raw/transitive-namespace-before.log | Compiler accepted the exact case; old Gate returned false clean. |
| Current transitive correction | raw/transitive-namespace-after.log; raw/transitive-native-complete.log | Fully expanded targets drive native namespace eligibility through a bounded WorkList dependency evaluator. Native completion passed before the declaration-frame correction. |
| Declaration-frame correction | raw/alias-declaration-frame-rustc.log; raw/alias-frame-before.log; raw/alias-frame-controls-gate.log; raw/alias-frame-controls-lint.log; raw/alias-frame-native-complete.log; raw/alias-target-commit.log | Compiler accepted the exact witness and old Gate returned false clean. All 58 Gate cases, direct inner module shadow and actual inner reimport controls, cycle refusal, 64-link chain, lint, native completion and normal hooks passed. |
| Prior affected qualification and publication | raw/alias-target-census-cli-tests.log; raw/alias-target-msrv.log; raw/alias-target-doc.log; raw/alias-target-push.log; raw/alias-target-pr-update.log | Package tests/doctests, workspace/fuzz MSRV, warnings-denied docs, push and PR update all exited 0 on 368b3fd. |
| Final forge feedback correction | tasks/forge-feedback.md; raw/review-feedback-runtime.log; raw/review-feedback-lint-final.log; raw/review-feedback-native-complete.log; raw/review-feedback-commit.log | All 59 Gate and 22 library cases, Source count fixture, native completion and normal signed hooks passed. Full paginated review records retained. |
| Final affected qualification and publication | raw/review-feedback-census-cli-tests.log; raw/review-feedback-msrv.log; raw/review-feedback-doc.log; raw/review-feedback-cross-check.log; raw/review-feedback-push.log; raw/review-feedback-pr-update.log; pr.json | Package tests/doctests, workspace/fuzz MSRV, warnings-denied docs, all three local cross-target checks, push and PR update exited 0 on 003de42. Composite unchanged-parser/release/fuzz reuse remains explicit. |
| Final native indexed-cache correction | raw/indexed-cache-before.log; raw/indexed-cache-runtime-final.log; raw/indexed-cache-lint-final.log; raw/indexed-cache-native-complete.log; raw/indexed-cache-commit.log; raw/indexed-cache-census-cli-tests.log; raw/indexed-cache-msrv.log; raw/indexed-cache-doc.log; raw/indexed-cache-push.log; raw/indexed-cache-pr-update.log; tasks/indexed-owner-cache-review.md | Reproduced old default-grant refusal, then all 24 library/60 Gate cases, native lifecycle, normal hooks and affected qualification passed on af8b27a. All publication commands exited 0. |
| Independent ownership and overlay review | tasks/ownership-delta-review.md | Three root/real-item offset collisions repaired. Actual Source item sets govern module HOME and variants; roots are fallback subjects only when no real item contains the hit. Exact whole-file struct/root, module-home, distinct-root and registered-rule controls pass. |
| Native reuse | current PurRDF main 80d2731bd; RAF pin 5db00aaae | Existing native ECMA evaluator and WorkList; existing RAF Rust token model, normal-form passes and literal decoder. PurRDF helper-census remains an unpublished binary, not a public Rust token-pattern/provider engine. No upstream implementation or revision-equivalence claim. |

Token search covers full files, including module-level macro invocations. Only
implementation bodies/initializers receive their existing body normal form;
module declaration names remain literal. Local functions remain within their
outer body context because Source does not independently index their blocks.
Normalized overlays retain original byte/line positions and walk sorted regions
in one tree pass. The original fingerprint production path is unchanged.

Matching work, frontier states and retained hits have explicit native grants;
exhaustion refuses. Byte/C strings do not enter ordinary-string regex matching.
Caller-owned proc-macro spans survive both successful and refused Gate runs.

The original failing root/first-item test and intermediate compile/lint failures
remain in raw logs. They are not reported as successful runs.

The four path matchers are implemented; the earlier unfinished statement is
superseded. Reference execution, grounded preferred-provider policy, remaining
generic/default-goal detectors, repository registration and complete CLI/target
protocols retain their original parent acceptance. This subset closes no parent.
Provider work is separate. Unindexed syntax supplies no source/provider identity.
Windows CI is explicitly skipped under the maintainer's direction; no Windows
runtime pass is claimed. Local cross-target compilation passed in the baseline.
