# Historical-fit assessment: PR #39 (issue #3), raf-census core, normal form v1

Branch `paudley/3-raf-census-core-normal-form-v1` against `origin/main` (c5fc94f), 12 commits, 47 files.
Sources: stage inputs under `.stage/raf-census-core-normal-form-v1/`, `AGENTS.md`, `CLAUDE.md`,
`CONTRIBUTING.md`, `SECURITY.md`, `docs/architecture/normal-form.md`, `docs/capabilities/census-*.md`,
the port source `/home/paudley/Active/purrdf/crates/helper-census/src`, and `git log origin/main`.
No timing runs and no mutation passes were made. Findings state facts and a severity hint; none proposes a fix.
Severity hints: H (likely blocks a downstream port or breaks a stated rule), M (will cost rework or
misleads a reader), L (small or contained), I (information).

Paths below are relative to the worktree unless they start with `/`.

---

## 1. Precedent: how earlier crates were introduced

Evidence: `git log origin/main --reverse` (the first three commits are `3f8697b feat(raf-okf)`,
`57d9a06 feat(raf-source)`, `79cc6fb feat(raf-workplan)`); `work/assessments/okf-profile-*`,
`source-index-*`; `docs/capabilities/source-index.md`, `okf-profile.md`; `tests/docs_contract.rs`.

What the branch matches:
* Capability flips `designed` to `delivered`, with code, test and example bindings written out in full
  (`docs/capabilities/census-normal-form.md:8-216`); the same shape as `source-index.md` and
  `okf-profile.md`, including `raf_census::test_crate::...` test paths (152 of them, against 618 in
  source-index and 180 in okf-profile).
* The in-flight change record is deleted on completion (`work/changes/census-normal-form.md`, 28 lines
  removed), the inventory in `tests/docs_contract.rs` is updated (designed 9 to 8, delivered 12 to 13,
  assessments 56 to 62), `docs/capabilities/index.md` is regenerated, `AGENTS.md`, `README.md`,
  `CHANGELOG.md` and the workspace `members` list are updated. The clock gate in `tests/repo_gates.rs`
  covers `crates/*` without change, and raf-census names no clock.
* Six assessments with `verdict: accepted`, `by: agent`, as for the earlier crates.

### F1.1 (M) Four of six assessments carry the whole API surface as inputs
`work/assessments/census-normal-form-{purpose,constraints,examples,verification}.md` each list the same
104 `kind: symbol` inputs (the `name:` lists are identical), 325 lines and 13.5K apiece. The earlier
crates' assessments list the symbols of their section: 4 to 30 (`okf-profile-verification.md` 4,
`source-index-*.md` 13 to 30). The census contract and usage assessments list 23 and 9. Any later
signature change to any of the 104 symbols makes four assessments stale at once, and each of the seven
sibling-port changes will touch that surface. The four carry no prose; this matches precedent
(`by: agent`) but not its scale.

### F1.2 (M) `cap:census-normal-form` has `depends: []`
`docs/capabilities/census-normal-form.md:217` against `origin/main` (also `[]`). The delivered capability
is built on `raf-source` (`crates/raf-census/src/census.rs:46-67` `Workspace::load_with`;
`units/mod.rs:96-` `workspace.items_by_source`), and the sibling designed capability `census-ledger.md:13`
declares `depends: [cap:census-normal-form, cap:source-index]`. `workplan-document-model.md:325`,
`workplan-entity-history.md:57` and `workplan-two-way-coverage.md:49` all declare `cap:source-index`
where they read it. With the empty edge, a change to `cap:source-index` does not reach this capability in
the impact graph.

### F1.3 (L) Example reaches into the test tree
`crates/raf-census/examples/baseline.rs:12-13` includes `../tests/support/builtin.rs` through `#[path]`.
`raf-source`'s one example (`registry_scan.rs`) is self-contained. The example is bound
(`census-normal-form.md:214-216`), so a test-support edit moves a bound example.

### F1.4 (L) SECURITY.md and the fuzz harness were not extended
`SECURITY.md:262-264` still says `raf-okf`, `raf-source` and `raf-workplan` implement the parsing
surfaces and "the rest apply as the other crates are built". `raf-census` now parses untrusted `.rs`
text itself (`syn::parse_str` in `units/sig.rs:62-92`, `proc_macro2` lexing in `normalize/form.rs`) and
reads JSON reports (`report.rs:197-199`, `serde_json::from_str` with no size cap). `fuzz/fuzz_targets/`
holds `load_workspace`, `parse_document`, `parse_source`, `workspace_input` and no census target, and
`census-reference-scan.md` promises that "the `parse-source` fuzz target" applies to reference source. The
branch bounds nesting (`MAX_NESTING` 128, `normalize/form.rs:15`) and file size (`options.rs:12`).

---

## 2. Doctrine fit

### F2.1 (M) Goldens regenerated in place after they were pinned, on the rule "never change normal form 1 in place"
`CONTRIBUTING.md:23-27`: "Any change to what a fingerprint hashes requires a `normal_form` version bump,
updated golden tests, and a migration path. Never change normal form 1 in place." `CLAUDE.md` lists the
same as a Never. Commit `08071b8` pinned the goldens; `3f8f469` (numbering of locals after every
parameter) and `ab62795` ("a doc comment in a body lexes to an attribute whose word doc was numbered like
a local ... Normal form 1 is not yet released, so the goldens are regenerated without a version bump")
each moved fingerprints inside the branch. Nothing on `origin/main` contains the v1 encoding, so no
recorded fingerprint was invalidated; the commit message states the exception, the rule text states none.
The version-gate and golden machinery (`tests/golden.rs`, `normalize/mod.rs:323` `manifest`) did what it
was built to do (the goldens failed until regenerated); the open question is whether an unreleased
"not yet released" exception is doctrine.

### F2.2 (L) Scope moved from the plan in a way the change record no longer tracks
`plan.md` ("Scope boundaries") put job classification with `change:census-ledger` and said T1 would add
an item "The index report lists the classified matches of every job" there. The branch instead delivers
`Job`, `JobForbidden`, `JobMatch`, `JobReport`, `MatchClass` and `Census::match_jobs`
(`crates/raf-census/src/job.rs`, 188 lines) and removes the claim "Matches are classified as HOME,
VARIANT or COPY and counted per job" from `work/changes/census-ledger.md` (-1 line, no replacement item).
Result: `docs/capabilities/census-ledger.md` ("For each job the census classifies every matching item as
one of three roles", and the Verification line "A classification fixture shows one `HOME`, one
`VARIANT` and one `COPY`") and `cap:census-normal-form` (`docs/architecture/normal-form.md` "Jobs",
`:270-281`) both state the classification. Two capabilities own one rule, one designed and one delivered.

### F2.3 (I) The shim bound
The issue says 6-30; the branch follows the in-repo page, 6 to 29 inclusive, with fixtures at 5, 6, 29 and
30 (`plan.md` disposition 4). This is a recorded judgement, not a defect.

### F2.4 (I) Licence route
`shim_key` was implemented from the normative text, not from the AGPL katamari copy
(`plan.md` "Prior art"). The port source is `MIT OR Apache-2.0 OR MulanPSL-2.0`, the repository's own
licence. Files carry SPDX headers; `make reuse` passed in the PR checks (`raw/pr39-checks.txt`: `reuse` SUCCESS).

### F2.5 (I) No `GhpRsq-Defect-Class` trailer on any of the 12 commits
`git log origin/main..HEAD --format='%(trailers)'` is empty for all 12. On `origin/main`, 10 of 264
commits carry the trailer, every one with the value `none`. The trailer appears to be added at merge; the
history gives no evidence either way for a branch.

---

## 3. Recurring failure classes in this repository's history

Counts are over the 264 commits of `origin/main` (subject greps; 43 are `fix(...)`).

| Class | Evidence | Branch status |
|---|---|---|
| Capability or module text overstates or drifts from the code, and the capability is reopened | 8 "reopen" subjects (`f28f457`, `fab717c`, `9a8ec25`, `0e9a8e3`, `9629d58`, `527fc40`, `c60fa0d` and others), 12 "hardening fixes as the current contract" and 17 "deliver ... again" subjects (`92521b0` to `11c7d0c`), 2023926 | Repeats inside the PR: `be93118` "correct the normal form page and capability against the code" lists seven false statements (constant has no print, doc comment is attribute tokens, a unit ships only outside test code, the shim predicate looks at the top level, `min_tokens` comes from `CensusOptions`, unlexable macro arguments, `ref mut`). Five of six tasks were BLOCKED by their first review (`validation.md` table). |
| Work on hostile input grows faster than linear, then gets a counted budget | `1531b15 feat(raf-source): a work budget bounds the glob table work`, `6349c91`, `1f7a652 perf(raf-workplan)`, `af0ce46`, `c8e5187`, 10 perf/budget subjects; `938c967` and `d5793a1` count steps in raf-okf cost tests | raf-census adds no budget or step-count test. Linear scans were read (`normalize/passes.rs:169-215` placeholder rewrite scans forward once per `{` and jumps to the end on an unterminated `\u{`); no cost test exists (`grep -i 'budget\|steps\|cost'` over `src` and `tests` finds only `MAX_NESTING`). |
| Time-based tests are forbidden and keep creeping in | 20 "clock" subjects (`01a15d8`, `e9aeae3`, `980c6e2`, `88d9e42`, `fb79f72`, `6de7de3`) | Clean: the gate in `tests/repo_gates.rs:20-32` covers `crates/raf-census` automatically. |
| Order or iteration dependence in an index | 10 "order" subjects (`1c5a765` "its answer no longer depends on the order", `90d89a2`, `4bae3a7`) | Mitigated: every group, member and unit list is sorted (`group.rs:121-129`, `job.rs:164-168`), reports re-run byte-identical (`validation.md`). |
| Platform-specific process behaviour (macOS EPERM) | `0351571`, `0851241`, `1e5dc74` | Not applicable to this crate. |
| A late fix changes a previous commit's pinned numbers | Pairs `08071b8`/`3f8f469` and `a8452e5`/`2ec6a1e` and the `ab62795` goldens regeneration in this PR | See F2.1. |

### F3.1 (M) Documentation-drift class repeats and the full-gate has not been run
`validation.md` ends with "The workflow's single full `make check` is pending", and hosted CI is
`pending` with test, clippy, fuzz, msrv, deny, cross-check and test-windows/macos in progress
(`raw/pr39-checks.txt`). The history shows the same class surfacing late in review (the `reopen` chain on
`source-index`). The branch has already produced one self-correcting docs commit (`be93118`).

### F3.2 (L) Four of twelve commits are corrections of earlier commits in the same PR
`3f8f469`, `e1073f9`, `e02a444`, `2ec6a1e`, `ab62795`, `be93118` (six, including docs) fix behaviour added
a commit earlier. The shim-key match (`e02a444`) and the "index requires the body and shim keys"
(`2ec6a1e`) are shape decisions of the reports and jobs, the surface other changes consume.

---

## 4. Consumers and blast radius

Delivered public surface: `crates/raf-census/src/lib.rs:43-71`. Consumers are the designed capabilities;
there is no code consumer on `main`. The only in-repo consumers are the crate's own tests and examples.

### 4.1 census-ledger (Job)

#### F4.1 (H) `Job` cannot hold what a ledger job has, and is not `#[non_exhaustive]`
`job.rs:15-33`: `JobForbidden { fingerprints, constants }` and `Job { id, home, entry_points, variants:
Vec<String>, forbidden }` are plain `pub` structs. Of the three result types only `MatchClass` is
`#[non_exhaustive]` (`job.rs:114`); `JobForbidden`, `Job`, `JobMatch` (`:126`) and `JobReport` (`:139`)
are not, while `Unit`, `Group`, `Member`, `HexTable`, `SharedConstant`, `FileCtx` and `CensusOptions`
all are. `census-ledger.md` requires per job `summary`, `enforced`, `forbidden.names` (regular
expressions), `forbidden.patterns`, `[[job.variant]]` rows with `symbol, file, detector, criterion,
anchor, reason`, and `[job.ext]`. The ledger either wraps `Job` or adds fields to it; adding a field to a
struct that outside code builds with literals is a breaking change, so the change that adds the ledger
(version 0.x, "a breaking change bumps the minor version", `CHANGELOG.md:6-7`) has to break a type this
PR shipped. `Job::new` and the builder methods (`job.rs:48-79`) exist, but the struct literal is also
public, so it does not prevent that.

#### F4.2 (M) Matching cannot say why a unit matched
`JobMatch` (`job.rs:126-136`) holds symbol, file, line and class. The ledger's staleness checks
("a `body` or `shim` fingerprint matches no unit", "a variant's detector no longer matches anything")
need per-forbidden-entry results. `Job::matches` (`job.rs:82-98`) is private and returns a bool for the
four tests combined, so the ledger must repeat the same four tests or the shim-key rule to learn which
entry matched. A second copy of the shim-key rule is the failure this crate exists to prevent.

#### F4.3 (M) Names and patterns are outside the match
`forbidden.names` and `forbidden.patterns` are ledger features; `match_jobs` (`job.rs:158`) matches only
fingerprints and constants. The class of a name hit or a pattern hit as `COPY` has to be assigned by the
ledger with `classify` (`job.rs:100`, private). `copies` (`job.rs:139-148`) is computed inside
`match_jobs`, so a ledger that adds name hits cannot reuse it.

#### F4.4 (L) Home, entry-point and variant strings compared by equality
`classify` compares the unit's `symbol` with the strings (`job.rs:100-109`); the ledger text says a home
"must resolve, through re-exports, to a defined item". `Unit::symbol` is the symbol "a shipping target
reaches, then the smallest path" (`units/mod.rs:70-72`). The ledger must resolve home, entry points and
variants to unit symbols before it builds a `Job`; `Census` offers `unit`/`units_of` by exact symbol and
no resolver, and it does not keep the `Workspace` (F4.5).

### 4.2 census-rule-api (FileCtx), detectors, patterns, reference-scan

#### F4.5 (H) `Census` drops the `Workspace`; rules, patterns and detectors need parsed files
`census.rs:28-33` keeps `options`, `units`, `files` and `grouping`. `load` (`census.rs:46-67`) builds a
`Workspace`, derives the units and `FileCtx` rows and discards it. `census-rule-api.md` defines
`Rule::check(&self, ctx: &FileCtx, file: &syn::File, out)`; `census-patterns.md` matches tokens of a lexed
file; `census-detectors.md` reads syntax in shipping and all targets; the ledger's "anchor must resolve to
a documented item" needs the definition's `documented` flag (the port kept `documented` and `module` on
a `Definition`, `/home/paudley/Active/purrdf/crates/helper-census/src/source.rs:161-174`; the delivered
`Unit`, `units/mod.rs:60-100`, has neither). A Gate built on `Census` has to load the workspace again to
get parsed files and definitions, or `Census::load` has to change.

#### F4.6 (H) `FileCtx` cannot be constructed outside the crate
`census.rs:11-22`: `#[non_exhaustive] pub struct FileCtx { package, path, shipping }` with no constructor
(`grep 'impl FileCtx'` finds none); the only construction is `census.rs:54`. `census-rule-api.md` has the
repository write `Rule::check(&self, ctx: &FileCtx, file: &syn::File, ...)` and `Rule::fixtures()`; a
rule author who unit-tests `check` directly, and the gate that runs fixtures through `check`, need a
`FileCtx` for a snippet. Inside the crate the Gate can build one; every repository rule test cannot.

#### F4.7 (M) `shipping` is one bit where the rule API says shipping or test/bench/example
`FileCtx::shipping` is "true when some target that ships reaches the file other than through test code"
(`census.rs:19-21`). `census-rule-api.md` says the context says "whether it is shipping code or a test,
bench or example target"; `census-detectors.md` splits detectors into `shipping` (9) and `all` (5) reads.
A file reached by both a shipping and a test target is `shipping = true`. That fits "shipping" detectors
and loses which non-shipping kind a file is; the page and the capability define the bit, but the
downstream text has two readings of it.

#### F4.8 (M) Reference scan needs a second census with no way to compare
`census-reference-scan.md`: reference code is "read-only context", never in first-party groups, and a
first-party fingerprint equal to a reference fingerprint is a "reference copy" with "the reference symbol
it matches". `Census::load(tree, roots, options)` suits one `Census` per package directory; there is no
lookup of units by fingerprint (`Census` has `unit`, `units_of`, `units`, `groups`), and `Unit` has no
origin field. The caller can build the index from `units()`; the match rule, once more, is the caller's.

### 4.3 cli, migration, debt-ratchet (reports)

#### F4.9 (M) Reports fit migration and debt, with three constraints
* debt-ratchet record is `{kind, members: [symbols]}` per group plus `job_copies`: `Group::kind` (serde
  lowercase, `group.rs:52-60`), `Group::member_symbols` (`group.rs:78`) and `JobReport::copies`
  (`job.rs:139`) give it directly. Member sets are sorted (`group.rs:121`). Fits.
* migration needs, for an older-version index, "the members of a fingerprint are the units that list it":
  `IndexUnit` lists `body` and `shim` per unit (`report.rs:104-122`), `required` makes the keys mandatory
  (`report.rs:112-116`, `:137-144`). Fits. But `Index::from_json` is a closed schema of version 1
  (`#[serde(deny_unknown_fields)]`, `report.rs:98`) and never reads `normal_form`; an index written by
  a later normal form with a different shape is refused as a bad report, an index of the same shape under
  another `normal_form` is accepted. The migration path for "the index report written before the upgrade"
  therefore depends on the shape of `Index` staying fixed across versions; nothing says it will.
* The migration text bounds inputs at 1 MiB and 32 levels with an error that names the file;
  `Baseline::from_json` and `Index::from_json` (`report.rs:216-240`) take any `&str`, `serde_json`'s
  recursion limit is 128, not 32, and they cannot name a file. The bound is left to the caller.
* A legacy baseline (the other input of `migrate`) is a different closed schema (see F5.4); the delivered
  `Baseline::from_json` cannot read it, so migration needs its own reader.

#### F4.10 (L) The sorted-key renderer is private
`report.rs:188-195` `render` (sorted keys through `Value`, two-space indent, trailing newline) is the
byte-stability rule for `census-debt.json` ("a stable key order and a trailing newline"), `dump-ledger`
and `migrate`'s output; it is not exported, so each consumer writes its own. Byte-determinism of the
reports is guarded by a test (`tests/reports.rs:274` `output_has_sorted_keys_and_one_trailing_newline`),
and `serde_json`'s `preserve_order` feature would break it silently for every other writer; no manifest
enables it today (`Cargo.toml:26` has `default-features = false, features = ["std"]`).

#### F4.11 (L) `cargo census fingerprint <SYM>` prints "body and shim"
`Unit::print.shim` (a shim form exists) and `Unit::shim_key` (the id the unit groups under) differ for a
trait-implementation method and for a `Self::` forwarder (`units/mod.rs` `build`, `shim_key` filter).
The CLI text says "its `body` and its `shim`" and the migration text says the mapped fingerprints equal
what `fingerprint` reports. Which of the two is "the shim" is not fixed by the capability the CLI reads.

#### F4.12 (I) Blast radius inside the PR
`AGENTS.md`, `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md` and `SECURITY.md` mention raf-census; only the
first three were changed. `Cargo.lock` gains 75 lines (blake3 pinned `=1.8.7`, `Cargo.toml:17`); no other
crate pins a dependency with `=`. `deny.toml` needed no change (plan: licences already allowed).

---

## 5. Divergences from the purrdf port that the seven sibling copies will meet

The issue itself says the seven copies "use different hashes (BLAKE3 vs SHA-256), thresholds (20 vs 30)".
v1 is declared fresh, so every sibling fingerprint changes; the points below are the ones a sibling port
cannot predict from the purrdf text.

#### F5.1 (H) Token thresholds count a different form than purrdf
purrdf's `structural_form` drops borrows, derefs, `ref`, value adapters, full-range reindexes, `let`
annotations and type-only turbofishes while it builds the list it measures: `structural_level` skips the
`erased` tokens (`purrdf .../normalize.rs:268-295`), and `MIN_TOKENS`, `LITERAL_TIER` and
`SHIM_MAX_TOKENS` compare `structural.len()` (`:131-158`). v1 measures the form before `erase_held`
(`crates/raf-census/src/normalize/mod.rs:160-163`; page `normal-form.md:178` "before `erase_held`,
delimiters included"). A body with `&`, `.clone()` or `.as_str()` noise has more tokens in v1 than in
purrdf, so bodies near 20, 29/30 and 6 fall on the other side of a threshold. A sibling that keeps
`min_tokens = 20` gets different groups, not only different hashes, and the difference is not in the
changelog or the capability's explanation.

#### F5.2 (M) Hash input differs from purrdf's, not only the hash function
purrdf hashes `form.join(" ")` (`normalize.rs:105-111`); v1 hashes each token as a little-endian `u32`
length plus its UTF-8 bytes (`normalize/fingerprint.rs:16-27`, `ENCODING`). Siblings that already use
BLAKE3 (the issue lists them) will still see every fingerprint move. The encoding id is pinned in the
golden manifest, so the choice is guarded; the sibling-visible effect is that no recorded `body:` or
`shim:` id carries over and migration by member symbols (`census-migration.md`) is the only path.

#### F5.3 (H) Source that does not lex is an error, where purrdf skips it
purrdf `relex` returns an empty stream on failure (`purrdf .../source.rs:916-927`,
`.ok().unwrap_or_default()`); v1 returns `Error::Source` and stops the whole load
(`units/mod.rs:192-` `build`, `error.rs:14-21`, the `Census::load` doc "Nothing falls back to a partial
result"). That matches `CLAUDE.md` "fail hard" and the commit `e1073f9`, and it means a single body,
initialiser or signature in any of seven sibling workspaces that this lexer or `syn` rejects stops the
census for that repository.

#### F5.4 (H) Report shapes and keys differ from the purrdf documents the siblings' scripts read
* Baseline: purrdf writes `min_tokens, units, isomorphic_groups, shim_groups, hex_tables,
  shared_constants` with member objects from `unit_json` (`purrdf .../census.rs:593-606`); v1 writes
  `normal_form, min_tokens, units, body_groups, shim_groups, hex_tables, shared_constants` with members
  `{symbol, file, line}` (`report.rs:33-45`, `:84-97`). The key `isomorphic_groups` becomes `body_groups`,
  the per-member `package` is gone, and a shim group reports its structural `tokens` where purrdf reports
  0 (`purrdf .../census.rs:19`; `group.rs:98-110` `tokens: print.tokens`).
* Index: purrdf's index is ledger-driven (a `symbols` map of the ledger's home, entry-point, variant and
  anchor paths with `module` and `documented`, groups, jobs) and is "the document
  `scripts/check-shared-helpers.py` reads" (`purrdf .../census.rs:607-`). v1's index lists every unit,
  every group and the jobs the caller supplies (`report.rs:149-186`); it has no `symbols` map. The issue's
  R10 names "symbols, groups, job matches". The v1 `units` list carries `shipping`, `tokens`, `body` and
  `shim`, which migration needs; the resolution of ledger paths that the sibling script reads is not in it.
* Both v1 documents are `deny_unknown_fields` closed schemas, so a sibling reader of a purrdf baseline or
  index cannot be pointed at a v1 file or the reverse; migration's "legacy baseline" is a separate reader.

#### F5.5 (M) Shim grouping is narrower than the port's
purrdf groups every unit with a shim fingerprint (`purrdf .../census.rs:43-49`) and has no `shim_key`; v1
puts a unit in a shim group only when it has a `shim_key` (no trait-implementation methods, no `Self::`
forwarders; `units/mod.rs` `build`, `group.rs:160-175`). Sibling shim groups and ledger `[[distinct]]` rows
of kind `shim` shrink by every such member. This is intended (the katamari refinement) and stated in the
page (`normal-form.md:164-168`); the effect on sibling counts is not.

#### F5.6 (M) Unit kinds and the shipping rule changed
purrdf units are "function or constant" and decide test-only from `#[test]`/`#[cfg(test)]` attributes
(`purrdf .../source.rs:146`, `:963-974`); v1 adds `Method` and `Static`, takes `shipping` from
`raf-source` target reach (`units/mod.rs:96-` `Workspace::is_shipping`) and keys one unit per source
item rather than per path. Unit counts, `shipping` flags and the `symbol` of an item reached by several
targets (the shipping reach, then the smallest path) can differ from the port in a sibling that uses
`#[path]` or shared support modules.

#### F5.7 (L) What was not ported
`capacity.rs`, `asm_reports.rs`, `non_rust.rs`, `layout.rs` rule hits, `structure.rs`, `policy.rs`
and `glossary/` have no counterpart (scoped out in `plan.md` "Prior art"). Sibling census code that calls
those has no landing place until the detector, pattern and CLI capabilities are built; the capability
text for them is `designed`.

---

## 6. Summary of severity hints

| # | Hint | Subject |
|---|---|---|
| F4.1 | H | `Job`/`JobForbidden`/`JobMatch`/`JobReport` open structs; ledger needs more fields |
| F4.5 | H | `Census` keeps no `Workspace`; rule API, patterns, documented anchors need parsed files |
| F4.6 | H | `FileCtx` non-exhaustive with no constructor; rule authors cannot build one |
| F5.1 | H | thresholds count tokens before `erase_held`; purrdf counts after |
| F5.3 | H | unlexable source is a hard error; purrdf skipped it |
| F5.4 | H | baseline/index keys and members differ from the documents sibling scripts read |
| F1.1, F1.2, F2.1, F3.1, F4.2, F4.3, F4.8, F4.9, F5.2, F5.5, F5.6 | M | see sections |
| F1.3, F1.4, F2.2, F3.2, F4.4, F4.7, F4.10, F4.11, F5.7 | L | see sections |
| F2.3, F2.4, F2.5, F4.12 | I | recorded judgements and process facts |
