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

# Strict mode

## Purpose

The default rules of the workplan accept a `delivered` capability that binds one code
path and one test, whatever its examples, the length of its sections or the share of
its guarantees that a test shows. That keeps the gate light for a repository that is
starting to document itself. Strict mode is for a repository that asks for more: it
turns the expectations of a well-documented capability into findings, so that every
delivered capability shows a runnable example, every guarantee names a test, every
public item and every line of code is explained by some capability, and the
completion of a change cannot understate what it touched. A repository opts in once,
in its coverage record, and adopts the mode with a recorded debt that can only
shrink.

## Contract

**The setting.** Strictness is a closed value, `default` or `strict`, ordered by
strength: `raf_okf::Strictness`, re-exported as `raf_workplan::Strictness`. The
coverage record `work/coverage/default.md` takes an optional key `strictness`; a
record without the key says `default`. A caller sets a floor with `Gate::strict()`
(the same as `Gate::strictness(Strictness::Strict)`), with
`Workplan::with_strictness(floor)`, or with the global `--strict` option of
`cargo workplan`. The repository is judged under the stronger of the record's value
and the floor, which `Workplan::strictness` returns: a caller can raise the
strictness and nothing lowers it, so a repository whose coverage record says
`strictness: strict` is judged strict without a flag. A coverage record that does
not load counts as `default`, and its own findings fail the check. `Opened::workplan`
and `Opened::workplan_after` keep the floor of their `Gate`, so `change reconcile`,
the static conditions of `change complete` and its check of the repository once the
record is removed are judged under the strictness of `check`. Under `strict`,
`Workplan::check` appends the strict findings after every finding of the default
rules and then applies the debt record. Under `default`, `check` judges no strict
unit and counts no line, so the default stays the lightweight mode it is without this
capability.

**Examples and the matrix.** Under strict, a `delivered` capability binds at least
one entry in `bindings.examples`; with none it is `strict-example-missing` at the key
`bindings`. It holds a `verification-matrix` generated block in its Verification
section; without one it is `strict-matrix-missing` at that section. That the bound
example runs is proven where the default rules prove it, by `verify` and condition 5
of the [completion gate](./workplan-complete-gate.md), which refuse a binding whose
only documentation test is `no_run` or `compile_fail`; strict mode adds no second
run.

**Claims.** A claim is a unit of the authored prose of the Contract and the
Constraints of a `delivered` capability, read outside fenced code, generated blocks
and PROPOSED blocks:

- in the Contract, a clause: it starts at a paragraph that begins with a bold run
  (`**The setting.**`) or at a `###` or `####` heading, and runs to the start of the
  next clause or the end of the section. Text before the first clause start is one
  clause, the opening. The statement of a heading clause is its heading line;
- in the Constraints, each top-level list item, and each paragraph or table outside
  a list.

Each claim has a key, `cap:<slug>#<section>/<key>`: the kebab slug of the bold lead
or the heading, `opening` for the opening, and for a Constraints unit `c-` and the
first 16 hexadecimal digits of the SHA-256 digest of its text with each run of
whitespace read as one space. A key that repeats in a section takes `-2`, `-3` and on
in document order.

A test maps to a claim through a marker on its own line directly after the statement,
or directly after the statement's claim marker:
`<!-- raf:tested-by <path>[,<path>...] -->`, with one space after the word and the
entries separated by `,` and no space. Each entry matches the `<item-path>` pattern of
the completion gate, and no entry repeats. The marker is a third family beside the
generated and the claim markers: [the OKF reader](./okf-profile.md) reads it as
`Body::tested_by`, a malformed one is `okf-parse-malformed-tested-by`, one with no
statement directly above it is `okf-parse-unattached-tested-by`, and one outside a
Capability document is `okf-tested-by-outside-capability`. Under either strictness,
an entry that is no path of the same capability's `bindings.tests` or
`bindings.examples` is `claim-test-unbound` at the marker's line. A Contract clause is
covered when it holds a marker anywhere in it; a heading clause is covered by a marker
directly after its heading. A Constraints unit is covered by a marker attached to it.
An evidence claim marker covers nothing here.

Under strict, each claim that no marker covers is `strict-claim-uncovered` at the
first line of the claim, with a message that names its key and the first 60
characters of its lead or item. A Contract with no clause start is one claim; when it
holds more than `strict::MIN_SECTION_WORDS` words it is `strict-claim-unit-collapsed`
at the Contract, whether or not a marker covers it, so one marker cannot stand for a
long Contract. `cargo workplan coverage claims [--capability cap:<slug>]` and
`Workplan::claims_report` list every claim of every delivered capability with its
key, line and tests, under either strictness, as a table with the columns Capability,
Section, Claim, Line and Tested by, and in JSON as objects of the kind
`coverage-claim` with the fields `capability`, `section`, `claim`, `line` and `tests`.
The command exits 1 when a claim of a delivered capability has no test, 0 when every
claim has one, and 2 when `--capability` names no loaded capability. A binding that
a marker names substantiates the section that holds the marker in the verification
matrix, as a code span of its path does ([generated documents](./workplan-generated-docs.md)).

**Explained items.** Under strict, the code and the tests directions of
[two-way coverage](./workplan-two-way-coverage.md) are enforced for every package
that holds an item that a `code`, `tests` or `examples` binding of a loaded capability
resolves to, beside the packages of `scope`. A gap of such a package has the kind the
default rule gives it, `coverage-item-unbound`, `coverage-reexport-unbound`,
`coverage-test-unbound` or `coverage-surface-unreadable`, and its message says which
capability binds which path of the package, the reason strict mode enforces it. A
package that no binding reaches and `scope` does not name is not enforced by this
rule. The strict rules compute this coverage themselves; the default coverage check
is the same under either strictness.

**Completion.** Strict mode keeps the six conditions of the completion gate, two of
which already hold under either strictness: condition 5 runs every test and example
binding of every listed capability with the proof `verify` uses, and condition 1
refuses on each `tense-mismatch` of a listed capability. Strict adds the process
obligation `contract-effect-understated`, raised by
[reconcile](./workplan-reconcile.md) when the record says `contract_effect: none`
and a bound public item differs from the merge base. A bound public item is an item of
the public surface that a `code` binding of any loaded capability covers, itself or
as an item below it in the sense of two-way coverage. Each file that holds one is read
at the merge base with the reconcile row `git cat-file blob <merge-base>:<prefix><path>`,
within the 1 MiB bound of a blob, and each side is parsed from its whole file through
`raf_source::parse_source`. Items are compared as `proc_macro2` token streams of
their whole span: a doc comment arrives as a `#[doc]` attribute and counts, while
whitespace and comments that are not doc comments are not tokens and do not. An
item is touched when its token stream is not among those of the items of its file at
the merge base, so an edit of a signature, a body, an attribute, a visibility or a
doc comment touches it, a reformatting or an edit of a plain comment does not, a new
bound public item and an item moved to another file are touched, and an item moved
within its file is not. A bound public item that the merge base has and the working
tree lacks, removed or renamed, is touched too, and a file absent at the merge base
makes every candidate in it touched. Each touched item is one obligation on the
change record, whose evidence names the item, its file and line, and the capability
that binds it; the evidence needed is `contract_effect: changes`, the capability
listed and the sections it owes reconciled. A file at the merge base that does not
parse is an error and not a pass. Condition 1 refuses on the obligation.

**Sections.** Under strict, each of the seven sections of a `delivered` capability
holds at least `strict::MIN_SECTION_WORDS`, 20, words of prose: the prose lines
outside PROPOSED and generated blocks, cut into words as the tense scan cuts them,
runs of letters, digits and `_`. A shorter section is `strict-section-thin` at the
section, and its fix says that a generated block does not count as prose. The Usage
section names a path of `bindings.examples` in a code span or in a tested-by marker,
so the verification matrix lists Usage for an example that `verify` runs; otherwise
it is `strict-usage-unbound` at Usage. The Constraints section holds at least one
list item, one limitation; otherwise it is `strict-limitations-missing` at
Constraints.

**Reasons.** A change item takes an optional key `reason` after `outcome`, a
non-empty line of text, accepted beside any outcome or none; an empty one is
`okf-item-reason-empty`. Under strict, an item whose outcome is `dropped` or
`moved:change:<slug>` and that has no `reason` is the obligation
`outcome-without-reason` on that item, and condition 2 of the completion gate refuses
on it as it refuses on `item-without-outcome`.

**Untouched sections.** Under strict, `contract_effect: none` leaves out
`section-untouched` for the Contract only: a Usage or an Examples section that the
record lists in `affects` and that has the text of the merge base is owed, as under
`contract_effect: changes`.

**Lines.** `cargo workplan coverage lines [--crate <package>]` and
`Workplan::lines_report` give the reverse coverage of every package the loader
reaches, the root package and packages that no capability binds included, under
either strictness. `cargo workplan coverage --reverse [--crate <package>]` is the same
command and prints the same bytes. The units of a package are those of every target
the [source index](./source-index.md) lists (library, binaries, integration tests,
examples, benches and build script): every item except a module and an `extern
crate`, every `impl` block from its header to its closing brace, and every item-level
macro invocation. The counted lines of a file are the union of the lines of its
units, and a file is counted once for its package however many targets read it.
Other lines (blank lines, `use` declarations, `mod` declarations, inner doc comments
and the SPDX header) are structural: neither covered nor orphan, and shown as a
count. A unit is covered when:

1. a binding resolves to it: a `code` binding to the item, a `tests` binding to the
   test function, an `examples` binding to the documented item, and an example target
   to every unit of its target;
2. it is below a covered item as two-way coverage reads below: the methods and
   associated items of a covered type or trait and the items of a covered module, a
   crate root covering nothing below it; an `impl` block is covered when its self
   type is, or its module when the index cannot place its self type;
3. it is reachable: a covered unit names a path that the source index resolves, from
   the module of that unit, to it. The paths are the `Ident (:: Ident)*` runs of the
   unit's tokens that no `.` precedes, and every prefix of each; a path that
   resolves to nothing of the workspace reaches nothing, one that names several
   sites reaches each, and a reached module reaches nothing below it;
4. for a macro invocation, its module is covered, or a covered unit names a path
   that does not resolve and whose last segment is an identifier inside the
   invocation; a covered invocation reaches the `macro_rules!` it invokes.

Reachability is a product of the source index: the graph of the items of every
target, with the edges contains, impl-of and references, whose closure from the bound
items is the covered set. Every other counted line is an orphan line. An item a macro
generates has no lines of its own: it is counted as the span of the invocation that
generates it. The items of a `#[cfg(test)]` module are units like any other and their
lines are test lines; a line is a shipping line otherwise. `cfg` is never evaluated,
so the report does not depend on the host.

The report prints three tables. The crate table has the columns Crate, Enforced,
Counted, Covered, Excepted, Orphan, Shipping orphan, Test orphan, Ceiling and Debt; the
module table Crate, Module, Counted and Orphan; the item table Crate, Item, Kind, File
and Orphan lines, with merged `a-b` ranges. Rows are sorted by package, module path,
file and first line, and each orphan line is listed once, under the innermost unit
that holds it. In JSON the objects are of the kinds `lines-crate`, `lines-module`,
`lines-orphan` (with `ranges` as `[[a,b],...]`), `lines-excepted` (with `exception`
and `reason`) and `lines-error` (a surface the index refused). The report holds no
absolute path and no order that depends on the host, and a file with CRLF line endings
reads as the same file with LF. The command exits 1 when it prints an orphan line, 0
when it prints none, and 2 when `--crate` names no package.

**Ceilings and exceptions.** The coverage record takes two more optional keys. Each
row of `exceptions` holds exactly one of `item`, an item path or a prefix
`<module>::*` below a crate root matched against every item of the index, private
items included, or `file`, a repository-relative path or a directory ending in `/`,
and a non-empty `reason`. Excepted lines are reported with the reason of the first
exception that matches them, and they are not orphan lines. Under either strictness
an exception that matches nothing is `coverage-exception-unmatched`, a row with
neither or both of `item` and `file`, a crate root, an absolute path or a `..`
segment is `okf-bad-exception`, and an empty reason is `okf-empty-reason`. Each row of
`ceilings` holds a `crate`, exactly one of `lines`, a whole number, or `percent`, a
decimal, and an optional `of`, one of `shipping`, `test` or `all`, by default `all`.
Under strict every package of the report is judged: its shipping orphan lines against
its `shipping` row, its test orphan lines against its `test` row and their sum against
its `all` row, and a class that no row of the package bounds, by its own row or by an
`all` row, is held to 0, so a package with no row has no orphan line. A percent
ceiling compares `orphan * 100 * scale <= percent * scale * counted` in whole numbers,
`scale` being the power of ten that makes the percent whole, so no division rounds and
a package with no counted line passes it. A class over its ceiling is
`coverage-lines-over-ceiling` at `ceilings`, with a message that names the package,
its counts, the ceiling and its largest orphan items. A row whose `crate` names no
package is `scope-crate-missing` at `ceilings`; a row of another shape, an `of` of
another value included, is `okf-bad-ceiling`, and a number that is not one is
`okf-bad-measure`. Under `default` no ceiling is judged, and the report shows them.

**Debt.** The strict debt of a repository is the singleton record
`work/debt/default.md`, of the document type `Debt` with the id `debt:default`; any
other file in `work/debt` is `singleton-other-file`. Its keys are `type`, `id`, the
optional `tags` and `timestamp`, `findings`, rows of exactly `kind` and `subject`, and
`lines`, rows of exactly `item`, a crate-qualified item path, and `orphans`, a count;
the rows are sorted, none repeats, and the body is empty. It follows
[the debt ratchet](../architecture/debt-ratchet.md):

- a strict finding whose kind and subject a `findings` row records is not reported,
  and a `findings` row that no present finding matches is `debt-resolved` at
  `findings`;
- an orphan item with a `lines` row holds that many orphan lines as debt, which do
  not count against its package's ceiling; fewer present orphan lines is
  `debt-resolved` at `lines`, and more is `coverage-lines-over-ceiling` naming the row;
  a present orphan item with no row is growth, whose lines count against the
  ceiling. Line debt is a count; `coverage lines` reports the ranges and the record
  never holds them;
- the subject of a row is the claim key for a claim kind, `cap:<slug>#<section>` for
  a section kind, `cap:<slug>` for a capability kind and the reported path for a
  coverage gap;
- what can be debt is closed by construction: only a finding that strict alone
  raises, a strict kind or a coverage kind of a package that strict alone enforces. A
  row of any other kind, `coverage-lines-over-ceiling` included, or whose subject does
  not have the form of its kind, is `debt-entry-invalid` at `findings`. A finding of
  the default rules is never debt, and an obligation is never debt: it belongs to an
  open change;
- a debt record while the repository is judged under `default` is
  `debt-without-strict`, whose fix is to set `strictness: strict` or remove the
  record.

The record is written only by `cargo workplan debt --write [--copyright <text>
--licence <id>]` and `Gate::write_debt(&RecordOptions)`, which returns a `DebtWrite`
or a `DebtError`, never from a `#[test]`. The write takes the present strict debt: it
refuses while a finding of the default rules exists (`DebtError::Findings`), refuses
growth against an existing record (`DebtError::Growth`, with the findings that
grew), and creates a missing record from the present debt, which is how a repository
adopts the mode. A coverage record that does not say `strictness: strict` is
`DebtError::NotStrict`. The write creates or rewrites only `work/debt/default.md`,
with the SPDX lines of `RecordOptions`, and two writes of the same debt give the
same bytes. `debt --write` exits 0 when it writes, 1 when it refuses with findings,
and 2 when the coverage record does not say `strictness: strict` or the tool cannot
run.

**The kinds.** The finding kinds of this capability are in the closed list
`Finding::KINDS`: `strict-example-missing`, `strict-matrix-missing`,
`strict-claim-uncovered`, `strict-claim-unit-collapsed`, `claim-test-unbound`,
`strict-section-thin`, `strict-usage-unbound`, `strict-limitations-missing`,
`coverage-exception-unmatched`, `coverage-lines-over-ceiling`, `debt-resolved`,
`debt-without-strict` and `debt-entry-invalid`, beside the profile kinds
`okf-bad-strictness`, `okf-item-reason-empty`, `okf-tested-by-outside-capability`,
`okf-bad-exception` and `okf-bad-ceiling` and the reader kinds
`okf-parse-malformed-tested-by` and `okf-parse-unattached-tested-by`. A strict finding
is a `Finding` with its kind, file, location, message and fix, and its JSON object
carries `kind`, `path`, `section`, `line`, `key`, `message` and `fix`. The obligation
kinds `outcome-without-reason` and `contract-effect-understated` are in
`ObligationKind::ALL`. `claim-test-unbound`, `coverage-exception-unmatched`,
`debt-without-strict` and the profile and reader kinds are raised under either
strictness, and only by the marker, the keys and the record of this capability, so a
repository that uses none of them has no finding of these kinds under `default`.

## Usage

A repository opts in through its coverage record:

```yaml
type: Coverage
id: coverage:default
strictness: strict
scope: [raf-workplan, cargo-workplan]
exclusions: []
ceilings:
  - crate: raf-source
    lines: 0
    of: shipping
exceptions:
  - file: "crates/raf-source/tests/support/"
    reason: "Shared fixture builders of the integration tests."
```

The gate of a repository test is the same call, and a caller can try the mode before
the record says it, with `raf_workplan::Gate::strict`:

```rust
// tests/workplan.rs
#[test]
fn workplan_is_strict() {
    raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR"))
        .strict()
        .assert_clean();
}
```

The command line takes the same floor and gives the reports:

```sh
cargo workplan --strict check
cargo workplan --strict change reconcile incremental-parsing
cargo workplan coverage claims --capability cap:streaming-parser
cargo workplan coverage lines --crate raf-okf
cargo workplan debt --write --copyright '2026 Example Inc.' --licence CC-BY-4.0
```

A claim names its test on the line after it:

```markdown
- An exclusion with an empty reason is an error of the record.
<!-- raf:tested-by raf_workplan::test_crate::coverage::an_exclusion_without_a_reason_fails -->
```

## Examples

A delivered capability with a one-sentence Examples section, a Usage that names no
bound example and a Contract clause without a test fails `cargo workplan --strict
check`, and passes `cargo workplan check` under a record that says `default`:

```text
docs/capabilities/streaming-parser.md: section `Examples`: the section holds 6 words of prose, fewer than 20 [strict-section-thin]; write at least 20 words of prose saying what the section covers; a generated block does not count as prose
docs/capabilities/streaming-parser.md: section `Usage`: no path of bindings.examples is named in the section [strict-usage-unbound]; name a path of bindings.examples in a code span in the Usage section, so the verification matrix lists Usage for an example that verify runs
docs/capabilities/streaming-parser.md: line 41: the claim `cap:streaming-parser#contract/chunked-input` ("Chunked input. The parser accepts input in arbitrary chunk") names no test [strict-claim-uncovered]; put a raf:tested-by marker after the statement, naming the bound tests or examples that show it, or move reasoning that no test can show into the Explanation
```

The reverse coverage of one package, under a record that bounds its shipping lines
and leaves its test lines at the default of 0:

```text
$ cargo workplan coverage lines --crate demo
| Crate | Enforced | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| demo | yes | 412 | 380 | 20 | 12 | 9 | 3 | shipping 10 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |
| demo | `demo::render` | 96 | 9 |
| demo | `demo::tests` | 40 | 3 |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
| demo | `demo::render::pad` | fn | crates/demo/src/render.rs | 51-59 |
| demo | `demo::tests::helper` | fn | crates/demo/src/tests.rs | 30-32 |
$ echo $?
1
```

Under strict the three test lines are over the default of 0 for the class `test`, so
`check` reports `coverage-lines-over-ceiling` for `demo`, and binding or reaching
`demo::tests::helper`, an exception with its reason or a `test` row in `ceilings`
clears it. A repository adopts the mode by setting `strictness: strict` and running
`cargo workplan debt --write` once; the diff that creates the record is the review of
what the repository accepts as debt:

```yaml
type: Debt
id: debt:default
findings:
  - kind: coverage-item-unbound
    subject: "demo::Frame"
  - kind: strict-claim-uncovered
    subject: "cap:streaming-parser#contract/chunked-input"
lines:
  - item: "demo::render::pad"
    orphans: 9
```

Once a test is bound to `demo::render::pad`, `check` reports its row as
`debt-resolved` until `debt --write` lowers the record, and a new orphan item fails
the check, and the write refuses it.

## Constraints

- Reachability is read from path references, so a function stored in a field and
  called through it, a call through a trait object to a type that nothing names, and
  an item used only from inside a macro are not reached. An exception with its reason
  takes the rest.
- A claim's test is cited, not judged: the marker proves that a mapping exists, and
  the [prose assessment](./workplan-prose-assessment.md) of the section, which reads
  the marker with the text, is where a reader judges whether the test shows the claim.
- The key of a Constraints unit is the digest of its text, so a reworded or
  re-punctuated item is a new claim, and a repeated lead key is numbered by its place,
  so a clause with the same lead inserted before it renumbers it. Against a debt
  record either is growth beside a resolved row: touching a recorded claim makes it
  owe a test.
- An item moved to another file reads as touched for `contract-effect-understated`.
- `strict::MIN_SECTION_WORDS` is 20 and is not configurable. A word count is a proxy:
  it catches the one-sentence section and judges no style.
- Under strict a package with no ceiling row is held to 0 orphan lines, and a class
  that no row bounds is held to 0.
- A caller cannot lower the strictness of a repository whose record says `strict`.
- A directory that the root workspace excludes and that is a cargo workspace of its
  own, such as a fuzz harness, is not in the source index, so its files have no lines
  in the report and are not judged. No exception names it, since an exception that
  matches nothing is `coverage-exception-unmatched`.
- An item a macro generates is counted as the span of its invocation, and an
  expansion that nothing reaches is orphan with the lines of the invocation; a
  `#[cfg(test)]` module is counted as test lines, in its own column and under the
  ceiling of its class.
- A package that no binding reaches and `scope` does not name is not enforced by the
  explained-items rule; its lines are judged by the ceilings all the same.
- An exclusion of the coverage record is not an exception: an excluded item whose
  lines nothing covers is orphan until an exception names it with its reason, so a
  reason written once for the two directions is written again for the lines.
- The debt record holds strict findings and line counts only, and its text is held to
  the 1 MiB bound of a document.

## Explanation

Strict mode is opt-in because its rules are a cost a repository pays for a stronger
account: a marker for every guarantee, a binding or an exception for every line, a
reason for every dropped item. A repository that is still writing its first
capability documents needs a gate it can pass, and a repository that relies on its
documents as the specification needs one that refuses a thin account. The default
rules stay as they are for the first, and the second turns on one key.

### Decision

The setting lives in the coverage record `work/coverage/default.md`, as the key
`strictness` beside `scope`. The record is already the one repository-level policy
record: `check` requires it as soon as capability documents exist, it states what the
gate enforces, and it is in the graph. Strict mode is "enforce more", and most of its
rules are coverage rules, so one place says what is enforced. The record has a closed,
validated schema, so `strictness: strct` is `okf-bad-strictness` with its location
instead of a silent `default`, and it has a writer. The value is a closed enumeration,
as `contract_effect` is, because the profile has no boolean value kind, and it is a
preset of every strict rule rather than a set of rules chosen one by one: no consumer
asks for one rule alone, and the debt eligibility that a rule set would give is
already closed by construction, since the strict rules are the only producer of debt.
Three other locations were weighed and rejected:

- a file under `docs/capabilities`: the directory holds capability documents and the
  generated index only, the index is never hand-written, and any other Markdown file
  there is a stray document;
- metadata in `Cargo.toml`: the workplan reads manifests only through package
  discovery, which exposes no metadata table; cargo accepts any unknown metadata key,
  so the closed schema would need a second validator; a workspace has both
  `[workspace.metadata]` and per-package tables, so which one counts is ambiguous; and
  a finding could not point at a document location;
- a new configuration file, such as `workplan.toml`: a second format and parser
  beside OKF for one key.

### Conditions to revisit

- A second repository-level setting appears that is not about coverage, so the
  coverage record stops being the natural home of the policy.
- A repository needs one strict rule without the others, so the preset splits into
  independently chosen rules.
- The OKF profile gains a boolean value kind that the setting would read better as.

The mode reconciles with [two-way coverage](./workplan-two-way-coverage.md) and
[evidence claims](./workplan-evidence-claims.md) by adding no second rule for a fact
they already judge. That a public item is explained and that a test supports a
guarantee are the code and tests directions of two-way coverage; strict mode runs the
same rule, with the same kinds, over a larger set of packages, the ones a binding
reaches. That a guarantee has a test is the opposite direction of the tests-to-claims
edge, at the grain of a claim, which two-way coverage does not have: its tests
direction asks only that some capability binds the test. That a statement rests on a
recorded execution is the evidence claim, and an evidence marker is not test coverage:
a bound test that exists and a run that was recorded are different facts with
different invalidation, so the tested-by marker is a family of its own and not an
attribute of `raf:claim`, whose malformed and unattached forms stay as they are.

A claim is a Contract clause rather than a paragraph because the documents state one
guarantee per bold-lead clause over several paragraphs, lists and tables; a marker on
each paragraph would demand tests of continuation text that carries no separate
claim. A Contract with no clause start would be one claim, which one marker could
cover however long the Contract is, so a long one is a finding of its own.

`contract-effect-understated` reads "touch a bound public item" literally: any part of
its span, its body and its doc comments included. A body is behaviour that callers
rely on as much as a signature is, and a doc comment is the contract a reader of the
API sees, so a change that alters either has a contract effect. Comparing token
streams keeps the rule to what the compiler and rustdoc read: a reformatting or a
plain comment changes no token. The comparison is by file, so the price of a move to
another file is a reconciled section.

Reverse coverage is `coverage lines` because the command is `coverage <direction>`,
and `lines` names what is counted; `coverage --reverse` is accepted as the same
command for the reader who asks for the reverse of the code direction, and a test
holds the two outputs equal. Every package the loader reaches is judged, because an
orphan line in a package no capability binds is still behaviour nobody explained.
Line debt is a count per item and not per package, so an orphan in one item cannot
be traded for a new one in another.

### Decision

The strict debt follows [the debt ratchet](../architecture/debt-ratchet.md), the
semantics of the census debt: growth fails, a resolved entry fails too until the
write lowers the record, and the write refuses growth except when it creates the
record. The workplan keeps its own record, `work/debt/default.md`, apart from the
authored coverage record as the census keeps its debt apart from its ledger, and it
shares no file with the census. Failing on resolved debt keeps the record without
slack: it always equals the present debt, so a regression after an improvement is a
visible failure, and the fix is one command.

### Conditions to revisit

- A repository needs to record as debt a finding that strict mode alone does not
  raise.
- The conditions to revisit of the debt ratchet page hold, and the two ratchets share
  their compare and write code.

## Verification

- Setting fixtures: a record that says `strict` is judged strict under a default
  floor, a strict floor judges a record that says `default` strict, `Opened::workplan_after`
  keeps the floor, a misspelt value is `okf-bad-strictness` beside `strict` and
  `default`, and `cargo workplan check` with no flag exits 1 on a record that says
  `strict` with one thin section and 0 on the same record saying `default`.
- One fixture per kind of this capability that raises it beside a passing neighbour:
  no example beside one example, no matrix beside the block, 19 words beside 20, a
  Usage without a bound example beside one that names it in a code span, a Constraints
  section without a list item beside one with one, a long Contract without a clause
  start beside one with clauses, an uncovered clause beside a clause covered by a
  marker on its second paragraph, a heading clause covered by a marker after its
  heading, a marker naming an unbound path beside a bound one, an exception that
  matches nothing beside one that matches, and each debt kind beside its repair.
- Marker fixtures: well-formed after a statement and after a claim marker; malformed
  with a space after a comma, a repeated entry, no entry and a `tests=` form; unattached
  after a blank line, after another comment and before a claim marker; not read in a
  fence; the fixtures of the claim marker unchanged.
- A bound package outside `scope` with an unbound public item fails under strict and
  passes once the item is bound, and the same package bound by no capability is not
  enforced.
- Git fixture repositories for completion under strict: a `contract_effect: none`
  change that edits a bound function's signature, its body, an attribute or a doc
  comment refuses under strict and completes under default; a reformatted body, a
  plain comment and an item moved within its file complete; a new bound public item
  refuses; a `dropped` item without `reason` refuses condition 2 under strict and
  completes with `reason`, and without it under default; an untouched Usage or
  Examples in `affects` of a `none` change refuses under strict and completes under
  default; each refusal leaves the tree byte-identical.
- Reverse coverage fixtures: a fully bound crate has 0 orphan lines; an unreferenced
  private helper is exactly its span as one range, beside the same helper reached
  from a bound function; binding a type covers its `impl` blocks and their methods;
  a macro invocation is orphan with its range and covered once a bound function names
  what it generates; a `#[cfg(test)]` module with one bound and one unbound test
  reports the unbound one in the test column; a workspace that excludes a directory
  of Rust files reports no row for it; text and JSON equal literal strings, two runs
  are byte-identical, CRLF reads as LF, and `--reverse` prints what `lines` prints.
- Ceiling fixtures: `lines: N` passes at N and fails at N+1 for each of `shipping`,
  `test` and `all`; a percent ceiling at its boundary; an empty package under a
  percent ceiling; a `shipping` row alone leaves the test lines held to 0; another
  `of` value is `okf-bad-ceiling`.
- Debt fixtures, one to one with the census ratchet: growth fails and the write
  refuses and writes nothing; a resolved row, a falling count and a count back under
  the ceiling fail with `debt-resolved` and pass after the write, the record equal to
  the present debt; with no record every strict finding stands and the write creates
  it; a default finding makes the write refuse; two writes are byte-identical; the
  write writes only `work/debt/default.md`; `debt --write` exits 0, 1 and 2.
- Under `default`, the counted work of `check` judges no strict unit and counts no
  line.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
<!-- raf:generated:end -->
