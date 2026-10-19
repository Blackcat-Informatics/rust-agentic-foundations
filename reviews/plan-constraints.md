# Plan constraints review

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/27-raf-source-the-registry-scan-example/.stage/raf-source-the-registry-scan-example/plan.md`

## Governing documents read

- `CLAUDE.md` (worktree root): read.
- `AGENTS.md` (worktree root): read; it is the canonical contract (CLAUDE.md:6).
- `CONTRIBUTING.md` (worktree root): read.
- `.deficiencies`: read. It holds only the notice and the marker (line 20); there are no entries.
- `Makefile`: read for the `check` and `cross-check` targets the gate rules name.
- `tests/repo_gates.rs` (`clock_rule`, the carve-out assertion) and `crates/raf-workplan/src/refs.rs` (process-wording patterns): read to confirm how the rules are enforced.
- `.baseline`, `.goals`, `CONSTITUTION.md`, `PRECOMMIT_GUIDANCE.md`, ADR directories (`docs/adr/`, `docs/decisions/`): **absent**. No nested `CLAUDE.md`/`AGENTS.md` exists under `crates/raf-source/` or `tests/`.
- User decisions this session (from the caller): "no timing runs", "Linux primary". Both are also written in CLAUDE.md:18-24.

## Findings

### BLOCKING 1: the Task 3 gate run leaves out `cross-check`, and the new Unix-only API would fail that gate

Governing text:
- CLAUDE.md:12-13: "**Gate before you claim done**: `make check`. ... never bypass a hook or a gate."
- AGENTS.md:133: "make check   # fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, cross-check, licenses, issue-refs"
- AGENTS.md:140: "**Run `make check` before claiming work is done.**"
- AGENTS.md:173-174: "**Never bypass a hook or a gate.** ... do not weaken a gate or a lint to get green. Fix the cause."
- Makefile:106: `RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target "$$target"` for the Apple, FreeBSD and Windows targets. `--all-targets` covers the raf-source examples, and once `test = true` is set it also covers their test builds.

Offending plan lines:
- Task 3 (plan.md:73): "`make fmt clippy test test-release doc deny reuse msrv fuzz licenses issue-refs` locally"
- Task 1 (plan.md:50, 55): "`ExitStatusExt::signal`" and "(`ExitStatusExt::from_raw`)". The plan does not gate these with `cfg`.

The list leaves out `cross-check`, which is one of the gates in `make check`. `std::os::unix::process::ExitStatusExt` does not exist on `x86_64-pc-windows-gnu`. Used without a `cfg`, `cargo check -p raf-source --all-targets --target x86_64-pc-windows-gnu` fails to compile the example and its tests. Today the example compiles on every target, because it uses no Unix-only API: `sh` and `/proc/meminfo` are only reached at run time. The Platforms rule (CLAUDE.md:21-24) does not cover a gate failure that this change introduces into a gate that runs on the Linux host. Leaving out the gate that would catch it amounts to bypassing a gate.

REWRITE TO (Task 3): "`make check` locally (fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, cross-check, licenses, issue-refs), green before the PR."

REWRITE TO (Task 1, add): "The signal reading and its unit tests are `#[cfg(unix)]` (`std::os::unix::process::ExitStatusExt`). On other targets `main` fails hard with `registry_scan needs a Unix system (sh, ulimit)`, so `make cross-check` compiles the example and its tests for every target, warnings denied. Acceptance (6): `make cross-check` passes."

### ADVISORY 1: the new `--cpu-limit` refusal has no run of its neighbouring valid case

Governing text: AGENTS.md:237-238: "**A refusal is a claim too.** When you add or tighten a refusal, run the case you believe is invalid and a neighbouring valid case, and confirm the second succeeds."

Offending plan line (plan.md:45): "Add `--cpu-limit SECONDS` (a whole number, at least 1; ...)". No acceptance item runs `--cpu-limit 0` or `--cpu-limit 1`.

REWRITE TO (Task 1 acceptance, add): "(7) `--cpu-limit 0` is refused with a message naming the flag, and `--cpu-limit 1` over the one-crate scratch registry is accepted and prints the `OK` line. Only the exit and the line are read."

Task 2 already meets this rule for the gate change: the planted reading is refused, and the clean example passes `repo_gates`.

### ADVISORY 2: the plan does not repeat the "tell every agent" half of the no-timing rule

Governing text: CLAUDE.md:18-20: "**No timing runs**: ... and tell every agent you start this rule."

Offending plan line (plan.md:10): "CLAUDE.md: no timing runs, no sleep-based waits; ..." The plan records the rule but does not require that any implementation, validation or review agent it starts is told the rule.

REWRITE TO (Standing goals, append): "Every agent started for Tasks 1-3 is told: no timing runs, wall-clock benchmarks, sleep-based waits or timing assertions; measure with `perf` or by counting allocations."

## Checked and compliant (no finding)

- **No timing runs / no sleep-based waits** (CLAUDE.md:18-20, AGENTS.md:151-172). The example drops `Instant`, `Duration`, `POLL`, `thread::sleep` and the clippy allow. The child limit is a kernel rlimit (`ulimit -t`), and the example's source names no clock. The `classify` unit tests use crafted statuses and read no time. Acceptance (4) is a one-off manual shell run that reads only the terminating signal, records it in `.stage/` (git-ignored, `.gitignore:5`), and adds no timing assertion or committed test. The CPU-time bound is a design choice that the requirements and design reviewers own; it breaks no written rule.
- **Linux primary** (CLAUDE.md:21-24). The example already depends on `sh`, `ulimit -v` and `/proc/meminfo`, and adding `ulimit -t` is consistent with that. The only platform concern is compile-time and is covered by BLOCKING 1.
- **Do not weaken a gate or a lint** (CLAUDE.md:14, AGENTS.md:173-174). Task 2 tightens the gate: it removes the carve-out at `tests/repo_gates.rs:354-355` and the stale-carve-out assertion at 641-646, which becomes moot. It also removes the reasoned `#[allow]`, and `clippy.toml` is left unchanged.
- **Every rule has a negative fixture** (AGENTS.md:235-236, CONTRIBUTING.md:45-46). Task 2 plants a reading in `crates/raf-source/examples/` that is reported with file and line, and adds a scratch-clone mutant for both the gate and clippy.
- **Docs reconciled with the gate** (AGENTS.md:162-163, CONTRIBUTING.md:95-98, `tests/repo_gates.rs:19-26, 310-312`). Task 2 updates AGENTS.md, CONTRIBUTING.md and the gate's module doc, which are the three places that state the carve-out.
- **No issue or PR numbers or process wording in committed files** (CLAUDE.md:16-17, AGENTS.md:280-285). Of the plan's text, "Closes #27" appears only in the PR body, and the PR body is not a committed file. Once the example is under the raf-source rule, its module doc must still avoid the bare words `Duration`, `elapsed`, `sleep` and `Instant`. The plan's `repo_gates` acceptance enforces that.
- **No deferrals** (CLAUDE.md:27, AGENTS.md:286-288). The "tracked separately" glob-settling cost is not scope taken from this issue. It is already stated as a constraint in `docs/capabilities/source-index.md:787, 1689`.
- **SPDX / licensing** (AGENTS.md:210-221). No new file is added, and the edited files keep their existing headers.
- **No new dependency.** `ExitStatusExt` is in `std`, and the `[[example]]` table in `crates/raf-source/Cargo.toml` adds no crate.
- **No attribution** (CLAUDE.md:28, AGENTS.md:292-293). Nothing in the plan adds credits or trailers.
- **`.deficiencies`** (AGENTS.md:11-24). It is empty below the marker, and the plan writes nothing to it.

VERDICT: NEEDS-REVISION
