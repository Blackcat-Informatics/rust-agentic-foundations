# T1 review: commit 03edbca

VERDICT: BLOCKED (two small required fixes, B1 and B2; everything else passes)

Reviewed only commit 03edbca, in a scratch clone at /tmp/t1rev.dl5M7s/r with
`git checkout 03edbca` (pwd checked). The worktree was not edited apart from this
file. No timing run was made.

## Required fixes

**B1. CHANGELOG.md:11-41 describes strict mode as shipped.** The [Unreleased] entry
says, in the present tense, that `strictness: strict`, `Gate::strict()`, `--strict`,
`coverage claims|lines|--reverse`, `debt --write` and `Gate::write_debt` judge, report
and write, and that "`raf-source` resolves paths from the modules of every target and
lists impl blocks". None of this exists at this commit. AGENTS.md:65-69 and
README.md:33-37 correctly call it **designed**, so the CHANGELOG contradicts them. The
brief requires these files to describe the feature as designed, not shipped.
Fix: open the entry with "strict mode is designed (capability document, change
record, decision page `arch:debt-ratchet`)" and put the behaviour and the breaking
changes in the future tense ("will add ...", "`DocType::ALL` will have seven
entries"). Alternatively, keep only the designed-documents sentence now and let each
of T2-T8 add its own present-tense line as its code lands. T8 then checks every
sentence against the delivered code.

**B2. The inventory does not reject a capability listed twice when the table copies
the duplicate.** I mutated the scratch tree: `work/changes/workplan-strict.md`
`changes: [cap:workplan-strict, cap:okf-profile, ..., cap:okf-profile, ...]`, with the
same duplicate added to the `workplan-strict` row of `INVENTORY`. The result:
`repository_documents_satisfy_the_profile` passes and `cargo workplan check` exits 0.
The profile has no `DuplicateEntry` rule for `changes`. That gap predates this commit,
but the new table is now the pin for these lists. Fix: one assertion in
`inventory_table_has_the_documented_shape` (tests/docs_contract.rs:1237): within each
`INVENTORY.changes` row, no capability slug repeats. Optionally also assert that each
listed slug is in `INVENTORY.capabilities`. That makes "lists exactly its
capabilities" hold in every configuration.

## (1) Capability and change record against the plan (Rev 1/2): PASS

Every decision is stated in docs/capabilities/workplan-strict.md, and none is
contradicted:
- Setting, floor and "nothing lowers it": :33-50. Location decision, the three
  rejected locations and the conditions to revisit: :454-486 (R1). Default stays
  lightweight, with counted-work tallies: :48-50 and :589-590 (R21, A9).
- Claim unit and key: :61-77 (`###`/`####` as in D§3.2). `raf:tested-by` as its own
  family, not an extension of `raf:claim`: :78-91 and :496-499.
- Collapse kind `strict-claim-unit-collapsed`: :94-98 and :501-505 (B2 of Rev 2).
- Orphan symbols: no new kind, the set of packages widened. One producer: "The strict
  rules compute this coverage themselves; the default coverage check is the same under
  either strictness": :108-117. "only producer of debt": :466.
- E3 literal touch rule (tokens of the whole span, `#[doc]` counts,
  whitespace/comments do not, removed/renamed/moved-file are touched, moved within the
  file is not, an unparsable base is an error): :119-143 and :507-513. Fixtures: :563-570.
- Census semantics (growth fails, resolved fails until the write, the write refuses
  growth except at creation): :242-283 and :525-532. `DebtKey` shape, line rows exactly
  `item`+`orphans`, count only, below = `debt-resolved`, above =
  `coverage-lines-over-ceiling` naming the row, `coverage-lines-over-ceiling` not
  eligible (`debt-entry-invalid`): :245-267 (B3, A4).
- `--reverse` alias, byte-identical output: :171-172 and :515-518 (E5). Every package
  the loader reaches is judged: :168-171 (F4).
- Ceiling classes `of: shipping|test|all`, default `all`, `okf-bad-ceiling`, percent
  without division, empty package passes: :218-240 (A2, A6).
- Fuzz limit, and no exception names the fuzz workspace: :429-432 (B3). The
  `raf:tested-by` marker the plan wants on it cannot be written before T3, because the
  reader rejects an unknown `raf:` comment (architecture page :268-271). It belongs to
  T8.
- R19 reconciliation prose: :488-499.

Change record work/changes/workplan-strict.md: the ten capabilities and eighteen
`affects` entries match D§6.2. Item 11 is reworded per A3. Item 12 (source index)
covers T5. Items 2-8 say "Under strict". `contract_effect: changes`.

Advisory A1 (wording, before T6): workplan-strict.md:229-232 says "a class that no row
of the package bounds, by its own row or by an `all` row, is held to 0". Read with
`all` counted as a class, a package with only `of: shipping, lines: 10` would have its
sum held to 0. That defeats the shipping row and contradicts the Examples (:370,
:385-387), where only the test class is over its ceiling. State it plainly:
"shipping and test are each held to 0 unless their own row or an `all` row bounds
them; the sum is judged only against an `all` row". Add a fixture for the
shipping-row-only package's sum.

## (2) The PROPOSED blocks: PASS (there are 12, not 13)

`grep` and `change reconcile` both find 12 blocks:
- okf-profile.md:180 and :317 (contract)
- source-index.md:1444 (contract)
- workplan-cli.md:395 (contract) and :450 (usage)
- workplan-complete-gate.md:352 (contract)
- workplan-document-model.md:423 and :522 (contract)
- workplan-generated-docs.md:154 (contract)
- workplan-reconcile.md:412 (contract)
- workplan-two-way-coverage.md:159 (contract) and :227 (constraints)

These are exactly the sections that D§6.1 T1 and the record's `affects` name. The
13th change is the plain Explanation paragraph of the designed
census-debt-ratchet.md:139-142, which is not a PROPOSED block.

Every name the blocks cite exists today:
- `TargetKind` segments `test_crate|bench_crate|example_crate|bin_crate` and
  `<pkg>::build_script` (raf-source/src/target.rs:361-371, :457)
- `Workspace::same_source`, `raf_source::parse_source`, `Lifecycle::open_with`
- the reconcile table order (`item-without-outcome`, `section-untouched`)

The blocks agree with the capability and leave the delivered text outside them true.

Minor advisories:
- workplan-cli.md:418 lists the JSON fields without `kind`, which workplan-strict.md:295
  includes.
- okf-profile.md:192-194 says "a wrong shape `okf-bad-exception`", while
  workplan-strict.md:224-226 also counts a crate root, an absolute path and `..` as
  `okf-bad-exception`. Say which layer checks the path form.
- README.md:21-22 still says "delivered: twelve capabilities". Eight of them are now
  `implementing`. AGENTS.md:68-69 says so; README does not.

## (3) tests/docs_contract.rs: sound, does not weaken any check, minimal; B2 is a gap

Before this commit, every capability that was not delivered needed a record named
after it. Now every table record must exist with its id, and its `changes` must equal
the table row exactly. Every capability that is not delivered must be in some row.
Unlisted documents are still rejected. Nothing is lost except the naming requirement,
which is the intended change. The content of `changes` was never checked before, so the
new check is stronger.

Scratch runs (the scratch test was removed afterwards):

| Case | Result |
| --- | --- |
| Suite at 03edbca | 30 passed |
| (a) Record file omits a capability the table gives it | REJECTED: `work/changes/beta.md: inventory expects `changes` [cap:beta], found []` |
| (b) Record lists `cap:alpha` twice, table once | REJECTED: `inventory expects `changes` [cap:alpha], found [cap:alpha, cap:alpha]` |
| (e) Table and file both drop open `beta` | REJECTED: `docs/capabilities/beta.md: inventory lists no change record whose `changes` names `cap:beta`` |
| (c) Duplicate copied into the table, on the real tree | ACCEPTED (B2) |
| (d) Record with no `changes` key | skipped by the new rule; caught by the profile (`okf-missing-key`) |

For (d), the comment at tests/docs_contract.rs:300 ("reported by the identity check")
should say "by the profile".

The census-debt-ratchet capability is listed by two records, so the rule is "at least
one", and the tree needs that.

AGENTS.md: no contradiction. Advisory: AGENTS.md:261-263 could add "opening or
completing a change" to the edits that change the table, as the doc comment at
tests/docs_contract.rs:58-66 now does.

## (4) The okf-profile architecture page: the per-task split is right; no step is impossible

crates/raf-okf/tests/schema.rs:131-141 reads docs/architecture/okf-profile.md and
requires one row per `DocType::ALL`, with keys in code order. Adding the `Debt` row or a
Coverage key at T1 fails it. Saying "seven types" (:19, :40) would be false while the
code has six. Architecture pages have no PROPOSED mechanism. So each edit goes with its
code:

| Task | Page edit |
| --- | --- |
| T2 | Coverage row `strictness` (and schema.rs:124-127) |
| T3 | Marker section :248-285, which today makes any other `raf:` word an error (:268-271) |
| T4 | `items` rule :111-114 (`reason`) |
| T6 | Coverage row `ceilings`, `exceptions` and their value rules beside :142 |
| T7 | `Debt` row, "six" to "seven" (:19, :40), the revisit condition at :325-326 |

Every step is possible in this order, and every commit stays green. Record the
reassignment in the plan, or in each task's checklist: Rev 1 put the page rewrite in
T1/T7, and only the table is pinned by a test, so T3 and T4 could be forgotten.

## (5) Reconcile output: reproduced; the stale evidence can be re-recorded with no timing run

The scratch clone's `origin/HEAD` pointed at the feature branch. Pointed at
`origin/main` (merge base 95c384c), `cargo workplan change reconcile workplan-strict`
exits 1 with:
- 12 unresolved-proposal (not 13; see (2))
- 12 item-without-outcome
- 9 assessment-missing: census-debt-ratchet explanation, workplan-generated-docs
  contract, and the seven workplan-strict sections
- 1 evidence-stale

The staleness predates this commit. `raf_workplan::exec` changed on main after 88d9e42
(0851241, 1e5dc74, both ancestors of 95c384c). It appears now only because
cap:workplan-complete-gate is listed.

I re-recorded it in the scratch clone:
`cargo run -q -p cargo-workplan -- workplan evidence record --update --id gate-kills-a-test-past-its-timeout --for cap:workplan-complete-gate --by agent --copyright '<the repo text>' --licence CC-BY-4.0 --from-binding raf_workplan::exec::system::tests::cargo::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran`
- It exited 0 with "recorded: ... (pass)".
- The only diff is the `raf_workplan::exec` input digest
  (19826c31... to 3c6c037d...).
- A second reconcile showed 0 evidence-stale. Reverted afterwards.

What the re-record runs:
- The production `SystemRunner` runs the binding through cargo: a `--list` pass, then
  `cargo test --frozen --package=raf-workplan --lib -- --exact ...`. It uses the
  `WallClock` of crates/raf-workplan/src/exec/clock.rs, the crate's one sanctioned
  clock, only as the watchdog of its limits and for its poll pauses. Nothing is asserted
  on it, and the record stores no time.
- The test itself (crates/raf-workplan/src/exec/system/tests/cargo.rs:470-523) builds a
  generated crate. The crate's list pass runs under `WallClock` with `BUILD_TIMEOUT`, as
  a watchdog only. The never-ending run is driven by a `Stepped` clock that moves only on
  pidfd/FIFO kernel events.
- Its assertions are on the stepped reading (`elapsed() == LIMIT`, the "5.0 s" message)
  and on `wall() >= LIMIT`. The second holds by construction, because the stepped part
  alone is LIMIT. Nothing is asserted on machine time.

So the re-record reads the machine clock only as a watchdog. It is a scripted-clock
test, not a timing run, and the rule allows it. The task that runs `change complete`
should run it, or any earlier task.

## (6) Wording, SPDX, assessments, inventory and command lists

- 1186 added lines. A grep for issue, previously, no longer, replac*, follow-up,
  tracker, `#N`, /pull/, /issues/, plan, task, branch, `T<n>`, later, "for now" and
  hex hashes (digests excluded) finds no hit. `make issue-refs` passes. No attribution
  in the commit.
- SPDX: the three new files carry the CC-BY-4.0 frontmatter lines. `make reuse` passes.
- Assessments, three spot-checked: okf-profile-contract, workplan-reconcile-contract
  and workplan-two-way-coverage-constraints. Only the section digest changed;
  `verdict: accepted`, `by: agent`. The sections changed only by their PROPOSED
  blocks, which are consistent with the delivered text, and the symbol inputs are
  unchanged. Accepting them again is justified. self_workplan passes, so every digest
  is current.
- INVENTORY counts are right: 22 capabilities (10 designed, 8 implementing, 4
  delivered), 10 open records, 3 architecture pages. The index catalogue is
  regenerated.
- AGENTS/README command lists: accurate, and say designed. CHANGELOG: B1.
- debt-ratchet.md:35 says "two entry points with one behaviour". The census's own
  command handles only ledgers without registered code (census-debt-ratchet.md:53-54).
  Advisory: "one rule".

## (7) Gates in the scratch clone at 03edbca: all pass

| Gate | Result |
| --- | --- |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | 30 / 1 / 68 passed |
| `cargo workplan check` | exit 0 |
| `make fmt`, `clippy`, `doc`, `deny`, `reuse`, `issue-refs` | each exit 0 |

Hooks: `core.hooksPath=.githooks` is set in the worktree.
