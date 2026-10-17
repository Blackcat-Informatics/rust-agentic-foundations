# Task 6 review: prose assessment

## Scope

This review covers the staged diff on top of 9bacdb3: 43 files, about 3330 lines
added and 162 removed. It reads:

* `crates/raf-workplan/src/assessment.rs` and its tests.
* The `trace::assessments` source and `ObligationKind::ALL` (now 13 kinds).
* `lifecycle/complete.rs` and `lifecycle/completion.rs` (condition 6).
* In `graph.rs`: `prose_of`, now linear, `code_span_paths` and
  `assessment-id-mismatch`.
* `digest::symbol_texts`.
* The shared helper `tests/support/assessed.rs`, which the cargo-workplan tests
  include through `#[path]`.
* Every changed fixture.
* SECURITY.md and CHANGELOG.md.
* The PROPOSED edits in prose-assessment, complete-gate, reconcile and
  document-model.
* The `INVENTORY` change and the four records.

The plan sections checked against are D3, D8, D13 to D17, Task 6, R3 and the edit
list.

All experiments ran in scratch worktrees that held HEAD plus the staged diff, and
both were removed afterwards. Nothing in the task worktree was staged, reset or
committed.

## Evidence

### (11) Gates

The task worktree's working tree equals its index. All of these exited 0:

* `make fmt`
* `make clippy` (`-D warnings`)
* `make test`, which is the workspace and includes `docs_contract` and
  `repo_gates`
* `cargo test -p raf-workplan --release`
* `make doc` (`-Dwarnings`)
* `make reuse`, `make deny`, `make issue-refs` and `make msrv`

`make docs-sync` ran twice in a scratch worktree. Neither run changed the tree, so
it is idempotent. `cargo workplan check` exits 0. `change reconcile` on the four
open records lists `assessment-missing` for each affected section, beside the
existing obligations.

### (1) Digest inputs and currency

The tests and the scratch fixtures show:

* An unchanged section stays current, and a second record is byte-identical.
* Each of these makes the assessment stale and names the input:
  * a section edit;
  * a change to a named symbol;
  * a change to any bound symbol when the section names none;
  * a bound symbol added or removed;
  * an evidence record that changed or no longer loads.
* These leave it current:
  * an edit of an unrelated section;
  * a regenerated block;
  * a CRLF-only rewrite of the whole capability (scratch fixture: condition 6
    holds and the change completes).

The record digest applies `lf`, the section digest goes through
`section_digest`, and the symbol digest applies `lf`.

Currency is strict in both directions:

* An input the record lacks counts as moved (`NewInput`).
* An input the section no longer has counts as moved (`NoLongerInput`).
* A duplicated or foreign row counts as moved.
* A record with no inputs is stale.

### (1) Named-symbol detection

This is the problem behind R1 below. Detection is exact-path only.

On this repository's own capability documents, 49 sections have code bindings:

* In 44 of them no bound path is named exactly, so every code binding is an input.
* In 4 of them some bound paths are named exactly and other bound symbols are named
  by a suffix path:
  * workplan-document-model Contract and Verification;
  * source-index Verification;
  * workplan-reconcile Verification.

Example: the document-model Contract names `raf_workplan::digest::Digest` and
`raf_workplan::Gate` exactly. Those become its only symbol inputs. It also
describes `Workplan::check`, `Workplan::load` and `Digest::section`, which are
bound as `raf_workplan::Workplan::check` and so on, and none of those are inputs.

A scratch fixture reproduces this:

* Bindings: `[demo::run, demo::other]`.
* Contract: "`demo::run` starts the work and `other` finishes it."
* After the Contract is accepted, `other` is changed to `panic!`. The assessment
  stays `Current`.

### (2) Obligations

* `assessment-missing`, `assessment-stale` (one per moved input, naming it) and
  `assessment-rejected` are raised from `impact.sections`, which is the change's
  `affects`.
* `suppressed` leaves them untouched under `contract_effect: none`. This is tested,
  and the Contract texts in reconcile and complete-gate (PROPOSED) give the reason.
  It is consistent with the reconcile Contract, which suppresses only
  `section-untouched` for three sections.
* `ALL` has 13 kinds with rank asserted at compile time. The kind-coverage tests
  are extended: the names test, `evidence_of` exhaustive, and every static kind
  raised by a fixture.

### (3) Condition 6

* Condition 6 is judged inside `evaluate`. It is therefore judged before any cargo
  run (the test asserts that only git calls are made) and again in the second
  evaluation before the writes, which is the TOCTOU re-check.
* A rejected verdict by a human and by an agent each refuse. An agent acceptance
  satisfies the gate.

I tried to make complete succeed wrongly with these scratch fixtures, each over a
baseline that completes. All of them refused:

| Attempt | Refused with |
| --- | --- |
| Record renamed to another file name | `okf-id-file-name` and missing |
| Record nested below `work/assessments` | `nested-directory` and missing |
| Record moved into `work/evidence` | `wrong-directory` and missing |
| A current copy with the same id beside a stale record at the right path | stale and `okf-id-file-name` |
| The usage record hand-edited to the contract id and subject | stale: section digest |
| The id of one section with the subject of another | `assessment-id-mismatch` and missing |
| The section edited after the open | `assessment-stale` |

### (4) Record writing

* **Id rule.** `<cap-slug>-<section>` cannot collide, because no section slug
  holds `-` (the seven slugs were checked).
* **Replace.** A record is replaced through stage-then-rename, and a non-file in
  the way is refused.
* **Refusals.** An unresolved bound symbol, a claim on an unloaded record, a
  record over the bound and a lone SPDX field each write nothing; each refusal
  test uses a snapshot.
* **Neighbours.** Each refusal sits beside a neighbour that records.
* **Context.** The context takes no actor and is byte-identical (tested).
* **SPDX.** The SPDX lines are written when both are given.

### (5) Generated-only exemption

The note is verified. `raf-okf` `section_is_empty` (`visible_content` with
generated blocks and HTML comments removed) is the same rule as
`assessment::is_authored`. A loaded capability therefore never has a generated-only
section, and `Judged::Exempt` cannot be reached through a loaded document. The
prose-assessment Contract states this. The complete-gate PROPOSED block, the
reconcile PROPOSED paragraph and complete-gate record item 2 do not (A2).

### (6) Regression

Every test diff was compared with HEAD:

* No assertion was removed or loosened.
* The exact refusal sets in the scenarios are extended (`and_assessments_missing`).
* `split_assessments` asserts both halves.
* The cargo-workplan `git status` count of 2 became an exact sorted list of three
  paths.
* The evidence-stale fault re-assesses, which isolates its kind.
* The new Scenario B stage ("settled and not assessed: condition 6 alone") checks
  the exact set, then assesses, commits and completes. It is meaningful.

### (7) `prose_of` and `code_span_paths`

* **Equivalence.** A scratch unit test copied the old `prose_of` from HEAD and ran
  it beside the new one over every section of 46 Markdown files (`docs/`,
  `docs/capabilities`, `docs/architecture`, `work/changes`). All 195 sections gave
  identical output. `code_span_paths` and `names_in_code_span` also agreed: 4708
  found paths and 36020 well-formed probes (every found path, every contiguous
  sub-path, every token).
* **Performance.** A Contract of 20,000 paragraphs, each with a code span and a
  fenced block (1.03 MB), ran through `prose_of` plus `code_span_paths` in 99 ms
  in debug.

### (8) Documents

* Delivered text outside the PROPOSED blocks is unchanged. The diffs touch only
  `>` blocks, bindings, generated blocks and `state`.
* complete-gate is `implementing`. `INVENTORY` has 5 implementing and 2 delivered,
  and the index regenerated.
* The record `workplan-complete-gate` has four items that match the code. Item 2
  is vacuous; see A2.
* The prose-assessment item is reworded to "who assessed by kind only". This is
  the edit-list entry decided by D8: the record stores no personal name. It is a
  design decision, not an unbuilt part, so D14 needs no move. The PR body must list
  it (R8).

### (9) Mutation re-run

All 9 mutants were killed. Each compiled and failed tests:

| Mutant | Failing tests |
| --- | --- |
| Section input dropped | 65 |
| Evidence input dropped | 4 |
| Currency compare always equal | 10 |
| Rejected treated as accepted | 6 |
| Condition 6 skipped | 2 (the condition-6 counts in `complete`) |
| Id collision (slug without section) | 40 |
| "All bound when none named" removed | 7 |
| Named-symbol detection always empty | 3 |
| `contract_effect: none` suppresses the assessment kinds | 7 |

### (10) Hygiene

* All 4 added files carry SPDX lines.
* The added lines have no tracker numbers, process wording or attribution.
* There is no `unsafe`.
* There is no manifest, lockfile or toolchain change.

## Findings

### REQUIRED

* **R1: a bound symbol named by a suffix path is not an input, so the assessment
  stays current when it should go stale.** `inputs_now` uses
  `named.contains(binding.path())`, an exact whole-path match.
  * **Where it bites.** Realistic documents name bound symbols as
    `Type::method` or by the final segment. This repository does it in 4 of its 49
    sections with code bindings, as shown in (1).
  * **What goes wrong.** Once a section names any bound path exactly, the bound
    symbols it describes by a shorter path drop out of the inputs. An edit to them
    leaves the assessment `Current`, and condition 6 passes. The scratch fixture
    reproduces this.
  * **The documents do not cover it.** The Constraints sentence admits only
    re-export paths.
  * **Fix.**
    * Treat a `code` binding as named when a code span names its full path or any
      `::`-suffix of it, including the final segment. Extra inputs only add
      staleness, never a silent "current".
    * State the rule in the prose-assessment Contract and Constraints.
    * Add a fixture: a section naming `demo::run` and `other` goes stale when
      `demo::other` changes.
  * **If the final segment alone is judged too broad:** use suffixes of at least
    two segments, state the remaining limit, and pin it with a test.

### ADVISORY

* **A1: churn.** 44 of the 49 sections with code bindings name no bound path
  exactly. Every edit to any bound code therefore stales their assessments
  (Purpose, Explanation, and so on). With 32 bindings on document-model, that is
  frequent. The R1 fix reduces it.
* **A2: the generated-only exemption cannot trigger.** The profile refuses such a
  section (`okf-section-empty`). The prose-assessment Contract says so; these do
  not:
  * condition 6 in the complete-gate PROPOSED block;
  * the reconcile PROPOSED paragraph;
  * item 2 of the `workplan-complete-gate` record.

  Reword them in Task 11 to say that such a document does not load.
  `is_authored` also restates the `raf-okf` `visible_content` rule. Two copies may
  drift, so consider sharing one.
* **A3: delivered text Task 11 must rewrite.** The complete-gate `description`
  and its index row still say "the five completion conditions". The delivered
  **Order.** paragraph lists conditions 1 to 4.
* **A4: missing pins.**
  * There is no CRLF test at the assessment level; the digest tests cover CRLF,
    and my scratch fixture confirms it.
  * No shipped test pins `prose_of` on a section with many blocks. The old code was
    quadratic in that case.
* **A5: digest scope of a type binding.** A type binding hashes only the item's
  span, not its impl methods. A section that names `raf_workplan::digest::Digest`
  does not go stale when the body of `Digest::section` changes. This is the D3
  symbol-digest design. R1's suffix rule would make `Digest::section` an input when
  it is bound.
* **A6: the helper skips quietly.** The shared helper skips sections it cannot
  assess. What they owe stays visible, and the scenario steps assert the count
  assessed, so this is acceptable.
* **A7: PR body.** List the prose-assessment item rewording, which drops the
  assessor name per D8 (R8).


## Round two: the R1 fix

### What changed

* A `code` binding now counts as named when a code span holds its full path or
  any `::`-suffix of it, down to the last segment (`names`).
* Tests were added for:
  * the round-one reproduction;
  * the last segment named beside a binding named in full;
  * a method named by `Type::method`;
  * a suffix that two bindings end in;
  * CRLF;
  * a `prose_of` performance pin;
  * an `is_authored` drift test against the profile's section-empty rule.
* The generated-only wording was aligned, and the Constraints state the
  prose-only limit.

### The new rule hides changes

Matching a suffix down to the last segment means more sections now count as
naming a bound symbol. Any section that names one leaves the fallback where every
`code` binding is an input. An incidental code span therefore narrows the inputs.

The doc comment on `names` says "a name taken too widely only adds an input,
which can make an assessment stale and never keeps one current". That is false.
A wide match switches `any_named` on, and that removes every other binding from
the inputs.

### Measured on this repository

I compared the input sets of the exact rule and the new rule, section by section.
19 of 50 sections lose inputs under the new rule. The new rule includes
everything the exact rule did in only 31 of them.

| Capability | Section | Inputs: exact rule | Inputs: new rule |
| --- | --- | --- | --- |
| workplan-complete-gate | Contract | 13 | 2 |
| workplan-complete-gate | Purpose | 13 | 1 |
| workplan-complete-gate | Usage | 13 | 2 |
| workplan-complete-gate | Constraints | 13 | 1 |
| workplan-complete-gate | Verification | 13 | 2 |
| workplan-reconcile | Contract | 25 | 3 |
| workplan-reconcile | Usage | 25 | 2 |
| source-index | Examples | 27 | 1 |
| source-index | Constraints | 27 | 1 |
| source-index | Explanation | 27 | 1 |
| workplan-document-model | Usage | 32 | 2 |

The rest of the 19 are in okf-profile, source-index, workplan-generated-docs,
workplan-reconcile and workplan-document-model.

Example: the workplan-complete-gate Contract describes the proofs the executor
demands (`--exact`, `--frozen`, `1 passed; 0 failed`, doc tests built and not
run). Those are the bound `bound::Executor` and `exec::Invocation`. Under the
exact rule they were inputs. Now the spans `change complete` and `verify` name
`Lifecycle::complete` and `lifecycle::verify`, and those two are the only symbol
inputs. A change to `bound::Executor` leaves the Contract assessment current.

### Reproduction

Scratch fixture on the staged code:

* Library: `run` and `complete`.
* Bindings: `[demo::run, demo::complete]`.
* Contract: "Type `cargo demo complete`; the runner then passes `--exact` to every
  test."
* After acceptance, `run` is edited. The assessment stays `Current`.

Under the round-one exact rule the same fixture goes stale, because nothing is
named exactly and so every binding is an input.

The Constraints sentence states the prose-only limit. It does not state that a
command word in a code span can switch a section from "every binding" to one
binding. And the limit is new: the R1 fix created it.

### Round-two findings

REQUIRED:

* **R2: the naming rule must only ever add inputs.** Widening what counts as
  "named" must never take an input away. A fix that is monotone:
  * Only a binding named by its full path, or by a suffix of at least two
    segments, counts toward leaving the every-binding fallback.
  * A binding named by its last segment alone is added to the inputs but does not
    end the fallback.
  * The inputs are therefore: when some binding is named strongly, the strongly
    named bindings plus the ones matched by last segment; otherwise every `code`
    binding.

  Under this rule a bare word in a code span can only add inputs. It still catches
  the round-one reproduction, because `demo::run` is named in full there. Inputs
  narrow only on a deliberate naming: a full path or a `Type::method`-style
  suffix. That is the D8 limit the Constraints already state.

  Also:
  * Correct the "never keeps one current" doc comment on `names`.
  * Add the fixture above, with a neighbour named by `Type::method` that narrows
    as intended.
  * Re-measure on the corpus: no section should lose inputs compared with the
    exact rule, except through a full path or a suffix of two or more segments.
  * If the implementer prefers another monotone rule, the bar is the same: the
    inputs never shrink because a code span matched more widely.
EOF  * Measured with the same approximation of the corpus: under the proposed rule,
    6 of 50 sections lose inputs compared with the exact rule, against 19 under
    the staged rule. All 6 come from a deliberate `Type::method` naming: the
    okf-profile Contract, the source-index Contract, the document-model Usage and
    Verification, and the generated-docs Contract and Constraints.

ADVISORY: the round-one advisories A2 (the wording), A4 (the CRLF and many-block
pins) and A5 (stated in the module docs and the Constraints) are addressed.

### Round-two gates

All of these exited 0 on the task worktree:

* `make fmt`, `make clippy`, `make test` (workspace)
* `cargo test -p raf-workplan --release`
* `make doc`, `make reuse`, `make deny`, `make issue-refs`, `make msrv`

### Round-two mutation re-run

All 10 mutants were killed. Each compiled and failed tests:

| Mutant | Failing tests |
| --- | --- |
| Section input dropped | 69 |
| Evidence input dropped | 4 |
| Currency compare always equal | 15 |
| Rejected treated as accepted | 6 |
| Condition 6 skipped | 3 |
| Id collision | 43 |
| "All bound when none named" removed | 7 |
| Named-symbol detection empty | 6 |
| Suffix matching removed (new) | 3 |
| `contract_effect: none` suppression | 6 |

The mutants do not test R2. No existing test pins that a wider match never
narrows the inputs; the fixture R2 asks for would.

The scratch worktree was removed.


## Round three: the R2 fix

### The rule

The staged fix (`Naming`, `naming` and `symbol_inputs` in `assessment.rs`) names
a `code` binding in one of two ways:

* **Strongly**, when a code span holds its whole path or a `::`-suffix of two or
  more segments. A strong name ends the all-bindings fallback.
* **Weakly**, when a code span holds its last segment alone. A weak name adds its
  binding as an input and never narrows.

The inputs are the strongly and weakly named bindings when at least one binding
is named strongly, and every binding otherwise. I traced the logic by hand: a
path with one segment is whole, so it counts as strong.

### My scenarios

Scratch fixtures on the staged code:

| Scenario | Result |
| --- | --- |
| Round-two reproduction: the span `cargo demo complete`, with `run` described in prose | `run` stays an input; its edit makes the assessment stale |
| Ambiguous weak name: `check` ends both `demo::plain::check` and `demo::Plan::check`, beside a strong `demo::run` | both are inputs; an edit of `plain::check` is reported |
| Strong-looking name the capability does not bind: `Plan::check`, `other::Thing` | no narrowing; every binding is an input; an edit of `complete` is reported |
| Deliberate `Plan::check` with `run` described in prose | narrows as the Constraints state; an edit of `run` stays current; an edit of `Plan::check` is reported |
| The strong name removed from the section | stale: the section, the symbol that is now an input (`demo::complete`) and the one no longer named |
| A strong inline `demo::run`, with a fenced `rust` example calling `demo::complete` | an edit of `complete` stays current (see A8) |

There is also the "strong name in the wrong capability" case: one capability's
section names, as `Type::method`, a symbol that only another capability binds.
Naming is judged only against the section's own capability's bindings, so this
cannot narrow anything.

An accidental strong name would need a 2-segment span equal to the tail of one
of the capability's own bindings, and spans such as `std::fs` or `cap:x#y` are
not. I found no realistic case beyond the deliberate naming that the Constraints
document.

### Corpus

My approximation reads PROPOSED text too.

* It finds 6 sections that lose inputs compared with the exact rule, and 2 that
  gain.
* Every loss comes from a deliberate strong name, such as `Gate::workspace` or
  `Workplan::write`.
* The implementer reports 4 losses with PROPOSED blocks excluded. That is
  consistent: the two extra ones, the okf-profile and source-index Contracts, take
  their names from PROPOSED text.

### Round-three findings

REQUIRED: none. R1 and R2 are resolved.

ADVISORY:

* **A8: fenced code.** The Contract says that only code spans outside fenced code
  name a binding. The Constraints sentence states the limit as "describes only in
  prose", but a bound symbol used only in a fenced example is not an input either,
  once the section has a strong name. Example: the document-model Usage calls
  `Gate::workspace(...).assert_clean()` in a fence.
  * Reword the Constraints to "only in prose or in fenced code".
  * Optionally, read the paths in fenced `rust` blocks as weak names. They would
    only ever add inputs.
* **A9: the documented limit in practice.** With a strong name present, a bound
  symbol the section describes only in prose is not an input. Example: the
  document-model Usage describes what `Workplan::check` does through
  `Gate::workspace`. This is the stated D8 limit. Assessors should know that
  naming a symbol inline is what makes it an input.
* Round-one advisories A1, A3, A6 and A7 still stand for Task 11 and the PR.

### Round-three gates

All of these exited 0 on the task worktree:

* `make fmt`, `make clippy`, `make test` (workspace, including `docs_contract`
  and `repo_gates`)
* `cargo test -p raf-workplan --release`
* `make doc`, `make reuse`, `make deny`, `make issue-refs`, `make msrv`

### Round-three mutation re-run

All 12 mutants were killed. Each compiled and failed tests:

| Mutant | Failing tests |
| --- | --- |
| Section input dropped | 70 |
| Evidence input dropped | 4 |
| Currency compare always equal | 15 |
| Rejected treated as accepted | 6 |
| Condition 6 skipped | 3 |
| Id collision | 45 |
| All-bindings fallback removed | 10 |
| Named-symbol detection empty | 6 |
| Suffix matching removed | 5 |
| **A weak name ends the fallback** (new) | 4 |
| **Weak matches dropped** (new) | 2 |
| `contract_effect: none` suppression | 6 |

The scratch worktree was removed. Nothing in the task worktree was staged,
reset or committed.

VERDICT: PASS
