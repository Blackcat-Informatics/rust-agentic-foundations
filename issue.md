# Issue #29: Index the coverage product instead of pinning entries x exclusions

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

Coverage judges every entry against every exclusion; the test pins that product exactly instead of replacing it with an index. Build an index so the work grows with entries plus exclusions, and replace the exact product pin with a linear-growth check.

Constraint: `exclusion_for` marks every exclusion that matches an entry, not only the first one it keeps, so that an exclusion no entry uses can be reported. An index must still return every matching exclusion for each entry (and the report of unused exclusions must stay the same); an index that stops at the first match would hide unused exclusions.


## Comments (0)

