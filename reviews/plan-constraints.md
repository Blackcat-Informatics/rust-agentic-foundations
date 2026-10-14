# Plan constraints review

- **Reviewed plan:** `.stage/raf-workplan-model-documents-check/plan.md`
- **Worktree:** `.worktrees/7-raf-workplan-model-documents-check` at 762fa99
- **Pass:** first review (no prior findings to discharge); the recheck of revision 2 is appended at the end

## Governing documents

Read:

- `AGENTS.md` (canonical repository contract), `CLAUDE.md` (defers to it)
- `docs/architecture/okf-profile.md` (the profile)
- `docs/capabilities/workplan-document-model.md`, `docs/capabilities/workplan-generated-docs.md` (the spec)
- `work/changes/workplan-document-model.md`, `work/changes/workplan-generated-docs.md` (the open change records)
- `docs/capabilities/workplan-cli.md`, `work/changes/workplan-cli.md` (owner of the CLI claims)
- `rust-toolchain.toml`, `Cargo.toml`, `deny.toml`, `Makefile`, `CHANGELOG.md`, `CONTRIBUTING.md`, `README.md`, `SECURITY.md`, `.deficiencies`
- `tests/support/mod.rs`, `tests/docs_contract.rs` (the gate the plan rewrites), `crates/raf-source/src/tree/mod.rs` (the `Tree` trait the plan writes through)
- The user's global attribution rule (`no-attribution.md`)

Absent, and recorded as absent: `.goals`, `.baseline`, `CONSTITUTION.md`, `PRECOMMIT_GUIDANCE.md`, ADR directories (`docs/adr/`, `docs/decisions/`), and nested `AGENTS.md`/`CLAUDE.md` files. Since there is no `.goals`, `CONSTITUTION.md` or ADR, the binding rules are the repository rules above, the profile, the two capability documents, and the one session decision the caller passed on: the Stage-pinned nightly, with no toolchain changes. `.deficiencies` holds only its notice and marker.

## Rules checked and found compliant

- No `#![feature]` (AGENTS.md:93-94), and workspace lints are inherited, not weakened (AGENTS.md:155-170). Plan:19 complies.
- Toolchain: the plan changes no toolchain (plan:18), and `rust-toolchain.toml` stays as it is. The Stage-pinned dated nightly is a session environment, not a repository change, so it does not conflict with AGENTS.md:85 ("floating nightly"). See A1 for how the gate evidence is reported.
- Naming: `raf-workplan` follows the lowercase `raf-*` rule (CLAUDE.md).
- Licensing: crate code is `MIT OR Apache-2.0 OR MulanPSL-2.0` and docs are CC-BY-4.0 (AGENTS.md:176-182). Plan:41 complies.
- Conventional commit subjects (AGENTS.md:249-251): all eight task subjects comply.
- No attribution (AGENTS.md:252-253, global rule): the plan adds none.
- No deferrals (AGENTS.md:246-248): every scope the plan declines (obligations output, JSON output, the CLI) has a tracked change: `work/changes/workplan-reconcile.md` and `work/changes/workplan-cli.md`. Plan:76 and plan:81 put no deferral wording in committed files.
- Fail hard and closed sets (AGENTS.md:194-196): the plan adds no fallback, and an unknown kind is an error (plan:126).
- Every rule has a negative fixture next to a valid case (AGENTS.md:199-202): plan:36 and plan:74 comply.
- Rust first (AGENTS.md:191-193): the generator logic is library code, the driver is a Rust example, and `make docs-sync` is thin Makefile glue.

## Findings

### B1 (BLOCKING): the delivered documents would keep claims about behaviour that is not built

Governing text:

- `docs/capabilities/workplan-document-model.md:176-178`: "Outside PROPOSED blocks, text states the current contract. A statement about behaviour that is not built belongs in a `designed` or `considered` capability or inside a PROPOSED block, never in a delivered contract."
- `AGENTS.md:218-219`: "Outside a proposal, the text states the current contract only."
- `docs/architecture/okf-profile.md:151`: "Outside PROPOSED blocks a document states the current contract only."

Offending plan line (plan:147): "Set both to `delivered`, with code, test and example bindings to the built items, and reconcile every Contract sentence against the code, removing or fixing any unbuilt claim."

This reconciles the Contract section only. The plan builds no `cargo workplan` binary, no `docs preview` and no reconcile obligations (J11, J13). The other sections still claim them:

- `workplan-generated-docs.md:70-74` (Usage: three `cargo workplan` commands, including `docs preview`)
- `workplan-generated-docs.md:99` (Constraints: "Preview output is printed and never written to a file.")
- `workplan-generated-docs.md:53`, `:60-61`, `:124` (`docs sync` named as the operation the code performs)
- `workplan-document-model.md:104-106` ("the reconcile obligations report each one as a gap") and `:135-136` ("The `cargo workplan` binary exposes the same checks")

None of these claims can go into a PROPOSED block: a delivered capability may hold none, and adding one would put the capability back under an open record. The change records also list only `#contract`, `#usage`, `#examples` and `#verification` in `affects`, but Constraints must change as well.

REWRITE TO: "Reconcile all seven sections of both documents against the built code, not only the Contract. Wherever the text names `docs sync` as the operation, name the `Workplan::sync` library call and the `docs_sync` example instead. Remove the `cargo workplan` command lines, the `docs preview` usage and the 'Preview output is printed' constraint from `workplan-generated-docs.md`, and the `cargo workplan` and reconcile-obligation claims from `workplan-document-model.md`. `docs/capabilities/workplan-cli.md` and `workplan-reconcile.md` already state them, and a plain cross-reference link may stay. Before completing the change records, add every section edited this way (such as `#constraints` or `#explanation`) to their `affects`. If a claim in a change record's `items` is not built, record its outcome as `moved:change:workplan-cli` or `moved:change:workplan-reconcile`, and make sure that record carries it."

### B2 (BLOCKING): moving the link rule into `raf-workplan` weakens the existing gate's tracked-path lookup

Governing text:

- `docs/architecture/okf-profile.md:205-207`: "A target inside the root is looked up in the list of tracked paths (the Git index), and a link whose target is a tracked symlink is a violation, so a symlink cannot lead a link outside the root."
- `AGENTS.md:135-136`: "do not weaken a gate or a lint to get green."
- `CONTRIBUTING.md:37-38`: "gates are never relaxed to get a change through."

Today `tests/docs_contract.rs:375-397` and `:873-894` resolve every link against `tracked_paths()` and `tracked_symlinks()`.

Offending plan lines:

- plan:32 (J6): "resolve to existing repository paths and heading anchors"
- plan:40 (J14): "Documents are read through `raf_source::Tree` (`DiskTree` / `MemoryTree`)."
- plan:140: "model rules and helpers are deleted ... and replaced by `Gate::workspace(gate_root()).assert_clean()`"

A `DiskTree` over a working checkout also sees untracked files (`/.stage/` and `target/`, for example). After the replacement, a link to an untracked file would pass `cargo test`, where today it fails. The plan does not say where the tracked-path set comes from.

REWRITE TO: "The public link checker takes the set of known repository paths and the set of symlinks as an explicit input; it never derives them from what happens to exist on disk. `tests/docs_contract.rs` passes the Git-index tracked paths and tracked symlinks, as `load_repo` does today, for both the `Gate` run and its check of the other Markdown files. `Gate::workspace` documents the path set it uses. A fixture holding a link to a file that exists but is untracked fails through the repository gate, beside a tracked target that passes."

### B3 (BLOCKING): R25 adds a second issue-reference pattern set

Governing text:

- `AGENTS.md:133-134`: "Both gate test files share one pattern set and one file list in `tests/support/mod.rs`."
- `tests/support/mod.rs:6-9`: "This module is the single definition site for ... the issue-reference and process-wording patterns".
- `CHANGELOG.md:59-60` and `:82-83` record the same invariant as released behaviour.

Offending plan lines:

- plan:73: "This ports the `docs_contract` scan rule for documents; the scan over every tracked file stays in `docs_contract` per J1."
- plan:111: "The issue-number reference rule for loaded documents (R25)."

A port leaves two pattern definitions that can drift, which J1 itself argues against at plan:27. The plan also misplaces the tracked-file scan: it lives in `tests/repo_gates.rs:226` (`make issue-refs`), not in `docs_contract`.

REWRITE TO: "`raf-workplan` holds the single definition of the issue-reference and process-wording matcher as a public function. `tests/support/mod.rs` deletes its pattern constants, and its `banned_in_line` calls the `raf-workplan` function, so `repo_gates` (every tracked file) and the `Gate` (loaded documents) share one pattern set. AGENTS.md:133-134 and `tests/support/mod.rs:6-9` are updated to name the new single definition site. The existing `repo_gates` fixtures (`tests/repo_gates.rs:557-561`) stay as the cross-check."

### B4 (BLOCKING): "writes through the tree" needs a change to the delivered `raf-source` capability, and the plan has no record for it

Governing text:

- `docs/capabilities/workplan-document-model.md:46-49`: "A change to a delivered capability puts the capability back under an open record, which moves it to `implementing` ... Everything that moves a capability between states is a change."
- `AGENTS.md:220-222`: "changing a state or adding a capability edits that table in the same commit".

The `raf_source::Tree` trait (`crates/raf-source/src/tree/mod.rs:146-219`) is read-only: `read`, `read_unseen`, `identity`, `list`, `entry`. `cap:source-index` is `delivered`.

Offending plan lines:

- plan:39 (J13): "The library gives `Workplan::sync() -> SyncReport`, which writes through the tree."
- plan:129: "`examples/docs_sync.rs` writes through `DiskTree` for this repository"

The plan has no `raf-source` task, no `work/changes/source-index.md`, and no state change for `cap:source-index`.

REWRITE TO: "`Workplan::sync()` reads through the read-only `Tree` and returns a `SyncReport` with the new full text of each file it would change. It writes nothing itself. `examples/docs_sync.rs` writes those texts under the manifest directory, only to paths the report names, and those paths came from the tree listing. `raf-source` is not modified." If writing has to go into `raf-source`, add a task that opens `work/changes/source-index.md`, moves `cap:source-index` to `implementing` (with the INVENTORY edit in the same commit), and delivers it again before completion.

### B5 (BLOCKING): committed documents would describe `raf-workplan` as unbuilt after it is built

Governing text:

- `docs/architecture/okf-profile.md:208-210`: "A committed file holds no ... wording that refers to how the work was planned or organised. The account describes the system as it is."
- `AGENTS.md:243`: "Documents describe the system, not its history."
- `AGENTS.md:220-223`: "changing a state or adding a capability edits that table in the same commit, until `raf_workplan::Gate` replaces it."

Offending plan line (plan:148): "Record the outcomes in the change records, then complete (delete) them, and update the AGENTS.md inventory."

`INVENTORY` is in `tests/docs_contract.rs:70-95`, not in AGENTS.md. Once Tasks 1-8 land, these statements become false, and the plan updates none of them:

- `docs/architecture/okf-profile.md:31-40` (the cross-document checks are in the document-contract test, which "`raf-workplan` replaces once it is implemented")
- `docs/architecture/okf-profile.md:173-174` ("None exist yet." about generated blocks)
- AGENTS.md:41-45, :49-56, :63, :74, :208-213 and :222-223
- README.md:31 ("designed — not yet published")
- CONTRIBUTING.md:35-36
- `tests/docs_contract.rs:61-62` ("`raf_workplan::Gate` replaces it", though J1 keeps INVENTORY)

REWRITE TO: "In the commit that sets both states to `delivered`, also do the following:

- Edit `INVENTORY` in `tests/docs_contract.rs` (both rows go to `delivered`).
- Update the AGENTS.md crate table, workspace-member prose, layout row and §6 workflow text to name `raf-workplan` as a member whose document model and generated blocks are delivered. The text must say that `docs_contract` keeps the inventory and repository-policy checks and calls `raf_workplan::Gate` for the rest.
- Rewrite `okf-profile.md:31-40` to say `raf-workplan` performs the cross-document checks, and replace 'None exist yet' with the three kinds.
- Update the README.md crate row and CONTRIBUTING.md:35-36.
- Replace the `tests/docs_contract.rs:61-62` comment with the retained role of INVENTORY.

Each edit states current fact only, with no history wording."

### A1 (ADVISORY): report the `make check` evidence exactly

Governing text:

- `AGENTS.md:126`: "Run `make check` before claiming work is done. It stops at the first failure."
- `AGENTS.md:137`: "A missing tool is a hard failure, not a skip."

Offending plan lines:

- plan:18: "the local `make cross-check` cannot run on the pinned toolchain, so hosted CI is its evidence"
- plan:149: "`make check` (all gates; cross-check from hosted CI)"

Locally, `make check` stops at `cross-check` (Makefile:124), so `licenses` and `issue-refs` never run through it. The session's no-toolchain-change decision permits the substitution, but the claim has to be stated exactly.

REWRITE TO: "Run `make check`. If it stops only at `cross-check` because the session-pinned toolchain lacks the three targets (toolchains are not changed), run `make licenses issue-refs` after it. In the PR body, cite the hosted CI cross-check job for the head commit as the cross-check evidence. Do not report that `make check` passed locally."

### A2 (ADVISORY): fixture literals for R25 will trip `make issue-refs`

Governing text: `AGENTS.md:240-245`: "`make issue-refs` scans every tracked text file and fails on a hit". The repository's precedent is to assemble such literals so no single literal matches: `tests/docs_contract.rs:44-53` and `:1492-1493` (`concat!("see PR", "#", "9")`), and `tests/support/mod.rs:18-20`.

Offending plan line (plan:73): "`#123`, `issue 12`, `PR 5` and GitHub issue or pull URLs in a loaded document each fail."

REWRITE TO: "R25 fixtures build each banned token with `concat!`, following the existing precedent, so no committed line of `crates/raf-workplan` holds a literal that `repo_gates` reports. No test function name or doc comment uses those forms either."

### A3 (ADVISORY): the generated `docs/capabilities/index.md` needs its SPDX lines

Governing text:

- `AGENTS.md:176-178`: "Every `*.md` file is CC-BY-4.0. It opens with the two HTML-comment SPDX lines".
- `docs/architecture/okf-profile.md:56`: "apart from its SPDX comment, every non-blank line lies inside a generated block".

Offending plan lines:

- plan:33 (J7): "`docs/capabilities/index.md` is created by the first sync, with no frontmatter and only a `catalogue` block."
- plan:146

REWRITE TO: "Task 8 writes `docs/capabilities/index.md` with the two HTML-comment SPDX lines and an empty `catalogue` block, and `make docs-sync` fills the block. Sync leaves the SPDX comment untouched. A fixture shows that an index with only the SPDX comment and a catalogue block is accepted, beside a hand-written index that is rejected."

### A4 (ADVISORY): record the outcomes in a commit before the one that deletes the records

Governing text: `AGENTS.md:232-233`: "The change record is then **deleted**; Git keeps the history." Plan:28 says the outcomes are "recorded first, as the previous crates did".

Offending plan lines: plan:148-150, where one commit both records the outcomes and deletes the records, so the outcomes never reach the history.

REWRITE TO: "Task 8 makes two commits. The first is `docs(changes): record the workplan model and generated-docs outcomes`, which sets each item's outcome. The second is the delivery commit, which deletes both records and sets the states, with the INVENTORY edit from B5 in the same commit."

### A5 (ADVISORY): name the dependency behind the `tempdir` fixture

Governing text:

- `deny.toml:29-31` (`multiple-versions = "deny"`, `wildcards = "deny"`)
- `AGENTS.md:89-92` (MSRV check over all targets)

Root `Cargo.toml` has no temporary-directory crate today.

Offending plan line (plan:139): "A `tempdir` fixture covers a clean and a broken repository."

REWRITE TO: "The on-disk Gate fixture creates a unique directory under `std::env::temp_dir()` and removes it on drop, with no new dependency. If a crate is used instead, the plan names it with an exact version, as a dev-dependency of `raf-workplan` only, and confirms `make deny` and `make msrv` with it."

First-pass verdict: NEEDS-REVISION

## Recheck

Rechecked against `.stage/raf-workplan-model-documents-check/plan.md`, revision 2. All plan line numbers below refer to revision 2.

### Prior findings

| Finding | Status | Evidence |
|---|---|---|
| B1 unbuilt claims in delivered documents | **DISCHARGED** | G15 (plan:47) requires every statement in every section to be built or listed in U, and makes any other unbuilt sentence block delivery. Task 9 step 4 (plan:215) reconciles every section. List U (plan:219-224) rewords the `cargo workplan` Usage, the CLI command block, the preview constraint, the mechanical-generation list and the obligations sentence. G14 (plan:46) and Task 8 (plan:203-207) now build `preview`. `affects` is widened (plan:236). Three sentences are still missing from U; see N1 (advisory). G15 already blocks delivery on them. |
| B2 link check weakened | **DISCHARGED** | G4 (plan:36): `docs_contract` builds a `MemoryTree` from `tracked_paths()`, so "a link to an untracked file still fails, and a tracked symlink is still a violation". R28 (plan:127) runs the tracked-tree check in addition to the checkout `Gate` run, so the existing gate strength is kept. |
| B3 second pattern set | **DISCHARGED** | G5 (plan:37) and Task 3 (plan:164-165): the matcher moves into `raf_workplan::refs`, `tests/support` calls it, and AGENTS.md is updated. AGENTS.md:133-134 is in list D (plan:229). The `repo_gates` and `make issue-refs` checks stay in place. |
| B4 sync writes through a read-only `Tree` | **DISCHARGED** | G6 (plan:38): "Sync returns texts; callers write. ... The `Tree` seam stays read-only, and `cap:source-index` is unchanged". `write` is limited to regular files already read through `DiskTree`, or the new `index.md`. |
| B5 stale committed documents | **DISCHARGED** | List D (plan:226-234) names `okf-profile.md:31-40, 173-174`, the AGENTS.md ranges, `README.md:31`, `CONTRIBUTING.md:32-36`, the `docs_contract.rs:61-62` comment and `INVENTORY`. One item in D raises a new workflow point; see N3. |
| A1 `make check` evidence | **DISCHARGED** | Plan:22 says "Nobody claims a local `make check` pass". Plan:241-242 run `make -k check`, then `make licenses issue-refs`, and use hosted CI cross-check. Plan:251 reports the actual validation. |
| A2 `concat!` fixtures | **DISCHARGED** | Plan:25, and plan:125 (R24/R25 "built with `concat!`"). |
| A3 SPDX on `index.md` | **DISCHARGED** | G12 (plan:44): the first sync writes both SPDX HTML-comment lines from `SyncOptions` and refuses when they are unset; the SPDX comment is exempt from `index-hand-written`. See N2 for the Rust side. |
| A4 outcomes before deletion | **DISCHARGED** | Task 9 steps 5-6 (plan:216-217) and the two commits (plan:245-246). See N4 for what each commit must hold to pass the hook. |
| A5 temporary-directory dependency | **DISCHARGED** | Plan:27 ("No new dependencies ... `std::env::temp_dir()` for disk fixtures") and R23 (plan:124). |

### New findings (all ADVISORY)

#### N1 (ADVISORY): three unbuilt sentences are missing from list U

Governing text: `workplan-document-model.md:176-178` (as in B1).

Offending plan lines: plan:219-224 (list U). U leaves out these sentences:

- `workplan-document-model.md:104-106`: "`status`, the catalogue and the reconcile obligations report each one as a gap" (reconcile is not built).
- `workplan-document-model.md:115-116`, `workplan-generated-docs.md:51`, `:53`, `:60-61` and `:124`, and `okf-profile.md:55`: these name `docs sync`, the CLI subcommand, as the operation the code performs.

G15 makes any sentence missed by U block delivery, so this is not blocking. Listing them now avoids a failed delivery review.

REWRITE TO: add to U:

- "document-model Contract, Binding resolution: 'the reconcile obligations report each one as a gap' becomes '`status` and the catalogue report each one as a gap; the reconcile obligations are defined by [reconcile](./workplan-reconcile.md)'."
- "Wherever the generated-docs and document-model text, and `okf-profile.md:55`, names `docs sync` as the operation, it names the sync (`Workplan::sync`/`write`, `make docs-sync`) instead. The CLI subcommand is mentioned only by link."

Also add `okf-profile.md:55` to list D.

#### N2 (ADVISORY): SPDX tag text in Rust source and fixtures

Governing text: `AGENTS.md:183-185`: "SPDX tags quoted in prose are wrapped in `REUSE-IgnoreStart` and `REUSE-IgnoreEnd` comments so `reuse lint` does not read them as headers," and "`uvx reuse@6.2.0 lint` must pass."

Offending plan line: plan:44 (G12). The renderer and the `index.md` fixtures will hold the `SPDX-FileCopyrightText`/`SPDX-License-Identifier` tag text in `.rs` files. `reuse lint` would read that text as the file's own licence or copyright. `make reuse` is in the per-task gate (plan:142), so the failure would be caught, but the plan should name the fix.

REWRITE TO: "Every SPDX tag string in `crates/raf-workplan` source and fixtures sits between `REUSE-IgnoreStart` and `REUSE-IgnoreEnd` comments, or is assembled with `concat!`. The `docs_sync` example takes the copyright and licence text this repository uses as explicit `SyncOptions` input, not as defaults, as G12 requires."

#### N3 (ADVISORY): editing the delivered `source-index.md` needs a stated disposition

Governing text: `workplan-document-model.md:46-47`: "A change to a delivered capability puts the capability back under an open record, which moves it to `implementing`".

Offending plan line: plan:227 (list D): "`docs/capabilities/source-index.md:1488-1500` (the Verification text citing binding fixtures in the document-contract test now cites the `raf-workplan` fixtures)".

`cap:source-index` is `delivered`, and no record of this PR lists it. The edit only re-points two Verification sentences and leaves the contract unchanged. Leaving them stale would instead break `okf-profile.md:208-210`.

REWRITE TO: name the reading the plan relies on: "This is a Verification-only citation correction with no contract effect, made without reopening `cap:source-index`." Ask the user to confirm that reading. If the user wants the workflow applied literally instead, add `cap:source-index` to `change:workplan-document-model`'s `changes` with `affects: cap:source-index#verification` for the duration of the PR. That moves it to `implementing` and edits `INVENTORY` until the delivery commit restores both.

#### N4 (ADVISORY): spell out what each Task 9 commit contains so both pass the pre-commit hook

Governing text:

- `AGENTS.md:127-131`: the hook runs `docs_contract` on the staged index.
- `AGENTS.md:220-222`: a state change edits `INVENTORY` in the same commit.
- `okf-profile.md:147-149`: `designed` and `delivered` rules.

Offending plan lines: plan:211-217 and plan:245-246. Steps 1-4 (blocks, state set to `delivered`, bindings) all come before the first commit (step 5). A commit holding `delivered` while the change records still exist fails the open-record rule. A commit holding non-empty bindings while the state is `designed` fails too.

REWRITE TO:

- "Commit 1 (`docs(changes): ...`) holds only the item outcomes in the two change records; the capability documents are untouched."
- "Commit 2 (`docs(capabilities): ...`) holds the following together: the state changes, the bindings, the synced blocks, the new `index.md`, the reconciled sections, list D, the `INVENTORY` edit, and the deletion of both change records."

#### N5 (ADVISORY): keep the Stage process files out of the repository

Governing text: `AGENTS.md:240-243` (no wording about the development process "in any committed file").

Offending plan lines:

- plan:137: "`tasks/TN-review.md`"
- plan:196: "`tasks/T7-dispositions.md`"

These are relative paths. Under the worktree root they would be new tracked files full of process wording.

REWRITE TO: "`$STAGE/tasks/TN-review.md` and `$STAGE/tasks/T7-dispositions.md` (under the git-ignored `/.stage/`); commits stage explicit files only, and never stage these."

### Recheck summary

All five blocking findings and all five prior advisories are discharged. The five new findings (N1-N5) are advisory: each is a convention mismatch with a cheap fix, and none violates a standing constraint or user decision. N3 also asks for the user to confirm one reading of the workflow.

VERDICT: PASS
