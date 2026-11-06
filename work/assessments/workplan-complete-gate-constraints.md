---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:workplan-complete-gate-constraints
subject: cap:workplan-complete-gate#constraints
inputs:
  - kind: section
    name: cap:workplan-complete-gate#constraints
    digest: b17ea08b62b1116759325dc92d88ff3c9bb88a9a8d1c906237ea07752aab1b17
  - kind: symbol
    name: raf_workplan::Gate::open
    digest: fc9150c625e88fd61248c6aed26b5f2e8e82b05690aced1d031a790071e4aa06
  - kind: symbol
    name: raf_workplan::Gate::tracked_list
    digest: 5227aab9da4cc93e1849155bbf1760fc6914a1ba43a677690f14270cbe20ebc8
  - kind: symbol
    name: raf_workplan::Opened::workplan_after
    digest: 5178bc0715f78c9211ed26735550f8a7d12951c58343e4208ef445e4ee8081a8
  - kind: symbol
    name: raf_workplan::git::head
    digest: f813006d08fdc863f53d2f16c021ced90ad81f1d91049d41b64193ce4d38d8ed
  - kind: symbol
    name: raf_workplan::lifecycle::Completion
    digest: 273c799147a4fbea6819362c4067b0121ef3431cf8e2674b1e82dc4f64c1f3f7
  - kind: symbol
    name: raf_workplan::lifecycle::Condition
    digest: c5c4eec2826817b4fa1cc212d48c0242c161124366d2cd0ac6b769cc5ed08442
  - kind: symbol
    name: raf_workplan::lifecycle::Failure
    digest: 7ffbdeb3d10cc7270b8a3333125059d96dbfec4402e109a23c9bf5de6824283a
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle
    digest: 541f21e3839f540ca72552675c5291ff81b648a8d516dd93960eee8d315e02c0
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle::complete
    digest: faceacdb4f25c1b77b9ab113dade2727361cf17cac17ee6b90da7d4fdc4ed224
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle::open_with
    digest: e64152626f7dd7cf67de6b8a0daca6e7711ad7fedc19b954c49bac923d5087e5
  - kind: symbol
    name: raf_workplan::lifecycle::VerifyReport
    digest: 3ba94893622450a3f35f0b82650df4fbf45e1b4a39397588763898501824b545
  - kind: symbol
    name: raf_workplan::lifecycle::verify
    digest: c253fe384ffe89088584214824e3faa37351e919d2286e4d8fcbade0ac79285a
  - kind: symbol
    name: raf_workplan::staged
    digest: 8db8e448245038687cbfedc1c338578ebbb351f1f27a998c6520ac8facd7d95f
verdict: accepted
by: agent
---
