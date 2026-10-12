### Stage 2 fix R5 (`df65ee4`): project-document consistency

**Gaps closed (G15b–d, A1)**
- CHANGELOG `[Unreleased]` now covers everything the tree delivers:
  - brand and README;
  - the 21 capability specs and 2 architecture pages;
  - the document contract;
  - the Rust repository gates;
  - the security contracts;
  - CI pins and hooks;
  - the changelog dedup;
  - the retired essay.
- AGENTS.md §6 names `tests/docs_contract.rs` as the format's enforcement today.
- AGENTS.md and CONTRIBUTING.md describe both hooks and Dependabot.
- The PR template's SPDX header now uses the two-line shape.

**Checks run:** every R5 grep; an SPDX-shape scan of every tracked Markdown file; `cargo test --workspace` (112 + 40); reuse 81/81; `git cliff --unreleased` rc 0; the hook passed.

**Review:** `G-R5-review` gave PASS. Its advisory about the hook description is fixed.

Every remediation group has now landed. The final full `make check` runs next.
