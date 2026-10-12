# G-R11 review: remediation group R11 (Stage 2 exit-audit F1-F5, V1-V5)

## Scope

Unstaged diff in the worktree, read with `git diff`. Nothing was edited, staged or committed there; all probes ran in mktemp clones, which were removed afterwards.

- Set A: `tests/support/mod.rs`, `tests/repo_gates.rs` (F1, V2).
- Set B: `tests/docs_contract.rs`, `docs/architecture/okf-profile.md`, six capability docs (`census-detectors`, `census-migration`, `census-cli`, `census-debt-ratchet`, `workplan-complete-gate`, `workplan-document-model`), `work/changes/census-migration.md`, `CONTRIBUTING.md` (F2-F5, V1, V3, V4).
- Findings source: `reviews/completion-audit-stage2-exit.md`.

## Evidence

- Worktree, `CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target`:
  - `make clippy fmt test msrv` exited 0 (docs_contract 113 passed, repo_gates 41 passed, `rust-validate 1.98.1 cargo check` ok).
  - `uvx reuse@6.2.0 lint`: compliant, 81/81.
  - `git hook run pre-commit`: exit 0.
- Per-set isolation (clone of HEAD, `cargo test --workspace --locked`):
  - Set A only: docs_contract 112, repo_gates 41, all passed.
  - Set A plus Set B: 113 and 41, all passed.
  - Set B only: 113 and 40, all passed. Each set stands alone.
- F1 hook probe (clone, `make hooks`, real `-S` signed commits, no bypass):
  - Positive control (Set A+B tree): commit `G`, accepted.
  - A second commit appended `See Blackcat-Informatics/rust-agentic-foundations#12 for it.` to `census-cli.md`. The pre-commit hook refused it, and no commit was created. The failures were `repo_gates::issue_refs_tracked_files_are_clean` (`census-cli.md:148: issue or pull request number`) and `docs_contract::repository_documents_satisfy_the_profile`.
- F2 probe (clone): a tracked symlink `docs/outside-link.svg` plus a README link to it.
  - Index state: docs_contract fails with ``README.md: line 76: link `docs/outside-link.svg` targets a tracked symlink``.
  - The same result under a `git checkout-index` snapshot with `RAF_GATE_ROOT`, which is what the hook uses.
  - The signed commit was refused by the hook, with the same message plus the `repo_gates` symlink refusal.
  - `git ls-files -s -z` runs `git -C CARGO_MANIFEST_DIR`, so it reads the index. Under the hook that is the index being committed, because git exports `GIT_INDEX_FILE` to hooks. It is the same source as `tracked_paths`. The snapshot supplies file contents only, and the symlink path is never read, because the loader substitutes `SYMLINK_TEXT`.
  - An unstaged symlink is not in the index: it is neither a path nor a symlink, so the link reports as broken. This is consistent.

## Parity table (Stage 1 `9f93e5d:Makefile` grep versus the new `banned_in_line`)

Old patterns: `#[0-9]{1,5}\b`, `/issues/N`, `/pull/N`, `\bPR ?#?N`, `CHANGE-N`, plus the case-insensitive process set. The new column was measured by calling `banned_in_line` per line.

| Line | Old grep | New gate | Note |
|---|---|---|---|
| `org/repo#12` | caught | caught | F1 fixed |
| `repo#12` | caught | caught | F1 fixed |
| `Fixes#12` | caught | caught | F1 fixed |
| `x#1` | caught | caught | |
| `see #12` | caught | caught | |
| `(#12)` | caught | caught | |
| `PR 11` | caught | caught | |
| `/pull/11` | caught | caught | |
| `CHANGE-17` | caught | caught | |
| `#000` | caught | caught | bare, no colour context |
| `fill="#000"` | caught | spared | colour context, deliberate |
| `#1f2` | spared | spared | |
| `&#38;` | caught | spared | character reference, deliberate |
| `#123456` | spared | spared | |
| `issue-reference` | spared | spared | |
| `#12345` | caught | caught | |
| `a#123456` | spared | spared | |
| `'#abc'` | spared | spared | |
| `:#123` | caught | spared | 3-4 digits after `:` treated as a colour; not in the stated corpus (see A1) |

Real tree: the old grep set returns no hit, and the new gate passes (`issue_refs_tracked_files_are_clean`). The only differences are the deliberate colour and character-reference cases, plus `:#123` (A1).

## Findings

Required: none.

Advisory:

- A1. `hash_number` spares a 3- or 4-digit token after `:`, `"` or `'`. `Fixes:#123` and `Closes:#1234` were caught by the old grep and now pass. Narrow the `:` exemption (for example to a CSS-like `colon + optional space` inside a style or property context), or add a negative fixture that records the accepted gap. This is not a regression against the stated corpus.
- A2. Stale "legacy" wording after the F4 extension (the contract itself is consistent):
  - `census-cli.md:47` and `:143` say `--ledger` names "the legacy ledger".
  - `census-migration.md` Constraints say "Migration needs the legacy baseline" and "The legacy ledger and baseline are read with the bounds…".
  - Say "the ledger to migrate" / "the baseline" instead.
- A3. `docs/architecture/normal-form.md:141-143` still says "The census never reads a ledger under a different version", while `migrate` now does. The sentence is about the check path, but one clause saying that `migrate` is the only reader would stop a reader seeing a contradiction. `census-normal-form.md:58-60` and `:178` are consistent with the new behaviour (they only name `migrate` as the remedy).
- A4. V5 is declined in the diff (no change), but I found no stated reason in the stage files or the tree. If the reason (cosmetic duplicate report, probe m06) is not given in the round report, record it in `validation.md`.
- A5. Some edited doc lines are very long (for example `census-migration.md` Purpose, `census-debt-ratchet.md:78`). This is cosmetic, because there is no wrap gate.

## Checks by item

1. F1: fixed. The word-character exemption is gone, the doc comment is updated, and `issue_refs_rejects_a_hash_number_after_a_word_character` covers `org/repo`, `repo`, `Fixes`, `x` and the full cross-repo form. The existing colour and entity fixtures still pass. The hook refusal is demonstrated.
2. F2: fixed and accurate. The okf-profile text now says "list of tracked paths (the Git index)" and "a link whose target is a tracked symlink is a violation". The fixture `rejects_link_to_a_tracked_symlink` passes, and the real-tree path was exercised through the clone probe.
3. F3: the `sole-name` exception is stated, with two matching Verification bullets. F5: the complete-gate Constraint states the proof by binding kind, and the "Proof" paragraph is scoped to test bindings. F4: the extension is consistent across the census-migration description, Purpose, Contract, Usage, Constraints and Verification, the census-cli table row, census-debt-ratchet (bump path and the "two cases" text) and the change-record claim. `docs_contract` accepts the whole set. The only residue is A2 and A3. No designed doc contradicts another on the contract.
4. V1 done (tests and examples). V2 done (both comments match the Makefile and `decode_text`). V3 done (two cases, consistent with the Explanation). V4 done (state `implementing`, named in the prose and in the Examples lead-in). V5 declined: see A4.
5. Each set passes alone: see Evidence.
6. All required commands pass: see Evidence.

VERDICT: PASS

## Recheck (follow-ups A1, A2, A3, A5, A4/V5)

- Parity, measured with `banned_in_line` in a scratch clone: `Fixes:#123`, `:#123`, `id:#1234` and `color: #123` are all caught now. `color: #1f2`, `fill="#000"` and `&#38;` stay spared, and `org/repo#12` is still caught. The `:` exemption is gone and the comment states the by-design `color: #123` failure. Every row of the original parity table now matches the old grep except the deliberate colour and character-reference cases.
- A2: `census-cli.md` lines 47 and 143 now say "the ledger to migrate".
- A3: `normal-form.md` names `migrate` as the one reader of an older-version ledger. `docs_contract` accepts it, and it joins Set B (13 files changed).
- A4/V5: `validation.md` line 89 records the decline reason (cosmetic double report; the gate still fails closed; deduplicating would need a checker refactor).
- Isolation, in a scratch clone of HEAD: Set A only gives docs_contract 112 and repo_gates 42, all passed. A+B gives 113 and 42, all passed.
- Worktree `cargo test --workspace --locked`: 113 and 42 passed. `make clippy`: exit 0.
- I did not re-inspect the A5 rewrap beyond the passing docs_contract (cosmetic).

VERDICT: PASS

## Recheck 2 (round-2 fix for N1)

Scope: the unstaged diff on top of `edf93d5` in `census-migration.md`, `census-debt-ratchet.md`, `census-cli.md`, `work/changes/census-migration.md` and `tests/docs_contract.rs`.

1. **Clean-job walk-through.** A clean job's forbidden `body:X` is carried by its home alone. The index report lists every unit with its `body` and `shim` fingerprints, including single-member units (`census-normal-form.md:56-57` and the sample at `:116-125`), so the unit carrying `X` is found. The algorithm then resolves the member symbols in the current source, computes the new fingerprint, and writes it, so no row is unmapped. The baseline report, which drops groups of one, is now excluded for current-format ledgers, and the text says why. The pass-through rule is correct against the ledger spec:
   - `table:`/`rule:` ids carry names, not hashes.
   - `[[pattern]]` and `[[distinct]]` hold symbols and members, not fingerprints.
   - `reference` and `max_file_bytes` are plain values.

   The Usage example (`cargo census index > …`, then `--ledger helpers-ledger.toml --out helpers-ledger.new.toml`) matches the default ledger name and the stated `index` output. The new Verification fixture bullet covers the clean job, the `rule:` id, the `[[pattern]]` and the `[[distinct]]` rows.
2. **Legacy path.** The legacy Contract bullets, Usage example and unmapped-row behaviour are unchanged. The pass-through rule is scoped to "a current-format ledger", and "baseline or index report has no entry" widens the unmapped-reason wording without changing the legacy case.
3. **Agreement.** census-migration (description, Purpose, Contract, Usage, bump text, Constraints, Verification), the census-cli table row, the debt-ratchet bump path and the change record all name the index report. `census-normal-form.md` and `census-ledger.md` define the index and need no change. No remaining text says the bump input is the baseline report. The one `baseline` sentence in the Constraints ("since the baseline") is generic and harmless.
4. **Symlink refusal.** The real tree tracks 0 symlinks (`git ls-files -s | grep -c ^120000`), so there are no false positives. In a scratch clone with a tracked `NOTES.md` symlink, a tracked `docs/x.svg` symlink and a README link to it, each symlink was reported once. `NOTES.md` produced no frontmatter or markdown noise, because symlinks are excluded from the link, scan and marker passes. The link violation (`targets a tracked symlink`) is a separate violation on the linking file, not a duplicate. The result is identical under the `git checkout-index` snapshot used by the hook. `rejects_a_tracked_symlink_that_nothing_links_to` and `rejects_link_to_a_tracked_symlink` both pass.
5. **Commands.** `cargo test --workspace --locked`: docs_contract 114 passed, repo_gates 42 passed. `make clippy msrv`: exit 0. `uvx reuse@6.2.0 lint`: compliant.

Advisory only: the doc lines are long in places (cosmetic).

VERDICT: PASS
