VERDICT: BLOCKED

# T6 review: commit ff8b258 (reverse coverage, `coverage lines` / `--reverse`, ceilings, exceptions)

The behaviour is right. Four things still block it. The commit does not build its own test target
under the repository's `-D warnings`, so `make clippy` fails. `self_workplan` fails on 12 stale
assessments. Two documented outputs (the excepted-lines table and the error table, and the JSON
objects `lines-excepted` and `lines-error`) are pinned by no test. The fuzz-workspace Constraint
says the opposite of what the plan decided, and nothing pins it.

I probed the engine and the CLI by hand and found no defect in the counting, the ceilings or the
exceptions.

## Scope reviewed

* `git show ff8b258` in full: 27 files.
* `crates/raf-workplan/src/coverage/lines.rs`, read end to end (1880 lines).
* `crates/raf-workplan/src/coverage/lines/tests.rs`, `render.rs`, `coverage.rs`, `graph.rs`,
  `strict.rs`, `work.rs` and `finding.rs`.
* `crates/cargo-workplan/src/{cli,command,coverage,main}.rs`.
* Tests: `raf-workplan/tests/lines.rs`, `cargo-workplan/tests/lines.rs`, and the `kinds.rs`,
  `complete.rs` and `support/cases.rs` edits.
* Docs: `workplan-strict.md`, `workplan-cli.md`, `workplan-document-model.md`,
  `workplan-two-way-coverage.md`, CHANGELOG, README, AGENTS.md.
* Plan: Rev 1, Rev 2 (B3, B4, A2, A4, A6), the Addendum, Rev 3, and design section 3.9.

File size: the repository has no file-size or complexity rule. `lines.rs` is 1880 lines, and
`graph.rs` (2452) and `raf-source/src/settle.rs` (3068) are larger. Its tests are in a separate
file. `clippy -D warnings` is clean once R1 is fixed.

I changed nothing in the worktree. I worked in the clone `scratch/t6review` at ff8b258. Builds went
to the cargo wrapper's target directory. Logs are in `scratch/t6-logs/`. I ran no timing runs and
no mutants. The two scratch probe files I wrote were deleted afterwards.

## Commands and exit status

| Command | Result |
| --- | --- |
| `cargo test -p raf-workplan -p cargo-workplan --locked` at ff8b258 | exit 101: `tests/lines.rs` does not compile (R1) |
| same, with the import deleted in the clone only | exit 0. raf-workplan: 287 unit tests, 14 `lines`, 4 `kinds`, 64 `complete`, 36 doc tests, all pass. cargo-workplan: 3 `lines` and all other binaries pass |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | exit 101: `docs_contract` 31 passed, `repo_gates` 68 passed, `self_workplan` FAILED (R2) |
| `make fmt clippy doc` at ff8b258 | exit 2: clippy, `unused import: CrateLines`, `tests/lines.rs:12:37` |
| `make fmt clippy doc` with the import deleted in the clone | exit 0 |
| `make issue-refs reuse` | exit 0 |
| `cargo workplan check` (built binary, clean clone tree) | exit 0 |
| `cargo workplan assess --list` in the worktree and in the clone | 12 stale, same in both |
| `cargo workplan coverage lines` on this repository | exit 1 (orphan lines printed). Rows: cargo-workplan, raf-okf, raf-source, raf-source-fuzz, raf-workplan, rust-agentic-foundations. Sorted, no absolute path |
| `--strict check` on this repository | exit 1, 662 lines. Expected, since adopting strict here is out of scope |
| Fixture probes through the binary (below) | as designed |

Fixture probes on the demo repository, through the binary:

* `check` with `strictness: strict`, `ceilings` of `lines: 0 of: shipping` and `crate: ghost`, an
  unmatched item exception and an unmatched directory exception, and an orphan helper plus a
  `#[cfg(test)]` module. It exits 1 and gives:
  * `scope-crate-missing` at `ceilings`;
  * two `coverage-exception-unmatched` findings;
  * a `coverage-lines-over-ceiling` for the shipping class, "its ceiling of shipping 0 lines";
  * a second one for the test class, "the default of 0 lines, since no row of `ceilings` bounds them".
  Text and JSON agree.
* A percent ceiling of 50 on 1 orphan line of 9 counted exits 0.
* Under `default`, a `lines: 0` ceiling exits 0. The same repository with `--strict` exits 1.
* `of: docs` gives `okf-bad-ceiling`.
* `coverage lines --crate demo` and `coverage --reverse` work.
* Library-level probes, each correct:
  * a file with no trailing newline;
  * a `tests/common/mod.rs` shared by two targets, counted once;
  * example and test targets reported as test lines;
  * a macro and an inline module inside a `#[cfg(test)]` module;
  * a trait with a default method and impls for it;
  * `const`, `static`, `type`, `union`, `const _` and an extern fn.

## Requirement checks that pass

* **Command and alias:** `coverage lines [--crate]` and `coverage --reverse` print the same bytes
  (tested in the CLI test and the parser test). Exit codes 0, 1 and 2 are tested, and `-q` hides an
  outcome but keeps a finding.
* **Percent without division:** `within()` compares `orphan*100*scale <= percent*scale*counted`
  with decimal-string arithmetic. Fractions of 80 digits are tested, and so are `usize::MAX`, the
  empty crate (0 of 0 under `percent: 0`) and both sides of the boundary.
* **Ceiling classes:** `of: shipping|test|all` is tested at N and N-1 for each class.
  A `shipping` row alone holds the test class to 0. No row holds both to 0. An `all` row bounds
  both. `okf-bad-ceiling` was observed through the binary.
* **Exceptions:** by item, prefix, file and directory, each beside an unmatched neighbour, and the
  first exception in record order gives the reason. `coverage-exception-unmatched` is raised under
  either strictness without counting a line. `scope-crate-missing` is raised from `ceilings`.
* **Determinism:** every map in the report path is a `BTreeMap`. The `HashMap`s are used only for
  lookup and memoisation. Rows are sorted by package, module, file and first line. CRLF equals LF,
  and two runs are byte-identical.
* **Default-mode cost (R21):** `under_default_check_counts_no_line_and_under_strict_it_sweeps_each_unit`
  pins 0 `LineUnits` under default and 2 under strict. The work-growth tests are counted work
  (linear in units plus exceptions), not timing.
* **New kinds:** `coverage-exception-unmatched` and `coverage-lines-over-ceiling` are in
  `Finding::KINDS` and `OWN_KINDS` (58), each with a named `Case`.
* **Docs:** CHANGELOG, README, AGENTS.md, workplan-cli, document-model, two-way-coverage and the
  Contract of workplan-strict agree with the code for everything I could compare. The okf-profile
  rows for `ceilings` and `exceptions` landed earlier and match. No issue or PR numbers, process
  wording or attribution in the diff. The new public items and the 3 + 14 + 7 new tests are bound
  in the capability documents (`docs_contract` and `repo_gates` pass).

## Findings

**R1. The test target does not compile under `-D warnings`.**
`crates/raf-workplan/tests/lines.rs:12` imports `CrateLines` and never uses it.
Consequence: `cargo test -p raf-workplan` fails with exit 101 (and `make clippy` with exit 2) at
this commit, so the gate sequence cannot have been green. Because the test target does not build,
the 14 `lines` tests cannot run in a normal check.
Required fix: `use raf_workplan::coverage::lines::{LinesReport, UnitLines};`. Rerun
`make fmt clippy doc` and the focused tests.

**R2. `self_workplan` fails: 12 assessments are stale, not current and accepted.**
`tests/self_workplan.rs:59` fails with "12 assessment(s) are stale or not accepted". The causes
are the T6 edits to the sections and symbols they cover.
* `cap:workplan-cli`: contract, usage, examples, constraints and verification. The sections and
  `cli::Args` / `command::run` changed.
* `cap:workplan-document-model#contract`: the section changed.
* `cap:workplan-two-way-coverage`: purpose, contract, usage, examples, constraints and
  verification. The section changed, and the `raf_workplan::coverage` symbol changed.

`assess --list` shows the same 12 in the worktree.
Consequence: the plan requires every task to keep every assessment current and accepted after
reading its context. This one does not, and the repository's own gate is red.
Required fix: read the context of each of the 12 (`assess <id> --context`), re-record each with a
verdict, accept them, and rerun `cargo test --locked --test self_workplan`.

**R3. The excepted-lines table, the error table and the JSON kinds `lines-excepted` and
`lines-error` are documented and pinned by no test.**
* They are described in `render.rs:36-47` and `render.rs:337-352` (table), and `render.rs:406`
  and `render.rs:415` (JSON).
* They are described in the `main.rs` crate docs and in `workplan-strict.md`, "In JSON the
  objects are of the kinds ... `lines-excepted` (with `exception` and `reason`) and
  `lines-error`".
* The only tests that touch them are `the_kinds_that_are_not_rules_are_not_rule_names`, which
  checks the names are not rule kinds, and library accessor tests.
* No test asserts the excepted header, the excepted row, the `| Error |` table or either JSON
  object. `the_report_renders_as_three_tables_and_as_json_with_the_same_rows` uses a report with
  neither.
* The cargo-workplan JSON member-order pins (`CRATE`, `MODULE`, `ORPHAN` in
  `cargo-workplan/tests/lines.rs`) omit both kinds.

Consequence: R10 requires text and JSON to each equal a literal string. Two rendered shapes that
the Contract promises can change unnoticed, for example the `{} {} | {} |` cell join in
`excepted_row`.
Required fix: add literal-string tests in `raf-workplan/tests/lines.rs`:
* a report with an exception: its text table and its `lines-excepted` JSON object;
* a report with a failed walk: its `| Error |` row and its `lines-error` object (the walk-failure
  fixture already exists in `lines/tests.rs`).

Add member-order pins for both kinds to the CLI JSON test, using a record with an exception.

**R4. The CLI tests never run a ceiling row, a class (`of:`) or an exception through the binary.**
The three tests in `cargo-workplan/tests/lines.rs` use a record with no `ceilings` or `exceptions`
key, so only the default of 0 is exercised. The library tests cover the rules, and I confirmed
the binary behaves correctly by hand (see the probes above). No committed test does.
Consequence: the plan puts the proof of ceilings (`of: shipping|test|all`), exceptions and
`coverage-exception-unmatched` on the shipped entry point. Wiring regressions such as the strict
floor, `Gate::open` or exit classification would pass the suite.
Required fix: add binary tests, run in both shapes with `both`:
* `--strict check` exits 1 with `coverage-lines-over-ceiling` for a record with an `of: shipping`
  row. The same repository with a raised row exits 0.
* `check` under `default` exits 1 with `coverage-exception-unmatched` for an unmatched exception
  and 0 for a matched neighbour.
* `coverage lines` with an exception prints the excepted table (this also serves R3).

**R5. The fuzz-workspace Constraint contradicts the plan, states a behaviour no fixture pins, and
the fixture the plan asked for was dropped.**
* Rev 2 B3 decided: the fuzz harness is a separate cargo workspace the root excludes, so it has no
  lines in the index and is not judged. The Constraint was to state that limit with a
  `raf:tested-by` marker naming a fixture "whose workspace excludes a directory of Rust files,
  which reports no row for it". The design's fixture list (section 3.9) says the same.
* The commit removed that Constraint and its Verification bullet. `workplan-strict.md:680` now
  says a directory holding a workspace of its own "is a package of the source index when the
  source roots reach it, so the line report judges its lines as those of any package".
* The binary agrees with the new text. This repository's report has a `raf-source-fuzz` row
  (37 counted, 2 orphan) and a `rust-agentic-foundations` root row.
* The new Constraint has no `raf:tested-by` marker and no fixture. The fixtures
  `a_package_that_no_capability_binds_is_reported_with_its_orphan_helper` and the others never
  load a nested `[workspace]`.

Consequence: the plan's B3 decision is silently reversed in the shipped text. Even if the
reversal is right (the shipped behaviour judges more, and strict then gives the fuzz package a
ceiling of 0), the claim is unpinned. It will also be an uncovered claim for the strict-clean
`cap:workplan-strict` that T8 must deliver.
Required fix: pick one and make the code, the plan note and the document agree.
* Keep the new behaviour: add a fixture with a nested `[workspace]` directory, once reached by the
  source roots (a row) and once not (no row), add a `raf:tested-by` marker for the Constraint, and
  record the reversal of B3 in the plan.
* Or restore the plan's behaviour.

## Notes (not findings, nothing to fix now)

* `CrateLines::debt()` and the Debt column are a constant 0 until the debt record exists. T7 must
  wire them, and the T7 review should check this.
* The over-ceiling message reads "has 1 orphan lines" for a count of 1. This is cosmetic.
* Rendering escapes `<` as `&lt;` inside item cells (for example ``demo::&lt;impl T for S>``), the
  same `table_code` escaping the other coverage reports use. It is consistent, and I did not
  count it.
