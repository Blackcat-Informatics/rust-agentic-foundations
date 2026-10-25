---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-ledger-usage
subject: cap:census-ledger#usage
inputs:
  - kind: section
    name: cap:census-ledger#usage
    digest: 6eac258591c49b9bad5f5f973a9cebbbfb0111d7450048ed3095a4b5b2344020
  - kind: symbol
    name: raf_census::Gate::check_locked
    digest: 2cb2ad081afa7034b3a8956b78b2fe67939b2e9ae71d0949365630d2de396e64
  - kind: symbol
    name: raf_census::Gate::write_debt_locked
    digest: d01da1d771e21d042943639206396b96659b1e5e2ea800f46ad69a6301673ecd
  - kind: symbol
    name: raf_census::LedgerExtension
    digest: 3cf51559c3301b4f81086b1935d12e26605ab2750fb46631aa68aab70a340652
  - kind: symbol
    name: raf_census::LedgerExtensionGraph
    digest: bb829e6f568456dd6a5be2efc1bbb11079625c5c56c5d24f0dd652f6a424fefe
  - kind: symbol
    name: raf_census::PreferredProvider
    digest: 7b658a1516d9e8a3ea7af76b94bc51ec1f8cb8b60f3c07775644f93c90ccec3c
verdict: accepted
by: agent
---
