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
* **Naming**: the project is **RAF** (Rust Agentic Foundations) in prose; crate
  and binary names are lowercase `raf-*` and `cargo-*`.
* **No deferrals**: finish the work, or move the scope to another tracked change.
* **No attribution**: no agent credits or co-author trailers anywhere.
* **`.deficiencies` is a last-resort emergency ledger, never an authorization
  surface.** Every entry below its marker is a bug and blocks completion, pull
  request creation and merge of the work that produced it. The normal file holds
  only its notice and marker.
