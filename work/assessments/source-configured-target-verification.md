---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:source-configured-target-verification
subject: cap:source-configured-target#verification
inputs:
  - kind: section
    name: cap:source-configured-target#verification
    digest: bb98fe18e3811436cbb313e32193a83c805c96e5a12d6f5d87472d4bcd52e1af
  - kind: symbol
    name: raf_source::Workspace::load_configured_target
    digest: 4d8063c4437aca30f71c041c0623f88ae9e69890db2a1cb0a67f32901ee34a03
  - kind: symbol
    name: raf_source::Workspace::par_load_configured_target
    digest: d7af4d436b31eb446f697f39b2bb053d9664f83ad4417c7a1f216eb6569fa246
verdict: accepted
by: agent
---
