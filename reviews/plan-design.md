# Design review (SOLID / ETHOS): plan revision 2 (recheck)

Reviewed plan: `.stage/m2-rest-of-the-planning-essay-entity/plan.md` (revision 2)

Code read (revision 1 pass, re-consulted here): `crates/raf-okf/src/profile/{schema,typed,
body_rules,violation,mod}.rs`, `crates/raf-okf/src/body/marker.rs`;
`crates/raf-workplan/src/{model,graph,load,rules,finding,gate,sync,git,exec,lib}.rs`,
`exec/invocation.rs`, `obligation.rs`, `obligation/{sources,tense}.rs`, `lifecycle.rs`,
`lifecycle/{session,complete,completion,untouched,impact}.rs`;
`crates/cargo-workplan/src/{cli,command,change,exit}.rs`;
`crates/raf-source/src/{lib,item,facts,workspace}.rs`; the five capability documents,
`docs/architecture/*`, AGENTS.md; fixture sources under `crates/raf-workplan/tests` and
`crates/cargo-workplan/tests/support` (grep for `pub fn` fixtures: 40 sites, most in
`tests/support/mod.rs`).

## Prior findings

| Id | Status | Evidence in revision 2 |
| --- | --- | --- |
| B1 revision-keyed staleness | DISCHARGED | D5: "a recorded input digest differs from the digest now (no revision, no git ...)"; D3 one `digest` module shared by evidence and assessments; `revision` informational only. |
| B2 `work/coverage.md` nests the change records | DISCHARGED | D1: `work/coverage/default.md` "(a directory, not a file in `work/` ...)"; Task 3 singleton rule and fixture. |
| B3 arbitrary argv spawn | DISCHARGED | D4: command text "is NEVER run by the tool (no new process kind)"; D12 "The tool starts no process other than `git`"; edit list row for the PROPOSED sentence. |
| B4 public surface unreachable | DISCHARGED | D9/Task 2: `Workspace::exported_items(target)` in raf-source with a record listing `cap:source-index`; fixtures for private-module re-export, globs, `pub(crate)`, `doc(hidden)`. |
| B5 claim marker parsed outside raf-okf | DISCHARGED | D2: parsed in `body/marker.rs`; "any other `<!-- raf:` comment ... is a finding"; R2 "misspelled `raf:` marker => finding". |
| B6 "last commit where it resolved" had no mechanism | DISCHARGED | D11: pickaxe `-S<final path segment> -n 1 -- <crate dir>` plus `rev-parse <commit>^`, identifier pattern (D12), limits stated in the Contract, R1 row. |
| B7 shallow clone silent partial | DISCHARGED | D11: `git rev-parse --is-shallow-repository`, refuse on `true`. |
| A1 `Workplan::write` cannot create a record | DISCHARGED | D13: `create_new` for new files, stage-then-rename for replacements. |
| A2 process kinds assembled in two places | STILL-OPEN | D7 names one `trace` source for the static kinds only; `evidence-changed` "(process kind)" still lands in both `session.rs:343-352` and `complete.rs:200-207` unless one helper is named. See A2 below. |
| A3 hard-coded type lists in two messages | STILL-OPEN | Not mentioned; `violation.rs:205-208` and `finding.rs:323-326` still enumerate three types. |
| A4 digest de-dup by source key | DISCHARGED | D3 "de-duplicated by source key". |
| A5 fixtures need a digest helper | DISCHARGED | D8 "one shared helper that records assessments"; Task 6 acceptance "all existing `complete` and scenario fixtures green". |
| A6 claims in PROPOSED blocks | DISCHARGED | D2 "Claims inside PROPOSED blocks are not claims until resolved". |

## New text reviewed (revision 2 only)

No BLOCKING finding.

### A2 (carried) `evidence-changed` joins `section-untouched` in two assembly sites

Evidence: `Lifecycle::reconcile` (`session.rs:343-352`) and `Lifecycle::evaluate`
(`complete.rs:200-207`) each call `untouched::raise` and `examples::raise`; D7 adds a third
process kind. A kind added to one and not the other makes `reconcile` print nothing while
`complete` refuses. REWRITE TO: one `Lifecycle::process_obligations(&self, opened, &impact,
effect)` returning every non-static kind, called from both; `ObligationKind::is_static`
names the same set. One sentence in Task 5.

### A3 (carried) type-list messages

REWRITE TO: build the list in `Rule::UnknownType`'s `Display` and `rule_entry`'s fix from
`DocType::ALL` (Task 1).

### A7 D9 exclusion digests make `check` fail on every edit of an excluded item

Plan: "Exclusions are `{item, reason, digest}` entries ... a changed item digest [is a]
finding (`coverage-exclusion-*`)".

Consequence: the scope crates exclude by reason mainly tests that guard internals; every edit
to such a test changes its digest and fails `make check` until the record is re-authored, and
no command to refresh a digest is named. The reason justifies the exclusion; the item's text
does not. The existing rule "an exclusion matching nothing is a finding" already catches a
removed item. REWRITE TO: `{item, reason}` only (prefix entries stay as is), or, if the
digest is kept, name `coverage exclude --refresh` in Task 9 and state in D9 why an edited
excluded item must be re-justified.

### A8 D9 "no Coverage record => whole workspace enforced" reaches every existing fixture

Plan: "A repository with no Coverage record has coverage enforced for the whole workspace (no
silent skip)."

Consequence: the fixture workspaces in `crates/raf-workplan/tests/support/mod.rs` and the
other test files define public items (40 `pub fn` sites) that no capability binds; with this
default, `Workplan::check` reports gaps in them, and `complete`'s `after_removal` check
(`complete.rs:303-322`) refuses every existing completion fixture. Task 7's acceptance lists
only the R4 fixtures. This is loud, not silent, so not blocking, but it is the same
regression class the plan already handles for condition 6 in D8. REWRITE TO: add to Task 7
"all existing fixtures and scenarios green" and let the shared fixture helper write a
`work/coverage/default.md` with an empty scope (or bind the fixture items) once, in
`tests/support`.

### A9 D1 forces the `Coverage` record into the trace shape

Plan: three types "sharing one trace shape: `subject`, `inputs` ..., a type-specific
`verdict`, `by`".

Consequence: a Coverage record has no verdict, no assessor and no inputs (its entries are
exclusions, D9); giving it those keys either leaves them meaningless-but-required or adds
optional keys the validator must special-case per type (`profile/mod.rs:253-306` already
branches on key name). Fat shared shape with one real sharer pair (ISP). REWRITE TO: the
trace shape is shared by `Evidence` and `Assessment` only; `Coverage` has `scope` and
`exclusions`. D1 names which keys each type has.

## Sound as designed

D2 (one marker scanner), D3 (one digest module, domain-separated, de-duplicated), D4/D5
(no spawn, no git for staleness), D6 (merge-base reading through `blob_at`/`MergeBase`), D7
(closed set growth is compile-enforced at `obligation.rs:129-143`), D8 (condition 6 through
`Condition`/`Failure`, shared fixture helper), D9's `exported_items` seam, D10 (fires on no
existing page: neither architecture page has a `## Decision` heading today, and both are
linked), D11/D12 (new rows as typed `Invocation` constructors with validated placeholders),
D13 (exclusive create plus stage-then-rename).

VERDICT: PASS
