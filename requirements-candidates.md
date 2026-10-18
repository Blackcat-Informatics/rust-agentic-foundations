# Requirement candidates — issue #14

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 14 | list-under-requirement-heading | - Model both rules in the settling of `use` paths, keeping resolution independent of declaration order. |
| B2 | body | 15 | list-under-requirement-heading | - Add an every-order fixture for each rule. |
| B3 | body | 16 | list-under-requirement-heading | - Re-run the rustc-oracle fuzz with private globs, with no new disagreements. |
| B4 | body | 17 | list-under-requirement-heading | - Remove the two sentences from the Contract's Constraints. |

## Comments that matched no rule

None.
