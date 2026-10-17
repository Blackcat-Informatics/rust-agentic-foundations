# Task 9 review: cargo-workplan commands

## Scope

The staged diff (23 files) on top of c54db19. I read:

* plan D4, D8, D11-D13, Task 9, R1-R4 and the carried items (T5: `--` and `TrustedArgv`);
* T4-T8 reviews, AGENTS.md, the SECURITY.md diff;
* the PROPOSED additions to `docs/capabilities/workplan-cli.md` and `workplan-evidence-claims.md`;
* the `work/changes/workplan-cli.md` diff;
* all of `crates/cargo-workplan` (main.rs docs, cli.rs, command.rs, evidence.rs, assess.rs, coverage.rs, history.rs, render.rs);
* the new tests.

Experiments ran in two scratch worktrees (`/tmp/t9rev` for the gates, `/tmp/t9fx` for the hostile runs and the mutants), each HEAD plus the staged diff applied with `git apply --index`. Both were removed afterwards. Nothing was added, reset, stashed or committed in the issue worktree, and its staged diff is byte-identical to what it was before the review.

## Evidence

### Gates (scratch = HEAD + staged diff, CARGO_BUILD_JOBS=6). Every one exited 0.

* `cargo test --workspace --locked` (1910 passed).
* `cargo test -p raf-workplan -p cargo-workplan --release --locked` and `make test-release`.
* `make fmt` (no change), `make clippy` (`-D warnings`) and `make doc` (`-Dwarnings`).
* `cargo test --test docs_contract` and `make issue-refs` (repo_gates).
* `make reuse`, `make deny` and `make msrv`.
* `cargo run -q -p cargo-workplan -- check`.
* `make docs-sync` twice: the tree was unchanged and the second run was idempotent.
* `docs preview workplan-cli`.

### (1) The trusted argv seam (built binary, hostile command lines)

**Words after `--` reach the command untouched.** `/tmp/argv.sh -- -q --format=json --root / --by human --from-binding a --id y --` received every word in order, the second `--` included. The record kept `by: agent` and was otherwise unaffected.

**Through real `cargo workplan` dispatch.** `cargo workplan evidence record ... -- /tmp/argv.sh workplan -- --id` passed `[workplan] [--] [--id]` through. The `workplan` word that cargo inserts does not shift the first-`--` scan. `cargo -q workplan ...` behaves the same.

**Refusals:**

* By clap (exit 2, nothing started): `--root -- ...`, `--format -- ...`, `-- evidence record ...`, `--copyright -- --licence x -- true`, `--id` given twice, `--from-binding=x -- true` (both sources), and neither source (`... --by agent` and `... --by agent --`).
* By the `word`/`undashed` parsers: `--copyright=--`, `--licence=--`, `--from-binding=--`, `--limit=--` and `--revision=--`.

**Refused before root discovery (no cargo started):**

* `-- ""`: "the program after `--` is the empty word".
* A non-UTF-8 word after `--`: "word 1 of the command is not UTF-8 ... nothing was run".
* A non-UTF-8 option value before `--` is a clap error.

**Long argv.** A 20000-word argv records normally.

**Backstop.** It is reachable: `exec -a -- cargo-workplan evidence record ... -- sh -c 'touch ran.txt'` (argv[0] is `--`) is refused with exit 2, and `ran.txt` is not created.

**No document text reaches the process.** The stored `command` is only written. `Program::Trusted` is reachable only through `TrustedArgv::from_command_line(std::env::args_os())`, unchanged since Task 5.

**`--from-binding x --` with an empty command** is accepted. No words are run, so this is not a "both" case (advisory A1).

### (2) Exit codes (built binary)

* **`history`:** `arch:normal-form` exits 0. `cap:nope` exits 2, and the message lists the ids.
* **`evidence record`:**
  * exit 0 for pass, 1 for fail (by exit status 3, and by a value over its limit), with the record written;
  * exit 2 with no record for `1e-3`, a program that cannot start, a file that exists without `--update`, an unknown `--for`, and limits with `--from-binding`. None of these runs the command.
* **`evidence claims` / `evidence for`:** 0 with an answer (an empty answer gets a note), 2 for an unknown id or capability.
* **`assess`:**
  * `--context`, `--verdict accepted` and `--verdict rejected` exit 0, and so does `--list` (current, and stale in the tests);
  * an unknown capability with `--context` exits 2;
  * an unknown capability with `--list` exits 0 (R3 below).
* **`coverage`:** 1 with gaps (whole workspace), 0 for `--crate raf_workplan`, 2 for `--crate nope`.
* **No mismatches found.** No path prints findings with exit 0, or a tool failure with exit 1.
* **`--list` exiting 0 on stale or rejected:** the Contract states this and gives the reason (`change reconcile` owns the obligation). It is consistent with `history` being an outcome. I accept it.

### (3) Validation before spawn

`process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts` covers 25 refusals, in both shapes, with and without `--root`. With the stubs recording, no cargo, no git, no marker file and a byte-identical tree; there is also a neighbour that starts a process.

Every refusal in that list is a clap-level refusal. None comes from `evidence::source` (R1).

### (4) JSON

**Pinned in tests:**

* the kinds and their non-collision with rule names (`tests/json.rs`);
* the member order of `evidence-recorded`, `evidence-claim`, `evidence-for`, `assess-context`, `assessment-recorded`, `assessment`, `history-entry`, `history-removed` and `coverage-gap`, in the binary tests;
* `coverage-excluded` and `coverage-error` in the raf-workplan render tests.

**Matches main.rs.** The pinned member orders match the table in main.rs.

**`-q`:**

* An outcome block prints nothing in text and `[]` in JSON.
* A `fail` record and a coverage gap stay.
* Checked live: `-q evidence for`, `-q --format json evidence claims`, `-q coverage code --crate raf-workplan` and `-q assess --list`.

### (5) Writes and replacement

**Read-only commands leave the tree unchanged.** I hashed the tree before and after `history`, `evidence claims` and `evidence for`, `assess --context` and `assess --list`, and both `coverage` directions with and without `-q`. The hashes were identical.

**Recording writes exactly one file.**

* `evidence record` and `assess --verdict` each write one file. The binary tests check this with `changed(before, after) == [RECORD]`.
* `--update` replaces the record.
* `--copyright`/`--licence` give the two SPDX lines on both kinds of record.
* A replacement without `--copyright` drops them (A4).

### (6) `--by` is required on `evidence record`

The record type has a `by` field, and the no-fallback policy rules out a default. The PROPOSED synopsis was updated in both workplan-cli and evidence-claims. I accept the departure from the brief.

### (7) Bindings

35 tests were added to `cap:workplan-cli`. Each one I sampled proves CLI behaviour that the workplan-cli PROPOSED Verification lists:

* parsing in both shapes;
* the argv seam;
* exit codes 0/1/2 through the binary;
* JSON members;
* `-q`;
* byte identity;
* the real-repository coverage.

I sampled 15: the 8 cli unit tests, the evidence unit test, the render unit test, and the tests `evidence::{a_command_that_passes..., a_failing_command..., a_request_that_cannot..., the_words_after...}` and `assess::the_context_is_the_same...`.

**Check and coverage.** `check` exits 0, so coverage is clean.

**The `workplan-cli` record was widened, not recreated:**

* same id;
* six honest items and no outcomes;
* `affects` names contract, usage, examples, constraints and verification, matching the PROPOSED blocks;
* `contract_effect: changes`.

### (8) Documents

* **Outside PROPOSED blocks,** the capability diffs touch only the `bindings` frontmatter and the generated verification matrix.
* **evidence-claims** edits are inside PROPOSED.
* **`docs preview workplan-cli`** exits 0.
* **README, CHANGELOG and SECURITY** describe commands that exist (but see A3).

### (9) Mutants (in `/tmp/t9fx`)

| Mutant | Result |
| --- | --- |
| M1: a `fail` record classed as an outcome | KILLED (3 binary tests) |
| M2: backstop `agree` accepts a shifted reading | KILLED (`evidence::tests::the_command_runs_only_when_both_readings...`) |
| M3: root discovery before `evidence::source` (spawn before validate) | **SURVIVED** |
| M4: `-q` hides a finding block | KILLED (render test) |
| M5: a second file written by `evidence record` | KILLED (3 binary tests) |
| M6a: `coverage code` ignores an unlistable surface | **SURVIVED** |
| M6b: a coverage gap classed as an outcome | KILLED (4 tests) |
| M7a: `--from-binding` accepts a code binding | KILLED by the raf-workplan lib test (`every_refusal_of_the_request_starts_nothing...`); survives the binary tests, which is acceptable since the binary only delegates |
| M7b: `--from-binding` path check replaced by the first binding | KILLED (2 binary tests) |

The first M2 and M6a variants and the first M7a variant broke the build, so I re-ran each with a variant that compiles. The results above are for those variants.

### (10) Hygiene

* SPDX lines are on all eight new files.
* The added lines hold no tracker numbers, process wording, attribution or `unsafe`.
* There is no new dependency (Cargo.lock is unchanged).

## Findings

### REQUIRED

1. **No test pins that a refusal by `evidence::source` starts no process (M3 survives).**
   * **Where the claim is made:** SECURITY.md ("refuses the run before the root is looked for"), the command.rs and evidence.rs docs, and the PROPOSED Contract ("refused before anything starts").
   * **The gap:** the stub fixture lists only clap-level refusals.
   * **Fix:** add to `process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts`, without `--root` (so cargo would be asked):
     * an empty program word (`-- ""`);
     * a non-UTF-8 word after `--`;
     * a run whose two readings differ (argv[0] set to `--` through `CommandExt::arg0`).

     Assert that `cargo_calls()` and `git_calls()` stay empty.
2. **`coverage code` exiting 1 for a surface that cannot be listed is untested at the binary (M6a survives).**
   * **Where the claim is made:** main.rs (the exit codes and the `coverage` command), the PROPOSED Contract, and the `workplan-cli` record item.
   * **Fix:** add a binary fixture whose crate root cannot be listed (for instance an unresolved `pub use`). Assert:
     * `coverage code` exits 1 with a `coverage-error` object (and its members in order);
     * `-q` keeps it;
     * `coverage tests` on the same fixture exits 0 when it has no test gap.
3. **`assess cap:<unknown>#<section> --list` exits 0 with a note**, but the PROPOSED Contract says "Each exits 2 when the tool cannot run: an id or a capability that no loaded document has". `--context` and `evidence for` do exit 2 for the same input. Either:
   * refuse (exit 2) a `--list` subject whose capability no document has, and add the case to `assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing`; or
   * narrow the Contract, the main.rs exit-code list and the record item to say `--list` with a subject is a filter, and that an unknown capability gets a note and exit 0.

   The first is the more consistent choice.

### ADVISORY

* **A1.** `evidence record ... --from-binding <path> --` (an empty command) is accepted. Nothing runs, but refusing a bare `--` beside `--from-binding` would make "a command and `--from-binding` exclude each other" literal.
* **A2.** `TrustedArgv::from_command_line` scans argv[0] too, so an argv[0] of `--` is caught only by the backstop. Its message ("an option took `--` as its value") is wrong for that case. Skipping argv[0] in the scan, or a neutral message, would fix it.
* **A3.** AGENTS.md says "`cargo-workplan` is **delivered** with all of its commands" and lists the new ones. README does the same. `cap:workplan-cli` is `implementing` and their Contract is still PROPOSED. Reword until Task 11 delivers it, for example by listing the new commands as "implementing".
* **A4.** `--update` without `--copyright`/`--licence` drops the SPDX lines of the record it replaces. Either state this in the Contract or require the pair again. Task 10/11 should pass the pair on every re-record.
* **A5.** In the uncommitted scratch, `history cap:workplan-cli` fails with git's raw "There is no path ... in the commit" for a bound file not yet committed. That is exit 2, correct behaviour, but the message is opaque. The fix is a library message (Task 4 territory), and Task 10 will meet it if it runs history before committing.
* **A6.** Small text and JSON nits:
  * `evidence claims x` notes "names x" rather than `evidence:x`.
  * The wildcard arms produce an undocumented `"other"` for `failed.reason` and for `moved.change`. They are unreachable today and exist only because of the non-exhaustive enums. Document them, or note them in main.rs.
  * The inner members of `assess-context.symbols[].definitions` (`file`, `text`) are not listed in the main.rs table.


## Recheck after fixes

The new staged diff has 26 files. It adds `crates/raf-workplan/src/history.rs`, `crates/raf-workplan/tests/history.rs` and `docs/capabilities/workplan-entity-history.md`. I applied it to fresh scratch worktrees made from HEAD and removed them afterwards. The staged diff in the issue worktree was byte-identical before and after.

### Gates

These all exited 0:

* `cargo test --workspace --locked` (1915 passed).
* `cargo test -p raf-workplan -p cargo-workplan --release --locked` and `make test-release`.
* `make fmt` (no change), `make clippy` and `make doc`.
* `docs_contract` and `make issue-refs`.
* `make reuse`, `make deny` and `make msrv`.
* `cargo run -p cargo-workplan -- check`.
* `make docs-sync` twice: the tree was unchanged and the second run was idempotent.
* `docs preview workplan-cli`.

`docs preview workplan-entity-history` exits 2 at line 113. That is the fenced PROPOSED block the Task 8 review already listed (lines 112-116); Task 11 resolves it by hand. The new A5 text in that block holds no fence, so this is not a new defect.

### Mutants

| Mutant | Result |
| --- | --- |
| M3: root discovery before `evidence::source` | KILLED (`process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts`) |
| M6a: `coverage code` ignores an unlistable surface | KILLED (`coverage::a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only`) |
| New M8: the `--list` subject check dropped | KILLED (`assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing`) |
| New M9: bare `--` beside `--from-binding` allowed | KILLED (`evidence::tests::a_program_named_the_separator_and_a_binding_beside_one_are_refused`) |
| New M10: `SymbolLog` never chosen | KILLED (`raf-workplan` `history::a_symbol_log_that_git_refuses_names_the_binding_and_the_file_to_commit`) |

### Live checks on the built binary

Each of these exits 2 with the message shown:

* `assess cap:nope#usage --list`: "`cap:nope` names no capability document".
* `evidence record ... --from-binding x --`: "`--from-binding` and a command after `--` exclude each other".
* argv[0] set to `--`: the program-name message.
* `history cap:workplan-cli` on the uncommitted scratch:
  * the message names the binding `...assess::assess` and the file `crates/cargo-workplan/src/assess.rs`;
  * it keeps git's own text and advises committing the file.

### The A5 library change (`HistoryError::SymbolLog`)

The mapping is narrow. Only `GitError::Failed` (git ran and exited non-zero) of the `-L` row of a bound symbol becomes `SymbolLog`. These stay `HistoryError::Git`:

* a git that cannot start, a timeout, output over the bound, or output that is not a log;
* every failure of the version, shallow, document-log and pickaxe rows.

The source error is boxed and returned by `Error::source`, and its text is embedded, so no information is lost.

There is one residual mis-classification (A7 below): any other non-zero exit of `git log -L` gets the same "commit the file" advice. Examples are a corrupt object, or a missing promisor object in a partial clone. Git's own text is still shown.

The capability text sits inside a PROPOSED block, and the new test is bound to `cap:workplan-entity-history`.

### R1-R3

All three are resolved.

### Advisories

A1-A6 are addressed: A4 is documented and has a test.

### New advisories

* **A7.** The `SymbolLog` text asserts "have no history" and "commit `<file>`" for every non-zero `git log -L`. Saying "`git log -L` could not read the lines from `HEAD`" (and "if the file is not committed, commit it") would not mislead on the rare corrupt or partial-clone failure.
* **A8.** `assess <section> --list` now runs `assess_context` to validate the subject. An existing assessment whose section can no longer be assessed (for instance a section that was removed) is then exit 2 when it is named, but is still listed by a bare `--list`. That is defensible. Consider stating it in the Contract.

VERDICT: PASS
