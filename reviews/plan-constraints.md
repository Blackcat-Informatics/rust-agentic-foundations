# Plan constraints review: raf-census core, normal form v1

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/3-raf-census-core-normal-form-v1/.stage/raf-census-core-normal-form-v1/plan.md`

## Governing documents

Read:
- `AGENTS.md` (canonical contract), `CLAUDE.md` (repository), `CONTRIBUTING.md` (AGENTS.md section 8 defers to it)
- `Cargo.toml` (workspace lints, members), `deny.toml`, `REUSE.toml`, `Makefile` (gate targets)
- `docs/architecture/normal-form.md` (`arch:normal-form`, the "decision" the issue cites), `docs/architecture/okf-profile.md`
- `docs/capabilities/census-normal-form.md`, `docs/capabilities/workplan-complete-gate.md` (completion condition 6), `docs/capabilities/workplan-current-rationale.md` (architecture-page constraints)
- `work/changes/census-normal-form.md`, `work/changes/census-ledger.md`
- `tests/docs_contract.rs` (INVENTORY, matrix rule, shape asserts), `tests/repo_gates.rs` (issue-refs, clock gate, lint inheritance, deny)
- `.deficiencies` (notice and marker only, no entries)
- Port source checked for the fail-hard rule: `/home/paudley/Active/purrdf/crates/helper-census/src/source.rs`, `normalize.rs`
- Dependency licences checked in the local registry: `blake3-1.8.5`, `blake3-1.8.7` and their dependencies

Absent (recorded): `.baseline`, `.goals`, `CONSTITUTION.md`, `PRECOMMIT_GUIDANCE.md`, `docs/adr/`, `docs/decisions/`, nested `AGENTS.md`/`CLAUDE.md` under `crates/`. No user decisions are recorded for this issue.

## Findings

### BLOCKING 1: the port carries purrdf's silent fallbacks into a fail-hard repository

Governing text:
- `AGENTS.md:230-232`: "**Fail hard.** Schemas are strict and closed: unknown keys are errors. A ledger or document that is stale, malformed or ambiguous is an error with a message that says how to fix it. No silent fallbacks, no best-effort parsing, no skipped checks."
- `CLAUDE.md` (Quick orientation, "Never"): "add a fallback where the design says fail hard"
- `CONTRIBUTING.md:30-31`: "Do not add silent fallbacks, best-effort parsing or skip-on-missing-tool behavior."

Plan lines:
- Prior art: "In scope: `normalize.rs` (...), the unit extraction and relex parts of `source.rs` (`relex`, `is_test_only`, parameter names)"
- Task 2: "Port from purrdf `normalize.rs` with its thresholds."

The port source falls back silently. `purrdf/crates/helper-census/src/source.rs:916-927` `relex` returns an empty `TokenStream` when the span slice is out of range or does not lex (`.and_then(|slice| TokenStream::from_str(slice).ok()).unwrap_or_default()`). Every body that fails would then share one empty-form fingerprint, which produces false groups. `is_test_only` (`source.rs:980-983`) returns `false` when its `cfg` arguments do not parse, so test-only code is silently counted as shipping. The plan says to port these functions and does not say how the port should handle a failure.

REWRITE TO (Prior art, appended to the in-scope sentence; Task 2 and Task 3 each get the matching check):
"Ported code keeps no silent fallback: a body whose span does not slice, or whose text does not re-lex, is an error from `Census::load` naming the file, line and symbol (never an empty stream); an unparsable `cfg` attribute is an error naming file and line (never 'not test-only'). Each refusal has a negative fixture beside a valid neighbour that loads."

### BLOCKING 2: the plan claims done and opens the PR without `make check`

Governing text:
- `CLAUDE.md` (Quick orientation): "**Gate before you claim done**: `make check`."
- `AGENTS.md:140`: "**Run `make check` before claiming work is done.** It stops at the first failure."
- `CONTRIBUTING.md:131`: "Run `make check` and make sure it is green." (under "Before you open a pull request")

Plan lines:
- Task 6: "Check: `cargo workplan check`, the three root test binaries, `make fmt clippy doc deny reuse issue-refs`."
- Task 7: "PR with `Closes #3` ... Stage 2/3 gates listed as unrun."

No task runs `make check`. The plan skips `make test` (the whole workspace), `test-release`, `msrv`, `fuzz` and `cross-check`. `make msrv` matters here. It runs `cargo check --workspace --all-targets --locked` on 1.98.1 (`Makefile:85`), so it is the only gate that proves the new member and its new `blake3` dependency build at the MSRV. `AGENTS.md:104-105` says "the MSRV build proves it on every change". Calling the change complete (`cargo workplan change complete`) and opening a PR that closes the issue are both claims that the work is done.

REWRITE TO (Task 6 check line):
"Check: `cargo workplan check`, then `make check` (fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, cross-check, licenses, issue-refs) green on the final tree before Task 7; a missing tool is a hard failure, not a skip."
In Task 7, change "Stage 2/3 gates listed as unrun" to say that only the review stages are unrun and that `make check` passed.

### ADVISORY 1: INVENTORY shape asserts and the six assessment records are not planned

Governing text:
- `tests/docs_contract.rs:62-63`: "Changing a state, adding a capability or adding a record edits this table in the same commit."
- `tests/docs_contract.rs:1144-1146,1160,1163`: `count("designed") == 9`, `count("implementing") == 0`, `count("delivered") == 12`, `records("assessment:") == 56`, `INVENTORY.records.len() == 58`.
- `docs/capabilities/workplan-complete-gate.md:271-274`: "Each authored section in `affects` has a current assessment with the verdict `accepted` ... the record `work/assessments/<capability slug>-<section>.md` exists"
- `AGENTS.md:256-258`: "changing a state or adding a capability edits that table in the same commit."

Plan lines: Task 1: "INVENTORY `census-normal-form` `implementing`". Task 6: "assessments re-read and accepted".

`change:census-normal-form` affects six sections (`work/changes/census-normal-form.md:10-15`), and none of them has an assessment yet, so nothing can be "re-read". The plan does not name the count edits. Task 1 changes designed 9→8 and implementing 0→1. Task 6 changes implementing 1→0 and delivered 12→13, adds six `assessment:census-normal-form-*` records (56→62, 58→64), and keeps the sorted order.

REWRITE TO:
- Task 1: "INVENTORY `census-normal-form` `implementing`, with `inventory_table_has_the_documented_shape` counts updated (designed 8, implementing 1)."
- Task 6: "`cargo workplan assess` records an accepted assessment for each of the six affected sections (purpose, contract, usage, examples, constraints, verification); the six `assessment:census-normal-form-*` ids are added, sorted, to `INVENTORY.records` and the counts updated (delivered 13, implementing 0, assessments 62, records 64) in the same commit as the state change."

### ADVISORY 2: the delivered capability needs the verification-matrix marker

Governing text:
- `tests/docs_contract.rs:334-351`: "A delivered capability of the inventory holds a generated verification matrix: its document has the marker line of the block." (finding: "a delivered capability holds a `verification-matrix` generated block")
- `docs/architecture/okf-profile.md:211`: `delivered`: "the capability binds at least one code path and one test"

Plan line: Task 6: "bindings (code, tests, examples), `make docs-sync`".

`make docs-sync` rewrites the generated blocks that already exist. The plan does not say to place the marker pair in Verification, and the capability has none today.

REWRITE TO (Task 6): "add `<!-- raf:generated:verification-matrix -->` / `<!-- raf:generated:end -->` under Verification, then `make docs-sync` fills it from the `tests` and `examples` bindings."

### ADVISORY 3: Task 1 changes a state but does not resync the catalogue

Governing text: `AGENTS.md:258-260`: "`make docs-sync` rewrites the generated blocks, including the catalogue in `docs/capabilities/index.md`." and `AGENTS.md:137-139`: "it is the repair for the stale-block finding of `Workplan::check`."

Plan line: Task 1 changes the state to `implementing`, but its check list does not include `make docs-sync`. The catalogue groups capabilities by state, so the Task 1 commit would leave a stale block that the pre-commit `docs_contract` would refuse.

REWRITE TO (Task 1): "... the `change:census-normal-form` item split; `make docs-sync`. Check: ..."

### ADVISORY 4: refusals without valid neighbours, and a rule without a negative fixture

Governing text:
- `AGENTS.md:237-238`: "When you add or tighten a refusal, run the case you believe is invalid and a neighbouring valid case, and confirm the second succeeds."
- `AGENTS.md:235-236`: "A rule or check with no input that makes it fail is not enforced."

Plan lines:
- R6: "`Fingerprint` parse refuses other kinds/widths (negative fixture)". No valid parse is named.
- R5: "trait-impl method and `Self::` forwarder produce no `shim` group". No neighbour that groups is named.
- R9: "one fixture per form and case" for hex tables. No near-miss that is not detected is named.

REWRITE TO:
- R6: "... refuses `blob:`, 15 and 17 hex digits and uppercase hex; parses `body:<16 lowercase hex>`, `shim:...`, `table:hex-lower`."
- R5: "... produce no `shim` group, while the same forwarder body as a free function and as an inherent method not calling through `Self::` does group."
- R9: "... plus a negative per form: a 15-digit, a mixed-case and an out-of-order digit table are not reported."

### ADVISORY 5: "with the reason" must not put narrative into the architecture page

Governing text:
- `docs/capabilities/workplan-current-rationale.md:142-143`: "A decision an architecture page records is resolved. The page does not narrate how the decision was reached."
- `AGENTS.md:282-283`: "Documents describe the system, not its history."

Plan line: "in which case the page changes in the same commit with the reason".

REWRITE TO: "in which case the page states the resolved rule and the commit message gives the reason; the page does not narrate the change."

Precedence note, not a finding: `CONTRIBUTING.md:26-27` says "Never change normal form 1 in place." No v1 code or fingerprint exists today (`README.md:35`: "no census code exists yet"), and the issue requires the v1 definition (R11, R16). Defining v1 in this change is therefore compliant. The plan should confine fingerprint-affecting page edits to this change, before the goldens are first committed in Task 2. After that, any edit that moves a pinned value falls under the version-bump rule.

### ADVISORY 6: delegated agents must be told the no-timing rule

Governing text: `CLAUDE.md` (Quick orientation): "Write no timing runs, wall-clock benchmarks, sleep-based waits or timing assertions. ... and tell every agent you start this rule."

Plan line: "Tasks (each: implement, independent review by a fresh reviewer, commit, push, issue update)". The plan does not say that the brief for each implementer or reviewer carries the rule.

REWRITE TO: "(each: implement, independent review by a fresh reviewer whose brief states the no-timing-runs and no-mutation-passes rule, commit, push, issue update)".

### ADVISORY 7: pin the audited `blake3` release

Governing text: `deny.toml:10-12`: "an addition should require someone to look at the new crate and decide ... allow = [\"MIT\", \"Apache-2.0\", \"MulanPSL-2.0\"]".

Plan line: "the `blake3` crate (1.8.x) ... every licence is already allowed by `deny.toml` (no new exception)."

This claim holds for `blake3` 1.8.7. Its dependencies are arrayvec, cfg-if, constant_time_eq, cpufeatures and the build dependency cc (with shlex and find-msvc-tools), all MIT/Apache-2.0 disjuncts. `blake3` 1.8.5, which is also in the local registry, depends on `arrayref`, whose licence is `BSD-2-Clause` and is not allowed. A `1.8` requirement admits 1.8.5.

REWRITE TO (Task 1): "`blake3 = \"1.8.7\"` (the audited release; its graph needs no licence exception)".

### ADVISORY 8: golden and fixture files must satisfy REUSE and the clock gate

Governing text:
- `AGENTS.md:216-218`: "Code, configuration, templates and brand assets are `MIT OR Apache-2.0 OR MulanPSL-2.0`, with the SPDX lines in the file's own comment style. Files that cannot carry a header are annotated in `REUSE.toml`."
- `AGENTS.md:151-163`: the `time_in_crates_` gate scans every tracked Rust file under the crate's `src`, `tests` and `examples`, fixture snippets included.

Plan lines: R10 "exact JSON goldens"; Task 2 fixtures under `crates/raf-census/tests/`.

REWRITE TO (Task 5): "JSON goldens are inline string constants in the Rust tests, or, if kept as `.json` files, are annotated in `REUSE.toml` in the same commit; no fixture snippet names a clock word (`Instant`, `sleep`, `SystemTime`, `UNIX_EPOCH`, the `*_timeout` waits)."

### ADVISORY 9: crate-status prose that becomes false

Governing text:
- `AGENTS.md:66`: "Every other crate below is **designed** and joins the workspace members list when it is built."
- `AGENTS.md:280-283`: "Do not write ... wording that refers to the development process that produced a file (plans, tasks, acceptance criteria, branches)."

Task 1 makes `raf-census` a member. Several statements then become stale: `AGENTS.md:52` and `:85` (member lists), `AGENTS.md:47-48` and `CONTRIBUTING.md:36-37` ("the census gate joins them when `raf-census` is built"), and `README.md:33-35,41` ("no census code exists yet", "designed — not yet published"). Task 1 says "`AGENTS.md` crate table/text", and Task 6 says "README/AGENTS ... where applicable". Neither names these lines.

REWRITE TO (Task 1): "AGENTS.md lines 47-48, 52, 66, 72, 85, CONTRIBUTING.md lines 36-37 and README.md lines 33-35, 41 restated for `raf-census` as a member that implements the normal form (the census gate stated as belonging to the Gate capability, without 'when ... built' wording); test names and comments carry no R-ids or task numbers."

## Checked and compliant

- The new crate inherits the workspace lints. No `#![feature]`, no `unsafe`, `missing_docs` and `unreachable_pub` are honoured (`AGENTS.md:104-105,194-206`; `repo_gates.rs:1279` lint inheritance).
- The PROPOSED blocks are added in the same commit as `implementing`, in sections listed in `affects` (`okf-profile.md:209-210,228-231`).
- The item split: the version gate is `implemented` here, the ledger-file fixture is `moved` to `change:census-ledger`, and the job-match producer is tracked by a new item in `change:census-ledger`. This complies with "No deferrals ... moved to another tracked change, with its own record" (`AGENTS.md:286-288`) and with the item outcomes (`AGENTS.md:262-263`).
- The completion route is `cargo workplan change complete` (`AGENTS.md:267-273`).
- Port licensing: the purrdf code is `MIT OR Apache-2.0 OR MulanPSL-2.0` from the same holder, and none of the AGPL katamari code is copied (`AGENTS.md:216-218`).
- The thresholds match `arch:normal-form` (purrdf `normalize.rs:48,53,60,64` gives 20/30/30/6).
- No timing runs and no mutation passes are planned. Determinism is checked by byte comparison.
- Conventional commits, no attribution, and no tracker numbers in committed files. `Closes #3` appears only in the PR body.
- `.deficiencies` has no entries, and the plan adds none.

VERDICT: NEEDS-REVISION
