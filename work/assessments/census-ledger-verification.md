---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-ledger-verification
subject: cap:census-ledger#verification
inputs:
  - kind: section
    name: cap:census-ledger#verification
    digest: 2af5f8bd23947b0b8e427be1798e1e06ecdbecee92240fc113b083802fed6bd7
  - kind: symbol
    name: raf_census::BenchLink
    digest: 0cfb32481d48d75a47a6aa9abb72aa76a9a48fa4a0e495b3a8fdf364da325853
  - kind: symbol
    name: raf_census::BoundClassification
    digest: fb0402bd1164583f60458fb3dad86986e6871e34e1fad7807309c63e4168497f
  - kind: symbol
    name: raf_census::Gate
    digest: 08a60f29fc0ccf17588f3c080c0e876738a4bdcb768d6314b5ab231381b1deb9
  - kind: symbol
    name: raf_census::Gate::assert_clean
    digest: 45cf504b0f3a9b33934d1581934a738fd056a845e17d60f3751a2b4d7c5e8a25
  - kind: symbol
    name: raf_census::Gate::check
    digest: 6842c6d0e43e130b46d2d4a920a36aceed6066507c15fd04b5522c0f229b2d74
  - kind: symbol
    name: raf_census::Gate::check_locked
    digest: ce909f37a05708cfef5daaa76233bc63be371354d4a6dbd1d7e41807648ec1ad
  - kind: symbol
    name: raf_census::Gate::check_tree
    digest: 7f0be8102776e749f22c8f133a2d80ac348d61c57ac3da79befbd42a1dae1009
  - kind: symbol
    name: raf_census::Gate::extensions
    digest: f4325ff16655a45eb8216b2658b9c48879346d2934b729c2f1d4922854c1acf1
  - kind: symbol
    name: raf_census::Gate::root
    digest: 33267fbb7491b78a5b2d9c396f48befa48ab7a2eed51b24f687c72a9e55ed19d
  - kind: symbol
    name: raf_census::Gate::rules
    digest: d32147afa2dc624555778ac873967a55756ee85a2100657e2751d3981c7cf051
  - kind: symbol
    name: raf_census::Gate::workspace
    digest: 0503ad24db1afab83d45d098775da0f6d1304ee95c9a905293ccd91d1b73bbd5
  - kind: symbol
    name: raf_census::Gate::write_debt_locked
    digest: dc35508f1201a7297986e1a6b820a562c70419ee871967514bc7b56a110546e0
  - kind: symbol
    name: raf_census::GateReport
    digest: d8bfe3d580285225c7aba0461f6b7554bbd8e5441843ae111dfa6ef808d3b58a
  - kind: symbol
    name: raf_census::GateReport::numeric_bounds
    digest: b112f8fc7f364d687e1d810ac34b6f3a5ac1f1644819ef6ad88a08d4db634904
  - kind: symbol
    name: raf_census::LEDGER_FILE
    digest: 13d0555b019cbfab0a0fb80e1a7d761d1adda926a620d160fa26e28390592aba
  - kind: symbol
    name: raf_census::Ledger
    digest: a829022b54891ec5201afa5db36a4dd012fea1e7ad1a705a05cd8bd0ae5a74f9
  - kind: symbol
    name: raf_census::LedgerDistinct
    digest: 17457f169b7261f1ddddabbf2a133314281331a298f382ee1d392ad9313475af
  - kind: symbol
    name: raf_census::LedgerExtension
    digest: 3cf51559c3301b4f81086b1935d12e26605ab2750fb46631aa68aab70a340652
  - kind: symbol
    name: raf_census::LedgerExtensionGraph
    digest: bb829e6f568456dd6a5be2efc1bbb11079625c5c56c5d24f0dd652f6a424fefe
  - kind: symbol
    name: raf_census::LedgerJob
    digest: 7a4f3a55759b1e0b149f4a81ab750d881516d2a828361ec66a817bfa55b35914
  - kind: symbol
    name: raf_census::LedgerPattern
    digest: c13696ca7ffc38d1ec42c017a63c8ac3cb6f62d983242d1a739a817b54770d75
  - kind: symbol
    name: raf_census::LedgerVariant
    digest: 6cff865dc28265fb4c7899974210c0a4f56598b76c1f42071b0cc512501c0ed3
  - kind: symbol
    name: raf_census::MAX_LEDGER_BYTES
    digest: 1f3a488b63a4eb88837d386174987fe204ede06ba0579342dc41b49a052c61e8
  - kind: symbol
    name: raf_census::NamePattern
    digest: 0c66e2a102cab95c5ca79473885359fa986216c057da61d0deebaaae9d6b93ac
  - kind: symbol
    name: raf_census::NumericBound
    digest: 7c34fed66afb06aab1a3b11cf3481ce6bd5b6809ec9addd1b005164d404ea8a5
  - kind: symbol
    name: raf_census::PatternMatcher
    digest: 4cea265cd00ec9016763b79d7e0b4db2e88f02c8f55d1cc6a21fc019a4766006
  - kind: symbol
    name: raf_census::PatternScope
    digest: 63f37e805fb303bce4b3800fe8be4728de43d732614ee66f7efde42884660f97
  - kind: symbol
    name: raf_census::PreferredProvider
    digest: 7b658a1516d9e8a3ea7af76b94bc51ec1f8cb8b60f3c07775644f93c90ccec3c
  - kind: symbol
    name: raf_census::SYMBOL_ANCHOR_PREFIX
    digest: 356e10fa9aa88605cafbb5e03b520149b50997e534cefb98e254898cdf5f38ab
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_locked
    digest: 3863d52464d8a048031a09a1ad8d8964af5139a92c1f8f647238791853e42085
  - kind: symbol
    name: raf_census::gate::evaluate
    digest: fc7d24ee648fd771eff2ebcadffcdf2009c73332290265e8caae5d73818c5d4b
  - kind: symbol
    name: raf_census::gate::owners
    digest: 42c47c5cb754e9d6284eb089c8ee9aa14bfaa04b41194c50f9416bcbd4b4a34c
  - kind: symbol
    name: raf_census::gate::providers
    digest: dffe549462a6285f2361bdd9496add31c823e1d9237cbe28fb9f100c0e5328d2
  - kind: symbol
    name: raf_census::report::ledger
    digest: 15a6b16c63b703b41ff0f6d37f77c62a3ca18e0eca54738ee8498736cb0fd493
verdict: accepted
by: agent
---
