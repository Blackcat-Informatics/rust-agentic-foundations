# T5 review: commits 87280b0, 1f66acc, a26e36d (HEAD a26e36d)

VERDICT: BLOCKED.

Existing behaviour is unchanged. Every resolution answer I dumped is byte-identical
between da83cdc and a26e36d, and the oracle lists are identical for three seed sets.
Target resolution passed all seven adversarial layouts I tried. `impls()` matches an
independent `syn` walk of this repository exactly (584 of 584). The gates pass.

Five things still need fixing (R1-R5):

* The scanner contradicts its own Contract on `const` and `gen`.
* The counted-work test cannot see a product implementation over types and impl blocks. My mutant MA survives the whole suite.
* The dev-dependency rule of test code has no fixture for a `#[cfg(test)]` helper. My mutant MG survives.
* Two committed sentences say a foreign root is refused. It is refused only when its index is out of range.

## How the review was done

* I did not edit the worktree. I made four in-repo clones under `scratch/`, each with
  `git clone --no-hardlinks` and a `pwd` check:
  * `t5review` at a26e36d, for the gates;
  * `t5review-base` at da83cdc;
  * `t5review-adv` at a26e36d, for the probes;
  * `t5review-mut` at a26e36d, for the mutants.
* Builds went only to the target directories the cargo wrapper assigned. Each clone's
  `target` is the wrapper's symlink, not a directory:
  `/opt/.cargo/target/{t5review-a6544a1a,t5review-base-2219880b,t5review-adv-a67dff6d,t5review-mut-ed1697f0}`,
  plus the `build-dir` of `~/.cargo/config.toml` under `/opt/.cargo/<hash>/`.
* There were no timing runs. Work was counted with the crate's `work::measure` tallies.
* My tools are in `scratch/t5review-tools/`:
  * `zz_dump.rs`, the answer dump;
  * `zz_probe_targets.rs`;
  * `zz_impls.rs`;
  * `zz_probe_reach.rs`;
  * `reach_hostile.rs.txt`;
  * `mutants.py`.
* Logs are in `scratch/t5review-logs/`.

## (1) Exactness of existing behaviour: PASS

* **rustc oracle** (`RAF_ORACLE_LIST=1`, release). I ran it at da83cdc and at a26e36d,
  on the default sweep and on the bases `0x5eed0000` and `0xbadc0de0`. The listings are
  identical (`oracle-*.log`):
  * default sweep: 754 probes (344 accepted, 410 refused);
  * `0x5eed0000`: 363 probes;
  * `0xbadc0de0`: 382 probes.
  * Every divergence class is 0 on both sides, so no divergence line was printed.
* **Answer dump** (`zz_dump.rs`, the base API only). It asks, for every library item and
  module:
  * `resolve` and `resolve_in` in each name space;
  * `resolve_from` from every module for every name of its crate;
  * `self::`/`super::` from parent and child;
  * every lib path and `crate::` path from each crate root.

  The answers are printed as paths and sites, not as ids. I ran it over:
  * the 34 programs of `fuzz/corpus/rustc-oracle`;
  * this repository at da83cdc and at a26e36d, each tree loaded by both resolvers.

  All four comparisons are byte-identical (`dump-*`). The repository dumps have 255,649
  and 261,494 answers, over 3,835 item paths and 193 modules.
* The test diff da83cdc..a26e36d only adds lines. `use_resolution` and every other
  existing fixture are untouched and pass.

## (2) Target resolution: PASS

Probes are in `zz_probe_targets.rs` (log `probe-targets.log`). Each answer below is the
right one.

* **L1:** a support file that test `a` includes through `#[path]` and test `b` through `mod common; pub mod util;`.
  * It is two modules, one site each.
  * `super::one` and `super::super::two` read inside their own target.
  * The other target's path is `NotFound`, and `test_crate::common::util` is `UnknownModule`.
* **L2:** `src/main.rs` with three inline levels and `x.rs`/`x/y.rs`.
  * `self`, `super` and `crate::` all resolve.
  * With no library, `demo` and `demo::x` are `UnknownModule`.
* **L3:** two bins with a `cli` module each, and a third bin that includes `one/cli.rs` through `#[path]`.
  * Each module is separate.
  * `crate::only_one` resolves from `one::cli` and is `NotFound` from `three::cli`.
* **L4:** an example with a `#[path]` module and a directory example.
* **L5:** `build.rs` with `mod gen;`, and `build = "tools/b.rs"` with a module.
  * Both are read.
  * The library is `NotFound` from the build script.
* **L6:** a test `it` beside the library modules `test_crate::it` and `test_crate::it::inner`.
  * Both modules are `Ambiguous`.
  * A module only the library has reads.
  * `my-test` is read as `my_test`.
  * A directory test with a module reads.
* **L7:** two packages each with a test `it`.
  * Each reads its own target.
  * A dev-dependency is visible from the test and not from the library.

## (3) `impls()`: PASS on completeness, order and spans (behaviour notes A4, A6)

* **This repository** (`zz_impls.rs`, `impls-repo.txt`):
  * `impls()` lists 1052 blocks over 584 distinct (file, line) pairs. The extra copies come from support files reached by up to 17 test targets.
  * An independent `syn` walk of every loaded file finds 584 module-level blocks: no missing, no extra, no member-count difference.
  * Ids equal positions and `impl_block` round-trips.
  * The order is strictly increasing by (file, start, module, target).
  * Every span starts with an attribute, doc or `impl` and ends with `}`, and every `line` holds `impl`.
  * There are no impls inside blocks in this repository.
* **`grep '^\s*(unsafe )?impl\b'`** gives 662 lines; 78 more than `impls()`. Every one is inside a string literal (fixture text in `symbol_index.rs`, `trait_keys.rs`, `streaming_code.rs`, `use_resolution.rs`, `reachability.rs`, `facts.rs`, `bound/tests.rs`, ...) or in the body of the `validated!` macro (`raf-workplan/src/exec/pattern.rs:363,380`).
* **Shapes** (`probe-reach.log`, `zz_impl_shapes`):
  * These are listed, with keys: `unsafe impl`, a negative impl, a blanket impl over `G<T>`, an impl through an alias, `crate::S`, `dyn Tr`, `[S]`, `(S, S)`, `fn()`, a block in a `#[cfg(any())]` module (cfg never evaluated), two cfg-alternative blocks, a doc+attribute block (span from the doc, line at `impl`), and `super::S` from a child module.
  * These are not listed: a block in a fn body (stated), a block inside `const _: () = { .. }` (not stated, A4), and a block from `gen!(S)` (covered by the general "a macro invocation ... is not expanded" sentence).
* **Test roles** follow the items' roles: `CfgTest` in a `#[cfg(test)]` module, `None` in a test target, as its items are.
* **Budget:** the charge is made at `fact.line` (`impls.rs` `record_block`). My mutant ML (`line` = span start) fails `every_impl_block_is_listed_in_file_order_...`.

## (4) Reachability

* **Node and Edge semantics agree with the Contract** (`probe-reach.log`):
  * A type contains its members and gets impl-of to every keyed block.
  * A block is reached from its header (an alias reached from `impl A`).
  * A variant or field type is reached as the enum's or struct's own tokens.
  * A trait reaches its default method and associated items, and those reach their own paths.
  * A `#[cfg(test)]` module root contains its test and helper.
  * Macro-home and `#[macro_export]` read as stated.
* **Determinism:** two runs are equal, and the answer is breadth first from the roots sorted by node.
* **Hostile input** (unit tests appended in `t5review-adv`, log `hostile.log`, release, counted). Each ends with a typed result.
  * **Delimiter nesting of 100,000:** the load refuses it with "exceeds the budget for delimiter nesting (128)".
  * **The walk's own lexer, 1,000,000 levels:** `Ok`, 1,000,000 tokens, no stack overflow.
  * **120 levels with a path at the bottom:** reached.
  * **A one-million-edge star** (four files of 250,000 `h();` each; one file alone exceeds the 2,000,000-token budget):
    * reached 5 nodes;
    * ReachSteps 3,000,018, ReachPaths 4, ReachUnits 1,000,004.
  * **A distinct star of n = 10k/20k/40k:** every counted kind grows exactly linearly (ReachSteps 4n+4, Lookups 3n+1).
  * **A 3-cycle:** 3 nodes.
  * **A glob cycle with a 20,000-segment path:** SegmentReads 20,004, ReachPaths 1, under a ceiling of 8 per segment.
  * **50,000 segments that name nothing:** SegmentReads 2.
  * **120 nested inline modules:** 361 nodes, two runs equal.
* **The approximations** (`zz_reach_approximations`):
  * A local, a parameter and a fn-local struct that share an item's name reach that item. Stated.
  * A macro body is read from the macro's module: `crate::m::mk!()` invoked in `n` reaches `m::helper` and not `n::helper`. Stated, and wrong in both directions as the Constraints say.
  * Edition 2015:
    * `::b::g` reaches `crate::b::g` (right);
    * `b::g` from `a` reaches nothing (right; rustc refuses it);
    * `self::sib` reaches it (right);
    * but `use b::g;` inside a fn body, which 2015 reads from the crate root, reaches nothing (A3).
  * 2018: `::b::g` reaches nothing (right).
* **The counted-work test is discriminating for runs and not for types:** see R2.

## (5) My mutants (`mutants.py`, clone `t5review-mut`, full `cargo test -p raf-source`, clone restored and diff-checked)

| Mutant | Result |
| --- | --- |
| MB per unit, a counted scan of every run of its file (items x references) | killed: `reach::tests::the_work_grows_..._not_their_product`, `the_work_of_a_walk_is_counted_exactly` |
| MM a trait contains nothing | killed: `a_member_is_read_from_the_module_of_its_block_and_a_trait_member_from_its_trait`, `an_impl_block_root_reaches_its_members_and_what_its_header_names` |
| MC a one-segment run is not read in the macro name space | killed: `an_item_a_macro_generates_is_not_reached_and_a_macro_named_by_a_reached_item_is` |
| ME an unresolved inherent block records no members | killed: `every_impl_block_is_listed_in_file_order_with_its_span_module_self_type_key_and_members` |
| ML `ImplBlock::line` is the span start | killed: same |
| MD a module path of two crates reads the first crate's module | killed: `a_module_of_the_library_at_the_crate_path_of_a_test_target_is_ambiguous_with_it` |
| MA per type, a counted scan of every impl block (types x blocks) | **SURVIVED** the whole suite (R2) |
| MG only `TestFn` is test code; a `CfgTest` helper is read as library code | **SURVIVED** the whole suite (R3) |

## (6) Documents

* **Proposed and delivered text.** The three PROPOSED paragraphs were removed one per
  commit, and no PROPOSED block is left in `source-index.md`. The state stays
  `implementing`, so the INVENTORY is unchanged. The capability follows AGENTS.md
  section 6: fixed sections, and current text outside a proposal.
* **Bindings.** All 34 new tests and the ten new public items are bound. The
  "unbound" names my grep found are functions inside fixture strings.
* **Added lines.** They hold no issue or PR numbers, no process words and no hashes,
  apart from the assessment digests, which are by design. `make issue-refs` exits 0.
* **Assessments.** I spot-checked `source-index-contract`, `-constraints` and
  `-verification`.
  * Each records `verdict: accepted` over tool-derived inputs, which now include the new symbols.
  * `workplan check` passes, and reconcile reports none of them stale or missing.
  * Their content is undermined by R1 and R5: the accepted Contract text says something the code does not do.

## (7) Gates in `t5review` (all exit 0; logs `gate-*.log`)

* `cargo test -p raf-source --locked`: 640 passed, 2 ignored. The same with `--release`.
* `make fmt clippy doc deny reuse issue-refs`. `clippy` is `--workspace --all-targets -D warnings`.
* `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`: 31, 68 and 1 tests.
* `cargo run -q -p cargo-workplan -- workplan check`. `cargo workplan` is not installed on the PATH, so I ran it through the package.

## Required fixes

**R1. The scanner treats `const` as declaring wherever it appears.** Location:
`crates/raf-source/src/reach.rs:228-230` (`DECLARES`, used by `Level::declaring`
`:299-316`).

* `fn f(p: *const Helper)` reaches `Other` through `*mut Other` and does not reach `Helper`.
* `&raw const VALUE` reaches nothing (`zz_scanner_edges`).
* This contradicts `docs/capabilities/source-index.md:1494`, which skips only "what ... `const` ... declares".
* `const C: T` is already dropped by the single-`:` rule, `const fn f` by `fn`, and `<const N: usize>` by `:`. So `const` gains nothing in `DECLARES` except this false negative.
* Fix: drop it, or treat it as declaring only when `*` or `raw` does not precede it. Add a fixture beside `a_lifetime_a_metavariable_a_declared_name_and_a_bound_name_are_no_run`.

**R2. The counted-work claim is tested on functions alone.** The Contract
(`source-index.md:1496`, "The work of a walk grows with the items and the references
... and not with their product") and `reach::tests::the_work_grows_...`
(`reach.rs:1067-1084`) use `fan()`: functions and calls only, with no types, impl
blocks or modules.

* Mutant MA scans every impl block for each reached type, counted under `ReachSteps`, which makes the work types x blocks.
* MA survives every raf-source test.
* Fix: add a counted fixture of n types, each with an inherent and a trait block and k members, inside modules opened by a root. Assert equal increments in n and in k, and exact counts. MA must fail it.

**R3. The dev-dependency rule has no fixture for a non-test helper of a
`#[cfg(test)]` module.** Mutant MG
(`reach.rs` `let test = item.test == TestRole::TestFn`) survives.
`reachability.rs:526-550` only reaches from a `#[test]` function. The Contract reads
"test code as test code, which also names the dev-dependencies". Fix: a `CfgTest`
helper that calls `b::thing()` reaches it.

**R4. Overclaim of foreign roots.** The claim appears in:

* `CHANGELOG.md:103` ("a root of another workspace is a `ReachError`");
* the test name `reachability::a_root_of_another_workspace_is_refused_beside_one_of_this_workspace` (`tests/reachability.rs:402`);
* its Verification line (`source-index.md:2271`).

`ItemId` and `ImplId` are bare indices (`reach.rs:856-858`), and the test passes only
because the foreign id is past the end. An in-range id of another workspace is
silently read as a node of this one. The Contract sentence ("a root that is not a
node of the workspace") is accurate. Fix: say "an id past the items or blocks of
this workspace" in the CHANGELOG, the test name and the Verification line. The
alternative is to make ids carry their workspace.

**R5. `gen` is never a segment.** `KEYWORDS` (`reach.rs:213-219`, "the strict and
reserved keywords of every edition") blocks `gen` in edition 2021 crates, where it is
an ordinary identifier. In my probe, `pub fn f() { gen() }` does not reach `fn gen`,
while `r#gen()` does. The Contract says "every other keyword is no segment", and in
2021 `gen` is not a keyword. Fix: read keywords by edition, or state "the keywords
of every edition, `gen` included" in the Contract and Constraints.

## Advisories

* **A1. `Self::helper()` reaches nothing.** A bound method that calls a private
  `Self::helper()` leaves `helper` unreached (`zz_scanner_edges`).
  * The Contract states the scanner rule (`Self::new` gives no run), but the Constraints' list of what is not reached (`source-index.md:1778-1784`) omits it.
  * It is the most common form of private call in this repository, so it will dominate T6's orphan lines.
  * Consider resolving `Self` in a block to its self-type key, or at least listing it.
* **A2.** `where Helper: Clone` drops `Helper` (the single-`:` rule). This follows the stated rule, but the limitation list does not mention it.
* **A3.** A 2015 `use` inside a fn body is crate-relative in rustc. Reach reads it from the module and finds nothing. This is not stated.
* **A4.** `impls()` does not list a block inside `const _: () = { .. }` or any other block expression. Only "inside a function body" is stated (`source-index.md:1492`, `workspace.rs` `impls` doc). "Every `impl` block of every target" overclaims for these and for macro-generated blocks.
* **A5. The walk's retained memory is outside every budget.**
  * Each run is a 64-byte `Run` plus two heap allocations plus its placement entry, kept for every lexed file until the walk ends.
  * For dense input (`a;` per run) that is about 50x the source bytes.
  * It is linear and bounded by `TotalBytes`, but outside the load's budget envelope.
  * Consider storing byte ranges instead of `String`s.
* **A6.** A negative impl records `trait_path` "Sync" for `impl !Sync for G<u8>`. This is nightly-only.

## Outside the repository

* The implementer's disclosed `/tmp/x.patch` is gone.
* I found nothing else of T5's outside the repository.
* There is a pre-existing leftover pattern that is not T5's: every run of the raf-source unit tests leaves `/tmp/raf-source-unit-<pid>-0/` (with `root/d/d` and `probe`).
  * The cause is the best-effort `Scratch` drop in `crates/raf-source/src/tree/disk/tests.rs:24-53`.
  * Dirs of that pattern exist from 03:58 onward, before the T5 commits.
* Another agent left `/tmp/cargo-workplan-evidence-*-3897027-*`, including a `-target` build directory, at 09:02. I ran no cargo-workplan tests.
* **Left by my own runs** (none written by me directly; I removed nothing):
  * raf-source unit-test leftovers of the pattern above, created during my gate and mutant runs. One of the second group may belong to a concurrent agent. I did not delete them; no `rm -rf`.
    * from the gate runs: `/tmp/raf-source-unit-2581454-0`, `/tmp/raf-source-unit-2642708-0`;
    * from the mutant runs: `/tmp/raf-source-unit-3905276-0`, `-3981575-0`, `-4079447-0`, `-4079624-0`, `-4191304-0`, `-118258-0`, `-231134-0`, `-317848-0`, `-378235-0`.
  * `/tmp/katconf.<pid>.{head,got,tail}` for pids 3715905, 3716950, 3719178, 3721458, 3723662, 3725901, 3726941, 3728126, 3728320, 3728530, 3728797, 3730699, 3731821, 3732895, 3734017, 3735112, 3736179 and 3737270 (08:53:42-48). These were written by the global `kache` rustc wrapper during a build in my session window.
  * The agent harness's own output files of my two background shell commands: `/tmp/claude-1000/-home-paudley-Active-rust-agentic-foundations/8b8743eb-ed1f-4370-a50c-d12ba3360f95/tasks/bb8wjmxuj.output` and `.../bh50499h9.output`.
  * The build directories the cargo wrapper assigned, listed above.
