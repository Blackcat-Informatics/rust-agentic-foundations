# Issue #14: raf-source: model rustc's visibility of re-exported private glob bindings and of ambiguity through inaccessible globs

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-source

## Body

## Context

`raf-source` resolves `use` paths with rustc's import-accessibility rule: a binding or glob that the importing module cannot access counts for nothing. Two further rustc visibility rules are not modelled yet, and the `cap:source-index` Contract lists them under Constraints:

1. **Re-exported private glob binding.** An explicit `pub use` of a binding that its module can reach only through a private glob is treated as visible as the import. rustc keeps it no more visible than the binding it re-exports, so a module that globs it can read a name rustc hides from it.
2. **Ambiguity through an inaccessible glob.** When the globs of a module bring a name in ambiguously, and one of the items comes through a glob the reading module cannot access, the index answers with the other item. rustc refuses the path, usually as E0659 and sometimes as E0573.

## Measured impact

In a rustc-oracle fuzz of generated programs with private globs, these two rules account for every remaining wrong answer on code rustc accepts: 12 queries in 10 of 5,522 programs. Eleven of the 12 were also wrong before the current resolver.

## Scope

- Model both rules in the settling of `use` paths, keeping resolution independent of declaration order.
- Add an every-order fixture for each rule.
- Re-run the rustc-oracle fuzz with private globs, with no new disagreements.
- Remove the two sentences from the Contract's Constraints.


## Comments (0)

