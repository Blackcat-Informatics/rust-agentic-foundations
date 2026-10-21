# Plan final review (Stage 1), pass 2

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/3-raf-census-core-normal-form-v1/.stage/raf-census-core-normal-form-v1/plan.md`
("Rev 2" authoritative over "Rev 1", which is authoritative over the earlier text).

Reports and dispositions examined: `reviews/plan-completeness.md` (B1-B6, A1-A7),
`reviews/plan-constraints.md` (B1-B2, A1-A9), `reviews/plan-design.md` (B1-B5, A6-A10),
`reviews/plan-enhancement.md` (items 1-15), `reviews/plan-adversary.md` (E1, F1-F12), the plan's
"Changed decisions", "Enhancements Considered and Declined", "Dispositions index" and "Rev 2".
Verified against: `issue.md`, `issue-analysis.md`, `prior-art-assessment.md`,
`docs/architecture/normal-form.md`, `docs/capabilities/census-normal-form.md`,
`docs/capabilities/census-ledger.md` (Contract, roles), `docs/capabilities/census-rule-api.md`
(`FileCtx`), `docs/capabilities/census-cli.md` (`job <ID>`), `work/changes/census-normal-form.md`,
`work/changes/census-ledger.md`, `Makefile` (`check`, `msrv`, `test`, `clippy`), and
`crates/raf-source/src/{workspace,facts,budget}.rs` (`items_by_source`, `is_shipping`,
`TestRole`, `Budget::Delimiters` = 128 exist as cited).

## Pass 1 summary (lens fixes verified as real)

Every BLOCKING finding of the five lenses has Rev 1 text that does what the Dispositions index
claims (entry point and `min_tokens`; the ported shim predicate with its fixtures and a grouping
neighbour; capability Examples from real output and the no-argument example run; the
literal-spelling pair; `Error::Source` fail-hard with valid neighbours and `TestRole`; units by
source identity; `syn` token sources and the `SymbolKind` set; exact report members; version
items; one golden registry; full ordering and negative neighbours; the `Fingerprint` enum). The
adopted enhancements (1, 3-15) each have concrete text and fixtures. The decline of enhancement
item 2 states a technical reason, a visible cost and why `explain` suffices. The single full
`make check` after Stage 2 fixes, with `make msrv` in T6, is accepted as the workflow's one
qualification: the reason and the cost (PR states it pending) are both in the plan.

## Pass 2: status of the pass-1 findings

### BLOCKING 1 (option B narrower than the classification contract): DISCHARGED
Rev 2 "Job matching (B1)" adopts the rewrite verbatim: `Job { id, home, entry_points, variants,
forbidden: JobForbidden { fingerprints, constants } }`, `Home` covers `home` or an `entry_points`
symbol (matching `cap:census-ledger` § Contract "the job's home or an entry point"), constants
match by value, `rule:` ids match nothing here with the reason stated, `names`/`patterns` stay out
with the reason stated, and the acceptance names every class and `copies: 2`.

### ADVISORY 2 (version-gate split ambiguous, goldens clause dropped): DISCHARGED
Rev 2 "Version-gate items (A2)": two `implemented` items, (a) goldens, (b) the gate with its
message; the ledger-file half is removed by rewording because `change:census-ledger` already holds
"A normal_form mismatch fails before any job is read". A definite choice with the ownership
argument given; nothing leaves the records unowned.

### ADVISORY 3 (hashed level below the literal tier; numbering): DISCHARGED
Rev 2 "Tower (A3)": `kind_literals` is the identity below 30 structural tokens, so R4 holds on the
final sequence; parameters are numbered first in signature order, then locals by first appearance,
one counter. Signature order is the sound choice: body-first-appearance would give `fn f(a, b)
{ b / a }` and `fn g(a, b) { a / b }` one structural form. T6 must restate § Structural form
accordingly (the page now says "by order of first appearance"); Rev 2 says the page states it.

### ADVISORY 4 (shim candidate's `body` print): DISCHARGED
Rev 2 last bullet states it once: the body print exists at or above `min_tokens` and is listed in
the index; grouping uses the shim print only. Consistent with `match_jobs` matching either print.

### ADVISORY 5 (whole-crate suite): DISCHARGED
Rev 2 "Whole-crate tests (A4)": every task runs `cargo test -p raf-census --locked`.

### ADVISORY 6 (completeness A5 undisposed; `max_file_bytes`): DISCHARGED
`tests/page.rs` asserts the page names every pass id of the pinned manifest, every threshold
value, every `VALUE_ADAPTERS` entry, every report member and the encoding id: falsifiable, and a
code constant changed without the page fails it. `CensusOptions` is `#[non_exhaustive]` with
`Default` and setters for `min_tokens` and `max_file_bytes` mapped to
`raf_source::LoadOptions.max_file_bytes`, with an above-cap error and an at-cap neighbour.

### ADVISORY 7 (PROPOSED blocks name Contract): DISCHARGED
Rev 2 "T1 PROPOSED blocks": all six affected sections, Contract carrying job matching, the version
gate wording, the shim predicate and the report members.

## New finding (introduced by the adopted rewrite; one line to fix)

### ADVISORY 8. The job fixture's entry point must be a unit symbol
Plan line (Rev 2, Job matching): "an `entry_points` symbol on a re-export of it".
A `pub use` re-export is not a unit (units are fn, method, const, static), so no unit's symbol
equals a re-export path and the "one `Home` (entry point)" assertion cannot be produced that way.
Resolution through re-exports is the ledger's job before it builds a `Job`; in this change the
`Job` symbols are unit symbols.
REWRITE TO: "an `entry_points` symbol naming a second unit with the same body (a wrapper the job
also owns)" and, appended to the `Job` sentence: "`home`, `entry_points` and `variants` are unit
symbols; resolving a ledger path through re-exports to a unit symbol is `change:census-ledger`."
The expected counts are unchanged: `Home` x2, `Variant` x1, `Copy` x2 (`copies: 2`).

## Coherence after Rev 2

Rev 2 overrides Rev 1's "Version gate" bullet and the earlier scope-boundary bullet 2 (`moved`)
cleanly and says so. No new contradiction between tasks, between the contract and the tasks, or
between changed and unchanged decisions was found. Task order, per-task review/commit/push/issue
update, focused checks, the no-argument bound example, the final PR task and the absence of a
Task 0 are unchanged from pass 1.

## Open user decisions

None.

VERDICT: PASS
