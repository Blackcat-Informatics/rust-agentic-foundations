# Requirement candidates — issue #10

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 5 | checkbox | - [ ] `cap:workplan-entity-history`: history of a capability and its bindings (`git log --follow`, `-L`), so how a design evolved can be investigated on purpose without loading everyone's working context |
| B2 | body | 6 | checkbox | - [ ] `cap:workplan-evidence-claims`: persisted execution evidence, and which current statements depend on which result; evidence changes invalidate the claims that depend on it |
| B3 | body | 7 | checkbox | - [ ] `cap:workplan-prose-assessment`: recorded human/agent assessments of explanatory prose, invalidated when their inputs change |
| B4 | body | 8 | checkbox | - [ ] `cap:workplan-two-way-coverage`: code→docs (public items no capability explains) and tests→claims, alongside the M1 docs→code and docs→tests checks |
| B5 | body | 9 | checkbox | - [ ] `cap:workplan-current-rationale`: shared architecture pages that state resolved current decisions, plus the "conditions that would justify revisiting" structure |
| B6 | body | 12 | checkbox | - [ ] Each capability reaches `delivered` through `cargo workplan change complete` |
| B7 | body | 13 | checkbox | - [ ] Its change record is deleted on completion; the history is in Git |
| B8 | body | 14 | checkbox | - [ ] No deferrals: any scope that leaves an item is `moved:` to a tracked change |
| C1.1 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 18 | list-under-requirement-heading | * AGENTS.md / CLAUDE.md: floating nightly in hosted CI, MSRV 1.98.1, workspace lints, no `#![feature]`, no |
| C1.2 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 22 | list-under-requirement-heading | * SECURITY.md trust model: workspace is trusted build input; no shell; argv vectors; document text reaches a |
| C1.3 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 25 | list-under-requirement-heading | * Reused, not duplicated: `exec` typed invocations (new rows only), `git` module, `Gate::open`, |
| C1.4 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 28 | list-under-requirement-heading | * **Owner decision on the record (coverage scope): "Workplan crates only (Recommended)".** Reading applied: |
| C1.5 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 35 | list-under-requirement-heading | * **Standing owner decision:** hosted CI counts as the evidence for `cross-check`; every other `make` gate is |
| C1.6 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 37 | list-under-requirement-heading | * The nominal dependency on the self-gate issue is not waited for: the tool this work needs (`change reconcile`, |
| C1.7 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 72 | modal | fixes what the evidence covers; the claim marker is the citation; both are required and a mismatch is a |
| C1.8 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 74 | modal | results per claim: all must hold (D2). |
| C1.9 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 100 | modal | * **D10 Current rationale.** A decision page is an architecture page with a `## Decision` heading; it must have |
| C1.10 | comment 1 (paudley, 2026-10-15T08:16:26Z) | 102 | modal | must be followed by `### Conditions to revisit`. An architecture page no document links to is reported. |
| C3.1 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 18 | list-under-requirement-heading | * AGENTS.md / CLAUDE.md: floating nightly in hosted CI, MSRV 1.98.1, workspace lints, no `#![feature]`, no |
| C3.2 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 22 | list-under-requirement-heading | * SECURITY.md trust model: workspace is trusted build input; no shell; argv vectors; document text reaches a |
| C3.3 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 25 | list-under-requirement-heading | * Reused, not duplicated: `exec` typed invocations (new rows only), `git` module, `Gate::open`, |
| C3.4 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 28 | list-under-requirement-heading | * **Owner decision on the record (coverage scope): "Workplan crates only (Recommended)".** Reading applied: |
| C3.5 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 35 | list-under-requirement-heading | * **Standing owner decision:** hosted CI counts as the evidence for `cross-check`; every other `make` gate is |
| C3.6 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 37 | list-under-requirement-heading | * The nominal dependency on the self-gate issue is not waited for: the tool this work needs (`change reconcile`, |
| C3.7 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 72 | modal | fixes what the evidence covers; the claim marker is the citation; both are required and a mismatch is a |
| C3.8 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 74 | modal | results per claim: all must hold (D2). |
| C3.9 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 100 | modal | * **D10 Current rationale.** A decision page is an architecture page with a `## Decision` heading; it must have |
| C3.10 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 102 | modal | must be followed by `### Conditions to revisit`. An architecture page no document links to is reported. |
| C3.11 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 283 | modal | * Task 1 reviewer: narrow the architecture-page sentence "a misspelt marker is never read as a plain comment" to non-`raf:generated:` comments (done in the Task 3 commit that touches that page, or earlier); Task 3 moves the record fixtures to their real directories; Task 5 states that `Number` accepts no exponent (a printed `1e-3` fails hard under D12) and refuses a `--for` capability with no code binding clearly; Task 7 maps crate names (`raf-okf` and `raf_okf`); Task 11 must handle `reconcile okf-profile` reporting an empty `bindings.examples` and the `verify` failure of the `no_run` `raf_workplan::Gate` example in `workplan-document-model` (tracked by issue 17). |
| C3.12 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 284 | modal | * Task 2 reviewer (advisory): add a fixture pinning `binding_site` to the first carrying glob; Task 7 coverage must treat the loud differences of `exported_items` (the several-binder carry refusal, the `pub(crate)` item next to a carried re-export, the unmodelled visibility rule) as documented limits of the surface it reads; Task 11: empty `bindings.examples` on source-index and okf-profile, the line-1254 tense wording, and the doctest order. |
| C3.13 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 285 | modal | * Task 4 reviewer (advisory, carry to the Task 4 text edits in Task 11 and Task 9): the Contract should say "a merge whose result equals one of its parents is not an entry"; the mixed path bases (document paths relative to the repository top, symbol files relative to the workspace) belong in the Contract Constraints; partial (blobless) clones fetch objects lazily, which contradicts "reads Git and writes nothing" — state it; the "stub git on PATH" Verification wording is rechecked when the command lands in Task 9. |
| C3.14 | comment 3 (paudley, 2026-10-17T02:54:07Z) | 286 | modal | * Task 5 reviewer (carry to Task 9): clap must take the words after `--` as trailing arguments consistently with `TrustedArgv::from_command_line` (pass `std::env::args_os()` itself) and refuse an option whose value is `--`; test the command runs through the binary. `Number::value()` stays public for display only — never compare with it. |

## Comments that matched no rule

- comment 2 (paudley, 2026-10-17T02:54:03Z)
