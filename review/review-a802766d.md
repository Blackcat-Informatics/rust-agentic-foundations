# Review of a802766d (delta 49124831..a802766d)

Branch `paudley/232-raf-source-follow-module-declarations`, PR 283, reviewed at 2026-11-16T15:27-06:00.

## Scope

The delta `git diff 49124831 a802766d`, judged against issue 232 and the requirement brief:

1. `BlockMods` in `crates/raf-source/src/facts.rs` (lines 1880-1990) now applies the own test attributes of constants, statics, impl blocks, traits and associated constants, and restores the outer role afterwards.
2. `Builder::block_module` in `crates/raf-source/src/index.rs:918-942` now pushes a `BlockModule` record for every declaring file, not only when `declare` creates the module.
3. `docs/capabilities/census-rule-api.md:99-103` now states which declarations the module-path check sees.

I also checked the tests added or extended in the delta (`module_tree.rs:1470`, `reachability.rs:975`), the source-index binding updates and the two renewed assessments. I read the rest of the branch only where these parts depend on it.

## Evidence

| Command (from the worktree, under `flock /tmp/raf-build.lock`) | Exit | Log |
|---|---|---|
| `cargo test -p raf-source --locked --test module_tree --test reachability --test configured_sources` | 0 | `focused-a802766d.log`: 17 + 51 + 30 passed, 0 filtered |
| `cargo run --locked --quiet` in `probe-a802766d/` (head source) | 0 | `probe-a802766d.log`, `probe-a802766d-2.log` |
| `cargo run --locked --quiet` in `probe-base-a802766d/` (same probe against `git archive 49124831`) | 0 | `probe-base-a802766d.log` |
| `prose-check` on the changed census paragraph, the new comments and the commit message | 0 | `prose-a802766d.log` (empty) |

The probe runs structural and configured (linux) loads through the public API (`Workspace::load`, `Workspace::load_locked_sources`, `Workspace::reachable`).

### Part 1: the test role

- Head probe: modules in each of these positions are tagged test: a `#[cfg(test)]` static, impl, trait, associated const or trait const nested in a plain function; `#[cfg(all(test, unix))]`; `#[cfg_attr(unix, cfg(test))]`; an impl inside a `#[cfg(test)]` function; a const inside a `#[test]` function; a `#[cfg(test)]` function nested inside a const initializer nested inside a function. These are n1 to n12.
- On the pre-delta source, n2 to n6, n8 and n11 are `false`, so the delta is what fixes them. That includes the shape of the new `gated.rs` assertion.
- Restoration holds. Each of these comes out non-test (r1 to r7): a sibling after a test-gated associated const in the same impl, a trait const after a test-gated trait const, a module after all of the gated items in the same function, a module after a nested `#[test]` function, a module after a gated function inside a const, `#[cfg(not(test))]`, and `#[cfg_attr(test, allow(dead_code))]`. The extended test's `plain_const.rs` assertion protects restoration.
- Applying the outer item's role twice (once in `Extractor::blocks` at `facts.rs:1859`, again in `within`) is harmless, because `role` (`facts.rs:975`) is idempotent for a fixed `own`.
- The new `summarize` calls on nested items can only raise the errors that the same items already raise at item-list level. No new class of refusal is introduced.

### Part 2: one record per declaring file

- The only consumers of the records are `reach.rs:994-1024` (owner lookup, filtered by `site.file`) and `platform.rs:467` (`any`, which is insensitive to repeats). Neither `raf-census` nor `raf-workplan` reads them; `raf-workplan` sees them only through `Workspace::reachable`.
- Module identity is unchanged, because `declare` is untouched. Ledger homes resolve through item paths, so they are unaffected. The configured load of the cfg_attr case still has one `consumer::p::{local@18}::g`.
- On the pre-delta source, the same fixture as the new reach test leaves `demo::p::bb` reaching nothing but itself. Owner tokens do not cover the nested module, so containment is the only path. At head, `bb` reaches `hb`, so the new test protects the fix.
- The new records do not cross files or owners:
  - Alternative files at different offsets stay apart: each root reaches only its own helper.
  - A non-owner function `zz` in the second file gains no containment: it reaches `hc` only.
- Repeats: when two `cfg_attr` alternatives name the same file, the file is walked twice under `p`. That duplication predates the delta and is identical on the base: 7 items, two `g`. The delta adds a second identical site, and both duplicate roots now reach the same 4 unique nodes; on the base, one of them reached only itself. Repeated edges are deduplicated by the walk: reached count equals unique count. A default file equal to an alternative is walked once.

### Part 3: the census-rule-api statement

The probe agrees with each listed case:

- Followed (a missing file refuses the load): function body, const, static, impl method, trait default method, trait const, and an expression generic argument.
- Not followed (the load succeeds with no file, and a present file is not loaded): array-length type, enum discriminant, `let` type, return type, type alias, `extern` block, macro invocation, impl associated type.

## Findings

### REQUIRED

**R1. The source-index Contract states the old test-role rule.** `docs/capabilities/source-index.md:2269-2270` says: "The test role of the item that holds the block applies to the module, and that of each function inside the block." After part 1, the role of a constant, static, impl block, trait or associated constant inside the block also applies. For the delta's own fixture (`#[cfg(test)] const C` inside the plain `fn plain`, `module_tree.rs:1478`), the sentence predicts a non-test module, and the code tags it test. The section sits under `## Contract`, which is the specification. The delta renewed `work/assessments/source-index-contract.md` without changing this sentence. Fix: name the items whose own test attributes apply (functions, constants, statics, impl blocks, traits and their associated constants), then renew the assessment.

### OPTIONAL

**O1. Statement-level `cfg(test)` does not set the role.** These two forms come out non-test: `fn f() { #[cfg(test)] { #[path = "s1.rs"] mod s1; } }`, and the same with `#[cfg(test)] let _x = { .. };` (s1 and s2 in the probe, identical on the base). rustc compiles these modules only under test. This predates the delta, which is about item attributes. Either read `cfg` on statements and expressions in `BlockMods`, or add this limit to the sentence fixed under R1.

**O2. The census-rule-api followed list is incomplete.** `docs/capabilities/census-rule-api.md:99` says a module declared "inside a function, constant, static, impl block or trait body" is followed. A module inside a const generic argument of a trait's supertrait bound (`trait U: Tr<{ #[path = "x.rs"] mod x; 1 }>`) is followed too: a missing file refuses. That is not part of a trait body, and the not-followed list does not mention it either. The impl-header case (`impl Tr<{ .. }> for S`) is arguably covered by "impl block". Configured loads drop an inactive statement in both positions correctly (CFGHDR probe), so behaviour is sound. Only the wording is narrower than the code. "impl block or trait" would match `source-index.md`, which says "traits ... are read".

**O3. The extended test's name is narrower than its content.** `module_tree.rs:1474` is still named `..._in_a_test_function_is_tagged_test_beside_one_in_a_plain_function` but now also protects a test-gated constant. Its doc comment was updated; renaming would also touch the two bindings.

**O4. Pre-existing:** two `cfg_attr` alternatives that name the same file index its items twice (see Part 2). This is outside the delta. The delta does not make it worse.

## Not verified

- Hook gates (`repo_gates`, `docs_contract`, `self_workplan`, `self_census`), `make check`, `make test-release`, the raf-source lib unit tests and the other raf-source test binaries (including `load_budgets` for the extra record charge), and the raf-census and raf-workplan suites. None was run.
- I did not run rustc on the probe sources. That rustc accepts a path module inside a const generic argument of an impl header or supertrait bound, and compiles statement-level `#[cfg(test)]` modules only under test, comes from language semantics, not a run here.
- I tested only the nightly host on Linux. I did not check stable, MSRV, fuzzing or other platforms.
- I did not determine which of two duplicate owners receives each repeated site in the same-file-alternatives case. Only the observable reach result was checked.

## Scratch

- Inside the repository (ignored `.stage/`, this directory): `probe-a802766d/`, `probe-base-a802766d/`, `base-49124831/` (a `git archive` export of the pre-delta tree), and the logs and prose inputs named above.
- Outside the repository, created by the cargo wrapper for the two probes: `/opt/.cargo/target/probe-a802766d-d45fc624/` and `/opt/.cargo/target/probe-base-a802766d-d145239a/`. I did not delete them; deletions outside the repository are not mine to make under the repository rules.
- The shared lock `/tmp/raf-build.lock` already existed.

VERDICT: BLOCKED
