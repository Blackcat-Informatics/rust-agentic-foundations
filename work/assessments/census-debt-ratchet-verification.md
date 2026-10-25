---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-debt-ratchet-verification
subject: cap:census-debt-ratchet#verification
inputs:
  - kind: section
    name: cap:census-debt-ratchet#verification
    digest: af0ecd017b22810c8bfd55e17a5e45498d4ab7354c73cfbb4660e4560df03b74
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
    digest: be38dfe6daad23a2cdf534867eae6ab53966df5715b18237b04e448425c31197
  - kind: symbol
    name: raf_census::Gate::write_debt_locked
    digest: d01da1d771e21d042943639206396b96659b1e5e2ea800f46ad69a6301673ecd
  - kind: symbol
    name: raf_census::MAX_DEBT_BYTES
    digest: 5ce8868952094a1684f8a526cd41cfb66b70204d6d5bb54e49f4c12c32932dcc
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_locked
    digest: 688afacdc763d3ae080198584332f69930b76c283af9c083a4bcdd735018a879
  - kind: symbol
    name: raf_census::gate::Gate::write_debt_with
    digest: bb89f2c575c9aa16274282771640412d102a142951ade3aa649e53590ca8e5f4
  - kind: symbol
    name: raf_census::gate::lock_publication
    digest: 984745f9f3e6dbc911f2162c77b5505cde733909211d02a37ab4a49eb9bf307f
  - kind: symbol
    name: raf_census::gate::read_debt
    digest: aef43f39e457fa70cc720c3049dfe8209e435c18a69a48b9cea973f339536c46
  - kind: symbol
    name: raf_census::gate::replace_record
    digest: e2d2da938498e165a79e596c8c7e5971ffb6b8c4d30647bd03b2a2fb6de79ad2
  - kind: symbol
    name: raf_census::report::debt
    digest: a7377b5482153e9e57ae82c9eedc44d5a16695b7d944f1fef8854c3643cc0bc2
verdict: accepted
by: agent
---
