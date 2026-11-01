---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-glob-cost
title: Bound incremental glob propagation and its instruction cost
description: Qualify incremental settling, queue and graph budgets, reference equivalence and native instruction growth.
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#usage
  - cap:source-index#examples
  - cap:source-index#constraints
  - cap:source-index#explanation
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "Glob propagation uses monotone per-view and per-read state, passes independent arrivals as deltas and retains exact companion, visibility and blocked-name semantics."
  - claim: "Counted operations cover rereads, copied lists, drops, lookup routes, queue attempts, subscriptions and undecided-import graph work, with exact budget success and one-step-short refusal."
  - claim: "Native allocation admission bounds retained import reads, queue growth, subscriptions and graph scratch under the original grants."
  - claim: "Reference-mode and incremental loads return equal probe and exported-surface answers over both differential seed families and the committed corpus, including thirteen declaration orders for the full comparison."
  - claim: "The nine admitted shapes have paired release-build instruction counts at 100, 200, 400 and 800 elements against the fixed resolver baseline; every count includes complete counter coverage."
  - claim: "At 800 elements each of the nine shapes costs at most 1.5 times its baseline; doubling from 400 to 800 costs at most 2.15 times for linear shapes, 3.4 times for the prelude, 4.3 times for either hub and 4.4 times for lookup fan-in."
  - claim: "The holder-scan diagnostic has paired complete-counter measurements at 100 and 200 elements, a typed FileTokens refusal at 400 and a typed TooLarge refusal at 800 under unchanged default limits. Refused inputs supply no numerical performance verdict."
  - claim: "Read-only real sibling-workspace trials compare the same inputs and answers; the registry retains its admitted loads without new budget errors and its largest work count stays below 2^27 or the default is explicitly reconsidered."
  - claim: "The capability states the actual counted steps and shape growth, including companion-path costs; instruction evidence establishes the cost contract without wall-clock measurements."
---

## Summary

Incremental propagation must preserve import identity and visibility while
bounding storage, counted work and native instruction growth. All cost and
equivalence claims remain open until their actual comparisons pass. No numerical
performance guarantee follows from a correctness result or a typed budget refusal.

The admitted instruction populations are chain, many-items, public fan-in,
private fan-in, companion fan-in, prelude, hub, ambiguous hub and lookup fan-in.
The holder-scan generator exceeds the ordinary per-file limits at larger sizes;
its refusals bound admitted inputs rather than establish instruction growth.
