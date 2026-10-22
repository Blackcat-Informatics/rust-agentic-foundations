# T3 review: commits a34db9a, 9533c6e, 98b0a9c, 7d35393 (HEAD 7d35393)

VERDICT: BLOCKED. There are four required fixes, all small. B1-B3 are rule bugs in
`strict/claims.rs` that the adversarial documents exposed. B4 is two survived mutants
in the claims report and CLI. The marker, the seam, the kinds, default-mode identity,
the docs and the gates all pass.

How the review was done: `git show`/`git diff` of the four commits. I made in-repo
scratch clones under `.stage/raf-workplan-strict-mode-force-coverage/scratch/t3review/`
with `git clone --no-hardlinks` and checked `pwd` each time:

* `head` at 7d35393, for the gates; left pristine.
* `main` at 95c384c.
* `adv` at 7d35393, for the probes and the CLI comparison binary.
* `mut` at 7d35393, for the mutants.

Builds went only to the target directories the cargo wrapper assigns
(`/opt/.cargo/target/{head-835f5840,main-933cc592,adv-6d310e02,mut-4474b098}`), each
passed as `CARGO_TARGET_DIR`. The wrapper made the in-clone `target` symlinks. I did
not edit the worktree. There were no timing runs.

**Baseline:** `cargo test --workspace --locked --no-fail-fast` in `head` passed
2162 tests with 0 failures, exit 0. The expected set of about 42 environment failures
did not appear in this environment, so the baseline failing set is empty. Every
result below is judged against "nothing fails".

## Required fixes

**B1. Claim keys are not unique: a numbered repeat can collide with a literal lead.**
`crates/raf-workplan/src/strict/claims.rs:674-688` (`number_repeats`) appends `-2` to
a repeated key without checking whether `<key>-2` is already a key of the section.
Probe results:

* Contract `**Foo.** a` / `**Foo.** b` / `**Foo 2.** c` gives `foo`, `foo-2`, `foo-2`.
* The reverse order (`**Foo 2.**` first) gives `foo-2`, `foo`, `foo-2`.

The key is documented as the identity of a claim:

* "the subject a debt row of the claim names" (`claims.rs:60-62`, `ClaimCoverage::key`);
* "stable for the debt and the report" (D§3.2);
* `docs/capabilities/workplan-strict.md:134-140`.

So T7's debt rows would be ambiguous. Fix: when a key is a repeat, take the first
`-<n>` that is no base key or assigned key of the section. Fixture: both orders above
give three distinct keys, and the existing `the-first`, `the-first-2`, `the-first-3`
case is unchanged.

**B2. A thematic break or an empty list item in Constraints is a claim.**
`claims.rs:552-580` and `list_content` (`claims.rs:595-608`) open a unit for `* * *`,
`- - -` and `---` after a blank line, and for `- ` (an empty item). Each gets
`strict-claim-uncovered` with leads such as `"* *"`, `"--"` and `""` (probe
`thematic-break-constraints`, `empty-items-constraints`). This contradicts the T2 fix
of the same commit series (a34db9a): `content.rs:154-190` (`lists_an_item`,
`is_thematic_break`) and `workplan-strict.md:223-226` say that an empty item and a
thematic break are none. Under strict, an author who puts a separator in Constraints
gets a false finding that no marker can sensibly fix. Fix: such a line ends the open
unit and opens none. Reuse `is_thematic_break` and the non-empty-item test, ideally
one `list_content` for both modules (today there are two near-copies). Fixture: those
lines produce no claim, and a neighbour item `- x` is one.

**B3. A comment line does not end a Constraints paragraph, against the stated rule.**
The rule, stated in two places: `docs/capabilities/workplan-strict.md:133-134` and
`crates/raf-workplan/src/strict.rs:47-48` both say "A blank line, a comment line, a
heading, a fence and a PROPOSED or generated block each end a paragraph."
`claims.rs:542-545` instead keeps a comment line inside the open unit ("belongs to
what it follows"), and the next text line continues it. Probe
`constraints-second-para-after-marker`: `Para one.` / marker / `Para two is a separate
guarantee with no marker.` is ONE unit, covered by the first paragraph's marker, so the
second paragraph is never judged. Probe `paragraphs-split-by-comment`: two paragraphs
around a `<!-- note -->` hash as one key. The Contract side does honour the rule
(`claims.rs:405-408`). The same rule is also missed for headings in the Contract:
`claims.rs:400-412` sets a boundary only for `###`/`####`. So a bold-lead paragraph
directly after a `#####` heading (probe `h5-then-bold`) starts no clause, although "a
heading ... ends a paragraph". Fix the code, preferred because the design's unit is
"each paragraph": a comment-only line ends an open Constraints paragraph or table (a
list item may continue with indented lines), and any ATX heading sets the Contract
boundary. Or narrow both doc sentences to what the code does. Fixture: `Para one.` /
marker / `Para two.` gives two units, the second uncovered.

**B4. Two mutants of the report survive: the capability part of the sort, and the
CLI `--capability` filter.** Each was applied alone in `mut` or `head` and restored,
and `git diff --exit-code` was clean after.

* **MR1:** in `claims.rs:191-197`, sort by `(section, line, capability)` instead of
  `(capability, section, line)`. The full `cargo test -p raf-workplan -p
  cargo-workplan --no-fail-fast` passes: 1195 passed, 0 failed, exit 0. SURVIVED.
* **MR2:** in `crates/cargo-workplan/src/coverage.rs:91`, `whole.capability(id)` ->
  `whole`. `cargo-workplan --test claims` gets 3 passed. SURVIVED.

Every report fixture holds one delivered capability
(`crates/raf-workplan/tests/claim_coverage.rs:440`,
`crates/cargo-workplan/tests/claims.rs:60,151`). So "sorted by capability" (the
Contract, the CLI doc of `main.rs`) and the `--capability` filter, which R16 names,
are pinned by nothing. Fix: give the report fixtures a second delivered capability
whose id sorts before `cap:demo`. The library and CLI table and JSON then interleave
by capability first, and `--capability cap:demo` prints only cap:demo's rows (not the
whole report).

## (1) Marker: PASS

* Grammar and placement match the plan. `crates/raf-okf/tests/tested_by.rs` covers
  these malformed forms, each at its line, beside the exact form: space after a
  comma, repeat, empty, `tests=`, a non-path, a missing space, uppercase,
  `tested-byx`, a quote prefix, inline, and split over lines. Unattached: after a
  blank line, after another comment, stacked, before a claim marker, after a fence,
  a PROPOSED block or a generated block, at line 1. It is not read in a fence, a code
  span or a PROPOSED block.
* "Inside a fence" in the plan's malformed list is implemented as "not read", which
  is consistent with `Body::claims` and with the docs that show the syntax in fences.
  Accepted.
* `crates/raf-okf/tests/claims.rs` is unchanged against main (`git diff 95c384c` is
  empty).
* Probes (scratch `adv/crates/raf-okf/tests/zz_adv.rs`):
  * Whole-CRLF document: read, statement `"A statement.\r\n"`.
  * Tab after the word, or before `-->`: `MalformedTestedBy`. A tab indent and a
    trailing tab: read.
  * Unicode path (`demo::café`, `édemo::a`): `MalformedTestedBy`. The `<item-path>`
    pattern is ASCII, as the completion gate's is.
  * Two markers on one line: `MalformedTestedBy`. Two stacked: `UnattachedTestedBy`
    at line 3.
  * Marker at EOF without a newline, LF or CRLF: read.
  * `> <!-- raf:tested-by ... -->`: `MalformedTestedBy`, as for the claim marker.
  * In a list item, or a nested item: read, the statement is the item. After a blank
    line inside the item: unattached.
  * In a table cell: malformed. After a table: the statement is the table.
  * Nested generics and a generic with a space: malformed, consistent with the gate
    pattern.

## (2) Claim units: FAIL on B1-B3, otherwise PASS

Correct, from the probes (scratch `adv/crates/raf-workplan/tests/zz_adv.rs`):

* A bold run mid-paragraph is not a clause. A bold lead after a comment line starts
  one.
* Inside an HTML block (`<div>` with no blank line) there is no clause.
* An indented bold run (in a list item) is not a clause.
* A `***Bold italic.***` lead is the clause `bold-italic`.
* Nested lists, and a table inside a list item, stay in their top-level item.
* A fence in Constraints ends the unit and is none.
* A Unicode lead is kebabed (`café-über`).
* A Contract that is only a fence has no claim.
* An opening beside a later lead is not collapsed and is reported uncovered.
* Collapse: 21 words with no start is `strict-claim-unit-collapsed` (plus uncovered
  when bare), 20 words is not, and a bold lead is not.
* `__Underscore lead.__` and `*italic*` are no clause, but collapse reports them once
  the Contract is longer than 20 words, so the rule is not vacuous (A1).

## (3) Default mode and matrices: PASS

* Library cases: I dumped `{:#?}` of `case.run()` under Default for every case of
  `all_cases()` (scratch `zz_dump.rs`). Main has 179 cases. All 179 are present at
  HEAD, and 178 are byte-identical. The one difference is `okf-parse-unknown-marker`:
  its message and fix now name `raf:tested-by`, with kind and location unchanged.
  This is a listed deviation, judged in (6). The 16 new cases under Default report
  nothing except their own reader, profile or default kind (`claim-test-unbound`,
  the okf ones).
* CLI: the main binary and the HEAD binary each ran on the main tree and on the HEAD
  tree, text and `--format json`. Commands: `check`, `-q check`, `status`,
  `coverage code`, `coverage tests`, `evidence claims gate-kills-a-test-past-its-timeout`,
  `change impact|reconcile census-cli` and `docs preview census-cli`. Stdout, stderr
  and exit code are the same in all 36 comparisons.
* Matrices: the HEAD binary's `docs sync` on the main tree rewrote nothing
  (`git status` clean). `make docs-sync` at HEAD rewrote nothing. The main binary's
  `check` on the HEAD tree is the same as HEAD's (exit 0, no `generated-stale`).
  `tests/generated.rs` only gains a test.
* `claim-test-unbound` comes only from `graph.rs:1627-1652`, iterating
  `Body::tested_by`, so only the new marker raises it.

## (4) `coverage claims`: PASS except B4

* On this repository, two runs are byte-identical in text (57 rows) and JSON.
  `--strict` is the same, and the run writes nothing.
* The literal pins in `cargo-workplan/tests/claims.rs:22-37` match the real output
  (the test passes, and the table equals `render::claims_table` of the library).
* Exit codes: 0 when every claim is covered (and silent under `-q`); 1 when one is
  not (kept whole under `-q`); 2 for `cap:nowhere` with the message; 2 for a usage
  error before the run.
* A5: `--capability` naming a loaded capability that is not delivered prints the
  header only and exits 0, which is consistent with the stated contract.

## (5) Seam and kinds: PASS

* `STRICT_CLAIM_UNCOVERED` and `STRICT_CLAIM_UNIT_COLLAPSED` are emitted only at
  `strict/claims.rs:224,235`. `claims::check` is `pub(super)` and is called only from
  `strict::check` (`strict.rs:120`), which is called only at `model.rs:436` behind
  the strictness test. `claims_report` makes no finding.
* `OWN_KINDS` is 56 (`tests/kinds.rs`), `RuleTag::ALL` 61 (`violation.rs:835`) and
  `ErrorTag::ALL` 48 (`error.rs:662`). The both-directions kinds test names a case
  for each new kind and passes. `coverage-claim` is in the JSON non-rule list
  (`cargo-workplan/tests/json.rs:262`).

## (6) Deviations: each accepted

* **Graph fixture line** (`tests/graph.rs:80`). `the_graph_has_every_kind_of_edge`
  needs an instance of the new `EdgeKind::TestedBy`. The marker names `CASE`, which
  already substantiates that section by code span, so every other assertion of the
  file is unchanged.
* **Strict-clean fixture markers** (`tests/support/strict.rs:69-75`). These are
  needed for the base to stay strict-clean under the new rule. The T2 strict tests
  pass on it unchanged.
* **Unknown-marker text.** I verified that only the message and fix differ. Keeping
  the old text would state a false list of markers.
* **`cap:okf-profile#verification` in `affects`** (`work/changes/workplan-strict.md:24`).
  The Verification of okf-profile now states the tested-by fixtures, so the change
  record must name it.
* **Tested-by before a claim marker is unattached at the tested-by line.** This
  matches the plan's "unattached: before a claim marker" and fixes one order
  (statement, claim marker, tested-by). A tested-by after a blank line followed by a
  claim marker gives `UnattachedClaim` at the claim (probe), which is also correct.

## (7) Mutants of my own

* MR3, a comment line not ending a Contract paragraph (`claims.rs:405`), was killed
  by `strict::claims::tests::a_unit_ends_on_the_line_before_the_next_unit`.
* MR4, the report listing every state (`claims.rs:176` `|| true`), was killed by
  `claim_coverage::the_report_lists_every_claim_of_every_delivered_capability_sorted_and_the_same_each_run`.
* MR5, a Constraints marker covering by its own line rather than its statement start
  (`claims.rs:697`), was killed by 7 `claim_coverage` tests (among them
  `constraints_are_cut_into_top_level_items_paragraphs_and_tables`) and 18 `strict`
  tests.
* MR1 and MR2 SURVIVED (B4).

The runner is `scratch/t3review/mutants.py`, with its log in `mutants.log` and the
MR1 log in `mr1.log`.

## (8) Docs: PASS

* PROPOSED vs delivered (AGENTS.md section 6): each T3 PROPOSED block was resolved
  into plain contract text in the commit that delivers it. The places are:
  * okf-profile Contract;
  * workplan-generated-docs Contract;
  * workplan-document-model Contract (the `claim-test-unbound` bullet), with the
    other strict-only rules kept in the PROPOSED block;
  * workplan-cli: `coverage claims` moved out, while `coverage lines` and
    `debt --write` stay proposed.

  No PROPOSED block still mentions tested-by, `coverage claims`,
  `claim-test-unbound` or the claim kinds. The architecture page gains
  "Tested-by markers" (the T3 addendum item). `cap:workplan-strict` stays
  `implementing`.
* Bindings: every new public item is bound. These include `raf_okf::TestedBy`,
  `TestedByEdge`, `Graph::tested_by`, `Workplan::claims_report` (also an example),
  `claims_table` and `claims_json`, plus `ClaimCoverage`/`ClaimsReport` through the
  `raf_workplan::strict` module binding. Every new test is bound, and the matrices
  list them. Default `check` exits 0.
* Added lines hold no issue or PR number, no process wording and no commit hash
  (grep of `git diff 68837e4 7d35393`). The only hex runs are claim keys. In the
  docs example, `c-3f0a1b2c4d5e6f70` is illustrative. `make issue-refs` passes.

## (9) Gates in `head`: PASS, nothing beyond the (empty) baseline

* `make fmt clippy doc deny reuse issue-refs` all exit 0 (reuse 548/548, issue-refs
  68/68).
* The full workspace test run passes 2162 of 2162.
* `cargo workplan check` exits 0.
* `make docs-sync` rewrites nothing.
* `cargo workplan --strict check` exits 1 as expected mid-change: 57
  `strict-claim-uncovered`, 4 `strict-usage-unbound` and the widened
  `coverage-item-unbound`/`coverage-test-unbound`, all owed to T8. There is no
  `strict-claim-unit-collapsed`.
* `change reconcile workplan-strict` exits 1, as expected for open items and
  proposals.

## Advisories (not blocking)

* **A1.** `__Lead.__` is CommonMark strong emphasis but starts no clause
  (`claims.rs:473-478`). Collapse catches a long Contract. Either accept `__` or say
  "`**`" in the rule text.
* **A2.** Raw HTML lines in Constraints become units: `<details>`, `</details>` and
  `- inside details` (probe `html-block-constraints`). `** * **`, a CommonMark
  thematic break, is a Contract clause keyed `clause` with lead `*`. Both are rare;
  consider them with B2.
* **A3.** A tested-by marker after a heading inside Constraints maps the heading line,
  which no unit holds, so it covers nothing and says nothing (probe
  `heading-in-constraints-with-marker`). Consider `claim-test-unbound`-style feedback
  or a sentence in the Constraints rule.
* **A4.** `list_content` is duplicated in `content.rs:169-181` and
  `claims.rs:595-608` with different return types; B2 is the moment to share one.
* **A6.** Setext headings are read as paragraph text (probes
  `setext-heading-in-contract` and `setext-in-constraints`). This is consistent with
  the reader, which knows only ATX headings, but in Constraints the lead then
  includes the `-----` underline.

## Left in place

* In-repo scratch only: `scratch/t3review/{head,main,adv,mut}`, the logs,
  `cmp.sh`/`cmp2.sh`/`mutants.py` and the dumps. `adv` holds the untracked probe
  files `crates/raf-okf/tests/zz_adv.rs`, `crates/raf-workplan/tests/zz_adv.rs` and
  `crates/raf-workplan/tests/zz_dump.rs`. `head`, `main` and `mut` are clean.
* Outside the repository: the wrapper-assigned build directories
  `/opt/.cargo/target/head-835f5840`, `/opt/.cargo/target/main-933cc592`,
  `/opt/.cargo/target/adv-6d310e02` and `/opt/.cargo/target/mut-4474b098`, plus the
  wrapper's shared build and kache caches. I created nothing in `/tmp`. The suites'
  self-removing temp directories and kache's transient `/tmp/kache-rustc-artifact-*`
  files are created by the tooling.
