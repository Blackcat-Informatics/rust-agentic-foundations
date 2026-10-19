# Plan constraints review

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/29-index-the-coverage-product-instead-of/.stage/index-the-coverage-product-instead-of/plan.md` (Rev 0)

## Governing documents

Read:
- `CLAUDE.md` (worktree root)
- `AGENTS.md` (worktree root, canonical contract)
- `CONTRIBUTING.md`
- `clippy.toml`, `deny.toml`, `.gitignore`, `crates/raf-workplan/Cargo.toml` (to check the rules the plan's tasks fall under)
- `crates/raf-workplan/src/refs.rs` (the issue-reference and process-wording pattern set)
- `docs/capabilities/workplan-two-way-coverage.md` (the capability the plan edits)
- `$STAGE/issue-analysis.md`

Absent (recorded): `.baseline`, `.goals`, `CONSTITUTION.md`, `PRECOMMIT_GUIDANCE.md`, `docs/adr/`, `docs/decisions/`, and nested `AGENTS.md`/`CLAUDE.md` under `crates/raf-workplan/` or `docs/`. Because there is no `.goals` and no CONSTITUTION, the standing constraints are the repository rules in CLAUDE.md, AGENTS.md and CONTRIBUTING.md, plus the user decisions the caller passed on: no timing runs (counts only); Linux is the primary platform; no issue or PR numbers and no process wording in committed files.

## Findings

### B1 (BLOCKING): the plan leaves `make cross-check` out of the gate it runs before handoff

Plan line 15: "`make check` before done (cross-check via hosted CI where targets are absent)"
Plan line 48 (Task 2): "`make check` minus cross-check locally (hosted CI covers it)"

Governing text:
- CLAUDE.md:9-10: "**Gate before you claim done**: `make check`. Install the hooks with `make hooks` and never bypass a hook or a gate."
- AGENTS.md:140: "**Run `make check` before claiming work is done.** It stops at the first failure."
- AGENTS.md:175-180: "A missing tool is a hard failure, not a skip. ... `make cross-check` needs the `x86_64-apple-darwin`, `x86_64-unknown-freebsd` and `x86_64-pc-windows-gnu` targets (`rustup target add ...`)."
- CONTRIBUTING.md:80: "install them with `rustup target add`, because a missing target is a failure."

The plan does not run one part of the gate and hands that part to CI. The repository says outright that a missing target is a failure, not a skip. The Linux-primary rule (CLAUDE.md, "Platforms": "a platform-specific failure does not block a change") is about how to treat a platform-specific failure that `make check` reports. It does not allow leaving out part of the gate, and no user decision quoted for this session allows it. The plan cites PR precedent, but precedent is not a user decision.

REWRITE TO (Task 2): "Install the cross-check targets (`rustup target add x86_64-apple-darwin x86_64-unknown-freebsd x86_64-pc-windows-gnu`) and run the full `make check`. If cross-check reports a platform-specific failure, record it as non-blocking under the Platforms rule in CLAUDE.md and name it in the handoff. If the targets cannot be installed, report `make check` as failing on the missing tool and do not claim the work is done." Line 15 must also drop "(cross-check via hosted CI where targets are absent)".

### A1 (ADVISORY): renaming the bound tests needs `make docs-sync` for the generated bindings table

Plan line 44: "update `docs/capabilities/workplan-two-way-coverage.md` (Verification text and bindings ...)"

Governing text, AGENTS.md:137-139: "**`make docs-sync` rewrites the generated documentation blocks** ... it is the repair for the stale-block finding of `Workplan::check`." The capability binds the product tests in its frontmatter (lines 38-41) and again in the generated table (lines 289-292). If the frontmatter is edited and the table is not regenerated, the `docs_contract` gate and the pre-commit hook fail.

REWRITE TO: "update the frontmatter bindings and the Verification text, then run `make docs-sync` to regenerate the bindings table, and commit both."

### A2 (ADVISORY): the changelog step is conditional on a fact that is already known

Plan line 45-46: "and the changelog if the repository keeps one for raf-workplan."

The repository keeps a root `CHANGELOG.md` with an `## [Unreleased]` section of hand-written entries (CHANGELOG.md:9). `Work` is `pub(crate)` (crates/raf-workplan/src/work.rs:20), so the public API does not change and no semver rule applies (CONTRIBUTING.md:104-106). No rule requires an entry for an internal change. Leaving the step conditional only means the executor has to decide it later.

REWRITE TO: "Add an `[Unreleased]` entry in the root `CHANGELOG.md` under `raf-workplan`: coverage judging uses an index, so its work grows with entries plus exclusions, and the report is unchanged." Alternatively, say plainly that no entry is written because the change is internal. Choose one.

### A3 (ADVISORY): the seeded random differential test must not add an unvetted dependency

Plan line 36: "a differential test: the indexed result equals a reference product implementation on seeded random exclusions/paths"

`crates/raf-workplan/Cargo.toml` has no random-number dev-dependency. Any new crate is subject to the `deny.toml` licence allowlist (deny.toml:12, `allow = ["MIT", "Apache-2.0", "MulanPSL-2.0"]`) and to `make deny` (AGENTS.md:133, 187). The plan does not say where the randomness comes from.

REWRITE TO: "a differential test over exclusions and paths from a small deterministic generator written in the test module (a fixed-seed linear congruential or xorshift step), with no new dependency."

### A4 (ADVISORY): the rewritten committed prose must state the current contract only, with no history or process wording

Plan line 44: "the product statement becomes the entries-plus-exclusions statement"

Governing text:
- AGENTS.md:255-256: "Outside a proposal, the text states the current contract only."
- AGENTS.md:280-285: "Do not write issue or pull request numbers, tracker links, or wording that refers to the development process that produced a file ... Documents describe the system, not its history."

Two committed texts describe the product today: the Verification paragraph of the capability (workplan-two-way-coverage.md:256-261) and the module doc comment of `crates/raf-workplan/src/coverage/tests.rs:4-8` ("each exclusion of the record is tested against each of them ... every exclusion must be tested against every entry"). The plan names only the capability. Both must be rewritten. Neither may say "no longer", "replaces the pin", "previously" or cite the earlier change, and the test-module reference implementation needs a doc comment that describes it as the reference, not as the old code.

REWRITE TO (Task 1): "rewrite the module doc of `coverage/tests.rs` and the Verification paragraph of the capability so they state the indexed contract as it now stands (work grows with entries plus exclusions; every matching exclusion is marked; the first in record order gives the reason), with no wording about what it replaced."

### A5 (ADVISORY): the commit type is not stated

Plan line 46: "Independent review, commit, push."

Governing text, AGENTS.md:289-291: "**Conventional commits.** `type(scope): summary` ... The subjects feed `git-cliff`, so they become the release notes."

REWRITE TO: "commit as `perf(raf-workplan): ...` (with the doc and test changes in the same commit, or separate `docs(workplan-two-way-coverage): ...` and `test(raf-workplan): ...` commits)."

## Checked and compliant

- No timing runs (CLAUDE.md "No timing runs"; AGENTS.md:151-172): every check counts `Work` kinds and uses the repository's `one > 0, three + one <= 2 * two` idiom. Nothing reads a clock, sleeps or benchmarks. Plan line 13 records the duty to pass the rule to every agent started. That duty covers the independent reviewer and the mutant runs in Task 1.
- Public API and semver: `Work` is `pub(crate)` (work.rs:20), so replacing `ExclusionTests` with `IndexLookups` and `ExclusionsIndexed` changes no public item. `HashMap` is not in the `clippy.toml` disallowed lists. No `#![feature]`, no lint weakened and no `#[allow]` is planned.
- Fail hard (AGENTS.md:230-232): the index adds no fallback, and an unmatched exclusion is still reported.
- Issue and PR numbers (CLAUDE.md "Never"; AGENTS.md:280-285): the plan's own references to issues 21 and 29 are in `.stage/`, which `.gitignore:5` ignores, so they are never committed. "Closes #29" goes in the PR body, which is not a committed file. AGENTS.md:292-293 still bars agent attribution in the PR.
- Linux primary: the plan adds no platform-specific code. The point about cross-check is B1.
- No deferrals (AGENTS.md:286-288): the "Declined" items are rejected design alternatives, not scope pushed to later.
- `.deficiencies`: the plan does not touch it.
- Capability workflow: the capability stays `delivered`, so the `INVENTORY` table needs no edit (AGENTS.md:256-258). The plan re-assesses stale assessments as the self-workplan gate requires (AGENTS.md:43-45).
- SPDX: no new file is planned. Any new file must carry the headers of AGENTS.md:212-218.

Out-of-lane note for the completeness reviewers (not a constraints finding): the plan says "the three product tests", but four tests read `exclusion_tests` (tests.rs:131, 150, 163, 182) and the capability binds all four (lines 38-41), including `without_exclusions_the_entries_are_judged_and_nothing_is_tested`.

VERDICT: NEEDS-REVISION
