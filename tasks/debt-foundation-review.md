<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Debt foundation review

Read-only inspection of the new model/codec/comparison, shape applicability,
focused debt fixtures and both governing ratchet documents. No product edits,
builds, tests or subagents. Gate/CLI delivery is not assessed.

## Defect: containment can conceal a changed exact debt record

Disposition after the owner's correction: resolved by source inspection.
`compare` now tests each recorded row for exact presence and emits MissingGroup
when absent. A present new strict subset is exempt from growth only when its
recorded superset is itself absent. The added regression asserts removal of the
smaller row, reverse addition as NewGroup, and an unchanged control. The existing
lone-subset shrinkage controls remain, while the superset assertion now requires
both the new larger row and missing old row. No tests or builds were run for
this disposition, and no Gate/CLI acceptance is inferred.

Two adjacent API descriptions need the same qualification: compare's “A strict
subset ... is shrinkage, never new growth” and NewGroup's “not contained in any
recorded group” are broader than the corrected rule. State that only a subset
replacing an absent recorded superset is shrinkage; an extra subset beside an
unchanged recorded superset is new debt. This is a wording correction to the
same fix, not another implementation finding.

Original finding and counterexample follow for provenance:

`crates/raf-census/src/report/debt.rs:118–129` compares group collections by
mutual containment rather than detecting every missing exact keyed entry.
`DebtRecord::new` at lines 85–90 rejects identical groups but accepts distinct
same-kind nested groups. The RDF reader ultimately calls that constructor.

Concrete accepted-model counterexample, with unchanged normal form and no jobs:

- Recorded Body groups: `{a,b}` and `{a,b,c}`.
- Present Body groups: only `{a,b,c}`.
- Every present group is contained in a recorded group, so no NewGroup is emitted.
  Every recorded group is contained in a present group, so no ReducedGroup is
  emitted either. `compare` therefore returns an empty vector although an exact
  recorded row disappeared. Reversing these two records also returns empty,
  concealing addition of the nested row beside its unchanged parent.

This contradicts the architecture's requirement that a missing recorded entry
fails until rewritten, the capability's exact-present-debt/no-slack rule, and
the method's own “Every difference requires action” claim. It is observable
through the public constructors and through canonically serialized valid-shaped
records; no digest or unknown-predicate bypass is needed.

Make missing/added exact rows observable while retaining the intended lone
strict-subset shrinkage behavior. Alternatively, if nested same-kind groups are
actually impossible present debt, establish that invariant from the real
producer and enforce it in the model constructor and decoded records. An
unstated producer assumption is insufficient for this public closed reader.
Add the two directions above beside existing lone-subset, superset growth and
unchanged controls; current fixtures only compare zero or one group per kind
and cannot expose the defect.
