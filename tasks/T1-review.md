# T1 review: the work budget in the index (commit 1531b15)

VERDICT: PASS

Task 1 meets its plan. The findings below do not block it. F1 needs a
disposition before Task 2 claims R1, and F2 to F4 are small corrections.

Method: I used scratch clones of 1531b15 under `/tmp/t1review.vFyl3E/`
(`gate`, `probe`, `mut`). Every measurement is a count: probe counters,
`Usage`, the `work.rs` tally and `perf stat -e instructions:u`. Nothing was
timed. I made no edits to the worktree apart from this file.

## (1) Charge sites, and the uncharged loops on the three shapes

Each charge site holds `&Meter` and returns through `?` before it does the
work it counts:

* `make_view`: `glob_scope.rs:1567` (the reads) and `:1582` (the names)
* `export_ids`: `:1372`
* `read_glob`: `:1784`
* `pass_on`: `:1875`
* `holding` through the `record_drops` closure: `:999`

No caller swallows a `LoadError`. The `.ok()` and `Err(_)` arms in `settle.rs`
match `Fail`, not `LoadError`. After a refusal in the search, `supplied`
returns early with `supply.refused` set, so no further charge is made, and the
error is returned at `:1004`.

I added counters (probe only) at every uncharged loop over glob reads, names
or items. The counts below are at n=64:

| uncharged loop | hub (work 540,928) | prelude (work 31,264) | fan-in (work 16,768) |
|---|---|---|---|
| `read_lists` items cloned (`:743-751`, under `export_ids`) | 266,240 (~n^3) | 8,128 | 256 |
| `read_glob` list clone (`:1778`) | 8,192 | 12,416 | 0 |
| `arrive` view loop and `glob_held` items (`:1934`) | 16,128 | 16,320 | 0 |
| `gather` `glob_held` items | 256 | 8,256 | 256 |
| `add_to_view` ids | 16,512 | 640 | 256 |

**The worst uncharged loop on these shapes:** the per-item clone in
`read_lists` (`:750`), reached from `export_ids` through
`split_globs_delivered` and `delivered`. On the hub it grows with n^3, at
about 0.49 times the work count. The charge in `export_ids` is per read, and
each read holds about one item under a name, so this loop grows no faster than
what is charged. Making every sibling's name ambiguous does not change that
(hub_amb at n=64: work 541,696, items cloned 266,496). On the three shapes, no
uncharged loop grows faster than the charge.

**F1 (major, not blocking): an uncharged quadratic loop outside the three
shapes.**

* **Where:** `globbed()` (`glob_scope.rs:682`) calls `delivered` and then
  `read_lists` (`:745`), and `globbed_any` (`:1231`) reads every glob read of
  a module. Neither is charged. Both run on the settle lookups:
  `settle.rs:522`, `:615`, `:1644`, `:1671`, `:1868` and `:2455`.
  `lookup_seen` has no `&Meter`, and it also runs after the load.
* **Shape used (probe):**
  `pub mod t { use crate::g{i}::*; ... use self::X as Y{k}; ... }`, with
  n globs and n imports. Each `g{i}` is `pub use crate::d::*`, and `d`
  defines `X`.
* **Counts:**

  | n | work | glob reads visited | items cloned | instructions:u |
  |---|---|---|---|---|
  | 512 | 2,050 | 6,554,112 | 3,932,160 | 6.76e9 |
  | 1024 | 4,098 | | | 2.69e10 |
  | 2048 | 8,194 | | | 1.08e11 |

  The work count is 4n+2. The reads visited are about 25n² and multiply by 4
  for each doubling.
* **Extrapolation:** the item-byte budget refuses this shape at about
  n=1.1e5 (item bytes are about 4.9 KB per n). Near that n the load would run
  about 2.6e14 user instructions with `Usage::work` of about 4e5. That is
  about 200 times the worst hub that the default work limit admits (see (3)).
* **What the doc says:** it discloses the queue in general: "the work budget
  counts the steps of the glob tables, not those of the queue and the graph"
  (`source-index.md:810`, `:1714`). The plan excludes `Lookups` on purpose. So
  this is not a Task 1 defect, but it means the registry scan without a
  timeout can still spin on such a crate.
* **Disposition needed (no deferrals):** either
  * charge the reads that `globbed`/`globbed_any` make during the load (the
    lookup needs an optional meter), or
  * have Task 2's honest-limit text and the source-index Constraints name this
    shape and its figure, and move the fix to a tracked change.

## (2) What `Meter::work` and `Usage::work` do

* `add` (`meter.rs:68-72`) uses `saturating_add` and sets the total before the
  check. So a refusal comes at the charge that passes the limit, and the count
  stays at the limit plus that charge, which the tests assert.
* The work is never released: no release path exists.
* Each load starts from zero, because `Workspace::load_with` builds a new
  `Meter` (`workspace.rs:159`).
* `exported_items_with` builds its own meter (`workspace.rs:559`) and makes no
  work charge.
* `Usage::work` is copied in `Meter::usage`, and `Workspace::usage` returns it
  (`workspace.rs:226`).
* Count saturation cannot be reached: a single charge is at most the number of
  reads or names, and once the total passes the limit the load ends. A caller
  that sets `max_work = usize::MAX` gets no limit, which is coherent.
* On a refusal the meter keeps the count that passed the limit, and the
  partial index is dropped with the `Err`, so no inconsistent state is visible.
* A `max_work = 0` load without globs loads with `work = 0`. This is tested in
  `load_budgets.rs`. A load whose glob targets are empty charges 0 units and
  is not refused.

## (3) The default limit of 2^31

* **Hub at the limit:** at 2n³ + 4n² + 4n steps, n = 1023 is accepted with
  2,145,388,542 steps, and n = 1024 is refused.
* **Memory at the limit:** the glob-byte peak is 0.97 MB at n=64, 3.45 MB at
  128 and 12.95 MB at 256, growing with about n². At n=1023 that is about
  207 MB, under 512 MiB. So no memory budget stops the hub first.
* **Instructions per step:** `perf` counts about 570 user instructions per
  step (634, 587 and 571 at n = 64, 128 and 256). The largest admitted hub
  therefore runs about 1.2e12 user instructions. The 2^31 limit is reachable
  by a hostile input well before memory is a problem.
* **Real crates (in-process sample, default options):**

  | crate | work |
  |---|---|
  | rustix-1.1.5 | 3,776 |
  | windows-core-0.62.2 | 2,138 |
  | the repository | 1,408 |
  | winapi-0.3.9 | 982 |
  | tokio-1.53.2 | 778 |
  | nix-0.30.1 | 659 |
  | bitflags-2.13.2 | 454 |
  | regex-syntax-0.8.11 | 377 |
  | clap_builder-4.6.7 | 164 |
  | windows-sys | 4 |
  | libc | 2 |
  | syn, serde | 0 |

  2^31 is about 5.7e5 times the highest figure in this sample.
* **Proposal (provisional):** 2^24 (16,777,216) keeps more than 4,000 times
  the sample's highest figure, caps the hub at n ≈ 203 and cuts the instruction
  ceiling to about 9.6e9. Thirteen crates are not a calibration. Task 4's full
  registry run must set the number. I do not ask for a change in Task 1.
* **F2 (minor):** the comment on `WORK` (`budget.rs:128-131`) says "below the
  largest `usize` of a 32-bit one". On i686 the crate does not compile, because
  of `glob_scope.rs:113`: `const COMPANION: usize = 1 << 40` (overflow). This
  predates the commit; it is also present on main. The statement is true but
  cannot be exercised, and `const { assert!(WORK < usize::MAX) }` is trivially
  true on 64-bit. All three cross-check targets are 64-bit.

## (4) Mutants (mine) and repeated runs

Each mutant was run in the `mut` clone and restored afterwards.

| mutant | result |
|---|---|
| M1: drop the `export_ids` charge (`:1372`) | killed: `the_glob_table_work_of_each_shape_is_the_count_of_its_steps` and `doubling_a_shape_...` fail |
| M2: drop the `scopes[target].len()` term of the `make_view` names (`:1577`) | killed: both `work_budget` tests and `load_budgets::glob_table_work_at_the_work_budget_loads_and_one_step_less_is_refused` fail |
| M3: drop `meter.work` from the `record_drops` closure (`:999`) | killed: `glob_scope::tests::the_holders_of_an_item_apart_from_each_reader_of_a_chain_...` fails on its `work >= steps` assertion |
| M4: drop the per-name charge in `read_glob` (`:1784`) | killed: both `work_budget` tests fail |

The new and changed tests passed five times out of five:

* 12 in the lib: `work_budget`, `meter::tests`, `budget::tests`,
  `glob_scope::tests`
* 3 in `load_budgets`

The probe also confirms the documented counts on the chain, the fan-in, the
prelude and the hub at n = 16, 32 and 64, and on the hub at 128 (4,260,352).

## (5) Docs

Six statements in `source-index.md` checked against the code and the probe:

1. **Chain is 2n.** True.
2. **Fan-in is 4n²+6n.** True: 1120 at 16 and 16768 at 64.
3. **Prelude is n(15n+17)/2.** True: 2056 and 31264.
4. **Hub is 2n³+4n²+4n.** True up to n=128.
5. **"A load without glob imports makes none".** True.
6. **The chain-of-two fixture is ten steps, refused at nine on line 3, and
   loads at 0 with no glob.** True.

**F3 (minor):** `source-index.md:767` says a refusal names "the glob import it
was made for". The steps of the search are charged to the first glob import of
the load (`glob_scope.rs:963-967`), not to the read that the search is for. The
item-byte charge there already behaved this way. Reword, or charge
`found[..].0.glob`.

**F4 (minor):** the `CHANGELOG.md:10-22` list of steps leaves out "each name a
new reader is given" (`read_glob`, `:1784`). The commit message, the module doc
and the capability doc include it.

Assessments: only the digests changed (verdict accepted), and the sections they
cover read true apart from F3. I found no issue or pull request numbers and no
process wording in the diff.

The test-only log (`meter.rs` `charges`) and its `record` call are both
`#[cfg(test)]`. The log is absent from non-test builds and from the integration
test binaries. It is recorded before `add` and does not change the result. It
is thread-local, and the settle runs on the calling thread, so the sum of the
log equals `Usage::work`.

## (6) Gates

These were run on the `gate` clone at 1531b15:

* `cargo test -p raf-source --locked`: exit 0. Every binary is ok, and the lib
  passes 93.
* `cargo clippy -p raf-source --locked --all-targets -- -D warnings`: exit 0.
* `make cross-check`: not run. The targets x86_64-apple-darwin,
  x86_64-unknown-freebsd and x86_64-pc-windows-gnu are not installed in the
  sysroot (and `rustup target list` fails for the floating "nightly" alias).
  Hosted CI must cover it.
* i686 check (outside the plan): it fails, for the reason in F2, which
  predates this commit.
