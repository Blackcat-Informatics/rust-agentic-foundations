---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-contract
subject: cap:census-cli#contract
inputs:
  - kind: section
    name: cap:census-cli#contract
    digest: 5c3a19a2b76e6af12a76dd6bfec550c935a177b67727733db52fd9ca7cc95009
  - kind: symbol
    name: cargo_census::run
    digest: b32049f8a2619291edf0a2a43585c53654afca013fc0074ed2898c73a9c661fd
  - kind: symbol
    name: raf_census::Gate::self_test
    digest: f2c6b0abfe9c026c6f57fcd57bc2e4c943f71a5cb8b494d5979cff3a06f4dd86
  - kind: symbol
    name: raf_census::capnp_history::persistence::admission::check
    digest: d48ff88f11f33c01b6de01d3b4c282da8041ff6e2c67a586921ab8f7227720db
  - kind: symbol
    name: raf_census::capnp_history::persistence::check
    digest: 2aea35e5b521469c9193a2fc098d2016796854bed2a2e6fc6692a5dd8ab7d23d
verdict: accepted
by: agent
---
