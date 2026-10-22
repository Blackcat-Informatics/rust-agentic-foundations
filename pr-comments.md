# PR #38 — comments and review threads

3 comment(s). **Every one needs a disposition** in
`review-debt.md`: addressed (with the commit hash) or declined (with the
reason posted in reply). Bot comments count.

## 1. paudley — 

T6 (reverse coverage: coverage lines, the --reverse alias, ceilings with of: shipping|test|all, exceptions) committed as ff8b258; independent review BLOCKED (unused import broke -D warnings, 12 stale assessments, unpinned excepted/error tables and JSON kinds, no CLI ceiling/exception tests, nested-workspace constraint unpinned); fixed in fec6c21 (pushed); recheck PASS. Nested workspaces reached by the source roots are judged as packages (pinned by a fixture). Checks: cargo test -p raf-workplan -p cargo-workplan, docs_contract/self_workplan/repo_gates, make fmt clippy doc deny reuse issue-refs, cargo workplan check: all exit 0.

## 2. paudley — 

T7 (the debt record type, the census-semantics ratchet, debt --write; built and proven on fixture repositories; this repository does not adopt strict, by the scope decision) committed as 4449ceb with review fix dc1115b, pushed. Review BLOCKED (a broken doc heading, one debt kind not written through, the completion check with a debt record unproven), fixed, recheck PASS. E2 fixture updates are listed with reasons in the stage validation index. Checks: cargo test -p raf-okf -p raf-workplan -p cargo-workplan, docs_contract/self_workplan/repo_gates, make fmt clippy doc deny reuse issue-refs, cargo workplan check: all exit 0.

## 3. paudley — 

Integrated the delivered census/source-root foundation and the maintainer's local policy commits through ordinary merges, preserving the existing strict implementation and unfinished adoption work.

Fixed a reproduced reachability bug in `07169f5`: packages of different editions sharing one physical source file reused the first edition's keyword-filtered runs. The cache now uses file and edition. Tests cover combined versus separate roots, reversed root order, swapped editions, raw-identifier neighbors, the real workplan line-coverage caller, and exact/one-byte-short retained-memory limits.

Focused verification passed: 24 reachability integration tests, 15 reachability unit tests, 19 line-coverage tests, 5 touched-surface tests, denied-warning clippy and workspace all-target compilation. The normal commit hook passed formatting, REUSE, 31 document-contract tests, 68 repository gates and the self-workplan gate. No bypass was used.

This is an intermediate repair, not completion of the revised contract. Shared RDF results, removal of the alternative coverage policy, actual zero-debt repository adoption and final full qualification remain in the same delivery. Parent RDF scope stays open for census debt and CLI consumers.

