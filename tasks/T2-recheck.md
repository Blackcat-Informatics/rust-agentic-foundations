VERDICT: PASS

# T2 recheck (fix commit 3f8f469 on 08071b8)

Commands: `cargo test -p raf-census --locked` exit 0 (12 unit, 19 forms, 4 golden, 1 doctest).
Probe rerun in the scratch clone at 3f8f469 (throwaway `tests/probe.rs`, worktree not edited).

1. False collision: CLOSED. `passes.rs` `next_number` now also reads `{$n` placeholders
   (`placeholder_numbers`), so the start stays a pure function of the shim form. Probe:
   `let x = 1; println!("{a} {x}")` now gives `let $1 = 1 ; println ! ( "{$0} {$1}" )`, and
   against `{x} {x}` the comparison is `equal_from=None, differs_through=Value` (was
   LiteralTier). `format!("{a}{z}")` likewise `$0`/`$1`. Fixtures
   `numbering_placeholder_parameter`, `_above_tier` (51 tokens) and `_versus_argument` are
   Distinct and pinned; no existing pin moved (monotonicity test still green). A user-written
   `"{$3}"` in a non-macro string would only raise the start number; harmless and
   deterministic.
2. Golden self-test: CLOSED. `golden.rs` `pin_problems(computed, pinned)` is the single
   comparison; `every_fixture_fingerprint_is_pinned` asserts it is empty on the real table, and
   `the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry` calls the same function
   with the passing neighbour and five negatives (unpinned, orphan, moved with the bump text,
   pinned twice, computed twice), each asserting the named problem.
3. Trailing-parameter pair: CLOSED. `numbering_unused_trailing_parameter` is Equal(Verbatim)
   (pins equal, probe agrees); `numbering_used_trailing_parameter` is Distinct (pins differ).

The report's fixture count (187) matches the table. No new findings.
