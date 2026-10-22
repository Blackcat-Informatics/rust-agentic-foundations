VERDICT: PASS

# T6 recheck: fix commit fec6c21 against findings R1-R5

I checked out fec6c21 in `scratch/t6review` and changed nothing in the worktree. There were no timing runs and no mutation passes.

## Commands and exit status
* `cargo test -p raf-workplan -p cargo-workplan --locked`: exit 0, every test result has 0 failed.
* `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`: exit 0 (31, 68 and 1 passed).
* `make clippy`: exit 0.
* `make fmt reuse issue-refs`: exit 0. This includes the SPDX lines now in `.rtk/filters.toml`.
* `cargo workplan check`: exit 0.
* `cargo workplan assess --list`: 0 stale.

## Findings
* **R1: closed.** The test target compiles and clippy is clean under `-D warnings`.
* **R2: closed.** `self_workplan` passes and `assess --list` shows 0 stale. The fix commit says it re-read the context of each of the 12. It also corrected two sentences that had become untrue, about `--strict` and the coverage reports in `workplan-cli` and `workplan-two-way-coverage`.
* **R3: closed.** New literal tests cover the excepted table with its JSON `lines-excepted` object, the `| Error |` table with its `lines-error` object, and the member order of `lines-excepted` through the binary. The tests pass.
* **R4: closed.** New binary tests run ceiling rows with `of:` classes, raised rows within the ceiling, and `default` not enforcing. They also cover an unmatched exception failing `check` under default beside its matched neighbour, each run in both shapes. The tests pass.
* **R5: closed.**
  * The fixture `a_directory_that_is_a_workspace_of_its_own_has_a_row_when_the_source_roots_reach_it_and_none_when_they_do_not` pins both sides: a nested workspace gets its own row and its own ceiling when reached, and no row when it is not.
  * The Constraint in `workplan-strict.md` carries the `raf:tested-by` marker.
  * plan.md records the reversal of Rev 2 B3 as "Rev 4".
