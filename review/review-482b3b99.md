<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Delivery review of 482b3b99

Written 2026-11-16 16:29 CST. Reviewer: independent; worked from the issue text, the diff and the
library source, not from the author's summary.

## Scope

- Commit `482b3b99` on `paudley/231-census-let-the-command-line-require`, diff
  `git diff origin/main...HEAD` (merge base `5791d3ab`; `origin/main` is `c485447d`, seven
  `raf-source` commits ahead of the merge base, none touching `cargo-census` or the census CLI
  capability).
- Requirement: `.stage/census-let-the-command-line-require/issue.md` (verbatim issue; no
  comments, no recorded user rulings beyond the issue).
- Files: `crates/cargo-census/src/cli.rs`, `crates/cargo-census/src/command.rs`,
  `crates/cargo-census/tests/delivery.rs`, `docs/capabilities/census-cli.md`, 14 renewed
  assessments under `work/assessments/` (census-cli and census-migration sections).

## Evidence

Commands, run from the worktree, one at a time under `flock /tmp/raf-build.lock`:

| Command | Exit | Scope | Log |
|---|---|---|---|
| `cargo test -p cargo-census --locked --test delivery` | 0 | 21 passed, 0 filtered; includes `module_paths_in_package_are_required_by_the_flag_beside_a_clean_run_without_it` | `reviews/review-test.log` |
| `cargo clippy -p cargo-census -p raf-census --all-targets --locked -- -D warnings` | 0 | both crates, all targets, warnings denied | `reviews/review-clippy.log` |
| `prose-check` on the commit message | 0 | message text | `reviews/prose-msg.log` |
| `prose-check docs/capabilities/census-cli.md` | 0 | whole capability document | `reviews/prose-doc.log` |

Code reading, by checklist item:

- (a) Exposure. `PolicyArgs` (`crates/cargo-census/src/cli.rs:94-111`) is flattened only into
  `Command::Check` (`cli.rs:79`) and `Command::Debt` (`cli.rs:89-90`). The argument is not
  `global` (the only `global = true` arguments are `--workspace` and the reference/registry
  options at `cli.rs:20` and `cli.rs:239-304`), so clap rejects it as an unexpected argument on
  every other subcommand and before the subcommand. `Debt` keeps `write` as
  `required = true`, so the flag cannot make a bare `debt` evaluate.
- (b) Wiring. `Command::Check(policy)` evaluates `policy.apply(gate)`
  (`crates/cargo-census/src/command.rs:64-65`) through `evaluate`, which reaches both
  `Gate::check` and `Gate::check_registry` (`command.rs:154-162`). `Command::Debt` rebinds
  `gate = policy.apply(gate)` before either `write_debt_registry` or `write_debt`
  (`command.rs:102-107`). In the library, both the plain path
  (`crates/raf-census/src/gate/mod.rs:611`) and the locked/registry path (`mod.rs:503`) call
  `module_path_findings`, which is gated by the field the builder sets (`mod.rs:238-240`,
  `mod.rs:522-531`). `write_debt_with` refuses on any finding before reading or publishing the
  record (`mod.rs:413-416`). With the flag absent, `apply` returns the gate unchanged
  (`cli.rs:104-110`), so the default `Gate` is the same value as before the change.
- (c) Persistence. The flag lives only in `PolicyArgs` and the `Gate` field
  `module_paths_in_package`; neither reaches ledger parsing, the debt record serializer
  (`report.debt.to_turtle()`, `mod.rs:438`) or the index/report writers. Module-path findings
  are appended as findings (`mod.rs:519`) and the library contract says they are never
  recorded as debt (`mod.rs:231-236`). No ledger key, report property or debt-record
  property is added in the diff.
- (d) Test. `crates/cargo-census/tests/delivery.rs:1885-1941` builds a two-package workspace
  where `demo` declares `#[path = "../other/src/shared.rs"] mod shared;`. Without the flag,
  `debt --write` and `check` succeed (valid neighbour, and proof the check is off by default).
  With the flag, `check` exits 1 and prints a finding naming `src/lib.rs:` and
  `in package \`other\``, and `debt --write` exits 1 with the record bytes unchanged. After the
  path attribute is removed, both flagged commands succeed. If the option were ignored on
  either command, the `Some(1)` assertions at `delivery.rs:1919` and `delivery.rs:1930` fail.
- (e) Capability text. `docs/capabilities/census-cli.md:75-76` and the Usage line at `:103`
  match the code: two commands, `Gate::require_module_paths_in_package`, findings printed by
  `check`, exit 1 and an unchanged record on `debt --write`, off by default, nothing
  persisted. "Of another package, or of none" matches the two finding forms in
  `crates/raf-census/src/gate/module_paths.rs:22-25`. The binding, the `raf:tested-by` marker
  and the generated verification-table row (`census-cli.md:145`) name the new test. The
  capability stays `delivered` with no open change record owning it, consistent with recent
  fixes to delivered capabilities.
- Assessments. All 14 renewed files keep their two licence lines; the census-migration
  renewals follow from the changed `cli` and `command` symbol digests they bind.
- Text hygiene. No added line in the diff and no line of the commit message matches a
  tracker number, process wording or agent attribution; the PR body's only tracker
  reference is its `Closes #231` line.

## Findings

1. **OPTIONAL.** The test's doc comment (`crates/cargo-census/tests/delivery.rs:1885-1888`)
   says the debt record "keeps its exact bytes either way", but the test only proves the bytes
   survive a refused write (`delivery.rs:1931`). It never compares a successful flagged write
   with an unflagged one. After `source(root, 2)` the sources match the original fixture
   again, so `assert_eq!(fs::read(&debt).unwrap(), record);` after `rewritten`
   (`delivery.rs:1940`) would cover that claim at no extra cost. The code reading under (c)
   already shows the option cannot reach the serializer, so the contract holds without it.
2. **OPTIONAL.** `census-cli.md:75` states "The other commands refuse the option by name."
   This holds because the argument is not `global` (verified above by reading the code), but
   no test runs it and the paragraph's `raf:tested-by` names only the new test. A one-line
   negative run, for example `index --require-module-paths-in-package` failing, could be added
   to the existing test. Without it, a later `global = true` would turn the refusal into
   silent acceptance with no test failing. The contract does not require this; clap enforces
   it structurally today.

No REQUIRED findings.

## Not verified

- The refusal of the option on other subcommands was established from the clap definitions
  only. The built binary was not kept in the target directory, and the brief limited cargo to
  the two commands above.
- The pre-commit hook, `make check`, the other `cargo-census` and `raf-census` test targets,
  and hosted CI were not run by this review.
- This review covers the commit on merge base `5791d3ab`. Main has moved on by seven
  `raf-source` commits (module declarations inside function bodies), and the merge result
  was not built or tested here. Those commits change the module walk that
  `foreign_module_paths` reads, so CI on the synchronized head is the evidence for the merged
  state.
- Observation outside this change: the shared `fixture()` helper creates fixtures under
  `std::env::temp_dir()` (`delivery.rs:76`), against the repository's test-fixture rule. This
  predates this change, and the change only reuses the helper.

VERDICT: PASS
