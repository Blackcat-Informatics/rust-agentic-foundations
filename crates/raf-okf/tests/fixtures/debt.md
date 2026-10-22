---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Debt
id: debt:default
findings:
  - kind: coverage-item-unbound
    subject: raf_okf::Actor
  - kind: strict-claim-uncovered
    subject: "cap:okf-profile#contract/the-four-states"
lines:
  - item: raf_okf::body::Body
    orphans: 12
  - item: raf_okf::Number
    orphans: 4084
---
