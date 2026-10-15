# PR #18 — comments and review threads

2 comment(s). **Every one needs a disposition** in
`review-debt.md`: addressed (with the commit hash) or declined (with the
reason posted in reply). Bot comments count.

## 1. paudley — 

# Plan: issue 8 — raf-workplan change lifecycle + cargo-workplan CLI

Revision 3 (integrates the five lens reviews of revision 1). Authoritative plan.
STAGE = `.stage/raf-workplan-change-lifecycle-cargo`. Reviews: `reviews/plan-*.md`; the
enhancement report's numbered items (E1..E15) are referenced below and are binding where this
plan adopts them.

## Issue summary

Add to `raf-workplan` the change lifecycle (`change new|impact|reconcile|complete`, `verify`) and a
new binary crate `cargo-workplan` (clap derive) exposing `check`, `status`, `docs sync|preview`,
`change new|impact|reconcile|complete`, `verify`, exit codes 0/1/2. The issue states "Capabilities
delivered: `cap:workplan-reconcile`, `cap:workplan-complete-gate`, `cap:workplan-cli`". Depends on
the merged document-model work (`Gate`, `Workplan`, `sync`, `preview`, `Roadmap`, `bindings`).

## Baseline, goals, governing decisions

* AGENTS.md / CLAUDE.md: stage-pinned nightly, MSRV 1.98.1, workspace lints (missing docs, unsafe,
  unreachable `pub`), no `#![feature]`, no fallback where the design says fail hard, no tracker
  references or process wording in committed files, no attribution, empty `.deficiencies`,
  `make check` is the gate, every rule has a negative fixture and every refusal a passing neighbour.
* The three capability documents are the contract (argv tables, patterns, proof rules, 1 MiB blob
  bound, Git >= 2.30). Where this plan edits a document the edit is listed in the Contract-edit
  table below; no Contract, Constraints or Verification statement outside the table is removed or weakened. A statement found unbuilt is either built in Task 11 or put to the owner before the PR is created; the implementer cannot waive it by recording a reason.
* SECURITY.md trust model: the checked workspace is trusted build input; no shell; argv vectors;
  document text reaches a process only after resolution and a stated pattern, in one of three
  forms (after `--`; inside one `<rev>:<path>`/`-L` argument; validated test filter). Spawned
  processes get a stated environment (`LC_ALL=C` where output is parsed; nothing else added).
* Reused, not duplicated: `Workplan`, `Gate`, `Workplan::write` stage-then-rename, `Workplan::
  preview`, `Roadmap`/`RoadmapEntry::gaps`, `Graph` queries (`proposals`, impact), `generated::
  analyze`, `Finding` + KINDS coverage pattern, `raf_source::{TestFn, Target}` facts.

## Decisions

* **D1 Layout.** Library logic in `raf-workplan`; binary crate `crates/cargo-workplan` (package and
  bin `cargo-workplan`) holds only clap parsing, rendering and exit-code mapping. The "no logic of
  its own" Constraint is proven by an equivalence test (D9).
* **D2 (replaces revision 1's D2) All three capabilities are delivered by this issue.** Reviewers
  showed that stopping at `implementing` defers work with no tracked home and would put a tracker
  reference in committed documents. So: Scenario A and Scenario B (the complete-gate Verification
  text, kept as written) are built here as integration fixtures (Task 10); the three change records
  are completed with the shipped `cargo-workplan change complete` itself (Task 11), which also makes
  the "capability docs reconciled" check non-circular; the three capabilities end `delivered`.
  Later dogfood/gate-wiring of the repository's other records is outside this issue's scope and is
  not claimed.
* **D3 One spawn path (fixes B1).** `exec` owns every spawn: `Programs { cargo, git, cwd }`, a
  `Runner` trait (real impl + in-process recording fake, E13), bounded stdout/stderr capture,
  closed stdin, wall-clock kill, `LC_ALL=C` for parsed output, `current_dir(root)` for every git
  call (E15). Time bounds (E12, Contract edit): 1800 s for a `cargo test`/`cargo run` that may
  compile, 300 s for an example's own run as the Contract states. Existing `Gate` git calls and
  `examples/docs_sync.rs` move onto it (D4).
* **D4 One load path.** `Gate::programs(Programs)` builder plus `Gate::open() -> Result<Opened,
  GateError>` (DiskTree, Workspace, loaded `Workplan`, tracked paths); `try_check` calls `open`.
  The binary, the example and `Lifecycle` all load through it. The git version gate is NOT part of `Gate::open`/`assert_clean` (their documented behaviour is unchanged); it runs first in `Lifecycle::open`, used only by `reconcile` and `complete` (the two commands whose Contracts state it; `impact`, `verify`, `new` do not need Git), so "2.29 exits 2 before any other process" holds on the reconcile path exactly as the reconcile Contract states (E4). Rerouting `examples/docs_sync.rs` onto `Gate::open` is behaviour-preserving and changes no statement in the generated-docs capability. Root discovery (`cargo
  metadata ...`) is a library function using the same runner (E4).
* **D5 Typed invocation (E2).** `Invocation { program, args: Vec<Arg> }` with `Arg::{Literal,
  Package, Target, Filter, DocFilter, RevPath, ...}`; validated newtypes are constructible only
  through the four pattern validators; the runner accepts only `Invocation`. One unit test walks
  every row of the argv tables. `Evidence` is a typed value (binding, argv, revision, verdict, proof
  counts, wall time) that `verify` yields and obligations cite.
* **D6 Obligations.** `Obligation { kind, capability, section, location, evidence }`;
  `ObligationKind::ALL` (the seven kinds) with a both-directions coverage test; sort by capability,
  section, kind, then line, then evidence (E15). Derived from existing values (E5): `binding-missing`
  from `roadmap::gaps` (made `pub(crate)`, called for `implementing` and `delivered` listed capabilities, because a delivered capability's empty list is an `okf-binding-required` finding, not a gap) and `binding-*` findings, `generated-stale` from `generated::analyze`,
  `unresolved-proposal` from `Graph::proposals()`. `Impact` is one value used by `impact` and
  `reconcile`. `contract_effect: none` suppresses only `section-untouched` for contract/usage/
  examples. Tense phrases: a `const` list with both-directions test, word-boundary matching, code
  spans/fences/PROPOSED blocks excluded, near-miss fixtures ("goodwill", "William").
* **D7 Git and "touched" (E3, E15).** Version >= 2.30; full ref from `symbolic-ref
  refs/remotes/origin/HEAD` -> `rev-parse --verify --end-of-options <ref>^{commit}` -> `merge-base HEAD
  <commit>` -> `cat-file blob <mb>:<path>` (1 MiB bound; a blob that exists but does not parse is an
  error). Touched = authored text differs: the section digest blanks generated blocks (fences and
  PROPOSED blocks stay). Absent default branch is an error naming `git remote set-head origin
  --auto`. On the default branch itself the merge base is `HEAD` and every affected section is
  reported. Linked-worktree fixture.
* **D8 `change new` (U1, E8).** Writes exactly one file (exclusive create, raf-okf writer, re-read to
  prove validity). Repeatable `--capability`; repeatable `--affects cap:<slug>#<section>`
  (validated against the seven sections and `--capability`); default `affects` is `cap:<slug>#
  contract` per capability. It prints the PROPOSED block template (for the author to place) and state guidance per
  listed capability: `delivered` or `designed` -> "set `state: implementing` in the same commit" (a PROPOSED block is a `check` finding in both states); `considered` and `implementing` need none. The R1 template check runs on a capability in each of the four states. The issue's
  "scaffolds ... a PROPOSED block" is read as printing the template, because the CLI Constraint
  fixes a one-file write; the PR body states this reading. Check: the printed template is placed in an `implementing` capability (a state that allows
  blocks); `Graph::proposals()` then lists exactly one block for the change, `check` exits 0, and
  `change reconcile` prints exactly one `unresolved-proposal` line.
* **D9 `complete` is a transition (E1, E9, B2).** Order: static conditions first (record valid,
  items resolved with moved targets existing, no PROPOSED blocks of this change, no `check` finding
  on the record or a listed capability on the post-deletion model, generated blocks current),
  then execution of bound tests/examples, then re-evaluation of the static conditions immediately
  before the writes (E15 TOCTOU). The post-deletion model is loaded through a `Tree` wrapper that
  hides the record. The writes are the record removal and the rewrite of the generated blocks the
  removal stales (Changed-by rows), written in a fixed order: the generated rewrite through `Workplan::write` first, then `remove_file` as the very last step, so an I/O failure never leaves a removed record beside stale blocks (Contract edit). Hiding the record means the listed capabilities must end `delivered` (or have another open record) for the post-deletion `check` to be clean, which is the Contract's rule. The
  state-support check is not re-encoded: it is "no `check` finding" (B2). The author, never `complete`, sets `state: delivered` before running `complete` (a not-delivered capability with no open record is `no-open-record`, a delivered one with an open record is `state-delivered-open-record`; the pre-deletion model is judged by conditions 1-5 only, because a `delivered` capability beside its still-open record is exactly the state `complete` resolves; the state rules are judged on the post-deletion model). Fixtures: left `implementing` refuses; `delivered` completes; `delivered` with an unresolved binding refuses. Refusal lists every
  failure, per obligation kind and per condition.
* **D10 Dependencies.** `clap` (derive) and `serde_json` in `cargo-workplan` only; `raf-workplan`
  gains none, except `rustix` (already in the workspace via raf-source, `process` feature, unix only) for the process-group kill, an owner-visible exception recorded in the CHANGELOG. Licences must sit in the `deny.toml` allow list; any `skip`/exception is mirrored in
  `fuzz/deny.toml` and justified; the full transitive tree is checked by `make deny`.
* **D11 Fixtures.** Real-cargo fixtures are generated into temp directories from in-test templates,
  each with its own `[workspace]`, a template `Cargo.lock`, no dependencies, pinned edition, a shared
  `CARGO_TARGET_DIR` per test binary, explicit `CARGO` override, a wall-clock budget assertion, and
  run wherever the workspace tests run (Task 8 checks the scope of the macOS and Windows CI jobs and widens `.github/workflows` if they run only a subset); only shell stub scripts are `cfg(unix)`, the same matrix runs through
  the in-process `Runner` fake on every platform (E11, E13). Index-fact refusals (harness = false,
  required-features, not a test target, hyphenated target names, `crate_path` stripping) come before
  any spawn (E10).
* **D12 CLI shape.** cargo-subcommand form (`cargo workplan <cmd>` and `cargo-workplan <cmd>` both
  tested), `--root`, `--format text|json`, `-q`; JSON is one array per run on stdout, one object per
  finding/obligation/binding with a `kind` discriminator and the extra fields (Contract edit,
  E14). `docs sync` gets `--copyright`/`--licence`, read only when the index must be created
  (E6, Contract edit); the Makefile's `docs-sync` target runs the binary.

### Declined

* **E7 batched proof runs.** The Contract's per-binding argv rows and per-binding proof are kept
  exactly; batching would change the stated argv table and weaken "exactly one test ran" locality.
  Cost accepted: more `cargo` spawns on a large repository. `complete` only runs the change's own
  capabilities' bindings.
* **Pluggable obligation-source trait (part of E2).** Sources are internal functions in this
  issue; a public extension trait is not added speculatively.

### Contract edits (the only document edits beyond reconciliation of facts)

| Document | Edit | Reason |
| --- | --- | --- |
| complete-gate | writes: record removal plus rewrite of the generated blocks it stales | D9 (E1) |
| complete-gate | order: static conditions before execution; re-check before writes | E9, E15 |
| complete-gate | time bounds for test/build runs | E12 |
| reconcile | "touched" ignores generated blocks | E3 |
| reconcile | `change new` options `--affects` (default `cap:<slug>#contract` per capability); printed template and state guidance | E8 |
| complete-gate | "never edits a capability document" becomes "never edits prose": it rewrites only generated blocks the removal stales | D9 (C9) |
| complete-gate, reconcile, cli | per-process capture bound: 16 MiB per stream (64 MiB for `cargo metadata`, as the cli Contract states) | D3 (C4) |
| complete-gate | the relative-source-path pattern used for a path after `<rev>:` refuses empty, `.` and `..` segments; two further validators (full reference name, commit id) back the git rows | Task 1 (A1) |
| reconcile | command list gains `git rev-parse --show-prefix`; the `<rev>:<path>` argument carries the prefix so a workspace below the repository top reads the right blob | Task 6 (T6 R1) |
| reconcile | `change new` takes `--copyright`/`--licence` and writes the SPDX comment lines when both are given | Task 9 |
| reconcile | the tense scan skips HTML comments, link destinations, generated blocks and the capitalised marker word `PROPOSED`, and reads whole words only | Task 5 |
| complete-gate | rustdoc doc-test list paths read `\` as `/` for the pattern check (names keep the printed form) | Task 3 |
| cli, reconcile | "never write files" is "never write a file in the repository" (cargo builds into the ignored target directory) | Task 6 |
| cli | `docs sync --copyright/--licence`; JSON encoding; cargo-subcommand form; Constraint on complete's writes | E6, E14, E1 |

Scenario A and Scenario B text stays as written; they are built.

## Prior art

None in this repo for this issue (no commits, branches or PRs cite it). Precedent: the merged
raf-workplan work (`Gate`, `preview`, sync stage-then-rename, `Finding` KINDS coverage). Defect-class
recurrence: only `none` recorded; UNKNOWN beyond that, not zero. Prior-art judgments: the
`cargo-workplan` crate is a new artifact; `raf-workplan` exists; the predecessor work is the
merged document model; the recurrence tally names no class.

## NEEDS-JUDGMENT dispositions

1. "incorporated into the current account" = the five conditions plus the post-deletion invariant (D9).
2. `complete` runs bound tests/examples itself; `verify` reports without persisting; one executor.
3. `contract_effect: none` = D6, recorded in the record's own field.
4. Exit codes: 0 success, 1 findings, 2 tool could not run.
5. "record" and "file" are the same artifact `work/changes/<slug>.md`.
6. "touched since the merge base" = D7.

## Completeness contract

| R-id | Task | Executable acceptance check |
| --- | --- | --- |
| R1 | 4, 9 | `change new` fixture: record loads clean; bad slug, existing file, missing capability fail; repeated `--capability`; the printed template placed in a capability (checked in each of the four states, after following the printed state guidance) gives exactly one `Graph::proposals()` block, `check` exit 0 and one `unresolved-proposal` line from `reconcile` (CLI test) |
| R18 | 4 | `new` writes `affects` `cap:<slug>#contract` per capability by default and the `--affects` values when given; fixture asserts the exact value and that `impact` then reports those sections |
| R2 | 4 | `impact` fixture: capabilities, sections, bindings, dependants, linking sections; tree byte-identical |
| R3, R19, R23 | 5, 6 | `reconcile` lines carry kind, location, evidence; deterministic; exit 1 iff non-empty; ordering tiebreak fixture |
| R4 | 5 | `unresolved-proposal` fixture; fixing removes the line |
| R5 | 5 | `item-without-outcome` fixture |
| R6 | 5 | `binding-missing` fixtures (empty required list; unresolved path); equals `RoadmapEntry::gaps` |
| R7 | 6 | real-git: edited-on-branch not reported, unchanged reported, only-generated-block-changed reported, default-branch case, worktree, `origin/main` local branch, `--output=x` branch, no `origin/HEAD`, version 2.29, oversize blob, unparsable blob |
| R8 | 5 | `generated-stale` fixture; sync removes it |
| R9 | 5 | `tense-mismatch` fixture and near misses |
| R10 | 7 | one `complete` refusal per condition AND per obligation kind (loop over `ObligationKind::ALL`), record untouched on every refusal; failing test and failing example are separate cases; a failing example appears under conditions 1 and 5; empty bindings satisfy condition 5 only in `designed`/`considered` (fixture lists the capability from a second open record so the post-deletion state rules stay satisfied); moved target missing refuses; `contract_effect: none` completes without usage/example edits and still refuses on other obligations |
| R11 | 6 | `verify` fixtures: passing + failing test, examples, doc tests, no slug (all capabilities), zero-test cases, hostile bindings; per-binding output; exit 0/1; nothing written |
| R12 | 8 | `docs preview` CLI test: markers absent, proposed text present, tree unchanged |
| R13 | 8, 9 | parse tests per command and option (repeated `--capability`, `--affects`, `--format`, `-q`); both argv shapes; exit 0, 1, 2 each exercised through the binary; Git-2.29 stub gives exit 2 |
| R14 | 5-7 | one fixture per obligation kind (5, 6) and a `complete` refusal for each (7) |
| R15 | 7 | happy path: file gone, the staged generated rewrite done, `Gate` clean on the tree after completion, no other change |
| R16 | 8, 9 (incl. `-q` suppresses everything but findings, tested) | integration tests run the built binary: a fixture per command asserting stdout, files written, exit code; read-only byte identity after `check`, `status`, `docs preview`, `impact`, `reconcile` (reconcile builds into the ignored cargo target directory only, as the reconcile Contract already states: it writes no file in the repository) |
| R17 | 11 | with the capabilities set to `delivered` by the author and the three documents reconciled, `cargo-workplan change reconcile` on each of the repository's own three records prints an empty list and exits 0; then `change complete` is run for each and exits 0; output recorded in `validation.md`; `make docs-sync` idempotent; `docs_contract` and `Gate` clean |
| R20, R21 | 7 | refusal until the conditions hold; deletion is the last step; the only writes are the removal and the staged generated rewrite |
| R22 | 10, 11 | the three documents state the built contract; end `delivered` with bindings that resolve and a generated verification matrix |
| R24 | all | built on the merged raf-workplan API |

## Tasks

Each task: focused tests (command named in the task report), independent review `tasks/TN-review.md`,
commit by explicit path, push, issue update. Whole-task scope; no deferrals.

### Task 1: `exec` — runner, typed invocation, patterns
`Programs`, `Runner` trait (real, recording fake), `Invocation`/`Arg` with validated newtypes (D5),
the four pattern validators, bounded capture, timeout kill, environment, time bounds (D3, E12). Tests:
every pattern accepts/rejects (`-Zunstable-options`, `--exact`, `;`, space, leading `-`); argv-table
walk; timeout kill and output bound each with a passing neighbour; stub programs on PATH and via
`$CARGO`.

### Task 2: `git` module, `Gate::open`/`programs`, one load path
Move `git_tracked` and `.git` detection onto the runner; version gate as a `git::require_version` function (called first by `Lifecycle::open`, built in Task 4; not by `Gate::open`) (2.29 refused, 2.30 and 2.45.1
accepted) running first; `Gate::open`; root discovery function (64 MiB bound; the `.cargo/config.toml`
`build.rustc` fixture proves discovery does not run it); retarget `examples/docs_sync.rs` to
`Gate::open`. Existing `Gate`/`docs_sync` tests stay green. (D3, D4, E4, B1)

### Task 3: Bound executors and proof parsers
Test, doc-test and example runners per the argv table and proof rules; libtest/rustdoc parsers on literal
strings (zero entries, two entries, `0 passed`, ignored, `no_run`, `compile_fail`, `foo_more`,
`Wrapper<T>::get`, strict `... ok`); index-fact refusals (E10); fixtures per D11 including a never-ending
test killed and refused with elapsed time.

### Task 4: `change new` and `change impact`
`lifecycle`: slug validation, exclusive create via raf-okf writer, `--capability`/`--affects`, printed
template and state guidance, `Impact` value (D6, D8, E5, E8). Tests per R1, R2, R18.

### Task 5: Static obligations
`Obligation`, kinds, sorting, rendering, derived sources, `contract_effect: none`, tense list (D6).
Fixtures per R3-R6, R8, R9, R14 (static kinds) with "fixing removes the line".

### Task 6: `section-untouched`, `example-failing`, `verify`
Git-backed digest comparison (D7), example-failing via the executors, `verify [<slug>]` and no-slug form,
typed `Evidence` (D5). Fixtures per R7, R11 and the `example-failing` kind (fails, then is gone once the
example builds and runs; a non-`#[test]` binding reports zero tests as failing).

### Task 7: `complete` (D9)
Static-first order, execution, TOCTOU re-check, post-deletion model, staged removal + generated rewrite.
Fixtures per R10, R14, R15, R20, R21 including hostile bindings never reaching cargo.

### Task 8: `cargo-workplan` crate — root, check, status, docs
Workspace member, clap derive, cargo-subcommand form, root discovery through the library, `--root`,
`--format`, `-q`, `check`, `status`, `docs sync` (with `--copyright/--licence`), `docs preview`; `deny.toml`
(and `fuzz/deny.toml` if needed), REUSE, lints, inventory, README/AGENTS/SECURITY crate list. Tests via
`CARGO_BIN_EXE_cargo-workplan` per R12, R13, R16; `check` equals `Gate`.

### Task 9: CLI lifecycle commands
`change new|impact|reconcile|complete`, `verify`; slug starting with `-` exits 2 before any spawn; missing
slug exits 2 (except `new`); JSON array encoding (D12); end-to-end happy path; the printed template
placed in a capability (each of the four states, after following the printed state guidance) gives exactly one `Graph::proposals()` block, `check` exit 0 and one `unresolved-proposal` line; Git-2.29 stub exit 2 through the binary.

### Task 10: Scenario A and Scenario B fixtures
`tests/scenarios.rs` in `raf-workplan`: Scenario A (one capability redesigned four times through successive
changes in a real temporary git repo; at the end the contract is stated directly, links resolve, no
PROPOSED block or change file on the reading path, the rationale retained, every earlier state recoverable
with `git show`) and Scenario B (a `considered` capability, which the author sets to `delivered` before `complete` runs, becomes `delivered`; `complete` refuses until
Usage, Examples and bindings match the renamed API and the dropped claim has outcome `dropped`; then
succeeds leaving real usage, API references, limitations and a traceability table). Written to the
complete-gate Verification text; each assertion maps to a sentence there.

### Task 11: Reconcile, dogfood and deliver
Makefile `docs-sync` runs the binary (E6). Commit order (every commit must pass the pre-commit `docs_contract`, which runs `Gate` on the committed tree, so a PROPOSED block in a `designed` capability, or a `delivered` capability beside its open record, cannot be committed): (1) commit the capabilities as `implementing` with bindings and the INVENTORY row; (2) add the Contract edits as PROPOSED blocks (allowed in `implementing`) and commit; (3) resolve them, re-run `reconcile` until its list is empty, commit; (4) in the working tree only, set `state: delivered`, run `change reconcile` on each record (empty list, exit 0), then `change complete` on each record (it deletes the record and rewrites the staled blocks), then commit the state edit, the INVENTORY row, the record deletions and the rewrite together. Reconcile the three documents to the built behaviour within the
Contract-edit table (no Contract, Constraints or Verification statement outside the Contract-edit table is removed or weakened; an unbuilt statement is built or put to the owner), add generated blocks,
bind code/tests/examples, follow the numbered commit order above (its step 4 includes running `cargo-workplan change reconcile` on each record: empty list, exit 0, then `change complete`: exit 0, output into `validation.md`); fix stale statements elsewhere (AGENTS.md, README, SECURITY.md,
INVENTORY, CHANGELOG), `make docs-sync` idempotent, `docs_contract` and `Gate` clean.

### Task 12: PR, publication and handoff
`make check` locally (every target; `cross-check` hard-fails locally while its toolchain is not installed, since toolchain installation is outside this task's authority, and hosted CI on the PR is its recorded evidence by the owner's decision on the record: "Hosted CI counts (Recommended)" — run every other gate locally, quote the decision in the PR body, merge only after the hosted cross-check job is green; each `make` target is run individually so none is skipped by the first failure), `.deficiencies` empty, artifact inventory, completeness review
against this plan, `stagectl pr-create` (`Closes #8`, validation, residual risks, readings U1/D2/D7),
plan and least-confident note posted to issue and PR.

## Carried items from task reviews

* Task 9: `change new` takes optional `--copyright`/`--licence` (as `docs sync` does) and writes the SPDX header lines, so a tool-created record passes `make reuse` (T4 A1); Contract-edit row added when built. Fix A2 (the delivered-capability reason also names `state-delivered-open-record`) and A3 (`sync_needed` accuracy) there too.
* Task 7: `complete` re-opens the gate (`Gate::open`) for the re-check, since `Opened` freezes source (T2 A3). Task 6: pin that `blob_at` takes a `merge_base` result only (T2 A2).
* Task 11: the time-bounds Contract edit says the 300 s example limit includes the build `cargo run` performs (T3 A1); module docs note result-shaped output from trusted workspace code is not a defence boundary (T3 A3).
* Task 9 also: A7 (T8) add a test that runs through the real `cargo workplan` path (a PATH-resolved `cargo-workplan` under a stub `cargo` subcommand dispatch); A1/A2 (T8) move the open-change ordering and `Location` rendering into the library if it is small.

## 2. paudley — 

## Least confident about, and what you may not know

* **Windows and macOS have not run any of the new tests yet.** The process-group kill has no Windows equivalent (only the child is killed), the Windows doc-test path separator fix rests on a unit test, and the real-cargo fixtures have only run on Linux. The first hosted run on this PR is the evidence.
* **The completion gate's post-deletion check fails on every finding in the repository**, so an unrelated broken link anywhere blocks completing a change. That is deliberate (the tree after completion must be clean) but strict.
* **Owner readings are listed in the PR body**: `change new` prints the PROPOSED template instead of writing it; all three capabilities are delivered here together with Scenarios A and B; `rustix` as a unix-only dependency of `raf-workplan`; the `docs_sync` exit-code change for a root with source and no capability document; two record claims marked `dropped`.
* `cargo workplan verify` with no slug exits 1 on this repository because of three bindings of other delivered capabilities (tracked in issue 17).
* Strict coverage modes are tracked in issue 16.

