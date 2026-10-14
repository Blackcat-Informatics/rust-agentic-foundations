**Current status (recheck 3, 2026-10-12): VERDICT: PASS.** REQUIRED 1-3, R-4 and R-6 are resolved. On every shape probed, index-side heap is 0.30-0.92 of the item-bytes charge, and figure (1) holds. Open items are advisory only (see "## Recheck 3").

# R3 review: file identity, traversal and work budgets

Reviewer: independent (opus/high). Tree: HEAD f5ede44 plus the uncommitted R3 working tree.
Contract: remediation-plan.md "### R3" (N1-N4, R2-review advisories 3/4, P-F8, P-F10), "Mutation rule",
"Per-item gates", "Registry harness"; the user decision in validation.md ("(1) - default to 2M").

## Scope

- `git diff` (30 files) plus the new files `crates/raf-source/src/{meter,intern}.rs` and
  `crates/raf-source/tests/{load_budgets,walk_stack}.rs`.
- Read in full: meter.rs, intern.rs, tree/{mod,memory,disk}.rs, modtree.rs (new and old), index.rs and
  impls.rs (charging), item.rs, options.rs, budget.rs, prescan.rs (token count), discovery.rs, members.rs,
  settle.rs (`Bound`), workspace.rs, registry_scan.rs, the new and changed tests, and the source-index.md,
  SECURITY.md and CHANGELOG.md changes.
- Declared out-of-plan changes were checked: the settle.rs `Bound` sorted list, the line-position fix and
  the public API changes (`Tree::identity`, `FileId`, `SharedStr`, `Option<Arc<TraitKey>>`, `Usage`,
  `Workspace::usage`, the new `LoadOptions` fields and `Budget` variants).

Scratch copies were made outside the repository: `/tmp/r3rev/old` (`git archive f5ede44`),
`/tmp/r3rev/new` (rsync of the working tree) and `/tmp/r3rev/mut`. All probes ran there. The worktree is
unchanged apart from this file.

## Evidence

### Gates (worktree, `ulimit -v 24 GiB`)

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test -p raf-source --locked` (debug) | exit 0, 285 tests |
| `cargo test -p raf-source --release --locked` | exit 0, 285 tests |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked` | exit 0 |
| `cargo test --locked --test docs_contract --test repo_gates` | exit 0 (139 + 55) |
| `cargo test --workspace --locked` | exit 0 |
| `make cross-check` | exit 0 |
| `make msrv` (1.98.1) | exit 0 |
| `make reuse` | exit 0 (219/219 files) |

- `cargo check --target x86_64-pc-windows-gnu --all-targets` fails only on the lib test with
  `within_watchdog is never used`. The same failure exists on f5ede44, and R8 owns it.
- Every integration test target, including the new ones, checks clean for Windows.
- No `unsafe`, no `#![feature]` and no lint change was added. `grep` of the added lines found no process
  wording or issue/PR numbers.

### Mutation rule

The mutations were run in `/tmp/r3rev/mut` with the script `/tmp/r3rev/mutate.sh` (output in
`mutate.out`). Each was reverted after its run.

| Mutation | Killed by |
|---|---|
| meter `sum > limit` changed to `>=` | `index_records_at_the_item_budget…`, `the_bytes_read_at_the_total_budget…` |
| facts charge `>` changed to `>=` | both fact-budget tests |
| prescan `tokens > max_tokens` changed to `>=` | both token tests |
| `depth > DirectoryDepth` changed to `>=` | all three depth tests in walk_stack.rs |
| drop the "keep the spelling that sorts first" update | `module_tree::a_file_reached_under_two_spellings…` |
| cycle check by identity disabled | `a_module_that_reaches_its_own_file_through_a_directory_symlink_is_a_cycle` |
| `MemoryTree::identity` keyed by spelling | `memory_tree::the_identity_of_a_file_is_its_normalised_key…` |
| `DiskTree::identity` keyed by spelling | `disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode…` |

Falsification on old code: the implementer's shims (`raw/R3-head-shims.rs.txt`) were re-run on f5ede44.

- The 20,000 memory chain on 2 MiB, 128 file modules on 512 KiB, 128 inline modules on 512 KiB and the
  129-level member pattern all abort with a stack overflow.
- The directory-symlink spelling, alias self-reach cycle, memory two-spellings, 200-level disk chain and
  token-limit shims fail.
- The implementer ran these on 0617985, not 6c744af. They need R2, so that is acceptable.

### (1) FileId / identity

- **Same refusals as `read`.** `DiskTree::identity` goes through the same `open_verified` as `read`
  (`contain`, then an `O_NONBLOCK|O_NOFOLLOW` open, then `verify_open_handle`). It then takes `(dev, ino)`
  from the verified handle's metadata.
  - It refuses exactly what `read` refuses, except `TooLarge` and `NotUtf8`.
  - It opens nothing that `read` would not open, so the containment rules hold.
  - The outside-symlink fixture gets `OutsideRoot`.
- **Sharing identities on Unix.** Two files share an identity only through a hard link (one inode) or a
  bind mount of the same filesystem. In both cases it is one file.
- **Non-Unix.** The identity is the canonical root-relative path that `contain` computed.
  - Windows `canonicalize` returns the on-disk case, so case-insensitive spellings collapse.
  - A Windows hard link gives two identities: the file is parsed twice. That is not wrong.
- **MemoryTree.** The identity is `collapse(path)`, the same key that `read` uses.
- **Tests.** The test suite has the fixtures for one identity per file (symlink, hard link, directory
  symlink, `..`) and for refusing what `read` refuses.

### (4) and (5) Behaviour preserved

A dump example was built against both the old and the new code. It prints:
- every file with its reaches;
- every `Item` (full `Debug`);
- `items_by_source`;
- `resolve_in` for every item path in all three name spaces, plus `resolve`.

The output was compared with `cmp`:

- **This repository at f5ede44:** 13,257 lines, identical.
- **This repository with the R3 tree:** 14,454 lines, identical.
- **161 registry crates:**
  - The sample was every 8th OK crate and every 6th ERR crate, plus aho-corasick, brotli, wasmi,
    chrono-tz, serde, regex-syntax, tokio, syn, petgraph, proptest, tower and futures-util.
  - Result: 161/161 identical, 328,356 lines in total. That is 131 loads with identical items and
    resolutions, and 30 identical first errors.

So module order, cycle detection, spans and resolution, including the settle.rs `Bound` change, are
unchanged on real code.

### (7) Registry harness

The full scan was re-run with the committed harness:
`registry_scan <index> --jobs 8 --max-memory 6G`.

- The output is line-for-line identical to `raw/registry-scan-R3.txt`.
- Tally: 1,021 OK, 0 misplaced and 0 `Budget` errors.
- web-sys ends with `ERR memory` through the `sh -c 'ulimit -v …; exec "$0" "$@"'` wrapper.
- Largest values: total 16.9 MB, item 304.7 MB (linux-raw-sys), tokens 1,505,567 (chrono-tz).
- The `#[macro_export]`-only oracle looks up the item's fact by span start through inline bodies. It is
  correct.

### Security probes re-run (`/tmp/raf-sec`, rebuilt into a fresh target)

The old binaries in `/tmp/raf-sec/target` were stale. Cargo reported them `Fresh`, but they were built
before the R3 edits, so a fresh build was used.

- `mem_deep 20000`: typed `directory nesting (128)` in 0.5 ms.
- `src_load_thread 2097152 deep`: typed in 22 ms.
- The S-2 amplification tree `ampbig` (l/m links, 18 levels, 64 KiB pad): typed `ItemBytes` in 4.0 s,
  maxrss 214 MB.
- The amp0/4096/65536 self-link trees: typed `ModuleCycle` in under 1 ms.

### (2) Budget honesty: counting-allocator probes

Probe source: `/tmp/r3rev/new/crates/raf-source/examples/probe.rs`, release, default `LoadOptions`.
"held" is the heap still live after the load; "charged" is total + item + fact from `Usage`.

| Probe | Source | Held / peak | Charged (total / item / fact) |
|---|---|---|---|
| `alias 10 300000`: 2^10 arrivals at a file with `pub type T = a::a::…` of 300 KB | 300 KB | 2,450 / 2,451 MiB | 0.3 / 4.5 / 2.4 MiB |
| `alias 12 300000` | 300 KB | allocation failure under a 12 GiB cap | (none) |
| `externcrate 12 1000000`: 2^12 arrivals at `extern crate c<1 MB> as x;` | 1 MB, 18 tokens | 3,931 MiB | 1.0 / 18.3 / 1.0 MiB |
| `glob 3000 3000`: one module of 3,000 fns globbed by 3,000 modules | 200 KB, 66k tokens | 3,735 / 7,459 MiB | 0.2 / 11.4 / 2.4 MiB |
| `structs 262145 12 3` | (n/a) | 409 / 426 MiB | 6.0 / 368 / 62.8 MiB (ratio 0.94) |
| `structs 2000 100000 40` (100 KB names) | (n/a) | 956 MiB | 191 / 384 / 191 MiB (ratio 1.25; item side about 1.5x) |
| `reexport 40 80000` (long-name renaming re-exports) | (n/a) | 52 MiB | 9.2 / 12.3 / 9.2 MiB (item side about 2.7x) |
| `attrs 262145 8` (2^18+1 attributes per file) | (n/a) | 202 MiB | 8.0 / 0 / 98 MiB (fact side about 1.9x) |

### Figure (2): parse peak per token

Probe source: `examples/peak.rs` calling `parse_source`, peak above the text.

- **The doc's own shape `statement_lists`:** 1,162 (100k), 1,209 (300k), 1,162 (400k), 1,140 (475k),
  1,210 (600k), 1,134 (1M) and 1,135 (2M, 2,164 MiB) bytes per token.
- **One body of 2^k+1 empty statements:** 1,458 bytes per token at 65,543, 262,151 and 1,048,583 tokens.
- **The doc's shape with the big list as large as the budgets admit** (2^k+1 ≤ t rather than ≤ t/2):
  - 475,000 tokens: 571 MiB (1,261 bytes per token);
  - 2,000,000 tokens: 2,383 MiB (1,249 bytes per token).
- **Other shapes at 400k tokens:** all below 630 bytes per token. These were arrays, tuples, lifetimes,
  fields, match arms, use groups, paths, macro bodies, calls, attributes, blocks, items, consts, `let`,
  closures, refs, generic arguments and `where`.

## Findings

1. **REQUIRED: the item-bytes budget misses two per-arrival copies, so retained memory is unbounded while
   every meter stays far below its limit.**
   - The copies are made in `index.rs:419` (`self.index.aliases.insert(id, (home, alias.clone()))`, a
     `PathFact` whose segments are owned `String`s) and `index.rs:423`
     (`self.index.extern_names.insert(id, crate_name.to_owned())`).
   - Both run once per arrival of the module walk. Neither copy is charged:
     - the item charge counts the item's interned path and name only;
     - the fact budget charges the file's own facts once.
   - A file reached 2^k times (a chain of files that each declare the next twice, within `ModulePaths`)
     holding a long type alias or `extern crate <long> as x;` keeps arrivals × length:
     - 2,450 MiB from 300 KB of source;
     - 3,931 MiB from 1 MB and 18 tokens;
     - the meters stay at 7 to 20 MiB;
     - this extrapolates to tens of GiB at k = 16, where the item charge is still under 512 MiB.
   - This falsifies figure (1) and "cannot exhaust a machine" at the defaults.
   - Fix: share these through the file's facts (as `ImplDecl` now borrows its `ImplFact`), or charge their
     owned bytes per item.
   - Add a fixture: a file reached many times holding a long alias path, and one holding a long
     `extern crate` name. Each is refused by `ItemBytes`, or held within the figure, measured.
   - Sweep `Builder::item`/`push_member` for any other per-arrival owned copy (the `assoc` and `foreign`
     keys are bounded by the charged path; these two are not).

2. **REQUIRED: the figure (2) constant of 1122 bytes per token is not an upper bound, so the stated parse
   peaks are wrong.**
   - Where it is stated:
     - `budget.rs` (`FILE_TOKENS` doc and `Budget::FileTokens`);
     - `options.rs` (`max_file_tokens`);
     - SECURITY.md, Bounds bullet;
     - source-index.md, the token bullet and figure (2).
   - What these say:
     - "at most 1122 bytes per token";
     - "about 2.1 GiB at the default";
     - "475,000 tokens keep the peak below 512 MiB".
   - Measured with a counting allocator:
     - one body of 2^k+1 empty statements costs 1,458 bytes per token;
     - the documented shape with its large list sized as the other budgets admit reaches 2,383 MiB at
       exactly 2,000,000 tokens (stated: about 2.1 GiB, "2137 MiB");
     - the same shape reaches 571 MiB at exactly 475,000 tokens (stated: below 512 MiB);
     - the documented shape itself measures 1,209 to 1,210 bytes per token at 300k and 600k.
   - The user decision asks for figure (2) as a formula of the limit. The formula stands; its constant is
     wrong.
   - Fix: re-derive the worst case including a single list of 2^k+1 statements.
     - Measured 1,458 bytes per token, so a constant of at least 1,460 is needed, and the default peak is
       about 2.7 GiB.
     - Restate the default figure, the low-memory recommendation (about 368,000 tokens for 512 MiB), the
       combined 3.6 / 4.35 GiB totals and the `FILE_TOKENS` doc.
     - Make the "costliest shape" fixture use the costlier variant.

3. **REQUIRED: the Contract and SECURITY.md assert a glob-edge bound that this revision does not have.**
   - The text:
     - source-index.md figure (1) counts "the glob-edge bytes (512 MiB)", states "at most 2.25 GiB, and
       1.5 GiB at the defaults", "Every term is a limit of `LoadOptions`" and "at the limits it sets a
       load cannot exhaust a machine";
     - SECURITY.md:76-78 states "at most 2.25 GiB with the glob-edge bytes".
   - The code has no glob-edge budget and no `LoadOptions` field for one. Glob scope tables are unbounded:
     - a 200 KB, 66k-token file loads at a 7.3 GiB peak, 3.7 GiB held, with the meters at 13.9 MiB;
     - web-sys ends `ERR memory` at the 8 GiB cap in the implementer's own scan.
   - Brief item (6) requires "cannot exhaust a machine" to be stated honestly at this revision. It is not.
   - Fix: either state factually that glob scope tables are outside the figures and not yet bounded, and
     qualify the "cannot exhaust" sentence, with no process wording, or bring the glob-edge budget into
     this change.
   - Findings 1 and 2 must also be fixed for the sentence to hold.

4. **ADVISORY: per-term charges undercount owned capacity and repeated names.**
   - Facts charge `len`, not `capacity`: 2^18+1 attributes per file hold about 1.9x the charged fact
     bytes.
   - Scope-table keys, `Link.name` and `Source.name` copy names without a charge: about 1.5x for long item
     names and about 2.7x for long-name re-exports on the item side.
   - The totals in these probes stayed within the sum of the limits, because the text and fact limits
     bind first. Still, "Figure (1) then holds by construction" and "the fixed charges are measured to
     cover what the index keeps" are not exact.
   - Fix: charge capacity (or `shrink_to_fit` fact lists), and charge the scope-table name keys.

5. **ADVISORY: `identity` and `read` are two separate opens.**
   - Each new file is opened, canonicalised and verified twice. That doubles the syscall cost.
   - A swap between the two calls can tie identity X to the content of Y. Both are inside the root, so
     containment holds, but the de-duplication can be inconsistent.
   - Fix: consider returning the identity from the handle that `read` uses, or document the window.

6. **ADVISORY: the registry_scan doc claims more than the cap gives.**
   - It says "so no number of `--jobs` can exhaust the machine". The cap is per child: total address space
     is jobs × `--max-memory`. The defaults are 32 × 8 GiB = 256 GiB on this 124 GiB host.
   - Fix: reword the doc.

7. **ADVISORY: one item-budget test does not check the file.**
   - `index_records_at_the_item_budget_load_and_one_byte_less_is_refused` checks the budget and the limit,
     but not `error.file`. The plan says "naming the budget and the file".

8. **ADVISORY: the 200-level DiskTree chain never overflowed the old walk on 2 MiB.**
   - On HEAD it fails on the missing budget, not on the stack. Stack safety of the shared discovery code
     is shown by the 20,000-level MemoryTree chain.
   - Fix: a 2,100-level disk chain, as in S-4, would show stack safety on disk too.

9. **ADVISORY: "source text (at most 1 GiB …)" in figure (1) is not enforced.**
   - `max_total_bytes` is caller-settable with no cap. Only the default comes from the
     `min(1 GiB, max(256 MiB, 4 × largest))` formula; it computes to 256 MiB, which is correct.
   - Fix: phrase it as "the limit set (256 MiB by default)".

10. **ADVISORY: "exactly when they name one file" assumes stable, unique inode numbers.**
    - Some FUSE or network mounts do not provide them.
    - Fix: the Contract could say so.

11. **ADVISORY: wording.**
    - The CHANGELOG sentence repeats "the tokens of a file" and has an unwrapped over-long line.
    - source-index.md has a few unwrapped long lines in the edited paragraphs.

12. **ADVISORY: `Bound::offer` uses `Vec::insert`.**
    - That is O(n) per binding for a name bound to many items, which is quadratic only for pathological
      many-glob ambiguity. Behaviour is identical to the `BTreeMap` (verified above).
    - R5c can revisit it.

## Verdict

These are confirmed:
- identity and containment (1);
- fixtures and mutations (3);
- the iterative walks and settle preserve behaviour (4, 5);
- the harness (7);
- every gate (8).

Budget honesty (2) and the Contract figures (6) are falsified:
- uncharged per-arrival copies give unbounded memory under all meters;
- the parse-peak constant is too low by about 30%;
- a glob-edge bound is stated that the code lacks.

VERDICT: BLOCKED

## Recheck

Scope: the implementer's fixes for REQUIRED 1-3 and the advisories (uncommitted tree, 34 changed or new
paths).

- The tree was re-synced to `/tmp/r3rev/new2`, with the probe, peak and dump examples copied in.
- It was built into a fresh target (`CARGO_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR` under `/tmp/r3rev/new2`).
- Every number below comes from my own runs, in release, with default `LoadOptions` unless stated.

### Gates (worktree, `ulimit -v 24 GiB`)

| Gate | Result |
|---|---|
| fmt | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test -p raf-source --locked`, debug | exit 0, 289 passed, 0 failed |
| `cargo test -p raf-source --locked --release` | exit 0, 289 passed, 0 failed |
| `RUSTDOCFLAGS=-Dwarnings cargo doc` | exit 0 |
| docs_contract + repo_gates | exit 0, 194 passed |
| workspace tests | exit 0 |
| `make cross-check` | exit 0 |
| `make msrv` | exit 0 |
| `make reuse` | exit 0 |

The new tests (`a_long_alias_path_…_charged_once`, `a_long_extern_crate_name_…_charged_once`, the
2,100-level disk chain, and the read_unseen tests in memory_tree and disk_tree_containment) appear in the
debug log as passing.

### REQUIRED 1 (uncharged per-arrival copies): RESOLVED

Same probes as before, `held / peak` against `Usage` (total / item / fact):

| Probe | Before | Now |
|---|---|---|
| `alias 10 300000` | 2,450 MiB held | 10.7 / 50.7 MiB, charged 0.3 / 6.9 / 2.4 |
| `alias 12 300000` | allocation failure under a 12 GiB cap | 27.6 / 50.7 MiB |
| `alias 16 300000` | not run (unbounded) | 368 / 401 MiB, charged 0.3 / 297 / 2.4 |
| `externcrate 12 1000000` | 3,931 MiB | 25.4 / 27.5 MiB |
| `externcrate 16 1000000` | not run (unbounded) | 366 / 399 MiB, charged 1.0 / 296 / 1.0 |

Memory now grows with the charged item bytes, not with arrivals × length. On the advisory shapes:
- `attrs 262145 8`: held = charged (1.00), before 1.9x. Capacity is now charged.
- `reexport 40 80000`: about 1.37x on the item side, before about 2.7x. Scope-table names are now charged.
- `structs 2000 100000 40` (100 KB names): now refused by `ItemBytes`, which charges the name copies.

### REQUIRED 2 (parse-peak constant): RESOLVED

`examples/peak.rs` runs `parse_source` alone; `onelistload` and `comboload` run a full `Workspace::load`,
so they include the fact extraction on the parse thread. Bytes per token at the peak:

- **One body of 2^k+1 empty statements** (`onelist`), at 2^k + 7, 8 and 9 tokens for k = 14, 16, 18, 20:
  1,457 to 1,458.
- **The same as a full load:** 1,460 at 2^20+1 statements.
- **One body at exactly 2,000,000 tokens:** 1,500 MiB.
- **The costliest arrangement I can build** (the largest 2^k+1 list grown last, below lists of 1,025):

  | Limit | Peak | Per token | Bound at 1,664 bytes per token |
  |---|---|---|---|
  | 2,000,000 tokens | 2,386.5 MiB | 1,249 | 3,174 MiB |
  | 320,000 tokens | 421 MiB | 1,378 | 508 MiB |

- **The documented `statement_lists` shape:** 1,135 per token at 2M and 1,199 at 320k.
- **Other shapes** (earlier sweep) were all below 630.

Every measurement is under `PARSE_BYTES_PER_TOKEN` = 1,664. The stated "about 3.1 GiB at the default" and
"320,000 keeps the peak below 512 MiB" hold. The implementer's 1,602 and 2,623 MiB are higher than mine, so
the constant has margin either way. The default of 2,000,000 tokens stands, as the user decided, with the
formula stated in the Contract, SECURITY.md, `budget.rs` and `options.rs`.

### REQUIRED 3 (glob-edge claim): RESOLVED

- Figure (1) now reads "Retained memory, apart from the scope tables of imports" and lists only bounded
  terms: item 512 MiB, source `max_total_bytes` (256 MiB by default, which a caller may raise), fact 256 MiB.
- source-index.md (figures and Constraints) and SECURITY.md (Bounds) now say plainly that the scope tables
  built by `use` and globs have no budget and that a glob-heavy input can exhaust a machine. They give the
  measured 166 KB to 7.3 GiB case.
- There is no glob-edge term, no "every term is a limit" claim covering it, and no process wording.
- `glob 1500 300` still holds 188 MiB under 3.4 MiB of meters, which matches the new text.

### read_unseen: single open, containment preserved: CONFIRMED

- **DiskTree.** `read_unseen` calls `open_verified` once (`contain`, then an `O_NONBLOCK|O_NOFOLLOW`
  open, then `verify_open_handle`), takes `(dev, ino)` from the verified handle, and reads the text from
  that same handle unless `seen` accepts the identity.
- **Provided `identity`.** It is `read_unseen(path, 0, |_| true)`, so it goes through the same
  containment.
- **MemoryTree.** It uses `collapse`, as `read` does.
- **Walker.** It calls `read_unseen` once per new spelling, and a known spelling makes no call.
- **Syscall check.** `strace -f -e openat` of the dump over this repository: every `.rs` path is opened
  exactly once (the old code also opened each once).
- **Outside paths.** The out-of-root symlink is still `OutsideRoot` through `identity`, which is the
  provided method on `read_unseen` (disk_tree_containment).

### Behaviour preserved: CONFIRMED

The dump (items, reaches, items by source, every resolution) was compared, old f5ede44 against the fixed
tree:
- this repository, both snapshots: identical;
- the same 161 registry crates: 161/161 identical.

The per-file storage of aliases and extern names changes no output.

### Advisories, spot-checked

- **registry_scan.** It reads MemTotal from `/proc/meminfo`, defaults jobs to min(processors, machine ÷
  max-memory), and refuses `--jobs × --max-memory` above the machine. The doc text is accurate.
  - Minor: when `--max-memory` is larger than the machine, `fit` is 0 and the default is still one job.
  - Minor: MemTotal rather than MemAvailable on a shared host.
- **Item at-limit test.** It now asserts the file (`src/a.rs`) and the line (3).
- **Disk chain.** It has 2,100 levels on a 2 MiB thread (built by rename), and it is typed.
- **Contract.** It states the raisable source limit and the unique-inode assumption.
- **CHANGELOG.** The sentence is fixed and wrapped.
- **Cosmetic, new.** `modtree.rs` `identify` has a stale duplicated first doc line ("The identity of the
  file at `path`, asked once per spelling.").
- **`Bound::offer`.** Left to R5c, as agreed.

### New findings

R-4. **REQUIRED: figure (1) is still exceeded, by inputs with no imports and no globs, because modules and
module-walk arrivals are charged only their size.**

- The source-index.md text says: "Each arrival of the module walk, each module … charge their size", and
  "The fixed charges are measured to cover what the index keeps for each item and import".
- A chain of files that each declare the next twice (`alias 16 3`, `alias 16 300000`, `externcrate 16 …`)
  holds 1.23x its charged item bytes (363 MiB held for 295 MiB charged). Other shapes stay at or below 1.0
  (unit structs 0.86, inline modules 0.82, a fan of path-attribute modules 0.66 to 1.10).
- Combining that chain with unit structs, doc-comment files and comment padding fills every budget below
  its limit: `PAD=4 probe mixed 16 28 138000 8300000`.
  - Charged: total 255.3 / 256 MiB, item 490.0 / 512 MiB, fact 254.7 / 256 MiB.
  - The load keeps 1,045.4 MiB and peaks at 1,091.5 MiB.
  - Figure (1) states "1 GiB at the defaults" for what a load keeps.
- The excess is small (2% held, 7% peak), but it falsifies a stated bound, and it scales with whatever
  share of the item budget the chain shape fills.
- Fix: give each module and each arrival a measured fixed charge, as items and imports have, covering:
  - the module's scope table and map entries;
  - the reach in its file's list;
  - the arrival record.

  Then add a fixture: a file-chain load whose held bytes, as Usage-checked, stay within the charge. Or
  restate figure (1) with the measured ratio.

R-5. **ADVISORY: one imports-only shape also exceeds 1 GiB, but the text excludes it.**

- Long-name re-exports spread over files (`reexportmulti 30 80000 28`) keep 1,029.7 MiB, with item 452.9,
  fact 192.7 and text 192.3 MiB, all under their limits.
- That is consistent with the text, because the excess is the scope tables of imports, which figure (1)
  now excludes. No action is needed beyond R-4. It is noted so that R5c's glob and import budget accounts
  for it.

VERDICT: BLOCKED

## Recheck 2

Scope: the R-4 fix (each module and each walk arrival charges its size plus a fixed 384 bytes) and the
advisories.

- The tree was re-synced to `/tmp/r3rev/new3` and built into a fresh target.
- The probe gained new shapes, and a per-load split: a clone of every file's facts gives the facts' own
  size, and the text capacity is summed. The rest of the held heap is the index side, which is set against
  `Usage.item_bytes`.

### Gates (worktree)

All exit 0:
- fmt, `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `cargo test -p raf-source --locked`, debug and release: 290 passed, 0 failed;
- rustdoc `-Dwarnings`, docs_contract + repo_gates (194 passed), workspace tests;
- `make cross-check`, `make msrv`, `make reuse`.

### R-4 shapes: chain shapes now covered

Index side held against item charged (the ratio):

| Shape | Ratio | Held / charged |
|---|---|---|
| `alias 16 3`, file chain, before 1.23 | 0.95 | 363 / 383 MiB |
| `alias 16 300000` | 0.95 | (n/a) |
| `externcrate 16 1000000` | 0.95 | (n/a) |
| `chainempty 16` (empty leaves) | 0.68 | (n/a) |
| `chainwide 14 4` (each file also holds 4 inline modules) | 0.59 | (n/a) |
| `fan2 2000 100` (one hub reached by 2,000 paths, 100 leaf files each) | 0.44 | (n/a) |
| `targets 2000 20` (one common file reached by 2,000 test targets) | 0.65 | (n/a) |
| `modfan 300 300` | 0.55 | (n/a) |
| `modfan 1000 1` | 0.86 | (n/a) |
| `inlinemods 200000` (flat) | 0.58 | (n/a) |
| `structs 262145` | 0.84 | (n/a) |
| `chaincfg 17` (cfg_attr alternatives) | refused typed by `ModulePaths` | (n/a) |

- `PAD=4 mixed 16 28 138000 8300000` is now refused by `ItemBytes`.
- With 82,000 structs it loads with 254 / 499 / 241 MiB charged and keeps 988.2 MiB (peak 1,025.8 MiB).
  That matches the new Contract text.

### R-4 not resolved: nested inline modules

Each module holds exactly one inline module `a`, nested d deep (`nestedinline n d`:
`mod bI { mod a { mod a { … } } }`, n copies):

| Shape | Index side held | Item charged | Ratio |
|---|---|---|---|
| `nestedinline 40000 5` | 540.9 MiB | 425.0 MiB | 1.27 |
| `nestedinline 2000 100` | 542.0 MiB | 382.6 MiB | 1.42 |
| `nestedinline 1700 120` | (n/a) | (n/a) | held/charged sum 1.35 overall |

The facts are exact: the facts clone equals `Usage.fact_bytes`.

**Likely cause** (not proven):
- A module's scope table is a `BTreeMap<String, Entry>` (`settle.rs:152`).
- A module whose scope holds a single name pays for a whole B-tree leaf node of 11 slots.
- `ITEM_INDEX_BYTES` (1,088) and the module charge of size + 384 were measured on scopes that hold many
  names, where nodes are full. Flat inline modules give 0.58; one-name scopes, at every level of a nested
  chain, give 1.27 to 1.42.

**Figure (1) falsified.** `mixed2 2600 100 20 8300000 7` combines:
- 2,600 copies of a 100-deep chain of inline modules, spread over files under 8 MB;
- 20 doc-comment files of 8.3 MB;
- 7 comment files of 8 MB.

Results:
- Charged: total 214.2 / 256 MiB, item 498.5 / 512 MiB, fact 234.7 / 256 MiB. Every budget is under its
  limit.
- The load keeps 1,206.5 MiB and peaks at 1,217.0 MiB, against the Contract's "1 GiB at the defaults" for
  what a load keeps.
- There are no imports or globs, so the carve-out does not apply.

R-6. **REQUIRED.** Charge a module's scope table for what it really allocates when it gets its first entry
(at least one B-tree leaf node), or store small scopes in a sorted `Vec` as `Bound` does. Then re-measure
the one-name-scope shapes:
- nested inline modules at depths 5, 100 and 120;
- a flat list of `mod bI { mod a {} }`;
- the mixed load above.

Add a fixture like `a_chain_of_files_…_charged_what_it_keeps` for nested inline modules.

### Advisories, spot-checked

- **registry_scan.** It now reads `MemAvailable`, refuses a `--max-memory` above it, and refuses `--jobs`
  × `--max-memory` above it. The doc text matches.
- **Duplicated doc line in `modtree.rs` `identify`.** Removed.
- **Not re-run this round:** the parse-peak, read_unseen and behaviour-equivalence results. The R-4 change
  touches only charges, so they still stand from the first recheck.

VERDICT: BLOCKED

## Recheck 3

Scope:
- `crates/raf-source/src/cost.rs`, which replaces the measured constants with charges derived from how
  the containers allocate;
- boxed scope entries;
- import bindings charged inside settle;
- `Workspace` equality ignoring `usage()`;
- the gates.

The tree was re-synced to `/tmp/r3rev/new4` and built into a fresh target. The probe split is the same as
in recheck 2 (facts measured by cloning, text by capacity, the rest set against `Usage.item_bytes`).

### Gates (worktree)

All exit 0:
- fmt, `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `cargo test -p raf-source --locked`, debug and release: 294 passed, 0 failed;
- rustdoc `-Dwarnings`, docs_contract + repo_gates (194), workspace tests;
- `make cross-check`, `make msrv`, `make reuse`.

### Are the cost.rs bounds true upper bounds? Checked against std and hashbrown

- **BTreeMap leaf node.** Std `B = 6`, `CAPACITY = 11`. A leaf is a parent pointer, two `u16` and eleven
  keys and values, so about 12 + 11(K+V) bytes, padded. `btree_leaf` = 16 + 11(K+V) + 8 bytes, which is at
  least that. An internal node adds 12 edges, which `btree_internal` adds.
- **BTreeMap fill.**
  - When an insert splits a full node, std's `splitpoint` leaves 5 or 6 entries on each side.
  - Bulk building (`collect`, `append`) fills nodes and then fixes the right border to at least
    `MIN_LEN = 5`.
  - Removal keeps every non-root node at `MIN_LEN = 5` or more.
  - So for n ≥ 12 the root is internal, there are at most n/5 leaves and at most 1 + n/25 internal nodes,
    and 1 + n/25 ≤ n/8 holds from n ≥ 12. For n ≤ 11 the map is one leaf, which `btree_map` charges once
    per map.
  - A map emptied by removals keeps its root leaf, which was charged when the map took its first entry.
    The bound holds for every n, including lone entries and n < 12.
- **Vec.**
  - `RawVec` grows to max(2 × capacity, required), and to at least 4 for elements of 1 KiB or less. After
    any growth the capacity is at most 2 × length, or 4 before the list holds four.
  - `vec_first + n × vec_slot` covers that.
  - `Bound`'s `reserve_exact(1)` then grows to 4, which is also covered.
- **hashbrown.**
  - The bucket count is a power of two of at least 8/7 of the capacity, and at least 4 or 8 for small
    tables. Control bytes are the buckets plus 16.
  - A resize doubles the table when it is full. That gives at most about 2.29 buckets per entry, and
    `hash_entry` charges 3.
  - `hash_map` charges 8 buckets plus a group, which covers the small-table minimum.
- **Shared strings: ADVISORY a.** `shared_text(len) = len + 16`, but an `Arc<str>` allocation is
  `ArcInner` laid out and padded to its 8-byte alignment, so up to 7 bytes more per string. That is not a
  strict upper bound per string. The surplus elsewhere in each charge absorbs it in every probe; rounding
  `len + 16` up to 8 would make the doc's "upper bound" literally true.
- **Mutation.** Each of these makes the matching cost.rs unit test fail:
  - dropping the internal-node term from `btree_entry`;
  - halving `vec_slot`;
  - charging 2 buckets instead of 3 in `hash_entry`.
- **ADVISORY b.** The BTreeMap test checks the formula against the counted node model, not against a
  real map's allocations. The probes below are the empirical check.

### Trying to beat the charges

Index side held against item charged, in release, with default options:

| Shape | Ratio |
|---|---|
| One-name scopes, `mod bI { X }` × 20,000, X = `mod`, `fn`, `struct`, `use`, `pub use … as`, `macro_rules`, `const`, `trait`, `enum`, `type`, `extern crate` | 0.56-0.64 |
| Nested one-name scopes, 2,000 × 100 deep (`nestedinline`, before 1.42) | 0.62 |
| Nested one-name scopes, 1,700 × 120 deep with a `fn` or `use` leaf | 0.62 |
| Nested one-name scopes, 40,000 × 5 deep | refused typed by `ItemBytes` |
| 50,000 tiny files, empty | 0.30 |
| 50,000 tiny files, one struct each | 0.44 |
| 5,000 bin targets with tiny roots | 0.46 |
| 2,000 test targets sharing a file | 0.42 |
| 20,000 trait impls on a foreign type (distinct trait keys) | 0.68 |
| 2,000 impls × 30 members | 0.54 |
| 30,000 local inherent + trait impls | 0.62 |
| 50,000 spellings of one file (`zI/../x.rs`) | 0.36 |
| File chain (`alias 16 3`) | 0.45 |
| Flat inline modules | 0.45 |
| Unit structs | 0.67 |
| Long names: the same 4 KB name defined in 2,000 modules | 0.85 |
| Long names: imported into 2,000 modules | 0.85 |
| Long names: in a file reached 2,000 times | 0.85 |
| Long-name re-exports | 0.82-0.92 |
| Long-name re-exports over 20 files (item 507.8 / 512 MiB) | 0.91 |

- The worst is 0.92. No shape beats the charge.
- My recheck-2 counterexample `mixed2 2600 100 20 8300000 7` (1,206 MiB) is now refused by `ItemBytes`.
  `fan2 2000 100` and `PAD=4 mixed …` are refused too.
- One-name scopes built by globs (`mod bI { use super::*; }` × 20,000) exhaust memory: an allocation
  failure under a 12 GiB cap. That is the documented carve-out (scope tables of globs are unbudgeted, and
  the Contract and SECURITY.md say so).

### Other checks

- **Workspace equality.** `PartialEq` compares exactly the four fields the old derived impl compared
  (packages, targets, files, index) and leaves out only the new `usage`. The equality tests that compare
  `DiskTree` and `MemoryTree` loads pass. It is harmless.
- **Behaviour preserved.** The dump compared against old f5ede44 is identical on both repository
  snapshots and on 161/161 registry crates.
- **Parse peak unchanged.** One list of 2^20+1 statements: 1,458 bytes per token. The costliest
  arrangement: 2,383 MiB at 2M tokens and 421 MiB at 320k. Both are under 1,664 bytes per token.
- **Fixture.** `nested_inline_modules_whose_scopes_hold_one_name_are_charged_what_they_keep` exists. As the
  implementer notes, the falsifying checks for the charge model are the cost.rs unit tests, mutated above,
  and these probes.

### Findings this round

- ADVISORY a: round `shared_text` up to the 8-byte alignment of the `Arc` allocation.
- ADVISORY b: optionally, check `btree_*` against a real map's allocations in the probe harness.
- Earlier advisories still open by agreement: `Bound::offer` is left to R5c.

VERDICT: PASS
