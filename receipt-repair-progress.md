<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

Addressed the general security-review finding in signed commit 2b8af84. The original mutable-reference alias case was reproduced as false acceptance; the original failure is retained. Further controls cover mutable input parameters, native type aliases, zero-argument borrowed returns, and a nominal Holder carrying a mutable reference.

The final fix requires every admitted native call receipt in returned argument/state lineage. Only direct tracked mutable arguments propagate state updates. It makes no nominal-field or native-signature purity inference. Unused native constructors/reads and hidden-reference updates whose receipts do not reach the result refuse. Valid direct state updates, immutable result forwarding and a Holder read remain admitted.

Fresh qualification: 26 library +75 Gate cases passed; all-target Clippy passed; the actual pinned public Hasher witness passed frozen empty/nonempty outputs and exact locked API home; independent final implementation review passed. Native capability completion passed. All normal signed commit hooks passed after refreshing every affected assessment. No hook or lint was bypassed.

The final full make check has passed workspace tests, including the two frozen-repository report comparisons, and is running the remaining gates. Earlier failures remain recorded. Hosted checks on the repaired head are pending; this is not a merge-ready claim. Parents #5/#53 and full CLI/self-gate scope remain open.
