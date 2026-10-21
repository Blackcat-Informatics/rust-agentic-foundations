# Plan completeness review: raf-census core, normal form v1

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/3-raf-census-core-normal-form-v1/.stage/raf-census-core-normal-form-v1/plan.md`

Inputs read in full: `issue.md` (0 comments), `issue-analysis.md` (R1-R18, NEEDS-JUDGMENT 1-5),
`prior-art-assessment.md` (NEEDS-JUDGMENT 1-7), `AGENTS.md`, `CLAUDE.md`,
`docs/architecture/normal-form.md`, `docs/capabilities/census-normal-form.md`,
`work/changes/census-normal-form.md`, `work/changes/census-ledger.md`, and the relevant parts of
`docs/capabilities/census-ledger.md` and `census-cli.md`. Spot-checked sources:
`crates/raf-source/src/{workspace.rs,item.rs,target.rs}` (`Workspace::load`, `items`, `source`,
`is_shipping`, `Item::body`, `Item::trait_path`, `Target::shipping` exist), and the port source
`purrdf/crates/helper-census/src/{normalize.rs,census.rs}`.

Ledger coverage: every R-id (R1-R18) has a contract row. Every NEEDS-JUDGMENT item of both ledgers
has a disposition. No requirement was found in the issue that the ledger missed (the issue has no
comments). Structure: Tasks 1-7, no Task 0. The header gives every task an independent review,
commit, push and issue update. Task 7 creates the PR with `Closes #3`, publishes the plan and hands
off.

## Requirement -> task -> check

| R | Requirement (short) | Task | Check as planned | Status |
|---|---|---|---|---|
| R1 | proc_macro2, bodies re-lexed, macro args normalised | T2 | golden `macro_arguments` on the `format!` pair, said to run "from `units::collect` on a MemoryTree workspace" | WEAK: `units::collect` does not exist until T3, and no task creates the `Census::load` entry point (F1) |
| R2 | structural form | T2 | goldens `structural_*`; locals, params and lifetimes collapse; different fn, method, field and type differ | OK |
| R3 | value form | T2 | one fixture per erasure, 11 adapters, binary `&`/`*`/`&&` kept | OK (ADVISORY A3: no const-turbofish or argument-taking adapter neighbour) |
| R4 | literal tier < 30 | T2 | 25-token pair differs, 35-token pair equal, 29/30 boundary | OK |
| R5 | shim form: single-expression, 6-30, own params only, shim_key | T2, T4 | 5/6/29/30 boundaries; trait-impl and `Self::` give no group | SILENT CUT: "single-expression" dropped; "only own parameters renamed" unchecked (F2) |
| R6 | `<kind>:` + 16 hex of BLAKE3[..8], four kinds | T2 | compare to `blake3::hash`; parse refuses other kinds and widths | OK (ADVISORY A4: valid neighbour per kind) |
| R7 | units, every field, int constants by value, tables, strings | T3 | MemoryTree lib + test target, every field asserted, `0xff`/`255` one value, test unit `shipping: false` | OK, provided F1 routes it through the entry point |
| R8 | BTreeMap grouping, >= 2, ordering | T4 | groups of one dropped; "three groups tying on count" | UNFALSIFIABLE in part: token direction and the fingerprint tiebreak are not pinned (A1) |
| R9 | hex tables (3 forms x 2 cases); shared constants > 0xffff in >= 2 packages | T4 | one fixture per form and case; 0x10000 vs 0xffff vs one package | OK (ADVISORY A2: no negative near-hex-table fixture) |
| R10 | baseline JSON (`min_tokens`, units, body/shim groups, hex tables, shared constants); index JSON (symbols, groups, job matches); byte-deterministic | T5 | exact JSON goldens; "keys per the capability shape"; jobs rendered from caller-supplied `JobMatch` | STUB/TEST-ONLY for job matches (F3); `min_tokens` has no behaviour, setter or check (F4); "keys per the capability shape" points at a shape this change replaces (F5) |
| R11 | normal-form.md normative v1 | T6 | "page states every form, threshold, unit kind, ordering, report member" | UNFALSIFIABLE (A5) |
| R12 | goldens pin every fixture; comment on bump | T2 | `tests/golden.rs` header; "every fixture's fingerprint pinned" | WEAK: the `forms` fixtures are not tied to the pins (A6) |
| R13 | collapse fixtures (locals, ref/adapter noise, literal spellings); called items do not collapse | T2 | "named fixtures listed in R2-R4" | UNMAPPED in part: no fixture shows that different literal spellings collapse a body fingerprint (F6) |
| R14 | twice gives identical bytes | T5 | two fresh loads compared; example run twice and `cmp` | OK (A7: also pin the example's fixture output) |
| R15 | `cap:census-normal-form` reconciled with the result | T6 | `change complete` exits 0; state `delivered`; INVENTORY | INSUFFICIENT: the completion gate passes with a stale report example (F5) |
| R16 | fresh versioned normal form v1 | T2, T5 | `NORMAL_FORM == 1`; `check_normal_form(2)` names `cargo census migrate` | OK |
| R17 | capability delivered | T6 | as R15, bindings name code, tests and examples | see F5 |
| R18 | depends on raf-source | T1 | path dependency | OK |

## Findings

### F1 BLOCKING: STRUCTURE/UNMAPPED. The named shipped entry point is created by no task

The plan states: "Each check runs through the public API `raf_census::Census::load(&tree, roots)`
over `raf_source::MemoryTree` fixtures and over `DiskTree` of this repository". No task creates
`Census` or `Census::load`. T3 creates `units::collect`, T4 a `group` module and T5 a `report`
module, and T5's check then calls `Census::baseline(..)`. The R1 row also says the normaliser is
"called from `units::collect` on a MemoryTree workspace", but R1 is assigned to T2, whose check
(`--test golden --test forms`) runs before `units::collect` exists.

REWRITE TO (Task 3, first sentence): "`Census::load(tree: &dyn raf_source::Tree, roots: &[&str],
options: &CensusOptions) -> Result<Census, CensusError>` (the public entry point: it loads the
`raf_source::Workspace` and collects the units) and the `units` module over it: fn, method ..."
Then append to the Task 3 check: "every `units` test builds its input with `Census::load` over a
`MemoryTree`, including the macro-argument pair (two functions whose bodies are
`format!("{x:02x}", x = b)` with the local `b` renamed `c`), which share one `body` fingerprint and
whose fingerprint equals the `macro_arguments` golden."

REWRITE TO (R1 row, Task column): "T2, T3".

### F2 BLOCKING: SILENT CUT. The issue's "single-expression forwarders" condition and the "only the function's own parameters renamed" rule are dropped from the shim form

The issue (R5) says: "**shim form**: single-expression forwarders (6–30 tokens), with only the
function's own parameters renamed". The plan's disposition 3 reads: "A body of 6 to 29 tokens
additionally has a shim form". That has no single-expression condition, and the R5 check covers only
the token boundaries and the two `shim_key` exclusions. The port source requires a single expression
(`shim_candidate` in purrdf `normalize.rs:578-594`: no `let`, no statement before the tail, at least
one call). No check shows that renaming locals other than parameters is kept out of the shim form,
or that different callees or fixed arguments give different shims.

REWRITE TO (disposition 3, second and third sentences): "A body of 6 to 29 structural tokens that
is a single expression (no `let`, no `;` before the tail, at least one call) additionally has a
shim form; a multi-statement body of the same size has none. In the shim form only the function's
own parameters are renamed; every other identifier and every literal is kept verbatim."

REWRITE TO (R5 row, check column): "boundaries 5, 6, 29, 30; `fn a(x: u32) -> u32 { helper(x, 7) }`
and `fn b(y: u32) -> u32 { helper(y, 7) }` share one `shim` group; `helper(y, 8)` and
`other(y, 7)` each have a different shim fingerprint; a 6-29-token body with a `let` or two
statements has no shim fingerprint; a trait-impl method and a `Self::` forwarder produce no `shim`
group (T4, grouped over a workspace from `Census::load` on a MemoryTree)".

### F3 BLOCKING: STUB / TEST-ONLY. Index "job matches" exist only as caller-supplied data, without a quoted user decision

The issue (R10) asks for "index JSON (symbols, groups, job matches)". The plan delivers "a `jobs`
member whose rows are produced from a `JobMatch` list the caller supplies", tested "on supplied
matches". It also states that "The census-cli `index` command shows no jobs until the ledger exists". No
census code computes a match. The only demonstration uses fabricated matches. The plan also adds a
new item to `change:census-ledger` ("The index report lists the classified matches of every job").
That moves part of this issue's deliverable to another change with no quoted decision from the user.
HOME/VARIANT/COPY classification is already tracked in `change:census-ledger` and can stay there.
Finding which units match a job's fingerprints and constants is core census work over the units
this change builds.

REWRITE TO (scope-boundary bullet 1, from "This change delivers"): "This change delivers job
matching over the census's own units: `Census::index(&[JobQuery])`, where a `JobQuery` is a job id
with its `body`/`shim`/`table` fingerprints and integer constants (by value). The census computes,
for each job, every unit whose body or shim fingerprint, hex table or integer constant matches,
sorted by symbol, and the index report's `jobs` member lists them. Classifying a match as HOME,
VARIANT or COPY, matching `names`, `patterns` and `rule:` ids, and reading jobs from a ledger file
stay with `change:census-ledger` (its existing item "Matches are classified as HOME, VARIANT or COPY
and counted per job")."

REWRITE TO (R10 row, check column, appended): "`index` over a MemoryTree workspace with two copies of
one body and a `JobQuery` naming that body fingerprint and the constant `0x10000` lists both body
copies and the unit holding `65536` under the job; a `JobQuery` whose fingerprint matches no unit
gives that job an empty match list."

The alternative is to keep the caller-supplied design and quote the user's decision to move job-match
production into `change:census-ledger` in the plan.

### F4 BLOCKING: UNMAPPED. `min_tokens` has no delivering task and no check

The issue lists `min_tokens` as a baseline member (R10). The capability Contract, which this change
must deliver or reconcile (R15), says: "A body with fewer than `min_tokens` structural tokens has no
body fingerprint. `min_tokens` defaults to 20". No task names where `min_tokens` is set: the
ledger's `[census] min_tokens` belongs to `change:census-ledger`. No fixture pins the 19/20 boundary
or the default. The plan mentions `min_tokens` only inside disposition 3 ("fingerprinted as `body`
only if it reaches `min_tokens`").

REWRITE TO (Task 3, appended): "`CensusOptions { min_tokens }`, default 20 (`CensusOptions::default`),
passed to `Census::load`; a unit with fewer structural tokens than `min_tokens` has no `body`
fingerprint."

REWRITE TO (contract, new row after R10): "| R10b | `min_tokens` threshold and report member | T3,
T5 | a 19-token body has no `body` fingerprint and a 20-token body has one under the default; with
`min_tokens: 25` the 20-token body has none; the baseline JSON's `min_tokens` is `20` by default and
`25` when set |"

### F5 BLOCKING: INSUFFICIENT check for R15/R17. Capability reconciliation is checked only by the completion gate

R15 asks that the capability be "reconciled with the result". The planned check is "`cargo workplan
change complete census-normal-form` exits 0; state `delivered`". That gate checks obligations,
outcomes, PROPOSED blocks, generated blocks and bound tests. It does not check that the text matches
what was built. The capability's Examples section shows a baseline of `normal_form`, `min_tokens`
and `groups` only, and an index without `jobs`. T5 builds a different shape (units, body and shim
groups, hex tables, shared constants, jobs). T1's PROPOSED blocks cover only "the scope boundaries
above; Verification reworded". The R10 check ("keys per the capability shape") also points at this
stale shape, so it would assert the wrong key set.

REWRITE TO (Task 1, the PROPOSED-block clause): "PROPOSED blocks in `cap:census-normal-form` for
Contract (units, single-expression shims, shared constants, `CensusOptions::min_tokens`, report
members, job matching as delivered), Usage (`Census::load`, `Census::baseline`, `Census::index`; the
`raf_census::Gate` and `cargo census` sentences moved to the changes that build them), Examples (the
real baseline and index JSON of the README fixture), Constraints and Verification (reworded to the
version gate)".

REWRITE TO (R15 row, check column): "`cargo workplan change complete census-normal-form` exits 0;
state `delivered`; INVENTORY updated; a `reports` test parses the baseline and index JSON blocks of
the capability's Examples section and asserts their top-level key sets equal those of
`Census::baseline(..).to_json()` and `Census::index(..).to_json()` over the same fixture".

REWRITE TO (R10 row, check column, replacing "keys per the capability shape"): "top-level keys
exactly `normal_form`, `min_tokens`, `units`, `body_groups`, `shim_groups`, `hex_tables`,
`shared_constants` (baseline) and `normal_form`, `units`, `groups`, `jobs` (index)".

### F6 BLOCKING: UNMAPPED in part. R13's "different literal spellings collapse" has no fixture

R13 says: "Fixtures prove that renamed locals, reference/adapter noise and different literal
spellings collapse". The R13 row reads "named fixtures listed in R2-R4", and none of R2-R4 is a
literal-spelling pair. R4 covers different string values and R7 covers unit constant values, not
body fingerprints. T2 lists "integer value spelling-independence" with no check. Inside the literal
tier (< 30 tokens) literals are kept verbatim, so this is where spelling must collapse.

REWRITE TO (R13 row, check column): "fixtures in `tests/forms.rs`, each pinned in `golden.rs`:
`collapse_renamed_locals` (R2), `collapse_borrow_adapter_noise` (R3, borrows plus `.clone()` plus a
`let` annotation in one pair), `collapse_int_spelling` (two < 30-token bodies differing only in
`0xff` vs `255` and `1_000` vs `1000` share one `body` fingerprint), `collapse_int_spelling_long`
(the same at 35 tokens), and `distinct_called_fn`, `distinct_method`, `distinct_field` (R2), each
pair asserting unequal fingerprints".

### A1 ADVISORY: UNFALSIFIABLE grouping order (R8)

"ordering fixture with three groups tying on count" fixes neither the direction of the token key nor
whether the fingerprint tiebreak runs. The port source sorts tokens descending (`census.rs:68-75`).
REWRITE TO: "ordering fixture: group A with 3 members; groups B (2 members, 28 tokens) and C
(2 members, 22 tokens); groups D and E (2 members, 22 tokens each). The expected order is A, B, then
C, D and E by ascending fingerprint. Members within a group are sorted by symbol, then file and
line."

### A2 ADVISORY: no negative hex-table fixture (R9)

AGENTS.md requires a negative fixture for every rule. Add: "`\"0123456789abcdeg\"`, a 15-digit
table and a mixed-case table are not reported".

### A3 ADVISORY: missing valid/invalid neighbours for value-form erasures (R3)

The architecture page keeps a turbofish with a const argument and drops only adapters "called with
no arguments". Add: "`f::<3>()` vs `f::<4>()` differ; `.as_ref()` dropped but `.borrow_mut(x)`-style
calls with arguments kept".

### A4 ADVISORY: fingerprint parse needs a valid neighbour per kind (R6)

Add: "`body:` and `shim:` with 16 lowercase hex accepted; `table:hex-lower` and `table:hex-upper`
accepted; 15 or 17 hex digits, uppercase hex and `blob:` refused".

### A5 ADVISORY: UNFALSIFIABLE R11 check

REWRITE TO: "`arch:normal-form` has sections for units, grouping and ordering, tables and shared
constants, and reports. A `raf-census` test reads the page and asserts that its `VALUE_ADAPTERS`
list, `MIN_SHIM_TOKENS`, `SHIM_MAX_TOKENS`, literal tier, default `min_tokens` and
report top-level keys equal the crate's constants and the golden report keys".

### A6 ADVISORY: "every fixture" pinned (R12)

REWRITE TO (Task 2): "one fixture table (`tests/fixtures/mod.rs`) is shared by `forms.rs` and
`golden.rs`. `golden.rs` asserts a pinned fingerprint for every entry, and fails when an entry has
no pin or a pin has no entry".

### A7 ADVISORY: the example's output is checked only for self-equality (R14)

A `cmp` of two runs over `crates` passes even if the output is empty. Name the fixture directory
(for example `crates/raf-census/tests/fixtures/workspace`) and add: "the example's output on it
equals the committed `baseline.json` golden and contains one `body` group with both fixture
symbols".

VERDICT: NEEDS-REVISION
