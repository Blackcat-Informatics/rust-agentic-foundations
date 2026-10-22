# Design: raf-workplan strict mode (force coverage, harder gates)

Read-only analysis of the worktree at `95c384c` (branch
`paudley/16-raf-workplan-strict-mode-force-coverage`). Nothing tracked was edited.
Scratch experiments live outside the worktree (`/tmp/raf16-scratch`, a Python
survey of the capability documents; `/tmp/raf16-proto`, a Rust prototype of reverse
coverage built against the worktree crates by path, its own target directory). No
timing run was made. The `cargo-workplan` binary used for the coverage reports was
the one already built in the sibling worktree of the coverage-index branch, whose
content is the merged `95c384c`; it was run read-only with `--root` on this worktree.

This file is in `.stage/` (git-ignored), so it may name tracker numbers. Nothing it
proposes for a committed file may: see section 7.

## 0. Decisions at a glance

| Question | Decision |
| --- | --- |
| Where the repository opts in | A new optional key `strictness: default\|strict` in the coverage record `work/coverage/default.md` |
| Library API | `raf_okf::Strictness { Default, Strict }` (re-exported by `raf_workplan`), `Gate::strict()`, `Gate::strictness(floor)`, `Workplan::strictness()`, `Workplan::with_strictness(floor)`. Effective = max(record, caller floor): a caller can raise, never lower |
| CLI | global `--strict` (raises the floor); new `coverage claims`, `coverage lines`, `debt --write` |
| Claim unit | Contract: a clause (a bold-lead paragraph or a `###` heading, up to the next one); Constraints: each top-level list item, paragraph or table |
| Claim -> test mapping | A new marker `<!-- raf:tested-by <path>[,<path>...] -->` after the statement (after its `raf:claim` marker when it has one); the matrix substantiation rule also counts it |
| Orphan symbol | No new kind: strict widens the existing `coverage-item-unbound`, `coverage-reexport-unbound`, `coverage-test-unbound` from `scope` to `scope` + every package a binding resolves into |
| `change complete` | "verify ALL bound items" and "zero tense-mismatch" already hold today; strict adds `contract-effect-understated` (process obligation) |
| Reason field | `reason` key of a change item (non-empty text); strict raises `outcome-without-reason` for `dropped`/`moved` without it (condition 2) |
| Baseline | Census semantics exactly (both directions fail until rewritten, write refuses growth except on adoption); a machine-written singleton record `work/debt/default.md` (`type: Debt`); the rule itself stated once in a new architecture page `arch:debt-ratchet` that both ratchets link |
| Reverse coverage | `cargo workplan coverage lines`; covered = inside a bound item, below one, or reachable by resolved path references; ceilings per crate (default 0 under strict), exceptions with reasons, debt rows |
| Repository adoption | In this change: set `strictness: strict`, fix the cheap findings (13), record the rest as debt (about 786 finding rows + 4 line rows) |

## 1. How today's gates work

### 1.1 Load and `check`

* `Workplan::load`/`load_with` (`crates/raf-workplan/src/model.rs:146-160`) reads
  `docs/capabilities`, `docs/architecture`, `work/changes` and the records of
  `work/evidence`, `work/assessments`, `work/coverage` through `raf-okf`. Every
  problem is a `Finding`; nothing panics.
* `Workplan::relate` (`model.rs:253-280`) builds the graph and appends, in order:
  graph findings, `bindings::check` (`bindings.rs:74`), `rules::check`
  (`rules.rs:28`, PROPOSED/state/no-open-record rules), `coverage::check`
  (`coverage.rs:563-606`), `rationale::check` (`rationale.rs:106`).
* `Workplan::check` (`model.rs:364-368`) = load findings + `generated::analyze`
  (index-missing, generated-stale, misplaced, mixed line endings). A clean `check`
  is a sync with nothing to write.
* `Gate` (`gate.rs:168-354`): `workspace(dir)`, `source_roots`, `tracked_paths`,
  `programs`, `open() -> Opened`, `try_check`, `check`, `assert_clean` (panics with
  `kind file:line message (fix)`). `Opened::workplan()` (`gate.rs:423`) loads the
  documents; `Opened::workplan_after` (`gate.rs:431-449`) is the staged view
  `complete` judges after removing the record. The repository's own gates call
  exactly this: `tests/self_workplan.rs:28-64` (`Gate::workspace(..).open()`,
  `workplan.check()` empty, every assessment current and accepted) and
  `tests/docs_contract.rs` (`Workplan::load_with` plus repository policy such as
  `INVENTORY`).

### 1.2 Where a `delivered` capability is accepted

A delivered capability passes today when:

1. the profile accepts it: `bindings.code` and `bindings.tests` are non-empty
   (`raf-okf/src/profile/mod.rs:542-553`, `okf-binding-required`; `examples` may be
   empty), every section is present, in order and non-empty (`okf-section-empty`),
   no placeholder;
2. every binding resolves to one symbol of the right role (`binding-unresolved`,
   `-ambiguous`, `-wrong-role`, `-macro-generated`);
3. it has no PROPOSED block and no open record (`state-delivered-proposal`,
   `state-delivered-open-record`, `rules.rs:146-153`);
4. its generated blocks are current (only if it has them: a missing
   verification-matrix is not a finding of the library; the repository adds that
   rule in `tests/docs_contract.rs` for the inventory);
5. links, anchors, issue references, current-rationale rules hold;
6. coverage holds for the crates of `scope` only.

Nothing measures section length, the Usage example, Limitations, or which claim a
test supports. Those are asserted only by the Scenario B detector in
`crates/raf-workplan/tests/scenarios.rs:1000-1165` (`Account::faults`: example
binding, matrix rows, `fenced_rust` usage that builds and runs, `>= 3` Constraints
bullets, `>= 4` API references), which is the "test-side detector" the issue means.

### 1.3 Findings

* `Finding { kind: &'static str, file, location: raf_okf::Location, message, fix }`,
  `#[non_exhaustive]` (`finding.rs:186-199`).
* The closed list: the `kinds!` macro (`finding.rs:14-178`) declares `kind::*`
  constants and `ALL_KINDS`; `Finding::KINDS = ALL_KINDS` (`finding.rs:297`). Kinds
  are 48 own kinds + one `okf-<rule>` per `raf_okf::Rule::ALL` + one
  `okf-parse-<error>` per `ErrorKind::ALL` (minus `missing-fence`, `index-text`).
* Both-directions test: `crates/raf-workplan/tests/kinds.rs:83-125`. `OWN_KINDS`
  pins the 48 own kinds; `every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind`
  runs `all_cases()` (`tests/support/cases.rs:1152-1164`); each `Case`
  (`cases.rs:16-120`) is a valid repository with one break, and `assert_exact`
  requires the one kind at the one place and nothing else. The CLI side
  (`crates/cargo-workplan/tests/json.rs:77,263`) proves every printed rule kind is in
  `KINDS` and no non-rule JSON kind equals one.
* Obligations are a second closed list: `ObligationKind` (`obligation.rs:76-128`,
  `ALL` of 13 with a compile-time rank check), covered both ways by
  `tests/complete.rs::the_cases_cover_every_kind_of_obligation_in_both_directions`.

### 1.4 `change reconcile` and `change complete`

* `Workplan::reconcile` gives the static kinds; `Lifecycle::reconcile`
  (`lifecycle/session.rs:371-379`) adds the process kinds through
  `process_obligations` (`session.rs:387-415`): `untouched::raise`
  (`lifecycle/untouched.rs:133`, section vs merge base via
  `git cat-file blob <merge-base>:<prefix><path>`), `changed::raise`
  (evidence at merge base), `examples::raise`.
* `contract_effect: none` suppresses only `section-untouched` for
  contract/usage/examples: `obligation::suppressed` (`obligation.rs:719-730`).
* `tense-mismatch` is raised for every listed `implementing`/`delivered`
  capability from `tense::PHRASES` over all prose (`obligation/sources.rs:394-406`).
* `Lifecycle::complete` (`lifecycle/complete.rs:187-220`): evaluate the static
  conditions (`evaluate`, `complete.rs:226-345`), run the bound tests and examples
  (`run_bound`, `complete.rs:377`), open again and re-evaluate, then rewrite the
  stale blocks and remove the record. The six conditions:
  1. obligation list empty (every kind, including `tense-mismatch`);
  2. every item has an outcome and a `moved` target exists;
  3. no PROPOSED block of the change;
  4. sync writes nothing;
  5. the runs: `impact.bindings` filtered to `Test | Example`
     (`complete.rs:329-335`), i.e. **every** test and example binding of every
     listed capability, judged by the executor's proof;
  6. a current accepted assessment of each authored section in `affects`;
  plus `check` of the repository after removal (`after_removal`,
  `complete.rs:354-375`), which fails on every finding anywhere.

### 1.5 `verify`

`lifecycle::verify` (`lifecycle/verification.rs:205-`) takes the bindings of a
change (or of every capability), keeps `Test | Example`, sorts them, runs each
through `bound::Executor` (list pass + run pass, exact proof), writes nothing, asks
git only for tracked paths. `cargo workplan verify` exits 1 when one fails.

### 1.6 Two-way coverage and evidence (the delivered #10 work)

* `coverage::check`: with no record and capability files, one
  `coverage-record-missing`; otherwise, for crates in `scope`, each unbound public
  item, unresolved re-export, test, and unreadable surface; and every unmatched
  exclusion (`coverage.rs:611-689`). `Bound::covers` (`coverage.rs:707-756`): a
  `code` binding covers the item, its same-source copies, and everything below
  its path except below a crate root.
* Evidence claims: `<!-- raf:claim evidence=<ids> -->` (`raf-okf/src/body/marker.rs:115`),
  `claim-evidence-missing`, `claim-evidence-not-for`, and the obligations
  `claim-unsupported`, `evidence-stale`, `evidence-changed`.

### 1.7 CLI

`crates/cargo-workplan/src/main.rs:13-243`: global `--root`, `--format text|json`,
`-q`; exit codes 0 (clean or outcome), 1 (findings), 2 (could not run)
(`exit.rs`); `coverage code|tests [--crate]` (`src/coverage.rs:22-41`) prints
`coverage::render` and exits 1 when a gap is printed. The binary holds no rule.

## 2. Settings design

### 2.1 One location: the coverage record

```yaml
---
# SPDX-FileCopyrightText: ...
# SPDX-License-Identifier: CC-BY-4.0
type: Coverage
id: coverage:default
strictness: strict          # optional; `default` when absent
scope: [raf-workplan, cargo-workplan]
exclusions: []
ceilings: []                # optional; reverse coverage, section 3.9
exceptions: []              # optional; reverse coverage, section 3.9
---
```

Justification (to be written into the Explanation of `cap:workplan-strict` as a
`### Decision` with `### Conditions to revisit`):

* It is already the one repository-level policy record. `check` requires it as soon
  as capability documents exist (`coverage-record-missing`), it states what the gate
  enforces (`scope`), and it is in the graph (`ScopeEdge`). Strict is "enforce more";
  putting it beside `scope` keeps one place to read what is enforced.
* It is a closed, validated schema: an unknown key or value is a profile finding with
  a location, so a misspelt `strictness: strct` fails hard instead of silently meaning
  default. The writer (`CoverageRecord::to_document`) and `record::replace` exist.
* Most strict rules are coverage rules (claims, orphan symbols, reverse lines,
  ceilings, exceptions); the reverse-coverage settings belong to this record anyway.
* Rejected, "a field beside `docs/capabilities/index.md`": the index is generated
  only (`index-hand-written`), the directory is flat and holds capability documents
  only; the repository's own `docs_contract` refuses any non-Markdown file in a
  document directory, and a Markdown file there is a capability or `stray-markdown`.
* Rejected, `Cargo.toml` metadata: `raf-workplan` reads manifests only through
  `raf-source` package discovery, which exposes no metadata table; cargo itself never
  refuses an unknown metadata key, so the closed-schema rule would need a second
  validator; a workspace has both `[workspace.metadata]` and per-package tables, so
  "which one" is ambiguous; and a finding could not point at a document location.
* Rejected, a new `workplan.toml`: a second format and parser beside OKF for one key.

The value is a closed enum, like `contract_effect: none|changes`, because the OKF
profile has no boolean value kind (`raf-okf/src/profile/schema.rs:13-57`).

### 2.2 Library API

`raf-okf` (typed value of the record key; `schema.rs:152` gains
`optional("strictness", ValueKind::Strictness)` after `id`/optional keys,
`CoverageRecord` gains `pub strictness: Strictness`, new rule
`Rule::BadStrictness { found }` -> `okf-bad-strictness`):

```rust
/// How hard the gate judges a repository. The order is the order of strength.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Strictness {
    /// The rules of the document model, coverage of `scope`, the completion gate.
    #[default]
    Default,
    /// Those rules and the strict rules of `cap:workplan-strict`.
    Strict,
}
```

`raf-workplan`:

```rust
pub use raf_okf::Strictness;

impl Gate {
    /// Judge under strict mode whatever the coverage record says.
    #[must_use] pub fn strict(self) -> Self { self.strictness(Strictness::Strict) }
    /// The least strictness to judge with. The coverage record can raise it and
    /// nothing lowers it.
    #[must_use] pub fn strictness(mut self, floor: Strictness) -> Self { self.floor = floor; self }
    /// Write `work/debt/default.md` as the present strict debt (section 3.8).
    pub fn write_debt(&self, options: &RecordOptions) -> Result<DebtWrite, DebtError>;
}
impl Opened { pub fn strictness_floor(&self) -> Strictness; }   // carried from the Gate
impl<'s> Workplan<'s> {
    /// max(the record's `strictness`, the floor). A record that does not load
    /// counts as `Default` (its own findings fail the check).
    pub fn strictness(&self) -> Strictness;
    #[must_use] pub fn with_strictness(self, floor: Strictness) -> Self;
    pub fn claims_report(&self) -> strict::ClaimsReport;          // section 3.2
    pub fn lines_report(&self) -> coverage::lines::LinesReport;   // section 3.9
}
```

* `Workplan::check` keeps its signature. When `strictness() == Strict` it appends
  `strict::check(self)` and then applies the debt (`debt::apply`), after
  `generated::analyze`. Strict findings are computed lazily in `check`, never in
  `relate`, so a default load does no extra work.
* `Opened::workplan()` and `Opened::workplan_after()` (`gate.rs:423,431`) apply the
  floor, so `Lifecycle::open_with(&gate.strict(), runner)` judges reconcile, the
  static conditions and the after-removal check under strict. `Lifecycle::open`
  (default gate) follows the record only.
* `obligation::suppressed` gains a `Strictness` parameter (pre-1.0, one caller,
  `untouched.rs:145`).
* Why "max" and no way to lower: a repository that opts in must not be bypassed by
  omitting a flag; the floor exists so a caller can try strict before committing the
  key (fail-hard direction only).

### 2.3 CLI

* Global `--strict` (beside `--root`, `--format`, `-q`): `Gate::workspace(root).strict()`.
  It changes `check`, `change reconcile`, `change complete`; `status`, `docs`,
  `history`, `evidence`, `assess`, `verify` and the coverage reports are the same
  under it (the reports are always complete; enforcement is `check`'s).
* `coverage claims [--capability cap:<slug>]`, `coverage lines [--crate <package>]`
  (section 3.9 on why not `coverage --reverse`).
* `debt --write [--copyright <text> --licence <id>]`: the census grammar
  (`cargo census debt --write`). Exit 0 written; 1 refused with findings printed
  (growth, or a finding that is not debt); 2 when the record does not set
  `strictness: strict`, or the usual could-not-run cases.

## 3. The rules

Each strict finding is produced by `crates/raf-workplan/src/strict.rs` (new, with
submodules `strict/content.rs`, `strict/claims.rs`) or by an existing module given
the strictness. Internally a strict rule returns `Owed { finding: Finding, debt:
DebtKey { kind, subject } }`; the public `Finding` shape is unchanged.

### 3.1 Examples and the verification matrix (R2)

Mechanism, for each loaded `delivered` capability:

* `bindings.examples` empty -> `strict-example-missing`, `Location::Key("bindings")`.
  Fix: "bind at least one example target or runnable documentation test in
  `bindings.examples`; strict mode asks every delivered capability for a usage that
  runs". Runnability itself is proven where it is today, by the executor in
  `verify` and condition 5 (a `no_run`/`compile_fail` doc test is refused there).
* no `verification-matrix` block in the Verification section ->
  `strict-matrix-missing`, `Location::Section("Verification")`. Fix: "add
  `<!-- raf:generated:verification-matrix -->` and `<!-- raf:generated:end -->` to
  the Verification section and run `cargo workplan docs sync`". A block elsewhere is
  already `generated-misplaced`/accepted by the model; strict asks for it in
  Verification only. Staleness stays `generated-stale`.

Extends: nothing but `strict::content`. Reads `Loaded<Capability>::value().bindings`
and `body().generated_blocks()`.
Cost: small (about 60 lines + 4 fixtures). Risk: none for default; the repository's
`docs_contract` matrix check duplicates the second rule for the inventory; keep it
(it pins the inventory independently of the record).
Today: 0 findings (all 12 delivered capabilities bind 1 to 5 examples and hold a
matrix).

### 3.2 Claim coverage (R3)

**Claim unit.** Read in the authored prose of the Contract and Constraints sections
of a `delivered` capability (outside fenced code, generated blocks, PROPOSED blocks;
headings inside fences do not count):

* Contract: a clause. A clause starts at a paragraph that begins with a bold run
  (`**The public surface.**`) or at a `###`/`####` heading, and runs to the start of
  the next clause or the end of the section. Text before the first clause start, if
  any, is one clause, the opening.
* Constraints: each top-level list item, and each paragraph or table outside a list.

Why not "every paragraph": the documents state one guarantee per bold-lead clause
over several paragraphs, lists and tables (e.g. the six conditions of the
completion gate are one clause); paragraph granularity would demand markers on
continuation paragraphs that carry no separate claim. Why not "explicit claim marker
only": then an unmarked claim is invisible and the rule is vacuous.

**Claim key** (stable for the debt and the report):
`cap:<slug>#<section>/<key>` where `<key>` is the kebab slug of the bold lead
(`the-public-surface`) or of the heading, `opening` for the opening, and for a
Constraints unit `c-<16 hex>` of the SHA-256 (`digest` module) of its text with
whitespace runs folded to one space. Repeats get `-2`, `-3` in document order. A
reworded Constraints item is a new claim (section 7, risk 4).

**Mapping.** A new marker, written on its own line directly after the statement,
or directly after the statement's `raf:claim` marker:

```markdown
- An exclusion with an empty reason is an error of the record.
<!-- raf:tested-by raf_workplan::test_crate::coverage::an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file -->
```

* Grammar: `<!-- raf:tested-by <path>[,<path>...] -->`, one space after the word,
  entries separated by `,` with no space, each matching the `<item-path>` pattern of
  the completion gate (`workplan-complete-gate.md:386`), no repeats. Parsed in
  `raf-okf/src/body/marker.rs` as a third marker family beside generated and claim
  (`family_of`), giving `Body::tested_by() -> &[TestedBy]` (`line()`, `paths()`,
  `statement()`, `section()`).
* It is **not** an extension of `raf:claim`: the existing fixture
  `crates/raf-okf/tests/claims.rs:157-183` pins `<!-- raf:claim evidence=a tests=b -->`
  as malformed, and `claims.rs:190-212` pins that a claim marker after a claim marker
  or after any comment is unattached. A new family keeps both fixtures as they are
  (`raf:tested-by` is no word any fixture uses: the marker words in fixtures are
  `generated`, `claim`, `note`, `foo`, `todo`, `clam`, `claims`, `anything`).
* Profile/reader kinds: `okf-parse-malformed-tested-by`,
  `okf-parse-unattached-tested-by` (new `ErrorKind`s, so new `okf-parse-*` kinds),
  and `okf-tested-by-outside-capability` (new `Rule`, like
  `okf-claim-outside-capability`).
* Default-mode rule (new syntax only): an entry that is not a path of the same
  capability's `bindings.tests` or `bindings.examples` -> `claim-test-unbound` at the
  marker line. Fix: "name a path of the capability's `bindings.tests` or
  `bindings.examples`, or bind the test there first".
* A unit is covered when it holds at least one `raf:tested-by` marker (for a
  Constraints item, attached to that item). Evidence markers do not count: evidence
  is a recorded execution, not a bound test (section 4).

**Strict finding.** Each uncovered unit -> `strict-claim-uncovered` at
`Location::Line(<first line of the unit>)`, message naming the key and the first 60
characters of the lead or item. Fix: "put `<!-- raf:tested-by <path>[,<path>...] -->`
after the statement, naming the bound tests or examples that show it, or move
reasoning that no test can show into the Explanation".

**Matrix rows -> claims.** `graph.rs:1768` `substantiated(body, path)` today: a
binding substantiates each non-Verification section whose prose names its path in a
code span, else Verification. Extended: also each section holding a `raf:tested-by`
marker that names the path. So a test named by a Contract marker shows
`[Contract](#contract)` in the matrix Section cell (the block format and columns are
unchanged, so every current matrix is byte-identical). The claim-level table is a
report, not a generated block:

```text
$ cargo workplan coverage claims --capability cap:workplan-two-way-coverage
| Capability | Section | Claim | Line | Tested by |
| --- | --- | --- | --- | --- |
| cap:workplan-two-way-coverage | constraints | c-3f0a1b2c4d5e6f70 | 199 | `raf_workplan::test_crate::coverage::...` |
| cap:workplan-two-way-coverage | contract | covered | 103 |  |
```

JSON kind `coverage-claim`: `capability`, `section`, `claim`, `line`, `tests`.
Exit 1 when a claim of a delivered capability has no test.

Extends: `raf-okf` body markers, `graph.rs` (a `TestedByEdge` from the section to the
binding, and `substantiated`), `generated.rs` (none: it reads `Binding::sections`),
`strict/claims.rs`, `cargo-workplan` (`coverage claims`). The architecture page
`docs/architecture/okf-profile.md#claim-markers` states the marker syntax and gains
the new family.
Cost: the largest document cost of all rules (section 6.3). Code about 900 lines
with fixtures.
Risks: (1) authors may cite a weak test; the rule proves a mapping exists, not that
the test shows the claim; the assessment of the section is where a reader judges
that (assessments include the section text, so the marker is assessed with it).
(2) Long marker lines; accepted (comments do not render).
Today: 196 units (95 Contract clauses, 101 Constraints units), 0 covered.

### 3.3 Orphan symbols (R4)

"Every public item of a bound crate path is explained by some capability, reusing
the two-way coverage direction." Mechanism: under strict, `coverage::check`
(`coverage.rs:563-606`) enforces the union of `scope` and the **bound packages**:
every package of the source workspace that holds an item some `code`, `tests` or
`examples` binding of a loaded capability resolves to. The gaps of a package enforced
only because it is bound are reported with the existing kinds
`coverage-item-unbound`, `coverage-reexport-unbound`, `coverage-test-unbound`,
`coverage-surface-unreadable`, with a message that says "which strict mode enforces
because `cap:<slug>` binds `<path>`" instead of "which the coverage record
enforces". Both directions, so a bound package's tests must support a capability too.

* No new kind: the rule is the same rule over a larger set, and a second kind for the
  same gap would duplicate the delivered direction (R19).
* Why not require the packages in `scope`: `scope` is the default-mode setting;
  adding a package there makes its gaps default findings, which no baseline may hold
  (section 3.8). Gaps enforced by strict alone are strict findings and debt-eligible.
* `coverage::check` gets `strictness` from `plan.strictness()`; the needed-package
  set (`coverage.rs:588-596`) gains the bound packages; `findings` (`611-689`) gets
  the reason for each package.

Cost: small (about 80 lines + 3 fixtures). Risk: the tests direction of a bound
package can be large (here raf-source has 231 unbound tests).
Today: raf-okf and raf-source are bound and outside `scope`: 211 public items
(raf-okf 177, raf-source 34) and 379 tests (raf-okf 148, raf-source 231);
re-exports 0. The root package `rust-agentic-foundations` (97 tests) is not bound,
so not enforced.

### 3.4 `change complete` under strict (R5)

* "requires `verify` to pass for ALL bound items of the listed capabilities": already
  condition 5. `evaluate` collects `impact.bindings` of kind `Test | Example` for
  every listed capability (`complete.rs:329-335`) and `run_bound` runs each one with
  the proof `verify` uses; `verify <slug>` and condition 5 share
  `bindings_of_change`. Strict adds nothing; the capability document says so and
  binds the existing tests (`test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists`,
  `test_crate::complete::a_failing_bound_test_refuses_condition_five_alone`).
* "zero tense-mismatch findings on those capabilities": already condition 1;
  `tense-mismatch` is raised for each listed `implementing`/`delivered` capability over
  all its prose (`obligation/sources.rs:394-406`) under either effect.
* New, strict only: "forbids `contract_effect: none` for changes that touch a bound
  public item". A process obligation `contract-effect-understated`
  (`ObligationKind::ContractEffectUnderstated`, needs git; rank after
  `SectionUntouched`), raised by a new `lifecycle/touched.rs` from
  `process_obligations` when the record says `contract_effect: none`:
  1. The candidates: every item of the public surface (`exported_items`) that a
     `code` binding of any loaded capability covers (`Bound::covers`), grouped by file.
  2. For each such file, read it at the same merge base as `untouched::raise`
     (`git cat-file blob <merge-base>:<prefix><path>`, the existing row of the
     reconcile invocation table, so no new git command), bounded by the source
     index's file bound. A file absent there makes every candidate in it touched.
  3. The **declaration** of an item: its tokens with `#[doc]` attributes removed and
     every function block replaced by `{}`, rendered by `proc_macro2` (a dependency of
     `raf-workplan`). For the working tree, the item's span text from the index, parsed
     as `syn::Item`/`ImplItem`/`TraitItem` by its `SymbolKind`; for the merge base,
     the whole blob through `raf_source::parse_source` (bounded, own stack) and a
     visitor that collects the declaration of every item at every depth (inline
     modules, impls, traits) into a multiset.
  4. A candidate whose declaration is not in the merge-base multiset of its file is
     touched: one obligation per item, subject the change record, `Part::Document`,
     evidence such as `` (`raf_okf::Document::parse` at crates/raf-okf/src/document.rs:<line>: its declaration differs from the merge base; `cap:okf-profile` binds it) ``,
     needed: "set `contract_effect: changes`, list the capability, and reconcile the
     sections it owes".
  So a body-only refactor or a rustdoc edit keeps `none`; a signature, field,
  variant, attribute (`#[non_exhaustive]`, `#[must_use]`) or visibility change does
  not. A move to another file reads as touched (stated limit).
  Condition 1 refuses on it; the headline counts it as an obligation.

Cost: about 450 lines + fixtures (touched signature, untouched body edit, doc-only
edit, moved within file, new item, removed file). Risks: one `git cat-file` per
file holding a bound public item (here about 120 files of raf-okf, raf-source,
raf-workplan, cargo-workplan), only for `none` changes under strict; a merge-base
blob that does not parse is an error (exit 2), as for documents.
Today: 0 (no open change has `contract_effect: none`).

### 3.5 Minimum content (R6)

For each loaded `delivered` capability, reading `Body::prose_lines()` without the
lines of PROPOSED and generated blocks (exactly what the tense scan reads,
`obligation/tense.rs:72-80`), words as the tense scan cuts them (runs of letters,
digits and `_`):

* `strict-section-thin`: a section with fewer than `strict::MIN_SECTION_WORDS = 20`
  words, `Location::Section(<heading>)`. Fix: "write at least 20 words of prose in
  the section saying what it covers; a section of a delivered capability is read on
  its own". The constant is part of the contract and not configurable (one number,
  closed; a repository that wants less does not opt into strict).
* `strict-usage-unbound` ("a runnable Usage example"): no `examples` binding of the
  capability substantiates Usage, i.e. no path of `bindings.examples` is named in a
  code span (or a `raf:tested-by` marker) in the Usage section. `Location::Section("Usage")`.
  Fix: "name a path of `bindings.examples` in a code span in the Usage section, so the
  verification matrix lists Usage for an example that `verify` runs". This reuses the
  substantiation rule, so the matrix shows the mapping and condition 5 proves it runs.
* `strict-limitations-missing`: the Constraints section holds no list item.
  `Location::Section("Constraints")`. Fix: "state each limitation of the capability
  as a list item of Constraints".

Extends: `strict/content.rs`; `graph::Binding::sections` for the Usage rule.
Cost: small (about 120 lines + 6 fixtures). Risk: a word count is a proxy; the
threshold is low on purpose (it catches the one-sentence section the issue names,
not style).
Today: `strict-section-thin` 1 (the Examples of `workplan-generated-docs`, one
sentence before a fence, about 6 words); `strict-usage-unbound` 12 (no Usage section
of the 12 delivered capabilities names a bound example: every matrix row of every
delivered document has the Section cell `[Verification](#verification)`);
`strict-limitations-missing` 0.

### 3.6 Reason for `dropped` and `moved` (R7)

* The field: a third optional key of a change item, `reason`, a non-empty one-line
  string, beside `claim` and `outcome`:

  ```yaml
  items:
    - claim: "The parser streams partial input."
      outcome: moved:change:incremental-parsing
      reason: "Streaming needs the buffer rework that change makes."
    - claim: "A window never allocates."
      outcome: dropped
      reason: "Recording a value allocates while the window fills; Constraints states it."
  ```

* `raf-okf`: `check_items` (`profile/mod.rs:682-743`) accepts `reason`;
  `ChangeItem` (`profile/typed.rs:154-161`, `#[non_exhaustive]`) gains
  `pub reason: Option<String>`; an empty or non-text value is a new rule
  `Rule::ItemReasonEmpty` -> `okf-item-reason-empty`, fix "give the item a non-empty
  `reason`, or remove the key"; the placeholder scan (`profile/mod.rs:754`) also reads
  `items[].reason`; the writer renders it after `outcome`. The fix text of
  `okf-item-unknown-key` (`finding.rs:473`) becomes "keep only `claim`, `outcome` and
  `reason` in an item". `reason` is accepted beside any outcome or none (an author
  may write it first); strict asks for it only where it is needed.
* Strict: an item with `dropped` or `moved:` and no `reason` -> obligation
  `outcome-without-reason` (`ObligationKind::OutcomeWithoutReason`, static, rank after
  `ItemWithoutOutcome`), subject the change record, part the item, needed: "a
  `reason` for the outcome". `Lifecycle::complete` counts it under condition 2 like
  `item-without-outcome` (`complete.rs:263-268` maps the kind to `Failure::Item`).
  It is an obligation, not a `check` finding, because items are judged by reconcile
  and completion today (an item without outcome is not a `check` finding either).
* Where the reason lives afterwards: in the record until completion; completion
  deletes the record and Git keeps it (the record's history is the account of what
  was dropped, as today).

Cost: small (about 150 lines across raf-okf and raf-workplan + fixtures). Risk:
none for default records (fixtures use `owner` and `note` as unknown item keys,
`raf-okf/tests/profile.rs:896`, `raf-workplan/tests/support/cases.rs:284`).
Today: 0 (no open record has a `dropped` or `moved` outcome).

### 3.7 Stricter obligations (R8)

`obligation::suppressed(kind, part, effect, strictness)`: under `Strict`, a
`contract_effect: none` change suppresses `section-untouched` for `contract` only;
`usage` and `examples` in `affects` must then differ from the merge base. No new
kind. `untouched::raise` (`lifecycle/untouched.rs:141-151`) passes the strictness.
Cost: tiny (about 20 lines + 2 fixtures: untouched Usage under `none` + strict
refuses, the same under default completes). Today: 0.

### 3.8 Baseline / ratchet and the census debt design (R9)

**What the census asks for** (`docs/capabilities/census-debt-ratchet.md`, designed,
open record `work/changes/census-debt-ratchet.md`): `census-debt.json` at the
workspace root with `normal_form`, `groups` (member sets, strict-subset = shrinkage)
and `job_copies` (counts); growth fails; shrinkage also fails until
`debt --write`; the write refuses growth and refuses while a non-debt finding
exists; adoption = the write creates a missing file; a `normal_form` change is
replaced by the write; the record always equals the present debt; review of the
creation diff is the control; two entry points `cargo census debt --write` and
`Gate::write_debt(&self) -> Result<DebtWrite, Error>`, never from a `#[test]`;
stable key order, trailing newline, byte-identical writes.

**What is shared, and what is a stated interface.**

* Shared, as one stated rule: everything in the list above except the census keys.
  It is written once, in a new architecture page `docs/architecture/debt-ratchet.md`
  (`arch:debt-ratchet`, a decision page with `## Decision` and
  `## Conditions to revisit`), linked from `cap:workplan-strict` and from the
  Explanation of `cap:census-debt-ratchet` (listed by the change with `affects:
  [cap:census-debt-ratchet#explanation]`; it stays `designed`; its own record stays
  open). The page states: a debt record is owned by one tool and holds only that
  tool's debt; entries are keyed; a present entry not recorded is growth; a recorded
  entry not present, or a count below its record, fails until the write; the write
  refuses growth unless the record is missing (adoption) or of another version; the
  write refuses while a finding that is not debt exists; the write takes the present
  state; the control is review.
* Shared names: `--write` on a `debt` command, `Gate::write_debt`, `DebtWrite`.
* Not shared, with reasons: the file (the two tools are adopted independently as
  dev-dependencies; a shared mutable file would make each tool parse and preserve
  the other's schema, and the census `normal_form` bump would invalidate workplan
  debt), and the code (`raf-census` is designed and not built; a shared crate
  designed against one real consumer would guess the second). The page's condition
  to revisit: "both ratchets are implemented and their compare and write code is the
  same apart from the keys".

So the workplan does not invent a second kind of ledger: it is the same ratchet over
strict findings.

**The workplan debt record.** A machine-written singleton, kept apart from the
authored coverage record as the census keeps `census-debt.json` apart from
`helpers-ledger.toml`:

```yaml
---
# SPDX-FileCopyrightText: ...
# SPDX-License-Identifier: CC-BY-4.0
type: Debt
id: debt:default
findings:
  - kind: coverage-item-unbound
    subject: "raf_okf::Actor"
  - kind: strict-claim-uncovered
    subject: "cap:okf-profile#contract/the-four-states"
lines:
  - crate: raf-okf
    orphans: 4084
---
```

* `raf-okf`: `DocType::Debt` (directory `work/debt`, prefix `debt:`, singleton id
  `debt:default` like `coverage:default`), schema `type`, `id`, optional `tags`,
  `timestamp`, required `findings` (rows of exactly `kind`, `subject`) and `lines`
  (rows of exactly `crate`, `orphans`); rows sorted, no duplicates; no body. Existing
  rules cover the rows (`okf-row-unknown-key`, `okf-row-missing-key`,
  `okf-duplicate-entry`, `okf-bad-measure`, `okf-bad-crate-name`,
  `okf-body-not-empty`, `okf-singleton-id`, whose fix text is made per type and stays
  identical for coverage).
* `raf-workplan`: `load` reads `work/debt/default.md` like the coverage record (any
  other file there is `singleton-other-file`); `debt.rs`:
  * `apply`: a present strict `Owed` whose key is recorded is not reported; a
    recorded `findings` row with no present key -> `debt-resolved`
    (`Location::Key("findings")`); a crate whose orphan lines are below its row, or at
    or under its ceiling while a row exists -> `debt-resolved` (`Key("lines")`); over
    its row -> `coverage-lines-over-ceiling` naming the row. Fix of `debt-resolved`:
    "lower the record with `cargo workplan debt --write` or `Gate::write_debt`; the
    record always equals the present debt".
  * a debt record while `strictness()` is `Default` -> `debt-without-strict`
    (`Location::Document`), fix "set `strictness: strict` in the coverage record, or
    remove `work/debt/default.md`".
  * a row whose `kind` is not a debt-eligible kind, or whose subject does not have
    the form of its kind -> `debt-entry-invalid` (`Key("findings")`).
  * eligible = every finding that strict alone raises (the strict kinds and the
    coverage kinds of packages enforced only by strict, and `coverage-lines-over-ceiling`).
    A default finding is never debt; obligations are never debt (they belong to an
    open change).
  * `write`: refuses (`DebtError::Findings`) while a default finding exists; refuses
    growth against an existing record (`DebtError::Growth` with the findings); with no
    record, creates it from the present debt (adoption); writes through
    `record::create`/`record::replace` with the SPDX lines of `RecordOptions`;
    `DebtError::NotStrict` when the record is not strict.
* Debt subject per kind: claim key; `cap:<slug>#<section>` for section rules;
  `cap:<slug>` for capability rules; the reported (exported) path for coverage gaps.

Note on the issue wording "fail only on new ones": the census design also fails on
resolved debt until the write. Coordinating means taking that semantics; the
failure has a one-command fix and keeps the record without slack (section 7).

Cost: medium (about 900 lines; the new `DocType` touches every exhaustive match on
it, the okf-profile capability and architecture page, `docs_contract`'s document
directories). Risk: `raf-okf/tests/schema.rs:209-210` pins the list of types
("Capability, Change, Architecture, Evidence, Assessment or Coverage"); a new type
changes that expected string (section 7). Fallback if that is unacceptable: a JSON
`workplan-debt.json` at the root, mirroring `census-debt.json` byte conventions,
read with `serde_json` (already in the lock file through `cargo-workplan`), REUSE
annotation in `REUSE.toml`.

### 3.9 Reverse coverage (R10, R11, R12)

**Units and lines.** For each package, every target's items as the source index lists
them (library, binaries, integration tests, examples, benches, build script). The
**counted lines** of a file are the union of the line ranges (`Item::span.lines`) of:

* every item except modules (`SymbolKind::Mod`) and `extern crate`;
* every `impl` block (header to closing brace), which needs a raf-source API
  (`Workspace::impls()`, section 6 task 5; `ImplFact` is `pub(crate)` today);
* every item-level macro invocation (`kinds! { ... }` at module level), which the
  index does not list as an item; found by the same lexing of a module that
  `bindings.rs` already does for `binding-macro-generated` (`Macros`,
  `bindings.rs:227-`), extended to keep each invocation's span.

Lines outside every unit (blank lines between items, `use`, `mod x;`, file `//!`
docs, the SPDX header) are structural: neither covered nor orphan; the report shows
them as a count so the ratio is visible. A file is counted once per package however
many targets read it (`tests/support/mod.rs` is in 30 targets); a line is covered
when any copy covers it.

**Covered.** A unit is covered when:

1. a binding resolves to it: `code` (the item), `tests` (the test function),
   `examples` (a documented item; an example target covers every unit of that
   target);
2. it is below a covered item, as `Bound::covers` reads "below" (methods and
   associated items of a covered type or trait, items of a covered module; a crate
   root covers nothing below it); an impl block is covered when its self type is
   covered (for an impl the index cannot place, when its module is);
3. it is **reachable**: a covered unit's text names a path that the source index
   resolves (`resolve_from` the unit's module) to it. The paths are the
   `Ident (:: Ident)*` runs of the unit's tokens not preceded by `.` (so method-call
   names are not paths) and every prefix of them; a path that resolves to nothing of
   the workspace (a local, a generic, std) reaches nothing; one that names several
   sites reaches each; a reached module reaches nothing below it (only items reached
   by name, and what is below them, count). Method calls are reached through rule 2:
   a reached type covers its impls and methods;
4. for a macro invocation: its module is covered, or a covered unit names a path
   that does not resolve and whose last segment is an identifier inside the
   invocation (the `binding-macro-generated` reading); a covered invocation reaches
   the `macro_rules!` it invokes.

Everything else counted is an **orphan line**. `#[cfg(test)]` modules: their items
are units like any other (the index tags `TestRole::CfgTest`, `TestFn`); a bound
unit test covers itself and the helpers it reaches; a test module whose tests are
excluded by the coverage record is excepted with that reason; the report gives
shipping and test lines in separate columns. Macro-generated items have no lines of
their own; their lines are the invocation's (rule 4). Derives add no lines. cfg is
never evaluated (`raf-source/src/cfg.rs:1-4`), so the report does not depend on the
host.

**Exceptions** (new optional key `exceptions` of the coverage record): rows of
exactly one of `item` (an item path, or `<module>::*` below a crate root, matched
against every item of the index, private ones included) or `file` (a repository-
relative path, or a directory ending in `/`), and a non-empty `reason`. Excepted
lines are reported with the first matching exception's reason and are not orphan.
An exception that matches nothing -> `coverage-exception-unmatched` (default mode,
like `coverage-exclusion-unmatched`; only records that use the new key can raise
it). Malformed rows -> `okf-bad-exception` (new rule: neither or both of `item`,
`file`; a crate root; an absolute or `..` path) and `okf-empty-reason`.

```yaml
exceptions:
  - file: "crates/raf-source/tests/support/"
    reason: "Shared fixture builders of the integration tests."
  - item: "raf_okf::profile::generated::*"
    reason: "Tables generated from the profile schema."
```

**Ceilings** (new optional key `ceilings`): rows of `crate` (a package name) and
exactly one of `lines` (a non-negative integer) or `percent` (a decimal, compared
exactly on its text as evidence limits are, against `orphan * 100 / counted`).
Under strict every enforced package (`scope` + bound packages + every package a row
names) is judged; a package without a row has the ceiling `lines: 0`. Over it ->
`coverage-lines-over-ceiling` at `Key("ceilings")`, message "`raf-okf` has 4084
orphan lines (587 shipping, 3497 test) of 16175 counted, over its ceiling of 0; the
largest orphan items are ...", fix "bind or reach the items that explain them, add an
exception with its reason, or raise the crate's ceiling". A row naming no package ->
`scope-crate-missing` at `Key("ceilings")` (the existing kind for a package name of
the record that is no package). Malformed rows -> `okf-bad-ceiling` and
`okf-bad-measure`. Without strict, ceilings are not judged and the report still
shows them.

**Report.** `Workplan::lines_report() -> LinesReport` (in a new
`coverage/lines.rs`, rendered by `coverage::render`):

```text
$ cargo workplan coverage lines --crate raf-okf
| Crate | Enforced | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| raf-okf | yes | 16175 | 12091 | 0 | 4084 | 587 | 3497 | 0 lines | 4084 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |
| raf-okf | `raf_okf::profile::violation` | 1152 | 246 |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
| raf-okf | `raf_okf::profile::violation::Rule::fmt` | fn | crates/raf-okf/src/profile/violation.rs | 340-412 |
```

Sorted by package, module path, file, first line; each orphan line is listed once,
under the innermost counted unit that holds it; ranges are merged `a-b`. JSON kinds
`lines-crate`, `lines-module`, `lines-orphan` (`ranges` as `[[a,b],...]`),
`lines-excepted` (with `exception`, `reason`), `lines-error` (a surface the index
refused). Exit 1 when an orphan line is printed, as `coverage code` exits on a gap.

**Why `coverage lines` and not `coverage --reverse`.** The command is
`coverage <direction>` with `code` and `tests`; a flag that changes the direction
would be a second grammar for one choice, and "reverse" of `code` is ambiguous (the
`code` direction already reads code to documentation). `lines` names what is
counted.

**Work.** Counted, never timed (`work.rs` tally): units judged, paths extracted,
resolutions asked, units reached. Tests in the style of
`coverage::tests::the_work_grows_with_the_entries_and_no_faster`: linear in
units + paths, each unit's text tokenized once, each path resolved at most once per
module (memoized).

Extends: `coverage.rs` (`Bound`, `ancestors`), `bindings.rs` (`Macros`),
`raf-source` (section 6 task 5), `raf-okf` (two keys), `cargo-workplan`.
Cost: the largest code task (about 1400 lines with fixtures).
Risks: (1) reachability by path is an approximation (a function stored in a field
and called through it, trait-object dispatch to an unbound type, and items used only
from macros are not reached); stated in Constraints, and exceptions take the rest;
(2) `raf-source` cannot resolve from a module of a non-library target today:
`resolve_from("raf_workplan::test_crate::scenarios", "refused")` and
`resolve_from("cargo_workplan::bin_crate::cargo_workplan::cli", "slug")` both give
`UnknownModule` (probed in `/tmp/raf16-proto`; `resolve.rs:1384-1392`
`module_named`), so reachability inside tests and binaries needs the raf-source task.

**Fixtures (R12).** A crate whose public surface is bound and whose private helpers
are reached reports 0 orphan lines; an unreferenced private helper reports exactly
its span (attributes and docs included) as one range, beside the same helper
referenced by a bound function (0); binding a type covers its inherent and trait
impl blocks and their methods, and binding a method covers nothing beside it; a
`macro_rules!` that generates a function, invoked at module level, is orphan with
the invocation's range when nothing names the generated function and covered when a
bound function calls it; a `#[cfg(test)] mod tests` with one bound and one unbound
test reports the unbound one and its private helper, and an exception
`<crate>::tests::*` excepts them with the reason; the report of a `MemoryTree` with
CRLF files equals that of LF files and two runs are byte-identical; a ceiling of
`lines: N` passes at N and fails at N+1; a percent ceiling at the boundary.

### 3.10 The new kinds

| Kind | List | Mode | Raised by |
| --- | --- | --- | --- |
| `strict-example-missing` | `Finding::KINDS` | strict | 3.1 |
| `strict-matrix-missing` | `Finding::KINDS` | strict | 3.1 |
| `strict-claim-uncovered` | `Finding::KINDS` | strict | 3.2 |
| `claim-test-unbound` | `Finding::KINDS` | default (new marker only) | 3.2 |
| `strict-section-thin` | `Finding::KINDS` | strict | 3.5 |
| `strict-usage-unbound` | `Finding::KINDS` | strict | 3.5 |
| `strict-limitations-missing` | `Finding::KINDS` | strict | 3.5 |
| `coverage-exception-unmatched` | `Finding::KINDS` | default (new key only) | 3.9 |
| `coverage-lines-over-ceiling` | `Finding::KINDS` | strict | 3.9 |
| `debt-resolved` | `Finding::KINDS` | strict | 3.8 |
| `debt-without-strict` | `Finding::KINDS` | default (new record only) | 3.8 |
| `debt-entry-invalid` | `Finding::KINDS` | strict | 3.8 |
| `okf-bad-strictness`, `okf-item-reason-empty`, `okf-tested-by-outside-capability`, `okf-bad-exception`, `okf-bad-ceiling` | `okf-<rule>` (new `raf_okf::Rule`s) | profile | 2.2, 3.6, 3.2, 3.9 |
| `okf-parse-malformed-tested-by`, `okf-parse-unattached-tested-by` | `okf-parse-<error>` (new `ErrorKind`s) | reader | 3.2 |
| `outcome-without-reason` | `ObligationKind` | strict | 3.6 |
| `contract-effect-understated` | `ObligationKind` | strict, needs git | 3.4 |

`OWN_KINDS` in `tests/kinds.rs` grows from 48 to 60. Each kind gets a named `Case`;
`Case` (`cases.rs:16-36`) gains `strict: bool` and a builder `strict()`, and `run()`
judges with `Workplan::with_strictness(Strict)`; the strict cases break one rule of a
new strict-clean base fixture (a delivered capability with every claim marked, a
Usage that names its bound doc test, a matrix, sections of 20+ words, its crate fully
bound, ceilings met). The obligation kinds join the cases of
`the_cases_cover_every_kind_of_obligation_in_both_directions` and
`each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes`.
JSON non-rule kinds added (`coverage-claim`, `lines-*`, `debt-written`) join the list
of `cargo-workplan/tests/json.rs:263`.

## 4. Reconciliation with the delivered two-way coverage and evidence work

| Fact | Delivered (default) | Strict adds | Duplicate? |
| --- | --- | --- | --- |
| A public item is explained | `coverage-item-unbound` for `scope` | the same kind for bound packages | No: same rule, larger set |
| A test supports a guarantee | `coverage-test-unbound`: some capability binds it (`scope`) | the same kind for bound packages | No |
| A guarantee has a test | nothing (bindings resolve; matrix lists rows by section) | claim unit, `raf:tested-by`, `strict-claim-uncovered`, `claim-test-unbound`, `coverage claims` | No: the opposite direction of the tests-to-claims edge, at claim grain |
| A statement rests on an execution | `raf:claim evidence=`, `claim-unsupported`, `evidence-*` | nothing; an evidence marker is not test coverage | No |
| A section names its bindings | matrix Section cell from code spans | the cell also from `raf:tested-by` | Extension of one rule |
| Lines are explained | nothing | `coverage lines`, ceilings, exceptions | No; reuses `Bound::covers` and the exclusions' reasons |

The direction table of `cap:workplan-two-way-coverage` gains no row; strict is a
separate capability that depends on it. The two-way capability's Contract gets one
sentence (under a PROPOSED block of this change) that strict widens enforcement to
bound packages, and its "What `check` enforces" names the strict reason in the
message.

## 5. The seven NEEDS-JUDGMENT items

1. **R19 vs R2/R3 (reconcile with #10).** Resolved by section 4: no kind of the
   delivered work is re-introduced; R4 reuses the coverage kinds over a larger set;
   R3 is the docs-to-tests direction at claim grain, which #10 does not have (#10's
   tests-to-claims is "bound by a capability"); evidence claims are untouched and do
   not count as test coverage, because "a test exists that shows it" and "a recorded
   run shows it" are different facts with different invalidation.
2. **R1 location.** The coverage record, key `strictness` (section 2.1), justified
   in the capability's Explanation as a decision with conditions to revisit ("a
   second repository-level setting that is not about coverage appears", "strict
   splits into independently chosen rules").
3. **R7 field.** `reason`, an optional non-empty string of a change item beside
   `claim` and `outcome`, validated by the profile in default mode, required by strict
   for `dropped`/`moved` as `outcome-without-reason` under condition 2 (section 3.6).
4. **R9 vs #4.** #4's design exists as the designed capability
   `cap:census-debt-ratchet` with an open change record; it is binding as design
   (semantics and names), not as code (nothing is built). Shared: the rule, stated
   once in `arch:debt-ratchet`, and the entry-point names. Stated interface: each
   tool owns one debt record of keyed entries; neither reads the other's. Not shared:
   the file and the code (section 3.8).
5. **R5 vs R14.** R5 applies only under strict. Its first two parts already hold in
   default mode (conditions 5 and 1), so only `contract-effect-understated` is new,
   and it is strict only; default behaviour is unchanged.
6. **R10/R11 threshold.** Under strict every enforced package has the ceiling
   `lines: 0` unless the record names another (`lines` or `percent`); a repository
   adopting strict records the present count as debt. Without strict the report is
   available and enforces nothing. Zero is the only default that needs no magic
   number and matches "strict".
7. **R18 vs R14.** In scope, and done by data, not by changing a default: the
   repository sets `strictness: strict` in its coverage record, so
   `tests/self_workplan.rs` and `tests/docs_contract.rs` judge it strict with no code
   change; the default stays as it is for every other repository and fixture. The
   change fixes what is cheap and records the rest as debt (section 6.3). If the
   maintainer reads "once the rules pass" as "with an empty debt record", the cost is
   in section 6.3 and needs its own change record for the documentation work.

(The prior-art ledger's own four questions: no file beyond the two named needs
creating or porting; no licence question arises; the `none` defect-class tallies say
nothing about this issue; #3 and #9 do not constrain it, and #4 is answered in item 4.)

## 6. Plan

### 6.1 Tasks, in dependency order

Every task ends with `make check` green and the hooks installed; every new public
item and test of `raf-workplan`/`cargo-workplan` is bound (default coverage of their
`scope`), and every edited section with an assessment is assessed again
(`self_workplan`). Every refusal is run beside its passing neighbour (AGENTS 5).

**T1. Open the change and write the design as documents.** (docs only)
* `cargo workplan change new workplan-strict --capability cap:workplan-strict ...`
  (record sketch in 6.2); new `docs/capabilities/workplan-strict.md`, state
  `designed`, empty bindings (sketch in 6.2); new `docs/architecture/debt-ratchet.md`
  (decision page); PROPOSED blocks of `change:workplan-strict` in the delivered
  capabilities it changes, each set to `implementing` in the same commit:
  `workplan-two-way-coverage` (contract, constraints), `workplan-complete-gate`
  (contract), `workplan-reconcile` (contract: two obligation kinds, the strict
  suppression), `workplan-cli` (contract, usage: `--strict`, `coverage claims|lines`,
  `debt --write`), `okf-profile` (contract: `strictness`, `reason`, `ceilings`,
  `exceptions`, `Debt`, the tested-by marker), `workplan-generated-docs` (contract:
  substantiation), `workplan-document-model` (contract: the `work/debt` directory),
  `source-index` (contract: target-scoped resolution and impl blocks); the Explanation
  of `census-debt-ratchet` links the decision page; `docs/architecture/okf-profile.md`
  claim-marker section; `INVENTORY` rows; `docs sync`.
* Check: `cargo test --test docs_contract --test self_workplan --test repo_gates`
  (no tracker number or process wording, `issue-reference` clean), `cargo workplan
  change reconcile workplan-strict` lists only the expected owed items.

**T2. The setting, the plumbing, and the document rules (R1, R2, R4, R6).**
* raf-okf: `Strictness`, the `strictness` key, `okf-bad-strictness`.
* raf-workplan: `Gate::strict`/`strictness`, `Opened` floor, `Workplan::strictness`/
  `with_strictness`, `strict.rs` + `strict/content.rs`, the six kinds of 3.1/3.5,
  the strict enforcement set in `coverage::check` (3.3), `Case::strict()` and the
  strict-clean base fixture.
* cargo-workplan: `--strict`.
* Check: per rule, the case that triggers it and its neighbour (`strict-example-missing`
  vs one example; `strict-matrix-missing` vs the block; `strict-section-thin` at 19
  vs 20 words; `strict-usage-unbound` vs a code span of the bound doc test;
  `strict-limitations-missing` vs one list item; a bound package outside `scope` with
  an unbound public item vs bound; the same package unbound by every capability is not
  enforced); default unchanged: `all_cases()` as today, and the strict-violating
  fixture has no finding without strict; `okf-bad-strictness` vs `strict`/`default`;
  the record's `strict` beats a default floor and a strict floor beats a default
  record; `Opened::workplan_after` keeps the floor; CLI (`cargo-workplan/tests/check.rs`):
  `--strict check` exits 0 on the strict-clean fixture, 1 with one thin section (and
  0 without `--strict`), 2 with `--root` at a directory without `docs/capabilities`;
  JSON objects of strict findings carry the documented fields.

**T3. Claim coverage (R3).**
* raf-okf: the `raf:tested-by` family, its two `ErrorKind`s, `Body::tested_by`,
  `okf-tested-by-outside-capability`; raf-workplan: `TestedByEdge`, the
  substantiation extension, `claim-test-unbound`, `strict/claims.rs` (units, keys,
  `strict-claim-uncovered`), `Workplan::claims_report`, `coverage::render` for it;
  cargo-workplan `coverage claims`.
* Check: marker fixtures (well-formed, after a claim marker, after a statement;
  malformed: space after comma, repeat, empty, `tests=` form, inside a fence;
  unattached: after a blank line, after another comment, before a claim marker), the
  existing `claims.rs` fixtures unchanged; unit fixtures (bold lead, `###` heading,
  opening, Constraints item/paragraph/table, repeated lead keys, a heading inside a
  fence is not a clause); a Contract clause with a marker on its second paragraph is
  covered, its neighbour clause is not; `claim-test-unbound` vs a bound path; the
  matrix row of a marked test lists Contract and an unmarked one stays Verification,
  and every existing matrix is byte-identical (the generated fixtures pass as they
  are); report table and JSON equal, sorted, two runs identical; CLI exit 0/1/2.

**T4. The change lifecycle under strict (R5, R7, R8).**
* raf-okf `reason`; `okf-item-reason-empty`; obligation kinds
  `outcome-without-reason` (condition 2) and `contract-effect-understated`
  (`lifecycle/touched.rs`, condition 1); `suppressed(.., strictness)`.
* Check: in git fixture repositories (as `tests/complete.rs` builds them): a `none`
  change that edits a bound function's signature refuses under strict and completes
  under default; a body-only edit and a doc-comment edit complete under strict; a new
  bound public item refuses; a moved-within-file item completes; `dropped` without
  `reason` refuses condition 2 under strict, with `reason` completes, and completes
  without it under default; an untouched Usage in `affects` of a `none` change refuses
  under strict and completes under default; the headline counts; each refusal leaves
  the tree byte-identical; `change reconcile --strict` exits 1/0, and 2 for a merge
  base blob that does not parse.

**T5. raf-source: resolution from every target, impl blocks.** (source-index)
* `resolve_from` (and `_in`) accept the module paths of every target
  (`<pkg>::test_crate::<t>::..`, `::bin_crate::`, `::example_crate::`), today refused
  by `module_named` (`resolve.rs:1384-1392`); `Workspace::impls()` lists impl blocks
  with span, module, self type key (from `ImplFact`).
* Check: a helper resolved from an integration test root, from a nested module of a
  binary, and from a support module included by two test targets (one site per
  target, `same_source` relates them); the existing resolution fixtures unchanged
  (`use_resolution.rs`, `rustc_oracle.rs`); `impls()` order and spans on the shapes
  fixtures; budgets counted (no timing).

**T6. Reverse coverage (R10, R11, R12).** (depends on T2, T5)
* `coverage/lines.rs`, `Workplan::lines_report`, render text and JSON, `ceilings`
  and `exceptions` keys (raf-okf: `okf-bad-ceiling`, `okf-bad-exception`),
  `coverage-exception-unmatched`, `coverage-lines-over-ceiling`, cargo-workplan
  `coverage lines`.
* Check: the R12 fixtures of 3.9; exceptions by item, prefix, file, directory, each
  beside an unmatched one; ceiling boundaries; strict off: report printed, nothing
  enforced; work tallies linear in units and paths at fixed exceptions and the other
  way round, with a ceiling linear in their sum; CLI exit 0 (no orphan), 1, 2
  (unknown `--crate`).

**T7. The debt record and the ratchet (R9).** (depends on T2, T3, T6)
* raf-okf `DocType::Debt` and its schema; raf-workplan load, `debt.rs`
  (`apply`, `write`, `DebtWrite`, `DebtError`), `Gate::write_debt`, the three debt
  kinds; cargo-workplan `debt --write`; `docs/architecture/debt-ratchet.md` final.
* Check: the census fixture set mapped one to one: growth (a new strict finding, a
  rising orphan count) fails and `--write` refuses and writes nothing; shrinkage (a
  resolved entry, a falling count, a count back under the ceiling) fails with
  `debt-resolved` naming the write, and passes after it, with the file equal to the
  present debt; no record: every strict finding stands and the write creates the
  record; a default finding makes the write refuse; `debt-without-strict` vs strict;
  `debt-entry-invalid` for a default kind and a malformed subject; two writes
  byte-identical; the write writes only `work/debt/default.md`; `debt --write` exit
  0, 1 (growth), 2 (not strict).

**T8. Adopt strict in this repository and deliver.** (depends on all)
* Make `cap:workplan-strict` itself strict-clean (every claim marked, Usage names a
  bound doc test, a matrix). Fix the 12 `strict-usage-unbound` (one sentence per
  Usage naming a bound example in a code span) and the 1 `strict-section-thin`.
  Set `strictness: strict` in `work/coverage/default.md`, add the exceptions that
  are true (shared test support), then `cargo workplan debt --write --copyright ...
  --licence CC-BY-4.0` (a `make workplan-debt` target beside `docs-sync` passes the
  SPDX values) to create `work/debt/default.md`; set the listed capabilities
  `delivered`, `docs sync`, assess the affected sections, update `INVENTORY`,
  `cargo workplan change complete workplan-strict`.
* Check: `make check`; `self_workplan` now judges strict; reviewing the debt diff is
  the adoption control; `cargo workplan --strict check` exits 0 and
  `cargo workplan coverage lines` lists the recorded counts.

### 6.2 The capability document and the change record

`docs/capabilities/workplan-strict.md` (designed; bindings fill as tasks land):

```yaml
---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-strict
title: "Strict mode"
description: "An opt-in set of harder rules: examples, claim coverage, explained public items and lines, fuller sections, stated reasons, and a debt record that can only shrink."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-two-way-coverage, cap:workplan-complete-gate, cap:workplan-reconcile, cap:workplan-evidence-claims, cap:workplan-generated-docs, cap:source-index]
---
```

Sections: Purpose (the default accepts a delivered capability with one code path and
one test; strict turns the expectations of a well-documented capability into
findings, for repositories that ask for it). Contract, as bold-lead clauses, one per
rule, each to carry its `raf:tested-by` marker once bound: **The setting.**
**Examples and the matrix.** **Claims.** **Explained items.** **Completion.**
**Sections.** **Reasons.** **Untouched sections.** **Lines.** **Ceilings and
exceptions.** **Debt.** **The kinds.** Usage (`strictness: strict`, `--strict`,
`Gate::workspace(..).strict().assert_clean()`, `coverage claims`, `coverage lines`,
`debt --write`, naming the bound doc test of `raf_workplan::Gate::strict`). Examples
(a refusal of each kind, the coverage-lines table, an adoption diff). Constraints
(reachability by path only; a claim's test is cited, not judged; a moved item reads
as touched; Constraints items reworded are new claims; the 20-word constant; the
ceiling default 0; strict cannot be lowered by a caller). Explanation (why opt-in;
`### Decision` the setting lives in the coverage record + `### Conditions to
revisit`; `### Decision` the debt follows the ratchet of
[the debt ratchet](../architecture/debt-ratchet.md) + conditions). Verification (the
fixture list of 6.1 and the matrix block).

`work/changes/workplan-strict.md`:

```yaml
---
# SPDX lines
type: Change
id: change:workplan-strict
title: "Strict mode"
description: "Add an opt-in strict mode to the workplan gate and adopt it in this repository."
changes: [cap:workplan-strict, cap:workplan-two-way-coverage, cap:workplan-complete-gate, cap:workplan-reconcile, cap:workplan-cli, cap:okf-profile, cap:workplan-generated-docs, cap:workplan-document-model, cap:source-index, cap:census-debt-ratchet]
affects: [cap:workplan-strict#purpose, cap:workplan-strict#contract, cap:workplan-strict#usage, cap:workplan-strict#examples, cap:workplan-strict#constraints, cap:workplan-strict#explanation, cap:workplan-strict#verification, cap:workplan-two-way-coverage#contract, cap:workplan-two-way-coverage#constraints, cap:workplan-complete-gate#contract, cap:workplan-reconcile#contract, cap:workplan-cli#contract, cap:workplan-cli#usage, cap:okf-profile#contract, cap:workplan-generated-docs#contract, cap:workplan-document-model#contract, cap:source-index#contract, cap:census-debt-ratchet#explanation]
contract_effect: changes
items:
  - claim: "A coverage record with strictness: strict, a Gate::strict() caller or --strict judges the repository under strict mode, and nothing lowers it."
  - claim: "A delivered capability binds an example and holds a verification matrix."
  - claim: "Every Contract clause and Constraints item of a delivered capability names a bound test or example in a raf:tested-by marker."
  - claim: "Every public item and test of a package that a capability binds is bound or excluded."
  - claim: "A contract_effect: none change that alters the declaration of a bound public item does not complete."
  - claim: "Every section of a delivered capability holds 20 words, Usage names a bound example, Constraints lists a limitation."
  - claim: "A dropped or moved item states its reason."
  - claim: "Usage and Examples in affects of a contract_effect: none change must be edited."
  - claim: "coverage lines reports orphan lines per crate, module and item, and strict holds each enforced crate to its ceiling."
  - claim: "The strict debt in work/debt/default.md can only shrink and always equals the present debt."
  - claim: "This repository runs under strict mode."
---

## Summary

...
```

(Item claims are the product's guarantees, not process steps; no tracker number.
Each item ends `implemented` before completion; a scope that leaves the change is a
`moved:change:<slug>` to a record that exists, with its `reason`.)

The `INVENTORY` of `tests/docs_contract.rs` gains `("workplan-strict", "designed")`
in T1, the listed delivered capabilities move to `implementing` with their PROPOSED
blocks in T1 and back to `delivered` in T8 together with `workplan-strict`.

### 6.3 Can this repository enable strict in this change, and at what cost

Strict rules applied to the tree at `95c384c` (scratch survey of the 12 delivered
documents, the `coverage code|tests` JSON reports, and the reverse-coverage
prototype):

| Rule | Findings today | Fix cost | In T8 |
| --- | --- | --- | --- |
| `strict-example-missing` | 0 | - | - |
| `strict-matrix-missing` | 0 | - | - |
| `strict-claim-uncovered` | 196 (95 clauses, 101 Constraints units) | a judged marker per unit, choosing among 1,565 bound tests; every Contract and Constraints section re-assessed | debt |
| orphan symbols (`coverage-item-unbound`) | 211 (raf-okf 177, raf-source 34) | bind or exclude each | debt |
| orphan symbols (`coverage-test-unbound`) | 379 (raf-okf 148, raf-source 231) | bind or exclude each | debt |
| `contract-effect-understated` | 0 (no `none` change open) | - | - |
| `strict-section-thin` | 1 (`workplan-generated-docs` Examples) | one or two sentences | fix |
| `strict-usage-unbound` | 12 (all delivered) | one sentence per Usage | fix |
| `strict-limitations-missing` | 0 | - | - |
| `outcome-without-reason` | 0 (no outcome set in the 9 open census records) | - | - |
| strict `section-untouched` | 0 | - | - |
| `coverage-lines-over-ceiling` (ceiling 0) | 4 crates, approx.: raf-source 6,084 (1,140 shipping, 4,944 test), raf-okf 4,084 (587, 3,497), raf-workplan 168 (138, 30), cargo-workplan 26 (2, 24) | bindings, exceptions (shared test support), or ceilings | exceptions where true, rest debt |
| not enforced (not bound) | root package 2,848 test lines, `raf-source-fuzz` 77 | - | - |

So T8 fixes 13 findings and records about 786 finding rows (196 + 211 + 379) and 4
line rows; the debt record is about 2,400 lines (about 120 KiB, inside the 1 MiB
document bound). Paying all of it in this change instead would mean 196 judged
markers across 12 documents (and re-assessing 24 sections), about 590 bindings or
exclusions for two crates, and bindings or exceptions for about 10,000 orphan lines:
a documentation change larger than the feature, which is the case the baseline
exists for.

The prototype's numbers are estimates: it resolves paths with `resolve_from` for
library modules and, because raf-source refuses other targets' modules today, by a
same-target suffix match for test and binary modules; it counts no impl headers and
no macro invocations. The real counts come from T6.

## 7. Conflicts with the repository rules, and underspecified points

1. **Tracker numbers and process wording.** The issue names #4, #8 and #10 and says
   "baseline", "adopting", "scope". Committed files (`workplan-strict.md`,
   `debt-ratchet.md`, the change record, code comments, test names) refer to
   `cap:census-debt-ratchet`, `cap:workplan-two-way-coverage`,
   `cap:workplan-evidence-claims` by slug, never to a number, and describe the system
   ("a repository that opts in records its present strict findings as debt"), never
   the work; `make issue-refs` and `issue-reference` (`refs.rs:51`: "this issue",
   "per the plan", "this branch", `#N`, `/issues/`, `/pull/`) gate it.
2. **"Fail only on new ones" vs the census ratchet.** Coordinating with the census
   design means resolved debt also fails until the write (no slack). Adopted; the
   message names the one command. This is a deliberate reading of the issue in favour
   of its own "instead of inventing a second ledger".
3. **"Default behaviour and every existing fixture are unchanged."** Every existing
   case keeps its verdict, and no existing document changes output. Text that does
   change by contract: the fix of `okf-item-unknown-key` (lists `reason`); the
   expected type list of `raf-okf/tests/schema.rs:209-210` (adds `Debt`) if the debt
   record is an OKF type, otherwise the JSON fallback of 3.8 avoids it. New
   default-mode findings arise only from new syntax or keys
   (`claim-test-unbound`, `coverage-exception-unmatched`, `debt-without-strict`, the
   new profile rules). Decision needed from the maintainer only on the type-list pin.
4. **Claim keys of Constraints items are text digests.** Rewording an uncovered item
   recorded as debt makes it a new claim (growth): touching a claim makes it owe a
   test. Stated in Constraints; bold-lead keys survive body edits.
5. **`coverage --reverse`** conflicts with the `coverage <direction>` grammar;
   resolved as `coverage lines` (3.9).
6. **"Every public item of a bound crate path"** is underspecified; resolved as the
   packages a binding of any kind resolves into, enforced in both directions (3.3).
7. **"Non-trivial length"** is underspecified; resolved as a closed constant of 20
   prose words per section (3.5). "A runnable Usage example" is resolved as a
   binding the Usage names, so runnability is proven by the executor and not by
   compiling prose code blocks (the Scenario B detector compiles a fence; the gate
   never runs document text, a stated constraint of the completion gate).
8. **"Reachable through the source index"** is underspecified; resolved as resolved
   path references plus "below" (3.9), with the stated limits, and it needs the
   raf-source extension (T5) because non-library modules are not resolvable today.
9. **"Verify ALL bound items" and "zero tense-mismatch"** already hold (3.4); writing
   them as new rules would duplicate conditions 5 and 1. The capability states them
   as guarantees it relies on.
10. **"Closed lists" and the debt.** Debt eligibility is closed by construction
    (only what strict alone raises); a row of any other kind is
    `debt-entry-invalid`, so the debt can never hide a default finding.
11. **"Fail hard" and the floor.** A caller can raise strictness and never lower it;
    a record that does not load counts as default for strictness and fails on its own
    findings, so strict is never silently dropped by a broken record: the broken
    record itself fails the gate.
12. **Exit codes.** `debt --write` refusing growth is exit 1 (findings), not 2;
    `debt --write` without `strictness: strict` is 2 (the tool cannot do what was
    asked). The census CLI exits 2 only for the registered-code case, which the
    workplan does not have.
13. **No timing.** Nothing here measures time; the scale fixtures of reverse
    coverage count work with the `work.rs` tally, as the coverage index tests do.
14. **Platforms.** "Identical across platforms" is proven on Linux with CRLF and LF
    `MemoryTree` fixtures and `/` paths; cfg is not evaluated, so the host does not
    enter the report. Other systems are minimal-effort per the repository rule.
