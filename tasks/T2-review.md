# T2 review: commits d2046a2, 9bfdaf6, 224ac69, 645f9e7, 68837e4 (HEAD 68837e4)

VERDICT: BLOCKED. Three small required fixes, all of them missing tests (B1-B3).
The design, the placement seam, the floor, the CLI, the kinds, the docs and the gates
all pass. Each fix is one fixture plus its neighbour, and none needs a code change.

How the review was done: `git show`/`git diff` of the commits, and scratch clones
made with `git clone --no-hardlinks`, `pwd` checked each time, at 68837e4 and at
main 95c384c. The worktree was not edited. There were no timing runs. The first
clones were under `/tmp`, and the user deleted them partway through. Results from
them that were already in hand are reported below and marked [/tmp]. Mutants MD-MF
and the baseline were re-run in
`.stage/raf-workplan-strict-mode-force-coverage/scratch/t2review/mut`; see "Checks not
completed" for what that environment limits.

## Required fixes

**B1. The generated-block exclusion of `strict-section-thin` is not pinned by any test.**
`crates/raf-workplan/src/strict/content.rs:129` (`&& !line.in_generated()`) can be
deleted without any test failing. Mutant MA: full `cargo test -p raf-workplan -p
cargo-workplan` exits 0 [/tmp]. The rule promises this exclusion in three places: the
Contract (`docs/capabilities/workplan-strict.md:187-190`, "outside PROPOSED and
generated blocks"), the module doc (`strict.rs:24-28`) and the fix text ("a generated
block ... do not count as prose", `content.rs:66`). The exclusion matters in practice.
A synced verification matrix adds about 2 words per row (`test`, plus the link text
`Verification`) and 3 for its header, so under the mutant a one-sentence Verification
section with a ten-row matrix passes. The fixtures of `strict.rs` only ever hold the
empty, unsynced matrix block.
Fix: in `tests/strict.rs`, extend `code_spans_fenced_code_comments_and_the_heading_are_not_prose`
(`:141`) or add a sibling test. A section of 19 prose words beside a generated block
whose body holds words (for example a filled `verification-matrix` in Verification)
must be `strict-section-thin`. The neighbour at 20 prose words must pass.

**B2. The rule "for each delivered capability" is not pinned.**
`content.rs:26` (`if loaded.value().state == State::Delivered`) can be widened to
every capability (mutant ME, `|| true`) and only the tests that also fail in the
unmutated baseline fail. Every strict fixture has one delivered capability, so no
test shows that an `implementing`, `designed` or `considered` capability is left
alone. Under strict this repository has 8 `implementing` and 10 `designed`
capabilities (the docs_contract counts), so this is the rule that keeps them out of
the strict findings.
Fix: a fixture with a non-delivered capability that breaks every content rule (no
example, no matrix, thin sections, no Usage span, no list item). Under strict it has
no `strict-*` finding, and its neighbour, the same capability delivered, has them.

**B3. The rule "no gap while a capability document does not load" is not pinned.**
`strict.rs:123-125` (the `capability_unloaded` early return) can be deleted (mutant
MF) and no test fails beyond the baseline. The module doc states the rule
(`strict.rs:51-52`: "while a capability document does not load no gap is reported, as
for `scope`"). Without it, one unparsable capability document turns every item it
binds into a false `coverage-item-unbound`.
Fix: a fixture in which a second capability document does not load (a frontmatter
error) and a bound package outside `scope` has a gap. Under strict only the load
finding is reported. The neighbour, with the document loading, reports the gap.

## (1) Placement seam: PASS

* `strict::check` (`strict.rs:78`) is the only producer of strict findings. It is
  called once, from `Workplan::check` (`model.rs` `check`), behind
  `self.strictness() == Strictness::Strict`. No other path reaches it.
* All the other `check()` call sites go through `Opened::workplan[_after]`, which
  applies the floor: `gate.rs:314/339` and `lifecycle/complete.rs:361/366`.
  `lifecycle/create.rs:587` (`change new`) reads only the load findings. The load
  (`relate`/`coverage::check`) takes no strictness, and the
  `gap_finding` refactor of `coverage.rs` keeps the default message byte for byte.
* Byte-for-byte comparison of default mode, main 95c384c against 68837e4 [/tmp]:
  * Library cases: a scratch test dumped `{:#?}` of `case.run()` under `Default` for
    every case of `all_cases()`. All 179 cases of main are identical at HEAD. The 10
    new cases are `okf-bad-strictness`, which is default by design, plus 9 strict
    cases. Under default the 9 strict cases report nothing.
  * CLI: the main binary and the HEAD binary were run on the main tree and on the
    HEAD tree, with text and `--format json` each. Commands: `check`, `-q check`,
    `status`, `coverage code`, `coverage tests`, `evidence claims`,
    `change impact workplan-strict`, `change reconcile workplan-strict`,
    `docs preview workplan-strict`. Stdout, stderr and exit codes are identical for
    every one (32 comparisons).
  * At HEAD, `--strict` leaves these unchanged: `status` (text and JSON), `coverage
    code|tests`, `change impact`, `evidence claims` and `docs sync --check`.
* I tried to break the seam with a record that says strict but does not load
  (`Strict` capitalised): the result is `okf-bad-strictness` alone, Default, fail hard.

## (2) Floor logic: PASS

`Workplan::strictness` is `record.max(floor)`, and a record that does not load reads
as Default. `with_strictness` and `Gate::strictness` both take `self.floor.max(floor)`,
so the floor is never lowered. `Opened::workplan` and `workplan_after` apply
`with_strictness(self.floor)` (`gate.rs:472`, `:498`). `Lifecycle` keeps
`gate.clone()`, so the second open in `complete` keeps the floor. Tests cover each
point; M7/M7b/M7c/M8 of validation.md kill the regressions.

## (3) Rules against the plan, with adversarial documents [/tmp]

The probes reported here were judged under both strictnesses. "Pass" means the rule
raised no finding. Each rule's own fixture and neighbour exist and are correct.

How words are counted (`tense.rs:180`, over `ProseLine::text`):
* Not counted: code spans, fenced code, HTML comments, the `##` heading line,
  PROPOSED and generated lines, and inline-link destinations.
* Counted: everything else. That includes table cells, `###` subheadings,
  blockquotes, HTML tag names and attributes, reference-definition and autolink URLs,
  and bare digits.
* A word is a run of Unicode alphanumerics or `_`. Contractions therefore split
  (`don't` = 2), and a CJK sentence with no spaces is 1 word.
* All of this matches D§3.5 ("as the tense scan cuts them").

| Rule | Probe | Result |
| --- | --- | --- |
| section-thin | 20 words in a table only | pass (tables count) |
| section-thin | `1 2 ... 20` | pass (digits count) |
| section-thin | 20 CJK characters | thin, counted as 1 word (fails toward refusing) |
| section-thin | 10 contractions | pass (counted as 20) |
| section-thin | 19 words padded by a PROPOSED quote | thin, correct (plus the default `state-delivered-proposal`) |
| section-thin | HTML tags only (`<details><summary>a b</summary><div class="x y z">`...) | pass (tag names count; advisory A3) |
| matrix-missing | matrix inside a code fence | refused, correct |
| matrix-missing | matrix in Usage only | refused, correct |
| matrix-missing | other block kind | refused, plus `generated-unknown-kind` |
| usage-unbound | path only in a link `[demo::run](..)` | refused (plan: code span) |
| usage-unbound | code span in a fence or in a PROPOSED block | refused, correct |
| usage-unbound | `<!-- \`demo::run\` -->` in Usage | pass (advisory A1) |
| usage-unbound | `demo::run()` or a `###` heading span | pass |
| usage-unbound | only path is in `tests`, not in `examples` | refused, correct |
| limitations-missing | item in a fence, an HTML comment or a PROPOSED block | refused, correct |
| limitations-missing | `* * *` thematic break; empty `- ` | pass (advisory A2) |
| limitations-missing | `-<TAB>item`; `> - item` | refused (fails toward refusing; advisory A2) |
| example-missing | `no_run` only | meets the rule; `verify` refuses it (bound.rs:774, real cargo) |
| widened (R4) | package bound by two capabilities, one strict-clean | each gap once; the reason names the first binding in graph order (`cap:aaa`) |
| widened (R4) | reached only through a `tests` binding | both directions enforced (items + tests) |
| widened (R4) | reached by an `implementing` capability | enforced ("loaded capability", as documented) |
| widened (R4) | `other` reached, in `scope` | judged once, by the default rule (test exists) |

The stated limit, that a package no binding reaches is not enforced, is pinned by
`a_bound_package_outside_scope_is_enforced_under_strict_and_an_unbound_one_is_not`
and stated in Constraints (`workplan-strict.md:487`).

## (4) 9bfdaf6: legitimate, not a weakening

* `rationale.rs:578`: the count of architecture pages goes from 2 to 3. This is true
  since T1 added `debt-ratchet.md`.
* `roadmap.rs:471-490`: the guard changes from "an open change named after the slug"
  to "an open change that lists it". AGENTS.md section 6 has no naming rule, and the
  `workplan-strict` record reopens eight capabilities, so the old guard could not
  hold. The guard still refuses an implementing capability with no record, and the
  new `elsewhere` case refuses one whose only record lists another capability.
  docs_contract already accepted this (`an_open_capability_may_be_listed_by_a_record_named_after_another`).
* Process note: both tests failed at 03edbca. The T1 gates did not run the
  raf-workplan crate tests.

## (5) CLI: PASS

`--strict` is a clap global flag (`cli.rs:59-63`) and takes no value; the parse test
exists. It is wired to `check` and `change reconcile|complete`
(`command.rs:31-42`), and `change::open` now goes through `Lifecycle::open_with`,
which is equivalent for the default gate. The plan's T2 exit-code cases each have a
test:
* 0 on the strict-clean repository;
* 1 with one thin section, and 0 for the same repository without the flag;
* 2 at a root with no `docs/capabilities` (also run by hand: exit 2 with the message);
* no flag, record `strictness: strict` with one thin section: 1;
* `--strict change complete`: refused, exit 1, nothing written; the same completion
  without the flag gives 0.

On this repository [/tmp], `--strict check` and a scratch record set to `strict`
give identical JSON, exit 1.

## (6) Kinds: PASS

* `OWN_KINDS` goes from 48 to 53 with the five `strict-*` kinds. `okf-bad-strictness`
  comes in through `Rule::ALL` and `record_rule_case`.
* Both directions are covered: `every_rule_and_every_reader_error_has_its_own_kind`
  and `every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind`
  pass. Each kind has a named strict `Case`, plus four cases of bound packages.
* JSON: `a_strict_finding_in_json_is_an_object_of_its_kind_with_the_fields_of_a_finding`
  asserts the field names `kind, path, section, line, key, message, fix` and the
  values.

## (7) My own mutants (not among the 16 of validation.md)

| Mutant | Result |
| --- | --- |
| MC `holds_matrix` accepts a matrix in any section | killed: `strict::verification_without_the_matrix_is_refused_and_with_the_block_is_not` (strict.rs:48) [/tmp] |
| MD the widened set also takes the `scope` packages | killed: `strict::a_package_of_scope_is_judged_by_the_default_rule_once_under_strict` (the only failure beyond baseline) |
| MA prose counts generated lines | **survives** (B1) [/tmp, clean environment, full suite exit 0] |
| ME content rules judge every state | **survives** beyond baseline (B2) |
| MF the unloaded-capability guard removed | **survives** beyond baseline (B3) |
| MB prose counts PROPOSED lines | survives; close to equivalent, since a delivered capability with a PROPOSED block already fails `state-delivered-proposal` (advisory A4) |

## (8) Docs: PASS

* PROPOSED and current text are split correctly. Each reopened capability moves only
  what T2 delivers out of its PROPOSED block: okf-profile **Strictness**, workplan-cli
  `--strict`, the JSON sentence and the Usage line. Nothing still to come is left
  outside.
* `cap:workplan-strict` is `implementing` with real bindings. Its body states the
  designed contract plainly, as the state table of workplan-document-model requires
  for `implementing`.
* The okf-profile architecture page gains the `strictness` row and rule (Addendum,
  T2 row).
* The CHANGELOG adds present-tense lines for T2 and names the breaking changes
  (`CoverageRecord` gains a field; `Rule`/`RuleTag` gain variants). README and
  AGENTS.md say **implementing**.
* Bindings: `workplan check` exits 0 with `scope` set to the two workplan crates. Under
  strict, raf-okf still has the same 177 unbound items as at design time, so the new
  raf-okf items are bound.
* No tracker numbers or process wording: a grep of the added lines and the commit
  messages found nothing, and `make issue-refs` exits 0.
* `d2046a2` fixes both T1 findings: B1 (the designed-only CHANGELOG line) and B2
  (duplicate or unknown slugs in an inventory row, with a test).

## (9) Widened findings: reproduced

`cargo workplan --strict --format json check` on 68837e4 gives 594 findings [/tmp]:

| Finding | raf-okf | raf-source | Total |
| --- | --- | --- | --- |
| `coverage-item-unbound` | 177 | 34 | 211 |
| `coverage-test-unbound` | 148 | 231 | 379 |
| `strict-usage-unbound` | | | 4 |

That is 590 widened findings, exactly the counts of D§3.3. The 4 `strict-usage-unbound`
findings are `workplan-current-rationale`, `workplan-entity-history`,
`workplan-evidence-claims` and `workplan-prose-assessment`; the other 8 of the
design's 12 are `implementing` now. There are 0 `strict-section-thin` findings, where
the design expected 1.

## (10) Gates, scratch clone at 68837e4 [/tmp]: all pass

| Gate | Result |
| --- | --- |
| `cargo test -p raf-okf -p raf-workplan -p cargo-workplan --locked` | 76 test binaries ok, 0 failed |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | 31 / 68 / 1 passed |
| `cargo run -p cargo-workplan -- workplan check` | exit 0 |
| `make fmt`, `clippy`, `doc`, `deny`, `reuse`, `issue-refs` | each exit 0 |

## Advisories (not blocking)

* **A1.** In Usage, a code span inside an HTML comment satisfies `strict-usage-unbound`,
  because `graph.rs:1793` `prose_of` does not blank comments. This is inherited from
  the substantiation rule that the matrix also uses. Either state it, or blank
  comments other than `raf:` markers when T3 adds `raf:tested-by`.
* **A2.** `is_list_item` (`content.rs:160`) has four edge cases:
  * it accepts the thematic break `* * *` and an empty `- `;
  * it refuses `-<TAB>item` and a list inside a blockquote.
  Consider requiring a non-blank item and rejecting a line made only of `*`/`-`/`_`
  and spaces.
* **A3.** HTML tag names and attributes, and reference-definition and autolink URLs,
  count as prose words. Either state this in Constraints, or skip `<...>` tags and
  `[x]: url` lines.
* **A4.** MB: nothing pins that PROPOSED text is not counted. The effect is low
  because of `state-delivered-proposal`.
* **A5.** R2 (Rev 1) asks for the `no_run` refusal by `cargo workplan verify` and by
  condition 5. The test drives `lifecycle::verify`, the library function the command
  and condition 5 share. This is sound in substance; a CLI or condition-5 assertion
  would match the plan's wording.
* **A6.** The CHANGELOG's "**docs:** strict mode ... is designed" line now sits beside
  built behaviour. Reword it to "designed documents" at T8, as the Addendum plans.

## Checks not completed or limited

* The `/tmp` scratch clones, their logs and the scratch test files were deleted by the
  user partway through. Every [/tmp] result above was observed before the deletion;
  the raw files are gone.
* MD, ME and MF and the baseline ran in the in-repo scratch clone with
  `TMPDIR=<scratch>/tmp`. 42 tests fail there with no mutant (baseline), because the
  CLI and gate tests expect a temporary directory outside any git work tree and cargo
  workspace. Mutant failures were judged beyond that baseline, so a killer among those
  42 tests cannot be ruled out for ME and MF. Their fixtures have one delivered
  capability and no unloaded document, so a kill there is unlikely.
* The first in-repo run used `CARGO_TARGET_DIR` inside the repo. That target directory
  (3.6 GiB) has been removed with `cargo clean --target-dir` (after adding its
  `CACHEDIR.TAG`). No in-repo build directory remains.
* `/tmp/t2review-q7k3` (a clone and scripts made before the no-`/tmp` rule arrived)
  still exists. It was not deleted, by the no `rm -rf` rule.
