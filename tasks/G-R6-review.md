# G-R6 review: Rust gates port

## Scope
Unstaged diff (Makefile, pre-commit hook, ci.yaml, AGENTS.md, CONTRIBUTING.md, tests/docs_contract.rs) plus new tests/support/mod.rs and tests/repo_gates.rs, against R6 in remediation-plan.md and the old shell gates (`git show HEAD:Makefile`).

## Evidence
- `cargo test --workspace --locked`: docs_contract 46 passed; repo_gates 38 passed.
- Filter counts: `licenses_` 8, `issue_refs_` 25, `citation_` 5 (all above 0). docs_contract fixtures use in-memory repos; the only filesystem use is the index-based `load_repo`.
- `make clippy` (-D warnings, all targets), `make fmt`, `make msrv` (1.98.1), `uvx reuse@6.2.0 lint`, `actionlint`: all clean. `git hook run pre-commit`: exit 0.
- The shipped digests match `sha256sum` (Mulan files eb7a1d71..., zh-Hans e536de48...). The gate passes on them, and the known vectors include a 64-byte block case.
- CI: `gates` job removed; `test` runs `cargo test --workspace --locked`, which includes repo_gates. `grep -c 'this\[ \]issue'` shows a single definition site (support/mod.rs).
- Parity probes, run by pointing `RAF_GATE_ROOT` at a copy and appending a probe line to a tracked file (the file list always comes from the real index, as designed):
  - HIT: `#12` at line start, after `(`, `[`, `-`; `PR#12`; `TaSk 3`; `AC1` and `ac1`; `/pull/7`; CRLF; BOM before `#12`; `issue 12`; `Pull Request 11`; `gh-12`; `CHANGE-1`; `THIS ISSUE`.
  - CLEAN (intended or old parity): `pr 12` (old grep was case-sensitive for `PR`; unchanged), `foo#12` (spared by the plan's prefix rule), `the plans` (old `the.plan.s` also missed).
  - Deleted tracked file: hard failure ("cannot read tracked file ... under ..."); 2 tests fail, not skipped.
  - `git ls-files` failure panics; zero text files fails via `scanned > 0`.
- Parity of patterns and scope: every old pattern is present; all tracked files are scanned (Makefile, CITATION.cff, yml included, since the old gate was not Markdown-only); both Mulan files and the zh fingerprint are checked. Additions per plan: `issue N`, `GH-N`, `pull request N`.
- docs_contract no longer walks the disk; untracked scratch docs cannot count (`load_repo` reads `tracked_paths()` only). `.stage/` is also gitignored.
- Docs: AGENTS.md and CONTRIBUTING.md statements match the tree (hook, `make licenses`/`issue-refs` as subset of `make test`, shared `tests/support/mod.rs`, hooksPath note, version agreement).

## Findings

### Required
1. **Silent skip of non-UTF-8 text (Latin-1) in the issue-reference gate.** `decode_text` returns `None` for any file with invalid UTF-8, and `scan_tree` then skips it uncounted and unreported. Probe: a tracked file with `see #12 caf\xe9` is CLEAN. The old `git grep -I` treated that file as text (only NUL bytes make it binary) and flagged it. This is a scope loss and a silent-pass path. Fix: treat only a NUL byte as binary (as git does), and for non-UTF-8 text either scan lossily (`String::from_utf8_lossy`) or fail hard naming the file. Add a negative fixture (Latin-1 bytes plus a banned token must hit). `read_text`/`load_repo` already panic on non-UTF-8 for the files they need, so that is consistent. The fixture `issue_refs_treats_nul_and_invalid_utf8_as_binary` and `..._skips_binary_ones` need updating to match.

### Advisory
2. **A filter typo can run 0 tests and pass.** `make licenses` and `make issue-refs` run `cargo test --test repo_gates licenses_|issue_refs_`; cargo exits 0 when nothing matches, so renaming the tests (or mistyping the Makefile) silently disables the target. Currently 8 and 25 tests run. Suggest a guard: a repo_gates test that reads the Makefile and asserts each filter prefix matches at least one test name, or have the recipe fail on "running 0 tests".
3. **`cargo test --test repo_gates --test docs_contract` stops at the first failing binary** (no `--no-fail-fast`). The hook still fails correctly, but a repo_gates failure hides docs_contract results. Add `--no-fail-fast`.
4. **`pr 12` lowercase is not matched** (old `PR ?#?[0-9]+` was case-sensitive, so this is parity). Confirm that is intended; `Pull Request 11` and `issue 12` are case-insensitive, so the policy is mildly inconsistent.
5. **Style:** CONTRIBUTING.md line "issue-reference scan, version agreement), all in Rust. Once `raf-census`..." and the AGENTS.md versioning sentence exceed the wrapping used elsewhere. `git add -N` (intent-to-add) paths would panic in the hook snapshot read; acceptable (fails hard), noting only.

VERDICT: BLOCKED

## Recheck
- Finding 1 (Latin-1): fixed. `decode_text` treats only NUL as binary and decodes the rest lossily. Re-probe on the real path (`RAF_GATE_ROOT` scratch copy): `see #12 caf\xe9` now HITS; a NUL-containing line stays skipped (parity with `git grep -I`, and covered by a fixture); clean control stays clean. Both new fixtures are present and assert the right thing.
- Finding 2 (zero-test filters): resolved by removing the filters. `make licenses` and `make issue-refs` run the whole repo_gates binary (40 tests, observed).
- Finding 3: the hook now passes `--no-fail-fast`.
- Finding 4: `pr 12` probe is CLEAN as documented and asserted.
- Finding 5: doc lines rewrapped.
- Runs: `make clippy test msrv` clean (docs_contract 46, repo_gates 40); `git hook run pre-commit` exit 0.

VERDICT: PASS
