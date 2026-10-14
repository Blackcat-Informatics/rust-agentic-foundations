**R1 fixed: raf-okf body memory is bounded** (commit c10abd9, pushed)

- **Gap:** S-1 / P-F1. Every line, prose line and PROPOSED block cloned its section's heading slug, so memory grew with lines × heading length. A 200 KB document exhausted the allocator, which breaks the okf-profile Contract's "no unbounded allocation".
- **Fix:** each section has one shared slug allocation (`Arc<str>`), used by the heading, the section, the anchor table and every line and block under it. Public accessors are unchanged and still return `&str`.
  - The plan named a section index. `Arc<str>` was chosen because it keeps the public signatures and the memory is equally linear. The reviewer accepted it.
- **Check:** `crates/raf-okf/tests/bounds.rs::a_long_heading_over_many_lines_stays_linear`.
  - A counting allocator with a 1 GiB ceiling compares two 1 MiB documents, one with a 60 KB heading and one with a 1-byte heading, for prose lines and for PROPOSED blocks.
  - After the fix, the peak difference is 307,181 bytes against an allowance of 311,296.
  - **On the old code it aborts:** `memory allocation of 61440 bytes failed`, SIGABRT. This was reproduced independently by the reviewer.
- **Also:** the new fuzz target `parse-document` (`Document::parse` + `Profile::validate`) is run by `make fuzz`.
- **Gates:**
  - `cargo test -p raf-okf --locked`: pass.
  - Workspace clippy with `-D warnings`: clean.
  - `make fuzz`: both targets ran 5000 times, no findings.
  - `make reuse`: 210/210.
  - Pre-commit hook: docs_contract 138 and repo_gates 55 passed.
- **Review:** `tasks/R1-review.md`, VERDICT: PASS. Three advisories: the margin is thin, the test has only one section, and there is no child-process harness for the old-code run.
