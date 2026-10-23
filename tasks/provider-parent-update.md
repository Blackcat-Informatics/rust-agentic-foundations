<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

## Priority and outcome

Highest priority: use PurRDF for every functionality it supplies, throughout RAF production code, tests, fixtures, examples and tooling. Prevent competing implementations from being introduced again by enforcing the preferred provider through Census.

This issue tracks the complete outcome. Existing implementation issues retain their acceptance criteria and ownership; this does not duplicate their implementations or treat focused tests as delivery.

## Current implementation and delivery status

Three independently qualified adoption units have landed:

- PR #54 (`44adeaa`): native scalar/date/decimal/hex helpers, allocation probes and shared Census BLAKE3/hex callers.
- PR #55 (`6cfa0b8`): native shared RDF storage and primitives.
- PR #38 (`d500d97`): complete default Workplan policy, native RDF command consumers, truthful compiler-proved platform exclusions and actual zero-debt adoption across all eight first-party packages. Both live completion lifecycles passed; all 113 assessments are current/accepted, normal hooks and all twelve hosted jobs passed, and the landed source matches the qualified head. Evidence is durably archived and completed worktrees are cleaned. Issues #16 and #37 are completed.

The full outcome remains open:

- Preferred-provider enforcement is not yet implemented. Census ledger/debt/import (#4), grounded rules with the exact locked-source consumer (#5/#48), then real CLI registration and RAF self-gate (#6/#12/#9) are now the primary dependency path.
- Resolver/native integration remains preserved in #28. Two further improvements compile and pass fourteen focused proof cases, but current ambiguous-hub triage is 16,257,228,363 instructions against 6,809,205,378: ratio 2.38754 still fails the original 1.5 ceiling. Broad qualification of that known failing candidate has stopped; all original semantic, allocation and performance requirements remain.
- The remaining source foundation in #48 is preserved. Complete three-host `cargo workplan verify` acceptance in #23 is advancing independently; PR #38's mechanism and existing platform CI do not themselves satisfy that command-level requirement.

Qualification still pins PurRDF `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`. Current upstream `217fbf9` was inspected read-only for relevant graph/allocation APIs; this bounded inspection is not whole-library equivalence or qualification of a new pin. Every new operation must prefer compatible native APIs.

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
- #48: extract a complete locked dependency/source/symbol foundation for grounded provider enforcement, using the existing shared resolver and verified Tree boundary. Exact locked source identity, aliases, eligible target edges and actionable incomplete evidence are required. Full portability in #43 and #44 is independent; all original requirements of those issues and #48 remain open until delivered.
- #4 and #5: Census ledger, rules/patterns and preferred-provider enforcement.
- #6, #12 and #9: real CLI registration, self-gate and end-to-end acceptance.

Upstream glob capability: https://github.com/Blackcat-Informatics/purrdf/issues/671 covers RDF-term matching, filesystem/Rust expansion into RDF, and RDF pattern/plan representation. It does not exist yet and is not evidence that RAF already delegates those operations.

