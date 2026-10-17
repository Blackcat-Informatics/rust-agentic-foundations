# Task 1 review: raf-okf record types and claim markers

## Scope

Staged, uncommitted change on origin/main (33 files): `raf-okf` record types
(Evidence, Assessment, Coverage), claim markers, the 26 finding kinds and the
workplan blast radius, the architecture page, the PROPOSED blocks of
`cap:okf-profile`, the two change records, INVENTORY. All experiments ran in a
scratch worktree (HEAD plus the staged diff, removed afterwards); the real
worktree was not touched.

## Evidence

* Gates in the scratch tree, all green: `cargo test -p raf-okf` (debug and
  release), `cargo test --workspace` (includes `docs_contract`, `repo_gates`,
  every pre-existing raf-workplan suite), `make fmt clippy doc reuse deny
  issue-refs licenses`, `make docs-sync` twice (no diff either time).
  (`docs_contract` fails in a scratch tree until the new files are indexed;
  that is an artefact of tracked-file reads, not of the change.)
* `cargo workplan check` on the tree: clean. `change reconcile okf-profile`:
  exit 0; lists only the two unresolved PROPOSED blocks, items without outcome
  and `bindings.examples is empty`. `verify okf-profile`: all 4 code + 21 test
  bindings resolve and pass.
* Blast radius: with a valid `work/evidence/x.md` and `work/coverage/default.md`
  added, `check` stays clean (LOADED constant) and only `docs_contract` reports
  "not in the inventory table", as designed. A claim in a capability passes
  `check`; one in a non-capability yields `okf-claim-outside-capability`
  (plain.rs test).
* Mutation (5 + 3 variants, all killed): over-limit comparison `>` to `<`/`>=`
  (records tests), heading boundary and list-item boundary of claim attachment
  (claims tests), unknown-marker refusal to `Ok(None)` (claims), the
  DocType-derived `listed` message hard-coded to three types (records), the
  crate-root exclusion arm disabled (records).
* Adversarial parse of claim markers (CRLF, tab indent, tab separator, no final
  newline, marker first in body, two markers, duplicate ids, 200k ids (refused
  by the 1 MiB bound), 1000 duplicate ids, 20k claims, 50k-line paragraph,
  unicode ids and unicode after `raf:`, short `<!-` lines, nested comments,
  multi-line comments, blockquote, fence, code span, PROPOSED, table,
  generated-end): no panic, no wrong attachment; cost linear (release 21 ms for
  900 KB).
* Schema fidelity: key sets and order in `schema.rs` match the architecture
  page table exactly for all three records; singleton `coverage:default`;
  crate-root exclusion and bare crate path refused; `[]` scope and `[]`
  exclusions allowed; subject, verdict, `by`, digest, revision rules match the
  plan. Every binding path in the repo satisfies the item-path pattern.
* Hygiene: no new dependency (no Cargo.toml in the diff), SPDX on all new
  files (reuse passes), no tracker numbers, process wording or attribution in
  added lines.

## Judgement of the implementer's choices

* No title/description/body on records: correct; body-empty rule is enforced
  and documented.
* Optional `values`/`limits` (no empty map in the YAML subset): correct.
* Evidence inputs symbol-only: matches D4 (code-binding digests). Subject
  `command` vs `binding:<path>`, `command`, `revision`, `values`, `limits`,
  `for`, `by` and per-input digests cover everything D4 and D6 need (a limit
  present at the merge base and absent now is expressible).
* Assessment must hold the section row of its subject; `record` and `symbol`
  rows allowed: matches D8.
* `limits` must name a value, `pass` cannot hold an over-limit value: matches
  D4 ("over limit => fail") and D12; D6's "value now above its limit" is a
  `fail` record. Not over-strict for Tasks 3-6.
* Mid-line `raf:generated:` kept as text: consistent with pre-existing
  start-of-line scanning, now stated on the architecture page.

## Findings

### REQUIRED

None.

### ADVISORY

1. A case or spacing variant of the generated marker (`<!-- RAF:GENERATED:x -->`,
   `<!--raf:generated:x-->`) is neither a generated block nor refused: it is
   silently text. This is pre-existing behaviour, but the new page sentence "a
   misspelt marker is never read as a plain comment" is only true outside
   `raf:generated:` comments. Consider refusing a `raf:generated:` comment that
   is not in exact marker position/spelling, or narrowing the sentence.
2. Marker scanning is per line: `<!--` newline `raf:claim evidence=a` newline
   `-->` is not seen (silent plain comment); `    <!-- raf:x -->` inside an
   indented code block is refused (the body model has no indented code); a
   claim under a setext heading attaches to the heading text and underline (only
   ATX headings stop a statement); unknown markers inside a PROPOSED block are
   not refused until it is resolved. Each is edge-case and consistent with the
   stated model; the page says "leading spaces" where tabs are also accepted.
3. `Number` accepts no exponent and a 400-digit literal parses to infinity.
   Task 5 must state that a tool printing `1e-3` hard-fails (D12) and decide on
   range.
4. An evidence record needs a non-empty `inputs` of symbol rows, so `--for` a
   capability with no code binding cannot be recorded; Task 5 needs a clear
   refusal. Scope duplicates are compared as written (`raf-okf` and `raf_okf`
   are distinct); Task 7 owns the name mapping.
5. For Task 11: `reconcile okf-profile` reports `bindings.examples is empty`
   (index Gaps column shows it too), and `verify workplan-document-model` fails
   on the existing example binding `raf_workplan::Gate` (a `no_run` doctest;
   file untouched by this change, so pre-existing). Both must be resolved
   before `change complete` of those records.
6. `tests/roadmap.rs` now asserts only that some capability is delivered plus
   that `okf-profile` and `source-index` are listed; weaker than before but
   the honest consequence of the state flip.

VERDICT: PASS
