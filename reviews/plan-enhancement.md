# Plan enhancement audit

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/8-raf-workplan-change-lifecycle-cargo/.stage/raf-workplan-change-lifecycle-cargo/plan.md` (Revision 1)

Standing constraints: no `.goals` file; AGENTS.md governs (fail hard, no fallback, no deferrals,
no new `#![feature]`, no attribution, `.deficiencies` empty). No item below conflicts with it;
items that require a same-task edit of a capability Contract say so, which the plan already
permits ("where code and document must differ, the document is edited in the same task and the
reason recorded").

Inputs read: the plan, `issue-analysis.md`, `prior-art-assessment.md`, the three contract
documents (`workplan-reconcile`, `workplan-complete-gate`, `workplan-cli`) and their change
records, the delivered `workplan-document-model` and `workplan-generated-docs`, the five
`considered` workplan capabilities that build on this issue (`evidence-claims`,
`prose-assessment`, `two-way-coverage`, `entity-history`, `current-rationale`), SECURITY.md,
AGENTS.md, Makefile, `deny.toml`, `tests/docs_contract.rs`, and `crates/raf-workplan/src/*`
(`gate`, `model`, `graph`, `rules`, `generated` (grep), `sync`, `preview`, `bindings`,
`roadmap`, `finding`), `examples/docs_sync.rs`, and `crates/raf-source/src/target.rs`.

## The concept the feature is a shadow of

The issue asks for an obligation list, a five-condition gate and a CLI. The structure
underneath is an **assurance / consistency relation over four models**: the documents (D), the
source index (S), the Git history (H) and execution (E). `check` is the static part of the
relation over D and S; `reconcile` is the same relation *scoped to a change's footprint* and
extended with H and E; an obligation is a **claim with no current evidence** (the Contract's
own "Evidence that resolves it" column); `verify` is the evidence producer; and `complete` is a
**state transition** whose target state must satisfy the whole relation. Four of the five next
workplan capabilities say literally "receives an obligation in the reconcile list" or "the
completion gate requires ..." (evidence-claims, prose-assessment, two-way-coverage) or share the
Git executor (entity-history). The definitive version of this issue therefore builds (a)
completion as a transition with a post-state invariant, (b) obligations as unsupported claims
over a typed `Evidence` value from pluggable sources, and (c) a section digest that ignores
mechanical text. Items 1 to 3 are those three.

## Items (TRANSFORMATION first)

### 1. Completion is a transition: check the post-deletion state, and repair what the deletion itself stales (TRANSFORMATION)

**Finding.** The plan's happy path cannot hold as written. `generated.rs:465` renders the
`relationships` block's `| Changed by | [slug](../../work/changes/slug.md) |` rows from the
open change records. Condition 4 ("`docs sync` would rewrite nothing") therefore requires the
row to be present *before* deletion, and deletion makes that very row stale (and its link
broken) *after*. Every listed capability that carries a `relationships` block is `generated-stale`
the instant `complete` succeeds, so `Gate` is red immediately after a "completed" change. Task 10
adds exactly these blocks to the three capabilities, so issue 9's dogfood would hit this first.
Independently, `rules.rs:164` raises `no-open-record` for a non-delivered capability with no
listing record and `state-delivered-open-record` for a delivered one that still has a record:
there is no capability state in which both the pre-complete and the post-complete tree pass
`check`, unless `complete` reasons about the post-deletion tree.

**Enhancement.** Define `complete` as: load the model *as it will be without the record* (a
`Tree` wrapper that hides one path over the `DiskTree`, or a `MemoryTree` rebuilt without it;
`Workplan::load_with` already takes `&dyn Tree`), and require (i) the five conditions on the
current tree, as the Contract says, and (ii) `check` on the post-deletion model to be empty
apart from the generated blocks that the deletion itself stales, which `complete` rewrites
through the existing `Workplan::write` staging (temporary files, then renames) with the record
removal as the last rename-class operation. Report the post-state findings in the refusal
(`after deletion cap:x in state implementing has no open record; set state delivered in the same
commit`). This is where the five conditions become the Contract's promise "a change is complete
when the current documentation says what the system now does".

**Contract edit (same task, reason recorded):** in `workplan-complete-gate` replace "The deletion
is the last step and the only write" with: the writes are the record removal and the rewrite of
the generated blocks that listing the record kept current (the Changed-by rows), staged
together; nothing else is written. Mirror in `workplan-cli` Constraints ("`change complete`
deletes one file" becomes "deletes the record and rewrites the generated blocks it staled").
Minimal alternative if the owner refuses the edit: `complete` prints the post-deletion findings
and exits 0 only when the sole remaining finding is the Changed-by staleness, with the fix
`docs sync` named; this keeps the Contract but leaves `Gate` red between two commands, which
the capability's Purpose argues against.

- Cost: M. Payoff: decisive; without it the headline capability is false on the first real use.
- Changes: Task 7 ("deletion last and only write (remove-file with failure surfaced, never
  partial)"), the R10/R15/R17 rows of the completeness contract (add "and `Gate` is clean on the
  tree after completion" to the happy-path fixture), D2, and Task 10's Contract edits.

### 2. Obligations as unsupported claims over a typed `Evidence`, from pluggable sources, with a typed argv (TRANSFORMATION)

**Enhancement.** Three linked design moves, all in code the plan writes from scratch:

- **`Evidence` is the value `verify` produces per binding**, shaped as the record the
  `workplan-evidence-claims` capability will persist: the binding, the exact argv vector run,
  the revision (`HEAD`), the verdict, the proof counts (listed entries, passed/failed/ignored,
  `... ok` lines matched), and the wall time. `Obligation.evidence` is then a typed
  `Evidence`-required enum (`Diff { section, merge_base }`, `Outcome`, `ResolvedBinding`,
  `SyncRun`, `PassingRun`, `Rewrite`) with one `Display` that renders the Contract's exact
  strings, instead of a free `String`. The stub-argv fixtures assert the same argv the evidence
  carries, so the test oracle and the user-facing record are one value.
- **`reconcile` is a union over obligation sources** evaluated in a fixed order:
  `Static(&Workplan)` (unresolved-proposal, item-without-outcome, binding-missing,
  generated-stale, tense-mismatch), `MergeBase(&Git)` (section-untouched) and
  `Execution(&Runner)` (example-failing). Keep `ObligationKind::ALL` closed with its
  both-directions test, as D6 says, but make the *source* set the extension point so
  evidence-claims (`claim-unsupported`), prose-assessment (`assessment-stale`) and
  two-way-coverage (`item-undocumented`) add a source and a kind without touching `reconcile`
  or `complete`. Condition 1 then absorbs them automatically.
- **Typed argv.** Build every invocation as data: `Invocation { program: Program::Cargo | Git,
  args: Vec<Arg> }` where `Arg` is `Literal(&'static str)`, `Package(Pkg)`, `Target(TargetArg)`,
  `Filter(Name)`, `DocFilter(ItemPath)`, `RevPath { rev: Commit, path: RelPath }` and, already
  shaped for entity-history, `LineRangePath`. The validated newtypes are constructible only
  through the D4 pattern validators, so "document text reaches a process only after resolution
  and a stated pattern, in one of three forms" (SECURITY.md) becomes a property of the type
  rather than a per-call-site discipline, and one unit test over `Arg` proves it for every row
  of the table. The `exec` runner takes an `Invocation` only.

- Cost: M (same code volume as the plan, organised once). Payoff: the next three capabilities
  plug in; the security invariant is type-carried; `verify`'s output is already the M2 record.
- Changes: D3 ("`Programs { cargo, git }` ... injectable"), D5, D6 ("`Obligation { kind,
  capability, section, location, evidence }`"), Tasks 1, 2, 4, 5 (`VerifyReport::passed`
  becomes a report of `Evidence` values).

### 3. "Touched" means authored text differs: digest a section with generated blocks blanked (TRANSFORMATION, shared with prose-assessment)

**Finding.** D7: "'Touched' = heading span text differs." A `docs sync` that rewrites the
`verification-matrix` inside Verification, or the `relationships` block wherever it sits, makes
the section differ from the merge base without a single authored word, so `affects:
cap:x#verification` is silently discharged by a mechanical operation. The prose-assessment
capability already states the domain rule ("a regenerated generated block leaves the
neighbouring assessment current").

**Enhancement.** Compare the section's text with generated blocks blanked (fences and PROPOSED
blocks stay, because editing those *is* authoring), using the same blanking `graph.rs::prose_of`
performs, and express it as one `section_digest(body, section) -> Digest` function that
prose-assessment will reuse with more inputs. Parse the merge-base blob through `raf_okf::Body`
(the 1 MiB bound is the same document bound) so the comparison is section-structured, not a
byte-range guess. Add the fixture "only a generated block changed on the branch: still
reported".

- Cost: S/M. Payoff: the gate cannot be satisfied by `docs sync`; one digest for two
  capabilities.
- Changes: D7, Task 6 ("heading-span comparison"), and one sentence in the reconcile Contract
  ("A section is touched when its heading span differs" gains "outside generated blocks"),
  recorded as a same-task edit.

### 4. One load path for the binary, `Gate` and `docs_sync`; one spawn path including root discovery (LEVERAGE)

**Finding.** `examples/docs_sync.rs:81-108` already duplicates `gate.rs:287-353` (`git ls-files`
tracked paths, `.git` detection, the UTF-8 and `RelPath` checks). The CLI's `check`, `status`,
`docs`, `change` and `verify` all need the identically-loaded `Workplan` (the same
`LinkContext`), so Task 8 as planned writes a third copy. Separately, D3 says "No other module
spawns" while Task 8 puts the `cargo metadata` root discovery (64 MiB bound, `build.rustc`
fixture) in the binary.

**Enhancement.** Add `Gate::open(&self) -> Result<Opened, GateError>` returning the `DiskTree`,
`Workspace` and loaded `Workplan` (and make `try_check` call it), so the binary, the gate and
the example load through one function. Make the executor a `pub` module (or expose
`raf_workplan::root::discover(&Programs) -> Result<PathBuf, _>`) so root discovery is the
library's tested spawn with its bound and its `.cargo/config.toml` fixture, and D9's "the binary
holds no logic" is literally true.

- Cost: S/M. Payoff: no third copy for the census to find later; D3 holds; the binary's
  equivalence test (D9) becomes trivial.
- Changes: D1, D3, D9, Task 8 ("root discovery by exactly `cargo metadata` ..."), Task 10
  (retarget `docs_sync` to `Gate::open` or note why it stays).

### 5. Reuse the existing derived values instead of recomputing them (LEVERAGE)

- `binding-missing` for an `implementing` capability is exactly `RoadmapEntry::gaps()`
  (`Gap::EmptyList`, `Gap::Unresolved`); for `delivered` it is the existing
  `okf-binding-required` and `binding-*` findings; for `designed`/`considered` the rule forbids
  bindings, so nothing is owed. Derive the obligation from `Workplan::status()` and the
  `binding-*` findings filtered to the listed capabilities rather than re-walking bindings.
- `generated-stale` is `generated::analyze(self).into_findings()` filtered to the listed
  capabilities' files (and the index, which every change touches through the catalogue).
- Make `Impact { capabilities, sections, bindings, dependants, linking_sections }` a value that
  `change impact` prints and `reconcile` iterates, so the footprint is computed once and the
  two commands cannot disagree.
- `unresolved-proposal` comes from `Graph::proposals()` filtered by change slug; the line is
  on the `ProposedBlock` marker.

- Cost: S. Payoff: fewer rules to keep equal; the obligation text stays identical to the
  `check` text the author already sees.
- Changes: Task 3 ("impact from the graph"), Task 4.

### 6. Dogfood the binary from the Makefile, and resolve the CLI's index-creation gap (LEVERAGE)

**Finding.** `make docs-sync` runs the `docs_sync` example with `--copyright`/`--licence`. The
CLI Contract's `docs sync` has no such options, and `Workplan::sync` refuses to create a missing
`docs/capabilities/index.md` without them (`SyncError::SpdxUnset`), so `cargo workplan docs
sync` on a repository without an index exits 2 with no way forward. The example stays (it is a
bound example of the delivered generated-docs capability), but after this issue the repository's
own workflow should exercise the binary.

**Enhancement.** Either add `--copyright <text> --licence <id>` to `docs sync` (Contract edit,
read only when the index must be created, mirroring the example) or state in the CLI Contract
that `docs sync` never creates the index and names the example; in both cases add the fixture
"missing index: `docs sync` exits 2 naming the field" or "creates it". Retarget `make
docs-sync` to `cargo run --locked -p cargo-workplan -- workplan docs sync --root '$(ROOT)' ...`
so the binary is run by every maintainer and by the hook path.

- Cost: S. Payoff: the CLI is used, not only tested; a real exit-2 dead end is closed.
- Changes: Task 8, Task 10 ("`make docs-sync` idempotent", AGENTS/README updates).

### 7. Batch the proof runs per target (UTILITY, Contract edit)

**Finding.** The argv table runs one list pass and one run pass *per binding*. `verify` with no
slug over this repository is roughly 150 test bindings, so about 300 `cargo test` spawns, each
with cargo's own up-to-date check; several minutes on a warm cache and the same cost inside
`complete`.

**Enhancement.** Group bindings by `(package, target)`: one list pass per target (`-- --list`,
no filter), select the bound names from the parsed output (each must appear exactly once, as
now), then one run pass per target with every bound name after `-- --exact --` (libtest accepts
several exact filters) and the proof `N passed; 0 failed; 0 ignored` where N is the number of
bound names in that target. Doc tests already list once per package. The security form (names
only after `--`) is unchanged; the per-binding proof is preserved by the parser. Record the
batched rows in the Contract's argv table in the same task.

- Cost: M. Payoff: `verify`/`complete` on a real repository become seconds, not minutes; the
  parser module (D5) is the same.
- Changes: D5, Task 2 ("Stub-cargo argv-equality fixtures against each row"), Task 5.

### 8. `change new --affects` and next-step guidance (UTILITY, Contract edit)

**Finding.** R18 says a change "names the documentation it affects when it is planned", but the
Contract's `change new` writes no `affects`, so every record needs a hand edit of its
frontmatter before any PROPOSED block is valid (`proposed-section-not-affected`) and before
`impact`/`reconcile` have any section to report. A change to a `delivered` capability also
needs the capability's state moved to `implementing` in the same commit
(`state-delivered-open-record`), which the tool knows and the author must remember.

**Enhancement.** Add a repeatable `--affects cap:<slug>#<section>` (validated against the seven
sections and against `--capability`), and print after creation: the PROPOSED block template
(U1), and for each named capability that is `delivered`, "set `state: implementing` in the same
commit". No second file is written, so the CLI Constraint holds.

- Cost: S. Payoff: R18 is met by the tool rather than by convention; the first `check` after
  `change new` is green.
- Changes: Task 3, D8/U1, the reconcile Contract's `change new` paragraph and the CLI table.

### 9. `complete` refuses on the static conditions before it executes anything (UTILITY/ROBUSTNESS)

**Enhancement.** Evaluate conditions 1 (static sources), 2, 3 and 4 and the post-state invariant
(item 1) first; spawn `cargo` for condition 5 and `example-failing` only when they hold. The
refusal still lists every static failure, matching the Contract's example output; it just does
not spend a test run (or execute the workspace) while the documents are inconsistent. State the
order in the Contract's Conditions paragraph.

- Cost: S. Payoff: a five-minute refusal becomes instant; executing untrusted-shaped input
  while the record is malformed is avoided.
- Changes: Task 7, the complete-gate Contract (one sentence).

### 10. Refuse from index facts before spawning; strip the crate path correctly (ROBUSTNESS)

**Finding.** `raf_source::Target` already records `harness`, `required_features`, `kind` and
`crate_path`. A `harness = false` test target does not understand `--list`, so the list pass
fails and the proof rule cannot apply; a target behind `required-features` cannot be built by the
fixed argv (no `--features` row), so the run fails for a reason the obligation would misname;
`<name>` must be the libtest name, which is the item path with `crate_path` removed
(`raf_workplan::test_crate::documents::x` becomes `documents::x`; a `#[cfg(test)] mod tests`
function in `src/lib.rs` becomes `bindings::tests::x`); `--test=<target>` takes the Cargo target
name, which for `tests/parse-errors.rs` is `parse-errors` while the index path spells
`parse_errors`.

**Enhancement.** Add typed refusals (`harness-off`, `required-features`, `not-a-test-target`)
raised from the index before any spawn, with the message naming the target and the key, and
fixtures for: an integration-test binding, a `#[cfg(test)]` module binding, a hyphenated test
target, a `harness = false` target, a `required-features` target, and a bench target.

- Cost: S. Payoff: precise refusals instead of libtest noise; the proof rule is never applied
  where it cannot hold.
- Changes: Task 2 (list of parser fixtures), D5.

### 11. Make the real-cargo fixtures viable: workspace isolation, lock files, toolchain, target dir, budget (ROBUSTNESS)

**Finding.** The plan runs real `cargo` on fixture crates (Tasks 5, 7, 8) but does not say
where they live or how they build. Each fixture under the repository root without its own
`[workspace]` table fails with "believes it's in a workspace when it's not"; `--frozen` fails
without a committed `Cargo.lock`; a fixture copied to a temp directory is outside the
repository's `rust-toolchain.toml`, so it builds on the default toolchain and must compile on
the MSRV; a fresh target directory per fixture recompiles everything; stub tests must override
`CARGO` because `cargo test` sets it for the outer process; and `make msrv`/`make clippy`
(`--all-targets`) must not be slowed by fixture crates.

**Enhancement.** State the fixture discipline once in the plan: fixtures are generated into a
temp directory from a few in-test string templates (as the existing `support/` helpers do),
each with `[workspace]`, a committed-in-template `Cargo.lock`, no dependencies, edition pinned,
and `CARGO_TARGET_DIR` set to one shared temp directory per test binary; stub tests set `CARGO`
explicitly and clear `RUSTFLAGS`-class variables that would change argv expectations; and a
wall-clock budget assertion keeps the real-cargo suite under a stated bound. Add the
`cfg(windows)` plan for these (they run there; only the shell stubs skip).

- Cost: S/M. Payoff: the fixtures work the first time and stay fast; CI on three platforms
  behaves identically.
- Changes: D11, Tasks 5, 7, 8.

### 12. Name the time bound for test and build runs (ROBUSTNESS, Contract edit)

**Finding.** The Contract gives a 300 s timeout for example targets only. D3 says "a wall-clock
timeout with kill" on every spawn but no value for `cargo test` (which includes a compile).
A hanging bound test, or a compile that never ends, hangs `complete` and `verify` forever;
a too-small bound fails a cold build.

**Enhancement.** One bound per invocation class with a stated default (for instance 1800 s for
a `cargo test`/`cargo run` that may compile, 300 s for the example's own run as written), set
once on `Programs`/the runner, recorded in the complete-gate Contract beside the examples rule,
with a fixture that a never-ending test binding is killed and refused with the elapsed time.

- Cost: S. Payoff: the gate never hangs; the bound is documented where the other one is.
- Changes: D3, Task 1 ("timeout kill"), Task 5.

### 13. An in-process `Runner` fake beside the PATH stubs (ROBUSTNESS)

**Enhancement.** With item 2's `Invocation` value, a recording `Runner` fake (returns canned
stdout/stderr/status per invocation, records every call) runs the whole argv matrix, the
hostile-binding matrix and the "exits 2 before any other process" ordering tests in-process on
every platform, with no temp files and no `cfg(unix)` skip. Keep one real PATH-stub and one
`$CARGO`-stub test per program as the Contract's Verification asks, so the end-to-end proof
remains.

- Cost: S/M. Payoff: the Windows job stops skipping the security matrix; the tests are pure and
  fast.
- Changes: D11 ("Argv-recording stubs ... POSIX shell scripts ... the Windows job skips them").

### 14. The two argv shapes of a cargo subcommand, and the JSON shape (ROBUSTNESS)

**Finding.** `cargo workplan check` reaches the binary as `[cargo-workplan, workplan, check]`;
`CARGO_BIN_EXE_cargo-workplan` tests call it as `[cargo-workplan, check]`. The plan's Task 8/9
tests exercise only the second form, so the first (the one every user types) is untested.
The CLI Contract says "`--format json` prints one object per finding" without fixing array vs
newline-delimited, and obligations carry fields (`capability`, `evidence`, `line`) the four
named keys do not cover.

**Enhancement.** Use clap's cargo-subcommand pattern (`bin_name = "cargo"`, a `workplan`
subcommand) and test both forms for every command; pin the JSON encoding (recommend one JSON
array per run on stdout, one object per finding/obligation/binding with a `kind` discriminator
and the extra fields present) in the Contract and a fixture.

- Cost: S. Payoff: the typed invocation works; agents get a stable schema.
- Changes: Task 8 (parse tests), Task 9 ("JSON object per finding with `kind`, `path`,
  `section`, `message`").

### 15. Small pins that make the stated guarantees testable (ROBUSTNESS)

- Obligation ordering: the Contract sorts "by capability, section and kind"; add the tiebreak
  (line, then evidence text) so "equal inputs print equal output" holds when one section has
  two PROPOSED blocks, and test it.
- Tense phrases: a `const` list with a both-directions test, word-boundary matching, and
  exclusion of code spans as well as fences and PROPOSED blocks; a near-miss fixture
  ("goodwill", "`will` in a code span", "William").
- Git: run every `git` with `current_dir(root)`, not the caller's directory, because the binary
  "runs from any directory inside the workspace"; when `refs/remotes/origin/HEAD` is unset the
  error names the fix (`git remote set-head origin --auto`); pin the behaviour on the default
  branch itself (merge base is `HEAD`, so every affected section is reported, an obligation and
  not an error); and a linked worktree (this repository's `.worktrees/`) fixture, since
  `symbolic-ref` and `cat-file` resolve through the common directory.
- `complete` TOCTOU: re-evaluate the static conditions after the long execution step and
  immediately before the writes, so an edit made during the test run cannot slip past.

- Cost: S. Payoff: each is a guarantee the Contract already makes, now with a negative fixture
  (AGENTS.md: "Every rule has a negative fixture").
- Changes: Tasks 4, 6, 7.

## Not raised

Scenario A/B placement (D2), the U1 reading of R1, and exit-code mapping are dispositioned in
the plan with reasons and belong to the completeness and constraint reviewers. The seven
obligation kinds are kept closed as the Contract fixes them; item 2 extends the sources, not the
kinds.

VERDICT: NEEDS-REVISION
