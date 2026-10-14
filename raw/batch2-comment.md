**Remediation batch: R5b, R-CI, R6, R7, R9, R10, R12s and a BOM span fix** (all pushed)

**R5b: import cycles settled by rustc's rules, independent of order** (5a5e499)
- **Namespaces:** names settle per namespace. An explicit import shadows globs only in the namespaces it binds.
- **Cycles:** a cycle that exactly one glob answers resolves through it. Cycles with no answering glob, or more than one, are `Cyclic`.
- **Visibility:** paths are read with rustc's accessibility rule, and an item is as visible as its most visible glob route.
- **Bounds:** both settle-loop bounds are checked in release.
- **Performance:** chains of 20,000 imports and import cycles settle in under a second, where they took minutes before.
- **Evidence against the rustc oracle:** 24,000+ generated programs, plus fresh-seed private-glob runs.
  - Programs with no order dependence and no panics.
  - Wrong `Cyclic` refusals: 0, down from 38.
  - Wrong `Ok` results on valid code: 12 queries in 10 of 5,522 programs, down from 96 in 71 on the old code.
- **Real code:** 117 imports in katamari and purrdf now bind the public item that rustc picks, not another crate's private module.
- **Stated limits:** two rustc visibility rules are not modelled, and the Contract lists them. Recording them as accepted limits was the orchestrator's call; the owner can overrule it.
- **Review:** `tasks/R5b-review.md`, PASS after four rechecks.

**R-CI: findings from the first hosted runs** (4d56b6f)
- Fixes for the floating nightly's new `map_or_default` lint. The local toolchain is updated to match CI.
- On macOS, the non-UTF-8 listing test now asserts that APFS refuses the file name.

**R6: one additive parser recursion budget** (b187300)
- **Gap:** G1 (CRITICAL). The Contract's carve-outs for known overflow shapes are removed.
- **Counter:** one per-level count of the tokens that start a nested syn parse, with reset rules and a limit of 1927. That limit is 64 MiB divided by twice the costliest measured debug unit.
- **Results:**
  - All 35 committed overflow shapes are refused in both debug and release, and none aborts. The old code aborted 39 times across them.
  - 39 shape families each load at their largest admitted size and are refused one size more. The tightest has 2.2× debug stack headroom.
  - The registry scan has 0 refusals; its highest score is 987.

**R7: fuzz reach** (a26e75d)
- **Targets:** three, `parse-source`, `parse-document` and the new `load-workspace`.
- **Limits:** a 400 KB input cap, plus resident-memory and allocation limits.
- **Fixed in review:** `--cfg fuzzing` made `proc-macro2` drop span locations, so `load-workspace` never reached the resolver. The targets now build with `--no-cfg-fuzzing`, a seed-load test guards against a repeat, and coverage rose from 4,501 to 11,772.

**R9: an empty root list is a typed `NoRoots` error** (bd6d3aa)
- This is the owner's decision "Typed NoRoots error".
- G15's impossible branch is removed.

**R10: manifest and error-text hardening** (655834b)
- **Manifests:** `members` and `exclude` are capped at 4096 entries each, and patterns are compiled once. The 880 KB probe took 2 minutes and now gets a typed refusal in 2 ms.
- **Error text:** every error `Display` escapes control characters, bidi controls included, and so does `PathError`.

**R12s: Verification tests on the real load path** (20b59c5)
- 8 MiB cap tests through a load.
- A real parse-thread `Spawn` failure under `ulimit`.
- A guard that the root manifest keeps `missing_docs` set to deny.
- CRLF, BOM and non-UTF-8 source loaded through `Workspace::load`.

**BOM span fix** (71bd2c7)
- **Bug:** found while writing the R12s tests. A leading BOM shifted every item span on line 1 by 3 bytes.
- **Fix and check:** line 1 now starts after the mark. On the old code, the test reports the first item starting at byte 0 instead of 3.

**For every item:** fixtures fail on the old code or carry a recorded mutation; there is an independent review file under `tasks/`; and clippy, both test profiles, docs_contract and repo_gates were green at each commit, with the hooks run.

**Still to do in Stage 2:**
- R5c: a budget for glob scope tables, which fixes web-sys's memory.
- R5d: resolve-path validation, plus `ResolveError` escaping.
- R11: API sealing.
- Then one `make check`, green hosted CI and a fresh completion audit.
