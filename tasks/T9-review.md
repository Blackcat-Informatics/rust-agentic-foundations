# T9 review: CLI lifecycle commands (staged on top of c0685a4)

## Scope

`git diff --cached` (31 files): `crates/cargo-workplan` (change.rs, cli.rs, command.rs, main.rs,
render.rs, six new/changed integration test files), `crates/raf-workplan` (SPDX header on
`change new`, A2/A3 fixes, additive accessors), README, AGENTS, SECURITY, CHANGELOG. Plan: D8, D9,
D12, Task 9, rows R1/R13/R16, Carried items. No capability document is touched (confirmed from the
diff stat).

## Evidence

1. Binary holds no logic. change.rs is one library call per command plus record building. The
   ordering (open changes, obligations, refusal text, `headline`, `is_repeated_in`) lives in the
   library. No duplicated rule found.
2. Exit codes, run through the built binary in a scratch copy of the repository and a scratch
   workspace:
   - new for an existing slug, unknown capability, `--affects` with an unknown section or a
     capability not listed, a missing or empty or multi-line SPDX value: exit 2, stdout empty.
   - One SPDX flag without the other: clap parse error, exit 2.
   - Unknown slug for impact, reconcile, complete and verify: exit 2, stdout empty.
   - Findings: reconcile with an obligation exits 1; complete refused exits 1; failing `verify`
     binding exits 1; success exits 0.
   - Stub `git` reporting 2.29: reconcile and complete exit 2 with `git --version` the only
     process when `--root` is given.
   - `verify` and `impact` never ask for the Git version, only `rev-parse` and `ls-files`.
   - Without `--root`, `cargo metadata` runs before `git --version` (ADVISORY 1).
3. `change new` writes exactly one file (status shows one untracked file). The header with
   `--copyright`/`--licence` is byte-identical (first four lines) to `work/changes/census-cli.md`.
   `uvx reuse lint` on a tree holding the generated record passes for it and fails only for the
   record made without SPDX flags. Unicode, quotes and backslashes are accepted and written
   verbatim; tab, newline and empty are refused.
4. JSON: `new-change`, `impact`, `obligation`, `refused`, `refusal`, `completed`, `binding` are
   documented in main.rs and exercised by tests. Quotes and backslashes are escaped (round-tripped
   through a JSON parser). Exit-2 runs print nothing on stdout.
5. `-q`: hides only outcomes and notes. Obligations, `refused`, `refusal` and failing `binding`
   records stay, and the exit code is unchanged. A mutant that showed everything and one that made
   `refused` an outcome were both killed.
6. Both argv shapes are in the cli parse tests and the process tests. The real
   `cargo workplan` dispatch test exists (`process.rs`, `cargo_dispatches_workplan_to_the_binary_...`).
7. Slug-first validation for every slug command (`process.rs`, `slug_commands()`, recording stubs).
   Parse tests cover `-rf` and `--since=x` with and without `--`.
8. Read-only: impact and reconcile leave my scratch tree byte-identical (md5 of every file
   before and after, including Cargo.lock).
9. End to end in a scratch workspace with a real crate: new, add proposal, impact, reconcile (2
   obligations, exit 1), complete refused (exit 1, 4 failures, text and JSON agree, `-q` keeps the
   refusal), fix, set delivered, `docs sync`, `verify`, reconcile (empty, exit 0), complete
   (exit 0, record removed, one file rewritten), check clean (exit 0).
10. Library: SPDX validation uses `trim`, empty and `char::is_control`. A2/A3 behave as the plan
    asks (`sync_needed` true also when the target was already stale). Accessors are additive.
11. Mutations (10 killed of 10 that compiled): verify always passes; `-q` shows all; obligations
    reversed; `requires` removed from `--copyright`; `refused` made an outcome; `sync_needed`
    forced false; control-character check removed; SPDX lines swapped; empty check removed; open
    changes unsorted; `is_repeated_in` inverted. Four further mutants did not compile (unused
    variable, dead variant) and were redone in a compiling form, which were then killed.
12. Docs say only what exists. README, AGENTS, SECURITY and CHANGELOG all describe the capability
    documents as still designed. `make issue-refs` passes. SPDX is present on all new files.
13. Gates, all green: `cargo test --workspace` (1571 passed, 0 failed); release test of
    raf-workplan and cargo-workplan; clippy `--workspace --all-targets -D warnings`; `cargo doc`
    with `-Dwarnings`; `cargo fmt --check`; `docs_contract` and `repo_gates`; `make reuse`,
    `deny`, `msrv`, `issue-refs`. The cargo-workplan integration tests were run 5 times, 139 passed
    each time, no flake.

## Findings

### REQUIRED

R1. The printed guidance of `change new` does not make `check` pass when a sync is needed,
because the new record is untracked.
- Reproduction: in a repository whose delivered capability has a `relationships` block,
  `change new qq --capability cap:workplan-document-model` prints "run `docs sync`". After
  `docs sync`, `check` reports `link-broken` for `../../work/changes/qq.md` (the new "Changed
  by" row points at a path Git does not list) until `git add work/changes/qq.md`.
- The implementer's own end-to-end test has to run `git add -A` before `docs sync`, with a
  comment saying so, but nothing the command prints says it. The CHANGELOG claims "after
  following the printed guidance: a clean check".
- Fix: when `sync_needed` is true, the printed note (library `Display` of `NewChange`) must say
  to track the record first, for example "`git add work/changes/<slug>.md`, then run `docs sync`;
  until the record is tracked, `check` reports the link to it as `link-broken`". Pin it in
  `new.rs` and the library test, and keep the JSON `sync_needed` field as it is. If the note only
  matters when a relationships block exists, `sync_needed` already encodes exactly that case.

### ADVISORY

A1. Without `--root`, root discovery (`cargo metadata`) runs before `git --version` for reconcile
and complete. The reconcile Contract sentence ("exits 2 before any other process starts") is
about reconcile's own invocation, which does start with `git --version`, and main.rs is worded
"before any other process of theirs". Not required to reorder. Consider a Task 11 Contract-edit
note or a test comment that says root discovery is outside that sentence.

A2. `char::is_control` does not refuse U+2028 and U+2029 (line and paragraph separators). Tools
that split lines on them would see two lines. Cheap to add `|| matches!(c, '\u{2028}' | '\u{2029}')`
with a test; the harm is unlikely (YAML comment, REUSE reads by `\n`).

A3. `change new` under `-q` prints nothing, including the path of the created file and the
sync note. This is what the Contract's `-q` rule says and is tested, but a script using `-q` gets
no notice that a sync is needed; the JSON `sync_needed` field is the way to get it. No change
needed.

VERDICT: BLOCKED

## Re-review

Re-checked at HEAD 9f1ab69 in a scratch worktree (removed afterwards).

| Finding | Fix location | Evidence | Status |
| --- | --- | --- | --- |
| R1 printed note omits tracking the record | `crates/raf-workplan/src/lifecycle/create.rs:238-243` (note says to stage the record first, prints `git add <path>` then `docs sync`); pinned in `crates/raf-workplan/tests/lifecycle.rs:1265-1270` and `crates/cargo-workplan/tests/complete.rs:454`; JSON `sync_needed` unchanged | `cargo test -p cargo-workplan --test new` 15 pass; mutant deleting the `git add` line fails 2 library lifecycle tests and the cargo-workplan end-to-end test `from_new_to_a_completed_change_through_the_binary` | resolved |

VERDICT: PASS
