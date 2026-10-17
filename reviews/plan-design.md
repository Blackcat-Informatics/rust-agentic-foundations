# Plan design review (SOLID / ETHOS)

Reviewed plan: `.stage/acceptance-scenarios-a-b-workplan-self/plan.md`

Code read: `tests/docs_contract.rs` (header, `INVENTORY`, `violations` expectations 255-315, repository tests 552-620), `tests/support/mod.rs`, `tests/repo_gates.rs:31`, `.githooks/pre-commit`, `.githooks/pre-merge-commit`, `Makefile`, root `Cargo.toml`, `.github/workflows/ci.yaml` (test invocations), `crates/raf-workplan/src/gate.rs`, `crates/raf-workplan/src/generated.rs:215-285`, `crates/raf-workplan/src/rules.rs:150-172`, `crates/raf-workplan/tests/gate.rs`, `crates/raf-workplan/tests/roadmap.rs:440-470, 568-630`, `crates/raf-workplan/tests/support/project.rs` (header), AGENTS.md 36-46 and 134-153, CONTRIBUTING.md 52-88.

## Sound parts (no finding)

- Moving `repository_documents_satisfy_the_document_model` (`tests/docs_contract.rs:582-588`) into `tests/self_workplan.rs` rather than copying it: one owner, and the hook does not run the Gate twice. The comment at `docs_contract.rs:584-586` about the `.git`-less snapshot must travel with it; it documents the fallback at `gate.rs:160-165`.
- Hook cost: `.githooks/pre-commit:38` runs one `cargo test` with two `--test` selectors; a third selector adds one root test-binary link and no second Gate load, because the Gate test moves. `.githooks/pre-merge-commit:8` execs `pre-commit`, so one edit covers both hooks. `make test` (`cargo test --workspace`) and CI (`ci.yaml:69`) pick up a new root test binary with no change; the Makefile needs comment edits only (`Makefile:9-10`).

## BLOCKING

### B1. Repository policy and a third capability inventory land in the wrong binary

Plan line 16: "test asserts the 12 M1 capabilities are `delivered`, each holds a generated `verification-matrix` block, and no `work/changes` record names an M1 capability".

Evidence:
- The states of those 12 are already pinned in `tests/docs_contract.rs:75-98` (`INVENTORY`) and again in `crates/raf-workplan/tests/roadmap.rs:452-465` (`BUILT`). A third slug list in `self_workplan.rs` means the next delivered capability edits three tables that fail with three different messages.
- "no `work/changes` record names an M1 capability" is already a model rule: `crates/raf-workplan/src/rules.rs:155` ("state delivered must not be listed by the open change record"), and the Gate's `assert_clean` runs it (`gate.rs:276-304`). Re-asserting it beside `assert_clean` is a second path to the same verdict.
- "holds a generated `verification-matrix` block" is genuinely new policy: `generated.rs:230-271` judges only blocks that exist (`generated-stale`, `generated-misplaced`, `generated-unknown-kind`); an absent block is silent. `docs_contract.rs:15-35` declares itself the home of "this repository's own policy, which no model rule states", and requires every rule family to go through `violations` so "a family that is left out of it fails a fixture test". Putting this rule in `self_workplan.rs` creates a second home for repository policy, outside the `violations` discipline, and the plan names no fixture test for it; a regression that drops the block from a delivered capability would be caught only by a test nobody has proven fires.

Consequence: two binaries own repository policy, one of them without a negative test, and the capability inventory is maintained in three places.

REWRITE TO: `tests/self_workplan.rs` owns exactly the Gate run plus an empty-load guard: `let opened = Gate::workspace(gate_root()).open()` (panic on `Err`), `assert!(!opened.workplan().capabilities().is_empty())`, then `opened.workplan().check()` must be empty, reported through the Gate (or keep `assert_clean()` and add the guard beside it). The matrix-presence rule goes into `docs_contract.rs` as a new family inside `violations`, keyed off `INVENTORY` entries whose state is `delivered` (reading each document through `read_tracked` and `raf_okf::Document`/`FencedBlock`, which the file already imports), with the fixture test its header requires (remove the block from one delivered capability in a `Repo` copy, assert the violation). No new slug list anywhere; the delivered/open-change assertion is dropped because `assert_clean` already enforces it.

## ADVISORY

### A1. The planted-fault fixture rebuilds a library test at the root

Plan line 14: "a planted fault (tracked doc with a broken link in a fixture tree) fails it"; line 22: "fixture test that the gate fails on a planted fault".

Evidence: `crates/raf-workplan/tests/gate.rs:160` (`a_broken_repository_panics_listing_every_finding`, asserts `link-broken` at `file:line`, lines 175-203), `gate.rs:351` (`a_link_to_a_file_the_tracked_set_leaves_out_is_link_broken`) and `gate.rs:403` (tracked paths from a real index). The root package has no fixture builder: `tests/support/mod.rs` holds only the gate-root and tracked-file readers, and `Scratch::repository`/`demo_files`/`synced` live in `crates/raf-workplan/tests/support`, unreachable from the root crate. The root `Cargo.toml:15-18` has no `tempfile`. A root fixture must therefore re-implement `gate.rs:37-86` (scratch dir, `put`, demo `Cargo.toml` + `src/lib.rs` + capability + coverage record) by hand.

Consequence: a second copy of a scratch-repository builder, and it proves nothing about this repository's wiring; a fault planted in a scratch tree exercises the library, which `gate.rs` already covers. The only fault a root test can meaningfully refuse is an empty or wrong-directory load, which `GateError::NoCapabilities` (`gate.rs:306-328`) plus the capability-count guard in B1 cover.

REWRITE TO: drop the root fixture test. Record `crates/raf-workplan/tests/gate.rs:160` and `:351` as the planted-fault evidence in `validation.md`, and keep the empty-load guard from B1 as the root binary's negative check.

### A2. A new binary over the shared `support` module trips `dead_code` under `-D warnings`

Plan line 22: "Create the test file (uses `tests/support/mod.rs` `gate_root()`)".

Evidence: `Makefile:58` runs clippy `--all-targets -- -D warnings`; `dead_code` is a rustc warning, so it is denied. `tests/support/mod.rs:25-28` already carries `#[allow(dead_code, reason = ...)]` on `banned_in_line` solely because `docs_contract` does not call it; `repo_gates.rs:31` and `docs_contract.rs:48` between them use every other function. A `self_workplan.rs` that does `mod support;` and calls only `gate_root()` leaves `tracked_paths`, `read_regular`, `read_tracked`, `check_handle` and `decode_text` dead in that binary.

Consequence: `make clippy` fails, or the plan adds five more per-function allows and erodes dead-code detection for both existing gate binaries.

REWRITE TO: one attribute at the inclusion site, following the repository's existing pattern: `#[allow(dead_code, reason = "this binary uses only gate_root; the readers serve repo_gates and docs_contract")] mod support;` in `self_workplan.rs`. State this in Task 1 so it is not discovered at `make check`.

VERDICT: NEEDS-REVISION
