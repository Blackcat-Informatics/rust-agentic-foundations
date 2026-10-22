# Issue #16: raf-workplan strict mode: force coverage and harder gates

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-workplan, crate:cargo-workplan

## Body

## Context
The default gates accept a `delivered` capability that has at least one code path and one test binding, even when `bindings.examples` is empty, a section holds a single sentence, or most of the Contract's claims have no test behind them. The expectations of a well-documented capability (a real usage example, API references, limitations, a traceability table) are currently asserted by a test-side detector, not by the gate.

A `strict` mode turns those expectations into enforced rules for projects that want them. It is opt-in so the default stays lightweight.

## Scope
- [ ] A `strict` setting. Library: `Gate::strict()` or a `Strictness` option. CLI: a `--strict` flag plus a repo-level setting (for example a field in a config file next to `docs/capabilities/index.md`, or `Cargo.toml` metadata). Propose one location and justify it in the capability document.
- [ ] Every `delivered` capability binds at least one runnable example or doc test and has a generated verification-matrix block.
- [ ] Coverage: every Contract and Constraints claim is covered by a bound test or example. Define the claim unit (a bullet, a bold-lead paragraph, or an explicit claim marker) and the mapping from matrix rows to claims. Report unsupported claims as findings with fix text.
- [ ] Every public item of a bound crate path is explained by some capability (orphan-symbol finding), reusing the two-way coverage direction.
- [ ] `change complete` additionally requires `verify` to pass for ALL bound items of the listed capabilities and zero tense-mismatch findings on those capabilities, and forbids `contract_effect: none` for changes that touch a bound public item.
- [ ] Minimum content rules per section for delivered capabilities, reported as findings: non-trivial length, a runnable Usage example, a Limitations statement.
- [ ] No `dropped` or `moved` item outcome without a stated reason field.
- [ ] Stricter obligations: `section-untouched` also applies to Usage and Examples for `contract_effect: none`.
- [ ] A baseline/ratchet option so adopting strict on an existing repository can start from today's findings and fail only on new ones. Coordinate with the census debt-ratchet design (#4) instead of inventing a second ledger.
- [ ] Reverse coverage mode (`cargo workplan coverage --reverse`, also enforced under strict with a threshold): report the source lines of a crate that map to no deliverable or specification, for example "crate x has 100 lines that no capability binding or contract explains". A line is covered when it lies inside the span of an item that a capability binds (code, test or example) or inside an item reachable from one through the source index; everything else is an orphan line. Report per crate, per module and per item with counts and the list of orphan items and line ranges, sorted for stable output, as a text table and as JSON.
- [ ] Reverse coverage gate: a configurable ceiling (maximum orphan lines or percentage per crate), a way to mark deliberate exceptions with a reason (generated code, test support, vendored code), and the baseline/ratchet option above so the count can only go down.
- [ ] Reverse coverage fixtures: a crate with a fully bound public surface reports zero orphan lines; an unbound private helper reports exactly its line range; a bound item covers its nested items and impl blocks; macro-generated items and `#[cfg(test)]` modules are handled and documented; the report is identical across runs and platforms.

## Acceptance criteria
- [ ] Each strict rule has a fixture that triggers it and a passing neighbour.
- [ ] Default (non-strict) behaviour and every existing fixture are unchanged.
- [ ] Strict findings carry the same `Finding {kind, file, location, message, fix}` shape, with the new kinds added to the closed `KINDS` list and covered by the both-directions coverage test.
- [ ] CLI integration tests cover `--strict` exit codes 0, 1 and 2.
- [ ] Documented in the capability documents as a new designed capability (for example `cap:workplan-strict`), following the capability-document workflow in AGENTS.md.
- [ ] Once the rules pass, the repository enables strict on its own docs.

## Depends on
- Builds on #8 (change lifecycle and the `cargo-workplan` CLI).
- Overlaps with #10: the two-way coverage and evidence-claims items there cover code-to-docs and tests-to-claims, which rules 3 and 4 above enforce under strict. Scope should be reconciled between the two so neither duplicates the other.



## Comments (7)

### paudley — 2026-10-20T03:56:48Z

# Plan (Rev 2): strict mode, claim coverage, reverse coverage and a debt ratchet for workplan (issue 16)

The design, with its derivations, repository measurements and the sketches of every
document, is `design.md` in this stage directory (cited as D§n); this plan is the contract and
the task list. Where they differ the plan wins.

## Superseded annex text
Rev 1/Rev 2 override these paragraphs of `design.md`: D§3.3 (`coverage::check` reads strictness: strict findings come from `strict::check` only), D§3.4 (body and doc-comment edits keep `none`: now they do not), D§3.8 (`lines` rows per crate; `coverage-lines-over-ceiling` as a findings row; the "eligible =" list), D§3.9 ("every enforced package (`scope` + bound + rows)": every workspace package is judged), D§6.1 T4's check (body/doc complete), T8's order and its `make check`, the "every task ends with `make check` green" sentence (the gate sequence applies), D§6.2's Constraints statements on macros and `cfg(test)` and its change item 11 (see A3 below).

## Issue summary
`cargo workplan` accepts a `delivered` capability with one code binding and one test binding
even when examples are missing, sections are one sentence, or most Contract claims have no test.
Issue 16 asks for an opt-in `strict` mode: a setting (library and CLI plus a repo-level
location), enforced rules (examples/doctest and a verification matrix, claim coverage with a
defined claim unit, orphan symbols, a stricter `change complete`, minimum content per section,
a reason on `dropped`/`moved` outcomes, stricter obligations), a baseline/ratchet coordinated with
the census debt ratchet, and reverse coverage (orphan source lines, ceilings, exceptions,
baseline), documented as a new capability `cap:workplan-strict`, with the repository enabling strict
on its own docs. Acceptance: a fixture and passing neighbour per rule; default behaviour and every
existing fixture unchanged; strict findings in the `Finding` shape with new kinds in the closed list
and the both-directions coverage test; CLI exit codes 0/1/2; the capability document; strict enabled.

## Standing goals / governing decisions
CLAUDE.md and AGENTS.md: no timing runs (counted work and fixtures only; tell every agent started);
no issue/PR numbers or process wording in committed files (`make issue-refs`); no attribution; no
deferrals; no lint weakened; the capability workflow of AGENTS.md section 6 (change record, `>
[!PROPOSED change:<slug>]` blocks, `change complete`); closed lists and fail-hard; the Rust
toolchain is never installed or selected by the workflow, so `make cross-check` cannot complete on this
host (the Apple and FreeBSD targets are not in the host's single dated nightly, selected through its environment; the Makefile's Windows command is run if that target is already installed; the
hosted `cross-check`, `test-macos` and `test-windows` jobs are the evidence, disclosed in the PR);
Linux is primary; breaking changes before the first release are acceptable (user decision in this session, about API shapes). The final gate sequence is `make fmt clippy test test-release doc deny reuse msrv fuzz` in order,
then `make licenses` and `make issue-refs`, because `make check` stops at its first failure and
cannot pass here.

## Prior art
No prior attempt. The delivered precedent is the two-way coverage and evidence work (D§1.6, D§4),
the completion gate (D§1.4), and the census debt ratchet (D§3.8), whose semantics the baseline
adopts without a second ledger.

## Decisions (D§0, D§5; the lens reviews check them)
* Setting: optional key `strictness: default|strict` in the coverage record `work/coverage/default.md`
  (the repository-level policy record `check` already requires; its schema is closed); library
  `Strictness`, `Gate::strict()/strictness(floor)`, `Workplan::strictness()/with_strictness()`; CLI global
  `--strict`; effective = the stricter of record and caller (a caller can raise, never lower).
* Claims: unit = a Contract clause (bold-lead paragraph or `###` heading) / a Constraints top-level
  item, paragraph or table; mapping marker `<!-- raf:tested-by <path>[,<path>...] -->` (not an extension
  of `raf:claim`, whose existing fixtures pin the malformed forms).
* Orphan symbols: no new kind; strict widens the existing `coverage-item/test/reexport-unbound` from
  `scope` to every package a binding resolves into.
* `change complete`: "verify all bound items" and "zero tense-mismatch" already hold (D§1.4); strict adds
  the process obligation `contract-effect-understated` (a `contract_effect: none` change that alters a
  bound public item's declaration, bodies and doc comments excluded).
* Reasons: a `reason` key on a change item; `outcome-without-reason` under strict.
* Baseline: census semantics exactly (growth fails; a resolved entry also fails until rewritten; the
  write refuses growth except on adoption); a machine-written singleton `work/debt/default.md` of a new
  `Debt` document type; the rule stated once in `arch:debt-ratchet`, linked by both ratchets.
* Reverse coverage: `cargo workplan coverage lines` (the command's shape is `coverage code|tests`), covered
  = inside a bound item, below one, or reachable by resolved path references; ceilings per crate (default
  0 under strict), exceptions with reasons, debt rows; needs raf-source to resolve from tests/binaries (T5).
* Adoption: strict enabled in this change; the cheap findings fixed (13); the rest recorded as debt
  (about 786 finding rows and 4 line rows), reviewed as the adoption control. Declared plainly: this
  is a recorded baseline under the issue's own ratchet item, not a hidden deferral, and the debt diff is
  part of the PR.
* The `Debt` type changes pinned expectations (four existing tests, listed in Rev 1 E2; the earlier 'one pin' was an undercount). "Every existing fixture unchanged" is read as unchanged behaviour: the pin is updated, and
  the PR says so. (Breaking changes before release are acceptable; the fallback of a JSON debt file is
  declined.)

## Completeness contract (ledger R1-R21)
| R | Requirement | Task | Executable check |
|---|---|---|---|
| R1 | `strict` setting: library, CLI, repo-level location justified in the capability | 2, 8 | D§2 tests of T2: record `strict` beats a default floor, a strict floor beats a default record, `Opened::workplan_after` keeps the floor, `okf-bad-strictness` vs `strict`/`default`, `--strict` flag; the location justification text in `cap:workplan-strict` |
| R2 | every delivered capability binds a runnable example/doctest and has a verification-matrix block | 2 | `strict-example-missing` vs one example; `strict-matrix-missing` vs the block |
| R3 | claim coverage with a defined unit and the matrix-row mapping; unsupported claims reported with fix text | 3 | marker fixtures (well-formed, malformed, unattached), unit fixtures, `strict-claim-uncovered`/`claim-test-unbound` vs covered neighbour, the matrix lists a marked test under Contract and existing matrices byte-identical; `coverage claims` report text+JSON |
| R4 | every public item of a bound crate path explained by some capability | 2 | a bound package outside `scope` with an unbound public item is enforced vs bound; the same package unbound by every capability is not enforced (stated limit) |
| R5 | `change complete` requires verify for ALL bound items, zero tense-mismatch, forbids `contract_effect: none` over a bound public item | 4 | git fixture repositories: signature edit refuses under strict and completes under default; body/doc edit completes; new bound public item refuses; moved-within-file completes; the existing conditions' fixtures unchanged |
| R6 | minimum content rules per delivered section | 2 | `strict-section-thin` 19 vs 20 words; `strict-usage-unbound` vs a code span of a bound doc test; `strict-limitations-missing` vs one list item |
| R7 | no `dropped`/`moved` without a reason | 4 | `outcome-without-reason` under strict, completes with `reason`, completes without it under default |
| R8 | `section-untouched` also for Usage and Examples under `contract_effect: none` | 4 | untouched Usage in `affects` of a `none` change refuses under strict, completes under default |
| R9 | baseline/ratchet coordinated with the census ratchet, no second ledger | 7 | census fixture set mapped one to one (growth fails and the write refuses; shrink fails with `debt-resolved` until written; no record: all strict findings stand and the write creates it; two writes byte-identical; writes only `work/debt/default.md`); `arch:debt-ratchet` linked by both |
| R10 | reverse coverage report: orphan lines per crate/module/item, counts, ranges, sorted, text and JSON | 5, 6 | the R12 fixtures; report identical across runs |
| R11 | gate: ceiling, exceptions with reason, baseline | 6, 7 | ceiling boundary, exceptions by item/prefix/file/directory each beside an unmatched one (`coverage-exception-unmatched`), `coverage-lines-over-ceiling`, debt rows |
| R12 | reverse-coverage fixtures: fully bound = 0 orphan lines; unbound private helper = exactly its range; bound item covers nested items/impl blocks; macro items and `#[cfg(test)]` handled and documented; identical across runs and platforms | 5, 6 | those fixtures by name; resolution from tests/binaries and `impls()` (T5) |
| R13 | each strict rule has a fixture that triggers and a passing neighbour | 2-7 | per-task checks above |
| R14 | default behaviour and every existing fixture unchanged | all | `all_cases()` as today; strict-violating fixtures yield no finding without strict; the one schema pin updated and declared |
| R15 | strict findings use `Finding {kind,file,location,message,fix}`; new kinds in the closed `KINDS` list and the both-directions coverage test | 2-7 | the existing both-directions test covers every new kind |
| R16 | CLI integration tests for exit codes 0, 1, 2 | 2, 3, 4, 6, 7 | `cargo-workplan/tests/*` per command |
| R17 | documented as a new designed capability `cap:workplan-strict` per the capability workflow | 1, 8 | change record, capability document `designed` at T1, `delivered` at T8 with `change complete` |
| R18 | once the rules pass, the repository enables strict on its own docs | 8 | `strictness: strict` set; `cargo workplan --strict check` exits 0 with the recorded debt; `self_workplan` judges strict |
| R19 | reconcile with the #10 work | 1 | D§4 recorded; no duplicated rule (orphan-symbol reuses the delivered kinds) |
| R20 | builds on the lifecycle and CLI (delivered) | - | satisfied |
| R21 | opt-in; the default stays lightweight | 2 | default fixtures unchanged; documented |

## Tasks (each: implement, independent review by a fresh reviewer, commit, push, issue update)
T1-T8 are those of D§6.1 (T1 change record and design documents; T2 setting, plumbing and the
document rules; T3 claim coverage; T4 change lifecycle under strict; T5 raf-source resolution from
every target and impl blocks; T6 reverse coverage; T7 the debt record and ratchet; T8 adopt strict in
this repository and deliver), with their focused checks; each task also runs `cargo test --locked --test
docs_contract --test self_workplan --test repo_gates`, `make fmt clippy doc deny reuse issue-refs`,
`cargo workplan check`, keeps every assessment current and accepted after reading its context, and
binds every new public item and test. Commit types: `feat(raf-workplan)`, `feat(raf-okf)`,
`feat(cargo-workplan)`, `feat(raf-source)` (T5), `docs(...)` (T1, T8). The PR task (after T8):
final sequence above, PR "Closes #16", the plan posted, the debt diff summarised, disclosures
(the cross-check limit, the schema pin), least-confident note.

## Rev 1: integration of the five lens reports (authoritative over anything above that differs)

### Decisions on the adversary's five escalations (each is the plan's recommendation; E1 is gated at merge)
* **E1 (R18, what "once the rules pass" means): adopt with a recorded baseline, gated on maintainer confirmation before merge.** The issue's own ratchet item says adoption "can start from today's findings and fail only on new ones", so a debt record is a designed outcome, not a hidden cut. To make it honest: (1) the debt is computed on the BASE and nothing else: a T8 check proves every debt row is also raised by `--strict check` on an unmodified checkout of the merge base, line counts are no higher than the base's, and any finding this change introduces is FIXED, never recorded; (2) the debt of the capabilities and packages this change edits (the nine capabilities of T1 and the `scope` crates) is paid in this change where the cost is a sentence or a marker (the 12 usage findings, the thin section, and the claim markers of `cap:workplan-strict` and of the sections this change rewrites); (3) a tracked follow-up issue is filed with the counts per rule to pay the remainder, named in the PR; (4) the PR states the debt counts and says, in its first paragraph, that merge needs the maintainer's confirmation that a recorded baseline satisfies "once the rules pass"; I will not merge #16 until the user confirms. Alternatives A-D of the report stay visible in the PR.
* **E2 (R14): existing fixtures.** At least four existing expectations change as a direct consequence of a new document type and new coverage-record keys: `crates/raf-okf/tests/schema.rs` (the coverage-record key list, the singleton/records test, the type list), `crates/raf-okf/tests/records.rs` (the type message) and, if `work/debt` is loaded, `crates/raf-workplan/tests/documents.rs` (the load directories). The plan lists them in T7, updates them, and the PR lists each with the reason. R14 is read as: no existing BEHAVIOUR changes (every `all_cases()` fixture and every default-mode finding stays identical). The JSON debt-file fallback is declined: it would still change the document inventory and would create a second kind of record.
* **E3 (R5): "touch a bound public item" is read literally**: `contract-effect-understated` is raised when a change declared `contract_effect: none` alters ANY part of a bound public item's span (declaration, body or doc comment), removes or renames it, or alters an item covered "below" a bound one (enhancement item). T4's fixtures are rewritten accordingly (a body-only edit now refuses under strict and completes under default; a comment outside any bound item completes).
* **E4 (R9): census semantics** are kept (growth fails; a resolved entry also fails until `debt --write` is rerun), the issue's "fail only on new ones" being the floor and coordination with the census ratchet being the stated requirement; the PR names the difference.
* **E5 (R10): `cargo workplan coverage --reverse` is accepted as an alias of `coverage lines`** with byte-identical output (a test compares them); `lines` is the primary spelling; nothing is declined.

### Blocking findings fixed
* Placement of strict findings (design B1/B2): `relate`/`coverage::check` stay strictness-free. `strict::check` is the SOLE producer of strict-only findings and of `Owed` rows: it computes the widened orphan-symbol coverage itself with `coverage::compute` over the set of bound packages; `debt::apply` consumes only `Owed`, never message text. The R4 fixture is reachable through `Case::strict()` with `Strictness` (not a bool).
* Debt keys (design B3): `DebtKey::{Finding{kind,subject}, Lines{item}}`; `coverage-lines-over-ceiling` is not eligible as a `findings` row (`debt-entry-invalid`). Per the enhancement item, line debt is recorded PER ITEM (crate, module path, item, range) so one orphan cannot be traded for another; a per-crate count is derived.
* Recorded decision (constraints B1): T1/T7 rewrite the decision page `docs/architecture/okf-profile.md` from six document types to seven, update its type and key tables (`Debt`, the `strictness`/`ceilings`/`exceptions` coverage-record keys, the item `reason` key, the `raf:tested-by` marker) and replace its revisit condition, which this change meets.
* T8 order (adversary F2): (a) `cargo workplan change complete workplan-strict` first (strictness still default, the nine capabilities delivered); (b) in the following commit set `strictness: strict` in `work/coverage/default.md`, add the true exceptions and run `debt --write`; the write is accepted at adoption because no record existed; then `self_workplan` judges strict. `state-delivered-open-record` cannot arise because the record is closed first. A check proves the debt equals the base-checkout findings (E1(1)).
* Reverse coverage covers EVERY package of the workspace the loader reaches (including the root package and packages no capability binds), not only `scope` and bound packages (adversary F4); a package the loader cannot reach (the separate fuzz workspace) is listed as an exception with a reason; a fixture has an unbound crate with an orphan helper.
* Structure: the plan now has numbered tasks below, with the PR task numbered T9 and ending with handoff; the design's "every task ends with `make check` green" is superseded by the gate sequence (the Rust toolchain is never installed or selected; `make cross-check` cannot complete here because the Apple and FreeBSD targets are not in the host's single toolchain, the Makefile's own command for the Windows target is run after T5 and at the end if that target is already installed, the hosted `cross-check`, `test-macos` and `test-windows` jobs are the evidence; if `rust-validate` is missing `make msrv` is reported as FAILED, not worked around; the PR template's `make check` box stays unticked).

### Completeness contract amendments (rows below replace the earlier rows of the same R)
| R | Amended executable check |
|---|---|
| R1 | the Explanation of `cap:workplan-strict` holds a `### Decision` naming the `strictness` key of `work/coverage/default.md`, the three rejected locations (a file under `docs/capabilities`, `Cargo.toml` metadata, a new config file) with their reasons, and a `### Conditions to revisit`; plus the T2 behaviour tests |
| R2 | `strict-example-missing` vs one example; a delivered capability whose only example binding is a `no_run` documentation test is refused by `cargo workplan verify` and by condition 5 (the existing `no_run` fixture of `raf-workplan/tests/bound.rs`), beside a runnable one that passes; `strict-matrix-missing` vs the block |
| R5 | strict fixtures for the conditions that already hold by default (verify all bound items; zero tense-mismatch) refuse and complete under strict exactly as under default; E3's literal `contract-effect-understated` fixtures; the existing conditions' fixtures unchanged |
| R10 | `cargo workplan coverage lines --crate <fixture>` on the R12 fixtures prints three tables: the crate table (counted, covered, excepted, orphan, shipping/test orphan, ceiling, debt), the module table, and the item table with merged `a-b` ranges sorted by package, module path, file and first line; `--format json` prints `lines-crate`, `lines-module` and `lines-orphan` objects (`ranges` as `[[a,b],...]`) with the same rows; text and JSON each equal a literal expected string; two runs byte-identical; `--reverse` output equals `lines` output; exit 1 when an orphan line is printed, 0 when none |
| R12 | the capability's Limitations and Explanation state how macro-generated items (counted as the span of the invoking item; unreachable expansions are excepted by reason) and `#[cfg(test)]` modules (counted as test lines, reported in their own column, enforced under their own ceiling) are handled, with fixtures that pin each statement; platform identity: the report contains no absolute path or host-dependent order and the fixtures compare it with literal strings (the hosted macOS and Windows jobs run them) |
| R15 | the twelve own kinds and the new `okf-*`/`okf-parse-*` kinds are in `Finding::KINDS` and each has a named `Case` in `kinds.rs::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` (`OWN_KINDS` 48 to 60); the two obligation kinds are in `ObligationKind::ALL` and covered by `complete.rs::the_cases_cover_every_kind_of_obligation_in_both_directions` and `each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes`; existing kinds that gain triggers (`scope-crate-missing` from `ceilings`, `okf-empty-reason` from `exceptions`, the debt record's singleton/row rules) get their own failing fixtures; the JSON objects of strict findings carry `kind`, `path`, `section`, `line`, `key`, `message`, `fix` |
| R16 | `cargo-workplan/tests/check.rs`: `cargo workplan --strict check` exits 0 on the strict-clean fixture, 1 with one thin section (0 there without `--strict`), 2 with `--root` at a directory without `docs/capabilities`; `cargo workplan --strict change reconcile <slug>` exits 1 for a `none` change with an untouched Usage, 0 after the edit, 2 for a merge-base blob that does not parse; `coverage claims` and `coverage lines` exit 0/1/2 (2: unknown `--capability`/`--crate`); `debt --write` exits 0 (written), 1 (growth), 2 (record not strict) |
| R18 | E1 above: `strictness: strict` set in a commit after `change complete`; `cargo workplan --strict check` exits 0; the debt check against the base checkout passes; `.deficiencies` holds only its notice and marker (checked before the PR); the PR carries the maintainer-confirmation line |
| R19 | the Explanation of `cap:workplan-strict` states the reconciliation with `cap:workplan-two-way-coverage` and `cap:workplan-evidence-claims` as prose; the Contract of `cap:workplan-two-way-coverage` gains the one sentence of the design's section 4; `OWN_KINDS` gains no kind for orphan symbols |
| R21 | default fixtures unchanged and a documented sentence in the capability that default is the lightweight mode |

### Enhancement dispositions
Adopted: reachability as a raf-source product (T5 builds item-to-item reachability, contains / impl-of / references, with the bindings as the root set, and `coverage lines` is a closure query over it; the same graph serves `contract-effect-understated`'s "covered below"); `Strictness` stays a closed enum with the preset semantics documented (a per-rule set is declined: no consumer, cost: a repository cannot enable one rule alone); per-item line debt (above); the single-claim collapse (a Contract with no bold lead or `###` is ONE claim, reported as `strict-claim-unit-collapsed` when it is longer than the minimum so claim coverage is not vacuous: a new strict kind, with fixture and neighbour). Declined, with cost: an `explain <path>` provenance command, test selection and the census consumer of the graph (not asked for; cost: the graph is built but only two consumers use it); the claim-key renumbering is documented in the capability instead of avoided.

### Tasks (numbered; each: implement, independent review by a fresh reviewer, commit, push, issue update)
* **T1** change record, capability `cap:workplan-strict` (`designed`), the decision pages (`arch:debt-ratchet`; `okf-profile` rewritten to seven types), PROPOSED blocks in the eight capabilities of D§6.1 T1, INVENTORY, CHANGELOG entry naming the breaking changes, command lists in AGENTS.md and README, the `work/debt/` layout row; `make hooks` verified.
* **T2** setting, plumbing, document rules (R1, R2, R4, R6, R21).
* **T3** claim coverage (R3).
* **T4** change lifecycle under strict (R5, R7, R8).
* **T5** raf-source: resolution from every target, impl blocks, item reachability.
* **T6** reverse coverage (R10-R12), `--reverse` alias.
* **T7** debt record and ratchet (R9), the four fixture updates of E2.
* **T8** `change complete`, then adoption in a following commit with the base-checkout debt check; pay the debt of the edited capabilities and packages; file the follow-up issue.
* **T9** handoff: the final gate sequence, `.deficiencies` check, PR "Closes #16" with the maintainer-confirmation paragraph, the debt summary, the plan posted, the disclosures (cross-check limit, E2 fixtures), the least-confident note, the follow-up issue; then stop and wait for the maintainer's confirmation before Stage 3.
Common to every task: `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`, `make fmt clippy doc deny reuse issue-refs`, `cargo workplan check`, assessments read and accepted, every new public item and test bound, no tracker numbers or process wording. Focused checks are those of D§6.1 as amended above.

### Not scope of this change (tracked elsewhere)
Paying the debt that this change does not edit is moved to a tracked follow-up issue (not a cut: the issue's own baseline item defines adoption from today's findings), and the PR asks the maintainer to confirm that reading.

## Rev 2: fixes to the final reviewer's findings (authoritative)
* **R18 check (B1):** `strictness: strict` is set in the commit after `change complete`; at the PR head `cargo workplan check` with NO `--strict` flag exits 0; the T2 CLI fixture shows the same no-flag `check` exits 1 on a record that says `strictness: strict` with one thin section; `tests/self_workplan.rs` asserts `workplan.strictness() == Strictness::Strict` on this repository and checks under that strictness; the base-checkout debt check passes; `.deficiencies` holds only its notice and marker; the PR carries the maintainer-confirmation paragraph.
* **R15 (B2):** thirteen own kinds (the twelve of D§3.10 plus `strict-claim-unit-collapsed`, triggered by a Contract with no clause start and more than `MIN_SECTION_WORDS` words) and the new `okf-*`/`okf-parse-*` kinds are in `Finding::KINDS`; each has a named `Case` in `kinds.rs::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` (`OWN_KINDS` 48 to 61).
* **Fuzz (B3):** the fuzz harness is its own cargo workspace, excluded from the root one, so it has no lines in the index and is not judged. The Constraints of `cap:workplan-strict` state this limit with a `raf:tested-by` marker naming a fixture whose workspace excludes a directory of Rust files, which reports no row for it. No exception row names it (an exception that matches nothing is `coverage-exception-unmatched`).
* **Dispositions (B4):** design A2 adopted in T6 (compare `orphan * 100 * scale <= percent_scaled * counted` without division; fixture: an empty crate under a percent ceiling); design A1 moot under E3; design A4 adopted at T8 (the inventory's `matrix_violations` and `a_delivered_capability_holds_a_verification_matrix` are removed because `strict-matrix-missing` enforces the rule once the repository is strict; cost: the rule depends on the record); the R8 Examples neighbour is a named T4 fixture (untouched Examples in `affects` of a `none` change refuses under strict and completes under default); enhancement items 3-9 and 13-15: items that extend beyond the issue (new commands, provenance, selection, extra presets) are declined with the cost that those consumers stay unbuilt, items on determinism and sorting are adopted in T6.
* **E3 whitespace (A2):** items are compared as `proc_macro2` token streams of their span (doc comments arrive as `#[doc]` attributes so they count; whitespace and non-doc comments do not), each side parsed from its whole file through `raf_source::parse_source`. Fixtures: a reformatted body completes; a changed attribute refuses.
* **Change record (A3):** it carries no item that only the adoption step makes true: D§6.2 item 11 is reworded to the product guarantee "a repository whose coverage record says `strictness: strict` is judged strict without a flag".
* **Line debt (A4):** a `lines` row is exactly `item` (crate-qualified item path) and `orphans` (the count); ranges are reported by `coverage lines` and never recorded; a row's count below its record is `debt-resolved`, above it is `coverage-lines-over-ceiling` naming the row, and a present orphan item with no row is growth. Line debt is a count only.
* **Scope crates (A5):** the orphan lines of raf-workplan and cargo-workplan are bound, reached or excepted with a reason in T8; no `lines` row names a `scope` crate.
* **Ceiling classes (A6):** a ceiling row takes an optional `of: shipping|test|all` (default `all`); fixtures: one per class at its boundary; `okf-bad-ceiling` for any other value; enhancement item 7 adopted in T6.
* **Declined enhancement item 2 (A7):** eligibility by construction, its main payoff, is already given by `strict::check` being the only producer of `Owed`, so a rule set would add a second closed list with no consumer today; the capability states the condition to revisit.
* **Heading (A8):** the closing section is "Not scope of this change (tracked elsewhere)": the debt of the packages and capabilities this change does not edit is a tracked follow-up issue named in the PR, with counts per rule.
* **R21 (A9):** default-mode cost is observed: under default, zero strict units are judged and zero line units counted (counted-work tallies pinned by a test).
* **Open decision to put to the maintainer in the PR (the final reviewer lists them):** E1 (recorded baseline satisfies "once the rules pass"), E4 (census semantics instead of "fail only on new ones"), E2 (the four fixture updates). The PR asks; merge waits.
* **Residual advisories (final reviewer, pass 2):** Rev 2 text overrides every earlier line that contradicts it (the Rev 0 Tasks paragraph and the lines the "Superseded annex text" note lists are superseded, not deleted, so the history of the review stays readable). Enhancement items 4, 5, 13 and 15 are assigned: 4 and 5 declined (extra commands, cost: unbuilt consumers), 13 declined (cost: no provenance view), 15 adopted in T3 (the heading-marker fixture: a `###` clause's statement is its heading line, and a `raf:tested-by` marker after the heading covers the clause). A package whose only ceiling row is a class row (`of: shipping` or `of: test`) still has the `all: 0` default under strict for the classes it names no row for; a fixture pins it.


### paudley — 2026-10-20T08:00:28Z

Scope decision from the maintainer: making this repository work under strict is a non-goal for this issue (the acceptance item "Once the rules pass, the repository enables strict on its own docs" moves to its own low-priority issue). The strict mode, its rules, claim coverage, reverse coverage and the debt ratchet are built and proven on fixture repositories here.

### paudley — 2026-10-20T20:35:43Z

T6 (reverse coverage: coverage lines, the --reverse alias, ceilings with of: shipping|test|all, exceptions) committed as ff8b258; independent review BLOCKED (unused import broke -D warnings, 12 stale assessments, unpinned excepted/error tables and JSON kinds, no CLI ceiling/exception tests, nested-workspace constraint unpinned); fixed in fec6c21 (pushed); recheck PASS. Nested workspaces reached by the source roots are judged as packages (pinned by a fixture). Checks: cargo test -p raf-workplan -p cargo-workplan, docs_contract/self_workplan/repo_gates, make fmt clippy doc deny reuse issue-refs, cargo workplan check: all exit 0.


### paudley — 2026-10-20T23:05:25Z

The repository's standing goals (RDF 1.2 only, wasmable core, greenfield, no build optionality, maximal utility and performance, Rust-first with a Python surface) bear on this issue.

**New reports must not add JSON (goal: RDF 1.2 only).**
- Finding: the strict-mode work (open PR) adds three more JSON outputs: `coverage claims`, `coverage lines`/`--reverse`, and `debt --write` printing a `debt-written` JSON kind. Evidence: `crates/cargo-workplan/src/render.rs` (`claims_json`, `lines_json`), `crates/cargo-workplan/src/command.rs`, `docs/capabilities/workplan-cli.md:207,210,250,448,726,751` on branch `16-*`. The scope text of this issue also asks for "a text table and as JSON".
- Direction: land the three reports with an RDF 1.2 serialization from the start, or hold them behind the vocabulary issue #40, so the JSON surface stops growing.

**Opt-in strictness (goals: no build optionality, maximal utility).**
- Finding: strict mode is opt-in ("so the default stays lightweight"); a `strictness` key or `--strict` flag chooses it. Evidence: this issue's body; `work/coverage/default.md` and `crates/cargo-workplan/src/cli.rs` (`--strict`) on the branch.
- Direction: the strict rules should be the default, with the baseline/debt record as the adoption path, so there is one mode and no flag. Keep the ratchet design (debt only shrinks) and drop the lightweight default. #37 then becomes the requirement that this repository satisfies the one mode, not a later opt-in.


### paudley — 2026-10-20T23:17:19Z

T7 (the debt record type, the census-semantics ratchet, debt --write; built and proven on fixture repositories; this repository does not adopt strict, by the scope decision) committed as 4449ceb with review fix dc1115b, pushed. Review BLOCKED (a broken doc heading, one debt kind not written through, the completion check with a debt record unproven), fixed, recheck PASS. E2 fixture updates are listed with reasons in the stage validation index. Checks: cargo test -p raf-okf -p raf-workplan -p cargo-workplan, docs_contract/self_workplan/repo_gates, make fmt clippy doc deny reuse issue-refs, cargo workplan check: all exit 0.


### paudley — 2026-10-21T01:47:58Z

Portfolio assessment: the implemented strict rules, claim coverage, reverse coverage and debt ratchet through T7 are useful and will be reused. Existing reviews cover the older opt-in/JSON design. The current dirty documentation/finalization changes have no final qualification, and the change-completion transaction remains unfinished.

The queue preserves the newer requirements in this issue's comments: strict rules become the default, obsolete lightweight/flag interfaces are removed, and the three new result reports use RDF 1.2. The intended delivery groups this work with the shared vocabulary/result-format change in #40 so production CLI renderers and documentation are changed coherently. No existing command or acceptance criterion is dropped by that grouping.

Repository adoption remains the distinct contract of #37, per the explicit earlier scope decision. Fixture-only proof of the strict implementation will not be counted as RAF satisfying strict mode. The next prerequisite is the census RDF report foundation already being completed in #3; then the shared workplan result vocabulary and revised strict semantics can be qualified together.


### paudley — 2026-10-21T03:56:05Z

The delivery queue now groups #40 + #16 + #37 in one implementation/landing unit, with separate acceptance for each. The existing strict checkout and PR #38 remain the surviving implementation. Historical opt-in/JSON reviews are useful evidence for their tested scope; they do not qualify the revised sole-default/RDF contract.

The dependency is concrete: after strict becomes the only policy, RAF's existing self-workplan gate immediately judges the actual repository adoption findings. The default flip therefore cannot land with adoption unpaid, a hidden lightweight mode, broad exclusions, or a baseline presented as zero debt. We will inventory the integrated repository, repair the first-party documentation/bindings/claims/source coverage, and complete the actual lifecycle before the flip lands. #37 retains its distinct zero-debt target and shrinking ratchet acceptance.

Machine results will use the shared IRI-versioned RDF 1.2 vocabulary, canonical N-Triples/Turtle and named triple-term provenance across every workplan result command. JSON format rejection and dependency/manifest gates remain explicit. Current census report profiles will migrate explicitly to the common vocabulary/identity law; unchanged fingerprint hash bytes do not require a normal-form bump.

This grouping does not close any issue administratively. In particular, #40's remaining census debt-record behavior stays mapped to #4, and the census CLI/protocol in #6/#12 must use the same vocabulary. The #40 parent remains open until its complete census acceptance is proved. Likewise, #3 remains open through the shared full-core WASM acceptance in #46 even after its coherent foundation lands.

No third implementation lane has started. Census integration/qualification and resolver correctness are the two active lanes; the shared strict/RDF delivery is prepared for activation after census lands.


