# Plan constraints review

Reviewed plan: `.stage/raf-workplan-change-lifecycle-cargo/plan.md` (Revision 2, as updated after the third constraints review)
Review type: third recheck. Findings C1-C6 from Revision 1, C7-C9 from the first Revision 2 review, C10-C11 from the second, plus a pass over the changed text

## Governing documents read

| Document | Status |
| --- | --- |
| `AGENTS.md` | read (canonical repository contract) |
| `CLAUDE.md` | read (defers to AGENTS.md; "Never" list, no deferrals, no attribution) |
| `SECURITY.md` | read (trust model, Subprocesses, security surface) |
| `deny.toml`, `fuzz/deny.toml` | read |
| `Makefile`, `.github/workflows/ci.yaml` | read |
| `Cargo.toml` | read (workspace lints, members) |
| `tests/repo_gates.rs` (header, lint and deny gates), `tests/docs_contract.rs` (INVENTORY rule) | read |
| `crates/raf-workplan/src/refs.rs`, `crates/raf-workplan/src/gate.rs` (git calls) | read |
| `docs/capabilities/workplan-reconcile.md`, `workplan-complete-gate.md`, `workplan-cli.md` | read (the designed contracts) |
| `docs/capabilities/workplan-document-model.md` (states table, PROPOSED-block validity, check rules), `workplan-generated-docs.md` | read |
| `work/changes/workplan-reconcile.md`, `workplan-complete-gate.md`, `workplan-cli.md` | read |
| `.deficiencies` | read: notice and marker only, no entries |
| `.gitignore` | `/.stage/` is ignored, so the plan, the reviews and `validation.md` are not committed files |
| `.goals`, `.baseline`, `CONSTITUTION.md`, ADR directory, `PRECOMMIT_GUIDANCE.md`, nested `AGENTS.md`/`CLAUDE.md` | **absent**. The repository rules and the capability documents were still checked |
| User decisions this session | One owner decision. The coordinator relayed it, and the plan quotes it at plan.md:248: "Hosted CI counts (Recommended)". See C8. U1, D2 and D7 remain planner readings for the PR body. |

## Status of all findings

| Finding | Status | Evidence |
| --- | --- | --- |
| C1 BLOCKING (deferral) | **DISCHARGED** | D2 (plan.md:38-45): all three capabilities are delivered here; Scenarios A and B are built (Task 10); the records are completed with the shipped tool (Task 11). |
| C2 BLOCKING (tracker reference in docs) | **DISCHARGED** | plan.md:132: Scenario text stays as written and is built. No document text names a tracker. |
| C3 ADVISORY (fuzz/deny.toml mirroring) | **DISCHARGED** | D10 (plan.md:94-96). |
| C4 ADVISORY (bounds stated in the specs) | **DISCHARGED** | Contract-edit rows plan.md:125 (time bounds) and plan.md:129 (16 MiB per stream, 64 MiB for `cargo metadata`). This meets `SECURITY.md:146`. |
| C5 ADVISORY (environment) | **DISCHARGED** | plan.md:27-28 and D3. |
| C6 ADVISORY (U1 label) | **DISCHARGED** | D8 states U1 as a reading for the PR body. |
| C7 BLOCKING (delivered `Gate`/`docs_sync` contract) | **DISCHARGED** | D4 (plan.md:54): the version gate "is NOT part of `Gate::open`/`assert_clean` (their documented behaviour is unchanged)". The `docs_sync` reroute changes no behaviour. |
| C8 BLOCKING (local `cross-check` replaced by hosted CI) | **DISCHARGED by owner decision** | See C8 below. |
| C9 ADVISORY (D9 relaxes reviewed Constraints) | **DISCHARGED** | Rows plan.md:123 and :128. The order in D9 (plan.md:91) is the generated rewrite first, then `remove_file` last. Task 11 writes the edits as PROPOSED blocks. |
| C10 ADVISORY (version gate scope; Task 2 wording) | **DISCHARGED** | D4 (plan.md:54): the gate runs in `Lifecycle::open`, "used only by `reconcile` and `complete` (the two commands whose Contracts state it; `impact`, `verify`, `new` do not need Git)". This matches `workplan-reconcile.md:62-65`. Task 2 (plan.md:190): "version gate in `Lifecycle::open`, not in `Gate::open`". |
| C11 ADVISORY (Task 11 commit order vs `docs_contract`) | **DISCHARGED** | Task 11 (plan.md:239) has four commits: (1) `implementing` with bindings and the INVENTORY row; (2) PROPOSED blocks, which are allowed in `implementing` (`workplan-document-model.md:150-153`); (3) the blocks resolved, with `reconcile` empty; (4) `delivered`, the INVENTORY row, the record deletions and the generated rewrite in one commit. Every commit can pass the pre-commit `docs_contract` without a bypass (`AGENTS.md:134-138`, `:144-145`). The INVENTORY row changes with each state (`AGENTS.md:229-231`). D9 (plan.md:92) now judges the record-present model by conditions 1-5 only and applies the state rules to the post-deletion model, which settles the earlier design note. |

### C8: discharged by owner decision, reported as a decision

The governing rules are unchanged: `CLAUDE.md:9-10`, `AGENTS.md:133` and `AGENTS.md:146-151` ("A missing tool is a hard failure, not a skip"). Under this review's precedence, only an explicit, quoted owner decision can let hosted CI stand in for a local gate. The plan now quotes one (plan.md:248):

> "hosted CI on the PR is its recorded evidence by the owner's decision on the record: "Hosted CI counts (Recommended)" — run every other gate locally, quote the decision in the PR body, merge only after the hosted cross-check job is green; each `make` target is run individually so none is skipped by the first failure"

This is recorded as an **owner decision**, not cleared as compliance. The plan keeps the substitution narrow:
- Only `cross-check` is covered by it.
- Every other `make check` target runs locally, one by one, so `licenses` and `issue-refs` still run after `cross-check` fails.
- The local failure stays reported as a hard failure.
- The decision is quoted in the PR body.
- Merge waits for the hosted `cross-check` job (`.github/workflows/ci.yaml:108-122`), which runs the same `make cross-check`.

Caveat on provenance: the decision reached this review through the coordinator. This reviewer has not seen the owner's question-tool answer itself. The PR body has to carry the decision verbatim so the owner can confirm or overrule it in review.

## Open findings

None.

## Notes (not findings)

- Task 11 states the sequence twice: the numbered order at plan.md:239, then a prose restatement at plan.md:240-245. They agree, but the implementer should follow the numbered commit order at plan.md:239. In the prose version, "set `state: delivered` ... then run `cargo-workplan change reconcile` ... and `change complete`" must stay inside step (4), working tree only, and is never committed apart from the deletion.
- D2's plan-only sentence "Later dogfood/gate-wiring ... is outside this issue's scope" (plan.md:44-45) must not be copied into any committed file or commit message (`AGENTS.md:259-260`).

## Checked and compliant (no finding)

- No `#![feature]`, no lint weakening. The new crate inherits `[lints] workspace = true` (`tests/repo_gates.rs:831-843`).
- Naming: `cargo-workplan` package and binary are lowercase `cargo-*` (`CLAUDE.md:14-15`).
- Subprocess rules hold: argv vectors, no shell, `--frozen`, the four patterns through validated newtypes (D5), `--end-of-options`, commit-id resolution, the `<rev>:<path>` form and fail-hard on a missing default branch. All match `SECURITY.md:138-144`, `workplan-complete-gate.md:63-90` and `workplan-reconcile.md:62-83`.
- Bounds, time limits and the environment are stated in the specs through the Contract-edit table.
- The delivered `Gate` and `docs_sync` contracts are unchanged (C7).
- `deny.toml` and `fuzz/deny.toml` stay in step (D10). New dependencies are limited to the binary crate.
- Each state change edits INVENTORY in the same commit. The PROPOSED blocks are written only in `implementing` capabilities. The record deletion lands in the same commit as `delivered`.
- `.deficiencies` stays empty (Task 12). The plan adds no attribution anywhere. `Closes #8` and the issue and PR posts go to the tracker, not to committed files. `validation.md` and the reviews sit in the ignored `.stage/`.

VERDICT: PASS
