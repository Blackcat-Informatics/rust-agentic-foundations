# Requirement candidates — issue #27

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 5 | checkbox | - [ ] Replace the per-crate timeout and the sleep poll with a bound on counted work (the index already reports `Usage` and refuses at its budgets), or remove the example if its purpose is served by the differential test against rustc. |
| B2 | body | 6 | checkbox | - [ ] Extend the gate to cover `examples/` once the example is clean, or state in the gate why it stays exempt. |
| B3 | body | 9 | checkbox | - [ ] `grep -nE 'std::time\|Instant\|Duration\|sleep\|recv_timeout\|SystemTime' crates/raf-source/examples` is empty, or the example is gone. |
| B4 | body | 10 | checkbox | - [ ] The repository gate passes with `examples/` in scope. |

## Comments that matched no rule

None.
