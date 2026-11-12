---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-verification
subject: cap:census-cli#verification
inputs:
  - kind: section
    name: cap:census-cli#verification
    digest: fa0d5a875ad79b4fab9ffe49efd6fa7f1a6299c024c3be8ca5088ff030f0f45e
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::cli
    digest: f35d44214c2ea51c71605e99affb2c601d87072a1d416d1918766b683aeaca10
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::command
    digest: 1cd77e23237a908e2b173f5e68dd8b133fc63393d125ea02475b00db735fd826
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::error
    digest: 5c18f105079fccd0601e8d1b0eb0620c3361511e1c07d81945bd54f062974ae2
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: 2fb90057d93a446e1e090746290c950208015de693e49d48ce534b4eb0fb2fe1
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
    digest: efc94d08846e404a1c7bf1c4ae162f0b16eaf78faf8ab0f6763bdc0e230fdcbc
  - kind: symbol
    name: raf_census::selftest
    digest: a32746ffdc747429786bdf6227fecf5bd5d0813fd7293a401dafde4b26e2b882
verdict: accepted
by: agent
---
