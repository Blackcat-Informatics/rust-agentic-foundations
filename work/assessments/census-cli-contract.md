---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-contract
subject: cap:census-cli#contract
inputs:
  - kind: section
    name: cap:census-cli#contract
    digest: 8df3694a949f89a57353a581b78fda2cad435e66711b7ed1ae3f81caafbff569
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
    digest: 20abd512768570c0fe638d0fca777ddf497d865220ec053dab561253a8f1eae0
verdict: accepted
by: agent
---
