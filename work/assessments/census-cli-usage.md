---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-usage
subject: cap:census-cli#usage
inputs:
  - kind: section
    name: cap:census-cli#usage
    digest: 5c89ce826c38969ba10e31906fd9c8431b727917e95f7045af5d3512876550a8
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: 60b8cdc17ed3bdb40d606342905e47d5d908574b087bfdf40b0b36f705f552ad
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
  - kind: symbol
    name: raf_census::capnp_history::reader_evidence::verify
    digest: 7cac3d238e4d3f80bea5edbe07e4480741c1271cee171b295fb4cf1b7f91a063
verdict: accepted
by: agent
---
