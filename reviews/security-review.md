# Stage 2 security review: scaffold workspace, project docs and capability specs

- Branch `paudley/1-scaffold-workspace-project-docs-and`, head `d6b6572`, base `origin/main` @ `720262d`
- Diff: `git diff origin/main...HEAD` (86 files)
- Mode: read-only. No repo file edited and nothing committed or posted. The `make check` suite was not run.
- Verdict: **no CRITICAL or HIGH findings.** The CI workflow and the hook are sound. There are 2 MEDIUM findings, both design gaps in the capability specs relative to what SECURITY.md promises. There are 5 LOW findings.

## Reviewed scope

| Area | Files |
| --- | --- |
| CI | `.github/workflows/ci.yaml` |
| Hook / gates | `.githooks/pre-commit`, `Makefile` |
| Supply chain / toolchain | `deny.toml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` |
| Reporting | `SECURITY.md`, `.github/ISSUE_TEMPLATE/{bug_report,feature_request,config}.yml`, `.github/PULL_REQUEST_TEMPLATE.md` |
| Designed security contracts | `docs/capabilities/source-index.md`, `census-reference-scan.md`, `census-cli.md`, `census-ledger.md`, `okf-profile.md`, `workplan-cli.md`, `workplan-entity-history.md`, `workplan-complete-gate.md`, `docs/architecture/okf-profile.md` |
| Context | stage `issue.md`, `plan.md` (A14, A15, adv. 9), `validation.md`, `AGENTS.md`, `CLAUDE.md`; `tests/docs_contract.rs` walker (for contrast) |

## Evidence

### E1. Linters

```
$ actionlint .github/workflows/ci.yaml; echo rc=$?        # actionlint 1.7.12
rc=0
$ shellcheck -x -s bash .githooks/pre-commit; echo rc=$?
rc=0
$ uvx zizmor --offline .github/workflows/ci.yaml          # zizmor 1.30.1, default persona
No findings to report. Good job! (15 suppressed)
$ uvx zizmor --offline --persona=pedantic ...             # pedantic persona
  1 help[concurrency-limits]   8 info[anonymous-definition]   6 info[superfluous-actions]
```

### E2. Action SHA pins vs upstream refs (read-only `git ls-remote` / `gh api`)

```
$ git ls-remote https://github.com/actions/checkout 'refs/tags/v7*'
9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0  refs/tags/v7.0.0     <- matches ci.yaml "# v7.0.0"
3d3c42e5aac5ba805825da76410c181273ba90b1  refs/tags/v7.0.1
$ git ls-remote https://github.com/astral-sh/setup-uv 'refs/tags/v10*'
bec219d24cd3e171d82865faccec33120bb574f4  refs/tags/v10.1.0    <- matches ci.yaml "# v10.1.0"
1c37ad07a6a961277cf70c0d37d6f313000f5884  refs/tags/v10.3.0
$ git ls-remote https://github.com/dtolnay/rust-toolchain refs/heads/stable refs/heads/master
e2a55d2ffb04f378e9626c28d38b36d230d1e12f  refs/heads/master
686976e191b89faba57d3206551f0f330d8cb249  refs/heads/stable
$ gh api repos/dtolnay/rust-toolchain/commits/29eef336… --jq '...'
{"author":"dtolnay","date":"2026-03-27T15:56:05Z","msg":"toolchain: stable"}  parents=[3c5f7ea…] verified=true
$ gh api repos/dtolnay/rust-toolchain/compare/29eef336…...stable  -> {"status":"diverged","ahead_by":17,"behind_by":1}
$ gh api repos/dtolnay/rust-toolchain/compare/29eef336…...master  -> {"status":"diverged","ahead_by":16,"behind_by":1}
$ gh api repos/dtolnay/rust-toolchain/compare/3c5f7ea…...master   -> {"status":"ahead","ahead_by":16,"behind_by":0}
  commits on stable since the pin include:
  83d06109b7 2026-10-09 Safely handle action inputs
  e2a55d2ffb 2026-10-09 Merge pull request #187 from dtolnay/pwn
```

The two tag pins match exactly. Both are lightweight tags, because no `^{}` peeled refs appear. The dtolnay pin is a real, GitHub-verified dtolnay commit and not an imposter fork commit: its parent is on `master`. No current branch can reach it, though, because dtolnay rebuilds `stable` on every release (see L2). purrdf pins the same SHA.

### E3. Workflow posture (manual)

- `permissions: contents: read` is set at workflow level (ci.yaml:11-12), and no job widens it.
- The triggers are `pull_request` and `push` to `main` only. There is no `pull_request_target`, `workflow_run` or `workflow_dispatch`.
- Every checkout sets `persist-credentials: false`.
- No `${{ … }}` expression appears in any `run:` step. No `secrets.*` are referenced, and there is no `actions/cache`.
- `setup-uv@bec219d…` defaults `enable-cache: "auto"`, so it caches on `push` and `pull_request`. No privileged workflow consumes that cache, so there is no poisoning path today.
- `setup-uv` `version` defaults to `""`, which means latest, because the repo has no `pyproject.toml`/`uv.toml`.

### E4. Hook with hostile filenames (throwaway repo under `mktemp -d`)

```
files committed: ./-rf  'a b.rs'  $'new\nline.rs'  './--prefix=evil'
$ git checkout-index --all --force --prefix="$snap/"; echo rc=$?
rc=0
$ ls -b1 "$snap"
--prefix\=evil
-rf
a\ b.rs
new\nline.rs
```

No filename is ever passed as an argument by the hook or the Makefile. Every gate takes a directory (`ROOT`) or uses `git grep --cached`. The hook has `set -euo pipefail` (pre-commit:11). It uses `mktemp -d`, which is mode 0700, with `trap 'rm -rf "$snapshot"' EXIT` (:16-17). The snapshot path is quoted everywhere in the hook (:18, :30-32). The `run` wrapper collects failures, so an earlier gate failing cannot mask a later one (:21-28, :35-38). The tracked mode is `100755`.

### E5. Makefile unquoted `ROOT` (dry run only)

```
$ make -n fmt reuse licenses ROOT='/tmp/x y/snap'
cargo fmt --all --check --manifest-path /tmp/x y/snap/Cargo.toml
uvx reuse@6.2.0 --root /tmp/x y/snap lint
set -e; root='/tmp/x y/snap'; ...                    <- licenses recipe is quoted correctly
$ cargo fmt --all --check --manifest-path /tmp/x y/snap/Cargo.toml; echo rc=$?
error: unexpected argument 'y/snap/Cargo.toml' found
rc=2                                                  <- fails closed
```

### E6. `cargo metadata` executes repository-configured programs

The probe ran in a throwaway package. The script lived in a separate directory, and the package's `.cargo/config.toml` set `build.rustc = "<script>"`:

```
$ cargo metadata --format-version 1 --offline           # full resolution, as the reference scan needs
error: process didn't exit successfully: `kache /tmp/tmp.gJVkZlsoMV/scripts/fake-rustc.sh -vV` (exit status: 2)
$ cargo metadata --format-version 1 --offline --no-deps
rc=0, script not invoked
```

Cargo resolved the `rustc` path from the scanned tree's `.cargo/config.toml` and spawned it. On this machine the global `RUSTC_WRAPPER=kache`, set in `~/.cargo/config.toml`, sat in front of it. Without that wrapper, the script would have been executed directly. `--no-deps` did not invoke it.

### E7. PGP key in SECURITY.md (throwaway `GNUPGHOME`)

```
sha256(block) = 6bb700ee9cc3d15b35231717654898b3f0c77897e4164124fd21c733ab5cfa62   (matches plan R16)
pub   rsa4096 2016-12-26 [SC] [expires: 2026-12-24]
      52E54B131C297D1EAB143BAA0C54057C8F9A0090
uid   Blackcat Informatics Inc. (Secure Inbound Key) <secure@blackcatinformatics.ca>
sub   rsa4096 2016-12-26 [E] [expires: 2026-12-24]
self-sig digest algo 8 (SHA-256); key expires after 10y0d
$ date -u   ->  Mon Oct 12 2026
```

The contact address in SECURITY.md is `security@blackcatinformatics.ca`, which matches purrdf, gmeow-gts and gmeow-ontology.

### E8. Other

- `cargo deny --version` is `cargo-deny 0.20.2` locally, and `--locked`, `--offline` and `--frozen` are all valid flags. CI installs an unpinned version (ci.yaml:95).
- The local advisory DB (`~/.cargo/advisory-dbs/advisory-db-3157…`) was last updated 2026-09-17.
- `Cargo.lock` holds only the root package, so the dependency graph is empty today.
- `git grep` for `curl|wget|| sh|sudo|--no-verify|pull_request_target|secrets.|GITHUB_TOKEN` matches nothing beyond the `cargo install` line and AGENTS.md's prohibition of `--no-verify`.
- `tests/docs_contract.rs:1172-1207` already skips symlinks while walking, and it resolves links against an in-memory set of repo paths. That is the behaviour the specs should require (see M1).

## Findings

### M1. MEDIUM: path containment in the designed contracts is lexical-only and incomplete

**Where**
- `docs/capabilities/source-index.md:39-45` and `:96`: `#[path]` containment, defined by `..` segments or an absolute path.
- `source-index.md:32-37`: "honouring explicit target paths", with no containment.
- `source-index.md:26-30`: `Tree`/`DiskTree` say nothing about symlinks or file types.
- `source-index.md:126-127` and `:133-134`: no symlink fixture, and the `DiskTree`≡`MemoryTree` equivalence test cannot model symlinks.
- `docs/architecture/okf-profile.md:120-122`: "Every relative link resolves to an existing file, and a link anchor resolves to a heading in the target", with no containment.
- These contradict `SECURITY.md:47-49`, which lists `#[path]` traversal, "symlinks that point outside the scanned tree" and out-of-repo relative links as attack surface.

**Problem.** The only containment rule is for `#[path]`, and it is stated in lexical terms. Nothing requires the final path to be checked against the root after the operating system resolves it. Four routes are left open:

1. A plain `mod inner;` that resolves to `src/inner.rs`, where `src/inner.rs` is a symlink to `~/.config/gh/hosts.yml` or `~/.ssh/id_ed25519`. No `#[path]` is involved, so no rule applies.
2. `#[path = "vendored/../../x.rs"]`, where `vendored` is a symlink to a directory outside the tree. Lexical normalisation keeps the path inside the root, but the operating system follows `vendored` and then `..` from its target.
3. A `Cargo.toml` target such as `[lib] path = "/home/dev/.aws/credentials"` or `[[bin]] path = "../../secret.rs"`. Package discovery reads it, and no rule constrains it.
4. A capability document link `[x](../../../../home/dev/.netrc#a)`. The check probes for the file's existence and reads the target to find the heading anchor.

**Exploit or failure scenario.** A developer, or a CI job that is not otherwise building the code (for example a docs-only or census-only job), runs `cargo census index` or `cargo workplan check` on a hostile checkout. That is exactly the use SECURITY.md:35-38 claims to support. The tool then reads files outside the repository:

- If the output echoes literals or index content (`index` JSON, `home-literal` and `vocabulary-literal` findings, parse diagnostics), the content can be disclosed to public CI logs.
- At minimum it gives an existence oracle ("link target not found" versus "anchor not found").
- A symlink to `/dev/zero` or a FIFO hangs the tool or exhausts its memory.
- A directory symlink loop recurses without bound.

**Required fix.** Amend the specs before implementation.

- In `source-index.md` Contract and Constraints, state that every path the crate opens is subject to containment. This covers mod-tree files with and without `#[path]`, explicit Cargo target paths and discovered `Cargo.toml` files. Containment is checked on the resolved path: canonicalize the root once, canonicalize each candidate, and require it to have the root as a prefix. A candidate outside the root is an error and is not read.
- State that `DiskTree` does not follow symlinks that leave the root, or refuses symlinks entirely, as `tests/docs_contract.rs` already does. It must also refuse non-regular files (FIFOs, devices) and must not recurse into a directory symlink, which handles loops.
- Add `DiskTree` fixtures for each route:
  - a file symlink that points out of the root;
  - a directory symlink that points out of the root;
  - the `link/../..` trick;
  - an explicit Cargo target `path` that escapes the root;
  - a symlink loop;
  - a FIFO.

  Then drop the claim that `MemoryTree` equivalence covers the disk path for these cases.
- In `docs/architecture/okf-profile.md:120`, add that a relative link must resolve inside the repository root. An escaping link is a violation, and its target is neither opened nor stat'ed.

### M2. MEDIUM: the specs spawn `cargo` in the scanned workspace, which executes repo-controlled programs, yet SECURITY.md claims untrusted-repository use

**Where**
- `docs/capabilities/census-reference-scan.md:29-31` and `:44-45`: `cargo metadata` subprocess with "`--locked` semantics … does not fetch".
- `docs/capabilities/workplan-cli.md:26-28`: "finds the root from `cargo metadata`".
- `docs/capabilities/workplan-entity-history.md:41-45`: `git log --follow` and `git log -L` over document-derived paths.
- `docs/capabilities/workplan-complete-gate.md:35` and `:57`: `cargo test` filtered by `bindings.tests`.
- Against these, `SECURITY.md:35-38` says "designed to run on developer machines and in CI against code and documents the reader may not control", and `SECURITY.md:50-55` names hostile manifests and argument injection as surface.

**Problem.** Evidence E6 shows that `cargo metadata` with full dependency resolution, which the reference scan needs, honours the scanned workspace's `.cargo/config.toml` and executes the configured `build.rustc` (and `rustc-wrapper`). The rustup proxy also honours the workspace's `rust-toolchain.toml`, including a `path =` custom toolchain. So the specs as written make `cargo census check` (with `reference = [...]`) and every `cargo workplan` command an arbitrary-code-execution primitive for whoever controls the scanned tree.

That is acceptable for the main use, the `Gate` inside `cargo test`, because the project's own code is already running there. It is not acceptable under the trust model SECURITY.md advertises.

Separately, "`--locked` semantics … does not fetch" is not true of `--locked`. `cargo metadata --locked` refuses to change `Cargo.lock`, but it still downloads missing packages. Only `--offline` or `--frozen` stops fetching, and `census-reference-scan.md:86-88` already expects a scan to fail when a source is not local. Finally, no spec requires argv-vector spawning (no shell) or `--`/`--end-of-options` separation for document-derived values passed to `git` or `cargo`.

**Exploit scenario.** A reviewer runs `cargo workplan status` on a hostile PR branch to see the roadmap, believing the command is read-only (`workplan-cli.md:84-86`). The branch contains `.cargo/config.toml` with `[build] rustc = "./tools/x.sh"`. `cargo metadata` spawns `./tools/x.sh -vV` with the reviewer's credentials and environment.

**Required fix.** Choose one of the following and make SECURITY.md and the specs agree.

- **(a) Narrow the trust model.** SECURITY.md states that the RAF CLIs invoke `cargo` (and `git`) inside the scanned workspace. Cargo honours that workspace's `.cargo/config.toml` and `rust-toolchain.toml`, so running RAF on a checkout carries the same trust as building it, and untrusted checkouts are out of scope. The "reading untrusted repositories" paragraph is reworded so that it covers only the in-process parsers.
- **(b) Minimise and specify.**
  - `workplan-cli.md` discovers the root without dependency resolution, using `cargo locate-project --workspace --message-format plain` or `cargo metadata --no-deps`. E6 shows that `--no-deps` did not invoke `build.rustc`. The spec should still note that the rustup proxy honours the workspace toolchain file.
  - `census-reference-scan.md` names the exact invocation (`cargo metadata --format-version 1 --frozen`), replacing "`--locked` semantics". It also states that this step executes workspace-configured tools.
  - The entity-history and complete-gate specs require argv vectors (never a shell), `--end-of-options` or `--` before any path or filter derived from a document, and validation of binding names against the source index before they reach `cargo test`. Each of these gets a fixture whose slug, binding or path begins with `-`.

### L1. LOW: CI tool versions float (cargo-deny, uv, reuse's transitive dependencies)

**Where:** `.github/workflows/ci.yaml:94-95` (`cargo install --locked cargo-deny`, no version), `ci.yaml:130-131` (`setup-uv` with no `version:`, so the latest uv), `ci.yaml:133` and `Makefile:8` (`uvx reuse@6.2.0` pins only the top-level package).

**Scenario.**
- A new cargo-deny release is compromised or behaves differently. It is built and run in the `deny` job on the next push. The impact is bounded: the token is `contents: read` and there are no secrets. But the job's verdict, which is the only fetching advisory check under plan A14, then rests on unreviewed code.
- Plan A14 splits the gate: offline locally with cargo-deny 0.20.2, fetching in CI with whatever version is latest. The two can disagree on `deny.toml` semantics. Major releases have renamed or removed keys before, so CI could fail or pass differently from `make check`.
- The same drift applies to uv and to reuse's Python dependency tree.

**Required fix.**
- Use `cargo install --locked cargo-deny@0.20.2`, matching the local version. A SHA-pinned installer action that verifies checksums also works.
- Add `version: "<x.y.z>"` and `enable-cache: false` to the `setup-uv` step.
- Optionally run reuse with a constraints or lock file. Record the pinned versions next to `REUSE ?=` so local and CI stay in lockstep.

### L2. LOW: the `dtolnay/rust-toolchain` pin is unreachable from upstream branches and predates upstream's input-injection hardening

**Where:** `.github/workflows/ci.yaml:27, 43, 59, 74, 91, 112`.

**Detail (E2).** `29eef33` is the legitimate `toolchain: stable` commit of 2026-03-27. GitHub reports it as verified, and its parent `3c5f7ea` is on `master`. It is now orphaned, because `stable` is rebuilt on each release and diverges from it. It also lacks `83d0610` "Safely handle action inputs" (2026-10-09). Before that commit, `inputs.toolchain`, `inputs.components` and `inputs.targets` were spliced into `run:` scripts through `${{ }}`.

**Scenario.**
- This workflow passes only literals (`nightly`, `1.98.1`, `rustfmt`, `clippy`), so it is **not exploitable today**. A later edit that passes a matrix or other non-literal value would inherit the injection pattern.
- Separately, an unreachable commit can eventually be garbage-collected upstream. The jobs would then fail closed, all at once.

**Required fix.** Repin all six uses to the current `stable` head `686976e191b89faba57d3206551f0f330d8cb249`, which contains `83d0610`, and update the comment's date. Add `.github/dependabot.yml` with `package-ecosystem: github-actions` so that SHA pins are bumped through reviewed PRs.

### L3. LOW: the SECURITY.md PGP key expires in 73 days, and its UID does not match the stated contact

**Where:** `SECURITY.md:96-157` (contact `:98`, key block `:100-156`).

**Detail (E7).** The primary key and the encryption subkey both expire on **2026-12-24**. The key's only UID is `secure@blackcatinformatics.ca`, but the policy says `security@blackcatinformatics.ca`. No fingerprint is printed for out-of-band verification.

**Scenario.** After 2026-12-24, `gpg --encrypt -r security@…` fails ("unusable public key"). A reporter then either sends a vulnerability in plaintext or gives up. Before that date, the UID mismatch makes `-r security@blackcatinformatics.ca` fail key lookup, which pushes reporters toward plaintext or toward trusting an unverifiable key.

**Required fix.**
- Extend the expiry (`gpg --quick-set-expire 52E54B131C297D1EAB143BAA0C54057C8F9A0090 <date> '*'`), or rotate the key, and republish the block.
- Add a `Fingerprint: 52E5 4B13 1C29 7D1E AB14 3BAA 0C54 057C 8F9A 0090` line.
- Either add a `security@` UID or state that the key belongs to the `secure@` inbox that serves `security@`.
- Update the R16 block-hash check to match. This affects every sibling repo that copies the block, so fix it at the source.

### L4. LOW: the parser specs carry no resource bounds, although SECURITY.md lists resource exhaustion

**Where:** `docs/capabilities/okf-profile.md:33-36` (YAML frontmatter parse), `source-index.md:39-41` (`syn` parse of every reached file), `census-ledger.md:28-29` (TOML), against `SECURITY.md:40-46`.

**Scenario.**
- A hostile `.rs` file with deeply nested expressions overflows the stack in `syn`'s recursive descent. A stack overflow aborts the process. It is not a catchable panic and not a typed error, so `cargo test` (the `Gate`) dies without a report.
- YAML frontmatter that uses anchors and aliases ("billion laughs") expands without bound if the chosen YAML crate expands aliases.
- A multi-gigabyte "source" file is read whole.

**Required fix.** Add explicit constraints and fixtures:
- a maximum file size per read;
- YAML aliases and anchors rejected, or a parser that does not expand them, since the closed-key schema has no need for them;
- a bounded nesting depth, or parsing on a thread with a stated stack size, so that deep nesting becomes a typed error naming the file.

### L5. LOW: unquoted `$(ROOT)` in Makefile recipes rejects valid commits when `TMPDIR` contains whitespace (fails closed)

**Where:** `Makefile:23` (`--manifest-path $(ROOT)/Cargo.toml`) and `Makefile:42` (`--root $(ROOT)`). The hook's snapshot comes from `mktemp -d` (`.githooks/pre-commit:16`), which honours `$TMPDIR`.

**Scenario (E5).** With `TMPDIR="/tmp/x y"`, the `fmt` and `reuse` gates exit 2 with argument errors. Every commit is refused, and that pushes the user toward bypassing the hook, which AGENTS.md forbids. There is no security bypass, because the gates fail closed. The `licenses` recipe already quotes correctly with `root='$(ROOT)'`.

**Required fix.** Quote the paths: `--manifest-path "$(ROOT)/Cargo.toml"` and `$(REUSE) --root "$(ROOT)" lint`.

## Checked and found sound (no finding)

- **Token and permissions.** Workflow-level `contents: read`, `pull_request` rather than `pull_request_target`, `persist-credentials: false` on every checkout, no secrets, no `${{ github.* }}` in `run:`.
- **SHA pins.** `actions/checkout` v7.0.0 and `astral-sh/setup-uv` v10.1.0 match their tags exactly. v7.0.1 and v10.2/10.3 exist, but their changelogs (E2) contain nothing security-relevant to this usage.
- **Cache poisoning.** No `actions/cache`. The setup-uv cache is scoped per ref, and no privileged workflow (release, `pull_request_target`, `workflow_run`) reads it. If a release workflow is added later, set `enable-cache: false` there.
- **Hook.** `set -euo pipefail`, a 0700 temp dir with an EXIT-trap cleanup, quoted paths, and no filenames passed as arguments, so hostile names are inert (E4). `git grep --cached` judges the index.
  - Note: the hook and the Makefile rules come from the working tree, not the snapshot. That is local-only.
  - Note: `make hooks` (`core.hooksPath .githooks`) means that committing on a checked-out untrusted branch runs that branch's hook. That carries the same trust as running its `make check`. CONTRIBUTING could say so.
- **deny.toml.** Only the crates.io registry is allowed; unknown registries and git sources are denied; `yanked = "deny"`; `wildcards = "deny"`; `multiple-versions = "deny"`; the licence list is short and deliberate. The offline-local/fetching-CI split (A14) is reasonable. Because hosted CI has never run, no fetched-advisory run has happened yet. The dependency graph is empty, so nothing is exposed now. Setting `[advisories] maximum-db-staleness` would bound the offline gate explicitly.
- **MSRV job.** `RUSTUP_TOOLCHAIN` is set at job level, and the version is asserted (ci.yaml:103-118).
- **Issue templates.** `config.yml` links SECURITY.md as a contact link, and the bug template tells reporters not to file vulnerabilities publicly.
- **SECURITY.md surface list.** Accurate and complete as a list of surfaces. The defects are in what the specs promise against them (M1, M2, L4) and in the key (L3).
