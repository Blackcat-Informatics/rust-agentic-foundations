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
- **`raf-workplan`:** loads a repository's capability, change and architecture
  documents through a `raf-source` `Tree` into typed values with their bodies,
  validating each with `raf-okf`. `Workplan::load` never fails and never panics:
  every problem is a `Finding` with a stable kind, a file, a location, a message
  and a fix. `Finding::KINDS` lists every kind, with one `okf-<rule>` kind for each
  rule of the profile, one `okf-parse-<error>` kind for each way a document fails
  to read, and the kinds of the flat document directories (`wrong-directory`,
  `nested-directory`, `stray-markdown`) and of unreadable input
  (`unreadable-file`, `unreadable-directory`). A file the tree refuses is reported
  by name and never skipped. The load also builds one `Graph` of the documents and
  the source they bind to: capabilities, their seven sections, changes, PROPOSED
  blocks, and the symbols, tests and examples the bindings resolve to, joined by
  binding, `depends`, `affects`, `changes` and section-granular link edges. The
  queries `code_of`, `capabilities_of`, `tests_for`, `sections_of` and
  `linking_sections` return typed, sorted values. The rules between documents are
  the kinds `depends-missing`, `changes-missing`, `moved-missing` and
  `depends-cycle`. The rules of PROPOSED blocks and states are the kinds
  `proposed-missing-change`, `proposed-change-not-listing`,
  `proposed-section-not-affected`, `state-considered-no-proposal`,
  `state-designed-proposal`, `state-delivered-open-record`,
  `state-delivered-proposal` and `no-open-record`. Links are judged against a
  caller-supplied `LinkContext` of tracked paths and tracked symlinks (or the
  files of the tree) as `link-broken`, `anchor-broken`, `link-outside` and
  `link-symlink`, over inline, reference, image and HTML `href` and `src`
  links, with anchors by GitHub rules. `plain::check` applies the same rules to
  tracked Markdown outside the document directories. `refs` holds the one
  pattern set for issue and pull request numbers and process wording, which
  `check` reports as `issue-reference` and the repository gates call.
  Each entry of `bindings` is resolved through the source index once, when the
  graph is built, and the result is kept: `Graph::bindings_of` returns the
  bindings of a capability with their sites or their resolve error, and
  `Binding::example_kind` says whether a bound example is an example target
  crate root or a documentation test item (`ExampleKind`). `check` reports the
  kinds `binding-unresolved`, `binding-ambiguous`, `binding-wrong-role` and
  `binding-macro-generated`. The roles are: `code` is a shipping item, `tests` is
  a `#[test]` function, `examples` is an example target's crate root or a
  documented item of a library whose documentation tests run; a binding that
  resolves to several definitions fits only when each one does, and the role of
  a re-exported item is judged on its definition. `binding-macro-generated` is
  reported when the path does not resolve and an item-level macro invocation of
  its parent module holds the name, so the symbol may be generated, which the
  index cannot see.
- **`raf-okf` rule and error tags:** `Rule::ALL` and `ErrorKind::ALL` list the
  payload-free `RuleTag` and `ErrorTag` of every variant, with `Rule::kind`,
  `ErrorKind::tag` and a kebab-case `name`, so a caller can name and map every rule
  and every reader error. Both tag enums are exhaustive, and an in-crate test keeps
  each list equal to the variants it names. The change is additive.
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
  stacks of their own; parsing runs on a dedicated stack; and committed fuzz
  targets cover the parser and a whole load of a workspace built in memory.
- **Public API of `raf-source` and `raf-okf`:** the data structs a caller reads
  (`Package`, `Target`, `SourceFile`, `Reach`, `Entry`, `SpanFact`, beside `Item`)
  are `#[non_exhaustive]`, so a later field is not a breaking change, and a `Tree`
  backend builds a listing entry with `Entry::new`. `Target` carries the test
  settings of its manifest table (`harness`, `required_features`, `test`,
  `doctest`) with cargo's defaults. `SpanFact` gives its lines as a 1-based
  half-open `lines` range, as `raf_okf::Span` does. The per-file facts the index
  is built from stay inside the crate; callers read items through the `Workspace`
  accessors. `LoadOptions` is `Clone` and not `Copy`. `raf_okf::Outcome` and
  `raf_okf::LinkTarget` are exhaustive, since the profile closes both sets.
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
- **`raf-workplan` roadmap:** `Workplan::status` returns one `Roadmap`, computed
  from the graph: every capability grouped by state in the order `delivered`,
  `implementing`, `designed`, `considered`, each group sorted by slug. An entry
  carries the id, slug, title, description, state, document path, the link
  target beside `docs/capabilities/index.md`, the ids of the changes that list
  the capability, and for an `implementing` capability its `Gap`s: a binding
  list that is empty, or an entry that does not resolve. `Roadmap::planned` is a
  view over the `designed` and `considered` groups, not a fifth state.
- **`raf-workplan` generated blocks:** `Workplan::sync` computes the whole new
  text of every stale or missing file as a `SyncReport`, writing nothing, and
  `Workplan::write` puts a report on a disk through a temporary file and a
  rename, refusing any target that is not an existing regular file or the new
  `docs/capabilities/index.md`. Three kinds of block are generated:
  `verification-matrix` (one row per `tests` and `examples` binding, sorted by
  path, with the sections that name it), `relationships` (depends on, depended
  on by, changed by) and `catalogue` (a table per state in the order delivered,
  implementing, designed, considered, with the gaps of an implementing
  capability). `check` judges staleness with the same renderers, so a clean
  `check` is a sync with nothing to write. New finding kinds: `index-missing`,
  `generated-stale`, `generated-unknown-kind`, `generated-misplaced`,
  `index-hand-written` (which replaces `okf-parse-index-text`) and
  `mixed-line-endings`. A block is written in the line ending of the body it
  sits in; a body that mixes CRLF and LF is refused. Table cell text is escaped
  by one fixed rule. The `docs_sync` example and `make docs-sync` run a sync,
  with `--check` for a read-only comparison.
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
