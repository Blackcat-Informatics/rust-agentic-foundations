VERDICT: PASS

# T7 recheck: fix commit dc1115b

Checked in the clone `scratch/t7review` at dc1115b. Nothing changed in the worktree.

| Check | Result |
| --- | --- |
| `cargo test -p raf-workplan --locked --test debt --test complete` | exit 0: `complete` 66 passed, `debt` 28 passed, 0 failed |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | exit 0: 31 / 1 / 68 passed |
| `make clippy` | exit 0 |

* R1 fixed: `crates/raf-workplan/src/strict.rs:18` is `//! # Examples, the matrix and the sections` again.
* R2 fixed: the write-through test (renamed `every_kind_a_row_can_hold_...`) now raises the macro-generated
  `pub use made::Made` and asserts the written row `coverage-reexport-unbound demo::Made`; it then
  holds. All ten `DEBT_KINDS` are written and read back.
* R3 fixed: two `complete` fixtures. A thin Purpose refuses with `strict-section-thin` after the record
  is removed and writes nothing; with a debt row holding it the change completes. A second, paid row
  refuses with exactly one `debt-resolved` and writes nothing; the row set equal to the present debt
  completes. The block in `workplan-complete-gate.md` is current text, both tests are bound and in the
  matrix, and `self_workplan` passes (assessment re-read, 0 stale).

Non-blocking: the "Not done here" paragraph of Validation: T7 in `validation.md` has a garbled sentence
("... but (proved in the fix commit below).") from the amendment. It is a stage file, not committed;
tidy it when convenient.
