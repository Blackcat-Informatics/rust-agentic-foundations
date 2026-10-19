# Validation: Task 1 (index the coverage product)

No timing runs: every check below is a count, an equality, or a pass/fail of a test.

## Mutants (scratch clone)

Clone: `git clone --no-hardlinks` of the worktree to `/tmp/raf-mutants-RmbicP/clone`
(`pwd` checked before every edit), the Task 1 diff applied and committed there as the
base. Each mutant edits `crates/raf-workplan/src/coverage.rs` in the clone only, runs
`cargo test -p raf-workplan --locked --lib coverage::` and
`cargo test -p raf-workplan --locked --test coverage`, and restores the base file.
The runner is `/tmp/raf-mutants-RmbicP/scripts/mutants.py`; the full output is
`/tmp/raf-mutants-RmbicP/run2.txt`. The worktree was never mutated.

| Mutant | Edit | Failing tests (observed failure) |
|---|---|---|
| M1 index stops at the first match | `return` right after the first lookup that finds an exclusion | `the_index_finds_the_first_match_and_marks_what_testing_every_exclusion_finds` (matched vector differs, entry `["a::a"]`); `an_exclusion_that_another_matches_first_is_marked_and_only_one_that_matches_nothing_is_reported` (more than one `coverage-exclusion-unmatched`); `equal_exclusions_on_a_hand_built_list_are_both_marked_and_the_first_is_the_match` (`[true; 4]` differs); `the_exclusions_are_indexed_once_and_each_entry_looks_up_its_paths_and_their_modules` (lookups `left == right` failed: 12); integration `the_first_matching_exclusion_gives_the_reason` (a shadowed exclusion reported unmatched). Lib exit 101, integration exit 101 |
| M2 whole path used as a prefix key | `ancestors(path).skip(1)` to `ancestors(path)` | cross product (entry `["a"]`: `Prefix(a)` matches `a`); R3 check test (reasons differ); hand-built duplicates; R1 test (`IndexLookups work passed the ceiling of 1408: 1409`); integration `a_prefix_matches_below_its_module_and_never_a_sibling_that_shares_its_text`, `the_first_matching_exclusion_gives_the_reason`. Exits 101 / 101 |
| M3 ancestors walk skips the nearest ancestor | `.skip(1)` to `.skip(2)` | cross product (entry `["a::a"]`); R3 check test; hand-built duplicates; R1 test (marks: 12); ten integration tests, among them `a_unit_test_module_is_excluded_by_its_prefix_with_the_reason_shown`, `an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file`, `the_first_matching_exclusion_gives_the_reason`. Exits 101 / 101 |
| M4 largest index chosen | `seen.min(position)` to `seen.max(position)` | cross product (first match differs, entry `["a::a"]`); R3 check test (reasons differ); hand-built duplicates (`Some(0)` differs); integration `the_first_matching_exclusion_gives_the_reason`. Exits 101 / 101 |
| M5 index built once per entry | `Judge::judge` rebuilds `ExclusionIndex::new` before each entry, keeping the flags | R2(b) `the_work_grows_with_the_entries_and_the_exclusions_together_and_no_faster` (growth assertion: `6490, 16300, 30510`, 30510 + 6490 > 2 * 16300); R2(c) `every_kind_of_work_is_held_to_a_ceiling_linear_in_the_entries_and_the_exclusions` (`ExclusionsIndexed work passed the ceiling of 4080: 4081`); R1 test (`ExclusionsIndexed work passed the ceiling of 1408: 1409`); `the_work_grows_with_the_exclusions_and_no_faster` (indexed counts differ). Integration tests pass (exit 0): the report is the same, only the work differs |
| M6 marking loop removed | `mark` no longer sets a flag; only the returned first match is marked | cross product (matched vector differs, entry `["a::a"]`); R3 check test (shadowed exclusions reported unmatched); hand-built duplicates; integration `the_first_matching_exclusion_gives_the_reason`. Exits 101 / 101 |
| M7 the product code | the slice-walking `matches` and `exclusion_for` restored, ticking `IndexLookups` once per exclusion tested against an entry, and the index's `exclusion_for` delegating to it | R2(b) (growth assertion: `4850, 13980, 27510`, 27510 + 4850 > 2 * 13980); R2(c) (`IndexLookups work passed the ceiling of 4080: 4081`); R1 test (`IndexLookups work passed the ceiling of 1408: 1409`). Integration tests pass (exit 0): the product and the index give the same report |

R2(b) and R2(c) fail against the product code (M7) and against the index built per
entry (M5), as required. M5 and M7 leave `tests/coverage.rs` green, which shows the
report of the index equals the report of the product on every integration fixture.

A first run of M6 removed the flag assignment and left `matched` unused, which the
`-D warnings` of the host's cargo configuration refused to compile; the recorded M6
uses `let _ = &matched;` in its place so it compiles and runs.

## Gates (worktree)

| Command | Exit | Result |
|---|---|---|
| `cargo test -p raf-workplan --locked --lib coverage::` | 0 | 9 passed |
| `cargo test -p raf-workplan --locked --test coverage` | 0 | 24 passed (file unchanged) |
| `cargo test -p raf-workplan --locked` | 0 | 36 test binaries ok, 0 failed |
| `cargo test --locked --test docs_contract --test self_workplan` | 0 | 28 + 1 passed |
| `make fmt clippy doc deny reuse issue-refs` | 0 | clean |
| `cargo run -q -p cargo-workplan -- workplan check` | 0 | no findings |
| `cargo run -q -p cargo-workplan -- workplan assess --list` | 0 | every assessment current and accepted (six of `cap:workplan-two-way-coverage` re-assessed after reading each `--context`) |
| pre-commit hook (fmt, reuse, `docs_contract`, `repo_gates`, `self_workplan` on the staged index) | 0 | passed |

Commit `1f7a652a8a4fddc3a3d9036087de4de7689526b4`, pushed to
`origin/paudley/29-index-the-coverage-product-instead-of`.

## Handoff (head 9ac04cf)

Local gates (each `make <target>`): fmt exit 0 ; clippy exit 0 ; test exit 0 ; test-release exit 0 ; doc exit 0 ; deny exit 0 ; reuse exit 0 ; msrv exit 0 ; fuzz exit 0 ; licenses exit 0 ; issue-refs exit 0 ; windows-gnu raf-source exit 0 ; cross-check exit 2 ; done ; 

`make cross-check` on this host, verbatim: `error: toolchain 'nightly-x86_64-unknown-linux-gnu' is not installed` / `make cross-check: target 'x86_64-apple-darwin' is not installed; run 'rustup target add x86_64-apple-darwin'` (exit 2). The Windows check by hand (`RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target x86_64-pc-windows-gnu`) exit 0. Hosted CI: 12 of 12 pass on 9ac04cf including cross-check, test-macos and test-windows.
