# PR #39: raf-census: normal form 1, fingerprints, grouping and reports

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Comments (2)

### paudley — 2026-10-20T21:58:10Z

# Plan: issue 3, raf-census core: normal form v1, fingerprints, grouping, reports

## Issue summary
Build the `raf-census` library crate's core: the normaliser (structural form, value form, literal
tier, shim form with the `shim_key` refinement), BLAKE3 fingerprints of four kinds, units extracted
from a workspace (fn, method, const, static with symbol, package, file, line, shipping flag, print,
integer constants by value, tables, strings), grouping, hex-table detection and shared large
constants, and the baseline and index JSON reports, byte-deterministic. The architecture page
`arch:normal-form` becomes the normative description of what was built; `cap:census-normal-form` is
reconciled and its change record completed with `cargo workplan change complete`.

## Baseline defaults / standing goals / governing decisions
* No `.baseline`, `.goals` or `CONSTITUTION.md` in this repository. Governing documents:
  `AGENTS.md` (canonical), `CLAUDE.md`, `docs/architecture/normal-form.md` (the "decision" the issue
  cites, NEEDS-JUDGMENT 2 of the issue ledger: its `## Decision` section is that decision; there are
  no ADR files in this repository), `docs/architecture/okf-profile.md`.
* Toolchain: floating nightly for gates, MSRV 1.98.1 on stable, no `#![feature]`, workspace lints
  (`unreachable_pub`, `missing_docs`, `unsafe_code` denied, clippy pedantic, every `allow` with a
  reason). No timing runs and no mutation passes anywhere; work is measured by counting.
* Fail hard, closed schemas, every rule with a negative fixture, a refusal is tested with its valid
  neighbour. Normal-form stability: any change to what a fingerprint hashes bumps `normal_form` and
  regenerates goldens.
* No tracker numbers or process wording in committed files; conventional commits; no attribution.
* Platforms: Linux primary; others minimal-effort.

## Prior art
* Port source: `/home/paudley/Active/purrdf/crates/helper-census` (22 files, all
  `MIT OR Apache-2.0 OR MulanPSL-2.0`, same copyright holder and licence as this repository: viable
  to port). In scope: `normalize.rs` (forms, fingerprints, value adapters, keywords, primitives),
  the unit extraction and relex parts of `source.rs` (`relex`, `is_test_only`, parameter names),
  the grouping and baseline/index parts of `census.rs`, and the hex-table and shared-constant
  detection from `layout.rs`/`rules.rs`. Out of scope here (other census changes): ledger
  (`ledger.rs`, `toml.rs`, census-ledger), detectors and patterns (`rules.rs` rule hits,
  `structure.rs`, `policy.rs`; census-detectors/patterns), `capacity.rs`, `asm_reports.rs`,
  `non_rust.rs` (purrdf-specific), `main.rs` (census-cli). purrdf's own workspace walker in
  `source.rs` is NOT ported: this repository walks through `raf-source` (`Workspace::items`,
  `Item::body`, `Workspace::source`, targets' `shipping`), as `AGENTS.md` states the census does.
* `shim_key` (katamari, `crates/katamari-helper-census/src/census.rs:57-61`) is AGPL-3.0-only:
  no code is copied. The rule is implemented from the normative text of `arch:normal-form`
  ("methods in a trait implementation" and "forwarders that call through `Self::`" have no shim
  key). The agent worktree copies under `katamari/.claude/worktrees/` are ignored.
* Hash route (prior-art NEEDS-JUDGMENT 6): the `blake3` crate (1.8.x) as the issue names; purrdf's
  `purrdf-hash` is not a dependency of this repository. `cargo tree` of `blake3` + `proc-macro2`
  + `serde_json`: every licence is already allowed by `deny.toml` (no new exception).
* Prior attempts: none (0 commits, 0 branches, 0 PRs). Recurrence: `none` 8 (trailer tally, not
  about this issue).

## NEEDS-JUDGMENT dispositions
Issue ledger:
1. R18 "Depends on #2": #2 is closed and delivered (`raf-source`, `raf-okf` are delivered crates);
   the dependency is satisfied. No action.
2. R16 "the decision": `arch:normal-form` `## Decision`; see governing decisions.
3. R4 vs R5 (literal tier vs shim form for small bodies): they are independent forms. Every body
   has a structural/value form; the literal tier only decides whether that form keeps literals
   (< 30 tokens) or abstracts them. A body of 6 to 29 tokens additionally has a shim form. A small
   forwarder therefore has both: a value form with literals verbatim (fingerprinted as `body` only
   if it reaches `min_tokens`) and a shim form (`shim`, unless `shim_key` excludes it). Stated
   normatively in the architecture page (T6).
4. R2 vs R3 (structural keeps punctuation, value drops `&`/`*`): the value form is the structural
   form with the value-form erasures applied. `body` fingerprints hash the value form (as
   `arch:normal-form` § Fingerprint says); `shim` hashes the shim form; `table` and `rule` are
   names, not hashes. The structural form is an intermediate and its token count (structural
   tokens) is what every threshold counts.
5. R13 fixture set: named in the completeness contract below (fixture modules under
   `crates/raf-census/tests/`).
Prior-art ledger: 1 → port source above; 2/3 → `shim_key` from the spec text; 4 → shim bounds are
6 to 29 inclusive (`MIN_SHIM_TOKENS` 6, `SHIM_MAX_TOKENS` 30 exclusive), the in-repo normative
text, since the issue's "6–30" is the two constants' names; fixtures pin 5, 6, 29 and 30;
5 → both documents exist and are the baseline: `arch:normal-form` is edited to state exactly what
was built (additions: unit kinds, shared constants, report shapes, ordering; any divergence found
while building is resolved in favour of the page unless the page is unimplementable, in which case
the page changes in the same commit with the reason); the capability text is reconciled through its
change record; 6 → `blake3` crate; 7 → none.

## Scope boundaries with other tracked changes
* The index report's "job matches" (R10): jobs, their HOME/VARIANT/COPY classification and their
  counting are the ledger's (`change:census-ledger` item "Matches are classified as HOME, VARIANT
  or COPY and counted per job"). This change delivers the index report with a `jobs` member whose
  rows are produced from a `JobMatch` list the caller supplies, with its shape, sorting and
  determinism tested on supplied matches; T1 adds to `change:census-ledger` the item "The index
  report lists the classified matches of every job" so the producer is tracked. The census-cli
  `index` command shows no jobs until the ledger exists; the report states the jobs it was given.
* "A ledger with a mismatched normal_form fails hard" (item 5 of `change:census-normal-form`): this
  change delivers the version gate `raf_census::check_normal_form(declared)` (error names both
  versions and `cargo census migrate`), tested with 2 and 0 failing and 1 passing; reading a ledger
  file is `change:census-ledger` (its item "A normal_form mismatch fails before any job is read").
  The change-record item is split: the gate `implemented` here, the ledger-file fixture `moved` to
  `change:census-ledger`. The capability Verification line is reworded to the gate.

## Completeness contract
| R | Requirement | Task | Acceptance check (real input, entry point, observed) |
|---|---|---|---|
| R1 | proc_macro2, bodies re-lexed, macro args normalised | T2 | golden `macro_arguments`: `format!("{x:02x}", x = b)` vs renamed local gives one value form; `Normalizer` called from `units::collect` on a `MemoryTree` workspace |
| R2 | structural form | T2 | goldens `structural_*`; renamed locals/params/lifetimes collapse; different called fn, method, field, type differ |
| R3 | value form | T2 | fixture per erasure (borrow, `&mut`, deref, `ref`, each of 11 adapters, `x[..]`, let annotation, type turbofish, `m[&k]`); binary `&`, `*`, `&&` kept (different fingerprint) |
| R4 | literal tier < 30 | T2 | 25-token pair differing in a string literal: different; 35-token pair: equal; boundary 29/30 |
| R5 | shim form 6-29, shim_key | T2, T4 | boundaries 5, 6, 29, 30; trait-impl method and `Self::` forwarder produce no `shim` group (T4 grouping over a MemoryTree workspace) |
| R6 | fingerprint `<kind>:` + 16 hex of BLAKE3[..8] | T2 | unit test against `blake3::hash` of a known form; `Fingerprint` parse refuses other kinds/widths (negative fixture) |
| R7 | units: fn, method, const, static; symbol, package, file, line, shipping, print, int constants by value, tables, strings | T3 | `units::collect` over a MemoryTree workspace with lib + test target: one unit per kind with every field asserted; `0xff` and `255` one constant value; test-target unit `shipping: false` |
| R8 | grouping BTreeMap, ≥2, ordering count desc then tokens then fingerprint | T4 | groups of one dropped; ordering fixture with three groups tying on count |
| R9 | hex tables lower/upper across string, byte-string, array; shared constants > 0xffff in ≥ 2 packages | T4 | one fixture per form and case; constant 0x10000 in two packages reported, 0xffff not, one package not |
| R10 | baseline JSON and index JSON, byte-deterministic | T5 | `Census::baseline(..).to_json()` and `index(..)`: exact JSON goldens over a fixture workspace; keys per the capability shape |
| R11 | normal-form.md normative v1 | T6 | page states every form, threshold, unit kind, ordering, report member; `docs_contract`/`self_workplan` pass |
| R12 | goldens pin every fixture, comment on bump | T2 | `tests/golden.rs` header comment; every fixture's fingerprint pinned |
| R13 | collapse/non-collapse fixtures | T2 | named fixtures listed in R2-R4 |
| R14 | twice → identical bytes | T5 | test runs `collect` + both reports twice from two fresh loads and compares bytes; plus over this repository's own `crates/` (real tree) |
| R15 | capability reconciled | T6 | `cargo workplan change complete census-normal-form` exits 0; state `delivered`; INVENTORY updated |
| R16 | fresh versioned normal form v1 | T2, T5 | `NORMAL_FORM == 1`; `check_normal_form(2)` error names `cargo census migrate` |
| R17 | capability delivered | T6 | as R15; bindings name the code, tests, examples |
| R18 | depends on raf-source | T1 | `raf-census` depends on `raf-source` by path |

Real entry point: `raf-census` is a library (the CLI is census-cli). Each check runs through the
public API `raf_census::Census::load(&tree, roots)` over `raf_source::MemoryTree` fixtures and over
`DiskTree` of this repository, plus a bound example `examples/baseline.rs` that prints the baseline
of a directory (run in T5 on this repository and on a fixture directory).

## Tasks (each: implement, independent review by a fresh reviewer, commit, push, issue update)

## Task 1: crate scaffold and change records
`crates/raf-census` (lib, `[lints] workspace = true`, SPDX headers, crate docs), workspace member,
deps `raf-source` (path), `proc-macro2` (span-locations off, default features), `syn` only if needed
for item kinds (else not), `blake3`, `serde`, `serde_json`; `Cargo.lock`; `make deny` passes.
`AGENTS.md` crate table/text: `raf-census` implementing; INVENTORY `census-normal-form`
`implementing`; PROPOSED blocks in `cap:census-normal-form` for the changed sections (the scope
boundaries above; Verification reworded); the added item in `change:census-ledger`; the
`change:census-normal-form` item split. Check: `cargo check -p raf-census`, `cargo test --locked
--test docs_contract --test self_workplan --test repo_gates`, `cargo workplan check`, `make deny reuse`.

## Task 2: the normaliser and fingerprints
`normalize` module: token source (re-lex body text with `proc_macro2`, recursing into groups,
macro arguments are groups), structural form, value-form erasures, literal tier, integer value
spelling-independence, shim form, `Fingerprint` type and `Kind`, `NORMAL_FORM`,
`check_normal_form`. Port from purrdf `normalize.rs` with its thresholds. Golden tests
`tests/golden.rs` (header comment: any change to a pinned value requires a `normal_form` bump) and
the fixture pairs of R2-R5, R13. Check: `cargo test -p raf-census --test golden --test forms`.

## Task 3: units
`units` module over `raf_source::Workspace`: fn, method (with `trait_impl`, `via_self` for
shim_key), const, static; symbol (item path), package, file, line, shipping (target), print (value
form fingerprint, shim fingerprint, token count), integer constants by value, string literals,
table candidates. Check: `cargo test -p raf-census --test units`.

## Task 4: grouping, hex tables, shared constants
`group` module: body groups and shim groups (shim_key), ordering, `table:hex-lower|upper`, shared
large constants. Check: `cargo test -p raf-census --test groups`.

## Task 5: reports and the example
`report` module: `Baseline` (normal_form, min_tokens, units, body/shim groups, hex tables, shared
constants) and `Index` (normal_form, units with symbols and fingerprints, groups, jobs from
supplied `JobMatch`es), `serde_json` with sorted, stable output and trailing newline;
`examples/baseline.rs`; double-run byte equality on fixtures and on this repository. Check:
`cargo test -p raf-census --test reports`, `cargo run -p raf-census --example baseline -- crates`
twice and `cmp`.

## Task 6: documents and completion
`arch:normal-form` normative v1 (all of R11), PROPOSED blocks resolved into the capability's
current text, bindings (code, tests, examples), `make docs-sync`, assessments re-read and accepted,
CHANGELOG present-tense entry, README/AGENTS state `delivered` for the capability where applicable,
`cargo workplan change complete census-normal-form`. Check: `cargo workplan check`, the three root
test binaries, `make fmt clippy doc deny reuse issue-refs`.

## Task 7: PR and handoff
Completeness review of the implementation against this plan and the issue, artifact inventory,
PR with `Closes #3`, the plan posted to issue and PR, least-confident note; Stage 2/3 gates listed
as unrun.

## Enhancements (superseded: see the Rev 1 section of the same name)
(See Rev 1.)

## Rev 1: integration of the five lens reports (authoritative over anything above that differs)

### Changed decisions
* **Job matches (R10; adversary escalation, design B4, completeness B3): option B, built here.**
  This change delivers `raf_census::Job` (id, home symbol, variant symbols, forbidden fingerprints,
  in memory) and `Census::match_jobs(&[Job]) -> Vec<JobReport>`: every unit whose `body` or `shim`
  fingerprint (or `table:` id) a job forbids is a match, classified `Home` (the unit's symbol is the
  job's home), `Variant` (a listed variant symbol) or `Copy` (anything else), counted per job; the
  index report's `jobs` member renders that output. The item "Matches are classified as HOME,
  VARIANT or COPY and counted per job" moves from `change:census-ledger` to
  `change:census-normal-form` in T1 (reading `[[job]]` tables from the ledger file stays with the
  ledger change, which constructs these `Job` values). No producer is missing: the example and the
  reports test build `Job`s through the public constructor and observe real classifications over a
  loaded workspace. The earlier "Scope boundaries" paragraph on job matches is superseded.
* **Version gate (adversary F6):** two separate change-record items: "The census implements
  normal_form 1 and `check_normal_form` refuses any other declared version with a message naming
  both versions and `cargo census migrate`" (implemented here; tested 0, 2 refuse with that text,
  1 passes), and "A ledger whose normal_form differs is refused before any job is read" (moved to
  `change:census-ledger`, where it already exists; the duplicate is removed from this record).
* **Shim candidate (completeness B2, design B5, adversary F2):** ported `shim_candidate` exactly:
  a body is a shim candidate when it is one expression (no `let`, no `;` before its tail), contains
  at least one call, and does not mention `self`, and it has 6 to 29 structural tokens. A unit with
  a shim form is not body-grouped ("a forwarder is never compared by structure"), so 20-29 token
  forwarders are in shim groups only. `shim_key` is computed once in `collect` as
  `Option<Fingerprint>` (None for trait-impl methods and `Self::` forwarders; design A7). Fixtures:
  only own parameters renamed (a forwarder renaming a non-parameter local differs); a different
  callee differs; a different fixed argument differs; two-statement and `let` bodies have no shim;
  boundaries 5/6/29/30; a trait-impl method and a `Self::` forwarder each with a shim-eligible
  neighbour that does group.
* **Normal form as a tower of passes (enhancement 1, TRANSFORMATION, adopted):** `normalize` is a
  `Form` (token tree whose tokens keep their source byte range) and a fixed sequence of pure passes
  (`rename_params`, `rename_locals`, `kind_literals`, `erase_held`); levels verbatim ⊑ shim ⊑
  literal-tier ⊑ structural ⊑ value. Tests: the monotonicity property over every registered fixture
  pair (equal at a finer level implies equal at every coarser one; token counts never grow);
  `Normalizer::explain(a, b) -> Divergence` names the finest level at which the two still differ
  and the first differing token with its source line; a pinned pass manifest (ordered pass ids,
  thresholds, encoding id) is a golden beside the fingerprints. Hashed bytes are the flattened
  final sequence.
* **Hashed encoding (enhancement 12):** each token is written as its UTF-8 length (u32 LE)
  followed by its bytes; the digest is BLAKE3 of the concatenation; the identifier is the first 8
  bytes as 16 lowercase hex digits rendered with `{:016x}` of the big-endian u64 (no hand-written
  hex loop or table: design A9). Stated normatively in the page; one fixture pins a hand-computed
  digest.
* **Units by source identity (design B1, enhancement 8):** collected through
  `Workspace::items_by_source()`; one unit per source item; the symbol is chosen deterministically
  (a shipping reach over a test reach, then the smallest path); `shipping` is true when any reach is
  in a shipping target; unit identity is `(file, line, symbol)`. Fixture: lib and bin both
  `mod shared;` → one unit, no group; and `tests/support/mod.rs` shared by two test binaries.
* **Token sources (design B2, enhancement 9, 10):** `syn` (the version `raf_source` re-exports) is
  a dependency from T1. Functions: body tokens from `Item::body`; parameters from parsing the
  signature text `span.start_byte..body.start_byte` with `syn`, every bound identifier of every
  pattern (`mut x`, `ref x`, tuple, struct patterns, `self`). Consts and statics: parse the item
  span with `syn` and take the initializer expression's tokens only (never the type or attributes);
  fixture: a const whose type names `0x10000` and whose initializer does not is not a shared large
  constant. Which `SymbolKind`s yield units: `Fn`, `Method` (inherent, trait-impl, trait default
  bodies), `Const`, `Static` (assoc consts included); every other kind yields none (test).
* **Fail hard (constraints B1, design B3, enhancement 11):** a slice that does not lex, a signature
  or initializer that does not parse, is `Error::Source { file, line, reason }`; tested by calling
  the normaliser on an unlexable slice and the unit builder on an unparsable signature, each with a
  valid neighbour that succeeds. Test-onlyness comes from `raf_source::TestRole` (not purrdf's
  `is_test_only`, which is not ported).
* **Entry point (completeness B1, B4, design A6, enhancement 15, adversary F10):**
  `Census::load(&impl Tree, roots, &CensusOptions) -> Result<Census, Error>` with
  `CensusOptions { min_tokens }` (default 20; 0 refused with `Error::Options`), built in T3; T2's
  checks go through `Normalizer` (its own public API), T3+ through `Census`. `min_tokens` fixtures:
  a 19-token body has no body fingerprint, a 20-token body has one, at the default and at 10.
* **Literal spellings (completeness B6, adversary F3):** golden pair: two short bodies differing
  only in `0xff` vs `255` (and `1_000` vs `1000`) fingerprint equal; a pair differing in value does
  not.
* **Inline format captures (enhancement 6, adopted):** in string literals of a macro's format
  argument, `{ident}` / `{ident:spec}` placeholders naming a renamed local or parameter are
  rewritten through the same rename map; `{}`, `{0}` and `{name}` naming a non-local are kept.
  Fixtures: renamed named argument (`format!("{x:02x}", x = b)` vs `y = c`), renamed inline capture
  (`format!("{n}: {c}")` vs `format!("{a}: {b}")`), each below and above the literal tier.
* **Lexical edges (enhancement 14):** `gen` and the weak keywords `union`, `raw`, `safe`,
  `macro_rules` are kept, not renamed; integer kind by syntax (an integer literal beyond `u128` is
  `$int`); nesting depth relies on `raf_source`'s delimiter budget (stated in the page, one fixture
  at the budget). Each pinned by a fixture.
* **Fingerprint type (design A8, completeness/adversary F12):** `Fingerprint` is an enum
  `Body(u64) | Shim(u64) | Table(TableKind) | Rule(RuleId)`; parse/display round-trip; accepted:
  one per kind (`table:hex-lower`, `rule:std-default-hasher`); refused: unknown kind, 15/17 hex
  digits, uppercase hex, unknown table name, empty rule id.
* **Collisions (enhancement 13):** grouping keeps the full 32-byte digest per unit; two units with
  one 8-byte fingerprint and different digests are `Error::Collision` naming both symbols; tested
  through the group builder with forged prints.
* **Ordering (completeness A, adversary F8):** groups sort by member count descending, then tokens
  descending, then fingerprint ascending; members by symbol, then file, then line. Fixture: groups
  differing on count; two tied on count differing on tokens; two tied on both, ordered by
  fingerprint.
* **Neighbours (constraints A4, adversary F9, F11):** hex tables: a 16-entry table with one wrong
  digit, a 15-entry table, and a mixed-case table are not reported; each form (str, byte str,
  `[char; 16]`, `[u8; 16]`) in both cases is. Const turbofish `::<3>` and partial range `x[1..]`
  are kept (different fingerprint from the erased neighbour).
* **Reports (enhancement 4, adversary F1):** exact members. Baseline: `normal_form`, `min_tokens`,
  `units` (count), `body_groups`, `shim_groups`, `hex_tables`, `shared_constants`. Index:
  `normal_form`, `units` (symbol, package, file, line, shipping, tokens, body, shim), `groups`
  (kind, fingerprint, members), `jobs` (id, home, matches with symbol and class, copies). Both
  `Serialize + Deserialize` with `deny_unknown_fields`; `to_json` → `from_json` round-trips;
  unknown-key negative fixture. JSON goldens are Rust string constants in test files (no new
  untracked-licence files).
* **Golden registry (completeness A, adversary F7, enhancement 7):** every fixture is one entry of
  one registry (`tests/support/fixtures.rs`); `tests/golden.rs` fails when a registry entry has no
  pinned value or a pinned value has no entry; `examples/goldens.rs` prints the table in the
  pinned form so a version bump is a paste. Header comment: any change to a pinned value requires a
  `normal_form` bump.
* **Example (adversary F5, completeness A):** `examples/baseline.rs` with no argument prints the
  baseline and index of a built-in `MemoryTree` fixture with one body group, one shim group and one
  job (what `change complete` runs); with a directory argument, of that directory. The check
  asserts the no-argument output contains the expected group fingerprints (not merely
  "identical twice").
* **Capability reconciliation (completeness B5, adversary F4):** T6 rewrites Examples with
  real output of `examples/baseline.rs` (fingerprints and keys pasted from the run), Usage names
  the library API that exists (`Census`, `Normalizer`, `Job`) and points to `census-cli` /
  `census-rule-api` as designed consumers, Verification lists the fixtures as built, and the
  reports test asserts the Examples' JSON keys. Six assessment records written and accepted for
  the affected sections.
* **Repository mechanics (constraints A1-A3, A5, A7-A9):** T1: INVENTORY state and the shape-test
  counts in `tests/docs_contract.rs`; `make docs-sync` in the same commit; `blake3 = "=1.8.7"`;
  AGENTS/CONTRIBUTING/README status lines made true. T6: verification-matrix marker added to the
  capability before `docs-sync`; the page states the current decision only (reasons go in commit
  messages). Every agent prompt carries the no-timing and no-mutation rule.
* **Full gate (constraints B2):** T6 runs `make msrv` (proves the crate and `blake3` on 1.98.1)
  with the focused gates. The workflow's single full `make check` runs after Stage 2 fixes settle
  (the repository's "gate before done"); the PR states it as pending.
* **span-locations (enhancement 3, design A10):** `proc-macro2` with `span-locations`; string
  literals carry absolute lines (`Span::byte_range` + body start), used by the units' strings.
* **Group and file context (enhancement 5, adopted):** one `Group { kind, fingerprint, tokens,
  members }` with `member_symbols()`; `Census::unit(symbol)`; `FileCtx { package, path, shipping }`
  as `cap:census-rule-api` names it, computed by `collect`.

### Tasks (numbered; each: implement, independent review by a fresh reviewer, commit, push, issue update)
* **T1** scaffold and records: crate, deps (`raf-source`, `syn`, `proc-macro2` span-locations,
  `blake3 =1.8.7`, `serde`, `serde_json`), workspace member, `Error`, `CensusOptions`; change
  record items (job matching moved in, version items split), PROPOSED blocks, INVENTORY and shape
  counts, docs-sync, status prose.
* **T2** normaliser: `Form`, passes, levels, thresholds, encoding, `Fingerprint`, `explain`,
  manifest, format captures, lexical edges, `check_normal_form`; registry, goldens, monotonicity,
  `examples/goldens.rs`.
* **T3** units and `Census::load`: by source identity, token sources, `FileCtx`, shim_key,
  integer constants, strings with lines, `min_tokens`.
* **T4** grouping, collisions, hex tables, shared constants, ordering, `Job` and `match_jobs`.
* **T5** reports with round-trip, `examples/baseline.rs`, determinism on fixtures and on this
  repository (`DiskTree` of `crates`).
* **T6** documents and completion (page, capability, assessments, marker, CHANGELOG, `make msrv`,
  `change complete`).
* **T7** PR and handoff.

## Enhancements Considered and Declined
* **Item 2, near-miss grouping by winnowed k-grams (TRANSFORMATION, declined).** Its `k`, window
  and threshold would be fixed by `normal_form = 1` with no measured data to choose them; any later
  tuning is a version bump that invalidates every ledger. The lesser structure suffices because the
  tower's `explain` makes the "same algorithm, reported distinct" condition observable per pair,
  which is the evidence the page's revisit condition asks for; a near-miss report becomes its own
  version-scoped change once that evidence exists. Cost: Type-3 clones stay invisible in v1
  reports.
* All other enhancement items (1, 3-15) adopted above.

### Dispositions index
Completeness B1-B6 → Entry point, Shim candidate, Job matches, Entry point/min_tokens,
Capability reconciliation, Literal spellings; its advisories → Ordering, Neighbours, Golden
registry, Example. Constraints B1 → Fail hard; B2 → Full gate; A1-A9 → Repository mechanics,
Neighbours, Reports (goldens as constants). Design B1-B5 → Units, Token sources, Fail hard, Job
matches, Shim candidate; A6-A10 → Entry point, Shim candidate, Fingerprint type, Hashed encoding,
span-locations. Adversary escalation → Job matches (option B, decided without a scope cut);
F1-F12 → Reports, Shim candidate, Literal spellings, Capability reconciliation, Example, Version
gate, Golden registry, Ordering, Neighbours, Entry point, Neighbours, Fingerprint type.

## Rev 2: fixes to the final reviewer's findings (authoritative)
* **Job matching (B1), as the final reviewer's REWRITE TO, adopted verbatim:** `raf_census::Job
  { id, home, entry_points, variants, forbidden: JobForbidden { fingerprints, constants } }` (in
  memory; the ledger change builds it from `[[job]]`) and `Census::match_jobs(&[Job]) ->
  Vec<JobReport>`: a unit matches when its `body` or `shim` fingerprint or `table:` id is in
  `forbidden.fingerprints`, or one of its integer constants equals by value an entry of
  `forbidden.constants`; a `rule:` id matches no unit in this change (detectors are
  `change:census-detectors`). `Home` when the unit's symbol is `home` or one of `entry_points`,
  `Variant` when a listed variant, `Copy` otherwise; `copies` is the `Copy` count. `names` and
  `patterns` stay with the ledger and patterns changes (regular expressions and pattern ids are
  theirs). Acceptance (T4 and the reports test): over one MemoryTree workspace, one `Job` whose
  fingerprints name the body group's fingerprint and whose constants name `0x10000`, with `home` on
  one copy, an `entry_points` symbol on a re-export of it and a `variants` symbol on a second copy,
  yields one `Home` (home), one `Home` (entry point), one `Variant`, one `Copy`, and the unit
  holding `65536` as a `Copy`; `copies: 2`; a job whose fingerprints match nothing has an empty
  match list. The index `jobs` member and the example's built-in fixture render exactly this.
* **Version-gate items (A2):** `change:census-normal-form` items after T1: (a) "Golden tests pin
  every fixture fingerprint" (implemented here); (b) "The census implements normal_form 1 and
  `check_normal_form` refuses any other version, naming both versions and `cargo census migrate`"
  (implemented here). The old combined item is removed from this record by rewording, not given a
  `moved` outcome: `change:census-ledger` already holds "A normal_form mismatch fails before any
  job is read", which is exactly the ledger-file half, so nothing leaves this record unowned.
* **Tower (A3):** below the literal tier (`< 30` structural tokens) `kind_literals` is the identity
  pass, so the final sequence keeps literals verbatim there (R4 holds); renaming numbers the
  function's own parameters first in signature order, then locals by first appearance in the body,
  one counter (`$0..`). The `body` print is the value-form sequence; the `shim` print is the shim
  form; both are stated once in the page's Fingerprint section.
* **Whole-crate tests (A4):** every task runs `cargo test -p raf-census --locked` (unit tests,
  integration tests and doctests), not only the named test binaries.
* **R11 falsifiable (completeness A5):** the T6 check is a test in `crates/raf-census/tests/page.rs`
  that reads `docs/architecture/normal-form.md` and asserts it names every pass id of the pinned
  manifest, every threshold with its value as the code's constants give it, every
  `VALUE_ADAPTERS` entry, every report member of both reports and the encoding id; a constant
  changed in code without the page fails it.
* **max_file_bytes (design A6):** `CensusOptions` is `#[non_exhaustive]` with `Default` and
  setters (`min_tokens`, `max_file_bytes`); `max_file_bytes` maps to
  `raf_source::LoadOptions.max_file_bytes` (default the ledger's 8 MiB), with a fixture: a file
  above the cap is a load error naming it, the neighbour at the cap loads.
* **T1 PROPOSED blocks (A-last):** name Purpose, Contract, Usage, Examples, Constraints and
  Verification (the six sections the change record affects); Contract carries the job matching,
  the version gate wording, the shim predicate and the report members.
* **Shim candidate's body print (A4 of the final review):** a shim candidate still has a `body`
  print when it reaches `min_tokens` (the index lists it), but it never joins a body group; only
  its shim print (when `shim_key` is Some) groups. Stated once in the page and in Contract.
* **Entry-point fixture (final review A8):** `Job` symbols are unit symbols (resolving a re-export
  to its item stays with `change:census-ledger`); in the job fixture the entry point is a second
  unit with the same body, not a `pub use`. Expected counts unchanged.
* **Page wording (final review note on A3):** T6 restates the page's "by order of first
  appearance" as: parameters in signature order, then locals by first appearance.


### paudley — 2026-10-20T21:58:16Z

**What I am least confident about, and why**
* **The normalisation choices now frozen into normal form 1.** Thresholds count structural tokens before erasures. Locals are numbered after the highest parameter number the body uses, placeholders included. Names inside attributes are kept. Integers are respelled to decimal before the literal tier. Each choice was adjudicated by an independent review and is pinned by a golden, but any later change to one of them is a `normal_form` bump plus a migration.
* **Near-miss (Type-3) duplicates.** Bodies with one statement added or reordered are not grouped. This was declined on purpose: its parameters would have to be frozen into v1 without data. `explain(a, b)` makes such misses visible pair by pair.

**What you may not know but should**
* The job-match classification moved from the ledger's change record into this change. The ledger change now only parses `[[job]]` tables into `raf_census::Job` and refuses a mismatched `normal_form`.
* The census reads `raf-source`'s parse. A file `raf-source` cannot load is a load error, never a skipped file.


