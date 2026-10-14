# T6 review: roadmap (`Workplan::status`, `Roadmap`)

Scope: staged diff on top of 2992c66 (roadmap.rs, tests/roadmap.rs, model.rs, lib.rs, CHANGELOG.md), against plan Task 6, G18, the R16 row, the
catalogue kind in workplan-generated-docs.md, and `status` in workplan-cli.md. Mutations ran in a /tmp copy (/tmp/t6); the worktree was not changed.

## Evidence

Gates on the staged state, all green:
- `cargo test -p raf-workplan --locked`, debug and `--release`: pass (roadmap binary: 10 tests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean.
- `cargo test --locked --test docs_contract --test repo_gates`: 141 and 57 pass.
- `make reuse`: compliant, 306/306.
- rustfmt --check on the four changed Rust files: clean.
- `cargo doc -p raf-workplan` with `-D warnings`: clean.
- The docs_contract scan covers the diff, and no process wording was found.
- SPDX headers are present on both new files.

Mutants, each run against tests/roadmap.rs:

| Mutant | Result |
| --- | --- |
| State rank swap (designed/considered) | killed |
| ORDER constant first element changed | killed |
| `gaps` computed for every state | killed |
| `gaps` never computed | killed |
| EmptyList push removed | killed |
| unresolved filter removed | killed |
| unresolved filter inverted | killed |
| open-changes filter inverted | killed |
| `partition_point` bound changed | killed |
| `planned` using the wrong group | killed |
| link_target mutated | killed |
| list key text "tests" changed | killed |
| **explicit slug sort removed** | **SURVIVES** (all 10 tests pass) |
| open_changes `sort()` / `dedup()` removed | survive; equivalent, because `Graph.changes_edges` is already sorted by (change, capability) and deduped |

## Findings

### REQUIRED

R1. The slug sort is not redundant, and the surviving mutant is a real gap in the tests, not an equivalent mutant.
- The loader yields entries in the tree's name order, and that order includes the `.md` suffix. `a-b.md` sorts before `a.md` ('-' 0x2d < '.' 0x2e), so slugs `a` and `a-b` load as `a-b`, `a`.
- Reproduced in /tmp/t6. With a fixture of `designed` capabilities `a` and `a-b`, the real code returns `["a", "a-b"]`. With the sort reduced to the state rank only, it returns `["a-b", "a"]`.
- The contract is "sorts each group by slug" (generated-docs catalogue), and `Graph` sorts by id, so the loader order is the wrong source.
- The current fixtures (eta/zeta, bare/mid, ...) have no prefix pair, so the mutant survives.
- Fix: add a test (e.g. `a_slug_that_prefixes_another_sorts_first`) with two same-state capabilities `a` and `a-b`, asserting `["a", "a-b"]`. Mutating the sort out then fails it. My /tmp test `prefix_slugs_sort_by_slug` does this: it passes on the real code and fails on the mutant.
- Drop the "equivalent mutant" claim.
- The same ordering also makes the fixture in `the_roadmap_does_not_depend_on_insertion_order_and_repeats` unable to catch it.

### ADVISORY

A1. Repository test brittleness (`this_repository_lists_every_capability_once_in_its_state_group`).
- It hardcodes `21`, so adding a capability document fails the test until the number is bumped. `plan.capabilities().len() == roadmap.len()` is circular (the same source).
- Non-vacuity would be better served by counting `docs/capabilities/*.md` minus `index.md` on disk and comparing that to the roadmap length. That also catches a document that fails to load.
- The `delivered` contains `okf-profile` and `source-index` assertion is fine (`contains`).

A2. Environment.
- The test locates the root through `CARGO_MANIFEST_DIR/../..`, so it does not depend on the cwd. It runs `git ls-files` there, so it requires a git checkout.
- It does not honour `RAF_GATE_ROOT` as `tests/support` does. The hook runs only `repo_gates` and `docs_contract`, so this test does not run in the snapshot today.
- When Task 7 moves the repository check into `docs_contract`, use `gate_root()` and `tracked_paths()` there. Nothing is wrong now.

A3. Wrong-role binding.
- A binding that resolves but has the wrong role (`binding-wrong-role`) is not a gap. This matches the literal contract ("empty or do not resolve"), and the `Gap` doc says so. It is reported by `check` for every state.
- No test pins the claim. Consider a fixture with an `implementing` capability whose `tests` list is a resolving non-test item and assert no gap.
- A practical consequence is that an `implementing` capability can show no gaps while `check` fails. State this in the CLI `status` help if relevant.

A4. Rustdoc says `status` "prints" the roadmap and the `catalogue` block "renders it". Neither the command nor the renderer exists yet (Task 5 and the CLI capability). The statements are forward-looking.

A5. Roadmap rows equal catalogue rows (the plan's Task 6 check, G18).
- This cannot be tested here because the renderer is Task 5. It must be the first assertion there (render each `RoadmapEntry` and compare to the block rows).
- The API serves it: id, slug, title, description, state, `link_target` (file name, correct because documents sit directly beside `index.md`; nested ones are findings) and `gaps`.
- `Gap` implements `Display` ("bindings.tests is empty"), which matches the CLI example.
- `open_changes` covers the CLI. No `Serialize` exists, so the CLI needs a mapper for `--format json`.
- Capabilities that failed validation are not entries (documented); the CLI should run `check` alongside.

A6. Dead redundancy.
- `open_changes.sort()` and `.dedup()` duplicate the graph's own ordering. They are harmless defence but are unkillable mutants. Keep, or drop with a comment.

## Answers to the review questions

1. Consumers: nothing is missing for Task 5 (link target, description, gaps) or for `status` (four states, `planned()` view, open changes).
2. Fidelity: gaps cover empty lists and unresolved entries, and cover a missing key or all-empty lists for `implementing` (`bare` gives three EmptyList gaps, tested). Wrong-role is not a gap, which is consistent with the spec (A3).
3. The sort is needed (R1).
4. The repository test is non-vacuous (state, path, title and description are compared per document) but uses a hardcoded count and the working-tree git (A1, A2).
5. Mutations done: see the table.
6. Docs, SPDX, wording and CHANGELOG are fine.
7. All gates pass.

VERDICT: BLOCKED

## Recheck

- R1: `slug_order_is_not_file_name_order` asserts the loader order `["a-b", "a"]` and the roadmap order `["a", "a-b"]`. With the sort reduced to the state rank in a /tmp copy it FAILS (left `["a-b","a"]`, right `["a","a-b"]`); on the real code it passes. Killed.
- Repository test: it counts capability files on disk independently of the loader, asserts that count is above zero and equals the roadmap length, and asserts the slugs are unique. It is not vacuous, and a document that fails to load breaks it. The document root is `RAF_GATE_ROOT` else `CARGO_MANIFEST_DIR/../..`. Run with `RAF_GATE_ROOT` at a snapshot copy that has no `.git`, it passes. The tracked list comes from `git ls-files` in the checkout (same as `tests/support`), and the test panics clearly when git is missing or fails. I did not exercise the no-git panic itself; it is by reading. Minor: the tracked list reflects the live checkout, not the snapshot (as `tests/support` does).
- Wrong-role test and docs rewording present.
- Gates on the staged state: raf-workplan tests (debug and release), workspace clippy with `-D warnings`, docs_contract (141) and repo_gates (57), `make reuse`, rustfmt: all pass.

VERDICT: PASS
