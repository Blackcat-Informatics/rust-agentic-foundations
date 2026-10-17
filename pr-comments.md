# PR #20 — comments and review threads

4 comment(s). **Every one needs a disposition** in
`review-debt.md`: addressed (with the commit hash) or declined (with the
reason posted in reply). Bot comments count.

## 1. paudley — 

# Plan: the rest of the planning essay (entity history, evidence claims, prose assessment, two-way coverage, current rationale)

Revision 4 (integrates the five lens reviews of revision 1 and the rechecks of revision 2). Authoritative plan.
STAGE = `.stage/m2-rest-of-the-planning-essay-entity`. Reviews: `reviews/plan-*.md`; the enhancement
report's numbered items (E1..E3 transformation, then the leverage/utility/robustness list) are
referenced where adopted.

## Issue summary

Carry five `considered` capabilities, each with a change record under `work/changes/`, to `delivered`
through the workplan lifecycle, using the shipped `cargo workplan` itself: `cap:workplan-entity-history`,
`cap:workplan-evidence-claims`, `cap:workplan-prose-assessment`, `cap:workplan-two-way-coverage`,
`cap:workplan-current-rationale`. Acceptance: each reaches `delivered` through `change complete`; its
record is deleted on completion; no deferrals (anything leaving an item is `moved:` to a tracked change).

## Baseline, goals, governing decisions

* AGENTS.md / CLAUDE.md: floating nightly in hosted CI, MSRV 1.98.1, workspace lints, no `#![feature]`, no
  fallback where the design says fail hard, no tracker numbers or process wording in committed files, no
  attribution, `.deficiencies` empty, every rule has a negative fixture and every refusal a passing neighbour,
  documents state the current contract only, commit subjects follow the conventional form.
* SECURITY.md trust model: workspace is trusted build input; no shell; argv vectors; document text reaches a
  process only after resolution and a stated pattern, in the forms SECURITY.md states (the three existing ones and the `-S<identifier>` argument of D18). The one further process, the trusted argv of D4, comes only from the command line after `--` of `evidence record` and never from document text. SECURITY.md is edited in the same
  tasks as the new git commands and the new file writes (Task 4, 5, 6, 9).
* Reused, not duplicated: `exec` typed invocations (new rows only), `git` module, `Gate::open`,
  `Workplan`/`Graph`, `section_digest`, `Lifecycle`, `obligation`/`reconcile`, the Finding/Obligation closed-kind
  coverage tests, `Workplan::write` stage-then-rename, the CLI render/exit layers.
* **Owner decision on the record (coverage scope): "Workplan crates only (Recommended)".** Reading applied:
  `check` ENFORCES coverage for the crates listed in the `scope` key of the repository's Coverage record
  (raf-workplan and cargo-workplan); the `coverage code|tests` REPORTS cover the whole workspace and name every
  gap (raf-okf and raf-source gaps are visible in the report, not enforced by `check`). A repository with no
  Coverage record gets one `coverage-record-missing` finding whose fix says to add the record (`scope: []` is
  the visible opt-out); nothing is skipped silently. Widening the scope is an
  edit of the record, tracked by the strict-mode issue; it is not claimed here.
* **Standing owner decision:** hosted CI counts as the evidence for `cross-check`; every other `make` gate is
  run locally and individually.
* The nominal dependency on the self-gate issue is not waited for: the tool this work needs (`change reconcile`,
  `change complete`) exists; the census self-gate part is not claimed.

## Decisions

* **D1 Record documents (E1 trace model).** Three new profile types in `raf-okf`; `Evidence` and `Assessment`
  share one **trace** shape: `subject`, `inputs` (rows `{kind, name, digest}`), a type-specific `verdict`, `by`
  (`human|agent`, never a name), plus type-specific keys; `Coverage` has its own keys (`scope`, `exclusions`). `Evidence` at `work/evidence/<id>.md`; `Assessment` at
  `work/assessments/<id>.md`; `Coverage` (singleton) at `work/coverage/default.md` (a directory, not a file in
  `work/`, because the loader reports other files there as nested). Flat directories, slug ids. `DocType`,
  `Rule::ALL`, `ErrorKind::ALL` grow: this is not source-compatible for exhaustive matches (crates are
  unpublished, 0.1.0; stated in the PR). A capability-named change record `okf-profile` lists `cap:okf-profile`,
  `cap:workplan-document-model` and is opened in Task 1; `docs/architecture/okf-profile.md` and the bound
  key-order test are updated in the same task.
* **D2 Claims.** A statement depends on evidence through `<!-- raf:claim evidence=<id>[,<id>...] -->` (a closed `key=value` attribute list, so further keys can be added without a second marker form) on its own
  line directly after the paragraph or list item it supports. Parsed in `raf-okf` `body/marker.rs` (fence
  skipping, spans, `section_of`); any other `<!-- raf:` comment that is not `raf:generated:*` or a claim is a
  finding (a misspelling never silently stops being a claim). Claims inside PROPOSED blocks are not claims
  until resolved. Unknown evidence id: finding naming file and line.
* **D3 Shared digest (E3).** `raf_workplan::digest`: in-crate SHA-256 with test vectors over a
  domain-separated, length-prefixed encoding; `section` (SectionDigest text, generated blocks blanked, LF),
  `symbol` (source-index span text, CRLF normalised, de-duplicated by source key), `record` (record text).
  Records persist one digest per input; obligations name the input that moved.
* **D4 Evidence (owner decision on the record: "Run the command (Recommended)").**
  `evidence record --id <id> --for cap:<slug>... [--limit name=num]... [--revision <commit-id>] [--copyright
  --licence] -- <argv...>` RUNS the argv after `--` and writes one record holding the command, the revision (when
  given), the verdict (exit status 0 = pass, otherwise fail, or fail when a printed value exceeds its limit),
  the measured values and the limits. The run uses a new trusted process variant in `exec` (`Program::Trusted`):
  the argv is the caller's own command, executed with no shell, stdin null, the 1800 s build bound, bounded
  output (16 MiB per stream) and the workspace as working directory. It is trusted build input exactly as the
  workspace's own build commands are; this is the one further stated process (the argv comes only from the command line, never from document text) and SECURITY.md names it. Values are the
  stdout lines of the form `name=number`; other output is discarded and never persisted. `--from-binding <bound
  path>` is a second source of the verdict: it runs the named test or example binding of a `--for` capability
  through the existing `bound::Executor` (a path matching no such binding exits 2 and nothing starts). Inputs
  recorded: the digests of the code bindings (symbol texts) of each `--for` capability at record time (`--for`
  fixes what the evidence covers; the claim marker is the citation; both are required and a mismatch is a
  finding). The record type is named `EvidenceRecord` in code to keep it apart from `bound::Evidence`. Several
  results per claim: all must hold (D2).
* **D5 Staleness.** `evidence-stale` = a recorded input digest differs from the digest now (no revision, no
  git, no reachability dependency: robust to squash merges and shallow clones). The record keeps the command text and the optional
  `revision` (validated against the commit-id pattern) as informational fields; neither is used for decisions,
  and neither reaches a process.
* **D6 Evidence change (reconcile).** `evidence-changed` (also a limit present at the merge base and absent now): against the merge base (existing `git` module,
  one `cat-file` per record): verdict changed pass->fail, a value now above its limit, or the record removed
  while present at the merge base. `claim-unsupported`: claim names no existing record, a failing record, or
  an over-limit value; a removed record yields `claim-unsupported` (and `evidence-changed`).
* **D7 Obligation kinds (closed set grows; reconcile Contract edit):** `claim-unsupported`, `evidence-stale`,
  `evidence-changed` (process kind), `assessment-missing`, `assessment-stale`, `assessment-rejected`.
  Static kinds are raised by one `trace` source in `Workplan::reconcile`.
* **D8 Assessments.** Verdicts `accepted|rejected`; whole section; an agent assessment satisfies the gate by
  itself (`by` states which); the record stores no personal name. Inputs: the section digest, the bound
  symbols the section names in code spans (all code-bound symbols of the capability when it names none), the
  evidence records its claims name. Completion-gate condition 6: a current
  `accepted` assessment for each authored section in a change's `affects` (generated-only sections exempt);
  `assess` replaces the record for the same subject. Existing scenario and `complete` fixtures get one shared
  helper that records assessments.
* **D9 Coverage.** Public surface = `Workspace::exported_items(target)` (a new raf-source query: items
  reachable from the crate root by public re-exports), delivered through a change record listing
  `cap:source-index`. Binding an item covers its descendants (impl items, nested items); `doc(hidden)` items are out by default, `cfg(test)` unit tests count like other tests and are excluded
  by prefix with a reason where they only guard internals. An unbound test is a gap like an unbound item. Exclusions are
  `{item, reason}` entries (an item path or a `<module>::*` prefix below a crate root, non-empty reason; a crate-root prefix is refused: take the crate out of `scope` instead); an exclusion matching
  nothing or with an empty reason is a finding naming the file. Every existing fixture and `complete`'s
  after-removal check run with a shared helper that writes a Coverage record whose scope is empty. `check` enforces the scope crates; reports cover the workspace.
* **D10 Current rationale.** A decision page is an architecture page with a `## Decision` heading; it must have
  a non-empty `## Conditions to revisit` list. A capability Explanation holding a `### Decision` subheading
  must be followed by `### Conditions to revisit`. An architecture page no document links to is reported.
  Whether a condition is met is judged by a person with the history command; nothing is guessed from wording.
* **D11 History.** Ids `cap:`, `change:`, `arch:`, `evidence:`, `assessment:` (matched against loaded documents, never passed to git); commits are de-duplicated across sources.
  Document form adds `--name-only` with record separator `0x1e`, giving the paths each commit touched; symbol
  form uses `-L` and reports the file as the path. `git rev-parse --is-shallow-repository`: refuse on `true`
  (error naming the missing history). A binding that no longer resolves: pickaxe
  `git log -S<final path segment> -n 1 --format=... -- <crate dir>` finds the commit that removed it; the report
  names that commit and its parent (`git rev-parse <commit>^`) as the last commit in which it resolved; the
  limits of the heuristic are stated in the Contract. Newest first; `--since YYYY-MM-DD` before the final `--`.
* **D12 Process rules.** All new `git` rows through `exec::Invocation` with validated placeholders (an
  identifier pattern for `-S`), 16 MiB bound, `LC_ALL=C`. The tool starts no process other than `git`,
  the already-built `cargo` rows and, for `evidence record --` only, the caller's own argv (D4); a killed or
  failing run records `fail`, a process that cannot start writes no record and exits 2. The argv runs exactly as
  given (no `--frozen` is added to a `cargo` inside it); the environment is the caller's own with `LC_ALL=C`
  added only for `git`; the `cargo-workplan` binary itself starts no other process. Parsing fails hard: a
  `--limit` whose name the command never printed, a printed value that is not a number, a name printed twice,
  and output past the bound are each an error (exit 2) with a negative fixture and a passing neighbour. The stored
  `command` field is never run: `Program::Trusted` can only be built from the command-line argv (a compile-fail
  doctest pins that no loaded document or record can build it). SECURITY.md lists the new commands and writes.
* **D13 Writes.** New record files are created exclusively (`create_new`), staged-then-renamed for replacements;
  `evidence record` and `assess` take `--copyright`/`--licence` so records carry SPDX lines.
* **D14 Items and moves.** Every item of the five records ends `implemented`, or is `moved:change:<slug>` to a
  new tracked record created with `change new` (one per moved item). A claim found untrue as written is
  reworded to what was built (with the reason in the PR body; any part of the claim that is not built is moved to a tracked record, so a rewording never hides a cut) or moved; nothing is `dropped`. The plan expects
  zero moves.
* **D16 Delivered capabilities touched.** The five records are widened (`changes`/`affects`) to also list the
  delivered `cap:workplan-reconcile` (six obligation kinds), `cap:workplan-complete-gate` (condition 6, the new
  Changed-by relation), `cap:workplan-cli` (new commands, exit codes, JSON kinds), `cap:workplan-document-model`
  (new record documents and check rules), `cap:okf-profile` and `cap:source-index` (through their own
  `okf-profile` and `source-index` records). Each is `implementing` while a listing record is
  open and returns to `delivered` when the last completes; Task 11 verifies their text, tables and states.
* **D18 Pickaxe form and text hygiene.** The `-S<identifier>` argument is a fourth stated form (a validated
  identifier inside one `-S` argument); SECURITY.md and the entity-history Constraints name it, and the commit id
  read back from `git` is checked against the commit-id pattern before `<commit>^` is built. No committed text
  (documents, comments, CHANGELOG, commit messages) names an issue, a mode or future work.
* **D19 Coverage ordering.** `work/coverage/default.md` is added in Task 7 together with the bindings and exclusions
  its enforcement needs, so every commit passes the pre-commit check; each capability is bound in the commit that
  adds its public items. Task 3's "no change when absent" applies to the evidence and assessment directories only.
* **D17 Helpers.** One `Lifecycle::process_obligations` helper assembles the process-backed obligations for
  `reconcile` and `complete`; type-list messages derive from `DocType::ALL`; `assess --context` output is
  identical for `human` and `agent` (tested).
* **D15 Commit discipline.** Every commit passes the pre-commit gate on the committed tree, including
  `tests/docs_contract.rs::check_inventory`, which requires a `work/changes/<slug>.md` NAMED AFTER every
  non-`delivered` capability and flags any file in a `DocType::ALL` directory the table does not list, and the
  state rules (a PROPOSED block only while `implementing` or `considered`; `delivered` beside an open record
  only inside the final `complete` commit). Owner-visible reading, no policy change: the repository policy stays.
  Consequences, applied in the FIRST task that changes a delivered capability: that capability is set
  `implementing`, a capability-named record is opened with `change new`, and the Inventory table gains a
  `records` list edited in every task that adds a record directory entry (evidence, assessment, coverage records).
  The six records of D16 are capability-named: `okf-profile`, `source-index`, `workplan-document-model`,
  `workplan-reconcile`, `workplan-complete-gate`, `workplan-cli`; each is completed in Task 11 with the five.
  Contract table edits that the pinned tests require (for example the evidence column of the reconcile obligation
  table pinned to `ObligationKind::ALL`) land in the same commit as the code that grows the list, inside the
  PROPOSED-block workflow of that capability.

### Declined (with reasons)

* A generated `claims` block, re-record limit inheritance, and `evidence-unused`/`assessment-orphan`/`exclusion-redundant`
  checks: none is in the Contract; limits are given explicitly on each record because a record is a result.

* Statement-level `tests=`/`examples=` claim support and machine-evaluated `condition-met` obligations (E2):
  the Contract defines tests->claims at binding granularity and conditions as human-judged; statement-level
  support belongs with the strict-mode issue, which lists claim coverage.
* A public obligation-source trait: the `trace` source is one internal function.
* Persisting execution output or logs: the Contract keeps a result, not a judgement, and stored output would grow the repository and age without a digest to expire it.
* A per-item digest on coverage exclusions: an item digest changes on any edit, so exclusions would reopen more often than they rot; an exclusion that matches nothing is already a finding.

### Edit list of PROPOSED text (so every PROPOSED claim is built or listed here)

| Capability | PROPOSED sentence | Edit | Reason |
| --- | --- | --- | --- |
| evidence-claims | optional command and revision | the record states the command and, when given, the revision; `--from-binding` is a second source of the verdict | D4 |
| evidence-claims | "Evidence for a revision other than the current one is reported as stale" | stale when a recorded input digest changed | D5 |
| entity-history | git rows | `--name-only` record form; shallow check; removed-binding pickaxe | D11 |
| entity-history | "last commit in which it resolved" | commit that removed it plus its parent | D11 |
| prose-assessment | record "names ... the assessor" | names only `by: human|agent` | D8 |
| two-way-coverage | "Every public item is bound ... or excluded" | `check` enforces the record's scope; reports cover the workspace; no record gives one `coverage-record-missing` finding and `scope: []` opts out visibly (the closed finding list and its test gain the kind); `cfg(test)` tests count | owner decision |
| entity-history | history follows a file move | the `-L` log ends where the file's history is renamed; stated in Constraints | D11 |
| reconcile, complete-gate, cli, document-model, okf-profile, source-index | contract tables and text for the new kinds, condition 6, commands, record documents, exported items | edited in the tasks that change the code (D15) | D16 |
| all five | "Open questions:" lists in Purpose | removed when the questions are resolved by the text; checked by a grep fixture in Task 11 | resolution |
| current-rationale | "page or an Explanation ... conditions to revisit" | decision page / `### Decision` rule | D10 |

## Prior art

None in this repo for the issue (no commits, branches or PRs cite it). Precedent: the merged lifecycle work —
typed process rows, closed-kind coverage tests, `Gate::open`, `Lifecycle`, the CLI layers, and the commit order
for delivering capabilities with the tool. Recurrence of defect classes: only `none`; UNKNOWN beyond that.

## NEEDS-JUDGMENT dispositions

1. The nominal dependency: recorded above; census self-gate part is not claimed.
2. The docs->code and docs->tests checks are the delivered document-model checks (baseline extended here).
3. "Inputs" of an assessment: D8. 4. "Shared architecture pages" and conditions form: D10.
5. The scaffold premise (documents and records exist on main) is true.

## Completeness contract

| R-id | Task | Executable acceptance check |
| --- | --- | --- |
| R1 | 4, 9, 10 | fixtures: rename followed (`--follow`) with touched paths; a function changed in one of several commits reports only that commit; deleted symbol reports the removing commit and its parent; `cap:`, `change:`, `arch:`, `evidence:` and `assessment:` ids; `-rf.rs` and `--output=x.rs` paths; subject containing 0x1f; 16 MiB bound error; git 2.29 => exit 2; id or `--since` starting with `-` or malformed date => exit 2 and no git started; shallow clone error; newest-first order; `--since` before the final `--` (stub argv); table and JSON entries equal; tree byte-identical; real use: `cargo workplan history cap:workplan-reconcile` and `history arch:normal-form` on this repository print their commits |
| R2 | 5, 9, 10 | fixtures: claim + record views (`evidence claims`, `evidence for`); a passing record replaced by an over-limit one => `claim-unsupported` and `evidence-changed` lines naming the value; a removed record => `claim-unsupported` with `evidence-changed`; changed bound code => `evidence-stale` naming the symbol; the written record holds command, revision (when given), verdict, values, limits and input digests, and recording changes no file under `docs/`; unknown evidence id => finding naming file and line; misspelled `raf:` marker => finding; `evidence record -- <argv>` runs the argv without a shell (stub proves argv, stdin closed, bounded output, timeout kill), writes exactly one file, derives the verdict from the exit status and printed `name=number` values against the limits (an over-limit value is `fail`), persists no output text, and a failing command still records a `fail`; the four parse failures of D12 (a limit the command never printed, a non-number value, a name printed twice, output past the bound) each exit 2 with a passing neighbour; `--from-binding` runs only a bound test or example of a `--for` capability and a path matching none exits 2 with nothing started; the shared `exec` trusted variant is never reachable from document text; exit codes 0/1/2; real use: this repository holds at least one claim and one evidence record and `change reconcile` is empty for them |
| R3 | 6, 9, 10 | assessment current for an unchanged section; invalidated by a section edit, a bound-symbol change and an evidence change, each naming the input; regenerated block leaves it current; digest identical across runs and SHA-256 vectors pass; `assess <target> --context` prints section, bound symbols, evidence, identically for human and agent; `--verdict accepted|rejected --by human|agent` writes one record; `--list` shows currency; `assessment-missing`/`assessment-stale`/`assessment-rejected` lines appear in `change reconcile`; `change complete` refuses an affected authored section without a current accepted assessment (condition 6) and a rejected one never satisfies; real use: this repository's affected sections carry assessments |
| R4 | 2, 7, 9, 10 | fixtures: bound vs unbound public item; private item absent; re-export through a private module is public; descendant of a bound item covered; bound vs unbound test; a `doc(hidden)` item is absent from `coverage code`; an unbound `cfg(test)` unit test appears in `coverage tests` and is absent once a `<module>::tests::*` exclusion with a reason lists it; exclusions with empty reason and unmatched item fail naming the file; reports identical across runs; `check` enforces the scope crates only and reports one `coverage-record-missing` finding (fixed by adding a record, `scope: []` as the opt-out) when no record exists; reports name raf-okf and raf-source gaps; every existing fixture (including `complete` and scenarios) stays green through the shared Coverage helper; real use: this repository's `check` is clean with scope = workplan crates and exclusions with reasons |
| R5 | 8, 10 | fixtures: decision page linked passes; unlinked page named; decision page without conditions named; capability Explanation `### Decision` without `### Conditions to revisit` named; link to missing page or heading fails; real use: the two architecture pages here carry `## Decision` and `## Conditions to revisit` and are linked from capabilities, `check` reports them clean |
| R6 | 11 | (also the delivered capabilities of D16: states `delivered` at HEAD, tables regenerated, text states the new contract) `cargo-workplan change reconcile <slug>` prints an empty list (exit 0) and `change complete <slug>` prints `completed: removed work/changes/<slug>.md` (exit 0) for each record; each capability has `state: delivered` at HEAD; outputs in `validation.md` |
| R7 | 11 | the records are absent at HEAD and `git log --diff-filter=D -- work/changes/<slug>.md` shows each deletion commit |
| R8 | 11 | every item `implemented` or `moved:change:<slug>` to an existing tracked record; none `dropped`; reworded claims are listed in the PR body |
| R9 | 11 | the lifecycle is run with the tool (reconcile then complete), commands recorded |
| R10, R11, R13 | all | one task set per capability; the five documents and records existed on main |
| R12 | — | nominal dependency, recorded; no check |

## Task coverage rows

Tasks 1 and 3 are checked by the R-row fixtures they enable (record-type fixtures, digest vectors, singleton rule); Task 11 greps that no 'Open questions' list remains in the five documents.

## Tasks

Each task: focused tests (command named in the task report), independent review `tasks/TN-review.md`, commit by
explicit path with a conventional subject, push, issue update. Whole-task scope; no deferrals.

## Task 1: raf-okf record types and claim markers
(Also: `cap:okf-profile` and `cap:workplan-document-model` set `implementing`; records `okf-profile` and `workplan-document-model` opened; Inventory `records` list.)
`Evidence`, `Assessment`, `Coverage` types (D1), claim marker parsing and unknown-marker finding (D2), violations,
tags, `Rule::ALL`/`ErrorKind::ALL`, usage docs, `docs/architecture/okf-profile.md` and the key-order test; open the
`okf-profile` and `workplan-document-model` records. Acceptance: one fixture per failing rule and the profile-page test passes.

## Task 2: raf-source exported items
(Also: `cap:source-index` set `implementing`; record `source-index` opened; Inventory `records` list.)
`Workspace::exported_items(target)` and docs; record listing `cap:source-index` opened. Acceptance: fixtures for
re-export through a private module, glob re-exports, `pub(crate)`, `doc(hidden)`.

## Task 3: raf-workplan records, digest and loading
Loader for `work/evidence`, `work/assessments`, `work/coverage` (singleton rule), graph nodes/edges, `digest`
module with vectors, exclusive-create writer, `check` integration with no change when absent. Acceptance:
digest vectors and determinism; documents load; singleton violation fixture.

## Task 4: entity history
`history` module over `exec`/`git` (D11, D12): document and symbol histories, ids, bounds, shallow refusal,
removed-binding pickaxe, table/JSON; SECURITY.md update. Acceptance: the R1 fixtures.

## Task 5: evidence and claims
(Also: `cap:workplan-reconcile` set `implementing`, record `workplan-reconcile` opened, and its obligation table gains the new kinds in the same commit as `ObligationKind::ALL`; `cap:workplan-complete-gate` is flipped and its record opened in Task 6.)
`evidence record|claims|for` (D4), static `trace` obligations and `evidence-changed` (D5..D7), SECURITY.md.
Acceptance: the R2 fixtures.

## Task 6: prose assessment
(Also: `cap:workplan-complete-gate` set `implementing`; record `workplan-complete-gate` opened; condition 6 and the table edits in the same commit; the reconcile table gains the assessment kinds with `ObligationKind::ALL`.)
`assess` functions, trace obligations, completion-gate condition 6 and the shared fixture helper (D8).
Acceptance: the R3 fixtures and all existing `complete` and scenario fixtures green.

## Task 7: two-way coverage
Coverage computation, exclusions, `check` enforcement by scope, reports (D9). Acceptance: the R4 fixtures.

## Task 8: current rationale
Rules and `check` integration (D10). Acceptance: the R5 fixtures.

## Task 9: cargo-workplan commands
(Also: `cap:workplan-cli` set `implementing`; record `workplan-cli` opened.)
`history`, `evidence record|claims|for`, `assess`, `coverage code|tests`; text/JSON, `-q`, exit 0/1/2, both argv
shapes, validation before any spawn, read-only byte identity; README/AGENTS/SECURITY/CHANGELOG. Acceptance:
binary integration tests per command with exit 0, 1 and 2.

## Task 10: real use on this repository
Author the Coverage record (scope and exclusions with reasons), at least one claim with an evidence record, the
architecture-page sections and links, and assessments; run every new command; the repository `check` is clean.
Acceptance: the real-use rows of R1..R5, outputs recorded.

## Task 11: reconcile and deliver with the tool
Commit order per D15: (1) the five capabilities are set `implementing` (`INVENTORY` in the same commit) and their bindings, added by Tasks 7-10 per D19, are confirmed complete (the D16 capabilities already are `implementing`, flipped by Tasks 1, 2, 5, 6 and 9); (2) Contract edits (the edit list
above) as PROPOSED blocks; (3) resolved text, `change reconcile` empty for every record, assessments recorded;
(4) working tree only: set `delivered`, run `change complete` per record one at a time, commit state edits,
rewrites and record deletions together; the six D16 records are completed
the same way. Acceptance: R6..R9 rows.

## Task 12: PR, publication and handoff
Every `make` gate run individually (cross-check by hosted CI per the standing decision), artifact inventory,
completeness review against this plan, `stagectl pr-create` (`Closes #10`, validation, residual risks, owner
readings D1/D4/D5/D8/D9/D10/D11 and the edit list), plan and least-confident note on issue and PR.

## Carried items from task reviews

* Task 1 reviewer: narrow the architecture-page sentence "a misspelt marker is never read as a plain comment" to non-`raf:generated:` comments (done in the Task 3 commit that touches that page, or earlier); Task 3 moves the record fixtures to their real directories; Task 5 states that `Number` accepts no exponent (a printed `1e-3` fails hard under D12) and refuses a `--for` capability with no code binding clearly; Task 7 maps crate names (`raf-okf` and `raf_okf`); Task 11 must handle `reconcile okf-profile` reporting an empty `bindings.examples` and the `verify` failure of the `no_run` `raf_workplan::Gate` example in `workplan-document-model` (tracked by issue 17).
* Task 2 reviewer (advisory): add a fixture pinning `binding_site` to the first carrying glob; Task 7 coverage must treat the loud differences of `exported_items` (the several-binder carry refusal, the `pub(crate)` item next to a carried re-export, the unmodelled visibility rule) as documented limits of the surface it reads; Task 11: empty `bindings.examples` on source-index and okf-profile, the line-1254 tense wording, and the doctest order.
* Task 4 reviewer (advisory, carry to the Task 4 text edits in Task 11 and Task 9): the Contract should say "a merge whose result equals one of its parents is not an entry"; the mixed path bases (document paths relative to the repository top, symbol files relative to the workspace) belong in the Contract Constraints; partial (blobless) clones fetch objects lazily, which contradicts "reads Git and writes nothing" — state it; the "stub git on PATH" Verification wording is rechecked when the command lands in Task 9.
* Task 5 reviewer (carry to Task 9): clap must take the words after `--` as trailing arguments consistently with `TrustedArgv::from_command_line` (pass `std::env::args_os()` itself) and refuse an option whose value is `--`; test the command runs through the binary. `Number::value()` stays public for display only — never compare with it.
* Task 6 reviewer (carry): reword the prose-assessment Constraints to "only in prose or in fenced code" (optionally read paths in fenced `rust` blocks as weak names); Task 11 rewrites the complete-gate frontmatter description / index row ("five completion conditions") and the delivered Order paragraph (conditions 1 to 4) and aligns the generated-only wording; Task 12 PR body lists the prose-assessment item rewording (assessor named by kind only) and the id-rule item; the shared test helper skips sections it cannot assess — scenarios assert the count.
* Task 7 (done) consequences for later tasks: four considered capabilities (entity-history, evidence-claims, prose-assessment, two-way-coverage) are already `implementing` with bindings; `workplan-cli` and `workplan-generated-docs` (formerly delivered) were reopened to `implementing` with records — Task 9 WIDENS the existing `workplan-cli` record (change new would refuse that slug) and `workplan-generated-docs` is a SEVENTH record Task 11 completes; Task 8 sets `current-rationale` to `implementing` when it binds; Task 11 restores the tests/roadmap.rs guard to require something `delivered`; a module binding hashes the whole module, so any edit in a bound module (tests included) stales dependent evidence/assessments — expect re-records in Tasks 10/11; budget 2-3 hours in Task 11 for ~1400 bound tests across the completions (or run records in sequence and accept long runs); advisories: pin the crate-root digest behaviour with a test, normalise `super::X` re-exports, bind two git::tests to document-model.
* Task 8 reviewer (carry to Task 11): `docs preview` exits 2 for four changes because a fenced block sits inside a PROPOSED quote — workplan-two-way-coverage (docs/capabilities/workplan-two-way-coverage.md lines 157-162), workplan-entity-history (workplan-entity-history.md 112-116), workplan-prose-assessment (workplan-prose-assessment.md 151-153), workplan-complete-gate (workplan-complete-gate.md 463-467 inside the block at 459); `check` accepts them and `change complete` counts unresolved blocks without preview, so Task 11 resolves these four blocks by hand as plain text. Also (advisory): setext headings are invisible to the rationale rules (state "ATX headings only" in the Contract); a one-line comment on `has_unloaded_documents`.
* Task 9 reviewer (advisory, carry): reword the `HistoryError::SymbolLog` message to "`git log -L` could not read the lines from HEAD" with "commit the file" as a conditional suggestion; one Contract sentence that `assess <section> --list` exits 2 when the section can no longer be assessed while a bare `--list` still shows it; every re-record in Tasks 10/11 passes --copyright/--licence (a replacement keeps only the SPDX lines it is given).

## 2. paudley — 

## Least confident about, and what you may not know

* **Hosted CI has never run on this branch.** The new tests use real git repositories and real cargo fixtures on all three platforms; the last time that pattern met hosted CI it found a Windows-only target-directory collision. Expect the Windows and macOS jobs to be the real test.
* **Module bindings hash the whole module.** Any edit in a bound module (tests included) makes dependent evidence and assessments stale. That is the intended meaning of "the code the statement is about changed", and it will cost re-recording work on every later change.
* **All 54 assessments are by one kind of assessor (an agent).** They fixed many false statements, but no human has read the sections.
* **`evidence record` runs arbitrary commands from the command line** (trusted workspace input, like a build). The compile-fail doctests pin that documents cannot reach that process.
* **Plan decisions changed during the work** (each recorded in the PR body): records and coverage are digest-based instead of revision-based, `evidence record` runs the command, four reopened capabilities, a seventh reopened record (`workplan-generated-docs`) beyond the six planned, and an extra reopening of entity history after the completion audit.
* Issue 17 stays open for its platform scope; strict-mode ideas are in issue 16 and code-review records in issue 19.

## 3. paudley — 

# Remediation plan (Stage 2) — issue 10 / PR 20

Inputs: gap-analysis.md (BLOCKED), reviews/security.md (BLOCKED), reviews/completion-audit-stage2.md (BLOCKED). Path: full. Hosted CI is green on 43c7e02 (all 12 jobs). No review threads.

All fixes go through the workplan lifecycle on the capabilities they touch: a capability whose text or bound behaviour changes is reopened (`implementing`, capability-named record via `change new`), its text and tests are changed, affected sections re-assessed, then it is completed again with `change complete`. Capabilities whose bound modules merely changed get their stale assessments/evidence re-read and re-recorded (as in "Task 12 stale cleanup"); the stale probe over all 12 capabilities must end clean.

## Gaps and fixes

| Id | Sev | Gap | Fix | Acceptance (executable) | Capability |
| --- | --- | --- | --- | --- | --- |
| S1 | HIGH | `record::replace`, `evidence::target_fits`, `record_assessment` follow directory links inside the root and can overwrite a document (reproduced with `work/coverage -> ../docs`, `work/evidence -> ../docs/capabilities`) | one shared per-directory containment check (as `files::create_new`) used by create, replace and both pre-checks; refuse a link or non-directory at `work` and `work/<type>` | fixtures: linked `work`, `work/evidence`, `work/assessments`, `work/coverage` each refused for create AND replace AND the pre-checks, tree byte-identical, each beside a passing real-directory neighbour; the two reproductions fail closed; mutant (check dropped in replace) dies | document-model (record writer), evidence-claims, prose-assessment |
| G1 | HIGH | `change complete` refusal headline counts obligations as "sections" | count distinct sections in `summary()` (or name the true count and fix the Contract: prefer distinct sections to match the Contract) | fixture: one affected section with 3 moved inputs prints "1 section"; two-section neighbour prints 2; mutant dies | complete-gate |
| G5 | MED | "with the time it ran" dropped; kill for too much output records zero time, unlisted | state the exact behaviour in the complete-gate text (timeout reason carries time; output-bound kill carries the output limit and no time) and list in the PR body | text assessed against `bound/run.rs`; fixture pins both reasons | complete-gate |
| G2 | HIGH | a claim marker split over lines (`<!--` / `raf:claim evidence=x` / `-->`) is read as a plain comment: Contract says never | a multi-line HTML comment whose first content line starts with `raf:` is a marker error (`MalformedClaim`: "a marker is one line"); a multi-line comment not starting with `raf:` stays plain | fixtures (3-line claim, 3-line unknown `raf:` comment, ordinary multi-line comment passes, inside fence not read); check exits 1 for the first two; mutant dies | okf-profile |
| G3 | MED | `evidence claims <id>` skips capabilities that fail to load and answers "no claim" exit 0 | fail hard like `evidence for`/`history`: if any capability document is unloaded, exit 2 naming it (reuse `has_unloaded_documents`/the unloaded set) | fixture: broken capability doc -> exit 2; neighbour: clean repo -> exit 0 | evidence-claims, cli |
| A3 | LOW | `evidence claims` and `assess --context` print document text raw (ESC/OSC/BEL/U+202E) | escape control and bidi characters in the CLI text output of these two commands (same helper as history/coverage tables); JSON already escapes | fixtures with ESC/OSC/U+202E in a claim statement and a section | cli |
| A1/A2/A4/A5/A6/A7 | LOW | SECURITY.md/Contract do not state: Ctrl-C does not reach the trusted command's group and `setsid` escapes the kill; a relative program resolves against the workspace root and the record keeps the text as typed; the cost of exported-items carry before the budget stops it; a hand-written record may hold more than 1024 values (bounded by the 1 MiB document bound); error messages quote the command's words unquoted; write checks are by path not handle | state each precisely in SECURITY.md and the evidence-claims Constraints/Contract; quote the command words in error messages (debug-style quoting) | text assessed; fixture for quoted command words | evidence-claims, SECURITY.md |
| C1 | LOW | carried review items not done: `HistoryError::SymbolLog` message wording (conditional "commit the file"); crate-root digest behaviour test; `binding_site` first-glob fixture (raf-source); comment on `has_unloaded_documents`; `super::X` re-exports not normalised in coverage (state the limit in the two-way-coverage Constraints); one Contract sentence for `assess <section> --list` exit 2; ATX-only headings sentence (current-rationale) | do each; state-only for `super::X` | each has a test or text and is assessed | entity-history, two-way-coverage, current-rationale, cli, source-index (test only) |
| P1 | MED | PR body stale (hosted CI now green), lacks the reworded-sentence lists and the module-binding/G5 notes | rewrite the PR body: current CI, full lists of reworded delivered sentences (from validation 11A-11E, G5), owner readings, the hook-bypass slip, the plan-adversary BLOCKED note | body contains the lists and no stale claim | — |
| P2 | LOW | roadmap guard fails while any pinned capability is reopened | keep as is (intentional: it asserts repository state); the reopening commits that need green gates keep the guard satisfied by reopening only in one commit that also keeps tests green — note in PR body; no code change | — | — |

## Order and commit boundary

1. Code + tests + text for S1, G1, G5, G2, G3, A3, A-docs, C1 on reopened capabilities, one commit per concern, hooks on (the roadmap guard and check_inventory must stay green in each commit: reopen all touched capabilities in the first commit with their `change new` records and `implementing` states, update INVENTORY and the roadmap guard to expect them (guard names the set that must be delivered: keep it to the capabilities that are delivered at that commit; restore the full set when they are re-delivered)).
2. Reconcile, assess affected sections honestly, probe all 12 capabilities for stale records and re-record them.
3. Complete each reopened capability again with the tool, one commit per capability (as in 11F).
4. PR body rewrite; Stage-2 exit audit.

Falsifiable overall acceptance: `check` exit 0; `assess --list` all current; stale-probe over 12 capabilities clean; full local gates once (`cargo test --workspace`, release, clippy, rustdoc, fmt, reuse, deny, msrv, issue-refs, licenses, docs-sync twice); hosted CI green on the new head; a fresh completion auditor passes.

## Revision 2 (integrates reviews/remediation-plan-review.md)

Every `REWRITE TO` line of reviews/remediation-plan-review.md is adopted VERBATIM and replaces the row or step it names:

* **P2 / Order 1 (roadmap guard and commit sequence):** C1 is ONE reopening commit (records via `change new`, `state: implementing`, `INVENTORY` states, the shape pins implementing |R| and delivered 12-|R|, `make docs-sync`) that also rewords the roadmap guard to "each of the 12 is delivered, or implementing with its capability-named open change; no gap in either case" (a bound test of workplan-document-model, so the guard reword is part of that capability's reopening; its Verification sentence is updated). C2..Ck are one commit per concern, each binding every new test and public item in the same commit. After every commit run `cargo test -p raf-workplan --test roadmap` and `cargo test --test docs_contract --test repo_gates` (the hook runs only the second).
* **G1:** count distinct (capability, section) subjects for condition 6 in `summary()`, in text and JSON headline alike; the complete-gate Contract 273-277 is rewritten per the review's REWRITE TO (a section is one or more `assessment-*` obligations, one `assessment-stale` per moved input, each printed once; the headline counts sections); the record `affects` lists #contract, #examples and #verification; fixture and mutant as the review states. The "or" is dropped.
* **G2:** the review's rule: a multi-line HTML comment whose first non-blank text after `<!--` is `raf:` in any letter case and is not `raf:generated:` is a marker error (`okf-parse-malformed-claim` for `raf:claim`, `okf-parse-unknown-marker` otherwise); multi-line `raf:generated:` comments, SPDX headers and ordinary multi-line comments stay plain; fixtures and the architecture page sentence as the review states.
* **P1 (PR body):** rewritten AFTER the final push with every item of the review's REWRITE TO (hosted CI result and head SHA, whole-repository `verify` result, rebinds, the nominal dependency reading, id-rule item, the record-item rewordings of all 12 records with reasons, the delivered-sentence rewordings and removals of 11A-11E/11D/G5 and of this remediation, owner readings, the hook-bypass slip, no `.stage` reference); acceptance: `gh pr view --json body` contains no 'has not run', 'none has run' or 'never run' and no `.stage`; one row per rewording counted in validation 11A-11E plus the remediation rows; head SHA equals `git rev-parse HEAD`.
* **C1:** only the not-yet-done items: SymbolLog wording (entity-history); crate-root digest pin (prose-assessment owns Digest); `binding_site` first-glob fixture (source-index, reopened because the test is bound there); `has_unloaded_documents` comment (document-model); `super::` limit in the two-way-coverage Constraints (a false gap row, never a missed gap); a lock for the `docs_sync` example test (generated-docs, test only); the id-rule item in the PR body. The `assess <section> --list` exit-2 sentence and the ATX-only sentence are already in the documents: no reopening for them.
* **Consciously accepted (listed in the PR body with reasons):** G4 (with the reason, or else a `has_unloaded_documents` guard on coverage enforcement — prefer the guard if cheap); A5 maintainability items (module sizes, split JSON rendering, one inventory edit per record); A6 change history after completion; security A5 (hand-written record with >1024 values is bounded by the 1 MiB document bound); the exports.rs doc wording (fix the one comment); validation A4 (final local gates and CI cover it).
* **Step 2 (after the last code commit):** (i) re-record each evidence record whose inputs moved (`evidence record --update --from-binding ...`); (ii) assess every affected section of each reopened capability; (iii) grep all documents for the behaviours G1, G2, G3, G5, S1, A3 and A6 change and re-read each hit; (iv) `assess --list` all current and the 12-capability probe with the seven obligation/finding kinds clean; re-read each stale record, re-record it only when every statement holds, otherwise reopen its capability; repeat from (i) until one pass is clean; (v) new assessment records go into `INVENTORY.records` and the shape pins in the same commit. Any later code change repeats step 2.
* **Final gates:** `make check` with every target except `cross-check` (owner decision: hosted CI counts for it), each named with its exit code: fmt, clippy, test, test-release (the target), doc, deny, reuse, msrv, fuzz, licenses, issue-refs; plus `make docs-sync` twice with no diff; hosted CI green on the final head.
* **Advisories adopted:** G3 refuses only when a CAPABILITY document is unloaded (`unloaded()` filtered to `cap:`; an unloaded change or evidence record leaves exit 0; Contracts state the exit); A3 moves the shared escape helper out of the history module for a third user; security A6 quotes with `TrustedArgv::text()`; G5 adds `elapsed` to `OutputTooLarge` so the delivered clause "with the time it ran" holds for both kills (restoring, not narrowing; test pins both reasons); S1 also covers `sync::judge` and the record removal in `complete.rs` (apply the same per-directory check or explain why not reachable) with fixtures; the probe clones forbid any hook bypass; the plan lists each change record's `affects`: workplan-document-model (#contract, #usage, #verification), workplan-evidence-claims (#contract, #constraints, #verification), workplan-prose-assessment (#constraints, #verification), workplan-complete-gate (#contract, #examples, #constraints, #verification), okf-profile (#contract, #verification), workplan-cli (#contract, #constraints, #verification), workplan-entity-history (#contract, #verification), workplan-two-way-coverage (#constraints), workplan-generated-docs (#verification, test only), source-index (#verification).

## Notes from the plan recheck (adopted)

* The table rows above this section are SUPERSEDED by Revision 2 wherever they differ (P2, Order step 1, G1 "or", G2, P1, C1); follow Revision 2.
* N1: reopened-capability text goes in PROPOSED blocks (AGENTS.md section 6) while the capability is `implementing`; they are resolved into plain text before completion.
* N2: the consciously-accepted list also names: type bindings do not hash method bodies (name the method to include it), and nothing keeps records current between changes.
* N3: adding `elapsed` to `ExecError::OutputTooLarge` is an API change for outside matchers without `..`: CHANGELOG line; Contract lines about the kills and SECURITY.md say "with the time it ran" for both kinds.
* N4: generated-docs also lists #contract/#constraints if `sync::judge` changes; two-way-coverage lists #contract if the G4 guard is built.
* N5: step 2 may add a section to an already-open record's `affects` as well as reopening a capability.

## 4. paudley — 

Stage 2 summary.

- Gaps from the gap analysis, security review and completion audit are fixed through the workplan lifecycle (link-following record writes, the completion-gate headline count, multi-line claim markers, the unloaded-capability exit, escaping of document text, and the documented limits). The macOS process-group kill race is fixed: a group that answers EPERM after its leader exited counts as done.
- Local validation: `check` exits 0, `assess --list` shows every assessment current, the stale probe over all twelve capabilities is clean, and the whole-repository `verify` passed all 1486 bindings. The local gates ran before the macOS fixes; the later changes are covered by hosted CI.
- Hosted CI: all 12 jobs pass on the branch tip, including the macOS job with its new process-group step.
- Completion audits: the last full audit passed every criterion except the PR body, whose statements were then corrected against `gh run list`, the CLI and the validation notes. No further audit was run (owner decision).
- Not run: the local cross-check (owner reading: hosted CI counts); the macOS mutant check, which was inferred from the kernel source.
- Rule added: no timing runs (CLAUDE.md).

