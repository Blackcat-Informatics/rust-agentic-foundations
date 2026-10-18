# T1 review: rustc differential harness

VERDICT: PASS

Findings F1 and F2 are open and must be closed by the end of Task 5. They do not block this commit.

## Verified
- `cargo test -p raf-source --locked --test rustc_oracle --test resolve_paths`: 11 and 5 passed, 1 ignored.
- `cargo clippy -p raf-source --all-targets` is clean. `cargo fmt --check` is clean.
- Default-run output (rustc 1.101.0-nightly 2026-09-30):

  | Item | Count |
  | --- | --- |
  | programs | 64 (64 judged) |
  | probes | 285 (133 accepted, 152 refused) |
  | divergences rule1 | 3 |
  | divergences rule2 | 1 |
  | divergences documented | 0 |
  | divergences unclassified | 0 |

  - The default run shows no documented divergence. The corpus supplies one (`glob_reexport_ambiguity_warned.rs`), and the corpus test requires one entry per known class.
- Missing `rustc` fails: every spawn panics with "the tool `rustc` cannot start". Nothing skips.
- The test asserts non-zero counts: `valid > 0`, `accepted > 0`, `rejected > 0`, a non-zero program count, and a non-empty corpus with at least one divergent entry and one entry per class.
- Two-phase compile: phase 1 reads verdicts from JSON diagnostics, mapped from line to owner. Phase 2 adds one `const _: () = assert!(probe::TAG == k)` per candidate item, so an accepted probe reveals which item it names. The lints `ambiguous_glob_reexports` and `ambiguous_glob_imports` run at `-W` and the noisy lint groups at `-A`. The harness does not use a blanket `-A warnings`, which matches Revision 2.
- `unclassified` fails the default test. `a_divergence_that_matches_no_class_is_unclassified` checks that path.
- The `documented` class exists and is tied to `ambiguous_glob_reexports`.
- Corpus replay:
  - It asserts that each entry still diverges as its recorded class.
  - It asserts that no divergence on any entry falls outside that class.
  - Entries marked `agree` must have no divergence.
  - `KNOWN.contains` makes a class that has left the list fail.
- Negative fixture: `a_wrong_index_answer_is_reported_beside_the_right_one_which_is_not` covers all three divergence kinds, with the untampered index agreeing beside them.
- A grep of the new test files and `resolve.rs`/`workspace.rs` for Instant, sleep, timeout, Duration, SystemTime and elapsed finds nothing. The only hits are `thread::available_parallelism` and `thread::scope`, which are not timing.
  - `support/mod.rs` has a pre-existing `Duration` watchdog (`within_watchdog`). It is not in this diff, and the oracle does not use it.
- A grep for issue or PR numbers and process words (task, M1, stage, plan, review) in the new committed text and the corpus finds nothing.
- SPDX headers are present on all new `.rs` files. The corpus files carry none by design: `REUSE.toml` annotates `fuzz/corpus/**` because a header would change the input.
- The oracle uses only the public `resolve_from_in`.
- `resolve_from` and `resolve_from_in` have rustdoc with `# Errors` and a doctest. Three passing-neighbour tests in `resolve_paths.rs` cover them.
- `ResolveError::UnknownModule` is a new variant on a `#[non_exhaustive]` enum, so adding it is not a semver break. It is covered by the `Display` hostile-text test and the match arm there.

## Findings
- F1 (open, closes in Task 5, not blocking): the work adds public API, `Workspace::resolve_from`, `resolve_from_in` and `ResolveError::UnknownModule`, but `CHANGELOG.md` `[Unreleased]` has no bullet for them, and `docs/capabilities/source-index.md` does not mention `resolve_from` yet.
  - Task 5 must add both, per Revision 2, "documented in the Contract".
  - The CHANGELOG bullet must not use process wording.
- F2 (open, recommendation): `write_corpus` is `#[ignore]`d. AGENTS.md says "no skipped checks".
  - It is a corpus generator, not a check, and `tests/corpus_replay.rs:211` has the same `#[ignore]` precedent, so it does not strictly have to change.
  - It can still mutate tracked files and shows up as "ignored" in every test run.
  - Do not bind it as capability evidence, because the workplan gate refuses ignored tests in bound evidence.
  - Preferred fix: drop it, since the six corpus files are committed and the minimiser is already covered by a normal test. Otherwise keep the precedent and note the reason in its doc comment.
- F3 (low, note): `classify` is deliberately broad. Any "index too permissive" divergence for which the all-public variant agrees is labelled rule 2 when rustc gave E0659 or E0573, and rule 1 otherwise.
  - A different visibility bug in the index would be filed as rule 1 or rule 2 instead of failing as unclassified.
  - The all-public agreement check keeps this narrow. For Task 4's "zero divergences" claim, rely on the corpus and per-rule fixtures, not on the class labels alone.
  - Once Tasks 2 and 3 remove `Rule1` and `Rule2` from `KNOWN`, any such divergence will fail.
- F4 (nit): the `write_corpus` doc comment has a ragged line break ("to be renamed after its shape; the / first of each kind"). The same function's doc says "`<class>.rs`", but the committed files carry shape names.
- F5 (note): the full-run claim of at least 5,522 programs with one divergence per rule was not checked, as instructed. The default run already finds rule 1 three times and rule 2 once.
