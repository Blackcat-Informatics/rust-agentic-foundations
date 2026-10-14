# Plan completion adversary: Stage 1

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`
- **Passes recorded:** pass 1 (revision 1), pass 2 (revision 2) and pass 3 (revision 3, final), each appended below. There is no fourth pass.
- **Inputs read:** `issue.md` (0 comments), `issue-analysis.md` (R1–R32), `prior-art-assessment.md`, `docs/capabilities/workplan-document-model.md`, `docs/capabilities/workplan-generated-docs.md`, `docs/capabilities/workplan-cli.md`, `docs/architecture/okf-profile.md`, `work/changes/workplan-{document-model,generated-docs}.md`, `tests/docs_contract.rs`, AGENTS.md, CONTRIBUTING.md, README.md, raf-okf and raf-source sources where the plan relies on them.
- **User scope decisions quoted:** pass 1 and pass 2 had none. Revision 3 quotes one at G20: "Citation fix, no reopen (Recommended)". It covers only the `cap:source-index` citation and narrows no issue scope.
- **Verdict vocabulary:** the caller asked for PASS or NEEDS-REVISION. NEEDS-REVISION means the same as FINDINGS-OPEN. Pass 3 uses ESCALATE-TO-HUMAN, as the pass cap requires.

## Pass 1 (revision 1)

### F1: PRE-EMPTIVE DESCOPE and UNFALSIFIABLE CRITERION (R4, R30; Task 8; Lessons)

> L147: "Set both to `delivered`, with code, test and example bindings to the built items, and reconcile every Contract sentence against the code, removing or fixing any unbuilt claim."

> L49: "Contract sentences must match the code. Rule: a contract claim needs a test or must be removed."

L14 makes the Contract, Constraints and Verification sections binding. L147 and L49 then let any unbuilt Contract claim be removed in the last task, with no list of which claims and no user decision. Task 8 also reconciles only "Contract" sentences. Several statements in the Contract, Usage, Constraints and Explanation sections of the two documents describe behaviour this plan does not build:

- `workplan-generated-docs.md:70-74` (Usage) shows `cargo workplan docs sync`, `cargo workplan check` and `cargo workplan docs preview <slug>`. The binary belongs to `cap:workplan-cli` (`workplan-cli.md:46-56`).
- `workplan-generated-docs.md:99` (Constraints) says "Preview output is printed and never written to a file." `docs preview` belongs to `workplan-cli.md:59-61`, and nothing in this plan builds it.
- `workplan-generated-docs.md:63-64` (Contract) says "Generation covers reference sections, relationship tables, status summaries, ...". That is wider than the closed set of three kinds in J7.
- `workplan-document-model.md:104-106` (Contract) says "`status`, the catalogue and the reconcile obligations report each one as a gap". Reconcile obligations belong to `cap:workplan-reconcile` (J11).
- `workplan-document-model.md:135-136` (Usage) says "The `cargo workplan` binary exposes the same checks and the change lifecycle".
- `workplan-document-model.md:219-223` (Explanation) describes the `binding-missing` and `example-failing` obligations.

As written, a delivered contract smaller than the binding spec would still count as "reconciled".

**REWRITE TO (Task 8, replacing L147):** "Set both to `delivered`, with code, test and example bindings to the built items. Reconcile the two documents against the following table, and remove or reword nothing outside it:

| File:lines | Statement | Disposition | Owner |
|---|---|---|---|
| generated-docs 70-74 | `cargo workplan …` usage | Reworded to `Workplan::sync`, `Workplan::check`, `examples/docs_sync.rs` and `make docs-sync` | cap:workplan-cli (open record `work/changes/workplan-cli.md`) keeps the CLI usage |
| generated-docs 99 | preview constraint | Removed here; already stated at `workplan-cli.md:59-61` | cap:workplan-cli |
| generated-docs 63-64 | 'reference sections … status summaries' | Reworded to the three kinds (J7) | this capability |
| document-model 104-106 | 'reconcile obligations report each one as a gap' | Reworded to `status` and the catalogue | cap:workplan-reconcile |
| document-model 135-136 | `cargo workplan` binary | Reworded to `Gate` and the library | cap:workplan-cli |
| document-model 219-223 | obligation kinds | Reworded as cap:workplan-reconcile's behaviour, linked | cap:workplan-reconcile |

Every other sentence in the Contract, Constraints and Verification sections of the two documents is built and bound to a test before `state: delivered` is set. A sentence found unbuilt that is not in this table blocks Task 8 and gets implemented, not removed."

**REWRITE TO (L49):** "Contract sentences must match the code. Rule: every Contract, Constraints and Verification sentence of the two documents is bound to a test. The only sentences reworded rather than built are those in the Task 8 table, and each one is owned by a named capability with an open change record."

### F2: UNMAPPED REQUIREMENT (R30 "Capability docs reconciled", R2 "always states the complete current contract")

> L59: "| R4, R30 | Both capabilities delivered; docs reconciled | 8 | Both docs are `state: delivered` with resolving bindings and synced matrices, the change records are completed, and `check` and `docs_contract` pass on this repository. |"

> L148: "Record the outcomes in the change records, then complete (delete) them, and update the AGENTS.md inventory."

J1, Task 5 and Task 7 change behaviour that other committed documents describe. One of those documents is a delivered capability. Neither `check` nor `docs_contract` can detect prose that has become false. The R30 row covers only the two documents, so these statements have no task:

- `docs/capabilities/source-index.md:1488-1490` and `:1497-1500` (delivered, Verification): "the document-contract test rejects a binding to the same misplaced shape" and "Binding fixtures in the document-contract test: …". J1 moves those fixtures into `crates/raf-workplan/tests`.
- `docs/architecture/okf-profile.md:31-40` ("That test adds the rules that span documents … The cross-document checks are what `raf-workplan` replaces once it is implemented.") and `:173-174` ("None exist yet."). Task 8 creates the first generated blocks.
- `AGENTS.md:209-213` ("Today the document-contract test … `raf-workplan` takes those over once implemented") and `:220-223` ("until `raf_workplan::Gate` replaces it"). J1 keeps `INVENTORY`, so this sentence becomes false whichever way the inventory goes. The crate row at `AGENTS.md:63` also says `designed`.
- `CONTRIBUTING.md:32-36` ("today a document-contract test … Once `raf-census` and `raf-workplan` exist, their census and workplan gates join it.").
- `tests/docs_contract.rs:14-19` (module doc) and `:61-62` ("`raf_workplan::Gate` replaces it.").
- `crates/raf-okf/tests/usage_docs.rs:4-6`, which relies on the doctest-pairs check "once, by the document-contract test of the root package".
- Naming error: J1 and L148 say "the AGENTS.md inventory table", but the inventory is `INVENTORY` in `tests/docs_contract.rs:70-95`. AGENTS.md has only the crate table at `:63`.

**REWRITE TO (new Task 8 bullet and R30 acceptance):** "Reconcile every committed statement this PR makes false:
- `source-index.md` Verification (1488-1490, 1497-1500): name the raf-workplan test file that now holds the binding-role fixtures.
- `okf-profile.md` 'Relation to OKF' (31-40) and 'Generated blocks' (173-174).
- AGENTS.md §6 (209-223) and the `raf-workplan` row (63).
- CONTRIBUTING.md 32-36.
- The `tests/docs_contract.rs` module doc (14-19) and the `INVENTORY` comment (61-62).
- `crates/raf-okf/tests/usage_docs.rs:4-6`, if the doctest-pairs test moves.

Set the two `INVENTORY` rows in `tests/docs_contract.rs` to `delivered`. Acceptance: `rg -n 'document-contract test|None exist yet|replaces once it is implemented|takes those over once implemented|Gate. replaces it' docs AGENTS.md CONTRIBUTING.md tests crates` returns only lines that are true after the change, and the PR body lists each one."

### F3: UNFALSIFIABLE CRITERION (R26; R5–R7; Task 1 check)

> L74: "One named fixture per rule in J10's list. A coverage test asserts every `Rule` variant has a fixture (exhaustive match), so a new rule without a fixture fails to compile."

> L60 and L90: "one end-to-end fixture per violation kind" / "one end-to-end fixture per raf-okf violation kind surfaces through `Workplan::check`"

There are two problems:

1. An exhaustive match proves only that each variant names some fixture function. It does not prove that the fixture produces that variant, because an arm can point at any fixture.
2. The house style marks public enums `#[non_exhaustive]`, for example `raf_okf::Rule` (`crates/raf-okf/src/profile/violation.rs:30`) and `raf_okf::ErrorKind` (`crates/raf-okf/src/error.rs:11`). A non-exhaustive enum can be matched exhaustively only inside its own crate. A coverage test in `crates/raf-workplan/tests`, where J1 puts the fixtures, would need a wildcard arm, so the "fails to compile" guarantee would be gone. raf-okf's kinds can never be matched exhaustively from raf-workplan, so "one fixture per raf-okf violation kind" has no list to check against.

**REWRITE TO (R26 acceptance):** "A `#[cfg(test)]` coverage module inside `crates/raf-workplan/src`, where `Rule` can be matched exhaustively even if it is `#[non_exhaustive]`, maps every `raf_workplan::Rule` variant to a named fixture through a match with no wildcard arm. For each variant the test runs the fixture through `Workplan::check` and asserts that the violations contain that variant and name the fixture's file. It also runs the neighbouring valid case and asserts there are no violations."

**REWRITE TO (L60 and L90):** "Task 1 lists every `raf_okf::Rule` and `raf_okf::ErrorKind` variant at 762fa99, each with the named end-to-end fixture that surfaces it through `Workplan::check` and names the file."

### F4: UNFALSIFIABLE CRITERION (Task 7; J1; R26)

> L140: "model rules and helpers are deleted, along with their fixtures (moved into the crate in Tasks 1–5), and replaced by `Gate::workspace(gate_root()).assert_clean()` plus this repository's own policy (J1)."

> L141: "**Check:** `cargo test --locked --test docs_contract --test repo_gates` passes"

> L27 (J1): "`docs_contract.rs` keeps only this repository's own policy: the AGENTS.md inventory table, the wording and placeholder scans over every tracked file, and the tracked-symlink rule."

The L141 check passes whether the fixtures were moved or simply deleted, and nothing says which of the roughly 150 `#[test]` functions go where. J1's "keeps only" list also leaves out repository tests that are not document-model rules and have no home in raf-workplan:

- The doctest-pairs tests: `usage_and_examples_blocks_are_the_crate_doctests`, `a_block_that_differs_by_one_line_is_reported`, `doctest_comparison_fixtures` and `a_tilde_fenced_usage_block_is_accepted_and_an_unclosed_block_is_refused` (`tests/docs_contract.rs:3262-3375`). `raf-okf/tests/usage_docs.rs:4-6` relies on them.
- The tests over Markdown outside `docs/` and `work/`: `rejects_a_link_split_across_lines_in_any_tracked_markdown_file` (:3383), `rejects_broken_link_outside_the_document_directories` (:2157) and `reads_markdown_outside_the_document_directories_with_or_without_frontmatter` (:3534).

**REWRITE TO (Task 7):** "The plan's Task 7 carries a disposition table for every `#[test]` in `tests/docs_contract.rs` at 762fa99. Each test is one of:
- `kept`, by name;
- `moved`, to `crates/raf-workplan/tests/<file>.rs::<name>`;
- `covered by`, naming the existing raf-okf fixture.

The doctest-pairs tests and the tests over Markdown outside the document directories are `kept`.

Check:
- every `moved` and `covered by` name exists and passes under `cargo test --locked -p raf-workplan` or `-p raf-okf`;
- every `kept` name passes under `--test docs_contract`;
- no row lacks a destination."

### F5: UNMAPPED REQUIREMENT (R22; document-model Verification is binding per L14)

> L70: "… `considered` with no PROPOSED block fails. One state fixture is accepted per state."

`workplan-document-model.md:229-231` requires "A state fixture per state: each accepted with matching content, and each rejected with a mismatch". The plan has rejection fixtures only for `delivered` and `considered`. Nothing rejects a `designed` capability with a PROPOSED block (`okf-profile.md:147`) or with non-empty bindings (`okf-profile.md:72-74`). Nothing rejects an `implementing` capability that no open record lists (`workplan-document-model.md:43`), or a `delivered` capability that an open record lists (`:44`). Task 3 (L109) names the open-record rule, but no acceptance check covers it.

**REWRITE TO (R22 acceptance):** "Each state has one accepted fixture and at least one rejected fixture. Every case runs through `Workplan::check` and names the rule:
- `considered` with no PROPOSED block, and with no open record listing it;
- `designed` with a PROPOSED block, with a non-empty binding list, and with no open record;
- `implementing` with no open record;
- `delivered` with a PROPOSED block, with an empty or comment-only section, with no code binding, with no test binding, and listed by an open record."

### F6: UNFALSIFIABLE CRITERION (R14, R20; Task 5)

> L65: "Sync twice gives the same bytes with no second change. Per kind: an edited row, a removed row and an added binding each make `check` fail naming the kind, and sync repairs it. Catalogue order: group order, slug order, `implementing` gaps."

> L125: "`catalogue`: state groups in order, slug order, gaps for `implementing`."

Every listed check compares sync's output with itself. A generator that wrote the wrong matrix columns, left out the "depended on by" row, or wrote catalogue rows without links or descriptions would pass all of them. The Contract fixes the content (`workplan-generated-docs.md:38-48`):
- a matrix row gives the bound item, its kind and the section it substantiates, sorted by path;
- the relationships table gives depends on, depended on by and the open changes that list the capability, each as a link;
- a catalogue row "links to the document and carries its description".

The last clause has no task line at all.

**REWRITE TO (R14 acceptance, plus a Task 5 catalogue bullet):** "For each kind, a `MemoryTree` fixture compares the exact bytes `Workplan::sync` writes with a literal expected block:
- a verification matrix with a test binding and an example binding, with columns item, kind and section, in path order, and one binding named in the Contract text so that its section is `contract`;
- a relationships table with a depends-on row, a depended-on-by row and an open-change row, each a relative link that `check` resolves;
- a catalogue with one capability per state, where each row links to its document and carries its description, in group order then slug order, and an implementing row shows an empty list and an unresolved binding as gaps.

Task 5 `catalogue` bullet adds: 'each row links to the document and carries its description'."

### F7: UNMAPPED REQUIREMENT (R12; document-model Contract `:94-96`)

> L63: "Queries in both directions are tested: capability to symbols and tests, symbol to capability, change to sections."

The Contract names four questions: "which code realises this capability, which capability explains this symbol, which tests address this guarantee, and which sections this change must update". "Which tests address this guarantee" asks about a section, not a whole capability. None of the three planned queries answers it, and Task 2 (L95, "It has queries in both directions") does not name the queries.

**REWRITE TO (R12 acceptance and Task 2):** "There is one graph query test for each of the Contract's four questions:
- capability to its bound code symbols;
- symbol to the capabilities that bind it, including through a re-export;
- `cap:<slug>#<section>` to the tests whose verification-matrix row substantiates that section;
- change to the sections in its `affects`.

Each test asserts the exact result set on a fixture where a decoy is excluded: an unbound symbol, or a test bound to a different section."

### F8: UNFALSIFIABLE CRITERION (Lessons; minor)

> L50: "Rule: run the gate on this repository and on katamari/purrdf documents where they exist."

No task runs the gate on katamari or purrdf, and "where they exist" makes the rule true even if nothing is run. The PR body could then claim a cross-repository demonstration that never happened.

**REWRITE TO:** Either "Rule: run the gate on this repository (R28, Task 7)", dropping the katamari/purrdf clause, or: "Task 8 runs `Gate::workspace(<absolute path>)` on the named katamari and purrdf checkouts and records the violation count and the first violations in the PR body as validation evidence. This is not an acceptance condition."

### Pass 1: checked and cleared

- **R13 and J5:** the `examples` role rests on raf-source's `example_crate` naming (`crates/raf-source/src/target.rs:368`), so the R13 row tests something real.
- **R15, R16 and J13:** `check`, `status` and `docs sync` are delivered as library entry points, `Workplan::check`, `Workplan::status` and `Workplan::sync`, plus `examples/docs_sync.rs` and `make docs-sync`. The issue's "Capabilities delivered" names only the model and generated docs. The binary is `cap:workplan-cli`, which has its own open record (`work/changes/workplan-cli.md`). This is not a descope. F1 covers the wording that has to follow from it.
- **R28 and R29:** J11 picks the stronger of the issue's two alternatives. R28's check runs the Gate on this repository and on the hook snapshot.
- **R23:** the Gate is used for real from `tests/docs_contract.rs` as a dev-dependency, and also on tempdir fixtures.
- **Declined enhancements:** JSON output and caching hide no issue work.
- **PLANNED-BY-REFUSAL:** none found. The hand-written index refusal and the `deferred` refusal are rules the spec states. Neither stands in for a requested capability.

Pass 1 verdict: NEEDS-REVISION (superseded by pass 2 below).

## Pass 2 (revision 2)

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`, revision 2. Line numbers below are revision-2 lines.
- **Pass:** 2 of 3.
- **Scope:** rule on F1–F8, and on the new text their fixes introduced. Text cleared at pass 1 and unchanged since is not reopened.

### Rulings on F1–F8

| Finding | Ruling | Evidence in revision 2 |
|---|---|---|
| F1 | **STILL-OPEN** | Fixed: L18, G14 (L46), G15 (L47) and list U (L219-224) replace the removal escape hatch with a closed list and a rule that blocks delivery; Lessons L65 is fixed too. Remaining: U leaves out a statement from the pass-1 table that no task builds: `workplan-document-model.md:104-106`, "`status`, the catalogue and the reconcile obligations report each one as a gap". Under G15 ("Any other unbuilt sentence blocks delivery") that sentence blocks Task 9, so R4 cannot be reached as planned. Separately, the delivered text names the operation `docs sync` (generated-docs `:50-51`, `:53`, `:59-61`; document-model `:115-116`), but only `Workplan::sync` and `make docs-sync` are built. U has no row for this, so under G15 Task 9 cannot decide whether those sentences count as built. |
| F2 | **DISCHARGED** | List D (L226-234) names `source-index.md:1488-1500`, `okf-profile.md:31-40, 173-174`, the AGENTS.md lines, `README.md:31`, `CONTRIBUTING.md:32-36`, the docs_contract comments, `usage_docs.rs:4-6` and `INVENTORY`. The R4/R30 row (L115) requires "Every file in list D is updated." Advisory only, not holding the verdict: also add `tests/docs_contract.rs:4-19` (the module doc, "What stays here is … the cross-document rules …", which G4 makes false) and `README.md:34-37` ("The API below is planned and is not available yet") to D. |
| F3 | **STILL-OPEN** | Fixed: G2 (L34) solves "the fixture really produces the kind" and the `#[non_exhaustive]` problem for raf-workplan's own kinds. Two gaps remain in the same mechanism. (a) G2 says only "raf-okf violations map into it". `raf_okf::Rule` alone has 36 variants (`crates/raf-okf/src/profile/violation.rs:31-189`), and `ErrorKind` has more. If several variants map to one kind, the `KINDS` coverage test passes with a handful of fixtures. L91's "one end-to-end fixture per raf-okf violation kind" is then unenforced, and the variant list from the pass-1 rewrite is still missing. (b) `kind: &'static str` is free-form, so a rule can emit a kind that is missing from `KINDS`, and the coverage test never checks it. |
| F4 | **DISCHARGED** | G4 (L36) keeps the doctest-pairs tests. Task 7 (L196) has a disposition table covering every `#[test]`, each with a destination. L200 adds a count check. The R18 row (L122) keeps the link checks over Markdown outside `docs/` and `work/`, with fixtures. L62 says ported tests keep their names. |
| F5 | **DISCHARGED** | Rule inventory L84: "Each state has an accepted and a rejected fixture", plus `proposed-in-designed` (L83) and `no-open-record` (L85). The spec's per-state accept and reject fixtures are now planned. The condensed text introduces a separate granularity problem, raised as N1. |
| F6 | **DISCHARGED** | G10 (L42) and "Generated content" (L93-108) fix the columns, the ordering, the section rule, and the catalogue link and description. The R14 row (L120) requires "The exact-output fixture per kind". |
| F7 | **DISCHARGED** | The R12 row (L118) names `code_of(cap)`, `capabilities_of(symbol)`, `tests_for(cap, section)`, `sections_of(change)` and `linking_sections(section)`, which covers all four Contract questions. |
| F8 | **DISCHARGED** | G19 (L51) drops the katamari/purrdf clause. The real-input demonstration is this repository (Tasks 7 and 9). |

### Rewrites for the open findings

**F1 REWRITE TO (add two entries to list U, L219-224):**
- "**document-model Contract, Binding resolution:** '`status`, the catalogue and the reconcile obligations report each one as a gap' becomes '`status` and the catalogue report each one as a gap; [reconcile](./workplan-reconcile.md) defines the matching `binding-missing` obligation.'" `workplan-reconcile.md:46` already defines `binding-missing` for an empty or unresolved binding, so the link is accurate.
- "**`docs sync` in delivered text** (generated-docs Contract `:50-51`, 'Sync' `:53` and 'Staleness' `:59-61`; document-model Checks `:115-116`): reworded to `Workplan::sync` (run in this repository by `make docs-sync`), so no delivered sentence names a command that does not exist."

**F3 REWRITE TO (G2):** "`Finding` is built only from an entry of `Finding::KINDS`, through a private constructor that takes a `Kind` constant defined beside `KINDS`, so no kind outside `KINDS` can be emitted. Every `raf_okf::Rule` and every `raf_okf::ErrorKind` variant at 762fa99 maps to its own kind in `KINDS`, and Task 1 lists them by name. The wildcard arm that `#[non_exhaustive]` requires maps to a separate `okf-unmapped` kind, which is the only kind the coverage test excludes, by name. The `KINDS` coverage test therefore requires one end-to-end fixture per raf-okf variant."

### New findings in revision-2 text

#### N1: UNMAPPED REQUIREMENT (R22, R26; acceptance not specific enough)

> L84: "`state-*` (one per state) | … `delivered`: no open record, no PROPOSED block, no empty required section (…), at least one code and one test binding. Each state has an accepted and a rejected fixture."

> L123: "| R22 | delivered rules | 3 | The `state-*` fixtures for all four states. |"

> L75: "`missing-reference` | `depends` and `changes` name existing capabilities; `moved:change:<slug>` names an existing change"

G2 checks coverage per kind, and each state only needs "a rejected fixture". So `state-delivered` is satisfied by a single fixture, for example a delivered capability listed by an open record. The issue's named failure, "a `delivered` capability has no PROPOSED blocks" (issue L23, R22), can then go without a fixture. Revision 1's R22 row named that fixture, and revision 2 dropped it.

`missing-reference` bundles three rules into one kind in the same way. As a result, two required checks no longer appear in any acceptance row:
- the binding Verification line "a `changes` entry for a missing capability fails" (`workplan-document-model.md:235-236`);
- the moved-outcome check (R9).

The R5–R10 row (L116) names only `deferred`.

The empty-section and binding-minimum halves of the delivered rule are raf-okf's `SectionEmpty` (`violation.rs:166`) and `BindingRequired` (`violation.rs:96`). Fixing F3 covers them.

**REWRITE TO:** "Each rule is its own kind in `Finding::KINDS`, so the G2 coverage test enforces one fixture per check failure. Split `state-delivered` into `delivered-proposed` and `delivered-open-record`, and split `missing-reference` into `depends-missing`, `changes-missing` and `moved-missing`.

R22 acceptance: named fixtures through `Workplan::check`, each beside the accepted delivered fixture:
- a delivered capability holding a PROPOSED block (`delivered-proposed`);
- one with an empty section, and one with a section holding only comments or only a generated block (raf-okf `SectionEmpty`);
- one with no code binding and one with no test binding (raf-okf `BindingRequired`);
- one listed by an open record (`delivered-open-record`).

R5–R10 acceptance adds the `changes-missing` and `moved-missing` fixtures."

#### N2: PLANNED-BY-REFUSAL (G13; R15)

> L45 (G13): "a document body holding `\r\n` is refused by `sync` and reported by `check` (`crlf-document`, fix "convert to LF"), with a fixture."

L18 makes the profile binding, and the profile accepts CRLF documents: "The frontmatter uses the line ending of the opening fence, `LF` or `CRLF`, throughout. The body is kept byte for byte." (`okf-profile.md:115-116`). The generated-docs Contract this PR delivers says "`docs sync` rewrites every generated block in the repository" (`workplan-generated-docs.md:53`).

For a CRLF document, G13 meets that sentence by refusing the document, and it adds a `check` failure on a document the profile accepts. Neither U nor D rewords the Sync sentence or the profile's line-ending rule. So the plan counts the refusal as delivering sync for that input, while the delivered contract keeps saying every block is rewritten.

**REWRITE TO (choose one and state it in G13):**
- **(a)** "Sync writes each block in the line ending of its document, and `\n` for the new `index.md`. A CRLF fixture asserts that the rewritten block uses `\r\n` and that every byte outside the block is unchanged. U rewords the generated-docs Sync clause 'with `\n` line endings' to 'in the line ending of the document, `\n` for the index'." This adds no new check failure.
- **(b)** Keep the refusal, and state it where it applies. Add to U the generated-docs Sync sentence ("a document whose body uses CRLF is refused; sync rewrites blocks only in LF documents"). Add `docs/architecture/okf-profile.md:115-116` to D, saying a body must be LF. The delivered contract and the profile then both state the refusal. Keep the `crlf-document` fixture as it is.

### Checked and cleared in the new text

- **G14 (`preview`):** built, with a fixture (Task 8, L206). The generated-docs Usage and Constraints statements about preview are now built. Advisory for the integration reviewer: `Workplan::preview(slug)` takes a capability slug and applies all of that capability's open changes. `workplan-cli.md:51` and `:59-61` define `docs preview <slug>` over a change slug, applying that change's blocks. Since the binary "holds no logic of its own" (`workplan-cli.md:98-99`), the CLI will need an entry point that takes a change slug. This is not one of my four grounds and does not block.
- **G9 (examples role, `documented == true`):** the issue asks only that bindings resolve, and today's `docs_contract` says "`examples` bindings only have to resolve" (`tests/docs_contract.rs:692`). G9 adds a role check on top of that. The check is looser than it could be, but an extra check is not a descope. Design B4 owns it.
- **Declined enhancements (L55-57):** `impact`, the two-way coverage queries and generalising the doctest-pairing rule belong to other capabilities with their own change records. None is issue scope.
- **G8 and G12 refusals:** these guard against a false green and a hand-written index. Neither stands in for a requested capability.
- **Real entry points:** Task 5 (L183) runs the shipped `docs_sync` example on a tempdir fixture, and Task 9 (L240) runs `make docs-sync` twice on this repository.

Pass 2 verdict: NEEDS-REVISION (superseded by pass 3 below).

## Pass 3 (revision 3, final)

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`, revision 3. Line numbers below are revision-3 lines.
- **Pass:** 3 of 3. This is the final pass, and there is no fourth.
- **Scope:** rule on the items still open after pass 2 (F1, F3, N1 and N2), and check the new text their fixes introduced. F2, F4–F8 and the pass-2 advisories stay settled, and unchanged text is not reopened.

### Rulings

| Finding | Ruling | Evidence in revision 3 |
|---|---|---|
| F1 | **DISCHARGED** | List U L226 covers `workplan-document-model.md:104-106`: "`status` and the catalogue report each one as a gap; [reconcile](./workplan-reconcile.md) defines the obligation it becomes." U L227 rewords every `docs sync` that names the operation, in both capability Contracts, both Checks lists and `okf-profile.md:55`, to "the sync operation (`Workplan::sync`)". G15 (L48) still blocks delivery on any other unbuilt sentence. The statements I found unbuilt across the three passes are all in U now, and U L229 also takes the new issue-reference rule into the Checks list. |
| F3 | **DISCHARGED** | G2 (L35) now says: "every `raf_okf::Rule` variant maps to its own kind `okf-<rule>`, every raf-okf parse-error kind maps to its own kind". The `okf-<rule>` naming rule makes the list of kinds follow directly from the 36 `Rule` variants (`violation.rs:31-189`) and the `ErrorKind` variants, so the list can be checked. Two further sentences close gap (b): "Findings are built only through `const` members of `KINDS`" and "every kind any fixture produces is in `KINDS`". The coverage test therefore enforces L92's "one end-to-end fixture per raf-okf violation kind". |
| N1 | **DISCHARGED** | L78 splits `depends-missing`, `changes-missing` and `moved-missing`, each with its own fixture, and Task 2's Check (L158) names all three. L85 has eight `state-*` kinds, "each with a rejected fixture". These include `state-delivered-proposal`, the issue's named failure (R22, issue L23), and `state-delivered-empty-section`. Under G2 the coverage test forces a fixture for each of them. |
| N2 | **STILL-OPEN → ESCALATE-TO-HUMAN** | The CRLF part is fixed. G13 (L46) now has sync write "each generated block in the document's own line ending", and U L228 rewords the Contract. The mixed-ending part is not. The fix brings in a narrower version of the same pattern: "A document with mixed line endings is refused by `sync` and reported by `check` (`mixed-line-endings`)". The profile governs only the frontmatter's line ending ("The frontmatter uses the line ending of the opening fence … The body is kept byte for byte", `okf-profile.md:115-116`). So a CRLF frontmatter with LF lines in the body, or a body with mixed endings, is a valid profile document. Three things go unstated as a result. First, the delivered Sync sentence "rewrites every generated block in the repository" (`workplan-generated-docs.md:53`) stays as written, while sync refuses these documents. Second, the document-model Constraint "this model adds rules between documents and the source" (`workplan-document-model.md:172-174`) does not cover a new single-document line-ending rule. Third, neither U nor D states the refusal. G13 itself already gives a deterministic answer for a mixed document ("detected from its first line break"), so the refusal is a contract choice, not a technical necessity. |

### N2: the decision for the user

The question is whether `check` and sync refuse a profile-valid document whose line endings are mixed. Choose one. Either is a single edit to G13 and list U, with no change to tasks or scope.

- **Choice A (recommended): keep the refusal and state it in the delivered contract.** It matches the repository's fail-hard rule, "a document that is stale, malformed or ambiguous is an error with a message that says how to fix it" (`AGENTS.md:195-196`).
  - **REWRITE TO (add to list U):** "**generated-docs Contract, Sync:** after 'A block of an unknown kind is an error' add 'and so is a document whose line endings are mixed; sync writes nothing to it.' **document-model Contract, Checks:** add the bullet 'a document whose line endings are mixed'. **document-model Constraints (172-174):** 'adds rules between documents and the source' becomes 'adds rules between documents and the source, and one line-ending rule for generated blocks'."
  - Keep the `mixed-line-endings` fixture.
- **Choice B: drop the refusal.**
  - **REWRITE TO (G13):** "A document with mixed line endings is not refused. Sync writes each generated block in the line ending of the document's first line break and leaves every other byte unchanged. The mixed fixture asserts exactly that." Remove `mixed-line-endings` from the rule inventory (L90) and from G2's `KINDS`. The contract and the profile need no further change.

Until one choice is recorded in the plan, the delivered Sync sentence and the plan's behaviour disagree on one class of profile-valid input.

### Checked and cleared in the new text

- **G20 (L53):** quotes the owner's decision verbatim and limits it to the `cap:source-index` citation. It narrows no issue scope.
- **G14 (L47) and the Task 8 fixture (L207):** `Workplan::preview(change)` is now keyed by change slug, as the CLI spec requires (`workplan-cli.md:51, 59-61`). The fixture includes a block from another change that is left as it is, so it can fail. The R-row at L131 still says `Workplan::preview(slug)`. That is a naming slip, not a finding.
- **G4 (L37):** the checks on Markdown outside `docs/` and `work/` run through `raf_workplan::plain::check` and are recorded as "kept". No coverage is lost.
- **List D (L231-240):** now includes `tests/docs_contract.rs` 4-19 and `README.md:34-37`, the two pass-2 advisories.
- **Declined enhancements (L57-58):** `impact` and the two-way coverage queries belong to capabilities with their own change records. Neither is issue scope.
- **Advisory for the design and constraints reviewers, not one of my grounds:** three kinds in the L85 inventory repeat raf-okf rules that G3 says raf-workplan does not duplicate. `state-delivered-empty-section`, `state-delivered-no-binding` and `state-designed-bindings` repeat raf-okf's `SectionEmpty` (`violation.rs:166`), `BindingRequired` (`:96`) and `BindingNotEmpty` (`:89`). G2 also does not say where the wildcard arm required by `#[non_exhaustive]` sends a future raf-okf variant. Each kind still has a fixture, so neither affects completion.

### Required user decision

N2 needs a decision: Choice A (recommended) or Choice B, as set out above. Every other completion finding from the three passes is discharged.

VERDICT: ESCALATE-TO-HUMAN
