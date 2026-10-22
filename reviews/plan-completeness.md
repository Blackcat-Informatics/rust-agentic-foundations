# Plan completeness review

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/16-raf-workplan-strict-mode-force-coverage/.stage/raf-workplan-strict-mode-force-coverage/plan.md` (Rev 0), with its annex `design.md` (D§n), against `issue.md` (body, 0 comments), `issue-analysis.md` (R1-R21, 7 NEEDS-JUDGMENT), `prior-art-assessment.md` (4 NEEDS-JUDGMENT) and the worktree at `95c384c`.

Source spot-checks:
- `crates/raf-workplan/src/lifecycle/complete.rs:329-335`: condition 5 runs every `Test | Example` binding of `impact.bindings`. This confirms the plan's claim that "verify ALL bound items" already holds.
- `crates/raf-workplan/src/obligation.rs:719-730`: `suppressed(kind, section, effect)` suppresses `section-untouched` for Contract, Usage and Examples. This confirms R8's starting point.
- `crates/raf-okf/tests/schema.rs:210`: pins the type list `"Capability, Change, Architecture, Evidence, Assessment or Coverage"`. This confirms that a `Debt` type changes an existing fixture.
- `crates/cargo-workplan/src/cli.rs:290-295`: `Coverage` has only `Code` and `Tests`, and there is no `--reverse`.
- `crates/raf-workplan/src/finding.rs:473`: holds the `okf-item-unknown-key` fix text. No test pins it.
- `crates/cargo-workplan/tests/check.rs`, `crates/raf-workplan/tests/{kinds,complete,scenarios}.rs`: exist.

Ledger coverage:
- All R1-R21 appear in the contract.
- I found no requirement in the issue that the ledger missed.
- All 7 ledger NEEDS-JUDGMENT items have a disposition (D§5 and the plan's Decisions).
- All 4 prior-art NEEDS-JUDGMENT items are disposed of (D§5, closing paragraph).

## Requirement -> task -> check

| R | Requirement | Task | Check (plan / D§6.1) | Status |
|---|---|---|---|---|
| R1 | `strict` setting: library, CLI, repo-level location justified in the capability | T2, T8 | Record vs floor precedence, `Opened::workplan_after` keeps the floor, `okf-bad-strictness` vs `strict`/`default`, `--strict` in `cargo-workplan/tests/check.rs`; justification as a `### Decision` in `cap:workplan-strict` | OK |
| R2 | Delivered capability binds a runnable example/doc test and has a matrix block | T2 | `strict-example-missing` vs one example; `strict-matrix-missing` vs the block; runnability proven by the executor (condition 5 / `verify`) | OK |
| R3 | Claim coverage, claim unit defined, matrix-row mapping, unsupported claims as findings with fix text | T3 | Marker, unit and coverage fixtures; matrix lists Contract for a marked test; existing matrices byte-identical; `coverage claims` text = JSON, sorted, two runs identical; CLI 0/1/2 | OK |
| R4 | Every public item of a bound crate path explained (orphan symbol), reusing two-way coverage | T2 | Bound package outside `scope` with an unbound public item vs bound; unbound package not enforced | OK |
| R5 | `change complete`: verify ALL bound items, zero tense-mismatch, forbids `contract_effect: none` when a bound public item is touched | T4 | Git fixtures: signature edit refuses under strict; **body-only and doc-comment edits complete under strict**; parts 1-2 "already hold", with no strict-mode fixture | **BLOCKING** (F2), ADVISORY (A2) |
| R6 | Minimum content: non-trivial length, runnable Usage example, Limitations | T2 | `strict-section-thin` at 19 vs 20 words; `strict-usage-unbound` vs a bound doc test's code span; `strict-limitations-missing` vs one list item | OK |
| R7 | No `dropped`/`moved` without a reason field | T4 | `outcome-without-reason` refuses under strict, completes with `reason`, completes without it under default | OK (A4: `okf-item-reason-empty` neighbour) |
| R8 | `section-untouched` also for Usage/Examples under `none` | T4 | Untouched Usage under `none` refuses under strict, completes under default | OK (A4: Examples) |
| R9 | Baseline/ratchet coordinated with the census ratchet, no second ledger | T7 | Census fixture set mapped one to one; `debt --write` exit 0/1/2; `arch:debt-ratchet` linked by both | OK |
| R10 | Reverse coverage report per crate/module/item, counts, orphan items and ranges, sorted, **text table and JSON** | T5, T6 | "the R12 fixtures; report identical across runs": no check of the per-module rows, the per-item ranges, or the JSON output | **BLOCKING** (F3) |
| R10 | Named entry point `cargo workplan coverage --reverse` | T6 | Replaced by `coverage lines`; `--reverse` listed under Declined with no quoted user decision | **BLOCKING** (F1) |
| R11 | Ceiling (lines or percent), exceptions with reasons, baseline so the count only goes down | T6, T7 | Ceiling boundary N/N+1 and percent boundary; exceptions by item/prefix/file/directory, each beside an unmatched one; debt line rows (rising fails, falling fails until write) | OK |
| R12 | Fixtures: fully bound = 0, private helper = exact range, nested/impl coverage, macro and `#[cfg(test)]` **handled and documented**, identical across runs/platforms | T5, T6, T8 | Fixtures named in D§3.9 (CRLF = LF, two runs byte-identical); the *documented* half has no task or check | **BLOCKING** (F4) |
| R13 | Each strict rule: triggering fixture + passing neighbour | T2-T7 | Per-task checks; `Case` with the strict-clean base, `assert_exact` | OK (A4) |
| R14 | Default behaviour and every existing fixture unchanged | all | `all_cases()` as today; strict fixtures silent without strict; **the `schema.rs:210` pin is edited, justified by an unquoted "user decision in this session"** | **BLOCKING** (F5) |
| R15 | `Finding` shape; new kinds in closed `KINDS`; both-directions test | T2-T7 | `OWN_KINDS` 48 -> 60, each kind a named `Case`; obligation kinds in the `complete.rs` both-ways tests; JSON non-rule kinds in `json.rs:263` | OK |
| R16 | CLI tests for `--strict` exit codes 0, 1, 2 | T2 (+T3, T4, T6, T7) | `cargo-workplan/tests/check.rs`: `--strict check` 0 on strict-clean, 1 on a thin section (0 without), 2 on a root with no `docs/capabilities` | OK |
| R17 | New designed capability `cap:workplan-strict` per the AGENTS.md workflow | T1, T8 | Change record, `designed` at T1 with PROPOSED blocks, `delivered` at T8 through `change complete` | OK |
| R18 | Once the rules pass, the repository enables strict on its own docs | T8 | `strictness: strict` set; `cargo workplan --strict check` exits 0 with recorded debt; `self_workplan` judges strict | OK (A1: debt reading) |
| R19 | Reconcile with the delivered two-way coverage/evidence work | T1 | "D§4 recorded" (D§4 lives in git-ignored `.stage/`) | ADVISORY (A3) |
| R20 | Builds on the lifecycle and CLI | - | Delivered and closed | OK |
| R21 | Opt-in; default stays lightweight | T2 | Default fixtures unchanged; strict computed lazily in `check` only | OK |
| - | Plan structure | - | No `## Task N` sections; PR task is unnumbered; the annex's per-task gate (`make check` green) contradicts the plan | **BLOCKING** (F6) |

## Findings

### F1 BLOCKING: SILENT CUT of the named entry point `cargo workplan coverage --reverse` (R10)
The issue names the shipped entry point verbatim: "Reverse coverage mode (`cargo workplan coverage --reverse`, also enforced under strict with a threshold)". The plan drops it under Declined with a design argument and no quoted user decision:

> "a `coverage --reverse` flag spelling (the command shape is `coverage code|tests`)"

After this plan, a user who runs the command the issue specifies gets a usage error.

REWRITE TO (pick one, in the Declined section and the R10 row):
- (a) "R10 entry point: `cargo workplan coverage --reverse [--crate <package>]` is accepted and prints exactly what `cargo workplan coverage lines` prints. T6 check: `cargo-workplan/tests/coverage.rs` runs both spellings on the same fixture and asserts byte-identical stdout and equal exit codes (0 with no orphan, 1 with one, 2 for an unknown `--crate`)." or
- (b) "Declined: the `coverage --reverse` spelling, replaced by `coverage lines`. User decision: \"<verbatim quote of the user's message authorizing the rename>\"." If no such quote exists, ask the user before Stage 2.

### F2 BLOCKING: R5 narrowed so that "touch" excludes bodies and doc comments
The issue says: "forbids `contract_effect: none` for changes that **touch** a bound public item". The plan redefines touch as "alters a bound public item's declaration, bodies and doc comments excluded". Its own check asserts the opposite of the literal requirement:

> "body/doc edit completes" (contract row R5); D§6.1 T4: "a body-only edit and a doc-comment edit complete under strict"

Editing a bound public function's body touches that item, and it can change the behaviour the Contract states. Counting such a change as delivered is a narrowing that no user decision supports.

REWRITE TO (pick one):
- (a) "`contract-effect-understated`: a `contract_effect: none` change under strict that alters the span text of a bound public item (attributes, signature and body; doc comments included) relative to the merge base. T4 check: a signature edit, a body-only edit and a doc-comment edit each refuse under strict and complete under default; a whitespace-only reformat [refuses/completes: state which]; an item moved within its file completes; a new bound public item refuses." or
- (b) Keep the declaration-only reading and add: "User decision: \"<verbatim quote authorizing that body and doc edits do not count as touching>\"", and state the narrowing in the Constraints of `cap:workplan-strict`.

### F3 BLOCKING: UNFALSIFIABLE check for the R10 report shape (per module, per item, JSON)
R10 requires a report "per crate, per module and per item with counts and the list of orphan items and line ranges, sorted for stable output, as a text table and as JSON". The contract row checks only:

> "R10 | ... | 5, 6 | the R12 fixtures; report identical across runs"

The D§6.1 T6 check names no JSON assertion and no module-table assertion. Unlike T3, it also has no "table and JSON equal" assertion.

REWRITE TO (R10 row and T6 check): "On the R12 fixture workspace, `cargo workplan coverage lines` prints the crate table, the module table and the orphan-item table, with exact expected rows (counted/covered/excepted/orphan/shipping/test counts; item path, kind, file, merged `a-b` ranges), sorted by package, module, file and first line. `cargo workplan --format json coverage lines` emits `lines-crate`, `lines-module`, `lines-orphan` (with `ranges` as `[[a,b],...]`) and `lines-excepted` objects whose values equal the text rows one to one, in the same order. Both outputs are byte-identical across two runs and across the LF and CRLF copies of the fixture."

### F4 BLOCKING: R12 "macro-generated items and `#[cfg(test)]` modules are handled and **documented**": the documented half is unplanned
D§3.9 defines the handling (rule 4 for macro invocations, test units with separate shipping and test columns). The capability sketch's Constraints list in D§6.2 does not include it:

> "Constraints (reachability by path only; a claim's test is cited, not judged; a moved item reads as touched; Constraints items reworded are new claims; the 20-word constant; the ceiling default 0; strict cannot be lowered by a caller)"

No task or check delivers the documentation.

REWRITE TO (D§6.2 Constraints and the T6/T8 check): "Constraints also state: a macro-generated item has no lines of its own and is counted on its invocation's lines, covered when its module is covered or a covered unit names the generated identifier; derives add no lines; `#[cfg(test)]` items are units like any other, reported in the test-orphan column, and are excepted by an exclusion's or exception's reason; cfg is never evaluated. Each statement carries a `raf:tested-by` marker naming the macro fixture and the `#[cfg(test)]` fixture of T6. Check: `cargo workplan coverage claims --capability cap:workplan-strict` lists both Constraints units as covered."

### F5 BLOCKING: R14 "every existing fixture unchanged" is met by reinterpretation, resting on an unquoted user decision
The plan reads: "'Every existing fixture unchanged' is read as unchanged behaviour: the pin is updated ... (Breaking changes before release are acceptable; the fallback of a JSON debt file is declined.)". In the standing goals it says: "breaking changes before the first release are acceptable (user decision in this session)".

`crates/raf-okf/tests/schema.rs:210` is an existing fixture, and the plan edits its expected string. A general decision that "breaking changes are acceptable" is not a decision on R14, and it is not quoted. D§7.3 itself says "Decision needed from the maintainer only on the type-list pin." A no-change alternative exists: the JSON fallback in D§3.8.

REWRITE TO (pick one):
- (a) "R14 / schema pin: User decision: \"<verbatim quote that permits editing the `schema.rs` type-list expectation, or that reads R14 as unchanged behaviour>\". T7 check: the only edited line of any existing test file is `crates/raf-okf/tests/schema.rs:210`, shown by `git diff --stat main -- '*/tests/*'` restricted to pre-existing files, and the PR body names it." or
- (b) "The debt record is `workplan-debt.json` at the workspace root (D§3.8 fallback); no existing fixture is edited. T7 check: `git diff main --name-only` lists no pre-existing file under `crates/*/tests/`."

### F6 BLOCKING: STRUCTURE (no per-task sections, unnumbered PR task, conflicting per-task gate)
- The plan has no `## Task 1` ... `## Task N` sections. It has one paragraph: "T1-T8 are those of D§6.1 ... with their focused checks". So the tasks, their validation and their commit units live only in the annex, which "the plan wins" over without stating them.
- The final task is not a task: "The PR task (after T8): final sequence above, PR \"Closes #16\", the plan posted, ...". It is not numbered and does not say it hands off.
- The two documents set contradictory gates. D§6.1 says "Every task ends with `make check` green", and the T8 check is "`make check`". The plan says `make check` "cannot pass here" and replaces it. A task's validation cannot be both.

REWRITE TO: replace the single `## Tasks` paragraph with `## Task 1` ... `## Task 9`. Each of Task 1-8 states:
- its deliverables (from D§6.1 T1-T8);
- its focused checks, copied from D§6.1 with `make check` replaced by "`cargo test --locked --test docs_contract --test self_workplan --test repo_gates`, `make fmt clippy doc deny reuse issue-refs`, `cargo workplan check`, plus the task's focused tests";
- "independent review by a fresh reviewer";
- its commit type and subject;
- "push";
- "issue comment summarising the task and its check output".

`## Task 9: Pull request and hand-off`:
- runs `make fmt clippy test test-release doc deny reuse msrv fuzz`, then `make licenses` and `make issue-refs`;
- opens the PR with "Closes #16";
- posts the plan;
- summarises the debt diff;
- discloses the cross-check limit and the schema pin (or F5's alternative);
- states the least-confident area;
- hands off.

The T8 check reads "the gate sequence of Task 9" in place of "`make check`".

### A1 ADVISORY: R18 is delivered with about 786 debt rows; the reading of "once the rules pass" should go to the maintainer
The plan's reading is reasonable under R9: "adopting strict on an existing repository can start from today's findings". D§5.7 notes that the maintainer may read R18 as "with an empty debt record". The plan declares the reading but does not ask the question.

REWRITE TO (Task 9): "The PR body quotes R9 and R18, states that strict is enabled with `work/debt/default.md` holding N finding rows and M line rows (exact counts from T7), and asks the maintainer to confirm that a recorded baseline satisfies 'once the rules pass'."

### A2 ADVISORY: R5 parts 1-2 ("verify ALL bound items", "zero tense-mismatch") have no strict-mode check
The plan relies on the existing default-mode fixtures, and the source confirms they hold (`complete.rs:329-335`). Strict changes the path through `Opened` and the floor, though, so nothing shows the conditions still refuse under strict.

REWRITE TO (T4 check, add): "Under `Lifecycle::open_with(&gate.strict(), ..)` and under `cargo workplan --strict change complete`: a failing bound test of a listed capability refuses condition 5, and a tense-mismatch phrase in a listed `implementing` capability refuses condition 1; each repair completes."

### A3 ADVISORY: R19's check, "D§4 recorded", points at a git-ignored file
`design.md` is in `.stage/` and is not committed.

REWRITE TO (R19 row): "The Explanation of `cap:workplan-strict` states the reconciliation with `cap:workplan-two-way-coverage` and `cap:workplan-evidence-claims`: orphan symbols reuse the `coverage-*` kinds over bound packages; claim coverage is the docs-to-tests direction at claim grain; evidence markers do not count as test coverage. Check: `OWN_KINDS` gains no `coverage-item`/`coverage-test`/`claim-unsupported` duplicate, and `cap:workplan-two-way-coverage#contract` gains the PROPOSED sentence of D§4."

### A4 ADVISORY: some neighbour fixtures are implied only by the both-directions test
- T4 names no `okf-item-reason-empty` fixture.
- T6 names no `okf-bad-ceiling`, `okf-bad-exception` or `okf-empty-reason` fixtures.
- R8's check names Usage but not Examples.

The both-directions test forces a triggering case for each kind, so these are not gaps in delivery. Naming them makes R13 auditable.

REWRITE TO:
- T4 check, add: "`okf-item-reason-empty` for `reason: \"\"` beside a non-empty `reason`; an untouched Examples in `affects` of a `none` change refuses under strict and completes under default."
- T6 check, add: "`okf-bad-ceiling` (both `lines` and `percent`), `okf-bad-exception` (both `item` and `file`; an absolute path; a `..` path; a crate root), each beside a well-formed row."

VERDICT: NEEDS-REVISION
