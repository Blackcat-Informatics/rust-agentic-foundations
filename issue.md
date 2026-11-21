# Issue #385: okf: validate closed change-item keys with native SHACL

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-okf

## Body

Validate the keys of each authored Change item map with native SHACL Core.
The map requires `claim` and permits `outcome` and `reason`. Reuse the
prepared header validator and bounded native dataset.

- Give each reached map a focus derived from its list position. Keep
  separate findings for separate items, including items with identical
  contents. Bound each focus and its keys before native graph allocation.
- Project only the maps before the existing first non-map refusal. Preserve
  the existing wrong-list and wrong-map diagnostics.
- Preserve individual unknown-key findings in author order and one
  `ItemClaimEmpty` finding for a missing claim. Preserve its location and
  order relative to the existing claim, outcome and reason value checks.
- Exercise a closed-map violation, a missing claim, distinct occurrences,
  an items list without maps and a valid neighbor through the profile.
  Preserve the canonical authoring round trip.

Parent #41 retains the other nested shapes, value constraints, repository
graph validation and final profile reconciliation.


## Comments (0)

