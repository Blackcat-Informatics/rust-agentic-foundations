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
    digest: ca78a7ba2c56ebd1ee2eb9109204ba8b3e7e642d6fd2235afe1b29557389e1c3
  - kind: symbol
    name: cargo_census::bin_crate::cargo_census::command
    digest: 1bd0dfaa59389ccb912bf782383a3f574a3a33e839ad69b1dad3e421ea82b513
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
    digest: a63eaa52a7d3e339b59bcb84d24b2c183da7948c50dad982450d412a309d71db
  - kind: symbol
    name: raf_census::migrate_index
    digest: 81cda6ca66752c2b1fe050b23fb780d1c79f76342e9e02b374c049f19106534a
  - kind: symbol
    name: raf_census::migrate_legacy
    digest: 31de85dfedebfacd06b44df458eff5c82c89308d823e11c59475eb0f787bddfc
verdict: accepted
by: agent
---
