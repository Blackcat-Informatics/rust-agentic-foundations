# Plan final review (Stage 1), pass 2 of 3

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/29-index-the-coverage-product-instead-of/.stage/index-the-coverage-product-instead-of/plan.md` (Rev 2; pass 1 reviewed Rev 1)

Reports examined: `reviews/plan-completeness.md` (F1 BLOCKING, F2 BLOCKING, F3-F6 advisory),
`reviews/plan-constraints.md` (B1 BLOCKING, A1-A5 advisory), `reviews/plan-design.md` (advisories
1-5, PASS), `reviews/plan-enhancement.md` (items 1-9), `reviews/plan-adversary.md` (F1-F5). The
orchestrator's dispositions were given to me as the plan text plus the caller's summaries for the
B1 disposition (Rev 1 and Rev 2). No user decision was quoted for any finding.

Code and host facts verified in pass 1 and relied on here: `crates/raf-workplan/src/coverage.rs`
(`check` 563-606, `findings` 611-689, `Bound` 700-756, `ancestors` 759-761, `matches` 781-789,
`exclusion_for` 792-806, `Judge` 832-943, `compute` 1054-1129; the only `add(Work::` sites are
799 and 846), `coverage/tests.rs`, `work.rs`, `tests/coverage.rs` (415-462, 903-931),
`raf-okf/src/profile/records.rs:503-512`, the capability (bindings 9-44, Contract 121-135,
Verification 254-261, matrix 285-292), `tests/docs_contract.rs` and `tests/self_workplan.rs`;
`Makefile:102-107,128`, `rust-toolchain.toml`, `.github/workflows/ci.yaml:106-120`; on the host
(read from disk, no shell): `RUSTUP_HOME=/home/paudley/stage/packages/rustup`,
`RUSTUP_TOOLCHAIN=$RUSTUP_HOME/active-toolchain`, `RUSTUP_AUTO_INSTALL=0`
(`~/stage/dotfiles/dot-bashrc:330-338`), the single installed toolchain
`nightly-2026-10-01-x86_64-unknown-linux-gnu` (`settings.toml`, only `*/bin/rustc`,
`stage.lock.toml`), its `lib/rustlib/` holding `manifest-rust-std-x86_64-pc-windows-gnu` and no
Apple or FreeBSD std, and the machine rule in `dot-claude/RULES.md:15` and
`hooks/rust-toolchain-guard.sh`.

## Pass 1 findings rechecked

### Finding 1 (was BLOCKING): cross-check disposition. DISCHARGED

Rev 2 Task 2 (lines 65-73) now states the mechanism that is true on this host: the single dated
nightly selected through `RUSTUP_TOOLCHAIN`, Windows std present, Apple and FreeBSD std absent,
the machine rule forbidding an agent to install or select a toolchain. It runs the available third
by hand (`RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target
x86_64-pc-windows-gnu`), records the recipe's own failure verbatim in validation.md, reports the
two absent targets as UNRUN locally with the hosted `cross-check` job as the only evidence, and
discloses that in the PR. Every fact in the paragraph matches the files I read. Nothing forbidden
is done and nothing runnable is skipped, so the decline of the constraints rewrite ("install the
targets") now rests on a verified technical reason (the std libraries are not in the Stage
toolchain and adding them is reserved to the user) with the cost stated (two targets unverified
locally, disclosed). The coordinator's judgment that this is a disclosed machine limit rather than
a user decision is defensible: the toolchain policy and the CI `cross-check` job are both the
user's own arrangements, and the PR disclosure puts the fact in front of the user at the point
where it matters. I record it under "Open user decisions" only as the residual choice the user
can make at PR time, not as a plan blocker.

Line 15 ("cross-check via hosted CI where targets are absent") is unchanged but is now an accurate
one-line summary of Task 2; no rewrite needed.

### Finding 2 (ADVISORY): R2(b) shape. DISCHARGED

Line 47 now reads "scale entries AND exclusions together in units 1, 2, 3 and require
`three + one <= 2 * two` on total work (the product's 9 + 1 > 2 * 4 fails; the index passes)",
consistent with the NJ1 idiom on lines 89-90.

### Finding 3 (ADVISORY): R4's unnamed tests. STILL-OPEN, cosmetic

Line 49: "the five tests `the_first_matching_exclusion_gives_the_reason`,
`an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file` and the other
coverage report tests that ... run pass unchanged (the tests the capability binds besides the four
replaced)". The check is executable as it stands (empty diff on the file, the whole file passes,
every bound test passes), so this no longer affects the verdict; but "five" now counts two names
plus an open-ended "other", which is a wobble an implementer will notice.

REWRITE TO: "`crates/raf-workplan/tests/coverage.rs` has an empty diff and every test in it passes,
among them `the_first_matching_exclusion_gives_the_reason`,
`an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file`,
`a_prefix_matches_below_its_module_and_never_a_sibling_that_shares_its_text`,
`an_item_is_excluded_by_its_exported_or_its_defining_path` and
`a_unit_test_module_is_excluded_by_its_prefix_with_the_reason_shown`; the capability Contract
text on exclusions (lines 121-135) has an empty diff."

### Finding 4 (ADVISORY): the scope sentence. STILL-OPEN

Design lines 35-37 are identical to Rev 1: "that is a product over crates, not over entries, and
is outside the issue; stated in the capability text next to the judging statement." The
coordinator reports this fixed; the text was not changed. As written, an implementer can carry
"outside the issue" into the committed capability. Task 1 runs `make issue-refs`, which is the
gate that would catch process wording, so the risk is a late red gate rather than a committed
defect; it stays ADVISORY.

REWRITE TO (lines 35-37): "Scope boundary: `check` also loops over exclusions x crate targets to
find the packages needed; that is a product over crates, not over entries, and this change leaves
it alone. The capability Verification text states it as a fact of the system, next to the judging
statement: the packages a record needs are found by testing the first segment of each exclusion
against each target."

### Finding 5 (ADVISORY): enhancement dispositions. PARTLY DISCHARGED

- Item 6 (dedupe paths): now disposed (line 98). DISCHARGED.
- Item 2 (effectiveness counts): now cites R4. DISCHARGED.
- Items 1 and 4: reasons unchanged and still acceptable; the cost of item 1 (`Bound::covers` stays
  uncounted, so the R2 ceiling bounds exclusion work and entries, not the covers walk) is still
  not stated. Cosmetic; no rewrite required for the verdict. If edited: append to line 97 "...
  reusing `ancestors`; the covers walk stays uncounted, as today."

## Integration check on Rev 2

No contradiction was introduced. The contract rows still map to Task 1; Task 1 keeps its focused
checks, review, commit, push and issue update; Task 2 is the final PR task; there is no Task 0.
The Rev 2 Task 2 by-hand command uses `RUSTFLAGS=-Dwarnings`, which replaces the user's
`~/.cargo/config.toml` `build.rustflags = ["-D", "warnings"]` with the same flag, so the check is
as strict as the recipe's.

## Notes for the implementer (unchanged from pass 1)

- R3's first bullet is already asserted by `the_first_matching_exclusion_gives_the_reason`
  (`tests/coverage.rs:923-930`); cite it rather than duplicate it.
- `test_names` yields at most 120 names; the R1 "10x the exclusions" check needs a base of 12 or
  fewer. `ExclusionsIndexed == exclusions` exposes a silent truncation.
- `KINDS` goes from 19 to 21.

## Open user decisions

None block the plan. Residual, surfaced by the PR disclosure in Task 2: if the user wants
`make cross-check` verified end to end on this host rather than by the hosted job for the Apple and
FreeBSD targets, the user adds those two targets to the Stage toolchain
(`rustup target add --toolchain nightly-2026-10-01-x86_64-unknown-linux-gnu x86_64-apple-darwin x86_64-unknown-freebsd`,
a change the machine rule reserves to the user) and the full `make check` is rerun before merge.

VERDICT: PASS
