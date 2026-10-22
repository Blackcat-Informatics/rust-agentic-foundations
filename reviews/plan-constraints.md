# Plan constraints review

Reviewed plan: `$STAGE/plan.md` (Rev 0), with its annex `$STAGE/design.md` (D§n), against
`$STAGE/issue-analysis.md` (R1-R21).

## Governing documents

Read:
- `CLAUDE.md` (worktree root)
- `AGENTS.md` (the canonical contract, sections 1-8, including the deficiency ledger)
- `CONTRIBUTING.md`
- `.deficiencies` (it holds the notice and marker, with no entries)
- `Makefile` (the gate targets, `cross-check` at :102-107, `msrv` at :80-86)
- `rust-toolchain.toml`
- `.github/PULL_REQUEST_TEMPLATE.md`, `.github/workflows/ci.yaml` (the job names only)
- `CHANGELOG.md` (header and the `[Unreleased]` practice)
- `docs/architecture/okf-profile.md` (a decision page; this repository's ADR form)
- `docs/architecture/normal-form.md` (only its existence; the plan does not touch the census normal form)
- `crates/raf-workplan/src/refs.rs` (the pattern set behind `make issue-refs`)
- `crates/raf-workplan/src/rationale.rs` (module doc: a condition to revisit is judged by a person, not by the gate)
- `docs/capabilities/census-debt-ratchet.md` (only the adoption semantics)

Recorded absent: `.baseline`, `.goals`, `CONSTITUTION.md`, `PRECOMMIT_GUIDANCE.md`, `docs/adr/`,
`docs/decisions/`, and any nested `CLAUDE.md`/`AGENTS.md` under `crates/`. The repository's
decision records are the `## Decision` / `## Conditions to revisit` pages in `docs/architecture/`.
I treat them as accepted ADRs, which makes them standing constraints.

User decisions in force (as the caller relayed them): no timing runs, counts only; never install or
select a toolchain; `make cross-check` cannot complete on this host, so judge the disclosure;
breaking changes before release are acceptable; Linux is primary; no deferrals, so judge the
recorded-debt adoption against the no-deferrals rule and the `.deficiencies` policy.

## Findings

### B1 (BLOCKING): the `Debt` type departs from the accepted OKF-profile decision without revising it

Governing text:
- `docs/architecture/okf-profile.md:19`: "RAF documents follow a closed profile of OKF v0.1. Each of the six types has a closed key set written in one order"
- `docs/architecture/okf-profile.md:40-43`: "`type` takes one of six values: ... the record types `Evidence`, `Assessment` and `Coverage`, which state a result. Any other value is an error."
- `docs/architecture/okf-profile.md:324-325` (Conditions to revisit): "A repository needs a document type outside the six, so the closed set of types refuses documents its tools must read."
- `docs/architecture/okf-profile.md:63,67` (the key table): Change items and Coverage keys as they stand today.
- `crates/raf-workplan/src/rationale.rs:40-42`: "Nothing here reads what a condition says or decides whether it has been met: that is judged by a person". No gate will catch this.

Offending plan text (plan.md:52-54, 62-65; D§3.8; D§6.1 T1 names only the "claim-marker section" of
`docs/architecture/okf-profile.md`):
> "a machine-written singleton `work/debt/default.md` of a new `Debt` document type"

The new type meets this decision's own condition to revisit. The plan never says so, and it revises
only the claim-marker section of the page. After this change the Decision would still say "six types",
the type table would have no `Debt` row, Coverage would be missing `strictness`/`ceilings`/`exceptions`,
and Change items would be missing `reason`. That contradicts an accepted decision without recording it.
The schema-pin disclosure (plan.md:62-65) covers only the test expectation. It does not cover the
decision.

REWRITE TO (add to the Decisions list and to T1/T7):
> "* The `Debt` type meets the revisit condition of `arch:okf-profile` ('A repository needs a document
>   type outside the six'). T1 rewrites that page's Decision to seven types and adds `Debt` (record type,
>   `work/debt`, keys `type`, `id`, optional `tags`, optional `timestamp`, `findings`, `lines`) to the line 40
>   list and to the type table. It adds `strictness`, `ceilings` and `exceptions` to the Coverage row and
>   `reason` to the item keys (line 112), and replaces the met condition with the next one (for example
>   'a tool needs a record type outside the seven'). Check: the page states seven types and every key the
>   profile accepts. `docs_contract` and `self_workplan` stay clean, and the page's assessment, if it
>   has one, is current."

### B2 (BLOCKING): the adoption write lets this change put its own new findings into the debt

Governing text:
- `AGENTS.md:286-288`: "**No deferrals.** Work is done now. Scope that leaves a change is moved to another tracked change, with its own record, before the first one completes. Do not write 'later', 'follow-up' or 'out of scope for now' as a substitute for either."
- `CLAUDE.md:27`: "**No deferrals**: finish the work, or move the scope to another tracked change."

Judgment on the adoption as a whole (asked for by the user): recording the findings that already exist
on the base tree complies.
- R9 says verbatim that "adopting strict on an existing repository can start from today's findings".
- The repository's own designed ratchet adopts in the same way (`docs/capabilities/census-debt-ratchet.md:71`).
- No R-row asks this change to bind raf-okf's items or to mark the 196 existing claims.

So that work never was scope of this change, and none leaves it. The debt record is a product record.
It is not `.deficiencies`, and the plan writes nothing to `.deficiencies` (see A1).

The defect is when the snapshot is taken. Section 6.3 measures at `95c384c`, but T8 writes the debt
after T1-T7 have landed. In that time this change adds Contract clauses and rewords Constraints items in
eight delivered capabilities. D§7.4 says "a reworded Constraints item is a new claim". The change also
adds new code to raf-workplan, cargo-workplan and raf-source, and that code may contain orphan lines.
The adoption write takes "the present state" (D§3.8), so all of this would go into the record as
"today's findings". Those are findings of this change's own work. Recording them as debt is the
deferral the rule forbids, and "reviewed as the adoption control" (plan.md:60) gives the reviewer no
criterion that would catch it.

Offending plan text (plan.md:58-61):
> "the rest recorded as debt (about 786 finding rows and 4 line rows), reviewed as the adoption control."

REWRITE TO:
> "the rest recorded as debt, limited to findings already present on the base tree: every `findings`
> row of `work/debt/default.md` must also be raised by `cargo workplan --root <checkout of the merge
> base> --strict check --format json` (the new binary over an unmodified checkout of the merge base,
> which holds no debt record), and every `lines` row must be no higher than that checkout's count for
> the same crate in `cargo workplan --root <that checkout> coverage lines --format json`. A row that is
> missing from the base, or a count higher than the base, is this change's own work: T8 fixes it
> (marks the new or reworded claim, binds or reaches the new code) and does not record it. The
> comparison is a T8 check, and its result is summarised in the PR together with the debt diff.
> `cap:workplan-strict` has no row at all."

### A1 (ADVISORY): no executable check that `.deficiencies` stays empty

Governing text: `CLAUDE.md:29-32`: "Every entry below its marker is a bug and blocks completion, pull
request creation and merge of the work that produced it. The normal file holds only its notice and
marker." The same rule is at `AGENTS.md:20-22`.

The plan never mentions `.deficiencies`. Today the file holds only its notice and marker.

REWRITE TO (add to the PR task): "Before the PR, `.deficiencies` is confirmed to hold only its notice
and marker. Nothing produced by this change is written below the marker, and the strict debt record is
never described as a deficiency or as accepted risk."

### A2 (ADVISORY): the debt payoff is filed under "Declined / out of scope"

Governing text: `AGENTS.md:288`: "Do not write 'later', 'follow-up' or 'out of scope for now' as a substitute for either."

Offending plan text (plan.md:104, 106-108):
> "## Declined / out of scope ... paying the 786 debt rows in this change (the issue's ratchet item exists for exactly this ...)"

The reasoning is sound (see B2), but the heading reads like the phrase the rule bans. Committed text
that copies this framing (the CHANGELOG, the capability document, the PR) would fail the rule's spirit.

REWRITE TO: move the item out of "Declined / out of scope" into the Decisions bullet on adoption, worded as:
"Not scope of this change: the issue's ratchet item defines adoption from the findings already present
on the base tree. The record shrinks only through the ratchet. No committed file describes its rows as
work to be done later."

### A3 (ADVISORY): the cross-check disclosure misstates the toolchain and does not name the hand check

Governing text:
- `AGENTS.md:96-97`: "`rust-toolchain.toml` names a **floating nightly**".
- `rust-toolchain.toml:4`: "Floating nightly for local work and the CI gates."
- `Makefile:106`: `RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target "$$target"`
- `.github/PULL_REQUEST_TEMPLATE.md:9`: "- [ ] `make check` passes (... cross-check ...)"
- `CLAUDE.md:12` (gate before claiming done) and `CLAUDE.md:21-24` (Platforms).

Judgment: under the user decision and `CLAUDE.md:21-24`, disclosing the limit is acceptable, and the
hosted jobs it cites exist (`ci.yaml` `cross-check`, `test-macos`, `test-windows`). The disclosure has
three faults:
- It says "pinned nightly", which is false.
- "Windows is checked by hand" names no command, so it cannot be repeated.
- The PR template's `make check` box would be ticked or left unexplained.

T5 changes raf-source, which is the only package `cross-check` builds, so the hand check has to come
after T5.

Offending plan text (plan.md:25-27):
> "(the Apple and FreeBSD targets are not in the pinned nightly; Windows is checked by hand; ..."

REWRITE TO: "(the Apple and FreeBSD targets are not installed for the floating nightly on this host,
and the workflow installs nothing; the Windows target is checked after T5 and again at the end with
`RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target x86_64-pc-windows-gnu`,
the Makefile's command, and only if that target is already installed; the hosted `cross-check`,
`test-macos` and `test-windows` jobs must pass before merge; the PR leaves the template's `make check`
box unticked and states which targets ran here)".

### A4 (ADVISORY): `make msrv` has no rule for the case where it cannot run without selecting a toolchain

Governing text:
- User decision: never install or select a toolchain.
- `AGENTS.md:175`: "A missing tool is a hard failure, not a skip."
- `Makefile:82`: the failure message suggests "run with MSRV_RUN= and RUSTUP_TOOLCHAIN=1.98.1".

The final sequence includes `msrv` (plan.md:29). If `rust-validate` is not on this host, the Makefile's
own hint is to select a toolchain, which the user decision forbids.

REWRITE TO (append to plan.md:29-31): "`make msrv` runs only through the installed `rust-validate`. If it
is missing, that is reported as a failure in the PR and is never worked around with `RUSTUP_TOOLCHAIN`,
`+<toolchain>` or `rustup`."

### A5 (ADVISORY): new triggers of existing kinds have no named negative fixtures

Governing text: `AGENTS.md:235-236`: "**Every rule has a negative fixture.** A rule or check with no input
that makes it fail is not enforced." `CONTRIBUTING.md:45-46` says the same.

D§3.8 and D§3.9 give existing kinds new triggers:
- `scope-crate-missing` for a `ceilings` row
- `okf-empty-reason` for an `exceptions` row
- `okf-bad-measure` for `ceilings`/`lines` rows
- `okf-row-unknown-key`, `okf-row-missing-key` and `okf-duplicate-entry` for debt rows
- `okf-body-not-empty` and `okf-singleton-id` for the debt record
- `singleton-other-file` for `work/debt`

The kind test (R15) proves that each kind is produced once. It does not prove these new inputs. None of
them appears in the completeness contract (plan.md:79-81, 78).

REWRITE TO (R11 and R9 rows): add "each new trigger of an existing kind (`scope-crate-missing` from
`ceilings`, `okf-empty-reason` from `exceptions`, `okf-bad-measure` from `ceilings`/`lines`, the row
rules, `okf-body-not-empty` and `okf-singleton-id` from the debt record, `singleton-other-file` in
`work/debt`) has a fixture beside a valid neighbour".

### A6 (ADVISORY): no CHANGELOG entry, and the repository-level command lists go stale

Governing text:
- `CHANGELOG.md:6-7`: "All notable changes to RAF are recorded here. ... Until 1.0.0 a breaking change bumps the minor version." The `[Unreleased]` section is written by hand for each change (`CHANGELOG.md:9-30`).
- `AGENTS.md:62-65`: the delivered `cargo-workplan` commands are "`check`, ... `assess` and `coverage code|tests`". `README.md:32` has the same list.
- `AGENTS.md:83-90`: the layout table.
- `AGENTS.md:283`: "Documents describe the system".

The plan adds `--strict`, `coverage claims|lines`, `debt --write`, `work/debt/`, a `reason` item key, and
breaking API changes (`obligation::suppressed`, `ChangeItem`, `CoverageRecord`, `DocType`). No task
touches `CHANGELOG.md`, `AGENTS.md:62-65`, `README.md:32` or the AGENTS.md layout table.

REWRITE TO (add to T8): "Add a `CHANGELOG.md` `[Unreleased]` entry per crate that names the breaking
changes, with no process wording. Update the command list at `AGENTS.md:62-65` and `README.md:32`, and
add `work/debt/` (and the existing `work/coverage/`) to the AGENTS.md layout table."

### A7 (ADVISORY): the hooks are not named in the plan

Governing text: `CLAUDE.md:12-13`: "Install the hooks with `make hooks` and never bypass a hook or a gate."
`AGENTS.md:141-142`.

D§6.1 says "the hooks installed", but the plan wins over the design (plan.md:5), and it leaves this out.

REWRITE TO (plan.md:92 task line): "Tasks (each: with `make hooks` installed in this worktree, implement,
independent review, commit through the hooks, push, issue update)".

## Checked and compliant

- **Timing:** the plan uses counted work (`work.rs` tallies and budgets) and fixtures only, and it tells
  every agent the rule (plan.md:21; D§3.9 "Work"; T5 "budgets counted"). This complies with `CLAUDE.md:18-20`
  and `AGENTS.md:151-172`.
- **Process wording and tracker numbers:**
  - Committed artifacts refer to capabilities by slug.
  - The change-record claims and the description in D§6.2 describe the system.
  - "Closes #16" appears only in the PR body, which is not a committed file.
  - Nothing in the proposed committed text matches `refs.rs` (`task[ ]`+digit, `this issue`, `#N`, and so on).
  
  This complies with `AGENTS.md:280-285`.
- **Capability workflow:**
  - T1 opens the change record and puts PROPOSED blocks in the delivered capabilities it moves to `implementing`.
  - `INVENTORY` is edited in the same commit as each state change.
  - T8 completes the change with `cargo workplan change complete`.
  - Each change item has an outcome.
  
  This complies with `AGENTS.md:251-274`.
- **Closed lists and fail hard:**
  - The new kinds go into `KINDS`/`OWN_KINDS` and `ObligationKind::ALL` with the both-directions tests.
  - `strictness` is a closed enum.
  - Debt eligibility is closed, so a default finding can never be debt.
  - A broken record still fails the gate.
  
  This complies with `AGENTS.md:230-232` and `CLAUDE.md:14-15`.
- **Breaking changes and the schema pin:** these are covered by the user decision and are disclosed
  (plan.md:62-65). The fix-text change of `okf-item-unknown-key` (D§7.3) belongs in that same PR
  disclosure. It is a wording addition, not a separate finding.
- **Gates:** the final sequence is `make check` minus `cross-check`, run target by target, plus
  `licenses`/`issue-refs`. No gate or lint is weakened. The repository's `docs_contract` matrix check is
  kept (D§3.1). This complies with `AGENTS.md:173-174` and `CONTRIBUTING.md:37-38`.
- **Licensing:** new Markdown, including `work/debt/default.md`, is CC-BY-4.0 through `RecordOptions` and
  `--licence CC-BY-4.0`. This complies with `AGENTS.md:212-215`.
- **Dependencies:** no new crate is added. `proc-macro2` is already a dependency of raf-workplan, the
  `digest` module already exists, and the JSON fallback that would have pulled `serde_json` is declined.
- **Rust first:** the only shell addition is a thin `make workplan-debt` target over `cargo run`
  (D§6.1 T8). This complies with `AGENTS.md:227-229`.
- **Commit types:** `feat(raf-workplan)`, `feat(raf-okf)`, `feat(cargo-workplan)`, `feat(raf-source)` and
  `docs(...)` are conventional commits (`AGENTS.md:289-291`). The plan says no attribution (`AGENTS.md:292-293`).
- **Platforms:** "identical across platforms" is proven on Linux with CRLF/LF `MemoryTree` fixtures, and
  other systems are minimal effort (`CLAUDE.md:21-24`).
- **Naming:** `raf-*` and `cargo-*` names, and "RAF" in prose (`CLAUDE.md:25-26`).
- **Release:** nothing is tagged or published (`AGENTS.md:294-295`).

VERDICT: NEEDS-REVISION
