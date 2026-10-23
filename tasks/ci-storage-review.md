<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native CI storage repair review

Read-only review of the actual workflow, captured Linux annotations, Cargo profile contracts and the production SystemRunner environment handling. No builds, tests or product edits performed; the existing local gate remains owner-run.

The captured annotation establishes a genuine runner disk-exhaustion failure: System.IO.IOException reports “No space left on device” while writing the runner's own diagnostic log. The unavailable job log prevents attributing exact byte consumption to one Cargo step; the annotation is not evidence of an assertion failure or a completed test run.

No concrete gate-semantic defect found in the repair. The Linux native test, macOS and Windows jobs set only dev/test debug metadata to line-tables-only and disable incremental compilation. Cargo documents debug metadata separately from debug assertions and overflow checks; line-tables-only retains filename/line backtrace information while omitting variable/type debugging detail. Its support predates the repository's MSRV. Assertion, overflow, optimization, panic and lint settings are not changed by these variables. See the [Cargo profile settings](https://doc.rust-lang.org/cargo/reference/profiles.html#debug).

The job-level variables also reach `cargo install --path crates/cargo-workplan --debug --locked`, not merely cargo test. That is desirable here: the installed Workplan binary and its dev dependencies receive the same reduced debug metadata, while the command remains a dev-profile executable. Its capability-bound child Cargo invocations inherit the environment: SystemRunner uses normal Command inheritance and does not clear those variables. The workflow still installs the real local command with the existing locked dependency and target directory settings. See [cargo install profile selection](https://doc.rust-lang.org/cargo/commands/cargo-install.html#compilation-options).

Test commands, exact-listing/runner tests, three-host verification steps and locked flags remain present. Release, fuzz, Clippy, documentation, MSRV and cross-check jobs do not inherit these job-local overrides. CARGO_INCREMENTAL disables cache persistence rather than assertions or source paths. No source-level test exclusions or alternate successful runner were added.

This is a coherent storage reduction, not proof that available disk is now sufficient. Reduced debug artifacts and absent incremental caches should lower retained build storage, including nested capability builds. The actual hosted rerun must establish completion under the runner's capacity. No redundant suite or additional numeric requirement is requested.
