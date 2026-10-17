---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Evidence
id: evidence:gate-kills-a-test-past-its-timeout
subject: binding:raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran
inputs:
  - kind: symbol
    name: raf_workplan::bound
    digest: 7dbc473fe74c817dcfa95ba2e9bd7b7017713e99480e563f2a3bd2e0a74bd500
  - kind: symbol
    name: raf_workplan::bound::Evidence
    digest: 2ce7c1dd4af05cd0e9372dcc53421b8ea3d84a82c2afc32df0b29e2963c6ee7d
  - kind: symbol
    name: raf_workplan::bound::Executor
    digest: fb60f3294ddb18cc2419096842b25f9150af9ff572c6fb766d261b35fc0363f1
  - kind: symbol
    name: raf_workplan::exec
    digest: 78d5c91922e2f407c03ff3d9e635096b996409fdd099bce2eba6f9f5e26536ff
  - kind: symbol
    name: raf_workplan::exec::Invocation
    digest: 541e0874a2cbff738b60b6c3762db18dc00952583480afab0cac3e52a7a4e65d
  - kind: symbol
    name: raf_workplan::exec::Limits
    digest: 5aa3fa838885868be1b202101d540a731f95ea25a8eb4cde1add45228ce426f6
  - kind: symbol
    name: raf_workplan::exec::Pattern
    digest: c0f6e6be93e7af51fe5c54d2a9cfb7c0af7b63280bcdad991d1931fe2b9ba49d
  - kind: symbol
    name: raf_workplan::exec::Programs
    digest: c2edea8efeb050a404c24889417638d15615445ab2d05bfaa7ef5d15a6bf32d4
  - kind: symbol
    name: raf_workplan::exec::SystemRunner
    digest: 0d4918eeaf321c913728a349dc72f74d0960bd33bef73a9741e2ac61d1a6f75b
  - kind: symbol
    name: raf_workplan::lifecycle::Completion
    digest: 572045a122c8d1d65c2827eff81140cb20f11a649f38baebbc1f2ce7bba835bc
  - kind: symbol
    name: raf_workplan::lifecycle::Condition
    digest: 843d0ed57ee69d0c115736fd2bc989b9e6821fd759bbb0542e459c2fd5f4b1fe
  - kind: symbol
    name: raf_workplan::lifecycle::Failure
    digest: b25dee026b61b4f30b925108798d77f28776568e45c676b2210cc4f5da4a6595
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle
    digest: 5052cb15c7581ba404dc8f734c673d90ed0aa8f114fef5a39d43bdce0faa9197
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle::complete
    digest: 4d1826b8934e16e66583f001eac26d31388339760ac3dd2312ee592ffdc933c0
  - kind: symbol
    name: raf_workplan::lifecycle::VerifyReport
    digest: d6211d239cfbe515c1b0c969fea1fd1df1930ad29cb0fc1c9e0e3abfd4ecbd55
  - kind: symbol
    name: raf_workplan::lifecycle::verify
    digest: 173351a58ff3ce22a6f7d55994dde9eeab7a3d2e0220180821a3cc8d71f96edf
verdict: pass
by: agent
for: [cap:workplan-complete-gate]
---
