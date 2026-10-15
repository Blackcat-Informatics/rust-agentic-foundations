# Issue #8: raf-workplan change lifecycle + cargo-workplan CLI

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-workplan, crate:cargo-workplan

## Body

## Context
This issue turns "update docs" from a final checkbox into a gate. A change names the documentation it affects when it is planned. `reconcile` produces a precise, reviewable obligation list that humans or agents resolve by writing prose. `complete` refuses until the change has been incorporated into the current account of the system, and then **deletes the change record**, leaving its history in Git and off the reading path.

Capabilities delivered: `cap:workplan-reconcile`, `cap:workplan-complete-gate`, `cap:workplan-cli`.

## Scope
- [ ] `change new <slug> --capability cap:x` scaffolds the change record and a PROPOSED block
- [ ] `change impact <slug>` lists the affected capabilities, sections, bindings and dependents
- [ ] `change reconcile <slug>` produces the obligation list:
  - unresolved proposals;
  - items without an outcome;
  - missing or broken bindings;
  - affected sections not touched since the merge base (via the git CLI);
  - stale generated blocks;
  - future-tense or proposal text in a `delivered` capability.

  Each obligation names the evidence needed to resolve it.
- [ ] `change complete <slug>` requires zero obligations, every item resolved, no PROPOSED blocks for the change, `docs sync` clean, and bound tests passing (`cargo test` filtered to the bound tests). It then deletes the change file. A `contract_effect: none` change records that and carries lighter obligations.
- [ ] `verify [<slug>]` runs bound tests and examples and reports pass/fail (no persisted evidence in M1)
- [ ] `docs preview <slug>` renders the capability as it will read once reconciled
- [ ] `cargo-workplan` binary (`clap` derive) exposing `check`, `status`, `docs sync|preview`, `change new|impact|reconcile|complete`, `verify`; exit codes 0/1/2

## Acceptance criteria
- [ ] Each obligation kind has a fixture that triggers it, and `complete` refuses in each case
- [ ] A happy-path fixture completes and the change file is gone
- [ ] CLI integration tests
- [ ] Capability docs reconciled

## Depends on
- #7

## Comments (0)

