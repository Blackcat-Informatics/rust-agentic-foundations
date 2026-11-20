---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-usage
subject: cap:census-cli#usage
inputs:
  - kind: section
    name: cap:census-cli#usage
    digest: cf206a98f4a810c6b4db09492e4e1b80eb548fbff2e194cbd26c3854be350c59
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: 60b8cdc17ed3bdb40d606342905e47d5d908574b087bfdf40b0b36f705f552ad
  - kind: symbol
    name: raf_census::Gate::self_test
    digest: f2c6b0abfe9c026c6f57fcd57bc2e4c943f71a5cb8b494d5979cff3a06f4dd86
  - kind: symbol
    name: raf_census::capnp_history::persistence::check
    digest: 3edb7199f2c8b9d3fd72bfba5b88e8c9592b27ad0052ada0042bf5a261f49d3f
  - kind: symbol
    name: raf_census::capnp_history::reader_evidence::verify
    digest: 650cd8b557b83363a5a92e3e06fe0b2433f5e3801b972e7d056e73a1579b189c
verdict: accepted
by: agent
---
