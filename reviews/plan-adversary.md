# Plan completion adversary review

- Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/27-raf-source-the-registry-scan-example/.stage/raf-source-the-registry-scan-example/plan.md`
- Pass: 1 (the caller gave no pass number and no prior findings)
- Inputs read: plan.md, issue.md (body and 0 comments), issue-analysis.md (R1-R6 and NEEDS-JUDGMENT 1-5), prior-art-assessment.md, references.md, brief.json, requirements.json, `crates/raf-source/examples/registry_scan.rs`, `crates/raf-source/Cargo.toml`, `tests/repo_gates.rs` (clock gate and fixtures), `clippy.toml`, `Makefile` (`check`, `cross-check`, `test`), `.github/workflows/ci.yaml` (`cross-check`, `test-windows`, `test-macos`), `AGENTS.md` and `CONTRIBUTING.md` (clock gate paragraphs), `CHANGELOG.md` and `crates/raf-source/src/glob_scope.rs` (glob settling cost), `docs/capabilities/source-index.md` (registry scan figures).
- User scope decisions quoted by the caller: none. No issue comment exists (coverage 0/0).

The plan is concrete and most of it can be checked. Task 2 (the gate) and the R2/R4 rows hold up against the code: removing the `dir == "examples"` branch in `clock_rule`, the stale carve-out assertion (`repo_gates.rs` 641-647), the `registry_scan.rs` entry in the exempt fixture (`repo_gates.rs` 1206), and the doc paragraphs in `AGENTS.md` 162-163, `CONTRIBUTING.md` 95-98 and `repo_gates.rs` 25-26 and 310-312 are all named or covered. The findings below are about R1's deliverable, how its checks are observed, one gate that is missing from the gate list, and contract rows.

---

## A1. ESCALATE-TO-HUMAN: R1 is met by a route the issue does not offer, and the contract rewrites R1 to fit it

Ground: PRE-EMPTIVE DESCOPE / UNMAPPED REQUIREMENT (R1 as written has no task that delivers it).

The issue gives a closed choice (issue.md L11): "Replace the per-crate timeout and the sleep poll with a bound on counted work (the index already reports `Usage` and refuses at its budgets), or remove the example if its purpose is served by the differential test against rustc."

The plan takes neither path and records its own requirement in their place:

> `| R1 | Replace the per-crate timeout and the sleep poll with a bound that is not a clock | 1 | the acceptance checks (1)-(5) of Task 1 |`

> `Replace the wall-clock timeout with a CPU-time limit enforced by the kernel ... --timeout SECONDS becomes --cpu-limit SECONDS (CPU seconds, same default of 900)`

> `1, 4 (counted work or remove the example): neither. ... the kernel's CPU-time limit replaces the timer and the example stays.`

What the evidence shows:
- The plan's reason for dropping counted work is backed by the code. The index's budgets bound memory and tokens, not steps. `glob_scope.rs` 46-51 and `CHANGELOG.md` 59-60 say glob settling "still costs quadratic or worse in the number of globs on three shapes", and `LoadOptions` (`options.rs` 23-48) has no work field. So the issue's premise, that the index already refuses at its budgets, does not bound run time. The plan is right to raise this.
- Its reason for keeping the example is also backed. `docs/capabilities/source-index.md` 648-696 quotes registry scan figures (1177 crates, `linux-raw-sys` 16.9 MB, `funty` 987, a 1505567-token file) that the differential test does not produce.
- A CPU-seconds `RLIMIT_CPU` is still a time measure. It does not count work. Whether a crate gets `OK` or `ERR cpu-limit` depends on the host's speed, and that dependence is exactly what "counted work" removes. The plan itself rejects "a blocking wait on a pidfd with a timer (still a clock)". A CPU-time limit is a timer on the process CPU-time clock that the example does not read itself. Taking that third route changes R1, and only the user can make that call.
- The plan says the missing step budget is "tracked separately as superlinear glob settling", but it names no tracked item, and nothing in the worktree (`work/`, docs, `CHANGELOG.md`) records one. The repository's no-deferrals rule needs a real tracked change before scope can move to it.

Choices for the user, as concrete alternatives:
- **A. Accept the kernel CPU-time limit** as R1's bound. The user widens R1 from "a bound on counted work" to "a bound the example does not read: the kernel's CPU-time limit (`ulimit -t`)". The plan stands, with A2-A7 fixed.
- **B. Counted work as the issue asks.** Add a work budget to the index: a `Budget` variant charged once per glob-settling step, carried by a `LoadOptions` field and reported in `Usage`. The example passes it from a `--max-steps N` flag and reports the `Budget` refusal as `ERR Budget`. There is no child time limit. This adds `crates/raf-source/src` work to this change, or it waits on a tracked item that the plan must name.
- **C. Counted work from the existing budgets only.** Drop the timeout and replace it with nothing except the index's current budgets, optionally passed in as flags. The plan then states that a crate with one of the three superlinear glob shapes is not bounded in time and that the operator stops the scan.
- **D. Remove the example.** The figures in `source-index.md` 648-696 then lose their source and must be restated or removed.

REWRITE TO (contract row, choice A): `| R1 | "Replace the per-crate timeout and the sleep poll with a bound on counted work (...), or remove the example (...)", delivered as the kernel's CPU-time limit per the user's decision: "<quoted user decision>" | 1 | Task 1 checks (1)-(6) |`

REWRITE TO (Decision, choice A): `Per the user's decision "<quote>", the counted-work bound is replaced by the kernel's CPU-time limit; the index has no step budget, and the step budget for glob settling is tracked as <tracked item id and title>.`

REWRITE TO (choice B, C or D): replace the R1 row's Requirement cell with the issue's verbatim text, and rewrite Task 1 to the chosen route with its own executable checks on `registry_scan`.

---

## A2. PLANNED-BY-REFUSAL (test-only): no check observes the bound working through the shipped example

Ground: the evidence that R1 is delivered is a unit test over crafted statuses plus a hand-written shell loop. Neither goes through `registry_scan`.

> `(2) unit tests of classify over crafted statuses (ExitStatusExt::from_raw) ... (3) the example run over a one-crate scratch registry copy of a small real crate prints the OK line and exits 0; (4) the kernel does the stopping: run the child wrapper by hand (sh -c 'ulimit -t 1; exec sh -c "while :; do :; done"') ... (5) mutant: classify treating signal 24 as abort fails (2).`

The capability R1 asks for is a crate that runs too long being stopped and reported on its own line while the scan continues. Checks (1)-(5) still all pass if `run_child` leaves `ulimit -t <n>` out of its wrapper:
- (1) is a grep;
- (2) and (5) exercise `classify` alone;
- (3) only runs a crate that finishes;
- (4) runs a shell the plan wrote, not the wrapper `run_child` builds.

The plan's own visible line, `ERR cpu-limit`, never has to appear in a real run.

REWRITE TO (add to Task 1 acceptance):
`(6) the shipped example stops a crate that runs past its limit: a scratch registry holds the small real crate from (3), named in validation.md, and a generated crate spin-0.1.0 whose lib.rs is the hub shape glob_scope.rs calls about cubic (a hub that N sibling modules glob and that globs them back, partly privately), with N large enough that it does not settle within one CPU second and stays under the glob-bytes budget; cargo run --release -p raf-source --example registry_scan -- <scratch> --jobs 1 --cpu-limit 1 prints "spin@0.1.0 | ERR cpu-limit :: stopped after 1 CPU seconds", the small crate's "| OK items=" line, "# crates=2", "# ok=1", "# err cpu-limit=1", and exits 0; the output is recorded in validation.md. No time is measured or asserted: only the printed lines are read. Without ulimit -t in run_child's wrapper, no signal 24 or 9 reaches the child and the cpu-limit line cannot appear, so (6) tells the two apart.`

REWRITE TO (sharpen (3)): `(3) cargo run --release -p raf-source --example registry_scan -- <scratch with one crate, named in validation.md> prints "<name>@<version> | OK items=", "# crates=1", "# ok=1" and exits 0 at the default --cpu-limit; the same run with --timeout 5 exits 1 with "unknown \`--timeout\`".`

---

## A3. UNFALSIFIABLE CRITERION: check (4) names one signal while the plan's own Decision allows two, and it does not run the shipped wrapper

> `(4) ... (sh -c 'ulimit -t 1; exec sh -c "while :; do :; done"') and see it end by SIGXCPU, recorded in validation.md as the evidence that ulimit -t ends a spinning child`

The Decision says the child may be ended "by SIGXCPU, or SIGKILL at the hard limit, both set by `ulimit -t`". Without `-S` or `-H`, both dash and bash `ulimit -t` set the soft and hard limits together, and the Linux `RLIMIT_CPU` check tests the hard limit before the soft one. So "see it end by SIGXCPU" may not be what happens. An implementer who sees SIGKILL would then have to reinterpret the criterion. The command is also not the wrapper text `run_child` builds (`ulimit -v <KiB>; ulimit -t <n>; exec "$0" "$@"`).

REWRITE TO: `(4) the wrapper text run_child builds ends a spinning child: sh -c 'ulimit -v 8388608; ulimit -t 1; exec "$0" "$@"' sh -c 'while :; do :; done'; echo "status=$?" prints status=152 (SIGXCPU) or status=137 (SIGKILL). validation.md records which one this kernel sends, and (2) maps that signal to cpu-limit. Only the status is read; no time is read or asserted.`

---

## A4. PRE-EMPTIVE DESCOPE of a required gate: Task 3 leaves `cross-check` out of `make check`, and the planned code uses Unix-only API in a target that gate builds for Windows

> `make fmt clippy test test-release doc deny reuse msrv fuzz licenses issue-refs locally;`

`Makefile` 118-130 defines `make check` as that list plus `cross-check`. `CLAUDE.md` says "Gate before you claim done: `make check`", `AGENTS.md` 175-179 calls a missing cross-check target "a hard failure, not a skip", and `.github/PULL_REQUEST_TEMPLATE.md` 9 lists cross-check. The gap is not abstract. The plan specifies `ExitStatusExt::signal` and `ExitStatusExt::from_raw`, which exist only under `std::os::unix::process`. `make cross-check` runs `cargo check -p raf-source --all-targets --target x86_64-pc-windows-gnu` (`Makefile` 45, 106), and `--all-targets` includes this example. The CI job `test-windows` runs `cargo test -p raf-source --locked` (`ci.yaml` 173-174), which builds the example today and, with the planned `test = true`, would build and run its unit tests too. The plan says nothing about how the example compiles on those targets.

REWRITE TO (Task 3): `make check (fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, cross-check, licenses, issue-refs) passes locally.`

REWRITE TO (Task 1, add): `The signal reading and its unit tests are #[cfg(unix)]; on other targets the example builds and its child path reports the signal-free outcome kinds. Acceptance: make cross-check passes, and cargo test -p raf-source --locked builds the example for x86_64-pc-windows-gnu (covered by cross-check --all-targets).`

---

## A5. UNMAPPED REQUIREMENT: R5 and R6 of the requirements ledger have no contract rows

issue-analysis.md lists:
- R5: "The repository rule is that time-based testing and wall-clock measurement are not allowed" (body L8)
- R6: "a repository gate keeps clock spellings out of the crate's source and tests" (body L8)

The completeness contract stops at R4. R5 governs the A1 choice directly, because the chosen bound is counted in CPU seconds, and check (4) depends on a CPU second passing.

REWRITE TO (add rows):
`| R5 | "The repository rule is that time-based testing and wall-clock measurement are not allowed" | 1 | no test or check reads or asserts a time: the classify tests use crafted statuses, (4) and (6) read only an exit status and printed lines; the example reads no clock (R3 grep) |`
`| R6 | "a repository gate keeps clock spellings out of the crate's source and tests" | 2 | the time_in_crates_ gate judges crates/raf-source/{src,tests,examples} with the raf-source word list; fixture in Task 2 (see A7) |`

---

## A6. UNFALSIFIABLE CRITERION: the R3 row points at "the grep above", and no grep is written above it

> `| R3 | Acceptance grep empty | 1 | the grep above |`

The plan never writes the command out. Task 1 (1) only says "the acceptance grep of the issue".

REWRITE TO: `| R3 | "grep -nE 'std::time|Instant|Duration|sleep|recv_timeout|SystemTime' crates/raf-source/examples is empty, or the example is gone." | 1 | grep -nE 'std::time|Instant|Duration|sleep|recv_timeout|SystemTime' crates/raf-source/examples prints nothing and exits 1 (the match is by substring, so doc words such as "sleeps" count) |`

---

## A7. UNFALSIFIABLE CRITERION: the Task 2 fixture does not say which word list judges the raf-source examples

> `fixtures: a planted reading in crates/raf-source/examples/ is reported with file and line`

The plan wants `examples` added to "the raf-source rule's directories", which means `SOURCE_RULE`, the list that also refuses `Duration` and `elapsed`. A fixture that plants only `Instant` would still pass if the examples were sent to `CRATE_RULE`, and then `Duration`, which the issue's grep refuses, would pass the gate in the example. R4's "examples/ in scope" needs to be pinned to the raf-source word list.

REWRITE TO: `fixture time_in_crates_judges_the_source_examples: clock_hits(Path::new("crates/raf-source/examples/registry_scan.rs"), "use std::time::Duration;\nlet t = s.elapsed();\n") contains "crates/raf-source/examples/registry_scan.rs:1: Duration" and "crates/raf-source/examples/registry_scan.rs:2: elapsed"; mutant in a scratch clone: routing raf-source examples to CRATE_RULE fails this fixture.`

---

## Required user decision

ESCALATE-TO-HUMAN on A1. Choose one:
- **A.** Keep the kernel CPU-time limit (`--cpu-limit`, `ulimit -t`) as R1's bound in place of counted work.
- **B.** Add a glob-settling step budget to the index and bound the example by it. This is in this change, or it waits on a tracked item the plan names.
- **C.** Drop the timeout and rely only on the index's existing budgets. A superlinear glob shape is then left without a time bound.
- **D.** Remove the example and restate the `source-index.md` registry figures.

A2-A7 stay open under any choice; under A they apply as written.

VERDICT: ESCALATE-TO-HUMAN
