---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-debt-ratchet-purpose
subject: cap:census-debt-ratchet#purpose
inputs:
  - kind: section
    name: cap:census-debt-ratchet#purpose
    digest: 5f5d5fb3eadd427da8212f89d7aad840477a619cefe63e37b4dccf5a9e1d7f57
  - kind: symbol
    name: raf_census::DEBT_FILE
    digest: 12ca19be3fbb9901282949df239635101deb59bd25e6d5092aa5492478c3ccbb
  - kind: symbol
    name: raf_census::DebtChange
    digest: 5ec2d49585bb7ffe0e43f5ba14109b50be5328b0faa26d205ed86bf7c654b401
  - kind: symbol
    name: raf_census::DebtGroup
    digest: b16eab50e56b334db21110a7dae4b03deeaa05d27c34d865df68e4e250a7ab02
  - kind: symbol
    name: raf_census::DebtRecord
    digest: 91dce952687750f24f1f44268a1e7b9883c1aa101c761e01320d24adf4fc1499
  - kind: symbol
    name: raf_census::DebtWrite
    digest: 23838e774604de525795b02470dbcd4dc179623554465557392a18083691a949
  - kind: symbol
    name: raf_census::Gate::write_debt
    digest: f4a32723c385e0b6d8f93300d2ffeed37f60ae31edd4cd6ed9e1c52d9c8aa8ff
  - kind: symbol
    name: raf_census::Gate::write_debt_locked
    digest: dc35508f1201a7297986e1a6b820a562c70419ee871967514bc7b56a110546e0
  - kind: symbol
    name: raf_census::MAX_DEBT_BYTES
    digest: 5ce8868952094a1684f8a526cd41cfb66b70204d6d5bb54e49f4c12c32932dcc
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_locked
    digest: 3863d52464d8a048031a09a1ad8d8964af5139a92c1f8f647238791853e42085
  - kind: symbol
    name: raf_census::gate::Gate::write_debt_with
    digest: a68f678e95cb06a373b066db37f1b49c9dcc5f146b711949502b60a9db13c06d
  - kind: symbol
    name: raf_census::gate::read_debt
    digest: 043bd1e79273fd2194672f8ef5cfaf6301301ebbfec4acc4df539879b9831ce5
  - kind: symbol
    name: raf_census::publication
    digest: 4499c8a8fa5113f559b29dcb8b6ee5664996115a2409962b17f1ec77b9d0a44c
  - kind: symbol
    name: raf_census::report::debt
    digest: 85048764f37539ad3878cc51c6ec0eb1bb872ae1e90815a5b7e5722ab6ef8bdf
verdict: accepted
by: agent
---
