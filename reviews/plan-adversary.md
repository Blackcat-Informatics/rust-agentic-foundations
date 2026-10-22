# Plan completion adversary review (Stage 1)

* Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/16-raf-workplan-strict-mode-force-coverage/.stage/raf-workplan-strict-mode-force-coverage/plan.md` (Rev 0), with its annex `design.md` (cited as D§n)
* Pass: 1 (this reviewer runs once; `plan-final-reviewer` verifies the fixes)
* Inputs read: `issue.md` (body, 0 comments), `issue-analysis.md` (R1-R21, seven NEEDS-JUDGMENT items), `plan.md`, `design.md`, `CLAUDE.md`, `AGENTS.md` sections 6 and 7, and `.deficiencies` (nothing below the marker). The worktree has no `.baseline`, `.goals`, `CONSTITUTION.md` or ADR directory, and `brief.json` lists `"adrs": []`.
* Quoted user scope decisions given to this reviewer: **none**. The plan cites one decision without quoting it: "breaking changes before the first release are acceptable (user decision in this session)" (plan.md:28).
* Repository facts were checked read-only at `95c384c`. No build was run and no timing was measured.

## The caller's question: does adopting strict with about 786 debt rows discharge R18?

R18 reads: "Once the rules pass, the repository enables strict on its own docs."

**The plan does not hide the cut, but it does not discharge R18 honestly without a user decision either.**

* **It is declared.** plan.md:58-61 states the debt plainly and puts the debt diff in the PR. R9 supports reading the debt as adoption: "so adopting strict on an existing repository can start from today's findings and fail only on new ones". So this is not a silent cut.
* **Most strict rules pass on existing artifacts only through the debt record.** The plan's own measurements are D§3.2 ("196 units ..., 0 covered") and D§6.3. The debt record would hold:
  * 196 of 196 claim units of the existing delivered documents (100%);
  * every unbound public item and test of raf-okf and raf-source (211 + 379);
  * every orphan line of four crates (about 10,362).
  So on this repository's existing documents, three of the issue's rule families (claims R3, orphan symbols R4, lines R10/R11) enforce only the no-growth ratchet. Only the content rules (R2, R6) pass outright, after the 13 fixes.
* **The plan's R18 check passes by construction.** `debt --write` adopts whatever strict raises when no record exists (D§3.8). So "`cargo workplan --strict check` exits 0 with the recorded debt" holds for any repository with no default finding. It cannot tell "the rules pass" from "the debt absorbed everything".
* **The design concedes the other reading.** D§5.7 says: "If the maintainer reads 'once the rules pass' as 'with an empty debt record', the cost is in section 6.3 and needs its own change record for the documentation work." The plan answers that question by itself. It lists "paying the 786 debt rows in this change" under "Declined / out of scope" (plan.md:106-108) and quotes no user decision. That is the implementer's scope judgment on an ambiguous acceptance criterion. Under AGENTS.md:286-288 ("Scope that leaves a change is moved to another tracked change, with its own record ... Do not write ... 'out of scope for now' as a substitute"), it cannot stand as written if the strict reading applies.
* **As sequenced, T8 cannot reach its own end state** (finding F2). It also leaves about 2,925 orphan lines in two packages neither enforced nor recorded (finding F4).

Result: **ESCALATE-TO-HUMAN (E1)**. Whichever choice the user makes, F1 and F2 must also be fixed.

## Findings

### F1. UNFALSIFIABLE CRITERION: the R18 check is satisfied by construction

Quote (plan.md:87):
> | R18 | once the rules pass, the repository enables strict on its own docs | 8 | `strictness: strict` set; `cargo workplan --strict check` exits 0 with the recorded debt; `self_workplan` judges strict |

Why it fails:
* Adoption writes the present debt, so `check` exiting 0 is guaranteed.
* `--strict` raises the floor whatever the record says, so the command does not show that the repository enabled anything.
* Nothing observes "`self_workplan` judges strict". D§5.7 says it happens "with no code change".

REWRITE TO (under E1 choice A or C; under B, replace part (c) with "`work/debt/default.md` does not exist and `cargo workplan check` exits 0"):
> | R18 | once the rules pass, the repository enables strict on its own docs | 8 | (a) at the PR head `work/coverage/default.md` holds `strictness: strict`, and `cargo workplan check` with no `--strict` flag exits 0; (b) `tests/self_workplan.rs` asserts `workplan.strictness() == Strictness::Strict` on the repository; (c) `cargo workplan coverage claims` exits 1, and its uncovered claims equal, one to one, the `strict-claim-uncovered` rows of `work/debt/default.md`. `cargo workplan coverage lines` prints, per crate, the orphan count of that file's `lines` rows, so the record holds the whole present gap and nothing else; (d) `work/debt/default.md` holds no row whose subject is `cap:workplan-strict` and no row of kind `strict-example-missing`, `strict-matrix-missing`, `strict-section-thin`, `strict-usage-unbound` or `strict-limitations-missing`; (e) the PR body quotes the user's E1 decision and lists the row count per kind and the orphan lines per crate as written by T7/T8 (not the D§6.3 estimates) |

### F2. UNMAPPED REQUIREMENT (insufficient task): the T8 order cannot reach `change complete` with `check` exiting 0

Quotes:
* plan.md:93-96: "T1-T8 are those of D§6.1 (... T8 adopt strict in this repository and deliver)"
* D§6.1 T8 (design.md:1010-1015): "Set `strictness: strict` in `work/coverage/default.md`, add the exceptions that are true (shared test support), then `cargo workplan debt --write ...` to create `work/debt/default.md`; set the listed capabilities `delivered`, `docs sync`, assess the affected sections, update `INVENTORY`, `cargo workplan change complete workplan-strict`."
* D§3.8 (design.md:636-638): "`write`: refuses (`DebtError::Findings`) while a default finding exists; refuses growth against an existing record ..."

Evidence:
* The strict content and claim rules judge only `delivered` capabilities (D§3.1 "for each loaded `delivered` capability", D§3.2, D§3.5).
* From T1 to T8, `cap:workplan-strict` and the eight re-delivered capabilities are `implementing` (D§6.2).
* A delivered capability listed by an open record is a **default** finding. `crates/raf-workplan/src/rules.rs:151-160` raises `state-delivered-open-record`.

So both possible orders fail:
* **Write before setting `delivered`.** The adoption record misses the claim and content findings of those nine capabilities. Setting them `delivered` then adds those findings as growth. `change complete` refuses on its after-removal `check`, and `debt --write` refuses growth.
* **Write after setting `delivered`, before completion.** The working tree holds `state-delivered-open-record`, so the write refuses.

This also means the "about 786" figure cannot be the adoption content under the D§6.1 order.

REWRITE TO (T8, in the plan itself):
> T8 order: (1) reconcile every PROPOSED block, fix the 13 findings, make `cap:workplan-strict` strict-clean, set the listed capabilities `delivered`, `docs sync`, assess, update `INVENTORY`; (2) `cargo workplan change complete workplan-strict` with the coverage record still `strictness: default`; (3) in the next commit of the same PR, set `strictness: strict`, add the true exceptions, run `cargo workplan debt --write --copyright ... --licence CC-BY-4.0` (adoption), and run the R18 check after this step. The change record carries no item that only step 3 makes true (remove the item "This repository runs under strict mode." from the D§6.2 sketch; the setting is repository data, not a capability guarantee).

Alternative, if the item must stay in the record:
> `Gate::write_debt`/`debt --write --after change:<slug>` writes the debt of `Opened::workplan_after(<record>)`, the view `change complete` judges. A T7 fixture shows it: an open change whose capabilities turn `delivered` writes the debt they will owe, and `change complete` then passes, beside the same write without `--after`, which refuses.

### F3. UNFALSIFIABLE CRITERION and a factual undercount: R14 says "the one schema pin", but at least four existing tests change

Quotes:
* plan.md:62-65: "The `Debt` type changes one pinned expectation (`crates/raf-okf/tests/schema.rs` lists the document types). 'Every existing fixture unchanged' is read as unchanged behaviour: the pin is updated, and the PR says so."
* plan.md:83: "| R14 | ... | the one schema pin updated and declared |"

Evidence: the existing expectations that the planned design changes.
1. `crates/raf-okf/tests/schema.rs:124-127` pins `keys(DocType::Coverage)` as `["type", "id", "tags", "timestamp", "scope", "exclusions"]`. The plan adds `strictness`, `ceilings` and `exceptions` to that schema (D§2.1, D§3.9). This changes even under the JSON-debt fallback.
2. `crates/raf-okf/tests/schema.rs:194-212`, `only_coverage_is_a_singleton_and_only_the_three_records_are_records`. Line 197-200 (records are Evidence, Assessment and Coverage), line 205 (only Coverage is a singleton) and line 208-211 (the type list) are all falsified by a singleton record type `Debt` (`debt:default`, D§3.8). The test's name becomes untrue, not only its string.
3. `crates/raf-okf/tests/records.rs:922-931`, `a_record_type_is_named_in_the_unknown_type_message`, pins "... Evidence, Assessment and Coverage".
4. `crates/raf-workplan/tests/documents.rs:356-377` pins the six load directories "in the order of `DocType::ALL`". If `load` reads `work/debt` (D§3.8), the list gains a seventh.
5. `crates/raf-workplan/src/finding.rs:473`: the default-mode fix text of `okf-item-unknown-key` changes (D§3.6). No test pins it, but it is a default-behaviour text change.
6. Inputs that default mode rejects today are accepted afterwards: an item key `reason`, and the coverage keys `strictness`, `ceilings` and `exceptions` (D§7.3).

Plus the growth that R15 itself requires: `OWN_KINDS` 48 -> 60, `ObligationKind::ALL` 13 -> 15, and the non-rule kind list of `cargo-workplan/tests/json.rs:263`.

REWRITE TO (plan.md:62-65 and the R14 row):
> | R14 | default behaviour and every existing fixture unchanged | all | `all_cases()` and every existing `Case` keep their verdict; every strict-violating fixture yields no finding without strict; the existing test expectations that change are exactly: `raf-okf/tests/schema.rs:124-127` (coverage keys), `schema.rs:194-212` (records, singletons, type list), `raf-okf/tests/records.rs:922-931` (type message), `raf-workplan/tests/documents.rs:356-377` (load directories), the `okf-item-unknown-key` fix text, and the R15 list growth. Each is changed under the user's quoted E2 decision and listed in the PR body |

### F4. PRE-EMPTIVE DESCOPE: reverse coverage is enforced under strict only for bound or `scope` packages, so a crate that no capability binds escapes the gate

Quotes:
* plan.md:55-57: "ceilings per crate (default 0 under strict)"
* D§3.9 (design.md:729-730): "Under strict every enforced package (`scope` + bound packages + every package a row names) is judged; a package without a row has the ceiling `lines: 0`."
* D§6.3 (design.md:1118): "| not enforced (not bound) | root package 2,848 test lines, `raf-source-fuzz` 77 | - | - |"

What the issue asks:
* R10: "report the source lines of a crate that map to no deliverable or specification, for example 'crate x has 100 lines that no capability binding or contract explains'", "also enforced under strict with a threshold".
* R11: "a configurable ceiling ... per crate".

The issue's own example is a crate with no binding. The "bound crate path" qualifier belongs to R4 (orphan symbols), not to R10/R11. Under the plan, the wholly unbound root package (2,848 orphan lines) and `raf-source-fuzz` (77) are neither judged nor recorded as debt when this repository adopts strict. No user decision covers this narrowing, and plan.md does not state it.

REWRITE TO (plan.md:55-57, and the R11 row):
> ceilings per crate: under strict every package of the source workspace is judged against its ceiling (a package without a row has `lines: 0`), whether or not a capability binds it. Fixture: a package bound by no capability, with orphan lines and no ceiling row, raises `coverage-lines-over-ceiling` under strict and nothing under default. T8 excepts with a reason, or records as `lines` debt rows, the root package and `raf-source-fuzz`.

Keeping the narrowed set instead requires a quoted user decision and a Constraints item of `cap:workplan-strict` stating that unbound packages are never judged.

### F5. UNFALSIFIABLE CRITERION: the R10 check does not observe what R10 asks for

Quote (plan.md:79):
> | R10 | reverse coverage report: orphan lines per crate/module/item, counts, ranges, sorted, text and JSON | 5, 6 | the R12 fixtures; report identical across runs |

"The R12 fixtures" check the classification of lines. Nothing checks the per-module and per-item tables, the merged ranges, the sort order or the JSON form.

REWRITE TO:
> | R10 | ... | 5, 6 | `cargo workplan coverage lines --crate <fixture>` on the R12 fixtures prints three tables: the crate table (Counted, Covered, Excepted, Orphan, shipping/test orphan, Ceiling, Debt), the module table, and the item table with merged `a-b` ranges sorted by package, module path, file and first line. `--format json` prints `lines-crate`, `lines-module` and `lines-orphan` objects (`ranges` as `[[a,b],...]`) carrying the same rows. Text and JSON each equal a literal expected string; exit 1 when an orphan line is printed, 0 when none |

### F6. UNFALSIFIABLE CRITERION: R12's "documented" and "identical across platforms" have no observation

Quote (plan.md:81):
> | R12 | ... macro items and `#[cfg(test)]` handled and documented; identical across runs and platforms | 5, 6 | those fixtures by name; resolution from tests/binaries and `impls()` (T5) |

Two gaps:
* "Documented" names no document. The Constraints list of D§6.2 does not mention the macro-invocation rule or the `#[cfg(test)]` rule.
* D§3.9 proves "identical across platforms" by comparing a CRLF tree with an LF tree on one host. That proves platform identity only if the expected output is a literal, which the hosted `test-macos`/`test-windows` jobs then check.

REWRITE TO:
> | R12 | ... | 5, 6 | the D§3.9 fixtures by name, each asserting the report against a literal expected text and JSON (so the hosted `test-macos` and `test-windows` jobs check the same bytes), plus a CRLF/LF pair and two runs byte-identical; the Contract or Constraints of `cap:workplan-strict` states the macro-invocation rule (D§3.9 rule 4) and the `#[cfg(test)]` rule, each with a `raf:tested-by` marker naming its fixture |

### F7. UNFALSIFIABLE CRITERION: the R16 check is "per command"

Quote (plan.md:85):
> | R16 | CLI integration tests for exit codes 0, 1, 2 | 2, 3, 4, 6, 7 | `cargo-workplan/tests/*` per command |

REWRITE TO:
> | R16 | ... | 2, 3, 4, 6, 7 | `cargo-workplan/tests/check.rs`: `cargo workplan --strict check` exits 0 on the strict-clean fixture, 1 on it with one thin section (0 there without `--strict`), 2 with `--root` at a directory without `docs/capabilities`; `cargo workplan --strict change reconcile <slug>` exits 1 for a `none` change with an untouched Usage, 0 after the edit, 2 for a merge-base blob that does not parse; `coverage claims` and `coverage lines` exit 0/1/2 (2: unknown `--capability`/`--crate`); `debt --write` exits 0 (written), 1 (growth), 2 (record not strict) |

### F8. UNFALSIFIABLE CRITERION: R15 says one "existing both-directions test" covers every new kind, but two new kinds are obligations judged by a different test

Quote (plan.md:84):
> | R15 | ... | 2-7 | the existing both-directions test covers every new kind |

`outcome-without-reason` and `contract-effect-understated` are `ObligationKind`s (D§3.10), not `Finding::KINDS`. They are proven by `tests/complete.rs::the_cases_cover_every_kind_of_obligation_in_both_directions` (complete.rs:986), not by `tests/kinds.rs::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` (kinds.rs:105).

REWRITE TO:
> | R15 | ... | 2-7 | the twelve own kinds and the new `okf-*`/`okf-parse-*` kinds of D§3.10 are in `Finding::KINDS` and each has a named `Case` in `kinds.rs::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` (`OWN_KINDS` 48 -> 60); the two obligation kinds are in `ObligationKind::ALL` and covered by `complete.rs::the_cases_cover_every_kind_of_obligation_in_both_directions` and `each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes`; the JSON objects of strict findings carry `kind`, `path`, `section`, `line`, `key`, `message`, `fix` |

### F9. UNFALSIFIABLE CRITERION: the R5 parts said to "already hold" are mapped to "fixtures unchanged"

Quote (plan.md:74):
> | R5 | ... | 4 | ... the existing conditions' fixtures unchanged |

The code supports the claim:
* condition 5 runs every `Test | Example` binding of every listed capability (`lifecycle/complete.rs:329-336`, with `impact.bindings` from `lifecycle/impact.rs:148-151`);
* condition 1 counts `tense-mismatch`.

The gap is the check. "Unchanged" observes nothing under strict, and the issue asks that `change complete` under strict *requires* both.

REWRITE TO (first two parts):
> `raf_workplan::test_crate::complete::a_failing_bound_test_refuses_condition_five_alone`, `raf_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists` and the `tense-mismatch` case of `complete.rs::each_kind_of_obligation_makes_complete_refuse_...` are bound in `cap:workplan-strict` and named by the `raf:tested-by` marker of its **Completion.** clause; a strict-gate variant (`Lifecycle::open_with(&gate.strict(), ..)`) of the failing-bound-test case refuses condition 5 the same way

The third part depends on E3.

### F10. UNFALSIFIABLE CRITERION: "runnable" in R2 is not observed

Quote (plan.md:71):
> | R2 | every delivered capability binds a runnable example/doctest ... | 2 | `strict-example-missing` vs one example; ... |

`strict-example-missing` checks only that `bindings.examples` is non-empty. D§3.1 delegates runnability to `verify` and condition 5, but the row names no check of it.

REWRITE TO:
> `strict-example-missing` vs one example; and a delivered capability whose only example binding is a `no_run` documentation test is refused by `cargo workplan verify` and by condition 5 (the existing `no_run` fixture of `raf-workplan/tests/bound.rs`, bound in `cap:workplan-strict`), beside a runnable one that passes

### F11. UNFALSIFIABLE CRITERION: R19, R1 and R21 point at no committed observation

Quotes:
* plan.md:88: "| R19 | ... | 1 | D§4 recorded; no duplicated rule ... |". D§4 is in the git-ignored stage directory.
* plan.md:70: "... the location justification text in `cap:workplan-strict` |"
* plan.md:90: "| R21 | opt-in; the default stays lightweight | 2 | default fixtures unchanged; documented |"

REWRITE TO:
* R19: "the Explanation of `cap:workplan-strict` states the reconciliation with `cap:workplan-two-way-coverage` and `cap:workplan-evidence-claims` (the D§4 table as prose); the Contract of `cap:workplan-two-way-coverage` gains the one sentence of D§4; `OWN_KINDS` gains no kind for orphan symbols"
* R1: "the Explanation of `cap:workplan-strict` holds a `### Decision` naming the `strictness` key of `work/coverage/default.md`, the three rejected locations of D§2.1 with their reasons, and a `### Conditions to revisit`"
* R21: "a default-mode `check` of the strict-violating fixture yields no strict finding, and a counted-work tally shows zero strict units judged and zero line units counted under default; the Purpose or Explanation of `cap:workplan-strict` states that strict is opt-in through the record or the caller"

## Required user decisions (ESCALATE-TO-HUMAN)

### E1. R18: what "once the rules pass" means for this repository

The plan picks A without a quoted decision.
* **A. Baseline adoption (the plan).** Strict is enabled in this PR with an adoption `work/debt/default.md` of about 786 finding rows and 4 line rows (exact counts from T7/T8), and R18 counts as delivered. Paying the debt is not scope of this issue, so remove it from "Declined / out of scope". The plan records the user's quote, and F1, F2 and F4 still apply.
* **B. Empty debt.** "The rules pass" means without debt. T8 pays every row in this change: 196 `raf:tested-by` markers with re-assessment of the affected sections, about 590 bindings or exclusions for raf-okf and raf-source, and bindings, true exceptions or ceilings for about 10,362 orphan lines. No debt record exists at the PR head.
* **C. Partial.** Strict is enabled with debt. This change pays everything strict raises on the documents and crates it already edits:
  * the claims of `cap:workplan-strict` and of the eight re-delivered capabilities, whose Contract sections are rewritten and re-assessed anyway;
  * the orphan lines and items of the `scope` crates raf-workplan (about 168 lines) and cargo-workplan (about 26).
  The debt holds only the four untouched delivered capabilities' claims and raf-okf and raf-source.
* **D. Split.** This PR delivers the mode and the ratchet, and R18 moves to another tracked issue with its own change record (AGENTS.md:286-288). The PR then says "Refs" rather than "Closes", or the user accepts closing with R18 open.

### E2. R14: may existing test expectations change?

The plan's "breaking changes before the first release are acceptable" is unquoted and is about API breakage, not about R14.
* **A.** R14 means "every existing case keeps its verdict and no existing document's output changes". The expectations listed in F3 are updated and listed in the PR.
* **B.** R14 is literal: no existing test expectation changes. Then the `strictness`/`ceilings`/`exceptions` keys cannot join the coverage record (`schema.rs:124-127`), and the debt cannot be an OKF type (`schema.rs:194-212`, `records.rs:922-931`, and probably `documents.rs:356-377`). D§2.1 and D§3.8 must be reopened, and the JSON fallback alone does not suffice.

### E3. R5: what "touch a bound public item" means

The issue says "forbids `contract_effect: none` for changes that touch a bound public item". The plan narrows "touch" to "alters the declaration, bodies and doc comments excluded", and its check reads "body/doc edit completes". Under strict, that passes exactly the edit the literal text forbids.
* **A. Literal.** Any change to the item's span text (declaration, body, attributes or doc comments) is touched. Under strict, a body edit and a doc-comment edit refuse; an unchanged item moved within its file completes; the same edits complete under default.
* **B. Declaration-only (the plan).** The narrowing is recorded with the user's quote. The Constraints of `cap:workplan-strict` state that a body or rustdoc edit under `none` is not refused.

### E4. R9: "fail only on new ones" versus the census semantics

The plan (plan.md:52-53) makes a resolved entry fail until `debt --write`, which the issue's text does not ask for. D§7.2 calls this "a deliberate reading".
* **A. Census semantics (the plan).** `debt-resolved` fails until the write.
* **B. Literal.** Only growth fails; resolved debt passes and the next write drops it.

The plan's choice is defensible and disclosed; it needs only the user's quote.

### E5. R10: the entry point the issue names

The issue names `cargo workplan coverage --reverse`. The plan ships `cargo workplan coverage lines` and lists the issue's spelling under Declined (plan.md:105-106), so the requested input will exit 2 as a usage error.
* **A.** Ship the issue's spelling: `cargo workplan coverage --reverse [--crate <package>]` prints the D§3.9 report.
* **B.** Keep `coverage lines` under the user's quoted decision, and say in the PR body that the issue's spelling is not accepted.

## Checked and clean

* **R3.** The claim unit, the `raf:tested-by` mapping, `strict-claim-uncovered` with fix text, matrix substantiation with byte-identical existing matrices, and the `coverage claims` report are each mapped to T3 with falsifiable fixtures.
* **R6, R7, R8.** Concrete trigger and neighbour fixtures (19 vs 20 words; `dropped` with or without `reason` under both modes; untouched Usage under `none` in strict vs default).
* **R9.** The census fixture set is mapped one to one, including "two writes byte-identical" and "writes only `work/debt/default.md`"; the semantic choice is E4.
* **R11.** The ceiling boundary, the four exception forms each beside an unmatched one, and debt rows. The enforced set is F4.
* **R13.** Covered by the per-task neighbours and the `all_cases()` harness.
* **R17, R20.** Mapped.
* **R5 "already holds".** True in code: `lifecycle/complete.rs:329-336` and `lifecycle/impact.rs:148-151`. Only the check wording is F9.
* **Final gate sequence.** `make fmt clippy test test-release doc deny reuse msrv fuzz`, then `make licenses` and `make issue-refs`, equals `make check` minus `cross-check` (`Makefile:118-130`). The cross-check limit is disclosed.
* **No planned-by-refusal of a capability.** Every exit-2 path in the plan is a could-not-run case beside a working path; E5 is about the entry point's name.
* **No timing.** Reverse-coverage scale is measured by counted work (D§3.9, D§7.13).

VERDICT: ESCALATE-TO-HUMAN
