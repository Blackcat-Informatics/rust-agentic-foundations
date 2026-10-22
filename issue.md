# Issue #53: Complete PurRDF adoption and enforce preferred-provider reuse

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

## Priority and outcome

Highest priority: use PurRDF for every functionality it supplies, throughout RAF production code, tests, fixtures, examples and tooling. Prevent competing implementations from being introduced again by enforcing the preferred provider through Census.

This issue tracks the complete outcome. Existing implementation issues retain their acceptance criteria and ownership; this does not duplicate their implementations or treat focused tests as delivery.

## Current implementation and delivery status

Substantial replacements are implemented in the existing source and strict-mode worktrees, but the complete migration is not merged or fully qualified:

- RDF parsing, terms, writing, traversal, vocabulary, hashing, hex conversion and SCC algorithms.
- Compatible scalar, date and text helpers, temporary-directory fixtures, allocation counters, allocation admission and resolver worklists.
- Current source integration passes 255 focused integration tests. Native allocation tests retain the hard allocation cap and original numerical allowances.
- Remaining concrete adoption prerequisites include reachability allocation admission, compiler-registered allocator coverage ownership, final contract reconciliation, actual lifecycle completion and mandatory repository gates.
- Resolver performance still fails its original instruction ceiling. That remains a blocker for its resolver delivery; identify and land independently complete adoption units where their actual dependency boundary permits it.

The audited native source is `~/Active/purrdf` main. Current RAF qualification pins `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`; the subsequently inspected main delta does not change the imported API closure. Registry version equality alone does not establish API equivalence.

## Completion requirements

- [ ] Audit actual production callers and all test, fixture, example and tooling helpers against the available PurRDF APIs. Replace every supplied functionality and remove the competing generic implementation.
- [ ] For remaining domain adapters, identify the exact policy they add and the concrete missing native operation or incompatible semantics. An unused dependency, dummy import, decorative call or redundant validation pass is not adoption.
- [ ] Preserve original ordering, outputs, failure propagation, allocation grants, hard caps and numerical acceptance limits. Native failures must remain actionable typed failures, never missing results or partial success.
- [ ] Qualify real callers and negative/valid neighbors, complete capability contracts and lifecycle records, run mandatory gates and normal hooks, and merge the completed adoption units.
- [ ] Implement Census policy expressing “always use library or project X for functionality,” with RAF's policy preferring `purrdf.*`.
- [ ] Ground preferred-provider findings in actual functionality and implementation/output evidence. Reject a competing implementation even when it imports or calls PurRDF; accept a real thin native adapter. Cover renamed dependencies/imports and actionable missing or ambiguous provider evidence.
- [ ] Register and run the policy through the actual Census CLI and RAF self-gate. A planted competing implementation must fail; the real migrated repository must pass without exclusions or a baseline that disguises duplication.
- [ ] Close only after these outcomes have landed and their required acceptance is complete. Keep unimplemented upstream capabilities explicit.

## Existing delivery mapping

- #28, #33, #35 and #24: source resolver and native primitives/admission integration. Preserve the resolver's original semantic and performance acceptance.
- #16, #37 and #40: strict-mode adoption, coverage ownership, native RDF consumers and completion gates.
- #43 and #44, then #48: source portability and dependency-symbol prerequisites for grounded provider enforcement.
- #4 and #5: Census ledger, rules/patterns and preferred-provider enforcement.
- #6, #12 and #9: real CLI registration, self-gate and end-to-end acceptance.

Upstream glob capability: https://github.com/Blackcat-Informatics/purrdf/issues/671 covers RDF-term matching, filesystem/Rust expansion into RDF, and RDF pattern/plan representation. It does not exist yet and is not evidence that RAF already delegates those operations.


## Comments (0)

