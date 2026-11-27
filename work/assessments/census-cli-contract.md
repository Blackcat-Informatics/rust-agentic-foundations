---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-contract
subject: cap:census-cli#contract
inputs:
  - kind: section
    name: cap:census-cli#contract
    digest: f0bb5376b8299210b6010335d40658f999a692bd9c15231c5873d9132bc716e3
  - kind: symbol
    name: cargo_raf_census::run
    digest: fdd3e38b79ad9b8a860af3e8586d2b875f58ca9fe4cfec50ea43654c50b5f52d
  - kind: symbol
    name: raf_census::Gate::init_ledger
    digest: cb109237dc9ea01dda2751059871fb0d33ef7202daf14ea05359890e35a52adc
  - kind: symbol
    name: raf_census::Gate::self_test
    digest: f2c6b0abfe9c026c6f57fcd57bc2e4c943f71a5cb8b494d5979cff3a06f4dd86
  - kind: symbol
    name: raf_census::capnp_history::persistence::admission::check
    digest: 7e675e082a6fea8c001bf90aec9f0a25e415b68e2b68c4a20643e90b158eaee6
  - kind: symbol
    name: raf_census::capnp_history::persistence::check
    digest: 2aea35e5b521469c9193a2fc098d2016796854bed2a2e6fc6692a5dd8ab7d23d
  - kind: symbol
    name: raf_census::capnp_history::persistence::migration::check
    digest: 84f5897595215f0580e430f73fc966829cb20b6a3e95674a59b6604e3c8aa13a
verdict: accepted
by: agent
---
