# Plan completion adversary: Stage 1

* Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/3-raf-census-core-normal-form-v1/.stage/raf-census-core-normal-form-v1/plan.md`
* Pass: 1 (single pass; no recheck)
* Inputs read: `issue.md` (0 comments), `issue-analysis.md` (R1-R18), `prior-art-assessment.md`, `AGENTS.md`, `CLAUDE.md`,
  `docs/architecture/normal-form.md`, `docs/capabilities/census-normal-form.md`, `census-ledger.md`, `census-cli.md`,
  `census-migration.md`, `work/changes/census-normal-form.md`, `census-ledger.md`, `census-cli.md`, `census-migration.md`,
  `census-detectors.md`, the port source (`purrdf/crates/helper-census/src/normalize.rs`, `census.rs`), the
  `raf-source` `Item` model and the `raf-workplan` bound-example runner.
* No `.baseline`, `.goals`, `CONSTITUTION.md` or ADR files exist, so there are no user scope decisions to quote. The
  issue has no comments.

## Escalation (user decision required)

### E1. R10 "job matches": the producer is moved out and the index `jobs` member is shown only on test-supplied input (PRE-EMPTIVE DESCOPE / PLANNED-BY-REFUSAL)

Plan, Scope boundaries:
> "This change delivers the index report with a `jobs` member whose rows are produced from a `JobMatch` list the caller
> supplies, with its shape, sorting and determinism tested on supplied matches; T1 adds to `change:census-ledger` the
> item "The index report lists the classified matches of every job" so the producer is tracked. The census-cli `index`
> command shows no jobs until the ledger exists; the report states the jobs it was given."

Plan, T5:
> "`Index` (normal_form, units with symbols and fingerprints, groups, jobs from supplied `JobMatch`es)"

The issue asks for "index JSON (symbols, groups, job matches)" (R10). In this change no shipped code produces a
`JobMatch`. Every real index report will have an empty `jobs` member, and the only non-empty `jobs` come from tests,
so the requested capability is shown only by a test. Moving work to another tracked change is allowed by
AGENTS.md section 7, and `change:census-ledger` already tracks "Matches are classified as HOME, VARIANT or COPY and
counted per job". Even so, moving part of an issue checkbox out of the issue is a scope decision. The plan made it on
the implementer's judgment, and no user decision is quoted.

Context the user needs for the choice:
* The port's index (`purrdf/.../census.rs:613-750`) builds both `symbols` (the ledger-referenced paths it resolved)
  and `jobs` from the ledger.
* The plan reads the issue's "symbols" as unit symbols. That matches the designed index of `cap:census-normal-form`
  and what `cap:census-migration` needs, but the plan does not say it made that reading.

Choices:
* **(A) Accept the split as planned.** The `jobs` member and the `JobMatch` type ship here, and the producer is
  `moved:change:census-ledger` through the new item. R10's row and the PR state that the job-matches part is the
  member's shape only and that its producer moved. Nothing ticks "job matches" as delivered.
* **(B) Deliver job matching here.**
  * Move the `change:census-ledger` item "Matches are classified as HOME, VARIANT or COPY and counted per job" into
    `change:census-normal-form`.
  * Define an in-memory `Job` (id, home, entry_points, forbidden names, constants and fingerprints, variants) that the
    ledger reader will later build. `Census::index(&jobs)` computes the matches over the loaded units.
  * Acceptance: a MemoryTree workspace plus one constructed `Job` gives an index JSON with one `HOME`, one `VARIANT`
    and one `COPY` match and `copies: 1`.
  * TOML reading stays in `change:census-ledger`.
* **(C) Leave job matches out of the v1 index report.** The capability's designed index has no `jobs` member, and
  `cargo census job <ID>` prints roles according to `cap:census-cli`. Edit the issue body to remove "job matches".

Whatever the choice, the R10 acceptance criterion must be rewritten (finding F1).

## Findings

### F1. R10 criterion points at a shape this change rewrites (UNFALSIFIABLE CRITERION)

> "| R10 | baseline JSON and index JSON, byte-deterministic | T5 | `Census::baseline(..).to_json()` and `index(..)`:
> exact JSON goldens over a fixture workspace; keys per the capability shape |"

The capability's current shapes are baseline = `normal_form, min_tokens, groups` and index = `normal_form, units`.
Neither contains what T5 builds: units, body and shim groups, hex tables, shared constants and jobs. This change also
rewrites the capability, so "keys per the capability shape" moves with whatever gets built.

REWRITE TO:
* Over the fixture workspace `tests/fixtures/report/` (two packages, one body group, one shim group, one hex table, a
  constant 0x10000 shared by both packages), `Census::load(..)?.baseline().to_json()` equals its committed golden
  byte for byte. Its top-level keys are exactly the list the plan fixes before T5 starts (for example
  `normal_form, min_tokens, units, body_groups, shim_groups, hex_tables, shared_constants`).
* `index().to_json()` equals its golden, with top-level keys exactly `normal_form, units, groups, jobs` under E1 (A)
  or (B), or `normal_form, units, groups` under (C).
* The keys of every unit, group and member object are listed in the plan. The same key lists appear verbatim in
  `arch:normal-form` and in the capability Examples.

### F2. R5 "single-expression forwarders" is unmapped, and the shim_key check is negative-only (UNMAPPED REQUIREMENT + UNFALSIFIABLE CRITERION)

NEEDS-JUDGMENT 3:
> "A body of 6 to 29 tokens additionally has a shim form. A small forwarder therefore has both: ..."

Completeness contract:
> "| R5 | shim form 6-29, shim_key | T2, T4 | boundaries 5, 6, 29, 30; trait-impl method and `Self::` forwarder produce
> no `shim` group (T4 grouping over a MemoryTree workspace) |"

The issue says "**shim form**: single-expression forwarders (6–30 tokens), with only the function's own parameters
renamed". The port's `shim_candidate` (`normalize.rs:578-594`) also requires one expression: no `let`, no statement
before the tail, and at least one call. The plan's disposition, which T6 makes normative, gives every 6 to 29 token
body a shim form, and no row checks the single-expression qualifier. Three other gaps:
* No row checks that only parameters are renamed.
* No row checks that the same callee with the same fixed arguments collapses while different fixed arguments do not.
* No row checks that a shim group forms at all. "Produce no `shim` group" also passes on a census that never groups
  shims (AGENTS.md section 5: "A refusal is tested with its valid neighbour").

REWRITE TO (NEEDS-JUDGMENT 3): "A body of 6 to 29 structural tokens that is a single-expression forwarder (no `let`,
no statement before the tail expression, at least one call) also has a shim form; any other body has none." The rest
of the disposition stays.

REWRITE TO (R5 row): all of these run through `Census::load(..)?.baseline()` over one MemoryTree workspace.
* Single-expression forwarders of 5 and 30 tokens have no shim fingerprint, and those of 6 and 29 tokens have one.
* A 12-token `let y = f(x); y` body has no shim fingerprint, and neither does a 12-token expression with no call.
* `fn a(t: &str) -> R { read_with(t, Limits::DEFAULT) }` and `fn b(s: &str) -> R { read_with(s, Limits::DEFAULT) }`,
  in two modules, share one `shim:` fingerprint and form one shim group of 2.
* The same pair with `Limits::STRICT` in one of them has different fingerprints.
* The same pair written as methods of `impl Tr for X` and `impl Tr for Y` forms no shim group.
* Two `Self::new(v)` forwarders form no shim group.
* The negative cases sit in the same workspace as the free-function pair, so they are judged next to a group that does
  form.

### F3. R13 "different literal spellings collapse" has no fixture (UNMAPPED REQUIREMENT)

> "| R13 | collapse/non-collapse fixtures | T2 | named fixtures listed in R2-R4 |"

Rows R2 to R4 name renamed locals, erasures, called items and string-literal pairs at the literal-tier boundary. None
of them names a pair that differs only in how a literal is spelled. R7's "`0xff` and `255` one constant value" checks
the unit's constant list, not a fingerprint. Inside the literal tier, where literals stay verbatim, this is the case
that needs proving.

REWRITE TO: "| R13 | ... | T2 | the R2-R4 fixtures, plus `literal_spelling`: two bodies under 30 tokens that are
identical except `0xcbf2_9ce4_8422_2325` and `14695981039346656037` (and `0xff`, `255` and `0b1111_1111`) have one
`body:` fingerprint, pinned in `tests/golden.rs`; the neighbouring pair `0xff` and `0x100` has different fingerprints |"

### F4. R15/R17 "reconciled" is judged only by the completion gate's exit code (UNFALSIFIABLE CRITERION)

> "| R15 | capability reconciled | T6 | `cargo workplan change complete census-normal-form` exits 0; state
> `delivered`; INVENTORY updated |"

> T1: "PROPOSED blocks in `cap:census-normal-form` for the changed sections (the scope boundaries above; Verification
> reworded)"

> T6: "PROPOSED blocks resolved into the capability's current text, ... assessments re-read and accepted"

The completion gate checks structure: outcomes, PROPOSED blocks, sync, the bound runs and assessment verdicts. It does
not check that the prose is true, and the implementer is the one who accepts the assessments. These sentences in the
current `cap:census-normal-form` cannot be true once the capability is `delivered` by this change, and T1 plans
rewording only for the scope boundaries and Verification:

| Section | Current text | Why it cannot stay |
|---|---|---|
| Usage | "The census runs as part of `raf_census::Gate` and through `cargo census`." | Neither ships in this change. |
| Examples | "The designed baseline report has this shape" and "The designed index report lists every unit" | They use the placeholder `body:3f9a1c0b7d2e4a65` and omit members T5 builds. |
| Contract | "The baseline report lists every group. The index report lists every unit with its fingerprints." | The reports gain further members. |
| Contract | "A ledger whose `normal_form` differs from the implemented version fails hard" | No ledger reader ships in this change. |
| Contract | "`min_tokens` defaults to 20 and `[census] min_tokens` sets it." | No ledger reader ships, and the planned entry point takes no `min_tokens` (see F10). |
| Examples | The `total`/`add_up` pair, which "the census puts ... in one body group". | No row tests it. |

REWRITE TO (R15/R17 and T1):
* T1 writes PROPOSED blocks for Contract, Usage, Examples and Verification. Each sentence in the table above is
  rewritten to state what ships:
  * the library entry points (`Census::load`, `baseline`, `index`) and the bound example;
  * the real report members, as listed in F1;
  * the `check_normal_form` gate, with reading the ledger left to `cap:census-ledger`;
  * how this API sets `min_tokens`.
* The two "designed" JSON blocks are replaced by real output for the `total`/`add_up` fixture, with real fingerprints
  taken from the goldens.
* The test `capability_example_pair_is_one_body_group` loads the capability's two Example functions into a
  MemoryTree workspace and asserts one body group of 2.
* Acceptance: a reviewer checks every row of that table as rewritten or removed, then `cargo workplan change complete
  census-normal-form` exits 0 and the state is `delivered`.

### F5. The bound example's no-argument run, which the completion gate executes, is unplanned (UNMAPPED: task insufficient for R17)

> "plus a bound example `examples/baseline.rs` that prints the baseline of a directory (run in T5 on this repository
> and on a fixture directory)"

> T5 check: "`cargo run -p raf-census --example baseline -- crates` twice and `cmp`"

`raf-workplan` runs a bound example as `cargo run --frozen --package=<pkg> --example=<example>` with no arguments
(`crates/raf-workplan/src/bound/plan.rs:358-383`). AGENTS.md section 6 makes a passing bound example a completion
condition. The plan only says what the example does with a directory argument. R15/R17's check
(`change complete` exits 0) depends on the run with no arguments, and the plan says nothing about that run.

REWRITE TO:
* With no arguments, `examples/baseline.rs` prints the baseline JSON of a stated input and exits 0. The input is
  either a MemoryTree built in the example (the capability's `total`/`add_up` pair) or the current directory; the
  plan picks one. This is the run the completion gate makes.
* With a directory argument, it prints that directory's baseline.
* T5 check: `cargo run --frozen -p raf-census --example baseline` exits 0 and prints the stated baseline.
  `cargo run -p raf-census --example baseline -- crates` run twice gives output that `cmp` reports equal.

### F6. The moved ledger-mismatch fixture loses the `cargo census migrate` assertion (UNMAPPED: the receiving row is insufficient)

> "The change-record item is split: the gate `implemented` here, the ledger-file fixture `moved` to
> `change:census-ledger`. The capability Verification line is reworded to the gate."

The text being moved is "A ledger fixture with `normal_form = 2` fails with a message that names
`cargo census migrate`". The receiving item ("A normal_form mismatch fails before any job is read") and the
`cap:census-ledger` Verification bullet do not mention the message, its two versions or `cargo census migrate`.
After the reword, no tracked record requires a real ledger mismatch to show the migrate instruction.

An item also carries exactly one outcome (`implemented`, `dropped` or `moved:change:<slug>`,
`raf-workplan/src/obligation.rs:448`), so "split" must mean rewriting the item as two claims.

REWRITE TO:
* T1 rewrites item 5 of `change:census-normal-form` as two claims:
  * "Golden tests pin every fixture fingerprint, and `check_normal_form` refuses every version but 1 with a message
    naming both versions and `cargo census migrate`" (`implemented`);
  * "A ledger file with a mismatched normal_form fails hard through `check_normal_form`, naming
    `cargo census migrate`" (`moved:change:census-ledger`).
* T1 rewords the existing `change:census-ledger` item to "A normal_form mismatch fails before any job is read, with
  the `check_normal_form` message naming both versions and `cargo census migrate`."

### F7. R12 "every fixture" cannot be checked (UNFALSIFIABLE CRITERION)

> "| R12 | goldens pin every fixture, comment on bump | T2 | `tests/golden.rs` header comment; every fixture's
> fingerprint pinned |"

Fixtures are spread over four test binaries (`golden`, `forms`, `units`, `groups`). Nothing makes a fixture added to
`forms` or `groups` without a pin fail.

REWRITE TO:
* Every fingerprint fixture is an entry of one shared table (for example `FIXTURES` in `tests/fixtures/mod.rs`) that
  `forms`, `units` and `groups` read.
* `tests/golden.rs` asserts that the set of pinned names equals the set of fixture names, so a fixture without a pin,
  or a pin without a fixture, fails.
* It also asserts that each recomputed fingerprint equals its pin.
* The header comment states that a moved pin needs a `normal_form` bump and regenerated pins.

### F8. R8 ordering: token direction is unstated and the fixture skips the sort keys (UNFALSIFIABLE CRITERION)

> "| R8 | grouping BTreeMap, ≥2, ordering count desc then tokens then fingerprint | T4 | groups of one dropped;
> ordering fixture with three groups tying on count |"

The token key has no direction; the port sorts tokens descending (`census.rs:68-75`). Three groups that tie on count
never test the count key, and they test the fingerprint key only if their token counts also tie.

REWRITE TO: an ordering fixture of four body groups:
* G1 has 3 members.
* G2, G3 and G4 have 2 members each; G2 has more tokens than G3 and G4, which have equal tokens.
* The baseline lists G1, then G2, then G3 and G4 in ascending fingerprint order (count descending, tokens
  descending, fingerprint ascending).
* A unit with no duplicate forms no group.
* The order of members within a group is stated in the plan and asserted.

### F9. R9 hex-table detection has positive fixtures only (UNFALSIFIABLE CRITERION)

> "| R9 | ... | T4 | one fixture per form and case; constant 0x10000 in two packages reported, 0xffff not, one package
> not |"

The constant half has negatives; the hex-table half does not. A detector that flags every 16-element string or array
passes. AGENTS.md section 5 says "Every rule has a negative fixture".

REWRITE TO: six positive fixtures (string, byte-string and array, each in lower and upper case) report
`table:hex-lower` or `table:hex-upper` on their unit. In the same workspace, at least one non-table neighbour of each
form reports nothing: a 16-character string outside the hex alphabet (`"0123456789abcdeg"`), a 15-digit prefix, and
an array of 16 non-digit bytes. The constant cases stay as written.

### F10. `min_tokens` has no boundary check and no planned way to set it (UNMAPPED REQUIREMENT)

> NEEDS-JUDGMENT 3: "(fingerprinted as `body` only if it reaches `min_tokens`)"

> "Each check runs through the public API `raf_census::Census::load(&tree, roots)`"

The issue's baseline reports `min_tokens` (R10). `arch:normal-form` and the capability make it the threshold for
having a body fingerprint, "20 by default, set by `[census] min_tokens`". No row has a 19/20 boundary, so an
implementation that ignores `min_tokens` passes. The named entry point takes no `min_tokens`, so the reported value
cannot be set.

REWRITE TO: add a row.
* Duplicate pairs of 19 and 20 structural tokens: the 19-token pair has no `body` fingerprint and no body group, and
  the 20-token pair has both.
* Through the entry point that sets the threshold (named in the plan, for example
  `Census::load_with(&tree, roots, &Settings { min_tokens: 25 })`), the 20-token pair has neither, and the baseline's
  `min_tokens` reads 25.

### F11. R3's "full-range" and "type-only" qualifiers have no neighbour fixture (UNFALSIFIABLE CRITERION)

> "| R3 | value form | T2 | fixture per erasure (borrow, `&mut`, deref, `ref`, each of 11 adapters, `x[..]`, let
> annotation, type turbofish, `m[&k]`); binary `&`, `*`, `&&` kept (different fingerprint) |"

The issue limits the erasures to a "full-range reindex" and "type-only turbofishes", and `arch:normal-form` says "a
turbofish with a const argument is kept". An implementation that erases every turbofish or every range index passes
the row as written.

REWRITE TO: add the following neighbours to R3:
* `v.f::<u8>()` and `v.f()` are equal, while `v.f::<3>()` and `v.f()` differ (a const turbofish is kept).
* `x[..]` and `x` are equal, while `x[1..]` and `x` differ (a partial range is kept).

### F12. R6 shows refusals but never the kinds the issue names being accepted (UNFALSIFIABLE CRITERION)

> "| R6 | ... | T2 | unit test against `blake3::hash` of a known form; `Fingerprint` parse refuses other kinds/widths
> (negative fixture) |"

The issue says "kinds are `body`, `shim`, `table`, `rule`". Nothing in this change produces a `rule:` id, so a parser
that refuses `rule:` passes every planned check.

REWRITE TO:
* `Fingerprint` parsing accepts `body:` and `shim:` followed by 16 lowercase hex digits, `table:hex-lower` and
  `rule:std-default-hasher`.
* It refuses `foo:0123456789abcdef`, `body:` followed by 15 or 17 hex digits, and `body:` followed by uppercase hex.
* Each refusal sits next to its accepted neighbour.

## Checked, no finding

* R1 (the `macro_arguments` golden with a renamed local), R2, R4 (25/35 pairs and the 29/30 boundary), R7, R14 (two
  fresh loads, the real `crates/` tree and two separate example processes), R16 and R18 have falsifiable checks on
  named inputs.
* Scope boundary 2 (the normal_form gate) moves no issue requirement. `change:census-ledger` already tracks reading the
  ledger, and the only gap is F6.
* The prior-art routes are sound: `blake3`, the MIT/Apache/MulanPSL port from purrdf, and `shim_key` taken from the
  specification text rather than the AGPL code.
* "Enhancements Considered and Declined" is still a placeholder ("Filled from the enhancement audit."). Nothing is
  declined yet. `plan-final-reviewer` must confirm that no entry added there carries R1-R18 work.

## Outside this reviewer's grounds (for the other reviewers)

* T6 and T7 reach the PR without running `make check`, though CLAUDE.md says "Gate before you claim done:
  `make check`". This is for constraint compliance.
* NEEDS-JUDGMENT 3 puts a 20 to 29 token forwarder in both body and shim groups. The port leaves units that have a
  shim fingerprint out of body groups (`census.rs:26-41`). This is a v1 normative choice for the design review.

VERDICT: ESCALATE-TO-HUMAN
