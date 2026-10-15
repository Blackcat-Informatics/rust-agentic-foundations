# T4 review: `change new` and `change impact`

## Scope

Staged, uncommitted diff on top of b6a367a: `lifecycle` (create, impact), `Display` for
`SectionId`, `BindingKind::key`, public `raf_okf::is_kebab`, CHANGELOG, tests/lifecycle.rs
and tests/support/disk.rs. Read against plan D6, D8, Task 4, rows R1/R2/R18, the
reconcile/cli/document-model capability contracts and rules.rs. Experiments ran in a
temporary worktree (removed); the staged index was not touched.

## Evidence

* Gates (staged tree): raf-workplan + raf-okf tests 709 pass / 0 fail, debug and release;
  workspace clippy -D warnings clean; `cargo doc` -Dwarnings clean; fmt clean;
  docs_contract (26) and repo_gates (57) pass; `make reuse` compliant (343/343);
  `make deny` ok.
* (1) Exactness: record keys/order, id, Summary, default and explicit `affects` asserted
  byte-for-byte (FRESH_BETA). Experiments: `work` or `work/changes` replaced by a symlink
  to an outside directory -> refused (`not a directory`), outside directory stays empty;
  dangling symlink at the target -> `Exists`, link target never created; 8 concurrent
  creations of one slug -> exactly 1 Ok, 7 `Exists`, file present. `create_new` is
  O_EXCL, so the failure path (`Exists`) never reaches a remove. Cleanup removes only
  after this call's own successful `create_new` (write failure or `Invalid`).
  `is_kebab` is ASCII lowercase/digit parts joined by single `-`, no empty part: rejects
  uppercase, Unicode, `-x`, `x-`, `a--b`, empty, `.`/`..`/separators; accepts digit-first
  and digits-only. Same rule as the cli Constraint (starts with letter or digit, never `-`).
* (2) `is_kebab` export is additive (a `pub(crate)` fn made `pub`, a doctest added). The
  okf-profile capability/docs name no exported item, so no statement is needed;
  usage_docs and docs_contract pass.
* (3) REUSE: a tool-created record has no SPDX lines; with it present `uvx reuse lint`
  reports 343/344 and the repo non-compliant (work/changes/*.md carry inline CC-BY-4.0
  lines; REUSE.toml has no glob for them). See ADVISORY A1.
* (4) Guidance vs rules.rs: delivered -> state-delivered-proposal and
  state-delivered-open-record; designed -> state-designed-proposal; considered and
  implementing accept a block and an open record. `StateAdvice::for_state` matches, and
  tests place the template in each of the four states and assert one `proposals()` block
  and the named finding. `sync_needed` = a file gains a generated-stale finding it did not
  have before, tested both ways.
* (5) `impact`: BTreeSet-sorted, deduplicated, pure read; dependants use edges whose `to`
  is a listed capability (`from` depends); linking sections via `linking_sections`;
  Unknown vs NotLoaded via `Workplan::change`. Tests compare against graph edges and the
  tree snapshot.
* (6) `SectionId` Display is `cap:<slug>#<section>`; `BindingKind::key` returns the same
  three strings as the two removed private copies; no behaviour change (existing bindings
  and roadmap tests pass).
* (7) Mutants, all killed by tests/lifecycle.rs: default affects dropped (6 fail), exclusive
  create -> truncating create (1), slug rule loosened (2), designed guidance wrong (4),
  dependants direction reversed (3).
* (8) Hygiene: SPDX headers present; no tracker numbers, process wording or attribution in
  added text; the tracker-shaped slug is built from two pieces.

## Findings

### REQUIRED

None.

### ADVISORY

* A1. A record written by `change new` fails `make reuse` in this repository until the
  author adds the two SPDX comment lines to its frontmatter (the author must also replace
  the placeholder title and description, so a hand edit is already part of the workflow,
  and the pre-commit reuse hook names the file). The contract gives no flag, and `docs sync`
  takes copyright/licence for the same reason. Not REQUIRED because the edit is available
  and unavoidable, but cheap to improve: optional copyright/licence inputs on
  `NewChangeRequest` (written as `# SPDX-...` frontmatter comments when set), or at least a
  line in the printed text saying the record needs its licence header. Decide with the CLI
  task, since a flag changes the cli contract.
* A2. For a `delivered` capability the printed reason ("a PROPOSED block is a finding in
  this state") is incomplete: the open record alone already yields
  `state-delivered-open-record` right after `change new` (the tests assert it). The advice
  is right; the reason could say so.
* A3. `sync_needed` is false when the target file was already stale before the record, so
  the message is silent although a sync is still owed; the field doc says "makes ... stale
  that was current", which is accurate.
* A4. Containment is checked with lstat on `work` and `work/changes` just before the
  `create_new`; a swap between the check and the open is not defended (workspace is trusted
  input per SECURITY.md). A failure leaves a newly created empty `work/changes` directory
  behind; harmless.
* A5. `impact.dependants` includes a pair where both capabilities are listed by the change;
  complete, but a reader may expect them excluded. Documented as "depend on a listed
  capability".

VERDICT: PASS
