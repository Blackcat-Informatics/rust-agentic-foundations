# raf-source: replacing the wall-clock tests with counts

(Design produced read-only by the planning agent; saved verbatim here. Dispositions of its open decisions are in plan.md Revision 3.)

## Summary

- **Scope.** 14 sites use the clock: 11 timing assertions and 3 hang watchdogs. The watchdogs wrap 31 tests, and about 25 of the affected tests are bound by `docs/capabilities/source-index.md`.
- **No public API change is needed.** Three mechanisms, and two crate rules, decide the design:
  - A **counting `Tree`** in `tests/support`, with a ceiling that fails the test instead of letting it hang. `Tree` is already public.
  - **Test-only work counters** in a new private `src/work.rs`. They compile to nothing outside `cfg(test)`. The repo already does this with the `DiskTree` hooks and `Workspace::index()`, and the settle tests already count rounds the same way.
  - **Counts the crate already reports:** `Usage`, budget refusals and their limits, and item-byte charges under `LoadOptions`.
- **Two rules shape the choice.** `unsafe_code = "deny"` rules out a counting allocator of our own. `cfg(test)` code is invisible to integration tests, so every test that needs an internal counter moves into a `src/**/tests` module.
- **Watchdogs are dropped.** A regression then hangs the suite, as the owner accepted.
- **One test needs a new probe.** The outside-FIFO swap test observes "never opened" with a 1 s timeout. It gets an inotify `IN_OPEN` probe through `rustix::fs::inotify`, which is already a Linux dev-dependency.
- **Every count assertion also checks a lower bound or an exact value.** Otherwise a counter that recorded nothing would pass vacuously.

## 1. Inventory

| # | Site (file:line) | Test | What it guards | Bound by source-index? |
|---|---|---|---|---|
| T1 | `src/facts.rs:1732` (clock at 1746–1753) | `facts::tests::a_large_attribute_on_a_verbatim_use_group_is_read_once_for_all_its_members` | The group's attributes are read once, not once per member (it used to be quadratic: 6000×6000). Ten seconds in debug, 0.5 s in release. | No. Verification describes it ("loads within 500 ms … 10 s"). |
| T2 | `tests/exported_items.rs:884` (890–903) | `a_module_of_three_thousand_pub_globs_is_listed_in_time_linear_in_its_names` | The globs that bring in a name are found by the name (`Routes::by_name` / `every`), not by scanning every glob for each name. | Yes (binding line 315; Verification ~2383) |
| T3 | `tests/exported_items.rs:930` (949–970) | `…_that_each_carry_a_re_export_is_listed_in_linear_time` | The reach is walked once per reader, not once per carried import. | Yes (317) |
| T4 | `tests/exported_items.rs:1051` helper `list_within` (1057–1061); test at :1065 (1072–1086, plus a release-only 10,000 case) | `readers_of_three_and_ten_thousand_carried_names_beside_sibling_shadows_list_in_bounded_time_and_bytes` | No re-walk of the reach for each shadowed name. The bytes half ("32 MiB beyond the load") is already a count. | Yes (320) |
| T5 | `tests/manifest_caps.rs:11`, helper `fast` at :57–63, used at :88, :96, :104, :113, :120 | `exclude_at_the_cap…`, `members_at_the_cap…`, `the_duplicate_pattern_probe_is_refused_at_once`, `duplicate_patterns_within_the_caps_load_at_once`, `distinct_patterns_at_both_caps_load_at_once` | The caps are refused before any pattern is expanded. Each distinct pattern is compiled once. The remaining cost is directories times wildcard excludes. | Yes, all five (215–219) |
| T6 | `tests/prescan_budgets.rs:577` (580–582) | `the_eighty_kilobyte_box_pattern_is_refused_in_under_a_second` | The pre-scan refuses the input before syn parses it. | Yes (147) |
| T7 | `tests/resolve_paths.rs:345` (clock at 7, 349–361) | `a_path_of_fifty_thousand_segments_resolves_in_linear_time` | Each segment is read once, and prefixes grow one segment at a time. | Yes (210) |
| T8 | `tests/trait_keys.rs:732` (740–747) | `matching_a_pattern_takes_time_in_proportion_to_pattern_and_name` | The glob matcher costs O(pattern × name), with no exponential backtracking. | No (the Contract at ~1021 claims it) |
| T9 | `tests/module_tree.rs:713` (714, 733–739; 30 s, release only) | `a_chain_of_files_that_each_declare_the_next_twice_is_refused_quickly_beside_a_small_diamond` | The `ModulePaths` budget (2^18) bounds a walk of 2^31 paths. | Yes (61) |
| T10 | `tests/module_tree.rs:858` (859, 872–876; 10 s) | `inline_modules_with_path_alternatives_are_bounded_by_the_module_path_budget` | Inline bodies under `cfg_attr` alternatives are counted by `ModulePaths`. | No |
| T11 | `tests/symbol_index.rs:993` (own thread with `recv_timeout(20 s)` at 1002–1010) | `items_on_one_long_line_of_wide_characters_get_exact_byte_offsets_in_linear_time` | A long line is indexed once (`Positions` marks), so column-to-byte conversion is linear. | Yes (266) |
| T12 | `tests/support/mod.rs:23,26` (`WATCHDOG` = 10 s), :111–120 `within_watchdog`; `Scratch::fifo` doc at :86–87 | Hang watchdog. Users are listed under T12a and T12b. | A backstop if a disk call blocks on the FIFO sentinel or loops on a symlink. | (see below) |
| T12a | `tests/disk_tree_containment.rs:14` import, :32 helper `read_within_watchdog` | Tests at :37 (bound 43), :59, :77 (bound 44), :150 (bound 46), :202, :217, :321 (bound 265) | Same backstop | Partly |
| T12b | `tests/workspace_containment.rs:20` import, :35 helper `load`, :153, :167, :177, :190 direct, :296 `discover_with` | Tests at :43 (bound 47), :66, :94 (bound 48), :119, :135, :161, :194, :210, :222, :267, :300 (bound 49), :462 (bound 262), :483 (bound 199) | A backstop around disk loads (FIFO sentinel, symlink loops) | Partly |
| T13 | `src/tree/disk/tests.rs:59–68` (local `within_watchdog`, 10 s) | Uses at :374 (test :368, bound 181), :402 (:395, bound 185), :426 (:420, bound 182), :439 (:433, bound 184), :476 (:465, bound 186), :633 (:607, bound 188), :670 (:646, bound 189), :721 helper `strategy::read` (→ :725, bound 196; :1060), :1085 (:1078) | A backstop around swap and strategy calls | Mostly |
| T14 | `src/tree/disk/tests.rs:515–535` `wait_until_blocked_in_open` (`Instant` deadline at 521/529, `sleep(5 ms)` at 532), :547 `outside_fifo_is_never_opened` (:565 watchdog, :566 `recv_timeout(1 s)`) | `swap::an_outside_fifo_reached_by_a_swap_before_the_open_is_never_opened` (:540) | The outside FIFO is never opened. The observation itself depends on time: a blocked writer's release is checked within 1 s. | Yes (187) |

**Out of scope, flagged for the owner:** `examples/registry_scan.rs:64,72,75,161,556–566` has a per-crate timeout and polls with `sleep`. It is an example binary, not a test, and is not bound.

## 2. Counters

### Already public
- `Workspace::usage() -> Usage` with `total_bytes`, `item_bytes`, `fact_bytes`, `glob_bytes`, `max_file_tokens`, `max_recursion` and `max_directory_depth`.
- `LoadOptions.max_*` fields, which turn any charged quantity into a pass/fail bound.
- `LoadErrorKind::Budget { budget, limit }`. A refusal at `limit` proves the counter reached `limit + 1` and stopped.
- `ExportError::CarryBudget`.
- `exported_items_with` already charges walk work (`WALK_STEP_BYTES` per glob-reach step, dominator work, and carry-search steps) to item bytes. This catches re-walk regressions (T3, T4) but not uncharged lookups (T2).
- The `Tree` trait (`read`, `read_unseen`, `identity`, `list`, `entry`). A counting wrapper can record every operation.

### Crate-private today
- `prescan::Scan { tokens, recursion }`, surfaced through `Usage`.
- `Walker.arrivals` (`modtree.rs:315`), which is private.
- `members::WalkMeter.left`, which is private.
- `FactAllowance` / `FactMeter`, surfaced through `Usage.fact_bytes`.
- `settle` rounds and waves, already read by unit tests (`the_imports_left_are_decided_in_rounds_that_do_not_grow_with_them`).
- `Meter` and `Workspace::index()` under `cfg(test)`.
- `Walks.reads` / `walk_noting(reads)` in `resolve.rs`.
- The glob matcher has no step count today.

### New, test-only: `src/work.rs`
A `Work` enum with these thread-local tallies:

| Counter | Where it ticks |
|---|---|
| `Attributes` | `facts::summarize`, once per attribute |
| `Positions` | `Positions::byte`, once per call |
| `ColumnSteps` | Characters walked in `Positions::byte`: the line index built once, plus `nth(skip)`, plus the `chars().count()` fallback |
| `PatternSteps` | Each iteration of `glob::matches_segment` |
| `ExcludeTests` | Each wildcard pattern tried in `members::Excludes::matches` |
| `SegmentReads` | Each segment pushed onto a prefix in `resolve_in` and `through_unresolved`, and each segment iteration in `walk_noting` |
| `Lookups` | `settle::lookup_seen` |
| `ViewReads` | `glob_scope::view_holds`, `view_ids`, and each name yielded by `view_names` and `glob_reads` |
| `WalkSteps` | Each place `exports.rs` charges `WALK_STEP_BYTES`, plus dominator work units |

- **API:** `add(Work, n)` under `cfg(test)`, and a no-op `#[inline]` shim otherwise. Tests use `measure(ceiling, || …) -> (T, Tally)`, which resets the tallies, runs the closure, and panics the moment any tally passes `ceiling`. A quadratic mutant therefore fails fast rather than running to completion.
- **Placement:** counters go in shared accessors (`glob_scope` views, `lookup_seen`, `Positions::byte`, `matches_segment`) rather than in their callers, so a caller-side regression can't avoid them.
- **Threads:** the tallies are thread-local. `exported_items`, `resolve`, `members()` and the module walk run on the calling thread. `extract` (facts) runs on the parse thread, so the facts tests call `extract` directly.
- **Public additions:** none are required. If the owner wants integration-level evidence for T9 and T10, the one natural addition is `Usage::module_paths: usize`: "module walks counted against `Budget::ModulePaths`". It would need Contract text and an extension of the bound `load_budgets::usage_reports_what_a_load_spent`. I recommend not adding it.

### New, tests only: `tests/support/counting.rs`
- `Counting<T: Tree>` records counts per operation (`reads`, `opens` = `read_unseen` calls, `opened_read` = `Opened::Read`, `identities`, `lists`, `entries`) and per path.
- It takes an operation ceiling and panics with the operation and path once the ceiling is passed. Tree calls run on the caller's thread, so the panic fails `Workspace::load` and the test.
- It overrides `identity` and forwards to the inner tree, because `DiskTree` has its own `identity`.
- It replaces the two ad hoc copies: `Recording` (`tests/module_tree.rs:494`) and `CountingReads` (`tests/workspace_containment.rs:350`).

## 3. Conversions

Each entry gives the guarantee kept, the counted assertion, and the mutant it must fail on.

- **T1 (facts, unit test).** Call `extract(&syn::parse_file(&text)?, &text, FactAllowance{spent: 0, limit: usize::MAX})` inside `measure` with a ceiling of 2, and keep the per-member assertions on the returned `FileFacts`.
  - Guarantee: the attributes of a group are read once for all its members.
  - Assert: `Attributes == 1` exactly.
  - Mutant: move `use_role(&group.attrs, …)` into the member loop (`facts.rs:1106`). The count becomes 6000 and the ceiling trips.
- **T11 (symbol_index) plus a new unit test in facts.rs.** The integration test drops its thread and `recv_timeout`, keeps the exact-offset assertions, and is renamed `items_on_one_long_line_of_wide_characters_get_exact_byte_offsets`. The new unit test `a_long_line_of_wide_characters_is_indexed_once_for_all_its_positions` runs `extract` directly on the same 20,000-item line.
  - Guarantee: column-to-byte conversion is linear in the line, however many items it holds.
  - Assert: `Positions > 0` and `ColumnSteps ≤ chars(line) + Positions × (MARK_EVERY + 1)`. The same expression is the ceiling.
  - Mutants: (a) drop the long-line branch so every lookup walks from the line start (Σ columns ≈ 10^10); (b) build `Positions::new` per item (re-indexes the line each time). Both trip the ceiling at once.
- **T8 (glob.rs unit test, plus trait_keys).** The new unit test is `glob::tests::a_segment_is_matched_in_at_most_pattern_times_name_steps`. Parse `crates/{"*a"×24}b`, match segment 1 against `"a"×40`, and also run k = 8, 16, 24.
  - Guarantee: matching costs at most (pieces + 1) × (chars + 2) steps. Derivation: `star_t` only grows, so backtracks ≤ t, and each run between backtracks makes ≤ p matches.
  - Assert: the result is `false`, `PatternSteps > 0`, and `PatternSteps ≤ (p+1)(t+2)`. The bound is also the ceiling.
  - Mutant: recursive backtracking over every split of every `*`. The ceiling trips.
  - `trait_keys:732` keeps its end-to-end check (one package) with no clock, renamed `a_star_heavy_member_pattern_that_matches_no_directory_leaves_the_member_alone`.
- **T5 (manifest_caps, integration, counting tree).** `load()` takes a `Counting<MemoryTree>`, and each test compares counts against a reference load.
  - `exclude_at_the_cap…`: tree ops at the cap equal the ops of the same manifest with no excludes. Over the cap, no `entry` or `list` call is made outside manifests.
  - `members_at_the_cap…`: at the cap, `lists == 0` and `entries` equals the single-member reference plus 4095 (one `entry` per glob's literal first segment). Over the cap, no expansion ops.
  - Rename `the_duplicate_pattern_probe_is_refused_at_once` → `…_is_refused_before_any_pattern_is_expanded`: the refusal is unchanged and no expansion op is made. Mutant: move the cap check after `members()` expansion.
  - Rename `duplicate_patterns_within_the_caps_load_at_once` → `…_load_with_the_tree_calls_of_one_of_each`: ops equal those of `members=["p"], exclude=["q*"]`. Mutant: drop `seen.insert` dedup in `members::patterns`, which makes 4096 expansions.
  - Rename `distinct_patterns_at_both_caps_load_at_once` → `…_load_with_no_tree_calls_for_the_excludes`.
  - **Plus new unit tests in `members.rs`** that call `members()` directly:
    - Distinct patterns: assert `ExcludeTests ≤ (members + found) × wildcards`, the cost the caps already bound, and `PatternSteps ≤ (members + found) × 21`. Here 21 = (2 + 1)(5 + 2), for the single one-segment wildcard `m*` against names of at most 5 characters. Mutant: remove the `segments.len() <= parts.len()` short-circuit in `matches_prefix`, which adds about 16.8M steps.
    - Duplicate patterns: assert `ExcludeTests == 2`. Mutant: no exclude dedup, which gives 8192.
- **T6 (prescan).** Rename to `the_eighty_kilobyte_box_pattern_is_refused_by_the_pre_scan_before_the_parser_runs`. Call `parse_source` with a visit closure that sets an `AtomicBool`.
  - Guarantee: the pre-scan refuses the input before syn runs.
  - Assert: `Budget::Recursion` and the flag is unset.
  - Mutant: run `syn::parse_file` before `prescan` in `parse_counted`. Visit runs (or the parser overflows and aborts), so the test fails.
  - The scan's own cost needs no counter here: the recursion budget refuses after at most about 1927 counted units, so only a bounded prefix is scanned.
- **T7 (moved to `resolve.rs` tests).** New name: `a_path_of_fifty_thousand_segments_reads_each_segment_a_bounded_number_of_times`. Load with `MemoryTree` and keep both 50,000-segment paths.
  - Guarantee: the cost of a path is linear in its segments.
  - Assert, per `resolve` call: `SegmentReads ≤ 4·(S+2)` (three name-space prefix scans and walks, plus the unresolved-prefix scan) and `Lookups ≤ 2·(S+2)`. The bound is the ceiling, and both counts must be > 0.
  - Mutant: re-walk each prefix from the crate root (one walk per prefix, about S²/2 reads). The ceiling trips at once.
  - Remove the test from `resolve_paths.rs`.
- **T2, T3, T4 (moved to a new `src/exports/tests.rs`).** Each test loads at sizes N, 2N and 3N (N = 500) outside `measure`, then lists inside it with a fail-fast ceiling of 64·N.
  - Assert exact linear growth with no tuned constant: `f(3N) − f(2N) ≤ f(2N) − f(N)` and `f(N) > 0`, for `ViewReads` (T2, T3, T4) and `WalkSteps` (T3, T4). The existing correctness checks stay (`items.len()`, `reexports.len()`, no `demo::Debug`).
  - T4 keeps its 32 MiB budget assertion at 3000. It is a byte count and already compliant. The release-only 10,000 case is dropped: the load alone takes 27 s in release, and the growth check proves linearity exactly.
  - Mutants:
    - T2: in `Surface::bound`, iterate `routes.list` and test `view_holds` on each route instead of `every.get(name)`. ViewReads grows as N².
    - T3: call `glob_reach` per carried import inside `unbound_imports`. WalkSteps grows as N².
    - T4: build a new `Carry` / re-walk the reach per shadowed name. WalkSteps grows as N².
  - New names: `…_is_listed_in_view_reads_linear_in_its_names`, `…_is_listed_in_walk_steps_linear_in_its_names`, `readers_of_carried_names_beside_sibling_shadows_list_in_linear_work_and_bounded_bytes`.
- **T9 (module_tree, integration).** Load through `Counting` with a ceiling of 200 ops. Keep the refusal assertion (`ModulePaths`, limit 262,144, file `src/m*`, a line). Rename to `a_chain_of_files_that_each_declare_the_next_twice_is_refused_by_the_module_path_budget_opening_each_file_once_beside_a_small_diamond`.
  - Guarantee: the module-path budget stops the walk at its limit, and each file is opened once per spelling. Every `mod` here has `#[path]`, so `identify` caches each spelling and the walk makes no tree calls per arrival.
  - Assert: `opens ≤ files + manifests` and `opened_read ≤ 31`.
  - Mutant: delete the `spellings` cache early return in `Walker::identify` (`modtree.rs:446`). That gives 2^18 opens and the ceiling trips.
  - Removing the `ModulePaths` check makes the walk hang, which the owner accepts. The refusal assertion still catches a budget that refuses at the wrong count.
- **T10.** Drop the clock and keep the refusal. Add a `Counting` ceiling: no tree op beyond discovery and `src/lib.rs`. The guarantee is that inline alternatives are counted.
- **T12, T13 (watchdogs).**
  - Delete `within_watchdog` and `WATCHDOG` (and the now-unused `mpsc`, `thread` and `Duration` imports) from `tests/support/mod.rs`, and `within_watchdog` from `src/tree/disk/tests.rs`. Call the operations directly and rewrite the `Scratch::fifo` doc comment.
  - This is safe because every `DiskTree` open is either `O_PATH` (Linux) or `O_NONBLOCK` (`other_unix.rs:290`, `handle_path.rs:118`). A containment regression that reaches the FIFO sentinel then returns `NotRegular` instead of `OutsideRoot`, which fails the assertion and does not hang. Only a combined regression, a broken containment check plus a blocking open, hangs the suite.
  - Test names are unchanged.
- **T14 (inotify probe).** Remove the writer thread, `own_task` and `wait_until_blocked_in_open`. Set up `rustix::fs::inotify::init(NONBLOCK | CLOEXEC)` and `add_watch(fifo, OPEN)`.
  - **Negative control:** the test opens the FIFO with `O_PATH`, and the queue must stay empty (`WouldBlock`). This confirms fsnotify skips `FMODE_PATH`, which the Linux strategy relies on. If it fails, fail hard with a message.
  - Run `tree.read` under the swap and keep `assert_outside`. Then assert the queue is empty.
  - **Positive control:** the test opens the FIFO `O_RDONLY | O_NONBLOCK` and exactly one `IN_OPEN` must arrive. This proves the probe is live.
  - Guarantee: under the Linux and OtherUnix strategies an outside FIFO is never opened.
  - Mutant: reopen the candidate before verification in `linux.rs`. That produces `IN_OPEN` and the test fails.

## 4. Commits

The pre-commit hook runs `self_workplan`. So **every commit that renames or moves a bound test must, in the same commit, update the `tests:` bindings and the Verification text, then run `make docs-sync` to regenerate the matrix.**

1. `tests/support/counting.rs`. Migrate `Recording` and `CountingReads` to it.
2. `src/work.rs` (the tallies, the ceiling and its own unit test) plus `mod work` in `lib.rs`.
3. facts.rs: T1 and the new wide-line unit test. symbol_index.rs: T11 rename. Bind both new unit tests and update binding 266 and Verification ~1866 and ~1934.
4. glob.rs and members.rs ticks and unit tests; manifest_caps.rs (T5); trait_keys.rs (T8). Update bindings 215–219 and Verification ~1848, and bind the glob and members unit tests.
5. resolve.rs tick and test; remove T7 from resolve_paths.rs. Update binding 210 and Verification ~2107.
6. glob_scope.rs and exports.rs ticks; new `src/exports/tests.rs` (T2–T4); remove them from exported_items.rs. Update bindings 315, 317, 320 and Verification ~2383–2389.
7. module_tree.rs (T9, T10). Update binding 61 and Verification ~1756.
8. prescan_budgets.rs (T6). Update binding 147 and Verification ~1916.
9. Remove the watchdogs in `tests/support/mod.rs`, `disk_tree_containment.rs`, `workspace_containment.rs` and `src/tree/disk/tests.rs` (T12, T13). Names are unchanged, so no binding edits.
10. T14 inotify. The name is unchanged; edit Verification ~2272–2275 if it describes the blocked writer.

Bound tests whose names change: 61, 147, 210, 215–219 (three of them), 266, 315, 317, 320. New bindings recommended: the T1 and wide-line facts unit tests, the glob and members unit tests (they back the Contract claim at ~1021), and T10 (optional).

## 5. Risks and owner decisions

1. **Counters only see the code they instrument.** A regression that only allocates, such as building each prefix with `segs[..k].join("::")`, passes the T7 counter. Options: (a) accept it, since counters sit at the shared accessors; (b) count allocations per thread through a vetted dev-dependency that keeps the `unsafe` code inside the dependency (needs a `deny.toml` review); (c) count instructions with perf. Option (c) is not portable: `perf_event_paranoid` and container seccomp often block it. Recommendation: (a).
2. **"Without blocking" can't be expressed as a count.** Blocking is unbounded waiting, not work. After the watchdogs go, a regression that removes `O_NONBLOCK` hangs the suite (owner-accepted), and only the CI job timeout bounds it. An optional count-free way to turn it into a failure: a `cfg(test)` hook that records the open flags in `other_unix` and `handle_path`, with an assertion that `NONBLOCK` (or `O_PATH`) is set.
3. **Removing a walk budget makes the walk hang.** This applies to T9 and T10 when the `ModulePaths` check is removed. It is accepted; a `Work` ceiling there would just duplicate the budget.
4. **T14 depends on kernel behaviour.** It needs fsnotify to skip `O_PATH` descriptors. The negative control makes that explicit and fails hard where it doesn't hold.
5. **The Contract still reports wall-clock figures** (around lines 699–712, 1029–1033, 1318–1319). They came from timing runs, which the rule now forbids. Decision needed. Recommendation: restate ~1021 and ~1318 as the step and segment-read bounds above, and ~1033 as "refused before any pattern is expanded".
6. **Moving tests into `src` changes their binding paths** (for example `raf_source::exports::tests::…`). The settle and disk tests already set the precedent.
7. **Optional lock-in.** A repo gate could forbid `std::time` in `crates/raf-source/{src,tests}`. It would have to exclude `examples/`.
