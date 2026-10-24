---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-debt-ratchet-usage
subject: cap:census-debt-ratchet#usage
inputs:
  - kind: section
    name: cap:census-debt-ratchet#usage
    digest: bcadc4e1dd99e5095026c30f6f53df204503e68b6664457df49e58941b8c5d66
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
    digest: bca356baebc9a084119adafb6886c02000d5f142e610e9d9a6a0feb88f1628ad
  - kind: symbol
    name: raf_census::Gate::write_debt_locked
    digest: 75368c373f300afd1a6f1143303d5c357b6480228ee2ad2b5382a3d74ae6f8c7
  - kind: symbol
    name: raf_census::MAX_DEBT_BYTES
    digest: 5ce8868952094a1684f8a526cd41cfb66b70204d6d5bb54e49f4c12c32932dcc
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_locked
    digest: 688afacdc763d3ae080198584332f69930b76c283af9c083a4bcdd735018a879
  - kind: symbol
    name: raf_census::gate::Gate::write_debt_with
    digest: 1de2dd3f22a7dd90050e51e813c1f14bfbe6a0a7d604a2e7207821c29c11f2bb
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
