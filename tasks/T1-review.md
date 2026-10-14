# T1 review: raf-workplan load, findings, raf-okf tags

## Scope
Uncommitted changes over origin/main 762fa99: raf-okf `RuleTag`/`ErrorTag`, `Rule::ALL`, `Rule::kind`, `ErrorKind::ALL`, `ErrorKind::tag`; new crate `crates/raf-workplan` (finding.rs, load.rs, model.rs, lib.rs, tests); CHANGELOG; workspace member. Judged against plan Task 1, G1-G3, G22, Rule inventory.

## Evidence
- Gates (CARGO_BUILD_JOBS=6): `cargo test --locked -p raf-okf -p raf-workplan` all green (documents 26, kinds 4, rules 1, parse_errors 1, in-crate tag tests, doctest). `cargo clippy --locked --workspace --all-targets -- -D warnings` clean. `docs_contract` 141 and `repo_gates` 57 pass. `make reuse` compliant (291/291). `rustfmt --check` clean on every changed or new .rs file. (`cargo fmt --check` and `make issue-refs` exit 1 only because the cargo wrapper complains about git-ignored `fuzz/target/debug`; the real output is clean and repo_gates passes. Environmental, not the change.)
- Mutation, in a copy under /tmp (the worktree untouched): (a) `RuleTag::UnknownKey` mapped to `OKF_RESOURCE_KEY` and `ErrorTag::Alias` to `OKF_PARSE_ANCHOR`: failed `documents::rejects_unknown_key`, `rules`, `parse_errors` and the `kinds` coverage test. (b) removed the `unreadable-directory` fixture and broke the Alias fixture document: the coverage test failed ("no fixture produces") and `parse_errors` failed. `assert_exact` also rejects any finding outside the case's file or beyond its declared `also` kinds, so fixtures cannot pass by accident.
- Hostile input via DiskTree (scratch crate /tmp/t1probe): binary (invalid UTF-8) file, 2 MiB file, dangling symlink, symlink to /etc/passwd, self-referential symlink: each gives `unreadable-file` (no panic). A directory named `dir.md` is treated as a directory (empty, nothing reported). Empty `.md` gives `stray-markdown` at line 1.
- No issue numbers or process wording in committed files (grep); SPDX headers present; every public item documented (workspace missing_docs lint passes).

## Requirement check (Task 1 against the shipped API)
- `Rule::ALL` with exhaustive in-crate test (G22): met, as `[RuleTag; 36]` (the payload-free idiom), plus the same for `ErrorKind` (43). Both lists are checked by exhaustive matches in-crate; `tag()`/`kind()` match exhaustively, so a new variant is a compile error in raf-okf.
- New crate and workspace member: met. `Workplan::load(&dyn Tree, &Workspace)` reads the three directories through the Tree, validates with `Profile`, produces typed `Loaded<T>` with bodies: met.
- G2 vocabulary: 83 kinds, all built from consts, `KINDS` listed, coverage test in both directions with named fixtures: met. Every Rule and every ErrorKind has its own end-to-end fixture with file and location.
- Directory rules (`wrong-directory`, `nested-directory`, `stray-markdown`) and unreadable-file/-directory: met with fixtures per directory.
- Ported builder/helpers, CHANGELOG entry: met.

## Findings

### REQUIRED
None.

### ADVISORY
1. Double finding for a document in the wrong directory (`okf-wrong-directory` plus `wrong-directory`). Both are mandated by the plan (G2 gives every Rule its own kind and fixture; the inventory lists `wrong-directory`), so it is not a defect, but it is mild noise and sits uneasily with G3 (no duplicated rules): raf-okf already enforces this rule. The workplan one adds the target directory in its fix and fires without a clean profile pass. Fixtures declare the pair explicitly via `also`. Consider documenting in the `Finding::KINDS` doc that the pair is intentional, or having later tasks (Gate rendering) group them. `okf-wrong-directory` also cannot be exercised in isolation.
2. Silent skips in the document directories: a non-`.md` file (`real.txt`), an upper-case `UP.MD`, and a symlink to a directory (`linkdir`) are ignored with no finding; a garbage `index.md` in `docs/architecture` or `work/changes` is ignored (only the capability index is parsed). The plan's stray-markdown rule only covers Markdown files, so this matches the inventory, but the "nothing silently skipped" claim in `lib.rs` and the `Workplan::load` docs is slightly broader than the behaviour. Either soften the doc sentence or add a finding kind in a later task.
3. A file over the read bound (more than 1 MiB + 1) is `unreadable-file` rather than `okf-parse-too-large`, by design of the READ_LIMIT; reasonable, and the message carries the tree's reason.
4. `RuleTag`/`ErrorTag` are exhaustive (not `#[non_exhaustive]`) by deliberate design, so each new `Rule` or `ErrorKind` variant is a breaking raf-okf change (pre-1.0, bumps the minor). The doc comments and CHANGELOG state the intent; the CHANGELOG calls the change "additive", which is true today but worth the PR stating that future variants are breaking for tag matchers. `okf-profile.md` is not touched and the contract is unaffected.
5. `Workplan::check()` clones the findings and equals `findings()` at this stage; fine as a placeholder for later checks since the plan grows it.

VERDICT: PASS
