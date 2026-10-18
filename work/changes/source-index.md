---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "Model the visibility rules of rustc for globs in the source index"
description: "The source index settles a re-export no wider than the binding it reads and refuses a name that globs bring in as different items, and resolves a path from a module."
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#usage
  - cap:source-index#constraints
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "A named pub use or a glob that reads a binding through a glob its module cannot access keeps the narrower of its own visibility and that of the binding, measured on the ancestor chain of the importing module."
  - claim: "A name that two globs bring in as different items is ambiguous in that module and visible to every reader that can access at least one of the items, who gets Ambiguous listing every candidate."
  - claim: "Workspace::resolve_from and resolve_from_in resolve a path as a use written in a module would, and ResolveError::UnknownModule names a module that is not in the index."
  - claim: "An exported_items listing that meets the ambiguity fails with ExportError::Resolve, and an item reached only through a capped re-export is not listed."
  - claim: "The resolution of the index is compared with rustc on generated programs, and the divergences that remain are stated as limits."
  - claim: "No bound test of the crate reads the clock: the tests count work, and the Contract states the counted bounds."
---

## Summary

The source index settles two more visibility rules of rustc for globs, in every
order of the declarations, adds the resolution of a path from a reading module, and
is compared with rustc on generated programs. The Contract, Usage, Constraints and
Verification of the capability state the rules, the entry points, the limits that
remain and the tests that bind them, and the wall-clock figures of the Contract are
replaced by the counted bounds the tests assert.
