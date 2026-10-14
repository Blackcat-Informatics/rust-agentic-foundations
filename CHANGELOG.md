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
  `Gate::workspace(dir)` is the dev-dependency entry point: it loads the
  documents and the source workspace (roots `["."]`, or `source_roots`) of a
  checkout on disk and runs `check`; `assert_clean` panics listing every finding
  as `kind file:line message (fix)`. Links are judged against the paths
  `git ls-files` lists inside a git work tree, or against a list given to
  `tracked_paths`; a path that is a symlink on disk is a tracked symlink. A
  `git` that cannot run, fails, is told by `GIT_DIR` or `GIT_WORK_TREE` about
  a repository it cannot use, or cannot list the index is a `GateError`. The one
  fallback is taken when `git` itself answers that the directory is not inside a
  git work tree (a copy of the files such as a commit hook's snapshot): every
  file of the directory then stands for a tracked path, which makes the rule
  that a link to an untracked file is broken weaker.
  A missing `docs/capabilities`, or one with no capability document, is an error
  (`GateError`, `try_check`) and not a clean verdict; a missing `work/changes` is
  an empty set of changes. The repository's own document-contract test now loads
  its tracked files through `raf-workplan` and keeps only the repository's policy:
  the capability inventory (now with the generated `docs/capabilities/index.md`),
  tracked symlinks, Markdown-only document directories, no frontmatter on a page
  outside the document directories, and the equality of the Usage and Examples
  blocks with the crate doctests. `docs/capabilities/index.md`
  is created by `make docs-sync`.
- **`raf-workplan` preview:** `Workplan::preview(change)` returns, for each
  capability a change record lists (sorted by id), the whole document as it reads
  once that change is reconciled, as `Preview { capability, path, text }`. The
  slug is taken with or without the `change:` prefix. Each PROPOSED block of the
  change is replaced by its content: the marker line goes, and so does the quote
  marker (`>` and one space after it, if there is one) of each following line,
  after the block's own indentation, which stays; a tab after `>` is kept as
  written. The blocks of other changes and
  all text outside blocks are kept byte for byte, each line keeps its own line
  ending, and a listed capability with no block of the change is returned
  unchanged. The result is a pure function of the loaded documents and writes
  nothing. A `PreviewError` names an unknown slug (with the open changes), a
  change record or a listed capability that did not load (with its findings), a
  listed capability with no document, a line whose quote prefix cannot be removed
  unambiguously (with file and line, including a quote that goes on into a fenced
  code block, which ends a block in the document model), and a reconciled text
  that does not read back. No finding kind is added.
- **`raf-workplan` processes:** the `exec` module is the one place a `cargo` or
  `git` process is started. An `Invocation` is a `Program` and a list of `Arg`s: fixed
  text, or a value built through one of the patterns of the completion gate by hand
  (`<name>`; `<pkg>`, `<bin>`, `<test>` and `<example>`; `<item-path>`, which allows
  `Wrapper<T>::get`; the relative source path of the list output, which refuses empty,
  `.` and `..` segments), or a full Git reference name or commit id. A value that
  fails its pattern is a `PatternError` and never becomes an argument, so document
  text reaches a command line only after its pattern. The constructors of
  `Invocation` are the rows of the argv tables and root discovery. `Programs` names
  `cargo` (from `$CARGO`, else `cargo`), `git` and the working directory, and each can
  be replaced. `SystemRunner` starts the process directly with no shell, null standard
  input and the argv vector as given, in the working directory, with `LC_ALL=C` when
  its output is parsed and no other change to the environment. It captures standard
  output and standard error, each up to a bound (16 MiB, or 64 MiB for `cargo
  metadata`), and kills the process, and on Unix its process group, when the bound is
  passed or the wall-clock limit is (1800 s for a build or test run, 300 s for an
  example's run and for a query). Each case is a typed `ExecError` and none is a skip.
  `FakeRunner` answers invocations from a script in process, on every platform, applies
  the same output bound and time limit and records every call. The general
  constructor of `Invocation` and the constructor of `Literal`, the type of fixed
  argument text, are private to the crate, so only the table-row constructors build an
  invocation. The group is signalled directly with `rustix` (the `process` feature, the
  version `raf-source` already uses, Unix only) and not through a `kill` program on
  `PATH`; a kill that cannot be sent is an `ExecError::Io`. `rustix` is the one
  dependency `raf-workplan` has beyond the RAF crates and `proc-macro2`.
- **`raf-workplan` git, root discovery and one load path:** the `git` module asks
  `git` its questions through an `exec` runner, in the C locale and in the runner's
  directory. `require_version` reads `git --version` (vendor suffixes such as
  `2.45.1.windows.1` and `2.39.5 (Apple Git-154)` are tolerated), accepts 2.30 and
  newer and refuses 2.29 with a typed `GitError`. `tracked_paths` is the listing the
  `Gate` made inline, moved onto the runner with the same results: `None` only when
  `git` answers that the directory is not inside a work tree, and every other failure
  an error; it now also runs under the runner's limits (16 MiB per stream, 300 s), so a
  listing larger than that is an error. `default_branch_ref` reads the full reference name
  `git symbolic-ref refs/remotes/origin/HEAD` prints and, when it is unset, fails with
  an error that names `git remote set-head origin --auto`; `resolve_commit`,
  `merge_base` and `blob_at` follow the argv rows of the reconcile contract, and
  `blob_at` reads up to 1 MiB (the document bound), returns `None` for a path absent
  at the merge base, and fails on a larger file instead of cutting it. `exec` gains
  the two rows the tracked-path listing needs, `git rev-parse --is-inside-work-tree`
  and `git ls-files -z`. `root::discover` runs `cargo metadata --format-version 1
  --no-deps --frozen` through the runner (64 MiB bound) and reads only the
  top-level `workspace_root` string: a small scanner validates the whole text as JSON
  without a JSON dependency, and a text that is not JSON, nests deeper than 128
  levels, has the member twice or not at all, or holds a non-string or a relative path
  is an error; a `workspace_root` key nested deeper is not the field. Cargo does not
  run the `build.rustc`, `build.rustc-wrapper` or `build.rustc-workspace-wrapper`
  program of the workspace's configuration for this command, which a fixture shows.
  `Gate::open` returns an `Opened` repository (the disk tree, the source `Workspace`
  and the tracked paths); `Opened::workplan` loads the documents as they are on disk at
  the call, because a `Workplan` borrows the workspace and cannot be kept beside it.
  `Gate::programs` names the programs the gate starts, `try_check`, `check` and
  `assert_clean` run on `open` and are unchanged, and the Git version gate is not part
  of the gate. `Workplan::change` is the shared lookup of a change record by slug
  (`ChangeLookupError::Unknown` or `NotLoaded`), and `Workplan::preview` is built on
  it with the same errors. The `docs_sync` example opens the repository through
  `Gate::open`: a root that is inside a git work tree but holds no `.git` entry now
  lists its tracked paths, and a root that holds source but no capability document is
  exit 2 with the gate's message instead of a sync with nothing to write.
- **`raf-workplan` bound executions:** the `bound` module runs the tests,
  documentation tests and examples a capability binds, through an `exec` runner only.
  An `Executor` over the source index plans a `Binding` before any process starts: a
  path that does not resolve, an item that is not a `#[test]` function, a target with
  `harness = false` or `required-features`, a test in a benchmark, example or
  build-script target, and a value that fails its pattern are each a typed `Refusal`
  that names the binding and the target, and nothing reaches `cargo`. The libtest name
  is the path within the crate (`documents::x`, `bindings::tests::x`), and the target is
  pinned as `--lib`, `--bin=<bin>` or `--test=<test>` under its Cargo name, hyphens
  included. A test passes only when the list pass shows exactly one `<name>: test`
  entry and the run pass reports `1 passed; 0 failed; 0 ignored` in one `test result:`
  line; zero entries, two entries, `0 passed` and an `#[ignore]`d test (listed, then
  reported as ignored) are refused. A documentation test is selected by the item part
  of the package's list (impl items as rustdoc writes them, `Wrapper<T>::get`), run
  with the bare item path as a substring filter, and passes only when every selected
  entry has the whole line `test <name> ... ok`; a `no_run` (`- compile ... ok`) or
  `compile_fail` (`- compile fail ... ok`) test is refused with a request for a
  runnable one, and a failing sibling the substring also matches fails the exit status
  and so the binding. An example is run with `cargo run` and standard input null, and
  its exit status decides. Each pass of a test or documentation test has the 1800 s
  limit and the run of an example the 300 s limit, which also covers the build `cargo
  run` does first (`Executor::with_limits` replaces them); a process killed at its limit
  or for its output is a failed binding that reports how long it ran, and a process that
  cannot be started is an `ExecError`, not a verdict. The result is an `Evidence` value
  per binding (the invocations that ran, the counts of the proof, the verdict as
  `BoundOutcome::{Passed, Failed, Unresolved}` and the wall time) with a `Display`;
  nothing is written. The fixtures run the real `cargo` against crates generated into a
  scratch directory on every platform.
- **`raf-workplan` change lifecycle, first steps:** the `lifecycle` module opens and
  reads a change. `Workplan::new_change` takes a `NewChangeRequest` (a slug, one or
  more `cap:<name>` capabilities, any number of `cap:<name>#<section>` `affects`
  entries) and writes `work/changes/<slug>.md`, the only file it writes, with the
  `raf-okf` writer: the id `change:<slug>`, the capabilities in `changes`,
  `contract_effect: changes`, an empty `items` list, a title and description for the
  author to replace and a non-empty `## Summary`. The slug must be a lowercase
  kebab-case slug (`raf_okf::is_kebab`, now public, the one pattern of the profile),
  so `Upper`, `-rf`, `--since=x`, an empty slug, a path separator and `..` are refused
  before anything is written; each capability must be a loaded capability; every
  `affects` entry must name a listed capability and one of the seven sections; with
  none given the record affects the `contract` section of each listed capability. A
  capability or `affects` entry given twice is listed once. The file is created with
  `create_new`, so an existing file or link is a typed `Exists` error and is never
  overwritten, `work/changes` is created when it is missing and refused when a file or
  link stands in its place, and the record is read back through the loader: one that
  has a finding (a slug that reads as a tracker reference) is removed again and
  reported as `Invalid`. `NewChangeError` is `#[non_exhaustive]` and tells a bad slug,
  an existing file, an unknown capability, a capability that did not load, a bad
  `affects` entry and a write failure apart. The `NewChange` result carries the path,
  the PROPOSED block template (`> [!PROPOSED change:<slug>]` and a placeholder line),
  one `CapabilityGuidance` per listed capability (`delivered` or `designed`: set
  `state: implementing` in the same commit, since a PROPOSED block is a finding in
  both; `considered` and `implementing`: nothing) and whether the new record makes a
  generated block stale (the relationship tables name the changes that list a
  capability), and its `Display` is the text a command prints. `Workplan::impact`
  returns one `Impact` value for a change slug: the capabilities in `changes`, the
  sections in `affects`, the bindings of those capabilities with what the source index
  made of each, the capabilities that depend on them and the sections that link to the
  affected sections, each list sorted and without repeats, with a `Display`; it writes
  nothing, and an unknown slug and a record that did not load are told apart as
  `ChangeLookupError`. `SectionId` now has a `Display` (`cap:<slug>#<section>`) and
  `BindingKind::key` names the list of a binding, replacing two private copies.
- **Workplan capabilities delivered:** the workplan document model and the
  generated documentation blocks are `delivered`, bound to the `raf-workplan` code,
  tests and examples that realise them, with their verification matrices,
  relationship tables and the catalogue generated by `make docs-sync`.
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
