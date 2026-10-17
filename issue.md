# Issue #9: Acceptance scenarios A & B + workplan self-gate

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: crate:raf-workplan, dogfood

## Body

## Context
These are the crate's two flagship acceptance tests, taken straight from the planning essay. Once they pass, the workplan gate is turned on for this repo and milestone 1's own change records are completed **with the tool itself**.

## Scope
### Scenario A: several redesigns, one readable present
- [ ] Fixture git repo, built in a temp dir: one capability redesigned 4 times through successive changes (decisions amended and reversed, interfaces and constraints evolving), modelled on the "Amended YYYY-MM-DD" ADR pattern
- [ ] At the end:
  - the capability states the active contract directly;
  - no PROPOSED blocks or change files remain on the reading path;
  - all links resolve;
  - the current rationale is retained;
  - every earlier state can be recovered with `git show`.

### Scenario B: planning produces actual docs
- [ ] The fixture capability starts `considered`, with open questions; the implementation renames the API and drops one guarantee
- [ ] `complete` refuses until Usage, Examples and the bindings match the real API and the dropped claim has an outcome
- [ ] It then succeeds, leaving the capability `delivered` with a real usage example, current API references, explicit limitations and a test-traceability table

### Dogfood
- [ ] `tests/self_workplan.rs` runs `raf_workplan::Gate` on this repo, wired into `make check` and the pre-commit hook
- [ ] Advance this repo's capability states with `cargo workplan status` and `change complete` for every M1 change record

## Acceptance criteria
- [ ] Both scenarios pass as integration tests
- [ ] Self-workplan and self-census gates both pass in CI
- [ ] Every M1 capability is `delivered`, with resolved bindings and a generated verification matrix

## Depends on
- #6
- #8

## Comments (0)

