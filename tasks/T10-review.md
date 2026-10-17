# T10 review: real use on this repository

Scope: the staged, uncommitted diff on top of 6f7afe1 (8 files). All experiments ran in a scratch worktree
(`git worktree add /tmp/t10rev HEAD` + the staged diff), since removed. No toolchain change; CARGO_BUILD_JOBS=6.

## Evidence

1. Truthfulness
   - normal-form `## Decision` matches the page it summarises (rename by first appearance, value-form noise removal,
     literal tier under 30 tokens, shim form for 6..29, first 8 bytes of BLAKE3, version bump rule, refusal until
     `cargo census migrate`). The census is `designed` (no code yet), so the Decision states the normative page, as
     the page itself does; no invented claim.
   - okf-profile `## Decision` matches the page: six types, closed ordered key sets, `id` instead of `resource`
     (`resource` rejected), strict YAML subset refused with line and column, PROPOSED blocks named after a change,
     records empty body and no title/description.
   - evidence-claims `### Decision` matches code: obligation/trace.rs compares each input symbol digest with the
     recorded one ("Nothing reads a revision or Git"); inputs are the code bindings of the `for` capabilities
     (evidence.rs `evidence_inputs`). Present tense, no history.
   - Conditions to revisit: specific and falsifiable except for the "often enough" thresholds (see A1).
2. Claim and evidence
   - Claim TRUE: bound/run.rs `start` turns `ExecError::Timeout` into a failed binding whose reason is the Display
     "`cmd` ran X s of its Y s limit and was killed; ..."; test tests/bound.rs `a_test_that_never_ends_is_killed...`
     asserts "killed" and "5 s limit" and `Ended::TimedOut`. A kill for too much output records `Duration::ZERO`
     (run.rs), so dropping "with the time it ran" from the general Bounds sentence is correct; the only remaining
     "time it ran" text is the new timeout-only paragraph.
   - Marker sits directly after the paragraph; record: subject `binding:` the real test, 16 symbol inputs, `for:
     [cap:workplan-complete-gate]`, SPDX lines, verdict pass.
   - Scratch re-run: `evidence claims gate-kills-a-test-past-its-timeout` shows line 421 with the two-line statement;
     `evidence for cap:workplan-complete-gate` lists the record (claims at 421); `change reconcile
     workplan-complete-gate` has 0 claim-/evidence- lines; `check` exit 0.
   - Neighbour: verdict -> fail gives `claim-unsupported ... has the verdict fail`; editing a line inside
     `bound::Executor::run` gives `evidence-stale ... made against raf_workplan::bound, which has changed`.
3. Commit split verified by building each state.
   - Commit 1 (4 files): `check` exit 0, docs_contract 28 passed, repo_gates 57 passed.
   - Commit 2 (4 more files): `check` exit 0, docs_contract 28, repo_gates 57, `assess --list` current,
     reconcile has no claim/evidence lines. Claim without its record is `claim-evidence-missing` (validation.md),
     so the claim and the record must share a commit, as proposed.
4. Assessment `assessment:workplan-current-rationale-explanation`: well-formed, SPDX lines present, digests
   current (`assess --list`: accepted agent current), subject section and symbol `raf_workplan::rationale`.
5. validation.md outputs re-run and equal: `coverage code` exit 1 (212 gaps: raf-okf 177, raf-source 35);
   `coverage tests` exit 1 (440: 140/215/85); `--crate raf-workplan` and `--crate cargo-workplan` code and tests
   exit 0 with empty tables; `history arch:normal-form` (2 rows) and `history cap:workplan-reconcile` equal;
   `history evidence:...` empty before commit, as stated.
6. INVENTORY `records` has the three ids in sorted order; the pin test asserts the same list; passes. The `affects`
   addition of `cap:workplan-evidence-claims#explanation` is honest: the Explanation gained text in this change.
7. Hygiene: SPDX on both new records, reuse 441/441, no tracker numbers, process wording or attribution in added
   lines.
8. Gates in scratch with all changes: cargo test --workspace --locked 1915 passed, 0 failed; fmt clean;
   clippy -D warnings clean; make reuse 0; make issue-refs 0 (repo_gates 57); make docs-sync twice, exit 0, diff
   hash identical before/after each; `cargo run -q -p cargo-workplan -- check` exit 0.

## Findings

REQUIRED: none.

ADVISORY
- A1. Conditions 1 and 2 of normal-form and the first two of evidence-claims use "often enough" with no measure.
  Genuine, but a number or a named observable would make them more falsifiable. Not blocking.
- A2. normal-form Decision has one unwrapped overlong line ("...groups forwarders. The fingerprint of a body...");
  okf-profile condition 1 is also slightly over the page's wrap width. Cosmetic; no gate flags it.
- A3. evidence-claims Decision (and the existing Constraint) omit that an input that no longer resolves also
  makes a record stale (trace.rs). Optional completeness.
- A4. The demonstration assessment goes stale on any edit of rationale.rs; fine for a demonstration, but record it
  again if Task 11 touches that module.

## Exact commit path sets
Commit 1:
  git add docs/architecture/normal-form.md docs/architecture/okf-profile.md docs/capabilities/workplan-evidence-claims.md work/changes/workplan-evidence-claims.md
Commit 2:
  git add docs/capabilities/workplan-complete-gate.md tests/docs_contract.rs work/evidence/gate-kills-a-test-past-its-timeout.md work/assessments/workplan-current-rationale-explanation.md

VERDICT: PASS
