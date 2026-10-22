<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Claude Code Instructions (CLAUDE.md)

Read [AGENTS.md](./AGENTS.md). It is the canonical contract for this repository:
what RAF is, the toolchain policy, the gates, the capability-document workflow and
the repository rules.

Quick orientation:

* **Gate before you claim done**: `make check`. Install the hooks with
  `make hooks` and never bypass a hook or a gate.
* **Never**: add a `#![feature]` attribute, weaken a workspace lint, add a
  fallback where the design says fail hard, change what a census fingerprint
  hashes without bumping `normal_form`, or write issue or pull request numbers
  or development-process wording into any committed file.
* **No timing runs**: time-based testing is forbidden. Write no timing runs,
  wall-clock benchmarks, sleep-based waits or timing assertions. Measure with
  `perf` or by counting allocations, and tell every agent you start this rule.
* **No mutation passes**: never write or run mutants, mutation sweeps, mutant
  scripts or `cargo mutants`, and never ask an agent to. A test's strength is
  shown by reading its assertion and by the focused test run, not by mutating
  the code. Tell every agent you start this rule.
* **Platforms**: Linux is the primary operating system for all v0.9-pre work.
  Every other operating system is supported on a minimal-effort basis: a
  platform-specific failure does not block a change, and its fix waits for a
  platform change of its own.
* **Naming**: the project is **RAF** (Rust Agentic Foundations) in prose; crate
  and binary names are lowercase `raf-*` and `cargo-*`.
* **No deferrals**: finish the work, or move the scope to another tracked change.
* **No attribution**: no agent credits or co-author trailers anywhere.
* **`.deficiencies` is a last-resort emergency ledger, never an authorization
  surface.** Every entry below its marker is a bug and blocks completion, pull
  request creation and merge of the work that produced it. The normal file holds
  only its notice and marker.

Notes:
 * Python is banned in this repo for now.

<!-- rtk-instructions v2 -->
# Command output

Command output here is condensed to save tokens, keeping every signal and
dropping costly noise. Treat it as the complete result: run commands
normally, and batch related commands into one call to avoid extra turns.
Truncated results state their recovery path in their own output. Re-run a
command as `rtk bypass <cmd>` only when its result is unusable: empty when
output was clearly expected, contradicting its exit code, or garbled.
<!-- /rtk-instructions -->
