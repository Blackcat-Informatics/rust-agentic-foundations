# R7 review: fuzz reach

## Scope

Independent review of remediation item R7 (fuzz reach: G8, G14, S-12) on top of R6. Reviewed in
`/tmp/raf-r7-review`, a `git archive e472918` copy with `raw/R6.patch` and `raw/R7.patch` applied
(`git apply -p1`) and staged with `git add -A`. Contract: "### R7" of `remediation-plan.md`.

## Evidence

| Gate | Result |
| --- | --- |
| `make fuzz` | exit 0; three targets run in order `parse-source`, `parse-document`, `load-workspace`; each run line carries `-max_len=400000 -rss_limit_mb=2048 -malloc_limit_mb=512`; no `warning:` line (the only `warning` match is the `-D warnings` flag in the clippy command echo) |
| `parse-source` INITED | `files: 56 min: 205b max: 390022b total: 7543806b`, `#57 INITED ... corp: 45/5410Kb` (all 56 files loaded; the largest, 390022 bytes, fits 400000) |
| `load-workspace` INITED | `4 files ... total: 1047b`, `#5 INITED cov: 4501 ft: 7765` |
| `make reuse` | compliant, 256/256 files |
| `docs_contract` + `repo_gates` (staged) | 139 passed, 55 passed |
| `make msrv` | exit 0, includes `raf-source-fuzz` |
| `corpus_replay` | 8 passed, 1 ignored (reads `fuzz/corpus/parse-source`) |
| Mutation probe | see REQUIRED 1 |

Corpus rename: no stale `parse_source` corpus path remains in Makefile, CI (`ci.yaml` calls `make fuzz FUZZ_DENY=--locked`,
so it is consistent with the Makefile, including `--locked` for deny), docs, `repo_gates` or `corpus_replay.rs`.
Bin names are kebab-case and match the Makefile loop and the docs. The Verification sentence in
`source-index.md` is restored and the 4096-byte text is gone. CONTRIBUTING, AGENTS, SECURITY,
census-reference-scan and CHANGELOG changes are accurate for what they describe and contain no process wording.

## Findings

### REQUIRED

1. **The `load-workspace` target never reaches `settle` or `resolve` for any input with source items; the
   S-12 fix is ineffective as built.** `cargo fuzz` compiles with `--cfg fuzzing`, and with that cfg
   `proc-macro2` drops span locations (line and column are 0), even though `raf-source` enables the
   `span-locations` feature. Every `Position` lookup in `facts.rs` then fails, and every file that holds an item ends in
   `LoadErrorKind::UnsupportedItem { "a position outside the text" }` at `src/lib.rs:0`. The harness treats a load
   error as a normal result and returns. Measured:
   - Instrumented harness over each seed alone: all four seeds (`seed_globs`, `seed_library`,
     `seed_targets`, `seed_workspace`) print `load-err src/lib.rs:0: ... a position outside the text`; only the
     empty input (manifest only, `items=0`) loads.
   - The same four seeds through `Workspace::load` in a normal build load fine (6, 11, 10, 4 items). The
     failure reproduces in a normal build with only `RUSTFLAGS="--cfg fuzzing"`.
   - Mutation tests: a panic injected in `Workspace::resolve` for `demo::S::m` fired (but only because the
     empty-manifest input runs `QUERIES`, not because source loaded). A panic injected unconditionally in `settle_glob`
     was never hit in 20000 runs. With `cargo fuzz run --no-cfg-fuzzing` the same `settle_glob` panic is found within
     5000 runs, seeds load (`items=10, 4, 11, 6`), and `INITED cov` rises from 4501 to 8441.
   - Consequence: the target fuzzes manifest parsing and discovery only. The claim in `source-index.md`
     ("loads it with `Workspace::load` and resolves paths through the index") and in the CHANGELOG is not true of the
     running fuzzer. `parse-source` is not affected in outcome (its visitor does not use positions) but it also
     runs without spans.
   - Fix: pass `--no-cfg-fuzzing` to `cargo fuzz run` (at least for `load-workspace`; preferably all three
     targets, since nothing in the harness uses `cfg(fuzzing)`), and re-measure INITED. Add a guard so the
     regression cannot go silent: for example, in the harness, `debug_assert!`/`assert!` once (via
     `std::sync::Once`) that a built-in known-good tree loads and resolves `demo::f`, or a replay test of the
     `load-workspace` seeds that asserts `Workspace::load` is `Ok` for each seed.

### ADVISORY

1. The plan's "about 1.8 MB" for the `parse-source` corpus is out of date: after R6's chain files the corpus is
   7.5 MB (56 files). The gate is met in substance (full corpus loaded, max 390022 < 400000); update the plan
   text only.
2. Limits are sane. A 400000-byte input is admitted to at most 320000 tokens (about 512 MiB parse peak per
   `budget.rs`, and the run peaked at 432 MB rss), so a legitimate in-budget input cannot exceed
   `rss_limit_mb=2048`. R3's 256/512 MiB item and fact budgets are per load totals and not single allocations;
   no single allocation of a load from at most 400 KB of input approaches 512 MB, so no false findings are
   expected. (Once REQUIRED 1 is fixed, re-run to confirm the peak with real loads.)
3. Harness reach beyond `resolve`: `resolve_in` (namespaces), `resolve_in_package`, `items_in` and
   `items_under` are not exercised. Cheap to add. Seed quality is otherwise good (glob cycles, `#[path]`,
   `pub use` renames, workspace members, explicit targets, `cfg(test)` modules, trait impls) once REQUIRED 1 is fixed.
4. `parse-document` runs with `lim: 4096` growth under `-max_len=400000`; this is libFuzzer's normal ramp and not a
   problem, but the corpus for it holds only 22 KB, so the larger limit is mostly unexercised in 5000 runs.

## Verdict

Superseded by the Recheck below.

## Recheck

Fresh `git archive e472918` copy (`/tmp/raf-r7-re`) with the regenerated R6 and R7 patches applied and staged.

- **Makefile:** `cargo fuzz run --no-cfg-fuzzing` for all three targets in the one loop; CI only calls `make fuzz`, so it is consistent. The comment explains why.
- **Shared input code:** `fuzz/fuzz_targets/workspace_input.rs` is used by the fuzz target (`mod workspace_input`) and by `crates/raf-source/tests/workspace_seeds.rs` (`#[path]`). The fuzz target runs `self_check` once (a seed with `pub mod a`, `fn f`, `struct S` must load with at least 3 items, else panic).
- **Injected panics (fuzzer built as the Makefile builds it):** unconditional panic in `settle_glob` found in the first run (`mut-settle`, deadly signal); panic in `Workspace::resolve` for `demo::a::f` found (`mut-resolve`). Both are now reached through loaded source, not the empty input.
- **Regression guard:** `cargo test -p raf-source --test workspace_seeds` passes normally (2 passed); with `RUSTFLAGS="--cfg fuzzing"` both tests FAIL with `src/lib.rs:0: unsupported item: a position outside the text`. Running the fuzzer without `--no-cfg-fuzzing` panics at the self-check (`the self-check seed does not load`) before any input, so the regression cannot go silent.
- **New harness calls reviewed:** `resolve_in` over Type, Value and Macro, `resolve_in_package` per package name, `items_in` per file, `items_under` per query and item path, `test_fns`. Results are dropped or black-boxed; load and resolution errors are normal; only panics, overflow or memory are findings. Cost is linear in items times paths times namespaces; at most 400000 bytes it stayed within limits (no rss or malloc findings over 5000 runs per target).
- **Gates:** `make fuzz` exit 0, three targets, no `warning:` line, no SUMMARY or limit line; `load-workspace` INITED cov 8507 (was 4501), `parse-source` INITED 56-file corpus, `parse-document` INITED cov 1963 with the ten full-document seeds (largest 116649 bytes); `make reuse` compliant; `docs_contract` 139 and `repo_gates` 55 passed (staged); `make msrv` exit 0; `corpus_replay` 8 passed; clippy `-D warnings` on `raf-source --all-targets` clean.

Remaining advisory (not blocking): the `load-workspace` corpus is four seeds of about 250 bytes, so within 5000 runs libFuzzer's length ramp stays near 283; a longer campaign (`FUZZ_RUNS`) will reach larger inputs. The plan text "about 1.8 MB" remains stale (corpus is 7.5 MB).

All findings from the first review are resolved.

VERDICT: PASS
