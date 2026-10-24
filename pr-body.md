<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

`Gate::check_locked` now resolves preferred providers through admitted locked source identities and proves that each supported consumer returns native computation. Every native call has a required receipt in returned lineage, rejecting discarded updates, stale snapshots, dead state, hidden-reference updates and parallel local computation; detector exemptions contain only proved definitions, so unlisted competitors still fail.

Configured binary, test, bench and example consumers retain ordinary library routes, custom library names, renamed dependencies and development eligibility. The documentation states the finite adapter grammar and its refusal boundaries.

Validation: 26 library tests and 75 Gate tests passed; affected all-target Clippy, the runnable provider example, and the pinned public PurRDF Hasher witness passed. The witness checks frozen empty and nonempty output bytes and an exact locked API home. Independent source review passed. Native capability completion and normal signed commit hooks passed, including the security-review repair in 2b8af84. The repair has fresh 101-case runtime, Clippy and pinned public Hasher receipts plus independent review. The final full make check passed (terminal0, raw/provider-make-check-receipts-final.txt). Fresh hosted Linux/macOS verification and final review capture remain required before merge. Windows hosted runtime remains unverified under the maintainer's explicit scoped CI exception.

Closes #61. Parents #5 and #53 remain open for their full detector, native-adoption and CLI/self-gate contracts.
