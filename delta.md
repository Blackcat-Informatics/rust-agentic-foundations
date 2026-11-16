# Delta reads for the ratcheted-rule change

## 08ea247d to dae51adf (fix for the independent review)

- Read: crates/raf-census/src/report/ledger.rs and declarations.rs.
- The ratchet check now runs after the patterns are read and receives them. It refuses a `ratchetedRule` that names a declared pattern id, wherever that pattern is forbidden.
- No other source changed in this delta. The test changes (the two-job negative fixture and the json-pointer-escape row) were not read line by line; their results are in the census_gate log.

## dae51adf to 92da1f1f (file-size split)

- Read: the diff of crates/raf-census/tests/census_gate.rs and the new census_gate/ratchet.rs, compared as sets of non-blank lines.
- Lines only in the old file: the moved test functions, the moved helpers, and the call sites that now name `ratchet::`.
- Lines only in the new file: the module doc comment and licence header, the `pub(super)` visibility on three helpers and two declaration checks, the `use super::{...}` import list, and the moved bodies.
- The parent file's import list changed to drop items used only by moved tests.
- Test bodies, assertions and fixtures are unchanged apart from paths and visibility.
- The moved set is wider than this branch's tests: the step 1 pattern-scope and pattern-ratchet tests moved with them. This was needed to bring census_gate.rs under the 5,000-line limit, since origin/main was already 5,480 lines.

## Net source diff at 92da1f1f (merging reading)

- Read every raf-census source hunk against origin/main.
- The ledger refuses an invalid `ratchetedRule` at load, and rejects a pattern id, an unenforced job, a rule the job does not forbid, a job-scoped detector, and a rule another job forbids outright.
- A rule the Gate requires cannot also be ratcheted. This is refused at evaluation.
- Counted sites are keyed by (rule, unit, ordinal), so two hits on one line count twice and a rule reached through two jobs counts once.
- Registered-rule and fact-detector hits take the start of their line, no earlier than the owning item's start. Built-in syntax hits keep their byte.
- `RegisteredDebtWrite.siteCount` is written only when positive, and refused when zero on read. The report form is unchanged.

## Observed nit, not changed

- The doc comment on `record_line` in crates/raf-census/src/gate/rules.rs names the parameter `owner`; the parameter is `key`. Wording only, no behaviour. Not reopened for review under the bound.
