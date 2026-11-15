---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-debt-ratchet-purpose
subject: cap:census-debt-ratchet#purpose
inputs:
  - kind: section
    name: cap:census-debt-ratchet#purpose
    digest: 88750728eb7bb310bf4b3655e8a4b3510b98b913394bad566a4428bec0cbb139
  - kind: symbol
    name: raf_census::DEBT_FILE
    digest: 12ca19be3fbb9901282949df239635101deb59bd25e6d5092aa5492478c3ccbb
  - kind: symbol
    name: raf_census::DebtChange
    digest: 234e40d31b7ef05eae41c1ea032a35ae3ca54a944209049c94057f20c963df71
  - kind: symbol
    name: raf_census::DebtGroup
    digest: b16eab50e56b334db21110a7dae4b03deeaa05d27c34d865df68e4e250a7ab02
  - kind: symbol
    name: raf_census::DebtRecord
    digest: de999c9a2d120bd2d5ff977f879d3de6205666529a24e344ca5260c5e6599400
  - kind: symbol
    name: raf_census::DebtSite
    digest: b00c2f6a42bcc192594cffbdca6e9442ee651ad4a5e0fb7475234ee193b1b714
  - kind: symbol
    name: raf_census::DebtWrite
    digest: 736c7a493c75f57eb0dd662cc5f18568b181043c13c0592961ebb13fa84bbe90
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
    name: raf_census::SiteKey
    digest: 1de89ed80944c05d8deb51b94d07b08da613a475900dc1d20a2f1f68b74ec5bd
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_locked
    digest: 3863d52464d8a048031a09a1ad8d8964af5139a92c1f8f647238791853e42085
  - kind: symbol
    name: raf_census::gate::Gate::write_debt_with
    digest: cc3320f0acdf6398ea6ecb2e75be7a907ae578cfd63c2023370041617718c93c
  - kind: symbol
    name: raf_census::gate::ratchet
    digest: 92df3862bd4275e01a5c8e16ac6c0b0b2da0127f33f2c2c8de01cd60890bc5d0
  - kind: symbol
    name: raf_census::gate::read_debt
    digest: 043bd1e79273fd2194672f8ef5cfaf6301301ebbfec4acc4df539879b9831ce5
  - kind: symbol
    name: raf_census::publication
    digest: 559709229d7bd1f7d03508be644c5b4d3b09b5eed011ab53129d17f34da24683
  - kind: symbol
    name: raf_census::report::debt
    digest: b47ed9bc4b1f1f59a53431f6413cdf341edfc885302c4f6beac37808bcbea28c
verdict: accepted
by: agent
---
