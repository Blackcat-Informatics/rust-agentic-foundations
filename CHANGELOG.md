<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Changelog

All notable changes to RAF are recorded here. The workspace ships one version
across all crates. Until 1.0.0 a breaking change bumps the minor version.

## [Unreleased]

- **`raf-source`:** `use` paths follow two more visibility rules of rustc. A named
  re-export or a glob that reads a binding through a glob the importing module
  cannot access keeps the narrower of its own visibility and the visibility of
  the binding it reads, measured on the importing module's ancestor chain, so a
  module that globs such a re-export cannot read a name rustc hides from it. A
  name that two globs bring in as different items is ambiguous in that module
  for every reader that can access at least one of the items, and such a reader
  gets `ResolveError::Ambiguous` listing every candidate, as rustc refuses the
  path; a path through a module that two globs bring in as different modules is
  ambiguous in the same way. For a name that two globs bring in as different
  items the reader gets `Ambiguous` in every declaration order. Where an explicit
  import, or a glob of a module that defines the item, supplies an item that
  another glob also carries as a companion (a hidden item riding along with
  it), the index gives that item, one answer in every order, even where rustc
  refuses the crate in some orders and accepts it in others.
  `Workspace::resolve_from(module, path)` and `resolve_from_in` resolve a path as
  a `use` written in the named module would, and `ResolveError::UnknownModule`
  names a module that is not in the index. The crate's resolution is compared
  with rustc on generated programs with private, public and crate-visible
  globs; the divergences that remain are a glob cycle rustc settles differently
  in a name space, the glob re-export rustc only warns about, and shapes on
  which rustc's own answer depends on declaration order. An item that a cycle
  of globs carries back to a module no longer leaves a companion item in that
  module's view, so the internal consistency check of a debug build no longer
  stops the load in the shapes fixed. The crate's tests no longer read the clock:
  they count work through a counting tree and test-only tallies, and the
  repository gate refuses clock use in the crate's source and tests.
- **`raf-source`:** a path whose first segment is a bare name that two globs bring
  in as different modules is ambiguous, as a later segment is, and an import that
  starts with it binds nothing. The `cfg` alternatives of one module that a glob
  brings in are one candidate at any segment. `Workspace::resolve_from` and
  `resolve_from_in` answer `Ambiguous`, as `resolve` does, for a reading module
  of a crate name that libraries of several packages share. A companion item
  that an explicit import or a defining glob supplies, one that arrives where the
  reader cannot access it, or one that a cycle of globs carries back, no longer
  makes the answer depend on the order of the declarations. The comparison with
  rustc probes bare first segments and fails an index answer that changes with
  the order of the declarations.
- **`raf-okf`:** reads and writes the RAF profile of OKF documents. It parses
  Markdown with YAML frontmatter in a closed subset, validates the closed key sets
  and typed values of capability, change and architecture documents and reports
  every violation with its file, rule and position, models the body (sections,
  links, fenced code, generated and PROPOSED blocks) and rewrites spans in it, and
  writes documents back deterministically so a parsed document renders to its own
  bytes. Input is bounded in size and nesting, anchors and aliases are refused, and
  the crate reads no file.
- **`raf-okf`:** the profile gains three record types beside the documents:
  `Evidence` (`work/evidence`), `Assessment` (`work/assessments`) and `Coverage`
  (the one record `work/coverage/default.md`). A record states a result in its
  frontmatter, with no title, description or body. Evidence and an assessment
  share the trace keys `subject`, `inputs` (rows of a kind, a name and a SHA-256
  digest), `verdict` and `by` (`human` or `agent`, never a name); evidence adds
  the capabilities it covers, the command, the commit, and measured values with
  their limits, and the coverage record holds the crates in scope and the
  exclusions with their reasons. `Profile::validate` returns `EvidenceRecord`,
  `AssessmentRecord` and `CoverageRecord`, and reports one rule for each fault.
  `Body::claims` reads the claim marker
  `<!-- raf:claim evidence=<slug>[,<slug>...] -->` with the paragraph or list item
  it supports; a malformed or unattached claim marker and any other `raf:` comment
  that is not a `raf:generated:` comment are parse errors, and a claim outside a
  capability is a violation. `DocType`, `RafDocument`, `Rule::ALL` and
  `ErrorKind::ALL` grow, so an exhaustive match over them names the new variants.
- **`raf-okf`:** a record is built with `EvidenceRecord::new`,
  `AssessmentRecord::new` or `CoverageRecord::new` (with `RecordHeader::new`,
  `Input::new`, `SectionRef::new`, `Measure::new` and `Exclusion::new`) and
  written by its `to_document`: keys in key-table order, numbers unquoted as
  written, an empty list as `[]`, absent optional keys left out and an empty body,
  so `Profile::validate` reads the text back as the same record.
  `EvidenceVerdict::name`, `AssessmentVerdict::name` and `EvidenceSubject::text`
  give the key values. The limits of an evidence record are judged against its
  values in time linear in their number.
- **`raf-okf`:** `Number::compare` and `Number::exceeds` order two numbers
  exactly on their decimal texts (sign, whole part without leading zeros,
  fraction without trailing zeros, any length), and `Profile::validate` judges the
  limits of a passing evidence record with them, so a value over its limit by less
  than an `f64` tells apart is refused. A limit of a failing evidence record may
  name no value, since a failing execution may have printed none.
- **`raf-source`:** `Workspace::exported_items` lists the public surface of a
  library or a binary: the items another crate can name through its crate root,
  each as an `ExportedItem` with the path it is exported under and the item that
  defines it, so a re-export under another name maps back to its definition. An
  item is exported through a route of `pub` declarations back to a `pub`
  definition, through `pub` modules and named, renamed and glob `pub use`
  re-exports, so a `pub` item of a private module is exported only through a
  re-export and a restricted item or a re-export that is not `pub` keeps a name in.
  The `pub` members of the inherent impls of an exported type and the members of an
  exported trait follow it; test code is never exported. A re-export of another
  crate, of an enum variant or of a path the index cannot follow is a `Reexport`
  and not an item, also where a `pub` glob carries it into the module that reads
  the glob. `Item::macro_export` tells a `#[macro_export]` macro.
  `#[doc(hidden)]` paths are left out unless `ExportOptions::include_hidden` lists
  them, flagged. An ambiguous or cyclic public name, or a `pub` glob of nothing,
  fails with the resolver's error, and the listing is charged to the item-bytes
  budget left after the load; a listing whose decision of a carried re-export
  passes it is `ExportError::CarryBudget`, naming the re-export.
- **`raf-workplan`:** `Workplan::load` reads the records: the evidence of
  `work/evidence`, the assessments of `work/assessments` and the coverage record
  `work/coverage/default.md` (`Workplan::evidence`, `Workplan::assessments`,
  `Workplan::coverage`). Any other file in `work/coverage` is
  `singleton-other-file`, and a repository without these directories loads as
  before. The graph gains record and crate nodes and the `for`, claim, assessment
  subject, `scope` and input edges (`Graph::evidence_of`, `Graph::claims_on`,
  `Graph::assessments_of`, `Graph::inputs_of`); a symbol a record input names is
  a node, and an input that resolves to nothing is no edge and no finding. `check`
  reports `claim-evidence-missing` and
  `claim-evidence-not-for` with the line of the marker, `evidence-for-missing`,
  `assessment-subject-missing` and `scope-crate-missing`. `NodeKind`, `EdgeKind`,
  `Node` and `Edge` grow. `digest::Digest` is the SHA-256 of a section, a symbol
  or a record over a domain-separated, length-prefixed encoding with CRLF read as
  LF, computed in the crate with no new dependency. `record::create` writes a new
  record exclusively and `record::replace` replaces one through a temporary file
  and a rename, each with optional SPDX lines and refusing a record that would not
  load clean.
- **`raf-workplan` history:** `Workplan::history(id, options, runner)` returns the
  `History` of a `cap:`, `change:`, `arch:`, `evidence:` or `assessment:` id: the
  commits of its document through its moves (`git log --follow --cc --root
  --name-only`, with the name it had at each, the merges whose resolution changed
  it and the first commit whatever `log.showRoot` says) and, for a capability, of each resolved `code` binding's
  span (`git log -L<start>,<end>:<file>`) and, for a binding of a module below its
  crate root, of each file its digest reads (the module's own file and the file of
  every module below it, with the document's `git log --follow` row), each commit
  once with all its `Source`s and paths, newest first by author date and then by
  commit id. A file of a bound module that no commit holds is
  `HistoryError::Uncommitted`, which names the binding and the file. A `code` binding
  that no longer resolves is a `RemovedBinding`: the newest commit that changed how
  often the last segment of its path occurs in its crate's directory
  (`git log --root -S<identifier> -n 1`) as the commit that removed it, and its
  first parent (`%P` of the same query) as the last commit in which it resolved.
  `--since` keeps the entries whose author date is at or after 00:00:00 UTC of its
  day, so a date selects the same commits on every clock and in every time zone; it
  is compared in process and reaches no `git` process, whose own `--since` reads
  the committer date. The id is
  matched against the loaded documents and never reaches a process; an unknown,
  unloaded or wrong-kind id, a binding that names no single symbol and any value
  that fails its pattern are refused before any process starts. Then `git
  --version` refuses a Git older than 2.30, and `git rev-parse
  --is-shallow-repository` refuses a shallow clone. A log over 16 MiB is
  `GitError::LogTooLarge`; a record is split at `0x1f` at most twice, so a subject
  keeps every byte but a newline. `history::render::table` and
  `history::render::json` write the same entries in the same order. `exec` gains
  the patterns `TreePath` (a path given only after `--` or after the colon of one
  `-L` argument, so it may begin with `-`), `Identifier` (one `-S` argument) and
  `Date` (`YYYY-MM-DD` from 1970 to 2099, the `--since` day), `LineRange`, and
  the argument variants and rows of the four commands; `git` gains
  `is_shallow`, `document_log`, `symbol_log` and `last_change` (`LastChange`).
  Table cells escape control and bidirectional formatting characters.
  `Pattern::ALL`, `Arg` and `GitError` grow. A `git log -L` that `git` refuses,
  as for a bound file that is not committed or whose copy in `HEAD` ends before
  the line the symbol starts at, is `HistoryError::SymbolLog`, which names the
  binding and the file, quotes `git` and names committing the file as one way to
  read the lines. The history writes nothing.
- **`raf-workplan` evidence:** `Workplan::record_evidence(root, runner, request)`
  runs one execution and writes the evidence record `work/evidence/<id>.md`. The
  verdict comes from one `VerdictSource`: the caller's own command
  (`exec::TrustedArgv`, the words after `--` of the command line of the running
  process, which `TrustedArgv::from_command_line` takes from `std::env::ArgsOs`
  and no document or record text can build; `Program::Trusted`,
  `Invocation::trusted`, `Arg::Trusted` and `Limits::trusted`), run with no shell,
  standard input null, the workspace root as its directory, the caller's
  environment, 16 MiB per stream and 1800 seconds, its exit status deciding pass
  or fail and its `name=number` output lines giving the values; or a bound test or
  example of a capability of `for`, run by the `bound::Executor`. A value over its
  limit (compared exactly on the decimal texts), a failing status and a signal
  record `fail`, `Recorded::failed` naming each reason, and so does a kill at the
  time limit, whatever the limits; a value that is not a number (an exponent
  included), a name printed twice, more than `MAX_VALUES` (1024) values, a limit a
  command that exited never printed and output past the bound are `MeasureError`s
  or `EvidenceError`s and write nothing, and so does a command that cannot start. The
  inputs are the `Digest::symbol` of each `code` binding of each capability of
  `for`, which must be loaded and bind code that resolves. `parse_limit` reads a
  `name=number` limit. `Workplan::evidence_claims` lists the statements resting on a
  record (file, line, section and text), and `Workplan::evidence_for` the records
  supporting a capability, by `for` and by claim. `Programs::path_of` returns an
  `Option`, `None` for the caller's own command.
- **`raf-workplan` claim and evidence obligations:** `ObligationKind` gains
  `claim-unsupported` (a claim whose evidence does not load, fails or holds a value
  over its limit, once for each that holds), `evidence-stale` (a record input whose digest now differs, or
  that no longer resolves, named), both from `Workplan::reconcile`, and the process
  kind `evidence-changed`, which `Lifecycle::reconcile` and `Lifecycle::complete`
  raise by reading each record a claim names at the merge base: a pass that now
  fails, a value over its limit or the merge base's, a limit gone, or the record
  gone. `Needed` gains `SupportingEvidence`, `RecordedAgain` and `ReviewedClaim`,
  with `ClaimProblem`, `OverLimit`, `InputMoved` and `EvidenceChange`.
- **`raf-workplan` prose assessment:** the `assessment` module records the
  verdict of a human or an agent on one authored section, `cap:<slug>#<section>`,
  as the record `work/assessments/<capability slug>-<section>.md`
  (`record_slug`), one for each section. Its inputs, each kept with its digest, are
  the section (generated blocks left out), the `code` bindings of the capability
  the section names in code spans (by the path or a `::`-suffix of two segments
  or more, beside those it names by the last segment alone; every `code` binding
  when it names none by a path), and the evidence records its claims name. `Workplan::assess_context` gives the
  assessor the section, the bound symbols with their source text and the evidence
  with its verdicts, the same for a human and an agent;
  `Workplan::assessment_record` builds the record and `Workplan::record_assessment`
  writes it, replacing the assessment of the section, refusing a bound symbol that
  does not resolve, a claim on a record that does not load and a record over the
  document bound; `Workplan::list_assessments` says of each record whether it is
  current and which inputs moved (`Currency`, `MovedInput`, `InputChange`). A
  section with nothing outside its generated blocks and comments is not assessed.
  `check` reports `assessment-id-mismatch` for an assessment named otherwise than
  its subject. The `prose_of` pass over a section is linear in its text and blocks.
- **`raf-workplan` assessment obligations and condition 6:** `ObligationKind` gains
  `assessment-missing`, `assessment-stale` (once for each input that moved, named)
  and `assessment-rejected`, raised by `Workplan::reconcile` for each authored
  section in `affects` whatever the `contract_effect`, with `Needed::Assessed`,
  `Needed::AssessedAgain` and `Needed::AcceptedAssessment`. `Lifecycle::complete`
  gains condition 6 (`Condition::Assessments`, `Failure::Assessment`): each
  authored section in `affects` needs a current assessment with the verdict
  `accepted`, by a human or an agent, judged with the static conditions before any
  bound run; a refusal counts the sections without one, each section once however
  many of its inputs moved. `complete` leaves the assessment records as they are.
- **`raf-workplan` two-way coverage:** the `coverage` module judges the public
  items and the tests of the workspace against the bindings of the capabilities.
  `Workplan::coverage_report` returns a `CoverageReport` of every package: the
  public items (from `Workspace::exported_items`, `doc(hidden)` left out) that no
  `code` binding covers, the public re-exports of the workspace the index cannot
  follow to an item (a type a macro generates) as `unresolved re-export` entries,
  the tests (`#[test]` functions, unit tests included) that no `tests` binding
  names, what the exclusions of the coverage record take out with their reasons,
  and each surface that cannot be listed as a typed `SurfaceError`, all sorted by
  package and path; `coverage::render` writes each direction as a table and as
  JSON. Binding an item covers the items below it, a binding is followed through
  re-exports to the definition, and an item bound only in `examples` is a gap.
  `check` enforces the crates of the record's `scope`: `coverage-item-unbound`,
  `coverage-reexport-unbound`, `coverage-test-unbound` and
  `coverage-surface-unreadable`, with `coverage-exclusion-unmatched` for an
  exclusion of nothing and one `coverage-record-missing` for a repository with
  capability documents and no record (`scope: []` enforces nothing). A `scope`
  entry written as a crate name is named as the package it means. The
  repository's own record enforces `raf-workplan` and `cargo-workplan` with no
  exclusion: their capabilities bind every public item and every test of them,
  unit tests included.
- **`raf-workplan` digests of modules:** `Digest::symbol` of a module below a
  crate root hashes, after the module's own span, the whole text of each file of
  the module and of the modules below it, so a module binding stands for the
  module's content in evidence and assessment inputs as it does in coverage, and
  an edit inside a bound module makes its records stale while an edit of a
  sibling module does not. A digest recorded against a module before this change
  differs from the one computed now.
- **`raf-workplan` current rationale:** `check` reports an architecture page that
  no capability, change or other architecture page links to
  (`architecture-page-unlinked`; a link from the page to itself does not count,
  and no page is reported while a document that may hold the link does not load),
  a decision page (an architecture page with a `## Decision` heading) without a
  `## Conditions to revisit` heading and a `## Conditions to revisit` heading with
  no list item (`decision-conditions-missing`), and a `### Decision` in the
  Explanation of a capability that no `### Conditions to revisit` follows before
  the next decision, or such a subheading with no list item
  (`explanation-conditions-missing`). Headings match their exact text and level;
  headings in fenced code and in PROPOSED blocks do not count. A condition is
  plain prose and nothing judges whether it has been met. A link to a page or a
  heading that does not exist stays `link-broken` or `anchor-broken`.
- **`raf-workplan` the time of an output kill:** `ExecError::OutputTooLarge`
  gains `elapsed`, the time the process had run when it was killed for writing
  more than the bound, and its message gives it, so a bound test or example killed
  for too much output fails its binding with the time it ran and the bound, as a
  kill at the timeout does, and its run keeps that time in place of zero. A
  pattern outside the crate that names the fields of `OutputTooLarge` without `..`
  names `elapsed` too.
- **`raf-workplan` writes through real directories:** `record::create`,
  `record::replace`, the checks `Workplan::record_evidence` and
  `Workplan::record_assessment` make before they write, `Workplan::write` and the
  removal of a completed record judge each directory on the way to the file
  (`work` and the directory of the record type, `docs` and `docs/capabilities`, or
  `work` and `work/changes`) and refuse one that is a symbolic link, whether it
  leads inside the root or out of it, or that is not a directory, before anything
  runs or is written. A directory link inside the root no longer lets a
  replacement land on another file, such as a capability document.
  `LifecycleError` gains `RecordRefused`, which `Lifecycle::complete` returns for
  a refused way to the record at either judgement of the static conditions, the
  first before any process runs and the second after the bound tests and
  examples ran; its `ran` field and its message say which.
- **`raf-okf` a marker split over lines:** a comment that `<!--` opens at the
  end of a line and whose first non-blank text below is `raf:` in any letter case,
  other than `raf:generated:`, is a malformed claim marker for `raf:claim` and an
  unknown marker for any other word, at the line of `<!--`, so a claim marker
  written over several lines is a parse error and never a plain comment. Split
  `raf:generated:` comments, SPDX headers and other multi-line comments stay plain.
- **`raf-workplan` claims while a capability does not load:**
  `Workplan::evidence_claims` returns the new
  `EvidenceLookupError::UnloadedCapabilities`, naming each capability document
  that exists and does not load, in place of an answer that leaves out the claims
  such a document may hold, so `evidence claims` exits 2 as `evidence for` does. A
  change record or an evidence record that does not load leaves the answer as it
  is.
- **`raf-workplan` coverage of unresolved re-exports and unloaded
  capabilities:** the path an unresolved re-export names is read from the module
  that declares it, the innermost inline module whose braces hold the declaration
  or else the module of its file, with each `super::` one module up, so a binding
  of the module it reaches covers it and a binding of the declaring module that
  the path leaves no longer does. While a capability document exists and does
  not load, `check` reports its own findings and no `coverage-item-unbound`,
  `coverage-test-unbound` or `coverage-reexport-unbound`, in place of one finding
  for every item and test that document binds.
- **`raf-workplan` the caller's command in messages:** the text of a trusted
  `Invocation`, which the messages of `ExecError` quote, is the command as
  `TrustedArgv::text` writes it and the evidence record stores it, each word a
  shell would split or expand in single quotes, so `sh -c 'yes >&2'` and
  `sh -c yes '>&2'` no longer read the same. `SECURITY.md` states the limits of
  the trusted command: an interrupt of `cargo-workplan` does not reach it, a
  descendant that leaves its process group escapes the kill, a relative program
  resolves against the workspace root, a hand-written record may hold more than
  1024 values, the write checks judge paths and not handles, and the time the
  exported-items listing may take before its budget stops it.
- **Escaped document text on a terminal:** `raf_workplan::escape::shown_lines`
  writes each control character but the line feed and the tab, and each
  bidirectional formatting character, as its `\u{..}` escape, the rule the history
  and coverage tables apply to their cells. The text output of `evidence claims`
  and `assess --context` goes through it, so a document cannot clear the screen,
  set the title of a terminal or reorder a line; the JSON forms are unchanged.
- **`cargo-workplan` record commands:** `history <id> [--since <YYYY-MM-DD>]`
  prints the table or the JSON of `Workplan::history` (exit 0, or 2 for an id no
  document has, a shallow clone or a Git older than 2.30). `evidence record --id
  <slug> --for cap:<slug>... --by human|agent [--limit name=number]... [--revision
  <commit>] [--update] [--copyright <text> --licence <id>]` runs the command after
  `--`, or the bound test or example of `--from-binding <path>`, and writes one
  evidence record: exit 0 for `pass`, 1 for `fail`, 2 when no record is written.
  Every word after the first `--` of the command line is the command, options
  included; the parser's reading and the `TrustedArgv` of `std::env::args_os()`
  must agree word by word, an option whose value would be `--` is a usage error,
  and a program name of `--` and `--from-binding` beside a `--` are refused before
  any process starts. A replacement holds only the SPDX lines its own run is
  given. `evidence claims <evidence-id>` and `evidence for cap:<slug>` print what
  rests on a record and what supports a capability. `assess cap:<slug>#<section>`
  takes one of `--context` (the same for `--by human` and `--by agent`),
  `--verdict accepted|rejected --by human|agent` (one record written, exit 0 for
  either verdict) and `--list` (each assessment with its currency, exit 0; a
  section given to it must exist).
  `coverage code|tests [--crate <package>]` prints one direction of the coverage
  report of the whole workspace or of one package, and exits 1 when it names a
  gap. Each id, day, limit, revision, section and package is checked when the
  arguments are parsed, before any process starts. The JSON kinds
  `history-entry`, `history-removed`, `evidence-recorded`, `evidence-claim`,
  `evidence-for`, `assess-context`, `assessment-recorded`, `assessment`,
  `coverage-gap`, `coverage-excluded` and `coverage-error` are documented in the
  crate documentation with their members in order.
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
- **`raf-workplan` obligations:** `Workplan::reconcile` returns the `Reconciled` list of
  a change, what the documentation still owes it, from the loaded documents alone. An
  `Obligation` has a closed `ObligationKind` (`unresolved-proposal`,
  `item-without-outcome`, `binding-missing`, `section-untouched`, `example-failing`,
  `generated-stale`, `tense-mismatch`, with `ALL`, `name` and a check that `ALL` lists
  every kind), a `Subject` (a capability, the change record or the capability index), a
  `Part` (a section, an item counted from 1, or the document), a `Place` (file and
  line) and a typed `Needed` evidence whose `Display` is the evidence column of the
  reconcile contract followed by the one thing it is about, such as `(bindings.tests is
  empty)`. Each is derived from a value the model holds: the PROPOSED blocks of the
  graph for the change (the line of the marker) and the blocks of capability documents
  that did not validate, which the load keeps (a block stays owed wherever it stands),
  the items of the record (a `moved`
  outcome that names no change record, or the record itself, is owed too), the roadmap
  gaps of each listed `implementing` or `delivered` capability (a `delivered` one owes
  `code` and `tests` only) with the bindings that resolve to an item their list does
  not take, and the stale and missing generated blocks of a sync in the listed
  capabilities and the index. A `delivered` capability whose only fault is an empty
  `code` or `tests` list is refused by the profile and does not load; reconcile reads
  the empty list as the binding it owes. Any other listed capability that did not load
  is `ReconcileError::CapabilityNotLoaded`, with the findings of its document, and an
  unknown change and a record that did not load are `ReconcileError::Change`.
  `tense-mismatch` reads the prose of a listed `implementing` or `delivered` capability
  against `obligation::tense::PHRASES` (`will`, `is planned`, `are planned`,
  `proposed`, `to be implemented`, `in the future`, `going to`, `not yet`) outside
  fenced code, code spans, HTML comments, PROPOSED blocks, generated blocks and link
  destinations; words match whole (`goodwill`, `William` and `willing` do not), a phrase
  may wrap over one line break, the marker name `PROPOSED` in capitals is not wording (`WILL` is), and a
  delivered capability with a PROPOSED block owes the same kind. The scan is linear in
  the size of the text. The list is sorted by subject, section (the document, then the seven sections in
  document order, then items), kind, line and then
  evidence text and has no repeats, so two blocks in one section print in document
  order and equal documents print equal text, whatever the load order; `Display` prints
  one line each (`cap:x  contract  unresolved-proposal  line 31: ...`) and nothing for
  an empty list. `contract_effect: none` suppresses `section-untouched` for the
  `contract`, `usage` and `examples` sections and nothing else (`obligation::suppressed`).
  `section-untouched` and `example-failing` need `git` and `cargo`; they are in the
  closed set, with their evidence, and `reconcile` does not raise them from the
  documents.
- **`raf-workplan` reconcile with a process, and verify:** `Lifecycle::open` is the
  entry point of the commands that start a process. It runs `git --version` before any
  other process and refuses a Git older than 2.30 (the Git version gate belongs to this
  entry point and not to `Gate::open`, and `change new`, `change impact` and `verify`
  do not go through it), then opens the repository; `Lifecycle::open_with` takes a
  `Gate` and any `Runner`. `Lifecycle::reconcile` returns the whole obligation list: the
  static kinds of `Workplan::reconcile` and the two that need a process. A section in
  `affects` is `section-untouched` when its authored text is the text it has at the
  merge base of `HEAD` and the default branch of `origin`: `git::default_merge_base`
  runs the three questions in order, and `blob_at` now takes only a `git::MergeBase`, a
  value that only `merge_base` makes, so a commit id from anywhere else (which `git`
  would report as an absent file) cannot be asked about; two `compile_fail` doctests pin
  that. The comparison is `lifecycle::section_digest`: the heading and the content of the
  section with the content of each generated block taken out on both sides, in LF line
  endings, so a regenerated block is not an edit, and the marker lines, fences and
  PROPOSED blocks are. A file or a section that is absent at the merge base counts as
  touched, the default branch itself reports every affected section, a merge-base file that
  exists and is not UTF-8 or does not parse as a body is an error, a blob over 1 MiB is an
  error, and a repository with no default branch is an error that names `git remote
  set-head origin --auto`. Nothing is asked of `git` when `contract_effect: none` leaves
  out every affected section. `git` names a file at a commit from the top of the work
  tree, so `Lifecycle::open` also runs `git rev-parse --show-prefix` (`git::show_prefix`,
  a `Prefix`) after the version gate and reads each document at `<prefix><path>`: a
  workspace in a subdirectory of a repository, however deep, compares correctly, a
  prefix outside the source-path pattern (a space, a non-ASCII letter) is an error and not
  a guess, a linked work tree is its own top, and a root outside a git work tree is an
  error. `example-failing` runs the example targets and documentation tests that the
  listed capabilities bind and that resolve through the executor; one that fails to build
  or run, runs nothing (a documented item with no documentation test, an item that is not
  an example), is killed at its limit or writes too much is owed, with a
  `RunProblem` in its evidence (`Needed::PassingRun` gains the field); test bindings are
  not run by `reconcile`. `lifecycle::verify` (also `Lifecycle::verify`) runs the bound
  tests and examples of a change or of every capability and returns a `VerifyReport` of
  per-binding `Evidence`: `Display` is one line a binding (`pass` or `fail`, the
  capability, the list, the path, and the counts or the reason), `passed()` is true only
  when every binding passed, a binding that does not resolve or whose value fails its
  pattern is a `fail` that never reaches `cargo`, and nothing is written. An unknown
  slug, a record that did not load, a listed capability that did not load, and (for every
  capability) any capability document that did not load are errors, as is a process that
  cannot be started. The tests use real `git` repositories (the merge-base matrix, a
  linked work tree, a local branch named like the remote one, a default branch named like
  an option), stub `git` programs that record their argv, and real `cargo` fixtures.
- **`raf-workplan` complete:** `Lifecycle::complete` is the transition from an open
  change to a delivered one, and returns a `Completion`: `Completed { removed, rewritten }`
  or `Refused { failures }`. The order is fixed. First the static conditions are judged,
  every one and with no early exit: the obligation list of `reconcile` (every kind except
  `example-failing`), the outcome of each item (a `moved` outcome names a record that
  exists), the PROPOSED blocks of the change in any capability document whether it loads
  or not, the generated blocks (`docs sync` would write nothing, from the one analysis
  `check` and `sync` use), and `check` of the repository as it will be once the writes
  are done, which is loaded through a tree that does not show the record and shows the
  rewritten generated blocks (`Opened::workplan_after`). The state rules are therefore
  judged there and not before: a `delivered` capability beside its still-open record is
  what `complete` resolves, and a capability that is not `delivered` and is listed by no
  other open record is the `no-open-record` finding, which says to move the state to
  `delivered`. The author sets the state; `complete` never edits prose. Only when no static
  condition failed are the bound tests and examples of the listed capabilities run through
  the executor, so a static failure spends no process on `cargo`; a failing example is an
  `example-failing` obligation and a failure of condition 5, a failing test is a separate
  failure, and a capability with empty `bindings.tests` and `bindings.examples` satisfies
  condition 5 only while it is `designed` or `considered`. Then the repository is opened
  again with the same `Gate` and the static conditions are judged again immediately before
  the writes, because the documents and the source can change while `cargo` runs; a
  failure, or a different set of bindings to run, refuses and names the change first. The
  writes are the rewrite of the generated blocks the removal stales (the `Changed by` rows
  of the relationship tables), through `Workplan::write`, which stages every file and
  then renames, and then, as the very last step, the removal of `work/changes/<slug>.md`;
  nothing else is written. A rewrite that fails leaves the record in place
  (`LifecycleError::Rewrite`), and a removal that fails after the rewrite names the
  files already rewritten (`LifecycleError::Remove`), so a removed record is never found
  beside stale blocks. A `Failure` is one of: an obligation (condition 1), an item
  without outcome (2), a PROPOSED block (3), a generated block or file `docs sync` would
  write or refuse (4), a bound test or a bound example that did not pass (5), a finding
  once the record is removed, or a change during the run; `Failure::condition` and
  `Failure::obligation_kind` say which. `Display` prints `refused:` and a count for each
  kind of failure (`refused: 2 obligations, 1 item without outcome, 1 stale generated
  block`), then one line for each failure that no other line repeats, and `completed:
  removed work/changes/<slug>.md` with a `rewrote <path>` line for each rewritten file.
  An unknown slug, a record or a listed capability that did not load, a process that
  cannot be started and a failing write are errors and not refusals. The tests use real
  `git` repositories and real `cargo` fixtures for the refusal of each kind of
  obligation (the record and the tree byte-identical afterwards, and the repair
  completing), each condition, the happy path (the gate clean afterwards and the
  tree's only changes the rewrite and the removal), and scripted runners for the order
  of the processes, hostile bindings, the repository changing during the run and the
  order of the writes.
- **`cargo-workplan`:** the cargo subcommand of the workplan, a new workspace member
  (package and binary `cargo-workplan`) with two dependencies of its own, `clap`
  (derive; the colour feature is off) and `serde_json`, both under the licences
  `deny.toml` allows with no exception and no skip. It runs as `cargo workplan
  <command>` (cargo passes the argument vector `cargo-workplan workplan <command>`) and
  as `cargo-workplan <command>`; both forms take the same arguments and give the
  same result. The commands are `check` (the findings of `Gate::try_check`, one line
  each in the library's order), `status` (the roadmap of `Workplan::status`: a row for
  each state in the order delivered, implementing, designed, considered, with the gaps
  of each implementing capability and the open changes that list it, and a row for the
  open changes; a gap is not a violation, so it exits 0), `docs sync` (the generated
  blocks of `Workplan::sync` through `Workplan::write`, `rewrote <path>` for each
  file; `--copyright <text>` and `--licence <id>` are read only when
  `docs/capabilities/index.md` has to be created, and a missing one is exit 2 that names
  the flag) and `docs preview <slug>` (each capability the change lists as it reads
  once reconciled, under a `==> <path> <==` line; it never writes). The options are
  `--root <dir>`, `--format text|json` and `-q`, each accepted before or after the
  command. Without `--root` the root is `raf_workplan::root::discover`, exactly
  `cargo metadata --format-version 1 --no-deps --frozen`, run in the current
  directory with the `cargo` named by `$CARGO` or else on `PATH`. The exit codes are 0
  (no finding), 1 (findings) and 2 (the tool could not run: a usage error, a
  directory that is not a repository, a `git` or `cargo` that cannot run, an unknown or
  unloaded change, a sync that refuses a file); standard output holds the findings in a
  stable order and standard error the diagnostics, and `-q` prints nothing but
  findings (so `status`, `docs sync` and `docs preview` print nothing under it, and a
  failure is still written to standard error). A slug is checked against the slug
  pattern of the profile when the arguments are parsed, so a slug that starts with `-`
  is a usage error, exit 2, before any process starts. `--format json` prints one JSON
  array for the run: one object for each printed record, `kind` first, then the fields
  of its kind (a rule of the document model has `path`, `section`, `line`, `key`,
  `message` and `fix`; the others are `rewrote`, `state`, `open-changes` and `preview`),
  documented in the crate documentation and pinned by tests; standard output is empty
  when the tool could not run. The binary holds no rule of its own: each command is a
  call into `raf-workplan` and a rendering of what it returns, and a test shows that
  `check` prints the findings of `Gate::workspace(..).check()` and exits 0 exactly
  when `assert_clean` does not panic. The tests run the built binary in both shapes
  of the argument vector against repositories written to a scratch directory: every
  command with exit 0, 1 and 2, `-q`, the JSON schema, the tree byte-identical after
  `check`, `status` and `docs preview`, root discovery from a subdirectory and
  `--root` overriding it, and with stub programs the exact `cargo metadata` argv
  from `$CARGO` and from `PATH`, no process before a slug that starts with `-`
  is refused, a missing `git`, and a `build.rustc` program that discovery does not run.
  The macOS and Windows CI jobs run these tests too. The change lifecycle commands
  and `verify` are described in the next entry.
- **`cargo-workplan` change commands:** `change new <slug> --capability cap:<name>
  [--capability ...] [--affects cap:<name>#<section> ...] [--copyright <text> --licence
  <id>]`, `change impact <slug>`, `change reconcile <slug>`, `change complete <slug>`
  and `verify [<slug>]`, each a call into `raf-workplan` (`Workplan::new_change`,
  `Workplan::impact`, `Lifecycle::reconcile`, `Lifecycle::complete`,
  `lifecycle::verify`) and a rendering of what it returns. `change new` writes exactly
  one file and prints the library's text: the path, the PROPOSED block to place, the
  state each listed capability needs and, when the record stales a generated block, the
  note to stage the record (`git add work/changes/<slug>.md`, since the table that names it links to a file git must track) and then run `docs sync`; `--capability` and `--affects` repeat, and `--copyright` and
  `--licence` are given together or not at all (a usage error otherwise). `change
  impact` prints the capabilities, sections, bindings, dependants and linking sections.
  `change reconcile` prints one line for each obligation and exits 1 exactly when there
  is one; it writes no file in the repository (a bound example builds into the cargo
  target directory). `change complete` prints `refused:` with the counts and the lines
  of the failures, exit 1, or `completed: removed <path>` with a `rewrote <path>` line
  for each file rewritten, exit 0. `verify` prints `pass` or `fail` for each bound
  binding and exits 1 when one fails; a run with nothing to run exits 0 and says so on
  standard error. Exit 2 is the tool that could not run: a usage error, a slug that
  starts with `-` or is not a slug (refused when the arguments are parsed, before any
  process starts), a slug with no record (and, for `change new`, one that has a
  record), a record that does not load (an `outcome` that is not one), a `git` that is
  missing or older than 2.30 (`change reconcile` and `change complete` ask `git
  --version` before any other process of theirs), a repository with no default branch.
  `--format json` prints, besides the earlier kinds, `new-change`, `impact`,
  `obligation`, `refused`, `refusal` (one for each failure, with the condition it is a
  fault of, its number in the Contract and whether another line repeats it), `completed`
  and `binding` objects, documented in the crate documentation and pinned by tests. The
  tests run the built binary against a fixture repository for each command with exit 0,
  1 and 2, in both shapes of the argument vector and through a real `cargo workplan`
  dispatch to a `cargo-workplan` found on `PATH`, with `-q`, the JSON objects, the tree
  byte-identical after `change impact` and `change reconcile`, a slug that starts with
  `-` for every command that takes one, a stub `git` of version 2.29 (only `git
  --version` runs), the printed template placed in a capability of each of the four
  states (after following the printed guidance: a clean `check` and one
  `unresolved-proposal` line), and the whole path from `change new` to a completed
  change.
- **`raf-workplan` for the command line:** `NewChangeRequest` takes an optional
  `copyright` and `licence`; when both are set the record opens with the two SPDX
  comment lines of the frontmatter, in the form the change records of this repository
  carry them, so a record the tool created passes `make reuse`; one without the other
  is `NewChangeError::SpdxUnset` naming the missing field, and a value that is empty or
  not one line (a control character, U+2028 or U+2029 included) is `NewChangeError::SpdxInvalid`. The state guidance for a `delivered`
  capability now names both findings it avoids (`state-delivered-proposal` and
  `state-delivered-open-record`) and the one for a `designed` capability names
  `state-designed-proposal`. `NewChange::sync_needed` is now true whenever the record
  changes what a sync writes, so it is no longer silent when the target file was
  already stale. The command line reads values through accessors instead of decomposing
  them: `Finding::section`, `line` and `key`; `Workplan::open_changes` (the loaded
  records sorted by slug, as `OpenChange`) and `RoadmapEntry::open_change_slugs`;
  `StateAdvice::name`; `Condition::name`, `Failure::name`, `Failure::is_repeated_in`
  and `Completion::headline`; `VerifyReport::lines`; and `BoundOutcome::name` and
  `reason`. All are additive.
- **Workplan capabilities delivered:** the workplan document model and the
  generated documentation blocks are `delivered`, bound to the `raf-workplan` code,
  tests and examples that realise them, with their verification matrices,
  relationship tables and the catalogue generated by `make docs-sync`.
- **Workplan change lifecycle capabilities delivered:** the reconcile step, the
  completion gate and the command line are `delivered`, bound to the `raf-workplan`
  and `cargo-workplan` code, tests and documentation tests that realise them, with
  their verification matrices and relationship tables. Their documents state the
  built contract: the options of `change new`, the `git` commands of `reconcile` with
  `git rev-parse --show-prefix`, the order of `complete` (static conditions, the bound
  runs, the second judgement, the rewrite of the generated blocks and the removal of
  the record), the time and output bounds of each process, the JSON encoding and the
  two forms of the command line.
- **`raf-workplan` documentation tests of an impl in another file:** rustdoc names a
  method by the module of its impl block, so a doc-test binding to a method whose
  impl block sits in another file than its type (`Workplan::sync`, whose impl is in
  `src/sync.rs` while the type is in `src/model.rs`) is listed and run under the
  module of that file, `sync::Workplan<'_>::sync`. Before, the list pass selected
  nothing and the binding was refused. An impl in an inline module of another file,
  or under another spelling of its self type, is still refused and never run wrongly.
  The module documentation of `bound` states that the proof is not a defence against
  the code it runs: output shaped like a result from a bound test, which is trusted
  workspace code, is read as the run reports it.
- **`make docs-sync`** runs `cargo-workplan docs sync` with the SPDX text of a new
  capability index, in place of the `docs_sync` example of `raf-workplan`; the
  example stays.
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
