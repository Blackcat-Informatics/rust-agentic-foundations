<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Changelog

All notable changes to RAF are recorded here. The workspace ships one version
across all crates. Until 1.0.0 a breaking change bumps the minor version.

## [Unreleased]

- **`raf-okf`:** reads and writes the RAF profile of OKF documents. It parses
  Markdown with YAML frontmatter in a closed subset, validates the closed key sets
  and typed values of capability, change and architecture documents and reports
  every violation with its file, rule and position, models the body (sections,
  links, fenced code, generated and PROPOSED blocks) and rewrites spans in it, and
  writes documents back deterministically so a parsed document renders to its own
  bytes. Input is bounded in size and nesting, anchors and aliases are refused, and
  the crate reads no file.
- **`raf-source`:** indexes Rust source for other tools. A `Tree` trait with disk
  and memory backends gives every path confinement to the workspace root; package
  and target discovery finds library, binary, test, bench, example and build
  targets; the module tree follows `mod`, `#[path]` and `cfg_attr` paths; and the
  symbol and use-alias index resolves a path through renames, globs and re-exports
  with typed failures. Files are size-capped and told apart by identity, so a file
  is read and parsed once whatever spellings reach it. A token pre-scan bounds
  recursion, with one count summed over every open delimiter level that keeps an
  admitted file within half of the parse stack; the tokens of each file and the bytes a load reads, keeps in its index
  and keeps as facts are budgeted, with limits a caller can set; the walks keep
  stacks of their own; parsing runs on a dedicated stack; and a committed fuzz
  target covers the parser.
- **Workspace scaffold:** a Cargo workspace on edition 2024 with a floating
  nightly development toolchain and a minimum supported Rust version of 1.98.1,
  shared workspace lints (missing documentation, unreachable `pub` and unsafe
  code denied, clippy pedantic), formatting and clippy settings, and a
  supply-chain policy for `cargo deny`.
- **Licensing:** first-party code is offered under `MIT OR Apache-2.0 OR
  MulanPSL-2.0` and every Markdown document under CC-BY-4.0, with the licence
  texts, a Chinese-language rendering of the licensing terms, and REUSE
  metadata checked by `reuse lint`.
- **Gates:** `make check` runs formatting, clippy, tests, release-mode tests of
  `raf-source`, documentation, `cargo deny`, REUSE, the minimum-version build, the
  fuzz harness checks and a bounded fuzz run, a `cargo check` for the Apple and
  Windows targets, the licence digests and the issue-reference gate. A pre-commit
  hook runs the fast subset on a checkout of the staged index, and a CI workflow
  runs the same gates.
- **Repository gates:** `tests/repo_gates.rs` checks the Mulan licence digests
  and the Chinese-rendering fingerprint, scans every tracked file for issue and
  pull request references and process wording, and requires `CITATION.cff` and
  the workspace package to carry one version. The gates are written in Rust, share
  one pattern set and file list with the document contract, and each rule has a
  negative fixture. `make licenses` and `make issue-refs` select them.
- **CI:** tool versions are pinned, a concurrency group cancels superseded pull
  request runs, Dependabot keeps the pinned actions current, and a
  `pre-merge-commit` hook runs the fast gates on merge commits that skip
  pre-commit.
- **Project documents:** an agent guide, contributing guide, code of conduct,
  security policy, issue and pull request templates, citation metadata and a
  release-notes configuration driven by conventional commits.
- **Brand and README:** the RAF brand guide, logo, social preview image and a
  README with the crate table and dev-dependency snippets.
- **Capability specifications:** 21 capability documents covering the census
  (normal form, fingerprints, ledger, detectors, rule API, patterns, debt
  ratchet, reference scan, command line, migration), the source index, the OKF
  profile and the workplan (document model, reconcile, completion gate, two-way
  coverage, evidence claims, entity history, generated sections, prose
  assessment, current rationale, command line), plus two architecture pages: the
  normal form and the RAF profile of OKF, which defines the seven body
  sections.
- **Document contract:** `tests/docs_contract.rs` runs in `cargo test` and
  enforces the OKF profile on every capability, change and architecture
  document: closed key sets, values, the seven sections, markers, links,
  bindings by state, and the exact capability inventory. It shares its pattern
  set and file list with the repository gates, and each rule has a negative
  fixture.
- **Security:** `SECURITY.md` states the trust model (the checked workspace is
  trusted build input) and the hardened contracts for parsing and subprocesses:
  containment, bounds, no-shell spawning and validated arguments. The
  capability specifications carry the same contracts.
- **Release notes:** each changelog entry is printed once per release.
- **Planning essay:** the original planning essay is retired; its content now
  lives in the capability documents and architecture pages.
