# Validation: Task 1 (the work budget in raf-source)

No timing runs. Every figure below is a count (`Usage::work`, the test-only charge log,
the `SupplySteps` tally).

## Charge sites (glob_scope.rs) and evidence that each holds the meter

| Site | Units charged | Holds `&Meter`, returns `Result<_, LoadError>` |
|---|---|---|
| `make_view` (view gather, beside `charge_work`) | one per glob read listed + one per name gathered (own scope names + names of each source view) | `fn make_view(&mut self, .., meter: &Meter) -> Result<usize, LoadError>` |
| `gather` (re-read of a name with hidden items or companions) | through `export_ids` | signature changed to take `site, meter` and return `Result<(Exported, bool), LoadError>`; its one caller is `make_view` |
| `pass_on` | one per active reader a change is passed on to, charged to the reader's glob before `arrive` | `fn pass_on(.., meter: &Meter) -> Result<(), LoadError>` |
| `read_glob` (loop that passes each name of the view to a new active reader) | one per name/name-space passed on | `pub(crate) fn read_glob(.., meter: &Meter) -> Result<(), LoadError>` |
| `arrive` -> `export_ids` (re-read of a module's table for a name) | one per glob read of the module that `delivered`/`read_lists` reads again | `export_ids` now takes `site, meter` and returns `Result`; every caller (`arrive` x3, `refresh`, `gather`) holds `&Meter` |
| `holding`'s `step` (supply search, dominators) | one per step, through the `charge` closure built in `record_drops`, which now charges items and work | `record_drops(&mut self, meter: &Meter) -> Result<usize, LoadError>`; the refusal is carried out through `Supply::refused` |

Design note: the charge for `arrive` sits in `export_ids`, not at the top of `arrive`.
The superlinear cost of the three shapes is `export_ids` re-reading every glob read of
the module each time a companion or a hidden item arrives (the module doc of
glob_scope.rs). Charging there also covers `refresh` and the re-reads inside the view
gather, which make the same re-read. The `pass_on` charge (one per reader) is what makes
a plain chain's linear work visible.

## Counts (Usage::work, default limit)

| Shape (n) | Count | Closed form, checked at n = 8, 16, 32 |
|---|---|---|
| plain chain | 50, 100, 200, 400 at n = 25..200 | 2n |
| private fan-in (`t` declared before `u`) | 2650 .. 161200 | 4n^2 + 6n |
| prelude | 4900 .. 301700 | n(15n + 17)/2 |
| hub (every other glob private, n even) | 260200 (50), 2040400 (100), 16160800 (200) | 2n^3 + 4n^2 + 4n |

Ratio per doubling, 25..200: hub 7.4-7.9, prelude 3.9-4.0, fan-in 3.9-4.0, chain 2.0.

## Mutants (scratch clone /tmp/raf-mutants-KMP4Sg/repo, `git clone --no-hardlinks` + patch; worktree never mutated)

| Mutant | Killed by | Observed failure |
|---|---|---|
| work charge dropped from `pass_on` alone | `work_budget::the_glob_table_work_of_each_shape_is_the_count_of_its_steps`, `work_budget::doubling_a_shape_multiplies_its_work_as_fast_as_the_shape_grows` | exact: `chain of 8` left 4, right 16; doubling: `chain: 4 then 4`. The hub stays cubic (2n^3 term from `export_ids`), so its doubling check does not fire on this mutant; the chain check (strengthened to "grows and at most doubles") does |
| re-read charge dropped from `export_ids` (the `arrive` site) | exact-count test; doubling test | exact: `fan-in of 8` left 48, right 304; doubling: `hub: 4224 then 16640` (ratio 3.94, under 4) |
| `>=` instead of `>` in `Meter::work`'s limit check | `meter::tests::work_up_to_the_limit_is_accepted_and_the_charge_that_passes_it_is_refused`, `work_budget::a_settle_heavy_shape_is_refused_at_the_charge_that_passes_a_small_limit`, `load_budgets::glob_table_work_at_the_work_budget_loads_and_one_step_less_is_refused` | `at the limit: ... Budget { budget: Work, limit: 10 }`; the refusal's charge log is one charge short of `all[..=place]`; the load at `max_work == spent` is refused |
| work count not fresh per load (thread-local counter shared by every meter) | `meter::tests::work_is_never_released_and_each_meter_counts_from_zero`, the three `work_budget` tests, `load_budgets::glob_table_work_at_the_work_budget_loads_and_one_step_less_is_refused` | second meter reads 80, not 0; `fan-in of 8` left 320, right 304; the refused load's log is `[0]` |
| work charge dropped from the supply search (`holding`'s step) | `glob_scope::tests::the_holders_of_an_item_apart_from_each_reader_of_a_chain_of_globs_are_found_in_steps_linear_in_the_chain` (after adding `work >= search steps` to `counted`) | `352 steps of work for 1388 of the search`. Before that assertion was added, the mutant survived |

## Default limit (provisional, for Task 4 calibration)

`Budget::Work.limit()` = 2^31 = 2147483648 on every target. It is below `usize::MAX` on
32-bit targets, so a saturating count can still pass it there; 2^32 would equal-or-pass
`usize::MAX` on 32-bit and the budget would never refuse. This is a placeholder. Task 4
must calibrate it from the registry scan and the repository. Concern: by the closed form,
the hub passes 2^31 at n = 1024 siblings (2n^3 alone is 2^31 there; n = 1023 gives about
2.1456e9, just under). The source-index doc quotes a hub "at 1,000 globs" as a measured
shape (n siblings make 2n globs, so that may be n = 500, about 2.5e8 steps). Either way, a
hub of about a thousand siblings sits at the limit. The registry figures decide whether that is acceptable.

## Task 2: the example without a clock

Acceptance (a), `grep -rnE 'std::time|Instant|Duration|sleep|recv_timeout|SystemTime' crates/raf-source/examples`:
baseline from `git show HEAD:...registry_scan.rs | grep -nE ...` (parent commit 1531b15): lines 64, 72, 75, 116, 161, 492, 561, 562, 572 (nine hits). Now: no output, exit 1.

Unit tests (`cargo test -p raf-source --locked --example registry_scan`): 4 passed (flag parser with 0 refused and 1 accepted, child argument parse, OK line with `work=`, tally of `work=` and `# err Budget`).

Acceptance (b), scratch registry /tmp/regscan.3BgtSK/reg with `tiny-0.1.0` (no globs) and `hub-0.1.0` (40 siblings, hub of 40 `pub use super::sibN::*;`):
- `--max-work 1000`: `hub@0.1.0 | ERR Budget :: src/hub.rs:4: exceeds the budget for glob table work steps (1000)`; `tiny@0.1.0 | OK ... work=0`; `# err Budget=1`; `# largest work=0`.
- no flag: both OK; hub `work=11400`; `# largest work=11400 (hub@0.1.0)`.
- `--max-work 0`: `registry_scan: --max-work must be at least 1`, exit 1.
- `--max-work 1`: tiny `OK ... work=0`; hub refused (1).
