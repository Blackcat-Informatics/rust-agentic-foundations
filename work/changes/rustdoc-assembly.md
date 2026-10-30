---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:rustdoc-assembly
title: Assemble checked Cargo and Rustdoc documentation
description: Admit actual compiler configuration, generated inputs, public API links and packaged multi-crate documentation.
changes: [cap:workplan-rustdoc]
affects:
  - cap:workplan-rustdoc#purpose
  - cap:workplan-rustdoc#contract
  - cap:workplan-rustdoc#usage
  - cap:workplan-rustdoc#examples
  - cap:workplan-rustdoc#constraints
  - cap:workplan-rustdoc#explanation
  - cap:workplan-rustdoc#verification
contract_effect: changes
items:
  - claim: "Crate and module include fragments retain checked source, package and version identities and compiler-resolved public API links, with explicit private, hidden and unresolved-target refusals."
  - claim: "Explicit Cargo documentation assembly respects selected packages, features, configuration, target directories and cumulative output, admitting required artifacts and links before publication."
  - claim: "Native Linux host examples verify long labels, nested-page and non-root URL navigation, source/API/capability/evidence links and coherent multi-crate publication."
  - claim: "Packaged static fragments work without a build-time generator; package and host examples verify the stated docs.rs path, versioned explorer links and actual asset/script constraints without publishing a release or changing hosting."
  - claim: "Required missing assets, unresolved links, unsupported configurations, changed source observations and changed owned output fail actionably while unrelated host files remain intact."
---

## Summary

Cargo and Rustdoc documentation assembly consumes shared named bundles. Actual
Rustdoc configuration and generated source inputs require observed compiler
receipts; metadata feature names and caller declarations do not replace them.
Checked source identities, compiler-resolved API links, admitted HTML artifacts
and package membership jointly govern publication. Static host output supplies
the common presentation and owned versioned files.
