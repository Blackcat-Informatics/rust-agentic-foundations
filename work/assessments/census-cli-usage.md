---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-usage
subject: cap:census-cli#usage
inputs:
  - kind: section
    name: cap:census-cli#usage
    digest: c7645a0ad34a833f203ef8895f0a3f81b326b0987b60d548b422c367c95de152
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: 60b8cdc17ed3bdb40d606342905e47d5d908574b087bfdf40b0b36f705f552ad
  - kind: symbol
    name: raf_census::Gate::self_test
    digest: f2c6b0abfe9c026c6f57fcd57bc2e4c943f71a5cb8b494d5979cff3a06f4dd86
  - kind: symbol
    name: raf_census::capnp_history::persistence::admission::check
    digest: d48ff88f11f33c01b6de01d3b4c282da8041ff6e2c67a586921ab8f7227720db
  - kind: symbol
    name: raf_census::capnp_history::persistence::check
    digest: 20abd512768570c0fe638d0fca777ddf497d865220ec053dab561253a8f1eae0
  - kind: symbol
    name: raf_census::capnp_history::reader_evidence::verify
    digest: 7cac3d238e4d3f80bea5edbe07e4480741c1271cee171b295fb4cf1b7f91a063
verdict: accepted
by: agent
---
