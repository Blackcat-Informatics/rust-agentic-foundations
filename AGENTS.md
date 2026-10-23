<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# AI Developer Agent Guide (AGENTS.md)

This file is the behavioral contract for agents and humans working in the RAF
(Rust Agentic Foundations) repository. [`CLAUDE.md`](./CLAUDE.md) defers to it.

---

## Deficiency emergency ledger (non-negotiable)

`.deficiencies` is the log of last resort for critically undone work. Every entry
below its marker is **100% unauthorized**, is **100% a bug**, and means the change
that produced it failed. It is used for work misrepresented to pass gates, work
misrepresented by an agent, or a discovery that an agent was fundamentally
defective. An entry is never an accepted risk, authorized descope, backlog, or
success with caveats.

The only normal contents are the tracked notice and marker, with no entries below
them. An entry blocks completion, pull request creation, and merge of the work that
produced it. Verify the defect against current code and give it a durable, visible
remediation owner before removing the entry. Removing an entry does not resolve the
bug. Never add ledger text to make incomplete work appear complete.

---

## 1. What this repository is

RAF is a set of Rust libraries and cargo subcommands that Blackcat projects use as
**dev-dependencies** to keep a code base and its documentation honest. Two tools
carry the weight:

- **Census** finds structurally duplicated code under a versioned normal form and
  holds it against a ledger.
- **Workplan** plans changes to the documented system, reconciles the documents
  with what was built, and refuses to complete a change until the capability
  documents state the delivered contract.

The repository dogfoods both. Its own design is written in the workplan format,
and the root package's test suite carries its gates. Today those are the
document-contract test (`tests/docs_contract.rs`), which runs the workplan checks
of `raf-workplan` over the tracked documents, the self-workplan test
(`tests/self_workplan.rs`), which runs the workplan gate from disk and requires
every assessment to be current and accepted, and the repository gates
(`tests/repo_gates.rs`: licence digests, issue-reference and process-wording
scan, version agreement, and the clock gate below); the census gate joins them when the census
`Gate` is built.

### Crates

`raf-okf`, `raf-source`, `raf-census`, `raf-rdf`, `raf-workplan` and `cargo-workplan` are workspace members. `raf-okf` and
`raf-source` are **delivered**: the frontmatter reader, the deterministic writer,
the body model and the profile validation of `raf-okf`, and the `Tree` trait of
`raf-source` with its disk and memory backends and path containment, its bounded
parse, package and target discovery, its module tree and its symbol and use-alias
index, read from the modules of every target, its impl blocks and the reachability
of items. `raf-workplan` delivers the document model (loading, the graph, binding
resolution, the check rules and the `Gate`), the generated blocks with their sync
and preview, the change lifecycle (`change new`, `impact`, `reconcile`,
`complete` and `verify`), entity history, execution evidence and its claims,
prose assessments, two-way coverage and the current-rationale check of the
architecture pages. `cargo-workplan` is **delivered** with the commands `check`,
`status`, `docs sync|preview`, `change new|impact|reconcile|complete`, `verify`,
`history`, `evidence record|claims|for`, `assess` and `coverage code|tests`, their
contract stated in the command-line capability. The complete policy
([workplan-strict](docs/capabilities/workplan-strict.md)) is **delivered**.
Every check and gate applies one policy for delivered sections, package and item
ownership, claim coverage with `raf:tested-by` and `coverage claims`, and change
completion with stated outcome reasons, literal `contract_effect: none` and owed
Usage and Examples edits. Reverse coverage is available through `coverage lines`
(also `coverage --reverse`), with the coverage record's ceilings and exceptions;
`debt --write` explicitly records debt that may only shrink. These rules apply
to every check, reconciliation and completion without an opt-in mode.
`raf-census` is **delivered** for its normal-form capability: the normaliser, full fingerprints, units, grouping, job matching and canonical RDF 1.2 baseline and index reports. Its remaining capabilities state their designed contracts. `cargo-census` is
**designed** and joins the workspace members list when it is built. The capability documents in `docs/capabilities/` are the specification.

| Crate | Role | Status |
|---|---|---|
| `raf-okf` | Reader and writer for Markdown with YAML frontmatter, and profile validation with closed key sets | delivered |
| `raf-rdf` | Closed canonical ground RDF 1.2 report profile over native PurRDF storage | delivered |
| `raf-source` | `syn` workspace walker: module tree, `#[path]`, symbol and `use`-alias index, impl blocks, item reachability, `Tree` trait with disk and memory backends | delivered |
| `raf-census` | Normal form v1, fingerprints, grouping, ledger, detectors, `Rule` trait, patterns, debt ratchet, reference scan | delivered for normal form v1, fingerprints, grouping and the reports |
| `cargo-census` | Thin command-line wrapper over `raf-census` | designed |
| `raf-workplan` | Capability and change model, graph, bindings, generated sections, obligations, completion gate | delivered |
| `cargo-workplan` | Thin command-line wrapper over `raf-workplan` | delivered |

`raf-source` is shared on purpose: the census needs its symbol index to resolve
ledger homes, and the workplan needs the same index to resolve code and test
bindings.

### Layout

| Path | Contents |
|---|---|
| `crates/` | The workspace member crates (`raf-okf`, `raf-source`, `raf-census`, `raf-rdf`, `raf-workplan` and `cargo-workplan`; each other crate joins `members` when it is built) |
| `docs/capabilities/` | One permanent capability document per capability |
| `docs/architecture/` | Shared current decisions (normal form, OKF profile, debt ratchet) |
| `work/changes/` | In-flight change records |
| `work/debt/` | The strict debt record `work/debt/default.md` of [strict mode](docs/capabilities/workplan-strict.md), written only by `cargo workplan debt --write` |
| `docs/` | Licensing translations and brand material |
| `.githooks/`, `.github/` | The pre-commit and pre-merge-commit hooks, CI workflow, Dependabot configuration, issue and pull request templates |

---

## 2. Toolchain

- `rust-toolchain.toml` names a **floating nightly** for development and CI gates.
  Nightly clippy and rustdoc carry lints stable lacks, so a gate finding is a real
  finding. It floats rather than naming a date so later sharpening is never frozen
  into invisible debt.
- The **minimum supported Rust version is 1.98.1** (`rust-version` in the root
  `Cargo.toml`), enforced on the stable channel. `make msrv` runs
  `cargo check --workspace --all-targets --locked` through `rust-validate 1.98.1`,
  and the same check on the fuzz harness manifest.
- The source uses **no unstable features**. A `#![feature(...)]` attribute is
  rejected; the MSRV build proves it on every change.
- Raising the MSRV is a changelog-recorded change that rides a minor bump.
- **Path containment is per platform.** `raf-source` verifies every handle it uses
  against the workspace root, before the use and again after it, and reads a file's
  contents only through a verified handle. On Linux a candidate is first taken as an
  `O_PATH` reference and the kernel's path for it (`/proc`) must lie under the root
  before it is opened for reading or listing through that path, so an outside
  candidate is never opened for reading or listing. On other Unix systems the path
  is walked beneath a descriptor of the root with `openat` (through `rustix`), one
  component at a time and never following a symlink, so no path is resolved by name
  and an outside candidate is never opened; a walk outside the root may only
  descend back into it, never climb, so a link cannot leave the root and return. On
  Windows the path the system gives for the handle (`GetFinalPathNameByHandleW`,
  through `windows-sys`, in the library's one `unsafe` block) must lie under the
  root; an outside file can be opened in a race and is then refused before any
  read, and a listing or an entry lookup goes by the handle's path, so a swap made
  and undone between the two checks can return outside names or kinds, never
  contents. Windows also refuses reserved device names, names ending in a dot or a
  space, and names holding a colon. `make cross-check` compiles every platform's
  strategy and tests (macOS, FreeBSD and Windows); a Linux host runs the other two
  strategies through a test override, and the macOS and Windows CI jobs run the
  native ones there.

---

## 3. Gates

```bash
make check   # fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, cross-check, licenses, issue-refs
make hooks   # point core.hooksPath at .githooks for this clone
```

- **`make docs-sync` rewrites the generated documentation blocks** and creates
  `docs/capabilities/index.md` when it is missing. It is not part of `make check`:
  it is the repair for the stale-block finding of `Workplan::check`.
- **Run `make check` before claiming work is done.** It stops at the first failure.
- **Install the hooks with `make hooks`.** The pre-commit hook runs fmt and reuse
  on a checkout of the staged index, and the `repo_gates`, `docs_contract` and
  `self_workplan` test binaries over that checkout. The binaries are compiled from the working
  tree, not the index; only the files they judge come from the index. A
  `pre-merge-commit` hook runs the same gates on merge commits.
- **`make licenses` and `make issue-refs` run the Rust gates** in
  `tests/repo_gates.rs`, so they are a subset of `make test`. The pattern set
  lives once, in `raf_workplan::refs`, which `raf-workplan` also applies to the
  documents it loads; the gate test files call it and share one file list in
  `tests/support/mod.rs`.
- **No test asserts time.** The `time_in_crates_` gate in `tests/repo_gates.rs`
  fails when a tracked Rust file under the `src`, `tests` or `examples` of any
  crate in `crates/`, or under the root `tests/`, names a clock or a wait on one;
  the directories come from `crates/*`, so a new crate is judged without a change
  to the gate. `raf-source` refuses `std::time`, `core::time`, `tokio::time`,
  `Instant`, `Duration`, `SystemTime`, `sleep`, `elapsed` and the `recv_timeout`,
  `wait_timeout`, `park_timeout`, `set_read_timeout` and `set_write_timeout`
  calls. The other crates refuse the same set without `std::time`, `Duration` and
  `elapsed` (the runner's limits and scripted data are made of them) and add
  `UNIX_EPOCH`. A name with a `_` suffix counts as the name, so `sleep_ms`,
  `sleep_until`, `park_timeout_ms`, `wait_timeout_ms` and `wait_timeout_while` are
  refused too. The `examples` of `raf-source` are judged by the same stricter rule
  as its `src` and `tests`. Two files are exempt:
  `crates/raf-workplan/src/exec/clock.rs`, the one clock reading, and
  `tests/repo_gates.rs`, which defines the words; the gate fails if either stops
  being tracked or stops doing that. `clippy.toml` also disallows
  `std::time::Instant`, `std::time::SystemTime`, the `std::thread` sleeps and
  `park_timeout` waits, the `Condvar` timeout waits and `Receiver::recv_timeout`,
  so `make clippy` rejects a new reading anywhere an `allow` with a reason does
  not say otherwise. A test that runs a real process through `SystemRunner::run`
  still goes through the runner's wall clock, as a watchdog for its limit; no
  test asserts what that clock reads.
- **Never bypass a hook or a gate.** Do not pass `--no-verify`, do not edit a hook
  to pass, and do not weaken a gate or a lint to get green. Fix the cause.
- **Python files are forbidden.** The repository gates reject tracked Python
  source, stubs, bytecode and executable archives (`.py`, `.pyw`, `.pyi`, `.pyc`,
  `.pyo`, `.pyz`, `.pyzw`, case-insensitively), and scripts with Python or PyPy
  interpreter shebangs, including extensionless scripts. The commit and merge
  hooks judge the staged snapshot; CI applies the same Rust gate. Implement
  repository tooling in Rust.
- A missing tool is a hard failure, not a skip. `make reuse` needs `uvx` or
  `reuse`; `make msrv` needs `rust-validate`; `make fuzz` needs `cargo-fuzz`
  (`cargo install --locked cargo-fuzz@0.13.2`); `make cross-check` needs the
  `x86_64-apple-darwin`, `x86_64-unknown-freebsd` and `x86_64-pc-windows-gnu`
  targets (`rustup target add x86_64-apple-darwin x86_64-unknown-freebsd
  x86_64-pc-windows-gnu`).
- `make test-release` runs the `raf-source` tests in release mode, because the stack
  a nesting level costs the parser differs between profiles. `make fuzz` checks the
  fuzz harness in `fuzz/` (its own workspace) with fmt, clippy and deny, then runs
  each of its targets over its committed corpus. `make cross-check` runs
  `cargo check --all-targets` of `raf-source`, warnings denied, for the three
  targets above. `make msrv` and `make fmt` also cover the fuzz harness manifest.
- `make deny` runs offline locally; CI fetches the advisory database.

### Workspace lints

The root `Cargo.toml` declares `[workspace.lints]`, and every crate inherits them
with `[lints] workspace = true`:

- `unreachable_pub`, `missing_docs` and `unsafe_code` are denied. Use `pub(crate)`
  for items that are not API. Library code has one `unsafe` block, the Windows
  `GetFinalPathNameByHandleW` call in `crates/raf-source/src/tree/disk/final_path.rs`,
  under an item-scoped `#[allow(unsafe_code)]` with its reason and a `SAFETY`
  comment. Allocation-bound tests use the native counting allocator through
  its safe measurement API.
- The fuzz harness in `fuzz/` is its own workspace and mirrors the same
  `[workspace.lints]` tables; `tests/repo_gates.rs` fails when they differ, and
  likewise when `fuzz/deny.toml` differs from `deny.toml` by more than its one
  crate-scoped exception.
- Clippy `all` and `pedantic` warn, and CI denies warnings.
- `allow_attributes_without_reason` is denied: every `#[allow]` states why, scoped
  as tightly as possible. Prefer fixing the code.

---

## 4. Licensing and SPDX

- **Every `*.md` file is CC-BY-4.0.** It opens with the two HTML-comment SPDX lines
  (copyright, then the CC-BY-4.0 licence identifier), exactly as the existing
  documents do.
  Markdown with YAML frontmatter carries them as YAML comments inside the frontmatter.
- Code, configuration, templates and brand assets are
  `MIT OR Apache-2.0 OR MulanPSL-2.0`, with the SPDX lines in the file's own comment
  style. Files that cannot carry a header are annotated in `REUSE.toml`.
- SPDX tags quoted in prose are wrapped in `REUSE-IgnoreStart` and
  `REUSE-IgnoreEnd` comments so `reuse lint` does not read them as headers.
- `uvx reuse@6.2.0 lint` must pass. See [`LICENSING.md`](./LICENSING.md).

---

## 5. Principles

- **Rust first.** Checks, generators, gates and tests are written in Rust: a
  library, a test, or an `xtask`. Shell is limited to thin glue such as the
  Makefile and hooks.
- **Prefer PurRDF throughout RAF.** Use the existing `purrdf.*` libraries for
  every reusable function they supply, including code in tests, examples and
  tools. Inspect the current `main` source in `~/Active/purrdf` before retaining
  a local implementation or selecting another dependency. Wire the real callers
  through the native API and remove the duplicated implementation. A missing
  required API must be identified precisely; it does not authorize an unstated
  fallback, a parallel implementation or silently changed semantics.
  Keep local adapters only for RAF-specific policy or a required contract that
  the public native API cannot supply, and state that exact boundary. This rule
  governs engineering choices; it is not a claim of Census provider enforcement.
- **Fail hard.** Schemas are strict and closed: unknown keys are errors. A ledger or
  document that is stale, malformed or ambiguous is an error with a message that
  says how to fix it. No silent fallbacks, no best-effort parsing, no skipped checks.
- **Normal-form stability.** A normal form version is fixed by the content of its
  first merge to the default branch. After that, any change that can move a census
  fingerprint is a new version with a migration path: a `normal_form` bump, updated
  golden tests, and `cargo census migrate`.
- **Every rule has a negative fixture.** A rule or check with no input that makes it
  fail is not enforced.
- **A refusal is a claim too.** When you add or tighten a refusal, run the case you
  believe is invalid and a neighbouring valid case, and confirm the second succeeds.

---

## 6. The capability-doc workflow

RAF plans its own work in the workplan format. The document-contract test
(`tests/docs_contract.rs`) enforces the format on every capability, change and
architecture document in `cargo test` and in the pre-commit hook. It loads the
tracked documents into `raf-workplan`, which reads them through `raf-okf`, resolves
their bindings through `raf-source` and runs the cross-document rules, and it adds
this repository's own policy, such as the capability inventory.

- **Capability documents** live in `docs/capabilities/<slug>.md`, one per
  capability. Frontmatter carries `type`, `id`, `title`, `description`, `state`
  (`considered`, `designed`, `implementing`, `delivered`), `bindings` and `depends`.
  The body has fixed sections: Purpose, Contract, Usage, Examples, Constraints,
  Explanation, Verification. Outside a proposal, the text states the current
  contract only. The document-contract test pins the set of capabilities in one
  inventory table (`INVENTORY` in `tests/docs_contract.rs`): changing a state,
  adding a capability, or opening or completing a change edits that table in the
  same commit. `make docs-sync`
  rewrites the generated blocks, including the catalogue in
  `docs/capabilities/index.md`.
- **Change records** live in `work/changes/<slug>.md`. Each names the capabilities it
  changes, the sections it affects, and its items. Every item ends with an outcome:
  `implemented`, `dropped`, or `moved` to another tracked change.
- **Proposed text** is written into the capability document itself inside a
  `> [!PROPOSED change:<slug>]` block, so a diff shows the code and the document it
  will produce side by side.
- **Completion** is a gate. A change is complete only when no obligation remains,
  every item has an outcome, no proposed block is left, generated sections are in
  sync, and the bound tests and examples pass. The change record is then
  **deleted**; Git keeps the history. The command that enforces this is
  `cargo workplan change complete`
  ([workplan completion gate](docs/capabilities/workplan-complete-gate.md)), and
  the records of this repository are completed with it.
- **Identifiers are slugs**, never numbers. Documents never refer to a tracker.

---

## 7. Repository rules

- **No tracker numbers or process references in any committed file.** Do not write
  issue or pull request numbers, tracker links, or wording that refers to the
  development process that produced a file (plans, tasks, acceptance criteria,
  branches). Documents describe the system, not its history. `make issue-refs`
  scans every tracked text file and fails on a hit; the pre-commit hook runs it
  on the staged index.
- **No deferrals.** Work is done now. Scope that leaves a change is moved to another
  tracked change, with its own record, before the first one completes. Do not write
  "later", "follow-up" or "out of scope for now" as a substitute for either.
- **Conventional commits.** `type(scope): summary` (`feat`, `fix`, `docs`, `refactor`,
  `test`, `ci`, `build`, `perf`, `chore`). The subjects feed `git-cliff`, so they
  become the release notes.
- **No agent attribution.** No agent credits, generated-by lines, signatures, or
  co-author trailers in commits, files or pull requests.
- Keep edits scoped, preserve unrelated work, and do not publish, tag or push a
  release unless the maintainer asks.

---

## 8. Releases

One workspace version covers every crate. Tags are `v<semver>` and match the
`tag_pattern` in `cliff.toml`. `CITATION.cff` and `[workspace.package]` carry the
same version, which `tests/repo_gates.rs` checks. See
[`CONTRIBUTING.md`](./CONTRIBUTING.md) for the policy.
