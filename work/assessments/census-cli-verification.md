---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-verification
subject: cap:census-cli#verification
inputs:
  - kind: section
    name: cap:census-cli#verification
    digest: 4e7270998c9fdb307416c51a633e1a5773a0fa656c86e7769565af7167302e46
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::cli
    digest: ee56e6a43b06a2345275feae8c27bda9f3cb4a7d4b1d420842be5d19ef170a84
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::command
    digest: 00be2a6a2a8d2a46aef93256d20116dc7d07a85835828fd91cd7d2afbde55124
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::error
    digest: 9c6652c4357da47724fbb7253f5c56be9ba95a3d6a0eab82bb65dde1c971755c
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: 2fb90057d93a446e1e090746290c950208015de693e49d48ce534b4eb0fb2fe1
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::registered
    digest: 5c31a8100be254cb0539aac78583b3bbf4f7393fab8a64c922fb1a1f2bd6adff
  - kind: symbol
    name: raf_census::CHECK_PROTOCOL_VERSION
    digest: f15bea745a3732843a125a042c9bafb0eed33727b8ce1264739dd85e83950f0f
  - kind: symbol
    name: raf_census::MAX_CHECK_RESPONSE_BYTES
    digest: c8a061c28be622be14d14f7667e80cf9fa3c2b5f3bc44a9dcbc0a5b5561caa0c
  - kind: symbol
    name: raf_census::RegisteredCheck
    digest: 0cf0dafd2c191031bf6850d83d15f5b4a10b6c044d03d412a3038423cc7445ba
  - kind: symbol
    name: raf_census::registered
    digest: f3d1486a09a28afb7d7df0ff9ff034396d3dc9b0db4a0cb27f0c5c6022bbb07d
verdict: accepted
by: agent
---
