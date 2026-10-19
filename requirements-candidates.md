# Requirement candidates — issue #29

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 3 | modal | Constraint: `exclusion_for` marks every exclusion that matches an entry, not only the first one it keeps, so that an exclusion no entry uses can be reported. An index must still return every matching exclusion for each entry (and the report of unused exclusions must stay the same); an index that stops at the first match would hide unused exclusions. |

## Comments that matched no rule

None.
