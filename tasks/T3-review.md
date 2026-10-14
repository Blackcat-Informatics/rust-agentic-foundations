# T3 review: proposal, state, link and reference rules

## Scope
Staged, uncommitted Task 3 on top of 73c2d71 (22 files): `crates/raf-workplan/src/{rules,links,plain,refs}.rs` (new),
`load.rs`, `model.rs`, `finding.rs`, `lib.rs`, the new and changed raf-workplan tests (`links`, `plain`, `refs`, `states`,
`graph`, `kinds`, `documents`, `support/*`), root `tests/support/mod.rs`, root `Cargo.toml`/`Cargo.lock` (dev-dependency),
AGENTS.md, CHANGELOG.md. Checked against plan Task 3, G2-G5, the rule inventory and R11/R18/R21/R22/R25, the document-model
capability, the OKF profile, `$STAGE/tasks/T3-ported.md`, and the old `tests/docs_contract.rs` code (read_markdown,
read_plain, check_links, check_relative, check_capability_state, check_cross_references).
Mutation work was done in a copy under /tmp (removed afterwards); the worktree was not touched.

## Gates on the staged state (all green)
- `cargo test --locked -p raf-workplan` (debug and `--release`): all pass, including the G2 coverage test over the 13 new kinds
  and the 3 doctests (LinkContext, plain::check, lib).
- `cargo test --locked -p raf-okf`: pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean (so `missing_docs` and the workspace lints hold).
- `cargo test --locked --test docs_contract --test repo_gates` (141 + 57 tests): pass. `make issue-refs`: rc 0.
- `make reuse`: compliant, 302/302 files. `cargo fmt --all --check` and rustfmt on the changed files: clean.
  `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked`: clean.
- Environment note, not a finding: the cargo wrapper created `fuzz/target/debug` inside the worktree on the first run and made
  those commands exit non-zero until it relocated it (`fuzz/target` is now a symlink into /opt/.cargo/target; the path is
  git-ignored). Reruns exit 0.

## (1) G4 oracle: tracked set, never the file system
- `judge_relative` asks `LinkContext::is_tracked` first; the tree is consulted only for the body of a tracked Markdown file when
  an anchor must be read. Probe results (tracked set supplied, tree also holds the file): untracked-but-on-disk `untracked.md`
  gives `link-broken`; `.git/config`, `%73ub/...`, `file.md?x=1` give `link-broken`; a named symlink gives `link-symlink` with or
  without an anchor; `/etc/passwd` and `../x.md` give `link-outside`; directory links, `./sub/../README.md#readme`, `<README.md>`
  and `sub//file.md` resolve; `README.md#Readme` is `anchor-broken`.
- Mutations: letting `is_tracked` fall back to `tree.entry` fails `the_oracle_is_the_tracked_set_and_not_the_tree` and
  `a_document_is_judged_against_the_supplied_tracked_set`. Dropping `with_symlinks` from `is_symlink` fails
  `a_symlink_the_caller_names_is_tracked_and_still_a_symlink` and `rejects_link_to_a_tracked_symlink`.
- `of_tree` semantics are described in `LinkContext` and `Workplan::load` ("the tracked paths of a tree that holds only tracked
  files"; "slow for a large working copy"). The wording is accurate but understates the hazard, see A1.

## (2) The moved matcher
- The matcher body (PROCESS_PHRASES through `banned_in_line`) was extracted from `git show HEAD:tests/support/mod.rs` and from
  `refs.rs` and diffed: identical except the return type (`Option<Banned>`, whose `description()` returns the same two strings
  as before) and added doc comments. `tests/support::banned_in_line` maps it back to `Option<&'static str>`, so every
  `repo_gates` test and the `docs_contract` caller are unchanged.
- Mutating one pattern each in the copy, `repo_gates` fails a named test every time: `/issues/` (issue_refs_rejects_an_issue_url),
  `/pull/`, `CHANGE-`, `issue `, `GH-`, `pull request `, `task ` (two tests), `this issue`, `per the plan`, `this branch`, the
  `1..=5` upper and lower digit bounds, the plan possessive, the acceptance label digit count, the `&` exclusion, the quoted
  3-digit colour exemption, ASCII case folding.
- Survivors, identical before the move (so not a regression): dropping the `#` half of the `&`/`#` exclusion (`##123`), narrowing the
  quoted-colour exemption from 3-4 digits to 3, and removing the word-boundary test in `pull_request_label` (`xPR 5`). The new
  `tests/refs.rs` does not kill them either. See A5.

## (3) Mutation of the rule code (raf-workplan tests, named failures)
Killed, with a named test beyond the generic kinds test:
- State rules: flipping considered, designed, delivered-proposal, delivered-open-record, and the `no-open-record` guard
  (`rejects_delivered_with_proposed_block`, `rejects_delivered_with_open_change_record`, `rejects_open_capability_without_change_record`,
  `rejects_each_non_delivered_state_without_an_open_record`, `accepts_each_state_with_matching_content`).
- `affects` section ignored (`|| true`) and `affects` inverted: `rejects_proposed_section_missing_from_affects`.
- Not-listing check removed: `rejects_proposed_change_that_does_not_list_the_capability`. Missing-change check removed or always on:
  `rejects_orphan_proposed_marker`, `a_block_that_names_a_broken_change_is_not_a_missing_change`.
- Links: absolute/outside link treated as ok (`rejects_link_leaving_the_repository`, `rejects_absolute_and_empty_link_targets_beside_relative_ones`,
  two attribute tests); empty target ignored (`rejects_an_empty_link_target`); `link-symlink` dropped (`rejects_link_to_a_tracked_symlink`,
  `a_symlink_the_caller_names_is_tracked_and_still_a_symlink`); same-document fragment check dropped
  (`a_fragment_link_is_judged_against_its_own_document`, `hostile_an_anchor_only_another_document_has_...`); relative anchor check
  always true (`rejects_broken_anchor`, `anchors_follow_github_rules`); unreadable anchor target (`rejects_a_link_whose_target_cannot_be_read_for_its_anchor`);
  directory links (`a_link_to_a_directory_resolves_and_its_anchor_is_not_read`).
- Anchor slugging (in raf-okf `github_anchor`/`Anchors`): space to hyphen and punctuation retention fail `anchors_follow_github_rules`;
  duplicate numbering starting at 2 fails `hostile_duplicated_heading_anchors_are_numbered_and_the_next_one_is_missing`;
  dropping `_` is killed by raf-okf only.
Survived (needs a test):
- S1: `entry.capability == id && entry.section == section` reduced to `entry.section == section`. No fixture has a change that lists
  two capabilities and affects a section of only one of them while the other holds a block in that section. Behaviour in the real code
  is right (probe: alpha block in Purpose with `cap:beta#purpose` affected gives `proposed-section-not-affected` naming `cap:alpha#purpose`), but nothing pins it. See R2.
- S2: unicode heading anchors (`is_alphanumeric` replaced by `is_ascii_alphanumeric`) survive in raf-workplan and raf-okf. The ported
  `anchors_follow_github_rules` has no non-ASCII heading. See A3.
- S3: the symlinks a walk of the tree reports (`Universe::walk` populating `symlinks`; `is_symlink` without `universe().symlinks`) are
  not covered, since `MemoryTree` has no symlinks. The plan assigns `link-symlink` to the Task 7 disk fixture, but `of_tree`'s
  documented behaviour is untested. See A2.
- S4: removing `is_tracked_file` from the anchor guard (a directory named `x.md`): negligible.

## (4) State rules versus raf-okf (G3)
- raf-okf has no considered/designed/delivered-proposal rule and no open-record rule (grep of `profile/`), so nothing is duplicated.
  Surfaced, not re-implemented: `okf-section-empty`, `okf-binding-required`, `okf-binding-not-empty`, `okf-proposed-outside-capability`,
  `okf-proposed-outside-section` (`surfaces_the_single_document_state_rules_from_the_profile` pins them through `check`; the plain path
  reuses `Rule::ProposedOutsideCapability` through `Finding::from_rule`).
- Strictly, `state-considered-no-proposal`, `state-designed-proposal` and `state-delivered-proposal` read only the capability's own
  state and body, so they are single-document in nature. They are not in raf-okf, the plan lists them for raf-workplan, and
  `rules.rs` groups them with the record rules; only `state-delivered-open-record`, `no-open-record` and the three `proposed-*` rules
  are truly cross-document. Acceptable (A4).
- Fixtures per state: considered accepted (beta in `valid()`) and rejected (`rejects_considered_without_proposed_block`, plus
  `no-open-record` via the not-listing test); designed accepted/rejected (`rejects_designed_with_proposed_block`, `no-open-record`);
  implementing accepted with and without a block, rejected for no record; delivered accepted (gamma, also with the change removed)
  and rejected twice. Met.

## (5) `plain::check` scope against the old docs_contract
Old rules per file class, and the new result:
- Top level, `docs/`, `work/` (scanned): issue references, split links, placeholders, PROPOSED outside a capability, marker errors,
  unclosed frontmatter and links: all kept. Equivalence of `is_scanned` (top-level or first component `docs`/`work`): yes.
  New and stricter: a PROPOSED block outside a capability also gets `proposed-missing-change` when `work/changes/<slug>.md` is not
  tracked (the old code reported that orphan check for every marker too).
- Everything else (`.github/`, `crates/**`): `Body::parse`, split links, links only, no placeholders, no references, no PROPOSED scan:
  same as old (probe: `TODO see #12` in a crate README gives no finding).
- Frontmatter in a file outside the document directories (for example `docs/other.md` with `type: Capability`): old code validated
  it with `Profile` (a typed capability, or its violations); new `plain::check` accepts it silently (probe: no findings). The old
  behaviour was an accident of the unified walk, and the plan and `reads_markdown_outside_the_document_directories_with_or_without_frontmatter`
  make the new behaviour deliberate, but the loss should be in the Task 7 disposition table (A6).
- Lost today, see R1: a non-capability `index.md` inside `docs/architecture` or `work/changes`. The old code reported it as a missing-
  frontmatter document and also read its links, split links, placeholders and markers. `load` now returns after the issue-reference
  scan, and `plain::check` is documented for files outside the document directories, so a caller following G4 never passes it. Probe:
  an `index.md` in each directory holding a broken link, `TODO`, a split link and a PROPOSED marker for a missing change gives zero
  findings. The capability `index.md` gets link checking only (no split-link or placeholder pass; its text outside generated blocks
  is refused by `Index::parse`, so this is low risk).
- A PROPOSED block in a capability that fails profile validation is not checked for `proposed-missing-change` (the old code recorded
  markers before validating). Probe: an invalid beta (empty title) with a ghost block reports only `okf-empty-text`. One fix then
  uncovers the next finding; the gate still fails. A7.

## (6) Behaviours the implementer flagged
- A broken change record gives `no-open-record` for the capabilities it lists (probes: `outcome: done`, `type: Bogus`). Judgement:
  acceptable for a hard-failing gate, because the record's own finding is in the same list, and a typed `changes` list is not available
  from a document that does not validate. It is inconsistent with `proposed-missing-change`, which is suppressed through `present`,
  and the message ("requires an open change record that lists cap:alpha") is misleading when the record exists. A cheap improvement is
  a hint in the fix text (A8). No change required.
- `issue-reference` also covers process wording: matches the old `docs_contract` and `repo_gates` single banned set (the kind says
  "reference", the message names which of the two), and the G5 text. Fine. The scan reads whole file text including frontmatter and is
  applied once per file (a stray page or an unreadable-to-model document is not reported twice, `a_reference_is_reported_once_...`).
  Nested documents are flagged `nested-directory` and not scanned for references; acceptable since they fail anyway.

## (7) Committed text
- No issue or pull request numbers or process wording in added lines (`make issue-refs` rc 0; the banned fixtures are built with
  `concat!`). No task, plan or stage identifiers in the staged diff. SPDX headers present on all new files (`make reuse` compliant).
  `missing_docs` satisfied (clippy `-D warnings`, workspace lints). CHANGELOG and AGENTS.md wording matches the code.

## Findings

### REQUIRED
- R1. Non-capability `index.md` files in `docs/architecture` and `work/changes` are accepted with no rule applied beyond
  issue references (links, split links, placeholders and PROPOSED blocks are all skipped), while `docs_contract` rejects them today and
  Task 7 removes that code. Run `plain::rules` on such an `index.md` in `load::read_file` (as is done for the stray page), or reject it,
  and add a fixture in `tests/links.rs` or `tests/plain.rs` with a broken link in each. The same pass for the capability index would be
  harmless; do at least the two others.
- R2. Pin the capability half of the `affects` entry match: add a states.rs fixture where one change lists two capabilities and
  affects only `cap:beta#purpose`, then a block in `cap:alpha`'s Purpose must yield `proposed-section-not-affected` (probe above).
  The mutation `entry.capability == id` removed survives all tests.

### ADVISORY
- A1. `Workplan::load` and `LinkContext::of_tree` say links are judged against "every file the tree lists". On a disk tree that
  includes untracked and ignored files and the `.git` directory, so a link to an untracked file passes: the exact failure G4 exists to
  prevent. State that plainly in both docs (Task 7's `Gate` must pass a git-tracked list), and consider whether `load` should keep a
  default that hides this.
- A2. Add a `Tree` wrapper test (like `Flaky`) that lists an `EntryKind::Symlink` so `of_tree`'s "tracked symlinks are the symlinks it
  lists" is pinned now and not only by the Task 7 disk fixture (S3). Also: `with_symlinks` registers the symlink as a tracked file but
  not its parent directories, so a link to that directory resolves only if some other tracked path is below it.
- A3. Add a non-ASCII heading to the anchor fixture (S2).
- A4. `rules.rs` module text calls all state rules cross-document; three of the four read one document. Say so, or leave the wording
  at "rules between a capability, its blocks and the records".
- A5. The three pre-existing survivors of the matcher (S-list in section 2) could get fixtures; they are in the moved code, not new.
- A6. Task 7 disposition table: record that frontmatter in plain Markdown outside the document directories is no longer profile
  validated (section 5), and that `unreadable-directory` is only reported from `Workplan::load`, not by `plain::check` with an
  `of_tree` context (an unlistable directory there makes links into it `link-broken`, which fails closed).
- A7. Consider running `proposed-missing-change` for the blocks of a capability that does not validate, as the old code did.
- A8. `LinkContext` holds `Rc`/`RefCell`/`OnceCell`, so it is neither `Send` nor `Sync`; fine for the single-threaded gate, but worth a
  sentence in its docs because the type is public.

VERDICT: BLOCKED

## Recheck

Re-run on the staged state after the fixes (copy in /tmp, removed afterwards).

Gates: raf-workplan tests (debug and release), raf-okf tests, workspace clippy `-D warnings`, `docs_contract` (141) and
`repo_gates` (57), `make issue-refs`, `make reuse` (302/302), `cargo fmt --all --check`, and `cargo doc` with warnings denied:
all pass.

R1 (resolved). My probe (an `index.md` holding a broken link, `TODO`, a split link and a PROPOSED marker for a missing change)
now gives, for both `docs/architecture/index.md` and `work/changes/index.md`: `link-broken`, `okf-placeholder`, `okf-split-link`,
`okf-proposed-outside-capability` and `proposed-missing-change`, each with file and line. Removing the `plain::rules` call for
non-capability indexes in `load::read_file` fails
`an_index_in_a_document_directory_other_than_the_capability_index_is_read_as_markdown`. The capability index still gets the generated-only
parse plus link checking.

R2 (resolved). Reducing the match to `entry.capability == id || true`, or inverting it, fails
`a_record_that_lists_two_capabilities_must_affect_the_section_of_each`; the earlier surviving mutant is killed.

Spot-check of the `no-open-record` suppression (`broken_record` = a `change:` id in `present` with no loaded change). It can hide a real
missing listing only while some record fails to load, and every route into that state carries its own finding, so the gate cannot go
green with the listing missing. Probed: a record with a bad outcome; a Change document in `docs/capabilities` (`wrong-directory` and
`okf-wrong-directory`); a record whose id does not match its file name (`okf-id-file-name`); a record missing required keys. Each reports the
record's finding and suppresses `no-open-record` for an unrelated unlisted capability, which reappears as soon as the record is repaired
(the control run with a good record reports it). A stray page without a fence is not counted as a broken record (it is `stray-markdown`
and the unlisted capability is still reported). Cost: while any record is broken, all unlisted capabilities are quiet, so repairs are
discovered one round later; acceptable under fail-hard. Mutating the suppression either way (always on, never on, record detection off)
fails `a_broken_record_does_not_make_the_capabilities_it_lists_unlisted` and the kinds/reference coverage tests.

Other fixes checked by mutation: the proposed-missing-change check for a capability that does not validate (removing it, or removing the
capability test in `load`, fails `the_blocks_of_a_capability_that_does_not_validate_still_name_a_record`); the unicode anchor mutant now
fails `anchors_follow_github_rules`; dropping the symlinks a tree reports fails `the_symlinks_a_tree_lists_are_tracked_symlinks`.
The docs on `Workplan::load` now say untracked files, ignored files, build output and `.git` pass as link targets on a disk tree.

No new findings.

VERDICT: PASS
