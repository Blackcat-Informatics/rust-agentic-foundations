# Validation: delivering the lifecycle capabilities with the shipped binary

Each command below was run in the worktree at the commit that resolved the contract
edits, after the author set `state: delivered` on the capability named, with
`cargo run -q --locked -p cargo-workplan -- ...` (so `$CARGO` is the real cargo).
Output and exit code are recorded verbatim.

## cap:workplan-reconcile

`state: delivered` set in `docs/capabilities/workplan-reconcile.md`.

```console
$ cargo run -q --locked -p cargo-workplan -- change reconcile workplan-reconcile
docs/capabilities/index.md  document  generated-stale  line 4: a docs sync run (the `catalogue` block is stale)
[exit 1]
```

```console
$ cargo run -q --locked -p cargo-workplan -- change complete workplan-reconcile
refused: 1 obligation, 1 stale generated block
docs/capabilities/index.md  document  generated-stale  line 4: a docs sync run (the `catalogue` block is stale)
[exit 1]
```

```console
$ cargo run -q --locked -p cargo-workplan -- docs sync
rewrote docs/capabilities/index.md
[exit 0]
```

```console
$ cargo run -q --locked -p cargo-workplan -- change reconcile workplan-reconcile
[exit 0]
```

```console
$ cargo run -q --locked -p cargo-workplan -- change complete workplan-reconcile
completed: removed work/changes/workplan-reconcile.md
rewrote docs/capabilities/workplan-reconcile.md
[exit 0]
```

## cap:workplan-complete-gate

`state: delivered` set in `docs/capabilities/workplan-complete-gate.md`.

```console
$ cargo run -q --locked -p cargo-workplan -- change reconcile workplan-complete-gate
docs/capabilities/index.md  document  generated-stale  line 4: a docs sync run (the `catalogue` block is stale)
[exit 1]
```

```console
$ cargo run -q --locked -p cargo-workplan -- docs sync
rewrote docs/capabilities/index.md
[exit 0]
```

```console
$ cargo run -q --locked -p cargo-workplan -- change reconcile workplan-complete-gate
[exit 0]
```

```console
$ cargo run -q --locked -p cargo-workplan -- change complete workplan-complete-gate
completed: removed work/changes/workplan-complete-gate.md
rewrote docs/capabilities/workplan-complete-gate.md
[exit 0]
```

## cap:workplan-cli

`state: delivered` set in `docs/capabilities/workplan-cli.md`.

```console
$ cargo run -q --locked -p cargo-workplan -- change reconcile workplan-cli
docs/capabilities/index.md  document  generated-stale  line 4: a docs sync run (the `catalogue` block is stale)
[exit 1]
```

```console
$ cargo run -q --locked -p cargo-workplan -- docs sync
rewrote docs/capabilities/index.md
[exit 0]
```

```console
$ cargo run -q --locked -p cargo-workplan -- change reconcile workplan-cli
[exit 0]
```

```console
$ cargo run -q --locked -p cargo-workplan -- change complete workplan-cli
completed: removed work/changes/workplan-cli.md
rewrote docs/capabilities/workplan-cli.md
[exit 0]
```

## Notes

- The capabilities were set `delivered` and completed one at a time. With all three
  `delivered` at once, the repository without the first record would still hold the
  other two records beside their `delivered` capabilities
  (`state-delivered-open-record`), and `complete` refuses on every finding there.
- Wall time of the three `change complete` runs (bound tests and examples included):
  78 s, 102 s and 40 s.
- At the commit that resolved the proposals (41eba82), `cargo-workplan change
  reconcile` printed nothing and exited 0 for each of the three records, with the
  capabilities still `implementing`.
- `cargo-workplan verify workplan-reconcile`, `verify workplan-complete-gate` and
  `verify workplan-cli` exited 0 at commit 0e999a1 (61, 67 and 43 bindings, every one passed,
  after the doc-test fix of e42ff32).
- `cargo-workplan verify` with no slug exits 1 on this repository, for bindings of
  other capabilities: `cap:source-index` binds two `cfg(windows)` tests
  (`raf_source::tree::disk::tests::windows::...`) that the Linux `--lib` list pass does
  not list, and `cap:workplan-document-model` binds the `no_run` doc test of
  `raf_workplan::Gate` as an example. Before e42ff32 it also failed on the
  `Workplan::sync` and `Workplan::preview` examples of `cap:workplan-generated-docs`,
  which now pass.

## Gates after the last commit (92e4de8)

All exit 0: `make docs-sync` twice (nothing written either time, `git status` clean),
`cargo test --locked --test docs_contract --test repo_gates` (26 and 57 passed; the
`Gate` of the repository is clean), `make reuse`, `make issue-refs`, `make licenses`,
`cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D
warnings`, `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked`,
`make deny`, `make msrv`, `cargo test --workspace --locked` (1776 passed, 0 failed).
`.deficiencies` holds only its notice and marker.

## Final local gates (head 9f1ab69 content; each make target run individually)

fmt, clippy, test, test-release, doc, deny, reuse, msrv, fuzz, licenses, issue-refs: PASS (logs in raw/make-*.log).
cross-check: FAIL locally (toolchain `nightly-x86_64-unknown-linux-gnu` and target `x86_64-apple-darwin` not installed). Owner decision on the record: "Hosted CI counts (Recommended)" — run every other gate locally, quote the decision in the PR body, merge only after the hosted cross-check job is green.
Completion audit: reviews/completion-audit.md (all requirements exercised through the real binary; three required items since fixed: T-review re-reviews written, two test git helpers hardened against a user ignore file, edit-table rows added).

## Stage 3 integration assessment

Base 97795488 is an ancestor of head d46f95e, so the branch is not behind; merge-tree is clean
(exit 0). The candidate tree equals the tested tree, so the hosted run on d46f95e (all 12 jobs,
including test-windows, test-macos and cross-check) covers the integration.

Completion audit: the audit of a72c5f5 (reviews/completion-audit.md) drove the real binary
through all seven obligation kinds, 14 complete refusals, the happy path, exit codes 0/1/2,
both argv shapes, -q, JSON and a Git 2.29 stub; it blocked on (1) task reviews without
re-review, (2) two test git helpers not hardened against a user ignore file, (3) edit-table rows
missing. All three are resolved: re-reviews appended (all eleven task reviews end PASS), helpers
hardened in 9f1ab69 and verified under a hostile ignore file, rows added to the plan and listed
in the PR body. Commits after the audited head (9f1ab69, d46f95e) touch test code only (git
helpers and fixture target directories); no product source, document or contract changed, so the
audit's behavioural evidence still applies to the merge candidate. The Windows failure found by
hosted CI on a72c5f5 was a fixture defect, fixed in d46f95e and verified by hosted re-run.
cross-check evidence is hosted CI by the owner's decision "Hosted CI counts (Recommended)".

Merge-time facts (recorded before merge): PR 18 headRefOid is d46f95ec65b57acff3bf9ccdaea532f414b8456d and all 12 hosted jobs pass on that head; review threads 0, reviews 0; `git diff --stat a72c5f5 d46f95e` touches only crates/*/tests paths (plus the stage-ignored files). The local make gates ran on 9f1ab69; the later commit changes test fixtures only, covered by the hosted run on d46f95e. The hosted cross-check job covers raf-source only (Makefile cross-check); non-Linux evidence for the new crates is the macOS and Windows test jobs.
