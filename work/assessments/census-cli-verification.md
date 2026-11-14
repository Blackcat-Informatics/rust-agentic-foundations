---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-verification
subject: cap:census-cli#verification
inputs:
  - kind: section
    name: cap:census-cli#verification
    digest: 1879bd40a18df457bd9fadfe1ad6efa6d55d32e5b5abb1de5f876f9af8ef98a8
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::cli
    digest: 3d709ad957aea9381996b393d87d0b78c527ead3761e6e053927ccdb9599a2c1
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::command
    digest: 021868d700fa17be31751cd1bd99bef7c9bc47bb4c2c912d7d9f2f3626324346
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::error
    digest: 331d8131efcf1a764775555b05c2124f98ed36e33df07eae42edb8cc197e4f56
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: a4871ea9cc74cb72b33d4d5a7e1651dae91ecbc96733d0f326a2f80bdefd4de2
  - kind: symbol
    name: raf_census::CHECK_PROTOCOL_VERSION
    digest: f15bea745a3732843a125a042c9bafb0eed33727b8ce1264739dd85e83950f0f
  - kind: symbol
    name: raf_census::DEBT_WRITE_PROTOCOL_VERSION
    digest: 040b9a659209a7f711df38dd99d36a6cfa1d7dd1bcc916218334eba9c5edf364
  - kind: symbol
    name: raf_census::Gate::self_test
    digest: f2c6b0abfe9c026c6f57fcd57bc2e4c943f71a5cb8b494d5979cff3a06f4dd86
  - kind: symbol
    name: raf_census::MAX_CHECK_RESPONSE_BYTES
    digest: c8a061c28be622be14d14f7667e80cf9fa3c2b5f3bc44a9dcbc0a5b5561caa0c
  - kind: symbol
    name: raf_census::MAX_INDEX_BYTES
    digest: efed7ed0974ed665e9f3871c20d5db193a9fed4bb3d1e065723002a833619423
  - kind: symbol
    name: raf_census::RegisteredCheck
    digest: 0cf0dafd2c191031bf6850d83d15f5b4a10b6c044d03d412a3038423cc7445ba
  - kind: symbol
    name: raf_census::RegisteredDebtWrite
    digest: 909a50042ab3a2fd43b76ada3ac34f463901895ae64a6baf0460403100ff6085
  - kind: symbol
    name: raf_census::RegisteredOperation
    digest: 98bcfe7fbfb82787e373bc668fe6ed676710da4e3558b9fd8822e7b7367fb640
  - kind: symbol
    name: raf_census::SelfTest
    digest: dac276ea90de583da4c32e7c1c6505d709d86f4ed5d191da69e4a0f824e0bd95
  - kind: symbol
    name: raf_census::SelfTestCase
    digest: 3807d56c2efb0da756205b73a5b5aebd2f3d9ab969df5644721d7f554374a880
  - kind: symbol
    name: raf_census::registered
    digest: 8a00ae8ab3f3c182586faebebe5c0daf1b4dd7368b7b4f0f75bece55ea4b4036
  - kind: symbol
    name: raf_census::selftest
    digest: a64d1bf9f0f15c51a0a6cc0d2146533530e4f9b46cb3f433c46dad397514c684
verdict: accepted
by: agent
---
