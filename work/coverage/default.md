---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Coverage
id: coverage:default
scope: [cargo-census, cargo-workplan, raf-census, raf-okf, raf-rdf, raf-source, raf-source-fuzz, raf-workplan, rust-agentic-foundations]
exclusions:
  - item: "raf_fixtures::assessed"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::assessed::*"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::files"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::files::*"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::fixture_signing"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::fixture_signing::*"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::git_repo"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::git_repo::*"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::layout"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::layout::*"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::registry"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::registry::*"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::strict"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::strict::*"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::workspace_input"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
  - item: "raf_fixtures::workspace_input::*"
    reason: "Shared test support of several packages; it states no capability, and each helper is exercised by the tests that use it."
---
