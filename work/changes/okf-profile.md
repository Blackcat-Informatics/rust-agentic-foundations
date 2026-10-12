---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:okf-profile
title: "Implement the raf-okf crate"
description: "Delivers the OKF parser, profile validation, body model and deterministic writer."
changes: [cap:okf-profile]
affects:
  - cap:okf-profile#contract
  - cap:okf-profile#usage
  - cap:okf-profile#examples
  - cap:okf-profile#verification
  - cap:okf-profile#constraints
  - cap:okf-profile#explanation
contract_effect: changes
items:
  - claim: "Documents parse into frontmatter and a body model with positioned errors."
    outcome: implemented
  - claim: "Profile validation enforces the closed key sets and typed values and reports every violation."
    outcome: implemented
  - claim: "The body model exposes sections, links, fenced code, generated blocks and PROPOSED blocks."
    outcome: implemented
  - claim: "The writer is deterministic and round-trips documents."
    outcome: implemented
  - claim: "The capability index is accepted only when it holds a generated block."
    outcome: implemented
  - claim: "Parsing is bounded in text size and nesting depth, and YAML anchors and aliases are refused."
    outcome: implemented
---

## Summary

This change delivers the raf-okf crate: the parser, the closed-key profile validation, the body model and the round-tripping writer.
