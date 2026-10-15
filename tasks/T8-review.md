# Task 8 review: cargo-workplan crate (root, check, status, docs)

## Scope

Staged, uncommitted diff on top of a83a1a0 (22 files, +3313/-10): the new
`crates/cargo-workplan` crate (cli, command, exit, render, main; 8 integration
test files plus support), the workspace member and Cargo.lock entries, the macOS
and Windows CI steps, AGENTS/README/SECURITY/CHANGELOG edits and the
`docs_contract` package-set pin. Read against plan.md (D1, D10, D12, Task 8,
R12/R13/R16), the T1-T7 reviews, AGENTS.md, SECURITY.md and the whole Contract of
`docs/capabilities/workplan-cli.md`. No capability document is edited (the diff
touches nothing under `docs/`). The repo has no INVENTORY file, so there was
nothing to edit there.

## Evidence

1. Thin binary. Every command is one library call: `Gate::try_check`,
   `Gate::open` + `Workplan::status/groups`, `Workplan::sync/write`,
   `Workplan::preview`, `root::discover`. No process is started by the binary
   (grep: no `process::Command`). No finding classification. Two small
   ordering/id rules live in the binary (A1) and one Location decomposition (A2).
2. Argv shapes. Ran the built binary both directly and as `cargo workplan ...`
   through real cargo with a PATH entry named `cargo-workplan`: check, status,
   json, `-q`, global options before and after the command, `--`, `--help`/`-h`,
   `--version`/`-V` (exit 0, stdout), no arguments (help on stderr, exit 2),
   unknown command/option, repeated `--format`, missing option value, invalid
   UTF-8 in `--root` (lossy in message, exit 2), in the slug, `--copyright`,
   `--format` and as the subcommand (all clap errors, exit 2, empty stdout). Clap
   errors exit 2 and help/version exit 0 as `main` maps them.
3. Exit mapping. Report exits 1 only if a Finding-class record exists; every
   `Err(Failure)` is 2; a failed stdout write is 2. No path found that prints
   findings and exits 0, or exits 1 for a tool failure. Judged decisions:
   unreadable `docs/capabilities` or non-UTF-8 entry, bad manifest, unloadable
   source, bad root = exit 2 (Contract: "unreadable file"); `check` equals
   `Gate::check` (which panics on those same errors); `status` exits 0 with gaps
   (the Contract's own example shows exit 0 with a gap); docs sync/preview
   refusals and unloaded change records exit 2. All consistent with the Contract.
4. JSON. One array per run, `kind` first, fixed member order, `[]` when empty,
   serde escaping (quote, backslash, control characters verified in a live run),
   exit 2 prints nothing on stdout (verified with a non-UTF-8 directory entry and
   a 70 MB cargo stub). Text and JSON modes list the same records in the same order.
5. Read-only. Ran check, status (text and json), `-q check`, `check --format=json`
   against the real repository: file bytes, mtimes, sizes and `git status` are
   identical before and after. Tests snapshot bytes for check/status/preview and
   sync stage-then-compare. `docs preview` writes nothing (code and test).
6. Slugs and processes. `-rf`, `--since=x`, `--`, empty, `A`, unicode, `a/b`,
   `change:x` all exit 2 from clap; the recording-stub test proves no cargo or git
   call. Root discovery argv exactly `metadata --format-version 1 --no-deps
   --frozen` from `$CARGO` and from PATH (verified by hand with recording stubs
   too); output over 64 MiB is refused (exit 2); `--root` starts no cargo.
7. Dependencies. clap 4.6.7 with default features off (std, derive, help, usage,
   error-context, suggestions), serde_json 1.0.154. Cargo.lock adds only
   cargo-workplan, clap tree, serde/serde_json/itoa/memchr/zmij. `make deny`,
   `make reuse`, `make msrv` (1.98.1) pass.
8. CI. `actionlint` clean. macOS and Windows steps run `cargo test -p
   cargo-workplan --locked`. By reading: process.rs is wholly `#![cfg(unix)]`,
   the permission test is `#[cfg(unix)]` with its imports inside the function, the
   canonicalisation in `Scratch::new` is `cfg(unix)` with no unused binding, shared
   support has a reasoned `dead_code` allow. No Windows-only lint seen; Windows and
   macOS were not executed.
9. Docs. Statements in README/AGENTS/SECURITY/CHANGELOG match what is built
   (change and verify stated as designed). docs_contract package-set pin adds
   `cargo-workplan`, correct.
10. Mutation re-run (5 mutants): exit 1 mapped to 0 (killed: unit and 20
    integration tests); quiet filter removed (killed); slug validation removed
    entirely (killed by unit, preview and process tests); JSON `kind` dropped
    (killed); `check` switched to load findings only, losing generated-block
    findings (killed by the stale-block test). A sixth, removing only the
    `starts_with('-')` test, survives: `is_kebab` already rejects it (equivalent,
    A5).
11. Hygiene. SPDX on every new file, `make issue-refs` and grep for tracker or
    process wording clean.
12. Gates, all green: `cargo test --workspace`; `cargo test --release -p
    raf-workplan -p cargo-workplan`; `make fmt`, `clippy` (-D warnings), `doc`,
    `deny`, `reuse`, `msrv`, `issue-refs`; `docs_contract` + `repo_gates`; the
    cargo-workplan suite run 5 more times: 5/5 green, no flake.

## Findings

REQUIRED: none.

ADVISORY:

* A1. `command.rs` sorts the open changes (slug tuple and the capability list)
  and strips the `change:` prefix (`bare`) itself; ordering and id form are
  library knowledge (`Workplan::changes()` is in directory order). Move to a
  library accessor when Task 9 adds the change commands that need the same list.
* A2. `render.rs::of_finding` re-decomposes `Location` into section/line/key,
  mirroring the `Display` match; a new `Location` variant would become three
  `null`s silently. Prefer a library accessor when convenient.
* A3. `status` exits 0 even when the documents have load findings (a stderr note
  says so and points at `check`). Consistent with the Contract; a script that wants
  a verdict must call `check`.
* A4. `-q -q`, `-qq` and a repeated `--format` are clap usage errors (exit 2). Safe,
  slightly unfriendly for wrappers that append flags.
* A5. The `starts_with('-')` branch of `cli::slug` is redundant with `is_kebab`
  (equivalent mutant); harmless defence in depth, with a distinct message.
* A6. Library message for a non-UTF-8 entry in `docs/capabilities` ends "a
  repository with no capability document has nothing to check", misleading but
  pre-existing and not this task's.
* A7. Tests simulate cargo's shape by inserting the word `workplan`; none runs
  through real `cargo workplan`. Verified by hand here, against a PATH-resolved
  binary named `cargo-workplan`; a PATH-stub test would pin it.
* A8. `-q docs preview` prints nothing (preview is not a finding); literal to the
  Contract, noted for Task 11 documentation.

VERDICT: PASS
