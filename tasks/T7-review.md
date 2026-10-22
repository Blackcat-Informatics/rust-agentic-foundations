VERDICT: BLOCKED

# T7 review: commit 4449ceb (the debt record and the ratchet, R9, and the E2 fixture updates)

The ratchet is right. I drove `debt --write` through the real binary on a fixture repository and
found no defect in the census semantics, the exit codes, the byte-determinism or the Debt column.
Three small things block it. A one-character edit turned a rustdoc heading into a plain comment.
The write-through test omits a debt kind that a fixture can raise, while its name and the evidence
say "every kind". The one PROPOSED block that is left in `workplan-complete-gate` states behaviour
that is built but proven by no fixture, and nothing in the plan will ever prove it.

## Scope reviewed

* `git show 4449ceb` (71 files), in particular `crates/raf-workplan/src/debt.rs` (769 lines, read
  end to end) and `debt/tests.rs`, the `strict.rs`, `strict/claims.rs`, `strict/content.rs`,
  `coverage/lines.rs`, `gate.rs`, `model.rs`, `record.rs`, `spdx.rs`, `history.rs` hunks,
  `cargo-workplan/src/{cli,debt,command,main}.rs`, and the raf-okf profile diff.
* Tests: `raf-workplan/tests/debt.rs` (names and the subject table, the all-kinds test),
  `support/cases.rs` (`debt_cases`), `kinds.rs`, `documents.rs`, `cargo-workplan/tests/debt.rs`
  and `json.rs`, and the raf-okf `schema.rs`, `render.rs`, `records.rs`, `profile.rs` edits.
* Docs: `workplan-strict.md` (Debt paragraph, Constraint, bindings), `debt-ratchet.md`,
  `architecture/okf-profile.md`, `capabilities/okf-profile.md`, `workplan-document-model.md`,
  `workplan-cli.md`, the remaining PROPOSED blocks, CHANGELOG.
* Plan: Rev 1 E2, Rev 2 "Line debt (A4)", Rev 3 scope; design 3.8, 3.10, T7. Evidence:
  "Validation: T7".

I changed nothing in the worktree. I worked in the clone `scratch/t7review` at 4449ceb and in a
throwaway fixture repository under `/tmp/t7probe`. Logs are in `scratch/t7-logs/`. No timing runs,
no mutation passes.

## Commands and exit status

| Command | Result |
| --- | --- |
| `cargo test -p raf-okf -p raf-workplan -p cargo-workplan --locked` | exit 0, 1591 passed, 0 failed |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | exit 0, 31 / 1 / 68 passed |
| `make clippy` | exit 0 |
| `cargo run --locked -q -p cargo-workplan -- check` | exit 0 |
| `git ls-files work/debt`, `ls work/debt` in the worktree | absent, as Rev 3 requires |
| Binary probes on a fixture (below) | as designed |

Binary probes (built `cargo-workplan`, a git-tracked demo crate with a delivered capability and
`strictness: strict`):

* No `--write`: exit 2 (clap). Default strictness: `check` gives `debt-without-strict`;
  `debt --write` exit 2 with the NotStrict message; `--strict debt --write` writes.
* Index missing (a default finding): `debt --write -q` prints the finding, exit 1, writes nothing.
* After `docs sync` and `git add`: `debt --write --copyright ... --licence ...` exit 0,
  `created: work/debt/default.md (14 finding(s), 1 item(s) ...)`, `check` then exit 0. The file is
  sorted, SPDX lines present. A second write exit 0 `unchanged`, bytes identical; JSON is one
  `debt-written` object.
* Growth (new public fn): `check` exit 1 with `coverage-item-unbound` and a
  `coverage-lines-over-ceiling` at `lines` naming the missing row; `debt --write` exit 1, record
  untouched.
* Shrinkage (fn removed): `check` exit 1 with two `debt-resolved` (findings and lines); `debt --write`
  exit 0 `replaced`, row gone, `check` exit 0.
* A second file in `work/debt`: `singleton-other-file`, write exit 1. A `lines` row of 0 orphans:
  `debt-entry-invalid`, and the write replaces it.
* A macro-generated unresolved re-export (`coverage-reexport-unbound`, subject `demo::Made`):
  against an existing record it is growth (exit 1); after the record is removed the write adopts
  it and `check` exits 0. The round trip works.

## Answers to the questions asked

* **Census semantics.** Both directions fail until rewritten (`debt-resolved` for a paid finding,
  a falling count, a gone item, a package back within its ceilings; the present finding or a count
  over its row for growth). `Opened::write_debt` refuses with `Findings` for anything no row can
  hold and with `Growth` when a record exists; with no record it adopts. Verified in code
  (`debt.rs:221-273`, `434-538`, `704-752`) and by the probes. `judged.growth` is fed from every
  place a present finding is not held, including a package newly over a ceiling, so the write
  cannot record growth silently. Sound.
* **Kinds with a negative fixture and a valid neighbour.** `debt-without-strict`, `debt-resolved`,
  `debt-entry-invalid` each have a named `Case` (`cases.rs`, `debt_cases`) and `OWN_KINDS` is 61.
  Neighbours: the record that equals the present debt passes (`a_paid_finding_fails_...`), the
  strict repository has no `debt-without-strict`, and the subject table pairs a fitting subject
  with a non-fitting one for all ten kinds, plus six kinds that are no debt.
* **E2.** The fixture changes are exactly the consequences of a seventh type: key lists, type
  lists, `DocType::listed`, the load directories (seven), the `docs_contract` directory list, the
  render fixture array 6 to 7, `OWN_KINDS` 58 to 61, `debt-written` in the JSON non-rule list.
  `debt_cases` only adds cases; no `all_cases()` fixture or default-mode finding changed, and the
  full suite passes. The one `history.rs` edit makes `Debt` behave as `Coverage` (UnknownKind).
* **CrateLines::debt and the Debt column.** Wired through `hold_debt` (`coverage/lines.rs`); the
  probe printed `| 0 lines | 0 |`, and the tests assert 3, 0 and 2 plus the binary table.
* **Docs.** The Debt paragraph of `cap:workplan-strict` matches the code sentence by sentence
  (checked against `debt.rs`: judged by rows, row-less package judged by ceilings, the eligible
  kinds, `debt-entry-invalid` locations, the error set, exit codes, `Unchanged`). The folded
  PROPOSED blocks (okf-profile, workplan-cli twice, document-model three times) now state what
  the code does; I found no stale "six types", "three records" or "no debt record" text.
  `debt-ratchet.md` and the Constraint about the missing version case are true.
* **Assessment verdicts.** I sampled by comparing the changed digests with the changed code:
  `workplan-entity-history-*` (only the `raf_workplan::history` symbol digest moved; its text is
  untouched and the `Coverage | Debt` arm mirrors coverage), `workplan-two-way-coverage-*` (only the
  `raf_workplan::coverage` digest moved; the page says nothing a debt record falsifies),
  `okf-profile-contract` and `workplan-document-model-contract` (the sections whose text was
  fixed). All sound. All 24 show 0 stale (`self_workplan` passes).
* **`coverage-reexport-unbound` covered only by a subject-form table.** Not adequate, see R2. The
  behaviour is correct (probed), but the claim in the evidence that a fixture cannot raise it is
  false.
* **No issue/PR numbers or process wording** in the diff (grepped added lines); no
  `work/debt/default.md`, no `strictness: strict` in this repository.

## Findings

### R1. A rustdoc heading was turned into a plain comment (required)

`crates/raf-workplan/src/strict.rs:19` reads `////! # Examples, the matrix and the sections`. The
commit's new paragraph was inserted above it and the line gained two slashes. Four slashes is not
a doc comment, so the heading and the line vanish from the `raf_workplan::strict` page and the
bullets below it run into the previous section. Nothing in the gates catches it (clippy, fmt,
`make doc` and the contract tests all pass).

Required fix: restore `//! # Examples, the matrix and the sections`. Consider a one-line check
only if a cheap one exists; otherwise the restore is enough.

### R2. The write-through test omits a kind that a fixture can raise (required)

`crates/raf-workplan/tests/debt.rs:182-241`, `every_kind_the_rules_raise_is_written_with_a_subject_of_its_form_and_then_holds`,
asserts nine kinds, and the evidence says "the nine kinds a fixture can raise".
`coverage-reexport-unbound` can be raised by a fixture: `cases.rs:1478-1491` raises it
(a `macro_rules!` item behind `pub use made::Made;`). Its debt row is therefore checked only for
its subject form (`debt.rs:448`), never written, never read back as held. I probed that path by
hand and it works, but nothing pins the subject the writer produces (`gap.path()`, here
`demo::Made`) against `subject_fits`. If `Gap::path` for a glob (`<module>::*`) or a rename ever
changed shape, the writer would write a row that its own reader calls `debt-entry-invalid`.

Required fix: add the unresolved re-export to the repository of that test (or a second small
fixture) so the written record contains `coverage-reexport-unbound <path>` and then holds, and
correct the test's claim and the Validation text so they say "ten" with the one kind a row cannot
hold (`coverage-surface-unreadable`) named. Keep the subject-form table.

### R3. The remaining PROPOSED block of `workplan-complete-gate` is built but unproven (required)

`docs/capabilities/workplan-complete-gate.md:381-384` says the check after the change record is
removed applies the strict debt record, so a held finding does not refuse completion and a
resolved row does. The code does it: `lifecycle/complete.rs:377` and `:382` call
`workplan_after(...).check()`, which runs `debt::judge` under the gate's strictness floor. So the
claim is true, not unbuilt. But no test completes a change in a repository that has a debt record
(`grep debt tests/complete.rs` is empty), and T8 does not add one: the repository itself has no
debt record (Rev 3), so `change complete workplan-strict` can never exercise it. The block would
therefore stay PROPOSED forever, or be folded without proof.

Required fix: add one `complete` fixture in `crates/raf-workplan/tests/complete.rs` (strict
repository with a debt record holding a finding that completes, and the same with a resolved row
that refuses with `debt-resolved`), bind it in `cap:workplan-complete-gate` and fold the block
into current text. If the maintainer prefers to leave it, the Validation text must say the claim
is built and unproven, not "no fixture proves it yet, so the block stays for the completion task",
because no later task completes it.

## Non-blocking observations

* `hold_debt` (`coverage/lines.rs`) fills the Debt column for any item that has a row, even when
  its package is within its ceilings (a stale row that `check` calls `debt-resolved`). The doc
  says "the fewer of the lines of each item and its row", so this matches the text.
* "Rows sorted" is a property of the writer and not a rule of the reader (design said a rule).
  The deviation is documented in the profile page and costs nothing: a hand-ordered record reads
  and the next write replaces it. Accepted.
* `debt-entry-invalid` is raised under either strictness, where design 3.10 said strict only. The
  docs state the delivered behaviour. Accepted.
* `history` of `debt:default` returns `UnknownKind`, like `coverage:default`, and no test names it.
  Same as the existing coverage case.
