# T3 review: bound executors and proof parsers

## Scope

Staged, uncommitted change on top of 8c5eeba: `crates/raf-workplan/src/bound.rs` and
`bound/{evidence,parse,plan,run,tests}.rs`, `tests/bound.rs`, the `lib.rs` module
line and docs, the CHANGELOG entry and the two CI jobs in `.github/workflows/ci.yaml`.
Read against plan D3, D5, D11, Task 3, T1/T2 reviews, AGENTS.md, SECURITY.md, and the
complete-gate Contract (argv table, four patterns, Proof, Examples, Verification) and the
reconcile `example-failing` kind. Experiments ran in a scratch worktree (removed); nothing
was staged, reset, stashed or committed in the review worktree.

## Evidence

### (1) Proof rules, tried against real cargo and the parsers
* Zero entries, two entries, wrong-name single entry, `0 passed`, `#[ignore]`, test behind
  a disabled `cfg(feature)`, a non-`#[test]` fn, the same name in lib and an integration
  test (pinned target runs only its own), a failing test: each refuses or fails, in both
  the literal-string unit tests and `tests/bound.rs` against real cargo.
* Doc tests: `no_run` (`- compile ... ok`) and `compile_fail` (`- compile fail ... ok`)
  refuse with the "bind a runnable doc test" text; `ignore`d doc test refuses; failing
  sibling `foo_more` fails the binding through exit status while `ok_lines` still
  counts the bound entry; `Wrapper<T>::get` passes; no doc test refuses; a selected entry
  absent from the output refuses; `-`-leading item path never reaches the runner.
* Forgery attempts: pass needs exit 0 AND exactly one `test result:` line with
  `1 passed; 0 failed; 0 ignored`. A failing test that prints a result-shaped line yields
  two result lines plus a non-zero exit, so it fails. Doc `... ok` lines are matched as the
  whole line and the first line naming the test decides; rustdoc prints a failing test's
  own output after the status lines and the exit status is checked before the ok count, so
  injected text cannot turn a failure into a pass. Residual: a test that writes a
  result-shaped line with raw stdout and calls `process::exit(0)` is counted as one passed
  test. The test is workspace code (trusted build input) and could equally be an empty
  body, so this is not a bypass of the trust model (ADVISORY A3).
* CRLF is covered (`str::lines`, unit test at tests.rs:83). Non-ASCII source paths or
  identifiers fail the patterns and refuse (fail closed).

### (2) Plan from index facts, checked on real cargo output
A scratch virtual workspace (members `my-lib`, `other`, edition 2018 and 2024) with
26 indexed `#[test]` fns, every one run through `Executor::run`: lib top level, nested
modules, `cfg(test)` modules, `#[path]` module, file modules and their children, bin
`src/main.rs`, `src/bin/tool-x.rs`, `src/bin/dirbin/main.rs` plus a child module,
`[[bin]] path=` rename, `tests/it.rs` with `mod common` (tests/common/mod.rs), `tests/multi/main.rs`
plus a child, `[[test]] path=` rename (`renamed`), `tests/hy-phen.rs`, a package whose `[lib] name`
and `path` differ (`custom_name`), a bin sharing the lib's name. All 24 runnable ones passed
with exactly the pinned target and name; the example-target test is refused (`TargetKind`);
`r#match` is refused (`NoTestListed`, safe). Macro-generated tests and fns inside fn
bodies are not in the index, so they do not resolve (safe). Doc probe in a workspace member
with hyphenated package and renamed lib: module, struct, const, enum, trait default method
pass; impl methods whose impl sits in another module than the type, root-level impls of a
type in a submodule, trait-impl methods, and `W<(A,B)>` refuse (`NoDocTest` / pattern),
never pass wrongly. Real rustdoc names carry the workspace-relative file part
(`crates/my-doc/src/lib.rs - ...`); only the item part is compared, which is correct.

### (3) Hostile bindings
`tests::hostile_*` (unit) and `hostile_bindings_are_unresolved_and_never_reach_cargo` (real
runner pointing at a cargo that does not exist, so any spawn is an error) cover
`-Zunstable-options`, `--include-ignored`, `--exact`, `;`, space, hyphen-led item path,
for tests and examples. Manifest-derived names (package, target, example) are
re-validated by `Package/TargetName/ExampleName::new` in the plan; a unit test feeds
hostile facts directly. Target names go in as `--bin=<n>`, `--test=<n>`, `--example=<n>`,
package as `--package=<p>`; none can start with `-`.

### (4) Argv exactness
`tests/bound.rs` asserts the exact vectors for test list/run (`test --frozen --package=fx-tests
--lib -- --list --exact -- tests::passes`, run pass without `--list`), doc list/run
(`--doc -- --list`, `--doc -- <item>`), and example (`run --frozen --package=.. --example=..`).
All match the Contract table.

### (5) Example limit
`Limits::example()` is 300 s around the single `cargo run`, so the build counts. The
Contract text ("`cargo run` ... a timeout of 300 seconds") is met literally and the module
docs and CHANGELOG say so. It fails closed, and an example's build is normally warm after
the test pass (cargo test builds examples). It is narrower than D3's sentence "1800 s for a
`cargo run` that may compile": ADVISORY A1.

### (6) CI workflow
`actionlint` rejects the changed file: `.github/workflows/ci.yaml:145:61: could not parse as
YAML: mapping values are not allowed in this context`. The plain scalar
`run: cargo test -p raf-workplan --locked --lib bound::` ends in a colon, which YAML reads as a
mapping indicator (the file at HEAD passes actionlint). Same defect on the Windows step.
That is REQUIRED R1. Step names and placement are otherwise sensible. Windows: see R2.

### (7) Mutation re-run (scratch copy, unit plus real-cargo tests)
| Mutant | Result |
| --- | --- |
| M1 proof loosened to exit status | killed (6 unit, 2 real) |
| M2 `--frozen` dropped | killed (21 unit, 1 real) |
| M3 / M3b target pin dropped (`--test`, `--bin` replaced by `--lib`) | killed (5+3, 4+1) |
| M4 ok line loosened to `ends_with("ok")` | killed (5 unit, 2 real) |
| M5 ignored counted as passed | killed (1 unit, 1 real) |
| M6 (own) doc exit status ignored | killed (1 unit, 2 real) |
| M7 (own) take first of several result lines | killed (unit only) |
| M8 (own) accept a single listed entry of any name | killed (unit only) |
| M9 (own) select doc entries by prefix | killed (2 unit) |
| M10 (own) list pass exit status ignored | killed (unit only) |

### (8) Timing and flakiness
`tests/bound.rs`: 19 tests, 6 to 7 s per run (7 runs: 7.0, 6.6, 6.7, 6.7, 13.0, 6.5, 6.5 s),
0 failures. The 13 s run overlapped a full-suite build. Budget assertions are 300 s per
fixture. Unit `bound::` tests 0.1 s.

### (9) Hygiene
SPDX on all new files; `make reuse` 338/338. A scan of added lines for tracker numbers,
issue or review wording, process wording, deferrals and agent credit finds nothing.
CHANGELOG describes the built behaviour.

### (10) Gates (all green)
`cargo test -p raf-workplan` (every binary) and `--release`; `make clippy` (`-D warnings`);
`make doc` (`-D warnings`); `make fmt`; `repo_gates` 57/57; `docs_contract` 26/26;
`make reuse`; `make deny` (advisories, bans, licences, sources ok).

## Findings

### REQUIRED

* R1. `.github/workflows/ci.yaml` is invalid YAML on both added `--lib bound::` steps
  (lines 145 and its Windows twin): a plain scalar may not end in a colon. The whole
  workflow fails to load, so every job in it stops running. Quote the command (for example
  `run: "cargo test -p raf-workplan --locked --lib bound::"`) or drop the trailing `::`
  (`--lib bound`), and confirm with `actionlint`.
* R2. Doc-test selection will find nothing on Windows. `parse::doc_entry` requires the file
  part to match `Pattern::SourcePath` (`[A-Za-z0-9_][A-Za-z0-9_./-]*`, no backslash), but
  rustdoc prints the file part with the platform separator (`src\lib.rs - good (line 5)` on
  Windows), so each entry is skipped and every doc-test binding ends `NoDocTest`. The
  fixtures `documentation_tests_follow_the_rules_in_edition_*` run in the added Windows job
  and are expected to fail there; the claim in the CHANGELOG and the CI comment that the
  fixtures run "on every platform" is then false. (I could not run Windows; the finding
  rests on rustdoc naming the test from the displayed path, and on the observed real
  output carrying the workspace-relative path, `crates/my-doc/src/lib.rs - ...`.) Fix by
  checking the file part with backslashes read as `/` (the name kept as printed for the
  `test <name> ... ok` match), and add a unit test with a `src\lib.rs - foo (line 4): test`
  list and its run lines. If this is deliberately not done, the Windows job must not run
  `--test bound`, and the claim must be narrowed; the Contract's fourth pattern needs the
  matching note in Task 11.

### ADVISORY

* A1. Example limit (D3/E12): 300 s includes `cargo run`'s build. This matches the
  Contract sentence and fails closed, but D3 distinguishes 1800 s for runs that may
  compile from 300 s for an example's own run. A cold example build in a large workspace
  can fail a correct binding. Task 11's Contract edit for time bounds should state that
  the 300 s covers the build, or `complete` should be allowed to build examples first
  (not in the argv table, so a Contract change). The CHANGELOG already says it.
* A2. Windows has no process-group kill (`terminate` is `Child::kill` only). The
  `forever` test fixture would leave the sleeping test binary running up to 120 s
  after the kill. It does not block the verdict (readers are detached), but leaves an
  orphan and may keep a handle in the shared target directory. Say so in the `exec` docs or
  use a job object.
* A3. A test that writes a result-shaped line to raw stdout and exits 0 is counted as one
  passed test. Workspace code is trusted input per SECURITY.md, so no change is needed;
  mention it in the module docs next to the proof.
* A4. Fail-closed gaps worth listing in the module docs: raw identifiers (`r#match`),
  non-ASCII names, multi-parameter generic self types (`W<A, B>` prints as `W<(A,B)>`
  for tuple forms), impl blocks outside the module of their self type, and trait-impl
  methods are refused rather than run. Only the last two documented cases are noted in
  `doc_item_path`'s comment.
* A5. `tests/bound.rs` fixtures share one target directory between fixtures that reuse
  package names (`fx-tests`) at different paths; cargo hashes by path so builds do not
  collide, but cold runs recompile each fixture. Fine today (6-7 s warm).

VERDICT: BLOCKED

## Re-review

Re-checked at HEAD 9f1ab69 in a scratch worktree (removed afterwards).

| Finding | Fix location | Evidence | Status |
| --- | --- | --- | --- |
| R1 CI YAML invalid | `.github/workflows/ci.yaml:145` and `:176` (`run: "cargo test -p raf-workplan --locked --lib bound::"`, quoted) | `actionlint .github/workflows/ci.yaml` exits 0 | resolved |
| R2 Windows separator in doc-test parse | `crates/raf-workplan/src/bound/parse.rs:127-129` (`file.replace(char 92, "/")` before the SourcePath match, printed name kept); commit e42ff32 area | unit test `bound::tests::a_doc_list_entry_with_the_platform_separator_is_read_and_keeps_its_printed_name` passes; mutant (replace removed) fails exactly that test (67 pass, 1 fail) | resolved |

VERDICT: PASS
