# Requirement candidates — issue #9

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 6 | checkbox | - [ ] Fixture git repo, built in a temp dir: one capability redesigned 4 times through successive changes (decisions amended and reversed, interfaces and constraints evolving), modelled on the "Amended YYYY-MM-DD" ADR pattern |
| B2 | body | 7 | checkbox | - [ ] At the end: |
| B3 | body | 15 | checkbox | - [ ] The fixture capability starts `considered`, with open questions; the implementation renames the API and drops one guarantee |
| B4 | body | 16 | checkbox | - [ ] `complete` refuses until Usage, Examples and the bindings match the real API and the dropped claim has an outcome |
| B5 | body | 17 | checkbox | - [ ] It then succeeds, leaving the capability `delivered` with a real usage example, current API references, explicit limitations and a test-traceability table |
| B6 | body | 20 | checkbox | - [ ] `tests/self_workplan.rs` runs `raf_workplan::Gate` on this repo, wired into `make check` and the pre-commit hook |
| B7 | body | 21 | checkbox | - [ ] Advance this repo's capability states with `cargo workplan status` and `change complete` for every M1 change record |
| B8 | body | 24 | checkbox | - [ ] Both scenarios pass as integration tests |
| B9 | body | 25 | checkbox | - [ ] Self-workplan and self-census gates both pass in CI |
| B10 | body | 26 | checkbox | - [ ] Every M1 capability is `delivered`, with resolved bindings and a generated verification matrix |

## Comments that matched no rule

None.
