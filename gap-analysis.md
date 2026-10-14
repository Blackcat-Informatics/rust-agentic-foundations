# Gap analysis: issue 2 / PR 13 (raf-okf + raf-source)

The gap-analyzer agent (opus, xhigh) produced this report inline, because its tool contract does not let it write files. The orchestrator saved it unchanged.

**Issue:** 2, "Shared crates: raf-okf + raf-source"
**Branch:** `paudley/2-shared-crates-raf-okf-raf-source` at 6c744af
**Base:** origin/main (2c41082)
**Files changed:** 144 (+30,583 / −1,811), 13 commits

## Evidence run in this stage

- **Overflow harness, debug:** `cargo test -p raf-source --locked --test overflow_harness composites_and_uncounted -- --nocapture`. 10 of 11 committed shapes **abort**; only `uncounted_move_closure` is refused.
- **Same harness, `--release`:** 4 of 11 abort (`composite_ref_paren`, `composite_double_ref_paren`, `uncounted_return`, `uncounted_fn_pointer`). Five exit 0 with the correct index, and one is refused.
- **Windows test build:** `cargo check -p raf-source --locked --all-targets --target x86_64-pc-windows-gnu` fails. The error is "function `within_watchdog` is never used" at `crates/raf-source/src/tree/disk/tests.rs:48` (dead code, made an error by `-D warnings`). The same check for `x86_64-apple-darwin` passes.
- **8 MiB source through a real load:** the probe crate's `Workspace::load` read an 8 MiB `src/lib.rs` of 424,986 items in 7.2 s (debug). 8 MiB + 1 byte gave `TooLarge` naming `src/lib.rs`, and `load_with` with `max_file_bytes = 999` was honoured.
- **Resolver probe on the real repository** (`DiskTree` over the worktree, roots `["crates"]`):
  - All 116 public re-exports and methods of both crates resolve to their definitions.
  - `raf_source::syn::File` gives **`Err(Cyclic)`**.
- **Resolver probe, `MemoryTree`, edition 2021, `serde` dependency:**
  - `pub use serde;` then `demo::serde::Serialize` gives **Cyclic**. `pub use std;` then `demo::std::fmt::Display` also gives **Cyclic**.
  - `pub use serde;` plus `impl crate::serde::Serialize for String {..}` makes the load fail: "the trait `crate::serde::Serialize` of an impl cannot be named by an absolute path".
  - The same failure happens with `mod a { use crate::serde::Serialize; impl Serialize for String {..} }`.
  - Controls behave correctly: `pub use ::serde;`, `pub extern crate serde;` and `pub use serde::de;` all give NotFound.
- **Licence expressions** (`cargo metadata --offline`): `unicode-ident 1.0.27` is `(MIT OR Apache-2.0) AND Unicode-3.0`, and `libfuzzer-sys 0.4.13` is `(MIT OR Apache-2.0) AND NCSA`.
- **Owner's `make check` log** (`raw/make-check.log`): lines 1365-1366 show `seed corpus: ... max: 360026b total: 1828381b`, then `INITED ... corp: 25/65Kb`. That means `-max_len=4096` cut the seed corpus to small sizes.

## Requirements coverage

The rows not listed here are as in `issue-analysis.md` §2a–2c, which I spot-checked. Status changes against Stage 2 are in bold.

| # | Requirement | Status | Location | Gap |
|---|---|---|---|---|
| IS-1..IS-9, IS-12 | raf-okf parse/profile/writer/body/index; Tree; discovery; targets; `#[path]` containment | COMPLETE | per issue-analysis | – |
| IS-10 / SC-43 / SC-48 | Symbol index and use-alias resolution; trait keys through local re-exports; typed NotFound/Cyclic | **PARTIAL** | `src/resolve.rs:291-305, 423-431`; `src/impls.rs:285-306` | G2 |
| IS-11 | Port provenance | Superseded by user decision 5 | – | – |
| SC-13 | Listing reads entries through the verified handle | PARTIAL (Linux only) | `src/tree/disk.rs:137, 257-291` | G3 |
| SC-15/16 | Other-Unix and portable verifiers | UNVERIFIED at runtime | `disk.rs:486-500` | G7 |
| SC-19b | Spawn error | UNVERIFIED | `src/parse.rs:63` | G11 |
| SC-23/24 | "an input that causes [an overflow] is added to the fuzz corpus **and to the budgets**" (`source-index.md:220-221`) | **UNMET** | `src/budget.rs` (5 budgets only) | G1 |
| SV-13 | Bounded fuzz run | UNVERIFIED (owner's log only) and structurally weak | `Makefile:84` | G6, G8 |
| OV-2 | Extra-key fixture per type | PARTIAL | `crates/raf-okf/tests/profile.rs:170-182` | G4 |
| SV-6 | 8 MiB parses at the default; over-cap and lowered cap through a load | PARTIAL (behaviour correct, no fixture) | `tests/read_cap.rs` is Tree-level only | G5 |

## Acceptance criteria status

| Criterion | Status | Evidence |
|---|---|---|
| Memory fixtures: nested mods, `#[path]`, re-export chains, test/bench/example targets, rejected escape | MET | `module_tree.rs`, `use_resolution.rs`, `discovery_targets.rs` |
| OKF round-trip goldens; unknown-key fixture fails | MET | `render::every_tracked_document_renders_to_its_own_bytes`; `profile::an_unknown_key_is_reported_at_that_key` (Architecture only, see G4) |
| `missing_docs = deny` | MET | workspace lints; both crates use `[lints] workspace = true` |
| Capability docs match what was built; state implementing/delivered | **NOT MET** | Both are `delivered`, but the Contract sentences in G1, G2 and G3 are false, and Verification bullets in G4 and G5 have no fixture |

## Handoff risks, adjudicated

1. **Hostile Rust input aborts, while the docs promise typed refusals.** The abort outcome is in the contract for *unknown* inputs; SECURITY.md describes typed refusal as an "aim". However, main's contract also says known overflowing inputs are "added to the fuzz corpus and to the budgets" (`source-index.md:220-221`, kept on the branch), and the approved plan (C4) says "covered by a budget". The branch committed 11 such inputs; 10 abort in debug and 4 in release. It added budgets for none of them. In Task 9 it added carve-out sentences (`source-index.md:216-218, 519-520`; `SECURITY.md:77-78`) without a user decision. **Gap: G1, CRITICAL.**
2. **Non-Linux containment never runs.** Decision 3 set the verification level at compiling everywhere plus Linux unit tests of the verifier functions, so runtime on macOS and Windows is covered. Two things are not covered by that decision. First, the dispatch glue (`disk.rs:271-290, 486-500`) runs nowhere. Second, the lib test target does not even type-check warning-free for Windows. Separately, the listing claim is false on those platforms. **Gaps: G7 (MEDIUM) and G3 (CRITICAL).**
3. **Resolver correctness rests on fixtures.** Partly rejected: 116 of 116 public paths and about 120 real bindings resolve on real code. Partly confirmed: a cheap differential found G2 on the repository's own `pub use syn;`. **Gap: G2, CRITICAL.** A rustdoc-JSON differential is recommended inside G2's fix.
4. **Licence exceptions vs `deny.toml` policy.** They are consistent with the written policy. The policy governs the `allow` list, which is unchanged. Both exceptions are crate-scoped and carry a reason, and the fuzz copy is pinned by `repo_gates::deny_fuzz_configuration_is_the_root_configuration_plus_one_exception`. The root exception is in the approved plan (C4). What remains is the owner's explicit "look and decide". **Gap: G10, LOW (merge precondition).**
5. **An empty root list silently loads an empty workspace.** This does not break the letter of AGENTS.md "Fail hard": no default is substituted, and the contract specifies the behaviour ("no roots find nothing", `source-index.md:550`; tested at `discovery_targets.rs:271-276`). It is still a vacuous-success hazard that the PR pushes onto every consumer. **Gap: G9, MEDIUM; needs an owner decision.**

## Stage 1 open findings, re-checked

- **Golden count lower bound `> 20`** (`crates/raf-okf/tests/render.rs:79`): acceptable as a non-vacuity guard, no action. The interim `timestamp` check in docs_contract is gone; only `accepts_optional_tags_and_timestamp` at `tests/docs_contract.rs:1088` remains. **Resolved.**
- **Task 2 parity decisions** (HTML-comment links, `[a]()`, one marker error per document): implemented and tested (T8 review). **Resolved.**
- **Task 5 aborts:** escalated to G1. The kebab-case warning is still present: G14.
- **Task 6→7 `mod test_crate;` ambiguity:** `symbol_index.rs:505-532` and `target_roots.rs:91-102`. **Resolved.**
- **stagectl `VERDICT:` parsing:** a tooling issue, no branch action.

## Gaps

### G1: CRITICAL. Known overflowing inputs have no budget; the contract was reworded to exempt them

- **Evidence**
  - `docs/capabilities/source-index.md:220-221` keeps "an input that causes one is added to the fuzz corpus and to the budgets".
  - Lines 216-218 and 519-520 (Constraints) and `SECURITY.md:77-78` were added on the branch to say that no budget counts these shapes. T9 review, orchestrator note 4, describes them as reworded so they "stay true".
  - `src/budget.rs` has five budgets only.
  - The harness results are in Evidence above (10/11 abort in debug, 4/11 in release).
  - Plan C4: "Every abort found is minimised, committed and covered by a budget."
  - No user decision covers this.
- **Consequence:** the delivered contract contradicts itself. Any consumer loading untrusted source (census, workplan) gets process aborts for inputs the project already knows about, and a document was weakened to match the code.
- **Fix**
  - Add pre-scan counters for the uncounted recursion shapes: keyword-prefix chains (`return`/`break`/`yield`/`move`), `&mut`/`*const`/`*mut` type-prefix runs, `fn(..) ->` return-type chains, and right-associative assignment chains.
  - Add a composite budget that bounds the sum, over open delimiter levels, of each level's prefix run. Debug costs about 8.8 KiB of stack per level, so roughly 7,400 levels fit in 64 MiB; a limit around 2,048 units admits real code.
  - Write each counter's definition into the Contract with a neighbouring-valid fixture.
  - Change the harness and corpus expectations for these shapes to `Refused`.
  - Delete the carve-outs at `source-index.md:216-218, 519-520` and `SECURITY.md:77-78`.
  - If the owner instead accepts aborts for known inputs, that must be recorded as an explicit owner decision, and it removes "and to the budgets". That is a contract weakening and must be labelled as one.
- **Acceptance**
  - `cargo test -p raf-source --locked --test overflow_harness composites_and_uncounted -- --nocapture` prints `Refused { .. }` for all 11 shapes, in both debug and `--release`.
  - A tightened assertion fails if any committed shape returns `Aborted`.
  - `grep -n "No budget counts" docs/capabilities/source-index.md SECURITY.md` returns nothing.

### G2: CRITICAL. The resolver mishandles `pub use <dependency>;`

- **Evidence**
  - The probes in Evidence above.
  - Real repository: `crates/raf-source/src/lib.rs:140` (`pub use syn;`) makes `raf_source::syn::File` resolve as `Cyclic`.
  - Cause: the import counts itself as its own blocker (`src/resolve.rs:291-305`) and is then reported as a cycle (`:423-431`).
  - Trait-key walk: `src/impls.rs:285-306` returns `Err(None)` when a local import binds a dependency crate name, which becomes `UnresolvedTrait` (`:328-334`).
  - Contract statements broken:
    - `source-index.md:353-356`: trait keys follow "the use declarations of a local module that the path passes through".
    - `:456-460`: `NotFound` for dependency items; `Cyclic` only when imports depend on each other.
  - No fixture has a single-segment crate re-export (grep over `crates/raf-source/tests`).
- **Consequence:** idiomatic, compiling code fails `Workspace::load` entirely (fail-hard on valid input), and dependency paths are reported as import cycles. Census ledger homes and workplan bindings will hit this in real Blackcat workspaces.
- **Fix**
  - Apply the rustc rule that an import never resolves through its own binding.
  - In edition 2018 and later, resolve a single-segment `use x;` whose head is an extern-prelude crate (a declared dependency for the target kind, a sysroot crate, or an `extern crate`) as an external crate alias. Resolving through it then gives `NotFound`.
  - Make trait keys follow such an alias to the dependency's absolute path (`serde::Serialize`).
  - Add a differential test: resolve every path in rustdoc JSON for `raf-okf` and `raf-source`, built on the nightly gate toolchain, against `Workspace::resolve`.
- **Acceptance**
  - New fixtures in `use_resolution.rs` and `trait_keys.rs`:
    - `pub use serde;` then `demo::serde::Serialize` gives `NotFound`;
    - `pub use std;` gives `NotFound`;
    - `pub use serde;` with `mod a { use crate::serde::Serialize; impl Serialize for String { fn serialize(&self) {} } }` loads, and `resolve("serde::Serialize::serialize")` is `Ok`.
  - In `this_repository.rs`, `ws.resolve("raf_source::syn::File")` matches `Err(ResolveError::NotFound(_))`.

### G3: CRITICAL. Listing containment holds on Linux only, but is stated for every platform

- **Evidence**
  - The Contract (`source-index.md:133-136`) says a listing "reads the entries through the handle, so a swap ... cannot list a directory outside the root". `SECURITY.md:57-58` says "for a directory listing alike" the handle is verified. `AGENTS.md:96-103` makes the same claim.
  - Code:
    - Other Unix: `listing = canonical.to_path_buf()` (`disk.rs:281`), then `fs::read_dir(&listing)` opens by path again after verification (`:137`).
    - Non-Unix: no handle is opened; path `fs::metadata` is used (`:285-290`), passed to a parameter misleadingly named `on_handle` (`:435-447`).
  - On other Unix, `fs::metadata(&after)` (`:273`, `:488`) also resolves the path again, so the "rename ... cannot redirect the handle" wording (`source-index.md:143-147`) overstates. AGENTS.md correctly calls it a residual window.
- **Consequence:** a security claim on a `delivered` capability is false on macOS, BSDs and Windows. A swap after verification can return the names of an outside directory.
- **Fix**
  - **Code:**
    - macOS: list through `/dev/fd/<fd>` (fdesc duplicates the verified descriptor; std-only). This must be confirmed on a macOS runner.
    - Other Unix: re-verify `(dev, ino)` of the path against the held handle after `read_dir` completes, and refuse on a mismatch.
    - Windows: open and hold a directory handle (`OpenOptionsExt::custom_flags(FILE_FLAG_BACKUP_SEMANTICS)`, with `share_mode` excluding `FILE_SHARE_DELETE`) and verify it, then rename the misnamed parameter.
  - **Docs:** scope "through the handle" to Linux and state the residual listing window per platform in the Contract, SECURITY.md and AGENTS.md. This scoping is covered by user decision 3 ("A residual rename race exists only on non-Linux"); it is not an unauthorised weakening.
- **Acceptance**
  - Add a `#[cfg(test)]` hook that fires between verification and `read_dir`, plus a test-only strategy override (see G7).
  - A new `swap::a_directory_swapped_after_verification_before_the_entries_are_read_is_refused` passes under both the Linux and the other-Unix strategies on Linux.
  - A macOS CI job runs `cargo test -p raf-source --locked swap` green.

### G4: HIGH. OV-2 extra-key fixture exists for Architecture only

- **Evidence:** `crates/raf-okf/tests/profile.rs:170-182` and DC `rejects_unknown_key` are Architecture-only. `okf-profile.md:307-308` promises a fixture "per type for ... each extra key". The checklist (`delivery-checklist.md:54`) maps it anyway.
- **Consequence:** the closed-key-set claim is unchecked for Capability and Change, including a valid key of the wrong type (for example `state` in a Change).
- **Fix:** loop over `DocType::ALL` with an unknown key, and with one key from each other type's table.
- **Acceptance:** `cargo test -p raf-okf --locked --test profile an_unknown_or_foreign_key_is_reported_on_every_type` passes. It must fail if a type's key set is temporarily widened in `src/profile/schema.rs`.

### G5: HIGH. SV-6 bound fixtures not exercised through a load

- **Evidence:** `source-index.md:575-577`. `tests/read_cap.rs` calls only `Tree::read` with 8 MiB of `"a"`, which is never parsed. No source-file test uses `load_with`. The behaviour is correct (8 MiB probe in Evidence above).
- **Consequence:** a Verification bullet of a `delivered` capability has no executing fixture.
- **Fix:** add load-level tests: exact 8 MiB valid Rust loads; 8 MiB + 1 is `TooLarge` naming the file; a lowered `LoadOptions::max_file_bytes` is honoured. Make the 8 MiB case release-only if debug time (7 s) matters.
- **Acceptance:** the new tests pass under `cargo test -p raf-source --locked --test read_cap` and `--release`.

### G6: HIGH. Hosted CI never ran

- **Evidence:** `raw/pr-13-checks.txt`: all 10 jobs report FAILURE and none started (billing). The fetching `cargo deny` (advisories) and the new jobs (`test-release`, `fuzz` with `FUZZ_DENY=--locked`, `cross-check`) have never run.
- **Consequence:** nothing hosted backs the gates; an advisory hit on `syn 3.0.8`, `toml 1.1.8` and the rest is unknown.
- **Fix:** restore billing and run `gh run rerun 38280511329 --repo Blackcat-Informatics/rust-agentic-foundations`.
- **Acceptance:** every check on the PR head reports success.

### G7: MEDIUM. Non-Linux dispatch and tests are never executed

- **Evidence**
  - `disk.rs:271-290` and `:486-500` run nowhere. Unit tests call only the verifier functions (`disk/tests.rs:113-165, 283-300`). `disk_tree_containment.rs` and `workspace_containment.rs` are `#![cfg(unix)]`.
  - `make cross-check` checks the lib target only (`Makefile:86-91`), and the Windows lib-test build fails (Evidence above).
  - Windows path semantics: std's verbatim-prefix join removes `..` lexically (rust-src `std/src/path.rs:1369-1380`), so "never clamps a `..` in text" (`source-index.md:125`) and line 95's "really reach" do not describe Windows. Containment still holds, because the result is canonicalised and prefix-checked, and this matches Win32 and rustc.
- **Consequence:** the fallback glue, the Windows test build and the platform semantics are unverified.
- **Fix**
  - Make the strategy (Linux / OtherUnix / Portable) a value chosen by `cfg!` with a `#[cfg(test)]` override, and run the containment and swap suites under OtherUnix and Portable on Linux.
  - Gate `within_watchdog` on `cfg(unix)`.
  - Change `cross-check` to `cargo check -p raf-source --locked --all-targets` with `-D warnings`.
  - Add macOS and Windows `cargo test -p raf-source` CI jobs.
  - State the Windows `..` semantics in the Contract (a factual correction, not a weakening).
- **Acceptance**
  - `RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target x86_64-pc-windows-gnu` exits 0.
  - `cargo test -p raf-source --locked strategy` runs the containment fixtures under each strategy on Linux.

### G8: MEDIUM. The fuzz gate cannot reach overflow-sized inputs

- **Evidence**
  - `Makefile:32,84` sets `-max_len=4096`. The log at 1365-1366 shows the seed corpus truncated.
  - The smallest committed aborting input is 32 KB (`composite_bang_paren.rs`), and a `return` chain needs about 17,700 levels in release.
  - `fuzz_targets/parse_source.rs:6-7` claims "a stack overflow is a finding".
  - The branch added "cuts inputs to 4096 bytes" (`source-index.md:619-621`) to match the gate.
- **Consequence:** the fuzzer cannot find the bug class it advertises.
- **Fix:** after G1, raise `FUZZ_MAX_LEN` (for example to 400,000) or run libFuzzer `-fork` mode, so any abort is a new finding. Restore the Verification sentence.
- **Acceptance:** `make fuzz FUZZ_MAX_LEN=400000` exits 0, and its `INITED` line shows the corpus loaded at full size (about 1.8 MB).

### G9: MEDIUM. Empty root list gives an empty workspace

- **Evidence:** `src/discovery.rs:193-209` (an empty loop gives `Ok`); `discovery_targets.rs:271-276`; PR body: "consumers must guard against that".
- **Consequence:** every consumer must repeat a non-vacuity guard. Without it, a census over zero roots passes vacuously.
- **Fix (owner decision required):** make `roots.is_empty()` a typed `LoadErrorKind::NoRoots`, and change the Verification line "no roots find nothing" to "no roots is an error". This tightens the contract toward fail-hard, but it reverses a written Verification bullet. Decision 2 does not cover it.
- **Acceptance:** `Workspace::load(&tree, &[])` returns `Err` with kind `NoRoots`, beside `&["."]` loading.

### G10: LOW. Licence exceptions await the owner's confirmation

- **Evidence:** `deny.toml:20-26`; `fuzz/deny.toml:28-33`; policy at `deny.toml:9-11`.
- **Fix:** the owner confirms in the PR.
- **Acceptance:** a recorded owner comment; `make deny` and `make fuzz` still pass.

### G11: LOW. The Spawn error path is never provoked

- **Evidence:** `src/parse.rs:63`; the test `error.rs::a_spawn_error_displays_the_file_and_carries_its_cause` builds the variant by hand.
- **Fix:** run a child test that sets a limit with `prlimit --nproc=1` or `sh -c 'ulimit -u 1'` (non-root, `--test-threads=1`) and expects `Spawn`.
- **Acceptance:** the child prints a Spawn result line naming the file.

### G12: LOW. Owner visibility for edits made without a recorded decision

- **Evidence:**
  - `Rule: Sync` in `docs/capabilities/census-rule-api.md:31,41`: this is consistent with `parse_source`'s `+ Send` visitor bound (`parse.rs:40`), and the open record lists `#contract` (`work/changes/census-rule-api.md:9-16`).
  - The `bin_crate` path is in the approved plan (C5) but differs from the precedent.
- **Fix:** list both under "Decisions that differ" in the PR body.
- **Acceptance:** the PR body names both.

### G13–G17: LOW cleanups

| Gap | Evidence | Fix | Acceptance check |
|---|---|---|---|
| G13 | Stale comment at `tests/docs_contract.rs:1570` (the unknown-key path rejects `resource`); it is actually the `ResourceKey` rule | Correct the comment | `grep -n "unknown-key path rejects" tests/docs_contract.rs` returns nothing |
| G14 | `non_kebab_case_bins` warning in `make fuzz` (`fuzz/Cargo.toml:28`; log 1309-1320) | Rename the binary to `parse-source` and keep the file path, or allow the lint with a stated reason | `make fuzz` output has no `warning:` |
| G15 | `let Some(declared) = &entry.manifest.package else { continue; }` (`src/workspace.rs:115-117`) can never trigger: `discovery.rs:105` only records packages | Carry the `PackageDecl` in `Found` | The `continue` is gone and tests are green |
| G16 | An inferred library root's error says "reached by manifest key `lib.path`" when that key is absent (`src/target.rs:211`; probe output) | Use `Origin::Discovery` for defaults | Over-cap inferred `src/lib.rs`: the error text has no `lib.path` |
| G17 | `fenced_code` (`tests/docs_contract.rs:3083-3096`) re-derives fence closure with a backtick-only rule; `FencedBlock` lacks `content()` and closed-state accessors | Add both accessors to raf-okf and use them | A `~~~rust` Usage fixture is accepted |

## Considered and rejected

- **Unresolvable `use` of a dependency is not a load error:** dependencies are outside the index by Constraint (`source-index.md:521-522`), and making it an error would refuse valid code.
- **An abort is a fail-hard breach:** the classifier accepts only a signal, the stack-overflow status or the runtime's message (`tests/support/child.rs:111-125`, plus its fixtures). The real gap is G1.
- **Linux listing or read race:** closed. `swap::*` tests pass, and listing goes through `/proc/self/fd/N`.
- **`entry()` check-then-use window on Linux:** it opens nothing; the read that follows is contained.
- **Windows lexical `..` as a containment bypass:** the result is canonicalised and prefix-checked (G7 is a documentation point only).
- **`Rule: Sync` contradicts the owned-facts rule:** no; rules run in a separate `parse_source` visitor.
- **Function-local impls break the load:** the probe loads them, and they are not indexed, as expected.
- **Work beyond the issue (trait keys, membership, crate roots):** disclosed and needed for bindings to resolve.
- **No shipped consumers:** census and workplan are `designed`; docs_contract is the required consumer and is wired at `tests/docs_contract.rs:908-925`.
- **raf-okf panics:** there is no `unwrap`, `expect` or `panic!` in `src`.
- **Repository rules:** `#![feature]`, `unsafe`, attribution, issue references, `.deficiencies`, lint weakening and deferral wording are all clean. Commits are signed and conventional.

## ETHOS / SOLID

- **SOLID:** SRP, OCP, ISP and DIP pass (the `Tree` seam, `&dyn Tree`, modular `src/`). LSP passes for regular files (disk equals memory).
- **Fail fast:** violated by G2 (refusing valid code) and challenged by G9.
- **One path:** G17 (a minor duplicate fence rule).
- **No rationalised shortcuts:** G1 and G8 reworded the contract to match the code.

## Reviewer feedback

There are no external reviews or threads on the PR. Task review notes:

- T8 notes 1–4: addressed.
- T8 judgment 7: accepted.
- T9 notes 1–5: applied. Note 4 is the source of G1's rewording.

## Priority ranking

1. **CRITICAL:** G1, G2, G3.
2. **HIGH:** G4, G5, G6.
3. **MEDIUM:** G7, G8, G9.
4. **LOW:** G10–G17.
