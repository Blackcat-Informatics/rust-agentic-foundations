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
document-contract test (`tests/docs_contract.rs`) and the repository gates
(`tests/repo_gates.rs`: licence digests, issue-reference and process-wording
scan, version agreement); the census and workplan gates join them once those
crates are implemented.

### Crates

`raf-okf` is a workspace member and is **implementing**: the frontmatter reader and
the deterministic writer are built. Every other crate below is **designed, not yet
implemented**, and joins the workspace members list when it is built. The capability
documents in `docs/capabilities/` are the specification.

| Crate | Role | Status |
|---|---|---|
| `raf-okf` | Reader and writer for Markdown with YAML frontmatter, and profile validation with closed key sets | implementing |
| `raf-source` | `syn` workspace walker: module tree, `#[path]`, symbol and `use`-alias index, `Tree` trait with disk and memory backends | designed |
| `raf-census` | Normal form v1, fingerprints, grouping, ledger, detectors, `Rule` trait, patterns, debt ratchet, reference scan | designed |
| `cargo-census` | Thin command-line wrapper over `raf-census` | designed |
| `raf-workplan` | Capability and change model, graph, obligations, bindings, generated sections, completion gate | designed |
| `cargo-workplan` | Thin command-line wrapper over `raf-workplan` | designed |

`raf-source` is shared on purpose: the census needs its symbol index to resolve
ledger homes, and the workplan needs the same index to resolve code and test
bindings.

### Layout

| Path | Contents |
|---|---|
| `crates/` | The workspace member crates (`raf-okf` today; each other crate joins `members` when it is built) |
| `docs/capabilities/` | One permanent capability document per capability |
| `docs/architecture/` | Shared current decisions (normal form, OKF profile) |
| `work/changes/` | In-flight change records |
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
  `cargo check --workspace --all-targets --locked` through `rust-validate 1.98.1`.
- The source uses **no unstable features**. A `#![feature(...)]` attribute is
  rejected; the MSRV build proves it on every change.
- Raising the MSRV is a changelog-recorded change that rides a minor bump.

---

## 3. Gates

```bash
make check   # fmt, clippy, test, doc, deny, reuse, msrv, licenses, issue-refs
make hooks   # point core.hooksPath at .githooks for this clone
```

- **Run `make check` before claiming work is done.** It stops at the first failure.
- **Install the hooks with `make hooks`.** The pre-commit hook runs fmt and reuse
  on a checkout of the staged index, and the `repo_gates` and `docs_contract`
  test binaries over that checkout. The binaries are compiled from the working
  tree, not the index; only the files they judge come from the index. A
  `pre-merge-commit` hook runs the same gates on merge commits.
- **`make licenses` and `make issue-refs` run the Rust gates** in
  `tests/repo_gates.rs`, so they are a subset of `make test`. Both gate test
  files share one pattern set and one file list in `tests/support/mod.rs`.
- **Never bypass a hook or a gate.** Do not pass `--no-verify`, do not edit a hook
  to pass, and do not weaken a gate or a lint to get green. Fix the cause.
- A missing tool is a hard failure, not a skip. `make reuse` needs `uvx` or
  `reuse`; `make msrv` needs `rust-validate`.
- `make deny` runs offline locally; CI fetches the advisory database.

### Workspace lints

The root `Cargo.toml` declares `[workspace.lints]`, and every crate inherits them
with `[lints] workspace = true`:

- `unreachable_pub`, `missing_docs` and `unsafe_code` are denied. Use `pub(crate)`
  for items that are not API.
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
- **Fail hard.** Schemas are strict and closed: unknown keys are errors. A ledger or
  document that is stale, malformed or ambiguous is an error with a message that
  says how to fix it. No silent fallbacks, no best-effort parsing, no skipped checks.
- **Normal-form stability.** Any change to what a census fingerprint hashes requires
  a `normal_form` version bump and updated golden tests.
- **Every rule has a negative fixture.** A rule or check with no input that makes it
  fail is not enforced.
- **A refusal is a claim too.** When you add or tighten a refusal, run the case you
  believe is invalid and a neighbouring valid case, and confirm the second succeeds.

---

## 6. The capability-doc workflow

RAF plans its own work in the workplan format. This section describes the design.
Today the document-contract test (`tests/docs_contract.rs`) enforces the format
on every capability, change and architecture document in `cargo test` and in the
pre-commit hook; `raf-workplan` takes over once implemented.

- **Capability documents** live in `docs/capabilities/<slug>.md`, one per
  capability. Frontmatter carries `type`, `id`, `title`, `description`, `state`
  (`considered`, `designed`, `implementing`, `delivered`), `bindings` and `depends`.
  The body has fixed sections: Purpose, Contract, Usage, Examples, Constraints,
  Explanation, Verification. Outside a proposal, the text states the current
  contract only. The document-contract test pins the set of capabilities in one
  inventory table (`INVENTORY` in `tests/docs_contract.rs`): changing a state or
  adding a capability edits that table in the same commit, until
  `raf_workplan::Gate` replaces it.
- **Change records** live in `work/changes/<slug>.md`. Each names the capabilities it
  changes, the sections it affects, and its items. Every item ends with an outcome:
  `implemented`, `dropped`, or `moved` to another tracked change.
- **Proposed text** is written into the capability document itself inside a
  `> [!PROPOSED change:<slug>]` block, so a diff shows the code and the document it
  will produce side by side.
- **Completion** is a gate. A change is complete only when no obligation remains,
  every item has an outcome, no proposed block is left, generated sections are in
  sync, and the bound tests and examples pass. The change record is then
  **deleted**; Git keeps the history.
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
