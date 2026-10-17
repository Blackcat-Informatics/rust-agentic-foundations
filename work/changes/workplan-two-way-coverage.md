---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-two-way-coverage
title: "The path a re-export names, and no gap cascade from an unloaded capability"
description: "The path an unresolved re-export names is read from the module that declares it, super:: included, and check reports no coverage gap while a capability document does not load."
changes: [cap:workplan-two-way-coverage]
affects:
  - cap:workplan-two-way-coverage#contract
  - cap:workplan-two-way-coverage#verification
contract_effect: changes
items:
  - claim: "The path an unresolved re-export names is read from the module that declares it, the innermost inline module or else the module of its file, with each super:: one module up, so a binding of the module it reaches covers it and a binding of the declaring module the path leaves does not."
  - claim: "While a capability document does not load, check reports its own findings and no coverage-item-unbound, coverage-test-unbound or coverage-reexport-unbound, and still reports an exclusion that matches nothing."
---

## Summary

Coverage read the path an unresolved re-export names from the module of its
file and kept a leading `super::` as written, so `pub use super::gen::Made;`
in `src/api.rs` named `made::api::super::gen::Made`: a binding of `made::gen`
did not cover it, and a binding of `made::api` did. The path is now read from
the module that declares it, an inline module included, with each `super::`
one module up. A capability document that did not load made `check` report
every item it binds as bound by no capability; while one does not load,
`check` now reports its own findings and no coverage gap.
