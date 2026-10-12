# G-R9 review: cliff.toml per-release dedup (G10)

## Scope
Unstaged `cliff.toml` diff (+29/-1) against HEAD. Compared with purrdf `cliff.toml` and commits 446c23318 and 38b0a1da9. Review was read-only; the scratch repo is /tmp/r9scratch.UNwq, outside the worktree.

## Evidence
1. Faithfulness. The `breaking_keys` pre-pass, the per-group `rendered_entries` reset, the `scope::message` key, `set_global`/`concat` and the `not in` test are line-for-line the same as purrdf. Only two things differ. The bullet uses `{{ entry_message }}` where purrdf repeats the equivalent pipeline expression. RAF's comments are reworded and say "the dedup keeps whichever duplicate sorts first (`sort_commits = "oldest"`)". The `\(\s*\)` preprocessor is added in purrdf's position, before the `{2,}` and trailing-whitespace rules.
2. RAF settings. Only the comments, body and one preprocessor line changed. `tag_pattern = "v[0-9]*"`, `commit_parsers` (including the `^ci` and `^build` groups and the `^Merge`/`^chore` skips), `protect_breaking_commits`, `sort_commits`, the header, the postprocessors and both PR-number strippers are untouched. `\(\s*\)` cannot interfere with them: the PR strippers run first and nothing else uses empty parentheses.
3. Key collisions, in the scratch clone with commits signed under the normal config (all showed `G`):
   - Same message, different scope (`feat(a): Same` and `feat(b): Same`): both kept.
   - Case-only difference (`fix(a): foo Bar` and `fix(a): foo bar`): both kept. `upper_first` only normalises the first character, which is the intended equivalence.
   - Same scope and message, different type group (`feat(gts): X` and `fix(gts): X`): kept as separate bullets, because `rendered_entries` resets per group. A blank scope uses the key `::msg`, which is distinct from any real scope. Only an exotic scope containing `::` could collide, and conventional scopes do not.
4. Merge scenario. The setup was a two-parent `--no-ff` merge whose branch holds `feat(gts): X` and a later `feat(gts)!: X (#99)`. The squash subject is on the main side, the branch commit has the same subject without `!`, and `Merge br` is a two-parent commit. Results:
   - OLD config: `feat(gts): X` is printed twice, once marked BREAKING and once plain. Pairs with a breaking commit on either side show the duplicate: `Brk` twice, `ok` twice.
   - NEW config: each pair appears exactly once and is **BREAKING** when either side has `!`. This holds with the breaking commit first (`z`), the breaking commit last (`w`, `gts`), and the breaking commit in the other group (`q`).
5. `git cliff --unreleased` on the worktree: rc 0. Its output is byte-identical to the old config on the real history, which has no duplicates.
6. `uvx reuse@6.2.0 lint`: compliant, 81/81 files.

## Findings
- F1 (LOW, advisory, not blocking). `breaking_keys` is computed over all groups, but `rendered_entries` is per group. So `fix(q)!: Qq` also marks an unrelated `feat(q): Qq` as BREAKING, and `feat(gts)!: X` marks `fix(gts): X`. This was reproduced above: `**BREAKING** **q:** Qq` appears under Features, and `**BREAKING** **gts:** X` under Bug Fixes. It requires an identical scope and subject text across different types, which is very rare. It errs toward over-marking, which is the safe direction. purrdf has the same behaviour, so the port is faithful to the spec. If it should be tightened, add `commit.group` to both keys. That would diverge from purrdf and is not required by R9.
- No blocking defects. Nothing was edited, staged or committed.

VERDICT: PASS
