# Plan constraints review (third recheck: evidence record runs the caller's command)

Reviewed plan: `.stage/m2-rest-of-the-planning-essay-entity/plan.md`, Revision 3 as saved after the A13/A14
edits and the owner's "Run the command (Recommended)" decision. I re-read the baseline paragraph (line 18-38),
D4, D5, D9, D12, the edit list (line 152-165) and the R2 and R4 rows.

## Governing documents

Read:

* `AGENTS.md` (canonical contract), `CLAUDE.md`
* `SECURITY.md` (trust model at line 35-44, Subprocesses paragraph at line 138-160)
* `deny.toml`, `REUSE.toml`, `.gitignore`, `.deficiencies` (notice and marker only, no entries)
* `Makefile` (the `check` gate list), `.githooks/pre-commit`
* `docs/architecture/okf-profile.md`
* The five capability documents and the change records for prose assessment and two-way coverage
* `tests/docs_contract.rs` (`INVENTORY`), `crates/raf-workplan/src/refs.rs`,
  `crates/raf-workplan/src/exec/invocation.rs` and `exec/pattern.rs`
* `.stage/.../issue-analysis.md`

Absent, recorded: `.goals`, `.baseline`, `CONSTITUTION.md`, ADR directories, `PRECOMMIT_GUIDANCE.md`,
nested `CLAUDE.md`/`AGENTS.md`. None exists in the worktree or the main checkout; `.goals` and `.baseline`
are listed in `.gitignore`.

User decisions on the record:

* **Coverage scope, "Workplan crates only (Recommended)".** Applied at line 28-34. **Compliant.**
* **Hosted CI counts as cross-check evidence.** Applied at line 35-36 and in Task 12. This decision overrides
  `AGENTS.md:148-153` and `CLAUDE.md` "Gate before you claim done: `make check`" for `cross-check` only. It is
  reported as a decision. **Compliant.**
* **Evidence record runs the command, "Run the command (Recommended)".** Applied in D4. This is a user
  decision; it binds the plan and is reported here as a decision, not cleared silently. It matches the
  capability's own PROPOSED text (`workplan-evidence-claims.md:58-59`: "`cargo workplan evidence record --id
  memory-bound -- cargo bench parse` runs the command"). It sits inside the trust model of `SECURITY.md:37-44`:
  "The checked workspace is **trusted build input** ... Running RAF on a checkout therefore carries the same
  trust as building that checkout. RAF is not a sandbox". **Compliant.**

## The `Program::Trusted` change against the written rules

| Rule | Governing text | Plan | Result |
| --- | --- | --- | --- |
| No shell, argv vector | `SECURITY.md:138-139`: "runs with no shell and takes its arguments as an argv vector" | D4: "executed with no shell" from the argv after `--` | Compliant |
| Input closed, output bounded, wall-clock limit | `SECURITY.md:156-157`: "a process runs with its input closed, its output bounded and a wall-clock limit" | D4: "stdin null, the 1800 s build bound, bounded output (16 MiB per stream)"; R2: "stub proves argv, stdin closed, bounded output, timeout kill" | Compliant |
| Document text never runs | `SECURITY.md:139-143` | R2: "the shared `exec` trusted variant is never reachable from document text" | Compliant. See A15 for the baseline wording. |
| Binary starts no process of its own | `SECURITY.md:143-144` | `Program::Trusted` lives in `raf-workplan` `exec` | Compliant, provided SECURITY.md keeps saying so (A16) |
| Evidence is a result, never prose | `workplan-evidence-claims.md:82-83` | D4: "other output is discarded and never persisted" | Compliant |
| Rust first, no new dependency, no `#![feature]`, no lint change | `AGENTS.md:202-204`, `deny.toml`, `AGENTS.md:99`, `AGENTS.md:164-181` | Uses the `exec` module | Compliant |
| Every refusal has a negative fixture and a passing neighbour | `AGENTS.md:210-214` | R2: a failing command records `fail`; `--from-binding` with a path matching no binding exits 2 with nothing started; a resolved one runs | Compliant |

No standing rule is broken by running the caller's command. Four text gaps remain, A15 to A18 below. Each is
cheap to fix.

## Status of earlier findings

| ID | Status | Evidence |
| --- | --- | --- |
| B1 | DISCHARGED | D5: staleness is digest-based. The stored revision and command text are informational. |
| B2 | DISCHARGED | Line 31-33: the `coverage-record-missing` finding; `scope: []` is the visible opt-out. |
| A1-A12 | DISCHARGED | As in the previous recheck; Revision 3 text unchanged on these points. |
| A13 residual text sync | DISCHARGED | Line 23 names the `-S` argument. Edit list line 161 covers `coverage-record-missing`, `scope: []` and the closed-kind test. Note for the integration reviewer: R4 still says "`doc(hidden)` and `cfg(test)` out", while D9 and line 161 say "`cfg(test)` tests count". |
| A14 `--from-binding` | DISCHARGED | D4: "a path matching no such binding exits 2 and nothing starts". R2 carries the fixture. D12: "SECURITY.md lists the new commands". |

## Remaining advisories

### A15 (ADVISORY). The baseline sentence calls the trusted argv a form for document text

`SECURITY.md:139-141`: "A value taken from a document reaches a process only after resolution ... and a
stated pattern, and then in one of three forms".

Plan line 22-23: "document text reaches a process only after resolution and a stated pattern, in the forms
SECURITY.md states (the three existing ones, the `-S<identifier>` argument of D18 and the trusted argv of D4)".

Read as written, document text could reach `Program::Trusted`. R2 says it never can, which is the required
property, but the governing sentence that SECURITY.md is edited from must not say the opposite.

REWRITE TO: "document text reaches a process only after resolution and a stated pattern, in the three forms
SECURITY.md states or the `-S<identifier>` argument of D18. The trusted argv of D4 is taken from the command
line after `--` only, never from a document or a record; the stored `command` field is never run."

### A16 (ADVISORY). D12 contradicts D4, and the SECURITY.md edit needs exact sentences

`SECURITY.md:138-139`: "Every spawned `cargo` or `git` process runs with no shell ..., and `cargo` runs with
`--frozen`."
`SECURITY.md:148-149`: "The change commands add two kinds of process".
`SECURITY.md:162`: "Each spec states its own bounds and exact invocations."
`SECURITY.md:189-191` lists "unintended inheritance or leakage of the environment" among the surfaces.

Plan D12, line 112-113: "The tool starts no process other than `git` (and the already-built `cargo` rows) for
these capabilities." D4 adds `Program::Trusted`, so D12 is now false.

The caller's argv may itself be `cargo` (as in the Usage example `-- cargo bench parse`), and it runs without
an added `--frozen`. Neither the plan nor the edit list states how the environment is handled.

REWRITE TO:

* D12: "The tool starts `git` rows, the already-built `cargo` rows, and one trusted process: the caller's
  argv for `evidence record` (D4), started by `raf-workplan`."
* Task 5 SECURITY.md edit: "`evidence record` runs the argv given after `--` exactly as given (a `cargo`
  there receives no added `--frozen`). It runs with no shell, stdin closed, 16 MiB per stream, a 1800 s
  limit, the workspace as working directory and the caller's environment (state the policy: inherited, or
  inherited with `LC_ALL=C`). The `cargo-workplan` binary still starts no process of its own."
* Edit list: add a row for the evidence-claims Contract stating the same argv, limits and environment, so the
  Contract states its own bounds and exact invocation.

### A17 (ADVISORY). The value and limit parse must fail hard

`AGENTS.md:205-207`: "Fail hard. ... No silent fallbacks, no best-effort parsing, no skipped checks."
`SECURITY.md:48-50`: "a failure never passes: a run ends in an error, a correct result or an abort, and never
in success with a wrong or partial result."

Plan D4, line 63 and 67-68: the verdict is "exit status 0 = pass, otherwise fail, or fail when a printed
value exceeds its limit"; "Values are the stdout lines of the form `name=number`; other output is discarded".
Line 66: "bounded output (16 MiB per stream)".

The plan does not say what happens in three cases, and each would read as `pass` by default:

* a `--limit` name that the command never prints;
* a `name=` line whose value is not a number, or a name printed twice;
* output past the 16 MiB bound, which could be truncated and the leftover lines parsed.

REWRITE TO (D4 and R2): "A `--limit` whose name the command does not print records `fail`, naming the
missing value. A `name=` line whose value is not a number, or a duplicate name, is an error and writes no
record. Output past 16 MiB on either stream is an error that says so and writes no record; it is never
truncated and parsed. R2 adds one fixture for each case, plus the passing neighbour: every limited value
printed and within its limit records `pass`."

### A18 (ADVISORY). The stored command text must never be run

Plan D5, line 76-78: "The record keeps the command text and the optional `revision` ... as informational
fields; neither is used for decisions, and neither reaches a process."

That sentence is right, and it must hold as D4 adds a runner. Nothing may re-run the stored text: no
re-record, refresh or `reconcile` path built from a record's `command`. Such a path would send document text
to `Program::Trusted`, against `SECURITY.md:139-143`.

REWRITE TO (D5 or R2): "No code path builds a `Program::Trusted` invocation from a record's `command` field.
Re-recording requires the argv on the command line again. R2: `Program::Trusted` has no constructor that
takes a loaded document or record (a compile-fail doctest, as `exec/invocation.rs:131-138` does for
`Literal::new`)."

## Checked and compliant

* Writes stay exclusive-create or stage-then-rename (D13), and every new record carries SPDX lines.
* D18 keeps tracker, mode and future-work wording out of committed text. `.stage/` is ignored by git.
* `.deficiencies` has no entries, and the plan adds none.

No BLOCKING finding remains. A15 to A18 are cheap text and fixture additions and should be applied before
Task 5 starts. A16 and A18 bound the new trusted process, so apply them before any `exec` code is written.

VERDICT: PASS
