---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-contract
subject: cap:census-cli#contract
inputs:
  - kind: section
    name: cap:census-cli#contract
    digest: cdb4bff79b9d2f550a2d8015e8418c69d20e5a44798c96a2b908d97ef1ed8784
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::cli
    digest: f61234902fabe75be1d689dde0cbeb04ad87c65572363e18e03ee979e393c47f
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::command
    digest: cb58d331365384fa9d8622760a577dd2e7c325492179809d926aa80b24c436d3
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::error
    digest: 9c6652c4357da47724fbb7253f5c56be9ba95a3d6a0eab82bb65dde1c971755c
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: 2fb90057d93a446e1e090746290c950208015de693e49d48ce534b4eb0fb2fe1
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::registered
    digest: d3b8361c67e06368afb9b6e849a7edb2d610ca72576fd537a4002dab80c58d8f
  - kind: symbol
    name: raf_census::CHECK_PROTOCOL_VERSION
    digest: f15bea745a3732843a125a042c9bafb0eed33727b8ce1264739dd85e83950f0f
  - kind: symbol
    name: raf_census::DEBT_WRITE_PROTOCOL_VERSION
    digest: 040b9a659209a7f711df38dd99d36a6cfa1d7dd1bcc916218334eba9c5edf364
  - kind: symbol
    name: raf_census::MAX_CHECK_RESPONSE_BYTES
    digest: c8a061c28be622be14d14f7667e80cf9fa3c2b5f3bc44a9dcbc0a5b5561caa0c
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
    name: raf_census::registered
    digest: efc94d08846e404a1c7bf1c4ae162f0b16eaf78faf8ab0f6763bdc0e230fdcbc
  - kind: symbol
    name: raf_workplan::exec::Invocation::example_debt_write
    digest: 7c7d9bfa387799e823f979d64a7fb231b2451c29c3bce58653a7b6b3360c0a58
  - kind: symbol
    name: raf_workplan::exec::Invocation::example_index
    digest: c6402860601816b8bdb5f1c243c38425b6e525df1b42e835b2646f29369be1e2
verdict: accepted
by: agent
---
