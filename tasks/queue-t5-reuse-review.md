<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# T5 donor reuse review

Read-only independent source/evidence adjudication at donor HEAD `dc1115b`, 2026-10-20. No build, test, probe, mutation run, commit or product edit was performed. The sole write is this ignored review artifact. This is not a fresh runtime PASS.

**Disposition: retain T5 implementation, but qualification remains open.** The original R1–R4 corrections and single-edition R5 correction are coherent in source and have historical executed fixtures. One concrete mixed-edition cache/contract mismatch remains to reproduce and repair before claiming the revised reachability implementation qualified. Current integration and production callers still need execution after census/main synchronization.

## Evidence inspected

- Original `tasks/T5-review.md`, verdict BLOCKED against `a26e36d`, including R1–R5 and A1–A6. No historical `T5-recheck.md` exists.
- Fix commit `39717b1e7bb6b0ffc8c43ce0ba8e894a5e0bde84` and current source: `reach.rs`, `workspace.rs`, `index.rs`, reachability integration tests, source-index Contract/Constraints and CHANGELOG.
- `scratch/t5fix-logs/final/test-debug.log`, `test-release.log`, `clippy.log`; intermediate `test-debug-2.log`. Final release log explicitly records the R1 pointer/raw-borrow, R3 dev-dependency, R4 out-of-range and R5 edition fixtures as `ok`; the unit log records expanded kinds/count and budget tests. Clippy log ends in a successful Finished line. These are useful historical implementation evidence, not authenticated fresh exit statuses or proof against later integration. No time measurements in those harness summaries were used as evidence.
- Historical mutant files/logs were discovered, not executed or adopted as a required workflow. The current ban forbids mutation runs. Historical oracle and original review behavior comparisons remain bounded to their recorded revisions.

## Original findings adjudication

| Finding | Current source and neighboring cases | Source conclusion; execution limit |
|---|---|---|
| R1 const falsely declares | `reach.rs:308` DECLARES omits const. `place` still drops a declared fn name and the single-colon rule drops constant bindings. Unit `const_declares_nothing_so_a_pointer_and_a_raw_borrow_name_their_path` tests `*const`, `*mut`, raw const/mut, const declarations/functions/blocks/parameters. Integration `a_pointer_to_a_type_and_a_raw_borrow_of_a_static_reach_them_and_a_constant_its_type` checks actual reached sets and a no-extra-reference const fn neighbor. | Original defect corrected in the intended scanner, with discriminating cases present and historical run evidence. Not rerun now. |
| R2 types × impl blocks invisible to count test | Walk::new builds `blocks_of_type` once by scanning blocks and keyed self types; next() uses that map instead of scanning all blocks for every type. `kinds` creates modules, structs, enums, aliases, constants/statics/functions, inherent and trait blocks and self-calling members. `the_work_over_types_impl_blocks_traits_and_members_grows_with_each_and_not_their_product` checks reached-node totals, equal increments across n/k, and exact values for all six counted kinds. | Correct implementation/index and fixture shape address the original blind spot. The n×blocks regression grows quadratically in n and would violate the asserted equal increments. Do not claim counted work proves uninstrumented instructions or allocations linear; map/sort operations retain their own costs. Exact formulas and count instrumentation must be rerun after integration. |
| R3 cfg(test) helper loses dev dependency | Unit construction uses `item.test != TestRole::None`; impl units likewise use block.test. `test_code_names_the_dev_dependencies_and_other_code_does_not` roots a TestFn, non-test cfg helper and cfg impl method, each expecting b::thing, beside ordinary a::api expecting no b::thing. `references` includes the module/test flag in its deduplication context and forwards it to `reach_run`. | Original missing case and production condition corrected; valid/refusal neighbors cover helper and impl paths. Historical run evidence exists; no fresh execution. |
| R4 foreign-root overclaim | Validation rejects only indices past items/blocks. Workspace public errors, CHANGELOG and renamed test now say precisely that. Test also supplies an in-range foreign item ID and expects the local node at that position. | Accurate bare-index contract restored; no false workspace-provenance guarantee remains in inspected API/test/changelog text. The analogous valid local item is exercised. |
| R5 gen always keyword | KEYWORDS_2018 and KEYWORD_2024 distinguish editions; raw identifiers bypass keyword rejection. Index::edition reads the reached module's target/package edition. The integration case covers bare/raw gen in 2015/2018/2021 and raw-vs-bare in 2024; unit tests also distinguish the 2018-added keywords. | Single-edition fix addresses the original example. Mixed-edition shared physical files remain an unresolved context error described below. |

## Remaining finding: edition-sensitive runs cached by file alone

**Priority: medium, qualification blocker for the currently stated per-reached-item edition contract.** `Walk::load` (`reach.rs:948–1005`) returns immediately when `files_runs.contains_key(file)`. Its first call invokes `lex(file, text, index.edition(module))`; `place`/`blocked` remove keyword runs during lexing. `files_runs` is `BTreeMap<RelPath, FileRuns>` (`reach.rs:855`), without edition. Later references (`reach.rs:1014`) read those already-filtered runs for every target sharing the path.

This loader supports one physical source file with multiple visits/targets: Workspace gathers all target roots, walks them and combines file contents by RelPath (`workspace.rs:198–218`); the index retains per-target editions (`index.rs:299–301`, `1281`). `by_unit` may legitimately share source byte ranges. Therefore a 2024 visit that lexes a shared file first removes bare gen for its 2021 visit too; the reverse visit order admits a bare keyword in the 2024 context. `Ctx::Module(module,test)` resolution deduplication does not repair a run discarded or admitted earlier.

This is **already disclosed** in source-index Constraints (around 1808): a shared file is lexed with the first reached item's edition. It is not an undisclosed behavior discovery. Nevertheless Workspace::reachable API docs (around 527), source-index Contract and CHANGELOG promise the reached item's package edition without this exception. Parent requested the behavior corrected, rather than extending that exception into a waiver. Original R5 single-edition tests cannot detect it.

### Exact reproduction fixture to add on activation

Use one MemoryTree, no disk or clock dependency:

```text
Cargo.toml: [workspace] members = ["a-new", "z-old"]
a-new/Cargo.toml: package name="a-new", version="0.1.0", edition="2024"
z-old/Cargo.toml: package name="z-old", version="0.1.0", edition="2021"
a-new/src/lib.rs: #[path = "../../shared.rs"] mod shared;
z-old/src/lib.rs: #[path = "../../shared.rs"] mod shared;
shared.rs:
    macro_rules! probe { () => { gen() }; }
    fn r#gen() {}
```

The unused macro definition is valid Rust source in both editions: its unexpanded body tokens need not form a 2024 expression. It provides a real shared source/edition neighbor without relying only on an invalid 2024 function body. Resolve each target's `shared::probe` in Namespace::Macro and `shared::gen` in Namespace::Value (using actual normalized crate paths returned by targets). Root each probe alone, then both together. The 2021 probe must reach its gen; 2024 probe must not, under the documented token-keyword rule. Combined output must equal the union of the separate walks and must agree for both input root orders. Ensure the first sorted root belongs to 2024; repeat with package names swapped to cover both first-lex directions. Also retain the original direct-function 2021 fixture and raw r#gen valid neighbor.

This fixture has **not been executed**; actual loader acceptance/resolution details must be verified on activation. The causal cache-key mismatch above is directly observable source evidence, while exact reached sets remain a prediction. If a fixture needs resolution adjustments, preserve the cross-edition same-RelPath condition rather than replacing it with two files.

Smallest candidate correction: retain edition-neutral lexical runs and apply edition filtering in each reading context, if that can preserve the existing single-pass placement/count law cleanly; otherwise key the lexical/placement cache by all lexical inputs `(file, edition)` and update counted/budget/documented laws accurately. Do not claim one lex per file if the latter intentionally lexes once per file/edition. Count retained cache copies, keep refusal diagnostics actionable, and add the paired-context case before choosing the implementation.

## Budget, advisory repairs and callers

The A5 repair is substantive: runs retain segment byte ranges rather than per-run owned strings; resolved texts deduplicate per context. Walk tables, run/segment capacities, placement lists, seen strings, block name data, reached nodes, opened modules and queue high-water charge through Meter::items on top of `Usage::item_bytes`. The public budget integration test loads exactly at the load-held limit, expects typed ReachError::Budget at the first walk charge, and then succeeds with headroom. Unit `walk_charge` searches the exact byte acceptance boundary and asserts the immediately lower refusal; the dense-run test checks linear retained-run cost and the 64-bit 56-byte value. These are count/space assertions, not timing assertions. Transient lexing/temporary vectors are not a proof of exact process RSS; preserve the documented retained-memory scope.

A1 Self/member behavior is now implemented through Home/Owner, indexed member lookup and block/trait name tables; integration tests cover private Self helper, self method across blocks, trait defaults, unknown Self names and enum variants. A2 where-bound type now has a scanner context and tests. A3 edition-2015 fn-local use limitation and A4 blocks inside const/block expressions are documented. A6 negative impl spelling retains !, and trait-name parsing excludes negative impls. These source changes are useful reuse, not fresh execution or broader rustc-equivalence proof.

The production reverse-lines path does use this implementation: `raf-workplan/src/coverage/lines.rs::cover` builds binding/extra roots, invokes `Workspace::reachable`, maps errors to an actionable walk failure and marks item/impl coverage from its result. Workspace forwards the real index, cached source files, original LoadOptions and load-held bytes. There is no observed alternate reach implementation or swallowed budget refusal. Default-policy adoption will make these answers directly gate relevant, so the shared-file context correction and caller integration coverage belong to the grouped delivery.

## Required execution after activation/integration

1. Add/run the mixed-edition shared-path refusal/valid neighbors and original R1–R5 targeted tests against the integrated source. Run count and exact-boundary budget fixtures after the cache fix, updating only genuinely changed laws.
2. Run actual workplan reverse-lines caller tests on shared targets and ordinary bindings, checking orphan/covered outcomes and propagated refusal. Compile affected real callers early, including examples and all targets.
3. Recheck target-resolution/impl indexing and source discovery affected by landed main; preserve original fixtures and contract decisions. Use focused source test groups during development and required release source tests in final qualification.
4. Refresh changed docs/assessments only after actual semantics settle; then required root/full gates and ordinary hooks at the final combined delivery. No new Stage panel or mutation runs are required. Historical passing logs cannot substitute for these current checks.
