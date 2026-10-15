# T1 review: exec (runner, typed invocation, patterns)

## Scope
Staged diff of Task 1 (crates/raf-workplan/src/exec.rs, src/exec/{fake,invocation,pattern,system,tests}.rs,
tests/exec.rs, lib.rs, CHANGELOG.md), read against plan D3, D5, Task 1, AGENTS.md, SECURITY.md (Subprocesses)
and the Invocation, pattern and time-bound text of workplan-complete-gate, workplan-reconcile, workplan-cli.
Experiments ran in a throwaway worktree at HEAD plus the staged diff (removed afterwards); nothing was staged,
reset or committed in the task worktree.

## Evidence
Gates (task worktree, CARGO_BUILD_JOBS=6): raf-workplan tests pass, raf-workplan release tests pass,
workspace clippy --all-targets -D warnings clean, doc -Dwarnings clean, fmt --check clean,
repo_gates (26) and docs_contract (57) pass, make reuse compliant (322/322).

1. Patterns: probe over ~110 values. Name rejects empty, leading digit, `::a`, `a::`, `a:::b`, `a:b`, leading `-`,
   space, newline, unicode; ItemPath accepts `Wrapper<T>::get`, `a<>`, `a<'a,&T>`, `a<std::fmt>::x` and rejects
   `a<T`, `a<T<U>>`, `a< T>`, `a<T>b`, `a<T>::`, `<T>::a`, `a<-T>`, `a<é>`; TargetName accepts `a-b`, `9`, rejects
   leading `-`, `.`, `/`, unicode; Commit accepts only 40/64 lowercase hex; RefName rejects `..`, `.lock`, `@{`,
   `~ ^ : space`, empty segments. All match the written regexes. SourcePath accepts `tests/parse_errors.rs`,
   `src/a-b/c.rs`, `src/.hidden/x.rs`, `a..b/c`, and refuses empty, `.` and `..` segments, `./a`, `a/`, unicode.
2. Argv: tests/exec.rs walks 13 rows and the stub test checks the process receives exactly that argv; every row
   equals the table (`--frozen`, `--package=`, `--lib|--bin=|--test=`, `--` before filter, `--end-of-options`,
   `<ref>^{commit}`, `<rev>:<path>` in one argument, metadata equals the cli Contract line). Outside the module:
   `Filter("x".into())`, `Package(String)`, `Commit(..)`, `From<String>`, `Default`, struct literals of
   Invocation and Output all fail to compile. `Arg::Literal(Box::leak(String::from("x;y").into_boxed_str()))`
   compiles (finding R2).
3. Real runner: no shell, stdin null, current_dir set, LC_ALL=C only when `c_locale`, no other env change (test
   plus reading). A child that traps TERM/INT/HUP is killed at the limit (SIGKILL). Orphan holding the pipes after
   the parent exits ends in Timeout, not a hang. Bound: exactly the bound passes, one byte over errors, bound 0
   with 1 byte errors, with 0 bytes passes. With PATH lacking `kill`, the timeout fires but the grandchild stays
   alive (finding R1).
   Mutation re-run, each killed: M1 identifier start accepts `-` (killed by exec::tests rejection walk); M2
   `--frozen` dropped from test_list (3 failures: row walk, filter-follows-separator, stub argv); M3 timeout
   ignored (3 failures: outlives-limit, descendant, family kill); M4 overflow threshold off by one (3 failures);
   M5 group kill disabled (2 failures).
4. Fake: applies the same `len > max_output` rule, stdout then stderr, as the real runner; records program and
   argv; unscripted is an error. It never applies the time limit (finding A2).
5. SPDX present on every new file (make reuse). No tracker numbers, issue/PR or process wording, no agent credit
   in the diff. CHANGELOG entry matches behaviour.

## Findings

### REQUIRED
R1. Group kill depends on an external `kill` found on the caller's PATH and ignores its failure
(`system.rs` `terminate`: `let _ = Command::new("kill")...status()`). Reproduced: run with `PATH=/nonexistent`,
a stub that backgrounds `sleep 60` and waits, limit 500 ms: the run returns Timeout but the grandchild keeps
running. Minimal images (for example debian slim) ship no `kill` binary, so the documented "kill reaches the
group" silently degrades to killing the direct child, which is a fail-soft path in a repo that says fail hard. Fix:
signal the group without a PATH lookup (an existing `rustix` with the `process` feature is already in the
lockfile via raf-source, or an absolute-path fallback chain), and report failure to signal as `ExecError::Io`
instead of discarding it; add a test with PATH emptied via the runner's search_path or a child-PATH-less probe.

R2. D5 says no Invocation can be built from raw strings outside the validators, but `Invocation::new` and
`Arg::Literal(&'static str)` are public, so any caller can build any argv (`Arg::Literal("--config=...")`) and a
leaked runtime string passes as a literal. The per-row constructors make `new` unnecessary outside the crate.
Fix: make `Invocation::new` `pub(crate)` and make `Literal` hold a newtype with a private field and a
`pub(crate)` constructor (or a closed enum of fixed words), and drop the doc sentence that offers `new` for
other commands. Update `a_literal_is_the_only_free_text_and_it_is_static` accordingly.

### ADVISORY
A1. SourcePath is narrower than its written pattern (refuses empty, `.`, `..` segments; the regex alone would
accept `a/../b`, `a//b`, `a/./b`). No legitimate relative list path is refused (checked `tests/parse_errors.rs`,
`src/a-b/c.rs`, dotted names) and the narrowing is documented in the module and CHANGELOG, but the Contract text
says "matches" the regex. Add the clause to the Contract edit set (Task 11) or keep it and say so in the
complete-gate pattern bullet. `Pattern::expression()` returns the un-narrowed regex, which is consistent with the
Contract but not with `matches`.

A2. FakeRunner enforces the output bound but not the time limit: a scripted `Output` whose `elapsed` exceeds
`limits.timeout()` is returned as Ok, while the real runner would have killed it. Its doc says "the bounds of the
Limits are applied". Return `ExecError::Timeout` when `elapsed > timeout`, or narrow the doc to the output bound.

A3. Limits::example is 300 s for the whole `cargo run --example`, which also compiles. D3 states 1800 s for a run
that may compile and 300 s for the example's own run; there is no build-only argv row, so the caller (Task 5/6)
cannot separate them. Resolve at the verify task: either a build step under `Limits::build()` first, or document
that 300 s covers a warm build.

A4. A descendant that calls setsid or moves to another group escapes the group kill; the run still ends in
Timeout, but the process survives. Already scoped in the module doc ("elsewhere only the process itself") for
non-Unix; add the setsid case to the doc.

A5. After the direct child is reaped by `try_wait` and its group is empty, `kill -PGID` could in principle name a
recycled id. Negligible; noting for completeness.


## Re-review after fixes
- R1 fixed and verified: `terminate` now signals the group with `rustix::process::kill_process_group` (SIGKILL, ESRCH
  tolerated) and reports other failures. Re-run of the reproduction with `PATH=/nonexistent` (stub that backgrounds
  `sleep 60`, 500 ms limit): Timeout returned and the grandchild is not alive afterwards.
- R2 fixed and verified: from an external test crate `Invocation::new`, `Literal::new`, `Literal("x")`, a leaked
  `&'static str` passed as `Arg::Literal`, and `Filter("x".to_string())` all fail to compile (private or type
  mismatch). Compile-fail doctests added.
- A2 fixed: the fake returns Timeout when `elapsed` exceeds the limit. A4 documented.
- Dependency: lockfile diff adds only the `rustix` edge to raf-workplan (unix-only target dependency, features
  `process` and `std`, same version requirement as raf-source); `cargo deny` (advisories, bans, licenses, sources)
  ok.
- Gates re-run: raf-workplan tests and release tests pass, clippy -D warnings, doc -Dwarnings, fmt, repo_gates (26),
  docs_contract (57), make reuse all clean.
- New ADVISORY A6: `ExecError::Timeout` formats the limit with `{:.0}`, so a 0.5 s limit prints "0 s limit";
  harmless at the real 300/1800 s limits, but `{:.1}` would match the elapsed figure.
- A1, A3, A5 remain advisory and open for the owning later tasks.

VERDICT: PASS
