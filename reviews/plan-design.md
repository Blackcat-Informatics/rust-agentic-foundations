# Plan design review (SOLID / ETHOS)

Reviewed plan: `.stage/raf-workplan-model-documents-check/plan.md`

Code read to ground the judgments:

- `crates/raf-okf/src/{lib,document,index,span,error}.rs`, `body/{mod,blocks,marker,link,heading}.rs`,
  `profile/{mod,typed,violation,body_rules}.rs`
- `crates/raf-source/src/{lib,workspace,resolve,target,item,options,discovery}.rs`, `tree/{mod,memory}.rs`
- `tests/docs_contract.rs` (all rule functions, the fixture builder, the binding tests, the
  plain-Markdown tests), `tests/support/mod.rs`, `.githooks/pre-commit`, `Makefile`, `Cargo.toml`,
  `fuzz/Cargo.toml`, `REUSE.toml`
- `docs/architecture/okf-profile.md`, `docs/capabilities/{workplan-document-model,workplan-generated-docs,workplan-cli,workplan-reconcile,workplan-complete-gate}.md`

Facts that settle questions the plan raises without a finding:

- J12 `roots: ["."]` is safe here. `discovery.rs:51-53` skips `target` and every dot-directory
  (`.worktrees`, `.stage`, `.githooks`), and `discovery.rs:120` stops below a `[package]` manifest
  without `[workspace]`. The walk also picks up `fuzz/` (a package with its own empty `[workspace]`)
  and the root package's `tests/*.rs`; both parse and are harmless.
- The exact crate path of an example target resolves to its root module item (`resolve.rs:1174-1187`),
  so the "example target's crate root" half of J5 is computable from the index as planned.

## BLOCKING

### B1. Sync "writes through the tree", but `raf_source::Tree` cannot write (J13, Task 5)

Plan: "`Workplan::sync() -> SyncReport`, which writes through the tree" (J13) and
"`examples/docs_sync.rs` writes through `DiskTree` for this repository" (Task 5).

Code: `crates/raf-source/src/tree/mod.rs:139-220` declares `Tree` as "A read-only tree of UTF-8
text files below one root" with `read`, `read_unseen`, `identity`, `list`, `entry` and no write
method; `DiskTree` and `MemoryTree` implement nothing else.

Consequence: Task 5 as written cannot be built. The two ways out are both wrong: (a) the library
writes with `std::fs` behind the `Tree`, which discards the containment the seam was chosen for in
J14 (`fs::write` follows a symlinked `docs/capabilities/x.md` out of the root; `DiskTree::read`
refuses it), or (b) a second, write-capable tree trait with one implementation. A separate
"Staleness" bullet beside "Sync" also invites two renderers of the same block, and a block that
`check` accepts and `sync` rewrites (or the reverse) is exactly the drift the staleness rule exists
to stop.

REWRITE TO: make sync pure and give it one renderer shared with `check`.

```rust
/// The full new text of every file whose generated blocks differ from what
/// the documents and the index produce, keyed by path. Empty when nothing is stale.
/// `docs/capabilities/index.md` is present when absent on the tree or when its
/// catalogue differs.
pub fn sync(&self) -> Result<BTreeMap<RelPath, String>, SyncError>;
```

`check` derives `Rule::GeneratedStale { kind }` (one per differing block, file named) and
`Rule::IndexMissing` from the same per-file render; `sync` returns the texts. Each file's edit goes
through `Document::replace_spans` / `Index::replace_spans` on the block `content()` spans (all or
nothing per file, as `body/mod.rs:379-420` guarantees). The example target and, next issue, the CLI
write each returned entry under the root with `std::fs` after `symlink_metadata` reports a regular
file or nothing, so no write follows a link. Idempotence is then the test `sync()` is empty after one
application, with no second code path. Note for Task 8: `REUSE.toml` has no annotation for
`docs/capabilities/index.md`, so a created index with only a `catalogue` block fails `make reuse`;
the plan should state that Task 8 adds the SPDX comment `Index::parse` allows (`index.rs:36-49`)
and that a later sync keeps it, with a determinism fixture whose index carries a header.

### B2. The source `Workspace` is optional on `Workplan::load`, so the binding rule and the gaps degrade silently (Task 4, §H)

Plan: "`Workplan::load` takes an optional source `Workspace`" (Task 4); Task 1's
`Workplan::load(tree, options)` leaves `options` undefined.

Code: every consumer of the loaded model needs resolution results: "bindings that resolve through
the source index" is a `check` rule (`workplan-document-model.md:115`), `status` and the
`catalogue` row must show "bindings that are empty or do not resolve"
(`workplan-generated-docs.md:46-47`, `workplan-cli.md:58-59`), and the verification matrix needs
each bound item's kind (`workplan-generated-docs.md:38-40`). `docs_contract.rs:929-939` today
refuses to run bindings against an empty index for the same reason ("A load that found nothing
would pass the check below").

Consequence: with `None`, `check` passes a capability whose bindings name nothing, the catalogue
shows an `implementing` row with no gap, and the matrix cannot be rendered; either each site grows
an `if let Some(ws)` branch with its own fallback, or the CLI and the completion gate must
rediscover that the "same checks" are not the same.

REWRITE TO: `Workplan::load(docs: &dyn Tree, source: &Workspace) -> Result<Workplan, LoadError>`
with both required. Fixtures that do not exercise bindings pass a two-file `MemoryTree` workspace,
as `docs_contract.rs:3020-3033` already does. `Gate::workspace(dir)` builds both from one
`DiskTree::new(dir)`. Remove `options` from Task 1 or name what it holds.

### B3. The link-existence oracle silently changes from the tracked file list to the disk (J6, J14, Task 7)

Plan: links "resolve to existing repository paths" (J6); `tests/docs_contract.rs` is "replaced by
`Gate::workspace(gate_root()).assert_clean()`" (Task 7); R18 fixtures are "ported from
`docs_contract`" and run on `MemoryTree` (Completeness table, J14).

Code: `docs/architecture/okf-profile.md:205-207` states the rule as "A target inside the root is
looked up in the list of tracked paths (the Git index), and a link whose target is a tracked
symlink is a violation". `docs_contract.rs:375-412` implements exactly that over `Repo` built from
`tracked_paths()` (`load_repo`, lines 875-894), and `rejects_link_to_a_tracked_symlink` (line
1269) fixtures it. `Gate::workspace` over `DiskTree` answers `tree.entry()` for anything on disk.
`MemoryTree` "has no symlinks" (`tree/memory.rs:35`).

Consequence: a document that links to an untracked file passes `make check` locally (the gate the
repository tells people to run) and fails on a fresh clone; the rule in the committed profile page
no longer matches the code, and Task 8's reconciliation would have to weaken the page. The symlink
fixture cannot be ported to `MemoryTree` at all, so the R18 claim is unverifiable as planned.

REWRITE TO: keep the oracle explicit and reuse what exists. `Workplan::load(docs: &dyn Tree, ..)`
treats the tree as the set of paths a link may name. `tests/docs_contract.rs` does not call
`Gate::workspace`; it builds a `MemoryTree::from_iter` from today's `load_repo` (path to text,
`""` for non-Markdown, tracked symlinks left out so a link to one reads as `NotFound` and the
tracked-symlink refusal stays in `docs_contract` as J1 says), loads the source with
`Workspace::load(&DiskTree::new(root), &["crates"])`, and calls `Workplan::load(..).check()`.
`Gate::workspace` keeps `DiskTree` for external consumers; its tempdir fixture (Task 7) carries the
link-to-symlink case, which only a disk can represent. Update okf-profile.md to say the oracle is
the tree the caller supplies, which for this repository is the git index.

### B4. The `examples` role in J5 needs a fact the index does not hold (Task 4)

Plan: "`examples` must resolve either to an example target's crate root ... or to an item whose
documentation holds a Rust code block (a documentation test). Anything else is a role violation."

Code: `crates/raf-source/src/item.rs:305-310` exposes `documented: bool` ("non-blank documentation
... or `#[doc = include_str!(..)]`") and nothing about the documentation's content; the facts stay
inside the crate (`workspace.rs:29-31`). `docs_contract.rs:690-692` today applies no role to
`examples`. The proof that a doc test exists and runs is already assigned to the completion gate's
`cargo test --doc -- --list` pass (`workplan-complete-gate.md:99-115`).

Consequence: to implement the sentence, Task 4 must re-read `Workspace::source(path)` and re-parse
attributes and doc comments (a second doc parser beside raf-source), and an `include_str!` doc test
is unreadable that way, so a real doctest binding is reported as a role violation. The alternative
is to ship `documented` while the Contract says "holds a Rust code block", which is the
contract-does-not-match-code failure the plan's own lessons list.

REWRITE TO: define the role from index facts only and write that into the Contract: an `examples`
entry resolves to the root item of a target with `TargetKind::Example` (`item.path ==
target.crate_path`), or to an item with `documented == true` in a target whose `doctest` is true.
Add the negatives the plan omits: an undocumented item, an item in a test target, a non-root item of
an example target. Leave "the doc test actually runs" to the completion gate.

## ADVISORY

### A1. Task 2 re-implements rules raf-okf already enforces (reuse)

Plan Task 2 lists "id equals slug; kebab-case slugs; `affects` slugs are listed in `changes`, and
its sections exist; `outcome: deferred` is refused by the closed set" as rules to implement.

Code: `profile/mod.rs:308-338` (`InvalidId`, `IdFileName`), `:555-574` (`BadAffectsSection`,
`AffectsNotInChanges`), `:638-657` (`BadOutcome`). The seven sections are required, so "its
sections exist" is `BadAffectsSection` plus `SectionMissing`.

Consequence: two implementations of one rule that can disagree, and a second set of fixtures to
keep in step. REWRITE TO: Task 2 adds only the cross-document rules (`depends` and `changes` name
existing capabilities, `moved:` names an existing change, acyclic `depends`); R24 and R8-R10 are
satisfied by Task 1's end-to-end fixtures that surface the raf-okf `Rule` through `check`.

### A2. `Violation { file, line, rule }` loses the location kinds raf-okf reports

Plan Task 1: "Every raf-okf violation becomes a `Violation { file, line, rule }`."

Code: `profile/violation.rs:16-25` locates a violation at a `Key`, a `Section`, a `Line` or the
`Document`; a `MissingKey` has no line. `workplan-cli.md:93-94` wants `kind`, `path`, `section`,
`message` per finding.

Consequence: a fabricated line (or an `Option<usize>`) for every key and section violation, and the
CLI has to recover the section from the message. REWRITE TO: `Violation { file: RelPath, at:
raf_okf::Location, rule: Rule }` with `Rule::Profile(raf_okf::Rule)` beside the crate's own
variants, `Display` delegating to raf-okf's. The R26 exhaustive-match coverage test enumerates the
crate's own variants only (`raf_okf::Rule` is `#[non_exhaustive]`).

### A3. `status` and the catalogue group the same states two ways (J4, Task 6, §O)

Plan: `Roadmap` has "planned, implementing (with each gap) and delivered entries"; the catalogue
block separately "groups capabilities by state ... `implementing` gaps".

Code: `workplan-cli.md:58-59` requires `status` to group "by state" and to name "the open changes",
and its example (`:85-88`) prints `delivered`, `implementing`, `designed` lines, not "planned".

Consequence: the CLI issue has to change the `Roadmap` type or recompute the grouping, and the
catalogue and `status` can disagree on a gap. REWRITE TO: one `Roadmap` grouped by `State` in the
catalogue order (`delivered`, `implementing`, `designed`, `considered`), each entry carrying slug,
title, description, gaps and open change slugs; the `catalogue` block is a rendering of that value;
"planned" is a label the CLI may print.

### A4. The verification-matrix section heuristic cannot see backticked symbols (Task 5)

Plan: "The section is the Verification section, unless a binding is named in another section's
text."

Code: `body/mod.rs:239` builds prose lines with code spans removed (`remove_code_spans`), so a
symbol written as `` `raf_workplan::Gate` `` is absent from `prose_lines()`; `Body::section_text`
(`:316-321`) returns the raw text.

Consequence: implemented over prose lines, the rule silently never fires and every row says
Verification; implemented over raw text, a symbol named in two sections has no stated winner, so
the output is unspecified. REWRITE TO: search `section_text` of each section in document order for
the binding path as a whole token; the first match wins, else Verification; state that in the
Contract and fixture a backticked mention in Usage.

### A5. R25 creates a second issue-reference matcher beside `tests/support`

Plan R25: "This ports the `docs_contract` scan rule for documents; the scan over every tracked file
stays in `docs_contract` per J1."

Code: `tests/support/mod.rs:180-220` (`hash_number`, `pull_request_label`, the non-process
`NUMBERED_FORMS`) is the matcher that `repo_gates` and `docs_contract` run over every tracked file.

Consequence: a document can pass `cargo workplan check` and fail `make issue-refs`, or the reverse,
once the two matchers drift. REWRITE TO: the crate exports the issue-reference matcher (one
function returning the matched form); `tests/support/mod.rs` calls it and keeps only the
process-wording forms, which are this repository's policy. The root package already dev-depends on
the crates.

### A6. The public link checker and the plain-Markdown rules are underspecified (J6)

Plan: "The link checker is a public function, so `docs_contract` checks the repository's other
Markdown files with the same code."

Code: anchors cross the loaded set in both directions (a capability links to
`../../SECURITY.md`, README links to `docs/capabilities/x.md#purpose`), so the checker must read
target bodies on demand through the tree. `docs_contract.rs:3383-3531` also applies split-link,
placeholder, marker-error and PROPOSED-outside-capability rules to `README.md`, `docs/BRAND.md`,
`.github/*.md`; the plan assigns those to nobody after J1.

Consequence: either `docs_contract` keeps a private copy of those body rules (the duplication J1
removes) or they are lost. REWRITE TO: a `Links` value holding `&dyn Tree` and a body cache, with
`check_file(&mut self, path: &RelPath) -> Vec<Violation>` that parses the file as `Document` or
`Body` and reports parse errors, split links, placeholders, PROPOSED blocks outside a capability and
every link target; `Workplan::check` runs it over the loaded documents, and `docs_contract` runs the
same value over every other tracked Markdown path.

## Task boundaries

Tasks 1 to 8 are ordered correctly for the consolidation (every rule exists before Task 7 deletes
its copy). Two boundary defects are covered above: Task 1's undefined `options` and Task 4's
optional workspace (B2), and Task 5's write path (B1). Task 2 shrinks under A1.

VERDICT: NEEDS-REVISION

## Recheck

Reviewed: `plan.md` revision 2. Only the text the revision introduced was reviewed (G1-G19, the
rule inventory, the generated-content section, Tasks 1-10).

### Prior findings

- **B1 (sync write path): DISCHARGED.** G6: "`Workplan::sync() -> SyncReport { edits: Vec<(RelPath,
  String)> }` computes the full new text ... `check`'s staleness comes from the same renderer ... The
  `Tree` seam stays read-only"; writing is a separate `write(edits, root)` with a regular-file
  precondition and temp-file rename. G12 covers the index SPDX comment through `SyncOptions`,
  "refused when unset", which is fail-hard and has a real second use (another repository's header).
  One note for the implementer, not a finding: `rename` onto a path that `symlink_metadata` reports
  as a symlink replaces the link with a file rather than following it; the "regular file already
  read through `DiskTree`" precondition already excludes that case, so the fixture for `write`
  should include a symlinked target being refused.
- **B2 (optional workspace): DISCHARGED.** G7 and Task 1: `Workplan::load(docs: &dyn Tree, source:
  &Workspace)`, both required; Task 1's undefined `options` is gone.
- **B3 (link oracle): DISCHARGED.** G4: `docs_contract` "builds a `MemoryTree` from the git-tracked
  paths ... That keeps the link oracle on the git index". The inventory lists `link-symlink` under
  R18's "ported fixtures" (plan line 86, 122); `MemoryTree` cannot represent a symlink
  (`tree/memory.rs:35`), so that one fixture must be the R23 disk fixture. The plan's R23 row allows
  it; Task 3's implementer should not try to express it on `MemoryTree`.
- **B4 (examples role): DISCHARGED.** G9 defines the role from `documented == true` and the target's
  `doctest`, and R13 adds "a non-doctest example" as a failing fixture.
- **A1 (duplicated single-document rules): DISCHARGED**, with one residue below (N2).
- **A2 (violation location): DISCHARGED.** G2 `Finding { kind, location: raf_okf::Location, message,
  fix }`.
- **A3 (status vs catalogue): DISCHARGED.** G18: one `Roadmap`, "A fixture asserts the catalogue
  rows equal the roadmap rows"; each entry carries its state and open changes, so the CLI can print
  either grouping.
- **A4 (matrix section rule): DISCHARGED.** G10: "contains the bound path in a code span, read from
  the raw section text by byte span", several sections in document order, else Verification.
- **A5 (second reference matcher): DISCHARGED.** G5 moves the matcher into `raf_workplan::refs` with
  `tests/support` calling it.
- **A6 (link checker and plain-Markdown rules): DISCHARGED.** R18 names the public link checker's
  callers (`README.md`, `AGENTS.md`, `CONTRIBUTING.md`, `docs/BRAND.md`), and Task 7's disposition
  table must give every existing plain-Markdown test (split links, placeholders, marker errors,
  PROPOSED outside a capability on `README.md`/`.github/*.md`) a row. The table is where the home of
  those rules gets decided; the reviewer of Task 7 should refuse a row that re-implements a
  raf-okf body rule inside `docs_contract` instead of calling `Body::split_links` /
  `placeholder_lines` through the public checker.

### New text

- **N1 (ADVISORY, G7): the `fuzz/` exclusion claim is wrong for the stated roots.** G7: "This
  repository uses `["crates", "."]` as needed so the root package's tests are bindable and `fuzz/` (a
  separate workspace) is excluded; a fixture asserts which packages are loaded." `discovery.rs:88-154`
  descends from `.` because the root manifest has `[workspace]`, reads `fuzz/Cargo.toml`, and records
  `raf-source-fuzz` as a package (`fuzz/Cargo.toml:9-10`); a package's own `[workspace]` table does
  not exclude it, and `Cargo.toml` `exclude` is not read. Consequence: the fixture that "asserts
  which packages are loaded" fails as written, or is written to the wrong expectation. REWRITE TO:
  either state that `"."` loads `raf-source-fuzz` too (harmless: its targets parse) and assert the
  four packages, or use `["crates"]` plus the root package's own path if its tests must be bindable.
- **N2 (ADVISORY, rule inventory line 76): `affects-section-missing` cannot fire on a validated
  capability.** raf-okf already refuses an `affects` section outside the seven
  (`profile/mod.rs:555-565`, `BadAffectsSection`) and a capability missing any of the seven
  (`body_rules.rs:86-93`, `SectionMissing`), and `Workplan` only holds typed, validated documents
  (G3). Consequence: a dead rule whose fixture has to break validation first, which the G2 coverage
  test will expose as a kind no fixture produces. REWRITE TO: drop the kind; an `affects` entry
  naming a missing capability is `missing-reference`, and the section half is raf-okf's.
- **N3 (ADVISORY, G2): the coverage test is one-directional.** "every kind in `KINDS` is produced by
  at least one named fixture" does not catch a `Finding` built with a `kind` string that is not in
  `KINDS` (a typo at one emit site). REWRITE TO: emit kinds through `const` items that are the
  members of `KINDS` (so a string literal never reaches a `Finding`), and have the coverage test also
  assert every kind produced across the fixtures is in `KINDS`.
- **N4 (note, G5):** moving the whole matcher, process-wording forms included, into the public crate
  makes "this branch" / "task 3" prose a `check` failure for every consumer, not only this repository.
  `okf-profile.md:208-210` states the process-wording rule as part of the profile, so this is
  consistent with the spec; recorded so the Contract sentence of the delivered document says so.

The design now has one renderer for staleness and sync, one finding vocabulary, one roadmap, one
reference matcher, required inputs, and an explicit existence oracle. The new items are change-cost
and fixture-accuracy matters, none a silent failure or a duplicate path.

VERDICT: PASS
