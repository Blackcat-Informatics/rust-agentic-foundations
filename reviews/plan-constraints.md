# Plan constraints review

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/9-acceptance-scenarios-a-b-workplan-self/.stage/acceptance-scenarios-a-b-workplan-self/plan.md`

## Governing documents

Read:
* `CLAUDE.md` (worktree copy; it includes the "No timing runs" rule)
* `AGENTS.md`
* `.deficiencies` (notice and marker only, no entries: clean)
* `.github/PULL_REQUEST_TEMPLATE.md` (the checklist the PR has to satisfy)
* `.githooks/pre-commit`, `Makefile`, `crates/raf-workplan/src/refs.rs` (the issue-refs pattern set), `crates/raf-okf/tests/validate_cost.rs`, `tests/docs_contract.rs`, `.gitignore`
* The user's global attribution rules

Recorded absent: `.baseline`, `.goals`, `CONSTITUTION.md`, `PRECOMMIT_GUIDANCE.md`, `docs/adr/`, `docs/decisions/`. There are no nested `AGENTS.md` or `CLAUDE.md` files.

User decisions in force:
1. The self-census gate waits for its dependency: "Leave the 6 work for later."
2. No timing runs on this repo.
3. No issue or PR numbers in committed files.

## User decisions recorded, not cleared silently

* **Census criterion left open.** The plan departs from `AGENTS.md:264-266` ("No deferrals. Work is done now. Scope that leaves a change is moved to another tracked change...") only because the owner said so, and it quotes that decision verbatim (plan:4, plan:19, plan:28, plan:33). The criterion stays in the open issue and the PR says "Part of", so no scope is silently dropped. The decision covers this. It is not a violation. The plan must also keep "later", "follow-up" and "pending the census" wording out of every committed file (`AGENTS.md:266`). Nothing in the plan proposes such text.

## Findings

### BLOCKING 1: the validation route conflicts with the no-timing decision, and the plan does not resolve it

Governing text:
* `CLAUDE.md` (worktree): "**No timing runs**: time-based testing is forbidden. Write no timing runs, wall-clock benchmarks, sleep-based waits or timing assertions. ... tell every agent you start this rule."
* Owner decision: no timing runs on this repo.
* Against it, `AGENTS.md:138` says "Run `make check` before claiming work is done", and `.github/PULL_REQUEST_TEMPLATE.md:9` says "`make check` passes (... release tests ...)".
* `make check` runs `test-release`, which runs `cargo test -p raf-okf --release --locked --test validate_cost` (`Makefile:63-65`). That is a wall-clock assertion (`crates/raf-okf/tests/validate_cost.rs:16,71-84`: `Instant::now()`, `elapsed < RELEASE_BOUND`). `make test` (`cargo test --workspace`) also runs that file in debug, where it still times the work with `Instant` but does not assert.

Offending plan lines:
* plan:15: "`make test` includes it; the pre-commit snapshot run passes"
* plan:25: "Record scenario evidence in validation.md ..., open the PR"

These lines name no full-gate command, so the plan never says how it meets the `make check` checklist without a timing run. An executor following `AGENTS.md:138` would run `make check` and break the owner decision. One following the owner decision would open a PR whose `make check` box is unverified, and the plan says neither.

REWRITE TO (add to Task 2, before "open the PR"):
> Validation: run every `make check` stage except the wall-clock probe. That means `make fmt clippy doc deny reuse msrv fuzz cross-check licenses issue-refs`, plus `cargo test --workspace --locked -- --skip within_the_bound` and `cargo test -p raf-source --release --locked`. `crates/raf-okf/tests/validate_cost.rs` is the one test not run locally, by owner decision (no timing runs). CI runs it. The PR body states this exception against the `make check` checklist item and does not tick that box as fully run. Every agent started for this work is told the no-timing rule. The new planted-fault test uses no sleeps, no timeouts and no `Instant`.

(If the owner wants a different resolution, record it as a quoted decision. Do not leave it implicit.)

### BLOCKING 2: a second hard-coded capability-state list that also forbids the documented reopen workflow

Governing text:
* `AGENTS.md:234-236`: "The document-contract test pins the set of capabilities in **one inventory table** (`INVENTORY` in `tests/docs_contract.rs`): changing a state or adding a capability edits that table in the same commit."
* `AGENTS.md:239-241`: "Change records live in `work/changes/<slug>.md`. Each names the capabilities it changes".
* The repository's own guard, `crates/raf-workplan/tests/roadmap.rs:467-468`, treats a built capability as "delivered, or implementing with an open change record named after it".

Offending plan line, plan:16:
> "test asserts the 12 M1 capabilities are `delivered`, each holds a generated `verification-matrix` block, and no `work/changes` record names an M1 capability"

That creates a second table of pinned states in `tests/self_workplan.rs`, against the one-table rule. It also makes any future change record that names one of these capabilities fail the self gate, which forbids the reopen path that the workflow and `roadmap.rs` permit.

REWRITE TO:
> The test takes the built set from `INVENTORY` (shared through `tests/support/mod.rs` or reached from `docs_contract.rs`), not from a second list. For each capability that `INVENTORY` pins as delivered, it asserts resolved bindings and a generated `verification-matrix` block. It applies the same built-or-reopened rule as `roadmap.rs`: delivered, or implementing under an open change record named after it. It does not assert that no change record names the capability.

### BLOCKING 3: "M1" is development-process wording and must not reach a committed file

Governing text:
* `CLAUDE.md`: "**Never**: ... write issue or pull request numbers or development-process wording into any committed file."
* `AGENTS.md:258-261`: "Do not write issue or pull request numbers, tracker links, or wording that refers to the development process that produced a file (plans, tasks, acceptance criteria, branches). Documents describe the system, not its history."

"M1" is a tracker milestone label. No committed file uses it today (a grep finds only module names such as `m1` in fixtures). The `refs.rs` scanner would not catch it, but the rule is broader than the scanner.

Offending plan lines:
* plan:16: "asserts the 12 M1 capabilities ..."
* plan:31: "'M1' ... taken as the twelve capabilities ..."

REWRITE TO (plan:16):
> test asserts that every capability `INVENTORY` pins as delivered ... (identifiers, doc comments and assertion messages in `tests/self_workplan.rs` say "delivered capabilities" or "built capabilities", never "M1", "milestone", "scenario A/B criterion", or any issue reference)

### ADVISORY 1: name the documents that describe the hook and `docs_contract`

Governing text:
* `AGENTS.md:139-143`: "The pre-commit hook runs fmt and reuse ... and the `repo_gates` and `docs_contract` test binaries".
* `AGENTS.md:40-46` and `AGENTS.md:222-227` say `docs_contract` runs the workplan checks.
* `AGENTS.md:261`: "Documents describe the system".

Moving `repository_documents_satisfy_the_document_model` (tests/docs_contract.rs:582-588) and adding a third hook binary makes all of the following stale:
* `AGENTS.md:40-46`, `:139-143` and `:222-227`
* `CONTRIBUTING.md:60`
* `Makefile:9-10`
* `.githooks/pre-commit:5-7` and `:10` ("The two test binaries")
* the module doc at `tests/docs_contract.rs:12-13` ("`Gate::workspace` runs the same checks from disk")

Offending plan line, plan:22: "adjust the Makefile and docs comments".

REWRITE TO:
> Update the descriptions of the hook and the gate binaries in `AGENTS.md` (sections 1, 3 and 6), `CONTRIBUTING.md`, the `Makefile` header, the `.githooks/pre-commit` header ("three test binaries"), and the `tests/docs_contract.rs` module doc, in the same commit as the move.

Also make sure the hook edit only adds the binary. `AGENTS.md:149-150` says "do not edit a hook to pass".

### ADVISORY 2: SPDX header and fixture placement for the new test

Governing text:
* `AGENTS.md:194-196`: code is `MIT OR Apache-2.0 OR MulanPSL-2.0` "with the SPDX lines in the file's own comment style".
* `.github/PULL_REQUEST_TEMPLATE.md:11`: "Every new file carries an SPDX header".

Offending plan line, plan:22: "Create the test file ...; fixture test that the gate fails on a planted fault."

REWRITE TO:
> `tests/self_workplan.rs` opens with the two `//` SPDX lines. The planted-fault tree is built in a temporary directory at test time, not committed. If it is ever committed, each file carries SPDX lines (or a `REUSE.toml` annotation) and sits outside the document directories, so the `docs_contract` directory rules do not read it.

### ADVISORY 3: PR text and commits

`AGENTS.md:267-271` requires conventional commits and forbids agent credits and co-author trailers in commits, files and pull requests. The plan does not say what the commits or the PR body will look like.

REWRITE TO (Task 2):
> Commits use `test(...)` / `ci(...)` / `docs(...)` conventional subjects with no issue numbers in the subject. The PR body ("Part of #9") carries no agent attribution.

## Compliant items (checked, no finding)

* The new gate is a Rust test, and the hook change is thin glue (`AGENTS.md:205-207`).
* A planted-fault negative fixture is planned (`AGENTS.md:213-214`).
* No new dependency, no `#![feature]`, no lint change and no census fingerprint change.
* `validation.md` and the plan live under `.stage/`, which is ignored (`.gitignore:5`), so the issue numbers in them are not committed.
* No `.deficiencies` entry is planned or present.
* No release, tag or publish step.

VERDICT: NEEDS-REVISION
