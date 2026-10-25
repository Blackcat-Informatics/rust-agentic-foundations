---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-migration-verification
subject: cap:census-migration#verification
inputs:
  - kind: section
    name: cap:census-migration#verification
    digest: ca4b107729ca618482df2092d023859f8bf05e81fec5ec73c740210f398cb854
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::cli
    digest: e626c39680d666be90a896b4c375a53f03246ec631e8ea2c6ed03e80f1d8fbc7
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::command
    digest: 7a0f2abbe1a9c06e5ba8e8fcb647bf04fec6dcc48a5b8246bea41fb6259210ab
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::main
    digest: 2fb90057d93a446e1e090746290c950208015de693e49d48ce534b4eb0fb2fe1
  - kind: symbol
    name: raf_census::MAX_INDEX_BYTES
    digest: 305a3ed3655f4e199b01edfd1a6f46d556489f2adf354a8689889b8380c16449
  - kind: symbol
    name: raf_census::Migration
    digest: a4b1700f9a59e69ba8b39c160bb236149fedd0f144060174483147a9ca8db047
  - kind: symbol
    name: raf_census::MigrationInput
    digest: 987b7c19088c8d8a1e31e5a7f6e107733f4162fd1aa6f7a2f945a92915c0703c
  - kind: symbol
    name: raf_census::UnmappedRow
    digest: d88de92b0beefbf7e1559f7bbc139e4066bc0292bae5970c6e596d424bc0d599
  - kind: symbol
    name: raf_census::gate::migrate
    digest: 2b70a834b943bb3ca2864f48118686e0146daa889095b76c0239b505916d5627
  - kind: symbol
    name: raf_census::migrate_index
    digest: 81cda6ca66752c2b1fe050b23fb780d1c79f76342e9e02b374c049f19106534a
  - kind: symbol
    name: raf_census::migrate_legacy
    digest: 6573cb1650969c6de315e33ac1350290fc9ab1dff3f09de3fbc6282b7e530120
verdict: accepted
by: agent
---
