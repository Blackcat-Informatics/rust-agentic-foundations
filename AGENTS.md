<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAF agent and contributor guide (AGENTS.md)

This file is the behavioural contract for agents and humans working in the RAF
(Rust Agentic Foundations) repository. It is the only instruction file:
[`CLAUDE.md`](./CLAUDE.md) only imports it. The project is **RAF** in prose; crate
and binary names are lowercase `raf-*` and `cargo-*`.

---

## 1. Goals

The project is bound by these goals. When requests conflict, they decide; they do
not replace the rules below.

- **Greenfield.** Build the best current design and correct the mistakes of the
  past. The limits of earlier work are lessons, not constraints. There are no
  compatibility shims, deprecated aliases or legacy interfaces; a breaking change
  that improves the design is made, with every caller updated.
- **State of the art.** Choose the state of the art over what already exists, on the
  Pareto frontier of quality, speed and efficiency.
- **Rust first.** Core functionality, checks, generators, gates and tests are Rust:
  a library, a test or an `xtask`. Shell is thin glue (the Makefile and the hooks).
  Python is forbidden (section 5).
- **Subsume, extend, enhance.** When something RAF uses holds it back, take over its
  function, extend it and improve on it. Do not work around it.
- **No optionality in the build, hard failures.** There are no optional components,
  conditional imports or feature-gated fallbacks. A missing requirement fails fast
  with a message that says how to fix it.
- **RDF 1.2 only.** No RDF 1.1 forms, quads or other shortcuts. Triple terms and
  RDF 1.2 are first-class.
- **Maximal utility.** After the other choices are made, support the broadest
  functionality possible.
- **Maximal performance.** Aim for the optimal Rust solution, and use the advanced
  features of the language where they pay.
- **Primary target first.** Through the 0.9 release, only the native Linux target is
  implemented and qualified. Other operating systems, architectures, WebAssembly and
  cross-compilation are reassessed after it: a platform-specific failure elsewhere
  does not block a change. See [`docs/TARGETS.md`](docs/TARGETS.md).

---

## 2. What RAF is

RAF is a set of Rust libraries and cargo subcommands that Blackcat projects use as
**dev-dependencies** to keep a code base and its documentation honest:

- **Census** finds structurally duplicated code under a versioned normal form and
  holds it against a ledger.
- **Workplan** plans changes to the documented system, reconciles the documents with
  what was built, and refuses to complete a change until the capability documents
  state the delivered contract.

The repository uses both on itself. The capability documents in
`docs/capabilities/` are the specification, and
[`docs/capabilities/index.md`](docs/capabilities/index.md) lists them with their
states.

| Crate | Role |
|---|---|
| `raf-okf` | Reader and writer for Markdown with YAML frontmatter, and profile validation with closed key sets |
| `raf-source` | `syn` workspace walker: module tree, `#[path]`, symbol and `use`-alias index, impl blocks, item reachability, `Tree` trait with disk and memory backends |
| `raf-rdf` | Closed canonical ground RDF 1.2 report profile over native PurRDF storage |
| `raf-census` | Normal form, fingerprints, grouping, ledger, debt ratchet, migration, detectors, reference scan |
| `raf-workplan` | Capability and change model, graph, bindings, generated sections, obligations, completion gate |
| `cargo-census`, `cargo-workplan` | Thin command-line wrappers over the two libraries |
| `raf-fixtures` | Unpublished test fixtures shared by several packages, so that no source file belongs to two packages and the locked Census gate can judge each one |

`raf-source` is shared on purpose: the census resolves ledger homes through its
symbol index, and the workplan resolves code and test bindings through the same
index.

| Path | Contents |
|---|---|
| `crates/` | The workspace member crates |
| `docs/capabilities/` | One permanent capability document per capability |
| `docs/architecture/` | Shared current decisions (normal form, OKF profile, debt ratchet) |
| `work/changes/` | In-flight change records |
| `work/assessments/` | Recorded prose assessments of capability sections |
| `work/debt/` | The strict debt record, written only by `cargo workplan debt --write` |
| `fuzz/` | The fuzz harness, a workspace of its own |
| `.githooks/`, `.github/` | The commit and merge hooks, the CI workflow and the templates |

---

## 3. Runtime authority

RAF inspects repository contents as data.

- Its analysis libraries never launch processes. RAF never runs tests, examples,
  documentation tests, sample programs or consumer commands. A document, manifest,
  environment variable, configuration file or stored result cannot authorize
  execution.
- Git access uses a native Rust library with read-only operations. Analysis never
  fetches missing objects, invokes hooks or filters, or changes an index, reference,
  configuration file or worktree. It needs no installed Git executable, process
  runner or host security-policy change.
- Missing data and unsupported operations fail with an actionable error; they never
  fall back to another program.
- Verification and change completion are static. They validate source bindings,
  document contracts, generated output and assessments, and they do not claim that a
  test passed or that a compiler accepted the workspace. Prospective completion uses
  an in-memory document overlay.
- RAF does not edit inspected source code or Git state. Explicit document,
  assessment, evidence metadata and ledger writes are allowed within their owned
  destinations.
- Documentation is the one execution exception, confined to the command-line
  documentation component. Reading existing artifacts starts no process. A
  documentation build or source-compilation query needs the caller's explicit
  `--trust-docs` flag on that invocation; trust is never inherited from the
  environment, repository data or a saved option. The flag authorizes the build's
  scripts and procedural macros with the caller's permissions, gives no sandbox
  guarantee, and never enables tests, examples or a general command runner.

Never execute an inspected workspace's tests or examples to check these rules; use
controlled RAF fixtures as data.

---

## 4. Toolchain

- `rust-toolchain.toml` names a **floating nightly** for development and CI. Nightly
  clippy and rustdoc carry lints that stable lacks, and floating means later
  sharpening is never frozen into invisible debt. A new nightly can therefore raise
  new lint findings in untouched code; they are fixed, not silenced. Do not select,
  install or pin another compiler.
- The **minimum supported Rust version is 1.98.1** (`rust-version` in the root
  `Cargo.toml`), checked on the stable channel by `make msrv` for the workspace and
  the fuzz harness. Raising it is a changelog-recorded change on a minor bump.
- The source uses **no unstable features**; a `#![feature(...)]` attribute is
  rejected.
- `syn`, `proc-macro2` and `quote` use exact shared requirements in
  `[workspace.dependencies]`, mirrored in the fuzz workspace, because the native
  parser resource bounds were measured on those implementations. A consumer must
  align its requirements with these pins; conflicting requirements fail during Cargo
  resolution. Changing a pin requires inspecting the replacement and re-measuring.
- PurRDF crates come from crates.io with the requirement `=4`.
- **Path containment** in `raf-source` verifies every handle against the workspace
  root before and after use, and reads contents only through a verified handle. Only
  the Linux strategy is validated. The other platform adapters stay in the source
  without validation claims, and cross-platform testing and cross-compilation are
  not done. The source-index capability states the per-platform contract.

---

## 5. Gates

```bash
make hooks      # use .githooks for this clone; do this once
make check      # every gate: fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, licenses, issue-refs
make docs-sync  # rewrite the generated documentation blocks; the repair for a stale-block finding
```

- **The required gates are the hooks and CI.** The pre-commit hook runs fmt and
  reuse on a checkout of the staged index, then the `repo_gates`, `docs_contract`,
  `self_workplan` and `self_census` test binaries over that checkout. The binaries
  are compiled from the working tree; only the files they judge come from the index.
  The pre-merge-commit hook runs the same gates on merge commits. CI runs on every
  pull request, and every hosted check is green before a merge.
- **Never bypass a hook or a gate.** Do not pass a verification-bypass flag, edit a
  hook to pass, or weaken a gate or a lint to get green. Fix the cause. If a change
  cannot pass, it is not committed.
- **Verify with the narrowest command that proves the claim.** During work, run the
  affected test binary or crate with the gate's own flags (`--locked`, warnings
  denied where the gate denies them). `make check` and whole-workspace runs are for
  integration and release work, not for every small change. Do not run by hand what
  the hook is about to run.
- **A missing tool is a hard failure, not a skip.** `make reuse` needs `uvx` or
  `reuse`, `make msrv` needs `rust-validate`, and `make fuzz` needs `cargo-fuzz`
  (`cargo install --locked cargo-fuzz@0.13.2`).
- **What the root gates check.** `tests/repo_gates.rs` checks licence digests,
  tracker references and process wording, version agreement, the clock rule, the
  Python ban and the first-party manifest policy. `tests/docs_contract.rs` runs the
  workplan checks over the tracked documents and pins the capability inventory.
  `tests/self_workplan.rs` requires every assessment to be current and accepted.
  `tests/self_census.rs` runs the production Census gate through the locked check
  over the workspace and fuzz roots, with the canonical ledger and exact adopted
  debt; that ledger requires `purrdf.*` to supply the BLAKE3 digest and hexadecimal
  text, with no exclusion and no debt.
- **No test asserts time.** Tracked Rust under any crate's `src`, `tests` or
  `examples`, and under the root `tests/`, may not name a clock or a wait on one
  (`Instant`, `SystemTime`, `sleep`, the `*_timeout` calls and their suffixed forms;
  `raf-source` also refuses `std::time`, `Duration` and `elapsed`). The one clock
  reading is `crates/cargo-workplan/src/trusted_docs/clock.rs`, for the documentation
  watchdog. `clippy.toml` disallows the same calls. Measure with instruction counts
  (`perf`) or allocation counts, never with wall-clock runs, benchmarks or sleeps.
- **No mutation testing.** Never write or run mutants, mutation sweeps or
  `cargo mutants`. A test's strength is shown by reading its assertion and by the
  focused run.
- **No Python.** The gates reject tracked Python source, stubs, bytecode and
  archives, and scripts with a Python interpreter line. Repository tooling is Rust.
- **Dependency policy.** `deny.toml` denies wildcard requirements but allows path
  dependencies without a version, and `fuzz/deny.toml` matches it apart from one
  crate-scoped exception. `raf-fixtures` is a path-only dev-dependency; a gate
  refuses it outside `[dev-dependencies]` unless the manifest declares
  `publish = false`. `make deny` runs offline locally, and CI fetches the advisory
  database.
- **Release-profile tests.** `make test-release` runs the `raf-source` tests in
  release mode, because the stack cost of a nesting level differs between profiles.
  The hosted job for it has its own execution limit.

### Lints

Every crate inherits `[workspace.lints]`, and the fuzz workspace mirrors the same
tables (a gate fails when they differ).

- `unreachable_pub`, `missing_docs` and `unsafe_code` are denied. Use `pub(crate)`
  for items that are not API.
- Library code has one `unsafe` block, the Windows final-path call in
  `crates/raf-source/src/tree/disk/final_path.rs`, under an item-scoped allow with
  its reason and a `SAFETY` comment.
- Clippy `all` and `pedantic` warn, and the gates deny warnings.
- `allow_attributes_without_reason` is denied: every `#[allow]` states why and is
  scoped as tightly as possible. Prefer fixing the code.

---

## 6. Engineering rules

### Design

- **Use the type system.** Make illegal states unrepresentable. Parse into domain
  types at the boundary and do not pass raw strings or integers that carry domain
  meaning. Prefer typestate to runtime flags, `let ... else` to nested matching, and
  borrowed parameters (`&str`, `&[T]`, `impl AsRef<Path>`) to owned ones.
- **Layering.** Third-party versions are declared once in `[workspace.dependencies]`.
  Binaries are thin and libraries hold the logic. Crates are small, there are no
  dependency cycles, and visibility is minimal. Keep a file under 5,000 lines; split
  it before it grows past that.
- **SOLID.** Apply it in design and in review, and refactor code that needs it when
  the work touches it.
- **Errors.** Library errors are typed, matchable and keep their source. Production
  paths do not call `.unwrap()` or `.expect()` unless the message states the
  invariant (`INVARIANT: ...`) that makes failure impossible.
- **Output.** Library crates never print (`println!`, `eprintln!`, `dbg!`). Output
  belongs to the command-line crates, examples and tests.
- **Fail hard.** Schemas are strict and closed: unknown keys are errors. A ledger or
  document that is stale, malformed or ambiguous is an error whose message says how
  to fix it. There are no silent fallbacks, best-effort parsing or skipped checks.
  A bound is enforced by refusing, never by truncating or sampling.
- **Prefer PurRDF.** Use the `purrdf.*` libraries for every reusable function they
  supply, in code, tests, examples and tools. Check the current PurRDF source before
  keeping a local implementation or choosing another dependency, wire the real
  callers through the native API, and remove the duplicate. A missing API is named
  precisely; it does not authorize a fallback, a parallel implementation or changed
  semantics. Keep a local adapter only for RAF-specific policy, and state that
  boundary.
- **Search before writing.** Use the census and the source index to find an existing
  solution before adding new code. The self-Census gate refuses a new duplicate.
- **Normal-form stability.** A normal form version is fixed by the content of its
  first merge to the default branch. After that, any change that can move a census
  fingerprint is a new version with a migration path: a `normal_form` bump, updated
  golden tests and `cargo census migrate`.
- **Report-form stability.** An additive optional property that is omitted at its
  default does not bump the report form; a change that alters existing bytes, or
  removes or renames something, does. See
  [`docs/architecture/normal-form.md`](docs/architecture/normal-form.md).

### Tests

- A permanent test runs on every change, forever, so it must earn its place. It
  protects a named behaviour, regression, boundary or failure path; its name states
  the behaviour and its comment states the invariant.
- **Every rule has a negative fixture.** A rule or check with no input that makes it
  fail is not enforced.
- **A refusal is a claim too.** When you add or tighten a refusal, run the case you
  believe is invalid and a neighbouring valid case, and confirm the second succeeds.
- Do not test getters, derives, compiler guarantees or private helper shapes, and do
  not mirror the implementation. Prefer fakes and in-memory trees to mocks. Extend
  an existing behavioural case before adding a new one, and remove tests that are
  outdated or overlap.
- A threshold in a test is not loosened to fit a number. The number is made to fit
  the threshold, or the design changes.
- Fixtures on disk live under the test's target directory (`CARGO_TARGET_TMPDIR`),
  never under a system temporary directory, and a consumer manifest written there
  carries its own `[workspace]` table.

---

## 7. The capability-document workflow

RAF plans its own work in the workplan format, and `tests/docs_contract.rs` enforces
it in `cargo test` and in the commit hook.

- **Capability documents** live in `docs/capabilities/<slug>.md`, one per
  capability. The frontmatter carries `type`, `id`, `title`, `description`, `state`
  (`considered`, `designed`, `implementing`, `delivered`), `bindings` and `depends`.
  The body has fixed sections: Purpose, Contract, Usage, Examples, Constraints,
  Explanation, Verification. Outside a proposal the text states the current contract
  only, including what is not delivered.
- **The inventory** is the `INVENTORY` table in `tests/docs_contract.rs`. Changing a
  state, adding a capability, or opening or completing a change edits that table in
  the same commit.
- **Bindings.** A claim names its tests with `raf:tested-by`, and the frontmatter
  binds code, tests and examples by path. A renamed test updates every binding.
- **Change records** live in `work/changes/<slug>.md`. Each names the capabilities
  and sections it changes and its items. Every item ends with an outcome:
  `implemented`, `dropped`, or `moved` to another tracked change.
- **Proposed text** is written into the capability document inside a
  `> [!PROPOSED change:<slug>]` block, so a diff shows the code beside the document
  it will produce.
- **Assessments** in `work/assessments/` record that a section was read against its
  inputs. One goes stale when its section or a bound symbol changes. Renew it with
  `cargo workplan assess` only after reading the section against the change, and
  restore the two licence comment lines that the renewal drops.
- **Completion is a gate.** `cargo workplan change complete` succeeds only when no
  obligation remains, every item has an outcome, no proposed block is left, the
  generated sections are in sync, and the bindings and affected assessments are
  current. It never runs tests or examples. The record is then deleted; Git keeps
  the history. A capability stays `implementing` while a record that owns it is
  open; nobody changes the state by hand.
- **Identifiers are slugs**, never numbers.

---

## 8. Repository rules

- **No tracker numbers or process references in any committed file.** Do not write
  issue or pull request numbers, tracker links, or wording about the development
  process that produced a file (plans, tasks, acceptance criteria, branches).
  Documents describe the system, not its history. `make issue-refs` scans every
  tracked text file, and the commit hook runs it on the staged index.
- **No deferrals.** Work is done now. Scope that leaves a change moves to another
  tracked change, with its own record, before the first one completes. Do not write
  "later", "follow-up" or "out of scope for now" in place of either.
- **Conventional commits.** `type(scope): summary` (`feat`, `fix`, `docs`,
  `refactor`, `test`, `ci`, `build`, `perf`, `chore`). The subjects feed `git-cliff`
  and become the release notes.
- **No agent attribution.** No agent credits, generated-by lines, signatures or
  co-author trailers in commits, files or pull requests.
- **Licensing.** Every `*.md` file is CC-BY-4.0 and opens with the two licence
  comment lines, as HTML comments, or as YAML comments inside the frontmatter. Code,
  configuration, templates and brand assets are
  `MIT OR Apache-2.0 OR MulanPSL-2.0`, with the lines in the file's own comment
  style. Files that cannot carry a header are annotated in `REUSE.toml`. Licence tags
  quoted in prose are wrapped in `REUSE-IgnoreStart` and `REUSE-IgnoreEnd` comments.
  `uvx reuse@6.2.0 lint` must pass. See [`LICENSING.md`](./LICENSING.md).
- **Releases.** One workspace version covers every crate. Tags are `v<semver>` and
  match the `tag_pattern` in `cliff.toml`. `CITATION.cff` and `[workspace.package]`
  carry the same version, which a gate checks. Do not publish, tag or push a release
  unless the maintainer asks. See [`CONTRIBUTING.md`](./CONTRIBUTING.md).
- **Scope.** Keep edits scoped and preserve unrelated work.
- **The deficiency ledger.** `.deficiencies` is the log of last resort for work that
  was misrepresented as done. Its normal contents are only the tracked notice and
  marker. Every entry below the marker is unauthorized, is a bug, and blocks
  completion, pull request creation and merge of the work that produced it. An entry
  is never an accepted risk, a backlog or a success with caveats. Remove one only
  after the defect is verified against current code and has a durable, visible
  owner; removing the entry does not resolve the bug. Never add ledger text to make
  incomplete work look complete.

---

## 9. Working practice

### Changes

- All work happens on a branch in a worktree under `.worktrees/` of the main
  checkout, and reaches the default branch through a reviewed pull request.
- **Only a merge to the default branch counts as done.** A commit, a branch or an
  open pull request is unfinished work. Land open work before starting more, and do
  not accumulate worktrees.
- Resolve a merge conflict by reading both sides. Never take one side whole.
- A change is reviewed independently before it merges. The reviewer gets the change
  and the requirement, never the author's conclusion, and reads the exact head that
  will merge. A finding is fixed, or shown from the code not to apply; a real gap
  that leaves the change is tracked before the merge.
- What an agent or a tool reports is a claim. Read the evidence it names before
  repeating it or building on it.

### Local data

- Notes, evidence, logs and scratch files go under `.stage/` in the repository, and
  Git work under `.worktrees/`. Nothing that must be kept goes in a system temporary
  directory, anywhere else on the host, or in another repository's tree.
- Evidence that serves no purpose is not kept. Remove a merged worktree and its
  build output.

### Commands

- One shell call does one thing. Do not chain unrelated steps.
- A deletion is its own call and names one exact path inside the repository. Remove
  worktrees with `stagectl cleanup` or `git worktree remove`. Never delete by glob or
  by variable, never use `git stash` or `git clean` in a shared checkout, and never
  discard uncommitted work.
- Searches and reads stay inside the repository, apart from instruction files that
  are named by their exact path.
- Do not issue a command that could raise a permission prompt, and never resend a
  refused call in another form.
- **One build at a time.** Serialize every cargo command, test suite and commit (the
  hook builds) with one lock, for example `flock /tmp/raf-build.lock cargo ...`. Do
  not work around the cargo wrapper; if a build is slow, wait.
- **A cargo command names its own tree.** Run it from the worktree it is meant for,
  or pass that worktree's `--manifest-path`. A result built from another directory
  is not evidence.
- **An exit status is the tool's own.** Send long output to a log under `.stage/`
  and read the status the tool returned. A status read through a pipe, or a line you
  echoed yourself, proves nothing.
- **Evidence comes from the gate's own command**, flag for flag. A hand-written
  command with other flags proves nothing about the gate.
- **Never wait on a watcher.** No `sleep`, `tail -f`, polling loop or second process
  that watches the first. Run the command in the foreground, or in the background
  and act on its completion. If you cannot name the event that will wake you, read
  the state once and act on it.
- A written time is read from the clock, never estimated.

### Delegation

- A brief is all a delegated agent knows. It states the goal, what is ruled out, the
  files to read, the exact scope, the worktree path and the rules that bind it,
  including the no-timing and no-mutation rules.
- One agent at a time writes in a worktree. Read-only work such as research and
  review runs beside it.
- The final check of a change that is about to merge is a reading of the merging
  diff itself, after the independent review and not in place of it.

### Research

- Use the web for upstream documentation, issue trackers and the state of the art
  when it helps; prefer that to guessing or re-deriving what is published. A query
  carries the technical question, never private data.
- Research is not copying. What you read informs the work, and what you write is
  your own: no copied or closely paraphrased code, definitions or prose from another
  work.
