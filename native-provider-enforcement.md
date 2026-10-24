---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:native-provider-enforcement
title: "Ground preferred-provider policy in native source identities"
description: "Resolve admitted provider APIs and prove returned native computation across every physical consumer reach."
changes: [cap:census-ledger, cap:census-detectors]
affects:
  - cap:census-ledger#contract
  - cap:census-ledger#usage
  - cap:census-ledger#examples
  - cap:census-ledger#constraints
  - cap:census-ledger#verification
  - cap:census-detectors#contract
  - cap:census-detectors#usage
  - cap:census-detectors#examples
  - cap:census-detectors#verification
contract_effect: changes
items:
  - claim: "Mutable references and borrowed update results never erase native mutation lineage."
    outcome: implemented
    reason: "Every native call emits an admitted required receipt; hidden input, alias and holder updates cannot disappear from returned lineage. Direct mutable calls and immutable result forwarding retain valid neighbours."
  - claim: "Preferred-provider consumers return actual admitted native results; mutation-event lineage rejects stale snapshots, dead state, decorative calls and parallel local computation."
    outcome: implemented
    reason: "Returned-value and mutation-event proof has paired refusal controls, and the pinned public Hasher returns distinct frozen empty and nonempty outputs."
  - claim: "Every source identity of a physical consumer body receives its own configured route, body and declaration-header proof."
    outcome: implemented
    reason: "Every physical reach is visited in its own configured context; selected target controls pass and an ordinary shared reach cannot inherit a development route."
  - claim: "Provider homes reuse validated public API receipts and require one exact definition across consumer contexts, including APIs with no consumer Census unit."
    outcome: implemented
    reason: "Validated public API homes retain exact defining sites and reject cross-consumer disagreement, unrelated owners and unproved descendants."
  - claim: "Selected detectors report unlisted competing code in the adapter package; home and entry-point exemptions contain only fully proved identities."
    outcome: implemented
    reason: "Provider rows remove package exemptions and detector controls report unlisted competing functions and non-function items beside proved native consumers."
  - claim: "Usage, examples, limitations and bound positive, refusal and neighbouring valid cases state the supported native provider contract."
    outcome: implemented
    reason: "The capability records the locked entry point, finite adapter grammar, actual targets and explicit unsupported proof boundaries with bound test controls."
---

## Summary

The native source graph supplies exact provider and consumer identities. A finite
adapter proof requires the returned value to come from admitted native APIs and
preserves every required native state update. Detector findings cover unlisted
competing code without granting package-wide or module-wide exemptions.
