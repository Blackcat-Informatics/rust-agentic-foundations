# T5 report

Status: DONE. Commit: see `git log` (feat(raf-census): baseline and index reports, example, determinism). Not pushed.

Files (crates/raf-census): src/report.rs (Baseline, Index and row types; Census::baseline, Census::index(&[JobReport]); to_json / from_json), src/error.rs (Error::Report), src/job.rs and src/group.rs (serde derives on MatchClass, GroupKind, lowercase), src/lib.rs (exports, docs), examples/baseline.rs, tests/support/builtin.rs (the built-in fixture and the printed text, shared by example and test), tests/reports.rs (9 tests).

Decisions:
- Members exactly as the plan lists. Baseline groups carry fingerprint, tokens, members (symbol, file, line); hex_tables rows carry symbol, file, line, table name; shared_constants rows carry value, packages, members.
- Output goes through serde_json::Value (a sorted map; the lockfile has no preserve_order), pretty, plus one trailing newline; a test checks key order. Closed schemas via deny_unknown_fields; a test covers unknown keys (top and nested), a missing key, wrong types, an unknown class, and an index read as a baseline.
- Fingerprints are strings; constant values are lowercase hex strings ("0x...") because a u128 does not fit a JSON number.
- Index `shim` is the unit's shim_key (what it groups and job-matches under), so a trait-impl method or Self:: forwarder shows null though it has a shim form; `body` is the body fingerprint when the body reaches min_tokens (shim candidates included); `tokens` is 0 for non-functions. Units order by symbol, file, line.
- Job classes render as home/variant/copy. The index takes the JobReports the caller matched.
- Example: no argument prints the built-in MemoryTree workspace (body group alpha/beta, shim group one/two, a hex table, 65536, job "sum") under headings `baseline` and `index`; a directory argument is a root of the current directory (absolute paths are their own tree), no jobs. The test pins the whole no-argument output as a Rust string constant and asserts the fingerprints body:6bb022e241a79a28 and shim:203616cb54a3e53d.
- Determinism: fixtures and this repository's crates via DiskTree, two fresh loads, byte-equal; the example run twice with and without `crates` is cmp-equal.

Note for T6: the capability's PROPOSED/designed JSON shows `groups` with members in the baseline and a baseline without `units`/`body_groups` split; the plan's member lists differ (baseline has body_groups, shim_groups, hex_tables, shared_constants; index has units/groups/jobs), so the page's examples must be rewritten from the example output.

Checks (exit 0): cargo test -p raf-census --locked (22 unit, 19 forms, 4 golden, 15 groups, 9 reports, 20 units, 1 doctest); example twice + cmp, and with `crates` twice + cmp (run from the repo root); docs_contract + self_workplan + repo_gates; make fmt clippy doc deny; cargo workplan check. No timing, no mutation runs.

## Fix commit (review findings)

Commit: see `git log` (fix(raf-census): ...). Not pushed.
1. report.rs: `IndexUnit.body` and `.shim` use `deserialize_with = "required"` (reads `Option<T>`; a missing key is an error, null is None). reports.rs adds the null-carrying form as the valid neighbour and two negatives (body key removed, shim key removed), each refused naming the key.
2. tests/support/builtin.rs `of_dir`: an absolute directory is read as its parent's tree with its file name as the root (root "." only when there is no parent); a relative one is a root of the tree at the current directory. The example calls it. New test an_absolute_directory_reports_the_same_as_the_relative_spelling compares `<abs>/crates` with `crates` over the repository root; recorded run: `-- crates` and `-- <abs>/crates` both exit 0 and `cmp` equal.

Checks (exit 0): cargo test -p raf-census --locked (22 unit, 19, 4, 15, 10 reports, 20, 1 doctest); example no-argument output unchanged (cmp); the two spellings cmp equal; make fmt clippy doc deny; docs_contract, self_workplan, repo_gates; cargo workplan check. No timing, no mutation runs.
