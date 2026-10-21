VERDICT: PASS

# T5 recheck (fix commit 2ec6a1e), clone scratch/t5review, worktree untouched

Commands:
- Missing-key probe (temporary test, deleted): Index::from_json with `body`/`shim` absent -> Err("missing field `body`"); with both `null` -> Ok. Exit 0.
- Example `-- crates` vs `-- <abs>/crates`: both exit 0, cmp identical (finding 2 fixed). `-- /nonexistent`: exit 1, named error. No-argument run twice: identical; `-- crates` twice: identical.
- cargo test -p raf-census --locked: exit 0 (22, 19, 4, 15, 10 reports, 20, 1 doctest).

Finding 1: fixed. report.rs `deserialize_with = "required"` on both fields; tests cover the null-carrying valid neighbour and removal of each key.
Finding 2: fixed. builtin::of_dir reads an absolute dir as its parent's tree with the file name as root; test an_absolute_directory_reports_the_same_as_the_relative_spelling added and the example now shares that code.

Remark (non-blocking): an absolute repository root now yields file paths prefixed with its directory name (the same as relative `crates` yields `crates/...`), so `-- <abs root>` differs from `-- .`. That follows from the stated rule (paths are relative to the parent tree); worth one sentence in the capability Examples if T6 shows a directory run.
