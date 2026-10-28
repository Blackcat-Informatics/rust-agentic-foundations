<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Contributing to RAF

Thanks for your interest in RAF (Rust Agentic Foundations). This repository holds
the libraries and cargo subcommands that Blackcat projects use as dev-dependencies:
`raf-okf`, `raf-source`, `raf-census`, `raf-rdf`, `cargo-census`, `raf-workplan` and
`cargo-workplan`, together with the capability documents that specify them. Issues
and pull requests are welcome.

## Ways to contribute

- **Report a bug or request a feature** — open an issue with a minimal reproduction
  (ideally a small Rust source tree, ledger or capability document that the gate
  mishandles).
- **Fix a bug or add a feature** — open a pull request against `main`.
- **Improve the docs** — corrections and clarifications to the capability documents
  in [`docs/`](./docs/) are very welcome.

## Design constraints that PRs must respect

- **Normal-form stability.** A census fingerprint is a promise: the same code
  hashes to the same value across releases, so ledgers and recorded debt stay
  valid. A normal form version is fixed by the content of its first merge to the
  default branch. After that, any change that can move a fingerprint is a new
  version with a migration path: a `normal_form` bump, updated golden tests, and
  `cargo census migrate`.
- **Strict schemas and hard failures.** Every ledger, manifest and document schema
  is closed: unknown keys are errors, not warnings. A stale, malformed or ambiguous
  input fails with a message that says how to fix it. Do not add silent fallbacks,
  best-effort parsing or skip-on-missing-tool behavior.
- **Dogfood gates.** RAF is the first user of its own tools. The root package's
  test suite carries the repository's own gates: a document-contract test that
  runs the `raf-workplan` checks over the capability, change and architecture
  documents, and the repository gates (licence digests, issue-reference scan,
  version agreement), all in Rust. The self-Census gate checks the workspace and
  fuzz roots against the canonical ledger and exact adopted debt. A change that makes a gate fail on RAF itself does not merge, and
  gates are never relaxed to get a change through. `make docs-sync` rewrites the
  generated documentation blocks that the workplan checks find stale.
- **The workplan completion gate.** A change is complete only when the capability
  documents state the delivered contract: no proposed block remains, every item has an
  outcome (`implemented`, `dropped`, or `moved` to another tracked change), the
  generated sections are in sync, and the bound tests and examples pass. The change
  record is then deleted; Git keeps the history.
- **Every rule has a negative fixture.** A detector, schema rule or check without an
  input that makes it fail is not enforced.
- **Rust first.** Checks, generators, gates and tests are written in Rust. The
  Makefile and hooks are thin glue over them.

## Development

```bash
make check   # fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, licenses, issue-refs
make hooks   # install the pre-commit and pre-merge-commit hooks (core.hooksPath = .githooks)
make help    # list every target
```

- **`make check` is the gate.** It stops at the first failure. CI runs the
  documented compilation, test and policy checks; local `make check` additionally
  runs optimized instrumented fuzzing. The pre-commit hook runs fmt and reuse on a checkout of the staged
  index, plus the `repo_gates`, `docs_contract`, `self_workplan` and `self_census` test binaries over
  that checkout. The binaries are compiled from the working tree, not the index.
  A `pre-merge-commit` hook runs the same gates on merge commits. Never bypass a
  hook.
- **Hooks follow the checkout.** With `core.hooksPath` set to `.githooks`,
  committing on a checked-out branch runs that branch's copy of the hook, so a
  hook change takes effect on the branch that carries it.
- **Native execution prerequisites.** Install `bubblewrap` for the `bwrap`
  executable, Git 2.36 or newer and OpenSSH's `ssh-keygen`. The host must support
  the required mount namespace. Fixtures create and verify an ephemeral SSH
  signing authority inside their Git administration; no maintainer private key
  is used. Ordinary Git-backed opening and tracked-oracle queries retain their
  Git 2.30 minimum; completed views need NUL-delimited worktree inventory.
  The ephemeral Linux test job installs these
  packages and an AppArmor profile attached to `/usr/bin/bwrap` that permits
  namespaces for bubblewrap and its nested trusted tests. Other execution trees
  retain Ubuntu's global restriction. The job checks native namespace execution
  before running the suite. Missing tools or capabilities
  are failures; no original-root fallback or skipped completion case applies.
- **Bound validation.** The self-workplan gate checks the native execution plan
  of every test and example binding without running it. Package jobs execute
  ordinary tests once, including doctests, and `raf-ci` runs documented standalone
  examples once per successful native plan. Workplan uses two exhaustive
  partitions of Cargo metadata test targets, with doctests and normal example
  compilation once on the second partition. Full `cargo workplan verify` is an
  explicit runtime audit. Native capability completion still executes its affected
  bindings and refuses missing, ignored or zero-test proof. A host-inapplicable
  binding never counts as an executed example.
- **CI budget.** Every job has a nine-minute execution budget, reserving headroom
  below the ten-minute job ceiling. Source, fixture,
  capability-document and build changes select every workspace package; unrelated
  text changes retain root repository gates. Compiler and harness lint jobs apply
  to source and asset changes. Licensing and both dependency-policy graphs are
  checked on every run. The native YAML gate rejects an absent or excessive job
  budget, including expression-based values whose bound cannot be established.
- **MSRV.** `make msrv` runs `cargo check --workspace --all-targets --locked` (and the
  same on the fuzz manifest) on Rust 1.98.1 through `rust-validate`. Install `rust-validate`, or point `MSRV_RUN`
  at a command that runs the 1.98.1 toolchain.
- **Fuzz harness.** `make fuzz` checks `fuzz/` (its own cargo-fuzz workspace) with
  fmt, clippy and an offline `cargo deny`, then runs each of its targets
  (`parse-source`, `parse-document`, `load-workspace`, `census-normalize` and `census-report-read`) over its committed corpus,
  with inputs of up to 400000 bytes and limits on resident memory and on one
  allocation. It needs `cargo-fuzz` (`cargo install --locked
  cargo-fuzz@0.13.2`); a missing tool is a failure. `make msrv` and `make fmt` cover
  the fuzz manifest too. The required local `make check` retains all five
  optimized instrumented targets, their corpora, assertions, overflow checks and
  mutation counts, using sixteen code generation units. Hosted CI checks harness
  Clippy, compiler-floor support and both dependency-policy graphs. It produces
  no sanitizer execution receipts. Ordinary Source tests replay parsing and
  workspace seeds; they do not replace complete fuzz runtime coverage.
- **Native validation.** `make test-release` runs the `raf-source` tests in release
  mode. Standard checks and CI use Linux only, including only its native disk
  strategy. Cross-platform testing and cross-compilation are forbidden. Other
  platform adapters remain in source without standard validation claims.
- **REUSE.** `make reuse` runs `uvx reuse@6.2.0 lint`, so it needs `uv` (or set
  `REUSE=reuse` with `reuse` installed). A missing tool is a failure, not a skip.
- **Supply chain.** `make deny` runs `cargo deny` offline on your machine; CI runs
  the fetching variant against the live advisory database.
- **Issue references.** `make issue-refs` fails on issue or pull request numbers and
  development-process wording in any tracked text file. Documents describe the
  system, not the history that produced it. It and `make licenses` run the Rust
  gates in `tests/repo_gates.rs`, so they are a subset of `make test`.
- **No clock in tests.** The `time_in_crates_` gate in `tests/repo_gates.rs` fails
  when Rust under the `src`, `tests` or `examples` of any crate in `crates/`, or
  under the root `tests/`, names a clock reading or a wait on one (`Instant`,
  `SystemTime`, `UNIX_EPOCH`, `sleep`, the `*_timeout` waits, and each of these
  with a `_` suffix such as `sleep_ms` or `wait_timeout_while`; `raf-source` also
  refuses `Duration` and `elapsed`). Only `crates/raf-workplan/src/exec/clock.rs`
  and the gate file are exempt. The examples of `raf-source` are judged by the same
  rule as its sources and tests, and `make clippy` also disallows the same types,
  the sleeps and the timeout waits everywhere an `allow` with a reason does not
  say otherwise. Script a clock in the test instead. A test that runs a real process
  still goes through the runner's wall clock as a watchdog; it asserts nothing
  about the time.

## Versioning & releases

**Semantic versioning.** A breaking API change requires a **major** bump; a
**minor** bump is additive and API-compatible, and a **patch** bump is bugfix-only.
Until 1.0.0 a breaking change bumps the minor version. All crates, the
`[workspace.package]` version, and `CITATION.cff` share **one** version.

**MSRV.** The supported floor is `rust-version` in `Cargo.toml` (currently
**1.98.1**) on the **stable** channel, enforced by `make msrv` and in CI. It is
what you build against as a consumer; it is unaffected by the toolchain
contributors run. Raising the MSRV is a notable, changelog-recorded change that
rides a minor bump and never ships in a patch release.

**Development toolchain.** `rust-toolchain.toml` names a **floating nightly** for
local work and CI gates, because nightly clippy and rustdoc carry lints stable
lacks, so a gate finding is a real finding, not a channel artifact. It floats rather
than naming a date because a date would freeze that surface and turn every later
sharpening into debt nobody sees. The *source* uses no unstable features: there
are no `#![feature(...)]` attributes anywhere and adding one is rejected, which is
exactly what the MSRV build proves.

**Changelog and tags.** Write **conventional-commit messages**
(`type(scope): summary`, for example `feat(census): ...` or `fix(source): ...`).
`git-cliff` turns them into the release notes via [`cliff.toml`](./cliff.toml), so
their wording becomes the changelog. Releases are tagged `v<semver>` (for example
`v0.1.0`), matching the `tag_pattern` of `v[0-9]*` in `cliff.toml`.

## Before you open a pull request

- Run `make check` and make sure it is green.
- Install the hooks with `make hooks`.
- `cargo clippy --workspace --all-targets` must be warning-free. The workspace lint
  table (clippy pedantic; `missing_docs`, `unreachable_pub` and `unsafe_code`
  denied) is inherited by every crate, and CI denies warnings. Prefer fixing code
  over adding `#[allow]`; when an allow is genuinely right, scope it tightly and
  give it a reason.
- Every file must carry an SPDX header: `MIT OR Apache-2.0 OR MulanPSL-2.0` for
  code, configuration and templates, and `CC-BY-4.0` for every Markdown document.
- Reconcile the capability documents with the change, and bump `normal_form` if a
  fingerprint changed.
- Keep changes focused; describe **what** changed and **why** in the PR description.
- Do not put issue or pull request numbers in committed files.

## Licensing of contributions

Contributions to **RAF** are accepted under **MIT OR Apache-2.0 OR
MulanPSL-2.0** and, under the project CLA, under terms that permit separate
proprietary/commercial licensing. Contributed documentation is accepted under
CC-BY-4.0.

By submitting a contribution you agree to license it under the terms above. For the
dual-licensing reservation to extend to your contribution, you agree to license it to
Blackcat Informatics® Inc. under terms that permit relicensing, including under
proprietary terms. A Contributor License Agreement (CLA) may be required before
substantial contributions are merged. See [`LICENSING.md`](./LICENSING.md) for the
full licensing scheme.

## Conduct

Be respectful and constructive — see [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).
