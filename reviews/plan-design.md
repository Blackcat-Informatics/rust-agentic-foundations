# Plan design review (SOLID / ETHOS)

Reviewed plan: `.stage/raf-workplan-change-lifecycle-cargo/plan.md`, revision 2 (recheck of the
revision 1 review below the fold).

Code read: `crates/raf-workplan/src/{lib,model,gate,finding,rules,roadmap,graph,bindings,sync,preview}.rs`,
`crates/raf-workplan/tests/kinds.rs`, `crates/raf-workplan/examples/docs_sync.rs`,
`crates/raf-okf/src/document.rs` (`Document::new`, `render`), `crates/raf-source/src/workspace.rs`
(`TestFn`, `test_fns`, `test_fn_of`), `crates/raf-source/src/target.rs` (`Target`), `Makefile`
(`docs-sync`), `docs/capabilities/{workplan-reconcile,workplan-complete-gate,workplan-cli}.md`,
`work/changes/{workplan-reconcile,workplan-complete-gate,workplan-cli}.md`.

## Recheck of revision 1 findings

### B1 (two `git` spawners; version gate ordering): DISCHARGED

D3 moves "Existing `Gate` git calls and `examples/docs_sync.rs`" onto the one runner; D4 adds
`Gate::programs(Programs)` and `Gate::open()`, has `try_check` call `open`, and makes the binary,
the example and `Lifecycle` load through it with the version gate first inside `open`. Task 2
names the move of `git_tracked` (`gate.rs:287-353`) and the `.git` detection. That is the
rewrite asked for: "version first" is now a code path and `check` *is* `Gate::try_check`.

### B2 (state-support check re-encoded): DISCHARGED

D9: "The state-support check is not re-encoded: it is 'no `check` finding'", evaluated on a
post-deletion model loaded through a `Tree` wrapper that hides the record. `raf_source::Tree` is
the existing seam (`model.rs:121`, `load::load` takes `&dyn Tree`), so the wrapper is cheap and
needs no change to the loader. Note the consequence this design correctly entails: with the
record hidden, `no-open-record` (`rules.rs:164-174`) fires for any listed capability that is not
`delivered` and not listed by another open record, so `complete` requires the listed
capabilities to end `delivered` (or to be carried by another record). That matches the
complete-gate State paragraph and Scenario B; the plan should say it in one sentence so an
implementer does not read the finding as a bug.

### A1 (slug lookup private to `preview.rs`): STILL-OPEN, advisory

Revision 2 does not name a shared lookup. `preview.rs:209-218,263-282` still hold the only
unknown-vs-not-loaded logic, and `Workplan::unloaded` is `pub(crate)` (`model.rs:167`). Five
callers (`preview`, `impact`, `reconcile`, `complete`, `verify`) and one CLI exit-code rule
("exit 2 when the record does not exist, except `new`") still want one answer.
REWRITE TO: `pub(crate) fn Workplan::change(&self, slug) -> Result<&Loaded<Change>, ChangeLookup>`
with `Unknown { slug, known }` and `NotLoaded { slug, findings }`; `preview` converts it. Add to
Task 4 or 5.

### A2 (condition 4 via `sync` needing SPDX): DISCHARGED

D6 derives `generated-stale` from `generated::analyze`; D9 checks "generated blocks current" among
the static conditions before any write, so the later `sync` that rewrites the Changed-by rows
runs only on a tree whose index exists and never hits `SyncError::SpdxUnset` (`sync.rs:244-247`).

### A3 (`docs sync` SPDX inputs): DISCHARGED — D12 adds `--copyright`/`--licence`, read only when
the index must be created, and the Makefile target runs the binary.

### A4 (argv placeholders from `TestFn`/`Target`): DISCHARGED — baseline list and D11 name
`raf_source::{TestFn, Target}` and the index-fact refusals (`crate_path` stripping, harness,
required-features) before any spawn.

### A5 (`binding-missing` duplicates `roadmap::gaps`): DISCHARGED with one residual (see A8)

D6 derives it "from `RoadmapEntry::gaps()` and `binding-*` findings".

### A6 (`target/` writes; timeouts): DISCHARGED — R16 "reconcile may write `target/` only", D11
shared `CARGO_TARGET_DIR`, D3 1800 s / 300 s bounds as a Contract edit.

### A7 (unparsable merge-base blob): DISCHARGED — D7 "a blob that exists but does not parse is an
error".

## New text (findings only where there is a consequence)

### A8. `RoadmapEntry::gaps()` is empty for every state but `implementing`

D6: "`binding-missing` from `RoadmapEntry::gaps()` and `binding-*` findings". Code:
`roadmap.rs:275-279` computes gaps only when `state == Implementing`; a `delivered` capability
with an empty `tests` list is reported by raf-okf as `okf-binding-required` (`finding.rs:71`,
`finding.rs:332-335`), whose kind does not start with `binding-`.

Consequence: for a change listing a `delivered` capability that lost its last test binding,
`reconcile` prints no `binding-missing` line while `complete` refuses on the `check` finding; the
Contract says the obligation list is the record of what `complete` will refuse on.

REWRITE TO: make `roadmap::gaps(id, graph)` (`roadmap.rs:293`) `pub(crate)` and call it for a
listed capability when its state is `implementing` or `delivered` (empty lists are allowed only in
`designed`/`considered`, per the complete-gate Constraint); `RoadmapEntry::gaps()` keeps its
implementing-only contract for `status`.

### A9. `Workplan::write` cannot carry the removal

D9: "The writes are the record removal and the rewrite of the generated blocks the removal stales
... staged together through `Workplan::write`". Code: `Workplan::write` (`sync.rs:285-313`) only
renames staged texts into existing files or creates the index; `judge` (`sync.rs:338-356`)
refuses any other path. There is no removal primitive, and a removal cannot share the rename
batch.

Consequence: the two writes are sequential and not atomic. If the plan does not fix the order, an
implementer may delete first and leave a tree with a stale Changed-by row and no record on an
I/O failure, contradicting "deletion is the last step" (R20/R21).

REWRITE TO: state the order: `Workplan::write(report)` for the generated rewrite first, then
`fs::remove_file(record)` last, failure surfaced; a failure between the two leaves the record in
place and the rewrite present, which the next `check` reports as `generated-stale` (loud) and the
next `complete` finishes. Edit the sentence in D9 and the complete-gate Contract edit accordingly.

### A10. The dev-dependency `Gate` now requires Git >= 2.30

D4 runs the version gate inside `Gate::open`, which `try_check` and therefore every
`Gate::workspace(..).assert_clean()` test call. Previously the Gate needed only `rev-parse` and
`ls-files` (`gate.rs:292-332`). This is the right one-path design and the failure is loud
(`GateError`), but it is a new requirement on every consumer's CI image.

REWRITE TO: add a `GateError` variant naming the version and the minimum, document the floor in
the `Gate` doc comment and in the document-model capability's constraints (a Contract-edit row),
and keep the existing "not inside a work tree" fallback (`gate.rs:282-286`) unchanged so a
snapshot without `.git` still checks.

### Not findings (read and accepted)

`Runner` trait with a real implementation and an in-process recording fake (D3, E13): the fake is
a named second implementation and the real path is still exercised by the PATH/`$CARGO` stub
fixtures, so this is a seam, not speculative generality. Typed `Invocation`/`Arg` (D5): the
right place to hold the "no document text as an option" invariant. Declined pluggable
obligation-source trait: correct. `Tree` wrapper for the post-deletion model: reuses the existing
seam. TOCTOU re-check before writes: sound.

## Summary

Both blocking findings are discharged by design changes that match the code's real seams. The
remaining items are advisory: one shared slug lookup (A1), the `implementing`-only limit of
`RoadmapEntry::gaps()` (A8), the sequential write order that `Workplan::write` forces (A9), and
documenting the new Git floor the Gate acquires (A10). None of them is a correctness hole that the
plan's tests would miss silently; A8 and A9 should be folded into D6 and D9 before Task 5 and 7
start.

VERDICT: PASS
