---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-debt-ratchet-examples
subject: cap:census-debt-ratchet#examples
inputs:
  - kind: section
    name: cap:census-debt-ratchet#examples
    digest: 2fbe556fde3e42dccb6ccd2b4fe4a752656f462d886aff7fbf6b3b7536cf8067
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
    digest: b68a166904ee1e38e2d7d1a55f8dbc740b217f9d53a1bb9c02ffaec6d0c00829
  - kind: symbol
    name: raf_census::Gate::write_debt_locked
    digest: d01da1d771e21d042943639206396b96659b1e5e2ea800f46ad69a6301673ecd
  - kind: symbol
    name: raf_census::MAX_DEBT_BYTES
    digest: 5ce8868952094a1684f8a526cd41cfb66b70204d6d5bb54e49f4c12c32932dcc
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_locked
    digest: a3f9ac545b0cc43dee0c848622f4fe3d4db4c836ad7d4d09914305f97b450e2f
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
    digest: 4df18d709c1961c032c659f38841e1a91fb61215611915c204761a312b2ea472
verdict: accepted
by: agent
---
