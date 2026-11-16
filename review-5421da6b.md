<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Delivery review at 5421da6b9489cae67abe9ee20c51e123f69ac512

Written 2026-11-15 23:36 -0600 (clock reading). Verdict at this exact head: **BLOCKED**, with one REQUIRED finding.

## Scope

- Worktree: `/home/paudley/Active/rust-agentic-foundations/.worktrees/272-workplan-a-failing-measured-record-does`.
- Branch: `paudley/272-workplan-a-failing-measured-record-does`.
- HEAD is 5421da6b and the worktree is clean.
- Range: `origin/main..HEAD`, eight commits: 09c1ca0b, 061849ef, 9efcb41a, f437b3ec, 2a4cf3de, 54ec7c9a, fbcdb745 and 5421da6b.
- Size: 39 files change, 13 code and document files plus 26 assessments.
- What 5421da6b changes: only `crates/raf-workplan/tests/strict.rs`, with 56 lines added and 7 removed.
- Contract: the verbatim issue at `.stage/workplan-a-failing-measured-record-does/issue.md`, and AGENTS.md sections 5 to 8. I read the issue first. The plan's `## User rulings` holds one ruling: both findings go in one change, as their own reviewed PR. The branch honours it.

## Evidence

Every cargo run met these conditions:

- It ran under `flock /tmp/raf-build.lock`, from the worktree, with `--locked`.
- Its output went to a log under `.stage/workplan-a-failing-measured-record-does/reviews/`.
- The shell recorded the tool's own exit status directly, never through a pipe.
- The cargo wrapper read `2026-11-15 20:39:04.316553472 -0600`, sha256 `857ea5b982024ba7...`, at both the start and the end of the run. Every run therefore counts.

No log has a line starting `/home/paudley/stage/scripts/cargo: line`.

| Command | Log | Exit | Scope |
|---|---|---|---|
| `cargo test --locked -p raf-workplan --test strict --test kinds` | `r8-tests.log` | 0 | 25 strict tests and 6 kinds tests passed. These include `a_failing_or_stale_measured_record_does_not_satisfy_the_requirement` and its two helpers. |
| `cargo test --locked -p raf-okf --test records` | `r8-okf.log` | 0 | 38 passed, including the `UnitWithoutValue` refusal beside the complete record |
| `cargo clippy --locked -p raf-workplan -p raf-okf --all-targets -- -D warnings` | `r8-clippy.log` | 0 | The two changed crates, all targets, with the gate's `-D warnings`. It is not the workspace gate. |
| `cargo test --locked --test self_workplan --test docs_contract` | `r8-gates.log` | 0 | Every assessment is current and accepted at HEAD, and the document contract and inventory hold |
| `prose-check` on the comments added in 5421da6b, and `--message-file` on its commit message | `r8-prose.log` | 0, 0 | No findings |

I ran no workspace gate, no hook, no mutation testing and no fuzzing.

## Acceptance on the production path

Every strict case runs through `check_in` (`tests/support/mod.rs:127-130`), which is `Workplan::load(..).check()`. That call reaches `strict::check` and then `measured_evidence` (`strict.rs:153-155`).

| Issue acceptance item | Code | Test at HEAD |
|---|---|---|
| A failing record does not satisfy the requirement and the finding names it. A passing record beside it satisfies the requirement. | `strict.rs:197-221` | `tests/strict.rs:731-747` covers failing only: kind, record id, file and key. `:771-787` covers failing beside passing current, where the whole `check_in` result is `[]`. |
| A passing record at stale digests does not satisfy the requirement. | `strict.rs:222-238`, `:280-287`; `digest.rs:415-432` | `tests/strict.rs:750-756` |
| A unit with no value is refused, beside a complete record that is accepted. | `raf-okf/src/profile/records.rs:1102-1113`; `violation.rs` `UnitWithoutValue`; `finding.rs:669-672` | `raf-okf/tests/records.rs:1525-1533` beside the clean `measured()`; the workplan kinds case `cases.rs:504-506` |
| A capability without the requirement is judged as before. | `strict.rs:153-155` | `tests/strict.rs:685-699`, where both the absent key and `false` give `[]` |

## Each combination against the three texts

The three texts are:

- The contract: `docs/capabilities/workplan-evidence-claims.md:67`.
- The module comment, `strict.rs:39-53`, and the function comment, `strict.rs:176-185`.
- The architecture page: `docs/architecture/okf-profile.md:222-226` and `:318-325`.

| Covering measured records | Code result | Texts agree | Bound test (`tests/strict.rs`) |
|---|---|---|---|
| Failing only | `[failed]`, naming the record | Yes, all three | `:731-747` |
| Failing and passing current | `[]` | Yes. The architecture page says so through its "unless" clause. | `:771-787`; `:866` with the refusal fixture under the normal allowance |
| Failing and passing stale | `[failed, missing]`. The missing finding names "`<input>` changed" and its fix is to record again. | Yes | `:897-913` |
| Failing and passing refused (`max_query_work = 0`) | `[failed, missing]`. The missing finding carries the refusal error and the refusal fix. | Yes | `:857-879` |
| Two failing | `[failed, failed]` | Yes, "once for each such record" | `:880-896` |
| Passing stale only | `[missing]`, naming the changed input | Yes | `:750-756`; `:757-768` for an input that no longer resolves |
| Two stale passing records | One `[missing]` that names both records and every changed input of each | Yes: "one finding for all such passing records" and "shared by all such records" | `:920-951`, new in 5421da6b |
| Passing refused only | `[missing]` with the refusal error. Its fix does not say to record again. | Yes | `:827-856` |
| No record | `[missing]`: "no evidence record that covers it states ..." | Yes | `:657-667`; `:670-681` for records that state no measurement |
| A unit with no value | Load refusal `okf-unit-without-value` at `units`, naming the unit | Yes. Contract, okf-profile capability `:722` and architecture `:321-323` agree. | `raf-okf/tests/records.rs:1525-1533`; `cases.rs:504-506` |

I checked every sentence of the three texts against the code. The code honours each one. Each one also has a bound test, with one exception (REQUIRED 1). The sentence "The findings are not debt" is covered by `tests/debt.rs:1031-1047`: every debt kind must start with `strict-` or `coverage-`, so a `measured-evidence-*` kind cannot become one.

## Earlier findings at this head

| Review | Finding | At 5421da6b |
|---|---|---|
| 061849ef | No test of the refusal path | Resolved: `tests/strict.rs:827-856`, beside the same record under the normal allowance at `:827` |
| 9efcb41a | A failing record hid a refused passing record | Resolved: the early return at `strict.rs:243-245` fires only with no reasons, and `tests/strict.rs:867-879` gives `[failed, missing]` |
| f437b3ec | Contract sentences contradicted each other | Resolved: one consistent rule at `workplan-evidence-claims.md:67` |
| 2a4cf3de | An untested refusal-precedence clause | Resolved: the clause is removed |
| 2a4cf3de | An unconditional architecture sentence | Resolved: `okf-profile.md:224-226` ends with the "unless" clause |
| 54ec7c9a | An over-broad naming claim for refused inputs | Resolved: "each input" is claimed only for stale records |
| fbcdb745 | An untested one-finding-for-several-records claim | Resolved. `stale_passing_records_share_one_finding` (`tests/strict.rs:920-951`) loads two stale passing records, `evidence:bench` with two inputs and `evidence:bench-other`. It asserts exactly one `measured-evidence-missing` that contains `evidence:bench`` ` (the backtick keeps `bench-other` from matching), `evidence:bench-other`, `demo::run changed` and `demo::second changed`. A finding for each record would fail the kinds assertion. |
| fbcdb745, carried OPTIONAL | "Each input" was shown with one input per record | Resolved by the same test, where `evidence:bench` has two changed inputs |
| fbcdb745, OPTIONAL 1 | The refused case did not check the message wording | Resolved: `:838-843` and `:844-852` assert "could not be checked against the source" |

## Findings

### REQUIRED

1. **The contract states what the refusal fix says, and no test checks it.**
   - The text, `docs/capabilities/workplan-evidence-claims.md:67`: "A refusal by the source index is reported with its error and not as a changed input, and its fix is to repair the source or raise the query allowance, not to record again."
   - The code honours the text. `strict.rs:250-251` sets the fix to "repair the source or raise the query allowance of the load ...; recording the evidence again cannot clear a refusal".
   - The gap: the bound test only checks what the fix does not say. `tests/strict.rs:853-856` and `:877` assert `!fix.contains("record an evidence record")`, and no other test names this fix text. A search of `crates/` for "query allowance" and "repair the source" finds only the production string and an unrelated comment. If the fix were replaced by any other text, including an empty string, every bound test would still pass and the sentence would be false.
   - Earlier tables (54ec7c9a, fbcdb745) list "fix says to repair or raise the allowance" as tested, but no earlier finding names this gap, so it is new.
   - Remedy: at `tests/strict.rs:853-856`, also assert that `refused[0].fix` contains "raise the query allowance", or "repair the source". The section text and the bound symbols do not change, so no assessment needs renewing.

### OPTIONAL

1. **A repeated assertion.** `tests/strict.rs:838-843` asserts that the message contains "could not be checked against the source", and the combined assertion at `:844-852` checks it again. Drop one of them.
2. **Carried from earlier reviews and unchanged.** These are not re-raised as REQUIRED.
   - The contract marker binds no test of the unit refusal. The refusal is tested in `raf-okf` (bound from `okf-profile.md:720`) and in the workplan kinds case.
   - No completion test uses a failing record.
   - When one stale and one refused passing record share a finding, the fix is the refusal fix only (`strict.rs:246-254`).
   - `strict.rs:50` (84 columns) and `:51` (102 columns) run past the comment's wrap width.
   - The test name says less than the test covers.

## Test value, prose and scope

- The new test in 5421da6b protects a stated contract choice: stale passing records are aggregated into one finding, while failing records each get their own. It reuses the existing behavioural case rather than adding a new `#[test]`. It names no clock and uses no mutation testing.
- `measured_only` filters to `measured-evidence-*` kinds. The aggregation case needs it, because the added `demo::second` is unbound. The filter cannot hide a finding of the rule under test.
- `prose-check` reports nothing for the new comments or the commit message. The earlier r7 run covered the unchanged documents and Rust comments.
- The subject `test(workplan): ...` is conventional. The added lines carry no tracker reference, process wording or attribution.
- Scope stays within the issue. The `stale_symbol_inputs` extraction (`digest.rs:405-432`) is shared by `strict.rs:281` and `diagram/semantic.rs:660-667`. The diagram still refuses a non-symbol input before it queries a digest. Evidence inputs can only be symbols (`raf-okf/src/profile/violation.rs:636-638`).

## Assessment renewals

- 5421da6b touches no assessment and no assessment input. Assessment inputs are sections and source symbols, not test functions (`work/assessments/workplan-evidence-claims-contract.md:5-33`). `self_workplan` passes at HEAD.
- All 26 renewed files keep lines 1 to 4 as `---`, the two SPDX comment lines and `type: Assessment`. I checked this mechanically, and all 26 headers are identical.

| Assessments | Changed input | Judgement |
|---|---|---|
| workplan-evidence-claims contract | the section; `raf_workplan::diagram` | **Justified.** I re-read line 67 against `strict.rs:186-287` and every sentence holds in the code. REQUIRED 1 is a test gap, not wrong text, so the renewal is accurate and needs no change. |
| workplan-evidence-claims constraints, examples, explanation, purpose, verification | `raf_workplan::diagram` | **Justified.** For every evidence input that loads, the extraction leaves the diagram's behaviour unchanged. |
| workplan-generated-docs contract, examples, explanation, purpose, usage, verification | `raf_workplan::diagram` | **Justified.** The stale reasons and `currency` come from the same function, so current and stale evidence stay distinct. |
| workplan-document-model constraints, examples, explanation, purpose | `raf_workplan::digest` | **Justified.** The new item is crate-private, and no public digest contract changes. |
| workplan-strict constraints, contract, examples, explanation, purpose, verification | `raf_workplan::strict` | **Justified.** That document does not describe measured evidence; the rule lives in workplan-evidence-claims. The new kind is not debt, so the debt-kind text still holds. |
| okf-profile constraints, explanation, purpose | `raf_okf::Rule`, `raf_okf::RuleTag` | **Justified.** These sections contain no rule list or rule count. |
| okf-profile contract | the section | **Justified.** `okf-profile.md:722` names `Rule::UnitWithoutValue` at `units`, which matches `records.rs:1102-1113`. |

## Coverage

Read:

- `.stage/workplan-a-failing-measured-record-does/issue.md`, whole; `plan.md` lines 1 to 40, including the User rulings.
- `/home/paudley/.claude/skills/rust-work/SKILL.md`, whole.
- AGENTS.md at the worktree root, whole.
- `review-fbcdb745.md` whole, the second half of `review-54ec7c9a.md`, and targeted searches of all six earlier reviews.
- The whole branch diff for `crates/` and `docs/`, uncompacted for `strict.rs`, the tests and the documents (saved as `r8-diff.txt`), and the full `git show 5421da6b`.
- `crates/raf-workplan/tests/strict.rs:613-951`.
- `crates/raf-workplan/tests/support/mod.rs:110-140` and `support/changes.rs:92-100`.
- `crates/raf-workplan/tests/debt.rs:1020-1048` and `crates/raf-workplan/src/debt.rs:120-140`.
- `crates/raf-workplan/src/strict.rs:39-53` and `:176-287`, through the diff.
- `crates/raf-workplan/src/digest.rs:405-432` and `diagram/semantic.rs:650-675`, through the diff.
- The `raf-okf` hunks in `records.rs`, `violation.rs` and `tests/records.rs`, and the workplan `finding.rs`, `kinds.rs` and `cases.rs` hunks.
- `docs/capabilities/workplan-evidence-claims.md:45-68` and `:144-147`, `docs/capabilities/okf-profile.md:719-723`, and `docs/architecture/okf-profile.md:221-226` and `:316-325`.
- `work/assessments/workplan-evidence-claims-contract.md`, whole, and lines 1 to 4 of all 26 renewed assessments.
- `Makefile:52-53` and the tail of `.stage/.../fix10-commit.log`.

Not read:

- The bodies of the other 25 renewed assessments beyond their headers. For the sections other than the evidence-claims contract, my judgements reuse the section readings of the earlier reviews, because no section or input of those assessments changed after fbcdb745.
- The rest of `strict.rs` (claims and coverage), which this change does not touch.
- The full Contract sections of workplan-strict, workplan-document-model and workplan-generated-docs.
- `tests/complete.rs`, read in earlier reviews and unchanged since.
- `validation.md`; I relied on my own runs.
- Hosted CI for this head.

VERDICT: BLOCKED
