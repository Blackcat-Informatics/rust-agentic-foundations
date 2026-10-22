## Context

RAF must satisfy the single workplan coverage policy delivered by #16. That policy becomes the default for every production caller; there is no separate lightweight mode, strictness flag, or coverage-record strictness switch. Repository adoption must therefore be ready before that default lands.

The original adoption inventory reported about 590 orphan symbols, 196 uncovered claims, thousands of orphan lines, 12 capabilities whose Usage named no bound example, and one thin section. Those are historical observations, not current acceptance measurements. The integrated repository must be measured again, including newly delivered first-party crates.

## Scope

- [ ] Inventory actual findings by capability, rule, package, public binding and source ownership.
- [ ] Repair the first-party code/documentation bindings, claim markers, examples, verification matrices, section content and reverse coverage reported by the rules.
- [ ] Give every permitted narrow exception a specific reason; do not suppress adoption with broad exclusions or a baseline presented as completed work.
- [ ] Reach zero first-party debt in `work/debt/default.md`, retaining the shrinking ratchet as enforcement. A development baseline may only shrink and does not satisfy the terminal zero-debt target.
- [ ] Wire the actual repository self-workplan gate through the sole default policy and reconcile capability/architecture documents and lifecycle records.

## Acceptance criteria

- [ ] `cargo workplan check` passes with no policy flag or strictness key, and the mandatory repository gate judges the same policy.
- [ ] Every delivered first-party crate satisfies the rules through real bindings, claims, examples, matrices and source coverage; any permitted exception is narrow and reasoned.
- [ ] First-party debt is zero, and tests prove the ratchet refuses growth and stale identities beside valid shrinking cases.
- [ ] The actual combined tree passes the required gates and lifecycle completion. Fixture-only strict-mode proof or an unpaid baseline does not complete adoption.

Delivery is grouped with #16 and #40 to make the sole-default flip, real adoption and RDF result formats coherent. The acceptance for this issue remains distinct from rule implementation and serialization.
