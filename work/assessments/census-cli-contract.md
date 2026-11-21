---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-contract
subject: cap:census-cli#contract
inputs:
  - kind: section
    name: cap:census-cli#contract
    digest: 87725699968998dcd7be324bc56c1e957b2d17d5deae59dd4981e39c9dc4b2c3
  - kind: symbol
    name: cargo_census::run
    digest: b32049f8a2619291edf0a2a43585c53654afca013fc0074ed2898c73a9c661fd
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
