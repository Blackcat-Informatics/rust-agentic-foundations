**R4 fixed: macro-generated impls, enum globs, macros in trait bodies, shared crate names** (commit d1c4a7f, pushed)

**Owner decisions applied:**
- **A1 (per-item marker):** an inherent impl of a type the index can't see no longer fails the load.
  - Its members are items marked `SelfType::Unresolved`, recorded under the written self path.
  - A path through it is `ResolveError::UnresolvedSelf`.
  - `Workspace::unresolved_impls()` lists them, so a consumer can refuse them on its own terms.
- **A2 (key crates by package):** each target resolves crate names through its own extern prelude.
  - The prelude holds the package's library, the dependencies its kind sees (with renames), and root `extern crate`.
  - Dev-dependencies are seen only by test code and targets.
  - Build-dependencies are seen only by `build.rs`, which does not see its own library, as in Cargo.
  - A crate name that stands for several libraries is `Ambiguous` whatever follows it, through every route: direct, `extern crate`, imports in each namespace, and globs. Trait keys through it are `Unresolved`, with a candidate per library.
  - `Workspace::resolve_in_package(package, path)` disambiguates.

**Other fixes in this item:**
- **CA-F5 / CA-F6:** macro calls in trait bodies are skipped, and globs of local enums bring in their variants.
- **A3 and the R2-review `bin_crate` collision:** items are grouped by target, so an overlap is `Ambiguous` in either file order, and a lib module named like a target segment keeps its items.
- **First-wins removed:** a name bound to several modules now gives a trait key with a candidate through each.
- **`workspace = true` without an entry** is a manifest error.
- **S-15 / G16:** error text fixed.

**Entry points:**
- **katamari `crates/`:** loads (13k+ items, 2 unresolved impls). It failed before.
- **purrdf `crates/`:** loads (83k items). Its two `purrdf` libraries resolve `Ambiguous(2)`, and `resolve_in_package` works for each. It failed before.
- **Registry:** 1,163 of 1,207 crates load, with 0 misplaced items and 0 budget refusals. No crate fails with the inherent-impl, macro-in-trait or enum-glob message; R3 had 100, 14 and 6 of them.
  - Every changed outcome is root-caused in the stage validation index.
  - The remaining self-alias failures (js-sys, wide, core-foundation, bincode, ena) are R5a's.

**Checks:**
- Every new fixture fails on the old code.
- A resolve dump of every public path in katamari, purrdf and this repository was **unchanged** by the final consistency rounds.
- Counting-allocator probes keep held memory ≤ charged for the new data.

**Gates:** raf-source 334 tests in debug and release, workspace clippy, rustdoc, docs_contract 139, repo_gates 55, cross-check, msrv, reuse, and the hook.

**Review:** `tasks/R4-review.md`, VERDICT: PASS after three rechecks.
