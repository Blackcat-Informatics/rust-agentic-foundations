---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-debt-ratchet-usage
subject: cap:census-debt-ratchet#usage
inputs:
  - kind: section
    name: cap:census-debt-ratchet#usage
    digest: a3a7e069da380b6b040c8e234cf07e1396a0190947b87a6ae191337e97d9eb1e
  - kind: symbol
    name: raf_census::DEBT_FILE
    digest: 12ca19be3fbb9901282949df239635101deb59bd25e6d5092aa5492478c3ccbb
  - kind: symbol
    name: raf_census::DebtChange
    digest: c672df768f71e79c4099102184c49eee372a82ce8df7112aeead2182332d84f4
  - kind: symbol
    name: raf_census::DebtGroup
    digest: b16eab50e56b334db21110a7dae4b03deeaa05d27c34d865df68e4e250a7ab02
  - kind: symbol
    name: raf_census::DebtRecord
    digest: fbf2786e19ab5605840e7158f3ca8df072e1609d4c2a5fb872ba4ca53510764f
  - kind: symbol
    name: raf_census::DebtWrite
    digest: 23838e774604de525795b02470dbcd4dc179623554465557392a18083691a949
  - kind: symbol
    name: raf_census::Gate::write_debt
    digest: f4a32723c385e0b6d8f93300d2ffeed37f60ae31edd4cd6ed9e1c52d9c8aa8ff
  - kind: symbol
    name: raf_census::Gate::write_debt_locked
    digest: d01da1d771e21d042943639206396b96659b1e5e2ea800f46ad69a6301673ecd
  - kind: symbol
    name: raf_census::MAX_DEBT_BYTES
    digest: 5ce8868952094a1684f8a526cd41cfb66b70204d6d5bb54e49f4c12c32932dcc
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_locked
    digest: d6094a34be80b07cd33928aa689b34ef8cf6e8e8396d61a13ae26318b3d48da7
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
    digest: 4df18d709c1961c032c659f38841e1a91fb61215611915c204761a312b2ea472
verdict: accepted
by: agent
---
