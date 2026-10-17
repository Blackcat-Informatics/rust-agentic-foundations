---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Evidence
id: evidence:gate-kills-a-test-past-its-timeout
subject: binding:raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran
inputs:
  - kind: symbol
    name: raf_workplan::bound
    digest: 73190dd9606a84d4f7588c60a5d4303ed27e1655f5cf574603666bc66bb6160b
  - kind: symbol
    name: raf_workplan::bound::Evidence
    digest: 2ce7c1dd4af05cd0e9372dcc53421b8ea3d84a82c2afc32df0b29e2963c6ee7d
  - kind: symbol
    name: raf_workplan::bound::Executor
    digest: fb60f3294ddb18cc2419096842b25f9150af9ff572c6fb766d261b35fc0363f1
  - kind: symbol
    name: raf_workplan::exec
    digest: 6dd17043f9e0fe6c938ba893a6aabc11da2717673eb704a3265b88b2c7c4da0a
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
    digest: b6e803b757ffe5d95ec2ab53edbff0cc94b9928e174478bc9569cf492e258531
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle
    digest: 5052cb15c7581ba404dc8f734c673d90ed0aa8f114fef5a39d43bdce0faa9197
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle::complete
    digest: 427eea7c60b9164c17b7aac716d4e3779ac31bed23c8e5bd7a2ce76b3cb63f51
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
