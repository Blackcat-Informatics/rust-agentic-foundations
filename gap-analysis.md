# Gap analysis: PR #39 (issue #3), raf-census core, normal form 1

**Issue:** #3 (raf-census core: normal form v1, fingerprints, grouping, reports)
**Branch:** `paudley/3-raf-census-core-normal-form-v1` at `be93118`
**Base:** `origin/main` (`c5fc94f`), 12 commits, 47 files changed (+9862 / -153)

Inputs: `issue.md`, `issue-analysis.md` (R1-R18, 0 comments), `prior-art-assessment.md`,
`historical-fit.md`, `plan.md` (Rev 2 authoritative), `validation.md`, `tasks/*`,
`branch-under-review.md`, `raw/pr39-*`, `completion-audit.md`, AGENTS.md, CLAUDE.md,
CONTRIBUTING.md, SECURITY.md, `docs/architecture/normal-form.md`, `docs/capabilities/census-*.md`,
`work/changes/*`, and the crate source read in full where cited.

User decisions applied (relayed by the coordinator, verbatim): "Note - breaking the API is
encouraged". A finding of the form "a later change would have to break this API" is therefore not
a gap by itself; API shape is judged on whether it is right for its consumers now. For the
completion audit's F2, the issue text governs.

Probes: a throwaway test in an ignored scratch clone
(`$S/scratch/gap-probe/crates/raf-census/tests/zz_gap_probe.rs`, never committed; the worktree is
clean). No timing runs, no wall-clock assertions and no mutation passes were made.

## Counts

| Severity | Count | Gap ids |
|---|---|---|
| CRITICAL | 2 | GAP-01, GAP-02 |
| HIGH | 4 | GAP-03, GAP-04, GAP-05, GAP-06 |
| MEDIUM | 5 | GAP-07, GAP-08, GAP-09, GAP-10, GAP-11 |
| LOW | 6 | GAP-12, GAP-13, GAP-14, GAP-15, GAP-16, GAP-17 |
| Total | 17 | |

GAP-01 to GAP-04 change what a fingerprint hashes. They must land together, before the first
merge, with one regeneration of the goldens and the capability Examples, under the decision that
GAP-07 asks for.

## Numbered gap list

### GAP-01 [CRITICAL] Item names used as values are renamed as locals, so different algorithms share a body fingerprint
*Source: completion audit F1, verified first-hand. Requirements R2, R11, R13.*

- **Evidence:**
  - `crates/raf-census/src/normalize/passes.rs:73-83`: `names_an_item` keeps a name only when it
    is a path segment, is followed by `(` or `!`, is capitalised, or is a primitive.
  - `passes.rs:378-403`: `Namer::ident` numbers every other identifier.
  - So `map(double)` and `map(triple)` both become `map ( $1 )`. A lowercase `static` or `const`
    read by name is renamed in the same way.
  - Audit probe P01: two bodies that map and filter through different functions get the same
    fingerprint, `body:44af1080ba7e6a41`, and form one group. P02 (two different lowercase
    statics) also forms one group.
  - The issue (R2) requires "names of items the body uses kept".
  - The normative page states the opposite of what the code does:
    - `docs/architecture/normal-form.md:91-92`: `rename_locals` renames "every other local name".
    - `normal-form.md:102`: "paths … are kept". A single-segment path expression is a path.
  - The result is a false group, which is the worse failure: a ledger job that forbids such a
    fingerprint matches unrelated code.
- **Required fix:**
  - Rename only the names that a body or its signature binds:
    - parameters;
    - names bound by `let`, closure parameters, `for`, `match` arms, `if let` and `while let`
      patterns;
    - labels, lifetimes and local type declarations.
  - Keep every other identifier verbatim as an item name.
  - State the rule on the page and in the capability Contract and Constraints.
- **Acceptance check:**
  - Registry pairs in `tests/support/fixtures.rs`, checked by
    `every_registered_pair_compares_as_registered`:
    - `map(double)` vs `map(triple)` is Distinct;
    - a lowercase static read vs another lowercase static is Distinct;
    - the same bodies with renamed closure parameters and `let` bindings are Equal(Value).
  - A `groups.rs` test over `Census::load` shows that the two different-function bodies are in no
    common group, beside a renamed-local neighbour that does group.
  - The goldens are regenerated and `the_tower_is_monotone_over_every_registry_pair` passes.

### GAP-02 [CRITICAL] A block after a capitalised name is read as a struct literal, so its locals are kept
*Source: completion audit F3, verified first-hand. Requirements R2, R11.*

- **Evidence:**
  - `passes.rs:98-107`: `opens_fields` treats any `{..}` after a capitalised identifier as a
    struct literal or pattern.
  - `passes.rs:88-96`: `names_a_field` then keeps a name at the start of that block, through
    `params_level` (`:245`) and `Namer::level` (`:362-363`).
  - In `if next == MAX { next } else { fallback }`, `next` inside the block is kept verbatim.
  - Audit P03/P15: `clamp_a` and `clamp_b`, which differ only in local names, get different
    fingerprints.
  - The page claims at `normal-form.md:110-111`: "Two bodies that differ only in the names of
    their parameters and locals have the same structural form". This claim is false as delivered.
- **Required fix:** do not open field position for the block of an `if`, `else if`, `while`,
  `match` or `for … in` head, where Rust admits no struct literal.
- **Acceptance check:**
  - A registry pair `if x == MAX { x } else { y }` vs the same with renamed locals is Equal(Value).
  - A neighbour with a real struct literal `S { x }` keeps `x` (Distinct when the field differs).
  - A `Census::load` test shows the two clamps in one body group.
  - The goldens are regenerated.

### GAP-03 [HIGH] Parameters are numbered in signature order, where the issue says first appearance; the governing Decision text was rewritten
*Source: completion audit F2. The issue governs (user decision relayed). Requirements R2, R11, R16.*

- **Evidence:**
  - `passes.rs:223-236`: `rename_params` numbers in signature order.
  - `passes.rs:268-302`: locals start after the highest parameter number used.
  - The issue (R2) says: "locals, parameters and lifetimes become `$0..` by first appearance".
  - The page's Decision on `origin/main` (`normal-form.md:26-27`) says "Local names are renamed by
    order of first appearance". The branch rewrote it to "Local names are renamed"
    (`normal-form.md:27`), and `:87-95` now states signature order.
  - Audit P13: `count_a(xs, k)` and `count_b(k, xs)` have byte-identical bodies but different
    fingerprints, so copies that reorder their parameters are missed.
- **Required fix:**
  - Number parameters and locals together, by first appearance, from the literal-tier level
    upward. This is a pure function of the shim form, so monotonicity holds.
  - Keep signature order at the `shim` level only. A forwarder's argument order relative to its
    signature is its meaning: `f(y, x)` and `f(x, y)` must stay distinct shims.
  - Restore the Decision wording, and restate the rule on the page and in the Contract.
- **Acceptance check:**
  - The registry pair `count_a`/`count_b` is Equal(Value).
  - The forwarder pair `fn a(x, y) { f(y, x) }` vs `fn c(x, y) { f(x, y) }` is Distinct at the
    shim level.
  - The monotonicity test passes, the goldens are regenerated, and `tests/page.rs` passes.

### GAP-04 [HIGH] Thresholds count tokens before the value-form erasures, so noise-only copies split across `min_tokens` and the literal tier
*Requirements R3, R4, R13. Disposes of historical-fit F5.1 and NEEDS-JUDGMENT 4.*

- **Evidence:**
  - Tier decided before erasure:
    - `crates/raf-census/src/normalize/mod.rs:161-163` decides the literal tier on
      `literal_tier.len()`, before `erase_held`.
    - `mod.rs:176-180` and `:192-205` decide `min_tokens` and the shim bounds on the structural
      count. The page states this rule at `normal-form.md:178-179` and the capability at
      `census-normal-form.md:253-254`.
  - Probe, through the public `Normalizer` API:
    - `let n = v.len(); if n > 3 { return 7; } n + 1` (21 structural tokens) and the same body
      with `.clone().as_slice().to_owned()` (33 structural tokens) both have 21 value tokens.
    - Their value forms differ (`… > 3 { return 7 ; } … + 1` vs `… > $int { return $int ; } … + $int`),
      and `explain` gives `equal_from=None`.
    - Control: the same noise below the tier collapses (`equal_from=Some(Value)`).
    - Probe 2: `if v.len() > 3 { 1 } else { 0 }` (15 tokens) has no body fingerprint. The same
      body with `.clone().as_slice()` (23 tokens) has `body:7ed517820c72f5a5`.
  - So two copies that differ only in the noise the value form exists to erase get different or
    missing fingerprints when the noise pushes one across 20 or 30. Helper-sized bodies sit in
    exactly that band.
  - R13's fixtures pass only because none of them straddles a threshold. The PR's
    least-confident note lists this choice first.
- **Required fix:**
  - Count every threshold on one count, the length of the value-level form. This is
    `erase_held` of the literal-tier form, and the literal-tier decision is taken on it before
    `kind_literals`.
  - Then equal value forms always mean equal threshold decisions, and monotonicity still holds.
  - Update the page Thresholds section, the capability Contract, `Print::tokens`, `Group::tokens`
    and the index `tokens` wording.
- **Acceptance check:**
  - The two probe pairs are registered:
    - the tier straddle is Equal(Value);
    - for the `min_tokens` straddle, both bodies have a body print or neither does, at the
      default and at 10.
  - `tests/page.rs` asserts the counting rule.
  - The goldens are regenerated and the monotonicity test passes.

### GAP-05 [HIGH] `Census::load` aborts the process on source that raf-source admits, because signatures are re-parsed with `syn` on the caller's stack
*Requirements R7 (token sources). Fail-hard doctrine. SECURITY.md parse-stack design.*

- **Evidence:**
  - Re-parses on the caller's thread:
    - `crates/raf-census/src/units/sig.rs:55-75`: `syn::parse_str::<ItemFn | ImplItemFn | TraitItemFn>`
      of the whole signature.
    - `sig.rs:103-123`: the same for whole `const` and `static` items.
    - Both are called from `units/mod.rs:219` and `:229`, inside `Census::load`
      (`census.rs:50`), on the caller's thread.
  - raf-source budgets parse recursion for a dedicated 64 MiB stack:
    - `crates/raf-source/src/parse.rs:18` sets `PARSE_STACK_BYTES`;
    - `budget.rs:83-89` admits 64 nested `<` per delimiter level and recursion 1927.
  - Probe results:
    - **2 MiB thread** (the default `#[test]` stack, where the designed `Gate::assert_clean`
      runs): a parameter type of 60 nested generic arguments aborts with
      `fatal runtime error: stack overflow` (SIGABRT). The same type inside a body loads.
    - **8 MiB thread** (main-thread size): 8 tuple levels × 60 generics also aborts. Yet
      `raf_source::Workspace::load_with` loads that tree on a 2 MiB thread.
    - At 24 × 60, raf-source refuses with its recursion budget.
  - Second symptom of the same placement: the census lexes every body, signature, initialiser
    and string literal on the caller's thread.
    - `proc-macro2` with `span-locations` keeps each lexed text in a thread-local source map
      (`proc-macro2-1.0.107/src/fallback.rs:464-480`, `u32` offsets).
    - The caller's thread therefore keeps roughly the workspace's source text after every load.
    - raf-source avoids this by parsing on per-file threads.
  - The capability Constraint `census-normal-form.md:597-598` ("Nesting depth is bounded by
    `MAX_NESTING` (128) and by the delimiter budget of `raf-source`") does not hold for this
    parse.
- **Required fix:** do one of the following, then make the Constraint and the page's Token
  source state the bound that holds.
  - Run all of the census's `proc_macro2` and `syn` work for one load (`units::collect`) on a
    scoped thread whose stack is `raf_source::PARSE_STACK_BYTES`.
  - Or take the parameter bindings and initialiser ranges from raf-source's own parse, so there
    is one parse and one path.
- **Acceptance check:**
  - A `units.rs` test runs `Census::load` inside a thread built with a 2 MiB stack:
    - the 8 × 60 signature returns `Ok` with one unit;
    - the 24 × 60 signature returns `Error::Load` naming the recursion budget.
  - The existing suites pass.

### GAP-06 [HIGH] Classification of name, pattern and rule matches lost its tracked owner when the ledger item moved here
*Disposes of historical-fit F2.2 and F4.3. Repository "no deferrals" rule.*

- **Evidence:**
  - The branch removed "Matches are classified as HOME, VARIANT or COPY and counted per job" from
    `work/changes/census-ledger.md`. The diff is -1 line, and the item was completed under
    `change:census-normal-form`.
  - What was delivered (`job.rs:82-108`, `census-normal-form.md:306-313`):
    - matching covers `body`, `shim_key` and `table:` ids plus integer constants;
    - `rule:` "matches no unit";
    - `forbidden.names` and `forbidden.patterns` are not matched.
  - `cap:census-ledger` still classifies "any other match of the job's `forbidden` entries"
    (`census-ledger.md:92-98`, Verification `:194`). Those entries include name regexes, pattern
    ids and rule ids. The CLI contract prints such a match with its reason
    (`census-cli.md:98`, `[name ^hex_encode.*]`).
  - Plan Rev 2 says names and patterns "stay with the ledger and patterns changes". Yet:
    - no item of `change:census-ledger` (`:18-24`), `change:census-patterns` (`:18-21`) or
      `change:census-detectors` (`:18-22`) owns classifying them;
    - the ledger record's description (`:7`) still says it delivers "job classification".
  - Plan Rev 2 also assigns to the ledger the resolution of home, entry-point and variant paths
    to unit symbols, and no item owns that either.
- **Required fix:**
  - Add tracked items to `work/changes/census-ledger.md` covering:
    - (a) `forbidden.names`, `forbidden.patterns` and `rule:` hits are classified HOME, VARIANT
      or COPY and counted with the fingerprint and constant matches, a unit counted once;
    - (b) home, entry-point and variant paths resolve through re-exports to unit symbols before
      jobs are matched.
  - Correct the record's description.
  - State in `cap:census-ledger`'s classification paragraph that fingerprint and constant
    matching is `cap:census-normal-form`'s, and that the ledger extends it.
- **Acceptance check:**
  - `cargo workplan --root . check` exits 0.
  - The `docs_contract` and `self_workplan` tests pass.
  - A grep of `work/changes/` finds an item that owns name, pattern and rule classification.

### GAP-07 [MEDIUM] Regenerating normal form 1's pins before the first merge has no stated basis in doctrine (user decision required)
*Disposes of historical-fit F2.1.*

- **Evidence:**
  - CLAUDE.md lists as a Never: "change what a census fingerprint hashes without bumping
    `normal_form`".
  - `CONTRIBUTING.md:23-27` says "Never change normal form 1 in place".
  - Commits `3f8f469`, `e1073f9` and `ab62795` regenerated pinned values without a bump. The
    commit message reasons "Normal form 1 is not yet released".
  - GAP-01 to GAP-04 move pins again.
  - No ledger, debt record, release or `origin/main` commit carries a v1 fingerprint. The rule's
    stated purpose ("ledgers and recorded debt stay valid") is not engaged, but the rule text has
    no exception.
  - The completion audit also leaves this to the owner.
- **Required fix:** obtain and quote in the PR the owner's decision. It is one of:
  - normal form 1 is defined at its first merge, so pre-merge regeneration keeps `NORMAL_FORM = 1`;
  - or the version is bumped.

  Then regenerate the pins once, in the same change as GAP-01 to GAP-04. If the decision makes a
  rule of when a version is defined, state it in CONTRIBUTING.md as a rule, not as history.

  An agent's judgment cannot close this gap.
- **Acceptance check:**
  - The decision is quoted on the PR.
  - `tests/golden.rs`, `examples/goldens.rs` output, `NORMAL_FORM` and the capability Examples
    agree under it.
  - `the_pinned_version_is_the_implemented_one` passes.

### GAP-08 [MEDIUM] `cap:census-normal-form` declares `depends: []` though it is built on the source index
*Disposes of historical-fit F1.2. Requirements R15, R17, R18.*

- **Evidence:**
  - `docs/capabilities/census-normal-form.md:217` declares `depends: []`.
  - The delivered API exposes and uses raf-source:
    - `Census::load(tree: &impl raf_source::Tree, …)` (`census.rs:46-49`);
    - `Error::Load(raf_source::LoadError)` (`error.rs:45`);
    - units come from `Workspace::items_by_source` (`units/mod.rs:128`).
  - `census-ledger.md:13` and the workplan capabilities that read the index declare
    `cap:source-index`. With the empty edge, a change to `cap:source-index` does not reach this
    capability in the impact graph.
- **Required fix:**
  - Set `depends: [cap:source-index]`.
  - Run `make docs-sync`.
  - Re-accept any assessment whose digest moves.
- **Acceptance check:**
  - `cargo workplan --root . check` exits 0.
  - The `docs_contract` and `self_workplan` tests pass.

### GAP-09 [MEDIUM] `blake3` is pinned exactly, which constrains every consumer's lockfile and guards nothing
*Disposes of historical-fit F4.12.*

- **Evidence:**
  - `crates/raf-census/Cargo.toml:15-17` pins `blake3 = "=1.8.7"`, "pinned to the exact version
    the normal form was built and its goldens recorded against". BLAKE3's output is fixed by its
    specification.
  - A nonconforming implementation is already caught by
    `the_digest_is_blake3_of_length_prefixed_tokens` and the goldens.
  - raf-census is a dev-dependency, which shares the consumer's lockfile. The exact pin forces
    every consumer workspace to 1.8.7:
    - sibling locks hold katamari 1.8.7, sparrow 1.8.7 and lillith_memetics 1.8.5;
    - any consumer dependency that later needs 1.8.8 cannot resolve.
  - Meanwhile `proc-macro2`, whose lexing defines the tokens, floats at `"1"` (`Cargo.toml:20`).
    The page lists that drift as a revisit condition (`normal-form.md:356-357`).
- **Required fix:**
  - Change to `blake3 = "1.8"` and correct the comment.
  - Not required: a fail-hard canary that hashes a fixed source at load and refuses a build
    whose token source moves the result would guard the real drift.
- **Acceptance check:**
  - No `=` requirement remains in `crates/raf-census/Cargo.toml`.
  - `make deny` and `cargo test -p raf-census --locked` pass.

### GAP-10 [MEDIUM] SECURITY.md and the fuzz harness do not cover the census's own parse surface
*Disposes of historical-fit F1.4. Related to GAP-05.*

- **Evidence:**
  - `SECURITY.md:262-264` names `raf-okf`, `raf-source` and `raf-workplan` as the crates
    implementing the parsing surfaces, "and the rest apply as the other crates are built".
  - raf-census now has parse surfaces of its own:
    - re-lexing (`normalize/form.rs:133`);
    - `syn` re-parsing (`units/sig.rs`);
    - literal decoding (`units/scan.rs:85`);
    - `from_json` with no size bound (`report.rs:197-199`).
  - `fuzz/fuzz_targets/` has no census target. The stack-exhaustion class of GAP-05 is what the
    existing `load_workspace` / `workspace_input` harness exists to find.
- **Required fix:**
  - State raf-census's surfaces and bounds in SECURITY.md (after GAP-05: the parse stack it uses;
    report reads are bounded by the caller).
  - Add a fuzz target that runs `Census::load` over `workspace_input`, with the GAP-05 reproducer
    in its committed corpus.
- **Acceptance check:**
  - `make fuzz` runs the new target over its corpus.
  - The `repo_gates` and `docs_contract` tests pass.

### GAP-11 [MEDIUM] The full gate has not run, and hosted CI was pending
*Disposes of historical-fit F3.1.*

- **Evidence:**
  - `validation.md:23-24`: "The workflow's single full `make check` is pending".
  - `raw/pr39-checks.txt`, state `pending`:
    - `test`, `clippy`, `test-release`, `fuzz`, `msrv`, `deny`, `cross-check`, `test-windows`
      and `test-macos` are in progress;
    - only `reuse`, `fmt` and `doc` have succeeded.
  - The repository rule is "Gate before you claim done: `make check`".
- **Required fix:**
  - After the fixes above settle, run `make check` once at the final head.
  - All Linux hosted checks must be green. A non-Linux platform failure is handled as CLAUDE.md
    states.
- **Acceptance check:**
  - `make check` exits 0 at the final head, and the run is recorded in `validation.md`.
  - `gh pr checks 39` shows every Linux job SUCCESS.

### GAP-12 [LOW] `from_json` accepts a report of any `normal_form`, and nothing says so
*Disposes of historical-fit F4.9 (first part).*

- **Evidence:**
  - `report.rs:211-219` and `:232-240` document the refusals (unknown key, missing key, wrong
    type) but read `normal_form` without checking it.
  - Probe: a baseline rewritten to `"normal_form": 7` reads back as `Ok(7)`.
  - Accepting is right for migration, which reads an older version's index. A caller that
    compares fingerprints must gate the version itself.
- **Required fix:**
  - State on both `from_json` functions, and in the page's Reports section, that reading does
    not gate the version and that a comparer applies `check_normal_form(report.normal_form)`.
  - Pin the rule with a test.
- **Acceptance check:** a reports test reads a report of another version, and
  `check_normal_form` on its `normal_form` refuses it.

### GAP-13 [LOW] Sorted report keys depend on serde_json's `preserve_order` feature staying off in the consumer's build
*Disposes of historical-fit F4.10.*

- **Evidence:**
  - `report.rs:188-195`: `render` relies on "A `Value` object is a sorted map". That holds only
    while no crate in the build graph enables serde_json's `preserve_order`.
  - The normative page says the reports "are rendered with sorted keys" (`normal-form.md:284-286`).
  - The sorted-keys test guards only RAF's own feature set.
- **Required fix:**
  - Call `Value::sort_all_objects()` before rendering. It is present in the locked 1.0.154.
  - Raise the manifest floor to the first release that has it.
- **Acceptance check:**
  - `output_has_sorted_keys_and_one_trailing_newline` and the Examples golden pass.
  - `render` no longer depends on the map type.

### GAP-14 [LOW] `units::collect` silently skips a source item whose ids do not resolve
*Fail-hard doctrine. Same class as T3 review finding 3.*

- **Evidence:**
  - `crates/raf-census/src/units/mod.rs:129`: `filter_map(|id| workspace.item(*id))` drops an
    unresolvable id.
  - `:134-136`: `let Some(&chosen) = reaches.first() else { continue; }` drops the item.
  - Either case can only arise from a raf-source inconsistency. It would then remove units
    silently.
- **Required fix:** an id that does not resolve is an `Error::Source` naming the file from the
  `items_by_source` tuple.
- **Acceptance check:**
  - Code reading shows no silent skip remains. No consistent workspace can produce the case.
  - The suites pass.

### GAP-15 [LOW] "The census gate joins them when `raf-census` is built" is now stale
- **Evidence:**
  - `AGENTS.md:47-48` and `CONTRIBUTING.md:36-37` still say the census gate joins the root gates
    "when `raf-census` is built".
  - `raf-census` is now a built workspace member (`Cargo.toml` members). No census gate exists:
    the `Gate` is `cap:census-rule-api`, which is designed.
- **Required fix:** reword both to "when the census `Gate` is built" (`cap:census-rule-api`).
- **Acceptance check:** the `repo_gates` (process-wording scan) and `docs_contract` tests pass.

### GAP-16 [LOW] The migration's 1 MiB input bound refuses the index report this change writes for a mid-size workspace
*Cross-capability conflict found while tracing consumers. Disposes of historical-fit F4.9 (bounds).*

- **Evidence:**
  - `census-migration.md:150-153` bounds `--old-baseline` reads to "at most 1 MiB". For a
    current-format ledger, that file is the index report (`census-migration.md` Contract).
  - The index of this repository's own `crates` at `be93118` is 1,966,837 bytes. This was
    measured from the `baseline` example; the audit counts 6,423 units.
  - So migrating RAF's own ledger would be refused.
- **Required fix:** add an item to `work/changes/census-migration.md` that bounds the index read
  so that it admits the index the census writes. One way is to bound it by the census's own load
  budget rather than the ledger's 1 MiB. Tracking it now satisfies the no-deferrals rule.
- **Acceptance check:**
  - The record holds the item.
  - `cargo workplan --root . check` exits 0.

### GAP-17 [LOW] `FileCtx` is delivered ahead of its consumer, in a shape that does not meet that consumer's contract
*Re-judged under the user decision. Disposes of historical-fit F4.6 and F4.7.*

- **Evidence:**
  - `census.rs:12-23`: `FileCtx { package, path, shipping: bool }`.
  - Its only consumer in this change is a test (`tests/units.rs:235-248`).
  - Its designed consumer requires "whether it is shipping code or a test, bench or example
    target" (`census-rule-api.md:44-45`). A boolean cannot say which.
  - raf-source already exposes `TargetKind` (`crates/raf-source/src/target.rs:16-28`).
- **Required fix:** do one of the following.
  - Give `FileCtx` the target kinds that reach the file, beside `shipping`, and state it in the
    Contract.
  - Or remove `FileCtx`, `Census::files` and `Census::file` from this change and its bindings,
    and leave the type to `change:census-rule-api`.
- **Acceptance check:**
  - Either a test asserts lib, test and example files' kinds,
  - or the bindings are gone and `cargo workplan --root . check` exits 0.

## Requirements coverage

| # | Requirement | Status | Location | Gap |
|---|---|---|---|---|
| R1 | `proc_macro2`, bodies re-lexed, macro arguments normalised | COMPLETE | `normalize/form.rs:132-142`; `tests/groups.rs` `renamed_locals_inside_macro_arguments_land_in_one_body_group` | - |
| R2 | structural form: item names kept; locals, params, lifetimes `$0..` by first appearance | PARTIAL | `normalize/passes.rs:73-107, 223-302` | GAP-01, GAP-02, GAP-03 |
| R3 | value form erasures | PARTIAL | `passes.rs` `erase_held`; `value_*` fixtures | each erasure works, but erased noise still moves thresholds (GAP-04) |
| R4 | literal tier < 30 | PARTIAL | `normalize/mod.rs:161-162` | tier decided on the pre-erasure count (GAP-04) |
| R5 | shim form 6-29, own params renamed, `shim_key` | COMPLETE | `normalize/shim.rs`; `units/mod.rs:254-258`; `group.rs:183-194`; `job.rs:88` | - |
| R6 | `<kind>:` + hex of BLAKE3[..8], four kinds | COMPLETE | `normalize/fingerprint.rs` | - |
| R7 | units: fn, method, const, static and their facts | PARTIAL | `units/mod.rs`, `units/scan.rs` | load aborts on admitted signatures (GAP-05); silent skip (GAP-14) |
| R8 | BTreeMap grouping, ≥ 2, count/tokens/fingerprint order | COMPLETE | `group.rs:121-164` | - |
| R9 | hex tables (str, byte str, arrays, both cases); shared constants > 0xffff in ≥ 2 packages | COMPLETE | `units/scan.rs:96-129`; `group.rs:207-239` | - |
| R10 | baseline and index JSON, byte-deterministic | COMPLETE | `report.rs` | "symbols" is met by `units[].symbol`, the shape the designed capability on `origin/main` specified. Caveats: GAP-12, GAP-13 |
| R11 | `normal-form.md` normative v1 | PARTIAL | `docs/architecture/normal-form.md` | false claims at `:91-92`, `:102`, `:110-111`; Decision rewritten (GAP-01, GAP-02, GAP-03); Constraint on bounds (GAP-05) |
| R12 | goldens pin every fixture; bump comment | COMPLETE | `tests/golden.rs:1-14, 249-305` | - |
| R13 | collapse / non-collapse fixtures | PARTIAL | `tests/support/fixtures.rs`; `forms.rs` | the listed fixtures hold, but negatives are missing for items as values (GAP-01), blocks after a constant (GAP-02) and threshold straddles (GAP-04) |
| R14 | twice gives identical bytes | COMPLETE | `tests/reports.rs` two-load tests; example run twice | - |
| R15 | capability reconciled | PARTIAL | `docs/capabilities/census-normal-form.md` | `depends: []` (GAP-08); must re-reconcile after GAP-01 to GAP-05 |
| R16 | fresh versioned normal form v1 | COMPLETE | `version.rs` | version handling of the pin regeneration: GAP-07 |
| R17 | capability delivered | COMPLETE | as R15 | - |
| R18 | depends on #2 (raf-source) | COMPLETE | `crates/raf-census/Cargo.toml:22` | edge missing in capability frontmatter (GAP-08) |

## Acceptance criteria status

| Criterion | Status | Evidence |
|---|---|---|
| Golden tests pin every fixture fingerprint; a comment states the bump rule | MET | `tests/golden.rs:6-14`; `every_fixture_fingerprint_is_pinned`; `the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry` |
| Renamed locals, ref/adapter noise and literal spellings collapse; different called items do not | NOT MET | Different items used as values collapse (GAP-01). Renamed locals after a constant do not collapse (GAP-02). Noise across a threshold does not collapse (GAP-04) |
| Running twice gives identical bytes | MET | `two_fresh_loads_of_the_fixtures_give_identical_bytes`, `…_of_this_repository_…`; example ×2 + `cmp` |
| `cap:census-normal-form` reconciled | PARTIAL | state delivered and `workplan check` clean at `be93118`; GAP-08; re-reconciliation needed after GAP-01 to GAP-05 |

## NEEDS-JUDGMENT resolutions (issue ledger)

| # | Item | Resolution | Evidence |
|---|---|---|---|
| 1 | R18 "Depends on #2" vs brief relation "mentions" | The issue body governs: it is a dependency. #2 is CLOSED and `raf-source` is delivered. Satisfied in code. The capability frontmatter omits the edge (GAP-08) | `crates/raf-census/Cargo.toml:22`; `census.rs:46-49`; `census-normal-form.md:217` |
| 2 | R16 "Per the decision" with no ADR | The binding record is `arch:normal-form` `## Decision`; the repository has no ADRs. The branch rewrote that Decision's renaming clause without a user decision (GAP-03) | `git show origin/main:docs/architecture/normal-form.md:25-27` vs branch `:25-34` |
| 3 | R4 vs R5: literal tier vs shim form for small bodies | Resolved, no gap. They are independent forms. A 6-29 token forwarder has a value form (literals kept below 30; body print at `min_tokens`, never body-grouped) and a shim form (shim level, literals verbatim) | `normalize/mod.rs:201-203`; `group.rs:183-203`; `normal-form.md:144-160` |
| 4 | R2 vs R3: which form each kind hashes | `body` hashes the value level, `shim` the shim level; `table`/`rule` are names. The threshold half, "structural (pre-erasure) tokens count", is a gap (GAP-04) | `normal-form.md:203-210`; `normalize/mod.rs:192-205` |
| 5 | R13 fixture set unnamed | Named by the registry `tests/support/fixtures.rs` and checked by `forms.rs`. Missing negatives are listed in GAP-01, GAP-02 and GAP-04 | `tests/forms.rs` `every_registered_pair_compares_as_registered` |

## Historical-fit findings: dispositions

| Finding | Hint | Disposition | Evidence or gap |
|---|---|---|---|
| F1.1 | M | Not a gap. The 104 inputs are the workplan tool's rule: a section that names no binding takes every binding as input | `crates/raf-workplan/src/assessment.rs:658-659` |
| F1.2 | M | Gap | GAP-08 |
| F1.3 | L | Not a gap. The deliberate single source lets the reports test compare the example's exact output | `examples/baseline.rs:12-13`; `tests/reports.rs:17-18` |
| F1.4 | L | Gap, raised to MEDIUM by the GAP-05 crash | GAP-10 |
| F2.1 | M | Needs an owner decision | GAP-07 |
| F2.2 | L | Gap, raised to HIGH: the remainder is unowned | GAP-06 |
| F2.3, F2.4, F2.5 | I | Not gaps: recorded judgements and process facts | - |
| F3.1 | M | Gap | GAP-11 |
| F3.2 | L | Not a gap. Correction commits were each reviewed and rechecked; history is not a committed-file defect | `validation.md` table |
| F4.1 | H | Not a gap under the user decision. `Job`/`JobMatch` serve their consumers now (the index `jobs` member, the example, the tests); the ledger change may reshape them | `job.rs`; `report.rs:322-337` |
| F4.2 | M | Not a gap under the user decision. Match reasons belong with the ledger/CLI change, which may extend `JobMatch` | `census-cli.md:98` |
| F4.3 | M | Ownership part is a gap (GAP-06); the API part is not a gap (user decision) | GAP-06 |
| F4.4 | L | Not a gap. `Unit` exposes `file` and `line`, so the ledger can map a resolved item to its unit symbol; the ownership of resolution is added in GAP-06(b) | `units/mod.rs:73-76` |
| F4.5 | H | Not a gap. The `Gate`, ledger, detectors, patterns and reference scan are all `raf_census` modules, and the user decision allows reshaping `Census::load` (for example a constructor from a `Workspace`) when they are built | `census-rule-api.md` Contract (`raf_census::Gate`) |
| F4.6 | H | Re-judged: right-for-consumer-now | GAP-17 (LOW) |
| F4.7 | M | Folded into GAP-17 | GAP-17 |
| F4.8 | M | Not a gap. The reference scan is in-crate and can index `units()` by fingerprint, and the API may be reshaped when it is built | - |
| F4.9 | M | Gap, split in two | GAP-12, GAP-16 |
| F4.10 | L | Renderer privacy is not a gap (in-crate consumers). The `preserve_order` dependence is a gap | GAP-13 |
| F4.11 | L | Not a gap. The capability states that the index `shim` is the `shim_key`; the CLI owns its own output | `census-normal-form.md:318-319` |
| F5.1 | H | Gap, on behaviour grounds independent of port parity | GAP-04 |
| F5.2 | M | Not a gap. Fresh v1 by the issue's decision; the encoding is pinned in the manifest | `tests/golden.rs:22-31` |
| F5.3 | H | Not a gap. Fail hard is doctrine, and raf-source lexes every admitted file first, so a body slice cannot fail to lex in practice. The opposite failure (abort) is GAP-05 | `error.rs:11-25`; T3 review |
| F5.4 | H | Not a gap. v1 is fresh (issue). The shapes follow the designed capability on `origin/main`, whose index already listed `units`. Legacy readers are owned by `change:census-migration` | `git show origin/main:docs/capabilities/census-normal-form.md` Examples; `work/changes/census-migration.md:18-24` |
| F5.5, F5.6 | M | Not gaps. Intended and normative | `normal-form.md:162-174, 224-247` |
| F5.7 | L | Not a gap. Owned by other census changes or specific to purrdf | `work/changes/census-*.md` |

## Reviewer feedback

| Source | Feedback | Status |
|---|---|---|
| PR #39 reviews and inline comments | none (`raw/pr39-reviews.json`, `raw/pr39-inline.json` empty; the two comments are the author's plan and least-confident note) | n/a |
| Least-confident note | thresholds count before erasures; locals after the highest parameter | open as GAP-04 and GAP-03 |
| T1-T6 reviews | every blocking finding closed by a recheck | ADDRESSED |
| T4 recheck note | a `match_jobs` doc line over the wrap width (`job.rs:156`) | cosmetic, not counted |
| Completion audit F1, F2, F3 | open | GAP-01, GAP-03, GAP-02 |

## Integration

- **Shipped consumers in this change:**
  - the bound `baseline` example (`examples/baseline.rs` through `tests/support/builtin.rs`);
  - the `goldens` example;
  - the crate's own suites.
- **Designed consumers:**
  - `cap:census-ledger`, `census-rule-api` (`Gate`), `census-detectors`, `census-patterns`,
    `census-reference-scan` and `census-debt-ratchet` are in-crate;
  - `census-cli` is a separate crate;
  - `census-migration` reads the index.
- No wiring gap: every public item is bound and exercised. No `TODO`, `todo!`, `unimplemented!`,
  placeholder or stub exists in the added code; a scan of the added lines found only a comment
  about format-string placeholders.
- Cross-capability issues found by tracing: GAP-06 (classification ownership), GAP-16 (migration
  bound vs index size) and GAP-17 (`FileCtx` vs the rule API).

## ETHOS/SOLID assessment

| Principle | Status | Notes |
|---|---|---|
| SRP | PASS | One concern per module: `normalize/*`, `units/*`, `group`, `job`, `report`, `version`, `options` |
| OCP | PASS | Passes are a fixed, versioned sequence by design; extension is a version bump |
| LSP / ISP | PASS | No trait hierarchy is introduced |
| DIP | PASS | `Census::load` takes `impl raf_source::Tree` |
| Fail fast | VIOLATION | Abort instead of a typed error (GAP-05); silent skip (GAP-14) |
| One path | CONCERN | Signatures and initialisers are parsed twice: by raf-source on its stack and by raf-census on the caller's (GAP-05) |
| Maximum utility | PASS | `CensusOptions` defaults are the ledger defaults, and `min_tokens` refuses 0 |

## Recommended actions, in order

1. Obtain the owner decision for GAP-07.
2. In one change, fix GAP-01, GAP-02, GAP-03 and GAP-04 in `normalize/passes.rs` and
   `normalize/mod.rs`. Add every new registry pair, regenerate `tests/golden.rs` from
   `examples/goldens.rs`, then the `BUILTIN` golden and the capability Examples from
   `examples/baseline.rs`. Correct the page (`:25-34`, `:87-111`, `:176-189`) and the capability
   Contract and Constraints. Re-accept the assessments whose digests move.
3. Fix GAP-05: run `units::collect` on the raf-source parse stack, or reuse raf-source's parse.
   Add the 2 MiB-thread test. Then fix GAP-10: update SECURITY.md and add the fuzz target with the
   reproducer.
4. Fix GAP-06 and GAP-16 (change-record items), GAP-08 (`depends`) and GAP-15 (wording). Run
   `make docs-sync`, `cargo workplan --root . check` and the root test binaries.
5. Fix GAP-09, GAP-12, GAP-13, GAP-14 and GAP-17.
6. GAP-11: run `make check` once at the final head and wait for green hosted CI.
