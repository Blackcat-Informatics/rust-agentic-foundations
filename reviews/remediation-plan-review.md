# Review: Stage 2 remediation plan (raf-okf + raf-source)

- **Plan reviewed:** `.stage/shared-crates-raf-okf-raf-source/remediation-plan.md` (R1-R13, external G6, declined advisories).
- **Read against:** `gap-analysis.md` (G1-G17), `reviews/completion-audit-stage2.md` (CA-F1..F7 plus its scope findings and registry survey), `reviews/security-review.md` (S-1..S-15), `reviews/architecture-review.md` (A1-A18 plus "Resolver checkability"), `reviews/quality-review.md` (Q-1..Q-7), `reviews/performance-review.md` (P-F1..F11), `issue-analysis.md` Stage 2, `historical-fit.md`, `plan.md` (decisions 1-5, C7), `validation.md` (Stage 2 user decisions), AGENTS.md, CLAUDE.md, SECURITY.md, `docs/capabilities/source-index.md`, `docs/capabilities/okf-profile.md`.
- **Code read to ground the findings:** `crates/raf-source/src/{resolve.rs:120-187, modtree.rs, discovery.rs:73-169, tree/disk.rs, index.rs:86-88}`, `crates/raf-source/Cargo.toml`, `crates/raf-okf/src/span.rs`, `crates/raf-source/src/facts.rs:20-29`.
- **Focus applied first:**
  1. Is every finding mapped?
  2. Does each fix close its gap on the real entry point?
  3. Is each check falsifiable against 6c744af?
  4. Does any fix add a fallback, weaken a contract or create debt?
  5. Is each decline correctly reasoned?
  6. Ordering and model tiers.
  7. Feasibility of R8 and R6.

## Overall

- **The plan's spine is right:**
  - User decisions A1, A2 and G9 are followed.
  - The G1 and G8 carve-outs are deleted rather than kept.
  - Rejected outcomes stay typed.
  - Each item carries its own doc edits.
  - The registry scan becomes the real-input regression.
- **Fail-hard holds for the decisions:**
  - A1 resolves to a typed `UnresolvedSelf`, not `Ok` and not `NotFound`.
  - A2 resolves to `Ambiguous`, not first-wins.
  - G9 returns `NoRoots`.
- **Why the plan still needs revision:**
  - Some fixes cannot meet their own checks: R1's 16x bound, and R3's "under 100 MB" probe.
  - Some checks would pass on HEAD: R6's grep, R9's filter, R11's `compile_fail`.
  - Some changes reverse or add Contract text outside the plan's own rule: A14, A15, S-14.
  - Four declines rest on reasoning the source text does not support: P-F10, A12, A16, A10.
  - The "disjoint, run in parallel" claim is false for R1 and R8.
  - About fifteen findings are unmapped or only partly mapped.

---

## BLOCKING findings

### B1. R1 and R8 are not disjoint from the raf-source chain (ordering)

> "Disjoint items (R1 raf-okf, R8 containment) run in parallel with that chain."

- **R8 and R3 edit the same code.**
  - R8 rewrites the open, verify and list paths in `crates/raf-source/src/tree/disk.rs` and the `Strategy` dispatch.
  - R3 adds a `FileId` "returned by `Tree`", which changes `tree/mod.rs`, `tree/disk.rs` and `tree/memory.rs`.
  - R11 (A14) changes the `Tree` trait bound again.
- **R8 and the chain edit the same documents.** R8 edits the containment Contract, `SECURITY.md:52-65`, `AGENTS.md:96-103` and the `Makefile` `cross-check` recipe. R6 edits `SECURITY.md:66-81`, and R3, R4, R5, R6 and R9 all edit `source-index.md`.
- **R1 and R7 edit the same build files.** R1 adds a fuzz target, a `make fuzz` change and `-rss_limit_mb` in `fuzz/` and the `Makefile`. R7 rewrites `FUZZ_MAX_LEN`, renames the fuzz binaries and edits the same `fuzz/Cargo.toml` and Makefile lines.
- **R1 and R12 edit the same crate.** R12's raf-okf parts (G17 `FencedBlock` accessors, P-F2/P-F3, Q-1, Q-2) edit `crates/raf-okf/src/body/*` and `profile/*`, which R1 restructures.
- **Consequence:**
  - Parallel implementers produce conflicting diffs on `disk.rs`, the `Tree` trait, `SECURITY.md`, the Makefile and `fuzz/Cargo.toml`.
  - The orchestrator then has to merge by hand, outside any review.
  - That is the failure pattern historical-fit records for the resolver.

**REWRITE TO:** "Ordering:
- **raf-source chain:** R2 → R3 → R8 → R4 → R5 → R6 → R7 → R9 → R10 → R11 → R12(raf-source).
  - R8 follows R3 because both change the `Tree` seam and `tree/disk.rs`.
  - R11 (A14) follows R8.
- **raf-okf chain:** R1 → R12(raf-okf).
- **Joins:** R7 starts only after R1 lands, because both edit `fuzz/` and the Makefile.
- **Docs:** every edit to `source-index.md`, `SECURITY.md` and `AGENTS.md` is made on top of the previous item's commit.
- **Parallelism:** only the raf-okf chain runs in parallel with the raf-source chain, and only up to R7."

### B2. R1's acceptance bound cannot be met by a correct fix, and the HEAD run can take down the host

> "A counting global allocator in the test binary asserts the peak is at most 16× the input size."

- **The bound is below what bounded parsing already costs.**
  - performance-review §2 measured, at 1 MiB with no long heading: 520k prose lines at 127 MB, and 1M blank lines at 124 MB. That is about 120x the input.
  - security-review "raf-okf body" measured about 120x for headings and quotes.
  - That cost is the per-line `Span` plus line records. It is linear and unrelated to the slug defect.
  - "A 1 MiB document ... with a 60 KB heading and short lines" therefore exceeds 16x after a correct fix.
  - The implementer would have to pick a shape that hides the defect, or loosen the bound.
- **The HEAD run is unsafe.**
  - On HEAD the same input asks for about lines × 60 KB, which is tens of GB.
  - With overcommit that is an OOM kill of the host or CI runner, not a clean abort.

**REWRITE TO:** "`crates/raf-okf/tests/bounds.rs::a_long_heading_over_many_lines_stays_linear`:
- **Inputs:** two 1 MiB documents with the same line count and shape, one with a 60 KB H2 heading and one with a 1-byte heading.
- **Assertion:** `peak(long) <= peak(short) + 4 × heading_bytes × sections + 64 KiB`. The difference comes only from the slug, so the bound is independent of the linear per-line cost.
- **Ceiling:** the counting allocator returns null above 1 GiB, so the HEAD run ends in `handle_alloc_error` (a falsifying abort) instead of exhausting the host.
- **PROPOSED blocks:** add the same pair for many PROPOSED blocks under one long heading (`marker.rs:191`)."

### B3. R3's new budgets do not keep the Contract's "cannot exhaust a machine", and its probe check is unreachable

> "Add an item-record budget (default 4 M) with a typed error." / "Correct the doc sentence to the measured figures. The bound itself is strengthened, so this is not a weakening." / "Security probes ... re-run, and finish typed in under 1 s and under 100 MB."

- **The probe check is unreachable.**
  - P-F4 measured about 3 KB per item record: the depth-16 diamond gave 524k items in 1.58 GB.
  - S-2's shape reaches exactly 2^18 arrivals, under the module-path budget, with at least one item per arrival. With `FileId` dedupe the file is parsed once, but it is still walked and its items recorded per arrival.
  - That gives at least 262k records at about 3 KB each, roughly 0.75 GB.
  - So `src_load.rs` "under 100 MB" cannot hold, and it will not be refused, since 4M records is far above it.
- **The budget allows the very exhaustion the Contract denies.**
  - A 4M record budget at the measured per-record cost allows about 12 GB.
  - "Correct the doc sentence to the measured figures" would then replace "a few tens of megabytes" (`source-index.md:197-198`) with gigabytes.
  - That is a weakening of "cannot exhaust a machine", which the plan claims is not happening.
- **The other budgets have no stated value.**
  - The depth limit for discovery has no stated value.
  - The `max_total_bytes` default is not checked against real workspaces. For example, `web-sys` has 1,727 files, and the scan set may hold larger crates.

**REWRITE TO:** "R3 fix:
- **Storage:** items of a file are recorded once per (file, target). Each further arrival records only (module path, file id), so cost per arrival is O(1) and not O(items). This is P-F4's advisory option.
- **Item-record budget:** sized so that budget × measured bytes-per-record ≤ 512 MiB.
- **Directory depth:** limit 128 below a root, matching module nesting.
- **Contract:** states each budget's worst-case time and memory as measured at its limit, and keeps 'cannot exhaust a machine' true at those figures.

R3 check:
- `src_load.rs` and `mem_deep.rs` end typed or load, within the stated worst-case figures.
- The registry scan shows 0 crates refused by `max_total_bytes`, the depth limit or the item-record budget, with the largest observed values recorded in `validation.md`."

### B4. R3's `FileId` cannot be "(dev, ino)" on Windows without a forbidden feature or FFI

> "Key visited files by canonical identity, returned by `Tree` as a new `FileId`: (dev, ino) on Disk, the normalised key on Memory."

- `make cross-check` builds `raf-source` for `x86_64-pc-windows-gnu`.
- As far as I know, std's Windows file-identity accessors (`MetadataExt::volume_serial_number`, `file_index`) sit behind the unstable `windows_by_handle` feature. Confirm this against MSRV 1.98.1.
- If they are still unstable, the plan as written pushes the implementer towards one of two forbidden routes:
  - `#![feature]`, which CLAUDE.md forbids;
  - FFI with `unsafe`, which `unsafe_code = deny` forbids.
- Any other route is unspecified.

**REWRITE TO:**
- "`FileId`:
  - On Unix `DiskTree`: (dev, ino) from the verified handle.
  - On other `DiskTree` platforms: the canonical root-relative path that `contain` already computes, which is std-only.
  - On `MemoryTree`: the normalised key.
- The kept spelling of a file reached under several spellings is the first in sorted order. This is deterministic and stated in the Contract next to 'one file has one spelling'."

### B5. New refusals lack neighbouring-valid fixtures and false-positive checks (AGENTS.md §5)

AGENTS.md:180-183 says: "**Every rule has a negative fixture.** ... **A refusal is a claim too.** When you add or tighten a refusal, run the case you believe is invalid and a neighbouring valid case, and confirm the second succeeds."

The plan adds these refusals:

- in R3: `max_total_bytes`, the discovery and glob depth limit, and the item-record budget;
- in R5: the glob-edge budget and A13's missing-glob error;
- in R10: the member-count budget.

Only refusal-side tests are named. None has an at-limit or limit-1 neighbour, and none is checked against the registry scan for false refusals.

Separately, S-4 required "a test that runs the deep-directory case on a 2 MiB debug thread". R3 instead says "at 20,000 levels, run in release without abort". That is weaker: the debug 2 MiB thread is exactly where `docs_contract` and every `cargo test` consumer runs.

**REWRITE TO:** "For each new budget (R3 total bytes, directory depth, item records; R5 glob edges; R10 members), add:
- a fixture at the limit that loads;
- a fixture one over that is refused with `LoadErrorKind::Budget`, naming the budget and the file.

S-4 checks:
- R3's deep-directory test runs on a `std::thread::Builder::new().stack_size(2 << 20)` thread in the debug profile and expects the typed error.
- One `DiskTree` deep-chain case is included (unix).

False-refusal check: the registry scan shows 0 crates refused by any budget introduced in Stage 2."

### B6. R4's A2 fix does not say how imports pick between same-named crates, and its path syntax collides with crate heads

> "**A2 (user decision):** crates are keyed by (package, crate name). A path headed by a crate name shared by more than one package resolves `Ambiguous`, and `<package>::<path>` resolves it."

- **Imports are left undefined.**
  - Today an import head resolves through `self.libs.get(first)`, keyed by crate name alone and ignoring the importer's dependencies (`resolve.rs:131, 174`).
  - In purrdf, a third package that depends on `purrdf-capi` writes `use purrdf::X;`.
  - rustc picks the crate from that package's dependency table. The plan defines only query-time behaviour.
  - An implementer will either make every such import `Ambiguous` or keep first-wins. The first wrongly refuses real code; the second is the banned first-wins rule.
- **The disambiguating form collides with crate heads.**
  - Package names may contain `-` and are not path segments.
  - The package ident `purrdf` equals the crate name `purrdf`, so `purrdf::X` cannot tell "package purrdf" from "crate purrdf".
  - The decision text says only "a package-qualified form disambiguates"; it does not fix this syntax.
- **R5 overlaps with this.** R5's extern-prelude definition ("a declared dependency for the target kind") is the same mechanism, but R5 is written as if crate names are unique.

**REWRITE TO:** "**A2:**
- **Keying:** `libs` and `crate_roots` are keyed by (package, crate name).
- **Imports:** an import's crate head resolves through the importing target's extern prelude. That is its own package's library (for non-library targets) plus its declared dependencies for the target kind, mapped to workspace packages through `path` and `workspace = true` entries and renames, plus the sysroot crates. So an import never sees two same-named libraries.
- **Queries:** `Workspace::resolve(path)` returns `Ambiguous` naming both targets when the head is a crate name of more than one package. A new `Workspace::resolve_in_package(package: &str, path: &str)` takes the Cargo package name as a separate argument, so no path syntax collides with a crate name.
- **R5:** R5's extern-prelude rule uses this same table.
- **Checks:**
  - a two-package fixture with `[lib] name = "shared"` in both;
  - a third package depending on one of them, whose `use shared::X` resolves to that one;
  - `resolve("shared::X")` is `Ambiguous`;
  - `resolve_in_package("p-two", "shared::X")` is `Ok`;
  - purrdf `crates/` loads."

### B7. R5's A4 rule is worded so that the reviewer's own fixture can pass with a wrong answer, and the check states no expected result

> "**A4:** settle import cycles as a strongly connected component. A cycle with no outside binding reports `Cyclic` for every member, whatever the declaration order." / Check: "The A4 cycle fixture, run in both declaration orders"

- The A4 fixture (architecture-review A4) has outside bindings: `pub use crate::c::*;` and `pub use crate::d::*;` each supply `X`.
- rustc rejects it, because explicit imports shadow globs, so the explicit `X` imports form a cycle whatever the globs hold.
- Under "a cycle with no outside binding", an implementer can settle `a::X` to `c::X` and `b::X` to `d::X`.
  - That answer is order-independent, so "run in both orders" still passes.
  - It is wrong and is not `Cyclic`.

**REWRITE TO:** "**A4:**
- **Rule:**
  - When no stuck import is a leaf, compute the strongly connected components of the explicit-import blocker graph.
  - Each component whose members are bound only by imports inside the component is settled as cyclic: every name it binds resolves `Cyclic`.
  - A glob in a member module never settles a name that an explicit import of that module binds, because explicit imports shadow globs.
- **Check:** the architecture-review A4 fixture asserts `resolve("demo::a::X")` and `resolve("demo::b::X")` are both `Err(Cyclic)`, in both declaration orders. A neighbour without the `b → a` import resolves `demo::a::X` to `demo::c::X`."

### B8. The R2 and R5 fixture lists are narrower than the source findings require, and the CA-F7 check does not test the CA-F7 fix

> R2 check: "covering `a/mod.rs` + `a/b.rs` and `a.rs` + `a/b.rs`."

CA-F2's required fixtures also include:

- a `#[path]` file with a child that sorts first (the `pathchild3` order dependence);
- a shared `tests/common/mod.rs` + `tests/common/a.rs` reached from a target that sorts after `tests/common`;
- `super::super` from such a child (the false `SuperAboveRoot`);
- the negative `demo::a::in_b` is `NotFound`.

> R5 check: "The G2 acceptance fixtures (`pub use serde;`, `pub use std;`, a trait impl through a local alias)."

CA-F1's real-crate shapes are absent from this list:

- `pub use serde::{self, Serialize};`, with `demo::Serialize` currently `Cyclic` (hmac, ripemd, sha3, the p-curves);
- `use std; impl std::fmt::Display for &Foo` keyed `std::fmt::Display` (rmp-serde);
- `pub use std::{self as s2};`.

> R5 check: "An 809 KB CA-F7 input loads in a child process without abort."

- That input recurses only because `use std;` resolves against itself, which is CA-F1.
- After R5's G2 part lands, the input stops recursing, even if `external_prefixes` and `absolute` remain mutually recursive with a `uses.len() + 8` cap.
- The check fails on HEAD, but it does not detect a missing worklist. CA-F7's required fix is a fixed, stack-safe bound.

**REWRITE TO:**
- **R2 fixtures:** add `module_tree.rs` fixtures for:
  - a `#[path = "gen/generated.rs"]` file whose children `a`, `ga` and `z` are each placed under `generated`;
  - `tests/common/mod.rs` + `tests/common/a.rs` reached from `tests/zz.rs`, with item paths asserted;
  - `super::super::top` from `src/a/b.rs`, which loads;
  - `demo::a::in_b` and `demo::a::b::in_a` as `NotFound` beside `demo::a::b::in_b` as `Ok`.
- **R2 registry check:** shows 0 `SuperAboveRoot` refusals in `brotli`, `structured-zstd` and `wasmi`.
- **R5 G2 fixtures:** add `pub use serde::{self, Serialize};` giving `NotFound` for `demo::serde::Serialize` and `demo::Serialize`; `use std;` + `impl std::fmt::Display for &Foo`, loading with key `std::fmt::Display`; and `pub use std::{self as s2};` giving `NotFound`.
- **R5 CA-F7 check:** a file with 20,000 chained local aliases (`use crate::m::T as A0; use A0 as A1; ...`) and `impl A19999 for X` loads, or fails typed, on a 2 MiB thread in debug and in release. It contains no self-referential import.

### B9. R6's additive budget is under-specified where false positives and evasions come from

> "the pre-scan counts every token that can begin a nested syn recursion frame, per open delimiter level. That covers prefix operators and keywords (`return`, `break`, `yield`, `become`, `move`, `async`, `const`, `unsafe`, `!`, `-`, `*`, `&`, `&mut`), closure bars, `fn(..) ->`, `impl Fn() ->`, `for<'a>`, assignment, binary operators and postfix chains. It refuses typed when the sum over open levels exceeds a limit derived from the measured debug frame cost, with a safety factor of at least 2."

- **No reset rule is given, which causes false refusals.**
  - If a level's count is not reset at statement and argument boundaries (`;`, `,`, `=>`, and the end of a block-like expression statement such as `if ... {}` or `match ... {}` without `;`), then a function body with a few thousand `&`, `*` or `+` across its statements passes any frame-derived limit (debug: 8.8 KiB per level, so about 3,700 with a factor of 2).
  - That is ordinary code.
  - The existing `<` and `else if` counters (`source-index.md:169-185`) each state their reset points; the new one must too.
- **The token list is incomplete, which repeats the S-5 failure.**
  - It omits `box` (S-11's quadratic shape, which R10 says R6 will bound), `static` (S-5 fix 1: "Count `async`, `async move`, `const` and `static`"), and `as` casts (P-F11: `x as u8 as u8 ...`).
  - It also omits the expression keywords that recurse without opening a delimiter at the same level: `if if x {} {} {}`, `match`, `while`, `let`, `..`, `extern "C" fn() ->` and `dyn`/`impl` type prefixes.
  - S-5 showed that a hand-enumerated list misses shapes. The list must be derived from the parser.
- **The frame cost is measured on the wrong thing.**
  - For left-nested postfix and binary chains, syn's parse loop is shallow.
  - The depth is paid by the fact visitor and by `Drop` of the nested `Expr` on the parse thread.
  - A limit derived from parser frames alone can be wrong for these shapes.
- **The false-positive metric cannot be read off a moving scan.** "This repository and the registry scan show no new refusals" mixes in every other Stage 2 change to the scan results.

**REWRITE TO:** "R6 fix:
- **Derivation:** the counted token set is derived from an audit of syn 3.0.8's recursive entry points:
  - expression: unary, atom, trailer, binary, cast, range, closure, `if`, `match`, `while`, `let`, `box`, `static`, `async`, `const`, `unsafe`, labelled block;
  - type: reference, pointer, `fn` with return type, `impl`/`dyn` bound with return type, `for<'a>`;
  - pattern: `box`, `&`, `ref`, `mut`, range.
  The audit is recorded in the Contract as the counter definition.
- **Reset rule:** a level's count returns to zero at `;`, `,`, `=>` and at a `}` that closes a block-like expression statement at that level, as the `else if` counter does.
- **Limit:** set from the measured debug stack cost per unit of parse plus fact visitor plus drop on the parse thread, for the costliest counted shape, with a factor of at least 2. The measured figures are written into the Contract.

R6 checks, in addition to the overflow harness in both profiles:
- every counted token has an over-limit fixture that is refused and an at-limit fixture that parses, in `prescan_budgets.rs`;
- the registry scan and this repository show 0 refusals with the new budget kind;
- the limit is not raised above the derived value to reach that."

### B10. R8 does not say which Contract sentences change, and its swap check cannot pass under Portable as designed

> "The Contract, SECURITY.md and AGENTS.md state the per-platform guarantee exactly." / "The new swap test `a_directory_swapped_after_verification_before_the_entries_are_read_is_refused` passes under each strategy."

- **Sentences that stay false unless changed.** Each is false on at least one platform after R8's code:
  - `source-index.md:122-123`: "neither opened nor read". The Linux `O_PATH` is an open; other Unix still opens the outside file non-blocking in a race (S-6).
  - `source-index.md:112-113`: `entry` gives `OutsideRoot`. The plan fixes this only on Linux, while S-7 says "all platforms".
  - `source-index.md:133-136`: listing "through the handle".
  - `source-index.md:139-147`: "no check is separated from the open it guards", and "cannot redirect the handle" on other Unix.
  - `SECURITY.md:57-65` and `AGENTS.md:96-103`.
  - Without a list, the reviewer cannot check "exactly", and Stage 1's G3 was exactly this drift.
- **The Portable strategy fails the planned swap test.**
  - The Portable strategy, run on Linux, has no Windows share-mode protection.
  - The plan gives re-verification after `read_dir` only to "Other Unix".
  - The planned swap test, run under Portable, would list the outside directory and fail. Otherwise the implementer quietly adds re-verification to Portable, unstated.
- **S-6's committed regression test is missing.** S-6's required fix includes "Add the FIFO-writer race as a Linux test". The plan only re-runs `/tmp/raf-sec/src/bin/race_open.rs`, which is not committed, so nothing guards the property afterwards.

**REWRITE TO:** "R8:
- **Docs:** R8 rewrites these sentences per platform: `source-index.md:112-113`, `:122-123`, `:129-150`, `SECURITY.md:52-65` and `AGENTS.md:96-103`.
  - Linux: 'an outside candidate is never opened for reading or listing; only an `O_PATH` reference is taken, which performs no I/O, device open or FIFO rendezvous'.
  - Other Unix and Windows: 'an outside file can be opened non-blocking in a race and is then refused before any read'. `entry` and listing are stated with their residual window.
- **Strategies:**
  - Other Unix and Portable both re-verify after `read_dir`, after each read and after `entry`'s lstat, and refuse on a mismatch.
  - Windows also holds the handle without `FILE_SHARE_DELETE`.
- **Checks:**
  - The swap test and a new `entry` swap test pass under Linux, OtherUnix and Portable on Linux.
  - A committed Linux test uses the existing `#[cfg(test)]` swap hook between the prefix check and the open to put an outside FIFO with a blocked writer thread in place. It asserts that the writer is never released (bounded wait) and the read is `OutsideRoot`. This test fails on 6c744af."

### B11. R8's all-platform `RelPath` refusal of Windows-reserved names creates false refusals on Linux and changes the Contract without a decision

> "`RelPath` refuses segments that are Windows-reserved or ambiguous (trailing dot or space, `con`, `nul`, `aux`, `prn`, `com1`–`com9`, `lpt1`–`lpt9`) on every platform (S-14)."

- **Discovery would refuse valid Linux trees.**
  - Discovery joins every listed directory name through `RelPath::join` (`discovery.rs:146-147`).
  - A Linux workspace with a directory named `aux` (common in TeX trees), `con`, or a file such as `nul.txt` or `notes.` under a root would fail `Workspace::load`.
  - That is a new false refusal of valid input on the platform that runs the gates.
- **No user decision covers the Contract change.**
  - The `RelPath` Contract (`source-index.md:91-94`) lists exactly what is refused.
  - Widening that list is neither a user decision nor a factual correction, so it breaks the plan's own rule: "A contract sentence changes only in two cases".
- **No check exists.** None tests the new refusal or its neighbour.
- **The source finding does not demand this.** S-14 is Info and says "consider".

**REWRITE TO:** "**S-14:**
- **Code:** on Windows only (`cfg(windows)`), `DiskTree` refuses a path whose segment is a reserved device name (case-insensitive, with any extension) or ends in a dot or a space, before canonicalising, with `OutsideRoot` naming the path. `RelPath` is unchanged on every platform.
- **Contract:** the Windows bullet of the containment Contract gains this, as a factual statement of the Windows platform semantics, together with the `..` lexical-join note.
- **Checks:** a `cfg(windows)` unit test, compiled by `make cross-check`; and a Linux test that a tree with `aux/` and `nul.txt` loads."

### B12. R11's A14 reverses an explicit Contract sentence without a user decision

> "**A14:** require `Tree: Sync`."

- `source-index.md:100` says: "The trait is object-safe and has no `Send` or `Sync` bound."
- The plan's own rule is: "A contract sentence changes only in two cases ... 1. A user decision changes the behaviour (A1, A2, G9). 2. The sentence is factually wrong".
- A14 is neither of those. The architecture review offered a non-reversing alternative: "or record that a parallel load would be a breaking change".

**REWRITE TO:** "**A14:**
- The `Tree` trait keeps no `Send` or `Sync` bound.
- The Constraints gain: 'Loading is serial; requiring `Tree: Sync` for a parallel load would be a breaking change to `Workspace::load`.'
- An alternative needs a recorded owner decision: ask whether to add `+ Sync` now. If the owner agrees, record it verbatim in `validation.md` and update `source-index.md:100`."

### B13. R11's A15 picks a direction that makes one `Span` type half-open in bytes and inclusive in lines

> "**A15:** both crates count line spans the same way (1-based, inclusive). Document it in each Contract."

- `raf_okf::Span` already documents "Both ranges are half open. Line numbers are 1-based" (`crates/raf-okf/src/span.rs:13`), with `lines: Range<usize>` and `bytes: Range<usize>`.
- Making raf-okf lines inclusive would have two effects:
  - a `Range<usize>` would carry inclusive semantics, which is a footgun;
  - one `Span` would mix half-open bytes with inclusive lines.
- It would also silently change the delivered okf-profile Contract and every `replace_spans` consumer.
- The architecture review's fix ran the other way: "Make `SpanFact` half-open with `Range<usize>` fields, or document the difference".

**REWRITE TO:** "**A15:**
- **Code:** `raf_source::SpanFact` replaces `start_line`/`end_line` with `lines: Range<usize>` (1-based, half-open), matching `raf_okf::Span`.
- **Contracts:** both say 'line ranges are 1-based and half-open'.
- **Check:** a test asserts that a one-line item has `lines == n..n+1`, and `docs_contract` and the workspace tests pass."

### B14. Several checks pass on 6c744af, or prove nothing about the change

> R6: "`grep -n "No budget counts" docs SECURITY.md` returns nothing."

- `docs` is a directory, and without `-r` grep skips it.
- `SECURITY.md:77` has lower-case "(no budget counts ...".
- So on HEAD this check returns nothing; it is vacuous.

> R9: "`cargo test -p raf-source --locked --test discovery_targets no_roots`."

- On HEAD no test matches `no_roots`, and a filter that matches 0 tests exits 0.

> R11: "A compile check that `raf_source::Target { .. }` cannot be built outside the crate (a doctest with `compile_fail`)."

- `Target { .. }` without a base expression is already a compile error on HEAD, so the doctest passes on HEAD.

> R12: "Each named test passes and fails on the pre-fix code where a behaviour changed."

- G4, G5, Q-4 and Q-6 test behaviour that is already correct, so this sentence exempts them from any falsification.
- G4's own acceptance said "It must fail if a type's key set is temporarily widened in `src/profile/schema.rs`". Q-4 named the mutation "passed `u64::MAX`" at `modtree.rs:189`. Both were dropped.

> R4: "New fixtures, one per gap, in `symbol_index.rs`, `trait_keys.rs` and `target_roots.rs`." / R11 A6: "Add `harness`, `required_features` and `doctest` to `Target`, read from the manifest."

- No fixture is named for A2, A3 (both sort orders), S-15 or G16.
- The new `Target` fields have no fixture asserting the values read.

**REWRITE TO:**
- **R6:** "`grep -rni 'no budget counts\|can end in such an abort\|can end in that abort' docs/capabilities/source-index.md SECURITY.md` returns nothing (it returns lines 216, 519 and SECURITY.md:77 on 6c744af)."
- **R9:** "`cargo test -p raf-source --locked --test discovery_targets -- --exact an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads` reports `1 passed`."
- **R11 sealing check:** "A `compile_fail` doctest destructures `Target` with every current field and no `..` (`let raf_source::Target { package, name, kind, root, shipping, crate_path } = t;`). It compiles on 6c744af and fails once `Target` is `#[non_exhaustive]`. The same check is made for `Package`, `SourceFile`, `Reach`, `Entry` and `SpanFact`."
- **R12 mutations:** "For each test of behaviour that was already correct (G4, G5, Q-6, Q-7), the named mutation is run once and recorded in the item's review: widen the Capability key set in `schema.rs`; pass `u64::MAX` at `modtree.rs:189`; set `missing_docs = "warn"` in the root `Cargo.toml`. The test must fail under it."
- **R4:** name each fixture, including:
  - `target_roots.rs::a_build_script_main_beside_a_lib_module_build_script_is_ambiguous`;
  - the same for an explicit-path bin, in both file-name orders;
  - the A2 fixtures of B6;
  - `module_tree.rs::a_non_literal_path_attribute_is_reported_as_such_from_the_attribute`;
  - `an_over_cap_inferred_lib_names_discovery_not_lib_path`.
- **R11 A6:** add `discovery_targets.rs::harness_required_features_test_and_doctest_are_read_from_each_target_table`, asserting the values beside defaults. Include `test = false`, which A6 also lists.

### B15. Four declines are wrongly reasoned

> "**P-F10 (debug walk needs 1 MiB of caller stack):** removed by R3's iterative traversal; checked there."

- R3 makes "discovery and glob expansion iterative". P-F10 is about the **module walk** (`modtree.rs`, where `enter` → `body` → `module` → `enter` recurses on the caller's stack to depth 128).
- R3 does not touch that walk, so the decline is false.

**REWRITE TO:** "**P-F10:** fixed in R3. Either the mod-tree walk (`Walker::enter`/`body`/`module`) uses an explicit stack, or the Contract states its measured caller-stack need (about 1 MiB in debug at depth 128). Check: the 128-level fixture runs on a 512 KiB thread in debug."

> "**A12 (path validation in `resolve`):** subsumed by P-F9's linear validation in R5."

- P-F9 is about cost.
- A12 is about wrong acceptance and a swallowed error arm: `demo::crate::a::x` and `demo::self::a::x` resolve `Ok`, and `Err(Fail::SuperAboveRoot) => {}` at `resolve.rs:387` swallows a failure into `NotFound`.
- That swallowed arm is the "`_ => {}` over a fallible case" pattern that AGENTS.md:175-177 ("No silent fallbacks") and plan section A ban.
- Linear validation leaves both in place.

**REWRITE TO:** "**A12:** fixed in R5. `resolve` returns `Invalid` for `crate`, `self`, `super` or `Self` in any position of a canonical path, and the swallowed `SuperAboveRoot` arm is removed. Fixtures: `demo::crate::a::x` and `demo::super::x` are `Invalid`, beside `demo::a::x` as `Ok`."

> "**A16 (small API redundancies):** convenience methods that do not affect correctness or stability. Kept."

- A16 includes `items_under("demo::a")` yielding `demo::ab::..`. That is a wrong answer for "the items under a path prefix" (`source-index.md:417-418`).
- It also includes `SourceFile::package`, which depends on walk order. Both affect correctness.

**REWRITE TO:** "**A16:**
- **Fixed in R11:** `items_under` matches whole segments (`demo::a` and `demo::a::*`, not `demo::ab`), with a fixture beside `demo::ab`. The `SourceFile::package` rule is stated: the package of the first target in target order.
- **Kept, with reason:** `SourceFile::test` versus `reached_only_through_tests()`, and `test_fns()` returning `Result`, because removing either changes a documented signature without a correctness gain."

> "**A10 (load stops at the first error):** the source-index Contract specifies first-error refusal for loads."

- The only such sentence, `source-index.md:290` ("The walk stops at the first error."), sits in the **Mod-tree following** paragraph and scopes to the walk.
- No sentence covers discovery, targets, manifests or index building, so the Contract does not yet state the load-level policy.
- The architecture review asked for exactly that statement: "keep fail-fast and state in the Contract that it is deliberate".

**REWRITE TO:** "**A10:** kept as fail-fast. R13 adds to the `Workspace::load` Contract: '`Workspace::load` returns the first error, in the order discovery, targets, module walk, index; it does not collect errors.'"

### B16. Findings that are unmapped or only partly mapped

Each needs an item, or a decline with a reason. The plan states complete coverage.

| Source finding | What is missing | REWRITE TO |
|---|---|---|
| S-13 (licence exceptions not version-pinned) | absent | R13: pin `unicode-ident@1` in both `deny.toml` files and `libfuzzer-sys@0.4` in `fuzz/deny.toml`, keeping `repo_gates`' equality gate. Or decline: "the owner sign-off is for the crate; a major bump shows in `Cargo.lock` review". |
| S-12 (fuzz manifests and a `MemoryTree` workspace; `-malloc_limit_mb`) | only parse_source and OKF covered | R7: add a `load-workspace` target over a structured `MemoryTree`, with `-malloc_limit_mb` on every target. Or decline with a reason. |
| CA scope: `use {::a::b}` (`Item::Verbatim`, plan.md:326) and the attribute-macro Verbatim refusals (8 crates) | absent | Declined advisories: "kept as a typed refusal under plan C7. It is rustc-valid but rare (8 of 1,177), and indexing it means guessing a token stream". Or raise it with the owner beside A1. |
| CA-F6 tail: 10 `UnresolvedTrait` crates not root-caused | absent | R4/R5 check: "after R5 the scan re-runs these ten; each remaining refusal is root-caused in `validation.md` and either fixed or shown to be correct". |
| CA-F2, CA-F1 and CA-F7 required fixtures | narrowed | See B8. |
| Q-3 nested unknown keys (`bindings.extra`, a Change item field) | absent | R12 G4: add the nested cases, or reword `okf-profile.md:307-308` to top-level keys as a labelled factual correction. |
| A9 remainder (`FileCtx: Sync`, `RuleHit: Send`, `LoadOptions: Copy` blocking a per-file visitor) | owner sign-off covered `Rule: Sync` only | Declined advisories: "census-rule-api is `designed`; its own change fixes `FileCtx`/`RuleHit` bounds. `LoadOptions` is `#[non_exhaustive]`; dropping `Copy` later is breaking, so decide now". Or drop `Copy` in R11. |
| A11 `code` bindings must be shipping | half of A11 dropped | R11: add the `code` → `is_shipping` check with its negative fixture, or decline with a reason. |
| Architecture "Resolver checkability" point 1 (interim and final glob tables agree) | absent | R5: add the debug assertion that re-walks every settled named import against the final tables. |
| G7 / G3 macOS and Windows CI jobs | silently dropped | See A10 below. Add the jobs to `ci.yaml` so the G6 rerun exercises them, or decline them explicitly under decision 3. |
| historical-fit R4 (docs_contract mutation run) and D14 (`quote` + `printing`) | absent | Declined advisories, with reasons, or a mutation spot-check in the exit audit. |
| P-F8 tail (`discovery.rs:104` O(P²) package scan) | absent | R3: `BTreeMap` by name, or decline (harmless below 10^5 packages). |
| G10 | only implied | R13: "G10 is closed by the Stage 2 sign-off; `make deny` and `make fuzz` still pass." |

---

## ADVISORY findings

### A1. Model tiers

> "The intricate raf-source items (R2–R6) use opus/high ... The other items use sonnet/medium." / "each item gets a sonnet/medium reviewer"

- R8 is the CRITICAL containment item, touching platform syscalls and three strategies, and it is at sonnet/medium.
- The reviewers of R2-R6 sit a tier below their implementers, which inverts the check where Stage 1 failed.

**REWRITE TO:** "R8 implementer at opus/high. Reviewers of R2–R6 and R8 at opus/high. Others sonnet/medium."

### A2. Split R5 into reviewable commits

R5 bundles G2, A4, CA-F7, P-F6, P-F5/S-3/P-F7, P-F9, A13 and, after B15, A12, under one subject that names only aliases and cycles. That is too much resolver change for one review.

**REWRITE TO:**
- R5a `fix(raf-source): resolve extern-prelude aliases and key traits without recursion` (G2, CA-F7);
- R5b `fix(raf-source): settle import cycles as components` (A4, P-F6, checkability assertion);
- R5c `perf(raf-source): share glob scope tables under a glob budget` (P-F5, S-3, P-F7);
- R5d `fix(raf-source): validate resolve paths and refuse unresolvable local globs` (P-F9, A12, A13).

### A3. Fuzz target naming churn

R1 adds `parse_document` and R7 renames it `parse-document`.

**REWRITE TO:** "R1 adds the target as `parse-document` (file `fuzz_targets/parse_document.rs`)."

### A4. Utility for A1 and A2

- **A1:** state the `path` an `Unresolved` impl member gets, which is the written self path in the impl's module, and that `items()`/`item_at()` carry the marker. Add `Workspace::unresolved_impls()`, so census-ledger can choose to fail (architecture A1 fix).
- **A2:** `resolve_in_package` (B6) is the utility half.
- **Cost and payoff:** about one accessor each. Consumers get fail-hard on their own terms.

### A5. G2 edge rules

- State edition 2015 behaviour: `use x;` is crate-relative.
- State a 2018+ name that is both a local item and an extern-prelude crate: `Ambiguous`, as rustc's E0659.
- Each gets a fixture.

### A6. A13 scope

Say that the typed error is raised on a lookup that would consult the missing glob, and that the **load** still succeeds. Otherwise a module generated by a macro, then globbed, becomes a new load refusal of valid code.

### A7. Glob work

- R5 names `Arc` sharing but not the propagation that makes a 4,000-glob chain linear. P-F5 proposes "wake only the globs watching a module whose scope changed".
- **REWRITE TO:** name it, give the glob-edge budget's value, and add S-3's `resolve_cost globs 2000` probe to the check.

### A8. R10 coverage

- S-9 deduplication passes the duplicate-pattern probe. A manifest of distinct glob excludes is still M×E.
- Add an `exclude` length cap (S-9: 4,096) with a typed `Manifest` error, and a distinct-pattern probe.
- For S-11, `box` must be in R6's token set (B9).

### A9. G5 in both profiles

A comment-padded 8 MiB `.rs` loads in about 0.4 s (Q-4, CA `cap8`), so the case need not be release-only. Run it in both profiles and avoid an `#[ignore]`/`cfg` split.

### A10. Hosted CI coverage for non-Linux

- Add `macos-latest` and `windows-latest` `cargo test -p raf-source --locked` jobs to `ci.yaml` now.
- The G6 rerun then turns "unverified, decision 3" into runtime evidence at no extra plan cost.
- If they are left out, record the decline.

### A11. R8 implementation notes (feasibility is fine without `unsafe`)

**Linux:**
- Open with `OpenOptions::new().read(true).custom_flags(libc::O_PATH | libc::O_NOFOLLOW)`.
- `File::metadata()` works on an `O_PATH` fd, and `fs::read_link("/proc/self/fd/N")` verifies it.
- Reopen `/proc/self/fd/N` **without** `O_NOFOLLOW`, because it is a magic link, and only after `fstat` shows a regular file or a directory.
- `entry` can be `fs::symlink_metadata("/proc/self/fd/<dirfd>/<name>")`, with `name` a single segment.

**Windows:**
- Define `FILE_FLAG_BACKUP_SEMANTICS` (`0x0200_0000`) as a local const, since there is no windows-sys dependency.
- Use `OpenOptionsExt::share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)`. Both are stable std.

**Strategy dispatch:** the `Strategy` must be a `#[cfg]`-gated enum. `cfg!` alone would require `libc::O_PATH` to compile on Windows.

### A12. Registry harness

- Define the "misplaced item" oracle in the harness: every item's path prefix equals a module path recorded in its file's `Reach`.
- Take the registry directory as an argument, with no `/tmp` path committed.
- Record the scanned `name@version` list with the tallies.
- Phrase R4's check as "no crate fails with those messages; each crate that now fails for another reason is root-caused". Do not use "now loads", which a correct later refusal would break.

### A13. Rustdoc differential

- Filter to `crate_id == 0` items.
- Expect external re-exports (`raf_source::syn::*`) to be `NotFound`, per the Contract.
- Consider a `make` target that runs it on the nightly gate toolchain, rather than an `#[ignore]` run only in evidence.

### A14. Per-item gates

- Each item's acceptance should include `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test -p <crate> --locked`, plus `--release` for raf-source.
- That keeps every signed commit green, rather than only the final HEAD.

### A15. Commit types

- G17 adds public API, so it is `feat(raf-okf)`.
- P-F2/P-F3 is `perf(raf-okf)`.
- The subjects feed `git-cliff` (AGENTS.md:230-232).

### A16. R13 state rule

Say what happens if the exit audit fails: both capabilities move to `implementing`, with the `INVENTORY` edit in the same commit (AGENTS.md:201-204).

### A17. Declines A17 and A18

- **A17:** cite R4/A3 as covering the `build_script` collision.
- **A18:** the architecture fix was only "say it in source-index.md Constraints". Add that sentence rather than declining.

### A18. A5 left open

> "Otherwise make them `#[non_exhaustive]` with accessors"

The plan leaves the choice to the implementer. Decide now: `pub(crate)` facts, with the integration-test assertions moved to unit tests. That is the architecture review's preferred fix.

---

## Coverage summary

| Report | Mapped to an item | Declined, correctly reasoned | Wrongly declined or unmapped |
|---|---|---|---|
| gap-analysis G1-G17 | G1 R6; G2 R5; G3 R8; G4/G5/G11/G13/G15/G17 R12; G6 external; G7 R8; G8/G14 R7; G9 R9; G12 R13; G16 R4 | – | G10 only implied (B16); G7/G3 CI jobs dropped (B16, A10) |
| completion audit F1-F7 | F1 R5; F2 R2; F3 R8; F4 R12; F5/F6 R4; F7 R5 | – | fixture narrowing (B8); Verbatim scope finding and the 10 un-root-caused crates (B16) |
| security S-1..S-15 | S-1 R1; S-2/S-4 R3; S-3 R5; S-5 R6; S-6/S-7/S-8/S-14 R8; S-9/S-10/S-11 R10; S-15 R4; S-12 R1/R7 (part) | – | S-13 (B16); S-12 remainder (B16); S-14 approach (B11); S-7 non-Linux (B10) |
| architecture A1-A18 | A1/A2/A3 R4; A4/A7/A13 R5; A5/A6/A8/A11/A14/A15 R11 | A17, A18 (with A17 note) | A10, A12, A16 (B15); A9 remainder, A11 half, checkability (B16); A14 and A15 approach (B12, B13) |
| quality Q-1..Q-7 | Q-1/Q-2/Q-6/Q-7 R12; Q-3 as G4; Q-4 as G5; Q-5 as G11 | – | Q-3 nested keys (B16) |
| performance P-F1..F11 | F1 R1; F2/F3 R12; F4/F8 R3; F5/F6/F7/F9 R5; F11 R6 | – | F10 (B15); F8 tail (B16) |
| issue-analysis Stage 2 | OV-2, SC-13, SC-15/16, SC-19b, SV-6, SV-13 all mapped | IS-11 per decision 5 | – |
| historical-fit | R1-R3, R5-R7 covered by items or decisions | R8 (tooling) | R4 and D14 (B16) |

VERDICT: NEEDS-REVISION

---

## Recheck (revision 2)

- **Plan rechecked:** `remediation-plan.md`, revision 2, read in full.
- **Method:** each prior finding is checked against the revised text, with the line that proves its status. Only the new text is reviewed for new problems.
- **Extra code read:** `crates/raf-source/src/workspace.rs:188-304`, the public item accessors.

### Prior findings

| # | Status | Evidence in revision 2 |
|---|---|---|
| B1 ordering | **DISCHARGED** | L35-37: R2 → R3 → R8 → R4 …; R1 → R12o; "R7 starts only after R1 lands"; docs edits stacked. |
| B2 R1 bound | **DISCHARGED** | L69-73: differential bound `peak(long) <= peak(short) + …`, a 1 GiB allocator ceiling, a child-process HEAD run, and the PROPOSED-block pair. |
| B3 R3 budgets vs "cannot exhaust a machine" | **STILL-OPEN** | See N1. L102 cannot coexist with the public `&Item` API, and L103 and L104 do not yet give a bound that both holds and admits real code. Part of this is my own earlier REWRITE TO, which N1 corrects. |
| B4 `FileId` on non-Unix | **DISCHARGED** | L97-100: (dev, ino) on Unix, the canonical root-relative path elsewhere ("no `#![feature]`, no `unsafe`"), and the normalised key on Memory. L101 gives the kept spelling. |
| B5 neighbour fixtures and false-refusal checks | **DISCHARGED** | L110, L112-113, L115 (R3); L227, L229 (R5c); L289 (R10); L59 (registry false-refusal rule). |
| B6 A2 imports and syntax | **DISCHARGED** | L161-174: extern-prelude resolution via the importer's dependencies, `resolve_in_package(package, path)`, and the fixtures. L188 shows R5a uses "the R4 table". |
| B7 A4 rule and result | **DISCHARGED** | L208-215: SCC over the explicit-import graph, globs never settle a name an explicit import binds, and both `Err(Cyclic)` in both orders plus the neighbour. |
| B8 fixture lists | **DISCHARGED** | R2 at L82-91; R5a at L191-199, including `{self, ..}`, `use std;` + `&Foo` and `{self as s2}`; CA-F7 alias chain at L199. |
| B9 R6 specification | **DISCHARGED** (advisory R2-1) | L247-252: syn-derived token set including `box`, `static`, `as`, `if`/`match`/`let`, `..`, `extern`; resets; frame cost covering parse, visitor and drop. L257, L259: paired fixtures and a 0-refusal check. |
| B10 R8 docs and strategies | **DISCHARGED** | L126: OtherUnix and Portable re-verify after `read_dir`, reads and `entry`. L133-137: the enumerated sentences. L140-141: swap and `entry` tests under three strategies, plus the committed FIFO-race test. |
| B11 S-14 | **DISCHARGED** | L130: `cfg(windows)` only, `RelPath` unchanged. L142-143: the Linux `aux/` + `nul.txt` neighbour and the Windows unit test. |
| B12 A14 | **DISCHARGED** | L304: no `Send`/`Sync` bound, with a Constraints sentence (case 3). |
| B13 A15 | **DISCHARGED** | L305-306: `SpanFact.lines: Range<usize>`, half-open, matching `raf_okf::Span`, with the test. |
| B14 non-falsifiable checks | **DISCHARGED** (advisory R2-4) | L260 grep with `-rni` and the HEAD hits named; L280 `--exact` with "1 passed"; L311 full-field destructuring; L52, L322, L330, L333 mutations; L152-179 and L298 named fixtures. |
| B15 declines | **DISCHARGED** | P-F10: L106, L113. A12: L236. A16: L307-309, L358. A10: L343, L360. |
| B16 unmapped findings | **DISCHARGED** | S-13 L344; S-12 L267-269; Verbatim L362; CA-F6 tail L363; Q-3 L321; A9 L300, L359; A11 L301-303; checkability L212; CI jobs L138; HF-R4 L364; HF-D14 L365; P-F8 tail L107; G10 L345. |

### New problems found in revision 2

**N1 (BLOCKING, the reason B3 is still open). R3's storage change contradicts the public API, and the item budget is either unsafe or refuses real code.**

> L102: "**Item records:** a file's items are recorded once per (file, target). Each further arrival records only (module path, file id), so the cost per arrival is O(1)." / L103: "**Item-record budget:** sized so that budget × the measured bytes per record ≤ 512 MiB." / L104: "**Total-bytes budget** (`max_total_bytes`) per load: set from the registry scan's largest observed value × 4, with a minimum of 256 MiB."

That L102 sentence came from my own B3 REWRITE TO, and it was wrong.

- **The API forces one stored `Item` per path.**
  - `Workspace::items()`, `items_under()`, `items_in()`, `item(id)` and `item_at()` all hand out `&Item` (`workspace.rs:206-243`). `items_by_source()` and `same_source()` group `ItemId`s per path instance (`:295-304`).
  - The Contract says a file reached by several paths "is indexed once for each path, so the same source item can appear as several `Item` values that ... differ in `path`" (`source-index.md:413-416`).
  - A borrowed `&Item` per path needs one stored `Item` per path, so the cost per arrival stays O(items).
  - Delivering O(1) needs two things:
    - an API change: owned or view items, with new id semantics;
    - a change to a Contract sentence that is unlabelled and covered by no user decision.
- **The 512 MiB sizing has no feasible value.**
  - At the per-record cost P-F4 measured (about 3 KB: 524k items in 1.58 GB), "≤ 512 MiB" gives about 175k item records per load.
  - `web-sys` (1,727 generated files) and large monorepos plausibly exceed that.
  - The plan's own L115 / L59 check (0 crates refused by the new budgets) and L108 ("cannot exhaust a machine") would then contradict each other, with no stated way out.
- **The total-bytes budget has a floor but no ceiling.** L104 lets the retained source text grow to 4× the largest crate seen, with no upper limit. The Contract then cannot state a worst case for "cannot exhaust a machine" from it.

**REWRITE TO:** "R3 item records:
- **Storage:** keep one `Item` per (path, target), as `source-index.md:413-416` and the `&Item` API require.
- **Smaller records:** cut the per-record cost by sharing per-file and per-source data. `file`, the written `self_type`/`trait_path`, the span and documentation flags live in one per-source record that each `Item` references by index or `Arc`, and module paths are interned.
- **Target:** at most 256 bytes per `Item`, measured and written into the Contract.
- **Item-record budget:** = 512 MiB ÷ the measured bytes per `Item`, about 2M at 256 B.
- **`max_total_bytes`:** = min(1 GiB, max(256 MiB, 4 × the largest scan value)).
- **Contract:** states worst-case retained memory as item budget × bytes per item + `max_total_bytes`, and keeps 'cannot exhaust a machine' at that figure.
- **Checks:** unchanged (L110-115), plus a unit test that asserts `size_of` of the per-path `Item` record is ≤ the stated figure.

If the owner prefers O(1) per arrival, that is an API and Contract change to `source-index.md:413-416`, and it needs a recorded user decision."

**Advisories on new text (non-blocking):**

- **R2-1 (R6, L248, L251).**
  - The audit list includes "atom" as a counted unit. An atom (an identifier or literal) opens no frame, and counting it doubles every binary chain and adds false-refusal pressure for no safety gain.
  - The revision also no longer states how levels combine.
  - **REWRITE TO:** drop "atom"; and "the budget is the sum, over open delimiter levels, of each level's count, plus one per open delimiter."
- **R2-2 (R5a, L189).**
  - "A name that is both a local item and an extern-prelude crate is `Ambiguous`" should be limited to the type namespace. rustc's E0659 concerns a module-scope type-namespace name against an extern crate; `fn serde()` beside the `serde` crate is not ambiguous.
  - **REWRITE TO:** "… a type-namespace name of the module that is also an extern-prelude crate …"
- **R2-3 (B1 residue, L36-37).**
  - R12o edits `tests/docs_contract.rs` (G17 at L325), as do R2 (L90) and R11 (A8, A11).
  - The stacking rule in L37 names only `source-index.md`, `SECURITY.md` and `AGENTS.md`.
  - **REWRITE TO:** add `tests/docs_contract.rs` and `okf-profile.md` to the stacked files, and state that R12o lands before R7 starts.
- **R2-4 (L52).** The mutation rule lists Q-7 but names no mutation for it. Name one: map CRLF to a two-line advance in the line counter.
- **R2-5 (L344).** Check `unicode-ident@1` against cargo-deny 0.20.2's package-spec syntax before relying on it. If `@` means an exact version, use the documented requirement form. `make deny` (L345) would catch a mismatch, so this costs only time.
- **R2-6 (L316 and L230).** Commit types do not match content:
  - R11's single `refactor` commit also carries a behaviour fix (`items_under`), new fields (A6) and a breaking `Copy` removal (A9);
  - R5c (`perf`) adds a refusal.
  - **REWRITE TO:** split R11 into `feat(raf-source)` (A6 fields, `resolve_in_package` docs), `fix(raf-source)` (A16 `items_under`) and `refactor(raf-source)` (sealing, `pub(crate)` facts, `Copy`). Give R5c `fix(raf-source)`.

### Areas the coordinator flagged

- **R3 budget sizing vs "cannot exhaust a machine":** still open (N1).
- **R5a-d split:**
  - Each commit is independently checkable.
  - R5a depends on R4's extern-prelude table (L188), and the order allows it.
  - R5b's checkability assertion (L212) lands before R5c rewrites the glob tables, so it guards R5c, which is the right order.
  - No new problem beyond R2-2 and R2-6.
- **R8 per-platform doc list and strategies:**
  - Complete for every sentence B10 named (L133-137), and the strategies are consistent (L120-126).
  - L146 keeps non-Linux runtime reported as unverified.
- **R11 A14 and A15 directions:** correct (L304-306).
- **Declines list (L356-365):** each reason now matches its source text. The `Item::Verbatim` decline cites the binding plan C7 text (plan.md:326), and HF-D14 cites a concrete need (`facts.rs:12`).

VERDICT: NEEDS-REVISION

---

## Recheck (revision 3)

- **Scope:** N1 and the six revision-2 advisories, plus anything those edits broke. Line numbers refer to revision 3.

### Prior findings

| # | Status | Evidence in revision 3 |
|---|---|---|
| N1 / B3 (R3 item records and budgets) | **STILL-OPEN, narrowed to N2** | **Fixed:** L104 keeps one `Item` per path, citing `source-index.md:413-416` and the `&Item` API, with no API or Contract change. L105-106: records ≤ 256 B through shared per-file data and interned paths, with a `size_of` + heap test. L107: item budget = 512 MiB ÷ measured bytes (≥ 2M). L108: `max_total_bytes = min(1 GiB, max(256 MiB, 4 × largest))`. **Still wrong:** L109's combined figure omits two memory components, so the Contract sentence it mandates would be false. See N2. |
| R2-1 (R6 atom; accumulation) | **DISCHARGED** | L253: no "atom" in the expression list. L257: "the sum, over all currently open delimiter levels, of each level's current count, plus one per open delimiter". |
| R2-2 (R5a namespace) | **DISCHARGED** | L194: "a local item in the type namespace … value-namespace items do not collide". |
| R2-3 (stacking; R12o before R7) | **DISCHARGED** | L38: "R7 starts only after R1 and R12o land". L39 adds `okf-profile.md` and `tests/docs_contract.rs` to the stacked files. |
| R2-4 (Q-7 mutation) | **DISCHARGED** | L344: "drop the BOM strip or CR handling in line counting". |
| R2-5 (cargo-deny pin syntax) | **DISCHARGED** | L354: syntax confirmed by "`make deny` failing on a deliberately wrong pin, then passing". |
| R2-6 (commit types) | **DISCHARGED** (advisory R3-1) | L323-325: R11 is split into `feat(raf-source)!`, `fix(raf-source)` and `test(docs-contract)`. L235: R5c is `fix(raf-source)`. |

### New problems found in revision 3

**N2 (BLOCKING). The "combined worst case ≤ about 2 GiB" leaves out two memory components, so the Contract sentence it mandates would be false.**

> L109: "**Combined worst case:** item records + total bytes + glob edges (R5c) ≤ about 2 GiB. The Contract states this figure, with the measured time at each limit."

The three listed budgets add up correctly: 512 MiB + 1 GiB + 512 MiB. But a load's peak memory has two more components, and neither is bounded by those three budgets:

- **The per-source data moved out of `Item`.** L105 moves file id, text spans, attributes and docs into shared per-file and per-source records. Those records grow with the number of source items, not with item records, and no budget or measured factor bounds them. They are bounded only indirectly, through `max_total_bytes` times a fact-per-byte factor that is not measured.
- **The transient parse peak of one file.**
  - performance-review §2 measured that "Whole-load cost on a valid 8 MiB file is `syn`: `a<b; ` repeated took 5.1 s and 2.6 GB (about 300x the file size)".
  - One admitted file at the default `max_file_bytes` therefore peaks above 2 GiB on its own, before any retained index.
  - Parses are serial (the dedicated thread per file is joined before the next file), so the peak adds at most one file's parse.

**Consequence:** the Contract would state a load bound of "≤ about 2 GiB" that a single 8 MiB file in the measured shape exceeds, while every budget passes. That is the same class of false resource sentence B3 was opened for ("a fraction of a second and a few tens of megabytes").

**REWRITE TO:** "**Combined worst case** = retained + transient, each measured and written into the Contract.
- **Retained** = item records (≤ 512 MiB) + per-source records + source text (`max_total_bytes` ≤ 1 GiB) + glob edges (≤ 512 MiB).
  - Per-source records are bounded by a per-source-record budget sized ≤ 256 MiB, or by a measured bytes-of-facts-per-source-byte factor × `max_total_bytes`.
  - Either way, they get an at-limit fixture and an over-limit fixture.
- **Transient** = the parse of one file at `max_file_bytes`, measured on the worst admitted shape (performance-review §2: about 300× the file size, so about 2.5 GiB at 8 MiB).
- **Contract sentence:** 'a load holds at most R of index and at most T more while one file is parsed', with R and T as measured.
- **Choice:** either keep 'cannot exhaust a machine' and lower the default `max_file_bytes` or add a token-count budget so that T stays within a stated figure; or state R + T as the figure.
- **Probe:** the performance probe (`a<b; ` × 8 MiB) is re-run, and its peak is recorded against T."

**Advisory (non-blocking):**

- **R3-1 (commit content).**
  - The `feat(raf-source)!` commit at L323 carries A8, but A8 changes raf-okf types (`Outcome`, `LinkTarget`) and `tests/docs_contract.rs:203, 440`.
  - The `fix(raf-source)` commit at L324 carries A15's `SpanFact` field change, which breaks every struct literal and pattern that names `start_line`/`end_line`.
  - **REWRITE TO:** move A8 into its own `feat(raf-okf)!: make Outcome and LinkTarget exhaustive` commit, and move A15 into the `feat(raf-source)!` commit, or mark that commit `fix(raf-source)!`.

### Edits checked for side effects (none broken)

- **R3:**
  - With one `Item` per path kept, `src_load` (L116, L119) checks against "the stated figures" rather than a fixed 100 MB, so it is consistent.
  - L106's heap measurement needs a counting allocator. Put it in an integration test binary, so the lib unit tests keep the system allocator.
- **R5c (L230):** the glob-edge budget sized "like the R3 item budget against ≤ 512 MiB" agrees with L109's sum.
- **R6:** the accumulation rule (L257) agrees with the reset rule (L256). A closing delimiter discards its level's count, so resets inside a level and closure of the level compose.
- **Ordering (L35-39):** R12o now precedes R7. R11's A8 edit of `tests/docs_contract.rs` is stacked after R12o's G17 edit and R2's fixture, so no parallel edit of that file remains.

VERDICT: NEEDS-REVISION

---

## Recheck (revision 3.1)

- **Scope:** N2 and R3-1 only. Line numbers refer to revision 3.1.

| # | Status | Evidence in revision 3.1 |
|---|---|---|
| N2 (combined worst case) | **STILL-OPEN, narrowed to N3** | **Fixed:** L110 adds a per-file pre-scan token budget so that the one-file parse peak is ≤ 512 MiB, with at-limit and over-limit fixtures and 0 registry refusals. L111 states two Contract figures (retained, and one-file parse peak), with the measured time at each limit. It also records the re-run `a<b; ` 8 MiB probe against figure (2). **Still wrong:** L109 bounds per-file data by a factor measured only on benign input, so the retained figure does not hold for hostile input. See N3. |
| R3-1 (commit content) | **DISCHARGED** | L325: A15 moves into `feat(raf-source)!: seal public types, add target test settings and use half-open line ranges`. L326: A8 is its own `feat(raf-okf)!` commit, including the `docs_contract` arm removal. L327: `fix(raf-source)` carries only A16. |

### New problem found in revision 3.1

**N3 (BLOCKING). The per-file data factor is measured on benign code, so figure (1) is not a bound for the untrusted input the Contract covers.**

> L109: "**Per-file data budget (N2):** the per-file side tables (attributes, docs, use trees, impl headers, spans) are measured and bounded as a stated factor of the retained source bytes. A unit test asserts that the measured factor on this repository and the largest scanned crate is ≤ the stated factor, so the factor's bound follows from `max_total_bytes`."

- **The factor is an average, not a bound.**
  - A factor measured on this repository and the largest registry crate is an average over ordinary code.
  - Hostile source can push the facts-per-byte factor far higher.
    - For example, `use a::{b,c,d,...}` gives one use-tree node for every two source bytes.
    - Runs of `#[a]` give one attribute fact for every four bytes.
    - Thousands of `fn f(){}` give one item, span set and attribute vector for every nine bytes.
  - With node sizes of tens of bytes, the factor on such input is 10-50×. At `max_total_bytes` = 1 GiB that is 10-50 GiB of facts. Every budget passes, and the benign-factor test stays green.
- **So figure (1) can be exceeded by an input every check admits.**
  - That repeats the false-resource-sentence class that B3 and N2 targeted.
  - The source-index Contract covers sources the reader does not control, as `SECURITY.md:46-50` says: "The in-process parsers read Rust ... that the reader may not control".
- **This is my error too.** The N2 rewrite offered "a measured bytes-of-facts-per-source-byte factor" without requiring it to be the worst-case factor.

**REWRITE TO:** "**Per-file data:** bounded by one of the following.
- **(a) Fact-record budget per load.** A typed `Budget` counts every use-tree node, attribute, impl header and span record, with budget × measured bytes per record ≤ 256 MiB. It gets an at-limit fixture and an over-limit fixture, and the registry scan shows 0 refusals.
- **(b) Worst-case factor.** The stated factor is the maximum bytes-of-facts per source byte over adversarial fixtures, one per fact kind at its densest spelling: a `use` group of single-letter names, a run of `#[a]`, a run of `fn f(){}`, and `impl` headers. The unit test asserts that the measured factor on those fixtures, as well as on this repository and the largest crate, is ≤ the stated factor. Figure (1) uses that factor × `max_total_bytes`.

Option (a) is preferred, because it does not depend on finding the densest spelling. The same note applies to L110's 'measured bytes per token': it must be measured on the costliest admitted shape. The `a<b; ` probe at L111 checks this, and it must stay within figure (2)."

### Edits checked for side effects (none broken)

- **Token budget:** L110's token budget is a new pre-scan refusal. Its fixtures and its 0-refusal registry check satisfy B5 and AGENTS.md §5. It does not conflict with R6's recursion budget, because they are separate counters in one lexer pass.
- **Contract:** L111's two-figure statement replaces L109's single 2 GiB figure from revision 3, and the probe is tied to figure (2).
- **Commits:** L325-327 now match their content. A8 still runs at R11's position, after R12o (L38), so its raf-okf and `docs_contract` edits stay ordered.

VERDICT: NEEDS-REVISION

---

## Recheck (revision 3.2)

- **Scope:** N3 only. Line numbers refer to revision 3.2.
- **Extra code read:** `crates/raf-source/src/facts.rs:95-279`, the owned fields of the fact records.

| # | Status | Evidence in revision 3.2 |
|---|---|---|
| N3 (per-file data bound) | **STILL-OPEN, narrowed to N4** | **Fixed:**<br>- L109: a typed fact-record budget per load replaces the benign-code factor. It covers attributes, docs, use-tree leaves, impl headers, mod declarations and spans.<br>- L110-111: at-limit and over-limit fixtures, and 0 registry refusals.<br>- L112: bytes per token measured on the costliest admitted shapes, each with a fixture.<br>- L113: figure (1) is itemised as 512 MiB + ≤ 1 GiB + 256 MiB + 512 MiB.<br>**Still wrong:** the budget counts records and assumes a fixed size per record. The fact records own variable-length strings, so the bound fails. See N4. |

### New problem found in revision 3.2

**N4 (BLOCKING). A count of fact records times a fixed per-record size does not bound fact memory, because the records own variable-length strings copied from the source.**

> L109: "It counts records, sized so that budget × the measured bytes per record ≤ 256 MiB. The bytes per record are measured on the costliest fact kind at its densest admitted spelling, with an adversarial fixture per kind."

- **The fact types carry owned strings whose length is set by the input, not by the kind:**
  - `AttrFact { path: String, args: String }` (`facts.rs:117-123`). `args` is "the rest of the attribute, such as `(Debug)` or `= "text"`". A `///` comment is a `#[doc = "..."]` attribute, so one doc record holds its whole text.
  - `ImplFact { trait_path: Option<String>, self_type: String, self_path: Option<Vec<String>>, generics: Vec<String>, .. }` (`:255-279`). The written self type and trait path are stored twice, as text and as segments.
  - `PathFact { segments: Vec<String> }` (`:97`).
  - Use names, `renamed_from`, cfg `predicate` and `path` (`:130-188`).
- **A record count therefore bounds only the fixed part of each record.**
  - One `#[doc = "<8 MiB>"]` record, or one `impl` header with a huge written type, is one record and holds megabytes.
  - Across `max_total_bytes` = 1 GiB, these copies add about 1-3 GiB on top of figure (1)'s 256 MiB. The multiplier comes from token rendering through `quote`, which inserts spaces, and from storing self and trait paths both as text and as segments.
  - Every budget passes.
- **The densest-spelling measurement cannot fix this.** Measuring the costliest kind at its densest spelling picks the worst fixed size per record. Any payload in the adversarial fixture either:
  - inflates the per-record figure, so the budget becomes too small and the registry check fails; or
  - is left out, so figure (1) is false.
- **This is my error too.** My N3 option (a) said "counts every ... record", with no mention of the payloads.

**REWRITE TO:** "**Fact budget, in bytes:**
- **What it charges:** the typed per-load budget charges each fact record's fixed size, plus the byte length of every `String` and `Vec` payload it owns. That covers attribute `path`/`args`, doc text, written `self_type`/`trait_path`, segments, generics, use names, cfg predicates and `path` values.
- **Limit:** 256 MiB.
- **Where it is charged:** where the fact is built, on the parse thread, so a single oversized file is refused while it is being extracted, with a typed `Budget` naming the file and line.
- **Fixtures:**
  - at the limit and one byte over;
  - an adversarial payload fixture per kind: a 1 MiB doc comment, an `impl` with a 1 MiB written self type, and a `use` group of long names;
  - the existing densest-spelling fixtures.
- **Registry scan:** 0 refusals.
- **Figure (1):** unchanged (512 MiB + ≤ 1 GiB + 256 MiB + 512 MiB), and now true by construction."

**Optional alternative:** stop owning the copied text. Store byte ranges into the retained source, which is already counted under `max_total_bytes`, and keep a record count. That also cuts memory, but it changes the `pub` fact types. That change is allowed here only because A5 already makes them `pub(crate)` in R11, and R11 lands after R3, so the ordering must account for it.

### Edits checked for side effects (none broken)

- **Figure (1) arithmetic:** L113 is correct, at 2,304 MiB. The token budget (L112) and figure (2) are consistent with the probe at L113.
- **Fixtures for the new budgets:** L119's at/over list does not mention the fact-record and token budgets, but L110 and L112 give their fixtures.
- **R6:** the fact budget is charged during fact extraction, and R6's recursion budget is charged in the pre-scan, so the two are independent.

VERDICT: NEEDS-REVISION

---

## Recheck (revision 3.3)

- **Scope:** N4, and the item-budget change the coordinator made alongside it. Line numbers refer to revision 3.3.

| # | Status | Evidence in revision 3.3 |
|---|---|---|
| N4 (fact budget in bytes) | **DISCHARGED** | L111: a typed per-load fact budget "counted in **bytes** with a 256 MiB limit", covering attributes including doc comments, use-tree leaves, impl headers, mod declarations, cfg predicates, `path` values and spans.<br>L112: "fixed `size_of` plus the length of every `String` and `Vec` it owns".<br>L113: charged on the parse thread, refused typed naming the file and line.<br>L115-116: a large-payload fixture per kind (1 MiB doc comment, 1 MiB written self type, `use` group of long names), plus at-limit and one-byte-over fixtures.<br>L117: 0 registry refusals.<br>L118: figure (1) holds by construction. |
| Item budget in bytes (coordinator's addition) | **ACCEPTED, sound** | L107: "counted in bytes with a 512 MiB limit. Each `Item` charges its `size_of` plus every owned or interned string the first time it is interned: path segments, names, written self types". L108: a long-path or long-name fixture plus at-limit and one-byte-over. L109: 0 registry refusals.<br>This closes the same hole N4 found, now for items: names and module paths are input-sized.<br>Charging interned strings once is correct, because sharing is the point of interning. Each arrival's distinct module path is a new interned string, so the S-2 symlink and diamond shapes are still charged per arrival. |

### Advisories (non-blocking)

- **R3.3-1 (L106).**
  - "`size_of::<Item>()` plus the measured heap per record ≤ 256 bytes" no longer matches the budget model. The heap part is now input-sized and charged by L107, so a 256 B cap on it cannot hold for long names.
  - **REWRITE TO:** "A unit test asserts `size_of::<Item>()` ≤ 256 bytes. Owned and interned payload is charged by the item-bytes budget (L107)."
- **R3.3-2 (L107).**
  - Interning has per-entry overhead (the table slot and the `Arc` header) beyond the string's length.
  - **REWRITE TO:** "... charges the string's length plus a fixed per-entry overhead the first time it is interned."
- **R3.3-3 (L120, L126).**
  - Figure (1) still labels the first term "item records (512 MiB)", and the at/over list at L126 says "item records".
  - **REWRITE TO:** "item bytes (512 MiB)" in both places, so the Contract names the unit the budget actually charges.
- **R3.3-4 (L241, glob-edge budget, R5c).**
  - "sized like the R3 item budget against ≤ 512 MiB" now reads as a byte budget, which is right. Today's glob bindings carry an `Origin` with a `Link` holding two `String`s (performance-review F7), which is an input-sized payload.
  - **REWRITE TO:** "a glob-edge budget counted in bytes (`size_of` plus every owned payload; shared `Arc` name sets charged once), 512 MiB." That keeps the glob term of figure (1) true by construction, like the fact and item terms.

### Edits checked for side effects (none broken)

- **Figure (1):** it still sums to 2,304 MiB (L120), and each term now has a byte budget behind it: items at L107, source text at L110, facts at L111, glob edges at L241 (see R3.3-4).
- **Charging location:**
  - The fact budget is charged on the parse thread (L113).
  - The item budget is charged at index build. That is on the caller's thread, but it allocates no deep recursion, so it does not interact with R6 or the P-F10 explicit stack.
- **Registry checks:** the false-refusal checks for both new byte budgets (L109, L117) feed the global rule at L59.

VERDICT: PASS
