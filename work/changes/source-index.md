---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "The public surface of a target in the source index"
description: "Adds Workspace::exported_items, which lists the items another crate can name through a library's or a binary's crate root, each under its exported path and with its defining item."
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#usage
  - cap:source-index#examples
  - cap:source-index#constraints
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "Workspace::exported_items lists the items of a library or binary target that a chain of pub declarations reaches from its crate root, through pub modules and pub use re-exports, named, renamed or glob, each with its defining item, kind, file and span, in one order in every load; a module that several public paths reach is listed under each and its contents are read once, under the first path in breadth-first order."
  - claim: "An item is exported only through a route of pub declarations back to a pub definition: a pub item of a private module only through a pub re-export, and never a pub(crate), pub(super), pub(in path) or private item, an item behind a re-export that is not pub, or test code."
  - claim: "The members of an exported type or trait are listed under its path: each member of a trait and each pub member of an inherent impl of a type; trait impl members, enum variants and struct fields are not items of the surface."
  - claim: "A public re-export that names nothing the target defines as an item of the index, of another crate of the workspace, of a crate outside it, of an enum variant or of a path the index cannot follow, is a Reexport and not an item, also where a pub glob carries it from a module it reaches."
  - claim: "Items and re-exports that doc(hidden) hides, on the item, on a module of the path or on the use declaration, are left out by default and flagged when ExportOptions includes them."
  - claim: "A public name that is ambiguous, goes through a cycle of imports or reads a glob that names nothing fails the listing with the resolver's error, a target that is not a library or a binary is refused, and what the listing holds and the work of deciding carried re-exports are charged to the item-bytes budget left after the load."
---

## Summary

The index gains a query for the public surface of a target: the items another
crate can name through its crate root, read from the settled scope tables. A
coverage check from code to documentation needs that surface, because a re-export
makes the exported path differ from the defining path and an item declared `pub`
inside a private module is public only when a re-export reaches it. Each listed
item carries the path it is exported under and the item that defines it, so a
re-export under another name maps back to its definition.
