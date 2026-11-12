---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-debt-ratchet-usage
subject: cap:census-debt-ratchet#usage
inputs:
  - kind: section
    name: cap:census-debt-ratchet#usage
    digest: 52eeb0249838190a883d76d2f80817d899620c93b0840cf5e766ee85bc257278
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
    digest: 8c1d011ff76066b8b0f503eff79bfe5e4deab230e3f3fa6f083b9dffee1db68b
  - kind: symbol
    name: raf_census::MAX_DEBT_BYTES
    digest: 5ce8868952094a1684f8a526cd41cfb66b70204d6d5bb54e49f4c12c32932dcc
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_locked
    digest: e048a8b0a3686a75ef4e03b34b7eae37e3893aaa64ae7b90f3d419182005aca8
  - kind: symbol
    name: raf_census::gate::Gate::write_debt_with
    digest: e61384d4586824a813469ff86685cce377e4b618378a964d6ac60d00792a54df
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
