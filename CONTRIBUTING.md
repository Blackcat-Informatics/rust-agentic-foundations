<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Contributing to RAF

Thanks for your interest in RAF (Rust Agentic Foundations). This repository holds
the libraries and cargo subcommands that Blackcat projects use as dev-dependencies:
`raf-okf`, `raf-source`, `raf-census`, `raf-rdf`, `cargo-raf-census`, `raf-workplan` and
`cargo-raf-workplan`, together with the capability documents that specify them. Issues
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
  `cargo raf-census migrate`.
- **Strict schemas and hard failures.** Every ledger, manifest and document schema
  is closed: unknown keys are errors, not warnings. A stale, malformed or ambiguous
  input fails with a message that says how to fix it. Do not add silent fallbacks,
  best-effort parsing or skip-on-missing-tool behavior.
- **Dogfood gates.** The private `raf-docs-contract` package runs Workplan over
  the capability, change and architecture documents. The private root package
  checks repository rules such as licence digests, issue references and version
  agreement. Its self-Census gate checks the workspace and fuzz roots against the
  canonical ledger and exact adopted debt. A change that makes a gate fail on
  RAF itself does not merge. `make docs-sync` rewrites stale generated documentation
  blocks.
- **The workplan completion gate.** A change is complete only when the capability
  documents state the delivered contract: no proposed block remains, every item has an
  outcome (`implemented`, `dropped`, or `moved` to another tracked change), the
  generated sections are in sync, and source bindings and assessments are valid. The change
  record is then deleted; Git keeps the history.
- **Every rule has a negative fixture.** A detector, schema rule or check without an
  input that makes it fail is not enforced.
- **Rust first.** Checks, generators, gates and tests are written in Rust. The
  Makefile and hooks are thin glue over them.

## Development

The [supported-target policy](docs/TARGETS.md) limits implementation and
qualification through the 0.9 release to the primary native Linux target.
Other operating systems and secondary compilation targets are subject to
reassessment after that release. Native Cargo target kinds and the
stable MSRV and nightly toolchains remain covered.

```bash
make hooks   # install the commit, merge-commit and pre-push hooks (core.hooksPath = .githooks)
make check   # static checks, all-target lints, compiler, policy and fuzz gates
make test    # run the workspace behavioral suites with fresh Census evidence
make help    # list every target
```

The trusted documentation integration fixtures use Cargo's test target
directory. Run them with an absolute `CARGO_TARGET_DIR` outside Git
worktrees when Cargo defaults to an in-checkout target.

The `self_census` integration test checks the workspace and fuzz roots
against the ledger and exact debt record. It shares the required syntax
rules and locked registry configuration with the `repository_census`
example. The test is read-only. The example can publish debt on an
explicit write request. The consumer guides show how to set up each gate
in another workspace. [Census how-to](CENSUS-HOWTO.md) · [Workplan how-to](WORKPLAN-HOWTO.md)

- **Staged static gate.** Run `make precommit` to check the effective staged
  tree. It requires the installed hooks and checks formatting,
  shipping lints, dependency bans and sources, licence headers (`make reuse`),
  tracked text, and Markdown links.
  Both commit hooks run this target. Repository policy, Workplan and Census
  tests run in the merge and CI gates.
- **Push gate.** The `pre-push` hook exports each pushed commit with
  `git archive` and runs the static Workplan check (source bindings, coverage
  and assessments) and all-target Clippy with warnings denied, the flags of the
  hosted lint job. It checks the pushed tree, not the worktree, and refuses the
  push on any finding. Expect about two minutes on a cold Clippy cache; the
  commit hook stays under its 60-second budget because these checks run here.
- **Merge gate.** `make merge-gate` runs `make check` and `make test` in
  order on the exact signed candidate. Supply an absolute `CARGO_TARGET_DIR`
  outside the repository. The [gate declaration](.ghprsq.toml) takes effect
  from the base commit. The merge tool records the gate's exit status and
  can reuse a pass for the same tree and command.
  Hosted CI checks compilation, policy and the workspace tests. The local
  gate also builds benchmarks and runs instrumented fuzzing.
- **Production lint coverage.** `make clippy` checks the locked workspace
  libraries and binaries. A private helper whose only caller is a test can
  fail this check. `make clippy-all` includes test, example, benchmark and fuzz
  targets. The Census gate and `make test` use `make clippy-production` for
  fresh receipts with the same shipping flags. The production lint helper
  invokes the same trusted `lint-capture` command available to an installed CLI.
  [Production lint helper](.githooks/production-clippy)
- **Hooks follow the checkout.** With `core.hooksPath` set to `.githooks`,
  committing on a checked-out branch runs that branch's copy of the hook, so a
  hook change takes effect on the branch that carries it.
- **Runtime and fixture authority.** Analysis libraries start no processes and
  require no namespace, PID supervision or executable Git helper. Git reads use
  a native read-only adapter. The maintainer-controlled Git fixtures use installed
  Git and OpenSSH ssh-keygen to create and verify ephemeral signatures inside
  their own temporary repositories; they never use a maintainer private key.
  No host policy override or elevated permission is required.
- **Static validation.** Verification and completion validate source bindings,
  document contracts, prospective generated output and current assessments.
  They never initiate consumer tests, examples, doctests or arbitrary commands.
  Maintainer-controlled RAF verification uses fixed fixtures as data and retains
  the mandatory gates below. Private documentation compilation requires explicit
  --trust-docs on each invocation, accepting arbitrary build-script and
  procedural-macro effects without a sandbox guarantee.
- **CI budget.** Every job has a nine-minute execution budget, and the hosted
  `test-release` job has fifteen minutes; the workflow gate refuses any job
  budget above fifteen minutes. Source, fixture,
  capability-document and build changes select every workspace package; unrelated
  text changes retain the root repository gates and the document contract. Compiler and harness lint jobs apply
  to source and asset changes. Licensing and both dependency-policy graphs are
  checked on every run. The native YAML gate rejects an absent or excessive job
  budget, including expression-based values whose bound cannot be established.
- **MSRV.** `make msrv` binds Cargo to Rust 1.98.1 and checks all targets in
  both locked workspaces. Dead code and other compiler warnings fail their
  compilation target. The audit finds direct dependencies that none of the
  package's eligible targets use. Install `rust-validate`, or point `MSRV_RUN`
  at a command that runs the 1.98.1 toolchain.
  The checks deny compiler warnings. Cargo resolves dependencies for the
  package and supplies them to individual targets. The dependency audit
  collects the compiler's findings across those targets and fails if every
  target that receives a dependency leaves it unused.
- **Fuzz harness.** `make fuzz` checks `fuzz/` (its own cargo-fuzz workspace) with
  fmt, Clippy and `cargo deny`, then runs each of its targets
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
- **Supply chain.** `make deny` and CI check both locked graphs online.
  `make precommit` checks dependency bans and sources in both workspaces.
- **Issue references.** `make issue-refs` fails on issue or pull request numbers and
  development-process wording in any tracked text file. Documents describe the
  system, not the history that produced it. It and `make licenses` run the Rust
  gates in `tests/repo_gates.rs`, so they are a subset of `make test`.
- **No clock in tests.** The `time_in_crates_` gate in `tests/repo_gates.rs` fails
  when Rust under the `src`, `tests` or `examples` of any crate in `crates/`, or
  under the root `tests/`, names a clock reading or a wait on one (`Instant`,
  `SystemTime`, `UNIX_EPOCH`, `sleep`, the `*_timeout` waits, and each of these
  with a `_` suffix such as `sleep_ms` or `wait_timeout_while`; `raf-source` also
  refuses `Duration` and `elapsed`). The documentation watchdog reads
  the clock in `crates/cargo-raf-workplan/src/trusted_docs/clock.rs`. The gate
  source is exempt so it can name the forbidden calls. Drive time-dependent
  test cases with a scripted clock.

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

### Registry packages

Use a clean release checkout with a current lockfile. Package the five RAF
libraries and both command packages. Check that the archives include their
runtime assets and that their normalized manifests resolve sibling libraries
from crates.io at the release version.

List the exact dependencies that the public RAF packages impose on a consumer in
each release's notes. Read the released manifests for the version requirements.
Explain parser and wire-reader pins alongside the first release that changes
them.

RAF 0.8.1 carries the following exact requirements into the consumer's lock
graph when the respective RAF packages are selected.

| Dependency | Requirement | Where it enters the graph |
| --- | --- | --- |
| `syn` | `=3.0.8` | Source parsing in Census and Source |
| `proc-macro2` | `=1.0.107` | Parser spans and tokens in Census, Source and Workplan |
| `quote` | `=1.0.47` | Native parser token construction in Source |
| `unicode-ident` | `=1.0.27` | Identifier admission in Source |
| `capnp` | `=0.28.1` | Cap'n Proto history and reader evidence in Census |
| `gix` | `=0.90.0` | Native Git access in Source |
| `imara-diff` | `=0.2.0` | Document history comparison in Workplan |

The source parser pins preserve the implementations used to qualify the
resource bounds. The `capnp` pin keeps descriptor and canonical payload reads
on the qualified Rust wire reader and its generated schema. Changing it
requires rechecking the framing, traversal limits and descriptor fixtures.
The PurRDF requirements use the `=5` release series. A consumer's manifest
requirements must resolve jointly with these requirements.

```sh
cargo publish --dry-run --locked --workspace \
  --exclude rust-agentic-foundations --exclude raf-docs-contract --exclude raf-fixtures
```

On the supported native Linux target, exercise both packaged executables directly
and through `cargo raf-census` and `cargo raf-workplan`. Compile a fresh consumer
against the packaged libraries on the declared stable Rust floor. Qualify the
static documentation from the package contents.

Configure a GitHub trusted publisher for each public crate with owner
`Blackcat-Informatics`, repository `rust-agentic-foundations`, workflow
`publish.yml` and environment `release`. Restrict the GitHub `release`
environment to the `main` branch. A new crate needs an initial upload with a
crates.io API token before its trusted publisher can be configured.

After maintainer approval, dispatch **Publish to crates.io** from `main` with
the full release commit SHA, the workspace version and `workspace` as the
package. The workflow checks that the selected commit passed push CI and Cargo
publishes the public packages in dependency order. If publication stops
partway, check each exact version in the registry. For versions already
published, confirm the archive's `.cargo_vcs_info.json` identifies the release
commit. Dispatch each missing package from that commit. Publish `raf-okf`,
`raf-rdf` and `raf-source` before `raf-census` and `raf-workplan`, then publish
`cargo-raf-census` and `cargo-raf-workplan`. Keep `main` at the release commit
until the registry has all seven packages. Verify fresh registry installs and
the static docs.rs builds before publishing the signed version tag and release
notes.

## Before you open a pull request

- Install the hooks with `make hooks`. Run `make check` and `make test`.
- `make clippy-all` must be warning-free. Every crate inherits the workspace
  lint table. Fix lint findings in the code. When a lint exception is necessary, use
  a tightly scoped `#[expect(..., reason = "...")]`.
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
