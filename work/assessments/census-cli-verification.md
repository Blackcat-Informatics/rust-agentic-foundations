---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-cli-verification
subject: cap:census-cli#verification
inputs:
  - kind: section
    name: cap:census-cli#verification
    digest: 1b491c7de8f455d03bafcc5e0a5433fca6e12e6e1aff4a3bae317e01b0383575
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: 60b8cdc17ed3bdb40d606342905e47d5d908574b087bfdf40b0b36f705f552ad
  - kind: symbol
    name: cargo_census::cli
    digest: 4023381cf1bfb714064091a2bac7086b5c3ce3a9a5b7dda292a125b0a9821638
  - kind: symbol
    name: cargo_census::command
    digest: f4858184ebce29565146e4cc232e8842f313cc421154793810801539e73e1329
  - kind: symbol
    name: cargo_census::error
    digest: d368c2653a7adfc685940b91a52e1ca7e946b508577849e120a7628a9dfa0f96
  - kind: symbol
    name: cargo_census::run
    digest: b32049f8a2619291edf0a2a43585c53654afca013fc0074ed2898c73a9c661fd
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
    name: raf_census::Gate::validate_tree_registrations
    digest: fb06b726fbf4cea861e4451b33c4baceb547f779b313b6ff9228baf3592914ca
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
    digest: fb2953459e033ae232258e09b7d49475b5a1cd37cc91298912e0106c4600008e
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
    name: raf_census::capnp_history
    digest: 784b5e120681131530f045f5c17255f285925668ab3f1ef739f4185f705b5b5b
  - kind: symbol
    name: raf_census::capnp_history::persistence::admission::check
    digest: 7e675e082a6fea8c001bf90aec9f0a25e415b68e2b68c4a20643e90b158eaee6
  - kind: symbol
    name: raf_census::capnp_history::persistence::check
    digest: 2aea35e5b521469c9193a2fc098d2016796854bed2a2e6fc6692a5dd8ab7d23d
  - kind: symbol
    name: raf_census::capnp_history::persistence::migration::check
    digest: 84f5897595215f0580e430f73fc966829cb20b6a3e95674a59b6604e3c8aa13a
  - kind: symbol
    name: raf_census::capnp_history::reader_evidence::verify
    digest: 7cac3d238e4d3f80bea5edbe07e4480741c1271cee171b295fb4cf1b7f91a063
  - kind: symbol
    name: raf_census::registered
    digest: 1c778e25726466d2e2dbece74fb0689f90331c3ac16c3d73d2bec07df97dd9a1
  - kind: symbol
    name: raf_census::selftest
    digest: 9557611b2160cfef1e2c87e3bfcbc232d7b92d8954077b7502ec54f9010b9967
verdict: accepted
by: agent
---
